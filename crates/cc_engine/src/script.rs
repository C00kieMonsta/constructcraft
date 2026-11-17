//! Scripts (macros): a list of commands with their arguments, as a JSON
//! array of `{"command": ..., "args": {...}}` objects or one such object per
//! line (`//` lines are comments). `ccad script` runs them, the app records
//! and plays them as macros.
//!
//! Arguments may hold two kinds of placeholders, resolved just before their
//! step runs: `{"$step": n}` (or `{"$step": n, "path": "/created/0"}`) is a
//! value of the result of the earlier step `n` (counted from 1; `path` is a
//! JSON pointer, `/id` by default), so a step can use what an earlier one
//! created; `{"$level": "active"}` is the active level (see
//! [`Session::set_active_level`]; without one, the lowest level). Recording
//! writes both (see [`link`] and [`mark_level`]).

use crate::error::{EngineError, Result};
use crate::session::Session;
use cc_core::LevelId;
use serde_json::{Map, Value, json};

/// Key of a reference to an earlier step's result.
pub const STEP_KEY: &str = "$step";
/// Key of the active level placeholder (its value is `"active"`).
pub const LEVEL_KEY: &str = "$level";
/// Deepest nesting of arguments and results walked.
const MAX_DEPTH: usize = 64;
/// Most values of one result searched for a created id.
const MAX_SEARCH: usize = 10_000;
/// Keys of integer arguments that are counts, indices or scales, never ids.
const NOT_IDS: &[&str] = &[
    "index",
    "indices",
    "edge",
    "remove",
    "count",
    "ports",
    "scale",
    "leaves",
    "panels",
    "side",
    "sides",
    "risers",
    "split",
    "winders",
    "number_start",
    "major_every",
    "dropped",
];

/// One step of a script.
#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    pub command: String,
    pub args: Value,
}

impl Step {
    pub fn to_json(&self) -> Value {
        json!({ "command": self.command, "args": self.args })
    }
}

/// Read a script.
pub fn parse(text: &str) -> Result<Vec<Step>> {
    let items: Vec<Value> = if text.trim_start().starts_with('[') {
        serde_json::from_str(text).map_err(|e| EngineError::invalid(format!("invalid script: {e}")))?
    } else {
        text.lines()
            .enumerate()
            .filter(|(_, l)| !l.trim().is_empty() && !l.trim_start().starts_with("//"))
            .map(|(i, l)| {
                serde_json::from_str(l)
                    .map_err(|e| EngineError::invalid(format!("invalid script, line {}: {e}", i + 1)))
            })
            .collect::<Result<_>>()?
    };
    items
        .into_iter()
        .map(|item| {
            let command = item
                .get("command")
                .and_then(Value::as_str)
                .ok_or_else(|| EngineError::invalid("each step needs a \"command\""))?;
            Ok(Step { command: command.to_string(), args: item.get("args").cloned().unwrap_or(json!({})) })
        })
        .collect()
}

/// A script with one step per line (the format macros are saved in).
pub fn to_text(steps: &[Step]) -> String {
    steps.iter().map(|s| format!("{}\n", s.to_json())).collect()
}

/// A failed step: its index (from 0) and the error.
#[derive(Debug, Clone, PartialEq)]
pub struct StepError {
    pub index: usize,
    pub command: String,
    pub error: EngineError,
}

/// Run the steps in order, each one undo step, stopping at the first error;
/// returns each completed step's result. Placeholders in the arguments are
/// resolved first (see the module documentation).
pub fn run(session: &mut Session, steps: &[Step]) -> std::result::Result<Vec<Value>, (Vec<Value>, StepError)> {
    let mut stepper = Stepper::new(steps.to_vec());
    match stepper.run_to(session, steps.len()) {
        Ok(()) => Ok(stepper.into_results()),
        Err(error) => Err((stepper.into_results(), error)),
    }
}

/// Most lines (or array items) of a script read line by line or checked.
pub const MAX_LINES: usize = 100_000;

