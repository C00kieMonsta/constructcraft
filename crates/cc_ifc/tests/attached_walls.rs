//! Walls attached to a roof, a floor or a ceiling (`Wall::attach_top`):
//! exported as their extrusion cut by a half space through each stretch of
//! their top (`IfcBooleanClippingResult`, representation `Clipping`; bounded
//! to its stretch where the plane would cut the wall elsewhere), with
//! quantities of the cut body; imported back as walls with an elevation
//! profile of the same shape. Expected values are worked out by hand from
//! the model's dimensions.

use cc_core::{Category, Ceiling, Document, EditCtx, Element, ElementId, ElementKind, Floor, Project, Roof, Wall};
use cc_ifc::parse::{Entity, StepFile, parse};
use cc_ifc::{ExportOptions, export};
use glam::{DAffine3, DMat3, DVec2, DVec3};

fn add(e: &mut EditCtx, category: Category, mark: &str, kind: ElementKind) -> ElementId {
    let level = e.project().levels[0].id;
    let type_id = e.project().default_type(category).unwrap();
    let id = ElementId(e.alloc_id());
    e.add_element(Element {
        id,
        level,
        type_id,
        mark: mark.into(),
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

fn wall(start: DVec2, end: DVec2, height: f64) -> ElementKind {
    ElementKind::Wall(Wall { start, end, height, ..Default::default() })
}

fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<DVec2> {
    vec![v(x0, y0), v(x1, y0), v(x1, y1), v(x0, y1)]
}

/// The issue's house: four walls 2.5 m high around 6 × 4 m under a gable
/// roof (ridge along x, slope 0.5, 0.3 m overhang over the walls' outer
/// faces, its eaves at 2.5 m) they are attached to; a wall attached to a
/// floor slab over part of it (2.8 m top) and one under a ceiling (2.6 m).
pub(crate) fn attached_walls_project() -> Project {
    let mut doc = Document::new(Project::from_template());
    doc.edit("attached walls", |e| {
        let corners = [v(0.0, 0.0), v(6.0, 0.0), v(6.0, 4.0), v(0.0, 4.0)];
        let marks = ["south", "gable", "north", "west"];
        let walls: Vec<ElementId> =
            (0..4).map(|i| add(e, Category::Wall, marks[i], wall(corners[i], corners[(i + 1) % 4], 2.5))).collect();
        let t = thickness(e.project());
        let shape = cc_core::RoofShape::Gable { slope: 0.5, ridge_deg: 0.0 };
        let roof = Roof::new(rect(-t / 2.0, -t / 2.0, 6.0 + t / 2.0, 4.0 + t / 2.0), 2.5, 0.3, shape);
        let roof = add(e, Category::Roof, "roof", ElementKind::Roof(roof));
        let floor = add(e, Category::Floor, "floor", ElementKind::Floor(Floor::new(rect(-1.0, 9.0, 3.0, 11.0), 2.8)));
        let under_floor = add(e, Category::Wall, "under floor", wall(v(0.0, 10.0), v(6.0, 10.0), 3.0));
        let ceiling =
            Ceiling { boundary: rect(-1.0, 19.0, 5.0, 21.0), height_offset: 2.6, cuts: Vec::new(), sketch: None };
        let ceiling = add(e, Category::Ceiling, "ceiling", ElementKind::Ceiling(ceiling));
        let under_ceiling = add(e, Category::Wall, "under ceiling", wall(v(0.0, 20.0), v(4.0, 20.0), 3.0));
        let attached = walls.iter().map(|w| (*w, roof)).chain([(under_floor, floor), (under_ceiling, ceiling)]);
        for (w, target) in attached {
            if let Some(ElementKind::Wall(w)) = e.element_mut(w).map(|x| &mut x.kind) {
                w.attach_top = Some(target);
            }
        }
    });
    doc.project
}

/// The default wall type's thickness.
fn thickness(p: &Project) -> f64 {
    p.element_type(p.default_type(Category::Wall).unwrap()).unwrap().thickness()
}

/// The default floor type's thickness.
fn floor_thickness(p: &Project) -> f64 {
    p.element_type(p.default_type(Category::Floor).unwrap()).unwrap().thickness()
}

/// The underside of the roof over a point `y` south of the ridge: 2.5 m at
/// the eaves, 0.3 m past the walls' outer faces, rising at 0.5.
fn roof_underside(t: f64, y: f64) -> f64 {
    2.5 + 0.5 * (y + t / 2.0 + 0.3)
}

/// The gable wall's volume: its mitered footprint (the outer face from
/// `-t/2` to `4 + t/2`, the inner one from `t/2` to `4 - t/2`) under a top
/// that rises from both ends to the ridge at `y = 2`. Half of it, with the
/// miter at `y = -s` for `s` across the wall: ∫ (a(2 + s) + (4 - s²)/4) ds
/// over `[-t/2, t/2]`, `a` the top at `y = 0`.
fn gable_volume(t: f64) -> f64 {
    let a = roof_underside(t, 0.0);
    2.0 * (2.0 * a * t + t - t.powi(3) / 48.0)
}

fn file(p: &Project) -> StepFile {
    parse(&export(p, &ExportOptions::default())).unwrap()
}

/// The `IfcWall` marked `mark` (its tag) and its id.
fn wall_entity<'a>(f: &'a StepFile, mark: &str) -> (u32, &'a Entity) {
    f.of_type("IFCWALL").find(|(_, e)| e.arg(7).as_str() == Some(mark)).unwrap()
}

/// A wall's body representation: its type and items.
fn body<'a>(f: &'a StepFile, wall: &Entity) -> (String, Vec<&'a Entity>) {
    let shape = f.deref(wall.arg(6)).unwrap();
    let rep =
        shape.arg(2).as_list().iter().filter_map(|r| f.deref(r)).find(|r| r.arg(1).as_str() == Some("Body")).unwrap();
    let items = rep.arg(3).as_list().iter().map(|i| f.deref(i).unwrap()).collect();
    (rep.arg(2).as_str().unwrap().to_string(), items)
}

fn point(f: &StepFile, p: &cc_ifc::parse::Param) -> DVec3 {
    let c: Vec<f64> = f.deref(p).unwrap().arg(0).as_list().iter().map(|x| x.as_f64().unwrap()).collect();
    DVec3::new(c[0], c[1], c.get(2).copied().unwrap_or(0.0))
}

fn direction(f: &StepFile, p: &cc_ifc::parse::Param, default: DVec3) -> DVec3 {
    match f.deref(p) {
        Some(d) => {
            let c: Vec<f64> = d.arg(0).as_list().iter().map(|x| x.as_f64().unwrap()).collect();
            DVec3::new(c[0], c[1], c.get(2).copied().unwrap_or(0.0)).normalize()
        }
        None => default,
    }
}

/// An `IfcAxis2Placement3D` as a transform.
fn axis_placement(f: &StepFile, p: &cc_ifc::parse::Param) -> DAffine3 {
    let e = f.deref(p).unwrap();
    let z = direction(f, e.arg(1), DVec3::Z);
    let x = direction(f, e.arg(2), DVec3::X);
    let x = (x - z * x.dot(z)).normalize();
    DAffine3::from_mat3_translation(DMat3::from_cols(x, z.cross(x), z), point(f, e.arg(0)))
}

/// An `IfcLocalPlacement` (with those it is relative to) as a transform.
fn placement(f: &StepFile, p: &cc_ifc::parse::Param) -> DAffine3 {
    match f.deref(p) {
        Some(e) => placement(f, e.arg(0)) * axis_placement(f, e.arg(1)),
        None => DAffine3::IDENTITY,
    }
}

/// The cuts of a nested clipping result, innermost first, and the solid they cut.
fn unnest<'a>(f: &'a StepFile, mut item: &'a Entity) -> (&'a Entity, Vec<&'a Entity>) {
    let mut cuts = Vec::new();
    while item.name == "IFCBOOLEANCLIPPINGRESULT" {
        assert_eq!(item.arg(0).as_enum(), Some("DIFFERENCE"));
        cuts.push(f.deref(item.arg(2)).unwrap());
        item = f.deref(item.arg(1)).unwrap();
    }
    cuts.reverse();
    (item, cuts)
}

