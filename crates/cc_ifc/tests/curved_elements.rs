//! Curved walls, ring and straight footings, pile caps and circular holes
//! as another program writes them (IFC2X3, no axes or types, everything in
//! world coordinates; #370): each becomes the native element that holds it
//! exactly, with the volume the file's solid has. Shapes no native element
//! holds, and hostile values, stay generic objects, never a panic.

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_core::pile::PileLayout;
use cc_core::sketch::SketchCurve;
use cc_core::{Category, Element, ElementKind, FoundationKind, FoundationShape, Project, StripHaunch, TypeData};
use cc_geom::ModelGeometry;
use cc_ifc::import;
use std::f64::consts::{PI, TAU};

/// A STEP file being written: entities numbered as they are added.
struct File {
    lines: Vec<String>,
    products: Vec<usize>,
}

impl File {
    fn new() -> Self {
        let lines = [
            "IFCPROJECT('0YvctVUKr0kugbFTf53O9L',$,'Tank',$,$,$,$,(#2),#3)",
            "IFCGEOMETRICREPRESENTATIONCONTEXT($,'Model',3,1.E-05,#7,$)",
            "IFCUNITASSIGNMENT((#4))",
            "IFCSIUNIT(*,.LENGTHUNIT.,$,.METRE.)",
            "IFCBUILDINGSTOREY('1XvctVUKr0kugbFTf53O9L',$,'Ground',$,$,#6,$,$,.ELEMENT.,0.)",
            "IFCLOCALPLACEMENT($,#7)",
            "IFCAXIS2PLACEMENT3D(#8,$,$)",
            "IFCCARTESIANPOINT((0.,0.,0.))",
            "IFCDIRECTION((0.,0.,1.))",
            "IFCAXIS2PLACEMENT2D(#11,$)",
            "IFCCARTESIANPOINT((0.,0.))",
        ];
        Self { lines: lines.iter().map(|l| l.to_string()).collect(), products: Vec::new() }
    }

    fn add(&mut self, entity: String) -> usize {
        self.lines.push(entity);
        self.lines.len()
    }

    fn point2(&mut self, x: f64, y: f64) -> usize {
        self.add(format!("IFCCARTESIANPOINT(({x:.9},{y:.9}))"))
    }

    /// A 2D placement at `(x, y)` turned `turn` degrees.
    fn place2(&mut self, x: f64, y: f64, turn: f64) -> usize {
        let at = self.point2(x, y);
        let (c, s) = (turn.to_radians().cos(), turn.to_radians().sin());
        let dir = self.add(format!("IFCDIRECTION(({c:.12},{s:.12}))"));
        self.add(format!("IFCAXIS2PLACEMENT2D(#{at},#{dir})"))
    }

    fn shape(&mut self, item: usize) -> usize {
        let rep = self.add(format!("IFCSHAPEREPRESENTATION(#2,'Body','SweptSolid',(#{item}))"));
        self.add(format!("IFCPRODUCTDEFINITIONSHAPE($,$,(#{rep}))"))
    }

    /// The solid of `profile` extruded `depth` up from `z`.
    fn extrusion(&mut self, profile: usize, z: f64, depth: f64) -> usize {
        let at = self.add(format!("IFCCARTESIANPOINT((0.,0.,{z:.9}))"));
        let position = self.add(format!("IFCAXIS2PLACEMENT3D(#{at},$,$)"));
        self.add(format!("IFCEXTRUDEDAREASOLID(#{profile},#{position},#9,{depth:.9})"))
    }

    fn extruded(&mut self, profile: usize, z: f64, depth: f64) -> usize {
        let solid = self.extrusion(profile, z, depth);
        self.shape(solid)
    }

    /// `profile`, drawn in the vertical plane through the origin along X
    /// (its `x` along X, its `y` up from `z`), revolved `angle` radians about
    /// `axis` (a direction in the profile's plane through `at`).
    fn revolved_about(
        &mut self,
        profile: usize,
        z: f64,
        angle: f64,
        at: (f64, f64, f64),
        axis: (f64, f64, f64),
    ) -> usize {
        let origin = self.add(format!("IFCCARTESIANPOINT((0.,0.,{z:.9}))"));
        let normal = self.add("IFCDIRECTION((0.,-1.,0.))".into());
        let x = self.add("IFCDIRECTION((1.,0.,0.))".into());
        let position = self.add(format!("IFCAXIS2PLACEMENT3D(#{origin},#{normal},#{x})"));
        let point = self.add(format!("IFCCARTESIANPOINT(({:.9},{:.9},{:.9}))", at.0, at.1, at.2));
        let dir = self.add(format!("IFCDIRECTION(({:.9},{:.9},{:.9}))", axis.0, axis.1, axis.2));
        let axis = self.add(format!("IFCAXIS1PLACEMENT(#{point},#{dir})"));
        let solid = self.add(format!("IFCREVOLVEDAREASOLID(#{profile},#{position},#{axis},{angle:.12})"));
        self.shape(solid)
    }

    /// `profile` revolved about the vertical axis through the origin.
    fn revolved(&mut self, profile: usize, z: f64, angle: f64) -> usize {
        self.revolved_about(profile, z, angle, (0.0, 0.0, 0.0), (0.0, 1.0, 0.0))
    }

