//! The macro editor: a panel under the views (like the graph editor, and
//! never at the same time) where a macro is written, checked and run step
//! by step, so each step's change is seen in the views above.
//!
//! The text is read by `cc_engine::script::parse_lines` when it changes (the
//! gutter's markers) and checked by `script::check` a moment after the last
//! keystroke; stepping goes through one `script::Stepper`, whose steps follow
//! the text as long as the steps that ran are unchanged. Each step is one
//! undo step of the model, as when a macro runs.

use crate::icons::{self, Icon, Tones};
use crate::state::AppState;
use crate::t;
use crate::theme::{self, PALETTE};
use cc_engine::script::{self, Diagnostic, LineItem, ScriptLine, Severity, Step, Stepper};
use egui::text::{CCursor, CCursorRange, LayoutJob, TextFormat};
use egui::{Align, Color32, FontId, Key, Modifiers, Rect, RichText, Sense, Stroke, Ui, pos2, vec2};
use serde_json::Value;
use std::path::PathBuf;

/// The text field's id (its focus decides where Ctrl+Z goes).
pub fn editor_id() -> egui::Id {
    egui::Id::new("macro-editor-text")
}

/// Seconds after the last keystroke before the text is checked.
const CHECK_DELAY: f64 = 0.4;
/// Size of the editor's text.
const FONT_SIZE: f32 = 13.0;

/// What the Result pane shows.
#[derive(Debug, Clone, PartialEq)]
enum Outcome {
    /// The last step run (from 1): its result as pretty JSON, or why it failed.
    Step { number: usize, command: String, result: Result<String, String> },
    /// A message about stepping (why it could not step back, how far a restart got).
    Message(String),
}

/// An action that waits for the user's confirmation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pending {
    New,
    Open,
    /// The restart stopped: start over without undoing the rest?
    Restart,
}

/// The macro editor's state (`AppState::macro_editor`).
#[derive(Default)]
pub struct MacroEditor {
    pub open: bool,
    pub text: String,
    pub path: Option<PathBuf>,
    /// Changed since it was opened or saved.
    pub dirty: bool,
    /// Whether the pointer is over the panel: its keys go to the macro.
    pub hovered: bool,
    /// Run to End makes one undo step.
    pub single_undo: bool,
    /// The text read line by line (see `script::parse_lines`).
    lines: Vec<ScriptLine>,
    /// The text is a JSON array (no per-line steps).
    array: bool,
    /// The line (from 1) of each step, by its number less one.
    step_lines: Vec<Option<usize>>,
    /// The last check's problems; `None` before any check.
    diagnostics: Option<Vec<Diagnostic>>,
    /// When the text changed last, while it waits to be checked.
    edited: Option<f64>,
    stepper: Stepper,
    /// The steps that ran were edited: only Restart steps on.
    stale: bool,
    last: Option<Outcome>,
    /// The line (from 0) the text cursor is on.
    cursor_line: usize,
    /// The cursor's character index when `cursor_line` was worked out.
    cursor_index: Option<usize>,
    /// Move the cursor to this line (from 1) and show it.
    goto: Option<usize>,
    confirm: Option<Pending>,
    /// Forget the text field's own undo (a new text).
    reset_undo: bool,
}

impl MacroEditor {
    /// The steps of the text as read now.
    fn steps(&self) -> Vec<Step> {
        self.lines
            .iter()
            .filter_map(|l| match &l.item {
                LineItem::Step { step, .. } => Some(step.clone()),
                _ => None,
            })
            .collect()
    }

    /// How many steps have run.
    pub fn position(&self) -> usize {
        self.stepper.position()
    }

    /// How many steps the text has.
    pub fn total(&self) -> usize {
        self.step_lines.len()
    }

    /// Read the text again after it changed at time `now` (it is checked
    /// [`CHECK_DELAY`] later), and let the stepper follow it.
    fn changed(&mut self, now: f64) {
        self.dirty = true;
        self.edited = Some(now);
        self.reparse();
    }

    fn reparse(&mut self) {
        self.lines = script::parse_lines(&self.text);
        self.array = self.text.trim_start().starts_with('[');
        self.step_lines = self
            .lines
            .iter()
            .filter_map(|l| matches!(l.item, LineItem::Step { .. }).then_some(l.line))
            .map(|line| if self.array { None } else { line })
            .collect();
        self.stale = !self.stepper.set_steps(self.steps());
    }

    /// Replace the whole text (opened, new, formatted).
    fn set_text(&mut self, text: String) {
        self.text = text;
        self.cursor_index = None;
        self.reparse();
    }

    /// Check the text now when it changed since the last check.
    fn check_if_edited(&mut self, session: &cc_engine::Session) {
        if self.edited.is_some() || self.diagnostics.is_none() {
            self.check(session);
        }
    }

    fn check(&mut self, session: &cc_engine::Session) {
        self.edited = None;
        self.diagnostics = Some(script::check(session, &self.text));
    }