/// A half space's plane in world coordinates (the wall placed at `world`):
/// a point on it and its normal, pointing to the half space's material.
fn cut_plane(f: &StepFile, half: &Entity, world: DAffine3) -> (DVec3, DVec3) {
    let plane = f.deref(half.arg(0)).unwrap();
    assert_eq!(plane.name, "IFCPLANE");
    let frame = world * axis_placement(f, plane.arg(0));
    let normal = frame.matrix3.z_axis.normalize();
    // AgreementFlag false: the material lies where the normal points.
    assert_eq!(half.arg(1).as_bool(), Some(false));
    (frame.translation, normal)
}

/// The quantities of `Qto_WallBaseQuantities` of the wall with id `wall`.
fn quantities(f: &StepFile, wall: u32) -> Vec<(String, f64)> {
    let (_, rel) = f
        .of_type("IFCRELDEFINESBYPROPERTIES")
        .find(|(_, r)| {
            r.arg(4).as_list().iter().any(|o| o.as_ref() == Some(wall))
                && f.deref(r.arg(5)).is_some_and(|s| s.arg(2).as_str() == Some("Qto_WallBaseQuantities"))
        })
        .unwrap();
    let set = f.deref(rel.arg(5)).unwrap();
    set.arg(5)
        .as_list()
        .iter()
        .map(|q| f.deref(q).unwrap())
        .map(|q| (q.arg(0).as_str().unwrap().to_string(), q.arg(3).as_f64().unwrap()))
        .collect()
}

