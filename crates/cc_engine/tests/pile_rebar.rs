//! Reinforcement of pile caps and piles (#343): a three-pile cap and its
//! piles reinforced in two commands, their bars in the bending schedule with
//! marks and exact lengths; the bottom bars of caps of two, four and a line
//! of piles, the mesh of single, round and ring caps; pile bars bounded by
//! the cap, the spiral's exact length; sets that follow a type's layout
//! change and go with their host; refused and repaired values, older files,
//! the IFC round trip and the schedule's host.

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_core::{ElementId, ElementKind, RebarLayout};
use cc_engine::{ErrorCode, Session};
use serde_json::{Value, json};
use std::f64::consts::{PI, TAU};

fn run(s: &mut Session, name: &str, args: Value) -> Value {
    s.execute(name, args).unwrap_or_else(|e| panic!("{name}: {e:?}"))
}

fn fails(s: &mut Session, name: &str, args: Value) -> ErrorCode {
    s.execute(name, args).map(|v| panic!("{name} should fail, gave {v}")).unwrap_err().code
}

fn id(v: &Value) -> u64 {
    v["id"].as_u64().unwrap_or_else(|| panic!("no id in {v}"))
}

fn created(v: &Value) -> Vec<u64> {
    v["created"].as_array().unwrap_or_else(|| panic!("no created in {v}")).iter().map(|x| x.as_u64().unwrap()).collect()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * b.abs().max(1.0)
}

/// Bar diameters of the template's types used here, meters.
const D10: f64 = 0.010;
const D12: f64 = 0.0125;
const D8: f64 = 0.008;
const D5: f64 = 0.005;
const BAR_12: &str = "Barra Ø 12,5 mm";

/// Caps: piles 40 cm round, 1.2 m apart, 0.4 m from the edge, 0.8 m deep,
/// their heads 0.1 m into the cap.
const SPACING: f64 = 1.2;
const EDGE: f64 = 0.4;
const DEPTH: f64 = 0.8;
const EMBED: f64 = 0.1;
const PILE: f64 = 0.4;

/// A session with the 40 cm pile type.
fn session() -> Session {
    let mut s = Session::default();
    run(&mut s, "pile_type_create", json!({"diameter": PILE, "kind": "bored"}));
    s
}

/// A cap of a new type of `layout` (with its extra values) at `at`.
fn cap(s: &mut Session, layout: Value, at: [f64; 2]) -> (u64, u64) {
    let mut args = layout;
    args["spacing"] = json!(SPACING);
    args["edge_distance"] = json!(EDGE);
    args["thickness"] = json!(DEPTH);
    args["embedment"] = json!(EMBED);
    let ty = id(&run(s, "pile_cap_type_create", args));
    (id(&run(s, "pile_cap_create", json!({"position": at, "type": ty}))), ty)
}

/// The bottom and top elevations of cap `cap`.
fn z_of(s: &mut Session, cap: u64) -> (f64, f64) {
    s.geometry().foundations[&ElementId(cap)].z_range
}

/// The set of `host` whose layout `pick` takes.
fn set_of(s: &Session, host: u64, pick: fn(&RebarLayout) -> bool) -> u64 {
    let found: Vec<u64> =
        s.project().rebar_of(ElementId(host)).filter(|(_, r)| pick(&r.layout)).map(|(e, _)| e.id.0).collect();
    assert_eq!(found.len(), 1, "one set of host {host}");
    found[0]
}

fn is_bottom(l: &RebarLayout) -> bool {
    matches!(l, RebarLayout::CapBottom { .. })
}

fn is_top(l: &RebarLayout) -> bool {
    matches!(l, RebarLayout::CapTop { .. })
}

fn is_sides(l: &RebarLayout) -> bool {
    matches!(l, RebarLayout::CapSides { .. })
}

fn is_stirrups(l: &RebarLayout) -> bool {
    matches!(l, RebarLayout::Stirrups { .. })
}

fn is_long(l: &RebarLayout) -> bool {
    matches!(l, RebarLayout::PileLongitudinal { .. })
}

fn is_spiral(l: &RebarLayout) -> bool {
    matches!(l, RebarLayout::PileSpiral { .. })
}

fn is_ties(l: &RebarLayout) -> bool {
    matches!(l, RebarLayout::PileTies { .. })
}

/// The center lines of a set's bars.
fn points(s: &mut Session, set: u64) -> Vec<Vec<glam::DVec3>> {
    s.geometry().rebar[&ElementId(set)].bars.iter().map(|b| b.points.clone()).collect()
}

/// The bars of a set: each one's length and lowest point, sorted by length.
fn bars(s: &mut Session, set: u64) -> Vec<(f64, f64)> {
    let g = s.geometry();
    let mut out: Vec<(f64, f64)> = g.rebar[&ElementId(set)]
        .bars
        .iter()
        .map(|b| (b.length(), b.points.iter().map(|p| p.z).fold(f64::INFINITY, f64::min)))
        .collect();
    out.sort_by(|a, b| a.0.total_cmp(&b.0));
    out
}

/// The lengths of a set's bars, sorted.
fn lengths(s: &mut Session, set: u64) -> Vec<f64> {
    bars(s, set).into_iter().map(|(l, _)| l).collect()
}

/// The two hooks that close a loop of bars `d` thick: 135° around a pin of
/// radius 2d, then 10 diameters straight (at least 5 cm).
fn hooks(d: f64) -> f64 {
    2.0 * (0.75 * PI * 2.0 * d + (10.0 * d).max(0.05))
}

