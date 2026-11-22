//! Tags of elements in plans: what a tag of a door, a window, a room, a
//! column, a beam or a footing writes and how it looks (its type, a
//! [`TagStyle`] in a `TypeData::Tag`), and how each plan adjusts the tag
//! of one element ([`TagPlacement`] in `View::tags`) or picks the type it
//! tags a target with (`View::tag_types`).
//!
//! Tags are not elements: every plan draws them for the elements it shows
//! (`cc_views::tag`). A tag's text lines are templates whose `{Field}`
//! placeholders (see [`crate::title_block::field_parts`]) name a schedule
//! field by its serde name (`{Width}`, `{Comments}`) or one of the tag
//! keys `Mark`, `TypeName`, `Name`, `Number`, `Area`, `Section` and `Level`.

use crate::{Element, ElementId, ElementKind, ElementType, ElementTypeId, LeaderSymbol, TypeData, View};
use glam::DVec2;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Most text lines a tag style has.
pub const MAX_LINES: usize = 8;
/// Most characters of a line's template.
pub const MAX_TEXT: usize = 256;
/// Smallest and largest text height, paper millimeters.
pub const MIN_TEXT_SIZE: f64 = 0.5;
pub const MAX_TEXT_SIZE: f64 = 50.0;
/// Largest symbol width or height and padding, paper millimeters.
pub const MAX_SYMBOL_SIZE: f64 = 200.0;
pub const MAX_PADDING: f64 = 50.0;
/// Thinnest and heaviest pen, paper millimeters.
pub const MIN_PEN: f64 = 0.01;
pub const MAX_PEN: f64 = 5.0;
/// Farthest a tag is kept from its default spot by repair, meters.
pub const MAX_OFFSET: f64 = 1e4;
/// Padding around the text of a symbol that fits it, paper millimeters.
pub const DEFAULT_PADDING: f64 = 1.0;

/// What a tag type tags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TagTarget {
    Door,
    Window,
    Room,
    Column,
    Beam,
    /// Pad footings and pile caps.
    Foundation,
}

impl TagTarget {
    pub const ALL: [TagTarget; 6] = [
        TagTarget::Door,
        TagTarget::Window,
        TagTarget::Room,
        TagTarget::Column,
        TagTarget::Beam,
        TagTarget::Foundation,
    ];

    /// The target of an element whose type is `ty`, if it is tagged.
    pub fn of_element(e: &Element, ty: Option<&TypeData>) -> Option<TagTarget> {
        Some(match &e.kind {
            ElementKind::Door(_) => TagTarget::Door,
            ElementKind::Window(_) => TagTarget::Window,
            ElementKind::Room(_) => TagTarget::Room,
            ElementKind::Column(_) => TagTarget::Column,
            ElementKind::Beam(_) => TagTarget::Beam,
            ElementKind::Foundation(_) => match ty {
                Some(TypeData::Foundation {
                    kind: crate::FoundationKind::Pad | crate::FoundationKind::PileCap,
                    ..
                }) => TagTarget::Foundation,
                _ => return None,
            },
            _ => return None,
        })
    }

    /// The target of `e` in `project`, if it is tagged.
    pub fn of(e: &Element, project: &crate::Project) -> Option<TagTarget> {
        Self::of_element(e, project.element_type(e.type_id).map(|t| &t.data))
    }

