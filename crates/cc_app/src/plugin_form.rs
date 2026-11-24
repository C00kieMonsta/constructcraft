//! The form of a plugin command: a dialog built from the JSON Schema of its
//! parameters (`input_schema`), in place of the palette's JSON arguments.
//! *Plugin Commands* and the palette's plugin entries open it ([`open`]);
//! Run goes through the session like the palette's commands, so errors, undo
//! and what the plugin prints behave the same. A command that waits for an
//! external program is left pending: the form keeps its state, hidden behind
//! the progress dialog of [`crate::plugin_job`], and shows the result when
//! the command ends ([`job_ended`]).
//!
//! What the form reads of each property of `input_schema.properties`:
//! `type` (`number`, `integer`, `boolean`, `string`, `array`, `object`; any
//! other, or none, is a JSON field), `title` (the label; else the name with
//! `_` as spaces), `description` (the label's tooltip), `default`, `enum`
//! (strings) with `enum_labels`, `minimum`/`maximum`, and the hint `ui`:
//! `length` (meters, shown and typed in the project's units), `element` and
//! `elements` (element ids, picked in the view or taken from the selection,
//! with an optional `category`), `point` (`[x, y]` in plan meters, picked in
//! a plan), `level`, `view` and `type` (ids; `type` with an optional
//! `category`) and `material` (a name), chosen from the project's, and
//! `save_path`/`open_path` (a file path typed or chosen with Browse… in the
//! app's save or open dialog, filtered by the property's `extensions`;
//! the plugin may write or read that file while the command runs). Required
//! fields come first, in the order `required` lists them; the others go
//! under *More options*, and are sent only once the user sets them, so the
//! command's own defaults apply.
//!
//! Picking hides the dialog and sets [`Tool::Pick`]: a click in the view
//! ([`pick_tool`] in 2D views, [`picked_element`] from the 3D view) fills
//! the field and brings the dialog back; Esc or another tool (anything that
//! leaves `Tool::Pick`) brings it back unchanged.

use crate::dialog::Dialog;
use crate::state::AppState;
use crate::t;
use crate::table;
use crate::theme::{self, PALETTE};
use crate::tools::Tool;
use cc_core::{Category, ElementId};
use cc_views::format;
use egui::{Align, Layout, RichText, Stroke, Ui};
use serde_json::{Map, Value, json};
use std::collections::BTreeSet;

/// Width of the dialog's body.
const WIDTH: f32 = 520.0;
/// A description's first line longer than this (characters) is not the title.
const TITLE_MAX: usize = 48;
/// Height the fields take before they scroll.
const FIELDS_HEIGHT: f32 = 420.0;

/// How lengths and numbers are written and read: the project's units and
/// the language's decimal symbol.
#[derive(Debug, Clone, Default)]
pub struct Fmt {
    pub units: cc_core::Units,
    pub comma: bool,
}

impl Fmt {
    /// The settings in force (`AppState::sync` sets the project's units each frame).
    pub fn current() -> Fmt {
        let (units, comma) = format::current();
        Fmt { units, comma }
    }
}

/// What a field holds, and so the widget that edits it.
#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    /// A number typed in a text field; with `length`, meters in the project's units.
    Number {
        integer: bool,
        length: bool,
    },
    Boolean,
    /// One string of `values`, shown as `labels`.
    Choice {
        values: Vec<String>,
        labels: Vec<String>,
    },
    Text,
    /// An element id.
    Element {
        category: Option<String>,
    },
    /// Element ids.
    Elements {
        category: Option<String>,
    },
    /// `[x, y]`, meters in plan.
    Point,
    /// A level id.
    Level,
    /// A view id.
    View,
    /// A type id.
    Type {
        category: Option<String>,
    },
    /// A material's name.
    Material,
    /// A file path, to write (`save`) or to read, of one of `extensions`
    /// (lowercase, without the dot; empty: any).
    Path {
        save: bool,
        extensions: Vec<String>,
    },
    /// Any JSON, typed; `want` is the schema's `object` or `array`, checked.
    Json {
        want: Option<String>,
    },
}

impl Kind {
    /// The category picks and selections keep to.
    pub fn category(&self) -> Option<&str> {
        match self {
            Kind::Element { category } | Kind::Elements { category } | Kind::Type { category } => category.as_deref(),
            _ => None,
        }
    }
}

/// Why a field cannot be sent.
#[derive(Debug, Clone, PartialEq)]
pub enum Problem {
    /// Required and not set.
    Missing,
    NotNumber,
    NotInteger,
    NotLength,
    Below(f64),
    Above(f64),
    /// Not valid JSON (the parser's message).
    Json(String),
    WantObject,
    WantArray,
}

impl Problem {
    /// What to tell the user about `field`.
    pub fn message(&self, field: &Field, fmt: &Fmt) -> String {
        let name = field.label.as_str();
        let amount = |v: f64| match field.kind {
            Kind::Number { length: true, .. } => format::length_unit_with(&fmt.units, fmt.comma, v),
            _ => plain_number(v, fmt.comma),
        };
        match self {
            Problem::Missing => t!("pform-missing", "field" => name),
            Problem::NotNumber => t!("pform-not-number", "field" => name),
            Problem::NotInteger => t!("pform-not-integer", "field" => name),
            Problem::NotLength => t!("pform-not-length", "field" => name),
            Problem::Below(min) => t!("pform-below", "field" => name, "min" => amount(*min)),
            Problem::Above(max) => t!("pform-above", "field" => name, "max" => amount(*max)),
            Problem::Json(error) => t!("pform-bad-json", "field" => name, "error" => error),
            Problem::WantObject => t!("pform-want-object", "field" => name),
            Problem::WantArray => t!("pform-want-array", "field" => name),
        }
    }
}

/// Why JSON arguments do not fit the form.
#[derive(Debug, Clone, PartialEq)]
pub enum Misfit {
    /// Not valid JSON (the parser's message).
    Json(String),
    NotObject,
    /// A property the form has no field for.
    Unknown(String),
    /// A value its field cannot hold (the field's label).
    Wrong(String),
}

impl Misfit {
    pub fn message(&self) -> String {
        match self {
            Misfit::Json(error) => t!("palette-bad-json", "error" => error),
            Misfit::NotObject => t!("pform-json-not-object"),
            Misfit::Unknown(name) => t!("pform-json-unknown", "name" => name),
            Misfit::Wrong(name) => t!("pform-json-wrong", "name" => name),
        }
    }
}

/// One parameter of the command.
#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub label: String,
    pub description: String,
    pub required: bool,
    pub kind: Kind,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    /// The schema's `default` (left out when its field cannot hold it).
    pub default: Option<Value>,
    /// Set by the user (typed, picked, chosen, cleared): an optional field
    /// is sent only then.
    pub touched: bool,
    /// The value: once touched, the user's (`None`: unset); before, the default.
    value: Option<Value>,
    /// The text of a typed field (a point's x and y).
    pub texts: [String; 2],
    /// Why the text typed is not a value.
    error: Option<Problem>,
    /// The choices of a level, view, type or material field: (value, label).
    pub options: Vec<(Value, String)>,
}