/// The piles of `cap` in layout order.
fn piles(s: &Session, cap: u64) -> Vec<u64> {
    let mut out: Vec<(u32, u64)> =
        s.project().piles_of(ElementId(cap)).map(|(e, p)| (p.cap.unwrap().index, e.id.0)).collect();
    out.sort();
    out.into_iter().map(|(_, id)| id).collect()
}

/// The done-when test of #343: a three-pile cap and its piles reinforced in
/// two commands, each one undo step, and the bending schedule lists their
/// bars with marks and lengths.
#[test]
fn a_three_pile_cap_and_its_piles_in_a_few_commands() {
    let mut s = session();
    let (cap, _) = cap(&mut s, json!({"layout": "triangle"}), [0.0, 0.0]);
    let (bottom, top) = z_of(&mut s, cap);
    assert!(close(top - bottom, DEPTH));
    let depth = s.doc.undo_depth();
    let out = run(
        &mut s,
        "pile_cap_rebar_create",
        json!({
            "caps": [cap], "type": BAR_12, "cover": 0.05, "bottom": {"count": 4, "bend": 0.3},
            "sides": {"spacing": 0.2}, "max_length": 0, "partition": "BLOCOS"
        }),
    );
    assert_eq!(s.doc.undo_depth(), depth + 1, "one undo step");
    // Bottom, top and side bars; a cap of three piles takes no stirrups.
    assert_eq!(created(&out).len(), 3, "{out}");
    assert_eq!(out["caps"], json!(1));
    assert_eq!(out["skipped"], json!([]));

    // The bottom bars: a band of 4 over each two piles, along the sides of the
    // triangle, as wide as a pile: offsets o = ±(0.2 - d/2) and a third of it.
    // They lie inside the side loops (8 mm): inset c = 0.05 + 0.008 + d/2. The
    // line through two piles is parallel to a side 0.4 off it; the hexagon's
    // corners are cut square to the pile directions, R + e from the center,
    // so the run between the cuts is 1.2 + (4/√3)(e - c - |o|/2). Legs of 0.3
    // turn up at both ends (room: the 0.8 m cap less the embedment, the cover
    // and the bar).
    let set = set_of(&s, cap, is_bottom);
    let c = 0.05 + D8 + D12 / 2.0;
    let half = PILE / 2.0 - D12 / 2.0;
    let run_of = |o: f64| SPACING + 4.0 / 3f64.sqrt() * (EDGE - c - o / 2.0) + 0.6;
    let (short, long) = (run_of(half), run_of(half / 3.0));
    let b = bars(&mut s, set);
    assert_eq!(b.len(), 12);
    for (i, (length, _)) in b.iter().enumerate() {
        let want = if i < 6 { short } else { long };
        assert!(close(*length, want), "bar {i}: {length} vs {want}");
    }
    // The bands lie on the piles' heads, one on the other: 0.1 above the bottom plus half a bar.
    let mut lows: Vec<f64> = b.iter().map(|(_, z)| *z).collect();
    lows.sort_by(f64::total_cmp);
    lows.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    let layer = |k: f64| bottom + EMBED + D12 / 2.0 + k * D12;
    assert_eq!(lows.len(), 3);
    for (k, z) in lows.iter().enumerate() {
        assert!(close(*z, layer(k as f64)), "{z} vs {}", layer(k as f64));
    }

    // Side loops at most 0.2 apart between the bottom and top bars' first
    // layers (0.8 - 2 × 0.054 - 0.05 = 0.642 m: three loops), around the
    // hexagon 0.054 inside its faces: perimeter 2√3 (b + s - 2c), the sides
    // b = s/√3 + e and s/(2√3) + e from the center; plus the hooks.
    let set = set_of(&s, cap, is_sides);
    let c = 0.05 + D8 / 2.0;
    let perimeter = 2.0 * 3f64.sqrt() * (SPACING * 3f64.sqrt() / 2.0 + 2.0 * EDGE - 2.0 * c);
    let loops = lengths(&mut s, set);
    assert_eq!(loops.len(), 3);
    assert!(loops.iter().all(|l| close(*l, perimeter + hooks(D8))), "{loops:?} vs {}", perimeter + hooks(D8));

    // The top mesh: both ways under the top, the first direction highest.
    let set = set_of(&s, cap, is_top);
    let highest = s.geometry().rebar[&ElementId(set)]
        .bars
        .iter()
        .flat_map(|b| b.points.iter().map(|p| p.z))
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(close(highest, top - 0.05 - D8 / 2.0), "{highest}");

    // The piles: 6 bars from 6 m below the head to 40 diameters above it (in
    // the cap, below its top less the cover), and a spiral of 10 cm pitch.
    let depth = s.doc.undo_depth();
    let out = run(
        &mut s,
        "pile_rebar_create",
        json!({"caps": [cap], "count": 6, "length": 6.0, "transverse": "spiral", "pitch": 0.1, "cover": 0.05,
               "max_length": 0, "partition": "ESTACAS"}),
    );
    assert_eq!(s.doc.undo_depth(), depth + 1, "one undo step");
    assert_eq!(created(&out).len(), 6, "{out}");
    assert_eq!(out["piles"], json!(3));
    let head = bottom + EMBED;
    for pile in piles(&s, cap) {
        let long = set_of(&s, pile, is_long);
        let b = bars(&mut s, long);
        assert_eq!(b.len(), 6);
        assert!(b.iter().all(|(l, z)| close(*l, 6.0 + 0.4) && close(*z, head - 6.0)), "{b:?}");
        // The helix from 5 cm below the head down to 6 m below it: 59.5 turns
        // of radius 0.2 - 0.05 - d/2, each sqrt((2πr)² + pitch²) long.
        let spiral = set_of(&s, pile, is_spiral);
        let r = PILE / 2.0 - 0.05 - D5 / 2.0;
        let want = 59.5 * (TAU * r).hypot(0.1);
        let l = lengths(&mut s, spiral);
        assert!(l.len() == 1 && close(l[0], want), "{l:?} vs {want}");
    }

    // The bending schedule: the marks with their lengths and counts.
    let marks = run(&mut s, "bar_marks_list", json!({}))["marks"].as_array().unwrap().clone();
    let find = |d: f64, length: f64| {
        marks
            .iter()
            .find(|m| {
                close(m["diameter"].as_f64().unwrap(), d) && (m["unit_length"].as_f64().unwrap() - length).abs() < 1e-6
            })
            .unwrap_or_else(|| panic!("a mark of {d} m bars {length} m long in {marks:?}"))
    };
    for length in [short, long] {
        let m = find(D12, length);
        assert_eq!(m["count"], json!(6));
        assert!(m["code"].as_str().unwrap().starts_with("U 30-"), "{m}");
    }
    let m = find(D10, 6.4);
    assert_eq!((m["count"].clone(), m["code"].clone()), (json!(18), json!("straight 640")));
    let m = find(D5, 59.5 * (TAU * (PILE / 2.0 - 0.05 - D5 / 2.0)).hypot(0.1));
    assert_eq!(m["count"], json!(3));
    assert!(m["code"].as_str().unwrap().starts_with("spiral R=15 p=10 L="), "{m}");
    let m = find(D8, perimeter + hooks(D8));
    assert_eq!(m["count"], json!(3));
    // Every mark has a name, and the schedule has a row for each.
    assert!(marks.iter().all(|m| !m["mark"].as_str().unwrap().is_empty()));
    let view = id(&run(&mut s, "bending_schedule_create", json!({})));
    let table = run(&mut s, "schedule_table", json!({"view": view}));
    let rows = table["rows"].as_array().unwrap();
    assert_eq!(rows.len(), marks.len());
    let codes: Vec<&str> = rows.iter().map(|r| r["cells"][3].as_str().unwrap()).collect();
    assert!(codes.iter().any(|c| c.starts_with("spiral ")) && codes.contains(&"straight 640"), "{codes:?}");

    // A rebar schedule's host column names each set's host.
    let (p, g) = s.project_and_geometry();
    for (e, r) in p.elements.values().filter_map(|e| match &e.kind {
        ElementKind::Rebar(r) => Some((e, r)),
        _ => None,
    }) {
        let host = p.element(r.host).unwrap();
        let name = if host.mark.is_empty() { host.id.to_string() } else { host.mark.clone() };
        let value = cc_views::field_value(p, g, e, cc_core::ScheduleField::Host).map(|(text, _)| text);
        assert_eq!(value.as_deref(), Some(name.as_str()));
    }
}

