//! Levels in elevations and sections (issue #453): the view's graphics hide
//! them by category, `level_head_set` hides one level in one view, moves
//! its heads to either end and lifts a head off its line with an elbow, so
//! levels close together stay readable; deleting a level drops its settings.

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_core::level_head::LevelHeads;
use cc_core::{LevelId, ViewId};
use cc_engine::Session;
use cc_views::Prim;
use serde_json::{Value, json};

fn run(s: &mut Session, name: &str, args: Value) -> Value {
    s.execute(name, args).unwrap_or_else(|e| panic!("{name}: {e:?}"))
}

fn elevation(s: &mut Session) -> ViewId {
    let views = run(s, "views_list", json!({"category": "elevation"}));
    ViewId(views[0]["id"].as_u64().unwrap())
}

fn texts(s: &mut Session, view: ViewId) -> Vec<String> {
    let d = s.drawing(view).unwrap();
    d.prims
        .iter()
        .filter_map(|p| match p {
            Prim::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn the_levels_category_hides_the_levels_of_a_view() {
    let mut s = Session::default();
    let view = elevation(&mut s);
    let name = s.project().levels[0].name.clone();
    assert!(texts(&mut s, view).contains(&name));
    run(&mut s, "view_graphics_set", json!({"view": view.0, "category": "level", "visible": false}));
    assert!(!texts(&mut s, view).contains(&name));
    assert!(s.drawing(view).unwrap().levels.is_empty());
    run(&mut s, "undo", json!({}));
    assert!(texts(&mut s, view).contains(&name));
}

#[test]
fn a_level_hides_in_one_view_and_its_heads_move() {
    let mut s = Session::default();
    let view = elevation(&mut s);
    let (first, second) = (s.project().levels[0].clone(), s.project().levels[1].clone());
    let out = run(&mut s, "level_head_set", json!({"view": view.0, "levels": [first.name], "hidden": true}));
    assert_eq!(out["levels"][0]["hidden"], true);
    let shown = texts(&mut s, view);
    assert!(!shown.contains(&first.name) && shown.contains(&second.name));
    // Back again, with heads at both ends and the end head lifted 0.6 m.
    let out = run(
        &mut s,
        "level_head_set",
        json!({"view": view.0, "levels": [first.id.0], "hidden": false, "heads": "both", "elbow": 0.6}),
    );
    assert_eq!(out["levels"][0]["heads"], "both");
    let d = s.drawing(view).unwrap();
    let handle = d.levels.iter().find(|l| l.level == first.id).unwrap();
    assert_eq!(handle.heads.len(), 2);
    let end = handle.heads.iter().find(|h| !h.start).unwrap();
    assert!((end.point.y - end.tip.y - 0.6).abs() < 1e-9);
    let head = &s.project().view(view).unwrap().level_heads[&first.id];
    assert_eq!(head.heads, LevelHeads::Both);
    // view_get lists every level as the view shows it.
    let got = run(&mut s, "view_get", json!({"view": view.0}));
    let levels = got["level_heads"].as_array().unwrap();
    assert_eq!(levels.len(), s.project().levels.len());
    // reset puts it back as by default, and the view keeps nothing for it.
    run(&mut s, "level_head_set", json!({"view": view.0, "levels": [first.id.0], "reset": true}));
    assert!(s.project().view(view).unwrap().level_heads.is_empty());
}

#[test]
fn level_head_set_refuses_what_it_cannot_do() {
    let mut s = Session::default();
    let view = elevation(&mut s);
    let level = s.project().levels[0].id.0;
    let plans = run(&mut s, "views_list", json!({"category": "floor_plan"}));
    let plan = plans[0]["id"].as_u64().unwrap();
    for args in [
        json!({"view": plan, "levels": [level], "hidden": true}),
        json!({"view": view.0, "levels": [], "hidden": true}),
        json!({"view": view.0, "levels": [level], "heads": "left"}),
        json!({"view": view.0, "levels": [level], "elbow": 1e9}),
        json!({"view": view.0, "levels": [987654], "hidden": true}),
    ] {
        assert!(s.execute("level_head_set", args.clone()).is_err(), "{args}");
    }
    assert!(s.project().view(view).unwrap().level_heads.is_empty());
}

#[test]
fn deleting_a_level_drops_its_heads() {
    let mut s = Session::default();
    let view = elevation(&mut s);
    let made = run(&mut s, "level_create", json!({"name": "Roof", "plans": false}));
    let level = LevelId(made["id"].as_u64().unwrap());
    run(&mut s, "level_head_set", json!({"view": view.0, "levels": [level.0], "heads": "start"}));
    assert!(s.project().view(view).unwrap().level_heads.contains_key(&level));
    run(&mut s, "level_delete", json!({"level": level.0}));
    assert!(s.project().view(view).unwrap().level_heads.is_empty());
}
