//! Pile cap reinforcement: the bars of a cap's sets, laid out from its
//! layout and outline when the geometry is built, so they follow its type.
//!
//! Each family is laid out by its role for whatever the cap is now, so a
//! set stays meaningful when the type's layout changes:
//!
//! - bottom bars (`CapBottom`) over the piles: in a cap of three piles a
//!   band between each two (along the triangle's sides); in a cap of four a
//!   band along each side of the square, and with `diagonals` one across each
//!   diagonal; each band `count` bars spread over a pile's width, centered on
//!   the line through the two piles' axes. In a cap of two piles or of a line
//!   of them, `count` bars along the line of the piles spread across the
//!   cap's width (a beam's bottom bars). In a cap of one pile, a round cap and
//!   a ring, a mesh `spacing` apart both ways (ring bars cut by the hole).
//!   Bands that cross lie one on the other, a bar diameter apart, in the
//!   order above (a mesh's first direction, along the cap, lowest);
//! - the top mesh (`CapTop`), both ways under the top, the first direction
//!   highest;
//! - side bars (`CapSides`): closed loops around the outline (around both
//!   faces of a ring) between the bottom and the top bars;
//! - stirrups (`Stirrups`): across a cap of two piles or of a line, as a
//!   beam's; other caps have no conventional stirrups, and lay out none.
//!
//! Bars run to the outline less the cover and the bars outside them: the
//! stirrups lie the cover inside the faces, the side loops inside the
//! stirrups, the bottom and top bars inside both, and their ends turn up
//! (bottom) or down (top) the side faces by `bend`, at most to the other
//! face less the cover. The bottom bars lie on the piles' heads: the lowest
//! bar is the embedment (at least the cover) above the cap's bottom.

use crate::pile::CapGeom;
use crate::polygon;
use crate::rebar::{Bar, LoopHooks, MAX_BARS, from_convex_corner, side_heights, stations};
use cc_core::pile::{CapOutline, PileLayout};
use cc_core::{Rebar, RebarLayout};
use glam::{DVec2, DVec3};

/// Chords of a round side loop.
const LOOP_CHORDS: usize = 64;

/// A pile cap as its bars see it.
pub(crate) struct CapInput<'a> {
    /// The cap's frame, plan and piles.
    pub cap: &'a CapGeom,
    /// The layout of its type's piles.
    pub layout: PileLayout,
    /// Elevations of its bottom and its top.
    pub z: (f64, f64),
    /// How far the piles' heads enter it.
    pub embedment: f64,
    /// The piles' diameter (or side).
    pub pile_size: f64,
    /// The largest diameter of the cap's stirrups (0: none, or a cap that lays
    /// out none) and of its side loops (0: none): the other bars lie inside them.
    pub stirrup: f64,
    pub skin: f64,
}

impl CapInput<'_> {
    /// Whether the cap is a beam over a line of piles: two, or a line.
    fn beam_like(&self) -> bool {
        is_beam_like(self.layout, self.cap.piles.len())
    }

    /// The lowest face of any bar: the piles' heads, at least the cover above the bottom.
    fn floor(&self, cover: f64) -> f64 {
        self.z.0 + cover.max(self.embedment.max(0.0))
    }

    /// The highest face of any bar: the cover below the top.
    fn ceiling(&self, cover: f64) -> f64 {
        self.z.1 - cover
    }
}

/// Whether a cap of `layout` with `piles` piles is a beam over a line of
/// piles (two, or a line), whose stirrups are laid out.
pub(crate) fn is_beam_like(layout: PileLayout, piles: usize) -> bool {
    matches!(layout, PileLayout::Pair | PileLayout::Line { .. }) && piles >= 2
}

/// The plan a cap's bars may stand in: its outline moved in by an inset.
#[derive(Debug, Clone, PartialEq)]
enum Region {
    Polygon(Vec<DVec2>),
    Disk(f64),
    Ring(f64, f64),
}

