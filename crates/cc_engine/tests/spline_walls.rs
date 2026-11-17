//! Spline walls, first stage (issue #307): walls whose location line is a
//! smooth free-form curve, a chain of cubic Bézier segments. The model,
//! the geometry, plans and the commands; what is not there yet is refused.

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_core::wall_spline::WallSpline;
use cc_core::{ElementId, ElementKind, ViewId, Wall};
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

fn num(v: &Value) -> f64 {
    v.as_f64().unwrap_or_else(|| panic!("not a number: {v}"))
}

fn point(v: &Value) -> DVec2 {
    DVec2::new(num(&v[0]), num(&v[1]))
}

/// The points between the ends of a gentle S from [0, 0] to [9, 0.5].
fn fit() -> Value {
    json!([[3, 1.5], [6, -1]])
}

fn spline_wall(s: &mut Session) -> u64 {
    id(&run(s, "wall_create", json!({"start": [0, 0], "end": [9, 0.5], "through": fit(), "height": 3.0})))
}

fn get(s: &mut Session, id: u64) -> Value {
    run(s, "element_get", json!({"id": id}))
}

fn wall(s: &Session, id: u64) -> Wall {
    s.project().element(ElementId(id)).and_then(|e| e.as_wall()).cloned().unwrap()
}

fn plan(s: &mut Session) -> u64 {
    run(s, "views_list", json!({"category": "floor_plan"}))[0]["id"].as_u64().unwrap()
}

#[test]
fn wall_create_takes_fit_points_or_control_points() {
    let mut s = Session::default();
    let steps = s.doc.undo_depth();
    let out = run(&mut s, "wall_create", json!({"start": [0, 0], "end": [9, 0.5], "through": fit(), "height": 3.0}));
    assert_eq!(s.doc.undo_depth(), steps + 1, "one undo step");
    let a = id(&out);
    // The curve in `params`: its control points, and the ends it starts and ends at.
    let control = out["params"]["spline"]["control"].as_array().unwrap().clone();
    assert_eq!(control.len(), 10, "three segments");
    assert_eq!(point(&out["params"]["start"]), DVec2::new(0.0, 0.0));
    assert_eq!(point(&out["params"]["end"]), DVec2::new(9.0, 0.5));
    assert_eq!(num(&out["params"]["bulge"]), 0.0);
    // Its figures in `derived`.
    let d = &out["derived"];
    let length = num(&d["length"]);
    assert!(length > 9.5 && length < 11.5, "{length}");
    assert_eq!(d["spline"]["segments"], json!(3));
    let radius = num(&d["spline"]["min_radius"]);
    assert!(radius > 0.3 && radius < 10.0, "{radius}");
    assert!(num(&d["spline"]["face_radius"]) < radius);
    assert_eq!(d["spline"]["knots"].as_array().unwrap().len(), 4);
    let thickness = num(&d["thickness"]);
    assert!((num(&d["volume"]) / (length * thickness * 3.0) - 1.0).abs() < 1e-3);
    // The same wall by its control points.
    let b = id(&run(
        &mut s,
        "wall_create",
        json!({"start": [0, 0], "end": [9, 0.5], "control": control, "height": 3.0, "base_offset": 4.0}),
    ));
    assert_eq!(wall(&s, a).spline, wall(&s, b).spline);

    let other = get(&mut s, b);
    assert!((num(&other["derived"]["length"]) - length).abs() < 1e-12);
    assert!((num(&other["derived"]["volume"]) - num(&d["volume"])).abs() < 1e-9);
    // Undo takes the wall away, redo brings it back.
    run(&mut s, "undo", json!({}));
    assert!(s.project().element(ElementId(b)).is_none());
    run(&mut s, "redo", json!({}));
    assert_eq!(wall(&s, b).spline, wall(&s, a).spline);
    // The ends may be repeated among the points through.
    let again = json!({"start": [0, 0], "end": [9, 0.5], "through": [[0, 0], [3, 1.5], [6, -1], [9, 0.5]], "base_offset": 20.0});
    let again = id(&run(&mut s, "wall_create", again));
    assert_eq!(wall(&s, a).spline, wall(&s, again).spline);
    // A location line on a face moves the body, not the curve.
    let c = id(&run(
        &mut s,
        "wall_create",
        json!({"start": [0, 0], "end": [9, 0.5], "through": fit(), "location_line": "FinishExterior", "base_offset": 8.0}),
    ));
    assert_eq!(wall(&s, c).spline, wall(&s, a).spline);
    let moved = num(&get(&mut s, c)["derived"]["length"]);
    assert!((moved - length).abs() < 0.2, "{moved} vs {length}");
    let body = s.geometry().walls[&ElementId(c)].clone();
    // Its exterior (left) face starts where the curve does: the centerline half a wall to the right.
    assert!(body.world(0.0, body.half).distance(DVec2::ZERO) < 1e-9);
    assert!((body.world(0.0, 0.0).length() - thickness * 0.5).abs() < 1e-9);
}

