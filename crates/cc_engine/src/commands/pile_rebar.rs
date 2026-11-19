//! Reinforcement of pile caps and piles: a cap's bottom bars over its
//! piles, its top mesh, its side bars and its stirrups; a pile's bars around
//! its section projecting into the cap, and a spiral or ties around them.

use super::rebar::{bar_type, partition_arg, splice, stirrup_hooks};
use super::{add_element, edit, ids_json};
use crate::error::{EngineError, Result};
use crate::refs::{self, Ref, finite};
use crate::registry::{Command, CommandKind};
use crate::session::Session;
use crate::t;
use cc_core::pile::PileSection;
use cc_core::{
    Category, ElementId, ElementKind, ElementTypeId, LevelId, Project, Rebar, RebarLayout, Splice, TypeData, pile_bars,
    wall_bars,
};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// The bottom bars of a pile cap.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct CapBottomBars {
    /// Bar type id or name (default: the command's `type`).
    #[serde(default, rename = "type")]
    pub type_ref: Option<Ref>,
    /// Bars in each band over two piles (caps of three and four piles), or along the line of the
    /// piles (caps of two piles and of a line), 1 to 50 (default 4).
    #[serde(default)]
    pub count: Option<u32>,
    /// Distance between the bars of the mesh of the other caps (one pile, round and ring caps),
    /// meters (default 0.15; 6" in imperial projects).
    #[serde(default)]
    pub spacing: Option<f64>,
    /// Caps of four piles: also a band across each diagonal of the square (default false).
    #[serde(default)]
    pub diagonals: Option<bool>,
    /// The legs the bars' ends turn up the side faces by, meters, at most to the top less the
    /// cover (default: 20 bar diameters; 0 for straight bars).
    #[serde(default)]
    pub bend: Option<f64>,
}

/// The top mesh of a pile cap.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct CapTopBars {
    /// Bar type id or name (default: the type nearest 8 mm; 3/8" in imperial projects).
    #[serde(default, rename = "type")]
    pub type_ref: Option<Ref>,
    /// Distance between the bars, both ways, meters (default 0.15; 6" in imperial projects).
    #[serde(default)]
    pub spacing: Option<f64>,
    /// The legs the bars' ends turn down the side faces by, meters, at most to the bottom bars
    /// (default 0: straight bars).
    #[serde(default)]
    pub bend: Option<f64>,
}

/// Closed bars around a pile cap or a pile: side loops, stirrups or ties.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct CapLoops {
    /// Bar type id or name (default: the type nearest 8 mm for side bars, 5 mm for stirrups;
    /// 3/8" in imperial projects).
    #[serde(default, rename = "type")]
    pub type_ref: Option<Ref>,
    /// Their spacing, meters (default 0.15; 6" in imperial projects): side loops at most this far
    /// apart over the height, stirrups along the cap.
    #[serde(default)]
    pub spacing: Option<f64>,
    /// The angle the two hooks that close each loop are bent by, degrees, 90 to 180 (default
    /// 135); 0 for none.
    #[serde(default)]
    pub hook: Option<f64>,
    /// The straight end of each hook, in bar diameters (default 10; at least 5 cm).
    #[serde(default)]
    pub tail: Option<f64>,
}

