//! The templates' reinforced concrete wall types (issue #366): one structure
//! layer of concrete, structural, so walls drawn with them take
//! reinforcement without being told; `wall_rebar_create` says which walls
//! it left out for not being structural.

use cc_core::{Category, ElementId, ElementTypeId, LayerFunction, MaterialClass, Project};
use cc_engine::Session;
use serde_json::{Value, json};

fn run(s: &mut Session, name: &str, args: Value) -> Value {
    s.execute(name, args).unwrap_or_else(|e| panic!("{name}: {e:?}"))
}

fn id(v: &Value) -> ElementId {
    ElementId(v["id"].as_u64().unwrap())
}

fn created(v: &Value) -> Vec<u64> {
    v["created"].as_array().unwrap().iter().map(|x| x.as_u64().unwrap()).collect()
}

/// The wall types whose only layer is a structure layer of concrete, with their thickness.
fn concrete_walls(p: &Project) -> Vec<(ElementTypeId, f64)> {
    p.types
        .values()
        .filter(|t| t.category() == Category::Wall)
        .filter(|t| match t.layers() {
            [layer] => {
                layer.function == LayerFunction::Structure
                    && p.material(&layer.material).is_some_and(|m| m.identity.class == MaterialClass::Concrete)
            }
            _ => false,
        })
        .map(|t| (t.id, t.thickness()))
        .collect()
}

fn structural(s: &Session, wall: ElementId) -> bool {
    s.project().element(wall).and_then(|e| e.as_wall()).unwrap().structural
}

#[test]
fn the_templates_have_three_structural_concrete_wall_types() {
    let mut s = Session::default();
    for (template, sizes) in [("metric", [0.15, 0.20, 0.25]), ("imperial", [6.0, 8.0, 10.0].map(|i| i * 0.0254))] {
        run(&mut s, "project_new", json!({ "template": template }));
        let walls = concrete_walls(s.project());
        let thicknesses: Vec<f64> = walls.iter().map(|(_, t)| *t).collect();
        assert_eq!(thicknesses.len(), 3, "{template}: {thicknesses:?}");
        for (got, want) in thicknesses.iter().zip(sizes) {
            assert!((got - want).abs() < 1e-12, "{template}: {got} != {want}");
        }
        for (ty, _) in &walls {
            let t = s.project().element_type(*ty).unwrap();
            assert!(t.structural_wall(), "{template}: {}", t.name);
            assert!(!t.name.contains('?'), "{template}: a name is missing");
        }
    }
    // The metric names, in the template's language (Portuguese by default).
    assert_eq!(cc_core::template_default("tpl-type-concrete-wall-15"), "Parede de concreto 15 cm");
    assert_eq!(cc_core::template_default("tpl-imp-type-concrete-wall-8"), "Parede de concreto 8\"");
}

#[test]
fn a_concrete_wall_is_structural_and_takes_reinforcement_without_being_told() {
    let mut s = Session::default();
    let (concrete, _) = concrete_walls(s.project())[1];
    let wall =
        id(&run(&mut s, "wall_create", json!({"start": [0, 0], "end": [6, 0], "height": 3.0, "type": concrete.0})));
    assert!(structural(&s, wall));
    // A masonry wall (the default type) is not.
    let masonry = id(&run(&mut s, "wall_create", json!({"start": [0, 4], "end": [6, 4], "height": 3.0})));
    assert!(!structural(&s, masonry));
    // No walls given: the concrete wall is reinforced, the masonry one reported.
    let out = run(&mut s, "wall_rebar_create", json!({}));
    assert_eq!((created(&out).len(), out["walls"].as_u64()), (4, Some(1)), "{out}");
    assert_eq!(out["not_structural"], json!([masonry.0]));
    assert!(out.get("warning").is_none(), "{out}");
    // Every set is hosted by the concrete wall.
    for set in created(&out) {
        let e = s.project().element(ElementId(set)).unwrap();
        assert!(matches!(&e.kind, cc_core::ElementKind::Rebar(r) if r.host == wall));
    }
}

#[test]
fn rebar_on_masonry_walls_alone_says_why_nothing_was_made() {
    let mut s = Session::default();
    let masonry = id(&run(&mut s, "wall_create", json!({"start": [0, 0], "end": [6, 0], "height": 3.0})));
    let out = run(&mut s, "wall_rebar_create", json!({}));
    assert_eq!((created(&out).len(), out["walls"].as_u64(), out["skipped"].as_u64()), (0, Some(0), Some(1)));
    assert_eq!(out["not_structural"], json!([masonry.0]));
    let warning = out["warning"].as_str().expect("a warning");
    assert!(warning.contains("not structural"), "{warning}");
    // Given by id, the wall is reinforced all the same.
    let out = run(&mut s, "wall_rebar_create", json!({"walls": [masonry.0]}));
    assert_eq!(created(&out).len(), 4);
}

#[test]
fn an_explicit_structural_flag_wins_and_a_new_type_changes_nothing() {
    let mut s = Session::default();
    let types = concrete_walls(s.project());
    let (concrete, _) = types[0];
    let wall = id(&run(
        &mut s,
        "wall_create",
        json!({"start": [0, 0], "end": [6, 0], "type": concrete.0, "structural": false}),
    ));
    assert!(!structural(&s, wall));
    let masonry = id(&run(&mut s, "wall_create", json!({"start": [0, 4], "end": [6, 4], "structural": true})));
    assert!(structural(&s, masonry));
    // Changing a wall's type keeps its flag.
    run(&mut s, "element_set", json!({"id": wall.0, "type": types[2].0.0}));
    assert_eq!(s.project().element(wall).unwrap().type_id, types[2].0);
    assert!(!structural(&s, wall));
}
