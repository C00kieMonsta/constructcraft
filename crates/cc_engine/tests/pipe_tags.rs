//! Tags of pipes, conduits and cable trays in services plans, and the
//! labels of fittings: moved, hidden and reset like other tags.

use cc_engine::Session;
use serde_json::{Value, json};

fn run(s: &mut Session, name: &str, args: Value) -> Value {
    s.execute(name, args).unwrap_or_else(|e| panic!("{name}: {e:?}"))
}

fn id(v: &Value) -> u64 {
    v.get("id").and_then(Value::as_u64).or_else(|| v.as_u64()).unwrap()
}

fn pipe(s: &mut Session, t: u64, a: [f64; 2], b: [f64; 2]) -> u64 {
    id(&run(s, "pipe_create", json!({"start": a, "end": b, "system": "cold_water", "type": t, "offset": 0.5})))
}

#[test]
fn pipe_tags_move_with_a_leader_and_take_tag_types() {
    let mut s = Session::default();
    let d25 = id(&run(&mut s, "pipe_type_create", json!({"series": "BR-PVC-SOLD", "size": "25 mm"})));
    let run_a = pipe(&mut s, d25, [0.0, 0.0], [4.0, 0.0]);
    let plan = id(&run(&mut s, "view_create", json!({"kind": "plumbing_plan"})));
    let got = run(&mut s, "tag_get", json!({"view": plan, "id": run_a}));
    assert_eq!(got["target"], json!("pipe"));
    assert_eq!(got["texts"], json!(["AF Ø25"]), "{got}");
    assert_eq!(got["drawn"], json!(true));
    assert_eq!(got["type"], Value::Null, "the template has no pipe tag type: the default style");
    let center = got["center"].clone();

    // Moved off the run: a leader runs back to it; reset puts it back.
    run(&mut s, "tag_set", json!({"view": plan, "ids": [run_a], "delta": [0.0, 1.5]}));
    let moved = run(&mut s, "tag_get", json!({"view": plan, "id": run_a}));
    assert_eq!(moved["placement"]["offset"], json!([0.0, 1.5]));
    let dy = moved["center"][1].as_f64().unwrap() - center[1].as_f64().unwrap();
    assert!((dy - 1.5).abs() < 1e-9, "{moved}");
    run(&mut s, "tag_set", json!({"view": plan, "ids": [run_a], "hidden": true}));
    assert_eq!(run(&mut s, "tag_get", json!({"view": plan, "id": run_a}))["drawn"], json!(false));
    run(&mut s, "tag_set", json!({"view": plan, "ids": [run_a], "reset": true}));
    assert_eq!(run(&mut s, "tag_get", json!({"view": plan, "id": run_a}))["center"], center);

    // A tag type of its own writes what it is told.
    let style = json!({"lines": [{"text": "{DiameterMark} - {Systems}", "size": 2.5}]});
    let ty = id(&run(&mut s, "tag_type_create", json!({"target": "pipe", "name": "Short", "style": style})));
    run(&mut s, "view_set", json!({"view": plan, "tag_types": {"pipe": ty}}));
    assert_eq!(run(&mut s, "tag_get", json!({"view": plan, "id": run_a}))["texts"], json!(["Ø25 - AF"]));
}

#[test]
fn fitting_labels_name_their_piece_and_move_by_their_runs() {
    let mut s = Session::default();
    let d25 = id(&run(&mut s, "pipe_type_create", json!({"series": "BR-PVC-SOLD", "size": "25 mm"})));
    let a = pipe(&mut s, d25, [0.0, 0.0], [2.0, 0.0]);
    let b = pipe(&mut s, d25, [2.0, 0.0], [2.0, 2.0]);
    // A 60° turn: the series sells no such elbow.
    pipe(&mut s, d25, [0.0, 10.0], [2.0, 10.0]);
    let r = 60f64.to_radians();
    pipe(&mut s, d25, [2.0, 10.0], [2.0 + r.cos(), 10.0 + r.sin()]);
    let plan = id(&run(&mut s, "view_create", json!({"kind": "plumbing_plan"})));
    let key = format!("{}-{}", a.min(b), a.max(b));

    let list = run(&mut s, "fitting_tags", json!({"view": plan}));
    assert_eq!(list["fitting_tags"], json!(false));
    let fittings = list["fittings"].as_array().unwrap().clone();
    let elbow = fittings.iter().find(|f| f["key"] == json!(key)).unwrap_or_else(|| panic!("{list}"));
    let name = format!("{} 90° PVC soldável 25 mm", cc_engine::i18n::tr("fitting-elbow"));
    assert_eq!(elbow["texts"], json!([name]));
    assert_eq!((elbow["missing"].clone(), elbow["drawn"].clone()), (json!(false), json!(false)));
    let missing = fittings.iter().find(|f| f["missing"] == json!(true)).unwrap_or_else(|| panic!("{list}"));
    assert_eq!(missing["texts"][1], json!(cc_engine::i18n::tr("fitting-missing-angle")));

    // Only services plans label fittings.
    let architecture = id(&run(&mut s, "view_create", json!({"kind": "floor_plan"})));
    assert!(s.execute("view_set", json!({"view": architecture, "fitting_tags": true})).is_err());
    run(&mut s, "view_set", json!({"view": plan, "fitting_tags": true}));
    assert_eq!(run(&mut s, "view_get", json!({"view": plan}))["fitting_tags"], json!(true));
    let shown = |s: &mut Session| -> Value {
        let list = run(s, "fitting_tags", json!({"view": plan}));
        list["fittings"].as_array().unwrap().iter().find(|f| f["key"] == json!(key)).unwrap().clone()
    };
    let before = shown(&mut s);
    assert_eq!(before["drawn"], json!(true));

    // Moved by its key (in any order), in one undo step; hidden; reset.
    let reversed = format!("{}-{}", a.max(b), a.min(b));
    run(&mut s, "fitting_tag_set", json!({"view": plan, "keys": [reversed], "delta": [0.5, 0.5]}));
    let after = shown(&mut s);
    assert_eq!(after["placement"]["offset"], json!([0.5, 0.5]));
    let dx = after["center"][0].as_f64().unwrap() - before["center"][0].as_f64().unwrap();
    assert!((dx - 0.5).abs() < 1e-9, "{after}");
    run(&mut s, "undo", json!({}));
    assert_eq!(shown(&mut s)["placement"]["offset"], json!([0.0, 0.0]));
    run(&mut s, "fitting_tag_set", json!({"view": plan, "keys": [key], "hidden": true}));
    assert_eq!(shown(&mut s)["drawn"], json!(false));
    run(&mut s, "fitting_tag_set", json!({"view": plan, "keys": [key], "reset": true}));
    assert_eq!(shown(&mut s)["drawn"], json!(true));
    for bad in ["x", "1-999999"] {
        assert!(s.execute("fitting_tag_set", json!({"view": plan, "keys": [bad], "hidden": true})).is_err(), "{bad}");
    }

    // A deleted run takes its fittings' adjustments along.
    run(&mut s, "fitting_tag_set", json!({"view": plan, "keys": [key], "offset": [1.0, 0.0]}));
    run(&mut s, "elements_delete", json!({"ids": [b]}));
    let v = s.project().view(cc_core::ViewId(plan)).unwrap();
    assert!(v.fitting_placements.is_empty(), "{:?}", v.fitting_placements);
}
