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
