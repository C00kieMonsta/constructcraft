//! Editable tags (phase 1): tag types say what the tags of doors, windows,
//! rooms, columns, beams and footings write and how they look; each plan
//! moves, turns, retypes or hides the tag of an element. The default tag
//! types draw what plans always drew.

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

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

/// A room of four walls 8 x 6 m with a door, a window, a room and a beam;
/// returns the session, the first floor plan and the door, window, room and beam ids.
fn house() -> (Session, u64, [u64; 4]) {
    let mut s = Session::default();
    let corners = [[0.0, 0.0], [8.0, 0.0], [8.0, 6.0], [0.0, 6.0]];
    let walls: Vec<u64> = (0..4)
        .map(|i| id(&run(&mut s, "wall_create", json!({"start": corners[i], "end": corners[(i + 1) % 4]}))))
        .collect();
    let door = id(&run(&mut s, "door_create", json!({"wall": walls[0]})));
    let window = id(&run(&mut s, "window_create", json!({"wall": walls[1], "mark": "J1"})));
    let room = id(&run(&mut s, "room_create", json!({"point": [4.0, 3.0], "name": "Sala"})));
    let beam = id(&run(&mut s, "beam_create", json!({"start": [0.0, 3.0], "end": [8.0, 3.0]})));
    run(&mut s, "element_set", json!({"id": beam, "mark": "V1"}));
    let plans = run(&mut s, "views_list", json!({"category": "floor_plan"}));
    (s, id(&plans[0]), [door, window, room, beam])
}

fn texts(s: &mut Session, view: u64, element: u64) -> Vec<(DVec2, String, f64, Anchor, f64)> {
    let d = s.drawing(cc_core::ViewId(view)).unwrap();
    d.prims
        .iter()
        .filter_map(|p| match p {
            Prim::Text { pos, text, size, anchor, angle, element: Some(e), .. } if e.0 == element => {
                Some((*pos, text.clone(), *size, *anchor, *angle))
            }
            _ => None,
        })
        .collect()
}

fn annotation_lines(s: &mut Session, view: u64, element: u64) -> usize {
    let d = s.drawing(cc_core::ViewId(view)).unwrap();
    d.prims
        .iter()
        .filter(|p| matches!(p, Prim::Line { style: LineStyle::Annotation, element: Some(e), .. } if e.0 == element))
        .count()
}

/// The prims of a drawing, to compare drawings.
fn prims(s: &mut Session, view: u64) -> Vec<Prim> {
    s.drawing(cc_core::ViewId(view)).unwrap().prims.clone()
}

#[test]
fn default_tag_types_draw_what_plans_always_drew() {
    let (mut s, plan, [door, window, room, _]) = house();
    let structural = id(&run(&mut s, "view_create", json!({"kind": "structural_plan"})));
    let before = (prims(&mut s, plan), prims(&mut s, structural));
    // The door's tag: a circle of 24 lines 2.6 mm across at 1:50, the mark in the middle.
    let door_texts = texts(&mut s, plan, door);
    assert_eq!(door_texts.len(), 1);
    assert_eq!((door_texts[0].1.as_str(), door_texts[0].2, door_texts[0].3), ("1", 2.0, Anchor::Center));
    assert_eq!(annotation_lines(&mut s, plan, door), 24);
    assert_eq!(annotation_lines(&mut s, plan, window), 6, "a hexagon");
    let room_texts: Vec<String> = texts(&mut s, plan, room).into_iter().map(|t| t.1).collect();
    assert_eq!(room_texts[0], "Sala");
    assert_eq!(room_texts.len(), 3);
    // A tag type per target, made with the default style: the plans do not change.
    let mut door_tag = String::new();
    for target in ["door", "window", "room", "column", "beam", "foundation"] {
        let t = run(&mut s, "tag_type_create", json!({"target": target}));
        assert_eq!(t["category"], "tag");
        if target == "door" {
            door_tag = t["name"].as_str().unwrap().to_string();
        }
    }
    assert_eq!(before, (prims(&mut s, plan), prims(&mut s, structural)));
    // Names are unique; targets are known.
    assert_eq!(
        fails(&mut s, "tag_type_create", json!({"target": "door", "name": door_tag})),
        ErrorCode::InvalidArguments
    );
    assert_eq!(fails(&mut s, "tag_type_create", json!({"target": "wall"})), ErrorCode::InvalidArguments);
    let again = run(&mut s, "tag_type_create", json!({"target": "door"}));
    assert_eq!(again["name"], format!("{door_tag} 2"));
}

