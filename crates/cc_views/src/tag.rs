//! Tags of elements in plans (see [`cc_core::tag`]): each target finds its
//! tag's default spot ([`TagSpot`]), and [`draw`] draws the tag there as
//! its type's style says, moved, turned, retyped or hidden as the view's
//! placement for the element says. Every primitive belongs to the element,
//! so a tag hides and highlights with it.
//!
//! The text lines are laid out on a pitch of 1.44 times the largest text
//! (3.6 mm for 2.5 mm text); an empty line is not drawn but keeps its place.

use crate::drawing::Anchor;
use crate::notes::{self, SymbolPens};
use crate::{Drawing2D, LineStyle, Pen};
use cc_core::tag::{LeaderMode, TagOrientation, TagPlacement, TagStyle, TagSymbol, TagTarget};
use cc_core::{Element, ElementId, ElementTypeId, Project, ScheduleField};
use cc_geom::ModelGeometry;
use glam::DVec2;
use std::borrow::Cow;
use std::collections::BTreeMap;

/// Nearly invisible fill that makes text tags clickable.
const PICK_AREA: [u8; 4] = [255, 255, 255, 1];
/// White behind an opaque tag text.
const OPAQUE: [u8; 4] = [255, 255, 255, 255];
/// Most characters of an expanded line.
const MAX_LINE_CHARS: usize = 1024;

/// What tags are drawn with in one view.
pub struct TagCtx<'a> {
    pub project: &'a Project,
    pub geom: &'a ModelGeometry,
    pub scale: u32,
    /// The view's adjustments, by element.
    pub placements: &'a BTreeMap<ElementId, TagPlacement>,
    /// The view's tag type per target.
    pub types: &'a BTreeMap<TagTarget, ElementTypeId>,
    /// Translates the keys some fields give (yes/no, kinds).
    pub label: &'a dyn Fn(&str) -> String,
}

impl TagCtx<'_> {
    /// Model meters per paper millimeter.
    pub fn mm(&self) -> f64 {
        self.scale as f64 / 1000.0
    }
}

/// How a tag's box sits on its spot's point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    /// Centered on it (doors, windows).
    Center,
    /// Its bottom-left corner on it (columns, footings).
    LeftBottom,
    /// Its bottom middle on it, along the spot's direction (beams).
    CenterBottom,
    /// The first line's baseline 0.4 mm above it, the lines centered under
    /// one another (rooms); a symbol is centered on it.
    FirstBaseline,
}

/// Where a target puts its tag before the view moves it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TagSpot {
    pub point: DVec2,
    /// The point of the element a leader runs to.
    pub anchor: DVec2,
    /// The host's direction, for tags that turn with it (already readable).
    pub host_dir: Option<DVec2>,
    pub align: Align,
}

/// A tag ready to draw: its style and expanded lines.
pub struct Tag<'a> {
    pub element: ElementId,
    pub style: Cow<'a, TagStyle>,
    /// Each line's text (empty lines keep their place) and size.
    pub lines: Vec<(String, f64)>,
    pub placement: TagPlacement,
}

/// The style `ctx`'s view tags `e` (of `target`) with.
pub fn style_of<'a>(ctx: &TagCtx<'a>, e: &Element, target: TagTarget) -> Cow<'a, TagStyle> {
    match ctx.project.tag_type_in(ctx.placements, ctx.types, e).map(|t| &t.data) {
        Some(cc_core::TypeData::Tag { style, .. }) => Cow::Borrowed(style),
        _ => Cow::Owned(target.default_style()),
    }
}

/// The tag of `e` in `ctx`'s view, or `None` when it is hidden or writes nothing.
pub fn prepare<'a>(ctx: &TagCtx<'a>, e: &Element, target: TagTarget) -> Option<Tag<'a>> {
    let placement = ctx.placements.get(&e.id).copied().unwrap_or_default();
    if placement.hidden {
        return None;
    }
    let style = style_of(ctx, e, target);
    let lines: Vec<(String, f64)> = style.lines.iter().map(|l| (expand_line(ctx, e, &l.text), l.size)).collect();
    let component = matches!(style.symbol, TagSymbol::Component(_));
    if lines.iter().all(|(t, _)| t.is_empty()) && !component {
        return None;
    }
    Some(Tag { element: e.id, style, lines, placement })
}