/// Reinforce pile caps (`caps`, or those of `level` or of the project that
/// have no reinforcement yet), one set per family, in one undo step. The
/// sets are laid out from each cap's layout and outline whenever the model
/// changes, so they follow its type: change a cap type's layout and its
/// caps' bars are laid out for the new one.
///
/// - `bottom` bars over the piles: in a cap of three piles a band of
///   `count` bars between each two piles (along the sides of the triangle),
///   in a cap of four a band along each side of the square (and with
///   `diagonals` one across each diagonal), each band as wide as a pile and
///   centered on the line through the two piles; in a cap of two piles or of
///   a line, `count` bars along the line of the piles across the cap's width;
///   in a cap of one pile, a round cap and a ring cap, a mesh `spacing` apart
///   both ways. They lie on the piles' heads (the embedment, at least the
///   cover, above the cap's bottom), crossing bands one on the other, and
///   their ends turn up the side faces by `bend`.
/// - `top`: a mesh both ways under the top, its ends turned down by `bend`.
/// - `sides`: horizontal closed loops around the cap's outline (both faces of
///   a ring) at most `spacing` apart between the bottom and the top bars,
///   closed by two hooks at a corner as stirrups are.
/// - `stirrups`: vertical closed stirrups across a cap of two piles or of a
///   line, every `spacing` along it, as a beam's; other caps have no simple
///   conventional stirrups and get none.
///
/// Each family is its values or `false` for none (all four by default;
/// stirrups only in caps of two piles or a line). Bars lie `cover` inside
/// the faces: the stirrups outermost, the side loops inside them, the bottom
/// and top bars inside both. Bottom and top bars longer than `max_length` are
/// lap spliced. Returns the sets created, the caps reinforced and those
/// `skipped` (`{cap, reason}`: already reinforced when chosen by level or
/// project, or too small for the cover).
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct PileCapRebarCreate {
    /// Pile cap ids (default: the caps of `level`, or of the project, that have no
    /// reinforcement).
    #[serde(default)]
    pub caps: Vec<u64>,
    /// Level id or name, when no caps are given.
    #[serde(default)]
    pub level: Option<Ref>,
    /// Bar type id or name of the bottom bars (default: the type nearest 10 mm; 1/2" in
    /// imperial projects).
    #[serde(default, rename = "type")]
    pub type_ref: Option<Ref>,
    /// The bottom bars: their values, or `false` for none.
    #[serde(default)]
    pub bottom: Option<Value>,
    /// The top mesh: its values, or `false` for none.
    #[serde(default)]
    pub top: Option<Value>,
    /// The side loops: their values, or `false` for none.
    #[serde(default)]
    pub sides: Option<Value>,
    /// The stirrups of caps of two piles or of a line: their values, or `false` for none.
    #[serde(default)]
    pub stirrups: Option<Value>,
    /// Concrete cover, meters (default 0.025; 1 1/2" in imperial projects).
    #[serde(default)]
    pub cover: Option<f64>,
    /// Longest bottom or top bar, meters (default: the stock length, 12 m, or 40' in imperial
    /// projects): longer bars are lap spliced. 0 keeps whole bars.
    #[serde(default)]
    pub max_length: Option<f64>,
    /// Lap length of the splices, meters (default: 40 bar diameters).
    #[serde(default)]
    pub lap: Option<f64>,
    /// The part of the structure the sets belong to (e.g. "FOUNDATION"), which schedules filter
    /// and group by (default: none).
    #[serde(default)]
    pub partition: Option<String>,
}

/// A family's values: `None` for `false` (none), the defaults for a missing
/// value or `true`.
fn family<T: DeserializeOwned + Default>(value: &Option<Value>, what: &str) -> Result<Option<T>> {
    match value {
        None | Some(Value::Bool(true)) => Ok(Some(T::default())),
        Some(Value::Bool(false)) | Some(Value::Null) => Ok(None),
        Some(v) => {
            serde_json::from_value(v.clone()).map(Some).map_err(|e| EngineError::invalid(format!("{what}: {e}")))
        }
    }
}

/// A bar type given by reference, else the type nearest `diameter`.
fn bar(p: &Project, r: &Option<Ref>, diameter: f64) -> Result<ElementTypeId> {
    match r {
        Some(r) => refs::type_of(p, Category::Rebar, Some(r)),
        None => bar_type(p, diameter),
    }
}

/// The diameter of a bar type (1 cm when it is none).
fn diameter(p: &Project, type_id: ElementTypeId) -> f64 {
    match p.element_type(type_id).map(|t| &t.data) {
        Some(TypeData::Rebar { diameter, .. }) if diameter.is_finite() && *diameter > 0.0 => *diameter,
        _ => 0.01,
    }
}