    /// The name commands use: `door`, `window`, `room`, `column`, `beam`, `foundation`.
    pub fn code(self) -> &'static str {
        match self {
            TagTarget::Door => "door",
            TagTarget::Window => "window",
            TagTarget::Room => "room",
            TagTarget::Column => "column",
            TagTarget::Beam => "beam",
            TagTarget::Foundation => "foundation",
        }
    }

    /// The target named `code` (any case).
    pub fn from_code(code: &str) -> Option<TagTarget> {
        let code = code.trim();
        TagTarget::ALL.into_iter().find(|t| t.code().eq_ignore_ascii_case(code))
    }

    /// Template key of the name of the target's default tag type.
    pub fn template_key(self) -> &'static str {
        match self {
            TagTarget::Door => "tpl-tag-door",
            TagTarget::Window => "tpl-tag-window",
            TagTarget::Room => "tpl-tag-room",
            TagTarget::Column => "tpl-tag-column",
            TagTarget::Beam => "tpl-tag-beam",
            TagTarget::Foundation => "tpl-tag-foundation",
        }
    }

    /// The style tags of this target have when no tag type says otherwise:
    /// the tags plans have always drawn.
    pub fn default_style(self) -> TagStyle {
        let line = |text: &str, size: f64| TagLine { text: text.into(), size };
        match self {
            TagTarget::Door | TagTarget::Window => TagStyle {
                lines: vec![line("{Mark}", 2.0)],
                symbol: if self == TagTarget::Door { TagSymbol::Circle } else { TagSymbol::Hexagon },
                width: 5.2,
                height: 5.2,
                ..TagStyle::default()
            },
            TagTarget::Room => TagStyle {
                lines: vec![line("{Name}", 2.5), line("{Number}", 2.2), line("{Area}", 2.0)],
                crossed_when_unplaced: true,
                ..TagStyle::default()
            },
            TagTarget::Column | TagTarget::Foundation => {
                TagStyle { lines: vec![line("{Mark} {Section}", 2.5)], ..TagStyle::default() }
            }
            TagTarget::Beam => TagStyle {
                lines: vec![line("{Mark} {Section}", 2.5)],
                orientation: TagOrientation::AlongHost,
                ..TagStyle::default()
            },
        }
    }
}

/// One line of a tag: a template of text and `{Field}`s, `size` paper millimeters high.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagLine {
    #[serde(default)]
    pub text: String,
    #[serde(default = "default_line_size")]
    pub size: f64,
}

fn default_line_size() -> f64 {
    2.5
}

/// The drawing around a tag's text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TagSymbol {
    /// The text alone.
    #[default]
    None,
    Circle,
    Hexagon,
    Rectangle,
    RoundedRectangle,
    Ellipse,
    Diamond,
    Triangle,
    /// A line under the text.
    Underline,
    /// A detail component's 2D pieces, drawn at paper size (a component
    /// meter is a paper millimeter times 1000), its texts' `{fields}` filled in.
    Component(ElementTypeId),
}

impl TagSymbol {
    /// Whether the symbol is as wide as it is high (a circle, a hexagon, a
    /// diamond or a triangle fitting text takes its larger side).
    pub fn is_regular(self) -> bool {
        matches!(self, TagSymbol::Circle | TagSymbol::Hexagon | TagSymbol::Diamond | TagSymbol::Triangle)
    }
}

/// When a tag gets a leader to its element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LeaderMode {
    /// When the tag was moved off its element.
    #[default]
    Auto,
    /// Whenever the element's point is outside the tag.
    Always,
    Never,
}

/// Which way a tag's text reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TagOrientation {
    #[default]
    Horizontal,
    /// Along the host (beams), kept readable left to right or bottom to top.
    AlongHost,
}

/// What a tag type draws (see the module documentation).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TagStyle {
    pub lines: Vec<TagLine>,
    pub symbol: TagSymbol,
    /// The symbol's size, paper millimeters; 0 fits the text plus `padding`.
    pub width: f64,
    pub height: f64,
    pub padding: f64,
    /// Pen weight of the symbol and leader, paper millimeters (`None`: the annotation pen).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pen: Option<f64>,
    /// Color of the symbol and leader.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<[u8; 3]>,
    /// Color of the text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_color: Option<[u8; 3]>,
    /// An opaque white box under the text.
    pub fill: bool,
    pub leader: LeaderMode,
    #[serde(default = "no_symbol")]
    pub leader_end: LeaderSymbol,
    pub orientation: TagOrientation,
    /// Crossed out when the element is not placed (a room not enclosed).
    pub crossed_when_unplaced: bool,
}

fn no_symbol() -> LeaderSymbol {
    LeaderSymbol::None
}

