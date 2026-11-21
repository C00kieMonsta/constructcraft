//! Walls whose layers have vertical extents of their own (see
//! [`cc_core::layer_extent`]): stepped bodies, layer volumes, openings,
//! profiles and arcs.

use crate::ModelGeometry;
use cc_core::wall_profile::WallProfile;
use cc_core::{Category, Document, Element, ElementId, ElementKind, ElementTypeId, Hosted, Layer, Project, Wall};
use glam::DVec2;

fn add(doc: &mut Document, category: Category, kind: ElementKind, type_id: Option<ElementTypeId>) -> ElementId {
    doc.edit("add", |e| {
        let level = e.project().levels[0].id;
        let type_id = type_id.or_else(|| e.project().default_type(category)).unwrap();
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
            plugin_data: Default::default(),
        })
    })
}

/// A copy of the template's default wall type (three layers: plaster,
/// masonry, plaster) with each layer's extent set by `extents`.
fn layered_type(doc: &mut Document, extents: [(f64, Option<f64>); 3]) -> ElementTypeId {
    doc.edit("type", |e| {
        let base = e.project().default_type(Category::Wall).unwrap();
        let mut ty = e.project().element_type(base).unwrap().clone();
        let cc_core::TypeData::Wall { layers, .. } = &mut ty.data else { panic!("a wall type") };
        assert_eq!(layers.len(), 3, "plaster, masonry, plaster");
        for (layer, (base_offset, height)) in layers.iter_mut().zip(extents) {
            layer.base_offset = base_offset;
            layer.height = height;
        }
        ty.name = format!("{} (extents)", ty.name);
        ty.uid = Default::default();
        ty.id = ElementTypeId(e.alloc_id());
        e.add_type(ty)
    })
}

fn layers(doc: &Document, type_id: ElementTypeId) -> Vec<Layer> {
    doc.project.element_type(type_id).unwrap().layers().to_vec()
}

fn wall(start: (f64, f64), end: (f64, f64), profile: Option<WallProfile>) -> ElementKind {
    ElementKind::Wall(Wall {
        start: DVec2::new(start.0, start.1),
        end: DVec2::new(end.0, end.1),
        height: 3.0,
        profile,
        ..Default::default()
    })
}

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 1e-9 * b.abs().max(1.0), "{what}: {a} vs {b}");
}

fn mesh_volume(g: &ModelGeometry, id: ElementId) -> f64 {
    g.body_of(id).map(crate::Solid::volume).sum()
}

/// The layers' volumes add up to the wall's, and its closed mesh holds it.
fn consistent(g: &ModelGeometry, id: ElementId) {
    let w = &g.walls[&id];
    let sum: f64 = w.layer_volumes().iter().sum();
    close(sum, g.volume(id), "layers add up");
    close(mesh_volume(g, id), g.volume(id), "closed mesh");
}

#[test]
fn insulation_starting_a_meter_up() {
    let mut doc = Document::new(Project::from_template());
    let ty = layered_type(&mut doc, [(1.0, None), (0.0, None), (0.0, None)]);
    let id = add(&mut doc, Category::Wall, wall((0.0, 0.0), (6.0, 0.0), None), Some(ty));
    let g = ModelGeometry::build(&doc.project);
    let t: Vec<f64> = layers(&doc, ty).iter().map(|l| l.thickness).collect();
    let w = &g.walls[&id];
    assert!(w.is_banded());
    assert_eq!(w.bands().len(), 2, "the outer layer, then the core and the inner one");
    let volumes = w.layer_volumes();
    close(volumes[0], t[0] * 6.0 * 2.0, "outer layer from 1 m up");
    close(volumes[1], t[1] * 6.0 * 3.0, "core");
    close(volumes[2], t[2] * 6.0 * 3.0, "inner layer");
    close(g.volume(id), t[0] * 12.0 + (t[1] + t[2]) * 18.0, "analytic volume");
    consistent(&g, id);
    assert_eq!(w.layer_range(0), Some((1.0, 3.0)));
    // By material: the two plasters together.
    let by_material = w.volumes_by_material();
    assert_eq!(by_material.len(), 2, "{by_material:?}");
    close(by_material[0].1, volumes[0] + volumes[2], "plaster");
    // Below 1 m the wall is thinner: the left face steps in.
    let c = w.column_at(3.0).unwrap();
    assert!(w.intervals_at_v(c, 3.0, w.half - 1e-3).iter().all(|iv| iv.0 >= 1.0 - 1e-9));
    assert_eq!(w.intervals_at(c, 3.0), vec![(0.0, 3.0)], "the union is the whole wall");
}

