//! Swept profiles: a profile (a closed outline held by the type) carried
//! along a path in plan, for baseboards, crown moldings, fascias, gutters and
//! copings. Creating them, creating their types and redrawing a type's profile.

use super::sketch::CurveArg;
use super::{add_element, created, edit};
use crate::error::{EngineError, Result};
use crate::refs::{self, Point, Ref, finite, point};
use crate::registry::{Command, CommandKind};
use crate::session::Session;
use crate::t;
use cc_core::profile_sweep::{MAX_PATH_POINTS, MAX_STATIONS, Sweep, baseboard, check_profile, path_from_curves};
use cc_core::sketch::Sketch;
use cc_core::{Category, ElementKind, ElementType, ElementTypeId, Project, TypeData};
use glam::DVec2;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// The sweep type a project gets with its first sweep: a baseboard (10 cm
/// tall, 4" in imperial projects; see [`cc_core::Defaults::baseboard`]).
pub fn new_sweep_type(p: &Project, id: ElementTypeId) -> ElementType {
    let (key, thickness, height) = p.defaults().baseboard;
    let profile = if p.measurement_system() == cc_core::MeasurementSystem::Metric {
        baseboard()
    } else {
        vec![DVec2::ZERO, DVec2::new(thickness, 0.0), DVec2::new(thickness, height), DVec2::new(0.0, height)]
    };
    ElementType {
        id,
        name: crate::i18n::tr(key),
        data: TypeData::Sweep {
            profile,
            material: p.materials.first().map(|m| m.name.clone()).unwrap_or_default(),
            sketch: None,
        },
        info: Default::default(),
        uid: Default::default(),
    }
}

/// A profile from points or curves (one of them), checked, with the curves
/// it was drawn with (kept with the type, so arcs stay arcs when it is edited again).
type Profile = (Vec<DVec2>, Option<Sketch>);
fn profile_from(points: Option<&[Point]>, curves: Option<&[CurveArg]>) -> Result<Option<Profile>> {
    let (profile, sketch) = match (points, curves) {
        (Some(_), Some(_)) => return Err(EngineError::invalid("give profile or curves, not both")),
        (Some(pts), None) => (pts.iter().map(|p| point(*p)).collect::<Result<Vec<_>>>()?, None),
        (None, Some(curves)) => {
            let sketch = super::sketch::sketch_of(curves)?;
            (super::sketch::outline(&sketch)?, Some(sketch))
        }
        (None, None) => return Ok(None),
    };
    check_profile(&profile).map_err(|p| EngineError::invalid(p.message()))?;
    Ok(Some((profile, sketch)))
}

/// Create a swept profile: the profile of its type carried along `path` in
/// plan, `base_offset` above the level, on the left of the path walking
/// along it (`flip` puts it on the right); `closed` runs the path back to
/// its first point (around a room). Instead of points, the path may be
/// drawn with `curves` (lines and arcs chained end to end, or one circle),
/// as `floor_create` takes them: arcs stay arcs (the profile follows them
/// square to the path, and the volume is exact), the path runs the way the
/// first curve runs and is closed when the chain closes; a circle runs
/// counter-clockwise, so the profile is inside it unless flipped. A
/// project without a sweep type gets one (a baseboard).
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct SweepCreate {
    /// Points of the path in plan (at least 2; 3 when closed). Give these or `curves`.
    #[serde(default)]
    pub path: Vec<Point>,
    /// The path drawn with curves instead of points: lines, arcs (`bulge` or `through`) and
    /// circles (alone), as in `floor_create`; rectangles and polygons make closed paths of lines.
    #[serde(default)]
    pub curves: Option<Vec<CurveArg>>,
    /// Run the path back to its first point (with `curves`: the curves must close).
    #[serde(default)]
    pub closed: bool,
    /// Level id or name (default: the lowest level).
    #[serde(default)]
    pub level: Option<Ref>,
    /// Type id or name (default: the category's default type).
    #[serde(default, rename = "type")]
    pub type_ref: Option<Ref>,
    /// Height of the path above the level, meters (default 0).
    #[serde(default)]
    pub base_offset: Option<f64>,
    /// Put the profile on the right of the path.
    #[serde(default)]
    pub flip: bool,
}