    /// A closed polyline profile through `pts`.
    fn polygon(&mut self, pts: &[(f64, f64)]) -> usize {
        let mut refs: Vec<String> = pts.iter().map(|(x, y)| format!("#{}", self.point2(*x, *y))).collect();
        refs.push(refs[0].clone());
        let line = self.add(format!("IFCPOLYLINE(({}))", refs.join(",")));
        self.add(format!("IFCARBITRARYCLOSEDPROFILEDEF(.AREA.,$,#{line})"))
    }

    fn rectangle(&mut self, x: f64, y: f64, turn: f64, w: f64, h: f64) -> usize {
        let at = self.place2(x, y, turn);
        self.add(format!("IFCRECTANGLEPROFILEDEF(.AREA.,$,#{at},{w:.9},{h:.9})"))
    }

    fn circle(&mut self, x: f64, y: f64, r: f64) -> usize {
        let at = self.place2(x, y, 0.0);
        self.add(format!("IFCCIRCLEPROFILEDEF(.AREA.,$,#{at},{r:.9})"))
    }

    /// The piece of the ring between `r_in` and `r_out` from the angle
    /// `from` to `to` (degrees, counter-clockwise), as a closed profile of two
    /// trimmed circles and two radial lines.
    fn sector(&mut self, r_in: f64, r_out: f64, from: f64, to: f64) -> usize {
        let (a, b) = (from.to_radians(), to.to_radians());
        let mut at = |r: f64, t: f64| self.point2(r * t.cos(), r * t.sin());
        let (p0, p1, p2, p3) = (at(r_out, a), at(r_out, b), at(r_in, b), at(r_in, a));
        let outer = self.add(format!("IFCCIRCLE(#10,{r_out:.9})"));
        let inner = self.add(format!("IFCCIRCLE(#10,{r_in:.9})"));
        let parts = [
            self.add(format!("IFCTRIMMEDCURVE(#{outer},(#{p0}),(#{p1}),.T.,.CARTESIAN.)")),
            self.add(format!("IFCPOLYLINE((#{p1},#{p2}))")),
            self.add(format!("IFCTRIMMEDCURVE(#{inner},(#{p2}),(#{p3}),.F.,.CARTESIAN.)")),
            self.add(format!("IFCPOLYLINE((#{p3},#{p0}))")),
        ];
        let segments: Vec<String> = parts
            .iter()
            .map(|c| format!("#{}", self.add(format!("IFCCOMPOSITECURVESEGMENT(.CONTINUOUS.,.T.,#{c})"))))
            .collect();
        let curve = self.add(format!("IFCCOMPOSITECURVE(({}),.F.)", segments.join(",")));
        self.add(format!("IFCARBITRARYCLOSEDPROFILEDEF(.AREA.,'ring sector',#{curve})"))
    }

    /// A product of `class` named `name`, with `rest` after its tag.
    fn product(&mut self, class: &str, name: &str, shape: usize, rest: &str) -> usize {
        let guid = format!("2{:021}", self.lines.len());
        let id = self.add(format!("{class}('{guid}',$,'{name}',$,$,#6,#{shape},${rest})"));
        self.products.push(id);
        id
    }

    fn footing(&mut self, name: &str, shape: usize, kind: &str) -> usize {
        self.product("IFCFOOTING", name, shape, &format!(",.{kind}."))
    }

    /// A round pile 30 cm across at `(x, y)` with its head at `head`, 12 m long.
    fn pile(&mut self, x: f64, y: f64, head: f64) {
        let profile = self.circle(x, y, 0.15);
        let shape = self.extruded(profile, head - 12.0, 12.0);
        self.product("IFCPILE", "Pile 30", shape, ",.BORED.,.CAST_IN_PLACE.");
    }

    /// An opening of `shape` through `host`.
    fn opening(&mut self, host: usize, shape: usize) -> usize {
        let guid = format!("3{:021}", self.lines.len());
        let opening = self.add(format!("IFCOPENINGELEMENT('{guid}',$,'Hole',$,$,#6,#{shape},$)"));
        let guid = format!("4{:021}", self.lines.len());
        self.add(format!("IFCRELVOIDSELEMENT('{guid}',$,$,$,#{host},#{opening})"));
        opening
    }

    fn text(mut self) -> String {
        let all: Vec<String> = self.products.iter().map(|p| format!("#{p}")).collect();
        let contained =
            format!("IFCRELCONTAINEDINSPATIALSTRUCTURE('3XvctVUKr0kugbFTf53O9L',$,$,$,({}),#5)", all.join(","));
        self.add(contained);
        let mut out = String::from("ISO-10303-21;\nHEADER;\nFILE_SCHEMA(('IFC2X3'));\nENDSEC;\nDATA;\n");
        for (i, line) in self.lines.iter().enumerate() {
            out.push_str(&format!("#{}={line};\n", i + 1));
        }
        out.push_str("ENDSEC;\nEND-ISO-10303-21;\n");
        out
    }
}

fn load(f: File) -> (Project, ModelGeometry) {
    let (p, _) = import(&f.text(), Project::from_template()).unwrap();
    let geom = ModelGeometry::build(&p);
    (p, geom)
}

