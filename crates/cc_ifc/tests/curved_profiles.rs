//! Slabs drawn with circles and arcs are written with exact curves (a lone
//! circle as `IfcCircleProfileDef`, other outlines as composite curves of
//! polylines and trimmed circles) and come back with the same curves, so
//! their areas and volumes are exact (#368). Trimmed circles of other
//! programs read as one arc, by parameter or by point and in either sense;
//! broken curves fall back to the chords or leave the slab out, never panic.

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_core::sketch::{Sketch, SketchCurve};
use cc_core::{Category, Element, ElementId, ElementKind, Floor, Foundation, FoundationShape, Project, TypeData, Uid};
use cc_geom::ModelGeometry;
use cc_ifc::{ExportOptions, export, import};
use glam::DVec2;
use std::f64::consts::PI;

fn add(p: &mut Project, type_id: cc_core::ElementTypeId, kind: ElementKind) -> ElementId {
    let id = ElementId(p.alloc_id());
    let level = p.levels[0].id;
    let e = Element {
        id,
        level,
        type_id,
        mark: String::new(),
        comments: String::new(),
        kind,
        uid: Uid::generate(),
        phase_created: Default::default(),
        phase_demolished: None,
        design_option: None,
        paints: Vec::new(),
    };
    p.elements.insert(id, e);
    id
}

fn floor(p: &mut Project, sketch: Sketch) -> ElementId {
    let profile = sketch.profile().unwrap();
    let mut floor = Floor::new(profile.outline, 0.0);
    floor.openings = profile.openings;
    floor.sketch = Some(sketch);
    let type_id = p.default_type(Category::Floor).unwrap();
    add(p, type_id, ElementKind::Floor(floor))
}

fn raft(p: &mut Project, sketch: Sketch) -> ElementId {
    let boundary = sketch.outline().unwrap();
    let mut raft = Foundation::new(FoundationShape::Slab { boundary }, -0.1);
    raft.sketch = Some(sketch);
    let type_id = p
        .types_of(Category::Foundation)
        .find(|t| matches!(t.data, TypeData::Foundation { kind: cc_core::FoundationKind::Slab, .. }))
        .map(|t| t.id)
        .unwrap();
    add(p, type_id, ElementKind::Foundation(raft))
}

fn round_trip(p: &Project) -> (String, Project) {
    let text = export(p, &ExportOptions { time_stamp: Some("2026-01-01T00:00:00".into()), file_name: "t.ifc".into() });
    let (back, _) = import(&text, Project::from_template()).unwrap();
    (text, back)
}

fn entities(text: &str, entity: &str) -> usize {
    text.lines().filter(|l| l.contains(&format!("={entity}("))).count()
}

fn same<'a>(p: &'a Project, original: &Project, id: ElementId) -> &'a Element {
    let uid = original.element(id).unwrap().uid;
    p.elements.values().find(|e| e.uid == uid).expect("the element comes back")
}

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() <= 1e-6, "{what}: {a} vs {b}");
}

/// The curves of each kind a sketch has: lines, arcs, circles.
fn kinds(sketch: &Sketch) -> (usize, usize, usize) {
    let count = |f: fn(&SketchCurve) -> bool| sketch.curves.iter().filter(|c| f(c)).count();
    (
        count(|c| matches!(c, SketchCurve::Line { .. })),
        count(|c| matches!(c, SketchCurve::Arc { .. })),
        count(|c| matches!(c, SketchCurve::Circle { .. })),
    )
}

#[test]
fn round_floor_and_raft_come_back_as_circles() {
    let mut p = Project::from_template();
    let circle =
        |x: f64, radius: f64| Sketch { curves: vec![SketchCurve::Circle { center: DVec2::new(x, 1.0), radius }] };
    let slab = floor(&mut p, circle(0.0, 2.0));
    let base = raft(&mut p, circle(10.0, 3.2));
    let (text, back) = round_trip(&p);
    assert_eq!(entities(&text, "IFCCIRCLEPROFILEDEF"), 2, "both profiles are circles");
    let (before, after) = (ModelGeometry::build(&p), ModelGeometry::build(&back));
    for (id, center, radius) in [(slab, DVec2::new(0.0, 1.0), 2.0), (base, DVec2::new(10.0, 1.0), 3.2)] {
        let e = same(&back, &p, id);
        let sketch = e.drawn_sketch().expect("the circle is kept as the sketch");
        let [SketchCurve::Circle { center: c, radius: r }] = sketch.curves.as_slice() else {
            panic!("one circle, not {:?}", sketch.curves);
        };
        assert!(c.distance(center) < 1e-9, "center {c}");
        close(*r, radius, "radius");
        close(after.volume(e.id), before.volume(id), "volume");
    }
    let ElementKind::Floor(f) = &same(&back, &p, slab).kind else { panic!("a floor") };
    close(f.area(), PI * 4.0, "floor area");
}

