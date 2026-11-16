//! Pipe series: sizing within a series and the pipe types it applies.

use cc_core::{Category, ElementType, ElementTypeId, PipeSeriesId, TypeData, Uid};
use cc_engine::{EngineError, ErrorCode, Session};
use serde_json::{Value, json};

fn run(s: &mut Session, name: &str, args: Value) -> Value {
    s.execute(name, args).unwrap_or_else(|e| panic!("{name}: {e:?}"))
}

fn id(v: &Value) -> u64 {
    v.get("id").and_then(Value::as_u64).or_else(|| v.as_u64()).unwrap()
}

/// Copy built-in series `code` into the project with a type of row `size`;
/// returns the type's id.
fn series_type(s: &mut Session, code: &str, size: &str) -> u64 {
    s.doc.edit("series", |e| {
        let mut series = cc_core::pipe_catalog::series(code).unwrap().clone();
        let sid = PipeSeriesId(e.alloc_id());
        series.id = sid;
        series.uid = Uid::generate();
        let de = series.row(size).unwrap().de;
        let material = e
            .project()
            .types_of(Category::Pipe)
            .find_map(|t| match &t.data {
                TypeData::Pipe { material, .. } => Some(material.clone()),
                _ => None,
            })
            .unwrap();
        e.pipe_series_mut().insert(sid, series);
        let mut data = TypeData::pipe(de, material);
        if let TypeData::Pipe { series, size: z, .. } = &mut data {
            *series = Some(sid);
            *z = size.to_string();
        }
        let tid = ElementTypeId(e.alloc_id());
        e.add_type(ElementType {
            id: tid,
            name: format!("{code} {size}"),
            data,
            info: Default::default(),
            uid: Default::default(),
        });
        tid.0
    })
}

fn fixture_type(s: &mut Session) -> u64 {
    let types = run(s, "types_list", json!({"category": "component"}));
    types
        .as_array()
        .unwrap()
        .iter()
        .map(id)
        .find(|t| run(s, "type_get", json!({"type": t}))["params"]["def"]["mep"]["Fixture"].is_object())
        .unwrap()
}

fn sized(s: &mut Session, run_id: u64) -> Value {
    let sizing = run(s, "mep_sizing", json!({}));
    sizing["pipes"].as_array().unwrap().iter().find(|p| p["id"] == json!(run_id)).unwrap().clone()
}

#[test]
fn sizing_keeps_a_run_in_its_series() {
    let mut s = Session::default();
    // A d63 PN10 water main and a DN 40 drain of the reinforced series (not
    // in the template), with seven toilets.
    let d63 = series_type(&mut s, "EU-PVCU-1452-PN10", "d63");
    let dn40 = series_type(&mut s, "BR-PVC-ESG-R", "DN 40");
    let water = id(&run(
        &mut s,
        "pipe_create",
        json!({"start": [0, 0.1], "end": [8, 0.1], "system": "cold_water", "type": d63}),
    ));
    let drain =
        id(&run(&mut s, "pipe_create", json!({"start": [0, 0], "end": [8, 0], "system": "sanitary", "type": dn40})));
    let toilet = fixture_type(&mut s);
    let fixtures: Vec<u64> = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]
        .into_iter()
        .map(|x| id(&run(&mut s, "component_place", json!({"position": [x, 1.5], "type": toilet}))))
        .collect();
    run(&mut s, "mep_connect", json!({"elements": fixtures}));

    // The speed is measured in the row's 57 mm inside, not 0.88 × 63.
    let main = sized(&mut s, water);
    let q = main["flow_l_s"].as_f64().unwrap();
    let in57 = q / 1000.0 / (std::f64::consts::PI * 0.057 * 0.057 / 4.0);
    assert!((main["speed_m_s"].as_f64().unwrap() - in57).abs() < 1e-9, "{main}");
    // 0.43 l/s: the series' smallest row is enough.
    assert_eq!((main["size"].clone(), main["required_mm"].clone()), (json!("d32"), json!(32.0)), "{main}");
    let outlet = sized(&mut s, drain);
    assert_eq!((outlet["size"].clone(), outlet["ok"].clone()), (json!("DN 100"), json!(false)), "{outlet}");

    // Applying makes the missing rows' types of the same series in one step.
    let depth = s.doc.undo_depth();
    let applied = run(&mut s, "mep_size_apply", json!({"downsize": true}));
    assert_eq!(s.doc.undo_depth(), depth + 1);
    let created = applied["created"].as_array().unwrap().clone();
    assert_eq!(created.len(), 2, "{applied}");
    for (run_id, code, size) in [(water, "EU-PVCU-1452-PN10", "d32"), (drain, "BR-PVC-ESG-R", "DN 100")] {
        let t = run(&mut s, "element_get", json!({"id": run_id}))["type"]["id"].as_u64().unwrap();
        assert!(created.contains(&json!(t)), "{applied}");
        let p = s.project();
        let ty = p.element_type(ElementTypeId(t)).unwrap();
        let (series, row) = p.pipe_row(&ty.data).unwrap();
        assert_eq!((series.code.as_str(), row.size.as_str(), ty.diameter()), (code, size, Some(row.de)));
        assert_eq!(ty.name, format!("{} {}", series.name, size));
    }
    // Nothing left to change; a second run creates nothing.
    let again = run(&mut s, "mep_size_apply", json!({"downsize": true}));
    assert_eq!(again, json!({"changed": [], "created": []}));
}