/// A spacing of at least the least one.
fn spacing(value: f64, what: &str) -> Result<f64> {
    let value = finite(value, what)?;
    if value < wall_bars::MIN_SPACING {
        return Err(EngineError::invalid(format!("{what} must be at least {}", wall_bars::MIN_SPACING)));
    }
    Ok(value)
}

/// A length from 0 to `max`.
fn length(value: f64, what: &str, max: f64) -> Result<f64> {
    let value = finite(value, what)?;
    if !(0.0..=max).contains(&value) {
        return Err(EngineError::invalid(format!("{what} must be 0 to {max} m")));
    }
    Ok(value)
}

/// The sets to add: host, level, bar type, layout and splice.
type Todo = Vec<(ElementId, LevelId, ElementTypeId, RebarLayout, Option<Splice>)>;

impl Command for PileCapRebarCreate {
    const NAME: &'static str = "pile_cap_rebar_create";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let p = s.project();
        let [default_cover, default_spacing, main_bar, top_bar, stirrup_bar] = p.defaults().rebar;
        let cover = finite(self.cover.unwrap_or(default_cover), "cover")?.max(0.0);
        // The caps: those given, else those of the level or the project not reinforced yet.
        let mut skipped: Vec<Value> = Vec::new();
        let caps: Vec<&cc_core::Element> = if self.caps.is_empty() {
            let level = self.level.as_ref().map(|r| refs::level(p, r)).transpose()?;
            let mut out = Vec::new();
            for e in p.elements.values().filter(|e| level.is_none_or(|l| e.level == l)) {
                if p.pile_cap(e).is_none() {
                    continue;
                }
                if p.rebar_of(e.id).next().is_some() {
                    skipped.push(json!({"cap": e.id.0, "reason": "already reinforced"}));
                    continue;
                }
                out.push(e);
            }
            out
        } else {
            let mut out: Vec<&cc_core::Element> = Vec::new();
            for id in &self.caps {
                let id = refs::element(p, *id)?;
                let e = p
                    .element(id)
                    .filter(|e| p.pile_cap(e).is_some())
                    .ok_or_else(|| EngineError::invalid(format!("element {} is not a pile cap", id.0)))?;
                if !out.iter().any(|o| o.id == id) {
                    out.push(e);
                }
            }
            out
        };
        let main_type = bar(p, &self.type_ref, main_bar)?;
        let spliced = |ty| splice(p, ty, self.max_length, self.lap);
        // Each family's set: its type, layout and splices.
        let mut sets: Vec<(ElementTypeId, RebarLayout, Option<Splice>)> = Vec::new();
        if let Some(b) = family::<CapBottomBars>(&self.bottom, "bottom")? {
            let ty = if b.type_ref.is_some() { bar(p, &b.type_ref, main_bar)? } else { main_type };
            let count = b.count.unwrap_or(4);
            if !(1..=pile_bars::MAX_BAND).contains(&count) {
                return Err(EngineError::invalid(format!("bottom: count must be 1 to {}", pile_bars::MAX_BAND)));
            }
            let layout = RebarLayout::CapBottom {
                count,
                spacing: spacing(b.spacing.unwrap_or(default_spacing), "bottom: spacing")?,
                diagonals: b.diagonals.unwrap_or(false),
                bend: length(b.bend.unwrap_or(20.0 * diameter(p, ty)), "bottom: bend", pile_bars::MAX_BEND)?,
            };
            sets.push((ty, layout, spliced(ty)?));
        }
        if let Some(b) = family::<CapTopBars>(&self.top, "top")? {
            let ty = bar(p, &b.type_ref, top_bar)?;
            let layout = RebarLayout::CapTop {
                spacing: spacing(b.spacing.unwrap_or(default_spacing), "top: spacing")?,
                bend: length(b.bend.unwrap_or(0.0), "top: bend", pile_bars::MAX_BEND)?,
            };
            sets.push((ty, layout, spliced(ty)?));
        }
        if let Some(b) = family::<CapLoops>(&self.sides, "sides")? {
            let ty = bar(p, &b.type_ref, top_bar)?;
            let layout = RebarLayout::CapSides {
                spacing: spacing(b.spacing.unwrap_or(default_spacing), "sides: spacing")?,
                hooks: stirrup_hooks(b.hook, b.tail)?,
            };
            sets.push((ty, layout, None));
        }
        let stirrups = match family::<CapLoops>(&self.stirrups, "stirrups")? {
            Some(b) => {
                let ty = bar(p, &b.type_ref, stirrup_bar)?;
                let layout = RebarLayout::Stirrups {
                    spacing: spacing(b.spacing.unwrap_or(default_spacing), "stirrups: spacing")?,
                    hooks: stirrup_hooks(b.hook, b.tail)?,
                };
                Some((ty, layout))
            }
            None => None,
        };
        let mut todo: Todo = Vec::new();
        let mut reinforced = 0;
        for e in caps {
            if let Some(reason) = cc_geom::pile::cap_room_problem(p, e, cover) {
                skipped.push(json!({"cap": e.id.0, "reason": reason}));
                continue;
            }
            reinforced += 1;
            todo.extend(sets.iter().map(|(ty, layout, splice)| (e.id, e.level, *ty, layout.clone(), *splice)));
            // Stirrups across a cap of two piles or of a line only.
            if let (Some((ty, layout)), Some(c)) = (&stirrups, p.pile_cap(e))
                && cc_geom::pile::cap_is_beam_like(c.cap.layout, c.cap.positions().len())
            {
                todo.push((e.id, e.level, *ty, layout.clone(), None));
            }
        }
        if todo.is_empty() {
            return Ok(json!({"created": [], "caps": reinforced, "skipped": skipped}));
        }
        let created = add_sets(s, todo, cover, self.partition.as_deref())?;
        Ok(json!({"created": ids_json(&created), "caps": reinforced, "skipped": skipped}))
    }
}