/// A square whose top side is an arc, with a round hole.
fn arched() -> Sketch {
    let pt = DVec2::new;
    Sketch {
        curves: vec![
            SketchCurve::Line { a: pt(0.0, 0.0), b: pt(4.0, 0.0) },
            SketchCurve::Line { a: pt(4.0, 0.0), b: pt(4.0, 4.0) },
            SketchCurve::Arc { a: pt(4.0, 4.0), b: pt(0.0, 4.0), bulge: 0.4 },
            SketchCurve::Line { a: pt(0.0, 4.0), b: pt(0.0, 0.0) },
            SketchCurve::Circle { center: pt(2.0, 1.5), radius: 0.5 },
        ],
    }
}

#[test]
fn outline_of_lines_and_an_arc_with_a_round_hole_comes_back() {
    let mut p = Project::from_template();
    let slab = floor(&mut p, arched());
    let (text, back) = round_trip(&p);
    assert_eq!(entities(&text, "IFCARBITRARYPROFILEDEFWITHVOIDS"), 1);
    assert_eq!(entities(&text, "IFCCOMPOSITECURVE"), 2, "the outline and the hole");
    // The arc, and the hole's two halves.
    assert_eq!(entities(&text, "IFCTRIMMEDCURVE"), 3);
    let e = same(&back, &p, slab);
    let sketch = e.drawn_sketch().expect("the curves are kept");
    assert_eq!(kinds(sketch), (3, 1, 1), "{:?}", sketch.curves);
    let (ElementKind::Floor(a), ElementKind::Floor(b)) = (&p.element(slab).unwrap().kind, &e.kind) else {
        panic!("a floor");
    };
    assert_eq!(b.openings.len(), 1);
    close(b.area(), a.area(), "area");
    let (before, after) = (ModelGeometry::build(&p), ModelGeometry::build(&back));
    close(after.volume(e.id), before.volume(slab), "volume");
}