/// A cap of two piles is a beam over them: bars along the piles across its
/// width, stirrups along it and side loops.
#[test]
fn a_two_pile_cap_has_bars_along_its_piles_and_stirrups() {
    let mut s = session();
    let (cap, _) = cap(&mut s, json!({"layout": "pair"}), [3.0, 1.0]);
    let (bottom, top) = z_of(&mut s, cap);
    let out = run(
        &mut s,
        "pile_cap_rebar_create",
        json!({"caps": [cap], "type": BAR_12, "cover": 0.05, "top": false, "sides": {"spacing": 0.2}, "max_length": 0}),
    );
    assert_eq!(created(&out).len(), 3, "bottom, sides and stirrups: {out}");
    // The cap is 2.0 × 0.8. Bottom bars inside the stirrups (5 mm) and the
    // side loops (8 mm): 4 bars over 2(0.4 - c) across, each 2(1.0 - c) long
    // with legs of 20 diameters (0.25) at both ends.
    let c = 0.05 + D5 + D8 + D12 / 2.0;
    let b = {
        let set = set_of(&s, cap, is_bottom);
        bars(&mut s, set)
    };
    assert_eq!(b.len(), 4);
    assert!(b.iter().all(|(l, _)| close(*l, 2.0 * (1.0 - c) + 0.5)), "{b:?}");
    assert!(b.iter().all(|(_, z)| close(*z, bottom + EMBED + D5 + D12 / 2.0)));
    // Stirrups every 0.15 over 2(1.0 - 0.0525) = 1.895: 13. Each 2(0.695 +
    // 0.645) around: 0.4 - 0.0525 either side, from the heads (0.1 above the
    // bottom) to the cover below the top, less half a bar each; and its hooks.
    let st = {
        let set = set_of(&s, cap, is_stirrups);
        lengths(&mut s, set)
    };
    let want = 2.0 * (2.0 * (EDGE - 0.0525) + (top - 0.05 - bottom - EMBED - D5)) + hooks(D5);
    assert_eq!(st.len(), 13);
    assert!(st.iter().all(|l| close(*l, want)), "{st:?} vs {want}");
    // Side loops 0.059 inside the faces (inside the stirrups), 1.882 × 0.682,
    // at most 0.2 apart over 0.632 m: three.
    let loops = {
        let set = set_of(&s, cap, is_sides);
        lengths(&mut s, set)
    };
    assert_eq!(loops.len(), 3);
    assert!(loops.iter().all(|l| close(*l, 2.0 * (1.882 + 0.682) + hooks(D8))), "{loops:?}");
}