/// Add the sets of `todo`, `cover` inside their hosts' faces, in one undo step.
fn add_sets(s: &mut Session, todo: Todo, cover: f64, partition: Option<&str>) -> Result<Vec<ElementId>> {
    for (_, _, _, layout, _) in &todo {
        if !layout.is_valid() {
            return Err(EngineError::invalid("a value is out of range"));
        }
    }
    let partition = partition_arg(partition)?;
    edit(s, t!("tx-create-rebar"), |e| {
        Ok(todo
            .into_iter()
            .map(|(host, level, ty, layout, splice)| {
                let rebar = Rebar { splice, partition: partition.clone(), ..Rebar::new(host, layout, cover) };
                add_element(e, level, ty, ElementKind::Rebar(rebar))
            })
            .collect::<Vec<_>>())
    })
}

/// Reinforce piles (`piles`, the piles of `caps`, or else the piles of
/// `level` or of the project that have no reinforcement yet), in one undo
/// step: longitudinal bars around the section (`count`, or as many as keep
/// them at most `spacing` apart; in a square pile at its corners and between
/// them) from `length` below the head (0: the whole pile, to the cover above
/// the toe) up to `projection` above it, into the cap; under a cap they
/// reach at most the cap's top less the cover. Around them, over the same
/// length, a spiral (a helix rising `pitch` a turn, drawn in chords with the
/// coil's exact length, `sqrt((2πr)² + pitch²)` a turn; in a square pile on
/// the circle inside its sides) or closed ties every `pitch`, closed by two
/// hooks (`hook`, `tail`). Bars longer than `max_length` are lap spliced in
/// lifts. The sets follow their piles: the piles of a cap whose type changes
/// keep theirs, and a pile the change adds takes the sets of the cap's first
/// reinforced pile. Returns the sets created, the piles reinforced and those
/// `skipped` (`{pile, reason}`: already reinforced when chosen by level or
/// project).
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct PileRebarCreate {
    /// Pile ids.
    #[serde(default)]
    pub piles: Vec<u64>,
    /// Pile cap ids whose piles are reinforced.
    #[serde(default)]
    pub caps: Vec<u64>,
    /// Level id or name, when no piles or caps are given (default: the whole project).
    #[serde(default)]
    pub level: Option<Ref>,
    /// Bar type id or name of the longitudinal bars (default: the type nearest 10 mm; 1/2" in
    /// imperial projects).
    #[serde(default, rename = "type")]
    pub type_ref: Option<Ref>,
    /// Longitudinal bars around the section, 1 to 200 (default: 6 in round piles, 4 in square
    /// ones, unless `spacing` is given; 0 for none).
    #[serde(default)]
    pub count: Option<u32>,
    /// Instead of `count`, the largest distance between the bars around the section, meters.
    #[serde(default)]
    pub spacing: Option<f64>,
    /// The length reinforced below the head, meters (default 0: the whole pile).
    #[serde(default)]
    pub length: Option<f64>,
    /// How far the longitudinal bars run on above the head into the cap, meters (default: 40
    /// bar diameters); under a cap, at most to its top less the cover.
    #[serde(default)]
    pub projection: Option<f64>,
    /// `spiral` (default in round piles), `ties` (default in square piles) or `none`.
    #[serde(default)]
    pub transverse: Option<String>,
    /// Bar type id or name of the spiral or ties (default: the type nearest 5 mm; 3/8" in
    /// imperial projects).
    #[serde(default)]
    pub transverse_type: Option<Ref>,
    /// The spiral's rise per turn, or the ties' spacing, meters (default 0.15; 6" in imperial
    /// projects).
    #[serde(default)]
    pub pitch: Option<f64>,
    /// Ties: the angle the two hooks that close each tie are bent by, degrees, 90 to 180
    /// (default 135); 0 for none.
    #[serde(default)]
    pub hook: Option<f64>,
    /// Ties: the straight end of each hook, in bar diameters (default 10; at least 5 cm).
    #[serde(default)]
    pub tail: Option<f64>,
    /// Concrete cover, meters (default 0.025; 1 1/2" in imperial projects).
    #[serde(default)]
    pub cover: Option<f64>,
    /// Longest longitudinal bar, meters (default: the stock length, 12 m, or 40' in imperial
    /// projects): longer bars are cut in lifts that lap. 0 keeps whole bars.
    #[serde(default)]
    pub max_length: Option<f64>,
    /// Lap length of the splices, meters (default: 40 bar diameters).
    #[serde(default)]
    pub lap: Option<f64>,
    /// The part of the structure the sets belong to (e.g. "FOUNDATION"), which schedules filter
    /// and group by (default: none).
    #[serde(default)]
    pub partition: Option<String>,
}