/// What one line of a script is (see [`parse_lines`]).
#[derive(Debug, Clone, PartialEq)]
pub enum LineItem {
    Blank,
    /// A `//` line.
    Comment,
    /// A step: its number (from 1, counting only the lines that are steps),
    /// the step, and the keys of its object other than `command` and `args`,
    /// which running it ignores.
    Step {
        number: usize,
        step: Step,
        ignored: Vec<String>,
    },
    Error(LineError),
}

/// Why a line is not a step.
#[derive(Debug, Clone, PartialEq)]
pub enum LineError {
    /// Not JSON (the parser's message).
    Json(String),
    /// JSON, but not an object.
    NotObject,
    /// An object with no `"command"` text.
    NoCommand,
    /// Past [`MAX_LINES`]: the rest of the script was not read.
    TooLong,
}

impl LineError {
    /// The message for the user, in the active language.
    pub fn message(&self) -> String {
        match self {
            LineError::Json(detail) => crate::t!("macro-check-json", "detail" => detail),
            LineError::NotObject => crate::t!("macro-check-not-object"),
            LineError::NoCommand => crate::t!("macro-check-no-command"),
            LineError::TooLong => crate::t!("macro-check-too-long", "max" => MAX_LINES),
        }
    }
}

/// One line of a script (see [`parse_lines`]).
#[derive(Debug, Clone, PartialEq)]
pub struct ScriptLine {
    /// Its line number, from 1; `None` for the items of the array format.
    pub line: Option<usize>,
    pub item: LineItem,
}

/// Read a script line by line without stopping at a bad line: in the line
/// format, an entry per source line (blank, comment, step or error), so an
/// editor can show what each line is; lines that are not steps get no step
/// number, so the numbers are those [`parse`] gives when every line is good.
/// The array format is one JSON value, not lines: its steps (and items that
/// are not steps) come with no line number, and JSON that does not parse is
/// one error at the line the parser stopped at. At most [`MAX_LINES`] lines
/// or items are read; a [`LineError::TooLong`] entry stands for the rest.
/// [`parse`] keeps its own behaviour (all or nothing).
pub fn parse_lines(text: &str) -> Vec<ScriptLine> {
    let mut number = 0;
    let mut step_item = |item: &Value| -> LineItem {
        match step_of(item) {
            Ok((step, ignored)) => {
                number += 1;
                LineItem::Step { number, step, ignored }
            }
            Err(e) => LineItem::Error(e),
        }
    };
    if text.trim_start().starts_with('[') {
        let items: Vec<Value> = match serde_json::from_str(text) {
            Ok(items) => items,
            Err(e) => return vec![ScriptLine { line: Some(e.line().max(1)), item: LineItem::Error(json_error(&e)) }],
        };
        let mut out: Vec<ScriptLine> =
            items.iter().take(MAX_LINES).map(|v| ScriptLine { line: None, item: step_item(v) }).collect();
        if items.len() > MAX_LINES {
            out.push(ScriptLine { line: None, item: LineItem::Error(LineError::TooLong) });
        }
        return out;
    }
    let mut out = Vec::new();
    for (i, l) in text.lines().enumerate() {
        if i >= MAX_LINES {
            out.push(ScriptLine { line: Some(i + 1), item: LineItem::Error(LineError::TooLong) });
            break;
        }
        let t = l.trim();
        let item = if t.is_empty() {
            LineItem::Blank
        } else if t.starts_with("//") {
            LineItem::Comment
        } else {
            match serde_json::from_str::<Value>(l) {
                Ok(v) => step_item(&v),
                Err(e) => LineItem::Error(json_error(&e)),
            }
        };
        out.push(ScriptLine { line: Some(i + 1), item });
    }
    out
}

fn json_error(e: &serde_json::Error) -> LineError {
    LineError::Json(e.to_string())
}

