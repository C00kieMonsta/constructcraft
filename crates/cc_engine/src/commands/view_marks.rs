//! Marks of sections and elevations in plans (see `cc_core::view_mark`):
//! view mark types, the type a section or an elevation is marked with, and
//! where each floor plan puts the marker of an elevation.

use super::annotate::MAX_TAG_OFFSET;
use super::edit;
use crate::error::{EngineError, Result};
use crate::refs::{self, Point, Ref, point};
use crate::registry::{Command, CommandKind};
use crate::session::Session;
use crate::t;
use cc_core::tag::{TagSymbol, is_detail_component};
use cc_core::view_mark::{MarkKind, MarkStyle, is_mark_type_of};
use cc_core::{ElementType, ElementTypeId, Project, TypeData, View, ViewKind};
use glam::DVec2;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// A mark kind by name: `section` or `elevation`.
fn kind(name: &str) -> Result<MarkKind> {
    MarkKind::from_code(name)
        .ok_or_else(|| EngineError::invalid(format!("unknown mark kind '{name}': use section or elevation")))
}

/// A view mark type of `kind`, by id or name.
fn mark_type(p: &Project, kind: MarkKind, r: &Ref) -> Result<ElementTypeId> {
    let id = match r {
        Ref::Name(n) => {
            let found: Vec<_> = p.mark_types_of(kind).filter(|t| t.name.trim() == n.trim()).map(|t| t.id).collect();
            match found[..] {
                [id] => id,
                [] => return Err(EngineError::not_found(format!("no {} mark type named '{n}'", kind.code()))),
                _ => return Err(EngineError::invalid(format!("several {} mark types are named '{n}'", kind.code()))),
            }
        }
        r => refs::element_type(p, r)?,
    };
    if !is_mark_type_of(p.element_type(id), kind) {
        return Err(EngineError::invalid(format!("type {} is not a {} mark type", id.0, kind.code())));
    }
    Ok(id)
}

