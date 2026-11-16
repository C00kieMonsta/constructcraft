//! Editable tags (phase 2): grid types draw their bubbles as their style
//! says, each view moves or hides a grid's bubbles; view mark types draw
//! the marks of sections and elevations in plans, and each plan moves the
//! markers of elevations. Without types, plans draw what they always drew.

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_core::ViewId;
use cc_core::view_mark::MarkKind;
use cc_engine::{ErrorCode, Session};
use cc_views::{Anchor, LineStyle, Prim};
use glam::DVec2;
use serde_json::{Value, json};

fn run(s: &mut Session, name: &str, args: Value) -> Value {
    s.execute(name, args).unwrap_or_else(|e| panic!("{name}: {e:?}"))
}

fn fails(s: &mut Session, name: &str, args: Value) -> ErrorCode {
    s.execute(name, args).map(|v| panic!("{name} should fail, gave {v}")).unwrap_err().code
}

fn id(v: &Value) -> u64 {
    v["id"].as_u64().unwrap()
}

fn first(s: &mut Session, category: &str) -> u64 {
    id(&run(s, "views_list", json!({ "category": category }))[0])
}

fn prims(s: &mut Session, view: u64) -> Vec<Prim> {
    s.drawing(ViewId(view)).unwrap().prims.clone()
}

/// The lines of `element` in `view` drawn in `style`.
fn lines(s: &mut Session, view: u64, element: u64, style: LineStyle) -> Vec<(DVec2, DVec2)> {
    s.drawing(ViewId(view))
        .unwrap()
        .prims
        .iter()
        .filter_map(|p| match p {
            Prim::Line { a, b, style: st, element: Some(e), .. } if e.0 == element && *st == style => Some((*a, *b)),
            _ => None,
        })
        .collect()
}

fn element_texts(s: &mut Session, view: u64, element: u64) -> Vec<(DVec2, String, f64)> {
    s.drawing(ViewId(view))
        .unwrap()
        .prims
        .iter()
        .filter_map(|p| match p {
            Prim::Text { pos, text, size, element: Some(e), .. } if e.0 == element => Some((*pos, text.clone(), *size)),
            _ => None,
        })
        .collect()
}

/// Texts that belong to no element (the marks of views).
fn free_texts(s: &mut Session, view: u64) -> Vec<(DVec2, String, f64, Anchor)> {
    s.drawing(ViewId(view))
        .unwrap()
        .prims
        .iter()
        .filter_map(|p| match p {
            Prim::Text { pos, text, size, anchor, element: None, .. } => Some((*pos, text.clone(), *size, *anchor)),
            _ => None,
        })
        .collect()
}

/// A plan at 1:50 with a wall and a grid from (0, -2) to (0, 8).
fn with_grid() -> (Session, u64, u64) {
    let mut s = Session::default();
    run(&mut s, "wall_create", json!({"start": [0.0, 0.0], "end": [8.0, 0.0]}));
    let grid = id(&run(&mut s, "grid_create", json!({"start": [0.0, -2.0], "end": [0.0, 8.0], "name": "A"})));
    let plan = first(&mut s, "floor_plan");
    run(&mut s, "view_set", json!({"view": plan, "scale": 50}));
    (s, plan, grid)
}