    /// Forget the run (another project was opened): the text stays.
    pub fn reset_run(&mut self) {
        self.stepper = Stepper::new(self.steps());
        self.stale = false;
        self.last = None;
        if self.confirm == Some(Pending::Restart) {
            self.confirm = None;
        }
    }

    /// The line (from 1) of step `number` (from 1).
    fn line_of(&self, number: usize) -> Option<usize> {
        number.checked_sub(1).and_then(|i| self.step_lines.get(i).copied().flatten())
    }

    /// Whether an error stands in the way of running up to step `target`
    /// (from 1): a problem on a line up to that step's, or, in the array
    /// format, in a step up to it or in no step. Running to the end, any error
    /// (a line after the last step that is no step was surely meant to be one).
    fn blocked(&self, target: usize, to_end: bool) -> bool {
        let Some(diagnostics) = &self.diagnostics else { return false };
        let limit = if to_end { Some(usize::MAX) } else { self.line_of(target) };
        diagnostics.iter().filter(|d| d.severity == Severity::Error).any(|d| match (d.line, limit, d.step) {
            (_, _, Some(step)) if self.array => step <= target,
            (Some(line), Some(limit), _) => line <= limit,
            _ => true,
        })
    }

    /// The step count Run to Cursor runs to: the steps before the first one
    /// at or after the cursor's line.
    fn cursor_target(&self) -> usize {
        let cursor = self.cursor_line + 1;
        self.step_lines.iter().position(|l| l.is_some_and(|l| l >= cursor)).unwrap_or(self.step_lines.len())
    }

    /// Show the result of the last step that ran, if any.
    fn show_last_result(&mut self) {
        let position = self.stepper.position();
        self.last = position.checked_sub(1).and_then(|i| {
            let result = self.stepper.results().get(i)?;
            let command = self.stepper.steps().get(i)?.command.clone();
            Some(Outcome::Step { number: i + 1, command, result: Ok(pretty(result)) })
        });
    }
}

fn pretty(v: &Value) -> String {
    serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string())
}

/// Open or close the panel; opening it hides the graph editor (which keeps its graph).
pub fn toggle(state: &mut AppState) {
    let e = &mut state.macro_editor;
    e.open = !e.open;
    if e.open {
        state.graph.open = false;
        if e.lines.is_empty() && !e.text.is_empty() {
            e.reparse();
        }
    }
}

// --- Running ---

/// Why the steps cannot run (or step back) now, if they cannot.
fn why_busy(state: &AppState) -> Option<String> {
    if state.session.is_recording() {
        return Some(t!("why-recording"));
    }
    let busy = state.sketch.is_some()
        || state.component_env.is_some()
        || crate::profile_sketch::is_open(state)
        || crate::jobs::busy(state)
        || state.dialogs.edit_start.is_some();
    busy.then(|| t!("macro-ed-why-busy"))
}

/// What a stepping button does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Step,
    StepBack,
    RunToCursor,
    RunToEnd,
    Restart,
}

impl Action {
    fn label(self) -> String {
        match self {
            Action::Step => t!("macro-ed-step"),
            Action::StepBack => t!("macro-ed-step-back"),
            Action::RunToCursor => t!("macro-ed-run-cursor"),
            Action::RunToEnd => t!("macro-ed-run-end"),
            Action::Restart => t!("macro-ed-restart"),
        }
    }

    fn keys(self) -> &'static str {
        match self {
            Action::Step => "F10",
            Action::StepBack => "Shift+F10",
            Action::RunToCursor => "Ctrl+F10",
            Action::RunToEnd => "F5",
            Action::Restart => "Ctrl+Shift+F5",
        }
    }

    fn icon(self) -> Icon {
        match self {
            Action::Step => Icon::MacroStep,
            Action::StepBack => Icon::MacroStepBack,
            Action::RunToCursor => Icon::MacroRunCursor,
            Action::RunToEnd => Icon::MacroRunEnd,
            Action::Restart => Icon::MacroRestart,
        }
    }

    /// Why it cannot be done now.
    fn unavailable(self, state: &AppState) -> Option<String> {
        if let Some(why) = why_busy(state) {
            return Some(why);
        }
        let e = &state.macro_editor;
        if self == Action::Restart {
            return (e.position() == 0).then(|| t!("macro-step-at-start"));
        }
        if e.stale {
            return Some(t!("macro-ed-stale"));
        }
        if self == Action::StepBack {
            return (e.position() == 0).then(|| t!("macro-step-at-start"));
        }
        if e.total() == 0 {
            return Some(t!("macro-ed-why-empty"));
        }
        let target = match self {
            Action::Step => e.position() + 1,
            Action::RunToCursor if e.array => return Some(t!("macro-ed-array-hint")),
            Action::RunToCursor => e.cursor_target(),
            _ => e.total(),
        };
        if e.position() >= e.total() {
            return Some(t!("macro-step-done"));
        }
        if target <= e.position() {
            return Some(t!("macro-ed-why-cursor"));
        }
        e.blocked(target, self == Action::RunToEnd).then(|| t!("macro-ed-why-errors"))
    }
}