/// The style is within its limits and a component symbol is a detail component.
fn check_style(p: &Project, style: &MarkStyle) -> Result<()> {
    if style.clone().repair() > 0 {
        return Err(EngineError::invalid(format!(
            "a mark style has a head from 0 to {} mm, texts of at most {} characters from {} to {} mm high \
             and a line weight from {} to {} mm",
            cc_core::view_mark::MAX_HEAD,
            cc_core::view_mark::MAX_TEXT,
            cc_core::view_mark::MIN_TEXT_SIZE,
            cc_core::view_mark::MAX_TEXT_SIZE,
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

/// Create a view mark type: how plans mark sections (`kind: "section"`)
/// or elevations (`"elevation"`). `style` changes the kind's default style
/// (the marks plans have always drawn), shaped like `params.style` in
/// `type_get`: `head` (diameter, paper mm; 0 draws it 0.5 m across in the
/// model, as elevation markers always were), `symbol` (`Circle`, `Hexagon`,
/// `Rectangle`, `RoundedRectangle`, `Ellipse`, `Diamond`, `Triangle`,
/// `Underline`, `None`, or `{"Component": id}` of a detail component drawn
/// at paper size, a component meter being 1000 paper mm, its texts' fields
/// filled in), `split` (a line across the head), `top`, `bottom` (the texts
/// in the head's halves) and `label` (beside it), each `{text, size}` with
/// `{Number}` (the drawing's number on its sheet, else its letter),
/// `{Sheet}` (its sheet's number, else a dash), `{Name}` and `{Letter}`
/// (the last word of the name), `arrow` (`Filled`, `Narrow`, `Open`,
/// `None`), `heads` (the ends of a section line with a head: `End`,
/// `Start`, `Both`, `None`), `line_weight` (mm), `color` and `arrow_color`
/// ([r, g, b]). A section or an elevation is marked with its own type
/// (`view_set` `mark_type`), else the project's first of its kind, else the
/// default style; `type_set` changes a mark type.
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct ViewMarkTypeCreate {
    /// What the type marks: `section` or `elevation`.
    pub kind: String,
    /// Type name (default: the language's name for the kind's mark).
    #[serde(default)]
    pub name: Option<String>,
    /// Fields of the style to change from the kind's default.
    #[serde(default)]
    pub style: Option<Value>,
}

impl Command for ViewMarkTypeCreate {
    const NAME: &'static str = "view_mark_type_create";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let kind = kind(&self.kind)?;
        let p = s.project();
        let base = kind.default_style();
        let style: MarkStyle = match &self.style {
            Some(patch) => {
                super::patch_value(&base, patch).map_err(|e| EngineError::invalid(format!("style: {}", e.message)))?
            }
            None => base,
        };
        check_style(p, &style)?;
        let taken = |n: &str| p.types_of(cc_core::Category::ViewMark).any(|t| t.name.trim() == n.trim());
        let name = match &self.name {
            Some(n) if n.trim().is_empty() || taken(n) => {
                return Err(EngineError::invalid(format!("the name '{n}' is empty or already used")));
            }
            Some(n) => n.trim().to_string(),
            None => {
                let base = crate::i18n::tr(kind.template_key());
                (1..1000)
                    .map(|i| if i == 1 { base.clone() } else { format!("{base} {i}") })
                    .find(|n| !taken(n))
                    .unwrap_or(base)
            }
        };
        let id = edit(s, t!("tx-create-view-mark-type"), |e| {
            let id = ElementTypeId(e.alloc_id());
            let data = TypeData::ViewMark { kind, style };
            Ok(e.add_type(ElementType { id, name, data, info: Default::default(), uid: Default::default() }))
        })?;
        Ok(super::types::type_out(s, id))
    }
}

/// A section's or an elevation's own mark type, as `view_get` shows it
/// (null for other views and for one marked with the project's first).
pub(crate) fn view_mark_type_json(p: &Project, v: &View) -> Value {
    match v.mark_type.filter(|_| MarkKind::of_view(&v.kind).is_some()) {
        Some(id) => json!({"id": id.0, "name": p.type_name(id)}),
        None => Value::Null,
    }
}

/// The mark type `r` (a type id or name, or `None` for the project's first
/// of the kind) makes of a section's or an elevation's.
pub(crate) fn apply_view_mark_type(p: &Project, v: &View, r: Option<&Ref>) -> Result<Option<ElementTypeId>> {
    let kind = MarkKind::of_view(&v.kind)
        .ok_or_else(|| EngineError::invalid("only sections and elevations have a mark type"))?;
    r.map(|r| mark_type(p, kind, r)).transpose()
}

/// Move the marker of an elevation in a floor plan: by `delta` from where
/// it is, to `offset` from where it sits by default (outside the model,
/// looking at it), or back there with `reset`. Each plan keeps its own
/// positions, in one undo step.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MarkMove {
    /// Floor plan id or name.
    pub plan: Ref,
    /// The elevation view (id or name) whose marker moves.
    pub view: Ref,
    /// Move by `[dx, dy]` meters in plan.
    #[serde(default)]
    pub delta: Option<Point>,
    /// Place the marker `[dx, dy]` meters from where it sits by default.
    #[serde(default)]
    pub offset: Option<Point>,
    /// Put the marker back where it sits by default.
    #[serde(default)]
    pub reset: bool,
}

impl Command for MarkMove {
    const NAME: &'static str = "mark_move";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let p = s.project();
        let plan = refs::view(p, &self.plan)?;
        if !p.view(plan).is_some_and(|v| matches!(v.kind, ViewKind::FloorPlan { .. })) {
            return Err(EngineError::invalid("elevation markers are moved in floor plans"));
        }
        let view = refs::view(p, &self.view)?;
        match p.view(view) {
            Some(v) if matches!(v.kind, ViewKind::Elevation { .. }) && !v.is_detail() => {}
            Some(v) if v.is_detail() => {
                return Err(EngineError::invalid("a detail view is marked by its callout, not in plans"));
            }
            _ => return Err(EngineError::invalid("only elevations have a marker to move")),
        }
        let current = p.view(plan).and_then(|v| v.mark_offsets.get(&view).copied()).unwrap_or(DVec2::ZERO);
        let new = match (self.delta, self.offset, self.reset) {
            (Some(d), None, false) => current + point(d)?,
            (None, Some(o), false) => point(o)?,
            (None, None, true) => DVec2::ZERO,
            _ => return Err(EngineError::invalid("give one of delta, offset or reset")),
        };
        if !(new.is_finite() && new.length() <= MAX_TAG_OFFSET) {
            return Err(EngineError::invalid(format!("a marker stays within {MAX_TAG_OFFSET} m of where it sits")));
        }
        edit(s, t!("tx-move-mark"), |e| {
            let v = e.view_mut(plan).ok_or_else(|| EngineError::not_found("view"))?;
            if new == DVec2::ZERO {
                v.mark_offsets.remove(&view);
            } else {
                v.mark_offsets.insert(view, new);
            }
            Ok(())
        })?;
        Ok(json!({ "plan": plan.0, "view": view.0, "offset": [new.x, new.y] }))
    }
}
