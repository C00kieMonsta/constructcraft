//! Spline walls (see [`cc_core::wall_spline`]): their bodies, volumes,
//! outlines, joins with other walls and the rooms they bound.

use crate::{ModelGeometry, polygon};
use cc_core::wall_spline::{SAGITTA, WallSpline};
use cc_core::{Category, Document, Element, ElementId, ElementKind, LocationLine, Project, Wall};
use glam::DVec2;

fn add(doc: &mut Document, category: Category, kind: ElementKind) -> ElementId {
    doc.edit("add", |e| {
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
    })
}

fn pts(points: &[(f64, f64)]) -> Vec<DVec2> {
    points.iter().map(|&(x, y)| DVec2::new(x, y)).collect()
}

fn straight(start: (f64, f64), end: (f64, f64)) -> ElementKind {
    ElementKind::Wall(Wall { start: DVec2::new(start.0, start.1), end: DVec2::new(end.0, end.1), ..Default::default() })
}

/// A wall through the fit points.
fn through(fit: &[(f64, f64)]) -> ElementKind {
    ElementKind::Wall(Wall { spline: WallSpline::through(&pts(fit)), ..Default::default() })
}

/// A wall with these control points.
fn control(points: &[(f64, f64)]) -> ElementKind {
    ElementKind::Wall(Wall { spline: Some(WallSpline { control: pts(points) }), ..Default::default() })
}


/// The volume of the closed mesh of a wall's body.
fn mesh_volume(g: &ModelGeometry, id: ElementId) -> f64 {
    g.body_of(id).map(crate::Solid::volume).sum()
}

fn finite(g: &ModelGeometry) {
    for s in &g.solids {
        for f in &s.faces {
            assert!(f.normal.is_finite() && f.verts.iter().all(|v| v.is_finite()), "a face of {:?}", s.element);
        }
    }
    for w in g.walls.values() {
        assert!(w.length.is_finite() && w.volume().is_finite());
        assert!([w.start_ext.0, w.start_ext.1, w.end_ext.0, w.end_ext.1].iter().all(|u| u.is_finite()));
    }
}

/// A half circle of radius `r` around the origin from (r, 0) to (-r, 0),
/// as two Bézier quarters (a circle to 0.03 %).
fn half_circle(r: f64) -> Vec<(f64, f64)> {
    let k = 0.552_284_749_8 * r;
    vec![(r, 0.0), (r, k), (k, r), (0.0, r), (-k, r), (-r, k), (-r, 0.0)]
}

#[test]
fn a_spline_through_points_in_a_line_is_the_straight_wall() {
    let mut doc = Document::new(Project::from_template());
    let line = add(&mut doc, Category::Wall, straight((0.0, 0.0), (7.0, 0.0)));
    let curve = add(&mut doc, Category::Wall, through(&[(0.0, 5.0), (2.0, 5.0), (3.5, 5.0), (7.0, 5.0)]));
    // Its ends follow the curve.
    let w = doc.project.element(curve).unwrap().as_wall().unwrap();
    assert_eq!((w.start, w.end), (DVec2::new(0.0, 5.0), DVec2::new(7.0, 5.0)));
    assert!((w.length() - 7.0).abs() < 1e-12);
    let g = ModelGeometry::build(&doc.project);
    let (a, b) = (&g.walls[&line], &g.walls[&curve]);
    assert!(b.spline.is_some() && b.arc.is_none() && b.is_curved());
    assert!((a.volume() - b.volume()).abs() < 1e-9, "{} vs {}", a.volume(), b.volume());
    assert!((mesh_volume(&g, line) - mesh_volume(&g, curve)).abs() < 1e-9);
    assert!((b.length - 7.0).abs() < 1e-12);
    // The same outline, 5 m up: its four corners, the others on its sides.
    let (fa, fb) = (a.footprint(), b.footprint());
    assert!((polygon::area(&fa) - polygon::area(&fb)).abs() < 1e-9);
    for p in &fa {
        assert!(fb.iter().any(|q| q.distance(*p + DVec2::new(0.0, 5.0)) < 1e-9), "{p}");
    }
    let h = b.half;
    for q in &fb {
        assert!((q.y - 5.0).abs() - h < 1e-9 && q.x > -1e-9 && q.x < 7.0 + 1e-9, "{q}");
    }
    finite(&g);
}

