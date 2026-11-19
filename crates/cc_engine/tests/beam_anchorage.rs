//! Anchorage of a beam's bars into the columns and pile caps it frames into
//! (#346): `rebar_create` `anchor` runs top, bottom and side bars on into
//! each support, up to its far face less the cover and at most the anchorage
//! length, bends top and bottom bars into a 90° hook where the support is too
//! shallow, and laps the bars of two beams in line inside the support between
//! them, along straight beams and around a ring of arc beams.
//!
//! The beams are 20 x 50 cm with 10 mm bars, a cover of 2.5 cm and no
//! stirrups: the bars' lines are 0.025 + 0.005 = 0.03 inside the faces, so
//! two bars lie 0.07 either side of the axis, bottom bars 0.03 above the
//! beam's bottom and top bars 0.03 under its top. The anchorage is 40
//! diameters (0.4 m) and the lap 40 diameters too (0.4 m) unless given.

use cc_core::{AnchorHook, Anchorage, ElementId, ElementKind};
use cc_engine::{ErrorCode, Session};
use cc_geom::rebar::Bar;
use serde_json::{Value, json};
use std::f64::consts::FRAC_PI_2;

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

/// A session with the 20 x 50 cm beam type `V20x50`.
fn session() -> Session {
    let mut s = Session::default();
    let profile = json!({"name": "V20x50", "material": "concrete", "shape": "rect", "params": {"b": 0.2, "h": 0.5}});
    run(&mut s, "profile_create", profile);
    run(&mut s, "profile_type_create", json!({"profile": "V20x50", "category": "beam"}));
    s
}

/// A square column type `side` wide.
fn column_type(s: &mut Session, side: f64) -> Value {
    let name = format!("P{}", (side * 100.0).round());
    let profile = json!({"name": name, "material": "concrete", "shape": "rect", "params": {"b": side, "h": side}});
    run(s, "profile_create", profile);
    run(s, "profile_type_create", json!({"profile": name, "category": "column"}))["id"].clone()
}

/// A column of `ty` at `(x, y)` from the level to 3 m, turned by `rotation` degrees.
fn column(s: &mut Session, ty: &Value, x: f64, y: f64, rotation: f64) -> ElementId {
    let c = id(&run(s, "column_create", json!({"position": [x, y], "rotation": rotation, "type": ty})));
    run(s, "element_set", json!({"id": c.0, "params": {"top": "Unconnected", "height": 3.0}}));
    c
}

/// A 20 x 50 cm beam from `a` to `b` whose top is 3 m above the level (2.5 to 3.0 m).
fn beam(s: &mut Session, a: [f64; 2], b: [f64; 2]) -> ElementId {
    id(&run(s, "beam_create", json!({"start": a, "end": b, "top_offset": 3.0, "type": "V20x50"})))
}

/// A set of bars in `host`: `layout` with the extra arguments of `more`.
fn set(s: &mut Session, host: ElementId, layout: &str, more: Value) -> ElementId {
    let mut args = json!({"host": host.0, "layout": layout, "cover": 0.025});
    if let (Some(a), Some(m)) = (args.as_object_mut(), more.as_object()) {
        a.extend(m.clone());
    }
    id(&run(s, "rebar_create", args))
}

/// The bars of a set (never none, so that checks over them all check something).
fn bars(s: &mut Session, set: ElementId) -> Vec<Bar> {
    let bars = s.geometry().rebar[&set].bars.clone();
    assert!(!bars.is_empty(), "the set {set:?} has no bars");
    bars
}

/// The lowest and highest x of a bar.
fn span_x(bar: &Bar) -> (f64, f64) {
    let xs = bar.points.iter().map(|p| p.x);
    (xs.clone().fold(f64::INFINITY, f64::min), xs.fold(f64::NEG_INFINITY, f64::max))
}

/// The bars of a set (some) measure `length` each and span `x0` to `x1`.
fn all_span(bars: &[Bar], x0: f64, x1: f64, length: f64) -> bool {
    !bars.is_empty()
        && bars.iter().all(|b| {
            let (lo, hi) = span_x(b);
            close(lo, x0) && close(hi, x1) && close(b.length(), length)
        })
}

