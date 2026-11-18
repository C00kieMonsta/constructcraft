//! Sweeps along arcs and circles (issue #366): `sweep_create` takes the path
//! as `curves`; the profile follows the arcs square to the path, the curved
//! sides are smooth, and the volume is exact (Pappus).

use cc_core::{ElementId, ElementKind, ViewKind};
use cc_engine::{ErrorCode, Session};
use cc_views::Prim;
use glam::DVec2;
use serde_json::{Value, json};
use std::f64::consts::{PI, TAU};

fn run(s: &mut Session, name: &str, args: Value) -> Value {
    s.execute(name, args).unwrap_or_else(|e| panic!("{name}: {e:?}"))
}

fn fails(s: &mut Session, name: &str, args: Value) -> ErrorCode {
    s.execute(name, args).expect_err("should fail").code
}

fn id(v: &Value) -> ElementId {
    ElementId(v["id"].as_u64().unwrap())
}

fn volume(s: &mut Session, id: ElementId) -> f64 {
    run(s, "element_get", json!({"id": id.0}))["derived"]["volume"].as_f64().unwrap()
}

fn relative(a: f64, b: f64) -> f64 {
    (a - b).abs() / b.abs()
}

/// The issue's haunch: a 15 × 15 cm triangle in the corner of a tank's wall
/// and its bottom slab, its right angle on the wall's inner face.
fn haunch_type(s: &mut Session) -> Value {
    let triangle = json!([[0, 0], [0.15, 0], [0, 0.15]]);
    run(s, "sweep_type_create", json!({"name": "Haunch 15x15", "profile": triangle}))["id"].clone()
}

/// Inner face radius of the tank.
const R: f64 = 3.0;

#[test]
fn a_haunch_swept_around_a_circle_has_the_exact_volume() {
    let mut s = Session::default();
    let ty = haunch_type(&mut s);
    // Counter-clockwise around the inner face: the profile, on the left, is inside the tank.
    let circle = json!([{"kind": "circle", "center": [0, 0], "radius": R}]);
    let haunch = id(&run(&mut s, "sweep_create", json!({"curves": circle, "type": ty})));
    // Pappus: the triangle's area, 0.15² / 2, times the path of its centroid,
    // which lies 0.15 / 3 = 0.05 inside the face: 2π (R - 0.05).
    let exact = 0.15 * 0.15 / 2.0 * TAU * (R - 0.05);
    let v = volume(&mut s, haunch);
    assert!(relative(v, exact) < 1e-9, "{v} vs {exact}");
    let got = run(&mut s, "element_get", json!({"id": haunch.0}));
    assert!(relative(got["derived"]["length"].as_f64().unwrap(), TAU * R) < 1e-12, "{got}");
    assert_eq!(got["params"]["closed"], true);
    assert_eq!(got["params"]["bulges"], json!([1.0, 1.0]));

    // The solid meshes the circle with chords, a hair short of that; its
    // sloped and upright sides are smooth, its flat bottom is not.
    let geom = s.geometry().clone();
    let solid = geom.solids.iter().find(|x| x.element == haunch).expect("a body");
    assert!(solid.volume() < exact && exact - solid.volume() < exact * 1e-3, "{}", solid.volume());
    let smooth = solid.faces.iter().filter(|f| f.smooth).count();
    assert!(smooth >= 2 * 120 && solid.faces.iter().any(|f| !f.smooth), "{smooth}");
    // Every section is square to the path: its points stand on circles around the center.
    for f in &solid.faces {
        for p in &f.verts {
            let r = p.truncate().length();
            assert!([R, R - 0.15].iter().any(|c| (r - c).abs() < 1e-9), "{p}");
        }
    }

    // The plan draws the two circles in short chords and nothing across them.
    let plan = s.project().views.values().find(|v| matches!(v.kind, ViewKind::FloorPlan { .. })).map(|v| v.id).unwrap();
    let lines: Vec<(DVec2, DVec2)> = s
        .drawing(plan)
        .unwrap()
        .prims
        .iter()
        .filter_map(|p| match p {
            Prim::Line { a, b, element: Some(e), .. } if *e == haunch => Some((*a, *b)),
            _ => None,
        })
        .collect();
    assert!(lines.len() >= 2 * 120, "{}", lines.len());
    let center = lines.iter().map(|(a, b)| *a + *b).sum::<DVec2>() / (2 * lines.len()) as f64;
    for (a, b) in &lines {
        assert!((a.distance(center) - b.distance(center)).abs() < 1e-6, "a line across: {a} {b}");
    }
    // A section through the tank draws it too.
    let section = run(&mut s, "view_create", json!({"kind": "section", "start": [-5, 0], "end": [5, 0]}));
    let drawing = s.drawing(cc_core::ViewId(section["id"].as_u64().unwrap())).unwrap();
    assert!(drawing.prims.iter().any(|p| p.element() == Some(haunch)));
}