#[test]
fn a_core_going_below_the_base() {
    let mut doc = Document::new(Project::from_template());
    let ty = layered_type(&mut doc, [(0.0, None), (-0.5, None), (0.0, None)]);
    let id = add(&mut doc, Category::Wall, wall((0.0, 0.0), (6.0, 0.0), None), Some(ty));
    let g = ModelGeometry::build(&doc.project);
    let t: Vec<f64> = layers(&doc, ty).iter().map(|l| l.thickness).collect();
    let w = &g.walls[&id];
    close(w.layer_volumes()[1], t[1] * 6.0 * 3.5, "core 3.5 m");
    close(g.volume(id), (t[0] + t[2]) * 18.0 + t[1] * 21.0, "analytic volume");
    consistent(&g, id);
    let (lo, hi) = g.body_of(id).filter_map(crate::Solid::bounds).next().unwrap();
    close(lo.z, -0.5, "the core's bottom");
    close(hi.z, 3.0, "top");
    // The nominal base stays (sills are measured from it); the union reaches down.
    close(w.base_z, 0.0, "nominal base");
    assert_eq!(w.intervals_at(&w.columns[0], 1.0), vec![(-0.5, 3.0)]);
}

#[test]
fn a_layer_shorter_than_the_wall() {
    let mut doc = Document::new(Project::from_template());
    // The inner layer 1.2 m high; the outer one asks for more than the wall has.
    let ty = layered_type(&mut doc, [(0.5, Some(9.0)), (0.0, None), (0.0, Some(1.2))]);
    let id = add(&mut doc, Category::Wall, wall((0.0, 0.0), (6.0, 0.0), None), Some(ty));
    let g = ModelGeometry::build(&doc.project);
    let t: Vec<f64> = layers(&doc, ty).iter().map(|l| l.thickness).collect();
    let w = &g.walls[&id];
    let volumes = w.layer_volumes();
    close(volumes[0], t[0] * 6.0 * 2.5, "clamped to the wall's top");
    close(volumes[2], t[2] * 6.0 * 1.2, "1.2 m high");
    consistent(&g, id);
    assert_eq!(w.layer_range(2), Some((0.0, 1.2)));
    assert_eq!(w.bands().len(), 3);
}

#[test]
fn layers_follow_a_sloped_profile() {
    let mut doc = Document::new(Project::from_template());
    // Top from 3 m down to 2 m; the outer layer from 1 m up, the inner one 2.5 m high.
    let ty = layered_type(&mut doc, [(1.0, None), (0.0, None), (0.0, Some(2.5))]);
    let sloped = WallProfile::sloped(6.0, (0.0, 0.0), (3.0, 2.0)).ok();
    let id = add(&mut doc, Category::Wall, wall((0.0, 0.0), (6.0, 0.0), sloped), Some(ty));
    let g = ModelGeometry::build(&doc.project);
    let t: Vec<f64> = layers(&doc, ty).iter().map(|l| l.thickness).collect();
    let w = &g.walls[&id];
    let volumes = w.layer_volumes();
    // The top is 3 - u / 6: the outer layer averages 1.5 m.
    close(volumes[0], t[0] * 9.0, "outer layer");
    close(volumes[1], t[1] * 15.0, "core");
    // 2.5 m to u = 3 where the top comes down to it, then the top.
    close(volumes[2], t[2] * (2.5 * 3.0 + 6.75), "inner layer");
    consistent(&g, id);
    // A column ends where the inner layer meets the top.
    assert!(w.columns.iter().any(|c| (c.left.1 - 3.0).abs() < 1e-9), "{:?}", w.columns);
}