#[test]
fn wall_create_refuses_what_makes_no_spline_wall() {
    let mut s = Session::default();
    let invalid = |s: &mut Session, args: Value, word: &str| {
        let (code, message) = fails(s, "wall_create", args);
        assert_eq!(code, ErrorCode::InvalidArguments, "{message}");
        assert!(message.contains(word), "'{word}' in: {message}");
    };
    let ends = |mut args: Value, end: [f64; 2]| {
        args["start"] = json!([0, 0]);
        args["end"] = json!(end);
        args
    };
    invalid(
        &mut s,
        ends(json!({"through": fit(), "control": [[0, 0], [1, 0], [2, 0], [3, 0]]}), [3.0, 0.0]),
        "not both",
    );
    let many: Vec<[f64; 2]> = (1..=80).map(|i| [i as f64, (i % 2) as f64]).collect();
    invalid(&mut s, ends(json!({"through": many}), [81.0, 0.0]), "at most 63");
    invalid(&mut s, ends(json!({"control": [[0, 0], [1, 0], [3, 0]]}), [3.0, 0.0]), "3n + 1");
    // The control points start and end where the wall does.
    invalid(&mut s, ends(json!({"control": [[0, 0], [1, 0], [2, 0], [3, 0]]}), [4.0, 0.0]), "from `start` to `end`");
    // The ends are the wall's own: always given.
    invalid(&mut s, json!({"through": fit()}), "missing field");
    invalid(&mut s, json!({"start": [0, 0], "end": [9, 0.5], "through": fit(), "bulge": 0.5}), "not both");
    invalid(&mut s, json!({"start": [0, 0], "end": [9, 0.5], "through": fit(), "mid": [4, 4]}), "not both");
    // A corner where two segments join.
    let corner = json!([[0, 0], [1, 0], [2, 0], [3, 0], [3, 1], [3, 2], [3, 3]]);
    invalid(&mut s, ends(json!({"control": corner}), [3.0, 3.0]), "corner");
    // A bend of 10 cm radius: too tight for any wall type of the template.
    let tight = json!([[0, 0], [0.0552, 0], [0.1, 0.0448], [0.1, 0.1]]);
    invalid(&mut s, ends(json!({"control": tight}), [0.1, 0.1]), "too tightly");
    // A loop.
    invalid(&mut s, ends(json!({"control": [[0, 0], [6, 4], [-2, 4], [4, 0]]}), [4.0, 0.0]), "cross itself");
    // Closed.
    invalid(&mut s, ends(json!({"through": [[4, 0], [2, 3]]}), [0.0, 0.01]), "ends meet");
    // An elevation profile.
    let profile = json!([{"kind": "rectangle", "corner": [0, 0], "opposite": [5, 2]}]);
    invalid(&mut s, json!({"start": [0, 0], "end": [9, 0.5], "through": fit(), "profile": profile}), "straight walls");
    // A curtain wall type.
    let curtain = run(&mut s, "curtain_type_create", json!({"name": "Glazing"}));
    invalid(&mut s, json!({"start": [0, 0], "end": [9, 0.5], "through": fit(), "type": curtain["id"]}), "curtain wall");
    assert_eq!(s.project().walls().count(), 0);
    assert_eq!(s.doc.undo_depth(), 1, "only the curtain type was made");
}

#[test]
fn element_set_patches_the_curve_in_one_step() {
    let mut s = Session::default();
    let w = spline_wall(&mut s);
    let before = get(&mut s, w);
    let steps = s.doc.undo_depth();
    // New control points: a single segment.
    let control = json!([[0, 0], [2, 2], [5, 2], [7, 0]]);
    let out = run(&mut s, "element_set", json!({"id": w, "params": {"spline": {"control": control}}}));
    assert_eq!(s.doc.undo_depth(), steps + 1);
    assert_eq!(point(&out["params"]["end"]), DVec2::new(7.0, 0.0), "the end follows the curve");
    assert_eq!(out["derived"]["spline"]["segments"], json!(1));
    // An end moved alone takes its knot and its handle along.
    let out = run(&mut s, "element_set", json!({"id": w, "params": {"end": [8.0, 1.0]}}));
    let c: Vec<DVec2> = out["params"]["spline"]["control"].as_array().unwrap().iter().map(point).collect();
    assert_eq!(c, vec![DVec2::new(0.0, 0.0), DVec2::new(2.0, 2.0), DVec2::new(6.0, 3.0), DVec2::new(8.0, 1.0)]);
    // Hostile patches are refused and change nothing.
    for (patch, word) in [
        (json!({"spline": {"control": [[0, 0], [1, 0]]}}), "3n + 1"),
        (json!({"spline": {"control": [[0, 0], [6, 4], [-2, 4], [4, 0]]}}), "cross itself"),
        (json!({"bulge": 0.4}), "not both"),
    ] {
        let (code, message) = fails(&mut s, "element_set", json!({"id": w, "params": patch}));
        assert_eq!(code, ErrorCode::InvalidArguments, "{message}");
        assert!(message.contains(word), "'{word}' in: {message}");
    }
    assert_eq!(s.doc.undo_depth(), steps + 2);
    // Without its spline it is the straight wall between its ends.
    let out = run(&mut s, "element_set", json!({"id": w, "params": {"spline": null}}));
    assert!(out["params"].get("spline").is_none() && out["derived"].get("spline").is_none());
    assert!((num(&out["derived"]["length"]) - DVec2::new(8.0, 1.0).length()).abs() < 1e-9);
    // And a straight wall takes one.
    let out = run(&mut s, "element_set", json!({"id": w, "params": {"spline": before["params"]["spline"]}}));
    assert_eq!(out["params"]["spline"], before["params"]["spline"]);
    assert_eq!(point(&out["params"]["end"]), DVec2::new(9.0, 0.5));
    for _ in 0..4 {
        run(&mut s, "undo", json!({}));
    }
    assert_eq!(get(&mut s, w)["params"], before["params"]);
}