/// A cap of four piles: a band along each side of the square and, asked
/// for, across each diagonal, in four layers.
#[test]
fn a_four_pile_cap_has_bands_along_its_sides_and_diagonals() {
    let mut s = session();
    let (cap, _) = cap(&mut s, json!({"layout": "square"}), [0.0, 0.0]);
    let (bottom, _) = z_of(&mut s, cap);
    let args = json!({"caps": [cap], "type": BAR_12, "cover": 0.05, "bottom": {"count": 3, "diagonals": true},
                      "top": false, "sides": false, "max_length": 0});
    run(&mut s, "pile_cap_rebar_create", args);
    // The square is ±1.0; the bars stand 0.05625 inside it (a = 0.94375).
    // Sides: 4 bands of 3, each 2a long; diagonals: 2 bands of 3, the middle
    // bar 2√2 a long, the others 2|o| shorter (o = 0.2 - d/2); legs of 0.25.
    let a = 1.0 - 0.05 - D12 / 2.0;
    let o = PILE / 2.0 - D12 / 2.0;
    let mut want = vec![2.0 * a; 12];
    want.extend([2.0 * 2f64.sqrt() * a; 2]);
    want.extend([2.0 * 2f64.sqrt() * a - 2.0 * o; 4]);
    let mut want: Vec<f64> = want.into_iter().map(|l| l + 0.5).collect();
    want.sort_by(f64::total_cmp);
    let b = {
        let set = set_of(&s, cap, is_bottom);
        bars(&mut s, set)
    };
    assert_eq!(b.len(), 18);
    for ((l, _), w) in b.iter().zip(&want) {
        assert!(close(*l, *w), "{l} vs {w}");
    }
    // Four layers: along the cap, across it, and the two diagonals.
    let mut lows: Vec<f64> = b.iter().map(|(_, z)| *z).collect();
    lows.sort_by(f64::total_cmp);
    lows.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    assert_eq!(lows.len(), 4);
    assert!(close(lows[3], bottom + EMBED + D12 / 2.0 + 3.0 * D12));
}

/// Caps with no conventional bands take a mesh: a single pile's, a round
/// cap's (bars ending on the circle) and a ring's (bars on either side of
/// its hole); a line of piles is a beam, like two.
#[test]
fn single_round_ring_and_line_caps() {
    let mut s = session();
    let only_bottom = |cap: u64, spacing: f64| {
        json!({"caps": [cap], "type": BAR_12, "cover": 0.05, "bottom": {"spacing": spacing, "bend": 0},
               "top": false, "sides": false, "stirrups": false, "max_length": 0})
    };
    // One pile: a 0.8 square, bars 0.05625 inside it (a = 0.34375): lines
    // 0.1 apart over 2a = 0.6875, floor(6.875) + 1 = 7 each way, each 2a.
    let (single, _) = cap(&mut s, json!({"layout": "single"}), [0.0, 0.0]);
    run(&mut s, "pile_cap_rebar_create", only_bottom(single, 0.1));
    let a = EDGE - 0.05 - D12 / 2.0;
    let l = {
        let set = set_of(&s, single, is_bottom);
        lengths(&mut s, set)
    };
    assert!(l.len() == 14 && l.iter().all(|x| close(*x, 2.0 * a)), "{l:?}");
    // A round cap of radius 1.4: chords 2√(R² - o²) of R = 1.4 - 0.05625 at
    // the stations 0.2 apart: 14 each way, from -1.3 to 1.3.
    let (round, _) = cap(&mut s, json!({"layout": "circle", "count": 6, "radius": 1.0}), [10.0, 0.0]);
    run(&mut s, "pile_cap_rebar_create", only_bottom(round, 0.2));
    let r = 1.0 + EDGE - 0.05 - D12 / 2.0;
    let offsets: Vec<f64> = (0..14).map(|i| -1.3 + 0.2 * f64::from(i)).collect();
    let total: f64 = 2.0 * offsets.iter().map(|o| 2.0 * (r * r - o * o).sqrt()).sum::<f64>();
    let l = {
        let set = set_of(&s, round, is_bottom);
        lengths(&mut s, set)
    };
    assert_eq!(l.len(), 28);
    assert!(close(l.iter().sum(), total), "{} vs {total}", l.iter().sum::<f64>());
    // A ring 1.6 to 2.4: lines 0.3 apart at ±0.15 … ±2.25; those that cross
    // the hole (|o| < 1.65625) in two pieces, the others whole. Side loops
    // round both faces, two heights of them.
    let (ring, _) = cap(&mut s, json!({"layout": "circle", "count": 8, "radius": 2.0, "ring_width": 0.8}), [0.0, 10.0]);
    let mut args = only_bottom(ring, 0.3);
    args["sides"] = json!({"spacing": 0.3});
    run(&mut s, "pile_cap_rebar_create", args);
    let c = 0.05 + D8 + D12 / 2.0;
    let (ri, ro) = (1.6 + c, 2.4 - c);
    let (mut count, mut total) = (0, 0.0);
    for i in 0..16 {
        let o: f64 = -2.25 + 0.3 * f64::from(i);
        if o.abs() < ri {
            count += 2;
            total += 2.0 * ((ro * ro - o * o).sqrt() - (ri * ri - o * o).sqrt());
        } else {
            count += 1;
            total += 2.0 * (ro * ro - o * o).sqrt();
        }
    }
    let l = {
        let set = set_of(&s, ring, is_bottom);
        lengths(&mut s, set)
    };
    assert_eq!(l.len(), 2 * count);
    assert!(close(l.iter().sum(), 2.0 * total), "{} vs {}", l.iter().sum::<f64>(), 2.0 * total);
    let c = 0.05 + D8 / 2.0;
    let loops = {
        let set = set_of(&s, ring, is_sides);
        lengths(&mut s, set)
    };
    let want = [TAU * (1.6 + c) + hooks(D8), TAU * (2.4 - c) + hooks(D8)];
    assert_eq!(loops.len(), 4);
    assert!(close(loops[0], want[0]) && close(loops[1], want[0]) && close(loops[3], want[1]), "{loops:?}");
    // A line of three piles: bars along it, and stirrups.
    let (line, _) = cap(&mut s, json!({"layout": "line", "count": 3}), [0.0, -10.0]);
    let out =
        run(&mut s, "pile_cap_rebar_create", json!({"caps": [line], "top": false, "sides": false, "cover": 0.05}));
    assert_eq!(created(&out).len(), 2);
    let c = 0.05 + D5 + D10 / 2.0;
    let l = {
        let set = set_of(&s, line, is_bottom);
        lengths(&mut s, set)
    };
    // 2 × 1.2 + 2 × 0.4 long, less the inset, with legs of 20 diameters (0.2).
    assert!(l.len() == 4 && l.iter().all(|x| close(*x, 2.0 * (1.6 - c) + 0.4)), "{l:?}");
    assert!(
        !{
            let set = set_of(&s, line, is_stirrups);
            lengths(&mut s, set)
        }
        .is_empty()
    );
}

