//! Wall elevation profiles (see [`cc_core::wall_profile`]): the outline of a
//! wall's face in its own elevation plane, `u` meters along its location
//! line from its start and `z` meters above its base, so its base and top
//! need not be level. `wall_profile_set` draws it (also `sketch_set` on a
//! wall, and `wall_create`'s `profile`), `wall_profile_slope` sets a base and
//! a top running straight from one end to the other, `wall_profile_remove`
//! takes it off; `sketch_get` gives its curves back.

use super::sketch::{CurveArg, sketch_of};
use super::{created, edit};
use crate::error::{EngineError, Result};
use crate::refs::{self, Point, finite, points};
use crate::registry::{Command, CommandKind};
use crate::session::Session;
use crate::t;
use cc_core::wall_profile::{ProfileError, WallProfile};
use cc_core::{Element, ElementId, ElementKind, Project, Wall};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// A profile's error as a command error.
pub(crate) fn profile_error(e: ProfileError) -> EngineError {
    EngineError::invalid(e.to_string())
}

/// Whether `wall` (of `e`) may take a profile: straight, not a curtain wall
/// (`curtain`: its type is one), not attached to a roof.
pub(crate) fn check_takes_profile(p: &Project, e: &Element, wall: &Wall) -> Result<()> {
    let id = e.id.0;
    if wall.spline.is_some() {
        return Err(EngineError::invalid(format!("wall {id} is a spline wall: it takes no elevation profile yet")));
    }
    if wall.arc().is_some() {
        return Err(EngineError::invalid(format!(
            "wall {id} is curved: only straight walls take an elevation profile"
        )));
    }
    if p.element_type(e.type_id).is_some_and(|t| t.curtain().is_some()) {
        return Err(EngineError::invalid(format!("wall {id} is a curtain wall: it takes no elevation profile")));
    }
    if wall.attach_top.is_some() {
        return Err(EngineError::invalid(format!(
            "the top of wall {id} is attached to a roof, ceiling or floor: detach it (wall_attach with detach) first"
        )));
    }
    if p.wall_framing_of(e.id).is_some() {
        return Err(EngineError::invalid(format!(
            "wall {id} is framed: framing follows walls with a level base and top only"
        )));
    }
    Ok(())
}

/// The wall `raw` names, checked to take a profile.
fn wall_of(p: &Project, raw: u64) -> Result<(ElementId, Wall)> {
    let id = refs::element(p, raw)?;
    let e = p.element(id).ok_or_else(|| EngineError::not_found(format!("no element {raw}")))?;
    let w = e.as_wall().ok_or_else(|| EngineError::invalid(format!("element {raw} is not a wall")))?;
    check_takes_profile(p, e, w)?;
    Ok((id, w.clone()))
}

/// Give a wall the profile, in one undo step.
pub(crate) fn set_profile(s: &mut Session, id: ElementId, profile: Option<WallProfile>) -> Result<()> {
    edit(s, t!("tx-wall-profile"), |e| {
        match e.element_mut(id).map(|x| &mut x.kind) {
            Some(ElementKind::Wall(w)) => w.profile = profile,
            _ => return Err(EngineError::not_found("wall")),
        }
        Ok(())
    })
}

/// The profile `curves` or `points` draw for `wall`.
pub(crate) fn profile_of(wall: &Wall, curves: Option<&[CurveArg]>, pts: Option<&[Point]>) -> Result<WallProfile> {
    let length = wall.length();
    match (curves, pts) {
        (Some(c), None) => WallProfile::from_sketch(sketch_of(c)?, length).map_err(profile_error),
        (None, Some(p)) => WallProfile::from_outline(points(p)?, length).map_err(profile_error),
        _ => Err(EngineError::invalid("give the profile's `curves` or its `points`, not both")),
    }
}

/// What `element_get` shows of a wall's profile: the area of its face, its
/// lowest and highest points above the wall's base (`min_z`, `max_z`), and
/// where it starts and ends along the wall.
pub(crate) fn derived(profile: &WallProfile) -> Value {
    let (start, end) = profile.span();
    let (min_z, max_z) = profile.z_range();
    json!({ "area": profile.area(), "min_z": min_z, "max_z": max_z, "start": start, "end": end })
}

