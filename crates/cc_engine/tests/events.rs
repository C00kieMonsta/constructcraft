//! Events an extension hears (`Extension::on_event`): fired by the session
//! after top-level commands, never while a handler or an extension command
//! runs; handlers run in `Session::with_event_scope`.

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_core::ElementId;
use cc_engine::{CommandInfo, ErrorCode, Event, EventKind, Extension, ModelChange, Session};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

type Log = Rc<RefCell<Vec<Event>>>;
type Reaction = Box<dyn FnMut(&mut Session, &Event) -> Option<cc_engine::Result<Value>>>;

/// Keeps every event it hears; `react` runs in the event's scope.
struct Recorder {
    log: Log,
    react: Option<Reaction>,
    /// What `react` returned, last first.
    results: Rc<RefCell<Vec<cc_engine::Result<Value>>>>,
}

impl Extension for Recorder {
    fn commands(&self) -> Vec<CommandInfo> {
        Vec::new()
    }

    fn run(&mut self, _: &mut Session, _: &str, _: Value) -> Option<cc_engine::Result<Value>> {
        None
    }

    fn listens(&self, _: EventKind) -> bool {
        true
    }

    fn on_event(&mut self, session: &mut Session, event: &Event) {
        self.log.borrow_mut().push(event.clone());
        let Some(react) = &mut self.react else { return };
        let results = self.results.clone();
        session.with_event_scope(event, "p", "Plugin P", |s| {
            if let Some(r) = react(s, event) {
                results.borrow_mut().push(r);
            }
        });
    }
}

struct Setup {
    session: Session,
    log: Log,
    results: Rc<RefCell<Vec<cc_engine::Result<Value>>>>,
}

fn setup(react: Option<Reaction>) -> Setup {
    let log: Log = Rc::default();
    let results = Rc::default();
    let mut session = Session::default();
    session.set_extension(Some(Box::new(Recorder { log: log.clone(), react, results: Rc::clone(&results) })));
    Setup { session, log, results }
}

fn take(log: &Log) -> Vec<Event> {
    std::mem::take(&mut *log.borrow_mut())
}