/// A straight beam between two 60 cm columns: the bars run 40 diameters into
/// each, and an end with no column keeps its cover. The length shows in the
/// set, its bar marks and the bending schedule.
#[test]
fn bars_run_straight_into_deep_columns() {
    let mut s = session();
    let p60 = column_type(&mut s, 0.6);
    column(&mut s, &p60, 0.0, 0.0, 0.0);
    column(&mut s, &p60, 6.0, 0.0, 0.0);
    let b = beam(&mut s, [0.0, 0.0], [6.0, 0.0]);
    let plain = set(&mut s, b, "bottom", json!({}));
    let bottom = set(&mut s, b, "bottom", json!({"anchor": true}));
    let top = set(&mut s, b, "top", json!({"anchor": true}));
    assert!(close(s.geometry().rebar[&bottom].diameter, 0.01));
    // Between the faces at 0.3 and 5.7, the cover off each end: 5.4 - 0.05 = 5.35.
    let plain_bars = bars(&mut s, plain);
    assert!(all_span(&plain_bars, 0.325, 5.675, 5.35), "{plain_bars:?}");
    // The columns are 0.6 deep: 0.6 - 0.025 leaves room for the 0.4 m anchorage past each
    // face, straight: from 0.3 - 0.4 = -0.1 to 5.7 + 0.4 = 6.1, 6.2 m.
    for set in [bottom, top] {
        let got = bars(&mut s, set);
        assert!(got.len() == 2 && got.iter().all(|b| b.points.len() == 2), "{got:?}");
        assert!(all_span(&got, -0.1, 6.1, 6.2), "{got:?}");
    }
    let params = run(&mut s, "element_get", json!({"id": bottom.0}));
    assert_eq!(params["params"]["anchorage"], json!({"length": 40.0, "hook": "auto"}));
    assert!(close(params["derived"]["bar_length"].as_f64().unwrap(), 6.2), "{params}");
    // Bar marks and the bending schedule measure the anchored bars.
    let marks = run(&mut s, "bar_marks_list", json!({}))["marks"].as_array().unwrap().clone();
    let mark = marks.iter().find(|m| m["sets"].as_array().unwrap().contains(&json!(bottom.0))).unwrap();
    assert!(close(mark["unit_length"].as_f64().unwrap(), 6.2), "{mark}");
    assert_eq!(mark["code"], json!("straight 620"));
    let schedule = id(&run(&mut s, "bending_schedule_create", json!({})));
    let text = run(&mut s, "schedule_table", json!({"view": schedule.0})).to_string();
    assert!(text.contains("straight 620") && text.contains("straight 535"), "{text}");

    // A beam with a column at its start only: the free end keeps its cover.
    column(&mut s, &p60, 20.0, 0.0, 0.0);
    let c = beam(&mut s, [20.0, 0.0], [25.0, 0.0]);
    let one = set(&mut s, c, "bottom", json!({"anchor": true}));
    assert!(all_span(&bars(&mut s, one), 19.9, 24.975, 5.075));

    // A column turned by 30°: the bar meets its face where its own line does, and
    // its far face 0.6 / cos 30° deeper along that line. The line y = v leaves the
    // column (centered at x = 40) through the faces x cos 30 + y sin 30 = ±0.3.
    let turned = 30f64.to_radians();
    column(&mut s, &p60, 40.0, 0.0, 30.0);
    column(&mut s, &p60, 46.0, 0.0, 0.0);
    let d = beam(&mut s, [40.0, 0.0], [46.0, 0.0]);
    let near = |v: f64| 40.0 + (0.3 - v * turned.sin()) / turned.cos();
    let far = |v: f64| 40.0 - (0.3 + v * turned.sin()) / turned.cos();
    // 40 diameters fit (the column is 0.693 deep along the bar).
    let fits = set(&mut s, d, "bottom", json!({"anchor": true}));
    for bar in bars(&mut s, fits) {
        let v = bar.points[0].y;
        let (lo, hi) = span_x(&bar);
        assert!(close(lo, near(v) - 0.4) && close(hi, 45.7 + 0.4), "{bar:?}");
    }
    // 80 do not: straight to the far face less the cover, then a hook up of the rest.
    let hooked = set(&mut s, d, "bottom", json!({"anchor": 80}));
    for bar in bars(&mut s, hooked) {
        let v = bar.points[1].y;
        let straight = near(v) - far(v) - 0.025;
        let (lo, _) = span_x(&bar);
        assert!(close(lo, far(v) + 0.025), "{bar:?}");
        let leg = bar.points[0].z - bar.points[1].z;
        assert!(close(leg, 0.8 - straight), "{leg} vs {}", 0.8 - straight);
    }
}

