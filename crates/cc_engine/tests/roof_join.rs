//! Roof joins (issue #381): a cross wing joined to a main roof runs on into
//! it up to the valleys, and the main roof is cut away under it. Analytic
//! figures: a gable main roof 10 × 8 (ridge along x at y = 4) and a gable
//! wing 4 wide from x 3 to 7 against its south eave (ridge along y), both
//! half pitch, 3 m up, no overhang: valleys from (3, 0) and (7, 0) up to
//! (5, 2), the wing keeping 28 m² and the main roof 76 m².

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_core::{ElementId, ElementKind};
use cc_engine::{ErrorCode, Session};
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

fn num(v: &Value) -> f64 {
    v.as_f64().unwrap_or_else(|| panic!("not a number: {v}"))
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

/// A gable roof over `boundary`, 3 m up (or `offset`), half pitch, its ridge along `ridge_deg`.
fn gable(s: &mut Session, boundary: Value, ridge_deg: f64, offset: f64) -> u64 {
    let r = id(&run(s, "roof_create", json!({"boundary": boundary, "offset": offset, "overhang": 0.0})));
    run(s, "element_set", json!({"id": r, "params": {"shape": {"Gable": {"slope": 0.5, "ridge_deg": ridge_deg}}}}));
    r
}

/// The main roof and the wing against its south eave (not joined yet).
fn roofs(s: &mut Session) -> (u64, u64) {
    let main = gable(s, json!([[0, 0], [10, 0], [10, 8], [0, 8]]), 0.0, 3.0);
    let wing = gable(s, json!([[3, -6], [7, -6], [7, 0], [3, 0]]), 90.0, 3.0);
    (main, wing)
}

fn derived(s: &mut Session, id: u64) -> Value {
    run(s, "element_get", json!({"id": id}))["derived"].clone()
}

/// The roof type's thickness.
fn thickness(s: &Session, id: u64) -> f64 {
    s.project().thickness_of(s.project().element(ElementId(id)).unwrap())
}

#[test]
fn a_wing_joins_the_main_roof() {
    let mut s = Session::default();
    let (main, wing) = roofs(&mut s);
    let t = thickness(&s, wing);
    let tv = t * 1.25f64.sqrt();
    assert!(close(num(&derived(&mut s, wing)["volume"]), 24.0 * tv));
    assert!(derived(&mut s, wing).get("join").is_none());

    // One undo step joins them.
    let depth = s.doc.undo_depth();
    let out = run(&mut s, "roof_join", json!({"roof": wing, "to": main}));
    assert_eq!(s.doc.undo_depth(), depth + 1);
    assert_eq!(out["params"]["join_to"], json!(main));
    let d = derived(&mut s, wing);
    assert_eq!((d["join"]["to"].clone(), d["join"]["active"].clone()), (json!(main), json!(true)), "{d}");
    assert!(d["join"]["issue"].is_null());
    assert_eq!(d["join"]["valleys"].as_array().unwrap().len(), 2);
    assert!(close(num(&d["join"]["valley_length"]), 6.0), "two valleys of 3 m: {}", d["join"]);
    assert!(close(num(&d["plan_area"]), 28.0) && close(num(&d["volume"]), 28.0 * tv), "{d}");
    let dm = derived(&mut s, main);
    assert_eq!(dm["join"]["cut_by"], json!([wing]));
    assert!(close(num(&dm["plan_area"]), 76.0) && close(num(&dm["volume"]), 76.0 * tv), "{dm}");
    // The wing's ridge ends where the valleys meet, at (5, 2).
    let ridges = d["ridges"].as_array().unwrap();
    assert!(ridges.iter().any(|r| close(num(&r[0][0]), 5.0) && close(num(&r[1][0]), 5.0)), "{ridges:?}");

    // The check counts the tiles on what each roof keeps; the join is no issue.
    let ty = s.project().element(ElementId(wing)).unwrap().type_id.0;
    run(
        &mut s,
        "type_set",
        json!({"type": ty, "params": {"covering": "CeramicFrench", "framing": {"system": "Timber"}}}),
    );
    let check = run(&mut s, "roof_check", json!({"id": wing}));
    assert!(close(num(&check["sloped_area"]), 28.0 * 1.25f64.sqrt()), "{check}");
    assert_eq!(check["join"]["active"], json!(true));
    assert_eq!(check["ok"], json!(true), "{check}");
    // A schedule's Area and TileCount count each part once.
    let schedule = id(&run(&mut s, "view_create", json!({"kind": "schedule", "category": "roof"})));
    run(&mut s, "schedule_set", json!({"view": schedule, "fields": ["Area", "TileCount"]}));
    let text = run(&mut s, "schedule_table", json!({"view": schedule})).to_string();
    let main_tiles = num(&run(&mut s, "roof_check", json!({"id": main}))["tiles"]);
    for cell in ["28.00".to_string(), "76.00".to_string(), format!("{}", num(&check["tiles"])), format!("{main_tiles}")]
    {
        assert!(text.contains(&cell) || text.contains(&cell.replace('.', ",")), "{cell}: {text}");
    }

    // A steeper main roof: the wing keeps less of its triangle (the main roof's
    // slab is thicker vertically, so the wing meets its top lower down).
    run(&mut s, "element_set", json!({"id": main, "params": {"shape": {"Gable": {"slope": 1.0, "ridge_deg": 0.0}}}}));
    let delta = t * (2f64.sqrt() - 1.25f64.sqrt());
    let triangle = 2.0 * (1.0 - delta) * (1.0 - delta);
    let d = derived(&mut s, wing);
    assert!(close(num(&d["plan_area"]), 24.0 + triangle), "{} against {}", d["plan_area"], 24.0 + triangle);
    assert!(close(num(&d["volume"]), (24.0 + triangle) * tv));
    let dm = derived(&mut s, main);
    assert!(close(num(&dm["volume"]), (80.0 - triangle) * t * 2f64.sqrt()), "{dm}");
    run(&mut s, "undo", json!({}));

    // Deleting the main roof frees the wing in the same step; undo joins them again.
    run(&mut s, "elements_delete", json!({"ids": [main]}));
    let joined_to = |s: &Session| match &s.project().element(ElementId(wing)).unwrap().kind {
        ElementKind::Roof(r) => r.join_to,
        _ => panic!("a roof"),
    };
    assert_eq!(joined_to(&s), None);
    assert!(close(num(&derived(&mut s, wing)["volume"]), 24.0 * tv));
    run(&mut s, "undo", json!({}));
    assert_eq!(joined_to(&s), Some(ElementId(main)));
    assert!(close(num(&derived(&mut s, wing)["volume"]), 28.0 * tv));

    // Refused: to itself, to what is no roof, round a loop; unjoining a roof joined to none.
    let wall = id(&run(&mut s, "wall_create", json!({"start": [20, 0], "end": [24, 0]})));
    assert_eq!(fails(&mut s, "roof_join", json!({"roof": wing, "to": wing})), ErrorCode::InvalidArguments);
    assert_eq!(fails(&mut s, "roof_join", json!({"roof": wing, "to": wall})), ErrorCode::InvalidArguments);
    assert_eq!(fails(&mut s, "roof_join", json!({"roof": main, "to": wing})), ErrorCode::InvalidArguments);
    assert_eq!(
        fails(&mut s, "element_set", json!({"id": main, "params": {"join_to": wing}})),
        ErrorCode::InvalidArguments
    );
    assert_eq!(fails(&mut s, "roof_unjoin", json!({"roof": main})), ErrorCode::NotFound);
    // A roof far off does not meet the wing.
    let far = gable(&mut s, json!([[40, 0], [44, 0], [44, 6], [40, 6]]), 90.0, 3.0);
    assert_eq!(fails(&mut s, "roof_join", json!({"roof": far, "to": main})), ErrorCode::InvalidArguments);
    // Set by hand it stays, inactive: the check says so.
    run(&mut s, "element_set", json!({"id": far, "params": {"join_to": main}}));
    let check = run(&mut s, "roof_check", json!({"id": far}));
    assert_eq!((check["ok"].clone(), check["join"]["issue"].clone()), (json!(false), json!("apart")), "{check}");

    // Unjoined, both are built as drawn.
    run(&mut s, "roof_unjoin", json!({"roof": wing}));
    assert!(derived(&mut s, wing).get("join").is_none());
    assert!(close(num(&derived(&mut s, wing)["volume"]), 24.0 * tv));
    assert!(close(num(&derived(&mut s, main)["volume"]), 80.0 * tv));
}

#[test]
fn copies_join_and_attached_walls_follow() {
    let mut s = Session::default();
    let (main, wing) = roofs(&mut s);
    run(&mut s, "roof_join", json!({"roof": wing, "to": main}));
    // A copy of the wing on the north side joins the main roof too.
    let copy = run(&mut s, "elements_copy", json!({"ids": [wing], "delta": [0.0, 14.0]}));
    let north = copy["created"][0].as_u64().unwrap();
    let d = derived(&mut s, north);
    assert_eq!(d["join"]["to"], json!(main), "{d}");
    assert!(close(num(&d["plan_area"]), 28.0));
    assert!(close(num(&derived(&mut s, main)["plan_area"]), 72.0));
    // Copied together, the copy of the wing joins the copy of the main roof.
    let copies = run(&mut s, "elements_copy", json!({"ids": [main, wing], "delta": [30.0, 0.0]}));
    let created: Vec<u64> = copies["created"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap()).collect();
    let joined: Vec<(u64, Value)> = created.iter().map(|c| (*c, derived(&mut s, *c)["join"]["to"].clone())).collect();
    let (wing2, to) = joined.iter().find(|(_, to)| !to.is_null()).cloned().unwrap();
    assert!(created.contains(&to.as_u64().unwrap()) && to != json!(main), "{joined:?}");
    assert!(close(num(&derived(&mut s, wing2)["plan_area"]), 28.0));

    // A wall along the wing's ridge, attached to the wing, rises under the
    // wing (4 m at its ridge) and on under the main roof (5 m at y = 4).
    let wall = id(&run(&mut s, "wall_create", json!({"start": [5, -6], "end": [5, 4], "height": 2.0})));
    run(&mut s, "wall_attach", json!({"ids": [wall], "top": wing}));
    let top = s
        .geometry()
        .body_of(ElementId(wall))
        .flat_map(|x| x.faces.iter().flat_map(|f| f.verts.iter().map(|p| p.z)))
        .fold(f64::NEG_INFINITY, f64::max);
    assert!((top - 5.0).abs() < 0.02, "{top}");
}
