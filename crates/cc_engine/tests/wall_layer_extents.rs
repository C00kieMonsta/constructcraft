//! Per-layer vertical extents of compound walls (issue #306, part 2): a
//! wall type's layers take a `base_offset` and a `height` through
//! `type_set`, the wall's body is stepped across it, and `element_get`
//! gives each layer's volume and extent.

use cc_core::ElementId;
use cc_engine::{ErrorCode, Session};
use cc_views::{LineStyle, Prim};
use serde_json::{Value, json};

fn run(s: &mut Session, name: &str, args: Value) -> Value {
    s.execute(name, args).unwrap_or_else(|e| panic!("{name}: {e:?}"))
}

fn fails(s: &mut Session, name: &str, args: Value) -> (ErrorCode, String) {
    let e = s.execute(name, args).map(|v| panic!("{name} should fail, gave {v}")).unwrap_err();
    (e.code, e.message)
}

fn num(v: &Value) -> f64 {
    v.as_f64().unwrap_or_else(|| panic!("not a number: {v}"))
}

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 1e-6, "{what}: {a} vs {b}");
}

/// A copy of the default wall type (plaster, masonry, plaster) and its layers as `type_get` shows them.
fn layered_type(s: &mut Session) -> (u64, Vec<Value>) {
    let walls = run(s, "types_list", json!({"category": "wall"}));
    let first =
        walls.as_array().and_then(|a| a.iter().find(|t| t["default"] == true)).map(|t| t["id"].clone()).unwrap();
    let copy = run(s, "type_duplicate", json!({"type": first, "name": "Insulated"}));
    let id = copy["id"].as_u64().unwrap();
    let layers = run(s, "type_get", json!({"type": id}))["params"]["layers"].as_array().unwrap().clone();
    assert_eq!(layers.len(), 3, "{layers:?}");
    (id, layers)
}

/// The type's layers with the outer one from 1 m up, the core 0.5 m deeper
/// and the inner one 2 m high.
fn with_extents(mut layers: Vec<Value>) -> Vec<Value> {
    layers[0]["base_offset"] = json!(1.0);
    layers[1]["base_offset"] = json!(-0.5);
    layers[2]["height"] = json!(2.0);
    layers
}

fn thicknesses(layers: &[Value]) -> Vec<f64> {
    layers.iter().map(|l| num(&l["thickness"])).collect()
}

#[test]
fn layer_extents_ride_on_type_set() {
    let mut s = Session::default();
    let (ty, layers) = layered_type(&mut s);
    assert!(layers[0].get("base_offset").is_none(), "default extents are not shown: {:?}", layers[0]);
    let steps = s.doc.undo_depth();
    let out = run(&mut s, "type_set", json!({"type": ty, "params": {"layers": with_extents(layers.clone())}}));
    assert_eq!(s.doc.undo_depth(), steps + 1, "one undo step");
    let got = out["params"]["layers"].as_array().unwrap();
    close(num(&got[0]["base_offset"]), 1.0, "outer offset");
    close(num(&got[1]["base_offset"]), -0.5, "core offset");
    close(num(&got[2]["height"]), 2.0, "inner height");
    assert!(got[0]["height"].is_null(), "up to the wall's top");
    // A height of null takes it back up to the top.
    let mut back = got.clone();
    back[2]["height"] = Value::Null;
    let out = run(&mut s, "type_set", json!({"type": ty, "params": {"layers": back}}));
    assert!(out["params"]["layers"][2].get("height").is_none());
    run(&mut s, "undo", json!({}));
    run(&mut s, "undo", json!({}));
    let undone = run(&mut s, "type_get", json!({"type": ty}))["params"]["layers"].clone();
    assert_eq!(undone, json!(layers), "undone");
}

#[test]
fn out_of_bounds_extents_are_refused() {
    let mut s = Session::default();
    let (ty, layers) = layered_type(&mut s);
    let steps = s.doc.undo_depth();
    let bad = |field: &str, value: Value, at: usize| {
        let mut l = layers.clone();
        l[at][field] = value;
        json!({"type": ty, "params": {"layers": l}})
    };
    for (field, value, at) in [
        ("base_offset", json!(150.0), 1),
        ("base_offset", json!(-101.0), 0),
        ("height", json!(0.0), 2),
        ("height", json!(-1.0), 2),
        ("height", json!(5000.0), 0),
    ] {
        let (code, message) = fails(&mut s, "type_set", bad(field, value, at));
        assert_eq!(code, ErrorCode::InvalidArguments, "{message}");
        assert!(message.contains(&format!("layer {}", at + 1)) && message.contains(field), "{message}");
    }
    // Floors take no extents.
    let floors = run(&mut s, "types_list", json!({"category": "floor"}));
    let floor = floors.as_array().and_then(|a| a.first()).map(|t| t["id"].clone()).unwrap();
    let mut fl = run(&mut s, "type_get", json!({"type": floor}))["params"]["layers"].clone();
    fl[0]["base_offset"] = json!(0.2);
    let (code, _) = fails(&mut s, "type_set", json!({"type": floor, "params": {"layers": fl}}));
    assert_eq!(code, ErrorCode::InvalidArguments);
    assert_eq!(s.doc.undo_depth(), steps, "nothing was changed");
}

