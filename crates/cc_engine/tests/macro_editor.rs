//! The macro editor's engine half (issue #310): reading a macro line by
//! line, checking it without running it, and running it step by step with
//! stepping back.

use cc_engine::Session;
use cc_engine::i18n::{Language, set_language};
use cc_engine::script::{self, LineError, LineItem, Severity, Stepper};
use serde_json::{Value, json};

const HOUSE: &str = r#"// Two walls and a window in the first one
{"command": "wall_create", "args": {"start": [0, 0], "end": [6, 0], "height": 3.0}}

{"command": "wall_create", "args": {"start": [6, 0], "end": [6, 4], "height": 3.0}}
{"command": "elements_list", "args": {"category": "wall"}}
{"command": "window_create", "args": {"wall": {"$step": 1}, "offset": 2.0, "sill": 1.0}}
"#;

fn steps(text: &str) -> Vec<script::Step> {
    script::parse(text).unwrap()
}

#[test]
fn lines_are_read_one_by_one_without_stopping_at_a_bad_one() {
    let text = "// comment\n\n{\"command\": \"undo\"}\nnot json\n[1]\n{\"args\": {}}\n  // indented\n\
                {\"command\": \"redo\", \"note\": 1}\n";
    let lines = script::parse_lines(text);
    assert_eq!(lines.len(), 8);
    assert!(lines.iter().enumerate().all(|(i, l)| l.line == Some(i + 1)));
    assert_eq!(lines[0].item, LineItem::Comment);
    assert_eq!(lines[1].item, LineItem::Blank);
    assert!(matches!(&lines[2].item, LineItem::Step { number: 1, step, ignored } if step.command == "undo"
        && step.args == json!({}) && ignored.is_empty()));
    assert!(matches!(lines[3].item, LineItem::Error(LineError::Json(_))));
    assert_eq!(lines[4].item, LineItem::Error(LineError::NotObject), "a line starting with [ is read as JSON");
    assert_eq!(lines[5].item, LineItem::Error(LineError::NoCommand));
    assert_eq!(lines[6].item, LineItem::Comment);
    // Bad lines take no number: the next step is the second.
    assert!(matches!(&lines[7].item, LineItem::Step { number: 2, ignored, .. } if ignored == &["note".to_string()]));

    // `parse` still stops at the first bad line.
    assert!(script::parse(text).is_err());

    // A line that is JSON but no object.
    let lines = script::parse_lines("{\"command\": \"undo\"}\n42\n");
    assert_eq!(lines[1].item, LineItem::Error(LineError::NotObject));

    // The array format: steps with no line, and a parse error at its line.
    let lines = script::parse_lines("[{\"command\": \"undo\"}, 3, {\"command\": \"redo\"}]");
    assert_eq!(lines.len(), 3);
    assert!(lines.iter().all(|l| l.line.is_none()));
    assert!(matches!(lines[1].item, LineItem::Error(LineError::NotObject)));
    assert!(matches!(lines[2].item, LineItem::Step { number: 2, .. }));
    let lines = script::parse_lines("[\n{\"command\": \"undo\"},\n{oops}\n]");
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].line, Some(3));
    assert!(matches!(lines[0].item, LineItem::Error(LineError::Json(_))));
}