/// Within 1e-6 of the value (the issue asks for 1e-4).
fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() <= 1e-6 * b.abs().max(1.0), "{what}: {a} vs {b}");
}

fn of(p: &Project, category: Category) -> Vec<&Element> {
    p.elements_of(category).collect()
}

fn total_volume(geom: &ModelGeometry, elements: &[&Element]) -> f64 {
    elements.iter().map(|e| geom.volume(e.id)).sum()
}

/// The middle radius of the walls of the tanks: 2.3 m, 20 cm thick, 3.38 m high.
const R: f64 = 2.3;
const WALL: f64 = 0.2;
const HEIGHT: f64 = 3.38;

/// The tank's cylinder: 2 pi R t h.
fn cylinder_volume() -> f64 {
    TAU * R * WALL * HEIGHT
}

/// The two arc walls of a full ring, of one 20 cm type, on the middle circle.
fn check_ring_walls(p: &Project, geom: &ModelGeometry) {
    assert!(of(p, Category::Component).is_empty(), "no component");
    let walls = of(p, Category::Wall);
    assert_eq!(walls.len(), 2, "a ring is two half circles");
    for w in &walls {
        let ElementKind::Wall(wall) = &w.kind else { panic!("a wall") };
        let arc = wall.arc().expect("an arc wall");
        close(arc.radius, R, "radius");
        assert!(arc.center.length() < 1e-9);
        close(arc.sweep.abs(), PI, "half a turn");
        assert!(arc.sweep < 0.0, "clockwise, the exterior side out");
        close(p.thickness_of(w), WALL, "thickness");
        close(p.wall_height(w, wall), HEIGHT, "height");
    }
    assert_eq!(walls[0].type_id, walls[1].type_id);
    // 9.7691 m³ in the issue.
    close(total_volume(geom, &walls), cylinder_volume(), "volume");
    assert!((total_volume(geom, &walls) - 9.7691).abs() < 1e-4);
}

#[test]
fn a_hollow_circle_wall_is_two_arc_walls() {
    let mut f = File::new();
    let profile = f.add(format!("IFCCIRCLEHOLLOWPROFILEDEF(.AREA.,$,#10,{},{WALL})", R + WALL / 2.0));
    let shape = f.extruded(profile, 0.0, HEIGHT);
    f.product("IFCWALL", "Parede concreto 20 cm", shape, "");
    let (p, geom) = load(f);
    check_ring_walls(&p, &geom);
    // The two halves meet end to end.
    let walls = of(&p, Category::Wall);
    let ends = |e: &Element| match &e.kind {
        ElementKind::Wall(w) => (w.start, w.end),
        _ => panic!(),
    };
    let ((a0, a1), (b0, b1)) = (ends(walls[0]), ends(walls[1]));
    assert!(a1.distance(b0) < 1e-9 && b1.distance(a0) < 1e-9);
}

#[test]
fn a_profile_of_two_concentric_circles_is_a_ring_of_walls() {
    let mut f = File::new();
    let outer = f.add(format!("IFCCIRCLE(#10,{})", R + WALL / 2.0));
    let inner = f.add(format!("IFCCIRCLE(#10,{})", R - WALL / 2.0));
    let profile = f.add(format!("IFCARBITRARYPROFILEDEFWITHVOIDS(.AREA.,$,#{outer},(#{inner}))"));
    let shape = f.extruded(profile, 0.0, HEIGHT);
    f.product("IFCWALLSTANDARDCASE", "Ring", shape, "");
    let (p, geom) = load(f);
    check_ring_walls(&p, &geom);
}

#[test]
fn a_band_of_a_ring_is_an_arc_wall() {
    // A quarter, three quarters, and 350 degrees (more than one bulge holds: two halves).
    for (to, pieces) in [(120.0, 1), (300.0, 1), (380.0, 2)] {
        let mut f = File::new();
        let profile = f.sector(R - WALL / 2.0, R + WALL / 2.0, 30.0, to);
        let shape = f.extruded(profile, 0.0, HEIGHT);
        f.product("IFCWALL", "Arc", shape, "");
        let (p, geom) = load(f);
        let walls = of(&p, Category::Wall);
        assert_eq!(walls.len(), pieces, "to {to}");
        let sweep = (to - 30.0f64).to_radians();
        let mut turned = 0.0;
        for w in &walls {
            let ElementKind::Wall(wall) = &w.kind else { panic!() };
            let arc = wall.arc().expect("an arc wall");
            close(arc.radius, R, "radius");
            turned += arc.sweep.abs();
        }
        close(turned, sweep, "the band's turn");
        // The ring's area times the height: sweep / 2 (r_out² - r_in²) h.
        let exact = sweep / 2.0 * ((R + WALL / 2.0).powi(2) - (R - WALL / 2.0).powi(2)) * HEIGHT;
        close(total_volume(&geom, &walls), exact, "volume");
        // It starts at 30 degrees, or ends there: clockwise.
        let first = match &walls.last().unwrap().kind {
            ElementKind::Wall(w) => w.end,
            _ => panic!(),
        };
        assert!(first.distance(glam::DVec2::from_angle(30f64.to_radians()) * R) < 1e-6, "{first}");
    }
}