impl Default for TagStyle {
    fn default() -> Self {
        Self {
            lines: Vec::new(),
            symbol: TagSymbol::None,
            width: 0.0,
            height: 0.0,
            padding: DEFAULT_PADDING,
            pen: None,
            color: None,
            text_color: None,
            fill: false,
            leader: LeaderMode::Auto,
            leader_end: LeaderSymbol::None,
            orientation: TagOrientation::Horizontal,
            crossed_when_unplaced: false,
        }
    }
}

impl TagStyle {
    /// Bring every value within the module's limits; returns how many were
    /// changed. A component symbol is checked against the project by
    /// [`repair_tag_types`].
    pub fn repair(&mut self) -> usize {
        let mut n = 0;
        if self.lines.len() > MAX_LINES {
            self.lines.truncate(MAX_LINES);
            n += 1;
        }
        for line in &mut self.lines {
            if line.text.chars().count() > MAX_TEXT {
                line.text = line.text.chars().take(MAX_TEXT).collect();
                n += 1;
            }
            if !(line.size.is_finite() && (MIN_TEXT_SIZE..=MAX_TEXT_SIZE).contains(&line.size)) {
                line.size = default_line_size();
                n += 1;
            }
        }
        for v in [&mut self.width, &mut self.height] {
            if !(v.is_finite() && (0.0..=MAX_SYMBOL_SIZE).contains(v)) {
                *v = 0.0;
                n += 1;
            }
        }
        if !(self.padding.is_finite() && (0.0..=MAX_PADDING).contains(&self.padding)) {
            self.padding = DEFAULT_PADDING;
            n += 1;
        }
        if self.pen.is_some_and(|w| !(w.is_finite() && (MIN_PEN..=MAX_PEN).contains(&w))) {
            self.pen = None;
            n += 1;
        }
        n
    }
}

/// How one plan adjusts the tag of one element.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct TagPlacement {
    /// How far the tag was moved from its default spot, meters in plan.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub offset: DVec2,
    /// How far a grid's start bubble was moved from where it sits, meters in
    /// the view (a grid's `offset` moves its end bubble).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub start_offset: DVec2,
    /// The tag's angle, degrees counter-clockwise (`None`: as its style orients it).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation_deg: Option<f64>,
    /// The tag type of this element in this plan (`None`: the plan's for its target).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag_type: Option<ElementTypeId>,
    /// The tag is not drawn.
    #[serde(default, skip_serializing_if = "is_false")]
    pub hidden: bool,
    /// When the tag gets a leader (`None`: as its style says).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leader: Option<LeaderMode>,
}

fn is_zero(v: &DVec2) -> bool {
    *v == DVec2::ZERO
}

fn is_false(b: &bool) -> bool {
    !*b
}

impl TagPlacement {
    /// Nothing adjusted: the tag as the plan draws it by default.
    pub fn is_default(&self) -> bool {
        *self == TagPlacement::default()
    }
}

/// Whether `ty` is a tag type of `target`.
pub fn is_tag_type_of(ty: Option<&ElementType>, target: TagTarget) -> bool {
    matches!(ty.map(|t| &t.data), Some(TypeData::Tag { target: t, .. }) if *t == target)
}

/// Repair a view's tag adjustments and tag types against the project's
/// `elements` and `types`: the tag offsets of older files become
/// placements; placements of missing elements go, offsets out of reach and
/// bad angles are reset, tag types that are not of the element's target are
/// dropped, and placements left with nothing adjusted are removed. Returns
/// how many problems were fixed (moving older offsets is not one).
pub fn repair_view_tags(
    view: &mut View,
    elements: &BTreeMap<ElementId, Element>,
    types: &BTreeMap<ElementTypeId, ElementType>,
) -> usize {
    let mut n = 0;
    for (id, offset) in std::mem::take(&mut view.tag_offsets) {
        let p = view.tags.entry(id).or_default();
        if p.offset == DVec2::ZERO {
            p.offset = offset;
        }
    }
    view.tags.retain(|id, p| {
        let Some(e) = elements.get(id) else {
            n += 1;
            return false;
        };
        for o in [&mut p.offset, &mut p.start_offset] {
            if !(o.is_finite() && o.length() < MAX_OFFSET) {
                *o = DVec2::ZERO;
                n += 1;
            }
        }
        // Only a grid has a start bubble.
        if p.start_offset != DVec2::ZERO && !matches!(e.kind, ElementKind::Grid(_)) {
            p.start_offset = DVec2::ZERO;
            n += 1;
        }
        if p.rotation_deg.is_some_and(|r| !(r.is_finite() && r.abs() <= 360.0)) {
            p.rotation_deg = None;
            n += 1;
        }
        if let Some(t) = p.tag_type {
            let target = TagTarget::of_element(e, types.get(&e.type_id).map(|t| &t.data));
            if target.is_none_or(|target| !is_tag_type_of(types.get(&t), target)) {
                p.tag_type = None;
                n += 1;
            }
        }
        !p.is_default()
    });
    view.tag_types.retain(|target, t| {
        let ok = is_tag_type_of(types.get(t), *target);
        n += usize::from(!ok);
        ok
    });
    n
}

