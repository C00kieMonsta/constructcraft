//! Editing a component's definition piece by piece: parameters, reference
//! planes and lines, and dimensions between planes that parameters drive
//! (see `cc_core::component_ref`). Every command names a component type;
//! the change goes to every type of that component in one undo step, each
//! type keeping its own parameter values (a value set here changes only the
//! type named).

use super::edit;
use super::types::validate;
use crate::error::{EngineError, Result};
use crate::refs::{self, Ref};
use crate::registry::{Command, CommandKind};
use crate::session::Session;
use crate::t;
use cc_core::component::SketchPlane;
use cc_core::component_ref::{self, PlaneAxis, RefDimension, RefLine, RefPlane};
use cc_core::{ComponentDef, ComponentParam, ElementTypeId, ParamKind, TypeData, expr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// The component of type `r`: its name, the type's definition and the
/// other types of the same component.
fn component_of(s: &Session, r: &Ref) -> Result<(ElementTypeId, String, ComponentDef, Vec<ElementTypeId>)> {
    let p = s.project();
    let id = refs::element_type(p, r)?;
    let Some(TypeData::Component { component, def }) = p.element_type(id).map(|t| &t.data) else {
        return Err(EngineError::invalid("not a component type"));
    };
    let siblings = p
        .types
        .values()
        .filter(|t| t.id != id && matches!(&t.data, TypeData::Component { component: c, .. } if c == component))
        .map(|t| t.id)
        .collect();
    Ok((id, component.clone(), def.clone(), siblings))
}

/// Change the definition of the component of type `r` with `change`, check
/// it and write it to every type of the component in one undo step. The
/// type named takes the new parameter values; the others keep theirs
/// (following `renames`, `[old, new]`), and take new parameters' values.
pub(crate) fn edit_def<T>(
    s: &mut Session,
    r: &Ref,
    renames: &[(String, String)],
    change: impl FnOnce(&mut ComponentDef) -> Result<T>,
) -> Result<T> {
    let (id, _, mut def, siblings) = component_of(s, r)?;
    let out = change(&mut def)?;
    validate(&def)?;
    let mut types = siblings.clone();
    types.push(id);
    edit(s, t!("tx-modify-type"), |e| {
        super::sheets::rename_sheet_fields(e, &types, renames);
        for sid in &siblings {
            let Some(TypeData::Component { def: theirs, .. }) = e.type_mut(*sid).map(|t| &mut t.data) else {
                continue;
            };
            let mut d = def.clone();
            for param in &mut d.params {
                let old = renames
                    .iter()
                    .rev()
                    .fold(param.name.clone(), |n, (from, to)| if n == *to { from.clone() } else { n });
                if let Some(q) = theirs.param(&old).filter(|q| q.kind == param.kind) {
                    param.value = q.value;
                    if param.accepts_text(&q.text) {
                        param.text.clone_from(&q.text);
                    }
                }
            }
            *theirs = d;
        }
        if let Some(TypeData::Component { def: mine, .. }) = e.type_mut(id).map(|t| &mut t.data) {
            *mine = def;
        }
        Ok(())
    })?;
    Ok(out)
}

/// A new name for a parameter, plane or line: valid, not reserved, unused.
fn check_name(def: &ComponentDef, name: &str) -> Result<String> {
    let name = name.trim();
    if !expr::is_valid_name(name) {
        return Err(EngineError::invalid(format!("'{name}' is not a valid name (letters, digits and _)")));
    }
    if def.name_taken(name) {
        return Err(EngineError::invalid(format!("the name '{name}' is already used")));
    }
    Ok(name.to_string())
}

/// The first free name `prefix1`, `prefix2`…
fn free_name(def: &ComponentDef, prefix: &str) -> String {
    (1..).map(|i| format!("{prefix}{i}")).find(|n| !def.name_taken(n)).unwrap_or_else(|| prefix.to_string())
}

/// An expression that must evaluate: a number, or names of the component.
pub(crate) fn check_expr(text: &str) -> Result<String> {
    let text = text.trim();
    if text.is_empty() {
        return Err(EngineError::invalid("an empty expression"));
    }
    Ok(text.to_string())
}

/// A number written so the expression parser reads it back exactly.
pub(crate) fn number(v: f64) -> String {
    let v = if v.abs() < 1e-12 { 0.0 } else { v };
    format!("{v}")
}

fn plane_axis(name: &str) -> Result<PlaneAxis> {
    match name.to_ascii_lowercase().as_str() {
        "x" => Ok(PlaneAxis::X),
        "y" => Ok(PlaneAxis::Y),
        "z" => Ok(PlaneAxis::Z),
        other => Err(EngineError::invalid(format!("unknown axis '{other}' (x, y or z)"))),
    }
}

pub(crate) fn sketch_plane(name: &str) -> Result<SketchPlane> {
    match name.to_ascii_lowercase().as_str() {
        "plan" => Ok(SketchPlane::Plan),
        "front" => Ok(SketchPlane::Front),
        "side" => Ok(SketchPlane::Side),
        other => Err(EngineError::invalid(format!("unknown view '{other}' (plan, front or side)"))),
    }
}

fn param_kind(name: &str) -> Result<ParamKind> {
    match name.to_ascii_lowercase().as_str() {
        "length" => Ok(ParamKind::Length),
        "angle" => Ok(ParamKind::Angle),
        "number" => Ok(ParamKind::Number),
        other => Err(EngineError::invalid(format!("unknown parameter kind '{other}' (length, angle or number)"))),
    }
}

/// The component of a type, as the modeling environment shows it: the
/// definition, the value of every parameter and reference plane, the value
/// of each dimension, where each connector is, the problems found, and the
/// component's types with their parameter values. A title block's pictures
/// show as `format`, `width`, `height` (pixels) and `bytes`, not their bytes.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ComponentGet {
    /// A type of the component: id or name.
    #[serde(rename = "type")]
    pub type_ref: Ref,
}