impl Field {
    fn new(name: &str, prop: &Value, required: bool, fmt: &Fmt) -> Field {
        let label = prop
            .get("title")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| humanize(name));
        let kind = kind_of(prop);
        let default = prop.get("default").filter(|d| fits(&kind, d)).cloned();
        let mut field = Field {
            name: name.to_string(),
            label,
            description: prop.get("description").and_then(Value::as_str).unwrap_or_default().trim().to_string(),
            required,
            minimum: prop.get("minimum").and_then(Value::as_f64),
            maximum: prop.get("maximum").and_then(Value::as_f64),
            kind,
            default,
            touched: false,
            value: None,
            texts: Default::default(),
            error: None,
            options: Vec::new(),
        };
        field.reset(fmt);
        field
    }

    /// Back to the default, untouched.
    pub fn reset(&mut self, fmt: &Fmt) {
        self.touched = false;
        self.error = None;
        self.value = self.default.clone();
        self.texts = self.default.as_ref().map(|d| texts_of(&self.kind, d, fmt)).unwrap_or_default();
    }

    /// Set the value (a pick, a choice, the selection, JSON).
    pub fn set_value(&mut self, value: Value, fmt: &Fmt) {
        self.texts = texts_of(&self.kind, &value, fmt);
        self.value = Some(value);
        self.touched = true;
        self.error = None;
    }

    /// Unset it: an optional field is then not sent.
    pub fn unset(&mut self) {
        self.value = None;
        self.texts = Default::default();
        self.touched = true;
        self.error = None;
    }

    /// The user typed `text` in the text `i` (a point's y is 1).
    pub fn set_text(&mut self, i: usize, text: String, fmt: &Fmt) {
        if let Some(t) = self.texts.get_mut(i) {
            *t = text;
        }
        self.touched = true;
        match parse(&self.kind, &self.texts, fmt) {
            Ok(value) => {
                self.value = value;
                self.error = None;
            }
            Err(problem) => {
                self.value = None;
                self.error = Some(problem);
            }
        }
    }

    /// The value shown: the user's, or else the default.
    pub fn shown(&self) -> Option<&Value> {
        self.value.as_ref()
    }

    /// The value sent: the user's; untouched, a required field's default.
    pub fn effective(&self) -> Option<&Value> {
        if self.touched || self.required { self.value.as_ref() } else { None }
    }

    /// Why the field cannot be sent.
    pub fn problem(&self) -> Option<Problem> {
        if let Some(e) = &self.error {
            return Some(e.clone());
        }
        match self.effective() {
            None => self.required.then_some(Problem::Missing),
            Some(Value::Array(ids))
                if self.required && ids.is_empty() && matches!(self.kind, Kind::Elements { .. }) =>
            {
                Some(Problem::Missing)
            }
            Some(v) => self.out_of_range(v),
        }
    }

    /// A problem other than being empty: what makes it show as invalid.
    pub fn invalid(&self) -> Option<Problem> {
        self.problem().filter(|p| *p != Problem::Missing)
    }

    fn out_of_range(&self, v: &Value) -> Option<Problem> {
        if !matches!(self.kind, Kind::Number { .. }) {
            return None;
        }
        let x = v.as_f64()?;
        if let Some(min) = self.minimum
            && x < min
        {
            return Some(Problem::Below(min));
        }
        if let Some(max) = self.maximum
            && x > max
        {
            return Some(Problem::Above(max));
        }
        None
    }
}