/// Pile bars project into the cap at most to its top less the cover; a free
/// pile's as far as asked. Ties close round the bars, a square pile's at
/// its corners.
#[test]
fn pile_bars_reach_into_the_cap_at_most_to_its_top() {
    let mut s = session();
    let (cap, _) = cap(&mut s, json!({"layout": "pair"}), [0.0, 0.0]);
    let (_, top) = z_of(&mut s, cap);
    let out = run(
        &mut s,
        "pile_rebar_create",
        json!({"caps": [cap], "projection": 2.0, "length": 3.0, "transverse": "ties", "cover": 0.05, "max_length": 0}),
    );
    assert_eq!(created(&out).len(), 4);
    for pile in piles(&s, cap) {
        // From 3 m below the head up to the cap's top less the cover: 3 + 0.65.
        let b = {
            let set = set_of(&s, pile, is_long);
            bars(&mut s, set)
        };
        assert_eq!(b.len(), 6, "a round pile's six");
        let set = set_of(&s, pile, is_long);
        let highest = points(&mut s, set).iter().flatten().map(|p| p.z).fold(f64::NEG_INFINITY, f64::max);
        assert!(close(highest, top - 0.05) && b.iter().all(|(l, _)| close(*l, 3.65)), "{b:?}");
        // Ties every 0.15 from 3 m below the head to 5 cm below it, round
        // ones 2π(0.2 - 0.05 - d/2) and their hooks.
        let ties = set_of(&s, pile, is_ties);
        let l = lengths(&mut s, ties);
        let want = TAU * (PILE / 2.0 - 0.05 - D5 / 2.0) + hooks(D5);
        assert_eq!(l.len(), (2.95f64 / 0.15).floor() as usize + 1);
        assert!(l.iter().all(|x| close(*x, want)), "{l:?} vs {want}");
    }
    // A free square pile, 5 m long: whole, from the cover above its toe to 2 m above its head.
    let square = id(&run(&mut s, "pile_type_create", json!({"side": 0.3, "kind": "precast"})));
    let free = id(&run(&mut s, "pile_create", json!({"position": [20, 0], "type": square, "length": 5.0})));
    let out =
        run(&mut s, "pile_rebar_create", json!({"piles": [free], "projection": 2.0, "cover": 0.05, "max_length": 0}));
    assert_eq!(created(&out).len(), 2, "bars and ties (a square pile): {out}");
    let b = {
        let set = set_of(&s, free, is_long);
        bars(&mut s, set)
    };
    assert!(b.len() == 4 && b.iter().all(|(l, _)| close(*l, 4.95 + 2.0)), "{b:?}");
    // Its four bars at the corners, inside the ties: 0.15 - 0.05 - 0.005 - 0.005 off the axes.
    let set = set_of(&s, free, is_long);
    let corner = 0.15 - 0.05 - D5 - D10 / 2.0;
    for bar in points(&mut s, set) {
        let p = bar[0].truncate() - glam::DVec2::new(20.0, 0.0);
        assert!(close(p.x.abs(), corner) && close(p.y.abs(), corner), "{p}");
    }
}