impl Command for ComponentGet {
    const NAME: &'static str = "component_get";
    const KIND: CommandKind = CommandKind::Query;
    fn run(self, s: &mut Session) -> Result<Value> {
        let (id, component, def, siblings) = component_of(s, &self.type_ref)?;
        let scope = def.scope(&BTreeMap::new(), &[(cc_core::component::WALL_THICKNESS, 0.15)]);
        let planes: Vec<Value> = def
            .all_ref_planes()
            .iter()
            .map(|p| json!({ "name": p.name, "axis": p.axis, "at": p.at, "value": scope.get(&p.name) }))
            .collect();
        let dimensions: Vec<Value> = (0..def.dimensions.len())
            .map(|i| json!({ "index": i, "dimension": def.dimensions.get(i), "value": def.dimension_value(&scope, i).ok() }))
            .collect();
        let connectors: Vec<Value> = def
            .resolve_connectors(&BTreeMap::new())
            .iter()
            .map(|k| {
                json!({
                    "index": k.index,
                    "name": k.name,
                    "system": k.system.name(),
                    "point": [k.point.x, k.point.y, k.point.z],
                    "direction": super::component_connector::direction_name(k.direction),
                    "diameter": k.diameter,
                })
            })
            .collect();
        let p = s.project();
        let types: Vec<Value> = std::iter::once(id)
            .chain(siblings)
            .filter_map(|tid| {
                let t = p.element_type(tid)?;
                let TypeData::Component { def, .. } = &t.data else { return None };
                let values: BTreeMap<&str, f64> = def.params.iter().map(|q| (q.name.as_str(), q.value)).collect();
                Some(json!({ "id": tid.0, "name": t.name, "values": values }))
            })
            .collect();
        let (_, piece_errors) =
            cc_geom::resolve_pieces(&def, &BTreeMap::new(), &[(cc_core::component::WALL_THICKNESS, 0.15)]);
        let mut problems = def.ref_problems();
        problems.extend(
            piece_errors.iter().map(|(i, e)| {
                format!("piece '{}': {e}", def.items.get(*i).map_or(String::new(), |it| it.name.clone()))
            }),
        );
        Ok(json!({
            "type": id.0,
            "component": component,
            "def": super::sheets::def_json(&def),
            "planes": planes,
            "dimensions": dimensions,
            "connectors": connectors,
            "problems": problems,
            "types": types,
        }))
    }
}

/// Add a parameter to a component (to every type, with the same value).
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ComponentParamAdd {
    /// A type of the component: id or name.
    #[serde(rename = "type")]
    pub type_ref: Ref,
    /// Name used in expressions (letters, digits and `_`, not starting with a digit).
    pub name: String,
    /// Value (meters for lengths, degrees for angles; default 0).
    #[serde(default)]
    pub value: Option<f64>,
    /// `length` (default), `angle`, `number` or `text` (a title block's
    /// per-sheet field, with `instance`).
    #[serde(default)]
    pub kind: Option<String>,
    /// Elements may override the value (sheets, for a title block's text parameter).
    #[serde(default)]
    pub instance: Option<bool>,
    /// Value of a `text` parameter (default empty).
    #[serde(default)]
    pub text: Option<String>,
}