impl Command for PileRebarCreate {
    const NAME: &'static str = "pile_rebar_create";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let p = s.project();
        let [default_cover, default_spacing, main_bar, _, stirrup_bar] = p.defaults().rebar;
        let cover = finite(self.cover.unwrap_or(default_cover), "cover")?.max(0.0);
        let mut skipped: Vec<Value> = Vec::new();
        let mut piles: Vec<ElementId> = Vec::new();
        let mut take = |id: ElementId| {
            if !piles.contains(&id) {
                piles.push(id);
            }
        };
        for id in &self.piles {
            let id = refs::element(p, *id)?;
            if !matches!(p.element(id).map(|e| &e.kind), Some(ElementKind::Pile(_))) {
                return Err(EngineError::invalid(format!("element {} is not a pile", id.0)));
            }
            take(id);
        }
        for id in &self.caps {
            let id = refs::element(p, *id)?;
            if !p.is_pile_cap(id) {
                return Err(EngineError::invalid(format!("element {} is not a pile cap", id.0)));
            }
            let mut of: Vec<(u32, ElementId)> =
                p.piles_of(id).map(|(e, pile)| (pile.cap.map_or(0, |c| c.index), e.id)).collect();
            of.sort();
            of.into_iter().for_each(|(_, id)| take(id));
        }
        if self.piles.is_empty() && self.caps.is_empty() {
            let level = self.level.as_ref().map(|r| refs::level(p, r)).transpose()?;
            for e in p.elements.values().filter(|e| level.is_none_or(|l| e.level == l)) {
                if !matches!(e.kind, ElementKind::Pile(_)) {
                    continue;
                }
                if p.rebar_of(e.id).next().is_some() {
                    skipped.push(json!({"pile": e.id.0, "reason": "already reinforced"}));
                    continue;
                }
                take(e.id);
            }
        }
        let bars_type = bar(p, &self.type_ref, main_bar)?;
        let tie_type = bar(p, &self.transverse_type, stirrup_bar)?;
        if self.count.is_some_and(|c| c > pile_bars::MAX_AROUND) {
            return Err(EngineError::invalid(format!("count must be 0 to {}", pile_bars::MAX_AROUND)));
        }
        let around = self.spacing.map(|v| spacing(v, "spacing")).transpose()?;
        let reach = length(self.length.unwrap_or(0.0), "length", pile_bars::MAX_LENGTH)?;
        let projection = length(
            self.projection.unwrap_or(cc_core::Anchorage::DEFAULT_LENGTH * diameter(p, bars_type)),
            "projection",
            pile_bars::MAX_BEND,
        )?;
        let pitch = spacing(self.pitch.unwrap_or(default_spacing), "pitch")?;
        let hooks = stirrup_hooks(self.hook, self.tail)?;
        let transverse = self.transverse.as_deref();
        if !matches!(transverse, None | Some("spiral" | "ties" | "none")) {
            return Err(EngineError::invalid(format!(
                "transverse must be spiral, ties or none, not '{}'",
                transverse.unwrap_or_default()
            )));
        }
        if (self.hook.is_some() || self.tail.is_some()) && transverse == Some("spiral") {
            return Err(EngineError::invalid("`hook` and `tail` go with ties"));
        }
        let spliced = splice(p, bars_type, self.max_length, self.lap)?;
        let mut todo: Todo = Vec::new();
        for id in &piles {
            let Some(e) = p.element(*id) else { continue };
            let round = matches!(
                p.element_type(e.type_id).map(|t| &t.data),
                Some(TypeData::Pile { section: PileSection::Round { .. }, .. })
            );
            let count = match (self.count, around) {
                (Some(n), _) => n,
                (None, Some(_)) => 0,
                (None, None) => {
                    if round {
                        6
                    } else {
                        4
                    }
                }
            };
            if count > 0 || around.is_some() {
                let layout =
                    RebarLayout::PileLongitudinal { count, spacing: around.unwrap_or(0.0), length: reach, projection };
                todo.push((e.id, e.level, bars_type, layout, spliced));
            }
            let spiral = match transverse {
                Some("spiral") => Some(true),
                Some("ties") => Some(false),
                Some(_) => None,
                None => Some(round),
            };
            match spiral {
                Some(true) => {
                    todo.push((e.id, e.level, tie_type, RebarLayout::PileSpiral { pitch, length: reach }, None))
                }
                Some(false) => todo.push((
                    e.id,
                    e.level,
                    tie_type,
                    RebarLayout::PileTies { spacing: pitch, length: reach, hooks },
                    None,
                )),
                None => {}
            }
        }
        let count = piles.len();
        if todo.is_empty() {
            return Ok(json!({"created": [], "piles": count, "skipped": skipped}));
        }
        let created = add_sets(s, todo, cover, self.partition.as_deref())?;
        Ok(json!({"created": ids_json(&created), "piles": count, "skipped": skipped}))
    }
}