#[test]
fn grids_draw_their_bubbles_as_their_style_says() {
    let (mut s, plan, grid) = with_grid();
    // The template's grid type: an 8 mm circle of 32 chords at the end, the
    // name 0.45 of it high at its center.
    let bubble = lines(&mut s, plan, grid, LineStyle::Annotation);
    assert_eq!(bubble.len(), 32);
    let texts = element_texts(&mut s, plan, grid);
    assert_eq!(texts.len(), 1);
    let r = 8.0 * 0.05 / 2.0;
    assert!((texts[0].0 - DVec2::new(0.0, 8.0 + r)).length() < 1e-9 && (texts[0].2 - 3.6).abs() < 1e-9, "{texts:?}");
    let got = run(&mut s, "tag_get", json!({"view": plan, "id": grid}));
    assert_eq!((got["target"].as_str(), got["ends"].as_str()), (Some("grid"), Some("end")));
    assert_eq!(got["bubbles"].as_array().unwrap().len(), 1);

    // Both ends, a hexagon, 5 mm text, a heavier red pen and a dashed line.
    let grid_type = id(&run(&mut s, "types_list", json!({"category": "grid"}))[0]);
    let style = json!({"ends": "Both", "symbol": "Hexagon", "text_size": 5.0, "pen": 0.5, "color": [200, 0, 0], "line_pattern": "Dash"});
    run(&mut s, "type_set", json!({"type": grid_type, "params": {"style": style}}));
    assert_eq!(lines(&mut s, plan, grid, LineStyle::Annotation).len(), 12, "two hexagons");
    let texts = element_texts(&mut s, plan, grid);
    assert_eq!(texts.len(), 2);
    assert!(texts.iter().all(|t| (t.2 - 5.0).abs() < 1e-9));
    assert!(texts.iter().any(|t| (t.0 - DVec2::new(0.0, -2.0 - r)).length() < 1e-9), "the start bubble");
    let d = s.drawing(ViewId(plan)).unwrap();
    let pens: Vec<_> = d
        .prims
        .iter()
        .filter_map(|p| match p {
            Prim::Line { style: LineStyle::Center, element: Some(e), pen, .. } if e.0 == grid => Some(*pen),
            _ => None,
        })
        .collect();
    assert_eq!(pens.len(), 1);
    let pen = pens[0].unwrap();
    assert_eq!((pen.weight, pen.color, pen.pattern), (Some(0.5), Some([200, 0, 0]), Some(cc_core::LinePattern::Dash)));

    // The grid's own ends come first.
    run(&mut s, "element_set", json!({"id": grid, "params": {"ends": "None"}}));
    assert!(lines(&mut s, plan, grid, LineStyle::Annotation).is_empty());
    assert!(element_texts(&mut s, plan, grid).is_empty());
    assert_eq!(lines(&mut s, plan, grid, LineStyle::Center).len(), 1, "the line stays");

    // Values out of range leave the model invalid: refused.
    let bad = json!({"type": grid_type, "params": {"style": {"text_size": -1.0}}});
    assert_eq!(fails(&mut s, "type_set", bad), ErrorCode::Rejected);
}

