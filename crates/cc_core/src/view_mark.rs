//! Marks of sections and elevations in plans: how a view mark type
//! (`TypeData::ViewMark { kind, style }`, category `ViewMark`) draws the
//! heads of a section or the marker of an elevation ([`MarkStyle`]), which
//! type a section or an elevation is marked with (`View::mark_type`;
//! missing: the project's first of its kind, else the kind's
//! [default style](MarkKind::default_style), what plans always drew), and
//! how far each floor plan moved an elevation's marker (`View::mark_offsets`).
//!
//! A mark's texts are templates whose `{Field}`s (see
//! [`crate::title_block::field_parts`]) are `Number` (the drawing's number on
//! its sheet, else its letter), `Sheet` (its sheet's number, else a dash),
//! `Name` (the view's name) and `Letter` (the last word of its name).

use crate::grid::BubbleEnds;
use crate::tag::TagSymbol;
use crate::{ElementType, ElementTypeId, TypeData, View, ViewId, ViewKind};
use glam::DVec2;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

/// Largest head, paper millimeters.
pub const MAX_HEAD: f64 = 100.0;
/// Most characters of a text's template.
pub const MAX_TEXT: usize = 256;
/// Smallest and largest text height, paper millimeters.
pub const MIN_TEXT_SIZE: f64 = 0.5;
pub const MAX_TEXT_SIZE: f64 = 50.0;
/// Farthest an elevation marker is kept from where it sits by repair, meters.
pub const MAX_OFFSET: f64 = 1e4;

/// What a view mark type marks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MarkKind {
    Section,
    Elevation,
}

impl MarkKind {
    pub const ALL: [MarkKind; 2] = [MarkKind::Section, MarkKind::Elevation];

    /// The kind of mark a view of `kind` has in plans, if any.
    pub fn of_view(kind: &ViewKind) -> Option<MarkKind> {
        match kind {
            ViewKind::Section { .. } => Some(MarkKind::Section),
            ViewKind::Elevation { .. } => Some(MarkKind::Elevation),
            _ => None,
        }
    }

    /// The name commands use: `section`, `elevation`.
    pub fn code(self) -> &'static str {
        match self {
            MarkKind::Section => "section",
            MarkKind::Elevation => "elevation",
        }
    }

    /// The kind named `code` (any case).
    pub fn from_code(code: &str) -> Option<MarkKind> {
        let code = code.trim();
        MarkKind::ALL.into_iter().find(|k| k.code().eq_ignore_ascii_case(code))
    }

    /// Template key of the name of the kind's default mark type.
    pub fn template_key(self) -> &'static str {
        match self {
            MarkKind::Section => "tpl-mark-section",
            MarkKind::Elevation => "tpl-mark-elevation",
        }
    }

    /// The style marks of this kind have when no type says otherwise: the
    /// marks plans have always drawn.
    pub fn default_style(self) -> MarkStyle {
        match self {
            MarkKind::Section => MarkStyle::default(),
            MarkKind::Elevation => MarkStyle {
                head: 0.0,
                split: false,
                top: MarkText::empty(SECTION_NUMBER_MM),
                bottom: MarkText::empty(SECTION_SHEET_MM),
                label: MarkText { text: "{Name}".into(), size: 2.5 },
                arrow: MarkArrow::Narrow,
                arrow_color: Some(ELEVATION_ARROW),
                ..MarkStyle::default()
            },
        }
    }
}

/// Paper height of a section's drawing number and sheet number, millimeters.
const SECTION_NUMBER_MM: f64 = 3.5;
const SECTION_SHEET_MM: f64 = 2.5;
/// The color of the arrow of elevation markers.
const ELEVATION_ARROW: [u8; 3] = [30, 90, 170];

/// A text of a mark: a template of text and `{Field}`s, `size` paper millimeters high.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkText {
    #[serde(default)]
    pub text: String,
    #[serde(default = "default_text_size")]
    pub size: f64,
}

fn default_text_size() -> f64 {
    2.5
}

impl MarkText {
    fn empty(size: f64) -> Self {
        Self { text: String::new(), size }
    }
}