/// `rectangle_room` → "Rectangle room".
fn humanize(name: &str) -> String {
    let spaced = name.replace('_', " ");
    let mut chars = spaced.trim().chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// The schema's type: a name, or the first not null of a list.
fn schema_type(prop: &Value) -> Option<&str> {
    match prop.get("type") {
        Some(Value::String(t)) => Some(t.as_str()),
        Some(Value::Array(types)) => types.iter().filter_map(Value::as_str).find(|t| *t != "null"),
        _ => None,
    }
}

/// The field kind of a property schema.
pub fn kind_of(prop: &Value) -> Kind {
    let ty = schema_type(prop);
    let category = || prop.get("category").and_then(Value::as_str).map(str::to_string);
    match prop.get("ui").and_then(Value::as_str) {
        Some("length") if matches!(ty, None | Some("number" | "integer")) => {
            return Kind::Number { integer: ty == Some("integer"), length: true };
        }
        Some("element") => return Kind::Element { category: category() },
        Some("elements") => return Kind::Elements { category: category() },
        Some("point") => return Kind::Point,
        Some("level") => return Kind::Level,
        Some("view") => return Kind::View,
        Some("type") => return Kind::Type { category: category() },
        Some("material") => return Kind::Material,
        Some(ui @ ("save_path" | "open_path")) => {
            let extensions = prop
                .get("extensions")
                .and_then(Value::as_array)
                .map(|list| list.iter().filter_map(Value::as_str).map(str::to_lowercase).collect())
                .unwrap_or_default();
            return Kind::Path { save: ui == "save_path", extensions };
        }
        _ => {}
    }
    match ty {
        Some("number") => Kind::Number { integer: false, length: false },
        Some("integer") => Kind::Number { integer: true, length: false },
        Some("boolean") => Kind::Boolean,
        Some("string") => {
            let values: Option<Vec<String>> = prop
                .get("enum")
                .and_then(Value::as_array)
                .and_then(|list| list.iter().map(|v| v.as_str().map(str::to_string)).collect());
            match values {
                Some(values) if !values.is_empty() => {
                    let labels: Option<Vec<String>> = prop
                        .get("enum_labels")
                        .and_then(Value::as_array)
                        .and_then(|list| list.iter().map(|v| v.as_str().map(str::to_string)).collect())
                        .filter(|labels: &Vec<String>| labels.len() == values.len());
                    Kind::Choice { labels: labels.unwrap_or_else(|| values.clone()), values }
                }
                _ => Kind::Text,
            }
        }
        other => Kind::Json { want: other.filter(|t| matches!(*t, "object" | "array")).map(str::to_string) },
    }
}

/// The fields of a command's parameters schema: the required ones first,
/// in the order `required` lists them, then the others.
pub fn fields(schema: &Value, fmt: &Fmt) -> Vec<Field> {
    let Some(props) = schema.get("properties").and_then(Value::as_object) else { return Vec::new() };
    let required: Vec<&str> = schema
        .get("required")
        .and_then(Value::as_array)
        .map(|r| r.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let mut out: Vec<Field> = Vec::new();
    for name in &required {
        if let Some(prop) = props.get(*name)
            && !out.iter().any(|f| f.name == *name)
        {
            out.push(Field::new(name, prop, true, fmt));
        }
    }
    for (name, prop) in props {
        if !required.contains(&name.as_str()) {
            out.push(Field::new(name, prop, false, fmt));
        }
    }
    out
}

/// Whether a field of `kind` can hold `v`.
pub fn fits(kind: &Kind, v: &Value) -> bool {
    match kind {
        Kind::Number { integer: true, .. } => v.is_i64() || v.is_u64() || v.as_f64().is_some_and(|x| x.fract() == 0.0),
        Kind::Number { .. } => v.is_number(),
        Kind::Boolean => v.is_boolean(),
        Kind::Choice { values, .. } => v.as_str().is_some_and(|s| values.iter().any(|x| x == s)),
        Kind::Text | Kind::Material | Kind::Path { .. } => v.is_string(),
        Kind::Element { .. } | Kind::Level | Kind::View | Kind::Type { .. } => v.is_u64(),
        Kind::Elements { .. } => v.as_array().is_some_and(|ids| ids.iter().all(Value::is_u64)),
        Kind::Point => v.as_array().is_some_and(|p| p.len() == 2 && p.iter().all(Value::is_number)),
        Kind::Json { want } => match want.as_deref() {
            Some("object") => v.is_object(),
            Some("array") => v.is_array(),
            _ => true,
        },
    }
}

/// A number as typed back: no grouping, the language's decimal symbol.
fn plain_number(v: f64, comma: bool) -> String {
    let text = format!("{v}");
    if comma { text.replace('.', ",") } else { text }
}

fn number_text(v: &Value, comma: bool) -> String {
    match (v.as_i64(), v.as_u64(), v.as_f64()) {
        (Some(i), _, _) => i.to_string(),
        (_, Some(u), _) => u.to_string(),
        (_, _, Some(f)) => plain_number(f, comma),
        _ => String::new(),
    }
}

/// The texts a typed field shows for `v`.
fn texts_of(kind: &Kind, v: &Value, fmt: &Fmt) -> [String; 2] {
    let length = |v: Option<&Value>| {
        v.and_then(Value::as_f64).map(|m| format::length_unit_with(&fmt.units, fmt.comma, m)).unwrap_or_default()
    };
    match kind {
        Kind::Number { length: true, .. } => [length(Some(v)), String::new()],
        Kind::Number { .. } => [number_text(v, fmt.comma), String::new()],
        Kind::Text | Kind::Path { .. } => [v.as_str().unwrap_or_default().to_string(), String::new()],
        Kind::Point => [length(v.get(0)), length(v.get(1))],
        Kind::Json { .. } => [serde_json::to_string_pretty(v).unwrap_or_default(), String::new()],
        _ => Default::default(),
    }
}

/// The value of a typed field's texts: `None` when empty.
fn parse(kind: &Kind, texts: &[String; 2], fmt: &Fmt) -> Result<Option<Value>, Problem> {
    let first = texts[0].trim();
    let length = |t: &str| format::parse_length_with(&fmt.units, t).ok_or(Problem::NotLength);
    match kind {
        Kind::Number { integer, length: is_length } => {
            if first.is_empty() {
                return Ok(None);
            }
            let v = if *is_length { length(first)? } else { format::parse_number(first).ok_or(Problem::NotNumber)? };
            if !*integer {
                return Ok(Some(json!(v)));
            }
            if v.fract() != 0.0 || v.abs() > 9.0e15 {
                return Err(Problem::NotInteger);
            }
            Ok(Some(json!(v as i64)))
        }
        Kind::Text => Ok((!texts[0].is_empty()).then(|| json!(texts[0]))),
        Kind::Path { .. } => Ok((!first.is_empty()).then(|| json!(first))),
        Kind::Point => {
            let second = texts[1].trim();
            if first.is_empty() && second.is_empty() {
                return Ok(None);
            }
            Ok(Some(json!([length(first)?, length(second)?])))
        }
        Kind::Json { want } => {
            if first.is_empty() {
                return Ok(None);
            }
            let v: Value = serde_json::from_str(first).map_err(|e| Problem::Json(e.to_string()))?;
            match want.as_deref() {
                Some("object") if !v.is_object() => Err(Problem::WantObject),
                Some("array") if !v.is_array() => Err(Problem::WantArray),
                _ => Ok(Some(v)),
            }
        }
        _ => Ok(None),
    }
}

/// The first field that cannot be sent, in the order shown.
pub fn first_problem(fields: &[Field]) -> Option<(&Field, Problem)> {
    fields.iter().find_map(|f| f.problem().map(|p| (f, p)))
}

/// The arguments: every field's value that is sent.
pub fn args(fields: &[Field]) -> Option<Value> {
    if first_problem(fields).is_some() {
        return None;
    }
    let map: Map<String, Value> =
        fields.iter().filter_map(|f| f.effective().map(|v| (f.name.clone(), v.clone()))).collect();
    Some(Value::Object(map))
}

/// The form as JSON arguments (fields that cannot be sent left out).
pub fn to_json_text(fields: &[Field]) -> String {
    let map: Map<String, Value> = fields
        .iter()
        .filter(|f| f.error.is_none())
        .filter_map(|f| f.effective().map(|v| (f.name.clone(), v.clone())))
        .collect();
    serde_json::to_string_pretty(&Value::Object(map)).unwrap_or_else(|_| "{}".to_string())
}

/// Fill the form from JSON arguments: properties left out are unset (an
/// optional field goes back to its default, not sent). Nothing changes
/// when they do not fit.
pub fn from_json(fields: &mut [Field], text: &str, fmt: &Fmt) -> Result<(), Misfit> {
    let value: Value = serde_json::from_str(text).map_err(|e| Misfit::Json(e.to_string()))?;
    let Value::Object(map) = value else { return Err(Misfit::NotObject) };
    for (name, v) in &map {
        let Some(field) = fields.iter().find(|f| &f.name == name) else { return Err(Misfit::Unknown(name.clone())) };
        if !fits(&field.kind, v) {
            return Err(Misfit::Wrong(field.label.clone()));
        }
    }
    for field in fields.iter_mut() {
        match map.get(&field.name) {
            Some(v) => field.set_value(v.clone(), fmt),
            None if field.required => field.unset(),
            None => field.reset(fmt),
        }
    }
    Ok(())
}

/// Why JSON arguments cannot run.
fn json_problem(text: &str) -> Option<String> {
    match serde_json::from_str::<Value>(text) {
        Err(e) => Some(t!("palette-bad-json", "error" => e)),
        Ok(Value::Object(_)) => None,
        Ok(_) => Some(t!("pform-json-not-object")),
    }
}

/// The title: the description's first line when it is short (a plugin's
/// `title` stands there when it gave no description), else the command's
/// name without the plugin's id, in words.
pub fn title_of(description: &str, command: &str) -> String {
    let first = description.lines().next().unwrap_or_default().trim();
    if !first.is_empty() && first.chars().count() <= TITLE_MAX {
        return first.to_string();
    }
    humanize(command.split_once(cc_engine::extension::SEPARATOR).map_or(command, |(_, short)| short))
}

/// The category a schema names (`wall`, `cable_tray`…), as the engine reads it.
pub fn parse_category(name: &str) -> Option<Category> {
    let wanted: String = name.trim().chars().filter(|c| *c != '_').flat_map(char::to_lowercase).collect();
    Category::NAMED.into_iter().find(|c| cc_engine::category_name(*c) == wanted)
}

/// A category as the interface names it (an unknown one as written).
fn category_label(name: &str) -> String {
    parse_category(name).map(crate::actions::t_category).unwrap_or_else(|| name.to_string())
}

/// Whether an element of `c` fits a field restricted to `category` (an
/// unknown category takes none).
fn accepts(category: Option<&str>, c: Category) -> bool {
    category.is_none_or(|name| parse_category(name) == Some(c))
}

/// The ids of the selected elements of `category`.
fn selected_of(state: &AppState, category: Option<&str>) -> Vec<u64> {
    let project = state.project();
    state
        .selection
        .iter()
        .filter(|id| project.element(**id).is_some_and(|e| accepts(category, e.category())))
        .map(|id| id.0)
        .collect()
}

/// `#12 Wall: Generic 200`, or a dash when unset.
fn element_text(project: &cc_core::Project, value: Option<&Value>) -> String {
    let Some(id) = value.and_then(Value::as_u64) else { return "—".to_string() };
    match project.element(ElementId(id)) {
        Some(e) => format!("#{id} {}: {}", crate::actions::t_category(e.category()), project.type_name(e.type_id)),
        None => t!("pform-element-missing", "id" => id),
    }
}

/// The plugin's name, from its id (the part of the command before `__`).
fn plugin_name(state: &AppState, command: &str) -> String {
    let id = command.split_once(cc_engine::extension::SEPARATOR).map_or(command, |(id, _)| id);
    state.plugins.listed.iter().find(|p| p.id == id).map(|p| p.name.clone()).unwrap_or_else(|| id.to_string())
}

/// What the dialog shows.
enum Mode {
    Form,
    /// The arguments as JSON; `message` says why they did not go back to the form.
    Json {
        text: String,
        message: Option<String>,
    },
    /// What the last run printed and returned (pretty JSON, `None` when
    /// empty); `json` holds the JSON it ran from, to go back to.
    Result {
        printed: Vec<String>,
        result: Option<String>,
        json: Option<String>,
    },
}

/// A field being picked in the view.
struct Picking {
    field: usize,
    point: bool,
    /// Why the last click was not taken.
    notice: Option<String>,
}

/// The open form of a plugin command.
pub struct PluginForm {
    command: String,
    title: String,
    /// The plugin's name, shown by the title.
    plugin: String,
    description: String,
    fields: Vec<Field>,
    mode: Mode,
    /// The last run's error (after what the command printed), over the footer.
    error: Option<String>,
    /// The field being picked; the dialog is hidden meanwhile.
    picking: Option<Picking>,
    /// The document revision the choices were read at.
    revision: Option<u64>,
    /// The command waits for an external program: what it printed so far.
    waiting: Option<Vec<String>>,
}

impl PluginForm {
    /// Whether its command waits for an external program.
    pub fn is_waiting(&self) -> bool {
        self.waiting.is_some()
    }

    fn prompt(&self) -> String {
        let Some(pick) = &self.picking else { return String::new() };
        let base = if pick.point {
            t!("pform-pick-point")
        } else {
            match self.fields.get(pick.field).and_then(|f| f.kind.category()) {
                Some(c) => t!("pform-pick-element-of", "category" => category_label(c)),
                None => t!("pform-pick-element"),
            }
        };
        match &pick.notice {
            Some(notice) => format!("{notice} {base}"),
            None => base,
        }
    }

    /// Why Run is disabled.
    fn blocked(&self, fmt: &Fmt) -> Option<String> {
        match &self.mode {
            Mode::Form => first_problem(&self.fields).map(|(f, p)| p.message(f, fmt)),
            Mode::Json { text, .. } => json_problem(text),
            Mode::Result { .. } => None,
        }
    }

    /// The JSON button: the form as JSON, or the JSON back in the form
    /// (staying in JSON, with the reason, when it does not fit).
    pub(crate) fn toggle_json(&mut self, fmt: &Fmt) {
        let back = match &self.mode {
            Mode::Json { text, .. } => Some(from_json(&mut self.fields, text, fmt)),
            _ => None,
        };
        match back {
            Some(Ok(())) => self.mode = Mode::Form,
            Some(Err(misfit)) => {
                if let Mode::Json { message, .. } = &mut self.mode {
                    *message = Some(misfit.message());
                }
            }
            None => self.mode = Mode::Json { text: to_json_text(&self.fields), message: None },
        }
    }

    /// Read the level, view, type and material choices again when the document changed.
    fn refresh_choices(&mut self, state: &mut AppState) {
        let revision = state.session.doc.revision();
        if self.revision == Some(revision) {
            return;
        }
        self.revision = Some(revision);
        for field in &mut self.fields {
            field.options = choices(state, &field.kind);
        }
    }

    /// The fields (tests).
    #[cfg(test)]
    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    /// Whether the result of a run is shown (tests).
    #[cfg(test)]
    pub fn shows_result(&self) -> bool {
        matches!(self.mode, Mode::Result { .. })
    }

    /// Whether the arguments are shown as JSON (tests).
    #[cfg(test)]
    pub fn shows_json(&self) -> bool {
        matches!(self.mode, Mode::Json { .. })
    }

    /// The field being picked (tests).
    #[cfg(test)]
    pub fn picking(&self) -> Option<usize> {
        self.picking.as_ref().map(|p| p.field)
    }

    /// The error shown over the footer (tests).
    #[cfg(test)]
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

/// The choices of a level, view, type or material field, from the engine's lists.
fn choices(state: &mut AppState, kind: &Kind) -> Vec<(Value, String)> {
    use cc_engine::commands::query::{LevelsList, MaterialsList, TypesList, ViewsList};
    let list = |out: cc_engine::Result<Value>| out.ok().and_then(|v| v.as_array().cloned()).unwrap_or_default();
    let named = |items: Vec<Value>| -> Vec<(u64, String)> {
        items.iter().filter_map(|v| Some((v["id"].as_u64()?, v["name"].as_str()?.to_string()))).collect()
    };
    match kind {
        Kind::Level => {
            named(list(state.session.run(LevelsList {}))).into_iter().map(|(id, n)| (json!(id), n)).collect()
        }
        Kind::View => named(list(state.session.run(ViewsList { category: None })))
            .into_iter()
            .map(|(id, _)| (json!(id), state.view_title(cc_core::ViewId(id))))
            .collect(),
        Kind::Type { category } => {
            let items = list(state.session.run(TypesList { category: category.clone() }));
            items
                .iter()
                .filter_map(|v| {
                    let (id, name) = (v["id"].as_u64()?, v["name"].as_str()?);
                    let label = match (category, v["category"].as_str()) {
                        (None, Some(c)) => format!("{}: {name}", category_label(c)),
                        _ => name.to_string(),
                    };
                    Some((json!(id), label))
                })
                .collect()
        }
        Kind::Material => list(state.session.run(MaterialsList {}))
            .iter()
            .filter_map(|v| v["name"].as_str().map(|n| (json!(n), n.to_string())))
            .collect(),
        _ => Vec::new(),
    }
}

/// Open the form of the plugin command `command`. A command without
/// parameters runs at once, the dialog showing only what it printed or
/// returned (or its error).
pub fn open(state: &mut AppState, command: &str) {
    open_prefilled(state, command, Map::new());
}

/// [`open`] with the fields named in `prefill` set to its values (a path
/// chosen for an exporter or importer). A command whose only parameters
/// are the prefilled ones runs at once.
pub fn open_prefilled(state: &mut AppState, command: &str, prefill: Map<String, Value>) {
    let Some(info) = state.session.describe(command) else {
        state.message = t!("pform-unknown", "name" => command);
        return;
    };
    let fmt = Fmt::current();
    let mut fields = fields(&info.input_schema, &fmt);
    for field in &mut fields {
        if let Some(v) = prefill.get(&field.name).filter(|v| fits(&field.kind, v)) {
            field.set_value(v.clone(), &fmt);
        }
    }
    // Element lists start from the selection.
    for field in &mut fields {
        if let Kind::Elements { category } = &field.kind {
            let ids = selected_of(state, category.as_deref());
            if !ids.is_empty() {
                field.set_value(json!(ids), &fmt);
            }
        }
    }
    let title = match state.plugins.command_title(command) {
        Some(title) => title.to_string(),
        None => title_of(&info.description, command),
    };
    let description = info.description.trim();
    let description = if description == title { String::new() } else { description.to_string() };
    let mut form = PluginForm {
        command: command.to_string(),
        title,
        plugin: plugin_name(state, command),
        description,
        fields,
        mode: Mode::Form,
        error: None,
        picking: None,
        revision: None,
        waiting: None,
    };
    let others = info.input_schema.get("properties").and_then(Value::as_object).is_some_and(|props| {
        props.keys().any(|name| !form.fields.iter().any(|f| f.name == *name && f.touched && prefill.contains_key(name)))
    });
    if !others {
        let args = args(&form.fields).unwrap_or_else(|| json!({}));
        if !run(state, &mut form, args) {
            return;
        }
    }
    state.dialogs.plugin_form = Some(form);
}

fn is_empty(v: &Value) -> bool {
    match v {
        Value::Null => true,
        Value::Object(map) => map.is_empty(),
        Value::Array(list) => list.is_empty(),
        _ => false,
    }
}

/// Run the command with `args`; false when the form closes (it ran and
/// had nothing to show: the status bar says it is done). A command that
/// waits for a program keeps the form, waiting.
fn run(state: &mut AppState, form: &mut PluginForm, args: Value) -> bool {
    let result = state.session.execute(&form.command, args);
    let printed = state.session.take_extension_messages();
    crate::plugin_console::record(state, &printed);
    match result {
        Err(e) if e.code == cc_engine::ErrorCode::Pending => {
            form.waiting = Some(printed);
            form.error = None;
            true
        }
        result => show_result(state, form, result.map_err(|e| e.message), printed),
    }
}

/// The command waiting for a program ended with `result` (it printed
/// `printed` since it started waiting): the open form shows it as a run
/// that ended at once would.
pub(crate) fn job_ended(state: &mut AppState, result: cc_engine::Result<Value>, printed: Vec<String>) {
    let Some(mut form) = state.dialogs.plugin_form.take() else { return };
    let mut lines = form.waiting.take().unwrap_or_default();
    lines.extend(printed);
    let result = result.map_err(|e| match e.code {
        cc_engine::ErrorCode::Cancelled => t!("job-plugin-cancelled"),
        _ => e.message,
    });
    if show_result(state, &mut form, result, lines) {
        state.dialogs.plugin_form = Some(form);
    }
}

/// Show what a run returned and printed; false when the form closes.
fn show_result(
    state: &mut AppState,
    form: &mut PluginForm,
    result: Result<Value, String>,
    printed: Vec<String>,
) -> bool {
    match result {
        Ok(out) if is_empty(&out) && printed.is_empty() => {
            state.message = t!("pform-done", "title" => form.title);
            false
        }
        Ok(out) => {
            let json = match &form.mode {
                Mode::Json { text, .. } => Some(text.clone()),
                _ => None,
            };
            let result = (!is_empty(&out)).then(|| serde_json::to_string_pretty(&out).unwrap_or_default());
            form.mode = Mode::Result { printed, result, json };
            form.error = None;
            true
        }
        Err(message) => {
            let mut lines = printed;
            lines.push(message);
            form.error = Some(lines.join("\n"));
            true
        }
    }
}

/// Run the open form as its Run button does (tests).
#[cfg(test)]
pub(crate) fn run_open(state: &mut AppState) {
    let Some(mut form) = state.dialogs.plugin_form.take() else { return };
    let args = args(&form.fields).unwrap_or_else(|| json!({}));
    if run(state, &mut form, args) {
        state.dialogs.plugin_form = Some(form);
    }
}

/// Every frame: the open form, unless one of its fields is being picked
/// (then the status bar prompts for the pick).
pub fn show(ctx: &egui::Context, state: &mut AppState) {
    let Some(mut form) = state.dialogs.plugin_form.take() else {
        // A pick left without its form (the document was replaced) ends.
        if matches!(state.tool, Tool::Pick { .. }) {
            state.set_tool(Tool::Select);
        }
        return;
    };
    // Behind the progress dialog until its command ends (a command the
    // session dropped, its plugins loaded again, gives the form back).
    if form.waiting.is_some() {
        if state.session.has_job() {
            state.dialogs.plugin_form = Some(form);
            return;
        }
        form.waiting = None;
    }
    if form.picking.is_some() {
        if matches!(state.tool, Tool::Pick { .. }) {
            state.hint.set(form.prompt());
        } else {
            // Esc or another tool: the form comes back as it was, from the
            // next frame (the Esc that ended the pick must not close it).
            form.picking = None;
            ctx.request_repaint();
        }
        state.dialogs.plugin_form = Some(form);
        return;
    }
    form.refresh_choices(state);
    let fmt = Fmt::current();
    if dialog(ctx, state, &mut form, &fmt) && state.dialogs.plugin_form.is_none() {
        state.dialogs.plugin_form = Some(form);
    }
}

/// The dialog; false when it closes.
fn dialog(ctx: &egui::Context, state: &mut AppState, form: &mut PluginForm, fmt: &Fmt) -> bool {
    let result_view = matches!(form.mode, Mode::Result { .. });
    let json_on = matches!(form.mode, Mode::Json { .. });
    let blocked = form.blocked(fmt);
    let (mut run_i, mut cancel, mut json, mut close, mut again) = (None, None, None, None, None);
    let id = ("plugin-form", form.command.clone());
    let response = Dialog::new(id, form.title.clone()).subtitle(form.plugin.clone()).width(WIDTH).show(
        ctx,
        |ui| body(ui, state, form, fmt),
        |f| {
            if result_view {
                close = Some(f.primary(t!("action-close")));
                again = Some(f.secondary(t!("pform-run-again")));
            } else {
                json = Some(f.toggle(t!("pform-json"), json_on));
                let run = f.primary(t!("palette-run"));
                if let Some(why) = &blocked {
                    f.disable(run);
                    f.tip(run, why.clone());
                }
                run_i = Some(run);
                cancel = Some(f.secondary(t!("action-cancel")));
            }
        },
    );
    if response.dismissed {
        return false;
    }
    let Some(clicked) = response.clicked.map(Some) else { return true };
    if clicked == close || clicked == cancel {
        return false;
    }
    if clicked == again {
        form.mode = match std::mem::replace(&mut form.mode, Mode::Form) {
            Mode::Result { json: Some(text), .. } => Mode::Json { text, message: None },
            _ => Mode::Form,
        };
    } else if clicked == json {
        form.toggle_json(fmt);
    } else if clicked == run_i {
        let args = match &form.mode {
            Mode::Json { text, .. } => serde_json::from_str::<Value>(text).ok().filter(Value::is_object),
            _ => args(&form.fields),
        };
        if let Some(args) = args {
            return run(state, form, args);
        }
    }
    true
}

fn body(ui: &mut Ui, state: &mut AppState, form: &mut PluginForm, fmt: &Fmt) {
    if !form.description.is_empty() {
        ui.label(theme::muted(&form.description));
        ui.add_space(4.0);
    }
    match &mut form.mode {
        Mode::Result { printed, result, .. } => result_body(ui, printed, result.as_deref()),
        Mode::Json { text, message } => json_body(ui, text, message.as_deref()),
        Mode::Form => {}
    }
    if matches!(form.mode, Mode::Form)
        && let Some(i) = fields_body(ui, state, &mut form.fields, &form.command, fmt)
    {
        let point = matches!(form.fields.get(i).map(|f| &f.kind), Some(Kind::Point));
        form.picking = Some(Picking { field: i, point, notice: None });
        // A field keeping the focus would keep Esc from cancelling the pick.
        ui.ctx().memory_mut(|m| {
            if let Some(id) = m.focused() {
                m.surrender_focus(id);
            }
        });
        state.set_tool(Tool::Pick { point });
    }
    if let Some(error) = &form.error {
        ui.add_space(6.0);
        egui::Frame::new()
            .fill(PALETTE.danger.gamma_multiply(0.1))
            .stroke(Stroke::new(1.0, PALETTE.danger))
            .corner_radius(theme::RADIUS)
            .inner_margin(egui::Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new(error).color(PALETTE.danger));
            });
    }
}

/// What a run printed and returned.
fn result_body(ui: &mut Ui, printed: &[String], result: Option<&str>) {
    if !printed.is_empty() {
        egui::ScrollArea::vertical().id_salt("pform-printed").max_height(160.0).auto_shrink([false, true]).show(
            ui,
            |ui| {
                for line in printed {
                    ui.label(line);
                }
            },
        );
    }
    if let Some(result) = result {
        ui.add_space(4.0);
        ui.label(RichText::new(t!("palette-result")).size(12.0).color(PALETTE.success));
        egui::ScrollArea::both().id_salt("pform-result").max_height(280.0).auto_shrink([false, true]).show(ui, |ui| {
            ui.add(egui::Label::new(RichText::new(result).monospace().size(12.0)).extend());
        });
    }
}

/// The arguments as JSON.
fn json_body(ui: &mut Ui, text: &mut String, message: Option<&str>) {
    ui.label(RichText::new(t!("palette-arguments")).size(12.0).color(PALETTE.text_muted));
    // Claim the editor's height from the first frame, so its scroll bar does
    // not flicker while the dialog grows to fit.
    let json_area = egui::ScrollArea::vertical().id_salt("pform-json").max_height(360.0).min_scrolled_height(360.0);
    json_area.auto_shrink([false, true]).show(ui, |ui| {
        let editor = egui::TextEdit::multiline(text).code_editor().desired_rows(10).desired_width(f32::INFINITY);
        let r = ui.add(editor);
        name_node(ui, &r, &t!("palette-arguments"));
    });
    if let Some(problem) = json_problem(text) {
        ui.label(RichText::new(problem).color(PALETTE.danger).size(12.0));
    }
    if let Some(message) = message {
        ui.label(RichText::new(message).color(PALETTE.warning));
    }
}

/// The fields in their sections; the index of a field whose Pick was clicked.
fn fields_body(ui: &mut Ui, state: &AppState, fields: &mut [Field], command: &str, fmt: &Fmt) -> Option<usize> {
    let (required, optional): (Vec<usize>, Vec<usize>) =
        (0..fields.len()).partition(|i| fields.get(*i).is_some_and(|f| f.required));
    let mut pick = None;
    egui::ScrollArea::vertical().id_salt("pform-fields").max_height(FIELDS_HEIGHT).auto_shrink([false, true]).show(
        ui,
        |ui| {
            let mut rows = |ui: &mut Ui, list: &[usize]| {
                for &i in list {
                    if let Some(field) = fields.get_mut(i)
                        && field_row(ui, state, field, fmt)
                    {
                        pick = Some(i);
                    }
                }
            };
            if !required.is_empty() {
                table::section(ui, &format!("pform-required-{command}"), t!("pform-parameters"), |ui| {
                    rows(ui, &required)
                });
            }
            if !optional.is_empty() {
                table::section_closed(ui, &format!("pform-optional-{command}"), t!("pform-more-options"), |ui| {
                    rows(ui, &optional)
                });
            }
        },
    );
    pick
}

/// Give a widget the field's name, for screen readers and UI tests (the
/// row's label is painted, not a widget).
fn name_node(ui: &Ui, r: &egui::Response, name: &str) {
    let name = name.to_string();
    ui.ctx().accesskit_node_builder(r.id, |node| node.set_label(name));
}

/// One field's row (and a JSON field's editor under it); true when its Pick was clicked.
fn field_row(ui: &mut Ui, state: &AppState, field: &mut Field, fmt: &Fmt) -> bool {
    let label = field.label.clone();
    let tip = (!field.description.is_empty()).then(|| field.description.clone());
    let pick = table::row_tip(ui, &label, tip.as_deref(), |ui| value_cell(ui, state, field, fmt));
    if let Kind::Json { .. } = field.kind {
        json_field(ui, field, fmt);
    }
    pick
}

/// The editor of a number, text, boolean or choice field (a plugin's
/// settings use these); other kinds show nothing.
pub(crate) fn plain_cell(ui: &mut Ui, field: &mut Field, fmt: &Fmt) {
    match field.kind.clone() {
        Kind::Number { .. } | Kind::Text => {
            // Empty, a required field says so; an optional one takes the command's default.
            let hint = if field.required { t!("pform-required-hint") } else { t!("pform-default") };
            let name = field.label.clone();
            text_field(ui, field, 0, None, &name, Some(hint), fmt);
        }
        Kind::Boolean => bool_field(ui, field, fmt),
        Kind::Choice { values, labels } => {
            let options: Vec<(Value, String)> = values.into_iter().map(|v| json!(v)).zip(labels).collect();
            combo(ui, field, &options, fmt);
        }
        _ => {}
    }
}

fn value_cell(ui: &mut Ui, state: &AppState, field: &mut Field, fmt: &Fmt) -> bool {
    match field.kind.clone() {
        Kind::Number { .. } | Kind::Text | Kind::Boolean | Kind::Choice { .. } => plain_cell(ui, field, fmt),
        Kind::Level | Kind::View | Kind::Type { .. } | Kind::Material => {
            let options = field.options.clone();
            combo(ui, field, &options, fmt);
        }
        Kind::Element { category } => return element_cell(ui, state, field, category.as_deref(), fmt),
        Kind::Elements { category } => elements_cell(ui, state, field, category.as_deref(), fmt),
        Kind::Point => return point_cell(ui, state, field, fmt),
        Kind::Path { save, extensions } => path_cell(ui, field, save, &extensions, fmt),
        Kind::Json { .. } => {
            ui.add_space(4.0);
            ui.label(RichText::new(t!("pform-json")).color(PALETTE.text_faint));
        }
    }
    false
}

/// A typed field over the text `i`, red with the reason on hover when invalid.
fn text_field(
    ui: &mut Ui,
    field: &mut Field,
    i: usize,
    width: Option<f32>,
    name: &str,
    hint: Option<String>,
    fmt: &Fmt,
) {
    let mut text = field.texts.get(i).cloned().unwrap_or_default();
    let invalid = field.invalid();
    let mut edit = egui::TextEdit::singleline(&mut text);
    if let Some(width) = width {
        edit = edit.desired_width(width);
    }
    if let Some(hint) = hint {
        edit = edit.hint_text(hint);
    }
    if invalid.is_some() {
        edit = edit.text_color(PALETTE.danger);
    }
    let r = ui.add(edit);
    name_node(ui, &r, name);
    if r.changed() {
        field.set_text(i, text, fmt);
    }
    if let Some(problem) = field.invalid() {
        r.on_hover_text(problem.message(field, fmt));
    }
}

/// A checkbox; an optional one without a default also has a third state,
/// not set (the command's default), which a click after "off" goes back to.
fn bool_field(ui: &mut Ui, field: &mut Field, fmt: &Fmt) {
    let shown = field.shown().and_then(Value::as_bool);
    let tri = !field.required && field.default.is_none();
    let mut checked = shown.unwrap_or(false);
    let unset = tri && shown.is_none();
    let r = ui.add(table::boxed(egui::Checkbox::without_text(&mut checked).indeterminate(unset)));
    name_node(ui, &r, &field.label);
    if r.clicked() {
        match (shown, tri) {
            (Some(true), _) => field.set_value(json!(false), fmt),
            (Some(false), true) => field.unset(),
            _ => field.set_value(json!(true), fmt),
        }
    }
    if unset {
        r.on_hover_text(t!("pform-unset"));
    }
}

/// A choice among `options`; "(default)" first for an optional field without a default.
fn combo(ui: &mut Ui, field: &mut Field, options: &[(Value, String)], fmt: &Fmt) {
    let unset_entry = !field.required && field.default.is_none();
    let shown = field.shown().cloned();
    let text = match &shown {
        Some(v) => options
            .iter()
            .find(|(o, _)| o == v)
            .map(|(_, label)| label.clone())
            .unwrap_or_else(|| v.as_str().map_or_else(|| v.to_string(), str::to_string)),
        None if unset_entry => t!("pform-default"),
        None => "—".to_string(),
    };
    let mut chosen: Option<Option<Value>> = None;
    let r = egui::ComboBox::from_id_salt(("pform-combo", field.name.as_str())).selected_text(text).show_ui(ui, |ui| {
        if unset_entry && ui.selectable_label(shown.is_none(), t!("pform-default")).clicked() {
            chosen = Some(None);
        }
        for (value, label) in options {
            if ui.selectable_label(shown.as_ref() == Some(value), label).clicked() {
                chosen = Some(Some(value.clone()));
            }
        }
    });
    name_node(ui, &r.response, &field.label);
    match chosen {
        Some(Some(value)) => field.set_value(value, fmt),
        Some(None) => field.unset(),
        None => {}
    }
}

/// Why nothing can be picked now.
fn pick_blocked(state: &AppState) -> Option<String> {
    if state.sketch.is_some() || state.component_env.is_some() {
        return Some(t!("pform-pick-busy"));
    }
    state.active_view.is_none().then(|| t!("pform-pick-no-view"))
}

/// A Pick button (disabled with `why`); true when clicked.
fn pick_button(ui: &mut Ui, why: Option<&str>, tip: String) -> bool {
    let r = ui.add_enabled(why.is_none(), egui::Button::new(t!("pform-pick")));
    let r = match why {
        Some(why) => r.on_disabled_hover_text(why),
        None => r.on_hover_text(tip),
    };
    r.clicked()
}

/// A button that takes the selection, disabled with `why`; true when clicked.
fn selection_button(ui: &mut Ui, why: Option<&str>) -> bool {
    let r = ui.add_enabled(why.is_none(), egui::Button::new(t!("pform-selection")));
    let r = match why {
        Some(why) => r.on_disabled_hover_text(why),
        None => r.on_hover_text(t!("pform-selection-tip")),
    };
    r.clicked()
}

/// `text` in what is left of a right-to-left cell, from its left edge, cut to fit.
fn cell_text(ui: &mut Ui, text: String) {
    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
        ui.add_space(4.0);
        ui.add(egui::Label::new(text).truncate());
    });
}

