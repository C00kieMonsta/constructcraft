//! Tags of elements in plans (see `cc_core::tag`): tag types, and how each
//! floor or ceiling plan adjusts the tag of an element (moved, turned,
//! another type, hidden, its leader).

use super::annotate::MAX_TAG_OFFSET;
use super::edit;
use crate::error::{EngineError, Result};
use crate::refs::{self, Point, Ref, point};
use crate::registry::{Command, CommandKind};
use crate::session::Session;
use crate::t;
use cc_core::tag::{LeaderMode, TagPlacement, TagStyle, TagSymbol, TagTarget, is_detail_component, is_tag_type_of};
use cc_core::view_mark::MarkKind;
use cc_core::{ElementId, ElementType, ElementTypeId, Project, TypeData, View, ViewId};
use glam::DVec2;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// A tag target by name: `door`, `window`, `room`, `column`, `beam`,
/// `foundation` (pad footings and pile caps), `pipe`, `conduit` or `cable_tray`.
pub(crate) fn target(name: &str) -> Result<TagTarget> {
    TagTarget::from_code(name).ok_or_else(|| {
        EngineError::invalid(format!(
            "unknown tag target '{name}': use door, window, room, column, beam, foundation, pipe, conduit or cable_tray"
        ))
    })
}

/// A leader mode by name: `auto`, `always` or `never`.
fn leader_mode(name: &str) -> Result<LeaderMode> {
    match name.trim().to_ascii_lowercase().as_str() {
        "auto" => Ok(LeaderMode::Auto),
        "always" => Ok(LeaderMode::Always),
        "never" => Ok(LeaderMode::Never),
        other => Err(EngineError::invalid(format!("unknown leader mode '{other}': use auto, always or never"))),
    }
}

fn leader_name(mode: LeaderMode) -> &'static str {
    match mode {
        LeaderMode::Auto => "auto",
        LeaderMode::Always => "always",
        LeaderMode::Never => "never",
    }
}

/// A tag type of `target`, by id or name.
pub(crate) fn tag_type(p: &Project, target: TagTarget, r: &Ref) -> Result<ElementTypeId> {
    let id = match r {
        Ref::Name(n) => {
            let found: Vec<_> = p.tag_types_of(target).filter(|t| t.name.trim() == n.trim()).map(|t| t.id).collect();
            match found[..] {
                [id] => id,
                [] => return Err(EngineError::not_found(format!("no {} tag type named '{n}'", target.code()))),
                _ => return Err(EngineError::invalid(format!("several {} tag types are named '{n}'", target.code()))),
            }
        }
        r => refs::element_type(p, r)?,
    };
    if !is_tag_type_of(p.element_type(id), target) {
        return Err(EngineError::invalid(format!("type {} is not a {} tag type", id.0, target.code())));
    }
    Ok(id)
}

/// The style is within its limits and a component symbol is a detail component.
fn check_style(p: &Project, style: &TagStyle) -> Result<()> {
    if style.clone().repair() > 0 {
        return Err(EngineError::invalid(format!(
            "a tag style has at most {} lines of at most {} characters, text sizes from {} to {} mm, \
             symbol sizes from 0 to {} mm, padding from 0 to {} mm and pens from {} to {} mm",
            cc_core::tag::MAX_LINES,
            cc_core::tag::MAX_TEXT,
            cc_core::tag::MIN_TEXT_SIZE,
            cc_core::tag::MAX_TEXT_SIZE,
            cc_core::tag::MAX_SYMBOL_SIZE,
            cc_core::tag::MAX_PADDING,
            cc_core::tag::MIN_PEN,
            cc_core::tag::MAX_PEN,
        )));
    }
    if let TagSymbol::Component(id) = style.symbol
        && !is_detail_component(p.element_type(id))
    {
        return Err(EngineError::invalid(format!("type {} is not a detail component", id.0)));
    }
    Ok(())
}