impl Region {
    /// `outline` moved `inset` inside (a ring's two faces apart); `None` when
    /// nothing is left.
    fn inset(outline: &CapOutline, inset: f64) -> Option<Region> {
        if !(inset.is_finite() && inset >= 0.0) {
            return None;
        }
        match outline {
            CapOutline::Polygon(p) => {
                let outer = polygon::ccw(p);
                let n = outer.len();
                if n < 3 || outer.iter().any(|q| !q.is_finite()) {
                    return None;
                }
                let moved = polygon::offset(&outer, -inset);
                // Every side keeps its direction, or the inset has eaten it.
                let side = |p: &[DVec2], i: usize| match (p.get(i), p.get((i + 1) % n)) {
                    (Some(a), Some(b)) => *b - *a,
                    _ => DVec2::ZERO,
                };
                let kept = moved.len() == n && (0..n).all(|i| side(&moved, i).dot(side(&outer, i)) > 1e-12);
                (kept && polygon::area(&moved).abs() > 1e-12).then_some(Region::Polygon(moved))
            }
            CapOutline::Disk { radius } => {
                let r = radius - inset;
                (r.is_finite() && r > 1e-6).then_some(Region::Disk(r))
            }
            CapOutline::Ring { inner, outer } => {
                let (a, b) = (inner + inset, outer - inset);
                (a.is_finite() && b.is_finite() && a > 1e-6 && b - a > 1e-6).then_some(Region::Ring(a, b))
            }
        }
    }

    /// The stretches `(t0, t1)` of the line `origin + t·dir` (`dir` a unit
    /// vector) inside the region, rising.
    fn stretches(&self, origin: DVec2, dir: DVec2) -> Vec<(f64, f64)> {
        // Where the line crosses the circle of radius `r` about the origin.
        let chord = |r: f64| {
            let b = origin.dot(dir);
            let disc = b * b - (origin.length_squared() - r * r);
            (disc > 0.0).then(|| (-b - disc.sqrt(), -b + disc.sqrt()))
        };
        let out = match self {
            Region::Polygon(p) => polygon::clip_line(p, origin, dir),
            Region::Disk(r) => chord(*r).into_iter().collect(),
            Region::Ring(inner, outer) => match (chord(*outer), chord(*inner)) {
                (Some((a0, a1)), Some((b0, b1))) => vec![(a0, b0), (b1, a1)],
                (Some(a), None) => vec![a],
                _ => Vec::new(),
            },
        };
        out.into_iter().filter(|(a, b)| a.is_finite() && b.is_finite() && b - a > 1e-6).collect()
    }

    /// The least and greatest `p · n` over the region.
    fn extent(&self, n: DVec2) -> (f64, f64) {
        match self {
            Region::Polygon(p) => p
                .iter()
                .map(|q| q.dot(n))
                .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| (lo.min(v), hi.max(v))),
            Region::Disk(r) | Region::Ring(_, r) => (-r, *r),
        }
    }

    /// The closed loops of a ring of bars around the region's faces, and the
    /// true length of each (circles exact).
    fn loops(&self) -> Vec<(Vec<DVec2>, Option<f64>)> {
        let circle = |r: f64| {
            let points = (0..LOOP_CHORDS)
                .map(|i| DVec2::from_angle(i as f64 / LOOP_CHORDS as f64 * std::f64::consts::TAU) * r)
                .collect();
            (points, Some(std::f64::consts::TAU * r))
        };
        match self {
            Region::Polygon(p) => vec![(p.clone(), None)],
            Region::Disk(r) => vec![circle(*r)],
            Region::Ring(inner, outer) => vec![circle(*outer), circle(*inner)],
        }
    }
}

/// The part of the polyline `points` between `from` and `to` along it.
fn cut(points: &[DVec3], from: f64, to: f64) -> Vec<DVec3> {
    let mut out: Vec<DVec3> = Vec::new();
    let mut walked = 0.0;
    for w in points.windows(2) {
        let (a, b) = (w[0], w[1]);
        let len = a.distance(b);
        let (s0, s1) = (walked, walked + len);
        walked = s1;
        if len <= 1e-12 || s1 < from - 1e-12 || s0 > to + 1e-12 {
            continue;
        }
        let at = |s: f64| a + (b - a) * ((s - s0) / len).clamp(0.0, 1.0);
        if out.is_empty() {
            out.push(at(from.max(s0)));
        }
        out.push(at(to.min(s1)));
    }
    out.dedup_by(|p, q| p.distance(*q) < 1e-9);
    out
}

