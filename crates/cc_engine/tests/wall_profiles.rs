//! Wall elevation profiles (issue #306): the outline of a wall's face in its
//! own elevation plane, so its base and top need not be level.

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_core::{ElementId, ViewId};
use cc_engine::{ErrorCode, Session};
use cc_views::{LineStyle, Prim};
use glam::DVec2;
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

/// A wall 6 m long and 3 m high along +x from the origin.
fn wall(s: &mut Session) -> u64 {
    id(&run(s, "wall_create", json!({"start": [0, 0], "end": [6, 0], "height": 3.0})))
}

fn derived(s: &mut Session, id: u64) -> Value {
    run(s, "element_get", json!({"id": id}))["derived"].clone()
}

fn num(v: &Value) -> f64 {
    v.as_f64().unwrap_or_else(|| panic!("not a number: {v}"))
}

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 1e-6, "{what}: {a} vs {b}");
}

/// A sloped top, 3 m at the start and 2 m at the end.
fn slope(s: &mut Session, wall: u64) -> Value {
    run(s, "wall_profile_slope", json!({"wall": wall, "top_end": 2.0}))
}

#[test]
fn a_sloped_top_in_one_step() {
    let mut s = Session::default();
    let w = wall(&mut s);
    let flat = num(&derived(&mut s, w)["volume"]);
    let thickness = num(&derived(&mut s, w)["thickness"]);
    let steps = s.doc.undo_depth();
    let out = slope(&mut s, w);
    assert_eq!(s.doc.undo_depth(), steps + 1, "one undo step");
    // element_get: the profile in `params`, its figures in `derived`.
    assert!(out["params"]["profile"]["outline"].as_array().is_some_and(|a| a.len() == 4), "{out}");
    let p = &out["derived"]["profile"];
    close(num(&p["area"]), 15.0, "face area");
    close(num(&p["min_z"]), 0.0, "lowest");
    close(num(&p["max_z"]), 3.0, "highest");
    close(num(&p["start"]), 0.0, "start");
    close(num(&p["end"]), 6.0, "end");
    close(num(&out["derived"]["volume"]), 15.0 * thickness, "volume");
    // The nominal height stays.
    close(num(&out["derived"]["height"]), 3.0, "height");
    run(&mut s, "undo", json!({}));
    close(num(&derived(&mut s, w)["volume"]), flat, "undone");
    assert!(run(&mut s, "element_get", json!({"id": w}))["params"].get("profile").is_none());
    // A base above the top is refused.
    let (code, _) = fails(&mut s, "wall_profile_slope", json!({"wall": w, "base_end": 3.5}));
    assert_eq!(code, ErrorCode::InvalidArguments);
}

#[test]
fn curves_points_and_removal() {
    let mut s = Session::default();
    let w = wall(&mut s);
    // A gable end: up to 4.5 m in the middle.
    let gable = json!([[0, 0], [6, 0], [6, 3], [3, 4.5], [0, 3]]);
    let out = run(&mut s, "wall_profile_set", json!({"wall": w, "points": gable}));
    close(num(&out["derived"]["profile"]["area"]), 18.0 + 4.5, "gable");
    // An arched top drawn with curves: a half circle of 3 m radius over 3 m.
    let arch = json!([
        {"kind": "line", "start": [0, 0], "end": [6, 0]},
        {"kind": "line", "start": [6, 0], "end": [6, 3]},
        {"kind": "arc", "start": [6, 3], "end": [0, 3], "bulge": 1.0},
        {"kind": "line", "start": [0, 3], "end": [0, 0]},
    ]);
    let out = run(&mut s, "wall_profile_set", json!({"wall": w, "curves": arch}));
    let exact = 18.0 + std::f64::consts::PI * 9.0 / 2.0;
    close(num(&out["derived"]["profile"]["area"]), exact, "arched area, exact");
    let got = run(&mut s, "sketch_get", json!({"id": w}));
    assert_eq!(
        (got["frame"].as_str(), got["drawn"].as_bool(), got["profile"].as_bool()),
        (Some("wall_elevation"), Some(true), Some(true))
    );
    assert!(got["curves"].as_array().unwrap().iter().any(|c| c["kind"] == "arc"), "{got}");
    // Removed: level again; removing twice is refused.
    run(&mut s, "wall_profile_remove", json!({"wall": w}));
    assert!(derived(&mut s, w).get("profile").is_none());
    assert_eq!(fails(&mut s, "wall_profile_remove", json!({"wall": w})).0, ErrorCode::InvalidArguments);
    // Without a profile sketch_get gives the wall's box.
    let got = run(&mut s, "sketch_get", json!({"id": w}));
    assert_eq!((got["drawn"].as_bool(), got["profile"].as_bool()), (Some(false), Some(false)));
    assert_eq!(got["curves"].as_array().unwrap().len(), 4);
    // sketch_set draws the profile too.
    run(
        &mut s,
        "sketch_set",
        json!({"id": w, "curves": [{"kind": "rectangle", "corner": [1, 0.5], "opposite": [5, 2.5]}]}),
    );
    close(num(&derived(&mut s, w)["profile"]["area"]), 8.0, "rectangle");
    // Both curves and points, or neither, is refused.
    assert_eq!(fails(&mut s, "wall_profile_set", json!({"wall": w})).0, ErrorCode::InvalidArguments);
}

