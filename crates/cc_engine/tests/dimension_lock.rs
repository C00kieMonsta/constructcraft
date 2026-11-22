//! Locked dimensions (#447): moving the element at one end of a locked
//! dimension moves the element at the other, along a chain of them, in one
//! undo step; edits that would still change a locked dimension are refused.

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_engine::{ErrorCode, Session};
use serde_json::{Value, json};

fn run(s: &mut Session, name: &str, args: Value) -> Value {
    s.execute(name, args).unwrap_or_else(|e| panic!("{name}: {e:?}"))
}

fn fails(s: &mut Session, name: &str, args: Value) -> (ErrorCode, String) {
    let e = s.execute(name, args).map(|v| panic!("{name} should fail, gave {v}")).unwrap_err();
    (e.code, e.message)
}

fn id(v: &Value) -> u64 {
    v["id"].as_u64().unwrap_or_else(|| panic!("no id in {v}"))
}

fn x_of(s: &mut Session, column: u64) -> f64 {
    run(s, "element_get", json!({"id": column}))["params"]["position"][0].as_f64().unwrap()
}

fn value(s: &mut Session, dim: u64) -> f64 {
    let d = run(s, "element_get", json!({"id": dim}));
    (d["params"]["b"][0].as_f64().unwrap() - d["params"]["a"][0].as_f64().unwrap()).abs()
}

/// Three 20 cm columns in a row at x = 0, 4 and 7, and the dimensions
/// between the faces of each pair (3.8 m and 2.8 m) in the ground plan.
fn row() -> (Session, [u64; 3], [u64; 2]) {
    let mut s = Session::default();
    let plan = run(&mut s, "views_list", json!({"category": "floor_plan"}))[0]["id"].clone();
    let columns = [0.0, 4.0, 7.0].map(|x| id(&run(&mut s, "column_create", json!({"position": [x, 0.0]}))));
    let dims = [(0.1, 3.9), (4.1, 6.9)].map(|(a, b)| {
        let d = run(&mut s, "dimension_create", json!({"view": plan, "a": [a, 0.0], "b": [b, 0.0], "kind": "linear"}));
        let refs = &d["params"]["refs"];
        assert!(!refs[0].is_null() && !refs[1].is_null(), "both ends on columns: {refs}");
        id(&d)
    });
    (s, columns, dims)
}

#[test]
fn moving_one_end_of_a_locked_dimension_moves_the_other() {
    let (mut s, [a, b, c], [ab, bc]) = row();
    run(&mut s, "dimension_lock", json!({"ids": [ab]}));
    let locked = run(&mut s, "element_get", json!({"id": ab}))["params"]["locked"].as_f64().unwrap();
    assert!((locked - 3.8).abs() < 1e-9, "{locked}");
    let depth = s.doc.undo_depth();
    run(&mut s, "elements_move", json!({"ids": [a], "delta": [1.0, 0.0]}));
    assert_eq!(s.doc.undo_depth(), depth + 1, "one undo step");
    assert!((x_of(&mut s, b) - 5.0).abs() < 1e-9 && (value(&mut s, ab) - 3.8).abs() < 1e-9);
    // The unlocked dimension just follows: its column stays.
    assert!((x_of(&mut s, c) - 7.0).abs() < 1e-9 && (value(&mut s, bc) - 1.8).abs() < 1e-9);
    // Moving the other end drags the first column back the other way.
    run(&mut s, "elements_move", json!({"ids": [b], "delta": [-0.5, 0.0]}));
    assert!((x_of(&mut s, a) - 0.5).abs() < 1e-9, "{}", x_of(&mut s, a));
    // Moving both ends together keeps the value; undo puts everything back.
    run(&mut s, "elements_move", json!({"ids": [a, b], "delta": [0.0, 2.0]}));
    assert!((value(&mut s, ab) - 3.8).abs() < 1e-9);
    for _ in 0..3 {
        run(&mut s, "undo", json!({}));
    }
    assert!((x_of(&mut s, a)).abs() < 1e-9 && (x_of(&mut s, b) - 4.0).abs() < 1e-9);
}