/// An element: its id, category and type, Pick and Selection; true when Pick was clicked.
fn element_cell(ui: &mut Ui, state: &AppState, field: &mut Field, category: Option<&str>, fmt: &Fmt) -> bool {
    let text = element_text(state.project(), field.shown());
    let matching = selected_of(state, category);
    let why_selection = (matching.len() != 1).then(|| match category {
        Some(c) => t!("pform-sel-one-of", "category" => category_label(c)),
        None => t!("pform-sel-one"),
    });
    let why_pick = pick_blocked(state);
    let mut pick = false;
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        if selection_button(ui, why_selection.as_deref())
            && let Some(id) = matching.first()
        {
            field.set_value(json!(id), fmt);
        }
        pick = pick_button(ui, why_pick.as_deref(), t!("pform-pick-tip-element"));
        cell_text(ui, text);
    });
    pick
}

/// Elements: how many, Selection and Clear.
fn elements_cell(ui: &mut Ui, state: &AppState, field: &mut Field, category: Option<&str>, fmt: &Fmt) {
    let count = field.shown().and_then(Value::as_array).map_or(0, Vec::len);
    let matching = selected_of(state, category);
    let why = matching.is_empty().then(|| match category {
        Some(c) => t!("pform-sel-some-of", "category" => category_label(c)),
        None => t!("pform-sel-some"),
    });
    let set = field.shown().is_some();
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        if ui.add_enabled(set, egui::Button::new(t!("pform-clear"))).clicked() {
            field.unset();
        }
        if selection_button(ui, why.as_deref()) {
            field.set_value(json!(matching), fmt);
        }
        cell_text(ui, t!("pform-elements", "count" => count));
    });
}