#[test]
fn outlines_that_are_no_profile_are_refused_with_the_point() {
    let mut s = Session::default();
    let w = wall(&mut s);
    // A C open to the end: a vertical line crosses it four times.
    let c = json!([[0, 0], [4, 0], [4, 1], [1, 1], [1, 2], [4, 2], [4, 3], [0, 3]]);
    let (code, message) = fails(&mut s, "wall_profile_set", json!({"wall": w, "points": c}));
    assert_eq!(code, ErrorCode::InvalidArguments);
    assert!(message.contains("more than twice") && message.contains("u = 1.000"), "{message}");
    // Crossing itself, past the wall's end, a hole.
    let bow = json!([[0, 0], [4, 3], [4, 0], [0, 3]]);
    assert!(fails(&mut s, "wall_profile_set", json!({"wall": w, "points": bow})).1.contains("cross"));
    let long = json!([[0, 0], [7, 0], [7, 3], [0, 3]]);
    assert!(fails(&mut s, "wall_profile_set", json!({"wall": w, "points": long})).1.contains("past the wall's ends"));
    let holed = json!([
        {"kind": "rectangle", "corner": [0, 0], "opposite": [6, 3]},
        {"kind": "rectangle", "corner": [1, 1], "opposite": [2, 2]},
    ]);
    assert!(fails(&mut s, "wall_profile_set", json!({"wall": w, "curves": holed})).1.contains("no holes"));
    // An open outline.
    let open =
        json!([{"kind": "line", "start": [0, 0], "end": [6, 0]}, {"kind": "line", "start": [6, 0], "end": [6, 3]}]);
    assert_eq!(fails(&mut s, "wall_profile_set", json!({"wall": w, "curves": open})).0, ErrorCode::InvalidArguments);
    // Nothing changed.
    assert!(derived(&mut s, w).get("profile").is_none());
}

#[test]
fn arc_curtain_attached_and_framed_walls_take_no_profile() {
    let mut s = Session::default();
    let arc = id(&run(&mut s, "wall_create", json!({"start": [0, 0], "end": [6, 0], "bulge": 0.3})));
    assert!(fails(&mut s, "wall_profile_slope", json!({"wall": arc, "top_end": 2.0})).1.contains("curved"));
    let curtain = id(&run(&mut s, "curtain_type_create", json!({"name": "CW"})));
    let cw = id(&run(&mut s, "wall_create", json!({"start": [0, 5], "end": [6, 5], "type": curtain})));
    assert!(fails(&mut s, "wall_profile_slope", json!({"wall": cw, "top_end": 2.0})).1.contains("curtain"));
    let profiled = json!([{"kind": "rectangle", "corner": [0, 0], "opposite": [6, 2]}]);
    assert!(
        s.execute("wall_create", json!({"start": [0, 9], "end": [6, 9], "type": curtain, "profile": profiled}))
            .is_err()
    );
    // Attached to a roof, the top is the roof's; with a profile, attaching is refused.
    let w = wall(&mut s);
    let roof = id(&run(&mut s, "roof_create", json!({"boundary": [[0, -1], [6, -1], [6, 1], [0, 1]], "offset": 3.0})));
    run(&mut s, "wall_attach", json!({"ids": [w], "top": roof}));
    assert!(fails(&mut s, "wall_profile_slope", json!({"wall": w, "top_end": 2.0})).1.contains("attached"));
    run(&mut s, "wall_attach", json!({"ids": [w], "detach": true}));
    slope(&mut s, w);
    assert!(fails(&mut s, "wall_attach", json!({"ids": [w], "top": roof})).1.contains("elevation profile"));
    // A patch that would give it both is refused by the model's checks.
    assert!(s.execute("element_set", json!({"id": w, "params": {"attach_top": roof}})).is_err());
    // Framing follows plain walls only, and a framed wall takes no profile.
    assert!(fails(&mut s, "wall_framing_create", json!({"walls": [w]})).1.contains("elevation profile"));
    let other = id(&run(&mut s, "wall_create", json!({"start": [10, 0], "end": [14, 0]})));
    run(&mut s, "wall_framing_create", json!({"walls": [other]}));
    assert!(fails(&mut s, "wall_profile_slope", json!({"wall": other, "top_end": 2.0})).1.contains("framed"));
    // Made an arc, a wall loses its profile in the same step.
    run(&mut s, "element_set", json!({"id": w, "params": {"bulge": 0.2}}));
    assert!(derived(&mut s, w).get("profile").is_none());
}