#[test]
fn a_chain_of_locked_dimensions_moves_together_and_takes_new_values() {
    let (mut s, [a, b, c], [ab, bc]) = row();
    run(&mut s, "dimension_lock", json!({"ids": [ab, bc]}));
    run(&mut s, "elements_move", json!({"ids": [a], "delta": [-1.0, 0.0]}));
    assert!((x_of(&mut s, b) - 3.0).abs() < 1e-9 && (x_of(&mut s, c) - 6.0).abs() < 1e-9);
    // A new value moves the far side, and the chain after it.
    run(&mut s, "dimension_set_value", json!({"id": ab, "value": 5.0}));
    assert!((x_of(&mut s, b) - 4.2).abs() < 1e-9 && (x_of(&mut s, c) - 7.2).abs() < 1e-9);
    assert_eq!(run(&mut s, "element_get", json!({"id": ab}))["params"]["locked"], json!(5.0));
    assert!((value(&mut s, bc) - 2.8).abs() < 1e-9);
}

#[test]
fn edits_that_would_break_a_lock_are_refused() {
    let (mut s, [a, b, c], [ab, bc]) = row();
    run(&mut s, "dimension_lock", json!({"ids": [ab, bc]}));
    // The far column of the chain is pinned: moving the first one cannot keep the second dimension.
    run(&mut s, "elements_pin", json!({"ids": [c], "pinned": true}));
    let (code, message) = fails(&mut s, "elements_move", json!({"ids": [a], "delta": [1.0, 0.0]}));
    assert_eq!(code, ErrorCode::Rejected);
    assert!(message.contains("locked") || message.contains("trav"), "{message}");
    assert!(x_of(&mut s, a).abs() < 1e-9 && (x_of(&mut s, b) - 4.0).abs() < 1e-9, "nothing moved");
    // Moving both ends of a locked dimension apart is refused too.
    run(&mut s, "elements_pin", json!({"ids": [c], "pinned": false}));
    run(&mut s, "dimension_lock", json!({"ids": [bc], "locked": false}));
    let pair = json!({"ids": [a, c], "delta": [1.0, 0.0]});
    run(&mut s, "elements_move", pair);
    assert!((x_of(&mut s, b) - 5.0).abs() < 1e-9, "b follows a; bc is free");
    // Only linear or aligned dimensions in plans with both ends on elements lock.
    let plan = run(&mut s, "views_list", json!({"category": "floor_plan"}))[0]["id"].clone();
    let free = id(&run(&mut s, "dimension_create", json!({"view": plan, "a": [20.0, 20.0], "b": [23.0, 20.0]})));
    assert_eq!(fails(&mut s, "dimension_lock", json!({"ids": [free]})).0, ErrorCode::Rejected);
    assert_eq!(fails(&mut s, "dimension_lock", json!({"ids": [a]})).0, ErrorCode::InvalidArguments);
}

#[test]
fn a_lock_is_saved_and_unlocking_frees_the_ends() {
    let (mut s, [a, b, _], [ab, _]) = row();
    run(&mut s, "dimension_lock", json!({"ids": [ab]}));
    let path = std::env::temp_dir().join(format!("ccad-dimension-lock-{}.ccad", std::process::id()));
    run(&mut s, "project_save", json!({"path": path}));
    let mut back = Session::default();
    run(&mut back, "project_open", json!({"path": path}));
    let _ = std::fs::remove_file(&path);
    run(&mut back, "elements_move", json!({"ids": [a], "delta": [1.0, 0.0]}));
    assert!((x_of(&mut back, b) - 5.0).abs() < 1e-9, "still locked after reopening");
    run(&mut back, "dimension_lock", json!({"ids": [ab], "locked": false}));
    assert!(run(&mut back, "element_get", json!({"id": ab}))["params"]["locked"].is_null());
    run(&mut back, "elements_move", json!({"ids": [a], "delta": [1.0, 0.0]}));
    assert!((x_of(&mut back, b) - 5.0).abs() < 1e-9, "b stays");
}