/// Do a stepping action, unless it is unavailable (then say why in the Result pane).
pub fn act(state: &mut AppState, action: Action) {
    // The buttons follow a check made a moment after typing: check now.
    state.macro_editor.check_if_edited(&state.session);
    if let Some(why) = action.unavailable(state) {
        state.macro_editor.last = Some(Outcome::Message(why));
        return;
    }
    // Steps written on the active level play on the level active now.
    state.session.set_active_level(state.active_level());
    let AppState { macro_editor: e, session, .. } = state;
    let outcome = match action {
        Action::Step => e.stepper.next(session).map(|_| ()),
        Action::RunToCursor => {
            let target = e.cursor_target();
            e.stepper.run_to(session, target)
        }
        Action::RunToEnd if e.single_undo => e.stepper.run_squashed(session, &t!("macro-ed-run-name")),
        Action::RunToEnd => {
            let end = e.stepper.steps().len();
            e.stepper.run_to(session, end)
        }
        Action::StepBack => {
            match e.stepper.back(session) {
                Ok(()) => e.show_last_result(),
                Err(err) => e.last = Some(Outcome::Message(err.message)),
            }
            return;
        }
        Action::Restart => {
            match e.stepper.restart(session) {
                Ok(_) => {
                    e.reset_run();
                }
                Err((count, err)) => {
                    e.last = Some(Outcome::Message(
                        t!("macro-ed-restart-partial", "count" => count, "error" => err.message),
                    ));
                    e.confirm = Some(Pending::Restart);
                }
            }
            return;
        }
    };
    match outcome {
        Ok(()) => e.show_last_result(),
        Err(failed) => {
            e.last = Some(Outcome::Step {
                number: failed.index + 1,
                command: failed.command,
                result: Err(failed.error.message),
            });
        }
    }
}

// --- Editing ---

/// Insert a step as a new line after the cursor's line (the command
/// palette's *Insert* in the editor's mode).
pub fn insert_step(state: &mut AppState, command: &str, args: Value, now: f64) {
    let e = &mut state.macro_editor;
    let line = Step { command: command.to_string(), args }.to_json().to_string();
    let mut lines: Vec<&str> = if e.text.is_empty() { Vec::new() } else { e.text.split('\n').collect() };
    // A text ending with a newline has an empty last piece, which stays last.
    let ends_with_newline = lines.last() == Some(&"");
    if ends_with_newline {
        lines.pop();
    }
    let at = (e.cursor_line + 1).min(lines.len());
    lines.insert(at, &line);
    let mut text = lines.join("\n");
    text.push('\n');
    e.set_text(text);
    e.changed(now);
    e.goto = Some(at + 1);
    e.open = true;
}

fn format(state: &mut AppState, now: f64) {
    let e = &mut state.macro_editor;
    match script::format(&e.text) {
        Ok(text) if text != e.text => {
            e.set_text(text);
            e.changed(now);
        }
        Ok(_) => {}
        Err(err) => state.message = t!("macro-ed-format-failed", "error" => err.message),
    }
}

fn new_macro(state: &mut AppState) {
    let e = &mut state.macro_editor;
    e.set_text(String::new());
    e.path = None;
    e.dirty = false;
    e.diagnostics = None;
    e.edited = None;
    clear_text_undo(state);
}

fn open_macro(state: &mut AppState) {
    let Some(path) = crate::actions::macro_dialog(state).pick_file() else { return };
    open_path(state, path);
}

/// Open the macro at `path` in the editor.
pub fn open_path(state: &mut AppState, path: PathBuf) {
    match cc_core::read_to_string_limited(&path, crate::actions::MAX_MACRO_BYTES) {
        Ok(text) => {
            let e = &mut state.macro_editor;
            e.set_text(text);
            e.path = Some(path);
            e.dirty = false;
            e.open = true;
            e.check(&state.session);
            state.graph.open = false;
            clear_text_undo(state);
        }
        Err(err) => state.dialogs.error = Some(t!("macro-ed-open-failed", "error" => err)),
    }
}

/// The text field's own undo starts again with a new text.
fn clear_text_undo(state: &mut AppState) {
    state.macro_editor.reset_undo = true;
}

fn save(state: &mut AppState, save_as: bool) -> bool {
    let path = match (&state.macro_editor.path, save_as) {
        (Some(p), false) => p.clone(),
        _ => {
            let Some(mut p) = crate::actions::macro_dialog(state).set_file_name("macro.ccmacro").save_file() else {
                return false;
            };
            if p.extension().is_none() {
                p.set_extension("ccmacro");
            }
            p
        }
    };
    match std::fs::write(&path, &state.macro_editor.text) {
        Ok(()) => {
            state.macro_editor.path = Some(path);
            state.macro_editor.dirty = false;
            true
        }
        Err(err) => {
            state.dialogs.error = Some(t!("macro-ed-save-failed", "error" => t!("err-io", "detail" => err)));
            false
        }
    }
}

