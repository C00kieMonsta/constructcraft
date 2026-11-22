//! Commands added at run time (plugins). An [`Extension`] set on a
//! [`Session`] lists its commands next to the built-in catalog and runs them
//! when [`Session::execute`] is asked for one of their names, so plugin
//! commands reach the GUI, the command line and the MCP server like the
//! built-in ones.
//!
//! A command of an extension is one undo step however many commands it
//! runs, and a failed one leaves nothing behind.
//!
//! An extension also hears [`Event`]s ([`Extension::on_event`]), which the
//! session fires itself, so every caller (GUI, `ccad`, scripts, MCP) gets
//! them:
//! - [`Event::ProjectOpened`] after `project_new`, `project_open` or any
//!   other command that replaces the document (an import), and once when the
//!   extension is set ([`Session::set_extension`]);
//! - [`Event::ProjectSaving`] before `project_save` writes, and
//!   [`Event::ProjectSaved`] after it wrote;
//! - [`Event::ModelChanged`] after each command that changed the document
//!   (`undo` and `redo` included), with the elements added, modified and
//!   removed, from the document's journal ([`cc_core::Document::start_journal`]),
//!   kept only while the extension listens to it.
//!
//! Events fire only for top-level commands: never while an extension's
//! command or an event handler runs (what those run is part of them), so a
//! handler cannot start a loop. Each step of a script or macro run by
//! `ccad script`, `ccad lua` or the macro editor is a top-level command: it
//! is its own undo step, and fires its own events. Handlers run in
//! [`Session::with_event_scope`]: queries, plus `plugin_data_set` for their
//! own plugin (none after `undo`/`redo`), no files. Their edits after
//! `model_changed` join the undo step of the command that fired it (a
//! dragged value stays one merged step); after the other events they are one
//! undo step of their own. They are never recorded in a macro.
//!
//! A command of an extension may also wait for work outside the session (a
//! plugin running an external program): [`Extension::start`] reports
//! [`Outcome::Pending`] and the session keeps the command open (its undo
//! step, its journal) until [`Extension::resume`] reports it done; see
//! [`Session::poll_job`]. Meanwhile the session runs only queries, and no
//! event fires.

use crate::error::{EngineError, ErrorCode, Result};
use crate::registry::CommandInfo;
use crate::session::Session;
use cc_core::ElementId;
use serde_json::{Map, Value, json};
use std::path::PathBuf;
use std::time::Duration;

/// Separates an extension's id from its command's name in the full name
/// (`<id>__<name>`). Built-in names never contain it.
pub const SEPARATOR: &str = "__";

/// Most ids in each list of a [`ModelChange`]; longer lists are cut and
/// marked `truncated`.
pub const MAX_EVENT_IDS: usize = 10_000;

/// The kinds of [`Event`], by the names plugins use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventKind {
    ProjectOpened,
    ProjectSaving,
    ProjectSaved,
    ModelChanged,
}

impl EventKind {
    pub const ALL: [EventKind; 4] =
        [EventKind::ProjectOpened, EventKind::ProjectSaving, EventKind::ProjectSaved, EventKind::ModelChanged];

    /// Its name (`model_changed`…).
    pub fn name(self) -> &'static str {
        match self {
            EventKind::ProjectOpened => "project_opened",
            EventKind::ProjectSaving => "project_saving",
            EventKind::ProjectSaved => "project_saved",
            EventKind::ModelChanged => "model_changed",
        }
    }

    /// The kind named `name`.
    pub fn from_name(name: &str) -> Option<EventKind> {
        EventKind::ALL.into_iter().find(|k| k.name() == name)
    }
}

/// What changed in the document during one command.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModelChange {
    /// The command's name (`undo`, `redo`, a plugin's `<id>__<name>`…).
    pub command: String,
    /// Elements added, modified and removed, each list in ascending order and
    /// at most [`MAX_EVENT_IDS`] long.
    pub added: Vec<ElementId>,
    pub modified: Vec<ElementId>,
    pub removed: Vec<ElementId>,
    /// A list was cut at [`MAX_EVENT_IDS`].
    pub truncated: bool,
    /// Something other than elements changed (types, views, levels,
    /// settings, the project's plugin data).
    pub other: bool,
}

impl ModelChange {
    /// The change of `command` from the document's journal, its lists cut at
    /// [`MAX_EVENT_IDS`].
    pub fn new(command: &str, changes: cc_core::ElementChanges) -> ModelChange {
        let mut truncated = false;
        let mut cut = |mut ids: Vec<ElementId>| {
            if ids.len() > MAX_EVENT_IDS {
                ids.truncate(MAX_EVENT_IDS);
                truncated = true;
            }
            ids
        };
        let added = cut(changes.added);
        let modified = cut(changes.modified);
        let removed = cut(changes.removed);
        ModelChange { command: command.to_string(), added, modified, removed, truncated, other: changes.other }
    }

    /// Whether it is an `undo` or a `redo`.
    pub fn is_history(&self) -> bool {
        matches!(self.command.as_str(), "undo" | "redo")
    }
}