/// The symbol a tag, grid or view mark type draws.
pub fn symbol_of(data: &TypeData) -> Option<TagSymbol> {
    match data {
        TypeData::Tag { style, .. } => Some(style.symbol),
        TypeData::Grid { style, .. } => Some(style.symbol),
        TypeData::ViewMark { style, .. } => Some(style.symbol),
        _ => None,
    }
}

/// The types of `types` drawn as a detail component that is not one of
/// `types` (deleted, or of another kind).
pub fn stale_symbols(types: &BTreeMap<ElementTypeId, ElementType>) -> Vec<ElementTypeId> {
    types
        .values()
        .filter(|t| matches!(symbol_of(&t.data), Some(TagSymbol::Component(c)) if !is_detail_component(types.get(&c))))
        .map(|t| t.id)
        .collect()
}

/// Give a tag, grid or view mark type drawn as a component that is gone
/// its plain symbol back: none for tags, a circle for grids and marks.
pub fn reset_symbol(data: &mut TypeData) {
    match data {
        TypeData::Tag { style, .. } => style.symbol = TagSymbol::None,
        TypeData::Grid { style, .. } => style.symbol = TagSymbol::Circle,
        TypeData::ViewMark { style, .. } => style.symbol = TagSymbol::Circle,
        _ => {}
    }
}

/// Reset the component symbols of tag, grid and view mark types that are
/// not detail components of `types` (deleted, or of another kind); returns how many.
pub fn repair_tag_types(types: &mut BTreeMap<ElementTypeId, ElementType>) -> usize {
    let bad = stale_symbols(types);
    for id in &bad {
        if let Some(t) = types.get_mut(id) {
            reset_symbol(&mut t.data);
        }
    }
    bad.len()
}

/// Whether `ty` is a detail component type.
pub fn is_detail_component(ty: Option<&ElementType>) -> bool {
    matches!(ty.map(|t| &t.data), Some(TypeData::Component { def, .. }) if def.kind == crate::ComponentKind::Detail)
}

impl crate::Project {
    /// The tag types of `target`, in id order.
    pub fn tag_types_of(&self, target: TagTarget) -> impl Iterator<Item = &ElementType> {
        self.types.values().filter(move |t| matches!(&t.data, TypeData::Tag { target: x, .. } if *x == target))
    }

    /// The tag type `view` tags `e` with: the element's own in the view,
    /// else the view's for its target, else the project's first of the
    /// target (`None`: there is none, and the target's default style applies).
    pub fn tag_type_for(&self, view: &View, e: &Element) -> Option<&ElementType> {
        self.tag_type_in(&view.tags, &view.tag_types, e)
    }

    /// [`Self::tag_type_for`] with a view's placements and tag types.
    pub fn tag_type_in(
        &self,
        placements: &BTreeMap<ElementId, TagPlacement>,
        view_types: &BTreeMap<TagTarget, ElementTypeId>,
        e: &Element,
    ) -> Option<&ElementType> {
        let target = TagTarget::of(e, self)?;
        let fits = |id: &ElementTypeId| self.element_type(*id).filter(|t| is_tag_type_of(Some(t), target));
        placements
            .get(&e.id)
            .and_then(|p| p.tag_type.as_ref())
            .and_then(fits)
            .or_else(|| view_types.get(&target).and_then(fits))
            .or_else(|| self.tag_types_of(target).next())
    }