/// `template` with its fields filled in for `e`, trimmed, its runs of
/// whitespace made single spaces.
pub fn expand_line(ctx: &TagCtx, e: &Element, template: &str) -> String {
    let raw = expand(ctx, e, template);
    let mut out = String::new();
    for word in raw.split_whitespace() {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(word);
    }
    out.chars().take(MAX_LINE_CHARS).collect()
}

/// `template` with each `{Field}` replaced by `e`'s value (unknown fields by nothing).
pub fn expand(ctx: &TagCtx, e: &Element, template: &str) -> String {
    use cc_core::title_block::{FieldPart, field_parts};
    let mut out = String::new();
    field_parts(template, |part| match part {
        FieldPart::Text { shown, .. } => out.push_str(shown),
        FieldPart::Field { key, .. } => out.push_str(&field(ctx, e, key.trim())),
    });
    out
}

/// The value of the field `key` for `e`, as a tag writes it.
pub fn field(ctx: &TagCtx, e: &Element, key: &str) -> String {
    let p = ctx.project;
    match key {
        "Mark" => e.mark.clone(),
        "TypeName" => p.type_name(e.type_id).to_string(),
        "Section" => {
            p.element_type(e.type_id).and_then(|t| crate::annotate::section_label(p, &t.data)).unwrap_or_default()
        }
        // A room not enclosed has no area.
        "Area" if matches!(e.kind, cc_core::ElementKind::Room(_)) => match ctx.geom.rooms.get(&e.id) {
            Some(boundary) => crate::format::area(cc_geom::polygon::area(boundary)),
            None => "—".into(),
        },
        _ => ScheduleField::from_name(key)
            .map(|f| crate::schedule::cell_text(p, ctx.geom, e, f, ctx.label))
            .unwrap_or_default(),
    }
}

/// Estimated width of `text` at `size` paper millimeters, in meters.
fn text_width(text: &str, size: f64, mm: f64) -> f64 {
    text.chars().count().max(1) as f64 * size * 0.6 * mm
}

/// Layout of a tag in its own frame (x along its direction, y across;
/// meters from its spot's point): its box and where its lines go.
struct Layout {
    lo: DVec2,
    hi: DVec2,
    /// The symbol's box (none for text alone).
    symbol: Option<(DVec2, DVec2)>,
    /// Each drawn line: position, text, size and anchor.
    texts: Vec<(DVec2, String, f64, Anchor)>,
    /// The box of the text (for an opaque fill).
    text_box: (DVec2, DVec2),
}