/// Width of a button with `label` in `ui`.
fn button_width(ui: &Ui, label: &str) -> f32 {
    let galley = egui::WidgetText::from(label).into_galley(
        ui,
        Some(egui::TextWrapMode::Extend),
        f32::INFINITY,
        egui::FontSelection::Default,
    );
    galley.size().x + 2.0 * ui.spacing().button_padding.x
}

/// A point: x and y lengths and Pick; true when Pick was clicked.
fn point_cell(ui: &mut Ui, state: &AppState, field: &mut Field, fmt: &Fmt) -> bool {
    let why = pick_blocked(state);
    let spacing = ui.spacing().item_spacing.x;
    let pick_w = button_width(ui, &t!("pform-pick"));
    let w = ((ui.available_width() - pick_w - 3.0 * spacing) / 2.0).max(24.0);
    let name = field.label.clone();
    text_field(ui, field, 0, Some(w), &format!("{name} x"), Some("x".into()), fmt);
    text_field(ui, field, 1, Some(w), &format!("{name} y"), Some("y".into()), fmt);
    pick_button(ui, why.as_deref(), t!("pform-pick-tip-point"))
}

/// A file path: the path typed, and Browse… to choose it in the save or
/// open dialog.
fn path_cell(ui: &mut Ui, field: &mut Field, save: bool, extensions: &[String], fmt: &Fmt) {
    let browse = t!("pform-browse");
    let w = (ui.available_width() - button_width(ui, &browse) - 2.0 * ui.spacing().item_spacing.x).max(24.0);
    let hint = if field.required { t!("pform-required-hint") } else { t!("pform-default") };
    let name = field.label.clone();
    text_field(ui, field, 0, Some(w), &name, Some(hint), fmt);
    if ui.button(browse).on_hover_text(t!("pform-browse-tip")).clicked()
        && let Some(path) = choose_path(save, extensions, None, field.texts[0].trim())
    {
        field.set_value(json!(path.to_string_lossy()), fmt);
    }
}