#[test]
fn a_door_in_a_ring_wall_is_hosted_by_the_half_it_stands_in() {
    let mut f = File::new();
    let profile = f.add(format!("IFCCIRCLEHOLLOWPROFILEDEF(.AREA.,$,#10,{},{WALL})", R + WALL / 2.0));
    let shape = f.extruded(profile, 0.0, HEIGHT);
    let wall = f.product("IFCWALL", "Ring", shape, "");
    // An opening 0.9 m wide and 2.1 m high through the wall at 250 degrees.
    let angle = 250f64.to_radians();
    let at = glam::DVec2::from_angle(angle) * R;
    let hole = f.rectangle(at.x, at.y, angle.to_degrees() + 90.0, 0.9, 0.6);
    let hole = f.extruded(hole, 0.0, 2.1);
    let opening = f.opening(wall, hole);
    let guid = format!("5{:021}", f.lines.len());
    let door = f.add(format!("IFCDOOR('{guid}',$,'Door',$,$,#6,$,$,2.1,0.9)"));
    let guid = format!("6{:021}", f.lines.len());
    f.add(format!("IFCRELFILLSELEMENT('{guid}',$,$,$,#{opening},#{door})"));
    let (p, _) = load(f);
    let door = of(&p, Category::Door);
    let ElementKind::Door(hosted) = &door[0].kind else { panic!("a door") };
    let host = p.element(hosted.host).unwrap();
    let ElementKind::Wall(w) = &host.kind else { panic!("in a wall") };
    // Its middle on the wall's arc, inside the half that holds it.
    let point = w.arc().unwrap().point(hosted.offset);
    assert!(point.distance(at) < 1e-3, "{point} vs {at}");
    assert!(hosted.offset > 0.45 && hosted.offset < w.length() - 0.45);
}

/// The haunched ring footing of the tank: 1.40 m wide, 0.45 m thick at the
/// beam's faces and 0.25 m at its edges, under a beam 0.40 m wide and 0.45
/// m high, its middle on the circle of 2.30 m.
fn haunched_section(f: &mut File) -> usize {
    let (a, b, c) = (R - 0.7, R - 0.2, R + 0.2);
    let d = R + 0.7;
    f.polygon(&[(a, 0.0), (d, 0.0), (d, 0.25), (c, 0.45), (c, 0.9), (b, 0.9), (b, 0.45), (a, 0.25)])
}

/// Its area: the two slopes and the middle under and in the beam.
const HAUNCHED_AREA: f64 = 2.0 * 0.5 * (0.45 + 0.25) / 2.0 + 0.4 * 0.9;

fn strip_type(p: &Project, e: &Element) -> (f64, f64, Option<StripHaunch>) {
    match p.element_type(e.type_id).map(|t| &t.data) {
        Some(TypeData::Foundation { kind: FoundationKind::Strip, width, thickness, haunch, .. }) => {
            (*width, *thickness, *haunch)
        }
        other => panic!("a strip type: {other:?}"),
    }
}

#[test]
fn a_revolved_haunched_section_is_a_ring_of_haunched_strips() {
    let mut f = File::new();
    let profile = haunched_section(&mut f);
    let shape = f.revolved(profile, -0.9, TAU);
    f.footing("Sapata corrida anel 140", shape, "STRIP_FOOTING");
    let (p, geom) = load(f);
    assert!(of(&p, Category::Component).is_empty());
    let strips = of(&p, Category::Foundation);
    assert_eq!(strips.len(), 2, "a ring is two half circles");
    for s in &strips {
        let ElementKind::Foundation(found) = &s.kind else { panic!() };
        let arc = found.shape.arc().expect("an arc strip");
        close(arc.radius, R, "radius");
        close(arc.sweep.abs(), PI, "half a turn");
        let (width, thickness, haunch) = strip_type(&p, s);
        close(width, 1.4, "width");
        close(thickness, 0.45, "thickness");
        let h = haunch.expect("an inverted T");
        close(h.edge_thickness, 0.25, "edge");
        close(h.beam_width, 0.4, "beam width");
        close(h.beam_height, 0.45, "beam height");
        // Its top is the beam's, at 0.
        let g = &geom.foundations[&s.id];
        close(g.z_range.1, 0.0, "top");
        close(g.z_range.0, -0.9, "bottom");
    }
    // Pappus: the section's area times the path of its middle, 10.2604 m³ in the issue.
    let exact = HAUNCHED_AREA * TAU * R;
    close(total_volume(&geom, &strips), exact, "volume");
    assert!((exact - 10.2604).abs() < 1e-4);
}