#[test]
fn a_door_cuts_every_layer_present_at_its_height() {
    let mut doc = Document::new(Project::from_template());
    // The outer layer from 1 m up, the core 0.5 m deeper.
    let ty = layered_type(&mut doc, [(1.0, None), (-0.5, None), (0.0, None)]);
    let id = add(&mut doc, Category::Wall, wall((0.0, 0.0), (6.0, 0.0), None), Some(ty));
    let door = Hosted { host: id, offset: 3.0, sill_height: 0.0, flip_facing: false, flip_hand: false };
    let d = add(&mut doc, Category::Door, ElementKind::Door(door), None);
    let g = ModelGeometry::build(&doc.project);
    let t: Vec<f64> = layers(&doc, ty).iter().map(|l| l.thickness).collect();
    let w = &g.walls[&id];
    let o = w.openings.iter().find(|o| o.element == d).unwrap();
    let (width, height) = (o.u1 - o.u0, o.z1 - o.z0);
    assert!(height > 1.0, "the door reaches above the outer layer's bottom");
    let volumes = w.layer_volumes();
    close(volumes[0], t[0] * (6.0 * 2.0 - width * (height - 1.0)), "outer layer");
    // The core runs on under the door.
    close(volumes[1], t[1] * (6.0 * 3.5 - width * height), "core");
    close(volumes[2], t[2] * (6.0 * 3.0 - width * height), "inner layer");
    consistent(&g, id);
    let c = w.column_at(3.0).unwrap();
    assert_eq!(w.layer_intervals(c, 1, 3.0), vec![(-0.5, 0.0), (height, 3.0)]);
}

#[test]
fn layers_of_an_arc_wall() {
    let mut doc = Document::new(Project::from_template());
    let ty = layered_type(&mut doc, [(1.0, None), (-0.5, None), (0.0, Some(2.0))]);
    let mut w = Wall { start: DVec2::new(4.0, 0.0), end: DVec2::new(0.0, 4.0), height: 3.0, ..Default::default() };
    w.bulge = (std::f64::consts::PI / 8.0).tan();
    let id = add(&mut doc, Category::Wall, ElementKind::Wall(w), Some(ty));
    let g = ModelGeometry::build(&doc.project);
    let wg = &g.walls[&id];
    let a = wg.arc.unwrap();
    let ring = |v0: f64, v1: f64| (a.radius_at(v0).powi(2) - a.radius_at(v1).powi(2)).abs() * 0.5 * a.sweep.abs();
    let volumes = wg.layer_volumes();
    for (k, height) in [2.0, 3.5, 2.0].into_iter().enumerate() {
        let (v0, v1, _) = wg.layers[k];
        close(volumes[k], ring(v0, v1) * height, "an exact ring sector");
    }
    let sum: f64 = volumes.iter().sum();
    close(sum, g.volume(id), "layers add up");
    // The chords lose a little.
    let mesh = mesh_volume(&g, id);
    assert!((mesh - sum).abs() < sum * 2e-3, "{mesh} vs {sum}");
}

/// One band spanning the whole wall (extents that change nothing) builds
/// the body the plain path does: same volume, same faces.
#[test]
fn one_band_over_the_whole_wall_is_the_plain_body() {
    let mut doc = Document::new(Project::from_template());
    let plain = doc.project.default_type(Category::Wall).unwrap();
    let banded = layered_type(&mut doc, [(0.0, Some(50.0)); 3]);
    let sloped = || WallProfile::sloped(6.0, (0.0, 0.0), (3.0, 2.0)).ok();
    let a = add(&mut doc, Category::Wall, wall((0.0, 0.0), (6.0, 0.0), sloped()), Some(plain));
    let b = add(&mut doc, Category::Wall, wall((0.0, 5.0), (6.0, 5.0), sloped()), Some(banded));
    for (host, offset) in [(a, 2.0), (b, 2.0)] {
        let door = Hosted { host, offset, sill_height: 0.0, flip_facing: false, flip_hand: false };
        add(&mut doc, Category::Door, ElementKind::Door(door), None);
    }
    let g = ModelGeometry::build(&doc.project);
    let (wa, wb) = (&g.walls[&a], &g.walls[&b]);
    assert!(!wa.is_banded() && wb.is_banded(), "{:?} {:?}", wa.layer_extents, wb.layer_extents);
    close(g.volume(b), g.volume(a), "volume");
    close(mesh_volume(&g, b), mesh_volume(&g, a), "mesh volume");
    let faces = |id: ElementId| g.body_of(id).map(|s| s.faces.len()).sum::<usize>();
    assert_eq!(faces(a), faces(b), "the same faces");
    consistent(&g, b);
}

