//! Niches (issue #304): recesses in one face of a wall, hosted like doors
//! and windows, that do not open the wall through.

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_core::{ElementId, ViewId};
use cc_engine::{ErrorCode, Session};
use cc_views::{LineStyle, Prim};
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

/// A wall 4 m long and 3 m high of the 25 cm type, along +x from the origin.
fn wall(s: &mut Session) -> u64 {
    id(&run(s, "wall_create", json!({"start": [0, 0], "end": [4, 0], "type": 4, "height": 3.0})))
}

fn plan(s: &mut Session) -> u64 {
    run(s, "views_list", json!({"category": "floor_plan"}))[0]["id"].as_u64().unwrap()
}

fn volume(s: &mut Session, element: u64) -> f64 {
    run(s, "element_get", json!({"id": element}))["derived"]["volume"].as_f64().unwrap()
}

#[test]
fn a_niche_recesses_one_face_of_its_wall() {
    let mut s = Session::default();
    let w = wall(&mut s);
    let whole = volume(&mut s, w);
    let steps = s.doc.undo_depth();

    // A new project carries a niche type, 60 x 60 x 10, sill 90, which the first niche takes.
    let template = run(&mut s, "types_list", json!({"category": "niche"}));
    assert_eq!(template.as_array().unwrap().len(), 1, "{template}");
    let niche = id(&run(&mut s, "niche_create", json!({"wall": w, "offset": 1.0})));
    assert_eq!(s.doc.undo_depth(), steps + 1, "one undo step");
    let types = run(&mut s, "types_list", json!({"category": "niche"}));
    let ids = |v: &Value| v.as_array().unwrap().iter().map(|t| t["id"].clone()).collect::<Vec<_>>();
    assert_eq!(ids(&types), ids(&template), "no other niche type is made");
    assert_eq!(types[0]["in_use"], 1);

    let got = run(&mut s, "element_get", json!({"id": niche}));
    assert_eq!(got["category"], "niche");
    let d = &got["derived"];
    for (key, value) in [("width", 0.6), ("height", 0.6), ("depth", 0.1), ("volume", 0.036)] {
        assert!((d[key].as_f64().unwrap() - value).abs() < 1e-9, "{key}: {d}");
    }
    assert_eq!(d["face"], "left");
    // The wall loses the recess and nothing else: it is not split or opened.
    assert!((whole - volume(&mut s, w) - 0.036).abs() < 1e-9);
    let g = s.geometry().walls[&ElementId(w)].clone();
    assert!(g.openings.is_empty() && g.columns.len() == 1);
    let r = g.recess_of(ElementId(niche)).unwrap();
    assert!((r.u0 - 0.7).abs() < 1e-9 && (r.u1 - 1.3).abs() < 1e-9);
    assert!((r.z0 - 0.9).abs() < 1e-9 && (r.z1 - 1.5).abs() < 1e-9);
    let (face, back) = r.across(g.half);
    assert!((face - 0.125).abs() < 1e-9 && (back - 0.025).abs() < 1e-9);

    // The other face, lower and typed by hand.
    let small = id(&run(
        &mut s,
        "niche_type_create",
        json!({"name": "Shelf", "width": 0.4, "height": 0.3, "depth": 0.08, "sill": 0.2}),
    ));
    let other =
        id(&run(&mut s, "niche_create", json!({"wall": w, "offset": 3.0, "type": "Shelf", "flip_facing": true})));
    let d = run(&mut s, "element_get", json!({"id": other}))["derived"].clone();
    assert_eq!(d["face"], "right");
    assert!((d["volume"].as_f64().unwrap() - 0.4 * 0.3 * 0.08).abs() < 1e-9);
    let _ = small;
    // A point beside the wall picks the face on its side.
    let by_point = id(&run(&mut s, "niche_create", json!({"point": [2.0, -0.5], "type": "Shelf"})));
    assert_eq!(run(&mut s, "element_get", json!({"id": by_point}))["derived"]["face"], "right");

    // It follows its wall, flips with the rotate command and goes with the wall.
    run(&mut s, "elements_turn", json!({"ids": [niche]}));
    assert_eq!(run(&mut s, "element_get", json!({"id": niche}))["derived"]["face"], "right");
    run(&mut s, "elements_delete", json!({"ids": [w]}));
    for e in [niche, other, by_point] {
        assert!(s.doc.project.element(ElementId(e)).is_none(), "{e} goes with its wall");
    }
}

