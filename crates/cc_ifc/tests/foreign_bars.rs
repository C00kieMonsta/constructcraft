//! A project and its bars as another program writes them (#369): the
//! project's `Name` is its number and `LongName` its name; bars take the
//! project's bar type of their nominal size (and grade) instead of one type
//! per source type name; a purely numeric tag (the other program's element
//! id) is not a mark.

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_core::{Category, Element, ElementKind, Project, TypeData};
use cc_ifc::{ExportOptions, export, import};

/// Three Ø12.5 CA-50 bars (two of two source types with numeric tags, one
/// with a mark of its own) and a Ø20 CA-50 bar, a size the template lacks.
/// Each bar is a straight line starting at x = 1, 2, 3 and 4 m.
const TANK: &str = "ISO-10303-21;
HEADER;
FILE_SCHEMA(('IFC4'));
ENDSEC;
DATA;
#1=IFCPROJECT('0YvctVUKr0kugbFTf53O9L',$,'00/0000',$,$,'GENERIC TEST TANK',$,(#2),#3);
#2=IFCGEOMETRICREPRESENTATIONCONTEXT($,'Model',3,1.E-05,#12,$);
#3=IFCUNITASSIGNMENT((#4));
#4=IFCSIUNIT(*,.LENGTHUNIT.,.MILLI.,.METRE.);
#10=IFCBUILDINGSTOREY('1XvctVUKr0kugbFTf53O9L',$,'Level 1',$,$,#11,$,$,.ELEMENT.,0.);
#11=IFCLOCALPLACEMENT($,#12);
#12=IFCAXIS2PLACEMENT3D(#13,$,$);
#13=IFCCARTESIANPOINT((0.,0.,0.));
#20=IFCRELCONTAINEDINSPATIALSTRUCTURE('3XvctVUKr0kugbFTf53O9L',$,$,$,(#100,#110,#120,#130),#10);
#100=IFCREINFORCINGBAR('2AvctVUKr0kugbFTf53O9L',$,'Rebar Bar:12.5 CA-50',$,$,#11,#101,'348213','CA-50',12.5,122.7,1000.,.MAIN.,.TEXTURED.);
#101=IFCPRODUCTDEFINITIONSHAPE($,$,(#102));
#102=IFCSHAPEREPRESENTATION(#2,'Body','AdvancedSweptSolid',(#103));
#103=IFCSWEPTDISKSOLID(#104,6.25,$,$,$);
#104=IFCPOLYLINE((#105,#106));
#105=IFCCARTESIANPOINT((1000.,0.,0.));
#106=IFCCARTESIANPOINT((1000.,1000.,0.));
#110=IFCREINFORCINGBAR('2BvctVUKr0kugbFTf53O9L',$,'Barra do vergalhao:12.5 CA-50',$,$,#11,#111,'348214','CA-50',12.5,122.7,1000.,.MAIN.,.TEXTURED.);
#111=IFCPRODUCTDEFINITIONSHAPE($,$,(#112));
#112=IFCSHAPEREPRESENTATION(#2,'Body','AdvancedSweptSolid',(#113));
#113=IFCSWEPTDISKSOLID(#114,6.25,$,$,$);
#114=IFCPOLYLINE((#115,#116));
#115=IFCCARTESIANPOINT((2000.,0.,0.));
#116=IFCCARTESIANPOINT((2000.,1000.,0.));
#120=IFCREINFORCINGBAR('2CvctVUKr0kugbFTf53O9L',$,'Bar',$,$,#11,#121,'N3','CA-50',12.5,122.7,1000.,.MAIN.,.TEXTURED.);
#121=IFCPRODUCTDEFINITIONSHAPE($,$,(#122));
#122=IFCSHAPEREPRESENTATION(#2,'Body','AdvancedSweptSolid',(#123));
#123=IFCSWEPTDISKSOLID(#124,6.25,$,$,$);
#124=IFCPOLYLINE((#125,#126));
#125=IFCCARTESIANPOINT((3000.,0.,0.));
#126=IFCCARTESIANPOINT((3000.,1000.,0.));
#130=IFCREINFORCINGBAR('2DvctVUKr0kugbFTf53O9L',$,'Rebar Bar:20 CA-50',$,$,#11,#131,'9001','CA-50',20.,314.2,1000.,.MAIN.,.TEXTURED.);
#131=IFCPRODUCTDEFINITIONSHAPE($,$,(#132));
#132=IFCSHAPEREPRESENTATION(#2,'Body','AdvancedSweptSolid',(#133));
#133=IFCSWEPTDISKSOLID(#134,10.,$,$,$);
#134=IFCPOLYLINE((#135,#136));
#135=IFCCARTESIANPOINT((4000.,0.,0.));
#136=IFCCARTESIANPOINT((4000.,1000.,0.));
#200=IFCREINFORCINGBARTYPE('2EvctVUKr0kugbFTf53O9L',$,'Rebar Bar:12.5 CA-50',$,$,$,$,$,$,.MAIN.,12.5,122.7,$,.TEXTURED.,$,$);
#201=IFCRELDEFINESBYTYPE('2FvctVUKr0kugbFTf53O9L',$,$,$,(#100),#200);
#210=IFCREINFORCINGBARTYPE('2GvctVUKr0kugbFTf53O9L',$,'Barra do vergalhao:12.5 CA-50',$,$,$,$,$,$,.MAIN.,12.5,122.7,$,.TEXTURED.,$,$);
#211=IFCRELDEFINESBYTYPE('2HvctVUKr0kugbFTf53O9L',$,$,$,(#110),#210);
#220=IFCREINFORCINGBARTYPE('2IvctVUKr0kugbFTf53O9L',$,'Rebar Bar:20 CA-50',$,$,$,$,$,$,.MAIN.,20.,314.2,$,.TEXTURED.,$,$);
#221=IFCRELDEFINESBYTYPE('2JvctVUKr0kugbFTf53O9L',$,$,$,(#130),#220);
ENDSEC;
END-ISO-10303-21;
";