#[test]
fn styles_set_fields_sizes_and_symbols() {
    let (mut s, plan, [door, window, _, _]) = house();
    let custom = run(
        &mut s,
        "tag_type_create",
        json!({"target": "door", "name": "Big", "style": {
            "lines": [{"text": "{Mark}", "size": 3.5}, {"text": "{Width} x {Height} {Unknown}", "size": 2.0}],
            "symbol": "Rectangle", "width": 0, "height": 0
        }}),
    );
    let t = run(&mut s, "tag_get", json!({"view": plan, "id": door}));
    assert_eq!(t["type"]["id"], custom["id"], "the project's first door tag type");
    let size = t["texts"][1].as_str().unwrap().to_string();
    let (w, h) = size.split_once(" x ").unwrap();
    assert!(w.starts_with(char::is_numeric) && h.ends_with(char::is_numeric), "the opening size: {size}");
    let drawn = texts(&mut s, plan, door);
    assert_eq!(drawn.iter().map(|t| t.2).collect::<Vec<_>>(), [3.5, 2.0]);
    assert_eq!(drawn[1].1, size);
    // A rectangle fitting the text.
    assert_eq!(annotation_lines(&mut s, plan, door), 4);
    // The window keeps its hexagon.
    assert_eq!(annotation_lines(&mut s, plan, window), 6);
    // type_set changes the style; out-of-range values are refused.
    run(&mut s, "type_set", json!({"type": custom["id"], "params": {"style": {"symbol": "Ellipse"}}}));
    assert_eq!(annotation_lines(&mut s, plan, door), 32);
    assert_eq!(
        fails(&mut s, "type_set", json!({"type": custom["id"], "params": {"style": {"width": -3.0}}})),
        ErrorCode::Rejected
    );
    assert_eq!(
        fails(&mut s, "tag_type_create", json!({"target": "door", "style": {"lines": [{"text": "x", "size": 0.0}]}})),
        ErrorCode::InvalidArguments
    );
    // Lines that write nothing draw no tag (a door with no mark, as before).
    run(
        &mut s,
        "type_set",
        json!({"type": custom["id"], "params": {"style": {"lines": [{"text": "{Comments}", "size": 2.0}]}}}),
    );
    assert!(texts(&mut s, plan, door).is_empty());
    assert_eq!(annotation_lines(&mut s, plan, door), 0);
    assert_eq!(run(&mut s, "tag_get", json!({"view": plan, "id": door}))["drawn"], false);
}