#[test]
fn what_spline_walls_do_not_take_yet_is_refused() {
    let mut s = Session::default();
    let w = spline_wall(&mut s);
    let straight = id(&run(&mut s, "wall_create", json!({"start": [0, 6], "end": [9, 6]})));
    let steps = s.doc.undo_depth();
    let refused = |s: &mut Session, name: &str, args: Value| {
        let (code, message) = fails(s, name, args);
        assert_eq!(code, ErrorCode::InvalidArguments, "{name}: {message}");
        assert!(message.contains("spline"), "{name}: {message}");
    };
    // Doors, windows and niches.
    refused(&mut s, "door_create", json!({"wall": w, "offset": 3.0}));
    refused(&mut s, "window_create", json!({"wall": w, "offset": 3.0}));
    refused(&mut s, "door_create", json!({"point": [3.0, 1.5]}));
    refused(&mut s, "niche_create", json!({"wall": w, "offset": 3.0}));
    // A door of another wall is not moved onto it.
    let door = id(&run(&mut s, "door_create", json!({"wall": straight, "offset": 3.0})));
    refused(&mut s, "element_set", json!({"id": door, "params": {"host": w}}));
    run(&mut s, "elements_delete", json!({"ids": [door]}));
    // An attached top.
    let floor = id(&run(
        &mut s,
        "floor_create",
        json!({"boundary": [[-2, -4], [12, -4], [12, 8], [-2, 8]], "height_offset": 3.2}),
    ));
    refused(&mut s, "wall_attach", json!({"ids": [w], "top": floor}));
    refused(&mut s, "element_set", json!({"id": w, "params": {"attach_top": floor}}));
    // An elevation profile.
    refused(&mut s, "wall_profile_slope", json!({"wall": w, "top_end": 2.0}));
    refused(&mut s, "wall_profile_set", json!({"wall": w, "points": [[0, 0], [5, 0], [5, 2], [0, 2]]}));
    // A curtain wall type.
    let curtain = run(&mut s, "curtain_type_create", json!({"name": "Glazing"}));
    let (code, message) = fails(&mut s, "element_set", json!({"id": w, "type": curtain["id"]}));
    assert_eq!(code, ErrorCode::InvalidArguments, "{message}");
    assert!(message.contains("curtain wall"), "{message}");
    // Reinforcement and framing.
    run(&mut s, "element_set", json!({"id": w, "params": {"structural": true}}));
    refused(&mut s, "wall_rebar_create", json!({"walls": [w]}));
    let (code, message) = fails(&mut s, "wall_framing_create", json!({"walls": [w]}));
    assert_eq!(code, ErrorCode::InvalidArguments);
    assert!(message.contains("curved"), "{message}");
    // By level, reinforcement leaves it out.
    run(&mut s, "element_set", json!({"id": straight, "params": {"structural": true}}));
    let out = run(&mut s, "wall_rebar_create", json!({}));
    assert_eq!(out["walls"], json!(1), "{out}");
    assert!(s.project().rebar_of(ElementId(w)).next().is_none());
    // A strip footing under it is left out too.
    let out = run(&mut s, "strip_footings_under_walls", json!({"walls": [w, straight]}));
    assert_eq!(out["skipped"], json!(1), "{out}");
    // Trim, extend and align.
    let rejected = |s: &mut Session, name: &str, args: Value| {
        let (code, message) = fails(s, name, args);
        assert_eq!(code, ErrorCode::Rejected, "{name}: {message}");
        assert!(message.to_lowercase().contains("spline"), "{name}: {message}");
    };
    rejected(&mut s, "trim_extend", json!({"id": w, "pick": [3, 1.5], "reference": straight}));
    rejected(&mut s, "trim_extend", json!({"id": straight, "pick": [3, 6], "reference": w}));
    rejected(&mut s, "trim_corner", json!({"a": w, "pick_a": [3, 1.5], "b": straight, "pick_b": [3, 6]}));
    rejected(&mut s, "element_align", json!({"id": w, "reference": straight}));
    rejected(&mut s, "element_align", json!({"id": straight, "reference": w, "reference_pick": [3, 1.5]}));
    // A wall with doors does not become a spline wall.
    let door = id(&run(&mut s, "door_create", json!({"wall": straight, "offset": 3.0})));
    let spline = get(&mut s, w)["params"]["spline"].clone();
    refused(&mut s, "element_set", json!({"id": straight, "params": {"spline": spline}}));
    assert!(s.project().element(ElementId(door)).is_some());
    // Nothing above left anything behind but what succeeded.
    assert!(wall(&s, w).attach_top.is_none() && wall(&s, w).profile.is_none());
    assert_eq!(s.project().hosted_by(ElementId(w)).count(), 0);
    assert!(s.doc.undo_depth() > steps);
    assert_eq!(s.project().repairs_needed(), 0);
}

