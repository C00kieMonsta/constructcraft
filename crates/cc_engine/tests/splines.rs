//! Splines in sketches: the `spline` curve argument (through points or
//! control points), outlines drawn with them, and component pieces whose
//! profile points carry Bézier handles.

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_engine::{ErrorCode, Session};
use glam::DVec2;
use serde_json::{Value, json};

fn run(s: &mut Session, name: &str, args: Value) -> Value {
    s.execute(name, args).unwrap_or_else(|e| panic!("{name}: {e:?}"))
}

fn volume(s: &mut Session, id: u64) -> f64 {
    run(s, "element_get", json!({"id": id}))["derived"]["volume"].as_f64().unwrap()
}

/// The area a closed spline through `fit` encloses, exactly.
fn spline_area(fit: &[[f64; 2]]) -> f64 {
    let fit: Vec<DVec2> = fit.iter().map(|p| DVec2::from_array(*p)).collect();
    cc_core::spline::swept_area(&cc_core::spline::interpolate(&fit, true)).abs()
}

#[test]
fn outlines_drawn_with_splines() {
    let mut s = Session::default();
    let fit = [[0.0, 0.0], [4.0, 0.0], [5.0, 3.0], [1.0, 4.0]];
    let curve = json!({"kind": "spline", "through": fit, "closed": true});
    let mass = run(&mut s, "mass_create", json!({"curves": [curve], "height": 2.0}))["id"].as_u64().unwrap();
    let expected = spline_area(&fit) * 2.0;
    let v = volume(&mut s, mass);
    assert!((v / expected - 1.0).abs() < 0.01, "{v} {expected}");

    // sketch_get gives it back by its control points, which sketch_set takes.
    let got = run(&mut s, "sketch_get", json!({"id": mass}));
    let back = &got["curves"][0];
    assert_eq!(back["kind"], "spline");
    assert_eq!(back["control"].as_array().unwrap().len(), 13, "4 segments: {back}");
    run(&mut s, "sketch_set", json!({"id": mass, "curves": [back]}));
    assert!((volume(&mut s, mass) - v).abs() < 1e-9);

    // An open spline closed by a line.
    let half = json!([
        {"kind": "spline", "through": [[10, 0], [12, 1.5], [14, 0]]},
        {"kind": "line", "start": [14, 0], "end": [10, 0]},
    ]);
    run(&mut s, "mass_create", json!({"curves": half, "height": 1.0}));

    // Refused: one point, both forms, a closed list of control points, control points that are not 3n + 1.
    for bad in [
        json!({"kind": "spline", "through": [[0, 0]]}),
        json!({"kind": "spline", "through": [[0, 0], [1, 1]], "control": [[0, 0], [1, 0], [2, 0], [3, 0]]}),
        json!({"kind": "spline", "control": [[0, 0], [1, 0], [2, 0], [3, 0]], "closed": true}),
        json!({"kind": "spline", "control": [[0, 0], [1, 0], [2, 0]]}),
    ] {
        let code = s.execute("mass_create", json!({"curves": [bad], "height": 1.0})).unwrap_err().code;
        assert_eq!(code, ErrorCode::InvalidArguments, "{bad}");
    }
}

#[test]
fn component_pieces_take_bezier_points() {
    let mut s = Session::default();
    // A closed spline through four points, extruded 0.5 m: Bézier points.
    let fit = [[0.0, 0.0], [1.0, 0.0], [1.2, 0.8], [0.1, 1.0]];
    let pts: Vec<DVec2> = fit.iter().map(|p| DVec2::from_array(*p)).collect();
    let control = cc_core::spline::interpolate(&pts, true);
    let points: Vec<Value> = cc_core::spline::segments(&control)
        .map(|[a, c1, c2, _]| {
            let t = |v: f64| format!("{v}");
            json!({"u": t(a.x), "v": t(a.y), "bezier": {"u1": t(c1.x), "v1": t(c1.y), "u2": t(c2.x), "v2": t(c2.y)}})
        })
        .collect();
    let def = json!({"items": [{
        "name": "Body",
        "shape": {"Profile": {"plane": "Plan", "points": points, "offset": "0", "depth": "0.5"}},
        "material": "",
        "in_plan": true,
    }]});
    let ty = run(&mut s, "component_create", json!({"name": "Curvy", "def": def}))["id"].as_u64().unwrap();
    let placed = run(&mut s, "component_place", json!({"type": ty, "position": [3.0, 3.0]}))["id"].as_u64().unwrap();
    let expected = spline_area(&fit) * 0.5;
    let v = volume(&mut s, placed);
    assert!((v / expected - 1.0).abs() < 0.01, "{v} {expected}");
    // The handles show in the definition.
    let got = run(&mut s, "component_get", json!({"type": ty}));
    assert!(got["def"]["items"][0]["shape"]["Profile"]["points"][0]["bezier"]["u1"].is_string(), "{}", got["def"]);
}
