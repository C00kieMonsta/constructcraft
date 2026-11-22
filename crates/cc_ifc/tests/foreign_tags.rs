//! Tags as other programs write them (#436): many write their own element id
//! in `IfcElement.Tag`, a plain number that is no mark, whatever the class.
//! This program's own files keep numeric marks.

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_core::{Category, Element, Project, Uid};
use cc_ifc::{ExportOptions, export, import};
use std::collections::{HashMap, HashSet};

const HOUSE: &str = include_str!("../../../examples/casa_simples.ccad");
const STRUCTURE: &str = include_str!("fixtures/structure.ccad");

fn export_of(project: &Project) -> String {
    export(project, &ExportOptions { time_stamp: Some("2026-01-01T00:00:00".into()), file_name: "t.ifc".into() })
}

/// The project with every element marked by a number, except the first
/// wall, marked `W1`.
fn marked(source: &str) -> Project {
    let mut project: Project = cc_core::from_ron(source).unwrap();
    let first_wall = project.elements.values().find(|e| e.category() == Category::Wall).map(|e| e.id);
    for (i, e) in project.elements.values_mut().enumerate() {
        e.mark = if Some(e.id) == first_wall { "W1".into() } else { (776_113 + i).to_string() };
    }
    project
}

/// The file with `system` as the program that wrote it.
fn written_by(text: &str, system: &str) -> String {
    let line = |l: &str| if l.starts_with("FILE_NAME(") { l.replace("ConstructCraft", system) } else { l.to_string() };
    let out: Vec<String> = text.lines().map(line).collect();
    assert_ne!(out.join("\n"), text.trim_end(), "the header names this program");
    out.join("\n")
}

fn by_uid(p: &Project) -> HashMap<Uid, &Element> {
    p.elements.values().map(|e| (e.uid, e)).collect()
}

/// Every numeric tag of another program's file is left out of the marks, of
/// walls, slabs, columns, beams and the rest alike; `W1` is kept.
#[test]
fn numeric_tags_of_other_programs_are_not_marks() {
    let mut categories = HashSet::new();
    for source in [HOUSE, STRUCTURE] {
        let original = marked(source);
        let text = written_by(&export_of(&original), "Other Modeler");
        let (p, _) = import(&text, Project::from_template()).unwrap();
        let imported = by_uid(&p);
        // Bars still carry this program's rebar property set, which keeps their
        // tag (#369); other programs' bars are in `foreign_bars.rs`.
        for e in original.elements.values().filter(|e| e.category() != Category::Rebar) {
            let Some(i) = imported.get(&e.uid) else { continue };
            let expected = if e.mark == "W1" { "W1" } else { "" };
            assert_eq!(i.mark, expected, "{:?} {}", e.category(), e.id);
            categories.insert(i.category());
        }
    }
    for category in
        [Category::Wall, Category::Door, Category::Window, Category::Floor, Category::Column, Category::Beam]
    {
        assert!(categories.contains(&category), "no {category:?} checked: {categories:?}");
    }
}

/// A file this program wrote, under its current or former name, keeps its
/// numeric marks.
#[test]
fn numeric_marks_of_this_program_come_back() {
    for source in [HOUSE, STRUCTURE] {
        let original = marked(source);
        let text = export_of(&original);
        for text in [text.clone(), written_by(&text, "ConstructCAD")] {
            let (p, _) = import(&text, Project::from_template()).unwrap();
            let imported = by_uid(&p);
            for e in original.elements.values().filter(|e| e.category() != Category::Room) {
                let Some(i) = imported.get(&e.uid) else { continue };
                assert_eq!(i.mark, e.mark, "{:?} {}", e.category(), e.id);
            }
        }
    }
}

/// Empty address lines are left out of the project address, not joined as
/// separators (", -4").
#[test]
fn empty_address_lines_are_left_out() {
    let mut project = Project::from_template();
    project.info.address = "Rua das Flores 12".into();
    let text = export_of(&project).replace("('Rua das Flores 12')", "('','Rua das Flores 12','  ')");
    assert!(text.contains("('','Rua das Flores 12','  ')"), "the address is written as one line");
    let (p, _) = import(&written_by(&text, "Other Modeler"), Project::from_template()).unwrap();
    assert_eq!(p.info.address, "Rua das Flores 12");
}