#[test]
fn move_rotate_mirror_copy_split_offset_and_delete() {
    let mut s = Session::default();
    let w = spline_wall(&mut s);
    let original = wall(&s, w);
    let length = original.length();
    // Move.
    run(&mut s, "elements_move", json!({"ids": [w], "delta": [2.0, -1.0]}));
    let moved = wall(&s, w);
    let shift = DVec2::new(2.0, -1.0);
    assert_eq!((moved.start, moved.end), (original.start + shift, original.end + shift));
    for (a, b) in moved.spline.as_ref().unwrap().control.iter().zip(&original.spline.as_ref().unwrap().control) {
        assert!(a.distance(*b + shift) < 1e-12);
    }
    run(&mut s, "undo", json!({}));
    // Rotate a quarter turn about the origin.
    run(&mut s, "elements_rotate", json!({"ids": [w], "center": [0, 0], "angle": 90.0}));
    let turned = wall(&s, w);
    assert!(turned.end.distance(DVec2::new(-0.5, 9.0)) < 1e-9, "{}", turned.end);
    assert!((turned.length() - length).abs() < 1e-9);
    assert_eq!(turned.spline.as_ref().unwrap().ends().unwrap(), (turned.start, turned.end));
    run(&mut s, "undo", json!({}));
    // Mirror across the X axis: the wall runs the other way, so its exterior face stays outside.
    run(&mut s, "elements_mirror", json!({"ids": [w], "axis": [[0, 0], [1, 0]], "copy": false}));
    let mirrored = wall(&s, w);
    assert!(mirrored.start.distance(DVec2::new(9.0, -0.5)) < 1e-12 && mirrored.end.distance(DVec2::ZERO) < 1e-12);
    assert!((mirrored.length() - length).abs() < 1e-9);
    let (was, now) = (original.spline_path().unwrap(), mirrored.spline_path().unwrap());
    let left = was.at(2.0, 0.3);
    assert!(now.at(length - 2.0, 0.3).distance(DVec2::new(left.x, -left.y)) < 1e-9);
    run(&mut s, "undo", json!({}));
    // Copy.
    let copies = run(&mut s, "elements_copy", json!({"ids": [w], "delta": [0.0, 5.0]}));
    let copy = copies["created"][0].as_u64().unwrap();
    assert!(wall(&s, copy).spline.is_some());
    assert!(wall(&s, copy).start.distance(DVec2::new(0.0, 5.0)) < 1e-12);
    // Split: two spline walls sharing the point, the curve kept.
    let out = run(&mut s, "element_split", json!({"id": copy, "point": [4.5, 5.2]}));
    let second = out["created"].as_u64().unwrap();
    let (a, b) = (wall(&s, copy), wall(&s, second));
    assert!(a.spline.is_some() && b.spline.is_some());
    assert_eq!(a.end, b.start);
    assert!((a.length() + b.length() - length).abs() < 1e-3);
    // (On the curve itself: within the flattening's sagitta of its chords.)
    assert!(original.spline_path().unwrap().locate(a.end - DVec2::new(0.0, 5.0)).1.abs() < 1e-3);
    // Offset: a spline wall alongside, 1 m to the left.
    let out = run(&mut s, "element_offset", json!({"id": w, "distance": 1.0}));
    let beside = wall(&s, id(&out));
    let path = original.spline_path().unwrap();
    for k in beside.spline.as_ref().unwrap().knots() {
        assert!((path.locate(k).1 - 1.0).abs() < 0.02, "{k}");
    }
    // Delete.
    run(&mut s, "elements_delete", json!({"ids": [w, copy, second]}));
    assert_eq!(s.project().walls().count(), 1);
    assert_eq!(s.project().repairs_needed(), 0);
}