#[test]
fn wall_create_takes_a_profile() {
    let mut s = Session::default();
    let profile = json!([
        {"kind": "line", "start": [0, 0], "end": [5, 0]},
        {"kind": "line", "start": [5, 0], "end": [5, 1]},
        {"kind": "line", "start": [5, 1], "end": [0, 2.5]},
        {"kind": "line", "start": [0, 2.5], "end": [0, 0]},
    ]);
    let out = run(&mut s, "wall_create", json!({"start": [0, 0], "end": [5, 0], "profile": profile}));
    close(num(&out["derived"]["profile"]["area"]), 5.0 * 1.75, "area");
    let bad = json!([{"kind": "rectangle", "corner": [0, 0], "opposite": [9, 2]}]);
    assert!(s.execute("wall_create", json!({"start": [0, 3], "end": [5, 3], "profile": bad})).is_err());
}

#[test]
fn moving_an_end_keeps_the_profile_in_place_in_the_same_step() {
    let mut s = Session::default();
    let w = wall(&mut s);
    slope(&mut s, w);
    let steps = s.doc.undo_depth();
    // The end pulled back to 3 m: the profile is cut there, its slope kept.
    run(&mut s, "element_set", json!({"id": w, "params": {"end": [3.0, 0.0]}}));
    assert_eq!(s.doc.undo_depth(), steps + 1);
    let p = derived(&mut s, w)["profile"].clone();
    close(num(&p["end"]), 3.0, "cut at the end");
    close(num(&p["area"]), 3.0 * (3.0 + 2.5) / 2.0, "area of what is left");
    run(&mut s, "undo", json!({}));
    close(num(&derived(&mut s, w)["profile"]["area"]), 15.0, "undone");
}

/// A plan cuts a sloped wall only where its top is above the cut plane;
/// past that it is seen below the cut.
#[test]
fn plans_cut_only_the_stretch_the_plane_passes_through() {
    let mut s = Session::default();
    let w = wall(&mut s);
    // 3 m at the start down to 0.6 m at the end: the 1.2 m cut leaves it 4.5 m along.
    run(&mut s, "wall_profile_slope", json!({"wall": w, "top_end": 0.6}));
    let view = run(&mut s, "views_list", json!({"category": "floor_plan"}))[0]["id"].as_u64().unwrap();
    let d = s.drawing(ViewId(view)).unwrap();
    let mine = |e: &Option<ElementId>| e.is_some_and(|e| e.0 == w);
    let fills: Vec<&Vec<DVec2>> = d
        .prims
        .iter()
        .filter_map(|p| match p {
            Prim::Fill { pts, element, cut: true, .. } if mine(element) => Some(pts),
            _ => None,
        })
        .collect();
    assert!(!fills.is_empty(), "the wall is cut");
    let reach = fills.iter().flat_map(|f| f.iter().map(|p| p.x)).fold(f64::NEG_INFINITY, f64::max);
    close(reach, 4.5, "the cut stretch ends where the top meets the plane");
    // Beyond, its faces are drawn as seen below the cut.
    let seen = d.prims.iter().any(|p| {
        matches!(p, Prim::Line { a, b, style: LineStyle::Projection, element, .. }
            if mine(element) && a.x.min(b.x) >= 4.5 - 1e-6 && a.x.max(b.x) > 5.9)
    });
    assert!(seen, "the low end is seen below the cut");
}