#[test]
fn views_move_and_hide_grid_bubbles() {
    let (mut s, plan, grid) = with_grid();
    let before = prims(&mut s, plan);
    // Moved aside: the bubble follows, with an elbow from the line's end.
    run(&mut s, "tag_set", json!({"view": plan, "ids": [grid], "offset": [1.0, 0.5]}));
    let r = 8.0 * 0.05 / 2.0;
    let texts = element_texts(&mut s, plan, grid);
    assert!((texts[0].0 - DVec2::new(1.0, 8.0 + r + 0.5)).length() < 1e-9, "{texts:?}");
    let leader = lines(&mut s, plan, grid, LineStyle::Annotation);
    assert_eq!(leader.len(), 32 + 3, "the bubble and a three-piece elbow");
    assert!(leader.iter().any(|(a, _)| (*a - DVec2::new(0.0, 8.0)).length() < 1e-9), "from the line's end");
    // The start bubble is moved on its own.
    let grid_type = id(&run(&mut s, "types_list", json!({"category": "grid"}))[0]);
    run(&mut s, "type_set", json!({"type": grid_type, "params": {"style": {"ends": "Both"}}}));
    run(&mut s, "tag_set", json!({"view": plan, "ids": [grid], "start_offset": [-1.0, 0.0]}));
    let got = run(&mut s, "tag_get", json!({"view": plan, "id": grid}));
    let starts: Vec<&Value> = got["bubbles"].as_array().unwrap().iter().filter(|b| b["start"] == true).collect();
    assert_eq!(starts.len(), 1);
    assert_eq!(starts[0]["center"], json!([-1.0, -2.0 - r]));
    assert_eq!(got["placement"]["start_offset"], json!([-1.0, 0.0]));
    // Hidden: the line stays, the bubbles go.
    run(&mut s, "tag_set", json!({"view": plan, "ids": [grid], "hidden": true}));
    assert!(lines(&mut s, plan, grid, LineStyle::Annotation).is_empty());
    assert_eq!(lines(&mut s, plan, grid, LineStyle::Center).len(), 1);
    // Grids take no rotation, type or leader; other elements no start bubble.
    assert_eq!(
        fails(&mut s, "tag_set", json!({"view": plan, "ids": [grid], "rotation": 30})),
        ErrorCode::InvalidArguments
    );
    let door_wall = run(&mut s, "elements_list", json!({"category": "wall"}))[0]["id"].clone();
    let door = id(&run(&mut s, "door_create", json!({"wall": door_wall})));
    let bad = json!({"view": plan, "ids": [door], "start_offset": [1.0, 0.0]});
    assert_eq!(fails(&mut s, "tag_set", bad), ErrorCode::InvalidArguments);
    // Reset: what the plan drew at first (the door and the type aside).
    run(&mut s, "elements_delete", json!({"ids": [door]}));
    run(&mut s, "type_set", json!({"type": grid_type, "params": {"style": {"ends": "End"}}}));
    run(&mut s, "tag_set", json!({"view": plan, "ids": [grid], "reset": true}));
    assert_eq!(prims(&mut s, plan), before);
    // Nothing adjusted is kept.
    assert!(s.project().view(ViewId(plan)).unwrap().tags.is_empty());
}

#[test]
fn elevations_and_sections_move_their_grid_bubbles() {
    let (mut s, _, grid) = with_grid();
    // A section along x shows the grids running across it, along y.
    run(&mut s, "elements_delete", json!({"ids": [grid]}));
    let grid = id(&run(&mut s, "grid_create", json!({"start": [3.0, -2.0], "end": [3.0, 8.0], "name": "1"})));
    let section = id(&run(&mut s, "view_create", json!({"kind": "section", "start": [-1.0, 3.0], "end": [9.0, 3.0]})));
    let before = element_texts(&mut s, section, grid);
    assert_eq!(before.len(), 1, "the bubble on top");
    run(&mut s, "tag_set", json!({"view": section, "ids": [grid], "offset": [0.5, 0.0]}));
    let after = element_texts(&mut s, section, grid);
    assert!((after[0].0 - before[0].0 - DVec2::new(0.5, 0.0)).length() < 1e-9, "{before:?} {after:?}");
    // Tags of doors are adjusted in plans only.
    let wall = run(&mut s, "elements_list", json!({"category": "wall"}))[0]["id"].clone();
    let door = id(&run(&mut s, "door_create", json!({"wall": wall})));
    assert_eq!(
        fails(&mut s, "tag_set", json!({"view": section, "ids": [door], "hidden": true})),
        ErrorCode::InvalidArguments
    );
}