#[test]
fn a_rectangle_revolved_part_way_is_an_arc_strip() {
    // 0.6 x 0.3 m about the axis, turned 120 degrees one way, then the other.
    for angle in [120f64.to_radians(), -120f64.to_radians()] {
        let mut f = File::new();
        let profile = f.rectangle(4.0, 0.15, 0.0, 0.6, 0.3);
        let shape = f.revolved(profile, -0.3, angle);
        f.footing("Strip", shape, "STRIP_FOOTING");
        let (p, geom) = load(f);
        let strips = of(&p, Category::Foundation);
        assert_eq!(strips.len(), 1);
        let ElementKind::Foundation(found) = &strips[0].kind else { panic!() };
        let arc = found.shape.arc().expect("an arc strip");
        close(arc.radius, 4.0, "radius");
        close(arc.sweep.abs(), angle.abs(), "turn");
        // It starts (or ends) on the profile's plane, along X.
        let FoundationShape::Strip { start, end, .. } = found.shape else { panic!() };
        let on_x = |q: glam::DVec2| (q - glam::DVec2::new(4.0, 0.0)).length() < 1e-9;
        assert!(on_x(start) || on_x(end), "{start} {end}");
        let middle = arc.point(arc.length() / 2.0);
        assert!(middle.y * angle.signum() > 0.0, "it turns the way the file does: {middle}");
        assert_eq!(strip_type(&p, strips[0]).2, None);
        close(geom.volume(strips[0].id), 0.6 * 0.3 * 4.0 * angle.abs(), "volume");
    }
}

/// The ring pile-cap beam of MP04: 50 x 70 cm on the middle circle.
fn ring_beam(f: &mut File, kind: &str) {
    let profile = f.rectangle(R, 0.35, 0.0, 0.5, 0.7);
    let shape = f.revolved(profile, -0.7, TAU);
    f.footing("Viga coroamento 50x70", shape, kind);
}

/// Its volume, 5.0580 m³ in the issue.
fn ring_beam_volume() -> f64 {
    0.5 * 0.7 * TAU * R
}

#[test]
fn a_ring_pile_cap_on_its_piles_is_a_native_ring_cap() {
    let mut f = File::new();
    // The piles first, half of them after the cap: the cap waits for all of them.
    for k in 0..3 {
        let a = (60.0 * f64::from(k) + 30.0).to_radians();
        f.pile(R * a.cos(), R * a.sin(), -0.6);
    }
    ring_beam(&mut f, "PILE_CAP");
    for k in 3..6 {
        let a = (60.0 * f64::from(k) + 30.0).to_radians();
        f.pile(R * a.cos(), R * a.sin(), -0.6);
    }
    let (p, geom) = load(f);
    assert!(of(&p, Category::Component).is_empty());
    let caps: Vec<_> = p.elements.values().filter(|e| p.pile_cap(e).is_some()).collect();
    assert_eq!(caps.len(), 1, "one ring cap");
    let c = p.pile_cap(caps[0]).unwrap();
    assert_eq!(c.cap.layout, PileLayout::Circle { count: 6, radius: R, ring_width: Some(0.5) });
    close(c.thickness, 0.7, "thickness");
    close(c.cap.embedment, 0.1, "embedment");
    close(c.cap.pile_length, 12.0, "pile length");
    // Its six piles are the cap's, where they stood.
    let piles: Vec<_> = p.piles_of(caps[0].id).collect();
    assert_eq!(piles.len(), 6);
    for (_, pile) in &piles {
        let index = pile.cap.unwrap().index as usize;
        assert!(pile.position.distance(c.pile_positions()[index]) < 1e-6);
        close(pile.position.length(), R, "on the ring");
        close(p.level_elevation(caps[0].level) + pile.cut_off, -0.6, "head");
    }
    close(geom.volume(caps[0].id), ring_beam_volume(), "volume");
    assert!((geom.volume(caps[0].id) - 5.0580).abs() < 1e-4);
    // Nothing for the sync or the repair to change.
    assert_eq!(p.clone().repair(), 0);
    let mut doc = cc_core::Document::new(p.clone());
    let before = doc.project.elements.len();
    doc.edit("rename", |ctx| ctx.info_mut().name.push('!'));
    assert_eq!(doc.project.elements.len(), before, "no piles added or removed");
}

#[test]
fn a_ring_footing_without_its_piles_is_a_ring_of_strips() {
    // A pile cap whose piles stand off the ring's middle circle, and one of no type.
    for kind in ["PILE_CAP", "NOTDEFINED"] {
        let mut f = File::new();
        ring_beam(&mut f, kind);
        for k in 0..6 {
            let a = (60.0 * f64::from(k)).to_radians();
            f.pile(2.2 * a.cos(), 2.2 * a.sin(), -0.6);
        }
        let (p, geom) = load(f);
        assert!(of(&p, Category::Component).is_empty(), "{kind}");
        let strips: Vec<_> = of(&p, Category::Foundation);
        assert_eq!(strips.len(), 2, "{kind}");
        assert!(strips.iter().all(|s| p.pile_cap(s).is_none()));
        let (width, thickness, haunch) = strip_type(&p, strips[0]);
        close(width, 0.5, "width");
        close(thickness, 0.7, "thickness");
        assert_eq!(haunch, None);
        close(total_volume(&geom, &strips), ring_beam_volume(), "volume");
        // The piles stay free.
        assert_eq!(p.elements_of(Category::Pile).count(), 6);
    }
}

#[test]
fn a_hollow_circle_footing_is_a_ring_of_strips() {
    let mut f = File::new();
    let profile = f.add(format!("IFCCIRCLEHOLLOWPROFILEDEF(.AREA.,$,#10,{},0.5)", R + 0.25));
    let shape = f.extruded(profile, -0.7, 0.7);
    f.footing("Ring", shape, "STRIP_FOOTING");
    let (p, geom) = load(f);
    let strips = of(&p, Category::Foundation);
    assert_eq!(strips.len(), 2);
    close(total_volume(&geom, &strips), ring_beam_volume(), "volume");
}