#[test]
fn a_spline_along_a_circle_approaches_the_arc_wall() {
    let r = 5.0;
    let mut doc = Document::new(Project::from_template());
    let arc = add(
        &mut doc,
        Category::Wall,
        ElementKind::Wall(Wall { start: DVec2::new(r, 0.0), end: DVec2::new(-r, 0.0), bulge: 1.0, ..Default::default() }),
    );
    // The usual Bézier quarters: a circle to 0.03 %.
    let exact = add(&mut doc, Category::Wall, control(&half_circle(r)));
    // And a spline through nine points of the circle.
    let fit: Vec<(f64, f64)> = (0..=8)
        .map(|i| {
            let a = std::f64::consts::PI * i as f64 / 8.0;
            (20.0 + r * a.cos(), r * a.sin())
        })
        .collect();
    let fitted = add(&mut doc, Category::Wall, through(&fit));
    let g = ModelGeometry::build(&doc.project);
    let a = &g.walls[&arc];
    // Bézier quarters: length and volume within 0.1 % of the arc wall's.
    let b = &g.walls[&exact];
    assert!((b.length / a.length - 1.0).abs() < 1e-3, "{} vs {}", b.length, a.length);
    assert!((b.volume() / a.volume() - 1.0).abs() < 1e-3, "{} vs {}", b.volume(), a.volume());
    // Interpolated: within 0.5 %.
    let c = &g.walls[&fitted];
    assert!((c.length / a.length - 1.0).abs() < 5e-3, "{} vs {}", c.length, a.length);
    assert!((c.volume() / a.volume() - 1.0).abs() < 5e-3, "{} vs {}", c.volume(), a.volume());
    // The meshes hold what the quantities say (exactly: both are the chords').
    for id in [exact, fitted] {
        let (mesh, volume) = (mesh_volume(&g, id), g.walls[&id].volume());
        assert!((mesh - volume).abs() < 1e-9 * volume, "{mesh} vs {volume}");
    }
    // The faces are concentric: every point of the outline lies on one of the two circles,
    // and the chords stray no further from them than the flattening allows.
    let h = b.half;
    for p in b.footprint() {
        let d = p.length();
        assert!((d - (r + h)).abs() < 3e-3 || (d - (r - h)).abs() < 3e-3, "{p}: {d}");
    }
    let face = b.path(0.0, b.length, -h);
    for w in face.windows(2) {
        assert!((((w[0] + w[1]) * 0.5).length() - (r + h)).abs() < 3e-3 + 2.0 * SAGITTA);
    }
    // Curved faces are facets of one smooth surface; the caps and the top are not.
    let solid = g.body_of(exact).next().unwrap();
    assert!(solid.faces.iter().any(|f| f.smooth) && solid.faces.iter().any(|f| !f.smooth));
    assert!(solid.faces.iter().filter(|f| f.smooth).all(|f| f.normal.z.abs() < 1e-9));
    finite(&g);
}

#[test]
fn the_location_line_moves_the_body_across() {
    let mut doc = Document::new(Project::from_template());
    let fit = [(0.0, 0.0), (3.0, 1.5), (6.0, -1.0), (9.0, 0.5)];
    let center = add(&mut doc, Category::Wall, through(&fit));
    let exterior = add(&mut doc, Category::Wall, through(&fit));
    doc.edit("line", |e| {
        if let Some(ElementKind::Wall(w)) = e.element_mut(exterior).map(|x| &mut x.kind) {
            w.location_line = LocationLine::FinishExterior;
            w.base_offset = 5.0;
        }
    });
    let g = ModelGeometry::build(&doc.project);
    let (a, b) = (&g.walls[&center], &g.walls[&exterior]);
    let h = a.half;
    // The exterior (left) face of the second is the first's centerline.
    let spline = doc.project.element(center).unwrap().as_wall().unwrap().spline_path().unwrap();
    for k in 0..=20 {
        let u = b.length * k as f64 / 20.0;
        let (_, v) = spline.locate(b.world(u, h));
        assert!(v.abs() < 1e-9, "{u}: {v}");
        // And its own centerline lies half a wall to the right of it.
        let (_, v) = spline.locate(b.world(u, 0.0));
        assert!((v + h).abs() < 1e-9, "{u}: {v}");
    }
    // The stored ends stay those of the curve drawn.
    let w = doc.project.element(exterior).unwrap().as_wall().unwrap();
    assert_eq!((w.start, w.end), (DVec2::new(0.0, 0.0), DVec2::new(9.0, 0.5)));
    assert!((a.length - b.length).abs() < 0.2 && a.length != b.length);
    finite(&g);
}