#[test]
fn checking_reports_each_problem_at_its_line_and_step() {
    set_language(Language::En);
    let s = Session::default();
    assert!(script::check(&s, HOUSE).is_empty(), "{:?}", script::check(&s, HOUSE));

    let text = [
        "{\"command\": \"wall_create\", \"args\": {\"start\": [0, 0], \"end\": [1, 0]}}", // 1: fine
        "{oops",                                                                          // 2: JSON
        "[1, 2]",                                                                         // 3: not an object
        "{\"args\": {}}",                                                                 // 4: no command
        "{\"command\": \"no_such_command\"}",                                             // 5: step 2
        "{\"command\": \"wall_create\", \"args\": {\"start\": [0, 0], \"lenght\": 3}}",   // 6: step 3
        "{\"command\": \"window_create\", \"args\": {\"wall\": {\"$step\": 0}}}",         // 7: step 4
        "{\"command\": \"window_create\", \"args\": {\"wall\": {\"$step\": 5}}}",         // 8: step 5
        "{\"command\": \"window_create\", \"args\": {\"wall\": {\"$step\": 99}}}",        // 9: step 6
        "{\"command\": \"window_create\", \"args\": {\"wall\": {\"$step\": \"x\"}}}",     // 10: step 7
        "{\"command\": \"window_create\", \"args\": {\"wall\": {\"$step\": 1, \"path\": \"id\"}}}", // 11: step 8
        "{\"command\": \"wall_create\", \"args\": {\"level\": {\"$level\": \"first\"}, \"start\": [0, 0], \"end\": [1, 0]}}", // 12: step 9
        "{\"command\": \"undo\", \"args\": [1]}",      // 13: step 10
        "{\"command\": \"undo\", \"comment\": \"x\"}", // 14: step 11
        // A placeholder fills a required argument.
        "{\"command\": \"wall_create\", \"args\": {\"start\": {\"$step\": 1, \"path\": \"/start\"}, \"end\": [1, 0]}}", // 15
    ]
    .join("\n");
    let d = script::check(&s, &text);
    let at = |line: usize| d.iter().filter(|d| d.line == Some(line)).collect::<Vec<_>>();
    assert!(at(1).is_empty() && at(15).is_empty(), "{d:#?}");
    for line in [2, 3, 4] {
        let found = at(line);
        assert_eq!(found.len(), 1, "line {line}: {d:#?}");
        assert_eq!((found[0].step, found[0].severity), (None, Severity::Error));
    }
    assert!(at(2)[0].message.starts_with("Invalid JSON"), "{}", at(2)[0].message);
    assert!(at(3)[0].message.contains("object"));
    assert!(at(4)[0].message.contains("command"));
    let one = |line: usize, step: usize, words: &str| {
        let found = at(line);
        assert_eq!(found.len(), 1, "line {line}: {d:#?}");
        assert_eq!(found[0].step, Some(step), "line {line}");
        assert!(found[0].message.contains(words), "line {line}: {}", found[0].message);
        found[0].severity
    };
    assert_eq!(one(5, 2, "Unknown command: no_such_command"), Severity::Error);
    // An unknown argument and a missing one.
    let six = at(6);
    assert_eq!(six.len(), 2, "{six:#?}");
    assert!(six.iter().any(|d| d.message.contains("no argument \"lenght\"")));
    assert!(six.iter().any(|d| d.message.contains("needs the argument \"end\"")));
    // `window_create` has no required argument besides the wall, which is a placeholder.
    assert_eq!(one(7, 4, "count from 1"), Severity::Error);
    assert_eq!(one(8, 5, "not an earlier step"), Severity::Error);
    assert_eq!(one(9, 6, "beyond the macro, which has 12 steps"), Severity::Error);
    assert_eq!(one(10, 7, "step number"), Severity::Error);
    assert_eq!(one(11, 8, "JSON pointer"), Severity::Error);
    assert_eq!(one(12, 9, "\"active\""), Severity::Error);
    assert_eq!(one(13, 10, "\"args\" must be a JSON object"), Severity::Error);
    assert_eq!(one(14, 11, "\"comment\" is ignored"), Severity::Warning);

    // Nothing ran.
    assert_eq!(s.doc.undo_depth(), 0);
    assert_eq!(s.project().walls().count(), 0);
}