#[test]
fn pads_strips_and_pile_caps_of_rectangles_are_native() {
    let mut f = File::new();
    // A pad 1.2 x 0.8 x 0.4 m turned 30 degrees.
    let profile = f.rectangle(10.0, 0.0, 30.0, 1.2, 0.8);
    let shape = f.extruded(profile, -0.4, 0.4);
    f.footing("Sapata 120x80", shape, "PAD_FOOTING");
    // A cap on two piles: 1.5 x 0.6 x 0.6 m, the piles 0.9 m apart.
    let profile = f.rectangle(20.0, 0.0, 90.0, 1.5, 0.6);
    let shape = f.extruded(profile, -0.6, 0.6);
    f.footing("Bloco 2 estacas", shape, "PILE_CAP");
    f.pile(20.0, -0.45, -0.5);
    f.pile(20.0, 0.45, -0.5);
    // The same cap turned 50 degrees, its points rounded by the file.
    let profile = f.rectangle(60.0, 0.0, 50.0, 1.5, 0.6);
    let shape = f.extruded(profile, -0.6, 0.6);
    f.footing("Bloco 2 estacas", shape, "PILE_CAP");
    let u = glam::DVec2::from_angle(50f64.to_radians()) * 0.45;
    f.pile(60.0 - u.x, -u.y, -0.5);
    f.pile(60.0 + u.x, u.y, -0.5);
    // A strip 6 x 0.6 x 0.3 m drawn in plan.
    let profile = f.rectangle(30.0, 0.0, 0.0, 6.0, 0.6);
    let shape = f.extruded(profile, -0.3, 0.3);
    f.footing("Strip 60", shape, "STRIP_FOOTING");
    // The haunched section, drawn in the plane of X and Z, extruded 5 m level towards -Y.
    let section = {
        let (a, b, c, d) = (-0.7, -0.2, 0.2, 0.7);
        f.polygon(&[(a, 0.0), (d, 0.0), (d, 0.25), (c, 0.45), (c, 0.9), (b, 0.9), (b, 0.45), (a, 0.25)])
    };
    let origin = f.add("IFCCARTESIANPOINT((40.,0.,-0.9))".into());
    let normal = f.add("IFCDIRECTION((0.,-1.,0.))".into());
    let x = f.add("IFCDIRECTION((1.,0.,0.))".into());
    let position = f.add(format!("IFCAXIS2PLACEMENT3D(#{origin},#{normal},#{x})"));
    let solid = f.add(format!("IFCEXTRUDEDAREASOLID(#{section},#{position},#9,5.)"));
    let shape = f.shape(solid);
    f.footing("Strip T", shape, "STRIP_FOOTING");
    let (p, geom) = load(f);
    assert!(of(&p, Category::Component).is_empty(), "{:?}", of(&p, Category::Component));
    let at = |x: f64| {
        *of(&p, Category::Foundation)
            .iter()
            .find(|e| geom.foundations[&e.id].footprint.iter().all(|q| (q.x - x).abs() < 3.5))
            .unwrap_or_else(|| panic!("a foundation at {x}"))
    };
    // The pad.
    let pad = at(10.0);
    let ElementKind::Foundation(found) = &pad.kind else { panic!() };
    let FoundationShape::Pad { position, rotation_deg } = found.shape else { panic!("a pad") };
    assert!(position.distance(glam::DVec2::new(10.0, 0.0)) < 1e-9);
    assert!(((rotation_deg - 30.0).rem_euclid(180.0)).min(180.0 - (rotation_deg - 30.0).rem_euclid(180.0)) < 1e-6);
    assert!(matches!(
        p.element_type(pad.type_id).map(|t| &t.data),
        Some(TypeData::Foundation { kind: FoundationKind::Pad, width, length, .. }) if (width - 1.2).abs() < 1e-9 && (length - 0.8).abs() < 1e-9
    ));
    close(geom.volume(pad.id), 1.2 * 0.8 * 0.4, "pad volume");
    // The two-pile cap, its piles the cap's.
    let cap = at(20.0);
    let c = p.pile_cap(cap).expect("a pile cap");
    assert_eq!(c.cap.layout, PileLayout::Pair);
    close(c.cap.spacing, 0.9, "spacing");
    close(c.cap.edge_distance, 0.3, "edge distance");
    close(c.cap.embedment, 0.1, "embedment");
    assert_eq!(p.piles_of(cap.id).count(), 2);
    close(geom.volume(cap.id), 1.5 * 0.6 * 0.6, "cap volume");
    let turned = at(60.0);
    assert_eq!(turned.type_id, cap.type_id, "caps alike share a type");
    assert_eq!(p.piles_of(turned.id).count(), 2);
    close(geom.volume(turned.id), 1.5 * 0.6 * 0.6, "turned cap volume");
    // The strips.
    let strip = at(30.0);
    let ElementKind::Foundation(found) = &strip.kind else { panic!() };
    let FoundationShape::Strip { start, end, bulge } = found.shape else { panic!("a strip") };
    assert!(bulge == 0.0 && (start.distance(end) - 6.0).abs() < 1e-9 && (start.y).abs() < 1e-9);
    close(geom.volume(strip.id), 6.0 * 0.6 * 0.3, "strip volume");
    let t = at(40.0);
    let ElementKind::Foundation(found) = &t.kind else { panic!() };
    let FoundationShape::Strip { start, end, .. } = found.shape else { panic!("a strip") };
    assert!(start.distance(glam::DVec2::new(40.0, 0.0)) < 1e-9 && end.distance(glam::DVec2::new(40.0, -5.0)) < 1e-9);
    assert!(strip_type(&p, t).2.is_some(), "an inverted T");
    close(geom.volume(t.id), HAUNCHED_AREA * 5.0, "haunched strip volume");
}