#[test]
fn a_spline_wall_meets_straight_walls_at_its_ends() {
    let mut doc = Document::new(Project::from_template());
    // An L at its start (a wall running down from the same point) and a T at
    // its end (which stops on the face of a wall passing by).
    let curve = add(&mut doc, Category::Wall, control(&[
        (0.0, 0.0),
        (1.0, 0.0),
        (2.0, 1.0),
        (3.0, 0.0),
        (4.0, -1.0),
        (5.0, 0.0),
        (6.0, 0.0),
    ]));
    let corner = add(&mut doc, Category::Wall, straight((0.0, 0.0), (0.0, -3.0)));
    let through_wall = add(&mut doc, Category::Wall, straight((6.0, -2.0), (6.0, 2.0)));
    let g = ModelGeometry::build(&doc.project);
    finite(&g);
    let (c, l, t) = (&g.walls[&curve], &g.walls[&corner], &g.walls[&through_wall]);
    let h = c.half;
    assert!(c.start_joined && c.end_joined && l.start_joined && !t.start_joined && !t.end_joined);
    // The L: a miter, as between two straight walls (the curve starts along +X).
    // The outer faces run on to the corner, the inner ones stop short of it (where the
    // curve's face, already bending, crosses the other wall's).
    assert!((c.start_ext.0 + h).abs() < 1e-6 && (c.start_ext.1 - h).abs() < 5e-3, "{:?}", c.start_ext);
    assert!((l.start_ext.0 - h).abs() < 5e-3 && (l.start_ext.1 + h).abs() < 1e-6, "{:?}", l.start_ext);
    // The corners of both walls coincide.
    assert!(c.world(c.start_ext.0, h).distance(l.world(l.start_ext.1, -h)) < 1e-6);
    assert!(c.world(c.start_ext.1, -h).distance(l.world(l.start_ext.0, h)) < 1e-6);
    // The T: the curve ends on the near face of the wall it meets, which opens there.
    for (u, v) in [(c.end_ext.0, h), (c.end_ext.1, -h)] {
        assert!((c.world(u, v).x - (6.0 - h)).abs() < 1e-6, "{}", c.world(u, v));
    }
    assert_eq!(t.face_gaps.len(), 1);
    let (side, u0, u1) = t.face_gaps[0];
    // (The curve dips a couple of millimeters just before its end.)
    assert!(side > 0.0 && (u0 - (2.0 - h)).abs() < 5e-3 && (u1 - (2.0 + h)).abs() < 5e-3, "{:?}", t.face_gaps);
    // Volumes: what the footprints hold, the joints shared without overlap.
    let free = {
        let mut alone = Document::new(Project::from_template());
        let id = add(&mut alone, Category::Wall, control(&[
            (0.0, 0.0),
            (1.0, 0.0),
            (2.0, 1.0),
            (3.0, 0.0),
            (4.0, -1.0),
            (5.0, 0.0),
            (6.0, 0.0),
        ]));
        ModelGeometry::build(&alone.project).volume(id)
    };
    let joined = g.volume(curve);
    // The miter keeps the volume (but for the bend of the inner face); the T takes half a wall off the end.
    let height = 3.0;
    assert!((free - joined - h * 2.0 * h * height).abs() < 1e-3, "{free} vs {joined}");
    assert!((g.volume(corner) - 3.0 * 2.0 * h * height).abs() < 1e-3);
}