#[test]
fn default_mark_types_draw_what_plans_always_drew() {
    let mut s = Session::default();
    run(&mut s, "wall_create", json!({"start": [0.0, 0.0], "end": [8.0, 0.0]}));
    let plan = first(&mut s, "floor_plan");
    let section = id(&run(&mut s, "view_create", json!({"kind": "section", "start": [0.0, 3.0], "end": [8.0, 3.0]})));
    let before = prims(&mut s, plan);
    // Today's section head: number 3.5 mm over sheet 2.5 mm; elevation
    // markers labeled with their name, 2.5 mm.
    let texts = free_texts(&mut s, plan);
    let name = s.project().view(ViewId(section)).unwrap().name.clone();
    let letter: String = name.split_whitespace().last().unwrap().chars().take(4).collect();
    assert_eq!(texts.iter().filter(|t| t.1 == letter && t.2 == 3.5 && t.3 == Anchor::Center).count(), 2);
    assert_eq!(texts.iter().filter(|t| t.1 == "-" && t.2 == 2.5).count(), 2);
    let elevations = run(&mut s, "views_list", json!({"category": "elevation"}));
    for e in elevations.as_array().unwrap() {
        let n = e["name"].as_str().unwrap();
        assert!(texts.iter().any(|t| t.1 == n && t.2 == 2.5 && t.3 == Anchor::LeftBottom), "{n}");
    }
    // The template has a mark type of each kind, of the default style: a
    // project without mark types (an older file) draws the same plan.
    let template: Vec<String> = [MarkKind::Section, MarkKind::Elevation]
        .map(|k| s.project().mark_types_of(k).map(|t| t.name.clone()).collect::<Vec<_>>().join(","))
        .into();
    assert_eq!(template, ["Marca de corte", "Marca de elevação"]);
    let mut bare = s.project().clone();
    bare.types.retain(|_, t| t.category() != cc_core::Category::ViewMark);
    let mut older = Session::new(cc_core::Document::new(bare));
    assert_eq!(prims(&mut older, plan), before);
    // More types of the default styles draw the same.
    let a = run(&mut s, "view_mark_type_create", json!({"kind": "section"}));
    let b = run(&mut s, "view_mark_type_create", json!({"kind": "elevation"}));
    assert_eq!(a["name"].as_str(), Some("Marca de corte 2"));
    assert_eq!(b["params"]["style"]["head"], json!(0.0));
    assert_eq!(prims(&mut s, plan), before);
}

#[test]
fn mark_types_restyle_section_and_elevation_marks() {
    let mut s = Session::default();
    run(&mut s, "wall_create", json!({"start": [0.0, 0.0], "end": [8.0, 0.0]}));
    let plan = first(&mut s, "floor_plan");
    run(&mut s, "view_set", json!({"view": plan, "scale": 100}));
    let section = id(&run(
        &mut s,
        "view_create",
        json!({"kind": "section", "start": [0.0, 3.0], "end": [8.0, 3.0], "name": "Corte B"}),
    ));
    let style = json!({"head": 16.0, "top": {"text": "{Letter}", "size": 4.0}, "bottom": {"text": "{Name}", "size": 2.0}, "heads": "Start"});
    let big = run(&mut s, "view_mark_type_create", json!({"kind": "section", "name": "Big", "style": style}));
    run(&mut s, "view_mark_type_create", json!({"kind": "section", "name": "Plain"}));
    // The project's first section mark type applies: the template's, until it goes.
    let template_section = s.project().mark_types_of(MarkKind::Section).next().unwrap().id.0;
    assert_eq!(free_texts(&mut s, plan).iter().filter(|t| t.1 == "B" && t.2 == 3.5).count(), 2);
    run(&mut s, "type_delete", json!({"type": template_section}));
    let mark = s.drawing(ViewId(plan)).unwrap().sections.iter().find(|h| h.view.0 == section).cloned().unwrap();
    assert!((mark.radius - 0.8).abs() < 1e-9, "16 mm at 1:100: {}", mark.radius);
    let texts = free_texts(&mut s, plan);
    assert_eq!(texts.iter().filter(|t| t.1 == "B" && t.2 == 4.0).count(), 1, "one head: {texts:?}");
    assert_eq!(texts.iter().filter(|t| t.1 == "Corte B" && t.2 == 2.0).count(), 1);
    // The view's own type comes first.
    run(&mut s, "view_set", json!({"view": section, "mark_type": "Plain"}));
    assert_eq!(run(&mut s, "view_get", json!({"view": section}))["mark_type"]["name"], json!("Plain"));
    assert_eq!(free_texts(&mut s, plan).iter().filter(|t| t.1 == "B" && t.2 == 3.5).count(), 2);
    run(&mut s, "view_set", json!({"view": section, "mark_type": null}));
    assert_eq!(run(&mut s, "view_get", json!({"view": section}))["mark_type"], Value::Null);
    // A section type on an elevation, a mark type on a plan: refused.
    let elevation = first(&mut s, "elevation");
    assert_eq!(
        fails(&mut s, "view_set", json!({"view": elevation, "mark_type": id(&big)})),
        ErrorCode::InvalidArguments
    );
    assert_eq!(fails(&mut s, "view_set", json!({"view": plan, "mark_type": id(&big)})), ErrorCode::InvalidArguments);
    let bad = json!({"kind": "section", "style": {"head": 500.0}});
    assert_eq!(fails(&mut s, "view_mark_type_create", bad), ErrorCode::InvalidArguments);

    // Elevation markers: a paper-size head with a hexagon, the name inside.
    let style =
        json!({"head": 10.0, "symbol": "Hexagon", "top": {"text": "{Name}", "size": 2.0}, "label": {"text": ""}});
    let template_elevation = s.project().mark_types_of(MarkKind::Elevation).next().unwrap().id.0;
    run(&mut s, "view_mark_type_create", json!({"kind": "elevation", "style": style}));
    run(&mut s, "type_delete", json!({"type": template_elevation}));
    let name = s.project().view(ViewId(elevation)).unwrap().name.clone();
    let texts = free_texts(&mut s, plan);
    assert!(texts.iter().any(|t| t.1 == name && t.2 == 2.0 && t.3 == Anchor::Center), "{texts:?}");
    let m = s.drawing(ViewId(plan)).unwrap().marks.iter().find(|m| m.view.0 == elevation).copied().unwrap();
    assert!((m.half.x - 1.6 * 0.5).abs() < 1e-9, "a 10 mm head at 1:100: {:?}", m.half);
    // Deleting the type leaves the view on the default.
    run(&mut s, "view_set", json!({"view": section, "mark_type": "Big"}));
    run(&mut s, "type_delete", json!({"type": id(&big)}));
    assert_eq!(s.project().view(ViewId(section)).unwrap().mark_type, None);
}