/// A cap whose type's layout changes keeps its sets, laid out for the new
/// layout; its piles keep theirs, a pile it gains takes the others' and a
/// pile it loses takes its own along, in the type's one undo step.
#[test]
fn a_layout_change_relays_the_sets_and_keeps_them() {
    let mut s = session();
    let (cap, ty) = cap(&mut s, json!({"layout": "triangle"}), [0.0, 0.0]);
    run(&mut s, "pile_cap_rebar_create", json!({"caps": [cap], "max_length": 0}));
    run(&mut s, "pile_rebar_create", json!({"caps": [cap], "max_length": 0}));
    let cap_sets: Vec<u64> = s.project().rebar_of(ElementId(cap)).map(|(e, _)| e.id.0).collect();
    let before = piles(&s, cap);
    let pile_sets: Vec<Vec<u64>> =
        before.iter().map(|p| s.project().rebar_of(ElementId(*p)).map(|(e, _)| e.id.0).collect()).collect();
    assert_eq!(
        {
            let set = set_of(&s, cap, is_bottom);
            bars(&mut s, set)
        }
        .len(),
        12
    );
    let depth = s.doc.undo_depth();
    run(&mut s, "type_set", json!({"type": ty, "params": {"pile_cap": {"layout": "Square"}}}));
    assert_eq!(s.doc.undo_depth(), depth + 1, "one undo step");
    // The same sets, the bottom bars now in four bands of four.
    assert_eq!(s.project().rebar_of(ElementId(cap)).map(|(e, _)| e.id.0).collect::<Vec<_>>(), cap_sets);
    assert_eq!(
        {
            let set = set_of(&s, cap, is_bottom);
            bars(&mut s, set)
        }
        .len(),
        16
    );
    let now = piles(&s, cap);
    assert_eq!(now.len(), 4);
    assert_eq!(&now[..3], &before[..]);
    for (pile, sets) in now.iter().zip(&pile_sets) {
        assert_eq!(&s.project().rebar_of(ElementId(*pile)).map(|(e, _)| e.id.0).collect::<Vec<_>>(), sets);
    }
    // The new pile's sets: copies of the first's, laid out in it.
    let layouts = |pile: u64| -> Vec<RebarLayout> {
        s.project().rebar_of(ElementId(pile)).map(|(_, r)| r.layout.clone()).collect()
    };
    assert_eq!(layouts(now[3]), layouts(now[0]));
    assert_eq!(
        {
            let set = set_of(&s, now[3], is_long);
            bars(&mut s, set)
        }
        .len(),
        6
    );
    // Two piles: the bottom bars along them; the two piles lost take their sets along.
    run(&mut s, "type_set", json!({"type": ty, "params": {"pile_cap": {"layout": "Pair"}}}));
    assert_eq!(piles(&s, cap), now[..2].to_vec());
    for gone in &now[2..] {
        assert!(s.project().element(ElementId(*gone)).is_none());
    }
    let count = s.project().elements.values().filter(|e| matches!(e.kind, ElementKind::Rebar(_))).count();
    assert_eq!(count, 3 + 2 * 2);
    assert_eq!(
        {
            let set = set_of(&s, cap, is_bottom);
            bars(&mut s, set)
        }
        .len(),
        4
    );
    // And back.
    run(&mut s, "undo", json!({}));
    run(&mut s, "undo", json!({}));
    assert_eq!(piles(&s, cap), before);
    assert_eq!(
        {
            let set = set_of(&s, cap, is_bottom);
            bars(&mut s, set)
        }
        .len(),
        12
    );
}

/// A mirrored cap keeps its sets, its bars the mirror image of what they were.
#[test]
fn a_mirrored_cap_keeps_its_bars() {
    let mut s = session();
    let (cap, _) = cap(&mut s, json!({"layout": "triangle"}), [1.0, 2.0]);
    run(&mut s, "pile_cap_rebar_create", json!({"caps": [cap], "max_length": 0}));
    run(&mut s, "pile_rebar_create", json!({"caps": [cap], "max_length": 0}));
    let all = |s: &mut Session| -> Vec<(u64, Vec<glam::DVec3>)> {
        let sets: Vec<u64> =
            s.project().elements.values().filter(|e| matches!(e.kind, ElementKind::Rebar(_))).map(|e| e.id.0).collect();
        sets.into_iter().map(|id| (id, points(s, id).concat())).collect()
    };
    let before = all(&mut s);
    let totals = |s: &mut Session, sets: &[(u64, Vec<glam::DVec3>)]| -> Vec<f64> {
        sets.iter().map(|(id, _)| lengths(s, *id).iter().sum()).collect()
    };
    let lengths_before = totals(&mut s, &before);
    // A mirrored copy is a new cap with piles of its own and, as any host's
    // copy, without the original's sets (rebar_copy copies them).
    let copies = run(&mut s, "elements_mirror", json!({"ids": [cap], "axis": [[5, 0], [5, 1]]}));
    let copy = copies["created"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap())
        .find(|c| s.project().is_pile_cap(ElementId(*c)));
    let copy = copy.unwrap();
    assert_eq!(piles(&s, copy).len(), 3);
    assert_eq!(s.project().rebar_of(ElementId(copy)).count(), 0);
    assert_eq!(all(&mut s).len(), before.len());
    // Mirrored about x = 5 in place: the same sets, as long, each in the mirror image of its box.
    run(&mut s, "elements_mirror", json!({"ids": [cap], "axis": [[5, 0], [5, 1]], "copy": false}));
    let after = all(&mut s);
    assert_eq!(
        after.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
        before.iter().map(|(id, _)| *id).collect::<Vec<_>>()
    );
    let lengths_after = totals(&mut s, &after);
    let bounds = |points: &[glam::DVec3]| {
        points.iter().fold((glam::DVec3::INFINITY, glam::DVec3::NEG_INFINITY), |(lo, hi), p| (lo.min(*p), hi.max(*p)))
    };
    // The piles stand where the mirrored layout puts them by their index, so a
    // pile's sets may be another pile's mirror image: each mirrored box is one set's.
    let boxes: Vec<(glam::DVec3, glam::DVec3)> = after.iter().map(|(_, b)| bounds(b)).collect();
    for (i, (_, a)) in before.iter().enumerate() {
        assert!(close(lengths_before[i], lengths_after[i]));
        let (lo, hi) = bounds(a);
        let want = (glam::DVec3::new(10.0 - hi.x, lo.y, lo.z), glam::DVec3::new(10.0 - lo.x, hi.y, hi.z));
        let found = boxes.iter().any(|(lo2, hi2)| lo2.distance(want.0) < 1e-6 && hi2.distance(want.1) < 1e-6);
        assert!(found, "no set in {want:?}");
    }
}