impl Command for ComponentParamAdd {
    const NAME: &'static str = "component_param_add";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let value = refs::finite(self.value.unwrap_or(0.0), "value")?;
        let kind = match self.kind.as_deref() {
            Some(k) if k.trim().eq_ignore_ascii_case("text") => ParamKind::Text,
            Some(k) => param_kind(k)?,
            None => ParamKind::default(),
        };
        let text = self.text.unwrap_or_default();
        if !text.is_empty() && kind != ParamKind::Text {
            return Err(EngineError::invalid("only a text parameter takes a text"));
        }
        if text.chars().count() > cc_core::title_block::MAX_SHEET_TEXT {
            return Err(EngineError::invalid("the text is too long"));
        }
        let name = edit_def(s, &self.type_ref, &[], |def| {
            let name = check_name(def, &self.name)?;
            let instance = self.instance.unwrap_or(false);
            def.params.push(ComponentParam { name: name.clone(), value, kind, instance, text, ..Default::default() });
            Ok(name)
        })?;
        Ok(json!({ "name": name }))
    }
}

/// Change a component parameter: its value (for the type named only), kind,
/// whether elements may override it, or its name (every expression,
/// dimension and type follows a rename).
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ComponentParamSet {
    /// A type of the component: id or name (the value changes for this type).
    #[serde(rename = "type")]
    pub type_ref: Ref,
    /// The parameter's current name.
    pub name: String,
    /// New value for the type named.
    #[serde(default)]
    pub value: Option<f64>,
    /// `length`, `angle` or `number`.
    #[serde(default)]
    pub kind: Option<String>,
    /// Elements may override the value.
    #[serde(default)]
    pub instance: Option<bool>,
    /// New name.
    #[serde(default)]
    pub rename: Option<String>,
}

impl Command for ComponentParamSet {
    const NAME: &'static str = "component_param_set";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let value = self.value.map(|v| refs::finite(v, "value")).transpose()?;
        let kind = self.kind.as_deref().map(param_kind).transpose()?;
        let (_, _, def, _) = component_of(s, &self.type_ref)?;
        let Some(param) = def.param(&self.name) else {
            return Err(EngineError::not_found(format!("no parameter '{}'", self.name)));
        };
        if param.from_script && (self.kind.is_some() || self.instance.is_some() || self.rename.is_some()) {
            return Err(EngineError::rejected(format!(
                "'{}' is declared by the component's script: change it there",
                self.name
            )));
        }
        let rename = match &self.rename {
            Some(new) if new.trim() != self.name => Some(check_name(&def, new)?),
            _ => None,
        };
        let renames: Vec<(String, String)> = rename.iter().map(|n| (self.name.clone(), n.clone())).collect();
        edit_def(s, &self.type_ref, &renames, |def| {
            let Some(p) = def.params.iter_mut().find(|p| p.name == self.name) else {
                return Err(EngineError::not_found(format!("no parameter '{}'", self.name)));
            };
            if let Some(v) = value {
                p.value = v;
            }
            if let Some(k) = kind {
                p.kind = k;
            }
            if let Some(i) = self.instance {
                p.instance = i;
            }
            if let Some(new) = &rename {
                def.rename(&self.name, new);
            }
            Ok(())
        })?;
        Ok(json!({ "name": rename.unwrap_or(self.name) }))
    }
}

/// Remove a component parameter; refused while an expression reads it.
/// Dimensions it labels lose the label (their planes keep their place).
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ComponentParamDelete {
    /// A type of the component: id or name.
    #[serde(rename = "type")]
    pub type_ref: Ref,
    /// The parameter's name.
    pub name: String,
}

impl Command for ComponentParamDelete {
    const NAME: &'static str = "component_param_delete";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        edit_def(s, &self.type_ref, &[], |def| {
            match def.param(&self.name) {
                None => return Err(EngineError::not_found(format!("no parameter '{}'", self.name))),
                Some(p) if p.from_script => {
                    return Err(EngineError::rejected(format!(
                        "'{}' is declared by the component's script: remove it there",
                        self.name
                    )));
                }
                Some(_) => {}
            }
            // Planes driven by it keep their current place.
            let scope = def.scope(&BTreeMap::new(), &[]);
            for i in 0..def.dimensions.len() {
                if def.dimensions.get(i).and_then(|d| d.label.as_deref()) == Some(self.name.as_str()) {
                    unlabel(def, &scope, i);
                }
            }
            def.params.retain(|p| p.name != self.name);
            Ok(())
        })
        .map_err(|e| match e.code {
            crate::error::ErrorCode::InvalidArguments if e.message.contains(&self.name) => {
                EngineError::rejected(format!("the parameter '{}' is still used: {}", self.name, e.message))
            }
            _ => e,
        })?;
        Ok(json!({ "deleted": self.name }))
    }
}