fn quantity(qs: &[(String, f64)], name: &str) -> f64 {
    qs.iter().find(|q| q.0 == name).unwrap().1
}

fn close(a: f64, b: f64, tol: f64, what: &str) {
    assert!((a - b).abs() <= tol, "{what}: {a} vs {b}");
}

fn close3(a: DVec3, b: DVec3, what: &str) {
    assert!(a.distance(b) < 1e-9, "{what}: {a} vs {b}");
}

#[test]
fn the_gable_wall_is_clipped_by_the_roof_planes() {
    let p = attached_walls_project();
    let t = thickness(&p);
    let f = file(&p);
    let (id, gable) = wall_entity(&f, "gable");
    let (kind, items) = body(&f, gable);
    assert_eq!(kind, "Clipping");
    assert_eq!(items.len(), 1);
    let (solid, cuts) = unnest(&f, items[0]);
    // The mitered footprint extruded to the ridge.
    assert_eq!(solid.name, "IFCEXTRUDEDAREASOLID");
    close(solid.arg(3).as_f64().unwrap(), roof_underside(t, 2.0), 1e-9, "extruded to the ridge");
    // Two unbounded half spaces: the ridge is where the planes cross, so neither cuts past it.
    assert_eq!(cuts.len(), 2);
    assert!(cuts.iter().all(|c| c.name == "IFCHALFSPACESOLID"), "{:?}", cuts.iter().map(|c| &c.name));
    // The planes are the roof's undersides south and north of the ridge, their material above.
    let world = placement(&f, gable.arg(5));
    let roof_planes = [
        (DVec3::new(6.0, 0.0, roof_underside(t, 0.0)), DVec3::new(0.0, -0.5, 1.0).normalize()),
        (DVec3::new(6.0, 4.0, roof_underside(t, 0.0)), DVec3::new(0.0, 0.5, 1.0).normalize()),
    ];
    for (half, (on, normal)) in cuts.iter().zip(roof_planes) {
        let (at, n) = cut_plane(&f, half, world);
        close3(n, normal, "plane normal");
        close((on - at).dot(n), 0.0, 1e-9, "a point of the roof plane lies on the cut");
    }
    // Quantities of the cut body.
    let qs = quantities(&f, id);
    close(quantity(&qs, "NetVolume"), gable_volume(t), 1e-9, "volume");
    close(quantity(&qs, "Height"), roof_underside(t, 2.0), 1e-9, "height to the ridge");
    // Along the centerline (0 to 4): two trapezoids from the top at the ends to the ridge.
    let side = 2.0 * 2.0 * (roof_underside(t, 0.0) + roof_underside(t, 2.0)) / 2.0;
    close(quantity(&qs, "GrossSideArea"), side, 1e-9, "side area");
}

#[test]
fn eave_walls_under_a_level_top_stay_swept_solids() {
    let p = attached_walls_project();
    let t = thickness(&p);
    let f = file(&p);
    for mark in ["south", "north"] {
        let (_, eave) = wall_entity(&f, mark);
        let (kind, items) = body(&f, eave);
        assert_eq!(kind, "SweptSolid", "{mark}");
        assert_eq!(items[0].name, "IFCEXTRUDEDAREASOLID");
        close(items[0].arg(3).as_f64().unwrap(), roof_underside(t, 0.0), 1e-9, "up to the roof");
    }
}

