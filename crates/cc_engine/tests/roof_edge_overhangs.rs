//! A roof's overhang per edge (issue #382): eaves, rakes and edges that
//! meet a wall reach out by their own amounts, their eaves staying on the
//! roof planes the roof's own overhang fixes.

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_core::{ElementId, ElementKind};
use cc_engine::{ErrorCode, Session};
use glam::DVec3;
use serde_json::{Value, json};

fn run(s: &mut Session, name: &str, args: Value) -> Value {
    s.execute(name, args).unwrap_or_else(|e| panic!("{name}: {e:?}"))
}

fn fails(s: &mut Session, name: &str, args: Value) -> ErrorCode {
    s.execute(name, args).map(|v| panic!("{name} should fail, gave {v}")).unwrap_err().code
}

fn id(v: &Value) -> u64 {
    v["id"].as_u64().unwrap_or_else(|| panic!("no id in {v}"))
}

const SLOPE: f64 = 0.5;
const OVERHANG: f64 = 0.5;

/// A 10 × 6 m roof 3 m above its level with a 0.5 m overhang, gable (ridge
/// along x: eaves are edges 0 and 2, rakes 1 and 3) or hip at half pitch.
fn roof(s: &mut Session, shape: Value) -> u64 {
    let rect = json!([[0, 0], [10, 0], [10, 6], [0, 6]]);
    let r = id(&run(s, "roof_create", json!({"boundary": rect, "offset": 3.0, "overhang": OVERHANG})));
    run(s, "element_set", json!({"id": r, "params": {"shape": shape}}));
    r
}

fn gable() -> Value {
    json!({"Gable": {"slope": SLOPE, "ridge_deg": 0.0}})
}

fn hip() -> Value {
    json!({"Hip": {"slope": SLOPE}})
}

fn vertices(s: &mut Session, id: u64) -> Vec<DVec3> {
    s.geometry().body_of(ElementId(id)).flat_map(|x| x.faces.iter().flat_map(|f| f.verts.clone())).collect()
}

/// The corners of the faces turned down: the roof's underside.
fn underside(s: &mut Session, id: u64) -> Vec<DVec3> {
    s.geometry()
        .body_of(ElementId(id))
        .flat_map(|x| x.faces.iter().filter(|f| f.normal.z < -0.1).flat_map(|f| f.verts.clone()))
        .collect()
}

fn bounds(s: &mut Session, id: u64) -> (DVec3, DVec3) {
    vertices(s, id).into_iter().fold((DVec3::INFINITY, DVec3::NEG_INFINITY), |(lo, hi), p| (lo.min(p), hi.max(p)))
}

fn derived(s: &mut Session, id: u64, key: &str) -> f64 {
    run(s, "element_get", json!({"id": id}))["derived"][key].as_f64().unwrap_or_else(|| panic!("no {key}"))
}

/// The underside's height above the roof base where the overhang is the
/// roof's: each plane passes its wall line `slope × overhang` above it and
/// rises inwards; over a convex outline the lowest plane is the roof.
/// `inside` gives the distance in from each sloped wall line.
fn plane_height(inside: &[f64]) -> f64 {
    SLOPE * OVERHANG + SLOPE * inside.iter().copied().fold(f64::INFINITY, f64::min)
}