/// Add a reference plane to a component: square to `axis` (`x`: a left/right
/// plane, `y`: front/back, `z`: a height), at the position `at`, an
/// expression that may read parameters and other planes (`Left + Width`).
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ComponentRefPlaneAdd {
    /// A type of the component: id or name.
    #[serde(rename = "type")]
    pub type_ref: Ref,
    /// `x`, `y` or `z`.
    pub axis: String,
    /// Position along the axis (meters): a number or an expression.
    pub at: String,
    /// Name (default `Plane1`, `Plane2`…).
    #[serde(default)]
    pub name: Option<String>,
}

impl Command for ComponentRefPlaneAdd {
    const NAME: &'static str = "component_ref_plane_add";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let axis = plane_axis(&self.axis)?;
        let at = check_expr(&self.at)?;
        let name = edit_def(s, &self.type_ref, &[], |def| {
            let name = match &self.name {
                Some(n) => check_name(def, n)?,
                None => free_name(def, "Plane"),
            };
            def.ref_planes.push(RefPlane { name: name.clone(), axis, at });
            Ok(name)
        })?;
        Ok(json!({ "name": name }))
    }
}

/// Move or rename a reference plane: `at` is its new position (an
/// expression); a rename follows into every expression and dimension.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ComponentRefPlaneSet {
    /// A type of the component: id or name.
    #[serde(rename = "type")]
    pub type_ref: Ref,
    /// The plane's current name.
    pub name: String,
    /// New position: a number or an expression.
    #[serde(default)]
    pub at: Option<String>,
    /// New name.
    #[serde(default)]
    pub rename: Option<String>,
}

impl Command for ComponentRefPlaneSet {
    const NAME: &'static str = "component_ref_plane_set";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let at = self.at.as_deref().map(check_expr).transpose()?;
        let name = edit_def(s, &self.type_ref, &[], |def| {
            if component_ref::ORIGIN_PLANES.iter().any(|(n, _)| *n == self.name) {
                return Err(EngineError::rejected(t!("msg-comp-origin-plane")));
            }
            let Some(plane) = def.ref_planes.iter_mut().find(|p| p.name == self.name) else {
                return Err(EngineError::not_found(format!("no reference plane '{}'", self.name)));
            };
            if let Some(at) = at {
                plane.at = at;
            }
            match &self.rename {
                Some(new) if new.trim() != self.name => {
                    let new = check_name(def, new)?;
                    def.rename(&self.name, &new);
                    Ok(new)
                }
                _ => Ok(self.name.clone()),
            }
        })?;
        Ok(json!({ "name": name }))
    }
}

/// Remove a reference plane and the dimensions on it; refused while an
/// expression reads it.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ComponentRefPlaneDelete {
    /// A type of the component: id or name.
    #[serde(rename = "type")]
    pub type_ref: Ref,
    /// The plane's name.
    pub name: String,
}

impl Command for ComponentRefPlaneDelete {
    const NAME: &'static str = "component_ref_plane_delete";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        edit_def(s, &self.type_ref, &[], |def| {
            if component_ref::ORIGIN_PLANES.iter().any(|(n, _)| *n == self.name) {
                return Err(EngineError::rejected(t!("msg-comp-origin-plane")));
            }
            if !def.ref_planes.iter().any(|p| p.name == self.name) {
                return Err(EngineError::not_found(format!("no reference plane '{}'", self.name)));
            }
            def.ref_planes.retain(|p| p.name != self.name);
            def.dimensions.retain(|d| d.from != self.name && d.to != self.name);
            def.equals.retain(|q| ![&q.a, &q.middle, &q.b].contains(&&self.name));
            Ok(())
        })
        .map_err(|e| match e.code {
            crate::error::ErrorCode::InvalidArguments => {
                EngineError::rejected(format!("the plane '{}' is still used: {}", self.name, e.message))
            }
            _ => e,
        })?;
        Ok(json!({ "deleted": self.name }))
    }
}

/// Add a reference line on one of the sketch planes (`plan`, `front` or
/// `side`, standing at `offset` along its normal axis): its ends are
/// `[u, v]` pairs of expressions in that plane.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ComponentRefLineAdd {
    /// A type of the component: id or name.
    #[serde(rename = "type")]
    pub type_ref: Ref,
    /// `plan`, `front` or `side`.
    pub plane: String,
    /// First end `[u, v]`: numbers or expressions.
    pub start: [String; 2],
    /// Second end `[u, v]`.
    pub end: [String; 2],
    /// Position of the sketch plane along its normal axis (default 0).
    #[serde(default)]
    pub offset: Option<String>,
    /// Name (default `Line1`, `Line2`…).
    #[serde(default)]
    pub name: Option<String>,
}