#[test]
fn a_straight_wall_butts_into_the_curved_side() {
    let r = 4.0;
    let mut doc = Document::new(Project::from_template());
    let curve = add(&mut doc, Category::Wall, control(&half_circle(r)));
    // From inside the half circle up to its crown, and from outside down to its side.
    let inside = add(&mut doc, Category::Wall, straight((0.0, 1.0), (0.0, r)));
    let at = DVec2::from_angle(0.6) * r;
    let outside = add(&mut doc, Category::Wall, straight((at.x * 1.6, at.y * 1.6), (at.x, at.y)));
    let g = ModelGeometry::build(&doc.project);
    finite(&g);
    let host = &g.walls[&curve];
    let h = host.half;
    for (id, face) in [(inside, r - h), (outside, r + h)] {
        let w = &g.walls[&id];
        assert!(w.end_joined && !w.start_joined);
        // Both corners of its end lie on the host's curved face (to the flattening's tolerance).
        for (u, v) in [(w.end_ext.0, w.half), (w.end_ext.1, -w.half)] {
            let d = w.world(u, v).length();
            assert!((d - face).abs() < 3e-3, "{id:?}: {d} vs {face}");
        }
    }
    // The host's faces open where they meet it: the inner (left) one and the outer one.
    assert_eq!(host.face_gaps.len(), 2, "{:?}", host.face_gaps);
    let sides: Vec<f64> = host.face_gaps.iter().map(|g| g.0).collect();
    assert!(sides.contains(&1.0) && sides.contains(&-1.0));
    for &(_, u0, u1) in &host.face_gaps {
        assert!(u1 - u0 > 1.5 * h && u1 - u0 < 3.0 * h, "{u0} {u1}");
    }
    // The host stays whole.
    assert!(!host.start_joined && !host.end_joined);
    let alone = {
        let mut d = Document::new(Project::from_template());
        let id = add(&mut d, Category::Wall, control(&half_circle(r)));
        ModelGeometry::build(&d.project).volume(id)
    };
    assert!((g.volume(curve) - alone).abs() < 1e-9);
}

#[test]
fn spline_walls_meet_each_other_and_arcs() {
    let mut doc = Document::new(Project::from_template());
    // Two splines sharing an end at a right angle, and an arc wall meeting the first's other end.
    let a = add(&mut doc, Category::Wall, through(&[(0.0, 0.0), (2.0, 0.5), (4.0, 0.0)]));
    let b = add(&mut doc, Category::Wall, control(&[(4.0, 0.0), (4.0, -1.0), (5.0, -2.0), (5.0, -3.0)]));
    let arc = add(
        &mut doc,
        Category::Wall,
        ElementKind::Wall(Wall { start: DVec2::new(0.0, 0.0), end: DVec2::new(-2.0, -2.0), bulge: 0.4, ..Default::default() }),
    );
    let g = ModelGeometry::build(&doc.project);
    finite(&g);
    let (wa, wb, wc) = (&g.walls[&a], &g.walls[&b], &g.walls[&arc]);
    assert!(wa.start_joined && wa.end_joined && wb.start_joined && wc.start_joined);
    // Shared corners: each wall's left face ends where the other's right face does.
    let h = wa.half;
    // (An end meeting a start: left meets left; two starts: left meets right.)
    assert!(wa.world(wa.end_ext.0, h).distance(wb.world(wb.start_ext.0, h)) < 1e-6);
    assert!(wa.world(wa.end_ext.1, -h).distance(wb.world(wb.start_ext.1, -h)) < 1e-6);
    assert!(wa.world(wa.start_ext.0, h).distance(wc.world(wc.start_ext.1, -h)) < 1e-6);
    assert!(wa.world(wa.start_ext.1, -h).distance(wc.world(wc.start_ext.0, h)) < 1e-6);
    for id in [a, b, arc] {
        let v = g.volume(id);
        let w = &g.walls[&id];
        assert!(v > 0.0 && (v / (w.length * 2.0 * w.half * 3.0) - 1.0).abs() < 0.1, "{id:?}: {v}");
    }
}