/// The arrow of a mark, pointing where the view looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MarkArrow {
    /// A filled triangle whose base runs along the section line, twice the head wide.
    #[default]
    Filled,
    /// A narrow filled triangle from the head's middle (elevation markers).
    Narrow,
    /// The outline of the wide triangle.
    Open,
    None,
}

/// What a view mark type draws (see the module documentation).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MarkStyle {
    /// Diameter of the head, paper millimeters; 0 (elevation markers) draws
    /// it 0.5 m across in the model at any scale, as plans always did.
    pub head: f64,
    /// The head: a shape, or a detail component drawn at paper size (a
    /// component meter being 1000 paper mm) whose texts' fields are filled in.
    pub symbol: TagSymbol,
    /// A line across the head between the top and bottom texts.
    pub split: bool,
    /// The text in the head's upper half.
    pub top: MarkText,
    /// The text in the head's lower half.
    pub bottom: MarkText,
    /// A text beside the head, below and to its right.
    pub label: MarkText,
    pub arrow: MarkArrow,
    /// The ends of a section line with a head.
    pub heads: BubbleEnds,
    /// Pen weight of a section line, paper millimeters.
    pub line_weight: f64,
    /// Color of the head, its texts and the section line (`None`: the annotation ink).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<[u8; 3]>,
    /// Color of the arrow (`None`: the color).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arrow_color: Option<[u8; 3]>,
}

impl Default for MarkStyle {
    fn default() -> Self {
        Self {
            head: 12.0,
            symbol: TagSymbol::Circle,
            split: true,
            top: MarkText { text: "{Number}".into(), size: SECTION_NUMBER_MM },
            bottom: MarkText { text: "{Sheet}".into(), size: SECTION_SHEET_MM },
            label: MarkText::empty(2.5),
            arrow: MarkArrow::Filled,
            heads: BubbleEnds::Both,
            line_weight: 0.7,
            color: None,
            arrow_color: None,
        }
    }
}

impl MarkStyle {
    /// Bring every value within the module's limits; returns how many were
    /// changed. A component symbol is checked against the project by
    /// [`crate::tag::repair_tag_types`].
    pub fn repair(&mut self) -> usize {
        let mut n = 0;
        if !(self.head.is_finite() && (0.0..=MAX_HEAD).contains(&self.head)) {
            self.head = 12.0;
            n += 1;
        }
        for t in [&mut self.top, &mut self.bottom, &mut self.label] {
            if t.text.chars().count() > MAX_TEXT {
                t.text = t.text.chars().take(MAX_TEXT).collect();
                n += 1;
            }
            if !(t.size.is_finite() && (MIN_TEXT_SIZE..=MAX_TEXT_SIZE).contains(&t.size)) {
                t.size = default_text_size();
                n += 1;
            }
        }
        if !(self.line_weight.is_finite() && (crate::tag::MIN_PEN..=crate::tag::MAX_PEN).contains(&self.line_weight)) {
            self.line_weight = 0.7;
            n += 1;
        }
        n
    }
}

/// Whether `ty` is a view mark type of `kind`.
pub fn is_mark_type_of(ty: Option<&ElementType>, kind: MarkKind) -> bool {
    matches!(ty.map(|t| &t.data), Some(TypeData::ViewMark { kind: k, .. }) if *k == kind)
}

/// Repair a view's mark type and the elevation markers it moved, against
/// the project's `types` and its `elevations` (the ids of its elevation
/// views): a mark type on a view with no mark, or not of its kind, goes;
/// offsets kept by a view that is no floor plan, of views that are no
/// elevations, out of reach or not finite go. Returns how many were fixed.
pub fn repair_view_marks(
    view: &mut View,
    elevations: &BTreeSet<ViewId>,
    types: &BTreeMap<ElementTypeId, ElementType>,
) -> usize {
    let mut n = 0;
    if let Some(t) = view.mark_type
        && MarkKind::of_view(&view.kind).is_none_or(|kind| !is_mark_type_of(types.get(&t), kind))
    {
        view.mark_type = None;
        n += 1;
    }
    let plan = matches!(view.kind, ViewKind::FloorPlan { .. });
    let before = view.mark_offsets.len();
    view.mark_offsets.retain(|id, o| {
        plan && elevations.contains(id) && o.is_finite() && o.length() < MAX_OFFSET && *o != DVec2::ZERO
    });
    n + (before - view.mark_offsets.len())
}