/// The editor's keys, while the pointer is over the panel or its text has
/// focus; returns whether it took Save (Ctrl+S, Ctrl+Shift+S).
pub fn keys(ctx: &egui::Context, state: &mut AppState, save_key: bool, save_as_key: bool) -> bool {
    let e = &state.macro_editor;
    let focused = ctx.memory(|m| m.has_focus(editor_id()));
    if !e.open || !(e.hovered || focused) {
        return false;
    }
    // The more specific combinations first: a key with no modifiers also
    // matches it pressed with Shift.
    let (restart, end, back, cursor, step) = ctx.input_mut(|i| {
        let restart = i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::F5);
        let end = i.consume_key(Modifiers::NONE, Key::F5);
        let back = i.consume_key(Modifiers::SHIFT, Key::F10);
        let cursor = i.consume_key(Modifiers::COMMAND, Key::F10);
        (restart, end, back, cursor, i.consume_key(Modifiers::NONE, Key::F10))
    });
    for (pressed, action) in [
        (restart, Action::Restart),
        (end, Action::RunToEnd),
        (back, Action::StepBack),
        (cursor, Action::RunToCursor),
        (step, Action::Step),
    ] {
        if pressed {
            act(state, action);
        }
    }
    if save_key || save_as_key {
        save(state, save_as_key);
        return true;
    }
    false
}

// --- The panel ---

/// The panel: toolbar, editor, and the Result and Problems panes.
pub fn show(ui: &mut Ui, state: &mut AppState) {
    state.macro_editor.hovered = ui.rect_contains_pointer(ui.max_rect());
    let now = ui.input(|i| i.time);
    // The check a moment after the last keystroke.
    match state.macro_editor.edited {
        Some(at) if now - at >= CHECK_DELAY => state.macro_editor.check(&state.session),
        Some(at) => ui.ctx().request_repaint_after(std::time::Duration::from_secs_f64(CHECK_DELAY - (now - at))),
        None => {}
    }
    toolbar(ui, state, now);
    ui.separator();
    let width = ui.available_width();
    egui::Panel::right("macro-result")
        .resizable(true)
        .default_size((width * 0.35).max(240.0))
        .size_range(200.0..=(width * 0.7).max(240.0))
        .frame(egui::Frame::new().fill(PALETTE.panel).inner_margin(egui::Margin { left: 8, ..Default::default() }))
        .show(ui, |ui| side(ui, state));
    egui::CentralPanel::no_frame().show(ui, |ui| {
        let e = &state.macro_editor;
        if e.stale {
            notice(ui, &t!("macro-ed-stale"), PALETTE.warning);
        }
        if e.array {
            notice(ui, &t!("macro-ed-array-hint"), PALETTE.text_muted);
        }
        editor(ui, state, now);
    });
}

/// A thin row with a message over the editor.
fn notice(ui: &mut Ui, text: &str, color: Color32) {
    egui::Frame::new().fill(PALETTE.surface).inner_margin(egui::Margin::symmetric(8, 3)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(RichText::new(text).size(12.0).color(color));
    });
}

/// A square icon button of the toolbar, named for assistive technology,
/// with its shortcut and why it is unavailable in the tooltip.
fn tool_button(ui: &mut Ui, icon: Icon, label: &str, keys: Option<&str>, why: Option<String>) -> egui::Response {
    let enabled = why.is_none();
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, response) = ui.allocate_exact_size(vec2(26.0, 26.0), sense);
    if enabled && response.hovered() {
        ui.painter().rect_filled(rect, theme::RADIUS as f32, PALETTE.hover);
    }
    let tones = Tones::of(theme::palette(ui.ctx()), None, enabled, false);
    icons::paint_toned(ui.painter(), rect.shrink(5.0), icon, tones, None);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, label));
    response.on_hover_ui(|ui| {
        ui.set_max_width(300.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new(label).family(theme::semibold()).color(PALETTE.text));
            if let Some(keys) = keys {
                crate::ribbon::key_cap(ui, keys);
            }
        });
        if let Some(why) = &why {
            ui.label(RichText::new(t!("tip-unavailable", "reason" => why)).color(PALETTE.warning));
        }
    })
}

