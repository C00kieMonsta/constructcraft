//! Data plugins keep in the project (`plugin_data_get`, `plugin_data_set`,
//! `plugin_data_list`): per element and per project, by plugin id, saved in
//! the file, undoable and repaired on load.

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_core::ElementId;
use cc_core::plugin_data::MAX_PLUGIN_DATA_BYTES;
use cc_engine::{ErrorCode, Session};
use serde_json::{Value, json};

fn run(s: &mut Session, name: &str, args: Value) -> Value {
    s.execute(name, args).unwrap_or_else(|e| panic!("{name}: {e:?}"))
}

fn fails(s: &mut Session, name: &str, args: Value) -> ErrorCode {
    s.execute(name, args).map(|v| panic!("{name} should fail, gave {v}")).unwrap_err().code
}

fn wall(s: &mut Session, x: f64) -> u64 {
    run(s, "wall_create", json!({"start": [x, 0], "end": [x + 4.0, 0]}))["id"].as_u64().unwrap()
}

fn get(s: &mut Session, plugin: &str, element: Option<u64>) -> Value {
    match element {
        Some(e) => run(s, "plugin_data_get", json!({"plugin": plugin, "element": e})),
        None => run(s, "plugin_data_get", json!({"plugin": plugin})),
    }
}

#[test]
fn set_get_and_remove_on_the_project_and_elements() {
    let mut s = Session::default();
    let w = wall(&mut s, 0.0);
    assert_eq!(get(&mut s, "rooms", None), Value::Null);
    assert_eq!(get(&mut s, "rooms", Some(w)), Value::Null);

    let project_data = json!({"version": 2, "zones": ["a", "b"], "ratio": 0.5});
    run(&mut s, "plugin_data_set", json!({"plugin": "rooms", "data": project_data}));
    let out = run(&mut s, "plugin_data_set", json!({"plugin": "rooms", "element": w, "data": [1, {"x": true}]}));
    assert_eq!(out, json!({"plugin": "rooms", "element": w, "stored": true}));
    assert_eq!(get(&mut s, "rooms", None), project_data);
    assert_eq!(get(&mut s, "rooms", Some(w)), json!([1, {"x": true}]));
    // Plugins do not see each other's data; a scalar is a value too.
    assert_eq!(get(&mut s, "other", Some(w)), Value::Null);
    run(&mut s, "plugin_data_set", json!({"plugin": "other", "element": w, "data": "text"}));
    assert_eq!(get(&mut s, "other", Some(w)), json!("text"));
    assert_eq!(get(&mut s, "rooms", Some(w)), json!([1, {"x": true}]));

    // Replaced, then removed with null.
    run(&mut s, "plugin_data_set", json!({"plugin": "rooms", "element": w, "data": {"n": 3}}));
    assert_eq!(get(&mut s, "rooms", Some(w)), json!({"n": 3}));
    let out = run(&mut s, "plugin_data_set", json!({"plugin": "rooms", "element": w, "data": null}));
    assert_eq!(out["stored"], false);
    assert_eq!(get(&mut s, "rooms", Some(w)), Value::Null);
    run(&mut s, "plugin_data_set", json!({"plugin": "rooms", "data": null}));
    assert_eq!(get(&mut s, "rooms", None), Value::Null);
    assert!(s.project().plugin_data.is_empty());
    assert_eq!(s.project().element(ElementId(w)).unwrap().plugin_data.len(), 1);
}

#[test]
fn each_set_is_one_undo_step() {
    let mut s = Session::default();
    let w = wall(&mut s, 0.0);
    run(&mut s, "plugin_data_set", json!({"plugin": "rooms", "data": {"a": 1}}));
    run(&mut s, "plugin_data_set", json!({"plugin": "rooms", "data": {"a": 2}}));
    run(&mut s, "plugin_data_set", json!({"plugin": "rooms", "element": w, "data": 7}));

    let out = run(&mut s, "undo", json!({}));
    assert!(out["undone"].as_str().is_some_and(|n| !n.is_empty()), "{out}");
    assert_eq!(get(&mut s, "rooms", Some(w)), Value::Null);
    assert_eq!(get(&mut s, "rooms", None), json!({"a": 2}));
    run(&mut s, "undo", json!({}));
    assert_eq!(get(&mut s, "rooms", None), json!({"a": 1}));
    run(&mut s, "undo", json!({}));
    assert_eq!(get(&mut s, "rooms", None), Value::Null);
    // The wall is still there: only the data was undone.
    assert!(s.project().element(ElementId(w)).is_some());
    run(&mut s, "redo", json!({}));
    run(&mut s, "redo", json!({}));
    run(&mut s, "redo", json!({}));
    assert_eq!(get(&mut s, "rooms", None), json!({"a": 2}));
    assert_eq!(get(&mut s, "rooms", Some(w)), json!(7));

    // Removing is undone too.
    run(&mut s, "plugin_data_set", json!({"plugin": "rooms", "element": w, "data": null}));
    run(&mut s, "undo", json!({}));
    assert_eq!(get(&mut s, "rooms", Some(w)), json!(7));
}