#[test]
fn rooms_are_bounded_by_a_spline_wall() {
    let mut doc = Document::new(Project::from_template());
    // Three straight walls and a spline closing them, leaving its ends square to the side walls.
    let ctrl = [(0.0, 4.0), (1.0, 4.0), (2.0, 5.0), (3.0, 4.0), (4.0, 3.0), (5.0, 4.0), (6.0, 4.0)];
    add(&mut doc, Category::Wall, straight((0.0, 4.0), (0.0, 0.0)));
    add(&mut doc, Category::Wall, straight((0.0, 0.0), (6.0, 0.0)));
    add(&mut doc, Category::Wall, straight((6.0, 0.0), (6.0, 4.0)));
    let curve = add(&mut doc, Category::Wall, control(&ctrl));
    let room = add(
        &mut doc,
        Category::Room,
        ElementKind::Room(cc_core::Room { position: DVec2::new(3.0, 2.0), number: "1".into(), name: "R".into(), height: 3.0 }),
    );
    let g = ModelGeometry::build(&doc.project);
    finite(&g);
    let h = g.walls[&curve].half;
    let boundary = g.rooms.get(&room).expect("the walls enclose the room");
    // Expected, from the curve itself: under its lower face (h to its right as it runs
    // +X), sampled finely, between the side walls' inner faces and above the bottom wall's.
    let control = pts(&ctrl);
    let n = 4000;
    let mut area = 0.0;
    let mut prev: Option<DVec2> = None;
    for i in 0..=n {
        let s = 2.0 * i as f64 / n as f64;
        let p = cc_core::spline::point_at(&control, s).unwrap();
        let t = cc_core::spline::direction_at(&control, s).unwrap();
        let q = p - t.perp() * h;
        if let Some(a) = prev {
            // The trapezoid under a → q, clipped to the room's width.
            let (x0, x1) = (a.x.clamp(h, 6.0 - h), q.x.clamp(h, 6.0 - h));
            let y_at = |x: f64| if (q.x - a.x).abs() > 1e-12 { a.y + (q.y - a.y) * (x - a.x) / (q.x - a.x) } else { a.y };
            area += (x1 - x0) * ((y_at(x0) + y_at(x1)) * 0.5 - h);
        }
        prev = Some(q);
    }
    let found = polygon::area(boundary);
    // The chords of the curved side stray at most the sagitta from it over its 6 m.
    assert!((found - area).abs() < 5e-3, "{found} vs {area}");
    assert!(area > 20.0);
}

#[test]
fn layers_follow_the_curve_and_keep_their_volumes() {
    let mut doc = Document::new(Project::from_template());
    let id = add(&mut doc, Category::Wall, through(&[(0.0, 0.0), (3.0, 1.5), (6.0, -1.0), (9.0, 0.5)]));
    let g = ModelGeometry::build(&doc.project);
    let w = &g.walls[&id];
    assert_eq!(w.layers.len(), 3, "plaster, masonry, plaster");
    // Layer volumes add up to the wall's, and each is its strip of the footprint.
    let volumes = w.layer_volumes();
    assert!((volumes.iter().sum::<f64>() - w.volume()).abs() < 1e-9 * w.volume());
    for (&(v0, v1, _), volume) in w.layers.iter().zip(&volumes) {
        let mut strip = w.path(0.0, w.length, v1);
        strip.extend(w.path(w.length, 0.0, v0));
        assert!((polygon::area(&strip) * 3.0 - volume).abs() < 1e-9, "{v0} {v1}");
    }
    // The inside of the first bend holds less than the outside.
    assert!(volumes[0] != volumes[2]);
}

