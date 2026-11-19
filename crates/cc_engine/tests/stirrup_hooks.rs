//! Hooks on closed stirrups (#344, #351): new stirrup sets are closed by two
//! hooks at one corner, counted in their length and weight, read as hooks by
//! bar marks and written and drawn as hooks by the bending schedule; older
//! sets (no hooks) keep their lengths.

use cc_core::{ElementId, RebarLayout, StirrupHooks};
use cc_engine::{ErrorCode, Session};
use cc_geom::rebar::Bar;
use serde_json::{Value, json};
use std::f64::consts::PI;

fn run(s: &mut Session, name: &str, args: Value) -> Value {
    s.execute(name, args).unwrap_or_else(|e| panic!("{name}: {e:?}"))
}

fn fails(s: &mut Session, name: &str, args: Value) -> ErrorCode {
    s.execute(name, args).map(|v| panic!("{name} should fail, gave {v}")).unwrap_err().code
}

fn id(v: &Value) -> u64 {
    v["id"].as_u64().unwrap_or_else(|| panic!("no id in {v}"))
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// A free 20 x 50 cm concrete beam, 3 m long.
fn beam(s: &mut Session) -> u64 {
    run(
        s,
        "profile_create",
        json!({"name": "V20x50", "material": "concrete", "shape": "rect", "params": {"b": 0.2, "h": 0.5}}),
    );
    let ty = run(s, "profile_type_create", json!({"profile": "V20x50", "category": "beam"}))["id"].clone();
    id(&run(s, "beam_create", json!({"start": [0, 0], "end": [3, 0], "top_offset": 3.0, "type": ty})))
}

/// The length of a bar's hooks: its length less its loop's.
fn hooks_of(bar: &Bar) -> f64 {
    bar.length() - Bar { hooks: None, ..bar.clone() }.length()
}

/// The issue's stirrup: 14 x 44 cm (center line) of 5 mm bar, its length with its two hooks.
#[test]
fn a_stirrup_counts_its_two_hooks() {
    let mut s = Session::default();
    let beam = beam(&mut s);
    // A cover of 2.75 cm leaves a center line 0.20 - 2 x 0.0275 - 0.005 = 0.14 wide
    // and 0.50 - 2 x 0.0275 - 0.005 = 0.44 high.
    let set = id(&run(&mut s, "rebar_create", json!({"host": beam, "layout": "stirrups", "cover": 0.0275})));
    let got = run(&mut s, "element_get", json!({"id": set}));
    assert_eq!(
        got["params"]["layout"],
        json!({"Stirrups": {"spacing": 0.15, "hooks": {"angle": 135.0, "tail": 10.0}}})
    );
    let geom = s.geometry().clone();
    let g = &geom.rebar[&ElementId(set)];
    assert!(close(g.diameter, 0.005), "{}", g.diameter);
    // The loop's sides measured to its sharp corners, 2 x (0.14 + 0.44) = 1.16 m, and two
    // hooks, each bent by 135° (3/4 pi) around a pin 3 diameters across (its center line
    // 2 diameters out: an arc of 3/4 pi x 0.010 m) with a straight end of 10 diameters
    // (0.05 m, the least a tie's hook has too): 1.16 + 2 x (0.0235619 + 0.05) = 1.3071239 m.
    let expected = 2.0 * (0.14 + 0.44) + 2.0 * (0.75 * PI * 2.0 * 0.005 + 0.05);
    assert!(g.bars.iter().all(|b| close(b.length(), expected)), "{} vs {expected}", g.bars[0].length());
    assert!(close(got["derived"]["bar_length"].as_f64().unwrap(), expected), "{got}");
    // The weight counts the hooks too.
    let weight = got["derived"]["weight"].as_f64().unwrap();
    assert!((weight - g.count() as f64 * expected * g.mass_per_meter()).abs() < 1e-9, "{weight}");

    // Bar marks and the bending schedule write the hooks.
    let marks = run(&mut s, "bar_marks_list", json!({}))["marks"].as_array().unwrap().clone();
    let codes: Vec<&str> = marks.iter().map(|m| m["code"].as_str().unwrap()).collect();
    assert_eq!(codes, ["stirrup 14x44 hook 135° 5 hook 135° 5"]);
    let legs = marks[0]["shape"]["legs"].as_array().unwrap();
    assert_eq!(legs.iter().filter(|l| l["hook"] == json!(true)).count(), 2, "{legs:?}");
    let schedule = id(&run(&mut s, "bending_schedule_create", json!({})));
    let table = run(&mut s, "schedule_table", json!({"view": schedule}));
    let text = table.to_string();
    assert!(text.contains("stirrup 14x44 hook 135° 5 hook 135° 5"), "{text}");
    assert!(text.contains(&cc_views::format::bar_length(expected)), "{text}");

    // 90° hooks with 12-diameter ends (6 cm) are another stirrup, another mark.
    let right = json!({"host": beam, "layout": "stirrups", "cover": 0.0275, "hook": 90, "tail": 12});
    let right = id(&run(&mut s, "rebar_create", right));
    let geom = s.geometry().clone();
    let r = &geom.rebar[&ElementId(right)].bars[0];
    assert!(close(r.length(), 1.16 + 2.0 * (0.5 * PI * 0.01 + 0.06)), "{}", r.length());
    let marks = run(&mut s, "bar_marks_list", json!({}))["marks"].as_array().unwrap().clone();
    let mut codes: Vec<&str> = marks.iter().map(|m| m["code"].as_str().unwrap()).collect();
    codes.sort();
    assert_eq!(codes, ["stirrup 14x44 hook 135° 5 hook 135° 5", "stirrup 14x44 hook 90° 6 hook 90° 6"]);
    // Without hooks: the closed outline alone, as older sets.
    let bare =
        id(&run(&mut s, "rebar_create", json!({"host": beam, "layout": "stirrups", "cover": 0.0275, "hook": 0})));
    let geom = s.geometry().clone();
    assert!(geom.rebar[&ElementId(bare)].bars.iter().all(|b| close(b.length(), 1.16) && b.hooks.is_none()));
    assert_eq!(run(&mut s, "bar_marks_list", json!({}))["marks"].as_array().unwrap().len(), 3);

    // Hooks that are no hooks, or on bars that take none, are refused.
    for bad in [
        json!({"host": beam, "layout": "stirrups", "hook": 45}),
        json!({"host": beam, "layout": "stirrups", "tail": -1}),
        json!({"host": beam, "layout": "stirrups", "tail": 500}),
        json!({"host": beam, "layout": "top", "hook": 135}),
    ] {
        assert_eq!(fails(&mut s, "rebar_create", bad), ErrorCode::InvalidArguments);
    }
    let bad = json!({"id": set, "params": {"layout": {"Stirrups": {"hooks": {"angle": 200.0, "tail": 10.0}}}}});
    assert_eq!(fails(&mut s, "element_set", bad), ErrorCode::Rejected);
    // An edit changes them, and takes them off.
    let edit = json!({"id": set, "params": {"layout": {"Stirrups": {"hooks": {"angle": 90.0, "tail": 10.0}}}}});
    run(&mut s, "element_set", edit);
    let geom = s.geometry().clone();
    assert!(close(geom.rebar[&ElementId(set)].bars[0].length(), 1.16 + 2.0 * (0.5 * PI * 0.01 + 0.05)));
    run(&mut s, "element_set", json!({"id": set, "params": {"layout": {"Stirrups": {"hooks": null}}}}));
    let geom = s.geometry().clone();
    assert!(close(geom.rebar[&ElementId(set)].bars[0].length(), 1.16));
}

/// Sets written before stirrups had hooks read without them and keep their lengths.
#[test]
fn older_stirrups_keep_their_lengths() {
    let old: RebarLayout = serde_json::from_value(json!({"Stirrups": {"spacing": 0.15}})).unwrap();
    assert_eq!(old, RebarLayout::Stirrups { spacing: 0.15, hooks: None });

    // A file's set without hooks is written as before (no field) and measured around its outline alone.
    let mut s = Session::default();
    let beam = beam(&mut s);
    let set = id(&run(&mut s, "rebar_create", json!({"host": beam, "layout": "stirrups", "cover": 0.0275})));
    let new = cc_core::to_ron(s.project()).unwrap();
    assert!(new.contains("hooks"), "new sets keep their hooks");
    let (loaded, repairs) = cc_core::from_ron_with_repairs(&new).unwrap();
    let kept = loaded.elements.get(&ElementId(set)).map(|e| e.kind.clone());
    let hooked = RebarLayout::Stirrups { spacing: 0.15, hooks: Some(StirrupHooks::default()) };
    assert!(matches!(kept, Some(cc_core::ElementKind::Rebar(r)) if r.layout == hooked) && repairs == 0);
    let mut file = s.project().clone();
    if let Some(cc_core::ElementKind::Rebar(r)) = file.elements.get_mut(&ElementId(set)).map(|e| &mut e.kind) {
        r.layout = old;
    }
    let text = cc_core::to_ron(&file).unwrap();
    assert!(!text.contains("hooks"), "nothing written for no hooks");
    let (loaded, _) = cc_core::from_ron_with_repairs(&text).unwrap();
    let mut s = Session::new(cc_core::Document::new(loaded));
    let geom = s.geometry().clone();
    assert!(geom.rebar[&ElementId(set)].bars.iter().all(|b| close(b.length(), 1.16)));
}

/// Columns (along the outline, overlapping rectangles, round), the typical
/// reinforcement and the beam of a strip footing get hooked stirrups too.
#[test]
fn every_kind_of_stirrup_takes_hooks() {
    let mut s = Session::default();
    // A 20 x 20 cm column: a loop 0.20 - 2 x 0.025 - 0.005 = 0.145 square, and two hooks.
    let column = id(&run(&mut s, "column_create", json!({"position": [9, 9], "height": 3.0})));
    let out = run(&mut s, "rebar_typical", json!({"elements": [column]}));
    let geom = s.geometry().clone();
    let ties = out["created"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| ElementId(v.as_u64().unwrap()))
        .find(|id| geom.rebar[id].bars[0].closed)
        .unwrap();
    let hooks = 2.0 * (0.75 * PI * 0.01 + 0.05);
    assert!(
        geom.rebar[&ties].bars.iter().all(|b| close(b.length(), 4.0 * 0.145 + hooks)),
        "{:?}",
        geom.rebar[&ties].bars[0]
    );
    // The typical reinforcement takes other hooks when asked.
    let other = id(&run(&mut s, "column_create", json!({"position": [12, 9], "height": 3.0})));
    run(&mut s, "rebar_typical", json!({"elements": [other], "hook": 90, "tail": 6}));
    let p = s.project();
    let layout =
        p.rebar_of(ElementId(other)).map(|(_, r)| r.layout.clone()).find(|l| matches!(l, RebarLayout::Stirrups { .. }));
    assert_eq!(
        layout,
        Some(RebarLayout::Stirrups { spacing: 0.15, hooks: Some(StirrupHooks { angle: 90.0, tail: 6.0 }) })
    );

    // A round column: its tie, a ring of chords, gets the two hooks.
    let round_profile = json!({"name": "C30", "material": "concrete", "shape": "round", "params": {"d": 0.3}});
    run(&mut s, "profile_create", round_profile);
    let round = run(&mut s, "profile_type_create", json!({"profile": "C30", "category": "column"}))["id"].clone();
    let round = id(&run(&mut s, "column_create", json!({"position": [0, 9], "height": 3.0, "type": round})));
    let ring = id(&run(&mut s, "rebar_create", json!({"host": round, "layout": "stirrups"})));
    let geom = s.geometry().clone();
    let bar = &geom.rebar[&ElementId(ring)].bars[0];
    assert!(close(hooks_of(bar), hooks), "{}", hooks_of(bar));
    let shape = cc_geom::bar_shape::summary(bar);
    assert!(shape.legs.first().is_some_and(|l| l.hook) && shape.hooked_loop().is_some(), "{shape:?}");

    // Overlapping rectangles in an L column: each one closed by hooks.
    let pts = [[0.0, 0.0], [0.6, 0.0], [0.6, 0.2], [0.2, 0.2], [0.2, 0.6], [0.0, 0.6]];
    let curves: Vec<Value> =
        (0..6).map(|i| json!({"kind": "line", "start": pts[i], "end": pts[(i + 1) % 6]})).collect();
    run(&mut s, "profile_create", json!({"name": "L60", "material": "concrete", "curves": curves}));
    let l_type = run(&mut s, "profile_type_create", json!({"profile": "L60", "category": "column"}))["id"].clone();
    let l = id(&run(&mut s, "column_create", json!({"position": [0, 0], "height": 3.0, "type": l_type})));
    let along = id(&run(&mut s, "rebar_create", json!({"host": l, "layout": "stirrups", "cover": 0.03})));
    let args = json!({"host": l, "layout": "stirrups", "cover": 0.03, "overlapping": true});
    let two = id(&run(&mut s, "rebar_create", args));
    let geom = s.geometry().clone();
    // Along the outline, the hooks close the loop at a convex corner: they bend the way it turns there.
    let outline = &geom.rebar[&ElementId(along)].bars[0];
    assert!(close(hooks_of(outline), hooks), "{}", hooks_of(outline));
    let p: Vec<glam::DVec2> = outline.points.iter().map(|p| p.truncate()).collect();
    let n = p.len();
    let turn = (p[0] - p[n - 1]).perp_dot(p[1] - p[0]);
    let area: f64 = (0..n).map(|i| p[i].perp_dot(p[(i + 1) % n])).sum();
    assert!(turn * area > 0.0, "the hooks' corner turns the way the loop does");
    let loops = &geom.rebar[&ElementId(two)].bars;
    assert!(loops.len() > 2 && loops.iter().all(|b| close(hooks_of(b), hooks)));

    // The beam of a strip footing.
    let strip = id(&run(&mut s, "strip_footing_create", json!({"start": [20, 0], "end": [24, 0]})));
    let args = json!({"strips": [strip], "transverse_spacing": 0, "longitudinal_spacing": 0, "beam": {"top": 0, "stirrups": 0.25}});
    let created = run(&mut s, "strip_rebar_create", args)["created"].as_array().unwrap().clone();
    let set = ElementId(created[0].as_u64().unwrap());
    let geom = s.geometry().clone();
    assert!(geom.rebar[&set].bars.iter().all(|b| b.closed && close(hooks_of(b), hooks)));
    let args = json!({"strips": [strip], "transverse_spacing": 0, "longitudinal_spacing": 0, "beam": {"top": 0, "stirrups": 0.25, "hook": 0}});
    let created = run(&mut s, "strip_rebar_create", args)["created"].as_array().unwrap().clone();
    let geom = s.geometry().clone();
    assert!(geom.rebar[&ElementId(created[0].as_u64().unwrap())].bars.iter().all(|b| b.hooks.is_none()));
}
