//! Reinforcement drawn in floor plans and sections (`view_set` `rebar`):
//! bars the cut plane crosses as dots, the rest of the bars within the
//! view's range as lines, and bar tags drawing their sets within the same range.

use cc_core::{ElementId, ElementKind, RebarLayout};
use cc_engine::{ErrorCode, Session};
use cc_views::{Drawing2D, Prim};
use serde_json::{Value, json};

fn run(s: &mut Session, name: &str, args: Value) -> Value {
    s.execute(name, args).unwrap_or_else(|e| panic!("{name}: {e:?}"))
}

fn id(v: &Value) -> u64 {
    v["id"].as_u64().unwrap_or_else(|| panic!("no id in {v}"))
}

/// The dots (fills) and lines a drawing has of set `set`.
fn of_set(d: &Drawing2D, set: u64) -> (usize, usize) {
    let fills = d.prims.iter().filter(|p| matches!(p, Prim::Fill { element: Some(e), .. } if e.0 == set)).count();
    let lines = d.prims.iter().filter(|p| matches!(p, Prim::Line { element: Some(e), .. } if e.0 == set)).count();
    (fills, lines)
}

/// A column with four vertical bars and stirrups every 15 cm.
fn column(s: &mut Session) -> (u64, u64) {
    let column = id(&run(s, "column_create", json!({"position": [0, 0]})));
    let bars = id(&run(s, "rebar_create", json!({"host": column, "layout": "perimeter", "count": 4})));
    let stirrups = id(&run(s, "rebar_create", json!({"host": column, "layout": "stirrups", "spacing": 0.15})));
    (bars, stirrups)
}

#[test]
fn plans_and_sections_draw_the_bars_within_their_range() {
    let mut s = Session::default();
    let (bars, stirrups) = column(&mut s);
    let geom = s.geometry().clone();
    let (vertical, loops) = (&geom.rebar[&ElementId(bars)], &geom.rebar[&ElementId(stirrups)]);
    // A plan draws no reinforcement until asked.
    let plan = s.project().views.values().find(|v| v.is_plan()).map(|v| v.id.0).unwrap();
    assert_eq!(of_set(&s.drawing(cc_core::ViewId(plan)).unwrap(), bars), (0, 0));
    let out = run(&mut s, "view_set", json!({"view": plan, "rebar": true}));
    assert_eq!(out["rebar"], json!(true));
    // The vertical bars cross the cut plane: a dot each. The stirrups below it are lines.
    let d = s.drawing(cc_core::ViewId(plan)).unwrap();
    assert_eq!(of_set(&d, bars), (vertical.bars.len(), 0));
    let (dots, lines) = of_set(&d, stirrups);
    assert_eq!(dots, 0);
    assert!(lines > 0);
    // A section across the column at its middle, looking to -y: the bars behind the cut
    // plane (y < 0) are lines, those in front are left out, and each stirrup is cut
    // where its two legs across the plane pass through it.
    let section =
        id(&run(&mut s, "view_create", json!({"kind": "section", "start": [-1, 0], "end": [1, 0], "depth": 2.0})));
    run(&mut s, "view_set", json!({"view": section, "rebar": true}));
    let behind = vertical.bars.iter().filter(|b| b.points[0].y < 0.0).count();
    assert!(behind > 0 && behind < vertical.bars.len());
    let d = s.drawing(cc_core::ViewId(section)).unwrap();
    assert_eq!(of_set(&d, bars), (0, behind));
    assert_eq!(of_set(&d, stirrups).0, 2 * loops.bars.len());
    // A section only 1 cm deep sees none of the vertical bars.
    run(&mut s, "view_set", json!({"view": section, "params": {"depth": 0.01}}));
    assert_eq!(of_set(&s.drawing(cc_core::ViewId(section)).unwrap(), bars), (0, 0));
    // Without `rebar`, a bar tag draws its set within the same range (only the cut stirrups).
    run(&mut s, "view_set", json!({"view": section, "rebar": false}));
    run(&mut s, "bar_tag_create", json!({"view": section, "set": stirrups, "position": [0.5, 1.0]}));
    run(&mut s, "bar_tag_create", json!({"view": section, "set": bars, "position": [0.5, 2.0]}));
    let d = s.drawing(cc_core::ViewId(section)).unwrap();
    assert_eq!(of_set(&d, bars), (0, 0));
    assert_eq!(of_set(&d, stirrups), (2 * loops.bars.len(), 0));
    // Only views that draw the model in plan, elevation, section or 3D.
    let schedule = id(&run(&mut s, "bending_schedule_create", json!({})));
    let e = s.execute("view_set", json!({"view": schedule, "rebar": true})).unwrap_err();
    assert_eq!(e.code, ErrorCode::InvalidArguments);
    // In a 3D view it is the view's own reinforcement switch.
    let three_d = id(&run(&mut s, "view_create", json!({"kind": "three_d"})));
    run(&mut s, "view_set", json!({"view": three_d, "rebar": true}));
    let v = s.project().view(cc_core::ViewId(three_d)).unwrap();
    assert!(matches!(v.kind, cc_core::ViewKind::ThreeD { rebar: true, .. }) && !v.rebar);
    assert!(matches!(
        &s.project().element(ElementId(bars)).unwrap().kind,
        ElementKind::Rebar(r) if matches!(r.layout, RebarLayout::Longitudinal { .. })
    ));
}