impl Command for ComponentRefLineAdd {
    const NAME: &'static str = "component_ref_line_add";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let plane = sketch_plane(&self.plane)?;
        let [su, sv] = self.start.clone().map(|e| check_expr(&e));
        let [eu, ev] = self.end.clone().map(|e| check_expr(&e));
        let (start, end) = ((su?, sv?), (eu?, ev?));
        let offset = check_expr(self.offset.as_deref().unwrap_or("0"))?;
        let name = edit_def(s, &self.type_ref, &[], |def| {
            let name = match &self.name {
                Some(n) => check_name(def, n)?,
                None => free_name(def, "Line"),
            };
            let line = RefLine { name: name.clone(), plane, offset, start, end };
            let scope = def.scope(&BTreeMap::new(), &[]);
            let ev = |e: &str| expr::eval(e, &|n: &str| scope.get(n));
            let ends = [&line.start.0, &line.start.1, &line.end.0, &line.end.1, &line.offset].map(|e| ev(e));
            if let Some(Err(e)) = ends.iter().find(|r| r.is_err()) {
                return Err(EngineError::invalid(format!("line '{name}': {e}")));
            }
            def.ref_lines.push(line);
            Ok(name)
        })?;
        Ok(json!({ "name": name }))
    }
}

/// Remove a reference line.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ComponentRefLineDelete {
    /// A type of the component: id or name.
    #[serde(rename = "type")]
    pub type_ref: Ref,
    /// The line's name.
    pub name: String,
}

impl Command for ComponentRefLineDelete {
    const NAME: &'static str = "component_ref_line_delete";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        edit_def(s, &self.type_ref, &[], |def| {
            if !def.ref_lines.iter().any(|l| l.name == self.name) {
                return Err(EngineError::not_found(format!("no reference line '{}'", self.name)));
            }
            def.ref_lines.retain(|l| l.name != self.name);
            Ok(())
        })?;
        Ok(json!({ "deleted": self.name }))
    }
}

/// Take a dimension's label or lock off: its moving plane stays where it is.
fn unlabel(def: &mut ComponentDef, scope: &component_ref::Scope, index: usize) {
    let Some(d) = def.dimensions.get_mut(index) else { return };
    d.label = None;
    d.locked = false;
    let to = d.to.clone();
    if let (Some(plane), Some(v)) = (def.ref_planes.iter_mut().find(|p| p.name == to), scope.get(&to)) {
        plane.at = number(v);
    }
}

/// Label dimension `index` with parameter `label`: its second plane (or the
/// first, when the second is an origin plane) moves to the other plus or
/// minus the parameter. A missing parameter is created with the current
/// distance.
fn label(def: &mut ComponentDef, index: usize, label: &str) -> Result<()> {
    let scope = def.scope(&BTreeMap::new(), &[]);
    let distance = def.dimension_value(&scope, index).map_err(|e| EngineError::invalid(e.to_string()))?;
    let (from, to) = moving_last(def, index)?;
    if let Some(d) = def.dimensions.get_mut(index) {
        d.label = Some(label.to_string());
        d.locked = false;
    }
    if def.param(label).is_none() {
        let name = check_name(def, label)?;
        def.params.push(ComponentParam {
            name,
            value: distance,
            kind: ParamKind::Length,
            instance: false,
            ..Default::default()
        });
    }
    let below = scope.get(&to).zip(scope.get(&from)).is_some_and(|(t, f)| t < f);
    let Some(plane) = def.ref_planes.iter_mut().find(|p| p.name == to) else {
        return Err(EngineError::not_found(format!("no reference plane '{to}'")));
    };
    plane.at = if below { format!("{from} - {label}") } else { format!("{from} + {label}") };
    Ok(())
}

/// Put the plane a dimension moves last (`to`): the other one when `to` is
/// an origin plane. Returns `(from, to)`; refused when both are origins.
fn moving_last(def: &mut ComponentDef, index: usize) -> Result<(String, String)> {
    let Some(d) = def.dimensions.get_mut(index) else { return Err(EngineError::not_found("dimension")) };
    let origin = |n: &str| component_ref::ORIGIN_PLANES.iter().any(|(o, _)| *o == n);
    if origin(&d.to) {
        if origin(&d.from) {
            return Err(EngineError::rejected(t!("msg-comp-origin-plane")));
        }
        std::mem::swap(&mut d.from, &mut d.to);
    }
    Ok((d.from.clone(), d.to.clone()))
}