impl crate::Project {
    /// The view mark types of `kind`, in id order.
    pub fn mark_types_of(&self, kind: MarkKind) -> impl Iterator<Item = &ElementType> {
        self.types.values().filter(move |t| matches!(&t.data, TypeData::ViewMark { kind: k, .. } if *k == kind))
    }

    /// The view mark type `view` (a section or an elevation) is marked
    /// with: its own, else the project's first of its kind (`None`: there is
    /// none, and the kind's default style applies).
    pub fn mark_type_for(&self, view: &View) -> Option<&ElementType> {
        let kind = MarkKind::of_view(&view.kind)?;
        view.mark_type
            .and_then(|id| self.element_type(id))
            .filter(|t| is_mark_type_of(Some(t), kind))
            .or_else(|| self.mark_types_of(kind).next())
    }

    /// The style `view`'s mark is drawn with (`None`: the view has no mark in plans).
    pub fn mark_style(&self, view: &View) -> Option<Cow<'_, MarkStyle>> {
        let kind = MarkKind::of_view(&view.kind)?;
        Some(match self.mark_type_for(view).map(|t| &t.data) {
            Some(TypeData::ViewMark { style, .. }) => Cow::Borrowed(style),
            _ => Cow::Owned(kind.default_style()),
        })
    }

    /// The ids of the project's elevation views.
    pub fn elevation_ids(&self) -> BTreeSet<ViewId> {
        self.views.values().filter(|v| matches!(v.kind, ViewKind::Elevation { .. })).map(|v| v.id).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn styles_are_repaired_within_their_limits() {
        let mut s = MarkStyle {
            head: -1.0,
            top: MarkText { text: "x".repeat(300), size: f64::NAN },
            line_weight: 0.0,
            ..MarkStyle::default()
        };
        assert_eq!(s.repair(), 4);
        assert_eq!((s.head, s.top.size, s.line_weight), (12.0, 2.5, 0.7));
        assert_eq!(s.top.text.chars().count(), MAX_TEXT);
        assert_eq!(s.repair(), 0);
        for kind in MarkKind::ALL {
            assert_eq!(kind.default_style().repair(), 0, "{kind:?}");
            assert_eq!(MarkKind::from_code(&kind.code().to_uppercase()), Some(kind));
        }
    }

    #[test]
    fn views_drop_bad_mark_types_and_offsets() {
        let mut p = crate::Project::from_template();
        let level = p.levels[0].id;
        let section = ElementTypeId(p.alloc_id());
        p.types.insert(
            section,
            ElementType {
                id: section,
                name: "S".into(),
                data: TypeData::ViewMark { kind: MarkKind::Section, style: MarkStyle::default() },
                info: Default::default(),
                uid: crate::Uid::generate(),
            },
        );
        let north = ViewId(p.alloc_id());
        let mut elevation = View::new(north, "N", 100, ViewKind::Elevation { direction: crate::Direction::North });
        elevation.mark_type = Some(section);
        assert_eq!(repair_view_marks(&mut elevation, &BTreeSet::new(), &p.types), 1, "a section type on an elevation");
        assert_eq!(elevation.mark_type, None);
        let mut plan = View::new(ViewId(p.alloc_id()), "P", 50, ViewKind::floor_plan(level));
        plan.mark_offsets.insert(north, DVec2::new(1.0, 2.0));
        plan.mark_offsets.insert(ViewId(98765), DVec2::new(1.0, 2.0));
        plan.mark_offsets.insert(plan.id, DVec2::new(f64::NAN, 0.0));
        let elevations: BTreeSet<ViewId> = [north].into();
        assert_eq!(repair_view_marks(&mut plan, &elevations, &p.types), 2);
        assert_eq!(plan.mark_offsets.len(), 1);
        assert_eq!(repair_view_marks(&mut plan, &elevations, &p.types), 0);
    }
}