#[test]
fn plans_move_turn_hide_and_retype_tags() {
    let (mut s, plan, [door, window, room, _]) = house();
    let home = texts(&mut s, plan, door)[0].0;
    let lines_home = annotation_lines(&mut s, plan, door);
    // Moved: a leader back to the door; turned: the text turns.
    run(&mut s, "tag_set", json!({"view": plan, "ids": [door], "offset": [1.0, -1.0], "rotation": 30}));
    let moved = texts(&mut s, plan, door);
    assert!((moved[0].0 - home - DVec2::new(1.0, -1.0)).length() < 1e-9);
    assert!((moved[0].4 - 30f64.to_radians()).abs() < 1e-9);
    assert_eq!(annotation_lines(&mut s, plan, door), lines_home + 1);
    let t = run(&mut s, "tag_get", json!({"view": plan, "id": door}));
    assert_eq!(t["placement"]["rotation"], 30.0);
    assert_eq!(t["placement"]["offset"], json!([1.0, -1.0]));
    // Leader modes: never, then always (even at home).
    run(&mut s, "tag_set", json!({"view": plan, "ids": [door], "leader": "never"}));
    assert_eq!(annotation_lines(&mut s, plan, door), lines_home);
    run(&mut s, "tag_set", json!({"view": plan, "ids": [door], "offset": null, "rotation": null, "leader": "always"}));
    assert_eq!(texts(&mut s, plan, door)[0].0, home);
    assert_eq!(annotation_lines(&mut s, plan, door), lines_home + 1, "the tag stands off the wall");
    // Hidden: nothing of the tag is drawn; shown again.
    run(&mut s, "tag_set", json!({"view": plan, "ids": [door, window], "hidden": true}));
    assert!(texts(&mut s, plan, door).is_empty() && texts(&mut s, plan, window).is_empty());
    run(&mut s, "tag_set", json!({"view": plan, "ids": [door, window], "reset": true}));
    assert_eq!(texts(&mut s, plan, door)[0].0, home);
    assert!(s.project().view(cc_core::ViewId(plan)).unwrap().tags.is_empty(), "default placements are removed");
    // Another type for one element, and for the plan.
    let square = id(&run(
        &mut s,
        "tag_type_create",
        json!({"target": "door", "name": "Square", "style": {"symbol": "Rectangle"}}),
    ));
    let plain =
        id(&run(&mut s, "tag_type_create", json!({"target": "door", "name": "Plain", "style": {"symbol": "None"}})));
    assert_eq!(annotation_lines(&mut s, plan, door), 4, "the first door tag type");
    run(&mut s, "tag_set", json!({"view": plan, "ids": [door], "type": "Plain"}));
    assert_eq!(annotation_lines(&mut s, plan, door), 0);
    run(&mut s, "tag_set", json!({"view": plan, "ids": [door], "type": null}));
    run(&mut s, "view_set", json!({"view": plan, "tag_types": {"door": plain}}));
    assert_eq!(annotation_lines(&mut s, plan, door), 0);
    assert_eq!(run(&mut s, "view_get", json!({"view": plan}))["tag_types"]["door"]["id"], plain);
    run(&mut s, "view_set", json!({"view": plan, "tag_types": {"door": null}}));
    assert_eq!(annotation_lines(&mut s, plan, door), 4);
    // A tag type of another target, a 3D view, an untagged element: refused.
    assert_eq!(
        fails(&mut s, "tag_set", json!({"view": plan, "ids": [window], "type": square})),
        ErrorCode::InvalidArguments
    );
    assert_eq!(
        fails(&mut s, "view_set", json!({"view": plan, "tag_types": {"window": square}})),
        ErrorCode::InvalidArguments
    );
    let three_d = run(&mut s, "view_create", json!({"kind": "three_d"}));
    assert_eq!(
        fails(&mut s, "tag_set", json!({"view": three_d["id"], "ids": [door], "hidden": true})),
        ErrorCode::InvalidArguments
    );
    let wall = s.project().elements.values().find(|e| e.category() == cc_core::Category::Wall).unwrap().id.0;
    assert_eq!(
        fails(&mut s, "tag_set", json!({"view": plan, "ids": [wall], "hidden": true})),
        ErrorCode::InvalidArguments
    );
    assert_eq!(
        fails(&mut s, "tag_set", json!({"view": plan, "ids": [door], "offset": [5000.0, 0.0]})),
        ErrorCode::InvalidArguments
    );
    // Room tags move too; deleting the type a plan uses drops the choice.
    run(&mut s, "tag_set", json!({"view": plan, "ids": [room], "delta": [0.5, 0.5]}));
    let r = texts(&mut s, plan, room);
    assert!((r[0].0 - DVec2::new(4.5, 3.5) - DVec2::new(0.0, 0.4 * 0.05)).length() < 1e-9, "{:?}", r[0].0);
    run(&mut s, "tag_set", json!({"view": plan, "ids": [door], "type": "Plain"}));
    run(&mut s, "type_delete", json!({"type": plain}));
    let v = s.project().view(cc_core::ViewId(plan)).unwrap();
    assert!(!v.tags.contains_key(&cc_core::ElementId(door)), "{:?}", v.tags);
}