/// The level's base for the roofs here: the eave tips of a roof with the
/// same overhang everywhere.
fn base(s: &mut Session, id: u64) -> f64 {
    underside(s, id).iter().map(|p| p.z).fold(f64::INFINITY, f64::min)
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// Vertical thickness of the slab: volume over plan area, the same for
/// every roof built from closed slabs of one type and slope.
fn thickness(s: &mut Session, id: u64) -> f64 {
    derived(s, id, "volume") / derived(s, id, "plan_area")
}

/// A gable roof with 0.762 m at the eaves and 0.3 m at the rakes: the plan
/// outline reaches exactly that far past each edge, and the eave tips, lower
/// than the roof's base, lie on the planes of the roof's own overhang.
#[test]
fn eaves_and_rakes_reach_their_own_overhangs() {
    let mut s = Session::default();
    let reference = roof(&mut s, gable());
    let z0 = base(&mut s, reference);
    let thick = thickness(&mut s, reference);
    run(&mut s, "elements_delete", json!({"ids": [reference]}));

    let r = roof(&mut s, gable());
    for (edge, overhang) in [(0, 0.762), (1, 0.3), (2, 0.762), (3, 0.3)] {
        run(&mut s, "roof_edge_set", json!({"id": r, "edge": edge, "overhang": overhang}));
    }
    let got = run(&mut s, "element_get", json!({"id": r}));
    assert_eq!(got["params"]["edge_overhangs"], json!([0.762, 0.3, 0.762, 0.3]));
    // The slopes are left alone.
    assert!(got["params"].get("edge_slopes").is_none(), "{got}");

    let (lo, hi) = bounds(&mut s, r);
    assert!(close(lo.x, -0.3) && close(hi.x, 10.3), "{lo} {hi}");
    assert!(close(lo.y, -0.762) && close(hi.y, 6.762), "{lo} {hi}");
    assert!(close(derived(&mut s, r, "plan_area"), 10.6 * 7.524));

    // Every corner of the underside lies on its plane (south rising from
    // y = 0, north from y = 6); the eave tips 0.262 m out and 0.131 m down.
    for p in underside(&mut s, r) {
        let want = z0 + plane_height(&[p.y, 6.0 - p.y]);
        assert!(close(p.z, want), "{p} should be at {want}");
    }
    assert!(close(lo.z, z0 + SLOPE * (OVERHANG - 0.762)), "{}", lo.z);
    // A closed slab of the same vertical thickness.
    assert!((thickness(&mut s, r) - thick).abs() < 1e-9);
}

/// An eave with no overhang (where the roof meets a wall) stops at its wall
/// line, as high as the plane is there; on a hip roof its neighbors end
/// against it until the roof rises past it. `null` gives the edge the
/// roof's overhang back, and with no edge of its own the roof is built as
/// before, bit for bit.
#[test]
fn an_edge_without_overhang_and_back() {
    let mut s = Session::default();
    let reference = roof(&mut s, hip());
    let z0 = base(&mut s, reference);
    let thick = thickness(&mut s, reference);
    let before = vertices(&mut s, reference);
    run(&mut s, "elements_delete", json!({"ids": [reference]}));

    for shape in [gable(), hip()] {
        let hipped = shape.get("Hip").is_some();
        let r = roof(&mut s, shape.clone());
        run(&mut s, "roof_edge_set", json!({"id": r, "edge": 0, "overhang": 0.0}));
        let (lo, hi) = bounds(&mut s, r);
        assert!(close(lo.y, 0.0) && close(hi.y, 6.5), "{shape}: {lo} {hi}");
        assert!(close(lo.x, -0.5) && close(hi.x, 10.5), "{shape}: {lo} {hi}");
        for p in underside(&mut s, r) {
            let mut inside = vec![p.y, 6.0 - p.y];
            if hipped {
                inside.extend([p.x, 10.0 - p.x]);
            }
            let want = z0 + plane_height(&inside);
            assert!(close(p.z, want), "{shape}: {p} should be at {want}");
        }
        // The south eave stays at the wall line, a quarter meter up (on a
        // hip roof, between the hips that start from the wall corners).
        let ends = if hipped { (0.0, 10.0) } else { (-0.5, 10.5) };
        let south: Vec<DVec3> = underside(&mut s, r)
            .into_iter()
            .filter(|p| close(p.y, 0.0) && p.x > ends.0 - 1e-9 && p.x < ends.1 + 1e-9)
            .collect();
        assert!(south.iter().any(|p| close(p.x, ends.0)) && south.iter().any(|p| close(p.x, ends.1)), "{south:?}");
        assert!(south.iter().all(|p| close(p.z, z0 + SLOPE * OVERHANG)), "{south:?}");
        assert!((thickness(&mut s, r) - thick).abs() < 1e-9, "{shape}: a closed slab");
        assert!(close(derived(&mut s, r, "plan_area"), 11.0 * 6.5));

        // Back to the roof's overhang: no edge of its own.
        let got = run(&mut s, "roof_edge_set", json!({"id": r, "edge": 0, "overhang": null}));
        assert!(got["params"].get("edge_overhangs").is_none(), "{got}");
        if hipped {
            assert_eq!(vertices(&mut s, r), before, "the roof is built as before");
        }
        // Every edge set to the roof's own overhang: the same roof.
        for edge in 0..4 {
            run(&mut s, "roof_edge_set", json!({"id": r, "edge": edge, "overhang": OVERHANG}));
        }
        let (lo, hi) = bounds(&mut s, r);
        assert!(close(lo.x, -0.5) && close(hi.x, 10.5) && close(lo.y, -0.5) && close(hi.y, 6.5));
        assert!(close(lo.z, z0), "{shape}: {}", lo.z);
        run(&mut s, "elements_delete", json!({"ids": [r]}));
    }
}

/// Bad calls are refused; flat roofs take an overhang per edge but no slope;
/// `element_set` takes them too, never negative.
#[test]
fn edge_overhangs_checked_and_on_flat_roofs() {
    let mut s = Session::default();
    let rect = json!([[0, 0], [10, 0], [10, 6], [0, 6]]);
    let r = id(&run(&mut s, "roof_create", json!({"boundary": rect, "offset": 3.0, "overhang": OVERHANG})));
    let edge = |extra: Value| {
        let mut args = json!({"id": r, "edge": 1});
        args.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        args
    };
    assert_eq!(fails(&mut s, "roof_edge_set", edge(json!({}))), ErrorCode::InvalidArguments);
    assert_eq!(fails(&mut s, "roof_edge_set", edge(json!({"overhang": -0.1}))), ErrorCode::InvalidArguments);
    assert_eq!(fails(&mut s, "roof_edge_set", edge(json!({"slope": 0.0}))), ErrorCode::InvalidArguments);
    assert_eq!(
        fails(&mut s, "roof_edge_set", json!({"id": r, "edge": 4, "overhang": 1.0})),
        ErrorCode::InvalidArguments
    );
    // The east edge reaches 2 m out.
    run(&mut s, "roof_edge_set", edge(json!({"overhang": 2.0})));
    let (lo, hi) = bounds(&mut s, r);
    assert!(close(hi.x, 12.0) && close(lo.x, -0.5) && close(hi.y, 6.5), "{lo} {hi}");
    assert!(close(derived(&mut s, r, "plan_area"), 12.5 * 7.0));

    let set = |v: Value| json!({"id": r, "params": {"edge_overhangs": v}});
    assert_eq!(fails(&mut s, "element_set", set(json!([null, -1.0]))), ErrorCode::InvalidArguments);
    run(&mut s, "element_set", set(json!([0.0])));
    let (lo, hi) = bounds(&mut s, r);
    assert!(close(lo.y, 0.0) && close(hi.x, 10.5), "{lo} {hi}");
}

/// Mirrored, each edge keeps its overhang: the south eave of 0.762 m
/// becomes the north one.
#[test]
fn mirror_keeps_edge_overhangs_on_their_edges() {
    let mut s = Session::default();
    let reference = roof(&mut s, gable());
    let z0 = base(&mut s, reference);
    run(&mut s, "elements_delete", json!({"ids": [reference]}));

    let r = roof(&mut s, gable());
    run(&mut s, "roof_edge_set", json!({"id": r, "edge": 0, "overhang": 0.762}));
    run(&mut s, "roof_edge_set", json!({"id": r, "edge": 1, "overhang": 0.0}));
    run(&mut s, "elements_mirror", json!({"ids": [r], "axis": [[0, 10], [1, 10]], "copy": false}));
    let got = run(&mut s, "element_get", json!({"id": r}));
    assert_eq!(got["params"]["edge_overhangs"], json!([0.762, 0.0]));
    // The footprint now spans y 14 to 20; edge 0 runs along y = 20, edge 1 along x = 10.
    let (lo, hi) = bounds(&mut s, r);
    assert!(close(hi.y, 20.762) && close(lo.y, 13.5), "{lo} {hi}");
    assert!(close(hi.x, 10.0) && close(lo.x, -0.5), "{lo} {hi}");
    for p in underside(&mut s, r) {
        let want = z0 + plane_height(&[20.0 - p.y, p.y - 14.0]);
        assert!(close(p.z, want), "{p} should be at {want}");
    }
}

/// A crafted file: overhangs past the edges and negative ones are dropped.
#[test]
fn edge_overhangs_are_repaired_on_load() {
    let mut s = Session::default();
    let r = roof(&mut s, hip());
    let mut p = s.project().clone();
    if let Some(ElementKind::Roof(roof)) = p.elements.get_mut(&ElementId(r)).map(|e| &mut e.kind) {
        roof.edge_overhangs = vec![Some(-1.0), None, Some(0.2), None, Some(1.0), Some(2.0)];
    }
    let text = cc_core::to_ron(&p).unwrap();
    let (q, repairs) = cc_core::from_ron_with_repairs(&text).unwrap();
    assert_eq!(repairs, 2, "the entries past the edges, a negative one");
    let Some(ElementKind::Roof(roof)) = q.element(ElementId(r)).map(|e| &e.kind) else { panic!("the roof stays") };
    assert_eq!(roof.edge_overhangs, vec![None, None, Some(0.2)]);
}