impl Tag<'_> {
    /// Largest text size, paper millimeters.
    fn largest(&self) -> f64 {
        self.lines.iter().fold(0.0_f64, |a, (_, s)| a.max(*s))
    }

    /// Width and height of the text block, meters.
    fn text_size(&self, mm: f64) -> (f64, f64) {
        let pitch = 3.6 * (self.largest() / 2.5);
        let width =
            self.lines.iter().filter(|(t, _)| !t.is_empty()).fold(0.0_f64, |a, (t, s)| a.max(text_width(t, *s, mm)));
        let first = self.lines.first().map_or(0.0, |(_, s)| *s);
        let n = self.lines.len().saturating_sub(1) as f64;
        (width, (n * pitch + first) * mm)
    }

    /// Half the size of the symbol's box, meters (`None`: text alone).
    fn symbol_half(&self, mm: f64) -> Option<DVec2> {
        if self.style.symbol == TagSymbol::None {
            return None;
        }
        let (tw, th) = self.text_size(mm);
        let pad = self.style.padding * mm;
        let fit = |set: f64, text: f64| if set > 0.0 { set / 2.0 * mm } else { text / 2.0 + pad };
        let mut half = DVec2::new(fit(self.style.width, tw), fit(self.style.height, th));
        if self.style.symbol.is_regular() {
            half = DVec2::splat(half.x.max(half.y));
        }
        Some(half)
    }

    /// Half the size of the tag's box at `align`, meters (unturned).
    pub fn half(&self, align: Align, mm: f64) -> DVec2 {
        let l = self.layout(align, mm);
        (l.hi - l.lo) / 2.0
    }

    fn layout(&self, align: Align, mm: f64) -> Layout {
        let (tw, th) = self.text_size(mm);
        let k = self.largest() / 2.5;
        let line = 3.6 * k * mm;
        let n = self.lines.len();
        // Lines centered on `c`, one under another.
        let centered = |c: DVec2| -> Vec<(DVec2, String, f64, Anchor)> {
            let top = (n.saturating_sub(1)) as f64 / 2.0;
            self.lines
                .iter()
                .enumerate()
                .filter(|(_, (t, _))| !t.is_empty())
                .map(|(i, (t, s))| (c + DVec2::new(0.0, (top - i as f64) * line), t.clone(), *s, Anchor::Center))
                .collect()
        };
        if let Some(half) = self.symbol_half(mm) {
            let (lo, hi) = match align {
                Align::Center | Align::FirstBaseline => (-half, half),
                Align::LeftBottom => (DVec2::ZERO, half * 2.0),
                Align::CenterBottom => (DVec2::new(-half.x, 0.0), DVec2::new(half.x, half.y * 2.0)),
            };
            let c = (lo + hi) / 2.0;
            let tb = DVec2::new(tw / 2.0, th / 2.0);
            return Layout { lo, hi, symbol: Some((lo, hi)), texts: centered(c), text_box: (c - tb, c + tb) };
        }
        let at = |i: usize| (n - 1 - i) as f64 * line;
        let drawn = |anchor: Anchor, pos: &dyn Fn(usize) -> DVec2| -> Vec<(DVec2, String, f64, Anchor)> {
            self.lines
                .iter()
                .enumerate()
                .filter(|(_, (t, _))| !t.is_empty())
                .map(|(i, (t, s))| (pos(i), t.clone(), *s, anchor))
                .collect()
        };
        match align {
            Align::Center => {
                let half = DVec2::new(tw / 2.0, th / 2.0);
                Layout { lo: -half, hi: half, symbol: None, texts: centered(DVec2::ZERO), text_box: (-half, half) }
            }
            Align::LeftBottom => {
                let hi = DVec2::new(tw, th);
                let texts = drawn(Anchor::LeftBottom, &|i| DVec2::new(0.0, at(i)));
                Layout { lo: DVec2::ZERO, hi, symbol: None, texts, text_box: (DVec2::ZERO, hi) }
            }
            Align::CenterBottom => {
                let (lo, hi) = (DVec2::new(-tw / 2.0, 0.0), DVec2::new(tw / 2.0, th));
                let texts = drawn(Anchor::CenterBottom, &|i| DVec2::new(0.0, at(i)));
                Layout { lo, hi, symbol: None, texts, text_box: (lo, hi) }
            }
            Align::FirstBaseline => {
                let chars = self.lines.iter().fold(0, |a, (t, _)| a.max(t.chars().count()));
                let width = (chars as f64 * (1.6 * k) + 2.0) * mm;
                let lo = DVec2::new(-width / 2.0, -(n.saturating_sub(1) as f64 * line));
                let hi = DVec2::new(width / 2.0, (0.4 + 3.0 * k) * mm);
                let texts = drawn(Anchor::CenterBottom, &|i| DVec2::new(0.0, -(i as f64 * line - 0.4 * mm)));
                let text_box = (DVec2::new(-tw / 2.0, lo.y), DVec2::new(tw / 2.0, hi.y));
                Layout { lo, hi, symbol: None, texts, text_box }
            }
        }
    }
}