/// The step a JSON value is, with the keys it ignores.
fn step_of(item: &Value) -> std::result::Result<(Step, Vec<String>), LineError> {
    let Value::Object(m) = item else { return Err(LineError::NotObject) };
    let command = m.get("command").and_then(Value::as_str).ok_or(LineError::NoCommand)?;
    let ignored = m.keys().filter(|k| *k != "command" && *k != "args").cloned().collect();
    Ok((Step { command: command.to_string(), args: m.get("args").cloned().unwrap_or(json!({})) }, ignored))
}

/// How bad a [`Diagnostic`] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// The step cannot run as written (or the script cannot be read).
    Error,
    /// It runs, but probably not as meant.
    Warning,
}

/// A problem [`check`] found.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Diagnostic {
    /// The line, from 1 (`None` for a step of the array format).
    pub line: Option<usize>,
    /// The step, from 1, when the problem is in one.
    pub step: Option<usize>,
    pub severity: Severity,
    /// In the active language.
    pub message: String,
}

/// Check a script without running it, in the order of its lines: lines
/// that are not steps (see [`parse_lines`]), commands the session does not
/// have (plugin commands count), `args` that is not an object, keys of a
/// step that running it ignores (warnings), argument names the command's
/// schema refuses (when it allows no others) and required arguments left
/// out, and placeholders that cannot resolve (`$step` that is not an
/// earlier step of the script or whose `path` is not a JSON pointer,
/// `$level` other than `"active"`). Values are not checked against the
/// schema, nor `$step` paths against what the steps will return. Nothing
/// runs and the document is not touched.
pub fn check(session: &Session, text: &str) -> Vec<Diagnostic> {
    let lines = parse_lines(text);
    let count = lines.iter().filter(|l| matches!(l.item, LineItem::Step { .. })).count();
    let catalog: std::collections::HashMap<String, Value> =
        session.commands().into_iter().map(|c| (c.name, c.input_schema)).collect();
    let mut out = Vec::new();
    for l in &lines {
        let mut add = |step: Option<usize>, severity: Severity, message: String| {
            out.push(Diagnostic { line: l.line, step, severity, message });
        };
        let (number, step, ignored) = match &l.item {
            LineItem::Blank | LineItem::Comment => continue,
            LineItem::Error(e) => {
                add(None, Severity::Error, e.message());
                continue;
            }
            LineItem::Step { number, step, ignored } => (*number, step, ignored),
        };
        let at = Some(number);
        for key in ignored {
            add(at, Severity::Warning, crate::t!("macro-check-ignored-key", "key" => key));
        }
        match catalog.get(&step.command) {
            None => add(at, Severity::Error, crate::t!("macro-check-unknown-command", "command" => step.command)),
            Some(schema) => {
                for message in check_args(&step.command, &step.args, schema) {
                    add(at, Severity::Error, message);
                }
            }
        }
        let mut problems = Vec::new();
        check_placeholders(&step.args, number, count, &mut problems, 0);
        for message in problems {
            add(at, Severity::Error, message);
        }
    }
    out
}

/// What the top level of a command's schema plainly refuses in `args`.
fn check_args(command: &str, args: &Value, schema: &Value) -> Vec<String> {
    let empty = Map::new();
    let args = match args {
        Value::Object(m) => m,
        // Commands read `null` as no arguments.
        Value::Null => &empty,
        _ => return vec![crate::t!("macro-check-args-object")],
    };
    let mut out = Vec::new();
    if let Some(props) = schema.get("properties").and_then(Value::as_object)
        && schema.get("additionalProperties") == Some(&Value::Bool(false))
    {
        for name in args.keys().filter(|k| !props.contains_key(*k)) {
            out.push(crate::t!("macro-check-unknown-arg", "command" => command, "name" => name));
        }
    }
    for name in schema.get("required").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str) {
        if !args.contains_key(name) {
            out.push(crate::t!("macro-check-missing-arg", "command" => command, "name" => name));
        }
    }
    out
}

