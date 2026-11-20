//! Roof joins (issue #381): a cross wing joined to a main roof is written
//! as the body the join leaves it (a B-rep), its quantities counting what
//! each roof keeps, with the join in `ConstructCAD_RoofJoin`; import keeps
//! those bodies as they are (the join is not rebuilt).

use cc_core::{Category, Document, EditCtx, Element, ElementId, ElementKind, Project, Roof, RoofShape};
use cc_ifc::{ExportOptions, export, import};
use glam::DVec2;

fn add(e: &mut EditCtx, roof: Roof) -> ElementId {
    let level = e.project().levels[0].id;
    let type_id = e.project().default_type(Category::Roof).unwrap();
    let id = ElementId(e.alloc_id());
    e.add_element(Element {
        id,
        level,
        type_id,
        mark: String::new(),
        comments: String::new(),
        kind: ElementKind::Roof(roof),
        uid: cc_core::Uid::generate(),
        phase_created: Default::default(),
        phase_demolished: None,
        design_option: None,
        paints: Vec::new(),
    })
}

fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<DVec2> {
    vec![DVec2::new(x0, y0), DVec2::new(x1, y0), DVec2::new(x1, y1), DVec2::new(x0, y1)]
}

/// A gable main roof 10 × 8 (ridge along x) and a gable wing from x 3 to 7
/// against its south eave, joined to it, half pitch, 3 m up, no overhang:
/// the wing keeps 28 m², the main roof 76 m². Returns the project and the
/// ids of the main roof and the wing.
pub(crate) fn roof_joins_project() -> (Project, ElementId, ElementId) {
    let mut doc = Document::new(Project::from_template());
    let (main, wing) = doc.edit("roofs", |e| {
        let main =
            add(e, Roof::new(rect(0.0, 0.0, 10.0, 8.0), 3.0, 0.0, RoofShape::Gable { slope: 0.5, ridge_deg: 0.0 }));
        let mut wing = Roof::new(rect(3.0, -6.0, 7.0, 0.0), 3.0, 0.0, RoofShape::Gable { slope: 0.5, ridge_deg: 90.0 });
        wing.join_to = Some(main);
        (main, add(e, wing))
    });
    (doc.project, main, wing)
}

/// The values of the quantities named `name` in `text`, in the order written.
fn quantities(text: &str, name: &str) -> Vec<f64> {
    let head = format!("IFCQUANTITYAREA('{name}',$,$,");
    text.match_indices(&head)
        .filter_map(|(at, _)| text[at + head.len()..].split(',').next()?.parse::<f64>().ok())
        .collect()
}

#[test]
fn joined_roofs_export_what_they_keep_and_import_as_they_are() {
    let (p, main, wing) = roof_joins_project();
    let geom = cc_geom::ModelGeometry::build(&p);
    assert!(geom.roof_joins[&wing].active() && geom.roof_joins[&main].active());
    let text = export(&p, &ExportOptions::default());
    // Two roofs as B-reps, each with its own projected and true areas.
    assert_eq!(text.matches("IFCROOF(").count(), 2);
    let mut projected = quantities(&text, "ProjectedArea");
    projected.sort_by(f64::total_cmp);
    assert_eq!(projected.len(), 2, "{projected:?}");
    assert!((projected[0] - 28.0).abs() < 1e-6 && (projected[1] - 76.0).abs() < 1e-6, "{projected:?}");
    let mut gross = quantities(&text, "GrossArea");
    gross.sort_by(f64::total_cmp);
    assert!((gross[0] - 28.0 * 1.25f64.sqrt()).abs() < 1e-6 && (gross[1] - 76.0 * 1.25f64.sqrt()).abs() < 1e-6);
    // The join: on both roofs, the wing naming the main roof's GlobalId.
    assert_eq!(text.matches("'ConstructCAD_RoofJoin'").count(), 2);
    assert_eq!(text.matches("'JoinedTo'").count(), 1);
    let main_uid = p.element(main).unwrap().uid;
    assert!(text.contains(&format!("IFCLABEL('{}')", cc_ifc::guid::encode(main_uid.0))), "the main roof's GlobalId");

    // Imported, both keep their bodies exactly, as objects of the roof class.
    let (imported, _) = import(&text, Project::from_template()).unwrap();
    assert_eq!(imported.elements.values().filter(|e| e.category() == Category::Roof).count(), 0);
    let roofs: Vec<&Element> = imported
        .elements
        .values()
        .filter(|e| {
            imported.element_type(e.type_id).is_some_and(|t| match &t.data {
                cc_core::TypeData::Component { def, .. } => def.ifc.as_ref().is_some_and(|i| i.entity == "IFCROOF"),
                _ => false,
            })
        })
        .collect();
    assert_eq!(roofs.len(), 2);
    let ig = cc_geom::ModelGeometry::build(&imported);
    let mut volumes: Vec<f64> = roofs.iter().map(|e| ig.volume(e.id)).collect();
    volumes.sort_by(f64::total_cmp);
    let (vw, vm) = (geom.volume(wing), geom.volume(main));
    assert!((volumes[0] - vw).abs() < 1e-6 && (volumes[1] - vm).abs() < 1e-6, "{volumes:?} vs {vw}, {vm}");
}