#[test]
fn an_arc_between_lines_is_an_open_path_with_the_exact_volume() {
    let mut s = Session::default();
    let ty = haunch_type(&mut s);
    // Up the right side 3 m, round a half circle of radius R counter-clockwise, down 3 m:
    // the curves out of order and the last line drawn backwards.
    let curves = json!([
        {"kind": "arc", "start": [R, 0], "end": [-R, 0], "bulge": 1.0},
        {"kind": "line", "start": [R, -3], "end": [R, 0]},
        {"kind": "line", "start": [-R, -3], "end": [-R, 0]}
    ]);
    let sweep = id(&run(&mut s, "sweep_create", json!({"curves": curves, "type": ty, "base_offset": 0.5})));
    let got = run(&mut s, "element_get", json!({"id": sweep.0}));
    assert_eq!(got["params"]["path"], json!([[R, -3.0], [R, 0.0], [-R, 0.0], [-R, -3.0]]));
    assert_eq!(got["params"]["bulges"], json!([0.0, 1.0, 0.0]));
    assert_eq!(got["params"]["closed"], false);
    // The lines meet the arc tangent to it, so no miter: the triangle's area
    // times 3 m, half the circle at the centroid's radius, and 3 m.
    let area = 0.15 * 0.15 / 2.0;
    let exact = area * (3.0 + PI * (R - 0.05) + 3.0);
    let v = volume(&mut s, sweep);
    assert!(relative(v, exact) < 1e-9, "{v} vs {exact}");
    assert!(relative(got["derived"]["length"].as_f64().unwrap(), 6.0 + PI * R) < 1e-12);

    // element_set round-trips the curves: the params back as they are change nothing,
    // and a quarter circle in place of the half one is taken.
    let params = got["params"].clone();
    run(&mut s, "element_set", json!({"id": sweep.0, "params": params}));
    assert!(relative(volume(&mut s, sweep), exact) < 1e-9);
    // Up 3 m, a quarter circle round to (0, R), on 3 m to the left: tangent all the way.
    let quarter = (PI / 8.0).tan();
    run(
        &mut s,
        "element_set",
        json!({"id": sweep.0, "params": {"path": [[R, -3], [R, 0], [0, R], [-3, R]], "bulges": [0, quarter, 0]}}),
    );
    let exact = area * (3.0 + PI / 2.0 * (R - 0.05) + 3.0);
    assert!(relative(volume(&mut s, sweep), exact) < 1e-9);
    // A bulge that is not a number, or an arc of a full turn, is refused.
    assert!(s.execute("element_set", json!({"id": sweep.0, "params": {"bulges": [0, 50, 0]}})).is_err());

    // Mirrored about x = 10 and rotated a quarter turn, it keeps its volume and its arc:
    // the mirror turns the arc the other way between the mirrored points.
    let mirrored = run(&mut s, "elements_mirror", json!({"ids": [sweep.0], "axis": [[10, 0], [10, 1]], "copy": true}));
    let copy = ElementId(mirrored["created"][0].as_u64().unwrap());
    assert!(relative(volume(&mut s, copy), exact) < 1e-9);
    let ElementKind::Sweep(sw) = &s.project().element(copy).unwrap().kind else { panic!("a sweep") };
    assert!(sw.bulges[1] < 0.0 && sw.flip);
    let arc = sw.segments()[1].2.expect("an arc");
    assert!(arc.center.distance(DVec2::new(20.0, 0.0)) < 1e-9 && (arc.radius - R).abs() < 1e-9, "{arc:?}");
    run(&mut s, "elements_rotate", json!({"ids": [sweep.0], "angle": 90, "center": [0, 0]}));
    assert!(relative(volume(&mut s, sweep), exact) < 1e-9);
    let ElementKind::Sweep(sw) = &s.project().element(sweep).unwrap().kind else { panic!("a sweep") };
    let arc = sw.segments()[1].2.expect("an arc");
    assert!(arc.center.length() < 1e-9 && (arc.radius - R).abs() < 1e-9);

    // A line meeting an arc at a corner is mitered: the centroid's path loses
    // 2 c tan(θ / 2) there (here the path turns 90° right, so it gains 2 × 0.05).
    let corner = id(&run(
        &mut s,
        "sweep_create",
        json!({"curves": [
            {"kind": "arc", "start": [R, 0], "end": [0, R], "bulge": quarter},
            {"kind": "line", "start": [0, R], "end": [0, R + 3.0]}
        ], "type": ty}),
    ));
    let exact = area * (PI / 2.0 * (R - 0.05) + 3.0 + 2.0 * 0.05);
    assert!(relative(volume(&mut s, corner), exact) < 1e-9);
    let geom = s.geometry().clone();
    let solid = geom.solids.iter().find(|x| x.element == corner).expect("a body");
    assert!(relative(solid.volume(), exact) < 1e-3, "{} vs {exact}", solid.volume());
}

