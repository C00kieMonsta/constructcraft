//! Flipping walls (`wall_flip`, and Space on a selection through
//! `elements_turn`): the exterior and interior sides swap while the
//! location line stays where it is.

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_core::{ElementId, ElementKind, RebarLayout, Wall, WallSide};
use cc_engine::Session;
use glam::DVec2;
use serde_json::{Value, json};

fn run(s: &mut Session, name: &str, args: Value) -> Value {
    s.execute(name, args).unwrap_or_else(|e| panic!("{name}: {e:?}"))
}

fn id(v: &Value) -> u64 {
    v["id"].as_u64().unwrap()
}

/// The template's 25 cm wall type: 2.5 cm finish, 20 cm core, 2.5 cm finish.
fn layered_type(s: &mut Session) -> u64 {
    let types = run(s, "types_list", json!({"category": "wall"}));
    for t in types.as_array().unwrap() {
        let ty = t["id"].as_u64().unwrap();
        let layers = run(s, "type_get", json!({"type": ty}))["params"]["layers"].clone();
        let thicknesses: Vec<f64> =
            layers.as_array().into_iter().flatten().map(|l| l["thickness"].as_f64().unwrap()).collect();
        if thicknesses.len() == 3 && (thicknesses.iter().sum::<f64>() - 0.25).abs() < 1e-9 {
            return ty;
        }
    }
    panic!("no layered 25 cm wall type in {types}");
}

fn wall(s: &Session, id: u64) -> Wall {
    s.project().element(ElementId(id)).and_then(|e| e.as_wall()).cloned().unwrap()
}

fn hosted(s: &Session, id: u64) -> cc_core::Hosted {
    s.project().element(ElementId(id)).and_then(|e| e.as_hosted()).cloned().unwrap()
}

/// The world `y` range of each layer of straight wall `id` (along X), in the type's order.
fn layer_ranges(s: &mut Session, id: u64) -> Vec<(f64, f64)> {
    let g = s.geometry().walls.get(&ElementId(id)).cloned().unwrap();
    let u = g.length / 2.0;
    g.layers
        .iter()
        .map(|(a, b, _)| {
            let (ya, yb) = (g.world(u, *a).y, g.world(u, *b).y);
            (ya.min(yb), ya.max(yb))
        })
        .collect()
}

/// The door's center on the wall's location line, the way it faces and
/// the side of its hinge (the wall's left normal and direction, flipped as
/// its facing and hand say), in the world.
fn door_frame(s: &Session, door: u64) -> (DVec2, DVec2, DVec2) {
    let h = hosted(s, door);
    let w = wall(s, h.host.0);
    let center = w.start + w.direction() * h.offset;
    let facing = w.normal() * if h.flip_facing { -1.0 } else { 1.0 };
    let hand = w.direction() * if h.flip_hand { -1.0 } else { 1.0 };
    (center, facing, hand)
}

/// The rounded vertices of `id`'s solids, sorted: its 3D shape as a set.
fn shape(s: &mut Session, id: u64) -> Vec<[i64; 3]> {
    let mut points: Vec<[i64; 3]> = s
        .geometry()
        .solids
        .iter()
        .filter(|solid| solid.element == ElementId(id))
        .flat_map(|solid| solid.faces.iter().flat_map(|f| f.verts.iter()))
        .map(|v| [(v.x * 1e6).round() as i64, (v.y * 1e6).round() as i64, (v.z * 1e6).round() as i64])
        .collect();
    points.sort_unstable();
    points.dedup();
    points
}

fn close(a: DVec2, b: DVec2) -> bool {
    a.distance(b) < 1e-9
}

