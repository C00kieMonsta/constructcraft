//! Gable roofs with a notch cut into an eave right up to a corner, or with
//! an opening one of their planes runs round (issue #380): the ridge runs
//! the whole length, nothing rises above it, and no line is left where a
//! plane meets itself again past the notch or the opening.

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_engine::Session;
use serde_json::{Value, json};

fn run(s: &mut Session, name: &str, args: Value) -> Value {
    s.execute(name, args).unwrap_or_else(|e| panic!("{name}: {e:?}"))
}

fn id(v: &Value) -> u64 {
    v["id"].as_u64().unwrap_or_else(|| panic!("no id in {v}"))
}

const SLOPE: f64 = 0.42;

/// A roof made a gable roof along X at 42 %, with the given edge slopes:
/// its element as `element_set` gives it back.
fn gable_roof(s: &mut Session, boundary: Value, holes: Value, edge_slopes: Value) -> Value {
    let args = json!({"boundary": boundary, "holes": holes, "overhang": 0, "offset": 3});
    let roof = id(&run(s, "roof_create", args));
    let params = json!({"shape": {"Gable": {"ridge_deg": 0, "slope": SLOPE}}, "edge_slopes": edge_slopes});
    run(s, "element_set", json!({"id": roof, "params": params}))
}

fn ridges(roof: &Value) -> Vec<[[f64; 2]; 2]> {
    let p = |v: &Value| [v[0].as_f64().unwrap(), v[1].as_f64().unwrap()];
    roof["derived"]["ridges"].as_array().unwrap().iter().map(|r| [p(&r[0]), p(&r[1])]).collect()
}

/// The roof's slab: its lowest and highest points.
fn heights(s: &mut Session, roof: &Value) -> (f64, f64) {
    s.geometry()
        .body_of(cc_core::ElementId(id(roof)))
        .flat_map(|solid| solid.faces.iter().flat_map(|f| f.verts.iter().map(|p| p.z)))
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), z| (lo.min(z), hi.max(z)))
}

/// The slab's thickness across the roof, from its type's layers.
fn thickness(s: &mut Session, roof: &Value) -> f64 {
    let t = run(s, "type_get", json!({"type": roof["type"]["id"]}));
    t["params"]["layers"].as_array().unwrap().iter().map(|l| l["thickness"].as_f64().unwrap()).sum()
}

/// One ridge, the whole 13.716 m length midway across the 7.01 m depth; the
/// slab's top at the ridge, and its volume its plan area times its thickness.
fn check_full_ridge(s: &mut Session, roof: &Value) {
    let lines = ridges(roof);
    assert_eq!(lines.len(), 1, "one ridge: {lines:?}");
    let [a, b] = lines[0];
    assert!((a[1] - 3.505).abs() < 1e-9 && (b[1] - 3.505).abs() < 1e-9, "{lines:?}");
    assert!(((a[0] - b[0]).abs() - 13.716).abs() < 1e-9, "the whole length: {lines:?}");
    let vertical = thickness(s, roof) * (1.0 + SLOPE * SLOPE).sqrt();
    let (bottom, top) = heights(s, roof);
    assert!((top - bottom - (3.505 * SLOPE + vertical)).abs() < 1e-9, "nothing above the ridge: {bottom} to {top}");
    let (volume, area) = (roof["derived"]["volume"].as_f64().unwrap(), roof["derived"]["plan_area"].as_f64().unwrap());
    assert!((volume - area * vertical).abs() < 1e-6, "{volume} vs {}", area * vertical);
}

#[test]
fn a_notch_up_to_a_corner_keeps_the_ridge() {
    let mut s = Session::default();
    run(&mut s, "project_new", json!({}));
    // The notch's sides, and the east and west ends, are gables.
    let boundary = json!([[0, 0], [13.716, 0], [13.716, 7.01], [11.125, 4.419], [8.534, 7.01], [0, 7.01]]);
    let roof = gable_roof(&mut s, boundary, json!([]), json!([null, 0, 0, 0, null, 0]));
    check_full_ridge(&mut s, &roof);
    // Ending 1 cm short of the corner (a piece of eave left there): the same
    // roof, with no line from the notch's inner point down to the ridge.
    let boundary =
        json!([[0, 0], [13.716, 0], [13.716, 7.01], [13.706, 7.01], [11.125, 4.419], [8.534, 7.01], [0, 7.01]]);
    let roof = gable_roof(&mut s, boundary, json!([]), json!([null, 0, null, 0, 0, null, 0]));
    check_full_ridge(&mut s, &roof);
}

#[test]
fn a_plane_round_an_opening_leaves_no_line() {
    let mut s = Session::default();
    run(&mut s, "project_new", json!({}));
    // Its sides steeper than 45° to the ridge are gables, its top a sloped eave:
    // the north plane runs round it and meets itself again below it.
    let boundary = json!([[0, 0], [13.716, 0], [13.716, 7.01], [0, 7.01]]);
    let roof = gable_roof(&mut s, boundary, json!([[[9.5, 6.5], [12.5, 6.5], [11.0, 4.4]]]), json!([]));
    let lines = ridges(&roof);
    assert!(!lines.iter().any(|[a, b]| (a[0] - 11.0).abs() < 1e-6 && (b[0] - 11.0).abs() < 1e-6), "{lines:?}");
    let vertical = thickness(&mut s, &roof) * (1.0 + SLOPE * SLOPE).sqrt();
    let (volume, area) = (roof["derived"]["volume"].as_f64().unwrap(), roof["derived"]["plan_area"].as_f64().unwrap());
    assert!((area - (13.716 * 7.01 - 3.0 * 2.1 / 2.0)).abs() < 1e-9, "{area}");
    assert!((volume - area * vertical).abs() < 1e-6, "the opening left out: {volume} vs {}", area * vertical);
}