/// The placeholders in the arguments of step `number` (of `count`) that
/// cannot resolve, as [`resolve`] reads them.
fn check_placeholders(v: &Value, number: usize, count: usize, out: &mut Vec<String>, depth: usize) {
    if depth > MAX_DEPTH {
        out.push(crate::t!("macro-check-deep"));
        return;
    }
    match v {
        Value::Object(m) if m.contains_key(STEP_KEY) => {
            match m.get(STEP_KEY).and_then(Value::as_u64) {
                None => out.push(crate::t!("macro-check-step-number")),
                Some(0) => out.push(crate::t!("macro-check-step-zero")),
                Some(n) if usize::try_from(n).map_or(true, |n| n > count) => {
                    out.push(crate::t!("macro-check-step-beyond", "n" => n, "count" => count));
                }
                Some(n) if usize::try_from(n).map_or(true, |n| n >= number) => {
                    out.push(crate::t!("macro-check-step-later", "n" => n));
                }
                Some(_) => {}
            }
            match m.get("path") {
                None => {}
                Some(Value::String(p)) if p.is_empty() || p.starts_with('/') => {}
                Some(_) => out.push(crate::t!("macro-check-path")),
            }
        }
        Value::Object(m) if m.contains_key(LEVEL_KEY) => {
            if m.get(LEVEL_KEY).and_then(Value::as_str) != Some("active") {
                out.push(crate::t!("macro-check-level"));
            }
        }
        Value::Object(m) => m.values().for_each(|v| check_placeholders(v, number, count, out, depth + 1)),
        Value::Array(a) => a.iter().for_each(|v| check_placeholders(v, number, count, out, depth + 1)),
        _ => {}
    }
}

/// A script rewritten with one step per line in the form [`to_text`]
/// writes, keeping comments and blank lines where they are and lines that
/// are not steps as they are. A script in the array format becomes the line
/// format; one that does not read is refused. The text ends with a newline
/// when it has any line.
pub fn format(text: &str) -> Result<String> {
    if text.trim_start().starts_with('[') {
        return parse(text).map(|steps| to_text(&steps));
    }
    if text.lines().nth(MAX_LINES).is_some() {
        return Err(EngineError::invalid(LineError::TooLong.message()));
    }
    let mut out = String::with_capacity(text.len());
    for (source, line) in text.lines().zip(parse_lines(text)) {
        match line.item {
            LineItem::Step { step, .. } => out.push_str(&step.to_json().to_string()),
            LineItem::Blank => {}
            LineItem::Comment => out.push_str(source.trim()),
            LineItem::Error(_) => out.push_str(source),
        }
        out.push('\n');
    }
    Ok(out)
}

/// Whether a step left an undo step that stepping back can undo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Undo {
    /// It left nothing to undo (a query, or an edit that changed nothing).
    Nothing,
    /// It left one step on top of the undo history: the history's depth
    /// below it and its identity ([`cc_core::Document::undo_top_id`]).
    Step { depth: usize, id: u64 },
    /// It changed the undo history otherwise (it undid, or replaced the
    /// document): it cannot be stepped back.
    Unknown,
}

/// The undo history's depth and top step.
fn history(session: &Session) -> (usize, Option<u64>) {
    (session.doc.undo_depth(), session.doc.undo_top_id())
}

/// What a step left, from the history before and after it ran.
fn undo_left(before: (usize, Option<u64>), after: (usize, Option<u64>)) -> Undo {
    match (after.0.checked_sub(before.0), after.1) {
        (Some(0), _) if after.1 == before.1 => Undo::Nothing,
        (Some(1), Some(id)) => Undo::Step { depth: before.0, id },
        _ => Undo::Unknown,
    }
}

/// What was run in one go and steps back at once: one step, or a run of
/// steps squashed into one undo step.
#[derive(Debug, Clone)]
struct Ran {
    steps: usize,
    undo: Undo,
}

/// Runs a script step by step (the macro editor's stepping): each step's
/// placeholders are resolved against the results so far, and each step run
/// is one undo step that [`Stepper::back`] undoes again, as long as nothing
/// else changed the model since.
#[derive(Debug, Clone, Default)]
pub struct Stepper {
    steps: Vec<Step>,
    results: Vec<Value>,
    ran: Vec<Ran>,
}