/// Columns 30 cm deep leave 0.3 - 0.025 = 0.275 m straight: top and bottom bars
/// end in a 90° hook whose leg makes up the 0.4 m (0.125 m), side bars run
/// straight to the far face less the cover.
#[test]
fn a_shallow_column_bends_the_bars_into_hooks() {
    let mut s = session();
    let p30 = column_type(&mut s, 0.3);
    column(&mut s, &p30, 0.0, 0.0, 0.0);
    column(&mut s, &p30, 6.0, 0.0, 0.0);
    let b = beam(&mut s, [0.0, 0.0], [6.0, 0.0]);
    let geom = s.geometry().clone();
    let bottom_z = geom.beams[&b].z_range.0;
    // The faces at 0.15 and 5.85; the runs from -0.125 to 6.125, 6.25 m, and two legs.
    let (x0, x1, along) = (-0.125, 6.125, 6.25);
    let bottom = set(&mut s, b, "bottom", json!({"anchor": true}));
    let top = set(&mut s, b, "top", json!({"anchor": true}));
    for (set, up) in [(bottom, true), (top, false)] {
        for bar in bars(&mut s, set) {
            assert!(all_span(std::slice::from_ref(&bar), x0, x1, along + 2.0 * 0.125), "{bar:?}");
            assert_eq!(bar.points.len(), 4, "{bar:?}");
            // Bottom bars hook up, top bars down, 0.125 m each.
            let sign = if up { 1.0 } else { -1.0 };
            assert!(close(bar.points[0].z - bar.points[1].z, sign * 0.125), "{bar:?}");
            assert!(close(bar.points[3].z - bar.points[2].z, sign * 0.125), "{bar:?}");
            assert!(close(bar.points[0].x, x0) && close(bar.points[1].x, x0) && close(bar.points[3].x, x1));
        }
    }
    assert!(bars(&mut s, bottom).iter().all(|b| close(b.points[1].z, bottom_z + 0.03)));
    // A hooked bar is a U in the bending schedule: its two legs and its run (625 cm).
    let marks = run(&mut s, "bar_marks_list", json!({}))["marks"].as_array().unwrap().clone();
    let u = |m: &&Value| m["code"].as_str().is_some_and(|c| c.starts_with("U ") && c.contains("-625-"));
    assert!(marks.iter().find(u).is_some_and(|m| m["count"] == json!(4)), "{marks:?}");
    let legs = marks.iter().find(u).map(|m| m["shape"]["legs"].clone()).unwrap_or_default();
    let legs: Vec<f64> = legs.as_array().unwrap().iter().map(|l| l["length"].as_f64().unwrap()).collect();
    assert!(legs.len() == 3 && close(legs[0], 0.125) && close(legs[1], along) && close(legs[2], 0.125), "{legs:?}");

    // Side bars never bend.
    let sides = set(&mut s, b, "sides", json!({"anchor": true}));
    let side_bars = bars(&mut s, sides);
    assert!(side_bars.iter().all(|b| b.points.len() == 2) && all_span(&side_bars, x0, x1, along));
    // No hooks: straight to the far face less the cover.
    let straight = set(&mut s, b, "bottom", json!({"anchor": true, "anchor_hook": "none"}));
    assert!(all_span(&bars(&mut s, straight), x0, x1, along));
    // A top bar hooked up has no room above it (0.5 cm under the column's top less the cover).
    let up = set(&mut s, b, "top", json!({"anchor": true, "anchor_hook": "up"}));
    assert!(bars(&mut s, up).iter().all(|b| b.points.len() == 2 && close(b.length(), along)));
    // A bottom bar hooked down: the column below leaves it room.
    let down = set(&mut s, b, "bottom", json!({"anchor": true, "anchor_hook": "down"}));
    assert!(bars(&mut s, down).iter().all(|b| close(b.points[0].z - b.points[1].z, -0.125)));
    // 20 diameters fit straight: 0.2 past each face.
    let short = set(&mut s, b, "bottom", json!({"anchor": 20}));
    assert!(all_span(&bars(&mut s, short), -0.05, 6.05, 6.1));
    // 100 diameters (1 m) want legs of 1 - 0.275 = 0.725 m: the column's top less the
    // cover leaves 3.0 - 0.025 - 2.53 = 0.445 m.
    let long = set(&mut s, b, "bottom", json!({"anchor": 100}));
    for bar in bars(&mut s, long) {
        assert!(close(bar.points[0].z - bar.points[1].z, 0.445) && close(bar.length(), along + 2.0 * 0.445));
    }
    // A second layer (six bars: five fit across 0.14 m at 3 cm) bends the same way, 3 cm higher.
    let six = set(&mut s, b, "bottom", json!({"anchor": true, "count": 6}));
    let got = bars(&mut s, six);
    assert_eq!(got.len(), 6);
    assert!(got.iter().all(|b| close(b.length(), along + 0.25) && close(b.points[0].z - b.points[1].z, 0.125)));
    assert_eq!(got.iter().filter(|b| close(b.points[1].z, bottom_z + 0.06)).count(), 1);
}