#[test]
fn plans_draw_the_layers_along_the_curve() {
    let mut s = Session::default();
    let w = spline_wall(&mut s);
    let view = plan(&mut s);
    let g = s.geometry().walls[&ElementId(w)].clone();
    let path = g.spline.clone().expect("a spline wall");
    let chords = path.points().len() - 1;
    assert!(chords > 20, "{chords}");
    assert_eq!(g.layers.len(), 3);
    let mine = |e: &Option<ElementId>| e.is_some_and(|e| e.0 == w);
    let lines = |s: &mut Session, style: LineStyle| -> Vec<(DVec2, DVec2)> {
        let d = s.drawing(ViewId(view)).unwrap();
        d.prims
            .iter()
            .filter_map(|p| match p {
                Prim::Line { a, b, style: st, element, .. } if *st == style && mine(element) => Some((*a, *b)),
                _ => None,
            })
            .collect()
    };
    // Medium detail: the two layer boundaries, each a chord per chord of the centerline,
    // at the layers' offsets.
    let thin = lines(&mut s, LineStyle::Thin);
    assert_eq!(thin.len(), 2 * chords, "two boundaries of {chords} chords");
    let mut offsets: Vec<f64> = thin.iter().map(|(a, b)| path.locate((*a + *b) * 0.5).1).collect();
    offsets.sort_by(f64::total_cmp);
    offsets.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
    let expected = [g.layers[1].1, g.layers[0].1];
    assert_eq!(offsets.len(), 2, "{offsets:?}");
    for (found, want) in offsets.iter().zip(expected) {
        assert!((found - want).abs() < 1e-6, "{found} vs {want}");
    }
    // The faces and the two ends, cut.
    let cut = lines(&mut s, LineStyle::Cut);
    assert_eq!(cut.len(), 2 * chords + 2);
    for (a, b) in &cut {
        let v = path.locate((*a + *b) * 0.5).1;
        assert!((v.abs() - g.half).abs() < 1e-6 || v.abs() < 1e-6, "{v}");
    }
    // The cut fills cover the footprint.
    let d = s.drawing(ViewId(view)).unwrap();
    let filled: f64 = d
        .prims
        .iter()
        .filter_map(|p| match p {
            Prim::Fill { pts, element, cut: true, .. } if mine(element) => Some(cc_geom::polygon::area(pts)),
            _ => None,
        })
        .sum();
    let footprint = cc_geom::polygon::area(&g.footprint());
    assert!(filled > footprint - 1e-9, "{filled} vs {footprint}");
    assert!(d.prims.iter().all(|p| match p {
        Prim::Line { a, b, .. } => a.is_finite() && b.is_finite(),
        Prim::Fill { pts, .. } => pts.iter().all(|p| p.is_finite()),
        _ => true,
    }));
    // Coarse: one region, no layer lines. Fine: as medium.
    run(&mut s, "view_set", json!({"view": view, "detail": "coarse"}));
    assert!(lines(&mut s, LineStyle::Thin).is_empty());
    assert_eq!(lines(&mut s, LineStyle::Cut).len(), 2 * chords + 2);
    run(&mut s, "view_set", json!({"view": view, "detail": "fine"}));
    assert_eq!(lines(&mut s, LineStyle::Thin).len(), 2 * chords);
    // A wall wholly below the cut plane is seen, not cut.
    run(&mut s, "element_set", json!({"id": w, "params": {"height": 0.9}}));
    assert!(lines(&mut s, LineStyle::Cut).is_empty());
    assert!(lines(&mut s, LineStyle::Projection).len() >= 2 * chords);
    // Picking follows the drawing: a point on the curve hits the wall, one off it does not.
    run(&mut s, "element_set", json!({"id": w, "params": {"height": 3.0}}));
    let d = s.drawing(ViewId(view)).unwrap();
    let on = path.at(path.length() * 0.4, 0.0);
    assert_eq!(d.hit_test(on, 0.01).map(|e| e.0), Some(w));
    assert_eq!(d.hit_test(DVec2::new(4.5, 3.0), 0.01), None);
}