#[test]
fn a_circular_hole_stays_a_circle_in_the_slab() {
    // The slab of the issue: a disk of 2.95 m with a hole of 1.25 m, 5 cm thick.
    let mut f = File::new();
    let profile = f.circle(0.0, 0.0, 2.95);
    let shape = f.extruded(profile, 3.0, 0.05);
    let slab = f.product("IFCSLAB", "Round slab", shape, ",.FLOOR.");
    let hole = f.circle(0.0, 0.0, 1.25);
    let hole = f.extruded(hole, 2.9, 0.25);
    f.opening(slab, hole);
    // A square slab with a round hole too.
    let profile = f.rectangle(10.0, 0.0, 0.0, 4.0, 4.0);
    let shape = f.extruded(profile, 3.0, 0.2);
    let square = f.product("IFCSLAB", "Square slab", shape, ",.FLOOR.");
    let hole = f.circle(10.5, 0.5, 0.5);
    let hole = f.extruded(hole, 2.9, 0.4);
    f.opening(square, hole);
    let (p, geom) = load(f);
    let floors = of(&p, Category::Floor);
    assert_eq!(floors.len(), 2);
    let floor = |name: &str| *floors.iter().find(|e| p.type_name(e.type_id).starts_with(name)).unwrap();
    let circles = |e: &Element| match &e.kind {
        ElementKind::Floor(fl) => fl
            .sketch
            .as_ref()
            .expect("a sketch")
            .curves
            .iter()
            .filter(|c| matches!(c, SketchCurve::Circle { .. }))
            .count(),
        _ => panic!("a floor"),
    };
    let round = floor("Round slab");
    assert_eq!(circles(round), 2, "the outline and the hole");
    // 1.12155 m³ in the issue.
    let exact = PI * (2.95f64.powi(2) - 1.25f64.powi(2)) * 0.05;
    close(geom.volume(round.id), exact, "volume");
    assert!((geom.volume(round.id) - 1.12155).abs() < 1e-4 * 1.12155);
    let square = floor("Square slab");
    assert_eq!(circles(square), 1, "the hole");
    close(geom.volume(square.id), (16.0 - PI * 0.25) * 0.2, "square slab volume");
}

/// What no native element holds, and hostile values, come in as before.
#[test]
fn other_shapes_and_hostile_values_stay_generic() {
    let mut f = File::new();
    // A section crossing the axis; an axis along X; an axis off the profile's plane.
    let crossing = f.rectangle(0.1, 0.15, 0.0, 0.6, 0.3);
    let shape = f.revolved(crossing, 0.0, TAU);
    f.footing("Crossing", shape, "STRIP_FOOTING");
    let tilted = f.rectangle(4.0, 0.15, 0.0, 0.6, 0.3);
    let shape = f.revolved_about(tilted, 0.0, TAU, (0.0, 0.0, 0.0), (1.0, 0.0, 0.0));
    f.footing("Tilted", shape, "STRIP_FOOTING");
    let off = f.rectangle(4.0, 0.15, 0.0, 0.6, 0.3);
    let shape = f.revolved_about(off, 0.0, TAU, (0.0, 0.0, 0.5), (0.0, 1.0, 0.0));
    f.footing("Off plane", shape, "STRIP_FOOTING");
    // A section no strip type has (an L), revolved.
    let l = f.polygon(&[(3.0, 0.0), (4.0, 0.0), (4.0, 0.3), (3.3, 0.3), (3.3, 1.0), (3.0, 1.0)]);
    let shape = f.revolved(l, 0.0, TAU);
    f.footing("L", shape, "STRIP_FOOTING");
    // A hollow section revolved: its void is no strip's.
    let at = f.place2(4.0, 0.2, 0.0);
    let hollow = f.add(format!("IFCRECTANGLEHOLLOWPROFILEDEF(.AREA.,$,#{at},0.6,0.4,0.05,$,$)"));
    let shape = f.revolved(hollow, 0.0, TAU);
    f.footing("Hollow", shape, "STRIP_FOOTING");
    // A wall as thick as its radius (a disk), one of a huge radius, a round pad.
    let full = f.add("IFCCIRCLEHOLLOWPROFILEDEF(.AREA.,$,#10,1.,1.)".into());
    let shape = f.extruded(full, 0.0, 3.0);
    f.product("IFCWALL", "Disk", shape, "");
    let huge = f.add("IFCCIRCLEHOLLOWPROFILEDEF(.AREA.,$,#10,1.E300,0.2)".into());
    let shape = f.extruded(huge, 0.0, 3.0);
    f.product("IFCWALL", "Huge", shape, "");
    let round = f.circle(30.0, 0.0, 0.6);
    let shape = f.extruded(round, -0.5, 0.5);
    f.footing("Round pad", shape, "PAD_FOOTING");
    // A cap on three piles in no layout, and a rectangle of no type.
    let profile = f.rectangle(40.0, 0.0, 0.0, 2.0, 1.0);
    let shape = f.extruded(profile, -0.6, 0.6);
    f.footing("Bloco 3", shape, "PILE_CAP");
    for x in [39.3, 39.9, 40.8] {
        f.pile(x, 0.0, -0.5);
    }
    let profile = f.rectangle(50.0, 0.0, 0.0, 2.0, 1.0);
    let shape = f.extruded(profile, -0.6, 0.6);
    f.footing("Block", shape, "NOTDEFINED");
    // Revolutions of no turn.
    let still = f.rectangle(4.0, 0.15, 0.0, 0.6, 0.3);
    let shape = f.revolved(still, 0.0, 0.0);
    f.footing("Still", shape, "STRIP_FOOTING");
    let (p, _) = import(&f.text(), Project::from_template()).unwrap();
    let names: Vec<String> = of(&p, Category::Component).iter().map(|e| p.type_name(e.type_id).to_string()).collect();
    for name in ["Crossing", "Tilted", "Off plane", "L", "Hollow", "Disk", "Round pad", "Bloco 3", "Block"] {
        assert!(names.iter().any(|n| n.starts_with(name)), "{name} is a component: {names:?}");
    }
    assert!(of(&p, Category::Wall).is_empty() && of(&p, Category::Foundation).is_empty());
    let free = |e: &&Element| matches!(&e.kind, ElementKind::Pile(x) if x.cap.is_none());
    assert_eq!(p.elements_of(Category::Pile).filter(free).count(), 3);
    // And the result builds.
    let _ = ModelGeometry::build(&p);
}