/// The first line writing `entity`, changed by `f`.
fn with_line(text: &str, entity: &str, f: impl Fn(&str) -> String) -> String {
    let key = format!("={entity}(");
    let mut done = false;
    text.lines()
        .map(|l| {
            if !done && l.contains(&key) {
                done = true;
                f(l)
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Broken curves in the written profile: the import still ends, and any
/// sketch it keeps draws the floor's outline.
#[test]
fn broken_curves_fall_back_without_panicking() {
    let mut p = Project::from_template();
    floor(&mut p, arched());
    let (text, _) = round_trip(&p);
    let radius = |r: &'static str| {
        move |l: &str| {
            let (head, _) = l.rsplit_once(',').unwrap();
            format!("{head},{r});")
        }
    };
    let variants = [
        with_line(&text, "IFCCIRCLE", radius("0.")),
        with_line(&text, "IFCCIRCLE", radius("-2.")),
        with_line(&text, "IFCCIRCLE", radius("1.E308")),
        with_line(&text, "IFCCIRCLE", radius("$")),
        // A trim that is no point.
        with_line(&text, "IFCTRIMMEDCURVE", |l| l.replacen("(#", "(IFCPARAMETERVALUE(1.E400),#", 1)),
        // An open outline: its first segment left out.
        with_line(&text, "IFCCOMPOSITECURVE", |l| {
            let (head, rest) = l.split_once("((").unwrap();
            let rest = rest.split_once(',').map_or(rest, |(_, r)| r);
            format!("{head}(({rest}")
        }),
        // A segment of a curve that does not exist.
        with_line(&text, "IFCCOMPOSITECURVESEGMENT", |l| l.replace(",.T.,#", ",.T.,#999999")),
    ];
    for text in variants {
        let (back, _) = import(&text, Project::from_template()).unwrap();
        for e in back.elements.values() {
            if let (Some(sketch), Some((outline, openings))) = (e.drawn_sketch(), e.sketched_profile()) {
                assert!(sketch.is_valid() && sketch.makes(outline, openings), "a sketch that draws the outline");
            }
        }
    }
}

/// A half disk of radius 1 as another program may write it: a line along
/// the diameter and the upper half circle trimmed as `trim`, its segment in
/// the composite curve running the curve's way when `same`.
fn half_disk(trim: &str, same: bool) -> String {
    let same = if same { ".T." } else { ".F." };
    format!(
        "ISO-10303-21;
HEADER;
FILE_SCHEMA(('IFC4'));
ENDSEC;
DATA;
#1=IFCPROJECT('0YvctVUKr0kugbFTf53O9L',$,'Arcs',$,$,$,$,(#2),#3);
#2=IFCGEOMETRICREPRESENTATIONCONTEXT($,'Model',3,1.E-05,#7,$);
#3=IFCUNITASSIGNMENT((#4));
#4=IFCSIUNIT(*,.LENGTHUNIT.,$,.METRE.);
#5=IFCBUILDINGSTOREY('1XvctVUKr0kugbFTf53O9L',$,'Ground',$,$,#6,$,$,.ELEMENT.,0.);
#6=IFCLOCALPLACEMENT($,#7);
#7=IFCAXIS2PLACEMENT3D(#8,$,$);
#8=IFCCARTESIANPOINT((0.,0.,0.));
#9=IFCDIRECTION((0.,0.,1.));
#10=IFCAXIS2PLACEMENT2D(#11,$);
#11=IFCCARTESIANPOINT((0.,0.));
#12=IFCCARTESIANPOINT((1.,0.));
#13=IFCCARTESIANPOINT((-1.,0.));
#14=IFCCIRCLE(#10,1.);
#15={trim};
#16=IFCPOLYLINE((#13,#12));
#17=IFCCOMPOSITECURVE((#18,#19),.F.);
#18=IFCCOMPOSITECURVESEGMENT(.CONTINUOUS.,.T.,#16);
#19=IFCCOMPOSITECURVESEGMENT(.CONTINUOUS.,{same},#15);
#20=IFCARBITRARYCLOSEDPROFILEDEF(.AREA.,$,#17);
#21=IFCEXTRUDEDAREASOLID(#20,#7,#9,0.2);
#22=IFCSHAPEREPRESENTATION(#2,'Body','SweptSolid',(#21));
#23=IFCPRODUCTDEFINITIONSHAPE($,$,(#22));
#24=IFCSLAB('2AvctVUKr0kugbFTf53O9L',$,'Half disk',$,$,#6,#23,$,.FLOOR.);
#25=IFCRELCONTAINEDINSPATIALSTRUCTURE('3XvctVUKr0kugbFTf53O9L',$,$,$,(#24),#5);
ENDSEC;
END-ISO-10303-21;
"
    )
}

#[test]
fn trimmed_circles_read_as_one_arc_by_parameter_or_point_in_either_sense() {
    let variants = [
        // From 0 to π counter-clockwise, by parameter.
        half_disk(
            "IFCTRIMMEDCURVE(#14,(IFCPARAMETERVALUE(0.)),(IFCPARAMETERVALUE(3.141592653589793)),.T.,.PARAMETER.)",
            true,
        ),
        // From π to 0 clockwise, run the other way by its segment.
        half_disk(
            "IFCTRIMMEDCURVE(#14,(IFCPARAMETERVALUE(3.141592653589793)),(IFCPARAMETERVALUE(0.)),.F.,.PARAMETER.)",
            false,
        ),
        // By its end points only.
        half_disk("IFCTRIMMEDCURVE(#14,(#12),(#13),.T.,.CARTESIAN.)", true),
        half_disk("IFCTRIMMEDCURVE(#14,(#13),(#12),.F.,.CARTESIAN.)", false),
    ];
    for text in variants {
        let (p, _) = import(&text, Project::from_template()).unwrap();
        let e = p.elements.values().find(|e| matches!(e.kind, ElementKind::Floor(_))).expect("a floor");
        let sketch = e.drawn_sketch().expect("the arc is kept");
        assert_eq!(kinds(sketch), (1, 1, 0), "{:?}", sketch.curves);
        let ElementKind::Floor(f) = &e.kind else { panic!("a floor") };
        close(f.area(), PI / 2.0, "half disk area");
    }
}
