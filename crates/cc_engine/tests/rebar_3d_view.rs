//! The 2D form of 3D reinforcement views (on sheets and in exports): the
//! bars over the model drawn faint, as the 3D view shows them, and bars
//! seen end-on as dots.

use cc_engine::Session;
use cc_views::{LineStyle, Prim};
use serde_json::{Value, json};

fn run(s: &mut Session, name: &str, args: Value) -> Value {
    s.execute(name, args).unwrap_or_else(|e| panic!("{name}: {e:?}"))
}

fn id(v: &Value) -> u64 {
    v["id"].as_u64().unwrap_or_else(|| panic!("no id in {v}"))
}

#[test]
fn reinforcement_views_draw_the_model_faint() {
    let mut s = Session::default();
    let column = id(&run(&mut s, "column_create", json!({"position": [0, 0]})));
    let bars = id(&run(&mut s, "rebar_create", json!({"host": column, "layout": "perimeter", "count": 4})));
    let count = s.geometry().rebar[&cc_core::ElementId(bars)].bars.len();
    let view = id(&run(&mut s, "view_create", json!({"kind": "rebar"})));
    let d = s.drawing(cc_core::ViewId(view)).unwrap();
    // The column's edges, faint and only faint, under the bars.
    let column_lines = |style: fn(&LineStyle) -> bool| {
        d.prims
            .iter()
            .any(|p| matches!(p, Prim::Line { element: Some(x), style: st, .. } if x.0 == column && style(st)))
    };
    assert!(column_lines(|st| *st == LineStyle::Faint));
    assert!(!column_lines(|st| matches!(st, LineStyle::Cut | LineStyle::Projection)));
    let lines = d
        .prims
        .iter()
        .filter(|p| matches!(p, Prim::Line { element: Some(x), style: LineStyle::Projection, .. } if x.0 == bars))
        .count();
    assert_eq!(lines, count);
    // Seen from above, the vertical bars are dots.
    let camera = json!({"target": [0, 0, 1.5], "yaw_deg": 0, "pitch_deg": 90, "distance": 10});
    run(&mut s, "view_set", json!({"view": view, "params": {"camera": camera}}));
    let d = s.drawing(cc_core::ViewId(view)).unwrap();
    let dots = d.prims.iter().filter(|p| matches!(p, Prim::Fill { element: Some(x), .. } if x.0 == bars)).count();
    assert_eq!(dots, count);
}