#[test]
fn plans_move_elevation_markers() {
    let mut s = Session::default();
    run(&mut s, "wall_create", json!({"start": [0.0, 0.0], "end": [8.0, 0.0]}));
    let plan = first(&mut s, "floor_plan");
    let elevation = first(&mut s, "elevation");
    let at = |s: &mut Session| {
        s.drawing(ViewId(plan)).unwrap().marks.iter().find(|m| m.view.0 == elevation).copied().unwrap()
    };
    let home = at(&mut s);
    assert_eq!(home.offset, DVec2::ZERO);
    assert_eq!(s.drawing(ViewId(plan)).unwrap().mark_at(home.center).map(|m| m.view.0), Some(elevation));
    run(&mut s, "mark_move", json!({"plan": plan, "view": elevation, "delta": [1.0, -2.0]}));
    run(&mut s, "mark_move", json!({"plan": plan, "view": elevation, "delta": [1.0, 0.0]}));
    let moved = at(&mut s);
    assert!((moved.center - home.center - DVec2::new(2.0, -2.0)).length() < 1e-9);
    assert_eq!(moved.offset, DVec2::new(2.0, -2.0));
    // Another plan keeps its own.
    let other = id(&run(&mut s, "view_duplicate", json!({"view": plan})));
    run(&mut s, "mark_move", json!({"plan": other, "view": elevation, "reset": true}));
    assert_eq!(at(&mut s).offset, DVec2::new(2.0, -2.0));
    // Refused: a section is no elevation, an elevation is no plan, too far.
    let section = id(&run(&mut s, "view_create", json!({"kind": "section", "start": [0.0, 3.0], "end": [8.0, 3.0]})));
    assert_eq!(
        fails(&mut s, "mark_move", json!({"plan": plan, "view": section, "reset": true})),
        ErrorCode::InvalidArguments
    );
    assert_eq!(
        fails(&mut s, "mark_move", json!({"plan": elevation, "view": elevation, "reset": true})),
        ErrorCode::InvalidArguments
    );
    assert_eq!(
        fails(&mut s, "mark_move", json!({"plan": plan, "view": elevation, "offset": [5000.0, 0.0]})),
        ErrorCode::InvalidArguments
    );
    // Deleting the elevation drops the plan's offset.
    run(&mut s, "view_delete", json!({"view": elevation}));
    assert!(s.project().view(ViewId(plan)).unwrap().mark_offsets.is_empty());
}