fn fails(s: &mut Session, name: &str, args: Value) -> EngineError {
    s.execute(name, args).expect_err(name)
}

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join("cc_engine_pipes");
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

/// The project's series of `code`, as `pipe_series_list` shows it.
fn listed(s: &mut Session, code: &str) -> Option<Value> {
    let list = run(s, "pipe_series_list", json!({}));
    list["series"].as_array().unwrap().iter().find(|x| x["code"] == json!(code)).cloned()
}

#[test]
fn the_catalog_lists_built_in_series() {
    let mut s = Session::default();
    let all = run(&mut s, "pipe_catalog", json!({}));
    let series = all["series"].as_array().unwrap();
    assert_eq!(series.len(), 22);
    let pn10 = series.iter().find(|x| x["code"] == json!("EU-PVCU-1452-PN10")).unwrap();
    assert_eq!((pn10["region"].clone(), pn10["use"].clone()), (json!("europe"), json!("supply")));
    assert!(pn10["sizes"].as_array().unwrap().contains(&json!("d63")));
    // One series' rows, in millimeters.
    let one = run(&mut s, "pipe_catalog", json!({"series": "eu-pvcu-1452-pn10"}));
    let one = &one["series"][0];
    let d63 = one["pipes"].as_array().unwrap().iter().find(|r| r["size"] == json!("d63")).unwrap();
    assert_eq!((d63["de"].as_f64(), d63["di"].as_f64()), (Some(63.0), Some(57.0)), "{d63}");
    assert!(!one["fittings"].as_array().unwrap().is_empty());
    assert_eq!(fails(&mut s, "pipe_catalog", json!({"series": "XX"})).code, ErrorCode::NotFound);
}