#[test]
fn bad_plugin_ids_unknown_elements_and_large_data_are_refused() {
    let mut s = Session::default();
    let w = wall(&mut s, 0.0);
    let revision = s.doc.revision();
    for bad in ["", "Rooms", "1rooms", "my-plugin", "a__b", "rooms_", &"a".repeat(33)] {
        assert_eq!(fails(&mut s, "plugin_data_set", json!({"plugin": bad, "data": 1})), ErrorCode::InvalidArguments);
        assert_eq!(fails(&mut s, "plugin_data_get", json!({"plugin": bad})), ErrorCode::InvalidArguments);
        assert_eq!(fails(&mut s, "plugin_data_list", json!({"plugin": bad})), ErrorCode::InvalidArguments);
    }
    assert_eq!(
        fails(&mut s, "plugin_data_set", json!({"plugin": "rooms", "element": 999_999, "data": 1})),
        ErrorCode::NotFound
    );
    assert_eq!(fails(&mut s, "plugin_data_get", json!({"plugin": "rooms", "element": 999_999})), ErrorCode::NotFound);
    // `data` is required (null removes, but it must be said).
    assert_eq!(fails(&mut s, "plugin_data_set", json!({"plugin": "rooms"})), ErrorCode::InvalidArguments);

    // At most the limit of compact JSON text: a string of n characters takes n + 2.
    let largest = "x".repeat(MAX_PLUGIN_DATA_BYTES - 2);
    run(&mut s, "plugin_data_set", json!({"plugin": "rooms", "element": w, "data": largest}));
    assert_eq!(get(&mut s, "rooms", Some(w)).as_str().map(str::len), Some(MAX_PLUGIN_DATA_BYTES - 2));
    let revision_after = s.doc.revision();
    assert!(revision_after > revision);
    let too_large = "x".repeat(MAX_PLUGIN_DATA_BYTES - 1);
    assert_eq!(
        fails(&mut s, "plugin_data_set", json!({"plugin": "rooms", "data": too_large})),
        ErrorCode::InvalidArguments
    );
    // Nested at most 64 deep.
    let nested = |n: usize| serde_json::from_str::<Value>(&format!("{}{}", "[".repeat(n), "]".repeat(n))).unwrap();
    assert_eq!(
        fails(&mut s, "plugin_data_set", json!({"plugin": "rooms", "data": nested(65)})),
        ErrorCode::InvalidArguments
    );
    assert_eq!(s.doc.revision(), revision_after, "a refused set changes nothing");
    run(&mut s, "plugin_data_set", json!({"plugin": "deep", "data": nested(64)}));
    assert_eq!(get(&mut s, "deep", None), nested(64));
    assert_eq!(get(&mut s, "rooms", None), Value::Null);
}

#[test]
fn list_tells_where_a_plugin_keeps_data() {
    let mut s = Session::default();
    let ids: Vec<u64> = (0..4).map(|k| wall(&mut s, 10.0 * f64::from(k))).collect();
    assert_eq!(run(&mut s, "plugin_data_list", json!({"plugin": "rooms"})), json!({"project": false, "elements": []}));
    for &w in [ids[3], ids[0], ids[2]].iter() {
        run(&mut s, "plugin_data_set", json!({"plugin": "rooms", "element": w, "data": {"w": w}}));
    }
    run(&mut s, "plugin_data_set", json!({"plugin": "other", "element": ids[1], "data": 1}));
    assert_eq!(
        run(&mut s, "plugin_data_list", json!({"plugin": "rooms"})),
        json!({"project": false, "elements": [ids[0], ids[2], ids[3]]})
    );
    run(&mut s, "plugin_data_set", json!({"plugin": "rooms", "data": true}));
    assert_eq!(
        run(&mut s, "plugin_data_list", json!({"plugin": "rooms"})),
        json!({"project": true, "elements": [ids[0], ids[2], ids[3]]})
    );
    assert_eq!(
        run(&mut s, "plugin_data_list", json!({"plugin": "other"})),
        json!({"project": false, "elements": [ids[1]]})
    );
    // Deleting an element takes its data with it.
    run(&mut s, "elements_delete", json!({"ids": [ids[2]]}));
    assert_eq!(run(&mut s, "plugin_data_list", json!({"plugin": "rooms"}))["elements"], json!([ids[0], ids[3]]));
}

