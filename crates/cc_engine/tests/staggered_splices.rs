//! Staggered lap splices of a column's vertical bars (#351): with `stagger`
//! every other bar around the section splices half a lap, or an offset,
//! higher than its neighbours; without it every bar is cut at the same heights.

use cc_core::{ElementId, ElementKind, Stagger};
use cc_engine::{ErrorCode, Session};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn run(s: &mut Session, name: &str, args: Value) -> Value {
    s.execute(name, args).unwrap_or_else(|e| panic!("{name}: {e:?}"))
}

fn fails(s: &mut Session, name: &str, args: Value) -> ErrorCode {
    s.execute(name, args).map(|v| panic!("{name} should fail, gave {v}")).unwrap_err().code
}

fn id(v: &Value) -> ElementId {
    ElementId(v["id"].as_u64().unwrap_or_else(|| panic!("no id in {v}")))
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// A free column 9 m high.
fn column(s: &mut Session) -> ElementId {
    let column = id(&run(s, "column_create", json!({"position": [0, 0]})));
    run(s, "element_set", json!({"id": column.0, "params": {"top": "Unconnected", "height": 9.0}}));
    column
}

/// The pieces `(from, to)` above the base of each bar of the set, by the bar's place in plan
/// (millimeters), and whether that place is a corner of the section.
fn pieces_by_bar(s: &mut Session, set: ElementId) -> BTreeMap<(i64, i64), Vec<(f64, f64)>> {
    let geom = s.geometry().clone();
    let r = &geom.rebar[&set];
    let base = r.bars.iter().map(|b| b.points[0].z).fold(f64::INFINITY, f64::min);
    let mut out: BTreeMap<(i64, i64), Vec<(f64, f64)>> = BTreeMap::new();
    for b in &r.bars {
        let key = ((b.points[0].x * 1000.0).round() as i64, (b.points[0].y * 1000.0).round() as i64);
        out.entry(key).or_default().push((b.points[0].z - base, b.points[1].z - base));
    }
    for v in out.values_mut() {
        v.sort_by(|a, b| a.0.total_cmp(&b.0));
    }
    out
}

fn same(got: &[(f64, f64)], want: &[(f64, f64)]) -> bool {
    got.len() == want.len() && got.iter().zip(want).all(|(a, b)| close(a.0, b.0) && close(a.1, b.1))
}

fn corner(key: &(i64, i64), all: &BTreeMap<(i64, i64), Vec<(f64, f64)>>) -> bool {
    let (mx, my) = all.keys().fold((0, 0), |m, k| (m.0.max(k.0.abs()), m.1.max(k.1.abs())));
    key.0.abs() == mx && key.1.abs() == my
}

/// Equal lifts: 9 m under a stock of 4 m lapping 0.5 m.
#[test]
fn every_other_bar_splices_higher() {
    let mut s = Session::default();
    let c = column(&mut s);
    let set = |stagger: Option<Value>| {
        let mut args = json!({"host": c.0, "layout": "perimeter", "count": 8, "max_length": 4.0, "lap": 0.5});
        if let Some(st) = stagger {
            args["stagger"] = st;
        }
        args
    };
    // Unstaggered: n = ceil((9 - 0.5) / (4 - 0.5)) = 3 pieces at a pitch p = 8.5 / 3, cut at p and
    // 2p, each p + 0.5 = 3.3333 m long; every bar the same.
    let p = 8.5 / 3.0;
    let plain = [(0.0, p + 0.5), (p, 2.0 * p + 0.5), (2.0 * p, 9.0)];
    let none = id(&run(&mut s, "rebar_create", set(None)));
    let bars = pieces_by_bar(&mut s, none);
    assert_eq!(bars.len(), 8);
    assert!(bars.values().all(|v| same(v, &plain)), "{bars:?}");
    let explicit = id(&run(&mut s, "rebar_create", set(Some(json!("none")))));
    assert_eq!(pieces_by_bar(&mut s, explicit), bars);
    let ElementKind::Rebar(r) = &s.project().element(explicit).unwrap().kind else { panic!("rebar") };
    assert_eq!(r.splice.unwrap().stagger, Stagger::None);

    // Half a lap, 0.25 m: n = ceil(8.5 / (4 - 0.5 - 0.25)) = 3 still, the same pitch. The staggered
    // bars are cut at p + 0.25 and 2p + 0.25: pieces of p + 0.75 = 3.5833 m (from the base),
    // p + 0.5 and p + 0.25 = 3.0833 m (to the top), none over 4 m.
    let half = id(&run(&mut s, "rebar_create", set(Some(json!("half_lap")))));
    let o = 0.25;
    let shifted = [(0.0, p + o + 0.5), (p + o, 2.0 * p + o + 0.5), (2.0 * p + o, 9.0)];
    let bars = pieces_by_bar(&mut s, half);
    for (key, v) in &bars {
        // Around a box: the corners, then every bar between two of them, alternate.
        let want: &[(f64, f64)] = if corner(key, &bars) { &plain } else { &shifted };
        assert!(same(v, want), "{key:?}: {v:?}");
    }
    assert_eq!(bars.values().filter(|v| same(v, &shifted)).count(), 4);
    let geom = s.geometry().clone();
    let r = &geom.rebar[&half];
    assert!(r.bars.iter().all(|b| b.length() <= 4.0 + 1e-9));
    // Each bar is 9 m and two laps, staggered or not.
    assert!(close(r.total_length(), 8.0 * (9.0 + 2.0 * 0.5)), "{}", r.total_length());
    assert!(close(r.total_length(), geom.rebar[&none].total_length()));

    // An offset of 0.6 m: n = ceil(8.5 / 2.9) = 3, the first staggered piece p + 1.1 = 3.9333 m.
    let off = id(&run(&mut s, "rebar_create", set(Some(json!(0.6)))));
    let bars = pieces_by_bar(&mut s, off);
    let o = 0.6;
    let shifted = [(0.0, p + o + 0.5), (p + o, 2.0 * p + o + 0.5), (2.0 * p + o, 9.0)];
    assert_eq!(bars.values().filter(|v| same(v, &shifted)).count(), 4, "{bars:?}");
    assert_eq!(bars.values().filter(|v| same(v, &plain)).count(), 4, "{bars:?}");

    // An offset of 1 m would make that first piece p + 1.5 = 4.33 m: the lifts are shortened for
    // every bar instead, n = ceil(8.5 / (4 - 0.5 - 1)) = 4 at a pitch q = 2.125, pieces of 2.625 m,
    // the staggered first 3.625 m and last 1.625 m.
    let long = id(&run(&mut s, "rebar_create", set(Some(json!(1.0)))));
    let bars = pieces_by_bar(&mut s, long);
    let q = 2.125;
    let plain4: Vec<(f64, f64)> =
        (0..4).map(|i| (q * i as f64, if i == 3 { 9.0 } else { q * (i + 1) as f64 + 0.5 })).collect();
    let shifted4 = [(0.0, q + 1.5), (q + 1.0, 2.0 * q + 1.5), (2.0 * q + 1.0, 3.0 * q + 1.5), (3.0 * q + 1.0, 9.0)];
    assert_eq!(bars.values().filter(|v| same(v, &plain4)).count(), 4, "{bars:?}");
    assert_eq!(bars.values().filter(|v| same(v, &shifted4)).count(), 4, "{bars:?}");
    let geom = s.geometry().clone();
    assert!(geom.rebar[&long].bars.iter().all(|b| b.length() <= 4.0 + 1e-9));
    assert!(close(geom.rebar[&long].total_length(), 8.0 * (9.0 + 3.0 * 0.5)));
}

/// Lifts at given heights; a splice moved to less than a lap below the top is left out.
#[test]
fn given_lifts_are_staggered_too() {
    let mut s = Session::default();
    let c = column(&mut s);
    let args =
        json!({"host": c.0, "layout": "perimeter", "count": 4, "lifts": [3.0, 6.0], "lap": 0.5, "stagger": "half_lap"});
    let set = id(&run(&mut s, "rebar_create", args));
    let bars = pieces_by_bar(&mut s, set);
    // Four corners around a box: two plain, two cut 0.25 m higher.
    let plain = [(0.0, 3.5), (3.0, 6.5), (6.0, 9.0)];
    let shifted = [(0.0, 3.75), (3.25, 6.75), (6.25, 9.0)];
    assert_eq!(bars.values().filter(|v| same(v, &plain)).count(), 2, "{bars:?}");
    assert_eq!(bars.values().filter(|v| same(v, &shifted)).count(), 2, "{bars:?}");
    // Opposite corners alike: neighbours differ.
    let keys: Vec<_> = bars.keys().copied().collect();
    let first = &bars[&keys[0]];
    let opposite = keys.iter().find(|k| k.0 == -keys[0].0 && k.1 == -keys[0].1).unwrap();
    assert!(same(first, &bars[opposite]));

    // A lift at 8.6 m moved to 8.85 m would leave 0.15 m, less than a lap, to the top: that bar runs
    // from 3.25 m to the top.
    let args =
        json!({"host": c.0, "layout": "perimeter", "count": 4, "lifts": [3.0, 8.6], "lap": 0.5, "stagger": "half_lap"});
    let set = id(&run(&mut s, "rebar_create", args));
    let bars = pieces_by_bar(&mut s, set);
    let plain = [(0.0, 3.5), (3.0, 9.0), (8.6, 9.0)];
    let shifted = [(0.0, 3.75), (3.25, 9.0)];
    assert_eq!(bars.values().filter(|v| same(v, &plain)).count(), 2, "{bars:?}");
    assert_eq!(bars.values().filter(|v| same(v, &shifted)).count(), 2, "{bars:?}");
}

/// Staggered bars are other shapes: their own marks, the same total.
#[test]
fn staggered_bars_get_their_own_marks() {
    let mut s = Session::default();
    let c = column(&mut s);
    let args =
        json!({"host": c.0, "layout": "perimeter", "count": 8, "max_length": 4.0, "lap": 0.5, "stagger": "half_lap"});
    run(&mut s, "rebar_create", args);
    let marks = run(&mut s, "bar_marks_list", json!({}))["marks"].as_array().unwrap().clone();
    // p + 0.5 (all the plain pieces and the staggered middle ones), p + 0.75 and p + 0.25.
    assert_eq!(marks.len(), 3, "{marks:?}");
    let count: u64 = marks.iter().map(|m| m["count"].as_u64().unwrap()).sum();
    assert_eq!(count, 24);
}

#[test]
fn stagger_is_refused_where_it_does_not_apply() {
    let mut s = Session::default();
    let c = column(&mut s);
    let beam = id(&run(&mut s, "beam_create", json!({"start": [3, 0], "end": [20, 0]})));
    let bad = [
        // Out of range: (4 - 0.5) / 2 = 1.75 at most, above 0.
        json!({"host": c.0, "layout": "perimeter", "max_length": 4.0, "lap": 0.5, "stagger": 1.8}),
        json!({"host": c.0, "layout": "perimeter", "max_length": 4.0, "lap": 0.5, "stagger": 0.0}),
        json!({"host": c.0, "layout": "perimeter", "max_length": 4.0, "lap": 0.5, "stagger": -0.2}),
        json!({"host": c.0, "layout": "perimeter", "max_length": 4.0, "stagger": "quarter"}),
        // Whole bars: nothing to stagger.
        json!({"host": c.0, "layout": "perimeter", "stagger": "half_lap"}),
        // Not a column's vertical bars.
        json!({"host": c.0, "layout": "stirrups", "max_length": 4.0, "stagger": "half_lap"}),
        json!({"host": beam.0, "layout": "bottom", "stagger": "half_lap"}),
    ];
    for args in bad {
        assert_eq!(fails(&mut s, "rebar_create", args.clone()), ErrorCode::InvalidArguments, "{args}");
    }
    let beam_bars = id(&run(&mut s, "rebar_create", json!({"host": beam.0, "layout": "bottom"})));
    let patch = json!({"id": beam_bars.0, "params": {"splice": {"stagger": "half_lap"}}});
    assert!(s.execute("element_set", patch).is_err());
}

#[test]
fn element_set_changes_the_stagger() {
    let mut s = Session::default();
    let c = column(&mut s);
    let args = json!({"host": c.0, "layout": "perimeter", "count": 4, "max_length": 4.0, "lap": 0.5});
    let set = id(&run(&mut s, "rebar_create", args));
    let stagger = |s: &Session| match &s.project().element(set).unwrap().kind {
        ElementKind::Rebar(r) => r.splice.unwrap().stagger,
        _ => panic!("rebar"),
    };
    run(&mut s, "element_set", json!({"id": set.0, "params": {"splice": {"stagger": "half_lap"}}}));
    assert_eq!(stagger(&s), Stagger::HalfLap);
    let got = run(&mut s, "element_get", json!({"id": set.0}));
    assert_eq!(got["params"]["splice"]["stagger"], json!("half_lap"), "{got}");
    run(&mut s, "element_set", json!({"id": set.0, "params": {"splice": {"stagger": {"offset": 0.6}}}}));
    assert_eq!(stagger(&s), Stagger::Offset(0.6));
    let bad = json!({"id": set.0, "params": {"splice": {"stagger": {"offset": 2.0}}}});
    assert!(s.execute("element_set", bad).is_err());
    assert_eq!(stagger(&s), Stagger::Offset(0.6));
    run(&mut s, "element_set", json!({"id": set.0, "params": {"splice": {"stagger": "none"}}}));
    assert_eq!(stagger(&s), Stagger::None);
}

/// Files without a stagger load as before; a stagger out of range resets to none, keeping the splices.
#[test]
fn files_keep_or_repair_the_stagger() {
    let mut s = Session::default();
    let c = column(&mut s);
    let args = json!({"host": c.0, "layout": "perimeter", "count": 4, "max_length": 4.0, "lap": 0.5});
    let plain = id(&run(&mut s, "rebar_create", args.clone()));
    let mut staggered = args;
    staggered["stagger"] = json!(0.6);
    let staggered = id(&run(&mut s, "rebar_create", staggered));
    let text = cc_core::to_ron(s.project()).unwrap();
    // An unstaggered set writes no stagger, as older files hold it.
    assert_eq!(text.matches("stagger").count(), 1, "only the staggered set");
    let (back, repairs) = cc_core::from_ron_with_repairs(&text).unwrap();
    assert_eq!(repairs, 0);
    assert_eq!(back.element(plain), s.project().element(plain));
    assert_eq!(back.element(staggered), s.project().element(staggered));
    // A crafted offset too large for the stock: none, the splices kept.
    assert!(text.contains("offset(0.6)"), "{text}");
    let crafted = text.replace("offset(0.6)", "offset(100.0)");
    let (back, repairs) = cc_core::from_ron_with_repairs(&crafted).unwrap();
    assert_eq!(repairs, 1);
    let ElementKind::Rebar(r) = &back.element(staggered).unwrap().kind else { panic!("rebar") };
    let splice = r.splice.expect("the splices kept");
    assert_eq!(splice.stagger, Stagger::None);
    assert!(close(splice.max_length, 4.0) && close(splice.lap, 0.5));
}