#[test]
fn element_get_gives_each_layers_volume_and_extent() {
    let mut s = Session::default();
    let (ty, layers) = layered_type(&mut s);
    let t = thicknesses(&layers);
    let wall = run(&mut s, "wall_create", json!({"start": [0, 0], "end": [6, 0], "height": 3.0, "type": ty}));
    let id = wall["id"].as_u64().unwrap();
    let plain = run(&mut s, "element_get", json!({"id": id}))["derived"].clone();
    close(num(&plain["volume"]), t.iter().sum::<f64>() * 18.0, "plain");
    let derived_layers = plain["layers"].as_array().unwrap();
    assert_eq!(derived_layers.len(), 3);
    close(num(&derived_layers[1]["base"]), 0.0, "plain base");
    close(num(&derived_layers[1]["top"]), 3.0, "plain top");

    run(&mut s, "type_set", json!({"type": ty, "params": {"layers": with_extents(layers.clone())}}));
    let d = run(&mut s, "element_get", json!({"id": id}))["derived"].clone();
    let l = d["layers"].as_array().unwrap();
    let expect = [(t[0] * 12.0, 1.0, 3.0), (t[1] * 21.0, -0.5, 3.0), (t[2] * 12.0, 0.0, 2.0)];
    for (k, (volume, base, top)) in expect.into_iter().enumerate() {
        close(num(&l[k]["volume"]), volume, "layer volume");
        close(num(&l[k]["base"]), base, "layer base");
        close(num(&l[k]["top"]), top, "layer top");
        close(num(&l[k]["thickness"]), t[k], "layer thickness");
        assert_eq!(l[k]["material"], layers[k]["material"]);
    }
    // The wall's volume is the true total.
    close(num(&d["volume"]), expect.iter().map(|e| e.0).sum(), "total");
    // A schedule of walls reads it too.
    let g = s.geometry();
    close(g.volume(ElementId(id)), num(&d["volume"]), "schedule volume");
}

/// A wall attached to a roof: the inner layer, 2 m high, meets the sloped
/// top of a gable wall; the body stays closed and its layers add up.
#[test]
fn layers_under_an_attached_top() {
    let mut s = Session::default();
    let (ty, layers) = layered_type(&mut s);
    let mut l = with_extents(layers);
    l[2]["height"] = json!(3.5);
    run(&mut s, "type_set", json!({"type": ty, "params": {"layers": l}}));
    let corners = [[0.0, 0.0], [8.0, 0.0], [8.0, 6.0], [0.0, 6.0]];
    let walls: Vec<u64> = (0..4)
        .map(|i| {
            let args = json!({"start": corners[i], "end": corners[(i + 1) % 4], "height": 3.0, "type": ty});
            run(&mut s, "wall_create", args)["id"].as_u64().unwrap()
        })
        .collect();
    let roof = run(&mut s, "roof_create", json!({"boundary": corners, "offset": 3.0, "overhang": 0.0}))["id"].clone();
    run(&mut s, "element_set", json!({"id": roof, "params": {"shape": {"Gable": {"slope": 0.5, "ridge_deg": 0.0}}}}));
    run(&mut s, "wall_attach", json!({"ids": walls, "top": roof}));
    let g = s.geometry();
    let gable = &g.walls[&ElementId(walls[1])];
    assert!(gable.is_banded() && !gable.top_profile.is_empty(), "a banded wall under the roof");
    let mut closed = gable.clone();
    (closed.start_joined, closed.end_joined) = (false, false);
    let volume = gable.volume();
    close(closed.to_solid([0; 3]).volume(), volume, "closed mesh");
    close(gable.layer_volumes().iter().sum(), volume, "layers add up");
    // The inner layer stops 3.5 m up where the gable rises past it.
    let (_, top) = gable.layer_range(2).unwrap();
    close(top, 3.5, "inner layer's top");
}

/// A section through a wall whose outer layer starts 1 m up: the cut is one
/// closed, stepped outline, filled in the wall's cut material.
#[test]
fn a_section_cuts_the_stepped_body() {
    let mut s = Session::default();
    let (ty, layers) = layered_type(&mut s);
    run(&mut s, "type_set", json!({"type": ty, "params": {"layers": with_extents(layers)}}));
    let wall = run(&mut s, "wall_create", json!({"start": [0, 0], "end": [6, 0], "height": 3.0, "type": ty}));
    let wall = ElementId(wall["id"].as_u64().unwrap());
    let section = run(&mut s, "view_create", json!({"kind": "section", "start": [3.0, -2.0], "end": [3.0, 2.0]}));
    let view = cc_core::ViewId(section["id"].as_u64().unwrap());
    let d = s.drawing(view).unwrap();
    let cut_lines = d
        .prims
        .iter()
        .filter(|p| matches!(p, Prim::Line { style: LineStyle::Cut, element: Some(e), .. } if *e == wall))
        .count();
    // Outer layer, core and inner layer: a stepped outline of more than four sides.
    assert!(cut_lines > 4, "{cut_lines}");
    let fills: Vec<&Vec<glam::DVec2>> = d
        .prims
        .iter()
        .filter_map(|p| match p {
            Prim::Fill { pts, cut: true, element: Some(e), .. } if *e == wall => Some(pts),
            _ => None,
        })
        .collect();
    assert!(!fills.is_empty(), "the outline closes and is filled");
    let (lo, hi) =
        fills.iter().flat_map(|f| f.iter()).fold((f64::MAX, f64::MIN), |(lo, hi), p| (lo.min(p.y), hi.max(p.y)));
    close(hi - lo, 3.5, "from the core's bottom to the top");
}