/// Lock dimension `index` at `distance` (its current one when `None`): its
/// moving plane is written as the other plus or minus it, so it follows the
/// other. A labeled dimension is already held by its parameter; a moving
/// plane another expression sets is refused (locking would undo it).
fn lock(def: &mut ComponentDef, index: usize, distance: Option<f64>) -> Result<()> {
    let scope = def.scope(&BTreeMap::new(), &[]);
    let current = def.dimension_value(&scope, index).map_err(|e| EngineError::invalid(e.to_string()))?;
    if def.dimensions.get(index).is_some_and(|d| d.label.is_some()) {
        return Ok(());
    }
    let was_locked = def.dimensions.get(index).is_some_and(|d| d.locked);
    let (from, to) = moving_last(def, index)?;
    let Some(plane) = def.ref_planes.iter_mut().find(|p| p.name == to) else {
        return Err(EngineError::not_found(format!("no reference plane '{to}'")));
    };
    if !was_locked && expr::eval(&plane.at, &|_: &str| None).is_err() {
        return Err(EngineError::rejected(t!("msg-comp-plane-driven", "name" => to.clone())));
    }
    let below = scope.get(&to).zip(scope.get(&from)).is_some_and(|(t, f)| t < f);
    let distance = number(distance.unwrap_or(current));
    plane.at = if below { format!("{from} - {distance}") } else { format!("{from} + {distance}") };
    if let Some(d) = def.dimensions.get_mut(index) {
        d.locked = true;
    }
    Ok(())
}

/// Dimension two parallel reference planes in a view (`plan`, `front` or
/// `side`, one that sees them edge-on). With `label`, a length parameter
/// drives it: the second plane moves to the first plus the parameter (the
/// parameter is created with the current distance when missing). A
/// `locked` one keeps its distance: the second plane follows the first.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ComponentDimensionAdd {
    /// A type of the component: id or name.
    #[serde(rename = "type")]
    pub type_ref: Ref,
    /// `plan`, `front` or `side`.
    pub view: String,
    /// The plane it measures from.
    pub from: String,
    /// The plane it measures to (the one a label moves).
    pub to: String,
    /// Parameter that drives the distance.
    #[serde(default)]
    pub label: Option<String>,
    /// Where the dimension line sits across the planes (meters, default 0).
    #[serde(default)]
    pub place: Option<f64>,
    /// Keep the distance: the second plane follows the first.
    #[serde(default)]
    pub locked: bool,
}

impl Command for ComponentDimensionAdd {
    const NAME: &'static str = "component_dimension_add";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let view = sketch_plane(&self.view)?;
        let place = refs::finite(self.place.unwrap_or(0.0), "place")?;
        let index = edit_def(s, &self.type_ref, &[], |def| {
            let d =
                RefDimension { view, from: self.from.clone(), to: self.to.clone(), label: None, place, locked: false };
            def.dimensions.push(d);
            let index = def.dimensions.len() - 1;
            let scope = def.scope(&BTreeMap::new(), &[]);
            def.dimension_value(&scope, index).map_err(|e| EngineError::invalid(e.to_string()))?;
            if let Some(l) = &self.label {
                label(def, index, l.trim())?;
            } else if self.locked {
                lock(def, index, None)?;
            }
            Ok(index)
        })?;
        Ok(json!({ "index": index }))
    }
}

/// Change a dimension between reference planes: label it with a parameter
/// (`label`, created when missing), take its label off (`label: ""`, the
/// planes stay), lock or unlock it (`locked`: a locked dimension keeps its
/// distance, its moving plane following the other), move its line
/// (`place`) or give it a new `value` (the label's parameter for this type,
/// or the moving plane's position).
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ComponentDimensionSet {
    /// A type of the component: id or name.
    #[serde(rename = "type")]
    pub type_ref: Ref,
    /// The dimension's index (from `component_get`).
    pub index: usize,
    /// Parameter that drives it; empty to take the label off.
    #[serde(default)]
    pub label: Option<String>,
    /// New distance (meters).
    #[serde(default)]
    pub value: Option<f64>,
    /// Where the dimension line sits across the planes.
    #[serde(default)]
    pub place: Option<f64>,
    /// Lock (`true`, at its distance) or unlock (`false`, the planes stay).
    #[serde(default)]
    pub locked: Option<bool>,
}