/// The layout of reinforcement set `set`.
fn layout(s: &Session, set: u64) -> RebarLayout {
    match &s.project().element(ElementId(set)).map(|e| &e.kind) {
        Some(ElementKind::Rebar(r)) => r.layout.clone(),
        other => panic!("no rebar {set}: {other:?}"),
    }
}

/// The dots of set `set` in a drawing, as their centers and radii.
fn dots(d: &Drawing2D, set: u64) -> Vec<(glam::DVec2, f64)> {
    d.prims
        .iter()
        .filter_map(|p| match p {
            Prim::Fill { pts, element: Some(e), .. } if e.0 == set && !pts.is_empty() => {
                let c = pts.iter().copied().sum::<glam::DVec2>() / pts.len() as f64;
                Some((c, pts.iter().map(|q| q.distance(c)).fold(0.0, f64::max)))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn dots_are_as_wide_as_the_bar_and_at_least_visible_on_paper() {
    let mut s = Session::default();
    let (bars, _) = column(&mut s);
    let diameter = s.geometry().rebar[&ElementId(bars)].diameter;
    let plan = s.project().views.values().find(|v| v.is_plan()).map(|v| v.id.0).unwrap();
    run(&mut s, "view_set", json!({"view": plan, "rebar": true}));
    for scale in [5, 100] {
        run(&mut s, "view_set", json!({"view": plan, "scale": scale}));
        let want = (diameter / 2.0).max(cc_views::rebar_view::MIN_DOT * scale as f64 / 1000.0);
        let found = dots(&s.drawing(cc_core::ViewId(plan)).unwrap(), bars);
        assert_eq!(found.len(), 4);
        assert!(found.iter().all(|(_, r)| (r - want).abs() < 1e-9), "1:{scale}: {found:?} vs {want}");
    }
}

#[test]
fn elevations_draw_every_bar_and_plans_stop_at_the_level_below() {
    let mut s = Session::default();
    let (bars, stirrups) = column(&mut s);
    let loops = s.geometry().rebar[&ElementId(stirrups)].bars.len();
    // An elevation has no cut: every bar as lines, no dots (the legs of a stirrup seen
    // end-on are its corners).
    let elevation = s
        .project()
        .views
        .values()
        .find(|v| matches!(v.kind, cc_core::ViewKind::Elevation { .. }))
        .map(|v| v.id.0)
        .unwrap();
    run(&mut s, "view_set", json!({"view": elevation, "rebar": true}));
    let d = s.drawing(cc_core::ViewId(elevation)).unwrap();
    assert_eq!(of_set(&d, bars), (0, 4));
    let (dots, lines) = of_set(&d, stirrups);
    assert!(dots == 0 && lines >= loops, "{dots} {lines} {loops}");
    // A level high above: its plan looks down to the level below it, above the column.
    let top =
        s.geometry().rebar[&ElementId(bars)].bars.iter().flat_map(|b| b.points.iter().map(|p| p.z)).fold(0.0, f64::max);
    let high = run(&mut s, "level_create", json!({"name": "High", "elevation": top + 20.0}));
    run(&mut s, "level_create", json!({"name": "Middle", "elevation": top + 10.0}));
    let plan = id(&run(&mut s, "view_create", json!({"kind": "floor_plan", "level": high["id"]})));
    run(&mut s, "view_set", json!({"view": plan, "rebar": true, "params": {"cut_height": 1.0}}));
    let d = s.drawing(cc_core::ViewId(plan)).unwrap();
    assert_eq!((of_set(&d, bars), of_set(&d, stirrups)), ((0, 0), (0, 0)));
}

#[test]
fn ring_bars_cut_by_a_radial_section_are_dots() {
    let mut s = Session::default();
    // A half cylinder of radius 2.2 m, 3 m high, its middle at (0, 2.2).
    let wall =
        id(&run(&mut s, "wall_create", json!({"start": [2.2, 0], "end": [-2.2, 0], "bulge": 1.0, "height": 3.0})));
    let out = run(&mut s, "wall_rebar_create", json!({"walls": [wall], "max_length": 0}));
    let sets: Vec<u64> = out["created"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap()).collect();
    let geom = s.geometry().clone();
    let rings: Vec<u64> =
        sets.iter().copied().filter(|x| matches!(layout(&s, *x), RebarLayout::WallHorizontal { .. })).collect();
    assert!(!rings.is_empty());
    let section =
        id(&run(&mut s, "view_create", json!({"kind": "section", "start": [0, 3.0], "end": [0, 1.4], "depth": 0.05})));
    run(&mut s, "view_set", json!({"view": section, "rebar": true}));
    let d = s.drawing(cc_core::ViewId(section)).unwrap();
    for ring in rings {
        let count = geom.rebar[&ElementId(ring)].bars.len();
        // Each ring bar crosses the section once, square to it: a dot each (the bit of
        // the ring curving away behind the cut, within the 5 cm depth, is a short line).
        assert_eq!(of_set(&d, ring).0, count, "set {ring}");
    }
}

#[test]
fn a_starter_bar_with_a_foot_below_the_cut_is_a_dot_and_its_foot() {
    let mut s = Session::default();
    // A wall lower than the plan's cut, its vertical bars bent at the foot.
    let wall = id(&run(&mut s, "wall_create", json!({"start": [0, 0], "end": [2, 0], "height": 1.0})));
    let args = json!({"walls": [wall], "faces": "exterior", "vertical": {"bottom": {"across": 0.3}}, "max_length": 0});
    let out = run(&mut s, "wall_rebar_create", args);
    let geom = s.geometry().clone();
    let vertical = out["created"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap())
        .find(|x| matches!(layout(&s, *x), RebarLayout::WallVertical { .. }))
        .unwrap();
    let count = geom.rebar[&ElementId(vertical)].bars.len();
    let plan = s.project().views.values().find(|v| v.is_plan()).map(|v| v.id.0).unwrap();
    run(&mut s, "view_set", json!({"view": plan, "rebar": true}));
    let d = s.drawing(cc_core::ViewId(plan)).unwrap();
    // Not cut (the wall ends below the cut), seen end-on: a dot each, and the foot a line.
    assert_eq!(of_set(&d, vertical), (count, count));
}

#[test]
fn bar_tags_in_ceiling_plans_still_draw_their_sets() {
    let mut s = Session::default();
    let (bars, stirrups) = column(&mut s);
    let loops = s.geometry().rebar[&ElementId(stirrups)].bars.len();
    let level = s.project().levels.first().map(|l| l.id.0).unwrap();
    let ceiling = id(&run(&mut s, "view_create", json!({"kind": "ceiling_plan", "level": level})));
    run(&mut s, "bar_tag_create", json!({"view": ceiling, "set": stirrups, "position": [1, 1]}));
    run(&mut s, "bar_tag_create", json!({"view": ceiling, "set": bars, "position": [1, 2]}));
    let d = s.drawing(cc_core::ViewId(ceiling)).unwrap();
    // The whole set seen from above, as before: the stirrups' outlines (four sides each),
    // the vertical bars as dots.
    assert_eq!(of_set(&d, stirrups), (0, 4 * loops));
    assert_eq!(of_set(&d, bars), (4, 0));
    // Where the bars stand.
    let geom = s.geometry().clone();
    for bar in &geom.rebar[&ElementId(bars)].bars {
        let at = bar.points[0].truncate();
        assert!(dots(&d, bars).iter().any(|(c, _)| c.distance(at) < 1e-6), "{at}");
    }
    // Ceiling plans take no `rebar` of their own.
    assert_eq!(
        s.execute("view_set", json!({"view": ceiling, "rebar": true})).unwrap_err().code,
        ErrorCode::InvalidArguments
    );
}
