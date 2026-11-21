//! Plugin commands that wait for an external program (`Outcome::Pending`):
//! waited for by a blocking session, left pending by a non-blocking one,
//! where only queries run until `poll_job` ends them (one undo step, events
//! after) or `cancel_job` undoes them.

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_engine::{CommandInfo, CommandKind, ErrorCode, Event, EventKind, Extension, JobInfo, Outcome, Session};
use serde_json::{Value, json};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

/// What the test sees of the extension.
#[derive(Default)]
struct Shared {
    /// What it waits for is done.
    ready: Cell<bool>,
    /// Resumes left before it is done on its own (blocking sessions).
    polls_left: Cell<u32>,
    /// Fail when it goes on.
    fail: Cell<bool>,
    cancelled: Cell<u32>,
    events: RefCell<Vec<Event>>,
}

/// `test__two_walls`: a wall, then it waits, then another wall.
struct Waiter {
    shared: Rc<Shared>,
    waiting: bool,
}

fn wall(s: &mut Session, x: f64) -> cc_engine::Result<Value> {
    s.execute("wall_create", json!({"start": [x, 0], "end": [x + 1.0, 0]}))
}

impl Extension for Waiter {
    fn commands(&self) -> Vec<CommandInfo> {
        vec![CommandInfo {
            name: "test__two_walls".into(),
            kind: CommandKind::Edit,
            group: "Test".into(),
            description: "Two walls, waiting in between".into(),
            input_schema: json!({"type": "object", "properties": {}}),
            reads_files: false,
        }]
    }

    fn run(&mut self, _: &mut Session, _: &str, _: Value) -> Option<cc_engine::Result<Value>> {
        None
    }

    fn start(&mut self, session: &mut Session, name: &str, _: Value) -> Option<Outcome> {
        if name != "test__two_walls" {
            return None;
        }
        if let Err(e) = wall(session, 0.0) {
            return Some(Outcome::Done(Err(e)));
        }
        self.waiting = true;
        Some(Outcome::Pending)
    }

    fn resume(&mut self, session: &mut Session, _wait: Duration) -> Outcome {
        assert!(self.waiting, "resumed while not waiting");
        let left = self.shared.polls_left.get();
        if !self.shared.ready.get() && left > 0 {
            self.shared.polls_left.set(left - 1);
            return Outcome::Pending;
        }
        self.waiting = false;
        // What it runs now is not refused: it is the command going on.
        if let Err(e) = wall(session, 2.0) {
            return Outcome::Done(Err(e));
        }
        if self.shared.fail.get() {
            return Outcome::Done(Err(cc_engine::EngineError::rejected("the program failed")));
        }
        Outcome::Done(Ok(json!({"walls": 2})))
    }

    fn cancel(&mut self, _: &mut Session) {
        self.waiting = false;
        self.shared.cancelled.set(self.shared.cancelled.get() + 1);
    }

    fn job(&self) -> Option<JobInfo> {
        self.waiting.then(|| JobInfo {
            plugin: "test".into(),
            plugin_name: "Test".into(),
            tool: "Simulator".into(),
            last_line: "step 3".into(),
        })
    }

    fn listens(&self, kind: EventKind) -> bool {
        kind == EventKind::ModelChanged
    }

    fn on_event(&mut self, _: &mut Session, event: &Event) {
        self.shared.events.borrow_mut().push(event.clone());
    }
}

fn session(blocking: bool) -> (Session, Rc<Shared>) {
    let shared = Rc::new(Shared::default());
    // Without blocking, it waits until the test says it is ready.
    if !blocking {
        shared.polls_left.set(u32::MAX);
    }
    let mut s = Session::default();
    s.set_blocking(blocking);
    s.set_extension(Some(Box::new(Waiter { shared: shared.clone(), waiting: false })));
    (s, shared)
}

fn walls(s: &mut Session) -> usize {
    s.execute("elements_list", json!({"category": "wall"})).unwrap().as_array().unwrap().len()
}

#[test]
fn a_blocking_session_waits_and_the_command_is_one_step() {
    let (mut s, shared) = session(true);
    assert!(s.is_blocking());
    assert!(Session::default().is_blocking(), "sessions block by default");
    shared.polls_left.set(3);
    let depth = s.doc.undo_depth();
    assert_eq!(s.execute("test__two_walls", json!({})).unwrap(), json!({"walls": 2}));
    assert_eq!(shared.polls_left.get(), 0, "it was resumed until done");
    assert!(!s.has_job());
    assert_eq!((walls(&mut s), s.doc.undo_depth()), (2, depth + 1));
    // One model_changed, with both walls.
    let events = shared.events.borrow().clone();
    assert_eq!(events.len(), 1, "{events:?}");
    let Event::ModelChanged(change) = &events[0] else { panic!("{events:?}") };
    assert_eq!((change.command.as_str(), change.added.len()), ("test__two_walls", 2));
    s.execute("undo", json!({})).unwrap();
    assert_eq!(walls(&mut s), 0);
}