/// Give a wall an elevation profile: the outline of its face drawn in its
/// own elevation plane, `u` meters along its location line from its start
/// and `z` meters above its base (`[u, z]` points), so its base and top need
/// not be level (a top following sloping ground, a stepped base, a gable, an
/// arched top). The outline is one closed loop that does not cross itself
/// and that every vertical line crosses at most twice (a bottom and a top
/// line along the wall, vertical ends and steps allowed), with no holes,
/// within the wall's length (it may stop short of its ends: the wall is then
/// only that long there) and at most 100 m below and 1000 m above its base.
/// Straight walls only, not curtain walls, framed walls or walls whose top is
/// attached to a roof. The wall's body is the outline across its thickness;
/// its height and top constraint stay its nominal box. Doors and windows
/// that do not lie inside the outline cut nothing. When the wall's length
/// changes the profile stays where it is along the wall, is cut at the
/// wall's ends and follows an end it reached when the wall grows there.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WallProfileSet {
    /// Wall id.
    pub wall: u64,
    /// The outline as curves in the wall's elevation frame (`[u, z]`
    /// coordinates; lines, arcs, splines, rectangles and polygons), kept
    /// so `sketch_get` gives them back.
    #[serde(default)]
    pub curves: Option<Vec<CurveArg>>,
    /// The outline as points `[[u, z], …]`, instead of `curves`.
    #[serde(default)]
    pub points: Option<Vec<Point>>,
}

impl Command for WallProfileSet {
    const NAME: &'static str = "wall_profile_set";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let (id, wall) = wall_of(s.project(), self.wall)?;
        let profile = profile_of(&wall, self.curves.as_deref(), self.points.as_deref())?;
        set_profile(s, id, Some(profile))?;
        Ok(created(s, id))
    }
}

/// Give a wall a base and a top running straight from one end to the other
/// (an elevation profile, see `wall_profile_set`): heights above the wall's
/// base at its start and at its end. The base defaults to 0 and the top to
/// the wall's height at both ends, so `{"wall": 5, "top_end": 1.2}` slopes
/// the top down to 1.2 m at the end (a wall along sloping ground).
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WallProfileSlope {
    /// Wall id.
    pub wall: u64,
    /// Height of the top at the wall's start, meters above its base (default: its height).
    #[serde(default)]
    pub top_start: Option<f64>,
    /// Height of the top at the wall's end, meters above its base (default: its height).
    #[serde(default)]
    pub top_end: Option<f64>,
    /// Height of the base at the wall's start, meters above its base (default 0).
    #[serde(default)]
    pub base_start: Option<f64>,
    /// Height of the base at the wall's end, meters above its base (default 0).
    #[serde(default)]
    pub base_end: Option<f64>,
}

impl Command for WallProfileSlope {
    const NAME: &'static str = "wall_profile_slope";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let p = s.project();
        let (id, wall) = wall_of(p, self.wall)?;
        let height = p.element(id).map_or(wall.height, |e| p.wall_height(e, &wall));
        let value = |v: Option<f64>, default: f64, what: &str| finite(v.unwrap_or(default), what);
        let base = (value(self.base_start, 0.0, "base_start")?, value(self.base_end, 0.0, "base_end")?);
        let top = (value(self.top_start, height, "top_start")?, value(self.top_end, height, "top_end")?);
        if top.0 <= base.0 || top.1 <= base.1 {
            return Err(EngineError::invalid("the top must be above the base at both ends"));
        }
        let profile = WallProfile::sloped(wall.length(), base, top).map_err(profile_error)?;
        set_profile(s, id, Some(profile))?;
        Ok(created(s, id))
    }
}

/// Take a wall's elevation profile off: its base and top are level again,
/// at its base and its height.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WallProfileRemove {
    /// Wall id.
    pub wall: u64,
}

impl Command for WallProfileRemove {
    const NAME: &'static str = "wall_profile_remove";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let p = s.project();
        let id = refs::element(p, self.wall)?;
        let wall = p
            .element(id)
            .and_then(Element::as_wall)
            .ok_or_else(|| EngineError::invalid(format!("element {} is not a wall", self.wall)))?;
        if wall.profile.is_none() {
            return Err(EngineError::invalid(format!("wall {} has no elevation profile", self.wall)));
        }
        set_profile(s, id, None)?;
        Ok(created(s, id))
    }
}