#[test]
fn a_wall_on_its_exterior_face_flips_about_that_face() {
    let mut s = Session::default();
    let ty = layered_type(&mut s);
    let w = run(
        &mut s,
        "wall_create",
        json!({"start": [0, 0], "end": [6, 0], "type": ty, "location_line": "FinishExterior", "height": 3.0}),
    );
    let door = run(&mut s, "door_create", json!({"wall": id(&w), "offset": 1.0}));
    let before = wall(&s, id(&w));
    let layers = layer_ranges(&mut s, id(&w));
    // The body lies right of the drawn line (y < 0), the exterior finish first, at the line.
    assert!(layers.iter().all(|(a, b)| *a >= -0.25 - 1e-9 && *b <= 1e-9), "{layers:?}");
    assert!(layers[0].1.abs() < 1e-9, "{layers:?}");
    let frame = door_frame(&s, id(&door));
    assert!(close(frame.0, DVec2::new(1.0, 0.0)), "{frame:?}");

    let out = run(&mut s, "wall_flip", json!({"walls": [id(&w)]}));
    assert_eq!(out["walls"], json!([id(&w)]), "{out}");
    let after = wall(&s, id(&w));
    // The location line stays: the same two points, walked the other way.
    assert_eq!((after.start, after.end), (before.end, before.start));
    assert_eq!(after.location_line, before.location_line);
    // The layers mirror about it: the exterior finish still at the line, the body now at y > 0.
    let flipped = layer_ranges(&mut s, id(&w));
    for ((a0, b0), (a1, b1)) in layers.iter().zip(&flipped) {
        assert!((a1 + b0).abs() < 1e-9 && (b1 + a0).abs() < 1e-9, "{layers:?} -> {flipped:?}");
    }
    // The exterior side now looks the other way.
    assert!(close(after.normal(), -before.normal()));
    // The door stays 1 m from the old start and faces the same way, hinged on the same side.
    let moved = door_frame(&s, id(&door));
    assert!(close(moved.0, frame.0) && close(moved.1, frame.1) && close(moved.2, frame.2), "{frame:?} {moved:?}");
    assert_eq!(hosted(&s, id(&door)).offset, 5.0);
}

#[test]
fn a_door_keeps_its_shape_in_a_flipped_wall() {
    let mut s = Session::default();
    let ty = layered_type(&mut s);
    let w = run(&mut s, "wall_create", json!({"start": [0, 0], "end": [6, 0], "type": ty, "height": 3.0}));
    let door = run(&mut s, "door_create", json!({"wall": id(&w), "offset": 1.5, "flip_hand": true}));
    let window = run(&mut s, "window_create", json!({"wall": id(&w), "offset": 4.0, "flip_facing": true}));
    let (door_shape, window_shape) = (shape(&mut s, id(&door)), shape(&mut s, id(&window)));
    assert!(!door_shape.is_empty() && !window_shape.is_empty());
    run(&mut s, "wall_flip", json!({"walls": [id(&w)]}));
    assert_eq!(shape(&mut s, id(&door)), door_shape);
    assert_eq!(shape(&mut s, id(&window)), window_shape);
}

#[test]
fn flipping_twice_gives_the_wall_back_and_undo_restores_it() {
    let mut s = Session::default();
    let ty = layered_type(&mut s);
    let w = run(
        &mut s,
        "wall_create",
        json!({"start": [0, 0], "end": [6, 0], "type": ty, "location_line": "CoreExterior", "height": 3.0}),
    );
    run(&mut s, "wall_join", json!({"id": id(&w), "end": "end", "joined": false}));
    run(&mut s, "door_create", json!({"wall": id(&w), "offset": 1.0}));
    let original = s.project().elements.clone();
    let depth = s.doc.undo_depth();
    run(&mut s, "wall_flip", json!({"walls": [id(&w)]}));
    // One undo step; the joins swapped ends with the ends.
    assert_eq!(s.doc.undo_depth(), depth + 1);
    assert!(!wall(&s, id(&w)).join_start && wall(&s, id(&w)).join_end);
    run(&mut s, "undo", json!({}));
    assert_eq!(s.project().elements, original, "undo restores");
    run(&mut s, "redo", json!({}));
    run(&mut s, "wall_flip", json!({"walls": [id(&w)]}));
    assert_eq!(s.project().elements, original, "flip twice is the identity");
}