/// Create a tag type: what plans write by the elements of `target` and
/// how. `style` changes the target's default style (the tags plans have
/// always drawn), shaped like `params.style` in `type_get`: `lines` (up to
/// 8 of `{text, size}`: text with `{Field}` placeholders, a schedule field
/// by name such as `{Width}`, `{Height}` or `{Comments}`, or `{Mark}`,
/// `{TypeName}`, `{Name}`, `{Number}`, `{Area}`, `{Section}`, `{Level}`,
/// and for pipes, conduits and cable trays `{Systems}` (the systems'
/// codes), `{Circuits}` (the codes in parentheses), `{DiameterMark}`
/// (`Ø25`), `{SlopeMark}` (`i = 1 %`, sloped pipes) and `{TraySize}`
/// (`300×100`); two spaces between words are kept;
/// `{{` and `}}` write braces; size in paper mm), `symbol` (`None`,
/// `Circle`, `Hexagon`, `Rectangle`, `RoundedRectangle`, `Ellipse`,
/// `Diamond`, `Triangle`, `Underline`, or `{"Component": id}` of a detail
/// component drawn at paper size, a component meter being 1000 paper mm,
/// with its texts' fields filled in), `width` and `height` (paper mm; 0 fits
/// the text plus `padding`), `pen` (mm), `color` and `text_color` ([r, g, b]),
/// `fill` (a white box under the text), `leader` (`Auto`, `Always`, `Never`),
/// `leader_end` (a leader symbol), `orientation` (`Horizontal`, `AlongHost`)
/// and `crossed_when_unplaced`. A plan tags a target with its own choice
/// (`view_set` `tag_types`), else the project's first tag type of it, else
/// the default style; `type_set` changes a tag type.
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct TagTypeCreate {
    /// What the type tags: `door`, `window`, `room`, `column`, `beam`,
    /// `foundation` (pad footings and pile caps), `pipe`, `conduit` or `cable_tray`.
    pub target: String,
    /// Type name (default: the language's name for the target's tag).
    #[serde(default)]
    pub name: Option<String>,
    /// Fields of the style to change from the target's default.
    #[serde(default)]
    pub style: Option<Value>,
}

impl Command for TagTypeCreate {
    const NAME: &'static str = "tag_type_create";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let target = target(&self.target)?;
        let p = s.project();
        let base = target.default_style();
        let style: TagStyle = match &self.style {
            Some(patch) => {
                super::patch_value(&base, patch).map_err(|e| EngineError::invalid(format!("style: {}", e.message)))?
            }
            None => base,
        };
        check_style(p, &style)?;
        let taken = |n: &str| p.types_of(cc_core::Category::Tag).any(|t| t.name.trim() == n.trim());
        let name = match &self.name {
            Some(n) if n.trim().is_empty() || taken(n) => {
                return Err(EngineError::invalid(format!("the name '{n}' is empty or already used")));
            }
            Some(n) => n.trim().to_string(),
            None => {
                let base = crate::i18n::tr(target.template_key());
                (1..1000)
                    .map(|i| if i == 1 { base.clone() } else { format!("{base} {i}") })
                    .find(|n| !taken(n))
                    .unwrap_or(base)
            }
        };
        let id = edit(s, t!("tx-create-tag-type"), |e| {
            let id = ElementTypeId(e.alloc_id());
            let data = TypeData::Tag { target, style };
            Ok(e.add_type(ElementType { id, name, data, info: Default::default(), uid: Default::default() }))
        })?;
        Ok(super::types::type_out(s, id))
    }
}

/// The plan `r` names (floor or ceiling plans hold tags).
fn plan(p: &Project, r: &Ref) -> Result<ViewId> {
    let view = refs::view(p, r)?;
    if p.view(view).and_then(View::level).is_none() {
        return Err(EngineError::invalid("tags are adjusted in floor and ceiling plans"));
    }
    Ok(view)
}

/// The target of the element `id`, which must be tagged.
fn tagged(p: &Project, id: u64) -> Result<(ElementId, TagTarget)> {
    let id = refs::element(p, id)?;
    let e = p.element(id).ok_or_else(|| EngineError::not_found("element"))?;
    let target = TagTarget::of(e, p).ok_or_else(|| {
        EngineError::invalid(format!(
            "element {} has no tag: doors, windows, rooms, columns, beams, pad footings, pile caps, \
             pipes, conduits and cable trays do",
            id.0
        ))
    })?;
    Ok((id, target))
}

/// The target of the element `id`, or `None` for a grid; other elements are refused.
fn tagged_or_grid(p: &Project, raw: u64) -> Result<(ElementId, Option<TagTarget>)> {
    let id = refs::element(p, raw)?;
    if p.element(id).is_some_and(|e| matches!(e.kind, cc_core::ElementKind::Grid(_))) {
        return Ok((id, None));
    }
    tagged(p, raw).map(|(id, target)| (id, Some(target)))
}

