//! The Plugin console: a window (not modal) listing what plugins printed,
//! their load errors, the errors of their event handlers (and the handlers
//! turned off), and when the plugins loaded again, each entry with its time
//! and plugin. It keeps the last [`MAX_ENTRIES`]; it can show one plugin's
//! entries, be cleared and be copied to the clipboard.
//!
//! It is fed by [`record`], called where the app takes the plugins'
//! messages from the session (they still go to the log and the status bar
//! as before), and by [`crate::plugins_ui::load`].

use crate::state::AppState;
use crate::t;
use crate::theme::{self, PALETTE};
use egui::{Context, RichText};
use std::collections::VecDeque;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

/// Most entries kept; older ones go first.
pub const MAX_ENTRIES: usize = 2000;

/// What an entry tells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A line a plugin printed.
    Output,
    /// A load error, or an event handler's error.
    Error,
    /// What the app did: the plugins loaded again, a development copy used.
    Info,
}

/// One entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Time since the app started.
    pub at: Duration,
    /// The plugin's id (empty: the app's own entry).
    pub plugin: String,
    pub kind: Kind,
    pub text: String,
}

impl Entry {
    /// The entry as one line of text (what Copy gives).
    pub fn line(&self) -> String {
        let plugin = if self.plugin.is_empty() { String::new() } else { format!("[{}] ", self.plugin) };
        format!("{} {plugin}{}", clock(self.at), self.text)
    }
}

/// The console's entries and what it shows.
#[derive(Debug, Default)]
pub struct Console {
    /// The window is open.
    pub open: bool,
    entries: VecDeque<Entry>,
    /// Show only this plugin's entries.
    pub filter: Option<String>,
}

/// When the app started, for the entries' times.
fn start() -> Instant {
    static START: OnceLock<Instant> = OnceLock::new();
    *START.get_or_init(Instant::now)
}

/// `hh:mm:ss.mmm` of a time since the app started.
pub fn clock(at: Duration) -> String {
    let ms = at.as_millis();
    let s = ms / 1000;
    format!("{:02}:{:02}:{:02}.{:03}", s / 3600, (s / 60) % 60, s % 60, ms % 1000)
}

/// Whether `text` (without the plugin's id) is an event handler's error,
/// as the plugin host words them (`model_changed handler 1: …`).
fn is_handler_error(text: &str) -> bool {
    let Some((head, _)) = text.split_once(": ") else { return false };
    let mut words = head.split(' ');
    let (Some(event), Some("handler"), Some(n)) = (words.next(), words.next(), words.next()) else {
        return false;
    };
    cc_engine::EventKind::from_name(event).is_some() && n.chars().all(|c| c.is_ascii_digit())
}

impl Console {
    /// Add an entry, dropping the oldest past [`MAX_ENTRIES`].
    pub fn push(&mut self, plugin: &str, kind: Kind, text: impl Into<String>) {
        let at = start().elapsed();
        self.entries.push_back(Entry { at, plugin: plugin.to_string(), kind, text: text.into() });
        while self.entries.len() > MAX_ENTRIES {
            self.entries.pop_front();
        }
    }

    /// Add a message of the plugin host: `[id] text`.
    pub fn push_message(&mut self, line: &str) {
        let (plugin, text) = match line.strip_prefix('[').and_then(|rest| rest.split_once("] ")) {
            Some((id, text)) if cc_plugins::plugin::valid_id(id) => (id, text),
            _ => ("", line),
        };
        let kind = if is_handler_error(text) { Kind::Error } else { Kind::Output };
        self.push(plugin, kind, text);
    }

    #[cfg(test)]
    pub fn entries(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter()
    }

    /// The entries shown: those of the plugin picked, or all.
    pub fn shown(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter().filter(|e| self.filter.as_ref().is_none_or(|f| *f == e.plugin))
    }