#[test]
fn a_wall_partly_under_a_floor_is_cut_by_a_bounded_half_space() {
    let p = attached_walls_project();
    let (t, tf) = (thickness(&p), floor_thickness(&p));
    let f = file(&p);
    let (id, w) = wall_entity(&f, "under floor");
    let (kind, items) = body(&f, w);
    assert_eq!(kind, "Clipping");
    let (solid, cuts) = unnest(&f, items[0]);
    close(solid.arg(3).as_f64().unwrap(), 3.0, 1e-9, "its own height");
    assert_eq!(cuts.len(), 1);
    // The slab's underside, level: carried on it would cut the part beyond the slab too.
    let half = cuts[0];
    assert_eq!(half.name, "IFCPOLYGONALBOUNDEDHALFSPACE");
    let world = placement(&f, w.arg(5));
    let (at, n) = cut_plane(&f, half, world);
    close3(n, DVec3::Z, "level");
    close(at.z, 2.8 - tf, 1e-9, "at the slab's underside");
    // Bounded where the top follows the slab: up to its edge plus half the wall (and 1 cm).
    let edge = 3.0 + t / 2.0 + 0.01;
    let boundary: Vec<f64> = f
        .deref(half.arg(3))
        .unwrap()
        .arg(0)
        .as_list()
        .iter()
        .map(|q| (world * axis_placement(&f, half.arg(2))).transform_point3(point(&f, q)).x)
        .collect();
    close(boundary.iter().copied().fold(f64::MIN, f64::max), edge, 1e-9, "the bound's end");
    assert!(boundary.iter().copied().fold(f64::MAX, f64::min) < 0.0, "past the wall's start");
    let qs = quantities(&f, id);
    let volume = t * (edge * (2.8 - tf) + (6.0 - edge) * 3.0);
    close(quantity(&qs, "NetVolume"), volume, 1e-9, "volume");
    close(quantity(&qs, "GrossSideArea"), volume / t, 1e-9, "side area");
}

#[test]
fn a_wall_under_a_ceiling_is_cut_level() {
    let p = attached_walls_project();
    let t = thickness(&p);
    let f = file(&p);
    let (id, w) = wall_entity(&f, "under ceiling");
    let (kind, items) = body(&f, w);
    assert_eq!(kind, "Clipping");
    let (_, cuts) = unnest(&f, items[0]);
    assert_eq!(cuts.len(), 1);
    assert_eq!(cuts[0].name, "IFCHALFSPACESOLID");
    let (at, n) = cut_plane(&f, cuts[0], placement(&f, w.arg(5)));
    close3(n, DVec3::Z, "level");
    close(at.z, 2.6, 1e-9, "at the ceiling");
    let qs = quantities(&f, id);
    close(quantity(&qs, "NetVolume"), t * 4.0 * 2.6, 1e-9, "volume");
    close(quantity(&qs, "Height"), 2.6, 1e-9, "height");
}

/// Imported back, each wall keeps its shape: a cut top becomes an elevation
/// profile, so the volumes are those of the attached walls.
#[test]
fn attached_walls_come_back_with_their_tops() {
    let original = attached_walls_project();
    let t = thickness(&original);
    let tf = floor_thickness(&original);
    let (imported, _) = crate::round_trip::round_trip(&original);
    let (ga, gb) = (cc_geom::ModelGeometry::build(&original), cc_geom::ModelGeometry::build(&imported));
    let by_mark = |p: &Project, mark: &str| p.elements.values().find(|e| e.mark == mark).unwrap().id;
    let edge = 3.0 + t / 2.0 + 0.01;
    let expected = [
        ("gable", gable_volume(t), true),
        ("under floor", t * (edge * (2.8 - tf) + (6.0 - edge) * 3.0), true),
        ("under ceiling", t * 4.0 * 2.6, false),
    ];
    for (mark, volume, profiled) in expected {
        let (a, b) = (by_mark(&original, mark), by_mark(&imported, mark));
        close(ga.volume(a), volume, 1e-9, mark);
        close(gb.volume(b), volume, 1e-6, mark);
        let w = imported.element(b).and_then(|e| e.as_wall()).unwrap();
        assert_eq!(w.profile.is_some(), profiled, "{mark}");
    }
}

/// Writes `fixtures/attached_walls.ccad` from [`attached_walls_project`]
/// (`cargo nextest run -p cc_ifc --test main write_attached_walls_fixture --run-ignored only`).
#[test]
#[ignore]
fn write_attached_walls_fixture() {
    let text = cc_core::to_ron(&attached_walls_project()).unwrap();
    std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/attached_walls.ccad"), text).unwrap();
}

#[test]
fn attached_walls_fixture_is_the_project() {
    let fixture = cc_core::from_ron(include_str!("fixtures/attached_walls.ccad")).unwrap();
    let f = file(&fixture);
    let (_, gable) = wall_entity(&f, "gable");
    assert_eq!(body(&f, gable).0, "Clipping");
}