#[test]
fn a_niche_never_opens_the_wall_through() {
    let mut s = Session::default();
    // The 10 cm partition: a 30 cm deep niche stops 1 cm short of its other face.
    let thin = id(&run(&mut s, "wall_create", json!({"start": [0, 0], "end": [4, 0], "type": 5, "height": 3.0})));
    run(&mut s, "niche_type_create", json!({"name": "Deep", "depth": 0.3}));
    let niche = id(&run(&mut s, "niche_create", json!({"wall": thin, "type": "Deep"})));
    let d = run(&mut s, "element_get", json!({"id": niche}))["derived"].clone();
    assert!((d["depth"].as_f64().unwrap() - 0.09).abs() < 1e-9, "{d}");
    assert!(s.geometry().walls[&ElementId(thin)].openings.is_empty());

    // One that would overlap a window cuts nothing; clear of it, it does.
    let w = wall(&mut s);
    run(&mut s, "wall_create", json!({"start": [0, 5], "end": [4, 5], "type": 4, "height": 3.0}));
    let window = id(&run(&mut s, "window_create", json!({"wall": w, "offset": 2.0})));
    let over = id(&run(&mut s, "niche_create", json!({"wall": w, "offset": 2.0, "sill": 1.0})));
    assert!(run(&mut s, "element_get", json!({"id": over}))["derived"]["volume"].is_null());
    let under = id(&run(&mut s, "niche_create", json!({"wall": w, "offset": 2.0, "sill": 0.1})));
    assert!(run(&mut s, "element_get", json!({"id": under}))["derived"]["volume"].as_f64().unwrap() > 0.0);
    let g = &s.geometry().walls[&ElementId(w)];
    assert_eq!((g.openings.len(), g.recesses.len()), (1, 1));
    assert_eq!(g.openings[0].element, ElementId(window));

    // Sizes out of range and names in use are refused.
    assert_eq!(fails(&mut s, "niche_type_create", json!({"depth": 0.0})), ErrorCode::InvalidArguments);
    assert_eq!(fails(&mut s, "niche_type_create", json!({"width": 60.0})), ErrorCode::InvalidArguments);
    assert_eq!(fails(&mut s, "niche_type_create", json!({"name": "Deep"})), ErrorCode::InvalidArguments);
    // A window type is not a niche type.
    assert!(s.execute("niche_create", json!({"wall": w, "type": "Janela 120x120"})).is_err());
}