    /// The plugins that have entries, by id.
    pub fn plugins(&self) -> Vec<String> {
        let mut ids: Vec<String> =
            self.entries.iter().filter(|e| !e.plugin.is_empty()).map(|e| e.plugin.clone()).collect();
        ids.sort();
        ids.dedup();
        ids
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// The entries shown, one per line.
    pub fn text(&self) -> String {
        self.shown().map(Entry::line).collect::<Vec<_>>().join("\n")
    }
}

/// The plugins' messages taken from the session: to the log and the console.
pub fn record(state: &mut AppState, lines: &[String]) {
    for line in lines {
        log::info!("{line}");
        state.plugins.console.push_message(line);
    }
}

/// Open or close the console.
pub fn toggle(state: &mut AppState) {
    state.plugins.console.open = !state.plugins.console.open;
}

/// The console window, when open.
pub fn show(ctx: &Context, state: &mut AppState) {
    if !state.plugins.console.open {
        return;
    }
    let mut open = true;
    let frame = egui::Frame::window(&ctx.style_of(ctx.theme()))
        .fill(PALETTE.panel)
        .stroke(egui::Stroke::new(1.0, PALETTE.border));
    egui::Window::new(t!("action-plugin-console"))
        .id(egui::Id::new("plugin-console"))
        .open(&mut open)
        .frame(frame)
        .collapsible(false)
        .resizable(true)
        .default_size([640.0, 320.0])
        .show(ctx, |ui| body(ui, state));
    if !open {
        state.plugins.console.open = false;
    }
}

fn body(ui: &mut egui::Ui, state: &mut AppState) {
    let console = &mut state.plugins.console;
    let mut copied = false;
    ui.horizontal(|ui| {
        let all = t!("pcon-all");
        let shown = console.filter.clone().unwrap_or_else(|| all.clone());
        let ids = console.plugins();
        egui::ComboBox::from_id_salt("plugin-console-filter").selected_text(shown).show_ui(ui, |ui| {
            ui.selectable_value(&mut console.filter, None, all);
            for id in ids {
                let label = id.clone();
                ui.selectable_value(&mut console.filter, Some(id), label);
            }
        });
        if ui.button(t!("pcon-clear")).clicked() {
            console.clear();
        }
        if ui.button(t!("pcon-copy")).clicked() {
            ui.ctx().copy_text(console.text());
            copied = true;
        }
    });
    ui.separator();
    egui::ScrollArea::vertical()
        .id_salt("plugin-console-entries")
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            let mut any = false;
            for entry in console.shown() {
                any = true;
                let color = match entry.kind {
                    Kind::Output => PALETTE.text,
                    Kind::Error => PALETTE.danger,
                    Kind::Info => PALETTE.text_muted,
                };
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(clock(entry.at)).monospace().size(11.5).color(PALETTE.text_faint));
                    if !entry.plugin.is_empty() {
                        ui.label(
                            RichText::new(format!("[{}]", entry.plugin)).monospace().size(11.5).color(PALETTE.accent),
                        );
                    }
                    ui.label(RichText::new(&entry.text).monospace().size(11.5).color(color));
                });
            }
            if !any {
                ui.label(theme::muted(t!("pcon-empty")));
            }
        });
    if copied {
        state.message = t!("pcon-copied");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_are_parsed_bounded_filtered_and_copied() {
        let mut c = Console::default();
        c.push_message("[quick_rooms] hello");
        c.push_message("[quick_rooms] model_changed handler 2: boom");
        c.push_message("[other] project_saved handler: not quite");
        c.push_message("no plugin here");
        c.push("", Kind::Info, "reloaded");
        let kinds: Vec<(&str, Kind, &str)> =
            c.entries().map(|e| (e.plugin.as_str(), e.kind, e.text.as_str())).collect();
        assert_eq!(
            kinds,
            [
                ("quick_rooms", Kind::Output, "hello"),
                ("quick_rooms", Kind::Error, "model_changed handler 2: boom"),
                ("other", Kind::Output, "project_saved handler: not quite"),
                ("", Kind::Output, "no plugin here"),
                ("", Kind::Info, "reloaded"),
            ]
        );
        assert_eq!(c.plugins(), ["other", "quick_rooms"]);
        c.filter = Some("quick_rooms".into());
        assert_eq!(c.shown().count(), 2);
        let text = c.text();
        assert_eq!(text.lines().count(), 2);
        assert!(text.lines().all(|l| l.contains("[quick_rooms] ")), "{text}");
        for i in 0..MAX_ENTRIES + 10 {
            c.push("p", Kind::Output, format!("line {i}"));
        }
        assert_eq!(c.entries().count(), MAX_ENTRIES, "bounded");
        assert_eq!(c.entries().last().unwrap().text, format!("line {}", MAX_ENTRIES + 9));
        c.clear();
        assert_eq!(c.entries().count(), 0);
        assert_eq!(clock(Duration::from_millis(3_723_004)), "01:02:03.004");
    }
}