fn toolbar(ui: &mut Ui, state: &mut AppState, now: f64) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        let e = &state.macro_editor;
        let name = e
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| t!("macro-ed-untitled"));
        let dirty = if e.dirty { " •" } else { "" };
        ui.label(theme::title(format!("{name}{dirty}")));
        ui.add_space(6.0);
        if tool_button(ui, Icon::New, &t!("action-new"), None, None).clicked() {
            if state.macro_editor.dirty {
                state.macro_editor.confirm = Some(Pending::New);
            } else {
                new_macro(state);
            }
        }
        if tool_button(ui, Icon::Open, &t!("action-open"), None, None).clicked() {
            if state.macro_editor.dirty {
                state.macro_editor.confirm = Some(Pending::Open);
            } else {
                open_macro(state);
            }
        }
        if tool_button(ui, Icon::Save, &t!("action-save"), Some("Ctrl+S"), None).clicked() {
            save(state, false);
        }
        if tool_button(ui, Icon::SaveAs, &t!("action-save-as"), Some("Ctrl+Shift+S"), None).clicked() {
            save(state, true);
        }
        ui.separator();
        for action in [Action::Step, Action::StepBack, Action::RunToCursor, Action::RunToEnd, Action::Restart] {
            let why = action.unavailable(state);
            if tool_button(ui, action.icon(), &action.label(), Some(action.keys()), why).clicked() {
                act(state, action);
            }
        }
        ui.add_space(4.0);
        ui.checkbox(&mut state.macro_editor.single_undo, t!("macro-ed-single-undo"))
            .on_hover_text(t!("macro-ed-single-undo-hint"));
        ui.separator();
        if tool_button(ui, Icon::MacroCheck, &t!("macro-ed-check"), None, None).clicked() {
            state.macro_editor.check(&state.session);
        }
        if tool_button(ui, Icon::MacroFormat, &t!("macro-ed-format"), None, None).clicked() {
            format(state, now);
        }
        if tool_button(ui, Icon::Plus, &t!("macro-ed-insert"), None, None).clicked() {
            crate::palette::open_insert(state);
        }
        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
            if tool_button(ui, Icon::Close, &t!("action-close"), None, None).clicked() {
                toggle(state);
            }
            let e = &state.macro_editor;
            let position = t!("macro-ed-position", "n" => e.position(), "total" => e.total());
            ui.label(theme::muted(position).size(12.0));
        });
    });
}

/// The Result and Problems panes.
fn side(ui: &mut Ui, state: &mut AppState) {
    let height = ui.available_height();
    ui.label(theme::title(t!("macro-ed-result")));
    let e = &mut state.macro_editor;
    egui::ScrollArea::both()
        .id_salt("macro-result")
        .max_height((height * 0.6 - 24.0).max(40.0))
        .auto_shrink([false, true])
        .show(ui, |ui| match &e.last {
            Some(Outcome::Step { number, command, result }) => {
                let heading = t!("macro-ed-result-step", "n" => number, "command" => command);
                ui.label(RichText::new(heading).size(12.0).color(PALETTE.text_muted));
                match result {
                    Ok(json) => {
                        ui.add(egui::Label::new(RichText::new(json).monospace().size(12.0)).extend());
                    }
                    Err(message) => {
                        ui.label(RichText::new(message).color(PALETTE.danger));
                    }
                }
            }
            Some(Outcome::Message(message)) => {
                ui.label(RichText::new(message).color(PALETTE.danger));
            }
            None => {}
        });
    ui.separator();
    ui.label(theme::title(t!("macro-ed-problems")));
    let mut goto = None;
    egui::ScrollArea::vertical().id_salt("macro-problems").auto_shrink([false, false]).show(ui, |ui| {
        let Some(diagnostics) = &e.diagnostics else { return };
        if diagnostics.is_empty() {
            ui.label(theme::muted(t!("macro-ed-no-problems")));
        }
        for d in diagnostics {
            let text = match (d.line, d.step) {
                (Some(line), _) => t!("macro-ed-problem", "line" => line, "message" => d.message),
                (None, Some(step)) => t!("macro-ed-problem-step", "step" => step, "message" => d.message),
                (None, None) => d.message.clone(),
            };
            let color = severity_color(d.severity);
            let r = ui
                .horizontal(|ui| {
                    let (dot, _) = ui.allocate_exact_size(vec2(10.0, 14.0), Sense::hover());
                    ui.painter().circle_filled(dot.center(), 3.5, color);
                    ui.add(egui::Label::new(RichText::new(&text).size(12.0).color(PALETTE.text)).sense(Sense::click()))
                })
                .inner;
            if r.clicked()
                && let Some(line) = d.line
            {
                goto = Some(line);
            }
            r.on_hover_cursor(egui::CursorIcon::PointingHand);
        }
    });
    if goto.is_some() {
        e.goto = goto;
    }
}

fn severity_color(severity: Severity) -> Color32 {
    match severity {
        Severity::Error => PALETTE.danger,
        Severity::Warning => PALETTE.warning,
    }
}

/// How each line of the text is drawn.
#[derive(Clone, Copy, Default, PartialEq)]
struct LineLook {
    comment: bool,
    /// The worst problem on the line.
    problem: Option<Severity>,
}