#[test]
fn an_arc_wall_keeps_its_arc_and_its_door() {
    let mut s = Session::default();
    let ty = layered_type(&mut s);
    // A half circle of radius 4 about the origin, drawn counter-clockwise; body outside it.
    let w = run(
        &mut s,
        "wall_create",
        json!({"start": [4, 0], "end": [-4, 0], "mid": [0, 4], "type": ty,
               "location_line": "FinishExterior", "height": 3.0}),
    );
    let door = run(&mut s, "door_create", json!({"wall": id(&w), "offset": 2.0}));
    let before = wall(&s, id(&w));
    let arc = before.arc().unwrap();
    // The door's center seen from the arc's center.
    let angle = |s: &Session| {
        let (e, w) = (s.project().element(ElementId(id(&w))).unwrap(), wall(s, id(&w)));
        let (start, end) = s.project().wall_axis(e, &w);
        let axis = Wall { start, end, ..w };
        (axis.point_at(hosted(s, id(&door)).offset) - arc.center).to_angle()
    };
    let at = angle(&s);
    run(&mut s, "wall_flip", json!({"walls": [id(&w)]}));
    let after = wall(&s, id(&w));
    let flipped = after.arc().unwrap();
    assert!(close(flipped.center, arc.center) && (flipped.radius - arc.radius).abs() < 1e-9);
    assert!((after.length() - before.length()).abs() < 1e-9);
    assert!(close(after.point_at(after.length() / 2.0), before.point_at(before.length() / 2.0)));
    // The body moved inside the arc: the centerline shrank, the door kept its angle.
    let (e, w2) = (s.project().element(ElementId(id(&w))).unwrap(), wall(&s, id(&w)));
    assert!(s.project().wall_length(e, &w2) < before.length());
    assert!((angle(&s) - at).abs() < 1e-9, "{} {at}", angle(&s));
    run(&mut s, "wall_flip", json!({"walls": [id(&w)]}));
    let back = wall(&s, id(&w));
    assert!(close(back.start, before.start) && close(back.end, before.end));
    assert!((back.bulge - before.bulge).abs() < 1e-12);
    assert!((angle(&s) - at).abs() < 1e-9);
}

#[test]
fn bars_stay_on_the_face_looking_the_same_way() {
    let mut s = Session::default();
    let w = run(&mut s, "wall_create", json!({"start": [0, 0], "end": [5, 0], "height": 3.0}));
    run(&mut s, "element_set", json!({"id": id(&w), "params": {"structural": true}}));
    run(&mut s, "wall_rebar_create", json!({"walls": [id(&w)], "faces": "exterior"}));
    let sides = |s: &Session| -> Vec<WallSide> {
        s.project().rebar_of(ElementId(id(&w))).filter_map(|(_, r)| r.layout.wall_side()).collect()
    };
    assert!(!sides(&s).is_empty() && sides(&s).iter().all(|side| *side == WallSide::Exterior));
    // The bars of the left face (north, +y) before.
    let ys = |s: &mut Session| -> Vec<f64> {
        let ids: Vec<ElementId> = s
            .project()
            .rebar_of(ElementId(id(&w)))
            .filter(|(_, r)| matches!(r.layout, RebarLayout::WallHorizontal { .. }))
            .map(|(e, _)| e.id)
            .collect();
        let g = s.geometry();
        ids.iter().filter_map(|i| g.rebar.get(i)).flat_map(|r| r.bars.iter()).map(|b| b.points[0].y).collect()
    };
    let north = ys(&mut s);
    assert!(!north.is_empty() && north.iter().all(|y| *y > 0.0), "{north:?}");
    run(&mut s, "wall_flip", json!({"walls": [id(&w)]}));
    assert!(sides(&s).iter().all(|side| *side == WallSide::Interior));
    let after = ys(&mut s);
    assert!(after.iter().zip(&north).all(|(a, b)| (a - b).abs() < 1e-9), "{north:?} -> {after:?}");
}

#[test]
fn dimensions_keep_measuring_a_flipped_wall() {
    let mut s = Session::default();
    let ty = layered_type(&mut s);
    let plan = run(&mut s, "views_list", json!({"category": "floor_plan"}))[0]["id"].clone();
    let w = run(&mut s, "wall_create", json!({"start": [0, 0], "end": [6, 0], "type": ty, "height": 3.0}));
    // From the wall's north face to a point 3 m north of it.
    let dim = run(&mut s, "dimension_create", json!({"view": plan, "a": [2.0, 0.125], "b": [2.0, 3.125]}));
    assert!(dim["params"]["refs"][0]["target"]["Element"] == json!(id(&w)), "{dim}");
    let ends = |s: &mut Session| {
        let d = run(s, "element_get", json!({"id": dim["id"]}));
        (d["params"]["a"].clone(), d["params"]["b"].clone())
    };
    let before = ends(&mut s);
    run(&mut s, "wall_flip", json!({"walls": [id(&w)]}));
    assert_eq!(ends(&mut s), before);
    // Moved, the dimension still follows the same face.
    run(&mut s, "elements_move", json!({"ids": [id(&w)], "delta": [0.0, 1.0]}));
    let (a, _) = ends(&mut s);
    assert!((a[1].as_f64().unwrap() - 1.125).abs() < 1e-9, "{a}");
}