/// Two beams in line over a column: the bars of each run half a lap past its
/// middle, so they lap by the lap length inside it; a column too shallow for
/// that takes each to its far face less the cover. `lap_at_joins` adds nothing there.
#[test]
fn bars_of_beams_in_line_lap_inside_the_column() {
    let mut s = session();
    let (p60, p30) = (column_type(&mut s, 0.6), column_type(&mut s, 0.3));
    for x in [0.0, 5.0, 10.0] {
        column(&mut s, &p60, x, 0.0, 0.0);
    }
    let a = beam(&mut s, [0.0, 0.0], [5.0, 0.0]);
    let b = beam(&mut s, [5.0, 0.0], [10.0, 0.0]);
    let sets = [set(&mut s, a, "bottom", json!({"anchor": true})), set(&mut s, b, "bottom", json!({"anchor": true}))];
    // A: 0.4 into the end column (from -0.1), and 0.3 + 0.2 past the face at 4.7 to 5.2;
    // B: from 5.3 - 0.5 = 4.8 to 9.7 + 0.4 = 10.1. They lap from 4.8 to 5.2: 0.4.
    assert!(all_span(&bars(&mut s, sets[0]), -0.1, 5.2, 5.3));
    assert!(all_span(&bars(&mut s, sets[1]), 4.8, 10.1, 5.3));
    // With `lap_at_joins` too: the same (no second half lap).
    let both = [
        set(&mut s, a, "bottom", json!({"anchor": true, "lap_at_joins": true})),
        set(&mut s, b, "bottom", json!({"anchor": true, "lap_at_joins": true})),
    ];
    assert!(all_span(&bars(&mut s, both[0]), -0.1, 5.2, 5.3));
    assert!(all_span(&bars(&mut s, both[1]), 4.8, 10.1, 5.3));
    // A lap of 0.5: 0.25 past the middle, within 0.6 - 0.025.
    let lap = [
        set(&mut s, a, "bottom", json!({"anchor": true, "lap": 0.5})),
        set(&mut s, b, "bottom", json!({"anchor": true, "lap": 0.5})),
    ];
    assert!(all_span(&bars(&mut s, lap[0]), -0.1, 5.25, 5.35));
    assert!(all_span(&bars(&mut s, lap[1]), 4.75, 10.1, 5.35));

    // A 30 cm column between two beams: half the column and half a lap (0.35) passes
    // its far face less the cover (0.275), so each bar stops there, without a hook.
    column(&mut s, &p60, 0.0, 10.0, 0.0);
    column(&mut s, &p30, 5.0, 10.0, 0.0);
    column(&mut s, &p60, 10.0, 10.0, 0.0);
    let c = beam(&mut s, [0.0, 10.0], [5.0, 10.0]);
    let d = beam(&mut s, [5.0, 10.0], [10.0, 10.0]);
    let short = [set(&mut s, c, "bottom", json!({"anchor": true})), set(&mut s, d, "bottom", json!({"anchor": true}))];
    // C to 4.85 + 0.275 = 5.125, D from 5.15 - 0.275 = 4.875: they lap by 0.25.
    assert!(all_span(&bars(&mut s, short[0]), -0.1, 5.125, 5.225));
    assert!(all_span(&bars(&mut s, short[1]), 4.875, 10.1, 5.225));
    assert!(bars(&mut s, short[0]).iter().all(|b| b.points.len() == 2));
}