/// Adjust the tags of elements in a floor or ceiling plan, in one undo
/// step: move them (`offset` from the default spot or `delta` from where
/// they are, meters in plan), turn them (`rotation`, degrees
/// counter-clockwise, instead of the style's orientation), tag them with
/// another tag type of their target (`type`), hide them (`hidden`) or set
/// when they get a leader to their element (`leader`: `auto`, when moved;
/// `always`; `never`). `null` clears a field (`offset` back home); `reset`
/// clears everything first. Each plan keeps its own adjustments.
///
/// Grids take part too, in plans and in the elevations and sections that
/// show them: `offset` and `delta` move a grid's end bubble (the top one in
/// elevations and sections) and `start_offset` its start bubble (the bottom
/// one), each with an elbow leader back to the line; `hidden` hides the
/// bubbles (the line stays). Grids take no `rotation`, `type` or `leader`.
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct TagSet {
    /// Plan id or name (or an elevation or a section, for grids only).
    pub view: Ref,
    /// Ids of the tagged elements (doors, windows, rooms, columns, beams,
    /// pad footings, pile caps, pipes, conduits and cable trays) or of grids.
    pub ids: Vec<u64>,
    /// Place the tags `[dx, dy]` meters from their default spot (`null`: back there).
    #[serde(default, deserialize_with = "refs::nullable", skip_serializing_if = "Option::is_none")]
    pub offset: Option<Option<Point>>,
    /// Move the tags by `[dx, dy]` meters from where they are.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delta: Option<Point>,
    /// Grids only: place the start bubble `[dx, dy]` meters (in the view's
    /// drawing coordinates) from where it sits (`null`: back there).
    #[serde(default, deserialize_with = "refs::nullable", skip_serializing_if = "Option::is_none")]
    pub start_offset: Option<Option<Point>>,
    /// The tags' angle, degrees counter-clockwise (`null`: as their style orients them).
    #[serde(default, deserialize_with = "refs::nullable", skip_serializing_if = "Option::is_none")]
    pub rotation: Option<Option<f64>>,
    /// Tag type id or name, of the elements' target (`null`: the plan's for the target).
    #[serde(default, rename = "type", deserialize_with = "refs::nullable", skip_serializing_if = "Option::is_none")]
    pub type_ref: Option<Option<Ref>>,
    /// Hide the tags, or show them again.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    /// `auto`, `always` or `never` (`null`: as the style says).
    #[serde(default, deserialize_with = "refs::nullable", skip_serializing_if = "Option::is_none")]
    pub leader: Option<Option<String>>,
    /// Put the tags back as the plan draws them by default, before the other fields apply.
    #[serde(default)]
    pub reset: bool,
}

/// Most tags one `tag_set` adjusts.
const MAX_IDS: usize = 10_000;