/// Layers with no extents of their own leave the wall exactly as it was:
/// the same faces, point for point, as a type whose file has no such fields.
#[test]
fn default_extents_change_nothing() {
    let mut doc = Document::new(Project::from_template());
    let plain = doc.project.default_type(Category::Wall).unwrap();
    let text = cc_core::to_ron(&doc.project).unwrap();
    assert!(!text.contains("base_offset"), "default extents are not written");
    let same = layered_type(&mut doc, [(0.0, None); 3]);
    let a = add(&mut doc, Category::Wall, wall((0.0, 0.0), (6.0, 0.0), None), Some(plain));
    let b = add(&mut doc, Category::Wall, wall((0.0, 0.0), (6.0, 0.0), None), Some(same));
    let g = ModelGeometry::build(&doc.project);
    assert!(!g.walls[&b].is_banded());
    let faces = |id: ElementId| -> Vec<Vec<glam::DVec3>> {
        g.body_of(id).flat_map(|s| s.faces.iter().map(|f| f.verts.clone())).collect()
    };
    assert_eq!(faces(a), faces(b));
    assert_eq!(g.volume(a), g.volume(b));
    let sum: f64 = g.walls[&a].layer_volumes().iter().sum();
    close(sum, g.volume(a), "layer volumes of a plain wall");
}

#[test]
fn a_layer_with_no_height_anywhere_is_absent() {
    let mut doc = Document::new(Project::from_template());
    let ty = layered_type(&mut doc, [(5.0, None), (0.0, None), (0.0, None)]);
    let id = add(&mut doc, Category::Wall, wall((0.0, 0.0), (6.0, 0.0), None), Some(ty));
    let g = ModelGeometry::build(&doc.project);
    let w = &g.walls[&id];
    assert_eq!(w.layer_volumes()[0], 0.0);
    assert_eq!(w.layer_range(0), None);
    assert!(w.layer_outlines(0).is_empty());
    consistent(&g, id);
}

#[test]
fn layer_outlines_in_the_elevation_frame() {
    let mut doc = Document::new(Project::from_template());
    let ty = layered_type(&mut doc, [(1.0, None), (-0.5, None), (0.0, Some(2.5))]);
    let sloped = WallProfile::sloped(6.0, (0.0, 0.0), (3.0, 2.0)).ok();
    let id = add(&mut doc, Category::Wall, wall((0.0, 0.0), (6.0, 0.0), sloped), Some(ty));
    let door = Hosted { host: id, offset: 3.0, sill_height: 0.0, flip_facing: false, flip_hand: false };
    add(&mut doc, Category::Door, ElementKind::Door(door), None);
    let g = ModelGeometry::build(&doc.project);
    let w = &g.walls[&id];
    let area = |poly: &[DVec2]| crate::polygon::area(poly);
    // Openings are left in: each outline is the layer's whole face.
    let outer = w.layer_outlines(0);
    assert_eq!(outer.len(), 1);
    close(area(&outer[0]), 9.0, "outer layer");
    close(area(&w.layer_outlines(1)[0]), 15.0 + 3.0, "core, 0.5 m deeper");
    close(area(&w.layer_outlines(2)[0]), 2.5 * 3.0 + 6.75, "inner layer");
}

#[test]
fn a_banded_wall_in_a_mitered_corner() {
    let mut doc = Document::new(Project::from_template());
    let ty = layered_type(&mut doc, [(1.0, None), (-0.5, None), (0.0, Some(2.0))]);
    let sloped = WallProfile::sloped(6.0, (0.0, 0.0), (3.0, 2.0)).ok();
    let a = add(&mut doc, Category::Wall, wall((0.0, 0.0), (6.0, 0.0), sloped), Some(ty));
    let b = add(&mut doc, Category::Wall, wall((6.0, 0.0), (6.0, 4.0), None), Some(ty));
    let g = ModelGeometry::build(&doc.project);
    for id in [a, b] {
        let w = &g.walls[&id];
        assert!(w.start_joined || w.end_joined, "they meet in a mitered corner");
        let mut closed = w.clone();
        (closed.start_joined, closed.end_joined) = (false, false);
        close(closed.to_solid([0; 3]).volume(), w.volume(), "closed mesh at the miter");
        let sum: f64 = w.layer_volumes().iter().sum();
        close(sum, w.volume(), "layers add up");
    }
}
