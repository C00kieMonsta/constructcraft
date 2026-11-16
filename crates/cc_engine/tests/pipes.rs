//! Pipe series: sizing within a series and the pipe types it applies.

use cc_core::{Category, ElementType, ElementTypeId, PipeSeriesId, TypeData, Uid};
use cc_engine::Session;
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