#[test]
fn layers_with_their_own_extents_step_a_spline_wall() {
    let mut doc = Document::new(Project::from_template());
    // The interior plaster stops 1 m short of the top; the exterior starts 0.5 m up.
    let ty = doc.edit("type", |e| {
        let base = e.project().default_type(Category::Wall).unwrap();
        let mut ty = e.project().element_type(base).unwrap().clone();
        let cc_core::TypeData::Wall { layers, .. } = &mut ty.data else { panic!("a wall type") };
        layers[0].base_offset = 0.5;
        layers[2].height = Some(2.0);
        ty.name = format!("{} (stepped)", ty.name);
        ty.uid = Default::default();
        ty.id = cc_core::ElementTypeId(e.alloc_id());
        e.add_type(ty)
    });
    let id = add(&mut doc, Category::Wall, through(&[(0.0, 0.0), (3.0, 1.5), (6.0, -1.0), (9.0, 0.5)]));
    doc.edit("retype", |e| {
        if let Some(x) = e.element_mut(id) {
            x.type_id = ty;
        }
    });
    let g = ModelGeometry::build(&doc.project);
    finite(&g);
    let w = &g.walls[&id];
    assert!(w.is_banded() && w.spline.is_some());
    let volumes = w.layer_volumes();
    let strip = |v0: f64, v1: f64| {
        let mut s = w.path(0.0, w.length, v1);
        s.extend(w.path(w.length, 0.0, v0));
        polygon::area(&s)
    };
    let heights = [2.5, 3.0, 2.0];
    for (k, &(v0, v1, _)) in w.layers.iter().enumerate() {
        assert!((strip(v0, v1) * heights[k] - volumes[k]).abs() < 1e-9, "layer {k}");
    }
    // The stepped mesh is closed and holds the layers' volumes.
    let mesh = mesh_volume(&g, id);
    assert!((mesh - w.volume()).abs() < 1e-6 * w.volume(), "{mesh} vs {}", w.volume());
}

#[test]
fn an_unrepaired_spline_wall_builds_without_panicking() {
    // Geometry of a project that skipped repair: a spline too tight for its wall, one that
    // crosses itself, one with a corner, a door on a spline wall and a curtain type.
    let mut p = Project::from_template();
    let level = p.levels[0].id;
    let wall_t = p.default_type(Category::Wall).unwrap();
    let door_t = p.default_type(Category::Door).unwrap();
    let mut next = 9000;
    let mut push = |p: &mut Project, type_id, kind| {
        next += 1;
        let id = ElementId(next);
        p.elements.insert(
            id,
            Element {
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
            },
        );
        id
    };
    let wall = |control: &[(f64, f64)]| {
        let control = pts(control);
        ElementKind::Wall(Wall {
            start: control[0],
            end: *control.last().unwrap(),
            spline: Some(WallSpline { control }),
            ..Default::default()
        })
    };
    let tight = push(&mut p, wall_t, wall(&[(0.0, 0.0), (0.05, 0.0), (0.1, 0.05), (0.1, 0.1)]));
    push(&mut p, wall_t, wall(&[(0.0, 10.0), (6.0, 14.0), (-2.0, 14.0), (4.0, 10.0)]));
    push(&mut p, wall_t, wall(&[(0.0, 20.0), (1.0, 20.0), (2.0, 20.0), (3.0, 20.0), (3.0, 21.0), (3.0, 22.0), (3.0, 23.0)]));
    push(&mut p, wall_t, wall(&[(0.0, 30.0), (f64::NAN, 30.0), (2.0, 31.0), (3.0, 30.0)]));
    let host = push(&mut p, wall_t, wall(&[(0.0, 40.0), (2.0, 41.0), (4.0, 39.0), (6.0, 40.0)]));
    push(
        &mut p,
        door_t,
        ElementKind::Door(cc_core::Hosted { host, offset: 3.0, sill_height: 0.0, flip_facing: false, flip_hand: false }),
    );
    let g = ModelGeometry::build(&p);
    finite(&g);
    // The door cuts nothing: spline walls host no openings yet.
    assert!(g.walls[&host].openings.is_empty() && g.walls[&host].columns.len() == 1);
    assert!(g.walls.contains_key(&tight));
    // Repair straightens the hostile ones and takes the door away.
    let repairs = p.repair();
    assert!(repairs >= 5, "{repairs}");
    assert_eq!(p.walls().filter(|(_, w)| w.spline.is_some()).count(), 1);
    assert_eq!(p.hosted_by(host).count(), 0);
    finite(&ModelGeometry::build(&p));
}