/// The direction a tag's x axis runs in: the view's angle for it, else
/// along its host when its style says so, else level.
fn direction(tag: &Tag, spot: &TagSpot) -> DVec2 {
    if let Some(deg) = tag.placement.rotation_deg.filter(|d| d.is_finite()) {
        return DVec2::from_angle(deg.to_radians());
    }
    match (tag.style.orientation, spot.host_dir) {
        // A unit direction is taken as it is, so the tag lies exactly along it.
        (TagOrientation::AlongHost, Some(d)) if (d.length_squared() - 1.0).abs() < 1e-9 => d,
        (TagOrientation::AlongHost, Some(d)) => d.try_normalize().unwrap_or(DVec2::X),
        _ => DVec2::X,
    }
}

/// Make `dir` read left to right or bottom to top.
pub fn readable(mut dir: DVec2) -> DVec2 {
    if dir.x < -1e-9 || (dir.x.abs() <= 1e-9 && dir.y < 0.0) {
        dir = -dir;
    }
    dir
}

/// The pen of a style's symbol and leader (`None`: the annotation pen).
fn pen(style: &TagStyle) -> Option<Pen> {
    (style.pen.is_some() || style.color.is_some()).then_some(Pen {
        color: style.color,
        weight: style.pen,
        ..Pen::default()
    })
}

/// Draw `tag` at `spot` (moved by its placement's offset), with a leader
/// back to the element when its leader mode says so, and record its box so
/// it can be dragged (`handle`: rooms are picked by their tag, so theirs is
/// recorded only once moved). `crossed` crosses the tag out (a room not
/// enclosed, when the style says so).
pub fn draw(d: &mut Drawing2D, ctx: &TagCtx, tag: &Tag, spot: TagSpot, crossed: bool) {
    let mm = ctx.mm();
    let id = Some(tag.element);
    let along = direction(tag, &spot);
    let up = along.perp();
    let angle = along.y.atan2(along.x);
    let layout = tag.layout(spot.align, mm);
    let offset = Some(tag.placement.offset).filter(|o| o.is_finite()).unwrap_or(DVec2::ZERO);
    let origin = spot.point + offset;
    let to = |v: DVec2| origin + (along * v.x + up * v.y);
    // The box, turned, as an upright box for picking and leaders.
    let (lh, center_local) = ((layout.hi - layout.lo) / 2.0, (layout.lo + layout.hi) / 2.0);
    let (s, c) = angle.sin_cos();
    let half = DVec2::new((lh.x * c).abs() + (lh.y * s).abs(), (lh.x * s).abs() + (lh.y * c).abs());
    let center = to(center_local);
    let moved = offset.length() > 1e-9;
    if spot.align != Align::FirstBaseline || moved {
        d.tags.push(crate::TagHandle { element: tag.element, center, half, offset });
    }
    let pen = pen(&tag.style);
    let outside = !(spot.anchor - center).abs().cmple(half).all();
    let leader = match tag.placement.leader.unwrap_or(tag.style.leader) {
        LeaderMode::Auto => moved && outside,
        LeaderMode::Always => outside,
        LeaderMode::Never => false,
    };
    if leader {
        // The leader stops at the edge of the tag's box.
        let dir = (spot.anchor - center).normalize_or_zero();
        let tx = if dir.x.abs() > 1e-12 { half.x / dir.x.abs() } else { f64::INFINITY };
        let ty = if dir.y.abs() > 1e-12 { half.y / dir.y.abs() } else { f64::INFINITY };
        let edge = center + dir * tx.min(ty);
        d.line_pen(spot.anchor, edge, LineStyle::Annotation, pen, id);
        if tag.style.leader_end != cc_core::LeaderSymbol::None {
            let u = (spot.anchor - edge).try_normalize().unwrap_or(DVec2::X);
            let pens = SymbolPens {
                stroke: (LineStyle::Annotation, pen),
                tick: (LineStyle::Annotation, pen),
                fill: notes::symbol_fill(tag.style.color),
            };
            notes::symbol_at(d, tag.style.leader_end, u, spot.anchor, 2.0 * mm, pens, id);
        }
    }
    if let Some((lo, hi)) = layout.symbol {
        let pts: Vec<DVec2> = symbol_outline(tag.style.symbol, lo, hi).into_iter().map(to).collect();
        match tag.style.symbol {
            TagSymbol::Underline => {
                if let [a, b] = pts[..] {
                    d.line_pen(a, b, LineStyle::Annotation, pen, id);
                }
            }
            TagSymbol::Component(type_id) => draw_component(d, ctx, tag, type_id, to((lo + hi) / 2.0), angle),
            _ => polyline_pen(d, &pts, pen, id),
        }
    }
    if tag.style.fill {
        let pad = DVec2::splat(0.4 * mm);
        let (lo, hi) = (layout.text_box.0 - pad, layout.text_box.1 + pad);
        d.fill(vec![to(lo), to(DVec2::new(hi.x, lo.y)), to(hi), to(DVec2::new(lo.x, hi.y))], OPAQUE, id);
    }
    for (pos, text, size, anchor) in &layout.texts {
        d.tag_text(to(*pos), text.as_str(), *size, *anchor, angle, id);
    }
    if spot.align == Align::FirstBaseline && layout.symbol.is_none() {
        let (lo, hi) = (layout.lo, layout.hi);
        d.fill(vec![to(lo), to(DVec2::new(hi.x, lo.y)), to(hi), to(DVec2::new(lo.x, hi.y))], PICK_AREA, id);
    }
    if crossed && tag.style.crossed_when_unplaced {
        let (p, s) = (origin, 2.5 * mm);
        d.line_pen(p + DVec2::new(-s, -s), p + DVec2::new(s, s), LineStyle::Annotation, pen, id);
        d.line_pen(p + DVec2::new(-s, s), p + DVec2::new(s, -s), LineStyle::Annotation, pen, id);
    }
}