impl Stepper {
    pub fn new(steps: Vec<Step>) -> Self {
        Self { steps, results: Vec::new(), ran: Vec::new() }
    }

    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    /// How many steps have run (the index of the next one).
    pub fn position(&self) -> usize {
        self.results.len()
    }

    /// The results of the steps run, in order.
    pub fn results(&self) -> &[Value] {
        &self.results
    }

    pub fn into_results(self) -> Vec<Value> {
        self.results
    }

    /// Whether every step has run.
    pub fn is_done(&self) -> bool {
        self.position() >= self.steps.len()
    }

    /// Run the next step as one undo step; returns its result. A step that
    /// fails leaves the position where it is (it may be tried again).
    pub fn next(&mut self, session: &mut Session) -> std::result::Result<Value, StepError> {
        let before = history(session);
        let out = self.execute(session)?;
        self.ran.push(Ran { steps: 1, undo: undo_left(before, history(session)) });
        Ok(out)
    }

    /// Run the steps up to `count` steps from the start (at most all of
    /// them), one at a time, stopping at the first error.
    pub fn run_to(&mut self, session: &mut Session, count: usize) -> std::result::Result<(), StepError> {
        while self.position() < count.min(self.steps.len()) {
            self.next(session)?;
        }
        Ok(())
    }

    /// Run the remaining steps as a single undo step named `name`, which
    /// [`Stepper::back`] steps back whole. At an error it stops, and the
    /// steps that ran before it are that one undo step. If one of them
    /// changed the undo history otherwise (an `undo` command), they stay one
    /// step each.
    pub fn run_squashed(&mut self, session: &mut Session, name: &str) -> std::result::Result<(), StepError> {
        let start = self.position();
        let before = history(session);
        // The run is one step of its own: nothing earlier merges into it.
        session.doc.break_merge();
        let mut each = Vec::new();
        let mut outcome = Ok(());
        while !self.is_done() {
            let step_before = history(session);
            if let Err(e) = self.execute(session) {
                outcome = Err(e);
                break;
            }
            each.push(undo_left(step_before, history(session)));
        }
        if each.contains(&Undo::Unknown) {
            self.ran.extend(each.into_iter().map(|undo| Ran { steps: 1, undo }));
        } else if self.position() > start {
            let undo = if session.doc.squash_since(before.0, name) {
                undo_left(before, history(session))
            } else {
                Undo::Nothing
            };
            self.ran.push(Ran { steps: self.position() - start, undo });
        }
        outcome
    }

    /// Step back over the last step run (or squashed run): undo its undo
    /// step, which must still be the top of the undo history, unchanged. A
    /// step that left nothing to undo (a query) steps back without undoing
    /// anything. When something else changed the model since (or the step
    /// changed the undo history otherwise), it refuses and changes nothing.
    pub fn back(&mut self, session: &mut Session) -> Result<()> {
        let Some(last) = self.ran.last() else {
            return Err(EngineError::invalid(crate::t!("macro-step-at-start")));
        };
        match last.undo {
            Undo::Nothing => {}
            Undo::Unknown => return Err(EngineError::rejected(crate::t!("macro-step-not-undoable"))),
            Undo::Step { depth, id } => {
                if history(session) != (depth + 1, Some(id)) {
                    return Err(EngineError::rejected(crate::t!("macro-step-changed")));
                }
                session.undo_to(depth);
            }
        }
        let steps = last.steps;
        self.ran.pop();
        self.results.truncate(self.results.len().saturating_sub(steps));
        Ok(())
    }

    /// Step back to the start while that is possible: `Ok` with how many
    /// steps were stepped back, or `Err` with that count and why it stopped
    /// ([`Stepper::position`] tells where).
    pub fn restart(&mut self, session: &mut Session) -> std::result::Result<usize, (usize, EngineError)> {
        let start = self.position();
        while self.position() > 0 {
            if let Err(e) = self.back(session) {
                return Err((start - self.position(), e));
            }
        }
        Ok(start)
    }

