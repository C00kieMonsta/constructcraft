//! Reinforcement sets laid out in a host come back as sets of that host,
//! laid out again, so their bars have the lengths they were exported with
//! (closed stirrups too) and go with the host (#368). When the host does not
//! come back, or the layout cannot be read, the bars come in as shaped bars,
//! stirrups closed again to their full length.

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_core::{
    BarFace, Beam, Category, Document, Element, ElementId, ElementKind, Project, Rebar, RebarLayout, TypeData, Uid,
};
use cc_geom::ModelGeometry;
use cc_ifc::{ExportOptions, export, import};
use glam::{DVec2, DVec3};

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
        plugin_data: Default::default(),
    };
    p.elements.insert(id, e);
    id
}

fn bar_type(p: &Project, diameter: f64) -> cc_core::ElementTypeId {
    p.types_of(Category::Rebar)
        .find(|t| matches!(t.data, TypeData::Rebar { diameter: d, .. } if (d - diameter).abs() < 1e-9))
        .map(|t| t.id)
        .unwrap()
}

/// A 5 m beam 15x40 with stirrups of 5 mm every 15 cm and three bottom bars of 10 mm.
fn beam_with_bars() -> (Project, ElementId, ElementId, ElementId) {
    let mut p = Project::from_template();
    let beam_t = p.default_type(Category::Beam).unwrap();
    assert!(matches!(p.element_type(beam_t).unwrap().data, TypeData::Beam { width: 0.15, height: 0.4, .. }));
    let beam = add(&mut p, beam_t, ElementKind::Beam(Beam::new(DVec2::ZERO, DVec2::new(5.0, 0.0), 3.0)));
    let stirrups = Rebar::new(beam, RebarLayout::Stirrups { spacing: 0.15, hooks: None }, 0.025);
    let (thin, thick) = (bar_type(&p, 0.005), bar_type(&p, 0.010));
    let stirrups = add(&mut p, thin, ElementKind::Rebar(stirrups));
    let bottom = Rebar::new(beam, RebarLayout::Longitudinal { face: BarFace::Bottom, count: 3 }, 0.025);
    let bottom = add(&mut p, thick, ElementKind::Rebar(bottom));
    (p, beam, stirrups, bottom)
}

fn text_of(p: &Project) -> String {
    export(p, &ExportOptions { time_stamp: Some("2026-01-01T00:00:00".into()), file_name: "t.ifc".into() })
}

fn same<'a>(p: &'a Project, original: &Project, id: ElementId) -> &'a Element {
    let uid = original.element(id).unwrap().uid;
    p.elements.values().find(|e| e.uid == uid).expect("the element comes back")
}

/// Each bar's length.
fn lengths(g: &ModelGeometry, id: ElementId) -> Vec<f64> {
    g.rebar[&id].bars.iter().map(|b| b.length()).collect()
}

fn polyline(bar: &[DVec3]) -> f64 {
    bar.windows(2).map(|w| w[0].distance(w[1])).sum()
}

#[test]
fn stirrups_come_back_in_their_beam_with_their_length() {
    let (p, beam, stirrups, bottom) = beam_with_bars();
    let before = ModelGeometry::build(&p);
    let stirrup = lengths(&before, stirrups);
    assert_eq!(stirrup.len(), 34);
    assert!((stirrup[0] - 0.88).abs() < 1e-9, "{}", stirrup[0]);
    let (back, _) = import(&text_of(&p), Project::from_template()).unwrap();
    let after = ModelGeometry::build(&back);
    let host = same(&back, &p, beam).id;
    for set in [stirrups, bottom] {
        let e = same(&back, &p, set);
        let ElementKind::Rebar(r) = &e.kind else { panic!("a set laid out in its host, not {:?}", e.kind) };
        assert_eq!(r.host, host);
        let ElementKind::Rebar(was) = &p.element(set).unwrap().kind else { panic!("a set") };
        assert_eq!(r.layout, was.layout);
        let (a, b) = (lengths(&before, set), lengths(&after, e.id));
        assert_eq!(a.len(), b.len());
        for (a, b) in a.iter().zip(&b) {
            assert!((a - b).abs() < 1e-9, "bar length {a} vs {b}");
        }
    }
    // The sets go with their beam.
    let id = same(&back, &p, stirrups).id;
    let mut doc = Document::new(back);
    doc.edit("delete the beam", |e| e.remove_element(host));
    assert!(doc.project.element(id).is_none());
    assert!(!doc.project.elements.values().any(|e| matches!(e.kind, ElementKind::Rebar(_))));
}

/// The line of the property `name`, its value changed by `f`.
fn with_property(text: &str, name: &str, f: impl Fn(&str) -> String) -> String {
    let key = format!("IFCPROPERTYSINGLEVALUE('{name}'");
    let changed: Vec<String> = text.lines().map(|l| if l.contains(&key) { f(l) } else { l.to_string() }).collect();
    assert_ne!(changed.join("\n"), text, "the file has {name}");
    changed.join("\n")
}

/// Without their host, or with a layout that cannot be used, the bars come
/// in as shaped bars, the stirrups closed to their whole length.
#[test]
fn bars_without_their_host_keep_their_length() {
    let (p, _, stirrups, _) = beam_with_bars();
    let text = text_of(&p);
    let stirrup = lengths(&ModelGeometry::build(&p), stirrups)[0];
    let variants = [
        with_property(&text, "Host", |l| {
            let (head, _) = l.split_once("IFCLABEL(").unwrap();
            format!("{head}IFCLABEL('0000000000000000000000'),$);")
        }),
        with_property(&text, "Layout", |l| l.replace("Stirrups(spacing:0.15)", "Stirrups(spacing:-1.0)")),
        with_property(&text, "Layout", |l| {
            let (head, _) = l.split_once("IFCTEXT(").unwrap();
            format!("{head}IFCTEXT('((('),$);")
        }),
    ];
    for text in variants {
        let (back, _) = import(&text, Project::from_template()).unwrap();
        let e = same(&back, &p, stirrups);
        let ElementKind::ShapedRebar(s) = &e.kind else { panic!("shaped bars, not {:?}", e.kind) };
        assert_eq!(s.bars.len(), 34);
        for bar in &s.bars {
            assert!((polyline(bar) - stirrup).abs() < 1e-9, "stirrup length {} vs {stirrup}", polyline(bar));
        }
    }
}