impl Command for TagSet {
    const NAME: &'static str = "tag_set";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let p = s.project();
        let view = refs::view(p, &self.view)?;
        let in_plan = p.view(view).and_then(View::level).is_some();
        let projected = p.view(view).is_some_and(|v| MarkKind::of_view(&v.kind).is_some());
        if self.ids.is_empty() || self.ids.len() > MAX_IDS {
            return Err(EngineError::invalid(format!("give 1 to {MAX_IDS} element ids")));
        }
        if matches!(self.offset, Some(Some(_))) && self.delta.is_some() {
            return Err(EngineError::invalid("give offset or delta, not both"));
        }
        let offset = match self.offset {
            Some(Some(o)) => Some(point(o)?),
            Some(None) => Some(DVec2::ZERO),
            None => None,
        };
        let delta = self.delta.map(point).transpose()?;
        let rotation = match self.rotation {
            Some(Some(r)) if r.is_finite() => Some(Some(r.rem_euclid(360.0))),
            Some(Some(_)) => return Err(EngineError::invalid("rotation must be a finite number")),
            Some(None) => Some(None),
            None => None,
        };
        let leader = match &self.leader {
            Some(Some(name)) => Some(Some(leader_mode(name)?)),
            Some(None) => Some(None),
            None => None,
        };
        let start_offset = match self.start_offset {
            Some(Some(o)) => Some(point(o)?),
            Some(None) => Some(DVec2::ZERO),
            None => None,
        };
        let current = p.view(view).map(|v| v.tags.clone()).unwrap_or_default();
        let mut placements: Vec<(ElementId, TagPlacement)> = Vec::new();
        for raw in &self.ids {
            let (id, target) = tagged_or_grid(p, *raw)?;
            match target {
                Some(_) if !in_plan => {
                    return Err(EngineError::invalid("tags are adjusted in floor and ceiling plans"));
                }
                None if !(in_plan || projected) => {
                    return Err(EngineError::invalid(
                        "grid bubbles are adjusted in floor and ceiling plans, elevations and sections",
                    ));
                }
                None if rotation.is_some() || self.type_ref.is_some() || leader.is_some() => {
                    return Err(EngineError::invalid(format!(
                        "grid {} takes offset, delta, start_offset, hidden and reset only",
                        id.0
                    )));
                }
                Some(_) if start_offset.is_some() => {
                    return Err(EngineError::invalid(format!(
                        "element {}: only grids have a start bubble (start_offset)",
                        id.0
                    )));
                }
                _ => {}
            }
            let mut t =
                if self.reset { TagPlacement::default() } else { current.get(&id).copied().unwrap_or_default() };
            if let Some(o) = offset {
                t.offset = o;
            }
            if let Some(d) = delta {
                t.offset += d;
            }
            if let Some(o) = start_offset {
                t.start_offset = o;
            }
            if t.offset.length() > MAX_TAG_OFFSET || t.start_offset.length() > MAX_TAG_OFFSET {
                return Err(EngineError::invalid(format!("a tag stays within {MAX_TAG_OFFSET} m of its element")));
            }
            if let Some(r) = rotation {
                t.rotation_deg = r;
            }
            match (&self.type_ref, target) {
                (Some(Some(r)), Some(target)) => t.tag_type = Some(tag_type(p, target, r)?),
                (Some(None), _) => t.tag_type = None,
                _ => {}
            }
            if let Some(h) = self.hidden {
                t.hidden = h;
            }
            if let Some(l) = leader {
                t.leader = l;
            }
            placements.push((id, t));
        }
        edit(s, t!("tx-set-tag"), |e| {
            let v = e.view_mut(view).ok_or_else(|| EngineError::not_found("view"))?;
            for (id, t) in &placements {
                if t.is_default() {
                    v.tags.remove(id);
                } else {
                    v.tags.insert(*id, *t);
                }
            }
            Ok(())
        })?;
        let tags: Vec<Value> = placements.iter().map(|(id, t)| placement_json(s.project(), *id, t)).collect();
        Ok(json!({ "view": view.0, "tags": tags }))
    }
}

fn placement_json(p: &Project, id: ElementId, t: &TagPlacement) -> Value {
    json!({
        "id": id.0,
        "offset": [t.offset.x, t.offset.y],
        "start_offset": [t.start_offset.x, t.start_offset.y],
        "rotation": t.rotation_deg,
        "type": t.tag_type.map(|ty| json!({"id": ty.0, "name": p.type_name(ty)})),
        "hidden": t.hidden,
        "leader": t.leader.map(leader_name),
    })
}

/// The tag of an element in a floor or ceiling plan: the plan's
/// adjustments (`placement`, see `tag_set`), the tag type it is drawn with
/// (`type`: null when the project has no tag type of the target and the
/// default style applies), its `target`, the `texts` its lines write
/// (empty lines included), whether it is `drawn` (hidden tags and tags
/// writing nothing are not), and, when drawn, its box (`center`, `half` its
/// size along and across the tag, meters in plan, and `rotation`,
/// degrees). For a grid (in a plan, an elevation or a section): its
/// `placement`, the `ends` with a bubble and the `bubbles` drawn (`start`,
/// `center`, `radius`).
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct TagGet {
    /// Plan id or name (or an elevation or a section, for a grid).
    pub view: Ref,
    /// Id of the tagged element or of a grid.
    pub id: u64,
}