    /// Add a tag type of the default style for every target that has none,
    /// named by `n` from the targets' template keys. The template carries one
    /// per target; plans draw a target of a project without one (older
    /// files) in its [default style](TagTarget::default_style).
    pub fn ensure_tag_types(&mut self, n: &dyn Fn(&str) -> String) {
        for target in TagTarget::ALL {
            if self.tag_types_of(target).next().is_some() {
                continue;
            }
            let id = ElementTypeId(self.alloc_id());
            let data = TypeData::Tag { target, style: target.default_style() };
            let name = n(target.template_key());
            self.types
                .insert(id, ElementType { id, name, data, info: Default::default(), uid: crate::Uid::generate() });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn styles_are_repaired_within_their_limits() {
        let mut s = TagStyle {
            lines: (0..12).map(|_| TagLine { text: "x".repeat(300), size: f64::NAN }).collect(),
            width: -1.0,
            height: f64::INFINITY,
            padding: 1000.0,
            pen: Some(0.0),
            ..TagStyle::default()
        };
        assert!(s.repair() > 0);
        assert_eq!(s.lines.len(), MAX_LINES);
        assert!(s.lines.iter().all(|l| l.text.chars().count() == MAX_TEXT && l.size == 2.5));
        assert_eq!((s.width, s.height, s.padding, s.pen), (0.0, 0.0, DEFAULT_PADDING, None));
        assert_eq!(s.repair(), 0);
        for target in TagTarget::ALL {
            assert_eq!(target.default_style().repair(), 0, "{target:?}");
            assert_eq!(TagTarget::from_code(&target.code().to_uppercase()), Some(target));
        }
    }

    #[test]
    fn placements_and_view_tag_types_are_repaired() {
        let mut p = crate::Project::from_template();
        assert!(TagTarget::ALL.iter().all(|&t| p.tag_types_of(t).count() == 1), "the template has one per target");
        p.ensure_tag_types(&|k: &str| k.to_string());
        assert!(TagTarget::ALL.iter().all(|&t| p.tag_types_of(t).count() == 1), "and adds none to a project with one");
        let level = p.levels[0].id;
        let room_type = p.default_type(crate::Category::Room).unwrap();
        let room = ElementId(p.alloc_id());
        p.elements.insert(
            room,
            Element {
                id: room,
                level,
                type_id: room_type,
                mark: String::new(),
                comments: String::new(),
                kind: ElementKind::Room(crate::Room {
                    position: DVec2::ZERO,
                    name: "R".into(),
                    number: "1".into(),
                    height: 3.0,
                }),
                uid: Default::default(),
                phase_created: Default::default(),
                phase_demolished: None,
                design_option: None,
                paints: Vec::new(),
                plugin_data: Default::default(),
            },
        );
        let door_tag = p.tag_types_of(TagTarget::Door).next().unwrap().id;
        let room_tag = p.tag_types_of(TagTarget::Room).next().unwrap().id;
        let mut view = View::new(crate::ViewId(9999), "v", 50, crate::ViewKind::floor_plan(level));
        view.tag_offsets.insert(room, DVec2::new(1.0, 0.0));
        view.tags.insert(ElementId(123456), TagPlacement { hidden: true, ..Default::default() });
        view.tags.insert(
            room,
            TagPlacement { tag_type: Some(door_tag), rotation_deg: Some(f64::NAN), ..Default::default() },
        );
        view.tag_types.insert(TagTarget::Window, room_tag);
        view.tag_types.insert(TagTarget::Room, room_tag);
        assert_eq!(repair_view_tags(&mut view, &p.elements, &p.types), 4);
        assert!(view.tag_offsets.is_empty());
        assert_eq!(view.tags.len(), 1);
        assert_eq!(view.tags[&room], TagPlacement { offset: DVec2::new(1.0, 0.0), ..Default::default() });
        assert_eq!(view.tag_types.len(), 1);
        assert_eq!(repair_view_tags(&mut view, &p.elements, &p.types), 0);
    }
}