#[test]
fn a_pending_command_lets_only_queries_run_and_ends_when_polled() {
    let (mut s, shared) = session(false);
    let depth = s.doc.undo_depth();
    let e = s.execute("test__two_walls", json!({})).unwrap_err();
    assert_eq!(e.code, ErrorCode::Pending, "{}", e.message);
    assert!(e.message.contains("Simulator"), "{}", e.message);
    assert!(s.has_job());
    let job = s.job().unwrap();
    assert_eq!(job.command, "test__two_walls");
    assert_eq!((job.info.plugin_name.as_str(), job.info.tool.as_str()), ("Test", "Simulator"));
    assert_eq!(job.info.last_line, "step 3");
    // Queries run; edits, undo, other plugin commands and files do not.
    assert_eq!(walls(&mut s), 1, "the first wall is there meanwhile");
    for (name, args) in [
        ("wall_create", json!({"start": [5, 5], "end": [6, 5]})),
        ("undo", json!({})),
        ("test__two_walls", json!({})),
        ("project_new", json!({})),
        ("project_save", json!({"path": "x.ccad"})),
    ] {
        let e = s.execute(name, args).unwrap_err();
        assert_eq!(e.code, ErrorCode::Rejected, "{name}: {}", e.message);
        assert!(e.message.contains("a plugin command is running"), "{name}: {}", e.message);
    }
    assert!(shared.events.borrow().is_empty(), "no event while it waits");
    // Not ready: still pending.
    assert_eq!(s.poll_job(), None);
    assert!(s.has_job());
    shared.ready.set(true);
    assert_eq!(s.poll_job(), Some(Ok(json!({"walls": 2}))));
    assert!(!s.has_job() && s.job().is_none());
    assert_eq!(s.poll_job(), None, "nothing waits any more");
    assert_eq!((walls(&mut s), s.doc.undo_depth()), (2, depth + 1), "one undo step");
    assert_eq!(shared.events.borrow().len(), 1, "model_changed once it ended");
    // Edits run again.
    s.execute("wall_create", json!({"start": [5, 5], "end": [6, 5]})).unwrap();
    s.execute("undo", json!({})).unwrap();
    s.execute("undo", json!({})).unwrap();
    assert_eq!(walls(&mut s), 0);
}

#[test]
fn a_cancelled_or_failed_pending_command_leaves_nothing() {
    let (mut s, shared) = session(false);
    let depth = s.doc.undo_depth();
    assert_eq!(s.execute("test__two_walls", json!({})).unwrap_err().code, ErrorCode::Pending);
    let e = s.cancel_job().unwrap().unwrap_err();
    assert_eq!(e.code, ErrorCode::Cancelled, "{}", e.message);
    assert_eq!(shared.cancelled.get(), 1);
    assert_eq!((walls(&mut s), s.doc.undo_depth()), (0, depth));
    assert!(!s.doc.can_redo(), "a cancelled command cannot be redone");
    assert_eq!(s.cancel_job(), None);
    assert!(shared.events.borrow().is_empty());

    shared.fail.set(true);
    shared.ready.set(true);
    assert_eq!(s.execute("test__two_walls", json!({})).unwrap_err().code, ErrorCode::Pending);
    assert_eq!(s.poll_job().unwrap().unwrap_err().code, ErrorCode::Rejected);
    assert_eq!((walls(&mut s), s.doc.undo_depth()), (0, depth));

    // Replacing the extension or the document cancels the command waiting.
    shared.fail.set(false);
    assert_eq!(s.execute("test__two_walls", json!({})).unwrap_err().code, ErrorCode::Pending);
    s.set_extension(None);
    assert!(!s.has_job());
    assert_eq!((shared.cancelled.get(), walls(&mut s)), (2, 0));
    let mut s2 = session(false);
    assert_eq!(s2.0.execute("test__two_walls", json!({})).unwrap_err().code, ErrorCode::Pending);
    s2.0.replace(cc_core::Document::new(cc_engine::new_project()));
    assert!(!s2.0.has_job());
    assert_eq!(s2.1.cancelled.get(), 1);
    assert!(!s2.0.is_blocking(), "the mode stays with a new document");
}

#[test]
fn scripts_and_waiting_wait_even_without_blocking() {
    let (mut s, shared) = session(false);
    shared.polls_left.set(2);
    // A macro's step waits: the next step may need what it made.
    let steps = cc_engine::script::parse(r#"[{"command": "test__two_walls", "args": {}}]"#).unwrap();
    let results = cc_engine::script::run(&mut s, &steps).unwrap();
    assert_eq!(results, vec![json!({"walls": 2})]);
    assert!(!s.is_blocking(), "the mode is put back");
    // wait_job ends a pending command.
    shared.polls_left.set(2);
    assert_eq!(s.execute("test__two_walls", json!({})).unwrap_err().code, ErrorCode::Pending);
    assert_eq!(s.wait_job(), Some(Ok(json!({"walls": 2}))));
    assert_eq!(s.wait_job(), None);
    assert_eq!(walls(&mut s), 4);
    // with_blocking for one call.
    shared.polls_left.set(1);
    let out = s.with_blocking(true, |s| s.execute("test__two_walls", json!({})));
    assert_eq!(out.unwrap(), json!({"walls": 2}));
    assert!(!s.is_blocking());
}
