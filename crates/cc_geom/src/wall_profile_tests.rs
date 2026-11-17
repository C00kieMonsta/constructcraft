//! Walls with elevation profiles (see [`cc_core::wall_profile`]): their
//! bodies, volumes, openings, cuts and joins.

use crate::ModelGeometry;
use cc_core::wall_profile::WallProfile;
use cc_core::{Category, Document, Element, ElementId, ElementKind, Hosted, Project, Wall};
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

fn wall(start: (f64, f64), end: (f64, f64), profile: Option<WallProfile>) -> ElementKind {
    ElementKind::Wall(Wall {
        start: DVec2::new(start.0, start.1),
        end: DVec2::new(end.0, end.1),
        height: 3.0,
        profile,
        ..Default::default()
    })
}

fn outline(points: &[(f64, f64)], length: f64) -> WallProfile {
    WallProfile::from_outline(points.iter().map(|&(u, z)| DVec2::new(u, z)).collect(), length).unwrap()
}

fn door(host: ElementId, offset: f64) -> ElementKind {
    ElementKind::Door(Hosted { host, offset, sill_height: 0.0, flip_facing: false, flip_hand: false })
}

fn thickness(doc: &Document, id: ElementId) -> f64 {
    doc.project.element(id).map_or(0.0, |e| doc.project.thickness_of(e))
}

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 1e-9 * b.abs().max(1.0), "{what}: {a} vs {b}");
}

/// The volume of the closed mesh of a wall's body.
fn mesh_volume(g: &ModelGeometry, id: ElementId) -> f64 {
    g.body_of(id).map(crate::Solid::volume).sum()
}

#[test]
fn a_sloped_top_holds_the_trapezoid() {
    let mut doc = Document::new(Project::from_template());
    let id = add(
        &mut doc,
        Category::Wall,
        wall((0.0, 0.0), (6.0, 0.0), WallProfile::sloped(6.0, (0.0, 0.0), (3.0, 2.0)).ok()),
    );
    let g = ModelGeometry::build(&doc.project);
    let t = thickness(&doc, id);
    // (3 + 2) / 2 high over 6 m.
    close(g.volume(id), 15.0 * t, "analytic volume");
    close(mesh_volume(&g, id), 15.0 * t, "mesh volume");
    let w = &g.walls[&id];
    assert!(w.is_profiled());
    close(w.base_z, 0.0, "nominal base");
    close(w.top_z, 3.0, "highest point");
    // Its top runs from 3 m down to 2 m.
    let (lo, hi) = g.body_of(id).filter_map(crate::Solid::bounds).next().unwrap();
    close(hi.z, 3.0, "top");
    close(lo.z, 0.0, "base");
}

#[test]
fn a_stepped_base_and_a_raised_one() {
    let mut doc = Document::new(Project::from_template());
    // A base 0.5 m up from 2 m on, and one running 1 m below the level at the start.
    let stepped = outline(&[(0.0, 0.0), (2.0, 0.0), (2.0, 0.5), (6.0, 0.5), (6.0, 3.0), (0.0, 3.0)], 6.0);
    let a = add(&mut doc, Category::Wall, wall((0.0, 0.0), (6.0, 0.0), Some(stepped)));
    let sunk = outline(&[(0.0, -1.0), (6.0, 0.0), (6.0, 3.0), (0.0, 3.0)], 6.0);
    let b = add(&mut doc, Category::Wall, wall((0.0, 5.0), (6.0, 5.0), Some(sunk)));
    let g = ModelGeometry::build(&doc.project);
    let t = thickness(&doc, a);
    close(g.volume(a), (2.0 * 3.0 + 4.0 * 2.5) * t, "stepped");
    close(mesh_volume(&g, a), (2.0 * 3.0 + 4.0 * 2.5) * t, "stepped mesh");
    close(g.volume(b), (3.5 * 6.0) * t, "sunk");
    close(mesh_volume(&g, b), (3.5 * 6.0) * t, "sunk mesh");
    let (lo, _) = g.body_of(b).filter_map(crate::Solid::bounds).next().unwrap();
    close(lo.z, -1.0, "below the level");
}

#[test]
fn a_profile_shorter_than_its_wall_builds_only_its_stretch() {
    let mut doc = Document::new(Project::from_template());
    let part = outline(&[(1.0, 0.0), (4.0, 0.0), (4.0, 2.0), (1.0, 2.0)], 6.0);
    let id = add(&mut doc, Category::Wall, wall((0.0, 0.0), (6.0, 0.0), Some(part)));
    let g = ModelGeometry::build(&doc.project);
    let t = thickness(&doc, id);
    close(g.volume(id), 3.0 * 2.0 * t, "volume");
    close(mesh_volume(&g, id), 3.0 * 2.0 * t, "closed mesh, both ends capped");
    let w = &g.walls[&id];
    let xs: Vec<f64> = w.footprint().iter().map(|p| p.x).collect();
    assert!(xs.iter().all(|x| (1.0 - 1e-9..=4.0 + 1e-9).contains(x)), "{xs:?}");
}

