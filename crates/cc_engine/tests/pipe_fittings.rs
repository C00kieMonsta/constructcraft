//! Fittings matched to the catalog of their pipes' series: the takeoff,
//! the cut lengths and the pipe schedule's series fields.

use cc_engine::Session;
use serde_json::{Value, json};

fn run(s: &mut Session, name: &str, args: Value) -> Value {
    s.execute(name, args).unwrap_or_else(|e| panic!("{name}: {e:?}"))
}

fn id(v: &Value) -> u64 {
    v.get("id").and_then(Value::as_u64).or_else(|| v.as_u64()).unwrap()
}

/// A type of built-in series `code` in size `size`, made through the catalog.
fn series_type(s: &mut Session, code: &str, size: &str) -> u64 {
    id(&run(s, "pipe_type_create", json!({"series": code, "size": size})))
}

fn pipe(s: &mut Session, t: u64, a: [f64; 2], b: [f64; 2]) -> u64 {
    id(&run(s, "pipe_create", json!({"start": a, "end": b, "system": "cold_water", "type": t, "offset": 0.5})))
}

fn fittings(s: &mut Session) -> Vec<Value> {
    run(s, "mep_fittings", json!({}))["fittings"].as_array().unwrap().clone()
}

#[test]
fn the_takeoff_names_catalog_fittings_and_flags_missing_ones() {
    let mut s = Session::default();
    let d25 = series_type(&mut s, "BR-PVC-SOLD", "25 mm");
    let d40 = series_type(&mut s, "BR-PVC-SOLD", "40 mm");
    let d20 = series_type(&mut s, "BR-PVC-SOLD", "20 mm");
    // An L of 25 mm, and a 40 mm run with a 20 mm branch (no 40 × 20 tee is sold).
    let leg = pipe(&mut s, d25, [0.0, 0.0], [2.0, 0.0]);
    pipe(&mut s, d25, [2.0, 0.0], [2.0, 2.0]);
    pipe(&mut s, d40, [0.0, 5.0], [4.0, 5.0]);
    pipe(&mut s, d20, [2.0, 5.0], [2.0, 7.0]);
    // A 60° turn of 25 mm: no elbow of that angle.
    pipe(&mut s, d25, [0.0, 10.0], [2.0, 10.0]);
    let a = 60f64.to_radians();
    pipe(&mut s, d25, [2.0, 10.0], [2.0 + a.cos(), 10.0 + a.sin()]);
    let items = fittings(&mut s);
    let elbow = items.iter().find(|f| f["status"] == json!("matched")).unwrap();
    assert_eq!(elbow["name"], json!(format!("{} 90° PVC soldável 25 mm", cc_engine::i18n::tr("fitting-elbow"))));
    assert_eq!(elbow["series"], json!("BR-PVC-SOLD"));
    assert_eq!(elbow["size"], json!("25 mm"));
    let tee = items.iter().find(|f| f["status"] == json!("composite")).unwrap();
    assert_eq!(tee["kind"], json!("tee"));
    let pieces = tee["pieces"].as_array().unwrap();
    assert_eq!(pieces[0]["kind"], json!("tee"));
    assert!(pieces[1..].iter().all(|p| p["kind"] == json!("reducer")), "{tee}");
    let missing = items.iter().find(|f| f["missing"] == json!(true)).unwrap();
    assert_eq!(missing["reason_code"], json!("angle"));
    assert_eq!(missing["reason"], json!(cc_engine::i18n::tr("fitting-missing-angle")));
    assert_eq!(run(&mut s, "mep_fittings", json!({}))["missing"], json!(1));
    // The leg loses the elbow's laying length (Z 14.4 mm) in its cut length.
    let got = run(&mut s, "element_get", json!({"id": leg}));
    let cut = got["derived"]["cut_length"].as_f64().unwrap();
    assert!((cut - (2.0 - 0.0144)).abs() < 1e-9, "{cut}");
}

#[test]
fn the_pipe_schedule_shows_series_size_inside_diameter_and_cut_length() {
    let mut s = Session::default();
    let d25 = series_type(&mut s, "BR-PVC-SOLD", "25 mm");
    pipe(&mut s, d25, [0.0, 0.0], [2.0, 0.0]);
    pipe(&mut s, d25, [2.0, 0.0], [2.0, 2.0]);
    let view = id(&run(&mut s, "view_create", json!({"kind": "schedule", "category": "pipe"})));
    let fields = json!(["Series", "Size", "InsideDiameter", "CutLength"]);
    run(&mut s, "view_set", json!({"view": view, "params": {"def": {"fields": fields}}}));
    let t = run(&mut s, "schedule_table", json!({"view": view}));
    let rows = t["rows"].as_array().unwrap();
    let row = rows.iter().find(|r| r["cells"][1] == json!("25 mm")).unwrap_or_else(|| panic!("{t}"));
    assert_eq!(row["cells"][0], json!("Tubo PVC soldável"));
    assert!(row["cells"][2].as_str().unwrap().contains("21"), "{row}");
    assert!(!row["cells"][3].as_str().unwrap().is_empty(), "{row}");
}