#[test]
fn beams_are_tagged_along_and_a_component_can_be_the_symbol() {
    let (mut s, _, [door, _, _, beam]) = house();
    let structural = id(&run(&mut s, "view_create", json!({"kind": "structural_plan"})));
    let beam_text = texts(&mut s, structural, beam);
    assert_eq!(beam_text.len(), 1);
    assert!(beam_text[0].1.starts_with("V1 "), "{}", beam_text[0].1);
    assert_eq!(beam_text[0].3, Anchor::CenterBottom);
    // A horizontal beam tag stays level when the style reads horizontally; a turned tag turns.
    run(&mut s, "tag_set", json!({"view": structural, "ids": [beam], "rotation": 90}));
    assert!((texts(&mut s, structural, beam)[0].4 - std::f64::consts::FRAC_PI_2).abs() < 1e-9);
    // A detail component as a door's symbol: its text filled in with the mark.
    run(
        &mut s,
        "detail_component_create",
        json!({"name": "Door bubble", "pieces": [
            {"kind": "stroke", "closed": true, "points": [
                {"u": -0.003, "v": -0.003}, {"u": 0.003, "v": -0.003}, {"u": 0.003, "v": 0.003}, {"u": -0.003, "v": 0.003}]},
            {"kind": "text", "x": 0, "y": 0, "text": "P{Mark}", "size": 0.002}
        ]}),
    );
    let component = s.project().types.values().find(|t| t.name == "Door bubble").unwrap().id.0;
    let plans = run(&mut s, "views_list", json!({"category": "floor_plan"}));
    let plan = id(&plans[0]);
    run(
        &mut s,
        "tag_type_create",
        json!({"target": "door", "style": {"symbol": {"Component": component}, "lines": []}}),
    );
    let drawn = texts(&mut s, plan, door);
    assert_eq!(drawn.len(), 1, "{drawn:?}");
    assert_eq!(drawn[0].1, "P1");
    assert!((drawn[0].2 - 2.0).abs() < 1e-9, "a component millimeter is a paper millimeter");
    // The component's square: 6 mm on paper, 0.3 m at 1:50.
    let d = s.drawing(cc_core::ViewId(plan)).unwrap();
    let sides: Vec<f64> = d
        .prims
        .iter()
        .filter_map(|p| match p {
            Prim::Line { a, b, style: LineStyle::Projection, element: Some(e), .. } if e.0 == door => {
                Some(a.distance(*b))
            }
            _ => None,
        })
        .collect();
    assert!(sides.len() >= 4 && sides.iter().filter(|l| (*l - 0.3).abs() < 1e-9).count() == 4, "{sides:?}");
    // A symbol that is not a detail component is refused.
    let wall_type = s.project().default_type(cc_core::Category::Wall).unwrap().0;
    assert_eq!(
        fails(&mut s, "tag_type_create", json!({"target": "door", "style": {"symbol": {"Component": wall_type}}})),
        ErrorCode::InvalidArguments
    );
}

#[test]
fn older_files_move_their_tags_into_placements_and_bad_values_are_repaired() {
    let (mut s, plan, [door, window, _, _]) = house();
    run(&mut s, "tag_move", json!({"view": plan, "id": door, "offset": [1.0, 2.0]}));
    let mut text = cc_core::to_ron(s.project()).unwrap();
    // As a version 28 file wrote it: the offsets in `tag_offsets`.
    let compact: String = text.split_whitespace().collect();
    let tags = format!("tags:{{({door}):(offset:(1.0,2.0),),}},");
    assert!(compact.contains(&tags), "{compact}");
    text = compact.replace(&tags, &format!("tag_offsets:{{({door}):(1.0,2.0),(999999):(1.0,1.0)}},"));
    text = text.replace(&format!("format_version:{}", cc_core::FORMAT_VERSION), "format_version:28");
    let (project, repairs) = cc_core::from_ron_with_repairs(&text).unwrap();
    assert_eq!(repairs, 1, "the offset of a missing element");
    let v = project.view(cc_core::ViewId(plan)).unwrap();
    assert!(v.tag_offsets.is_empty());
    assert_eq!(v.tags[&cc_core::ElementId(door)].offset, DVec2::new(1.0, 2.0));
    // Hostile values: a style out of range and a placement with a type of another target.
    let mut p = project;
    p.ensure_tag_types(&|k: &str| k.to_string());
    let window_tag = p.tag_types_of(cc_core::tag::TagTarget::Window).next().unwrap().id;
    let door_tag = p.tag_types_of(cc_core::tag::TagTarget::Door).next().unwrap().id;
    if let Some(cc_core::TypeData::Tag { style, .. }) = p.types.get_mut(&door_tag).map(|t| &mut t.data) {
        style.lines[0].size = f64::NAN;
        style.symbol = cc_core::tag::TagSymbol::Component(cc_core::ElementTypeId(424242));
    }
    let v = p.views.get_mut(&cc_core::ViewId(plan)).unwrap();
    v.tags.entry(cc_core::ElementId(window)).or_default().tag_type = Some(door_tag);
    v.tags.entry(cc_core::ElementId(door)).or_default().offset = DVec2::new(f64::INFINITY, 0.0);
    v.tag_types.insert(cc_core::tag::TagTarget::Door, window_tag);
    assert_eq!(p.repair(), 5);
    let v = p.view(cc_core::ViewId(plan)).unwrap();
    assert!(v.tags.is_empty() && v.tag_types.is_empty(), "{:?} {:?}", v.tags, v.tag_types);
    assert_eq!(p.repair(), 0);
}