#[test]
fn doors_inside_the_body_cut_it_and_the_others_are_left_out() {
    let mut doc = Document::new(Project::from_template());
    let id = add(
        &mut doc,
        Category::Wall,
        wall((0.0, 0.0), (6.0, 0.0), WallProfile::sloped(6.0, (0.0, 0.0), (3.0, 2.0)).ok()),
    );
    let inside = add(&mut doc, Category::Door, door(id, 1.5));
    // Near the low end the top is under the door's head.
    let outside = add(&mut doc, Category::Door, door(id, 5.4));
    let g = ModelGeometry::build(&doc.project);
    let w = &g.walls[&id];
    let cut: Vec<ElementId> = w.openings.iter().map(|o| o.element).collect();
    assert_eq!(cut, vec![inside], "{outside:?} does not fit");
    let o = &w.openings[0];
    let t = thickness(&doc, id);
    let expected = 15.0 * t - (o.u1 - o.u0) * (o.z1 - o.z0) * t;
    close(g.volume(id), expected, "less the door");
    close(mesh_volume(&g, id), expected, "mesh less the door");
    assert!(g.opening_fills.contains_key(&inside) && !g.opening_fills.contains_key(&outside));
}

#[test]
fn a_level_cut_splits_the_wall_where_its_top_crosses_it() {
    let mut doc = Document::new(Project::from_template());
    let id = add(
        &mut doc,
        Category::Wall,
        wall((0.0, 0.0), (6.0, 0.0), WallProfile::sloped(6.0, (0.0, 0.0), (3.0, 1.0)).ok()),
    );
    let g = ModelGeometry::build(&doc.project);
    let w = &g.walls[&id];
    // The top is 2 m high at 3 m along.
    let columns = w.cut_columns(2.0);
    let solid: Vec<(f64, f64)> = columns
        .iter()
        .filter(|c| c.is_solid_at(2.0))
        .map(|c| (c.left.0.max(c.right.0), c.left.1.min(c.right.1)))
        .collect();
    assert_eq!(solid.len(), 1, "{columns:?}");
    assert!(solid[0].0 < 1e-9 && (solid[0].1 - 3.0).abs() < 1e-9, "{solid:?}");
    assert!(w.solid_at(2.9, 2.0) && !w.solid_at(3.1, 2.0));
    // Under 1 m it is solid all along.
    assert!(w.cut_columns(0.9).iter().all(|c| c.is_solid_at(0.9)));
}

#[test]
fn a_profiled_wall_joins_a_plain_one() {
    let mut doc = Document::new(Project::from_template());
    // An L corner at (6, 0): the profiled wall's low end meets a plain wall.
    let a = add(
        &mut doc,
        Category::Wall,
        wall((0.0, 0.0), (6.0, 0.0), WallProfile::sloped(6.0, (0.0, 0.0), (3.0, 2.0)).ok()),
    );
    let b = add(&mut doc, Category::Wall, wall((6.0, 0.0), (6.0, 4.0), None));
    let g = ModelGeometry::build(&doc.project);
    let (wa, wb) = (&g.walls[&a], &g.walls[&b]);
    assert!(wa.end_joined && wb.start_joined, "they meet in a mitered corner");
    let t = thickness(&doc, a);
    // The corner square is shared by the miter: each wall has its own half.
    let va = g.volume(a);
    assert!(va > 15.0 * t * 0.95 && va < 15.0 * t * 1.05, "{va}");
    // Its body closed at the mitered joint holds that volume.
    let mut closed = wa.clone();
    (closed.start_joined, closed.end_joined) = (false, false);
    close(closed.to_solid([0; 3]).volume(), va, "closed mesh and analytic volume");
    let vb = g.volume(b);
    assert!(vb > 4.0 * 3.0 * t * 0.95 && vb < 4.0 * 3.0 * t * 1.05, "{vb}");
    // A profile short of that end joins nothing there.
    let mut doc = Document::new(Project::from_template());
    let part = outline(&[(0.0, 0.0), (5.0, 0.0), (5.0, 3.0), (0.0, 3.0)], 6.0);
    let a = add(&mut doc, Category::Wall, wall((0.0, 0.0), (6.0, 0.0), Some(part)));
    let b = add(&mut doc, Category::Wall, wall((6.0, 0.0), (6.0, 4.0), None));
    let g = ModelGeometry::build(&doc.project);
    assert!(!g.walls[&a].end_joined && !g.walls[&b].start_joined);
    close(g.volume(a), 5.0 * 3.0 * t, "short of the corner");
}

#[test]
fn a_profile_overrides_a_roof_the_top_is_attached_to() {
    let mut doc = Document::new(Project::from_template());
    let slab = add(
        &mut doc,
        Category::Floor,
        ElementKind::Floor(cc_core::Floor::new(
            vec![DVec2::new(-1.0, -1.0), DVec2::new(7.0, -1.0), DVec2::new(7.0, 1.0), DVec2::new(-1.0, 1.0)],
            5.0,
        )),
    );
    let mut w = Wall {
        start: DVec2::ZERO,
        end: DVec2::new(6.0, 0.0),
        profile: WallProfile::sloped(6.0, (0.0, 0.0), (3.0, 2.0)).ok(),
        ..Default::default()
    };
    w.attach_top = Some(slab);
    // Set past the edit's checks, as a file could hold it.
    let id = add(&mut doc, Category::Wall, ElementKind::Wall(w));
    let g = ModelGeometry::build(&doc.project);
    let t = thickness(&doc, id);
    close(g.volume(id), 15.0 * t, "the profile, not the slab");
}