#[test]
fn space_on_a_selection_flips_walls_and_turns_the_rest() {
    let mut s = Session::default();
    let w = run(&mut s, "wall_create", json!({"start": [0, 0], "end": [5, 0], "height": 3.0}));
    let column = run(&mut s, "column_create", json!({"position": [10.0, 10.0]}));
    let pinned = run(&mut s, "wall_create", json!({"start": [0, 5], "end": [5, 5], "height": 3.0}));
    run(&mut s, "elements_pin", json!({"ids": [id(&pinned)], "pinned": true}));
    let depth = s.doc.undo_depth();
    let out = run(&mut s, "elements_turn", json!({"ids": [id(&w), id(&column), id(&pinned)]}));
    assert_eq!(s.doc.undo_depth(), depth + 1, "one undo step");
    assert_eq!(out["changed"], json!([id(&w), id(&column)]), "{out}");
    assert_eq!(out["pinned_skipped"], json!([id(&pinned)]), "{out}");
    assert_eq!(wall(&s, id(&w)).start, DVec2::new(5.0, 0.0));
    assert_eq!(wall(&s, id(&pinned)).start, DVec2::new(0.0, 5.0));
    let ElementKind::Column(c) = &s.project().element(ElementId(id(&column))).unwrap().kind else { panic!() };
    assert_eq!(c.rotation_deg, 90.0);
    // A door selected with its wall goes with it: it keeps facing the same way.
    let door = run(&mut s, "door_create", json!({"wall": id(&w), "offset": 2.0}));
    let frame = door_frame(&s, id(&door));
    let out = run(&mut s, "elements_turn", json!({"ids": [id(&w), id(&door)]}));
    assert_eq!(out["changed"], json!([id(&w)]), "{out}");
    let moved = door_frame(&s, id(&door));
    assert!(close(moved.0, frame.0) && close(moved.1, frame.1) && close(moved.2, frame.2), "{frame:?} {moved:?}");
    // `wall_flip` leaves pinned walls alone too, and refuses what is not a wall.
    let out = run(&mut s, "wall_flip", json!({"walls": [id(&pinned)]}));
    assert_eq!(out["pinned_skipped"], json!([id(&pinned)]), "{out}");
    assert!(s.execute("wall_flip", json!({"walls": [id(&column)]})).is_err());
}

#[test]
fn a_spline_wall_reverses_its_curve() {
    let mut s = Session::default();
    let w = run(
        &mut s,
        "wall_create",
        json!({"start": [0, 0], "end": [8, 0], "through": [[4, 2]], "location_line": "FinishExterior",
               "height": 3.0}),
    );
    let before = wall(&s, id(&w));
    run(&mut s, "wall_flip", json!({"walls": [id(&w)]}));
    let after = wall(&s, id(&w));
    let (a, b) = (before.spline.clone().unwrap(), after.spline.clone().unwrap());
    assert_eq!(b.control, a.control.iter().rev().copied().collect::<Vec<_>>());
    assert_eq!((after.start, after.end), (before.end, before.start));
    assert!(close(after.point_at(after.length() / 2.0), before.point_at(before.length() / 2.0)));
    run(&mut s, "wall_flip", json!({"walls": [id(&w)]}));
    assert_eq!(wall(&s, id(&w)), before);
}

#[test]
fn a_curtain_wall_keeps_its_own_grid_in_place() {
    let mut s = Session::default();
    let grid = json!({"layout": "FixedDistance", "spacing": 1.0, "justification": "Start"});
    let mullion = json!({"width": 0.05, "depth": 0.1, "material": ""});
    let curtain = json!({
        "vertical": grid,
        "horizontal": grid,
        "panel": {"kind": "Glass", "thickness": 0.02, "material": ""},
        "vertical_mullion": mullion,
        "horizontal_mullion": mullion,
        "border_mullion": mullion,
    });
    let ty = run(&mut s, "curtain_type_create", json!({"name": "CW", "curtain": curtain}));
    let w = run(&mut s, "wall_create", json!({"start": [0, 0], "end": [6, 0], "type": id(&ty), "height": 3.0}));
    run(&mut s, "curtain_grid_set", json!({"element": id(&w), "axis": "Vertical", "lines": [1.0, 2.5]}));
    run(&mut s, "wall_flip", json!({"walls": [id(&w)]}));
    assert_eq!(wall(&s, id(&w)).curtain.vertical, Some(vec![3.5, 5.0]));
    let g = run(&mut s, "curtain_get", json!({"element": id(&w)}));
    assert_eq!(g["vertical"], json!([3.5, 5.0]));
}
