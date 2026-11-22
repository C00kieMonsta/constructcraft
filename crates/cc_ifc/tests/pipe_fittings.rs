//! Pipes of a series export their series and size on the segment types,
//! and the fittings matched to the series' catalog export as typed
//! `IfcPipeFitting`s, one type per catalog row (composites one fitting
//! per piece).

use cc_core::{
    Category, Document, EditCtx, Element, ElementId, ElementKind, ElementType, ElementTypeId, LevelId, MepSegment,
    Project, TypeData,
};
use cc_ifc::{ExportOptions, export};
use glam::DVec2;

fn add(e: &mut EditCtx, level: LevelId, type_id: ElementTypeId, a: [f64; 2], b: [f64; 2]) -> ElementId {
    let id = ElementId(e.alloc_id());
    let mut run = MepSegment::new(DVec2::from_array(a), DVec2::from_array(b), 0.5);
    run.systems = e.project().systems.keys().next().copied().into_iter().collect();
    e.add_element(Element {
        id,
        level,
        type_id,
        mark: String::new(),
        comments: String::new(),
        kind: ElementKind::Pipe(run),
        uid: Default::default(),
        phase_created: Default::default(),
        phase_demolished: None,
        design_option: None,
        paints: Vec::new(),
        plugin_data: Default::default(),
    })
}

/// A type of the project's copy of built-in series `code`, size `size`.
fn series_type(e: &mut EditCtx, code: &str, size: &str) -> ElementTypeId {
    let sid = match e.project().pipe_series_by_code(code) {
        Some(s) => s.id,
        None => {
            let mut s = cc_core::pipe_catalog::series(code).unwrap().clone();
            s.id = cc_core::PipeSeriesId(e.alloc_id());
            s.uid = cc_core::Uid::generate();
            let id = s.id;
            e.pipe_series_mut().insert(id, s);
            id
        }
    };
    let de = e.project().pipe_series[&sid].row(size).unwrap().de;
    let id = ElementTypeId(e.alloc_id());
    e.add_type(ElementType {
        id,
        name: format!("{code} {size}"),
        data: TypeData::Pipe {
            diameter: de,
            material: String::new(),
            series: Some(sid),
            size: size.into(),
            inside: None,
        },
        info: Default::default(),
        uid: cc_core::Uid::generate(),
    })
}

/// A 25 mm L (two 90° elbows), a 40 mm run with a 20 mm branch (a tee and
/// reducers: no 40 × 20 tee is sold) and a 60° turn (no such elbow), of
/// BR-PVC-SOLD.
pub fn pipes_project() -> Project {
    let mut doc = Document::new(Project::from_template());
    let level = doc.project.levels[0].id;
    doc.edit("pipes", |e| {
        let d25 = series_type(e, "BR-PVC-SOLD", "25 mm");
        let d40 = series_type(e, "BR-PVC-SOLD", "40 mm");
        let d20 = series_type(e, "BR-PVC-SOLD", "20 mm");
        add(e, level, d25, [0.0, 0.0], [2.0, 0.0]);
        add(e, level, d25, [2.0, 0.0], [2.0, 2.0]);
        add(e, level, d25, [2.0, 2.0], [4.0, 2.0]);
        add(e, level, d40, [0.0, 5.0], [4.0, 5.0]);
        add(e, level, d20, [2.0, 5.0], [2.0, 7.0]);
        add(e, level, d25, [0.0, 10.0], [2.0, 10.0]);
        let a = 60f64.to_radians();
        add(e, level, d25, [2.0, 10.0], [2.0 + a.cos(), 10.0 + a.sin()]);
    });
    doc.project
}

fn lines<'a>(text: &'a str, entity: &str) -> Vec<&'a str> {
    text.lines().filter(|l| l.contains(&format!("={entity}("))).collect()
}

#[test]
fn pipe_segment_types_carry_their_series() {
    let text = export(&pipes_project(), &ExportOptions::default());
    assert!(text.contains("'ConstructCAD_PipeSeries'"));
    let props = lines(&text, "IFCPROPERTYSINGLEVALUE");
    assert!(props.iter().any(|l| l.contains("'Code'") && l.contains("'BR-PVC-SOLD'")));
    assert!(props.iter().any(|l| l.contains("'Size'") && l.contains("'25 mm'")));
    assert!(props.iter().any(|l| l.contains("'InsideDiameter'") && l.contains("0.0216")), "{props:?}");
}

#[test]
fn catalog_fittings_are_typed_by_their_rows() {
    let project = pipes_project();
    let text = export(&project, &ExportOptions::default());
    let fittings = lines(&text, "IFCPIPEFITTING");
    // Two elbows, a tee and its reducers, and the missing 60° elbow.
    let elbows: Vec<_> = fittings.iter().filter(|l| l.contains("'Elbow 90") && l.contains("vel 25 mm'")).collect();
    assert_eq!(elbows.len(), 2, "{fittings:?}");
    assert!(elbows.iter().all(|l| l.contains(".BEND.")));
    assert!(fittings.iter().any(|l| l.contains("'Tee PVC sold") && l.contains(".JUNCTION.")), "{fittings:?}");
    assert!(fittings.iter().any(|l| l.contains("'Reducer PVC sold") && l.contains(".TRANSITION.")));
    assert!(fittings.iter().any(|l| l.contains("'Elbow'") && l.contains(".BEND.")), "the missing one stays generic");
    // One type per row: the elbows share theirs.
    let types = lines(&text, "IFCPIPEFITTINGTYPE");
    assert_eq!(types.iter().filter(|l| l.contains("'Elbow 90") && l.contains("vel 25 mm'")).count(), 1, "{types:?}");
    let geom = cc_geom::ModelGeometry::build(&project);
    assert_eq!(geom.mep.fittings.iter().filter(|f| f.category == Category::Pipe).count(), 4);
}

/// Writes `fixtures/pipes.ccad` from [`pipes_project`]
/// (`cargo nextest run -p cc_ifc --test main write_pipes_fixture --run-ignored only`).
#[test]
#[ignore]
fn write_pipes_fixture() {
    let text = cc_core::to_ron(&pipes_project()).unwrap();
    std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/pipes.ccad"), text).unwrap();
}

#[test]
fn the_pipes_fixture_is_current() {
    let fixture = cc_core::from_ron(include_str!("fixtures/pipes.ccad")).unwrap();
    let text = export(&fixture, &ExportOptions::default());
    assert_eq!(
        lines(&text, "IFCPIPEFITTINGTYPE").len(),
        lines(&export(&pipes_project(), &ExportOptions::default()), "IFCPIPEFITTINGTYPE").len()
    );
}