/// The app's save (`save`) or open dialog for a file of one of
/// `extensions` (any when empty), the filter named `filter` (else the
/// extensions), starting at `current` when it is a path. A saved path
/// without an extension gets the first one.
pub(crate) fn choose_path(
    save: bool,
    extensions: &[String],
    filter: Option<&str>,
    current: &str,
) -> Option<std::path::PathBuf> {
    let mut dialog = rfd::FileDialog::new();
    if !extensions.is_empty() {
        let list = extensions.iter().map(|e| format!("*.{e}")).collect::<Vec<_>>().join(", ");
        let name = match filter {
            Some(f) => f.to_string(),
            None => t!("plug-file-filter", "extensions" => list),
        };
        dialog = dialog.add_filter(name, extensions);
    }
    let current = std::path::Path::new(current);
    if !current.as_os_str().is_empty() {
        if let Some(dir) = current.parent().filter(|d| d.is_dir()) {
            dialog = dialog.set_directory(dir);
        }
        if let Some(name) = current.file_name() {
            dialog = dialog.set_file_name(name.to_string_lossy());
        }
    }
    if !save {
        return dialog.pick_file();
    }
    let mut path = dialog.save_file()?;
    if path.extension().is_none()
        && let Some(first) = extensions.first()
    {
        path.set_extension(first);
    }
    Some(path)
}

