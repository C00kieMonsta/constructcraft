//! Walls with elevation profiles (see `cc_core::wall_profile`): exported as
//! their outline swept across the wall (clipped at mitered ends) with the
//! profile in `ConstructCAD_WallProfile`, imported back as the same walls.

use cc_core::sketch::{Sketch, SketchCurve};
use cc_core::wall_profile::WallProfile;
use cc_core::{Category, Document, EditCtx, Element, ElementId, ElementKind, Hosted, Project, Uid, Wall};
use cc_ifc::{ExportOptions, export};
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

fn wall(start: DVec2, end: DVec2, profile: WallProfile) -> ElementKind {
    ElementKind::Wall(Wall { start, end, height: 3.0, profile: Some(profile), ..Default::default() })
}

/// A sloped wall in an L corner with a plain wall (a mitered joint), a
/// stepped base with a door, a profile short of both ends and an arched top.
pub(crate) fn wall_profiles_project() -> Project {
    let mut doc = Document::new(Project::from_template());
    doc.edit("profiles", |e| {
        let sloped = WallProfile::sloped(6.0, (0.0, 0.0), (3.0, 2.0)).unwrap();
        add(e, Category::Wall, wall(v(0.0, 0.0), v(6.0, 0.0), sloped));
        add(e, Category::Wall, ElementKind::Wall(Wall { start: v(6.0, 0.0), end: v(6.0, 4.0), ..Default::default() }));
        let stepped = WallProfile::from_outline(
            vec![v(0.0, 0.0), v(2.0, 0.0), v(2.0, 0.5), v(6.0, 0.5), v(6.0, 3.0), v(0.0, 3.0)],
            6.0,
        )
        .unwrap();
        let host = add(e, Category::Wall, wall(v(0.0, 8.0), v(6.0, 8.0), stepped));
        let door = Hosted { host, offset: 4.0, sill_height: 0.5, flip_facing: false, flip_hand: false };
        add(e, Category::Door, ElementKind::Door(door));
        let short = WallProfile::from_outline(vec![v(1.0, -0.5), v(4.0, -0.5), v(4.0, 2.0), v(1.0, 2.0)], 6.0).unwrap();
        add(e, Category::Wall, wall(v(0.0, 14.0), v(6.0, 14.0), short));
        let arch = Sketch {
            curves: vec![
                SketchCurve::Line { a: v(0.0, 0.0), b: v(4.0, 0.0) },
                SketchCurve::Line { a: v(4.0, 0.0), b: v(4.0, 2.0) },
                SketchCurve::Arc { a: v(4.0, 2.0), b: v(0.0, 2.0), bulge: 1.0 },
                SketchCurve::Line { a: v(0.0, 2.0), b: v(0.0, 0.0) },
            ],
        };
        add(e, Category::Wall, wall(v(10.0, 0.0), v(10.0, 4.0), WallProfile::from_sketch(arch, 4.0).unwrap()));
    });
    doc.project
}

fn profiles(p: &Project) -> Vec<(Uid, WallProfile)> {
    let mut out: Vec<(Uid, WallProfile)> = p.walls().filter_map(|(e, w)| Some((e.uid, w.profile.clone()?))).collect();
    out.sort_by_key(|(uid, _)| *uid);
    out
}

#[test]
fn profiled_walls_round_trip() {
    let original = wall_profiles_project();
    crate::round_trip::check(&original);
    let (imported, _) = crate::round_trip::round_trip(&original);
    let (a, b) = (profiles(&original), profiles(&imported));
    assert_eq!(a.len(), 4);
    assert_eq!(a.len(), b.len());
    for ((ua, pa), (ub, pb)) in a.iter().zip(&b) {
        assert_eq!(ua, ub);
        assert_eq!(pa.outline.len(), pb.outline.len());
        for (p, q) in pa.outline.iter().zip(&pb.outline) {
            assert!(p.distance(*q) < 1e-9, "{p} vs {q}");
        }
        assert_eq!(pa.sketch.is_some(), pb.sketch.is_some(), "the curves come back");
    }
    // The same bodies: equal volumes, the door cut in its stepped wall.
    let (ga, gb) = (cc_geom::ModelGeometry::build(&original), cc_geom::ModelGeometry::build(&imported));
    let volumes = |g: &cc_geom::ModelGeometry, p: &Project| {
        let mut v: Vec<(Uid, f64)> = p.walls().map(|(e, _)| (e.uid, g.volume(e.id))).collect();
        v.sort_by_key(|(uid, _)| *uid);
        v
    };
    for ((ua, va), (ub, vb)) in volumes(&ga, &original).iter().zip(&volumes(&gb, &imported)) {
        assert_eq!(ua, ub);
        assert!((va - vb).abs() < 1e-6, "{va} vs {vb}");
    }
    assert_eq!(gb.walls.values().map(|w| w.openings.len()).sum::<usize>(), 1);
}

#[test]
fn profiled_walls_export_their_sweep_and_profile() {
    let text = export(&wall_profiles_project(), &ExportOptions::default());
    assert_eq!(text.matches("ConstructCAD_WallProfile").count(), 4);
    // The mitered ends of the sloped wall and its neighbour: one clipped sweep.
    assert!(text.contains("IFCBOOLEANCLIPPINGRESULT(.DIFFERENCE."), "a mitered end is clipped");
    assert!(text.contains("IFCHALFSPACESOLID("));
    assert!(text.contains("'Clipping'"));
}

/// Writes `fixtures/wall_profiles.ccad` from [`wall_profiles_project`]
/// (`cargo nextest run -p cc_ifc --test main write_wall_profiles_fixture --run-ignored only`).
#[test]
#[ignore]
fn write_wall_profiles_fixture() {
    let text = cc_core::to_ron(&wall_profiles_project()).unwrap();
    std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/wall_profiles.ccad"), text).unwrap();
}

#[test]
fn wall_profile_fixture_round_trips() {
    let fixture = cc_core::from_ron(include_str!("fixtures/wall_profiles.ccad")).unwrap();
    crate::round_trip::check(&fixture);
    assert_eq!(profiles(&fixture).len(), 4);
}