/// The looks of the text's lines (indexed from 0).
fn line_looks(e: &MacroEditor) -> Vec<LineLook> {
    let count = e.text.split('\n').count();
    let mut looks = vec![LineLook::default(); count];
    if !e.array {
        for l in &e.lines {
            if let (Some(line), LineItem::Comment) = (l.line, &l.item)
                && let Some(look) = looks.get_mut(line - 1)
            {
                look.comment = true;
            }
        }
    }
    for d in e.diagnostics.iter().flatten() {
        if let Some(look) = d.line.and_then(|l| l.checked_sub(1)).and_then(|i| looks.get_mut(i)) {
            look.problem = match (look.problem, d.severity) {
                (Some(Severity::Error), _) | (_, Severity::Error) => Some(Severity::Error),
                _ => Some(Severity::Warning),
            };
        }
    }
    looks
}

/// The text laid out with its colors: comments muted, the command named
/// emphasized, placeholders in the accent color, lines with a problem underlined.
fn highlight(text: &str, looks: &[LineLook]) -> LayoutJob {
    let font = FontId::monospace(FONT_SIZE);
    let plain = |color: Color32, problem: Option<Severity>| TextFormat {
        font_id: font.clone(),
        color,
        underline: problem.map_or(Stroke::NONE, |s| Stroke::new(1.0, severity_color(s))),
        ..Default::default()
    };
    let mut job = LayoutJob::default();
    job.wrap.max_width = f32::INFINITY;
    for (i, line) in text.split_inclusive('\n').enumerate() {
        let look = looks.get(i).copied().unwrap_or_default();
        let (body, newline) = match line.strip_suffix('\n') {
            Some(body) => (body, "\n"),
            None => (line, ""),
        };
        if look.comment {
            job.append(body, 0.0, plain(PALETTE.text_faint, look.problem));
        } else {
            for (piece, kind) in tokens(body) {
                let color = match kind {
                    Token::Command => PALETTE.syntax_command,
                    Token::Placeholder => PALETTE.accent,
                    Token::Other => PALETTE.text,
                };
                job.append(piece, 0.0, plain(color, look.problem));
            }
        }
        job.append(newline, 0.0, plain(PALETTE.text, None));
    }
    job
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Token {
    /// The value of a `"command"` key.
    Command,
    /// A `"$step"` or `"$level"` key.
    Placeholder,
    Other,
}

/// A line cut in pieces for its colors: JSON strings found by their quotes
/// (with escapes), the string after a `"command"` key, placeholder keys.
fn tokens(line: &str) -> Vec<(&str, Token)> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut after_command = false;
    let mut chars = line.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c != '"' {
            continue;
        }
        // The string from this quote to the closing one.
        let mut end = line.len();
        let mut escaped = false;
        for (j, d) in chars.by_ref() {
            if escaped {
                escaped = false;
            } else if d == '\\' {
                escaped = true;
            } else if d == '"' {
                end = j + 1;
                break;
            }
        }
        let string = line.get(i..end).unwrap_or_default();
        let inner = string.trim_matches('"');
        let rest = line.get(end..).unwrap_or_default().trim_start();
        let is_key = rest.starts_with(':');
        let kind = if after_command && !is_key {
            Token::Command
        } else if is_key && (inner == script::STEP_KEY || inner == script::LEVEL_KEY) {
            Token::Placeholder
        } else {
            Token::Other
        };
        after_command = is_key && inner == "command";
        if kind != Token::Other {
            out.push((line.get(start..i).unwrap_or_default(), Token::Other));
            out.push((string, kind));
            start = end;
        }
    }
    out.push((line.get(start..).unwrap_or_default(), Token::Other));
    out.retain(|(s, _)| !s.is_empty());
    out
}

/// The character index where line `line` (from 1) starts.
fn line_start(text: &str, line: usize) -> usize {
    text.split('\n').take(line.saturating_sub(1)).map(|l| l.chars().count() + 1).sum()
}