    /// Run the next step and keep its result (no stepping-back record).
    fn execute(&mut self, session: &mut Session) -> std::result::Result<Value, StepError> {
        let index = self.position();
        let Some(step) = self.steps.get(index) else {
            return Err(StepError {
                index,
                command: String::new(),
                error: EngineError::invalid(crate::t!("macro-step-done")),
            });
        };
        let fail = |error| StepError { index, command: step.command.clone(), error };
        let args = active_level(session, &step.args).and_then(|l| resolve(&step.args, &self.results, l, 0));
        let out = session.execute(&step.command, args.map_err(fail)?).map_err(fail)?;
        self.results.push(out.clone());
        Ok(out)
    }
}

/// What a recorded step made: the ids allocated while it ran (ids are shared
/// by every kind of object, so none of them names an older object) and its
/// result.
#[derive(Debug, Clone)]
pub(crate) struct Made {
    pub ids: std::ops::Range<u64>,
    pub result: Value,
}

/// The level `{"$level": "active"}` stands for, when the arguments hold one:
/// the session's active level, else the lowest level.
fn active_level(session: &Session, args: &Value) -> Result<Option<LevelId>> {
    if !has_level_placeholder(args, 0) {
        return Ok(None);
    }
    match session.active_level() {
        Some(level) => Ok(Some(level)),
        None => crate::refs::level_or_first(session.project(), None).map(Some),
    }
}

fn has_level_placeholder(v: &Value, depth: usize) -> bool {
    if depth > MAX_DEPTH {
        return false;
    }
    match v {
        Value::Object(m) => m.contains_key(LEVEL_KEY) || m.values().any(|v| has_level_placeholder(v, depth + 1)),
        Value::Array(a) => a.iter().any(|v| has_level_placeholder(v, depth + 1)),
        _ => false,
    }
}

/// `args` with its placeholders replaced by the values they stand for.
fn resolve(args: &Value, results: &[Value], level: Option<LevelId>, depth: usize) -> Result<Value> {
    if depth > MAX_DEPTH {
        return Err(EngineError::invalid("script arguments are nested too deeply"));
    }
    match args {
        Value::Object(m) if m.contains_key(STEP_KEY) => step_value(m, results),
        Value::Object(m) if m.contains_key(LEVEL_KEY) => match (m.get(LEVEL_KEY).and_then(Value::as_str), level) {
            (Some("active"), Some(level)) => Ok(json!(level.0)),
            _ => Err(EngineError::invalid(format!("\"{LEVEL_KEY}\" must be \"active\""))),
        },
        Value::Object(m) => m
            .iter()
            .map(|(k, v)| Ok((k.clone(), resolve(v, results, level, depth + 1)?)))
            .collect::<Result<_>>()
            .map(Value::Object),
        Value::Array(a) => {
            a.iter().map(|v| resolve(v, results, level, depth + 1)).collect::<Result<_>>().map(Value::Array)
        }
        v => Ok(v.clone()),
    }
}

/// The value `{"$step": n, "path": …}` refers to.
fn step_value(m: &Map<String, Value>, results: &[Value]) -> Result<Value> {
    let n = m.get(STEP_KEY).and_then(Value::as_u64).unwrap_or(0);
    let step = usize::try_from(n)
        .ok()
        .and_then(|n| n.checked_sub(1))
        .and_then(|i| results.get(i))
        .ok_or_else(|| EngineError::invalid(format!("\"{STEP_KEY}\": {n} is not an earlier step")))?;
    let path = match m.get("path") {
        None => "/id",
        Some(Value::String(p)) => p.as_str(),
        Some(_) => return Err(EngineError::invalid("\"path\" must be a JSON pointer such as \"/id\"")),
    };
    step.pointer(path)
        .filter(|v| !v.is_null())
        .cloned()
        .ok_or_else(|| EngineError::invalid(format!("step {n} has no result at \"{path}\"")))
}