#[test]
fn component_symbols_and_bad_values_are_repaired() {
    let mut s = Session::default();
    run(
        &mut s,
        "detail_component_create",
        json!({"name": "Grid bubble", "pieces": [
            {"kind": "stroke", "closed": true, "points": [
                {"u": -0.004, "v": -0.004}, {"u": 0.004, "v": -0.004}, {"u": 0.004, "v": 0.004}, {"u": -0.004, "v": 0.004}]},
            {"kind": "text", "x": 0, "y": 0, "text": "<{Name}>", "size": 0.003}
        ]}),
    );
    let component = s.project().types.values().find(|t| t.name == "Grid bubble").unwrap().id.0;
    let grid = id(&run(&mut s, "grid_create", json!({"start": [0.0, -2.0], "end": [0.0, 8.0], "name": "C"})));
    let plan = first(&mut s, "floor_plan");
    let grid_type = id(&run(&mut s, "types_list", json!({"category": "grid"}))[0]);
    run(&mut s, "type_set", json!({"type": grid_type, "params": {"style": {"symbol": {"Component": component}}}}));
    let texts = element_texts(&mut s, plan, grid);
    assert_eq!(texts.iter().map(|t| t.1.as_str()).collect::<Vec<_>>(), ["<C>"]);
    // A section mark drawn as the component, its fields filled in.
    run(&mut s, "view_create", json!({"kind": "section", "start": [0.0, 3.0], "end": [8.0, 3.0], "name": "Corte Z"}));
    let style = json!({"symbol": {"Component": component}, "top": {"text": ""}, "bottom": {"text": ""}});
    let section_type = s.project().mark_types_of(MarkKind::Section).next().unwrap().id.0;
    run(&mut s, "type_set", json!({"type": section_type, "params": {"style": style}}));
    let texts = free_texts(&mut s, plan);
    assert_eq!(texts.iter().filter(|t| t.1 == "<Corte Z>").count(), 2, "a head at each end: {texts:?}");

    // Hostile values are repaired on load.
    let mut p = s.project().clone();
    let elevation = p.views.values().find(|v| matches!(v.kind, cc_core::ViewKind::Elevation { .. })).unwrap().id;
    if let Some(cc_core::TypeData::Grid { style, .. }) =
        p.types.get_mut(&cc_core::ElementTypeId(grid_type)).map(|t| &mut t.data)
    {
        style.text_size = Some(f64::NAN);
        style.symbol = cc_core::tag::TagSymbol::Component(cc_core::ElementTypeId(424242));
    }
    let v = p.views.get_mut(&ViewId(plan)).unwrap();
    v.mark_offsets.insert(elevation, DVec2::new(f64::INFINITY, 0.0));
    v.mark_offsets.insert(ViewId(55555), DVec2::new(1.0, 0.0));
    v.mark_type = Some(cc_core::ElementTypeId(grid_type));
    v.tags.entry(cc_core::ElementId(grid)).or_default().start_offset = DVec2::new(f64::NAN, 1.0);
    assert_eq!(p.repair(), 6);
    let v = p.view(ViewId(plan)).unwrap();
    assert!(v.mark_offsets.is_empty() && v.mark_type.is_none() && v.tags.is_empty());
    assert!(matches!(&p.types[&cc_core::ElementTypeId(grid_type)].data,
        cc_core::TypeData::Grid { style, .. } if style.symbol == cc_core::tag::TagSymbol::Circle && style.text_size.is_none()));
    assert_eq!(p.repair(), 0);
}