#[test]
fn curves_that_make_no_path_are_refused() {
    let mut s = Session::default();
    let depth = s.doc.undo_depth();
    let circle = json!([{"kind": "circle", "center": [0, 0], "radius": 2}]);
    let whole = circle.get(0).cloned().unwrap();
    let refused = [
        // Points and curves.
        json!({"path": [[0, 0], [1, 0]], "curves": circle}),
        // Neither.
        json!({}),
        // A circle of no size, a radius that is not a number.
        json!({"curves": [{"kind": "circle", "center": [0, 0], "radius": 0}]}),
        json!({"curves": [{"kind": "circle", "center": [0, 0], "radius": f64::NAN}]}),
        json!({"curves": [{"kind": "arc", "start": [0, 0], "end": [1, 0], "bulge": "x"}]}),
        // A circle with other curves, curves apart, a branch (the path touching itself).
        json!({"curves": [whole, {"kind": "line", "start": [2, 0], "end": [4, 0]}]}),
        json!({"curves": [
            {"kind": "line", "start": [0, 0], "end": [1, 0]},
            {"kind": "line", "start": [5, 5], "end": [6, 5]}
        ]}),
        json!({"curves": [
            {"kind": "line", "start": [0, 0], "end": [2, 0]},
            {"kind": "arc", "start": [2, 0], "end": [2, 2], "bulge": 1},
            {"kind": "line", "start": [2, 2], "end": [2, 0]},
            {"kind": "line", "start": [2, 0], "end": [4, 0]}
        ]}),
        // An ellipse, and a closed path asked of open curves.
        json!({"curves": [{"kind": "ellipse", "center": [0, 0], "radii": [2, 1]}]}),
        json!({"curves": [{"kind": "arc", "start": [0, 0], "end": [2, 0], "bulge": 0.5}], "closed": true}),
    ];
    for args in refused {
        let code = s.execute("sweep_create", args.clone()).expect_err(&args.to_string()).code;
        assert_eq!(code, ErrorCode::InvalidArguments, "{args}");
    }
    assert_eq!(
        fails(&mut s, "sweep_create", json!({"path": [[0, 0], [1, 0]], "closed": true})),
        ErrorCode::InvalidArguments
    );
    assert_eq!(s.doc.undo_depth(), depth);
}

#[test]
fn files_without_arcs_load_and_curved_sweeps_save() {
    let mut s = Session::default();
    let straight = id(&run(&mut s, "sweep_create", json!({"path": [[0, 0], [4, 0]]})));
    let circle = json!([{"kind": "circle", "center": [10, 0], "radius": 2}]);
    let round = id(&run(&mut s, "sweep_create", json!({"curves": circle})));
    let text = cc_core::to_ron(s.project()).unwrap();
    // A straight sweep writes no bulges, as files before them.
    let mut old = s.project().clone();
    old.elements.remove(&round);
    let old_text = cc_core::to_ron(&old).unwrap();
    assert!(!old_text.contains("bulges"));
    let (loaded, repairs) = cc_core::from_ron_with_repairs(&old_text).unwrap();
    assert_eq!(repairs, 0);
    assert!(matches!(&loaded.element(straight).unwrap().kind, ElementKind::Sweep(sw) if sw.bulges.is_empty()));
    // The circle comes back as a circle.
    let (loaded, repairs) = cc_core::from_ron_with_repairs(&text).unwrap();
    assert_eq!(repairs, 0);
    let ElementKind::Sweep(sw) = &loaded.element(round).unwrap().kind else { panic!("a sweep") };
    assert_eq!(sw.bulges, [1.0, 1.0]);
    assert!(sw.closed && (sw.length() - TAU * 2.0).abs() < 1e-12);
}