#[test]
fn elevations_sections_and_3d_views_show_the_curved_body() {
    let mut s = Session::default();
    let w = spline_wall(&mut s);
    // Joined with a straight wall at its end, so a joint is in the picture.
    run(&mut s, "wall_create", json!({"start": [9, 0.5], "end": [9, -4]}));
    let g = s.geometry().walls[&ElementId(w)].clone();
    let chords = g.spline.as_ref().unwrap().points().len() - 1;
    let elevation = id(&run(&mut s, "view_create", json!({"kind": "elevation", "direction": "south"})));
    let section =
        id(&run(&mut s, "view_create", json!({"kind": "section", "start": [10, -3], "end": [-1, -3], "depth": 8.0})));
    let three_d = run(&mut s, "views_list", json!({"category": "three_d"}))[0]["id"].as_u64().unwrap();
    for view in [elevation, section, three_d] {
        let d = s.drawing(ViewId(view)).unwrap_or_else(|| panic!("view {view} draws"));
        let mut faces = 0;
        let (mut lo, mut hi) = (DVec2::splat(f64::INFINITY), DVec2::splat(f64::NEG_INFINITY));
        for p in &d.prims {
            match p {
                Prim::Fill { pts, element: Some(e), .. } if e.0 == w => {
                    assert!(pts.iter().all(|p| p.is_finite()));
                    faces += 1;
                    for p in pts {
                        (lo, hi) = (lo.min(*p), hi.max(*p));
                    }
                }
                Prim::Line { a, b, .. } => assert!(a.is_finite() && b.is_finite()),
                _ => {}
            }
        }
        // The curved face shows as many facets (those turned to the viewer).
        assert!(faces > chords / 4, "view {view}: {faces} faces of {chords} chords");
        if view != three_d {
            // About 9 m wide (less the joint at its end), 3 m high.
            assert!(hi.x - lo.x > 8.9 && hi.x - lo.x < 9.4, "view {view}: {lo} {hi}");
            assert!((hi.y - lo.y - 3.0).abs() < 1e-6, "view {view}: {lo} {hi}");
        }
    }
    // A section through the wall cuts it: a region of its thickness (more where it crosses on
    // the slant), 3 m high.
    let across =
        id(&run(&mut s, "view_create", json!({"kind": "section", "start": [4.5, -3], "end": [4.5, 3], "depth": 8.0})));
    let d = s.drawing(ViewId(across)).unwrap();
    let cut: Vec<&Vec<DVec2>> = d
        .prims
        .iter()
        .filter_map(|p| match p {
            Prim::Fill { pts, element: Some(e), cut: true, .. } if e.0 == w => Some(pts),
            _ => None,
        })
        .collect();
    assert!(!cut.is_empty(), "the section cuts the wall");
    let (lo, hi) = cut
        .iter()
        .flat_map(|pts| pts.iter())
        .fold((DVec2::splat(f64::INFINITY), DVec2::splat(f64::NEG_INFINITY)), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
    assert!((hi.y - lo.y - 3.0).abs() < 1e-6 && hi.x - lo.x >= 2.0 * g.half - 1e-6 && hi.x - lo.x < 6.0 * g.half);
    // Smooth facets draw no seams: of the wall's upright lines only its ends and silhouettes.
    let d = s.drawing(ViewId(elevation)).unwrap();
    let seams = d
        .prims
        .iter()
        .filter(|p| matches!(p, Prim::Line { a, b, element: Some(e), .. } if e.0 == w && (a.x - b.x).abs() < 1e-9))
        .count();
    assert!(seams < 12, "{seams} upright lines for {chords} chords");
}

#[test]
fn rooms_tags_dimensions_and_schedules_work() {
    let mut s = Session::default();
    // A room closed by three straight walls and a spline.
    for (a, b) in [([0, 4], [0, 0]), ([0, 0], [6, 0]), ([6, 0], [6, 4])] {
        run(&mut s, "wall_create", json!({"start": a, "end": b}));
    }
    let control = json!([[0, 4], [1, 4], [2, 5], [3, 4], [4, 3], [5, 4], [6, 4]]);
    let w = id(&run(&mut s, "wall_create", json!({"start": [0, 4], "end": [6, 4], "control": control})));
    let room = id(&run(&mut s, "room_create", json!({"point": [3, 2], "name": "Hall"})));
    let area = cc_geom::polygon::area(&s.geometry().rooms[&ElementId(room)]);
    // Close to the rectangle between the faces: the curve gives as much as it takes.
    assert!((area - 5.85 * 3.85).abs() < 0.15, "{area}");
    // Length, area and volume in a wall schedule.
    let d = get(&mut s, w)["derived"].clone();
    let (length, volume) = (num(&d["length"]), num(&d["volume"]));
    assert!(length > 6.3 && length < 7.0, "{length}");
    let schedule = id(&run(&mut s, "view_create", json!({"kind": "schedule", "category": "wall"})));
    run(
        &mut s,
        "schedule_set",
        json!({"view": schedule, "fields": ["Length", "Area", "Volume"], "sorting": [{"field": "Length", "descending": true}]}),
    );
    let table = run(&mut s, "schedule_table", json!({"view": schedule}));
    let text = table.to_string();
    for value in [length, length * 3.0, volume] {
        let cell = format!("{value:.2}");
        assert!(text.contains(&cell) || text.contains(&cell.replace('.', ",")), "{cell}: {text}");
    }
    // A dimension from the curved wall to the bottom wall follows the wall when it moves.
    let view = plan(&mut s);
    let half = num(&d["thickness"]) * 0.5;
    // From a point of its lower face (drawn, so the end can hold on to it) to the bottom wall's face.
    let face = wall(&s, w).spline_path().unwrap().at(1.5, -half);
    let out = run(
        &mut s,
        "dimension_create",
        json!({"view": view, "a": [face.x, face.y], "b": [face.x, half], "offset": 0.5}),
    );
    let dim = id(&out);
    assert_eq!(out["params"]["refs"][0]["target"]["Element"], json!(w), "{}", out["params"]["refs"]);
    let value = |s: &mut Session| {
        let p = get(s, dim)["params"].clone();
        point(&p["a"]).distance(point(&p["b"]))
    };
    let before = value(&mut s);
    assert!((before - (face.y - half)).abs() < 1e-6, "{before} vs {}", face.y - half);
    run(&mut s, "elements_move", json!({"ids": [w], "delta": [0.0, 0.5]}));
    assert!((value(&mut s) - before - 0.5).abs() < 1e-6, "{} vs {before}", value(&mut s));
    // The plan tags the room the curve bounds.
    let d = s.drawing(ViewId(view)).unwrap();
    assert!(d.prims.iter().any(|p| matches!(p, Prim::Text { text, .. } if text.contains("Hall"))));
    assert_eq!(s.project().repairs_needed(), 0);
}

#[test]
fn a_file_with_bad_splines_is_repaired_on_opening() {
    let mut s = Session::default();
    let good = spline_wall(&mut s);
    let nan = id(&run(
        &mut s,
        "wall_create",
        json!({"start": [0, 0], "end": [9, 0.5], "through": fit(), "base_offset": 4.0}),
    ));
    let tight = id(&run(
        &mut s,
        "wall_create",
        json!({"start": [0, 0], "end": [9, 0.5], "through": fit(), "base_offset": 8.0}),
    ));
    let stale = id(&run(
        &mut s,
        "wall_create",
        json!({"start": [0, 0], "end": [9, 0.5], "through": fit(), "base_offset": 12.0}),
    ));
    let hosting = id(&run(&mut s, "wall_create", json!({"start": [0, 9], "end": [9, 9]})));
    let door = id(&run(&mut s, "door_create", json!({"wall": hosting, "offset": 3.0})));
    let mut p = s.doc.project.clone();
    let edit = |p: &mut cc_core::Project, id: u64, f: &dyn Fn(&mut Wall)| {
        if let Some(ElementKind::Wall(w)) = p.elements.get_mut(&ElementId(id)).map(|e| &mut e.kind) {
            f(w);
        }
    };
    edit(&mut p, nan, &|w| w.spline.as_mut().unwrap().control[2].x = f64::NAN);
    edit(&mut p, tight, &|w| {
        w.spline = Some(WallSpline {
            control: vec![DVec2::ZERO, DVec2::new(0.05, 0.0), DVec2::new(0.1, 0.05), DVec2::new(0.1, 0.1)],
        })
    });
    // Ends that are not the curve's, and a bulge.
    edit(&mut p, stale, &|w| {
        w.start = DVec2::new(-3.0, -3.0);
        w.bulge = 0.7;
    });
    // A door in a wall that became a spline wall behind the commands' back.
    let spline = WallSpline::through(&[DVec2::new(0.0, 9.0), DVec2::new(4.0, 10.0), DVec2::new(9.0, 9.0)]);
    edit(&mut p, hosting, &|w| w.spline = spline.clone());
    let dir = std::env::temp_dir().join(format!("ccad-spline-walls-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("damaged.ccad");
    let text = cc_core::to_ron(&p).unwrap();
    assert!(text.contains("format_version: 32"), "spline walls are format 32");
    std::fs::write(&path, &text).unwrap();
    let out = run(&mut s, "project_open", json!({"path": path}));
    assert!(out["repairs"].as_u64().unwrap() >= 5, "{out}");
    assert!(wall(&s, good).spline.is_some());
    // Hostile curves: the straight wall between the ends.
    for id in [nan, tight] {
        let w = wall(&s, id);
        assert!(w.spline.is_none() && w.bulge == 0.0, "{id}");
        assert!(num(&get(&mut s, id)["derived"]["volume"]) > 0.0);
    }
    assert!((wall(&s, nan).length() - DVec2::new(9.0, 0.5).length()).abs() < 1e-9);
    // Stale ends and a bulge: the curve wins.
    let w = wall(&s, stale);
    assert_eq!((w.start, w.end, w.bulge), (DVec2::ZERO, DVec2::new(9.0, 0.5), 0.0));
    // The door is gone, the wall is a spline wall.
    assert!(s.project().element(ElementId(door)).is_none());
    assert!(wall(&s, hosting).spline.is_some());
    assert_eq!(s.project().repairs_needed(), 0);
    // A file of the previous format still opens (it has no splines).
    let old = text.replace("format_version: 32", "format_version: 31");
    assert!(cc_core::from_ron_with_repairs(&old).is_ok());
    let _ = std::fs::remove_dir_all(&dir);
}

/// A small deterministic generator (no dependency): xorshift.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }

    fn between(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.next()
    }
}

/// Random smooth splines, alone and with walls meeting them, through
/// geometry and every kind of view: nothing panics, nothing is not a number.
#[test]
fn random_smooth_splines_never_panic() {
    let mut rng = Rng(0x5EED_1234_ABCD_0307);
    let lines = ["Centerline", "FinishExterior", "FinishInterior", "CoreCenter"];
    let mut made = 0;
    for round in 0..40 {
        let mut s = Session::default();
        // Three to seven points, each 1.5 to 4 m on, turning up to 70° either way.
        let count = 3 + (rng.next() * 5.0) as usize;
        let mut at = DVec2::new(rng.between(-5.0, 5.0), rng.between(-5.0, 5.0));
        let mut heading = rng.between(0.0, std::f64::consts::TAU);
        let mut points = vec![at];
        for _ in 1..count {
            heading += rng.between(-1.2, 1.2);
            at += DVec2::from_angle(heading) * rng.between(1.5, 4.0);
            points.push(at);
        }
        let through: Vec<[f64; 2]> = points.iter().map(|p| p.to_array()).collect();
        let line = lines[round % lines.len()];
        let ends = (points[0].to_array(), points[count - 1].to_array());
        let args = json!({"start": ends.0, "end": ends.1, "through": through, "location_line": line});
        let created = s.execute("wall_create", args);
        let Ok(created) = created else {
            // Too tight or self-crossing: refused with a reason, nothing made.
            assert_eq!(s.project().walls().count(), 0);
            continue;
        };
        made += 1;
        let w = id(&created);
        let path = wall(&s, w).spline_path().unwrap();
        // A wall leaving each end, and one butting into its side.
        let end = *points.last().unwrap();
        let away = DVec2::from_angle(rng.between(0.0, std::f64::consts::TAU)) * rng.between(1.0, 3.0);
        let _ = s.execute("wall_create", json!({"start": end.to_array(), "end": (end + away).to_array()}));
        let start = points[0];
        let back = DVec2::from_angle(rng.between(0.0, std::f64::consts::TAU)) * rng.between(1.0, 3.0);
        let _ = s.execute("wall_create", json!({"start": (start + back).to_array(), "end": start.to_array()}));
        let u = path.length() * rng.between(0.2, 0.8);
        let side = if rng.next() < 0.5 { 1.0 } else { -1.0 };
        let (foot, out) = (path.at(u, 0.0), path.at(u, side * rng.between(1.0, 3.0)));
        let _ = s.execute("wall_create", json!({"start": out.to_array(), "end": foot.to_array()}));
        // Geometry.
        let geom = s.geometry().clone();
        for solid in &geom.solids {
            for f in &solid.faces {
                assert!(f.normal.is_finite() && f.verts.iter().all(|v| v.is_finite()), "round {round}");
            }
        }
        for g in geom.walls.values() {
            assert!(g.volume().is_finite() && g.volume() >= 0.0 && g.length.is_finite(), "round {round}");
            assert!(g.footprint().iter().all(|p| p.is_finite()), "round {round}");
        }
        let volume = geom.volume(ElementId(w));
        let g = &geom.walls[&ElementId(w)];
        assert!(volume > 0.0 && volume < g.length * 2.0 * g.half * 3.0 * 1.2, "round {round}: {volume}");
        // Views: the plan at each detail level, an elevation and the 3D view.
        let view = plan(&mut s);
        for detail in ["coarse", "medium", "fine"] {
            run(&mut s, "view_set", json!({"view": view, "detail": detail}));
            let d = s.drawing(ViewId(view)).unwrap();
            assert!(d.prims.iter().all(|p| match p {
                Prim::Line { a, b, .. } => a.is_finite() && b.is_finite(),
                Prim::Fill { pts, .. } => pts.iter().all(|p| p.is_finite()),
                _ => true,
            }));
        }
        if round % 4 == 0 {
            let elevation = id(&run(&mut s, "view_create", json!({"kind": "elevation", "direction": "east"})));
            assert!(s.drawing(ViewId(elevation)).is_some());
            let three_d = run(&mut s, "views_list", json!({"category": "three_d"}))[0]["id"].as_u64().unwrap();
            assert!(s.drawing(ViewId(three_d)).is_some());
        }
        assert_eq!(s.project().repairs_needed(), 0, "round {round}");
    }
    assert!(made >= 20, "most random curves make walls: {made}");
    // Unrepaired hostile curves through geometry and a plan: garbage in, no panic out.
    let mut s = Session::default();
    let view = plan(&mut s);
    let w = spline_wall(&mut s);
    let mut p = s.doc.project.clone();
    let template = p.elements.get(&ElementId(w)).cloned().unwrap();
    for i in 0..60u64 {
        let n = 1 + (rng.next() * 4.0) as usize;
        let mut control = Vec::new();
        for _ in 0..3 * n + 1 {
            control.push(DVec2::new(rng.between(-3.0, 3.0), rng.between(-3.0, 3.0) + 20.0 * i as f64));
        }
        let mut e = template.clone();
        e.id = ElementId(50_000 + i);
        e.uid = Default::default();
        if let ElementKind::Wall(wall) = &mut e.kind {
            (wall.start, wall.end) = (control[0], control[3 * n]);
            wall.spline = Some(WallSpline { control });
        }
        p.elements.insert(e.id, e);
    }
    let geom = cc_geom::ModelGeometry::build(&p);
    for solid in &geom.solids {
        for f in &solid.faces {
            assert!(f.verts.iter().all(|v| v.is_finite()));
        }
    }
    let v = p.view(ViewId(view)).cloned().unwrap();
    assert!(cc_views::generate(&p, &geom, &v).is_some());
    // Repair straightens every one of them that makes no wall.
    let before = p.walls().filter(|(_, w)| w.spline.is_some()).count();
    p.repair();
    let after = p.walls().filter(|(_, w)| w.spline.is_some()).count();
    assert!(after < before && after >= 1, "{before} -> {after}");
    assert_eq!(p.repairs_needed(), 0);
}

/// A wall type that grows too thick for the curve of a wall of it, or turns
/// into a curtain wall type, is stopped: the edit would leave the model
/// needing repair.
#[test]
fn a_type_change_that_breaks_a_spline_wall_is_refused() {
    let mut s = Session::default();
    let w = spline_wall(&mut s);
    let ty = s.project().element(ElementId(w)).unwrap().type_id.0;
    let before = run(&mut s, "type_get", json!({"type": ty}));
    let mut layers = before["params"]["layers"].clone();
    layers[1]["thickness"] = json!(4.0);
    let steps = s.doc.undo_depth();
    let (code, message) = fails(&mut s, "type_set", json!({"type": ty, "params": {"layers": layers}}));
    assert_eq!(code, ErrorCode::Rejected, "{message}");
    assert!(message.contains("invalid"), "{message}");
    assert_eq!(s.doc.undo_depth(), steps);
    assert_eq!(run(&mut s, "type_get", json!({"type": ty}))["params"], before["params"]);
    assert!(wall(&s, w).spline.is_some());
    // A little thicker is fine: the body follows.
    layers[1]["thickness"] = json!(0.2);
    run(&mut s, "type_set", json!({"type": ty, "params": {"layers": layers}}));
    assert!(wall(&s, w).spline.is_some());
    assert!(
        (s.geometry().walls[&ElementId(w)].half * 2.0 - num(&get(&mut s, w)["derived"]["thickness"])).abs() < 1e-12
    );
    assert_eq!(s.project().repairs_needed(), 0);
}