/// Bars along the polylines `lines` (each its developed bar, legs and run),
/// cut by the set's splices where longer than the stock.
fn spliced(r: &Rebar, lines: Vec<Vec<DVec3>>) -> Vec<Bar> {
    let mut bars = Vec::new();
    for (index, points) in lines.into_iter().enumerate() {
        let length: f64 = points.windows(2).map(|w| w[0].distance(w[1])).sum();
        if !(length.is_finite() && length > 1e-6) {
            continue;
        }
        let pieces = r.splice.map_or_else(|| vec![(0.0, length)], |s| s.pieces(length, index));
        for (from, to) in pieces {
            if bars.len() >= MAX_BARS {
                return bars;
            }
            let piece = cut(&points, from, to);
            if piece.len() >= 2 {
                bars.push(Bar::new(piece, false));
            }
        }
    }
    bars
}

/// The bars of the set `r` (bars `d` thick) in the cap `c`.
pub(crate) fn cap_bars(c: &CapInput, r: &Rebar, d: f64) -> Vec<Bar> {
    let cover = r.cover;
    if !(cover.is_finite() && d.is_finite() && d > 0.0) {
        return Vec::new();
    }
    let (floor, ceiling) = (c.floor(cover), c.ceiling(cover));
    let room = ceiling - floor;
    if !(room.is_finite() && room > d) {
        return Vec::new();
    }
    let stirrup = if c.beam_like() { c.stirrup } else { 0.0 };
    let world = |p: DVec2, z: f64| c.cap.world(p).extend(z);
    // The bottom and top bars: inside the stirrups and the side loops.
    let main = || Region::inset(&c.cap.outline, cover + stirrup + c.skin + d / 2.0);
    let bottom = |layer: usize| floor + stirrup + d / 2.0 + d * layer as f64;
    let top = |layer: usize| ceiling - stirrup - d / 2.0 - d * layer as f64;
    // A bar from `a` to `b` (in the cap's frame) at `z`, its ends bent by `leg` (up when positive).
    let bar = |a: DVec2, b: DVec2, z: f64, leg: f64| -> Vec<DVec3> {
        let (a, b) = (world(a, z), world(b, z));
        if leg.abs() > 1e-6 { vec![a + DVec3::Z * leg, a, b, b + DVec3::Z * leg] } else { vec![a, b] }
    };
    // Bars across the region along `dir`, at `offsets` from the line through `through`.
    let lines = |region: &Region, through: DVec2, dir: DVec2, offsets: &[f64], z: f64, leg: f64| {
        let mut out = Vec::new();
        for o in offsets {
            let origin = through + dir.perp() * *o;
            for (t0, t1) in region.stretches(origin, dir) {
                out.push(bar(origin + dir * t0, origin + dir * t1, z, leg));
            }
        }
        out
    };
    // A mesh `spacing` apart both ways, the first direction (along the cap) at `z(0)`.
    let mesh = |region: &Region, spacing: f64, z: &dyn Fn(usize) -> f64, leg: &dyn Fn(f64) -> f64| {
        let mut out = Vec::new();
        for (layer, dir) in [DVec2::X, DVec2::Y].into_iter().enumerate() {
            let (lo, hi) = region.extent(dir.perp());
            let offsets: Vec<f64> = stations(lo, hi, spacing).into_iter().map(|o| inside(o, lo, hi)).collect();
            out.extend(lines(region, DVec2::ZERO, dir, &offsets, z(layer), leg(z(layer))));
        }
        out
    };
    match r.layout {
        RebarLayout::CapBottom { count, spacing, diagonals, bend } => {
            let Some(region) = main() else { return Vec::new() };
            // Up the side faces, at most to the top less the cover.
            let leg = |z: f64| bend.min(ceiling - z).max(0.0);
            let count = count.min(cc_core::pile_bars::MAX_BAND) as usize;
            let piles = &c.cap.piles;
            let pile = |i: usize| piles.get(i).copied();
            let bands: Vec<(usize, usize, usize)> = match (c.layout, piles.len()) {
                (PileLayout::Triangle, 3) => vec![(0, 1, 0), (1, 2, 1), (2, 0, 2)],
                (PileLayout::Square, 4) => {
                    let mut out = vec![(0, 1, 0), (3, 2, 0), (1, 2, 1), (0, 3, 1)];
                    if diagonals {
                        out.extend([(0, 2, 2), (1, 3, 3)]);
                    }
                    out
                }
                _ => Vec::new(),
            };
            let lines = if !bands.is_empty() {
                // A band over each two piles, as wide as a pile.
                let half = (c.pile_size / 2.0 - d / 2.0).max(0.0);
                let offsets = spread(count, half);
                let mut out = Vec::new();
                for (i, j, layer) in bands {
                    let (Some(a), Some(b)) = (pile(i), pile(j)) else { continue };
                    let dir = (b - a).normalize_or_zero();
                    if dir == DVec2::ZERO {
                        continue;
                    }
                    let z = bottom(layer);
                    out.extend(lines(&region, a, dir, &offsets, z, leg(z)));
                }
                out
            } else if c.beam_like() {
                // Along the line of the piles, spread across the cap's width.
                let (lo, hi) = region.extent(DVec2::Y);
                let mid = (lo + hi) / 2.0;
                let offsets: Vec<f64> = spread(count, ((hi - lo) / 2.0).max(0.0))
                    .into_iter()
                    .map(|o| inside(mid + o, lo, hi) - mid)
                    .collect();
                let through = DVec2::Y * mid;
                let z = bottom(0);
                lines(&region, through, DVec2::X, &offsets, z, leg(z))
            } else {
                mesh(&region, spacing, &bottom, &leg)
            };
            spliced(r, lines)
        }
        RebarLayout::CapTop { spacing, bend } => {
            let Some(region) = main() else { return Vec::new() };
            // Down the side faces, at most to the bottom less the cover.
            let leg = |z: f64| -bend.min(z - floor).max(0.0);
            spliced(r, mesh(&region, spacing, &top, &leg))
        }
        RebarLayout::CapSides { spacing, hooks } => {
            let Some(region) = Region::inset(&c.cap.outline, cover + stirrup + d / 2.0) else { return Vec::new() };
            let hooks = hooks.and_then(|h| LoopHooks::of(&h, d));
            let heights = side_heights(0, spacing, bottom(0), top(0));
            let loops = region.loops();
            let mut bars = Vec::new();
            for z in heights {
                for (outline, length) in &loops {
                    if bars.len() >= MAX_BARS {
                        return bars;
                    }
                    let outline = if hooks.is_some() { from_convex_corner(outline.clone()) } else { outline.clone() };
                    let points = outline.iter().map(|p| world(*p, z)).collect();
                    bars.push(Bar { true_length: *length, hooks, ..Bar::new(points, true) });
                }
            }
            bars
        }
        // Across a cap of two piles or of a line, as a beam's: none in other caps.
        RebarLayout::Stirrups { spacing, hooks } if c.beam_like() => {
            let (lo, hi) = c.cap.outline.bounds();
            let inset = cover + d / 2.0;
            let (u0, u1) = (lo.x + inset, hi.x - inset);
            let (v0, v1) = (lo.y + inset, hi.y - inset);
            let (z0, z1) = (floor + d / 2.0, ceiling - d / 2.0);
            if !(v1 - v0 > 1e-6 && z1 - z0 > 1e-6) {
                return Vec::new();
            }
            let hooks = hooks.and_then(|h| LoopHooks::of(&h, d));
            stations(u0, u1, spacing)
                .into_iter()
                .take(MAX_BARS)
                .map(|u| {
                    let at = |v: f64, z: f64| world(DVec2::new(u, v), z);
                    Bar { hooks, ..Bar::new(vec![at(v0, z0), at(v1, z0), at(v1, z1), at(v0, z1)], true) }
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

/// Why a cap of `outline`, `thickness` deep, whose piles enter it by
/// `embedment`, has no room for bars `cover` inside its faces (`None` when
/// it has room for bars of up to 2 cm).
pub(crate) fn room_problem(outline: &CapOutline, thickness: f64, embedment: f64, cover: f64) -> Option<&'static str> {
    const BAR: f64 = 0.02;
    if Region::inset(outline, cover + BAR).is_none() {
        return Some("the cover leaves no room in the cap's plan");
    }
    let room = thickness - cover.max(embedment) - cover;
    if !(room.is_finite() && room > 2.0 * BAR) {
        return Some("the cap is too thin for the cover and the piles' embedment");
    }
    None
}

/// `v` moved a hair inside `[lo, hi]` when it lies on an end: a line along
/// a side of the region would cut nothing.
fn inside(v: f64, lo: f64, hi: f64) -> f64 {
    const HAIR: f64 = 1e-9;
    if hi - lo > 2.0 * HAIR { v.clamp(lo + HAIR, hi - HAIR) } else { v }
}

/// `n` offsets spread evenly over `[-half, half]` (the middle for one).
fn spread(n: usize, half: f64) -> Vec<f64> {
    match n {
        0 => Vec::new(),
        1 => vec![0.0],
        _ => (0..n).map(|i| -half + 2.0 * half * i as f64 / (n - 1) as f64).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regions_clip_lines_to_their_outline() {
        let square = CapOutline::Polygon(vec![
            DVec2::new(-1.0, -1.0),
            DVec2::new(1.0, -1.0),
            DVec2::new(1.0, 1.0),
            DVec2::new(-1.0, 1.0),
        ]);
        let r = Region::inset(&square, 0.1).unwrap();
        let s = r.stretches(DVec2::new(0.0, 0.5), DVec2::X);
        assert_eq!(s.len(), 1);
        assert!((s[0].0 + 0.9).abs() < 1e-12 && (s[0].1 - 0.9).abs() < 1e-12, "{s:?}");
        assert!(r.stretches(DVec2::new(0.0, 0.95), DVec2::X).is_empty());
        assert!(Region::inset(&square, 1.0).is_none());
        // A disk: the chord; a ring: the two pieces either side of the hole.
        let disk = Region::inset(&CapOutline::Disk { radius: 1.0 }, 0.2).unwrap();
        let s = disk.stretches(DVec2::ZERO, DVec2::Y);
        assert!((s[0].0 + 0.8).abs() < 1e-12 && (s[0].1 - 0.8).abs() < 1e-12);
        let ring = Region::inset(&CapOutline::Ring { inner: 1.0, outer: 2.0 }, 0.1).unwrap();
        let s = ring.stretches(DVec2::ZERO, DVec2::X);
        assert_eq!(s.len(), 2);
        assert!((s[0].0 + 1.9).abs() < 1e-12 && (s[0].1 + 1.1).abs() < 1e-12, "{s:?}");
        assert_eq!(ring.stretches(DVec2::new(0.0, 1.5), DVec2::X).len(), 1);
        assert!(Region::inset(&CapOutline::Ring { inner: 1.0, outer: 1.1 }, 0.1).is_none());
        // Values a file may hold.
        assert!(Region::inset(&square, f64::NAN).is_none());
        assert!(Region::inset(&CapOutline::Polygon(vec![DVec2::NAN; 3]), 0.1).is_none());
        assert!(disk.stretches(DVec2::NAN, DVec2::X).is_empty());
    }

    #[test]
    fn polylines_are_cut_along_their_length() {
        let l = vec![DVec3::new(0.0, 0.0, 0.3), DVec3::ZERO, DVec3::new(2.0, 0.0, 0.0)];
        let piece = cut(&l, 0.2, 1.3);
        let want = [DVec3::new(0.0, 0.0, 0.1), DVec3::ZERO, DVec3::new(1.0, 0.0, 0.0)];
        assert!(piece.len() == 3 && piece.iter().zip(want).all(|(a, b)| a.distance(b) < 1e-12), "{piece:?}");
        assert_eq!(cut(&l, 0.0, 2.3).len(), 3);
    }
}