/// The imported bar whose first point is at `x` m.
fn bar_at(p: &Project, x: f64) -> &Element {
    p.elements
        .values()
        .find(|e| match &e.kind {
            ElementKind::ShapedRebar(r) => {
                r.bars.first().and_then(|b| b.first()).is_some_and(|q| (q.x - x).abs() < 1e-6)
            }
            _ => false,
        })
        .unwrap_or_else(|| panic!("no bar at {x}"))
}

fn rebar_data(p: &Project, e: &Element) -> (String, f64, String) {
    let ty = p.element_type(e.type_id).unwrap();
    match &ty.data {
        TypeData::Rebar { diameter, grade, .. } => (ty.name.clone(), *diameter, grade.clone()),
        other => panic!("{other:?}"),
    }
}

/// `Name` is the project number and `LongName` its name.
#[test]
fn project_name_is_the_long_name() {
    let (p, _) = import(TANK, Project::from_template()).unwrap();
    assert_eq!(p.info.name, "GENERIC TEST TANK");
    assert_eq!(p.info.number, "00/0000");
    // Only a `Name`: the project's name, as before.
    let only_name = TANK.replace("'00/0000',$,$,'GENERIC TEST TANK'", "'Tank',$,$,$");
    let (p, _) = import(&only_name, Project::from_template()).unwrap();
    assert_eq!((p.info.name.as_str(), p.info.number.as_str()), ("Tank", ""));
    // Only a `LongName`: the name too.
    let only_long = TANK.replace("'00/0000',$,$,'GENERIC TEST TANK'", "$,$,$,'Tank'");
    let (p, _) = import(&only_long, Project::from_template()).unwrap();
    assert_eq!((p.info.name.as_str(), p.info.number.as_str()), ("Tank", ""));
}

/// Export writes the number as `Name` and the name as `LongName`, so both
/// come back where they were; without a number the name is the `Name`.
#[test]
fn project_name_and_number_round_trip() {
    let mut p = Project::from_template();
    p.info.name = "Generic test tank".into();
    p.info.number = "00/0000".into();
    let text = export(&p, &ExportOptions::default());
    let line = text.lines().find(|l| l.contains("=IFCPROJECT(")).unwrap();
    assert!(line.contains(",$,'00/0000',$,$,'Generic test tank',"), "{line}");
    let (again, _) = import(&text, Project::from_template()).unwrap();
    assert_eq!((again.info.name.as_str(), again.info.number.as_str()), ("Generic test tank", "00/0000"));

    p.info.number.clear();
    let text = export(&p, &ExportOptions::default());
    let (again, _) = import(&text, Project::from_template()).unwrap();
    assert_eq!((again.info.name.as_str(), again.info.number.as_str()), ("Generic test tank", ""));
}