impl Command for SweepCreate {
    const NAME: &'static str = "sweep_create";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let (path, bulges, closed) = match &self.curves {
            Some(_) if !self.path.is_empty() => return Err(EngineError::invalid("give path or curves, not both")),
            Some(curves) => {
                let sketch = super::sketch::sketch_of(curves)?;
                let made = path_from_curves(&sketch.curves).map_err(|e| EngineError::invalid(e.message()))?;
                if self.closed && !made.closed {
                    return Err(EngineError::invalid("closed: the curves do not close"));
                }
                (made.points, made.bulges, made.closed)
            }
            None => (points_path(&self.path, self.closed)?, Vec::new(), self.closed),
        };
        let p = s.project();
        let level = refs::level_or_first(p, self.level.as_ref())?;
        let base_offset = finite(self.base_offset.unwrap_or(0.0), "base_offset")?;
        let ty = match (&self.type_ref, p.default_type(Category::Sweep)) {
            (None, None) => None,
            (r, _) => Some(refs::type_of(p, Category::Sweep, r.as_ref())?),
        };
        let sweep = Sweep { path, closed, base_offset, flip: self.flip, bulges };
        if !sweep.is_valid() {
            return Err(EngineError::invalid(format!(
                "the path is too long or too short: at most {MAX_PATH_POINTS} points and {MAX_STATIONS} stations \
                 (arcs count one every 3°)"
            )));
        }
        let id = edit(s, t!("tx-create-sweep"), |e| {
            let type_id = match ty {
                Some(t) => t,
                None => {
                    let id = ElementTypeId(e.alloc_id());
                    e.add_type(new_sweep_type(e.project(), id))
                }
            };
            Ok(add_element(e, level, type_id, ElementKind::Sweep(sweep)))
        })?;
        Ok(created(s, id))
    }
}

/// A path from points: checked, repeated points dropped (and the last when
/// it repeats the first of a closed path).
fn points_path(points: &[Point], closed: bool) -> Result<Vec<DVec2>> {
    if points.len() > MAX_PATH_POINTS {
        return Err(EngineError::invalid(format!("the path has more than {MAX_PATH_POINTS} points")));
    }
    let mut path = points.iter().map(|p| point(*p)).collect::<Result<Vec<_>>>()?;
    path.dedup_by(|a, b| a.distance(*b) < 1e-6);
    if closed && path.len() > 1 && path.first().zip(path.last()).is_some_and(|(a, b)| a.distance(*b) < 1e-6) {
        path.pop();
    }
    let needed = if closed { 3 } else { 2 };
    if path.len() < needed {
        return Err(EngineError::invalid(format!("the path needs at least {needed} distinct points")));
    }
    Ok(path)
}

/// Create a sweep type: its profile drawn as points `[x, y]` (x across, to
/// the left of the path; y up from it, meters) or as `curves` (lines, arcs,
/// circles, rectangles… making one closed outline), shown in `material`.
/// Without a profile it is a baseboard 1.5 cm × 10 cm.
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct SweepTypeCreate {
    /// Type name (default: the template's baseboard name).
    #[serde(default)]
    pub name: Option<String>,
    /// The profile's points, in order (a closed outline that does not cross itself).
    #[serde(default)]
    pub profile: Option<Vec<Point>>,
    /// The profile drawn with curves instead of points.
    #[serde(default)]
    pub curves: Option<Vec<CurveArg>>,
    /// Material name (default: the project's first material).
    #[serde(default)]
    pub material: Option<String>,
}

impl Command for SweepTypeCreate {
    const NAME: &'static str = "sweep_type_create";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let profile = profile_from(self.profile.as_deref(), self.curves.as_deref())?;
        let material = match self.material {
            Some(m) => Some(refs::material(s.project(), &Ref::Name(m))?),
            None => None,
        };
        let name = self.name.filter(|n| !n.trim().is_empty());
        let id = edit(s, t!("tx-create-sweep-type"), |e| {
            let id = ElementTypeId(e.alloc_id());
            let mut ty = new_sweep_type(e.project(), id);
            if let TypeData::Sweep { profile: p, material: m, sketch: k } = &mut ty.data {
                if let Some((profile, sketch)) = profile {
                    *p = profile;
                    *k = sketch;
                }
                if let Some(material) = material {
                    *m = material;
                }
            }
            if let Some(name) = name {
                ty.name = name;
            }
            Ok(e.add_type(ty))
        })?;
        Ok(json!({ "id": id.0 }))
    }
}

/// Redraw the profile of a sweep type, as points `[x, y]` or `curves`; every
/// sweep of the type follows. The type keeps the curves (`type_get` shows
/// them as its `sketch`), so arcs stay arcs when the profile is edited again.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SweepProfileSet {
    /// Sweep type id or name.
    #[serde(rename = "type")]
    pub type_ref: Ref,
    /// The profile's points, in order.
    #[serde(default)]
    pub profile: Option<Vec<Point>>,
    /// The profile drawn with curves instead of points.
    #[serde(default)]
    pub curves: Option<Vec<CurveArg>>,
}

impl Command for SweepProfileSet {
    const NAME: &'static str = "sweep_profile_set";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let (profile, sketch) = profile_from(self.profile.as_deref(), self.curves.as_deref())?
            .ok_or_else(|| EngineError::invalid("give profile or curves"))?;
        let p = s.project();
        let id = refs::element_type(p, &self.type_ref)?;
        if !matches!(p.element_type(id).map(|t| &t.data), Some(TypeData::Sweep { .. })) {
            return Err(EngineError::invalid("not a sweep type"));
        }
        let points = profile.len();
        edit(s, t!("tx-modify-type"), |e| {
            if let Some(TypeData::Sweep { profile: p, sketch: k, .. }) = e.type_mut(id).map(|t| &mut t.data) {
                *p = profile;
                *k = sketch;
            }
            Ok(())
        })?;
        Ok(json!({ "type": id.0, "points": points }))
    }
}