#[test]
fn importing_series_and_their_types() {
    let mut s = Session::default();
    let pipes_before = run(&mut s, "types_list", json!({"category": "pipe"})).as_array().unwrap().len();
    let out = run(&mut s, "pipe_series_import", json!({"code": "EU-PVCU-1452-PN10", "sizes": ["all"]}));
    let sid = out["series"].as_u64().unwrap();
    let rows = cc_core::pipe_catalog::series("EU-PVCU-1452-PN10").unwrap().pipes.len();
    assert_eq!(out["created"].as_array().unwrap().len(), rows, "{out}");
    let pipes = run(&mut s, "types_list", json!({"category": "pipe"})).as_array().unwrap().len();
    assert_eq!(pipes, pipes_before + rows);
    // A type of the series: its diameter is the row's, its inside the row's di.
    let types = run(&mut s, "pipe_series_get", json!({"series": sid}))["types"].as_array().unwrap().clone();
    assert_eq!(types.len(), rows);
    let d63 = types
        .iter()
        .map(|t| run(&mut s, "type_get", json!({"type": t})))
        .find(|t| t["params"]["size"] == json!("d63"))
        .unwrap();
    assert_eq!(d63["params"]["series"], json!("EU-PVCU-1452-PN10"));
    assert!((d63["params"]["diameter"].as_f64().unwrap() - 0.063).abs() < 1e-12);
    assert!((d63["derived"]["inside_diameter"].as_f64().unwrap() - 0.057).abs() < 1e-12, "{d63}");
    assert!((d63["derived"]["wall"].as_f64().unwrap() - 0.003).abs() < 1e-12);
    // Again with skip: nothing to do, no undo step.
    let depth = s.doc.undo_depth();
    let again = run(&mut s, "pipe_series_import", json!({"code": "EU-PVCU-1452-PN10", "sizes": ["all"]}));
    assert_eq!((again["series"].as_u64(), again["created"].clone()), (Some(sid), json!([])), "{again}");
    assert_eq!(s.doc.undo_depth(), depth);
    // Replace keeps the series' id and uid, and its types.
    let uid = s.project().pipe_series_by_id(PipeSeriesId(sid)).unwrap().uid;
    let before = listed(&mut s, "EU-PVCU-1452-PN10").unwrap()["types"].clone();
    let replaced = run(&mut s, "pipe_series_import", json!({"code": "EU-PVCU-1452-PN10", "on_clash": "replace"}));
    assert_eq!(replaced["series"].as_u64(), Some(sid));
    assert_eq!(s.project().pipe_series_by_id(PipeSeriesId(sid)).unwrap().uid, uid);
    assert_eq!(listed(&mut s, "EU-PVCU-1452-PN10").unwrap()["types"], before);
    // An unknown size and a bad file are refused without an undo step.
    let depth = s.doc.undo_depth();
    let err = fails(&mut s, "pipe_series_import", json!({"code": "US-PEX", "sizes": ["d63"]}));
    assert_eq!(err.code, ErrorCode::NotFound);
    let bad = json!({"format": 1, "code": "X", "name": "X", "material": "PVC", "use": "supply",
        "joint": "solvent", "pipes": [{"size": "a", "de": 20, "e": 15, "di": -10}]});
    let err = fails(&mut s, "pipe_series_import", json!({"text": bad.to_string()}));
    assert_eq!(err.code, ErrorCode::InvalidArguments, "{err:?}");
    assert_eq!(fails(&mut s, "pipe_series_import", json!({})).code, ErrorCode::InvalidArguments);
    assert_eq!(s.doc.undo_depth(), depth);
    assert!(listed(&mut s, "US-PEX").is_none() && listed(&mut s, "X").is_none());
}

#[test]
fn series_files_round_trip_and_rename() {
    let mut s = Session::default();
    run(&mut s, "pipe_series_import", json!({"code": "US-CU-L"}));
    // Export to text and to a file, then import the text back under a new code.
    let text = run(&mut s, "pipe_series_export", json!({"series": "US-CU-L"}))["text"].as_str().unwrap().to_string();
    let path = scratch("us-cu-l.json");
    run(&mut s, "pipe_series_export", json!({"series": "us-cu-l", "path": path}));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), text);
    let half = cc_core::pipe_catalog::series("US-CU-L").unwrap().pipes[0].size.clone();
    let out = run(&mut s, "pipe_series_import", json!({"text": text, "on_clash": "rename", "sizes": [half]}));
    let copy = listed(&mut s, "US-CU-L-2").unwrap();
    assert_eq!(copy["id"], out["series"]);
    assert_eq!(out["created"].as_array().unwrap().len(), 1, "{out}");
    // The copy holds the same rows.
    let a = run(&mut s, "pipe_series_get", json!({"series": "US-CU-L"}));
    let b = run(&mut s, "pipe_series_get", json!({"series": "US-CU-L-2"}));
    assert_eq!((a["pipes"].clone(), a["fittings"].clone()), (b["pipes"].clone(), b["fittings"].clone()));
    // From the file with skip: the code is taken.
    let skipped = run(&mut s, "pipe_series_import", json!({"path": path}));
    assert_eq!(skipped["skipped"], json!(true));
}

#[test]
fn deleting_a_series_in_use() {
    let mut s = Session::default();
    let made = run(&mut s, "pipe_type_create", json!({"series": "EU-PE100-SDR11", "size": "d32"}));
    let t = made["id"].as_u64().unwrap();
    let err = fails(&mut s, "pipe_series_delete", json!({"series": "EU-PE100-SDR11"}));
    assert_eq!(err.code, ErrorCode::Rejected);
    assert!(err.message.contains("in_use"), "{err:?}");
    let out = run(&mut s, "pipe_series_delete", json!({"series": "EU-PE100-SDR11", "detach": true}));
    assert_eq!(out["detached"], json!([t]));
    assert!(listed(&mut s, "EU-PE100-SDR11").is_none());
    // The type is free and keeps its diameters.
    let ty = run(&mut s, "type_get", json!({"type": t}));
    assert!(ty["params"].get("series").is_none(), "{ty}");
    assert!((ty["params"]["diameter"].as_f64().unwrap() - 0.032).abs() < 1e-12);
    let row = cc_core::pipe_catalog::series("EU-PE100-SDR11").unwrap().row("d32").unwrap().di;
    assert!((ty["derived"]["inside_diameter"].as_f64().unwrap() - row).abs() < 1e-12, "{ty}");
    // Undo brings the series and the link back.
    s.doc.undo();
    assert_eq!(run(&mut s, "type_get", json!({"type": t}))["params"]["series"], json!("EU-PE100-SDR11"));
}

