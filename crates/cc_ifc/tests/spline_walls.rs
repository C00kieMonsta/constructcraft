//! Spline walls (see `cc_core::wall_spline`), first stage: no sweep
//! describes their bodies, so each is exported as an `IfcWall` whose body is
//! a closed shell of facets (no axis), typed by its wall type with its
//! material layer set. Such a wall comes back from a file by the import's
//! fallback, as a component that keeps its shape and its class; reading it
//! back as a spline wall is a later stage.

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_core::wall_spline::WallSpline;
use cc_core::{Category, Document, EditCtx, Element, ElementId, ElementKind, Project, Wall};
use cc_ifc::{ExportOptions, export, import};
use glam::DVec2;

fn add(e: &mut EditCtx, category: Category, kind: ElementKind) -> ElementId {
    let level = e.project().levels[0].id;
    let type_id = e.project().default_type(category).unwrap();
    let id = ElementId(e.alloc_id());
    e.add_element(Element {
        id,
        level,
        type_id,
        mark: String::new(),
        comments: String::new(),
        kind,
        uid: Default::default(),
        phase_created: Default::default(),
        phase_demolished: None,
        design_option: None,
        paints: Vec::new(),
    })
}

fn v(x: f64, y: f64) -> DVec2 {
    DVec2::new(x, y)
}

fn straight(start: DVec2, end: DVec2) -> ElementKind {
    ElementKind::Wall(Wall { start, end, ..Default::default() })
}

fn spline(control: Vec<DVec2>) -> ElementKind {
    ElementKind::Wall(Wall { spline: Some(WallSpline { control }), ..Default::default() })
}

/// A free spline wall through four points; a room of three straight walls
/// closed by a spline (mitered corners at both its ends); and a straight
/// wall ending on the curved side of a third.
pub(crate) fn spline_walls_project() -> (Project, Vec<ElementId>) {
    let mut doc = Document::new(Project::from_template());
    let ids = doc.edit("splines", |e| {
        let through = WallSpline::through(&[v(0.0, -8.0), v(3.0, -6.5), v(6.0, -9.0), v(9.0, -7.5)]).unwrap();
        let free = add(e, Category::Wall, ElementKind::Wall(Wall { spline: Some(through), ..Default::default() }));
        add(e, Category::Wall, straight(v(0.0, 4.0), v(0.0, 0.0)));
        add(e, Category::Wall, straight(v(0.0, 0.0), v(6.0, 0.0)));
        add(e, Category::Wall, straight(v(6.0, 0.0), v(6.0, 4.0)));
        let closing = add(
            e,
            Category::Wall,
            spline(vec![v(0.0, 4.0), v(1.0, 4.0), v(2.0, 5.0), v(3.0, 4.0), v(4.0, 3.0), v(5.0, 4.0), v(6.0, 4.0)]),
        );
        let k = 0.552_284_749_8 * 4.0;
        let host = add(
            e,
            Category::Wall,
            spline(vec![
                v(24.0, 0.0),
                v(24.0, k),
                v(20.0 + k, 4.0),
                v(20.0, 4.0),
                v(20.0 - k, 4.0),
                v(16.0, k),
                v(16.0, 0.0),
            ]),
        );
        add(e, Category::Wall, straight(v(20.0, 1.0), v(20.0, 4.0)));
        vec![free, closing, host]
    });
    (doc.project, ids)
}

fn options() -> ExportOptions {
    ExportOptions { time_stamp: Some("2026-01-01T00:00:00".into()), file_name: "t.ifc".into() }
}

#[test]
fn spline_walls_are_written_as_closed_shells_and_come_back() {
    let (original, splines) = spline_walls_project();
    assert_eq!(original.repairs_needed(), 0);
    let text = export(&original, &options());
    // Seven walls, three of them shells of facets with no predefined sweep.
    assert_eq!(text.matches("IFCWALL(").count(), 7);
    assert_eq!(text.matches("IFCFACETEDBREP(").count(), 3);
    assert!(text.matches(",.NOTDEFINED.);").count() >= 3);
    assert!(text.contains("'Body','Brep'"));
    // Their layers are their type's: one layer set, used by the straight walls only.
    assert_eq!(text.matches("IFCMATERIALLAYERSET(").count(), 1);
    // Their quantities are those of the bodies written.
    let geom = cc_geom::ModelGeometry::build(&original);
    for id in &splines {
        let volume = geom.volume(*id);
        let mesh: f64 = {
            let mut closed = geom.walls[id].clone();
            closed.start_joined = false;
            closed.end_joined = false;
            closed.face_gaps.clear();
            closed.to_solid([0; 3]).volume()
        };
        assert!(volume > 0.5 && (mesh - volume).abs() < 1e-9 * volume, "{id:?}: {mesh} vs {volume}");
    }
    // The file reads back without error: straight walls as walls, the spline walls by the
    // fallback, as components of the same shape that keep their ids.
    let (imported, report) = import(&text, Project::from_template()).unwrap();
    assert_eq!(report.schema, "IFC4");
    assert_eq!(report.skipped_total(), 0, "{:?}", report.skipped);
    assert_eq!(imported.walls().count(), 4);
    assert_eq!(report.generic.get("IfcWall").copied(), Some(3), "{:?}", report.generic);
    assert_eq!(imported.repairs_needed(), 0);
    let back = cc_geom::ModelGeometry::build(&imported);
    for id in &splines {
        let uid = original.element(*id).unwrap().uid;
        let found = imported.elements.values().find(|e| e.uid == uid).expect("the wall comes back");
        assert_eq!(found.category(), Category::Component);
        let (a, b) = (geom.volume(*id), back.volume(found.id));
        assert!((a - b).abs() < 1e-6 * a, "{id:?}: {a} vs {b}");
    }
    // And writes again.
    let again = export(&imported, &options());
    assert!(again.contains("IFCWALL("));
}

/// Writes `fixtures/spline_walls.ccad` from [`spline_walls_project`]
/// (`cargo nextest run -p cc_ifc --test main write_spline_walls_fixture --run-ignored only`).
#[test]
#[ignore]
fn write_spline_walls_fixture() {
    let text = cc_core::to_ron(&spline_walls_project().0).unwrap();
    std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/spline_walls.ccad"), text).unwrap();
}

#[test]
fn spline_wall_fixture_exports() {
    let (fixture, repairs) = cc_core::from_ron_with_repairs(include_str!("fixtures/spline_walls.ccad")).unwrap();
    assert_eq!(repairs, 0);
    assert_eq!(fixture.walls().filter(|(_, w)| w.spline.is_some()).count(), 3);
    let text = export(&fixture, &options());
    assert_eq!(text.matches("IFCFACETEDBREP(").count(), 3);
    assert!(import(&text, Project::from_template()).is_ok());
}
