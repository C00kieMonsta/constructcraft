//! Walls with elevation profiles (see `cc_core::wall_profile`): exported as
//! their outline swept across the wall (clipped at mitered ends) with the
//! profile in `ConstructCAD_WallProfile`, imported back as the same walls;
//! walls whose layers have extents of their own (see `cc_core::layer_extent`)
//! as one item per layer, the extents in `ConstructCAD_WallLayers`.

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

/// A copy of the default wall type (plaster, masonry, plaster) whose outer
/// layer starts 1 m up, whose core runs 0.5 m deeper and whose inner layer
/// is 2 m high.
fn layered_type(e: &mut EditCtx) -> cc_core::ElementTypeId {
    let base = e.project().default_type(Category::Wall).unwrap();
    let mut ty = e.project().element_type(base).unwrap().clone();
    let cc_core::TypeData::Wall { layers, .. } = &mut ty.data else { panic!("a wall type") };
    layers[0].base_offset = 1.0;
    layers[1].base_offset = -0.5;
    layers[2].height = Some(2.0);
    (ty.id, ty.uid, ty.name) = (cc_core::ElementTypeId(e.alloc_id()), Uid::generate(), "Insulated".into());
    e.add_type(ty)
}

/// Walls of [`layered_type`]: a straight one with a door, one with a sloped
/// top in a mitered corner and an arc wall.
fn add_layered_walls(e: &mut EditCtx) {
    let ty = layered_type(e);
    let typed = |e: &mut EditCtx, kind: ElementKind| {
        let id = add(e, Category::Wall, kind);
        if let Some(x) = e.element_mut(id) {
            x.type_id = ty;
        }
        id
    };
    let plain =
        typed(e, ElementKind::Wall(Wall { start: v(0.0, 20.0), end: v(6.0, 20.0), height: 3.0, ..Default::default() }));
    let door = Hosted { host: plain, offset: 3.0, sill_height: 0.0, flip_facing: false, flip_hand: false };
    add(e, Category::Door, ElementKind::Door(door));
    let sloped = WallProfile::sloped(6.0, (0.0, 0.0), (3.0, 2.0)).unwrap();
    typed(e, wall(v(0.0, 26.0), v(6.0, 26.0), sloped));
    // A plain wall meeting its low end in a mitered corner: its layers are clipped there.
    add(e, Category::Wall, ElementKind::Wall(Wall { start: v(6.0, 26.0), end: v(6.0, 30.0), ..Default::default() }));
    let bulge = (std::f64::consts::PI / 8.0).tan();
    typed(
        e,
        ElementKind::Wall(Wall { start: v(14.0, 20.0), end: v(10.0, 24.0), height: 3.0, bulge, ..Default::default() }),
    );
}

/// A sloped wall in an L corner with a plain wall (a mitered joint), a
/// stepped base with a door, a profile short of both ends, an arched top and
/// the walls of [`add_layered_walls`].
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
        add_layered_walls(e);
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
    assert_eq!(a.len(), 5);
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
    assert_eq!(gb.walls.values().map(|w| w.openings.len()).sum::<usize>(), 2);
}

#[test]
fn profiled_walls_export_their_sweep_and_profile() {
    let text = export(&wall_profiles_project(), &ExportOptions::default());
    assert_eq!(text.matches("ConstructCAD_WallProfile").count(), 5);
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
    assert_eq!(profiles(&fixture).len(), 5);
}

/// A wall's uid, its layers' extents (`base_offset`, `height`) and their volumes.
type LayerFigures = (Uid, Vec<(f64, Option<f64>)>, Vec<f64>);

/// Each wall's layers' extents and volumes, by uid.
fn layer_figures(p: &Project) -> Vec<LayerFigures> {
    let g = cc_geom::ModelGeometry::build(p);
    let mut out: Vec<_> = p
        .walls()
        .filter(|(e, _)| p.element_type(e.type_id).is_some_and(|t| cc_core::layer_extent::any(t.layers())))
        .map(|(e, _)| {
            let layers = p.element_type(e.type_id).map(|t| t.layers().to_vec()).unwrap_or_default();
            let extents = layers.iter().map(|l| (l.base_offset, l.height)).collect();
            (e.uid, extents, g.walls[&e.id].layer_volumes())
        })
        .collect();
    out.sort_by_key(|(uid, _, _)| *uid);
    out
}

#[test]
fn layered_walls_round_trip() {
    let original = wall_profiles_project();
    let (imported, _) = crate::round_trip::round_trip(&original);
    let (a, b) = (layer_figures(&original), layer_figures(&imported));
    assert_eq!(a.len(), 3, "a straight, a sloped and an arc wall");
    assert_eq!(a.len(), b.len());
    for ((ua, ea, va), (ub, eb, vb)) in a.iter().zip(&b) {
        assert_eq!(ua, ub);
        assert_eq!(ea, eb, "the extents come back on the type");
        for (x, y) in va.iter().zip(vb) {
            assert!((x - y).abs() < 1e-6, "layer volume {x} vs {y}");
        }
    }
    // One wall type, still named as it was.
    let types: Vec<String> = imported
        .types_of(Category::Wall)
        .filter(|t| cc_core::layer_extent::any(t.layers()))
        .map(|t| t.name.clone())
        .collect();
    assert_eq!(types, vec!["Insulated".to_string()]);
}

#[test]
fn layered_walls_export_one_item_per_layer() {
    let text = export(&wall_profiles_project(), &ExportOptions::default());
    // The extents on the type, the nominal sizes on each of its walls.
    assert_eq!(text.matches("'ConstructCAD_WallLayers'").count(), 4, "one type and three walls");
    assert!(text.contains("'Extents'"));
    // Each of the three walls a body of three items, one per layer.
    let items = |l: &str| l.rsplit('(').next().map_or(0, |list| list.matches('#').count());
    let bodies = text.lines().filter(|l| l.contains("IFCSHAPEREPRESENTATION(") && l.contains("'Body'"));
    assert_eq!(bodies.filter(|l| items(l) == 3).count(), 3);
}