/// Deleting a cap removes its sets and its piles' sets; deleting a free pile its own.
#[test]
fn deleting_a_cap_or_a_pile_removes_its_sets() {
    let mut s = session();
    let (other, _) = cap(&mut s, json!({"layout": "pair"}), [20.0, 0.0]);
    let (cap, _) = cap(&mut s, json!({"layout": "triangle"}), [0.0, 0.0]);
    let other_sets = created(&run(&mut s, "pile_cap_rebar_create", json!({"caps": [other]}))).len()
        + created(&run(&mut s, "pile_rebar_create", json!({"caps": [other]}))).len();
    let free = id(&run(&mut s, "pile_create", json!({"position": [9, 0], "length": 6.0})));
    run(&mut s, "pile_cap_rebar_create", json!({}));
    run(&mut s, "pile_rebar_create", json!({}));
    let rebar = |s: &Session| {
        s.project().elements.values().filter(|e| matches!(e.kind, ElementKind::Rebar(_))).count() - other_sets
    };
    // Three cap sets, two per pile (bars and a spiral), on the four piles.
    assert_eq!(rebar(&s), 3 + 4 * 2);
    // Run again over the project: everything is reinforced already.
    let again = run(&mut s, "pile_cap_rebar_create", json!({}));
    assert_eq!((created(&again).len(), again["skipped"][0]["reason"].clone()), (0, json!("already reinforced")));
    let again = run(&mut s, "pile_rebar_create", json!({}));
    assert_eq!(again["skipped"].as_array().unwrap().len(), 6);
    run(&mut s, "elements_delete", json!({"ids": [cap]}));
    assert_eq!(rebar(&s), 2);
    run(&mut s, "elements_delete", json!({"ids": [free]}));
    assert_eq!(rebar(&s), 0);
    run(&mut s, "undo", json!({}));
    run(&mut s, "undo", json!({}));
    assert_eq!(rebar(&s), 11);
    // A copy of a cap's sets to a cap of the same type, and of a pile's to another pile.
    let ty = s.project().element(ElementId(cap)).unwrap().type_id.0;
    let second = id(&run(&mut s, "pile_cap_create", json!({"position": [30, 0], "type": ty})));
    let out = run(&mut s, "rebar_copy", json!({"from": cap, "to": [second]}));
    assert_eq!(created(&out).len(), 3);
    assert_eq!(fails(&mut s, "rebar_copy", json!({"from": cap, "to": [other]})), ErrorCode::InvalidArguments);
    let (from, to) = (piles(&s, cap)[0], piles(&s, second)[0]);
    let out = run(&mut s, "rebar_copy", json!({"from": from, "to": [to]}));
    assert_eq!(created(&out).len(), 2);
}

/// Values out of range are refused; a crafted file loses the sets that its
/// hosts do not take, and the values that are not usable.
#[test]
fn hostile_values_are_refused_and_repaired() {
    let mut s = session();
    let (cap, _) = cap(&mut s, json!({"layout": "triangle"}), [0.0, 0.0]);
    let column = id(&run(&mut s, "column_create", json!({"position": [8, 0]})));
    let bad = [
        json!({"caps": [column]}),
        json!({"caps": [cap], "bottom": {"count": 0}}),
        json!({"caps": [cap], "bottom": {"count": 1000}}),
        json!({"caps": [cap], "bottom": {"spacing": 0.001}}),
        json!({"caps": [cap], "bottom": {"bend": -1}}),
        json!({"caps": [cap], "bottom": {"bend": 50}}),
        json!({"caps": [cap], "bottom": {"zigzag": 1}}),
        json!({"caps": [cap], "top": {"spacing": -1}}),
        json!({"caps": [cap], "sides": {"hook": 45}}),
        json!({"caps": [cap], "cover": "far"}),
    ];
    for args in bad {
        assert_eq!(fails(&mut s, "pile_cap_rebar_create", args.clone()), ErrorCode::InvalidArguments, "{args}");
    }
    let pile = piles(&s, cap)[0];
    let bad = [
        json!({"piles": [cap]}),
        json!({"caps": [pile]}),
        json!({"piles": [pile], "count": 1000}),
        json!({"piles": [pile], "spacing": 0.001}),
        json!({"piles": [pile], "projection": 50}),
        json!({"piles": [pile], "length": -2}),
        json!({"piles": [pile], "transverse": "zigzag"}),
        json!({"piles": [pile], "transverse": "spiral", "hook": 90}),
        json!({"piles": [pile], "pitch": 5}),
    ];
    for args in bad {
        assert_eq!(fails(&mut s, "pile_rebar_create", args.clone()), ErrorCode::InvalidArguments, "{args}");
    }
    // A cap the cover leaves no room in is skipped, not refused.
    let out = run(&mut s, "pile_cap_rebar_create", json!({"caps": [cap], "cover": 0.5}));
    assert_eq!(created(&out).len(), 0);
    assert_eq!(out["skipped"][0]["cap"], json!(cap));
    // element_set refuses what repair would undo.
    let sets = created(&run(&mut s, "pile_cap_rebar_create", json!({"caps": [cap]})));
    let set = sets[0];
    let bad = json!({"id": set, "params": {"layout": {"CapBottom": {"count": 0, "spacing": 0.15}}}});
    assert_eq!(fails(&mut s, "element_set", bad), ErrorCode::Rejected);
    let bad = json!({"id": set, "params": {"layout": {"PileSpiral": {"pitch": 0.1}}}});
    assert_eq!(fails(&mut s, "element_set", bad), ErrorCode::Rejected);
    run(&mut s, "element_set", json!({"id": set, "params": {"layout": {"CapBottom": {"count": 2, "spacing": 0.2}}}}));
    // A crafted file: a cap's set in a column, a pile's in a cap, a cap set
    // with a spacing that is no number, and a pile set whose pile is gone.
    let mut p = s.project().clone();
    let mut add = |host: u64, layout: RebarLayout| {
        let mut e = p.element(ElementId(set)).unwrap().clone();
        e.id = ElementId(p.alloc_id());
        e.kind = ElementKind::Rebar(cc_core::Rebar::new(ElementId(host), layout, 0.05));
        p.elements.insert(e.id, e.clone());
        e.id
    };
    let in_column = add(column, RebarLayout::CapTop { spacing: 0.15, bend: 0.0 });
    let in_cap = add(cap, RebarLayout::PileSpiral { pitch: 0.1, length: 0.0 });
    let nan = add(cap, RebarLayout::CapTop { spacing: f64::NAN, bend: 0.0 });
    let orphan = add(999_999, RebarLayout::PileTies { spacing: 0.15, length: 0.0, hooks: None });
    let fine = add(pile, RebarLayout::PileTies { spacing: 0.15, length: 2.0, hooks: None });
    let text = cc_core::to_ron(&p).unwrap();
    let (back, repairs) = cc_core::from_ron_with_repairs(&text).unwrap();
    assert!(repairs >= 4, "{repairs}");
    for gone in [in_column, in_cap, nan, orphan] {
        assert!(back.element(gone).is_none(), "{gone:?}");
    }
    assert!(back.element(fine).is_some() && back.element(ElementId(set)).is_some());
    assert_eq!(back.repairs_needed(), 0);
    // A cap removed from the file: its piles and every set of theirs go.
    let mut p = back.clone();
    p.elements.remove(&ElementId(cap));
    assert!(p.repair() > 0);
    assert_eq!(p.repairs_needed(), 0);
    assert!(!p.elements.values().any(|e| matches!(e.kind, ElementKind::Rebar(_) | ElementKind::Pile(_))));
    // Not numbers in the geometry: nothing laid out, and no panic.
    let mut p = back;
    if let Some(ElementKind::Rebar(r)) = p.elements.get_mut(&ElementId(set)).map(|e| &mut e.kind) {
        r.cover = f64::NAN;
    }
    let _ = cc_geom::ModelGeometry::build(&p);
}