impl Command for TagGet {
    const NAME: &'static str = "tag_get";
    const KIND: CommandKind = CommandKind::Query;
    fn run(self, s: &mut Session) -> Result<Value> {
        let p = s.project();
        if let (id, None) = tagged_or_grid(p, self.id)? {
            return grid_get(s, &self.view, id);
        }
        let view = plan(p, &self.view)?;
        let (id, target) = tagged(p, self.id)?;
        let drawing = s.drawing(view);
        s.geometry();
        let (p, geom) = (s.project(), s.geom());
        let v = p.view(view).ok_or_else(|| EngineError::not_found("view"))?;
        let e = p.element(id).ok_or_else(|| EngineError::not_found("element"))?;
        let placement = v.tags.get(&id).copied().unwrap_or_default();
        let ctx = cc_views::tag::TagCtx {
            project: p,
            geom,
            scale: v.scale,
            placements: &v.tags,
            types: &v.tag_types,
            label: &crate::i18n::tr,
        };
        let style = cc_views::tag::style_of(&ctx, e, target);
        let texts: Vec<String> = style.lines.iter().map(|l| cc_views::tag::expand_line(&ctx, e, &l.text)).collect();
        let ty = p.tag_type_for(v, e).map(|t| json!({"id": t.id.0, "name": t.name}));
        let handle = drawing.as_ref().and_then(|d| d.tags.iter().find(|t| t.element == id).copied());
        let drawn = drawing.as_ref().is_some_and(|d| d.prims.iter().any(|q| q.element() == Some(id)));
        let shown = !placement.hidden
            && (texts.iter().any(|t| !t.is_empty()) || matches!(style.symbol, TagSymbol::Component(_)));
        Ok(json!({
            "view": view.0,
            "id": id.0,
            "target": target.code(),
            "placement": placement_json(p, id, &placement),
            "type": ty,
            "texts": texts,
            "drawn": shown && drawn,
            "center": handle.map(|h| [h.center.x, h.center.y]),
            "half": handle.map(|h| [h.half.x, h.half.y]),
        "rotation": handle.map(|h| h.rotation.to_degrees()),
        }))
    }
}

/// `tag_get` of a grid: the view's adjustments of its bubbles, the ends
/// with a bubble and each bubble drawn (`start`, `center`, `radius`).
fn grid_get(s: &mut Session, r: &Ref, id: ElementId) -> Result<Value> {
    let p = s.project();
    let view = refs::view(p, r)?;
    let shows = p.view(view).is_some_and(|v| v.level().is_some() || MarkKind::of_view(&v.kind).is_some());
    if !shows {
        return Err(EngineError::invalid("grids are drawn in floor and ceiling plans, elevations and sections"));
    }
    let drawing = s.drawing(view);
    let p = s.project();
    let v = p.view(view).ok_or_else(|| EngineError::not_found("view"))?;
    let e = p.element(id).ok_or_else(|| EngineError::not_found("element"))?;
    let cc_core::ElementKind::Grid(grid) = &e.kind else { return Err(EngineError::not_found("grid")) };
    let placement = v.tags.get(&id).copied().unwrap_or_default();
    let (_, _, ends) = cc_views::grid_style(p, e, grid);
    let bubbles: Vec<Value> = drawing
        .as_ref()
        .map(|d| d.bubbles.iter().filter(|b| b.element == id).collect::<Vec<_>>())
        .unwrap_or_default()
        .into_iter()
        .map(|b| json!({"start": b.start, "center": [b.center.x, b.center.y], "radius": b.radius}))
        .collect();
    Ok(json!({
        "view": view.0,
        "id": id.0,
        "target": "grid",
        "placement": placement_json(p, id, &placement),
        "ends": ends.code(),
        "bubbles": bubbles,
    }))
}

/// A view's tag type per target, as `view_get` shows them.
pub(crate) fn view_tag_types_json(p: &Project, v: &View) -> Value {
    if v.level().is_none() {
        return Value::Null;
    }
    let map: serde_json::Map<String, Value> = v
        .tag_types
        .iter()
        .map(|(target, id)| (target.code().to_string(), json!({"id": id.0, "name": p.type_name(*id)})))
        .collect();
    Value::Object(map)
}

/// The tag types `patch` (target name → type id or name, or `null` to
/// follow the project's first) makes of a plan's.
pub(crate) fn apply_view_tag_types(
    p: &Project,
    v: &View,
    patch: &BTreeMap<String, Option<Ref>>,
) -> Result<BTreeMap<TagTarget, ElementTypeId>> {
    if v.level().is_none() {
        return Err(EngineError::invalid("only floor and ceiling plans have tag types"));
    }
    let mut out = v.tag_types.clone();
    for (name, r) in patch {
        let target = target(name)?;
        match r {
            Some(r) => {
                out.insert(target, tag_type(p, target, r)?);
            }
            None => {
                out.remove(&target);
            }
        }
    }
    Ok(out)
}