/// This program's own pads and plain strips, straight and arc, apart from
/// each other, come back as the same native footings, of the same types
/// and volumes.
#[test]
fn this_programs_pads_and_strips_come_back_native() {
    let mut p = Project::from_template();
    let level = p.levels[0].id;
    let type_of = |p: &mut Project, kind: FoundationKind, width: f64, length: f64, thickness: f64| {
        let id = cc_core::ElementTypeId(p.alloc_id());
        let data = TypeData::Foundation {
            kind,
            width,
            length,
            thickness,
            material: "Concreto".into(),
            haunch: None,
            pile_cap: None,
        };
        let name = format!("{kind:?} {width} x {length}");
        p.types.insert(
            id,
            cc_core::ElementType { id, name, data, info: Default::default(), uid: cc_core::Uid::generate() },
        );
        id
    };
    let pad = type_of(&mut p, FoundationKind::Pad, 1.2, 0.8, 0.4);
    let strip = type_of(&mut p, FoundationKind::Strip, 0.6, 1.0, 0.3);
    let shapes = [
        (pad, FoundationShape::Pad { position: glam::DVec2::new(10.0, 0.0), rotation_deg: 30.0 }),
        (
            strip,
            FoundationShape::Strip { start: glam::DVec2::new(20.0, 0.0), end: glam::DVec2::new(26.0, 0.0), bulge: 0.0 },
        ),
        (
            strip,
            FoundationShape::Strip { start: glam::DVec2::new(34.0, 0.0), end: glam::DVec2::new(30.0, 4.0), bulge: 0.4 },
        ),
    ];
    for (type_id, shape) in shapes {
        let id = cc_core::ElementId(p.alloc_id());
        let kind = ElementKind::Foundation(cc_core::Foundation::new(shape, -0.2));
        let e = Element {
            id,
            level,
            type_id,
            mark: String::new(),
            comments: String::new(),
            kind,
            uid: cc_core::Uid::generate(),
            phase_created: Default::default(),
            phase_demolished: None,
            design_option: None,
            paints: Vec::new(),
        };
        p.elements.insert(id, e);
    }
    let geom = ModelGeometry::build(&p);
    let options = cc_ifc::ExportOptions { time_stamp: Some("2026-01-01T00:00:00".into()), file_name: "t.ifc".into() };
    let (back, _) = import(&cc_ifc::export(&p, &options), Project::from_template()).unwrap();
    let again = ModelGeometry::build(&back);
    assert!(of(&back, Category::Component).is_empty());
    for e in p.elements_of(Category::Foundation) {
        let i = back.elements.values().find(|x| x.uid == e.uid).expect("it comes back");
        let (ElementKind::Foundation(a), ElementKind::Foundation(b)) = (&e.kind, &i.kind) else { panic!() };
        assert_eq!(std::mem::discriminant(&a.shape), std::mem::discriminant(&b.shape), "{:?}", b.shape);
        assert_eq!(back.type_name(i.type_id), p.type_name(e.type_id));
        close(again.volume(i.id), geom.volume(e.id), "volume");
        close(b.top_offset, a.top_offset, "top");
        let (x, y) = (&geom.foundations[&e.id], &again.foundations[&i.id]);
        close(y.area(), x.area(), "area");
    }
}