/// Files made before these layouts read and write as they did: no new
/// field is written for the sets and marks they hold.
#[test]
fn older_files_are_unchanged() {
    let mut s = Session::default();
    let beam = id(&run(&mut s, "beam_create", json!({"start": [0, 0], "end": [5, 0], "top_offset": 3.0})));
    run(&mut s, "rebar_typical", json!({"elements": [beam]}));
    run(&mut s, "bar_marks_update", json!({}));
    let text = cc_core::to_ron(s.project()).unwrap();
    for new in ["pitch:", "CapBottom", "CapTop", "CapSides", "PileLongitudinal", "PileSpiral", "PileTies"] {
        assert!(!text.contains(new), "{new} written");
    }
    let (back, repairs) = cc_core::from_ron_with_repairs(&text).unwrap();
    assert_eq!(repairs, 0);
    assert_eq!(cc_core::to_ron(&back).unwrap(), text);
    // A frozen mark of a spiral keeps its pitch and its turns.
    let leg = cc_core::bar_marks::MarkLeg {
        length: 50.0,
        radius: Some(0.15),
        sweep: 360.0 * 59.5,
        pitch: 0.1,
        ..Default::default()
    };
    let shape = cc_core::bar_marks::MarkShape { closed: false, legs: vec![leg] };
    assert!(shape.is_valid());
    let flat = cc_core::bar_marks::MarkShape { legs: vec![cc_core::bar_marks::MarkLeg { pitch: 0.0, ..leg }], ..shape };
    assert!(!flat.is_valid(), "a flat arc turns at most once");
}

/// IFC: a cap's set and a pile's come back as sets of the imported cap and
/// pile, laid out again with their lengths.
#[test]
fn ifc_round_trip_restores_the_sets_in_their_hosts() {
    let mut s = session();
    let (cap, _) = cap(&mut s, json!({"layout": "triangle"}), [0.0, 0.0]);
    run(&mut s, "pile_cap_rebar_create", json!({"caps": [cap], "max_length": 0}));
    run(&mut s, "pile_rebar_create", json!({"caps": [cap], "max_length": 0}));
    let p = s.project().clone();
    let before = cc_geom::ModelGeometry::build(&p);
    let text = cc_ifc::export(&p, &cc_ifc::ExportOptions::default());
    let (back, _) = cc_ifc::import(&text, cc_core::Project::from_template()).unwrap();
    let after = cc_geom::ModelGeometry::build(&back);
    let mut sets = 0;
    for e in p.elements.values() {
        let ElementKind::Rebar(r) = &e.kind else { continue };
        let host = p.element(r.host).unwrap();
        let mine = back.elements.values().find(|x| x.uid == e.uid).expect("the set comes back");
        let ElementKind::Rebar(again) = &mine.kind else { panic!("a set laid out in its host, not {:?}", mine.kind) };
        assert_eq!(again.layout, r.layout);
        assert_eq!(back.element(again.host).unwrap().uid, host.uid);
        let total = |g: &cc_geom::ModelGeometry, id: ElementId| g.rebar[&id].total_length();
        assert!((total(&before, e.id) - total(&after, mine.id)).abs() < 1e-6);
        sets += 1;
    }
    assert_eq!(sets, 3 + 3 * 2);
}