/// Replace the ids that earlier steps created by references to their results
/// (`made[i]` is what `steps[i]` made). Integers at keys of counts and
/// indices, and ids no earlier result holds, stay as they are.
pub(crate) fn link(steps: &mut [Step], made: &[Option<Made>]) {
    for (i, step) in steps.iter_mut().enumerate() {
        let earlier = made.get(..i).unwrap_or_default();
        link_value(&mut step.args, None, earlier, 0);
    }
}

fn link_value(v: &mut Value, key: Option<&str>, earlier: &[Option<Made>], depth: usize) {
    if depth > MAX_DEPTH || key.is_some_and(|k| NOT_IDS.contains(&k)) {
        return;
    }
    match v {
        Value::Object(m) if m.contains_key(STEP_KEY) || m.contains_key(LEVEL_KEY) => {}
        Value::Object(m) => {
            for (k, v) in m.iter_mut() {
                link_value(v, Some(k), earlier, depth + 1);
            }
        }
        Value::Array(a) => {
            for v in a {
                link_value(v, key, earlier, depth + 1);
            }
        }
        Value::Number(n) => {
            let Some(id) = n.as_u64() else { return };
            // The latest step that made the id and shows it in its result.
            let found = earlier.iter().enumerate().rev().find_map(|(j, made)| {
                let made = made.as_ref().filter(|m| m.ids.contains(&id))?;
                Some((j, find_id(&made.result, id)?))
            });
            if let Some((j, path)) = found {
                *v = if path == "/id" { json!({ STEP_KEY: j + 1 }) } else { json!({ STEP_KEY: j + 1, "path": path }) };
            }
        }
        _ => {}
    }
}

/// The JSON pointer of the first place `result` holds `id` (`/id` first).
fn find_id(result: &Value, id: u64) -> Option<String> {
    if result.get("id").and_then(Value::as_u64) == Some(id) {
        return Some("/id".into());
    }
    let mut budget = MAX_SEARCH;
    let mut path = String::new();
    find_in(result, id, &mut path, &mut budget, 0).then_some(path)
}

fn find_in(v: &Value, id: u64, path: &mut String, budget: &mut usize, depth: usize) -> bool {
    if depth > MAX_DEPTH || *budget == 0 {
        return false;
    }
    *budget -= 1;
    let len = path.len();
    let found = match v {
        Value::Number(n) => n.as_u64() == Some(id),
        Value::Object(m) => m.iter().any(|(k, v)| {
            path.truncate(len);
            path.push('/');
            path.push_str(&k.replace('~', "~0").replace('/', "~1"));
            find_in(v, id, path, budget, depth + 1)
        }),
        Value::Array(a) => a.iter().enumerate().any(|(i, v)| {
            path.truncate(len);
            path.push('/');
            path.push_str(&i.to_string());
            find_in(v, id, path, budget, depth + 1)
        }),
        _ => false,
    };
    if !found {
        path.truncate(len);
    }
    found
}

/// Replace the references to `level` (by id or name) at `level` keys by the
/// active level placeholder, so the step plays on the level active then.
pub(crate) fn mark_level(v: &mut Value, level: LevelId, name: &str, depth: usize) {
    if depth > MAX_DEPTH {
        return;
    }
    match v {
        Value::Object(m) if m.contains_key(STEP_KEY) || m.contains_key(LEVEL_KEY) => {}
        Value::Object(m) => {
            for (k, v) in m.iter_mut() {
                let is_level = match &*v {
                    Value::Number(n) => n.as_u64() == Some(level.0),
                    Value::String(s) => s.trim().eq_ignore_ascii_case(name.trim()),
                    _ => false,
                };
                if k == "level" && is_level {
                    *v = json!({ LEVEL_KEY: "active" });
                } else {
                    mark_level(v, level, name, depth + 1);
                }
            }
        }
        Value::Array(a) => {
            for v in a {
                mark_level(v, level, name, depth + 1);
            }
        }
        _ => {}
    }
}