/// The services plan `r` names, with its level and discipline (fitting labels live there).
fn services_plan(p: &Project, r: &Ref) -> Result<(ViewId, cc_core::LevelId, cc_core::MepDiscipline)> {
    let view = refs::view(p, r)?;
    match p.view(view).map(|v| &v.kind) {
        Some(cc_core::ViewKind::FloorPlan { level, mep: Some(d), .. }) => Ok((view, *level, *d)),
        _ => Err(EngineError::invalid("fittings are labelled in services floor plans")),
    }
}

/// A fitting key in its canonical form (`"15-12"` is `"12-15"`).
fn fitting_key_arg(key: &str) -> Result<String> {
    let ids: Option<Vec<ElementId>> = key.split('-').map(|s| s.trim().parse::<u64>().ok().map(ElementId)).collect();
    match ids {
        Some(ids) if !ids.is_empty() && ids.len() <= cc_core::tag::MAX_FITTING_RUNS => {
            Ok(cc_core::tag::fitting_key(&ids))
        }
        _ => Err(EngineError::invalid(format!(
            "'{key}' is not a fitting key: the ids of the runs meeting at the fitting joined by '-', \
             as fitting_tags lists them"
        ))),
    }
}

/// Adjust the labels of fittings in a services floor plan (shown with
/// `view_set` `fitting_tags`), in one undo step: move them (`offset` from
/// their default spot or `delta` from where they are, meters in plan; a
/// label moved off its fitting gets a leader to it), turn them
/// (`rotation`), hide them (`hidden`) or set when they get a leader
/// (`leader`: `auto`, `always`, `never`); `reset` clears everything first.
/// Fittings are not elements: a fitting is named by its `key`, the ids of
/// the runs meeting at it joined by `-` (`fitting_tags` lists them), and
/// its label keeps its adjustments while those runs meet there (they go
/// when one of the runs is deleted).
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct FittingTagSet {
    /// Services plan id or name.
    pub view: Ref,
    /// Keys of the fittings (`"12-15"`), of the plan's level and discipline.
    pub keys: Vec<String>,
    /// Place the labels `[dx, dy]` meters from their default spot (`null`: back there).
    #[serde(default, deserialize_with = "refs::nullable", skip_serializing_if = "Option::is_none")]
    pub offset: Option<Option<Point>>,
    /// Move the labels by `[dx, dy]` meters from where they are.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delta: Option<Point>,
    /// The labels' angle, degrees counter-clockwise (`null`: level).
    #[serde(default, deserialize_with = "refs::nullable", skip_serializing_if = "Option::is_none")]
    pub rotation: Option<Option<f64>>,
    /// Hide the labels, or show them again.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    /// `auto`, `always` or `never` (`null`: auto).
    #[serde(default, deserialize_with = "refs::nullable", skip_serializing_if = "Option::is_none")]
    pub leader: Option<Option<String>>,
    /// Put the labels back as the plan draws them by default, before the other fields apply.
    #[serde(default)]
    pub reset: bool,
}

impl Command for FittingTagSet {
    const NAME: &'static str = "fitting_tag_set";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let (view, level, discipline) = services_plan(s.project(), &self.view)?;
        if self.keys.is_empty() || self.keys.len() > MAX_IDS {
            return Err(EngineError::invalid(format!("give 1 to {MAX_IDS} fitting keys")));
        }
        if matches!(self.offset, Some(Some(_))) && self.delta.is_some() {
            return Err(EngineError::invalid("give offset or delta, not both"));
        }
        let offset = match self.offset {
            Some(Some(o)) => Some(point(o)?),
            Some(None) => Some(DVec2::ZERO),
            None => None,
        };
        let delta = self.delta.map(point).transpose()?;
        let rotation = match self.rotation {
            Some(Some(r)) if r.is_finite() => Some(Some(r.rem_euclid(360.0))),
            Some(Some(_)) => return Err(EngineError::invalid("rotation must be a finite number")),
            Some(None) => Some(None),
            None => None,
        };
        let leader = match &self.leader {
            Some(Some(name)) => Some(Some(leader_mode(name)?)),
            Some(None) => Some(None),
            None => None,
        };
        let keys: Vec<String> = self.keys.iter().map(|k| fitting_key_arg(k)).collect::<Result<_>>()?;
        let geom = s.geometry();
        let shown: std::collections::BTreeSet<String> =
            cc_views::plan_fittings(geom, level, discipline).map(|f| cc_core::tag::fitting_key(&f.runs)).collect();
        if let Some(k) = keys.iter().find(|k| !shown.contains(*k)) {
            return Err(EngineError::not_found(format!("no fitting {k} in this plan")));
        }
        let current = s.project().view(view).map(|v| v.fitting_placements.clone()).unwrap_or_default();
        let mut placements: Vec<(String, TagPlacement)> = Vec::new();
        for key in keys {
            let mut t =
                if self.reset { TagPlacement::default() } else { current.get(&key).copied().unwrap_or_default() };
            if let Some(o) = offset {
                t.offset = o;
            }
            if let Some(d) = delta {
                t.offset += d;
            }
            if t.offset.length() > MAX_TAG_OFFSET {
                return Err(EngineError::invalid(format!("a tag stays within {MAX_TAG_OFFSET} m of its element")));
            }
            if let Some(r) = rotation {
                t.rotation_deg = r;
            }
            if let Some(h) = self.hidden {
                t.hidden = h;
            }
            if let Some(l) = leader {
                t.leader = l;
            }
            placements.push((key, t));
        }
        edit(s, t!("tx-set-fitting-tag"), |e| {
            let v = e.view_mut(view).ok_or_else(|| EngineError::not_found("view"))?;
            for (key, t) in &placements {
                if t.is_default() {
                    v.fitting_placements.remove(key);
                } else {
                    v.fitting_placements.insert(key.clone(), *t);
                }
            }
            Ok(())
        })?;
        let p = s.project();
        let tags: Vec<Value> = placements.iter().map(|(k, t)| fitting_placement_json(p, k, t)).collect();
        Ok(json!({ "view": view.0, "fittings": tags }))
    }
}