#[test]
fn data_survives_saving_and_opening() {
    let mut s = Session::default();
    let w = wall(&mut s, 0.0);
    let deep = json!({"a": {"b": {"c": {"d": {"e": {"f": {"g": {"h": {"i": [1, 2.5, "\"quoted\"\n", null]}}}}}}}}});
    run(&mut s, "plugin_data_set", json!({"plugin": "rooms", "data": deep}));
    run(&mut s, "plugin_data_set", json!({"plugin": "rooms", "element": w, "data": {"label": "Sala ção"}}));
    let dir = std::env::temp_dir().join(format!("ccad-plugin-data-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("data.ccad");
    run(&mut s, "project_save", json!({"path": path}));

    let mut reopened = Session::default();
    let out = run(&mut reopened, "project_open", json!({"path": path}));
    assert_eq!(out["repairs"], 0, "{out}");
    assert_eq!(get(&mut reopened, "rooms", None), deep);
    assert_eq!(get(&mut reopened, "rooms", Some(w)), json!({"label": "Sala ção"}));
    let _ = std::fs::remove_dir_all(&dir);

    // A project without plugin data writes no field for it.
    let text = cc_core::to_ron(&Session::default().doc.project).unwrap();
    assert!(!text.contains("plugin_data"));
}

#[test]
fn opening_drops_bad_entries() {
    let mut s = Session::default();
    let w = wall(&mut s, 0.0);
    run(&mut s, "plugin_data_set", json!({"plugin": "rooms", "data": {"ok": 1}}));
    run(&mut s, "plugin_data_set", json!({"plugin": "rooms", "element": w, "data": {"ok": 2}}));
    // Crafted entries a command would refuse: a bad id, text that is no
    // JSON and a value over the limit.
    let mut p = s.doc.project.clone();
    p.plugin_data.insert("Not-An-Id".into(), "1".into());
    p.plugin_data.insert("broken".into(), "{\"a\":".into());
    let e = p.elements.get_mut(&ElementId(w)).unwrap();
    e.plugin_data.insert("huge".into(), format!("\"{}\"", "x".repeat(MAX_PLUGIN_DATA_BYTES)));
    e.plugin_data.insert("a__b".into(), "2".into());
    let text = cc_core::to_ron(&p).unwrap();

    let (opened, repairs) = cc_core::from_ron_with_repairs(&text).unwrap();
    assert_eq!(repairs, 4);
    assert_eq!(opened.plugin_data.keys().collect::<Vec<_>>(), ["rooms"]);
    assert_eq!(opened.element(ElementId(w)).unwrap().plugin_data.keys().collect::<Vec<_>>(), ["rooms"]);

    // Through the engine: the same count is reported.
    let dir = std::env::temp_dir().join(format!("ccad-plugin-data-bad-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("bad.ccad");
    std::fs::write(&path, &text).unwrap();
    let mut reopened = Session::default();
    let out = run(&mut reopened, "project_open", json!({"path": path}));
    assert_eq!(out["repairs"], 4, "{out}");
    assert_eq!(get(&mut reopened, "rooms", Some(w)), json!({"ok": 2}));
    assert_eq!(get(&mut reopened, "broken", None), Value::Null);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn copies_carry_their_data() {
    let mut s = Session::default();
    let w = wall(&mut s, 0.0);
    run(&mut s, "plugin_data_set", json!({"plugin": "rooms", "element": w, "data": {"tag": "A"}}));
    let out = run(&mut s, "elements_copy", json!({"ids": [w], "delta": [0, 5], "count": 2}));
    let copies: Vec<u64> = out["created"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap()).collect();
    assert_eq!(copies.len(), 2);
    for c in &copies {
        assert_eq!(get(&mut s, "rooms", Some(*c)), json!({"tag": "A"}));
    }
    // Pasting an exported set carries it too.
    let set = run(&mut s, "elements_export", json!({"ids": [w]}));
    let out = run(&mut s, "elements_paste", json!({"elements": set["elements"], "delta": [0, 20]}));
    let pasted = out["created"][0].as_u64().unwrap();
    assert_eq!(get(&mut s, "rooms", Some(pasted)), json!({"tag": "A"}));
    // Each copy's data is its own.
    run(&mut s, "plugin_data_set", json!({"plugin": "rooms", "element": copies[0], "data": {"tag": "B"}}));
    assert_eq!(get(&mut s, "rooms", Some(w)), json!({"tag": "A"}));
}