fn changed(log: &Log) -> Vec<ModelChange> {
    take(log)
        .into_iter()
        .filter_map(|e| match e {
            Event::ModelChanged(c) => Some(c),
            _ => None,
        })
        .collect()
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ccad-events-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn wall(s: &mut Session, x: f64) -> ElementId {
    let before: Vec<u64> = ids(s);
    s.execute("wall_create", json!({"start": [x, 0], "end": [x + 4.0, 0]})).unwrap();
    let id = ids(s).into_iter().find(|i| !before.contains(i)).unwrap();
    ElementId(id)
}

fn ids(s: &mut Session) -> Vec<u64> {
    let list = s.execute("elements_list", json!({"category": "wall"})).unwrap();
    list.as_array().unwrap().iter().map(|e| e["id"].as_u64().unwrap()).collect()
}

#[test]
fn project_events_carry_their_paths() {
    let Setup { mut session, log, .. } = setup(None);
    // Set on a session holding a project: it hears that project.
    assert_eq!(take(&log), vec![Event::ProjectOpened { path: None, new: true }]);
    let dir = scratch("project");
    let path = dir.join("a.ccad");
    session.execute("project_save", json!({"path": path})).unwrap();
    assert_eq!(
        take(&log),
        vec![Event::ProjectSaving { path: path.clone() }, Event::ProjectSaved { path: path.clone() }]
    );
    // Saved again to its own file.
    session.execute("project_save", json!({})).unwrap();
    assert_eq!(take(&log).len(), 2);
    session.execute("project_open", json!({"path": path})).unwrap();
    assert_eq!(take(&log), vec![Event::ProjectOpened { path: Some(path.clone()), new: false }]);
    session.execute("project_new", json!({})).unwrap();
    assert_eq!(take(&log), vec![Event::ProjectOpened { path: None, new: true }]);
    // Failed commands fire nothing (saving needs a path; a missing file).
    assert!(session.execute("project_save", json!({})).is_err());
    assert!(session.execute("project_open", json!({"path": dir.join("missing.ccad")})).is_err());
    assert_eq!(take(&log), vec![]);
    // Queries fire nothing.
    session.execute("project_info", json!({})).unwrap();
    assert_eq!(take(&log), vec![]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn saving_fires_before_the_file_is_written() {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let sink = seen.clone();
    let react: Reaction = Box::new(move |_, e| {
        if let Event::ProjectSaving { path } | Event::ProjectSaved { path } = e {
            sink.borrow_mut().push(path.exists());
        }
        None
    });
    let Setup { mut session, .. } = setup(Some(react));
    let dir = scratch("saving");
    session.execute("project_save", json!({"path": dir.join("b.ccad")})).unwrap();
    assert_eq!(*seen.borrow(), vec![false, true]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn model_changed_lists_added_modified_and_removed_elements() {
    let Setup { mut session, log, .. } = setup(None);
    take(&log);
    let a = wall(&mut session, 0.0);
    let c = changed(&log);
    assert_eq!(c.len(), 1);
    assert_eq!((c[0].command.as_str(), c[0].added.clone()), ("wall_create", vec![a]));
    assert!(c[0].modified.is_empty() && c[0].removed.is_empty() && !c[0].truncated);
    let b = wall(&mut session, 10.0);
    take(&log);
    session.execute("element_set", json!({"id": a.0, "mark": "W1"})).unwrap();
    let c = changed(&log);
    assert_eq!((c[0].command.as_str(), c[0].modified.clone()), ("element_set", vec![a]));
    session.execute("elements_delete", json!({"ids": [b.0, a.0]})).unwrap();
    let c = changed(&log);
    assert_eq!((c[0].removed.clone(), c[0].added.clone()), (vec![a, b], vec![]));
    session.execute("undo", json!({})).unwrap();
    let c = changed(&log);
    assert_eq!((c[0].command.as_str(), c[0].added.clone()), ("undo", vec![a, b]));
    session.execute("redo", json!({})).unwrap();
    let c = changed(&log);
    assert_eq!((c[0].command.as_str(), c[0].removed.clone()), ("redo", vec![a, b]));
    // Not for failed commands, nor for edits that change nothing.
    assert!(session.execute("element_set", json!({"id": 999_999, "mark": "x"})).is_err());
    assert!(session.execute("wall_create", json!({"start": [0, 0]})).is_err());
    assert_eq!(take(&log), vec![]);
    // Other data than elements: `other`, with empty lists.
    session.execute("level_create", json!({"name": "Roof", "elevation": 9})).unwrap();
    let c = changed(&log);
    assert!(c[0].other && c[0].added.is_empty() && c[0].modified.is_empty() && c[0].removed.is_empty());
    assert_eq!(c[0].command, "level_create");
    let payload = Event::ModelChanged(c[0].clone()).payload();
    assert_eq!(payload, json!({"command": "level_create", "added": [], "modified": [], "removed": [], "other": true}));
}

#[test]
fn long_id_lists_are_cut() {
    let many: Vec<ElementId> = (1..=10_005).map(ElementId).collect();
    let changes = cc_core::ElementChanges { added: many.clone(), modified: vec![ElementId(1)], ..Default::default() };
    let c = ModelChange::new("x", changes);
    assert_eq!((c.added.len(), c.modified.len(), c.truncated), (cc_engine::extension::MAX_EVENT_IDS, 1, true));
    assert_eq!(Event::ModelChanged(c).payload()["truncated"], json!(true));
    let c = ModelChange::new("x", cc_core::ElementChanges { added: many[..3].to_vec(), ..Default::default() });
    assert!(!c.truncated && Event::ModelChanged(c).payload().get("truncated").is_none());
}

#[test]
fn handlers_store_their_own_data_in_one_step_and_nothing_else() {
    let react: Reaction = Box::new(|s, e| match e {
        Event::ModelChanged(c) => {
            // Its own data goes through (and fires nothing: no loop) ...
            let stored = s.execute("plugin_data_set", json!({"plugin": "p", "data": c.added.len()}));
            let again = s.execute("plugin_data_set", json!({"plugin": "p", "data": [c.command.clone()]}));
            // ... the model, other plugins' data and files do not.
            let set = s.execute("element_set", json!({"id": c.added.first().map_or(0, |i| i.0), "mark": "X"}));
            let other = s.execute("plugin_data_set", json!({"plugin": "q", "data": 1}));
            let file = s.execute("project_save", json!({"path": "x.ccad"}));
            Some(Ok(json!([stored.is_ok(), again.is_ok(), code(set), code(other), code(file)])))
        }
        _ => None,
    });
    let Setup { mut session, log, results } = setup(Some(react));
    session.start_recording();
    let depth = session.doc.undo_depth();
    wall(&mut session, 0.0);
    assert_eq!(changed(&log).len(), 1);
    assert_eq!(results.borrow_mut().pop().unwrap().unwrap(), json!([true, true, "rejected", "rejected", "rejected"]));
    // The wall, then the handler's edits as one step named after the plugin.
    assert_eq!(session.doc.undo_depth(), depth + 2);
    assert!(session.doc.undo_name().unwrap().contains("Plugin P"), "{:?}", session.doc.undo_name());
    assert_eq!(session.execute("plugin_data_get", json!({"plugin": "p"})).unwrap(), json!(["wall_create"]));
    // Only the wall is recorded in the macro.
    let steps = session.stop_recording();
    assert_eq!(steps.iter().map(|s| s.command.as_str()).collect::<Vec<_>>(), vec!["wall_create"]);
    // After undo and redo, handlers only query: storing would clear the
    // redo history and come back on every undo.
    session.execute("undo", json!({})).unwrap();
    assert_eq!(results.borrow_mut().pop().unwrap().unwrap()[0], json!(false));
    assert_eq!(session.execute("plugin_data_get", json!({"plugin": "p"})).unwrap(), Value::Null);
    assert!(session.doc.can_redo());
    session.execute("undo", json!({})).unwrap();
    assert!(ids(&mut session).is_empty());
    session.execute("redo", json!({})).unwrap();
    session.execute("redo", json!({})).unwrap();
    assert_eq!(session.execute("plugin_data_get", json!({"plugin": "p"})).unwrap(), json!(["wall_create"]));
    assert_eq!(ids(&mut session).len(), 1);
}

fn code(r: cc_engine::Result<Value>) -> String {
    match r {
        Ok(_) => "ok".into(),
        Err(e) => serde_json::to_value(e.code).unwrap().as_str().unwrap().to_string(),
    }
}

#[test]
fn a_macro_step_steps_back_with_its_handler_edits() {
    let react: Reaction = Box::new(|s, e| match e {
        Event::ModelChanged(c) => Some(s.execute("plugin_data_set", json!({"plugin": "p", "data": c.command}))),
        _ => None,
    });
    let Setup { mut session, .. } = setup(Some(react));
    let steps =
        cc_engine::script::parse(r#"[{"command": "wall_create", "args": {"start": [0, 0], "end": [3, 0]}}]"#).unwrap();
    let mut stepper = cc_engine::script::Stepper::new(steps);
    let depth = session.doc.undo_depth();
    stepper.next(&mut session).unwrap();
    assert_eq!(session.doc.undo_depth(), depth + 2);
    stepper.back(&mut session).unwrap();
    assert_eq!(session.doc.undo_depth(), depth);
    assert!(ids(&mut session).is_empty());
    assert_eq!(session.execute("plugin_data_get", json!({"plugin": "p"})).unwrap(), Value::Null);
}

#[test]
fn events_do_not_fire_inside_scopes_or_handlers() {
    let Setup { mut session, log, .. } = setup(None);
    take(&log);
    // What a plugin command (a scope) runs is part of it.
    let scope = cc_engine::PluginScope {
        plugin: "p".into(),
        edits: cc_engine::ScopeEdits::All,
        files: cc_engine::FileAccess::All,
    };
    session.with_plugin_scope(scope, |s| {
        s.execute("wall_create", json!({"start": [0, 0], "end": [2, 0]})).unwrap();
    });
    assert_eq!(take(&log), vec![]);
    // A handler that panics loses nothing: the extension stays.
    let react: Reaction = Box::new(|_, _| panic!("handler bug"));
    let log2: Log = Rc::default();
    let ext = Recorder { log: log2.clone(), react: Some(react), results: Rc::default() };
    session.set_extension(Some(Box::new(ext)));
    assert_eq!(take(&log2).len(), 1);
    session.execute("level_create", json!({"name": "L", "elevation": 5})).unwrap();
    assert_eq!(take(&log2).len(), 1);
    assert!(session.extension().is_some());
    assert_eq!(session.execute("no_such", json!({})).unwrap_err().code, ErrorCode::UnknownCommand);
}