/// Elevations come from the body: its sloped top is a line from 3 m to 2 m.
#[test]
fn elevations_follow_the_profile() {
    let mut s = Session::default();
    let w = wall(&mut s);
    slope(&mut s, w);
    let view = id(&run(&mut s, "view_create", json!({"kind": "elevation", "direction": "south"})));
    let d = s.drawing(ViewId(view)).unwrap();
    let sloped = d.prims.iter().any(|p| match p {
        Prim::Line { a, b, element: Some(e), .. } if e.0 == w => {
            let (lo, hi) = (a.y.min(b.y), a.y.max(b.y));
            (lo - 2.0).abs() < 1e-6 && (hi - 3.0).abs() < 1e-6
        }
        _ => false,
    });
    assert!(sloped, "the sloped top is drawn");
}

#[test]
fn doors_schedules_and_bars_follow_the_profile() {
    let mut s = Session::default();
    let w = wall(&mut s);
    slope(&mut s, w);
    let thickness = num(&derived(&mut s, w)["thickness"]);
    let inside = id(&run(&mut s, "door_create", json!({"wall": w, "offset": 1.5})));
    // Near the low end the top is under a door's head: it is left out of the wall.
    let outside = id(&run(&mut s, "door_create", json!({"wall": w, "offset": 5.4})));
    let g = s.geometry().walls[&ElementId(w)].clone();
    let cut: Vec<u64> = g.openings.iter().map(|o| o.element.0).collect();
    assert_eq!(cut, vec![inside], "{outside} does not fit");
    let o = &g.openings[0];
    let door = (o.u1 - o.u0) * (o.z1 - o.z0);
    close(num(&derived(&mut s, w)["volume"]), (15.0 - door) * thickness, "less the door that fits");
    // A wall schedule's area is the profile's less that door.
    let schedule = id(&run(&mut s, "view_create", json!({"kind": "schedule", "category": "wall"})));
    let table = run(&mut s, "schedule_table", json!({"view": schedule})).to_string();
    let area = format!("{:.2}", 15.0 - door);
    assert!(table.contains(&area) || table.contains(&area.replace('.', ",")), "{area}: {table}");
    // Bars stay inside the body: none above its top where it slopes.
    run(&mut s, "elements_delete", json!({"ids": [inside, outside]}));
    run(&mut s, "element_set", json!({"id": w, "params": {"structural": true}}));
    run(&mut s, "wall_rebar_create", json!({"walls": [w]}));
    let geom = s.geometry();
    let top = |u: f64| 3.0 - u / 6.0;
    let mut count = 0;
    for r in geom.rebar.values() {
        for bar in &r.bars {
            for p in &bar.points {
                count += 1;
                assert!(p.z <= top(p.x) + 1e-6 && p.z >= -1e-6, "a bar point outside the wall: {p}");
            }
        }
    }
    assert!(count > 0, "the wall has bars");
}

#[test]
fn a_damaged_profile_in_a_file_is_repaired_on_opening() {
    let mut s = Session::default();
    let w = wall(&mut s);
    slope(&mut s, w);
    let mut p = s.doc.project.clone();
    if let Some(cc_core::ElementKind::Wall(wall)) = p.elements.get_mut(&ElementId(w)).map(|e| &mut e.kind) {
        // A point that is no number: no profile.
        if let Some(profile) = &mut wall.profile {
            profile.outline[1].y = f64::NAN;
        }
    }
    let dir = std::env::temp_dir().join(format!("ccad-wall-profile-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("damaged.ccad");
    std::fs::write(&path, cc_core::to_ron(&p).unwrap()).unwrap();
    let out = run(&mut s, "project_open", json!({"path": path}));
    assert!(out["repairs"].as_u64().unwrap() >= 1, "{out}");
    assert!(derived(&mut s, w).get("profile").is_none());
    assert!(num(&derived(&mut s, w)["volume"]) > 0.0);
}