/// Closed outline from point to point.
fn polyline_pen(d: &mut Drawing2D, pts: &[DVec2], pen: Option<Pen>, id: Option<ElementId>) {
    for w in pts.windows(2) {
        d.line_pen(w[0], w[1], LineStyle::Annotation, pen, id);
    }
    if let (Some(&last), Some(&first)) = (pts.last(), pts.first())
        && pts.len() > 2
    {
        d.line_pen(last, first, LineStyle::Annotation, pen, id);
    }
}

/// The points of a symbol filling the box `lo`..`hi` in the tag's frame
/// (the two ends of an underline).
fn symbol_outline(symbol: TagSymbol, lo: DVec2, hi: DVec2) -> Vec<DVec2> {
    let c = (lo + hi) / 2.0;
    let r = (hi - lo) / 2.0;
    let around = |n: usize| -> Vec<DVec2> {
        (0..n)
            .map(|i| {
                let v = DVec2::from_angle(i as f64 / n as f64 * std::f64::consts::TAU);
                c + DVec2::new(v.x * r.x, v.y * r.y)
            })
            .collect()
    };
    match symbol {
        TagSymbol::Circle => around(24),
        TagSymbol::Hexagon => around(6),
        TagSymbol::Ellipse => around(32),
        TagSymbol::Diamond => {
            vec![c - DVec2::new(r.x, 0.0), c - DVec2::new(0.0, r.y), c + DVec2::new(r.x, 0.0), c + DVec2::new(0.0, r.y)]
        }
        TagSymbol::Triangle => vec![lo, DVec2::new(hi.x, lo.y), DVec2::new(c.x, hi.y)],
        TagSymbol::Rectangle => vec![lo, DVec2::new(hi.x, lo.y), hi, DVec2::new(lo.x, hi.y)],
        TagSymbol::RoundedRectangle => {
            let rad = r.x.min(r.y) * 0.4;
            let mut pts = Vec::new();
            let corners = [
                (DVec2::new(hi.x - rad, lo.y + rad), -0.25),
                (DVec2::new(hi.x - rad, hi.y - rad), 0.0),
                (DVec2::new(lo.x + rad, hi.y - rad), 0.25),
                (DVec2::new(lo.x + rad, lo.y + rad), 0.5),
            ];
            for (k, start) in corners {
                for j in 0..=4 {
                    let a = (start + j as f64 / 16.0) * std::f64::consts::TAU;
                    pts.push(k + DVec2::from_angle(a) * rad);
                }
            }
            pts
        }
        TagSymbol::Underline => vec![lo, DVec2::new(hi.x, lo.y)],
        TagSymbol::None | TagSymbol::Component(_) => Vec::new(),
    }
}