/// Bars of a size the template has take its type, whatever their source
/// type names; a size it lacks makes one type, with the file's grade.
#[test]
fn bars_take_the_project_type_of_their_size() {
    let base = Project::from_template();
    let before = base.types_of(Category::Rebar).count();
    let (p, report) = import(TANK, base).unwrap();
    assert_eq!(report.imported.get(&Category::Rebar), Some(&4), "{report:?}");
    for x in [1.0, 2.0, 3.0] {
        let (name, diameter, _) = rebar_data(&p, bar_at(&p, x));
        assert_eq!(name, "Barra Ø 12,5 mm", "bar at {x}");
        assert!((diameter - 0.0125).abs() < 1e-9);
    }
    let (_, diameter, grade) = rebar_data(&p, bar_at(&p, 4.0));
    assert!((diameter - 0.020).abs() < 1e-9);
    assert_eq!(grade, "CA-50");
    assert_eq!(p.types_of(Category::Rebar).count(), before + 1);
}

/// A Ø6.3 CA-50 bar, the usual stirrup size, takes the template's Ø 6,3 mm
/// type, which carries the grade (#437).
#[test]
fn stirrup_bars_take_the_template_type() {
    let tank = TANK
        .replace("'Rebar Bar:20 CA-50'", "'Barra do vergalhao:6.3 CA-50'")
        .replace("'9001','CA-50',20.,314.2", "'9001','CA-50',6.3,31.2")
        .replace("IFCSWEPTDISKSOLID(#134,10.,", "IFCSWEPTDISKSOLID(#134,3.15,")
        .replace(".MAIN.,20.,314.2,", ".MAIN.,6.3,31.2,");
    assert!(!tank.contains("20 CA-50"), "the Ø20 bar is now Ø6.3");
    let base = Project::from_template();
    let before = base.types_of(Category::Rebar).count();
    let (p, _) = import(&tank, base).unwrap();
    let (name, diameter, grade) = rebar_data(&p, bar_at(&p, 4.0));
    assert_eq!(name, "Barra Ø 6,3 mm");
    assert!((diameter - 0.0063).abs() < 1e-9);
    assert_eq!(grade, "CA-50");
    assert_eq!(p.types_of(Category::Rebar).count(), before);
}

/// A type of the size but of another grade does not take the bars.
#[test]
fn bars_keep_their_grade() {
    let mut base = Project::from_template();
    let id = base.types_of(Category::Rebar).find(|t| t.name == "Barra Ø 12,5 mm").unwrap().id;
    if let Some(TypeData::Rebar { grade, .. }) = base.types.get_mut(&id).map(|t| &mut t.data) {
        *grade = "CA-60".into();
    }
    let (p, _) = import(TANK, base).unwrap();
    let e = bar_at(&p, 1.0);
    assert_ne!(e.type_id, id);
    let (_, diameter, grade) = rebar_data(&p, e);
    assert!((diameter - 0.0125).abs() < 1e-9);
    assert_eq!(grade, "CA-50");
    // The three bars share the one new type.
    assert_eq!(bar_at(&p, 2.0).type_id, e.type_id);
    assert_eq!(bar_at(&p, 3.0).type_id, e.type_id);
}

/// A numeric tag is the other program's element id: no mark; another tag is.
#[test]
fn numeric_tags_are_not_marks() {
    let (p, _) = import(TANK, Project::from_template()).unwrap();
    assert_eq!(bar_at(&p, 1.0).mark, "");
    assert_eq!(bar_at(&p, 2.0).mark, "");
    assert_eq!(bar_at(&p, 3.0).mark, "N3");
    assert_eq!(bar_at(&p, 4.0).mark, "");
}