impl Command for ComponentDimensionSet {
    const NAME: &'static str = "component_dimension_set";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let value = self.value.map(|v| refs::finite(v, "value")).transpose()?;
        if value.is_some_and(|v| v <= 0.0) {
            return Err(EngineError::invalid(t!("msg-invalid-value")));
        }
        let place = self.place.map(|v| refs::finite(v, "place")).transpose()?;
        edit_def(s, &self.type_ref, &[], |def| {
            if def.dimensions.get(self.index).is_none() {
                return Err(EngineError::not_found(format!("no dimension {}", self.index)));
            }
            match self.label.as_deref().map(str::trim) {
                Some("") => {
                    let scope = def.scope(&BTreeMap::new(), &[]);
                    unlabel(def, &scope, self.index);
                }
                Some(l) => label(def, self.index, l)?,
                None => {}
            }
            match self.locked {
                Some(true) => lock(def, self.index, None)?,
                Some(false) if def.dimensions.get(self.index).is_some_and(|d| d.locked) => {
                    let scope = def.scope(&BTreeMap::new(), &[]);
                    unlabel(def, &scope, self.index);
                }
                _ => {}
            }
            if let Some(v) = value {
                set_distance(def, self.index, v)?;
            }
            if let (Some(p), Some(d)) = (place, def.dimensions.get_mut(self.index)) {
                d.place = p;
            }
            Ok(())
        })?;
        Ok(json!({ "index": self.index }))
    }
}

/// Make dimension `index` measure `value`: its label's parameter takes it,
/// a locked one keeps the new distance, or else its moving plane goes there
/// (refused when that plane's position is an expression other than a number).
fn set_distance(def: &mut ComponentDef, index: usize, value: f64) -> Result<()> {
    let Some(d) = def.dimensions.get(index).cloned() else { return Err(EngineError::not_found("dimension")) };
    if let Some(label) = &d.label {
        if let Some(p) = def.params.iter_mut().find(|p| p.name == *label) {
            p.value = value;
        }
        return Ok(());
    }
    if d.locked {
        return lock(def, index, Some(value));
    }
    let scope = def.scope(&BTreeMap::new(), &[]);
    let origin = |n: &str| component_ref::ORIGIN_PLANES.iter().any(|(o, _)| *o == n);
    let (fixed, moving) = if origin(&d.to) { (&d.to, &d.from) } else { (&d.from, &d.to) };
    let (Some(f), Some(m)) = (scope.get(fixed), scope.get(moving)) else {
        return Err(EngineError::invalid(t!("msg-invalid-value")));
    };
    let Some(plane) = def.ref_planes.iter_mut().find(|p| p.name == *moving) else {
        return Err(EngineError::rejected(t!("msg-comp-origin-plane")));
    };
    if expr::eval(&plane.at, &|_: &str| None).is_err() {
        return Err(EngineError::rejected(t!("msg-comp-plane-driven", "name" => moving.clone())));
    }
    plane.at = number(if m < f { f - value } else { f + value });
    Ok(())
}

/// Remove a dimension between reference planes (the planes stay; a locked
/// one's moving plane no longer follows).
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ComponentDimensionDelete {
    /// A type of the component: id or name.
    #[serde(rename = "type")]
    pub type_ref: Ref,
    /// The dimension's index.
    pub index: usize,
}

impl Command for ComponentDimensionDelete {
    const NAME: &'static str = "component_dimension_delete";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        edit_def(s, &self.type_ref, &[], |def| {
            if self.index >= def.dimensions.len() {
                return Err(EngineError::not_found(format!("no dimension {}", self.index)));
            }
            if def.dimensions.get(self.index).is_some_and(|d| d.locked) {
                let scope = def.scope(&BTreeMap::new(), &[]);
                unlabel(def, &scope, self.index);
            }
            def.dimensions.remove(self.index);
            Ok(())
        })?;
        Ok(json!({ "deleted": self.index }))
    }
}