/// A detail component drawn as the tag at `center`, turned `angle`: its
/// meters are paper millimeters times 1000, its texts' fields filled in.
fn draw_component(d: &mut Drawing2D, ctx: &TagCtx, tag: &Tag, type_id: ElementTypeId, center: DVec2, angle: f64) {
    let p = ctx.project;
    let Some(cc_core::TypeData::Component { component, def }) = p.element_type(type_id).map(|t| &t.data) else {
        return;
    };
    if def.kind != cc_core::ComponentKind::Detail {
        return;
    }
    let Some(e) = p.element(tag.element) else { return };
    let place = cc_core::detail_component::Place {
        t: cc_core::transform::PlanTransform { angle, mirror: false, translation: center },
        scale: f64::from(ctx.scale.max(1)),
    };
    let lookup = |n: &str| p.detail_def(n);
    cc_core::detail_component::walk(&lookup, component, def, &Default::default(), place, &mut |v| {
        let mut pieces = v.pieces.clone();
        for t in &mut pieces.texts {
            t.text = expand(ctx, e, &t.text);
        }
        crate::detail_component::draw_pieces(d, &p.fill_patterns, &pieces, &v.place, ctx.scale, Some(tag.element));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_fill_fields_and_keep_braces() {
        let project = Project::empty();
        let geom = ModelGeometry::default();
        let (placements, types) = (BTreeMap::new(), BTreeMap::new());
        let ctx = TagCtx {
            project: &project,
            geom: &geom,
            scale: 50,
            placements: &placements,
            types: &types,
            label: &|k| k.to_string(),
        };
        let mut e = cc_core::Element {
            id: ElementId(1),
            level: cc_core::LevelId(1),
            type_id: ElementTypeId(1),
            mark: "P1".into(),
            comments: "  two   words ".into(),
            kind: cc_core::ElementKind::Room(cc_core::Room {
                position: DVec2::ZERO,
                number: "7".into(),
                name: "Sala".into(),
                height: 3.0,
            }),
            uid: Default::default(),
            phase_created: Default::default(),
            phase_demolished: None,
            design_option: None,
            paints: Vec::new(),
        };
        assert_eq!(expand_line(&ctx, &e, " {Mark}  {Comments} {{x}} {Nothing}"), "P1 two words {x}");
        assert_eq!(expand_line(&ctx, &e, "{Name} {Number} {Area}"), "Sala 7 —", "a room not enclosed has no area");
        e.mark.clear();
        assert_eq!(expand_line(&ctx, &e, "{Mark} {Section}"), "");
    }

    #[test]
    fn regular_symbols_fit_the_larger_side() {
        let tag = Tag {
            element: ElementId(1),
            style: Cow::Owned(TagStyle { symbol: TagSymbol::Circle, ..TagStyle::default() }),
            lines: vec![("ABCDEFGH".into(), 2.5)],
            placement: TagPlacement::default(),
        };
        let half = tag.half(Align::Center, 0.1);
        assert!((half.x - half.y).abs() < 1e-12 && half.x > 0.0);
        for symbol in [TagSymbol::Rectangle, TagSymbol::Diamond, TagSymbol::Triangle, TagSymbol::Ellipse] {
            let pts = symbol_outline(symbol, DVec2::new(-1.0, -1.0), DVec2::ONE);
            assert!(pts.len() >= 3 && pts.iter().all(|p| p.abs().max_element() <= 1.0 + 1e-12), "{symbol:?}");
        }
    }
}