#[test]
fn the_check_command_reads_text_or_a_file() {
    set_language(Language::En);
    let mut s = Session::default();
    let out = s.execute("macro_check", json!({ "text": HOUSE })).unwrap();
    assert_eq!(out, json!({ "steps": 4, "diagnostics": [] }));
    let out = s.execute("macro_check", json!({ "text": "{\"command\": \"nope\"}\n{bad" })).unwrap();
    assert_eq!(out["steps"], 1);
    assert_eq!(out["diagnostics"][0]["line"], 1);
    assert_eq!(out["diagnostics"][0]["step"], 1);
    assert_eq!(out["diagnostics"][0]["severity"], "error");
    assert_eq!(out["diagnostics"][1]["line"], 2);
    assert_eq!(out["diagnostics"][1]["step"], Value::Null);

    let dir = std::env::temp_dir().join(format!("cc_macro_check_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("house.ccmacro");
    std::fs::write(&path, HOUSE).unwrap();
    let out = s.execute("macro_check", json!({ "path": path.to_string_lossy() })).unwrap();
    assert_eq!(out["steps"], 4);
    let _ = std::fs::remove_dir_all(&dir);
    assert!(s.execute("macro_check", json!({})).is_err());
}

#[test]
fn formatting_keeps_comments_blank_lines_and_bad_lines() {
    let mut s = Session::default();
    let text = "  // a comment  \n\n{ \"command\" : \"wall_create\",\n{\"command\": \"undo\",   \"args\": { } }\n";
    let out = s.execute("macro_format", json!({ "text": text })).unwrap();
    let formatted = out["text"].as_str().unwrap();
    let canonical = script::to_text(&steps("{\"command\": \"undo\"}"));
    assert_eq!(formatted, format!("// a comment\n\n{{ \"command\" : \"wall_create\",\n{canonical}"));
    // Formatting again changes nothing, and the steps are the same.
    let again = s.execute("macro_format", json!({ "text": formatted })).unwrap();
    assert_eq!(again["text"], out["text"]);
    let house = s.execute("macro_format", json!({ "text": HOUSE })).unwrap();
    assert_eq!(steps(house["text"].as_str().unwrap()), steps(HOUSE));
    assert!(house["text"].as_str().unwrap().starts_with("// Two walls"));
    // The array format becomes one step per line.
    let array = s.execute("macro_format", json!({ "text": "[{\"command\": \"undo\"}]" })).unwrap();
    assert_eq!(array["text"].as_str(), Some(canonical.as_str()));
    assert!(s.execute("macro_format", json!({ "text": "[{bad" })).is_err());
}

#[test]
fn stepping_forward_and_back_leaves_the_project_as_it_was() {
    let mut s = Session::default();
    let before = state(&s);
    let mut stepper = Stepper::new(steps(HOUSE));
    assert_eq!((stepper.position(), stepper.is_done()), (0, false));

    // Each edit is one undo step; the query makes none.
    let depths: Vec<usize> = (0..4)
        .map(|_| {
            stepper.next(&mut s).unwrap();
            s.doc.undo_depth()
        })
        .collect();
    assert_eq!(depths, [1, 2, 2, 3]);
    assert!(stepper.is_done());
    assert_eq!(stepper.results().len(), 4);
    // The window is in the wall the first step made.
    let wall = stepper.results()[0]["id"].clone();
    let window = s.execute("element_get", json!({ "id": stepper.results()[3]["id"] })).unwrap();
    assert_eq!(window["params"]["host"], wall, "{window:#}");
    assert!(stepper.next(&mut s).is_err(), "no step is left");

    // Back over the window, then over the query (nothing to undo).
    stepper.back(&mut s).unwrap();
    assert_eq!((stepper.position(), s.doc.undo_depth()), (3, 2));
    stepper.back(&mut s).unwrap();
    assert_eq!((stepper.position(), s.doc.undo_depth()), (2, 2));
    // Forward again: the window still finds its wall.
    stepper.run_to(&mut s, 4).unwrap();
    assert_eq!(s.doc.undo_depth(), 3);

    assert_eq!(stepper.restart(&mut s), Ok(4));
    assert_eq!(stepper.position(), 0);
    assert_eq!(s.doc.undo_depth(), 0);
    assert_eq!(state(&s), before, "stepping back restores the project");
    assert!(stepper.back(&mut s).is_err(), "nothing to step back over at the start");
}

#[test]
fn stepping_back_after_another_edit_is_refused() {
    let mut s = Session::default();
    let mut stepper = Stepper::new(steps(HOUSE));
    stepper.run_to(&mut s, 2).unwrap();
    s.execute("project_info_set", json!({ "name": "Changed" })).unwrap();
    let depth = s.doc.undo_depth();
    let err = stepper.back(&mut s).unwrap_err();
    assert!(!err.message.is_empty());
    assert_eq!((stepper.position(), s.doc.undo_depth()), (2, depth), "nothing changed");
    assert_eq!(s.project().info.name, "Changed");
    // Undoing the other edit makes the step the top again.
    s.doc.undo();
    stepper.back(&mut s).unwrap();
    assert_eq!(stepper.position(), 1);

    // An undo and another edit in its place: the depth is the same, the step is not.
    s.doc.undo();
    s.execute("project_info_set", json!({ "name": "Other" })).unwrap();
    assert_eq!(s.doc.undo_depth(), 1);
    assert!(stepper.back(&mut s).is_err());
    assert_eq!(stepper.position(), 1);
    assert_eq!(stepper.restart(&mut s).map_err(|(n, _)| n), Err(0));
}

#[test]
fn a_squashed_run_is_one_undo_step() {
    let mut s = Session::default();
    let before = state(&s);
    let mut stepper = Stepper::new(steps(HOUSE));
    stepper.next(&mut s).unwrap();
    stepper.run_squashed(&mut s, "macro").unwrap();
    assert!(stepper.is_done());
    assert_eq!(s.doc.undo_depth(), 2, "the first step, then the rest as one");
    assert_eq!(s.doc.undo_name(), Some("macro"));
    assert_eq!(s.project().walls().count(), 2);

    // Stepping back undoes the run whole.
    stepper.back(&mut s).unwrap();
    assert_eq!((stepper.position(), s.doc.undo_depth()), (1, 1));
    assert_eq!(s.project().walls().count(), 1);

    // Run it all as one step: one undo restores the start.
    stepper.restart(&mut s).unwrap();
    stepper.run_squashed(&mut s, "macro").unwrap();
    assert_eq!(s.doc.undo_depth(), 1);
    assert!(s.doc.undo());
    assert_eq!(state(&s), before);
}

#[test]
fn a_failing_step_stops_the_run_and_can_be_stepped_back_to() {
    let mut s = Session::default();
    let text = "{\"command\": \"wall_create\", \"args\": {\"start\": [0, 0], \"end\": [6, 0]}}\n\
                {\"command\": \"window_create\", \"args\": {\"wall\": {\"$step\": 1, \"path\": \"/nothing\"}}}\n";
    let mut stepper = Stepper::new(steps(text));
    let err = stepper.run_to(&mut s, 2).unwrap_err();
    assert_eq!((err.index, err.command.as_str()), (1, "window_create"));
    assert_eq!(stepper.position(), 1);
    // The same through `script::run`, which reports the results so far.
    let mut other = Session::default();
    let (results, err) = script::run(&mut other, &steps(text)).unwrap_err();
    assert_eq!((results.len(), err.index), (1, 1));
    // A squashed run that fails keeps what ran as one step.
    let mut third = Session::default();
    let mut squashed = Stepper::new(steps(text));
    assert!(squashed.run_squashed(&mut third, "macro").is_err());
    assert_eq!((squashed.position(), third.doc.undo_depth()), (1, 1));
    squashed.back(&mut third).unwrap();
    assert_eq!(third.doc.undo_depth(), 0);
}

/// The project as saved, less the id counter (ids are never given twice,
/// so undoing does not take them back).
fn state(s: &Session) -> String {
    let ron = cc_core::to_ron(s.project()).unwrap();
    ron.lines().filter(|l| !l.trim_start().starts_with("next_id:")).collect::<Vec<_>>().join("\n")
}