#[test]
fn pipe_types_from_the_catalog() {
    let mut s = Session::default();
    let depth = s.doc.undo_depth();
    let a = run(&mut s, "pipe_type_create", json!({"series": "BR-PPR-PN20", "size": "25 mm"}));
    assert_eq!(s.doc.undo_depth(), depth + 1);
    let b = run(&mut s, "pipe_type_create", json!({"series": "br-ppr-pn20", "size": "32 mm", "name": "PPR 32"}));
    // The series is imported once.
    assert_eq!(a["series"], b["series"]);
    assert_eq!(s.project().pipe_series.values().filter(|x| x.code == "BR-PPR-PN20").count(), 1);
    assert_eq!(run(&mut s, "type_get", json!({"type": b["id"]}))["name"], json!("PPR 32"));
    let err = fails(&mut s, "pipe_type_create", json!({"series": "BR-PPR-PN20", "size": "33 mm"}));
    assert!(err.message.contains("unknown_size"), "{err:?}");
    let err = fails(&mut s, "pipe_type_create", json!({"series": "NOPE", "size": "1"}));
    assert_eq!(err.code, ErrorCode::NotFound);
}

#[test]
fn type_set_moves_a_pipe_type_between_rows() {
    let mut s = Session::default();
    let made = run(&mut s, "pipe_type_create", json!({"series": "BR-PVC-SOLD", "size": "25 mm", "name": "Supply"}));
    let t = made["id"].as_u64().unwrap();
    let pipe = id(&run(&mut s, "pipe_create", json!({"start": [0, 0], "end": [4, 0], "type": t})));
    // Another size of the series resizes the type and its pipes in one step.
    run(&mut s, "type_set", json!({"type": t, "params": {"size": "50 mm"}}));
    let ty = run(&mut s, "type_get", json!({"type": t}));
    assert!((ty["params"]["diameter"].as_f64().unwrap() - 0.05).abs() < 1e-12, "{ty}");
    let el = run(&mut s, "element_get", json!({"id": pipe}));
    let derived = &el["derived"];
    assert_eq!((derived["size"].clone(), derived["series_code"].clone()), (json!("50 mm"), json!("BR-PVC-SOLD")));
    assert!((derived["inside_diameter"].as_f64().unwrap() - 0.044).abs() < 1e-12, "{el}");
    // Another series of the project by code, with one of its sizes.
    run(&mut s, "pipe_series_import", json!({"code": "EU-PVCU-1452-PN10"}));
    run(&mut s, "type_set", json!({"type": t, "params": {"series": "EU-PVCU-1452-PN10", "size": "d63"}}));
    let ty = run(&mut s, "type_get", json!({"type": t}));
    assert_eq!(ty["params"]["series"], json!("EU-PVCU-1452-PN10"));
    assert!((ty["params"]["diameter"].as_f64().unwrap() - 0.063).abs() < 1e-12, "{ty}");
    // A size the series lacks is refused, without an undo step.
    let depth = s.doc.undo_depth();
    let err = fails(&mut s, "type_set", json!({"type": t, "params": {"size": "60 mm"}}));
    assert!(err.message.contains("unknown_size"), "{err:?}");
    assert_eq!(s.doc.undo_depth(), depth);
    // Null makes it free, keeping its diameter.
    run(&mut s, "type_set", json!({"type": t, "params": {"series": null}}));
    let ty = run(&mut s, "type_get", json!({"type": t}));
    assert!(ty["params"].get("series").is_none() && ty["params"].get("size").is_none(), "{ty}");
    assert!((ty["params"]["diameter"].as_f64().unwrap() - 0.063).abs() < 1e-12);
}