/// The issue's case: a ring beam of four arc beams between four columns. The
/// bottom bars of neighbouring beams lap by the lap length inside each column
/// and their lengths in the schedule include the anchorage.
#[test]
fn a_ring_of_arc_beams_laps_its_bottom_bars_inside_each_column() {
    let mut s = session();
    let p60 = column_type(&mut s, 0.6);
    let r = 4.0;
    let corners = [[r, 0.0], [0.0, r], [-r, 0.0], [0.0, -r]];
    for c in corners {
        column(&mut s, &p60, c[0], c[1], 0.0);
    }
    let mut sets = Vec::new();
    for k in 0..4 {
        let (a, b) = (corners[k], corners[(k + 1) % 4]);
        let args = json!({"start": a, "end": b, "radius": r, "top_offset": 3.0, "type": "V20x50"});
        let beam = id(&run(&mut s, "beam_create", args));
        sets.push(set(&mut s, beam, "bottom", json!({"anchor": true})));
    }
    // The two bars of each beam follow circles of radius 4 ∓ 0.07 (counter-clockwise, the
    // left side inward). Each meets the near face of a column (y = 0.3 for the column on
    // +X) at the angle asin(0.3 / ρ) from the column's middle line, and the column is
    // 2 ρ asin(0.3 / ρ) deep along it (some 0.6 m, more than half of it and half a lap
    // past the near face leaves the cover). The bar runs ρ asin(0.3 / ρ) + 0.4 / 2 past each
    // face: half a lap past each column's middle. Its length is the arc at its radius
    // between the faces, ρ (π/2 - 2 asin(0.3 / ρ)), and the two extensions, which is
    // ρ π/2 + 0.4.
    let radii = [r - 0.07, r + 0.07];
    let length = |rho: f64| rho * FRAC_PI_2 + 0.4;
    let geom = s.geometry().clone();
    let angle = |p: glam::DVec3| p.y.atan2(p.x);
    for (k, set) in sets.iter().enumerate() {
        let bars = &geom.rebar[set].bars;
        assert_eq!(bars.len(), 2);
        for bar in bars {
            let rho = bar.points[0].truncate().length();
            let want = radii.iter().copied().find(|x| (x - rho).abs() < 1e-9).unwrap_or_else(|| panic!("{rho}"));
            assert!(bar.points.iter().all(|p| (p.truncate().length() - want).abs() < 1e-9));
            assert!(close(bar.length(), length(want)), "{} vs {}", bar.length(), length(want));
            // Half a lap (0.2) past the middles of the columns at each end.
            let (Some(first), Some(last)) = (bar.points.first(), bar.points.last()) else { panic!() };
            let start = k as f64 * FRAC_PI_2;
            let turn = |a: f64| (a + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI;
            assert!(close(turn(angle(*first) - start) * want, -0.2), "{first}");
            assert!(close(turn(angle(*last) - start - FRAC_PI_2) * want, 0.2), "{last}");
        }
    }
    // The bars of neighbouring beams on one circle lap by 0.4 inside each column.
    for k in 0..4 {
        for rho in radii {
            let on = |set: &ElementId| {
                geom.rebar[set].bars.iter().find(|b| (b.points[0].truncate().length() - rho).abs() < 1e-9).unwrap()
            };
            let (this, next) = (on(&sets[k]), on(&sets[(k + 1) % 4]));
            let (end, start) = (angle(*this.points.last().unwrap()), angle(next.points[0]));
            let lap = ((end - start).rem_euclid(std::f64::consts::TAU)) * rho;
            assert!(close(lap, 0.4), "{lap}");
        }
    }
    // The schedule: two marks of four bars, each as long as the anchored arcs.
    let marks = run(&mut s, "bar_marks_list", json!({}))["marks"].as_array().unwrap().clone();
    assert_eq!(marks.len(), 2, "{marks:?}");
    for rho in radii {
        let m = marks.iter().find(|m| (m["unit_length"].as_f64().unwrap() - length(rho)).abs() < 1e-6);
        let m = m.unwrap_or_else(|| panic!("no mark of {} in {marks:?}", length(rho)));
        assert_eq!(m["count"], json!(4));
        assert!((m["total_length"].as_f64().unwrap() - 4.0 * length(rho)).abs() < 1e-6);
    }
    let schedule = id(&run(&mut s, "bending_schedule_create", json!({})));
    let text = run(&mut s, "schedule_table", json!({"view": schedule.0})).to_string();
    for rho in radii {
        assert!(text.contains(&cc_views::format::bar_length(length(rho))), "{text}");
    }
}

/// Splices cut the anchored bar, hooks included: the anchorage adds to the bar first.
#[test]
fn anchored_bars_are_spliced_whole() {
    let mut s = session();
    let (p60, p30) = (column_type(&mut s, 0.6), column_type(&mut s, 0.3));
    column(&mut s, &p60, 0.0, 0.0, 0.0);
    column(&mut s, &p60, 6.0, 0.0, 0.0);
    let b = beam(&mut s, [0.0, 0.0], [6.0, 0.0]);
    // 6.2 m under a stock of 6 m lapping 0.5: two pieces at a pitch of (6.2 - 0.5) / 2 = 2.85,
    // the second bar's cuts half a pitch off: (0, 1.925), (1.425, 4.775), (4.275, 6.2).
    let spliced = set(&mut s, b, "bottom", json!({"anchor": true, "max_length": 6, "lap": 0.5}));
    let got = bars(&mut s, spliced);
    assert_eq!(got.len(), 5);
    let total: f64 = got.iter().map(Bar::length).sum();
    assert!(close(total, 6.2 + 0.5 + 6.2 + 1.0), "{total}");
    assert!(got.iter().any(|b| close(span_x(b).0, -0.1) && close(span_x(b).1, -0.1 + 3.35)));
    assert!(got.iter().any(|b| close(span_x(b).0, -0.1 + 4.275) && close(span_x(b).1, 6.1)));

    // Hooked: 0.125 + 6.25 + 0.125 = 6.5 m in two pieces at a pitch of 3.0, each 3.5 long:
    // the first from its hook to x = -0.125 + 3.375, the second from -0.125 + 2.875 to its hook.
    column(&mut s, &p30, 0.0, 10.0, 0.0);
    column(&mut s, &p30, 6.0, 10.0, 0.0);
    let c = beam(&mut s, [0.0, 10.0], [6.0, 10.0]);
    let hooked = set(&mut s, c, "bottom", json!({"anchor": true, "max_length": 6, "lap": 0.5}));
    let got = bars(&mut s, hooked);
    let total: f64 = got.iter().map(Bar::length).sum();
    assert!(close(total, 6.5 + 0.5 + 6.5 + 1.0), "{total}");
    let first = got.iter().find(|b| close(b.length(), 3.5) && close(span_x(b).0, -0.125)).unwrap();
    assert_eq!(first.points.len(), 3, "{first:?}");
    assert!(close(first.points[0].z - first.points[1].z, 0.125) && close(span_x(first).1, 3.25));
    let last = got.iter().find(|b| close(b.length(), 3.5) && close(span_x(b).1, 6.125)).unwrap();
    assert!(last.points.len() == 3 && close(span_x(last).0, 2.75), "{last:?}");
}

/// `lap_at_joins` acts where the beam carries on into another beam, the
/// anchorage where it frames into a column: one extension at each end.
#[test]
fn laps_at_joins_and_anchorage_share_a_beam() {
    let mut s = session();
    let p60 = column_type(&mut s, 0.6);
    column(&mut s, &p60, 0.0, 0.0, 0.0);
    let a = beam(&mut s, [0.0, 0.0], [5.0, 0.0]);
    let b = beam(&mut s, [5.0, 0.0], [10.0, 0.0]);
    let args = json!({"anchor": true, "lap_at_joins": true, "max_length": 0});
    let (sa, sb) = (set(&mut s, a, "bottom", args.clone()), set(&mut s, b, "bottom", args));
    // A: 0.4 into the column, half a lap (0.2) past the joint at 5; B: from 4.8 to its free end.
    assert!(all_span(&bars(&mut s, sa), -0.1, 5.2, 5.3));
    assert!(all_span(&bars(&mut s, sb), 4.8, 9.975, 5.175));
}

/// A tie beam between pile caps anchors into them as into columns, its hooks
/// kept inside the cap's thickness less the cover.
#[test]
fn a_tie_beam_anchors_into_its_pile_caps() {
    let mut s = session();
    let pile = run(&mut s, "pile_type_create", json!({"diameter": 0.25, "kind": "bored"}))["id"].clone();
    let deep = json!({"layout": "single", "edge_distance": 0.4, "thickness": 0.6, "pile_type": pile});
    let deep = run(&mut s, "pile_cap_type_create", deep)["id"].clone();
    let shallow = json!({"layout": "single", "edge_distance": 0.15, "thickness": 0.6, "pile_type": pile});
    let shallow = run(&mut s, "pile_cap_type_create", shallow)["id"].clone();
    let tie = |s: &mut Session, y: f64, ty: &Value| {
        for x in [0.0, 5.0] {
            run(s, "pile_cap_create", json!({"position": [x, y], "type": ty}));
        }
        id(&run(s, "beam_create", json!({"start": [0, y], "end": [5, y], "top_offset": 0.0, "type": "V20x50"})))
    };
    // 80 cm caps 60 cm thick under a beam from -0.5 to 0: 0.4 straight past the faces at 0.4 and 4.6.
    let b = tie(&mut s, 0.0, &deep);
    let get = run(&mut s, "element_get", json!({"id": b.0}));
    assert!(get["derived"]["start_join"]["pile_cap"].is_u64(), "{get}");
    let bottom = set(&mut s, b, "bottom", json!({"anchor": true}));
    assert!(all_span(&bars(&mut s, bottom), 0.0, 5.0, 5.0));
    // 30 cm caps leave 0.275 straight; a bottom bar hooked up gets the 0.125 left, one
    // hooked down only the 0.6 - 0.5 + 0.03 - 0.025 = 0.105 the cap has under it.
    let c = tie(&mut s, 10.0, &shallow);
    let up = set(&mut s, c, "bottom", json!({"anchor": true}));
    assert!(bars(&mut s, up).iter().all(|b| close(b.points[0].z - b.points[1].z, 0.125)));
    let down = set(&mut s, c, "bottom", json!({"anchor": true, "anchor_hook": "down"}));
    for bar in bars(&mut s, down) {
        assert!(close(bar.points[0].z - bar.points[1].z, -0.105), "{bar:?}");
        assert!(close(bar.length(), 4.7 + 2.0 * 0.275 + 2.0 * 0.105), "{}", bar.length());
    }
}

/// Values that are no anchorage, or on bars that take none, are refused;
/// `element_set` edits and removes it; a file's bad anchorage is reset on
/// load, keeping the set; older files load as they were.
#[test]
fn anchorages_are_checked_edited_and_repaired() {
    let mut s = session();
    let p60 = column_type(&mut s, 0.6);
    let col = column(&mut s, &p60, 0.0, 0.0, 0.0);
    column(&mut s, &p60, 6.0, 0.0, 0.0);
    let b = beam(&mut s, [0.0, 0.0], [6.0, 0.0]);
    for bad in [
        json!({"host": b.0, "layout": "bottom", "anchor": 5}),
        json!({"host": b.0, "layout": "bottom", "anchor": 500}),
        json!({"host": b.0, "layout": "bottom", "anchor": -40}),
        json!({"host": b.0, "layout": "bottom", "anchor": "yes"}),
        json!({"host": b.0, "layout": "bottom", "anchor": true, "anchor_hook": "sideways"}),
        json!({"host": b.0, "layout": "bottom", "anchor_hook": "down"}),
        json!({"host": b.0, "layout": "stirrups", "anchor": true}),
        json!({"host": col.0, "layout": "perimeter", "anchor": true}),
    ] {
        assert_eq!(fails(&mut s, "rebar_create", bad), ErrorCode::InvalidArguments);
    }
    assert_eq!(fails(&mut s, "rebar_typical", json!({"anchor": 500})), ErrorCode::InvalidArguments);
    // `false` is no anchorage, and nothing is written for it.
    let off = set(&mut s, b, "bottom", json!({"anchor": false}));
    assert!(run(&mut s, "element_get", json!({"id": off.0}))["params"].get("anchorage").is_none());

    // An edit sets, changes and removes it.
    let bottom = set(&mut s, b, "bottom", json!({}));
    let edit = json!({"id": bottom.0, "params": {"anchorage": {"length": 30, "hook": "none"}}});
    let got = run(&mut s, "element_set", edit);
    assert_eq!(got["params"]["anchorage"], json!({"length": 30.0, "hook": "none"}));
    assert!(all_span(&bars(&mut s, bottom), 0.0, 6.0, 6.0));
    run(&mut s, "element_set", json!({"id": bottom.0, "params": {"anchorage": {"length": 45}}}));
    assert!(all_span(&bars(&mut s, bottom), -0.15, 6.15, 6.3));
    let refused = [
        json!({"id": bottom.0, "params": {"anchorage": {"length": 500}}}),
        json!({"id": bottom.0, "params": {"anchorage": {"length": 0}}}),
    ];
    for bad in refused {
        assert_eq!(fails(&mut s, "element_set", bad), ErrorCode::Rejected);
    }
    let stirrups = set(&mut s, b, "stirrups", json!({}));
    let bad = json!({"id": stirrups.0, "params": {"anchorage": {"length": 40}}});
    assert_eq!(fails(&mut s, "element_set", bad), ErrorCode::Rejected);
    let bad = json!({"id": bottom.0, "params": {"anchorage": {"hook": "sideways"}}});
    assert_eq!(fails(&mut s, "element_set", bad), ErrorCode::InvalidArguments);
    run(&mut s, "element_set", json!({"id": bottom.0, "params": {"anchorage": null}}));
    assert!(all_span(&bars(&mut s, bottom), 0.325, 5.675, 5.35));
    run(&mut s, "undo", json!({}));
    assert!(all_span(&bars(&mut s, bottom), -0.15, 6.15, 6.3));

    // A file's anchorage out of range, or on a column's bars, is reset; the sets stay.
    let col_bars = set(&mut s, col, "perimeter", json!({}));
    let mut file = s.project().clone();
    for (set, length) in [(bottom, 500.0), (col_bars, 40.0), (off, f64::NAN)] {
        if let Some(ElementKind::Rebar(r)) = file.elements.get_mut(&set).map(|e| &mut e.kind) {
            r.anchorage = Some(Anchorage { length, hook: AnchorHook::Down });
        }
    }
    let text = cc_core::to_ron(&file).unwrap();
    let (loaded, repairs) = cc_core::from_ron_with_repairs(&text).unwrap();
    assert!(repairs >= 3, "{repairs}");
    for set in [bottom, col_bars, off] {
        let Some(ElementKind::Rebar(r)) = loaded.elements.get(&set).map(|e| &e.kind) else { panic!("{set:?} kept") };
        assert!(r.anchorage.is_none());
    }

    // A set without an anchorage is written without one and loads as it was.
    run(&mut s, "element_set", json!({"id": bottom.0, "params": {"anchorage": null}}));
    let text = cc_core::to_ron(s.project()).unwrap();
    assert!(!text.contains("anchorage"), "nothing written for none");
    let (loaded, repairs) = cc_core::from_ron_with_repairs(&text).unwrap();
    assert_eq!(repairs, 0);
    let mut s = Session::new(cc_core::Document::new(loaded));
    assert!(all_span(&bars(&mut s, bottom), 0.325, 5.675, 5.35));
}

/// The typical reinforcement anchors the beams' top and bottom bars when asked;
/// its columns are not affected.
#[test]
fn the_typical_reinforcement_takes_the_anchorage() {
    let mut s = session();
    let p60 = column_type(&mut s, 0.6);
    let c = column(&mut s, &p60, 0.0, 0.0, 0.0);
    column(&mut s, &p60, 6.0, 0.0, 0.0);
    let b = beam(&mut s, [0.0, 0.0], [6.0, 0.0]);
    run(&mut s, "rebar_typical", json!({"elements": [b.0, c.0], "anchor": 30, "anchor_hook": "none"}));
    let p = s.project();
    let anchored: Vec<_> = p.rebar_of(b).filter(|(_, r)| r.anchorage.is_some()).map(|(_, r)| r.anchorage).collect();
    assert_eq!(anchored, vec![Some(Anchorage { length: 30.0, hook: AnchorHook::None }); 2]);
    assert!(p.rebar_of(c).all(|(_, r)| r.anchorage.is_none()));
    // Without it, nothing changes.
    let d = beam(&mut s, [0.0, 5.0], [6.0, 5.0]);
    run(&mut s, "rebar_typical", json!({"elements": [d.0]}));
    assert!(s.project().rebar_of(d).all(|(_, r)| r.anchorage.is_none()));
}