/// The editor: a gutter and the text, which scrolls both ways.
fn editor(ui: &mut Ui, state: &mut AppState, now: f64) {
    let area = ui.available_rect_before_wrap();
    let e = &mut state.macro_editor;
    let line_count = e.text.split('\n').count().max(1);
    let digit = ui.ctx().fonts_mut(|f| f.glyph_width(&FontId::monospace(FONT_SIZE - 1.0), '0'));
    let digits = line_count.to_string().len().max(2) as f32;
    let gutter_w = 14.0 + digits * digit + 18.0;
    let gutter = Rect::from_min_size(area.min, vec2(gutter_w, area.height()));
    let text_rect = Rect::from_min_max(pos2(gutter.right(), area.top()), area.max);
    ui.painter().rect_filled(text_rect, 0.0, PALETTE.canvas);
    ui.painter().rect_filled(gutter, 0.0, PALETTE.surface);
    ui.painter().vline(gutter.right(), gutter.y_range(), Stroke::new(1.0, PALETTE.border));

    // A jump to a line (a problem clicked, a command inserted): move the cursor there.
    let goto = e.goto.take();
    if goto.is_some() || e.reset_undo {
        let mut text_state = egui::text_edit::TextEditState::load(ui.ctx(), editor_id()).unwrap_or_default();
        if let Some(line) = goto {
            let index = line_start(&e.text, line);
            text_state.cursor.set_char_range(Some(CCursorRange::one(CCursor::new(index))));
            ui.memory_mut(|m| m.request_focus(editor_id()));
        }
        if std::mem::take(&mut e.reset_undo) {
            text_state.clear_undoer();
        }
        text_state.store(ui.ctx(), editor_id());
    }

    let looks = line_looks(e);
    let next_line = if e.stale { None } else { e.line_of(e.position() + 1) };
    let mut layouter = |ui: &Ui, text: &dyn egui::TextBuffer, _wrap: f32| {
        ui.ctx().fonts_mut(|f| f.layout_job(highlight(text.as_str(), &looks)))
    };
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(text_rect));
    let scroll = egui::ScrollArea::both().id_salt("macro-text").auto_shrink([false, false]).show(&mut child, |ui| {
        // The next step's line, behind the text.
        let background = ui.painter().add(egui::Shape::Noop);
        let size = vec2(text_rect.width() - 12.0, text_rect.height() - 12.0).max(vec2(40.0, 20.0));
        let output = egui::TextEdit::multiline(&mut e.text)
            .id(editor_id())
            .code_editor()
            .frame(egui::Frame::NONE)
            .margin(egui::Margin::symmetric(6, 4))
            .desired_width(size.x)
            .min_size(size)
            .lock_focus(true)
            .layouter(&mut layouter)
            .show(ui);
        let rows: Vec<Rect> =
            output.galley.rows.iter().map(|r| r.rect().translate(output.galley_pos.to_vec2())).collect();
        if let Some(row) = next_line.and_then(|l| rows.get(l - 1)) {
            let clip = ui.clip_rect();
            let band = Rect::from_x_y_ranges(clip.x_range(), row.y_range());
            ui.painter().set(background, egui::Shape::rect_filled(band, 0.0, PALETTE.accent_soft));
        }
        if let Some(row) = goto.and_then(|l| rows.get(l - 1)) {
            ui.scroll_to_rect(*row, Some(Align::Center));
        }
        (output.response.response.changed(), output.cursor_range, rows)
    });
    let (changed, cursor, rows) = scroll.inner;
    if changed {
        e.changed(now);
    }
    if let Some(range) = cursor {
        let index = range.primary.index.0;
        if e.cursor_index != Some(index) {
            e.cursor_index = Some(index);
            e.cursor_line = e.text.chars().take(index).filter(|c| *c == '\n').count();
        }
    }
    gutter_marks(ui, state, gutter, &rows);
    ui.advance_cursor_after_rect(area);
}

/// The gutter: line numbers, a check by the steps that ran, an arrow by the
/// next step, and a dot by lines with a problem (its messages on hover).
fn gutter_marks(ui: &mut Ui, state: &AppState, gutter: Rect, rows: &[Rect]) {
    let e = &state.macro_editor;
    let painter = ui.painter().with_clip_rect(gutter);
    let font = FontId::monospace(FONT_SIZE - 1.0);
    let position = e.position();
    let mut ran = std::collections::HashSet::new();
    if !e.stale {
        ran.extend(e.step_lines.iter().take(position).flatten().copied());
    }
    let next = if e.stale { None } else { e.line_of(position + 1) };
    for (i, row) in rows.iter().enumerate() {
        if row.bottom() < gutter.top() || row.top() > gutter.bottom() {
            continue;
        }
        let line = i + 1;
        let y = row.center().y;
        painter.text(
            pos2(gutter.right() - 18.0, y),
            egui::Align2::RIGHT_CENTER,
            line.to_string(),
            font.clone(),
            if next == Some(line) { PALETTE.accent_text } else { PALETTE.text_faint },
        );
        let mark = pos2(gutter.left() + 8.0, y);
        if next == Some(line) {
            let arrow = vec![mark + vec2(-3.5, -4.5), mark + vec2(4.0, 0.0), mark + vec2(-3.5, 4.5)];
            painter.add(egui::Shape::convex_polygon(arrow, PALETTE.accent, Stroke::NONE));
        } else if ran.contains(&line) {
            let stroke = Stroke::new(1.5, PALETTE.success);
            painter.line_segment([mark + vec2(-3.5, 0.0), mark + vec2(-1.0, 3.0)], stroke);
            painter.line_segment([mark + vec2(-1.0, 3.0), mark + vec2(4.0, -3.5)], stroke);
        }
        let problems: Vec<&Diagnostic> = e.diagnostics.iter().flatten().filter(|d| d.line == Some(line)).collect();
        if problems.is_empty() {
            continue;
        }
        let worst =
            if problems.iter().any(|d| d.severity == Severity::Error) { Severity::Error } else { Severity::Warning };
        let dot = pos2(gutter.right() - 8.0, y);
        painter.circle_filled(dot, 3.5, severity_color(worst));
        let cell = Rect::from_x_y_ranges(gutter.x_range(), row.y_range()).intersect(gutter);
        let message = problems.iter().map(|d| d.message.as_str()).collect::<Vec<_>>().join("\n");
        ui.interact(cell, egui::Id::new(("macro-gutter", line)), Sense::hover()).on_hover_text(message);
    }
}