/// Something that happened to the session's project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// A project was opened or started (`new`: it has no file yet).
    ProjectOpened { path: Option<PathBuf>, new: bool },
    /// The project is about to be written to `path`.
    ProjectSaving { path: PathBuf },
    /// The project was written to `path`.
    ProjectSaved { path: PathBuf },
    /// A command changed the document.
    ModelChanged(ModelChange),
}

impl Event {
    pub fn kind(&self) -> EventKind {
        match self {
            Event::ProjectOpened { .. } => EventKind::ProjectOpened,
            Event::ProjectSaving { .. } => EventKind::ProjectSaving,
            Event::ProjectSaved { .. } => EventKind::ProjectSaved,
            Event::ModelChanged(_) => EventKind::ModelChanged,
        }
    }

    /// What a handler gets, as JSON: `{ path?, new }`, `{ path }`, or
    /// `{ command, added, modified, removed, truncated?, other? }` (ids as
    /// numbers; `path`, `truncated` and `other` only when set).
    pub fn payload(&self) -> Value {
        let ids = |list: &[ElementId]| Value::Array(list.iter().map(|id| json!(id.0)).collect());
        // Lossy: a path that is not Unicode still reaches the handler.
        let text = |p: &std::path::Path| Value::String(p.to_string_lossy().into_owned());
        match self {
            Event::ProjectOpened { path, new } => {
                let mut out = Map::new();
                if let Some(p) = path {
                    out.insert("path".into(), text(p));
                }
                out.insert("new".into(), json!(new));
                Value::Object(out)
            }
            Event::ProjectSaving { path } | Event::ProjectSaved { path } => json!({ "path": text(path) }),
            Event::ModelChanged(c) => {
                let mut out = Map::new();
                out.insert("command".into(), json!(c.command));
                out.insert("added".into(), ids(&c.added));
                out.insert("modified".into(), ids(&c.modified));
                out.insert("removed".into(), ids(&c.removed));
                if c.truncated {
                    out.insert("truncated".into(), json!(true));
                }
                if c.other {
                    out.insert("other".into(), json!(true));
                }
                Value::Object(out)
            }
        }
    }
}

/// What starting or resuming an extension's command gives.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    /// The command ended, with its result.
    Done(Result<Value>),
    /// The command waits for work outside the session (an external
    /// program); the session goes on with it through [`Extension::resume`].
    Pending,
}

/// A command left waiting, as its extension describes it (see
/// [`Session::job`]).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct JobInfo {
    /// The plugin's id.
    pub plugin: String,
    /// The plugin's name, in the current language.
    pub plugin_name: String,
    /// What it waits for (a program's name), in the current language.
    pub tool: String,
    /// The last line that work wrote (empty when none yet).
    pub last_line: String,
}

/// A source of commands beyond the built-in catalog.
pub trait Extension {
    /// The commands it adds, with their full names.
    fn commands(&self) -> Vec<CommandInfo>;

    /// Run the command `name` (one of [`Extension::commands`]) on `session`,
    /// or `None` when the name is not one of its own. The extension is taken
    /// out of the session while it runs, so commands it runs through
    /// [`Session::execute`] reach the built-in catalog only.
    fn run(&mut self, session: &mut Session, name: &str, args: Value) -> Option<Result<Value>>;

    /// Start the command `name`, like [`Extension::run`], except that it may
    /// leave the command waiting for work outside the session
    /// ([`Outcome::Pending`]); the session then calls [`Extension::resume`]
    /// until it is done, or [`Extension::cancel`]. The default runs it at
    /// once.
    fn start(&mut self, session: &mut Session, name: &str, args: Value) -> Option<Outcome> {
        self.run(session, name, args).map(Outcome::Done)
    }

    /// Go on with the command left waiting: wait at most `wait` for what it
    /// waits for, and when that is done, run the command on (in its scope
    /// again) until it ends or waits again. The extension is taken out of
    /// the session meanwhile, as in [`Extension::run`].
    fn resume(&mut self, _session: &mut Session, _wait: Duration) -> Outcome {
        Outcome::Done(Err(EngineError::new(ErrorCode::Internal, "no command is waiting")))
    }

    /// Stop the command left waiting and what it waits for; the session
    /// fails it as cancelled and undoes what it did.
    fn cancel(&mut self, _session: &mut Session) {}

    /// The command left waiting, if any.
    fn job(&self) -> Option<JobInfo> {
        None
    }

    /// Messages for the user since the last call (what plugins printed).
    fn take_messages(&mut self) -> Vec<String> {
        Vec::new()
    }

    /// Whether it handles events of `kind`: the session fires only those
    /// (and keeps the document's journal for [`Event::ModelChanged`] only
    /// then).
    fn listens(&self, _kind: EventKind) -> bool {
        false
    }

    /// Handle `event`, after the command that caused it finished (before
    /// `project_save` writes, for [`Event::ProjectSaving`]). The extension is
    /// taken out of the session meanwhile, and nothing it does fails that
    /// command. Run handlers in [`Session::with_event_scope`].
    fn on_event(&mut self, _session: &mut Session, _event: &Event) {}
}

/// Whether `name` could be the full name of an extension command.
pub fn is_extension_name(name: &str) -> bool {
    name.contains(SEPARATOR)
}