/// The fixture `fixtures/roof_joins.ccad`: a gable main roof 12 × 8 with a
/// 0.5 m overhang, a gable wing on its south side with a bay joined to the
/// wing (a chain), and a hip wing on its north side, all joined.
pub(crate) fn roof_joins_fixture() -> Project {
    let mut doc = Document::new(Project::from_template());
    doc.edit("roofs", |e| {
        let gable = |fp, overhang, ridge_deg| Roof::new(fp, 3.0, overhang, RoofShape::Gable { slope: 0.5, ridge_deg });
        let main = add(e, gable(rect(0.0, 0.0, 12.0, 8.0), 0.5, 0.0));
        let mut south = gable(rect(3.0, -6.0, 7.0, 0.0), 0.5, 90.0);
        south.join_to = Some(main);
        let south = add(e, south);
        let mut bay = gable(rect(0.0, -4.5, 3.0, -2.5), 0.3, 0.0);
        bay.join_to = Some(south);
        add(e, bay);
        let mut north = Roof::new(rect(6.0, 8.0, 10.0, 13.0), 3.0, 0.5, RoofShape::Hip { slope: 0.5 });
        north.join_to = Some(main);
        add(e, north);
    });
    doc.project
}

/// Writes `fixtures/roof_joins.ccad` from [`roof_joins_fixture`]
/// (`cargo nextest run -p cc_ifc --test main write_roof_joins_fixture --run-ignored only`).
#[test]
#[ignore]
fn write_roof_joins_fixture() {
    let text = cc_core::to_ron(&roof_joins_fixture()).unwrap();
    std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/roof_joins.ccad"), text).unwrap();
}

/// Every join of the fixture is built, and each roof is written as one
/// body with its join, closed (its volume its plan area times its
/// vertical thickness).
#[test]
fn roof_join_fixture_exports_every_join() {
    let p = cc_core::from_ron(include_str!("fixtures/roof_joins.ccad")).unwrap();
    assert_eq!(p.elements.len(), roof_joins_fixture().elements.len());
    let geom = cc_geom::ModelGeometry::build(&p);
    assert_eq!(geom.roof_joins.len(), 4);
    for (id, j) in &geom.roof_joins {
        assert!(j.active(), "{id:?}: {j:?}");
        let t = p.thickness_of(p.element(*id).unwrap()) * 1.25f64.sqrt();
        assert!((geom.volume(*id) - j.plan_area * t).abs() < 1e-6, "{id:?}: {} vs {}", geom.volume(*id), j.plan_area);
    }
    let text = export(&p, &ExportOptions::default());
    assert_eq!(text.matches("IFCROOF(").count(), 4);
    assert_eq!(text.matches("'ConstructCAD_RoofJoin'").count(), 4);
    assert_eq!(text.matches("'JoinedTo'").count(), 3);
}