/// The ways equality `index` can be made to hold, best first, as a plane and
/// its new position: the middle written `(a + b) / 2`; or, when the middle
/// cannot move (an origin plane, or one a labeled or locked dimension
/// moves), the outer plane a dimension between `a` and `b` measures from,
/// half that dimension from the middle (so `Width` stays centered); or an
/// outer plane mirroring the other about the middle.
fn equal_candidates(def: &ComponentDef, index: usize) -> Vec<(String, String)> {
    let Some(q) = def.equals.get(index) else { return Vec::new() };
    let origin = |n: &str| component_ref::ORIGIN_PLANES.iter().any(|(o, _)| *o == n);
    let driven = |n: &str| def.dimensions.iter().any(|d| d.to == n && (d.label.is_some() || d.locked));
    let movable = |n: &str| !origin(n) && !driven(n);
    let scope = def.scope(&BTreeMap::new(), &[]);
    let mut out = Vec::new();
    if movable(&q.middle) {
        out.push((q.middle.clone(), q.expression()));
    }
    let m = &q.middle;
    for d in &def.dimensions {
        let spans = (d.from == q.a && d.to == q.b) || (d.from == q.b && d.to == q.a);
        if !spans || !movable(&d.from) {
            continue;
        }
        let half = match (&d.label, d.locked) {
            (Some(label), _) => format!("{label} / 2"),
            (None, true) => {
                let (Some(f), Some(t)) = (scope.get(&d.from), scope.get(&d.to)) else { continue };
                number((t - f).abs() / 2.0)
            }
            (None, false) => continue,
        };
        // The from plane on its side of the middle, the to plane following it.
        let below = scope.get(&d.from).zip(scope.get(&d.to)).is_some_and(|(f, t)| f < t);
        out.push((d.from.clone(), if below { format!("{m} - {half}") } else { format!("{m} + {half}") }));
    }
    for (moving, other) in [(&q.b, &q.a), (&q.a, &q.b)] {
        if movable(moving) {
            out.push((moving.clone(), format!("2 * {m} - {other}")));
        }
    }
    out
}

/// Keep the plane `middle` halfway between the parallel planes `a` and `b`
/// (an equality, EQ): its position becomes `(a + b) / 2`, so it follows
/// whatever moves them. When the middle cannot move (an origin plane, or one
/// a labeled or locked dimension sets), an outer plane moves instead: with a
/// dimension between `a` and `b`, the plane it measures from goes half of it
/// from the middle (a `Width` centered on `CenterX`), else one outer plane
/// mirrors the other. Shown in `view` (`plan`, `front` or `side`) as two
/// equal dimensions.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ComponentEqualAdd {
    /// A type of the component: id or name.
    #[serde(rename = "type")]
    pub type_ref: Ref,
    /// `plan`, `front` or `side`.
    pub view: String,
    /// One outer plane.
    pub a: String,
    /// The plane kept halfway (not an origin plane).
    pub middle: String,
    /// The other outer plane.
    pub b: String,
    /// Where the dimensions' line sits across the planes (meters, default 0).
    #[serde(default)]
    pub place: Option<f64>,
}

impl Command for ComponentEqualAdd {
    const NAME: &'static str = "component_equal_add";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let view = sketch_plane(&self.view)?;
        let place = refs::finite(self.place.unwrap_or(0.0), "place")?;
        if self.a == self.b || self.a == self.middle || self.b == self.middle {
            return Err(EngineError::invalid("an equality needs three different planes"));
        }
        let index = edit_def(s, &self.type_ref, &[], |def| {
            for name in [&self.a, &self.middle, &self.b] {
                if def.ref_plane(name).is_none() {
                    return Err(EngineError::invalid(format!("no reference plane '{name}'")));
                }
            }
            let before = def.ref_problems().len();
            let q =
                cc_core::RefEqual { view, a: self.a.clone(), middle: self.middle.clone(), b: self.b.clone(), place };
            def.equals.push(q);
            let index = def.equals.len() - 1;
            // The first way that holds without new problems (a cycle, say).
            for (plane, at) in equal_candidates(def, index) {
                let mut tried = def.clone();
                if let Some(p) = tried.ref_planes.iter_mut().find(|p| p.name == plane) {
                    p.at = at;
                }
                if tried.equal_holds(index).is_ok() && tried.ref_problems().len() <= before {
                    *def = tried;
                    return Ok(index);
                }
            }
            Err(EngineError::rejected(t!("msg-comp-equal-fixed", "name" => self.middle.clone())))
        })?;
        Ok(json!({ "index": index }))
    }
}

/// Remove an equality (EQ); its middle plane stays halfway as written.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ComponentEqualDelete {
    /// A type of the component: id or name.
    #[serde(rename = "type")]
    pub type_ref: Ref,
    /// The equality's index (from `component_get`).
    pub index: usize,
}

impl Command for ComponentEqualDelete {
    const NAME: &'static str = "component_equal_delete";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        edit_def(s, &self.type_ref, &[], |def| {
            if self.index >= def.equals.len() {
                return Err(EngineError::not_found(format!("no equality {}", self.index)));
            }
            def.equals.remove(self.index);
            Ok(())
        })?;
        Ok(json!({ "deleted": self.index }))
    }
}