fn fitting_placement_json(p: &Project, key: &str, t: &TagPlacement) -> Value {
    let mut v = placement_json(p, ElementId(0), t);
    if let Some(m) = v.as_object_mut() {
        m.remove("id");
        m.remove("start_offset");
        m.remove("type");
        m.insert("key".into(), json!(key));
    }
    v
}

/// The fittings of a services floor plan (its level's fittings of its
/// discipline) and their labels: each fitting's `key` (the runs meeting
/// there, as `fitting_tag_set` takes it), `runs`, `kind`, `point`, the
/// label's `texts` (the catalog pieces' names, else the kind and sizes;
/// a `missing` fitting adds why), the plan's `placement`, and whether the
/// label is `drawn` (the plan's `fitting_tags` is on and it is not hidden)
/// with its box (`center`, `half`, `rotation` degrees).
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct FittingTags {
    /// Services plan id or name.
    pub view: Ref,
}

impl Command for FittingTags {
    const NAME: &'static str = "fitting_tags";
    const KIND: CommandKind = CommandKind::Query;
    fn run(self, s: &mut Session) -> Result<Value> {
        let (view, level, discipline) = services_plan(s.project(), &self.view)?;
        let drawing = s.drawing(view);
        s.geometry();
        let (p, geom) = (s.project(), s.geom());
        let v = p.view(view).ok_or_else(|| EngineError::not_found("view"))?;
        let ctx = cc_views::tag::TagCtx {
            project: p,
            geom,
            scale: v.scale,
            placements: &v.tags,
            types: &v.tag_types,
            label: &crate::i18n::tr,
        };
        let fittings: Vec<Value> = cc_views::plan_fittings(geom, level, discipline)
            .map(|f| {
                let key = cc_core::tag::fitting_key(&f.runs);
                let (texts, missing) = cc_views::fitting_lines(&ctx, f);
                let placement = v.fitting_placements.get(&key).copied().unwrap_or_default();
                let handle = drawing.as_ref().and_then(|d| d.fitting_tags.iter().find(|t| t.key == key).map(|t| t.tag));
                json!({
                    "key": key,
                    "runs": f.runs.iter().map(|r| r.0).collect::<Vec<_>>(),
                    "kind": f.kind.key().trim_start_matches("fitting-"),
                    "point": [f.point.x, f.point.y, f.point.z],
                    "texts": texts,
                    "missing": missing,
                    "placement": fitting_placement_json(p, &key, &placement),
                    "drawn": handle.is_some(),
                    "center": handle.map(|h| [h.center.x, h.center.y]),
                    "half": handle.map(|h| [h.half.x, h.half.y]),
                    "rotation": handle.map(|h| h.rotation.to_degrees()),
                })
            })
            .collect();
        Ok(json!({ "view": view.0, "fitting_tags": v.fitting_tags, "fittings": fittings }))
    }
}