/// A JSON field's editor, under its row, with what is wrong with it.
fn json_field(ui: &mut Ui, field: &mut Field, fmt: &Fmt) {
    table::padded(ui, |ui| {
        let mut text = field.texts[0].clone();
        let editor = egui::TextEdit::multiline(&mut text).code_editor().desired_rows(3).desired_width(f32::INFINITY);
        let r = ui.add(editor);
        name_node(ui, &r, &field.label);
        if r.changed() {
            field.set_text(0, text, fmt);
        }
        if let Some(problem) = field.invalid() {
            ui.label(RichText::new(problem.message(field, fmt)).color(PALETTE.danger).size(12.0));
        }
    });
}

// --- Picking in the view ---

/// The prompt of the pick under way, for the options bar.
pub(crate) fn prompt(state: &AppState) -> String {
    state.dialogs.plugin_form.as_ref().map(PluginForm::prompt).unwrap_or_default()
}

/// Why the last click was not taken (shown with the prompt).
fn notice(state: &mut AppState, text: String) {
    if let Some(pick) = state.dialogs.plugin_form.as_mut().and_then(|f| f.picking.as_mut()) {
        pick.notice = Some(text);
    }
}

/// The category the element being picked keeps to.
fn picking_category(state: &AppState) -> Option<String> {
    let form = state.dialogs.plugin_form.as_ref()?;
    let pick = form.picking.as_ref()?;
    form.fields.get(pick.field)?.kind.category().map(str::to_string)
}

/// [`Tool::Pick`] in a 2D view: an element of the field's category under
/// the pointer is highlighted and taken by a click; a point (in plans only)
/// snaps as the drawing tools' points do.
#[allow(clippy::too_many_arguments)]
pub(crate) fn pick_tool(
    ui: &Ui,
    resp: &egui::Response,
    painter: &egui::Painter,
    state: &mut AppState,
    view: &cc_core::View,
    point: bool,
    xf: &crate::paint::Xf,
    drawing: &cc_views::Drawing2D,
) {
    let Some(raw) = resp.hover_pos().map(|p| xf.to_world(p)) else { return };
    if point {
        if !view.is_plan() {
            if resp.clicked() {
                notice(state, t!("pform-pick-plan"));
            }
            return;
        }
        let shift = ui.input(|i| i.modifiers.shift);
        let s = crate::tools::snap_at(state, view.level(), raw, None, &[], xf.scale, shift);
        crate::tools::draw_snap(painter, xf, s);
        if resp.clicked() {
            picked_point(state, s.point);
        }
        return;
    }
    let category = picking_category(state);
    let hits = drawing.hits_at(raw, 6.0 / xf.scale);
    let project = state.project();
    let wanted = hits
        .iter()
        .copied()
        .find(|id| project.element(*id).is_some_and(|e| accepts(category.as_deref(), e.category())));
    let lit: BTreeSet<ElementId> = wanted.into_iter().collect();
    crate::modify::paint_ghost(painter, xf, drawing, &lit, &cc_core::PlanTransform::IDENTITY);
    if resp.clicked() {
        picked_element(state, wanted.or(hits.first().copied()));
    }
}

/// A click while picking (2D or 3D) on `picked`: an element of the field's
/// category fills it and brings the form back; another one is refused with a
/// notice; a point field asks for a plan.
pub(crate) fn picked_element(state: &mut AppState, picked: Option<ElementId>) {
    let Some(form) = state.dialogs.plugin_form.as_mut() else { return };
    let Some(pick) = form.picking.as_mut() else { return };
    if pick.point {
        pick.notice = Some(t!("pform-pick-plan"));
        return;
    }
    let Some(id) = picked else { return };
    let Some(field) = form.fields.get_mut(pick.field) else { return };
    let Some(element) = state.session.doc.project.element(id) else { return };
    let category = field.kind.category().map(str::to_string);
    if !accepts(category.as_deref(), element.category()) {
        let name = category.as_deref().map(category_label).unwrap_or_default();
        pick.notice = Some(t!("pform-pick-not", "category" => name));
        return;
    }
    field.set_value(json!(id.0), &Fmt::current());
    form.picking = None;
    state.set_tool(Tool::Select);
}