/// The editor's confirmations: discarding unsaved changes, starting over
/// without undoing steps that cannot be undone.
pub fn dialog(ctx: &egui::Context, state: &mut AppState) {
    let Some(pending) = state.macro_editor.confirm else { return };
    let (id, title, body, action) = match pending {
        Pending::New | Pending::Open => {
            ("macro-discard", t!("macro-ed-discard-title"), t!("macro-ed-discard-body"), t!("action-discard"))
        }
        Pending::Restart => {
            ("macro-restart", t!("macro-ed-restart-title"), t!("macro-ed-restart-body"), t!("macro-ed-start-over"))
        }
    };
    let r = crate::dialog::Dialog::new(id, title).kind(crate::dialog::Kind::Warning).width(420.0).show(
        ctx,
        |ui| {
            ui.label(body);
        },
        |f| {
            f.danger(action);
            f.secondary(t!("action-cancel"));
        },
    );
    match r.clicked {
        Some(0) => {
            state.macro_editor.confirm = None;
            match pending {
                Pending::New => new_macro(state),
                Pending::Open => {
                    state.macro_editor.dirty = false;
                    open_macro(state);
                }
                Pending::Restart => state.macro_editor.reset_run(),
            }
        }
        Some(_) => state.macro_editor.confirm = None,
        None if r.dismissed => state.macro_editor.confirm = None,
        None => {}
    }
}

/// Open a confirmation: starting over, or discarding changes (tests and the
/// every-dialog check).
#[cfg(test)]
pub fn ask(state: &mut AppState, restart: bool) {
    state.macro_editor.confirm = Some(if restart { Pending::Restart } else { Pending::New });
}

#[cfg(test)]
impl MacroEditor {
    /// Whether the steps that ran were edited since.
    pub fn is_stale(&self) -> bool {
        self.stale
    }

    /// The problems of the last check (`None` before one).
    pub fn problems(&self) -> Option<&[Diagnostic]> {
        self.diagnostics.as_deref()
    }

    /// Replace the text as if it had been typed.
    pub fn load_text(&mut self, text: &str) {
        self.set_text(text.to_string());
        self.changed(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_mark_the_command_and_the_placeholders() {
        let line =
            r#"{"command": "wall_create", "args": {"wall": {"$step": 1}, "s": "a\"b", "level": {"$level": "active"}}}"#;
        let marked: Vec<(&str, Token)> = tokens(line).into_iter().filter(|(_, k)| *k != Token::Other).collect();
        assert_eq!(
            marked,
            [
                ("\"wall_create\"", Token::Command),
                ("\"$step\"", Token::Placeholder),
                ("\"$level\"", Token::Placeholder)
            ]
        );
        let joined: String = tokens(line).into_iter().map(|(s, _)| s).collect();
        assert_eq!(joined, line, "the pieces make the line");
        // An unclosed string and other text read without trouble.
        assert_eq!(tokens(r#"{"command": "abc"#).iter().map(|(s, _)| *s).collect::<String>(), r#"{"command": "abc"#);
        assert!(tokens("").is_empty());
    }

    #[test]
    fn line_starts_count_characters() {
        let text = "ab\nçd\n\nx";
        assert_eq!([1, 2, 3, 4].map(|l| line_start(text, l)), [0, 3, 6, 7]);
    }

    fn editor_with(text: &str) -> MacroEditor {
        let mut e = MacroEditor::default();
        e.set_text(text.to_string());
        e
    }

    #[test]
    fn errors_block_the_steps_after_them() {
        let session = cc_engine::Session::default();
        let mut e = editor_with("{\"command\": \"undo\"}\nnot json\n{\"command\": \"redo\"}\n");
        e.check(&session);
        assert_eq!(e.step_lines, [Some(1), Some(3)]);
        assert!(!e.blocked(1, false), "the first step comes before the bad line");
        assert!(e.blocked(2, false));
        assert!(e.blocked(1, true), "running to the end, any error stops it");
        e.cursor_line = 1;
        assert_eq!(e.cursor_target(), 1, "Run to Cursor stops before the step after the cursor");
        // The array format: no lines, steps by number.
        let mut e = editor_with("[{\"command\": \"undo\"}, {\"command\": \"nothing_such\"}]");
        e.check(&session);
        assert_eq!(e.step_lines, [None, None]);
        assert!(!e.blocked(1, false));
        assert!(e.blocked(2, false));
    }
}