#[test]
fn plans_schedules_and_ifc_show_the_niche() {
    let mut s = Session::default();
    let w = wall(&mut s);
    let niche = id(&run(&mut s, "niche_create", json!({"wall": w, "offset": 1.0})));
    let view = plan(&mut s);
    let lines = |s: &mut Session, style: LineStyle| {
        let d = s.drawing(ViewId(view)).unwrap();
        d.prims
            .iter()
            .filter(|p| matches!(p, Prim::Line { element: Some(e), style: st, .. } if e.0 == niche && *st == style))
            .count()
    };
    // The plan cuts at 1.2 m, through the niche (0.9 to 1.5): its two sides and its back, cut.
    assert_eq!(lines(&mut s, LineStyle::Cut), 3);
    let d = s.drawing(ViewId(view)).unwrap();
    let cover = d
        .prims
        .iter()
        .any(|p| matches!(p, Prim::Fill { element: Some(e), color, .. } if e.0 == niche && color[3] == 255));
    assert!(cover, "the wall's fill is covered where the niche is");
    // The wall's left face is open along the niche: no wall line crosses its mouth.
    let mouth = glam::DVec2::new(1.0, 0.125);
    let closed = d.prims.iter().any(|p| match p {
        Prim::Line { a, b, element: Some(e), style, .. } if e.0 == w && *style == LineStyle::Cut => {
            cc_geom::polygon::closest_on_segment(*a, *b, mouth).0.distance(mouth) < 1e-6
        }
        _ => false,
    });
    assert!(!closed, "the face line stops at the niche");
    // Above the cut it is dashed.
    run(&mut s, "element_set", json!({"id": niche, "params": {"sill_height": 2.0}}));
    assert_eq!((lines(&mut s, LineStyle::Cut), lines(&mut s, LineStyle::Hidden)), (0, 3));
    run(&mut s, "element_set", json!({"id": niche, "params": {"sill_height": 0.9}}));

    // A niche schedule lists it with the volume it takes out; the wall's area is untouched.
    let schedule = id(&run(&mut s, "view_create", json!({"kind": "schedule", "category": "niche"})));
    let table = run(&mut s, "schedule_table", json!({"view": schedule}));
    assert_eq!(table["rows"].as_array().unwrap().len(), 1, "{table}");
    assert!(table.to_string().contains("0.036") || table.to_string().contains("0,036"), "{table}");
    let walls = id(&run(&mut s, "view_create", json!({"kind": "schedule", "category": "wall"})));
    let table = run(&mut s, "schedule_table", json!({"view": walls})).to_string();
    assert!(table.contains("12.00") || table.contains("12,00"), "4 m x 3 m, no opening: {table}");

    // IFC: an opening element that is a recess, as deep as the niche, voiding the wall.
    let dir = std::env::temp_dir().join(format!("ccad-niche-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("niche.ifc");
    run(&mut s, "ifc_export", json!({"path": path}));
    let ifc = std::fs::read_to_string(&path).unwrap();
    let recess = ifc.lines().find(|l| l.contains("IFCOPENINGELEMENT") && l.contains(".RECESS.")).expect("a recess");
    assert!(recess.contains("Nicho") || recess.contains("Niche"), "{recess}");
    assert_eq!(ifc.matches("IFCRELVOIDSELEMENT").count(), 1);
    assert!(ifc.contains("ConstructCAD_Niche"));
    assert!(!ifc.contains("IFCWINDOW("), "a niche is not a window");

    // It saves and loads back, and undo takes it away.
    let ron = cc_core::to_ron(&s.doc.project).unwrap();
    let back = cc_core::from_ron(&ron).unwrap();
    assert_eq!(back.element(ElementId(niche)), s.doc.project.element(ElementId(niche)));
    let geom = cc_geom::ModelGeometry::build(&back);
    assert_eq!(geom.walls[&ElementId(w)].recesses.len(), 1);
}

/// A niche component made with `component_create` (the New Component
/// dialog's path) has no pieces, as its recess is its shape: it was refused
/// as "the component has no pieces", and a niche saved to a `.ccomp` file
/// did not load back.
#[test]
fn a_new_niche_component_needs_no_pieces() {
    let mut s = Session::default();
    let w = wall(&mut s);
    let made = run(&mut s, "component_create", json!({"name": "Nicho X", "kind": "Niche"}));
    let ty = id(&made);
    let niche = id(&run(&mut s, "niche_create", json!({"wall": w, "offset": 1.0, "type": ty})));
    let d = run(&mut s, "element_get", json!({"id": niche}))["derived"].clone();
    assert!((d["volume"].as_f64().unwrap() - 0.036).abs() < 1e-9, "{d}");

    // It saves to a component file and loads into another project.
    let dir = std::env::temp_dir().join(format!("ccad-niche-comp-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("nicho.ccomp");
    run(&mut s, "component_save", json!({"component": "Nicho X", "path": path}));
    let mut other = Session::default();
    run(&mut other, "component_load", json!({"path": path}));
    let types = run(&mut other, "types_list", json!({"category": "niche"}));
    assert!(types.as_array().unwrap().iter().any(|t| t["name"] == "Nicho X"), "{types}");
}

/// The niche types of a project, as `(name, width, height, depth, sill)`.
fn niche_types(s: &Session) -> Vec<(String, f64, f64, f64, f64)> {
    s.project()
        .types_of(cc_core::Category::Niche)
        .map(|t| {
            let (w, h) = t.opening_size().unwrap();
            (t.name.clone(), w, h, t.opening_depth().unwrap(), t.default_sill())
        })
        .collect()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// Issue #304: every new project, metric or imperial, carries one niche
/// type, so the Niche tool has something to place; `niche_create` with no
/// type uses it and makes no other.
#[test]
fn new_projects_carry_a_niche_type() {
    use cc_core::units::INCH;
    let inches = |v: f64| v * INCH;
    for (template, sizes) in
        [("metric", (0.6, 0.6, 0.1, 0.9)), ("imperial", (inches(24.0), inches(24.0), inches(4.0), inches(36.0)))]
    {
        let mut s = Session::default();
        run(&mut s, "project_new", json!({ "template": template }));
        let types = niche_types(&s);
        assert_eq!(types.len(), 1, "{template}: {types:?}");
        let (_, w, h, d, sill) = types[0].clone();
        assert!(close(w, sizes.0) && close(h, sizes.1) && close(d, sizes.2) && close(sill, sizes.3), "{types:?}");
        // The same sizes `niche_type_create` gives by default.
        let made = id(&run(&mut s, "niche_type_create", json!({})));
        let made = s.project().element_type(cc_core::ElementTypeId(made)).unwrap();
        assert_eq!(made.opening_size(), Some((w, h)), "{template}");
        assert_eq!((made.opening_depth(), made.default_sill()), (Some(d), sill), "{template}");
        run(&mut s, "undo", json!({}));

        let w = id(&run(&mut s, "wall_create", json!({"start": [0, 0], "end": [4, 0], "type": 4, "height": 3.0})));
        let niche = id(&run(&mut s, "niche_create", json!({"wall": w})));
        assert_eq!(niche_types(&s), types, "{template}: no new type");
        let ty = s.project().element(ElementId(niche)).unwrap().type_id;
        assert_eq!(s.project().element_type(ty).unwrap().name, types[0].0);
        let got = run(&mut s, "element_get", json!({"id": niche}));
        assert!(close(got["params"]["sill_height"].as_f64().unwrap(), sizes.3), "{got}");
    }
}

/// A project with no niche type (an older file) still gets the default
/// one with its first niche, in the same undo step.
#[test]
fn a_project_without_a_niche_type_gets_one() {
    let mut project = cc_engine::new_project();
    project.types.retain(|_, t| t.category() != cc_core::Category::Niche);
    let mut s = Session::new(cc_core::Document::new(project));
    let w = wall(&mut s);
    let steps = s.doc.undo_depth();
    assert!(niche_types(&s).is_empty());
    run(&mut s, "niche_create", json!({"wall": w, "offset": 1.0}));
    assert_eq!(s.doc.undo_depth(), steps + 1, "the type and the niche in one undo step");
    let types = niche_types(&s);
    assert_eq!(types.len(), 1, "{types:?}");
    let (_, w, h, d, sill) = types[0].clone();
    assert!(close(w, 0.6) && close(h, 0.6) && close(d, 0.1) && close(sill, 0.9), "{types:?}");
}

/// Issue 402: a plan tells a niche cut halfway up its wall from a slot cut
/// down to the floor (the sill's edge, seen below the cut), and one above
/// the cut from one below it (longer dashes).
#[test]
fn plans_tell_niches_apart_by_their_height() {
    let mut s = Session::default();
    // 15 cm masonry, 3 m long and high; the plan cuts at 1.2 m.
    let w = id(&run(&mut s, "wall_create", json!({"start": [0, 0], "end": [3, 0], "type": 3, "height": 3.0})));
    let niche = |s: &mut Session, offset: f64, sill: f64| {
        id(&run(s, "niche_create", json!({"wall": w, "offset": offset, "sill": sill})))
    };
    // The default niche is 60 cm high: 0 to 0.6 below the cut, 0.9 to 1.5
    // through it, 2.4 to 3.0 above it.
    let low = niche(&mut s, 0.4, 0.0);
    let mid = niche(&mut s, 1.1, 0.9);
    let high = niche(&mut s, 1.8, 2.4);
    run(&mut s, "niche_type_create", json!({"name": "Slot", "height": 2.0}));
    let slot = id(&run(&mut s, "niche_create", json!({"wall": w, "offset": 2.5, "sill": 0.0, "type": "Slot"})));
    let view = plan(&mut s);
    let d = s.drawing(ViewId(view)).unwrap();
    let lines = |of: u64| -> Vec<(LineStyle, Option<cc_core::LinePattern>)> {
        d.prims
            .iter()
            .filter_map(|p| match p {
                Prim::Line { element: Some(e), style, pen, .. } if e.0 == of => {
                    Some((*style, pen.and_then(|p| p.pattern)))
                }
                _ => None,
            })
            .collect()
    };
    let count = |of: u64, style: LineStyle| lines(of).iter().filter(|l| l.0 == style).count();

    // Cut halfway up: cut sides and back, and the sill's edge along the face.
    assert_eq!(count(mid, LineStyle::Cut), 3, "{:?}", lines(mid));
    assert_eq!(count(mid, LineStyle::Projection), 1, "{:?}", lines(mid));
    // Cut down to the wall's base: no sill to see.
    assert_eq!(count(slot, LineStyle::Cut), 3, "{:?}", lines(slot));
    assert_eq!(count(slot, LineStyle::Projection), 0, "{:?}", lines(slot));
    // Below the cut: hidden lines in their own pattern.
    assert_eq!(lines(low), vec![(LineStyle::Hidden, None); 3]);
    // Above the cut: hidden lines with longer dashes.
    assert_eq!(lines(high), vec![(LineStyle::Hidden, Some(cc_core::LinePattern::Dash)); 3]);
}