/// A point picked in a plan fills the point field and brings the form back.
pub(crate) fn picked_point(state: &mut AppState, p: glam::DVec2) {
    let Some(form) = state.dialogs.plugin_form.as_mut() else { return };
    let Some(pick) = &form.picking else { return };
    if !pick.point {
        return;
    }
    if let Some(field) = form.fields.get_mut(pick.field) {
        field.set_value(json!([p.x, p.y]), &Fmt::current());
    }
    form.picking = None;
    state.set_tool(Tool::Select);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metric() -> Fmt {
        Fmt { units: cc_core::Units::default(), comma: false }
    }

    fn schema() -> Value {
        json!({
            "type": "object",
            "properties": {
                "width": {"type": "number", "ui": "length", "minimum": 0.1, "description": "Size along x."},
                "count": {"type": "integer", "default": 3, "maximum": 10},
                "name": {"type": "string", "title": "Room name"},
                "door": {"type": "string", "enum": ["south", "north"], "enum_labels": ["South side", "North side"]},
                "side": {"type": "string", "enum": ["a", "b"], "enum_labels": ["only one"]},
                "flag": {"type": "boolean"},
                "wall": {"type": "integer", "ui": "element", "category": "wall"},
                "walls": {"type": "array", "ui": "elements", "category": "wall"},
                "at": {"type": "array", "ui": "point"},
                "level": {"type": "integer", "ui": "level"},
                "finish": {"type": "string", "ui": "material"},
                "extra": {"type": "object"},
                "anything": {"description": "No type."},
            },
            "required": ["width", "wall", "count"],
        })
    }

    fn field<'a>(fields: &'a [Field], name: &str) -> &'a Field {
        fields.iter().find(|f| f.name == name).unwrap()
    }

    fn field_mut<'a>(fields: &'a mut [Field], name: &str) -> &'a mut Field {
        fields.iter_mut().find(|f| f.name == name).unwrap()
    }

    #[test]
    fn a_schema_makes_fields_required_first() {
        let fields = fields(&schema(), &metric());
        let names: Vec<&str> = fields.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(&names[..3], ["width", "wall", "count"], "required ones first, as `required` lists them");
        assert!(fields[3..].iter().all(|f| !f.required));
        assert_eq!(field(&fields, "width").kind, Kind::Number { integer: false, length: true });
        assert_eq!(field(&fields, "count").kind, Kind::Number { integer: true, length: false });
        assert_eq!(field(&fields, "name").kind, Kind::Text);
        assert_eq!(field(&fields, "flag").kind, Kind::Boolean);
        assert_eq!(field(&fields, "wall").kind, Kind::Element { category: Some("wall".into()) });
        assert_eq!(field(&fields, "walls").kind, Kind::Elements { category: Some("wall".into()) });
        assert_eq!(field(&fields, "at").kind, Kind::Point);
        assert_eq!(field(&fields, "level").kind, Kind::Level);
        assert_eq!(field(&fields, "finish").kind, Kind::Material);
        assert_eq!(field(&fields, "extra").kind, Kind::Json { want: Some("object".into()) });
        assert_eq!(field(&fields, "anything").kind, Kind::Json { want: None });
        // Labels: the title, else the name in words; the description is the tooltip.
        assert_eq!(field(&fields, "name").label, "Room name");
        assert_eq!(field(&fields, "width").label, "Width");
        assert_eq!(field(&fields, "width").description, "Size along x.");
        assert_eq!(humanize("rectangle_room"), "Rectangle room");
    }

    #[test]
    fn enum_labels_show_when_they_match_the_values() {
        let fields = fields(&schema(), &metric());
        let Kind::Choice { values, labels } = &field(&fields, "door").kind else { panic!("not a choice") };
        assert_eq!(values, &["south", "north"]);
        assert_eq!(labels, &["South side", "North side"]);
        // Labels of another length are ignored: the values show.
        let Kind::Choice { labels, .. } = &field(&fields, "side").kind else { panic!("not a choice") };
        assert_eq!(labels, &["a", "b"]);
    }

    #[test]
    fn defaults_fill_the_form_and_optional_fields_are_sent_only_when_set() {
        let fmt = metric();
        let mut fields = fields(&schema(), &fmt);
        assert_eq!(field(&fields, "count").texts[0], "3", "the default shows");
        assert_eq!(first_problem(&fields).map(|(f, p)| (f.name.clone(), p)), Some(("width".into(), Problem::Missing)));
        assert!(args(&fields).is_none(), "required fields missing");
        field_mut(&mut fields, "width").set_text(0, "2.5".into(), &fmt);
        field_mut(&mut fields, "wall").set_value(json!(7), &fmt);
        // A required field's default is sent; optional ones left alone are not.
        assert_eq!(args(&fields), Some(json!({"width": 2.5, "wall": 7, "count": 3})));
        field_mut(&mut fields, "flag").set_value(json!(false), &fmt);
        field_mut(&mut fields, "name").set_text(0, "Kitchen".into(), &fmt);
        let sent = args(&fields).unwrap();
        assert_eq!((sent["flag"].clone(), sent["name"].clone()), (json!(false), json!("Kitchen")));
        // Emptied, an optional text is unset again.
        field_mut(&mut fields, "name").set_text(0, String::new(), &fmt);
        assert!(args(&fields).unwrap().get("name").is_none());
        // A default its field cannot hold is left out.
        let odd = fields_of(json!({"properties": {"n": {"type": "number", "default": "two"}}}));
        assert_eq!(odd[0].default, None);
    }

    fn fields_of(schema: Value) -> Vec<Field> {
        fields(&schema, &metric())
    }

    #[test]
    fn fields_check_what_is_typed() {
        let fmt = metric();
        let mut fields = fields(&schema(), &fmt);
        let width = field_mut(&mut fields, "width");
        width.set_text(0, "abc".into(), &fmt);
        assert_eq!(width.problem(), Some(Problem::NotLength));
        width.set_text(0, "0.05".into(), &fmt);
        assert_eq!(width.problem(), Some(Problem::Below(0.1)), "not clamped: shown as invalid");
        assert!(width.message_of_problem(&fmt).contains("0.10 m"), "{}", width.message_of_problem(&fmt));
        let count = field_mut(&mut fields, "count");
        count.set_text(0, "2.5".into(), &fmt);
        assert_eq!(count.problem(), Some(Problem::NotInteger));
        count.set_text(0, "11".into(), &fmt);
        assert_eq!(count.problem(), Some(Problem::Above(10.0)));
        count.set_text(0, "4".into(), &fmt);
        assert_eq!(count.effective(), Some(&json!(4)));
        let extra = field_mut(&mut fields, "extra");
        extra.set_text(0, "{\"a\": ".into(), &fmt);
        assert!(matches!(extra.problem(), Some(Problem::Json(_))));
        extra.set_text(0, "[1]".into(), &fmt);
        assert_eq!(extra.problem(), Some(Problem::WantObject));
        extra.set_text(0, "{\"a\": 1}".into(), &fmt);
        assert_eq!(extra.effective(), Some(&json!({"a": 1})));
        let at = field_mut(&mut fields, "at");
        at.set_text(0, "1".into(), &fmt);
        assert_eq!(at.problem(), Some(Problem::NotLength), "x without y");
        at.set_text(1, "2.5".into(), &fmt);
        assert_eq!(at.effective(), Some(&json!([1.0, 2.5])));
        // Required elements must not be empty.
        let mut listed =
            fields_of(json!({"properties": {"ids": {"type": "array", "ui": "elements"}}, "required": ["ids"]}));
        listed[0].set_value(json!([]), &fmt);
        assert_eq!(listed[0].problem(), Some(Problem::Missing));
    }

    impl Field {
        fn message_of_problem(&self, fmt: &Fmt) -> String {
            self.problem().map(|p| p.message(self, fmt)).unwrap_or_default()
        }
    }

    #[test]
    fn lengths_read_and_show_in_the_project_units() {
        let mut units = cc_core::Units::default();
        units.length.unit = cc_core::LengthUnit::Millimeter;
        units.length.decimals = 0;
        let mm = Fmt { units, comma: true };
        let schema = json!({"properties": {
            "w": {"type": "number", "ui": "length", "default": 2.8},
            "p": {"type": "array", "ui": "point", "default": [1.5, 0.25]},
        }});
        let mut fields = fields(&schema, &mm);
        assert_eq!(field(&fields, "w").texts[0], "2800 mm");
        assert_eq!(field(&fields, "p").texts, ["1500 mm".to_string(), "250 mm".to_string()]);
        let w = field_mut(&mut fields, "w");
        w.set_text(0, "1250".into(), &mm);
        assert_eq!(w.effective(), Some(&json!(1.25)), "typed in millimeters, sent in meters");
        w.set_text(0, "3,5 m".into(), &mm);
        assert_eq!(w.effective(), Some(&json!(3.5)), "a unit typed overrides the project's");
        // Plain numbers take the language's decimal symbol.
        let mut n = fields_of(json!({"properties": {"n": {"type": "number", "default": 0.5}}}));
        assert_eq!(texts_of(&n[0].kind, &json!(0.5), &mm)[0], "0,5");
        n[0].set_text(0, "1,25".into(), &mm);
        assert_eq!(n[0].effective(), Some(&json!(1.25)));
    }

    #[test]
    fn json_goes_back_and_forth_with_the_form() {
        let fmt = metric();
        let mut fields = fields(&schema(), &fmt);
        field_mut(&mut fields, "width").set_text(0, "4".into(), &fmt);
        field_mut(&mut fields, "door").set_value(json!("north"), &fmt);
        let text = to_json_text(&fields);
        let as_json: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(as_json, json!({"width": 4.0, "count": 3, "door": "north"}));
        // Back: the same arguments; what JSON leaves out is unset.
        let edited = r#"{"width": 5, "wall": 12, "count": 2, "walls": [1, 2], "at": [0, 1.5]}"#;
        from_json(&mut fields, edited, &fmt).unwrap();
        assert_eq!(args(&fields), Some(serde_json::from_str::<Value>(edited).unwrap()));
        assert_eq!(field(&fields, "door").effective(), None, "left out: not sent");
        assert_eq!(field(&fields, "width").texts[0], "5.00 m");
        // What does not fit stays out of the form, which is left as it was.
        let before = args(&fields);
        assert_eq!(from_json(&mut fields, r#"{"nope": 1}"#, &fmt), Err(Misfit::Unknown("nope".into())));
        assert_eq!(from_json(&mut fields, r#"{"wall": "a"}"#, &fmt), Err(Misfit::Wrong("Wall".into())));
        assert_eq!(from_json(&mut fields, r#"{"door": "east"}"#, &fmt), Err(Misfit::Wrong("Door".into())));
        assert_eq!(from_json(&mut fields, "[1]", &fmt), Err(Misfit::NotObject));
        assert!(matches!(from_json(&mut fields, "{", &fmt), Err(Misfit::Json(_))));
        assert_eq!(args(&fields), before);
        // A required field left out of the JSON is missing.
        from_json(&mut fields, r#"{"wall": 12, "count": 2}"#, &fmt).unwrap();
        assert_eq!(first_problem(&fields).map(|(f, p)| (f.name.clone(), p)), Some(("width".into(), Problem::Missing)));
    }

    #[test]
    fn titles_and_categories() {
        assert_eq!(title_of("Rectangular room", "quick_rooms__rectangle_room"), "Rectangular room");
        let long = "Four walls around a rectangle, its floor and its room, with an optional door.";
        assert_eq!(title_of(long, "quick_rooms__rectangle_room"), "Rectangle room");
        assert_eq!(title_of("", "quick_rooms__wall_totals"), "Wall totals");
        assert_eq!(parse_category("wall"), Some(Category::Wall));
        assert_eq!(parse_category("cable_tray"), Some(Category::CableTray));
        assert_eq!(parse_category("Door"), Some(Category::Door));
        assert_eq!(parse_category("nonsense"), None);
        assert!(accepts(None, Category::Door));
        assert!(accepts(Some("wall"), Category::Wall));
        assert!(!accepts(Some("wall"), Category::Door));
        assert!(!accepts(Some("nonsense"), Category::Wall), "an unknown category takes nothing");
    }

    #[test]
    fn empty_results_close_the_form() {
        assert!(is_empty(&Value::Null) && is_empty(&json!({})) && is_empty(&json!([])));
        assert!(!is_empty(&json!(0)) && !is_empty(&json!({"id": 1})) && !is_empty(&json!("")));
    }
}
