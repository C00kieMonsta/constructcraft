//! Regions covered by polygons: the union of simple polygons in a plane,
//! optionally less what other polygons cover and kept within another
//! polygon, as rings a painter can fill.
//!
//! The polygons' edges are first welded (`weld`): points closer than a
//! tolerance become one, and every edge is split where another crosses it
//! or a point lies on it, so the edges meet only at their ends. They go as
//! constraints into a constrained Delaunay triangulation, every triangle is
//! in or out of the region by its centroid, and the region's outline is the
//! edges between a triangle in it and one out of it: always closed loops,
//! whatever the input. Bounded by the number of points and edges, errors
//! as values.

use crate::polygon;
use glam::DVec2;
use spade::{ConstrainedDelaunayTriangulation, HasPosition, Point2, Triangulation};
use std::collections::HashMap;

/// Points closer than this, meters, are one point, and a point this close
/// to an edge lies on it (rounding of points computed in different ways).
const TOL: f64 = 5e-8;
/// Most rounds of splitting edges where they cross or pass through points.
const WELD_ROUNDS: usize = 5;
/// Most edges the welding may make.
const MAX_EDGES: usize = 600_000;
/// Most steps the welding may take: pairs of edges compared, points looked
/// at along an edge, cells an edge is filed under.
const MAX_WELD_WORK: usize = 80_000_000;
/// Most cells along a side of the grids that find edges and points near one another.
const WELD_SIDE: usize = 256;
/// A triangle lower than this, meters, between two parts of the region is a
/// gap of rounding, not of the region: it is closed.
const THIN: f64 = 1e-7;
/// Most triangles of a gap looked at from one of them for the region on both sides.
const MAX_GAP_ROW: usize = 64;
/// Polygons smaller than this, square meters, cover nothing.
const MIN_AREA: f64 = 1e-12;
/// With more polygons than this, a grid finds those around a point.
const GRID_FROM: usize = 24;
/// Most cells along a side of that grid.
const GRID_SIDE: usize = 64;

#[derive(Debug, Clone, Copy, Default)]
struct V {
    position: Point2<f64>,
}

impl HasPosition for V {
    type Scalar = f64;
    fn position(&self) -> Point2<f64> {
        self.position
    }
}

/// The region covered by any of `polys` (simple polygons of either winding,
/// which may overlap, touch or cross), within `clip` (a simple polygon) when
/// given: rings to fill, each a simple polygon running counter-clockwise
/// around a piece of the region with its holes joined to the outline by
/// bridges walked there and back ([`polygon::keyhole`]; a piece whose holes
/// could not be joined comes as triangles). The rings do not overlap.
///
/// `None` when the polygons have more than `max_points` points in all, a
/// coordinate is not a finite number, the polygons cross one another too
/// many times (`MAX_EDGES`, `MAX_WELD_WORK`), the triangulation fails or
/// the outlines found do not make a region (one left open, a hole inside
/// none of them): never a region that covers more than the polygons do.
/// Nothing covered is `Some` of no rings.
pub fn covered(polys: &[Vec<DVec2>], clip: Option<&[DVec2]>, max_points: usize) -> Option<Vec<Vec<DVec2>>> {
    covered_within(polys, &[], clip, max_points, MAX_EDGES)
}

/// The region covered by any of `polys` and by none of `except` (simple
/// polygons too, of either winding, which may overlap one another and the
/// others), within `clip` when given: rings as [`covered`] gives them, and
/// `None` in the same cases, the points of `except` counting among the
/// `max_points`.
pub fn covered_except(
    polys: &[Vec<DVec2>],
    except: &[Vec<DVec2>],
    clip: Option<&[DVec2]>,
    max_points: usize,
) -> Option<Vec<Vec<DVec2>>> {
    covered_within(polys, except, clip, max_points, MAX_EDGES)
}

/// [`covered_except`], the welding making at most `max_edges` edges.
fn covered_within(
    polys: &[Vec<DVec2>],
    except: &[Vec<DVec2>],
    clip: Option<&[DVec2]>,
    max_points: usize,
    max_edges: usize,
) -> Option<Vec<Vec<DVec2>>> {
    let finite = |p: &[DVec2]| p.iter().all(|q| q.is_finite() && q.abs().max_element() < 1e9);
    let clip = clip.filter(|c| c.len() >= 3);
    if polys.iter().chain(except).any(|p| !finite(p)) || clip.is_some_and(|c| !finite(c)) {
        return None;
    }
    let whole = |p: &&[DVec2]| p.len() >= 3 && polygon::area(p) > MIN_AREA;
    let polys: Vec<&[DVec2]> = polys.iter().map(Vec::as_slice).filter(whole).collect();
    if polys.is_empty() {
        return Some(Vec::new());
    }
    let except: Vec<&[DVec2]> = except.iter().map(Vec::as_slice).filter(whole).collect();
    let points = polys.iter().chain(&except).map(|p| p.len()).sum::<usize>() + clip.map_or(0, <[DVec2]>::len);
    if points > max_points {
        return None;
    }
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| cover(&polys, &except, clip, max_edges))).ok().flatten()
}

/// Makes points closer than [`TOL`] one point: the first of them seen.
#[derive(Default)]
struct Welder {
    cells: HashMap<(i64, i64), Vec<DVec2>>,
}

impl Welder {
    fn cell(p: DVec2) -> (i64, i64) {
        ((p.x / TOL).floor() as i64, (p.y / TOL).floor() as i64)
    }

    /// The point standing for `p`: one within [`TOL`] already seen, else `p` itself.
    fn point(&mut self, p: DVec2) -> DVec2 {
        // No negative zero, so equal points have equal bits.
        let p = p + DVec2::ZERO;
        let (cx, cy) = Self::cell(p);
        for dy in -1..=1 {
            for dx in -1..=1 {
                let near = self.cells.get(&(cx + dx, cy + dy)).into_iter().flatten();
                if let Some(q) = near.copied().find(|q| q.distance_squared(p) <= TOL * TOL) {
                    return q;
                }
            }
        }
        self.cells.entry((cx, cy)).or_default().push(p);
        p
    }
}

/// The bits of a point, to tell equal points.
fn bits(p: DVec2) -> (u64, u64) {
    (p.x.to_bits(), p.y.to_bits())
}

/// A grid over a box: `side` cells along each axis.
#[derive(Clone, Copy)]
struct Cells {
    lo: DVec2,
    cell: DVec2,
    side: usize,
}

impl Cells {
    /// Over `lo`..`hi`, about `count` cells in all (at most [`WELD_SIDE`] a side).
    fn over(lo: DVec2, hi: DVec2, count: usize) -> Self {
        let side = ((count as f64).sqrt().ceil() as usize).clamp(1, WELD_SIDE);
        Self { lo, cell: ((hi - lo) / side as f64).max(DVec2::splat(1e-6)), side }
    }

    fn at(&self, p: DVec2) -> (usize, usize) {
        let c = ((p - self.lo) / self.cell).floor().clamp(DVec2::ZERO, DVec2::splat((self.side - 1) as f64));
        (c.x as usize, c.y as usize)
    }

    /// The indices of the cells over the box `lo`..`hi`.
    fn span(&self, lo: DVec2, hi: DVec2) -> impl Iterator<Item = usize> + use<> {
        let (a, z, side) = (self.at(lo), self.at(hi), self.side);
        (a.1..=z.1).flat_map(move |y| (a.0..=z.0).map(move |x| y * side + x))
    }
}

/// Segments already met, by the bits of their ends (the lower first).
type Known = std::collections::HashSet<((u64, u64), (u64, u64))>;

/// Add the segment `a`-`b` to `segs` unless it has no length or is there already.
fn push(known: &mut Known, segs: &mut Vec<[DVec2; 2]>, a: DVec2, b: DVec2) {
    let (ka, kb) = (bits(a), bits(b));
    if ka != kb && known.insert(if ka < kb { (ka, kb) } else { (kb, ka) }) {
        segs.push([a, b]);
    }
}

/// The edges of `rings` (closed, their points welded), none twice.
fn edges(rings: &[&[DVec2]]) -> Vec<[DVec2; 2]> {
    let mut known = Known::new();
    let mut segs: Vec<[DVec2; 2]> = Vec::new();
    for ring in rings {
        for i in 0..ring.len() {
            push(&mut known, &mut segs, ring[i], ring[(i + 1) % ring.len()]);
        }
    }
    segs
}

/// `segs` as segments that meet only at their ends: each split where
/// another crosses it and at every point within [`TOL`] of it, none twice.
/// A round splits them all; splitting moves a segment by up to [`TOL`], so
/// two may cross again, and up to `rounds` rounds are made until none
/// splits (one round leaves that unchecked). `None` past `max_edges` edges
/// or [`MAX_WELD_WORK`].
fn weld(mut segs: Vec<[DVec2; 2]>, welder: &mut Welder, rounds: usize, max_edges: usize) -> Option<Vec<[DVec2; 2]>> {
    let mut known = Known::new();
    let mut work = 0usize;
    let mut spend = |steps: usize| {
        work = work.saturating_add(steps);
        (work <= MAX_WELD_WORK).then_some(())
    };
    for _ in 0..rounds {
        if segs.is_empty() {
            break;
        }
        let lo = segs.iter().flatten().fold(DVec2::INFINITY, |a, p| a.min(*p));
        let hi = segs.iter().flatten().fold(DVec2::NEG_INFINITY, |a, p| a.max(*p));
        let grid = Cells::over(lo, hi, segs.len());
        let boxes: Vec<(DVec2, DVec2)> = segs.iter().map(|[a, b]| (a.min(*b), a.max(*b))).collect();
        let mut over: Vec<Vec<u32>> = vec![Vec::new(); grid.side * grid.side];
        for (i, (lo, hi)) in boxes.iter().enumerate() {
            // A long edge lies over many cells.
            let (a, z) = (grid.at(*lo), grid.at(*hi));
            spend((z.0 - a.0 + 1) * (z.1 - a.1 + 1))?;
            for c in grid.span(*lo, *hi) {
                over[c].push(i as u32);
            }
        }
        // The points edges are split at: their ends, and where two cross.
        let mut points: Vec<Vec<DVec2>> = vec![Vec::new(); grid.side * grid.side];
        let mut seen = std::collections::HashSet::new();
        let mut add = |points: &mut Vec<Vec<DVec2>>, p: DVec2| {
            if seen.insert(bits(p)) {
                let (x, y) = grid.at(p);
                points[y * grid.side + x].push(p);
            }
        };
        for [a, b] in &segs {
            add(&mut points, *a);
            add(&mut points, *b);
        }
        let mut crossings = 0usize;
        for (c, cell) in over.iter().enumerate() {
            for (n, &i) in cell.iter().enumerate() {
                spend(cell.len() - n)?;
                for &j in &cell[n + 1..] {
                    let (bi, bj) = (boxes[i as usize], boxes[j as usize]);
                    let corner = bi.0.max(bj.0);
                    if corner.x > bi.1.min(bj.1).x || corner.y > bi.1.min(bj.1).y {
                        continue;
                    }
                    // Each pair once: in the cell of the corner of their boxes' overlap.
                    let (x, y) = grid.at(corner);
                    if y * grid.side + x != c {
                        continue;
                    }
                    let ([a, b], [p, q]) = (segs[i as usize], segs[j as usize]);
                    let (d1, d2) = (b - a, q - p);
                    let denom = d1.perp_dot(d2);
                    if denom.abs() <= 1e-12 * d1.length() * d2.length() {
                        continue;
                    }
                    let (t, u) = ((p - a).perp_dot(d2) / denom, (p - a).perp_dot(d1) / denom);
                    if t > 0.0 && t < 1.0 && u > 0.0 && u < 1.0 {
                        let at = welder.point(a + d1 * t);
                        add(&mut points, at);
                        crossings += 1;
                        if crossings > max_edges {
                            return None;
                        }
                    }
                }
            }
        }
        // Each edge through the points on it, in order.
        let mut next: Vec<[DVec2; 2]> = Vec::with_capacity(segs.len());
        known.clear();
        let mut split = false;
        for ([a, b], (lo, hi)) in segs.iter().zip(&boxes) {
            let d = *b - *a;
            let len2 = d.length_squared();
            let mut on: Vec<(f64, DVec2)> = Vec::new();
            for c in grid.span(*lo - TOL, *hi + TOL) {
                spend(points[c].len() + 1)?;
                for p in &points[c] {
                    let t = (*p - *a).dot(d) / len2;
                    if t > 0.0 && t < 1.0 && (*a + d * t).distance_squared(*p) <= TOL * TOL && *p != *a && *p != *b {
                        on.push((t, *p));
                    }
                }
            }
            if on.is_empty() {
                push(&mut known, &mut next, *a, *b);
                continue;
            }
            split = true;
            on.sort_by(|x, y| x.0.total_cmp(&y.0));
            let mut from = *a;
            for (_, p) in on {
                push(&mut known, &mut next, from, p);
                from = p;
            }
            push(&mut known, &mut next, from, *b);
        }
        if next.len() > max_edges {
            return None;
        }
        segs = next;
        if !split {
            break;
        }
    }
    Some(segs)
}

/// A box around points.
#[derive(Debug, Clone, Copy)]
struct Box2 {
    lo: DVec2,
    hi: DVec2,
}

impl Box2 {
    fn of(points: &[DVec2]) -> Self {
        let first = points.first().copied().unwrap_or(DVec2::ZERO);
        let (lo, hi) = points.iter().fold((first, first), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
        Self { lo, hi }
    }

    fn holds(&self, p: DVec2) -> bool {
        p.x >= self.lo.x && p.x <= self.hi.x && p.y >= self.lo.y && p.y <= self.hi.y
    }
}

/// Finds the polygons whose box holds a point.
struct Index {
    boxes: Vec<Box2>,
    /// The grid over all the boxes (none for a few polygons): its origin,
    /// cell size, side and the polygons reaching each cell.
    grid: Option<(DVec2, DVec2, usize, Vec<Vec<u32>>)>,
}

impl Index {
    fn new(polys: &[Vec<DVec2>]) -> Self {
        let boxes: Vec<Box2> = polys.iter().map(|p| Box2::of(p)).collect();
        if boxes.len() < GRID_FROM {
            return Self { boxes, grid: None };
        }
        let lo = boxes.iter().fold(DVec2::INFINITY, |a, b| a.min(b.lo));
        let hi = boxes.iter().fold(DVec2::NEG_INFINITY, |a, b| a.max(b.hi));
        let side = ((boxes.len() as f64).sqrt().ceil() as usize).clamp(1, GRID_SIDE);
        let cell = ((hi - lo) / side as f64).max(DVec2::splat(1e-9));
        let mut cells = vec![Vec::new(); side * side];
        let at = |p: DVec2| ((p - lo) / cell).floor().clamp(DVec2::ZERO, DVec2::splat((side - 1) as f64));
        for (i, b) in boxes.iter().enumerate() {
            let (a, z) = (at(b.lo), at(b.hi));
            for y in a.y as usize..=z.y as usize {
                for x in a.x as usize..=z.x as usize {
                    if let Some(c) = cells.get_mut(y * side + x) {
                        c.push(i as u32);
                    }
                }
            }
        }
        Self { boxes, grid: Some((lo, cell, side, cells)) }
    }

    /// Whether any polygon holds `p`.
    fn covers(&self, polys: &[Vec<DVec2>], p: DVec2) -> bool {
        let holds = |i: usize| self.boxes.get(i).is_some_and(|b| b.holds(p)) && polygon::contains(&polys[i], p);
        match &self.grid {
            None => (0..polys.len()).any(holds),
            Some((lo, cell, side, cells)) => {
                let c = ((p - *lo) / *cell).floor();
                if c.x < 0.0 || c.y < 0.0 || c.x >= *side as f64 || c.y >= *side as f64 {
                    return false;
                }
                cells.get(c.y as usize * side + c.x as usize).is_some_and(|c| c.iter().any(|i| holds(*i as usize)))
            }
        }
    }
}

/// The triangulation of the points of `segs` with them as constraints.
/// Welded segments do not cross: when one does cross an earlier one, it is
/// split there with `split`, else there is no triangulation (`None`).
fn triangulation(segs: &[[DVec2; 2]], split: bool) -> Option<ConstrainedDelaunayTriangulation<V>> {
    let mut cdt = ConstrainedDelaunayTriangulation::<V>::new();
    let mut handles = HashMap::new();
    for [a, b] in segs {
        let mut handle = |p: DVec2| {
            if let Some(h) = handles.get(&bits(p)) {
                return Some(*h);
            }
            let h = cdt.insert(V { position: Point2::new(p.x, p.y) }).ok()?;
            handles.insert(bits(p), h);
            Some(h)
        };
        let (Some(a), Some(b)) = (handle(*a), handle(*b)) else { continue };
        if a == b || !cdt.try_add_constraint(a, b).is_empty() || cdt.exists_constraint(a, b) {
            continue;
        }
        if !split {
            return None;
        }
        cdt.add_constraint_and_split(a, b, |p| V { position: p });
    }
    Some(cdt)
}

fn cover(polys: &[&[DVec2]], except: &[&[DVec2]], clip: Option<&[DVec2]>, max_edges: usize) -> Option<Vec<Vec<DVec2>>> {
    // Around the middle of the polygons, for precision, their points welded.
    let all = Box2::of(&polys.iter().flat_map(|p| p.iter().copied()).collect::<Vec<_>>());
    let center = ((all.lo + all.hi) * 0.5).round();
    let mut welder = Welder::default();
    let mut moved = |ring: &[DVec2]| -> Vec<DVec2> { ring.iter().map(|q| welder.point(*q - center)).collect() };
    let polys: Vec<Vec<DVec2>> = polys.iter().map(|p| moved(p)).collect();
    let except: Vec<Vec<DVec2>> = except.iter().map(|p| moved(p)).collect();
    let clip: Option<Vec<DVec2>> = clip.map(&mut moved);
    let rings: Vec<&[DVec2]> = polys.iter().chain(&except).chain(clip.iter()).map(Vec::as_slice).collect();
    // One round of welding nearly always leaves no crossing; when the
    // triangulation meets one, the edges are welded until none is left.
    let segs = weld(edges(&rings), &mut welder, 1, max_edges)?;
    let cdt = match triangulation(&segs, false) {
        Some(cdt) => cdt,
        None => triangulation(&weld(segs, &mut welder, WELD_ROUNDS, max_edges)?, true)?,
    };

    // Each triangle is in the region or not, by its centroid.
    let position: Vec<DVec2> = cdt.vertices().map(|v| DVec2::new(v.position().x, v.position().y)).collect();
    let at = |i: usize| position.get(i).copied().unwrap_or(DVec2::ZERO);
    let index = Index::new(&polys);
    let left_out = Index::new(&except);
    let mut inside = vec![false; cdt.num_all_faces()];
    let corners = |f: &spade::handles::FaceHandle<spade::handles::InnerTag, V, (), spade::CdtEdge<()>, ()>| {
        f.vertices().map(|v| at(v.fix().index()))
    };
    for f in cdt.inner_faces() {
        let [a, b, c] = corners(&f);
        let p = (a + b + c) / 3.0;
        if clip.as_ref().is_none_or(|clip| polygon::contains(clip, p))
            && index.covers(&polys, p)
            && !left_out.covers(&except, p)
            && let Some(slot) = inside.get_mut(f.fix().index())
        {
            *slot = true;
        }
    }
    // Gaps of rounding between parts of the region are closed, so they make
    // one ring (painters leave a hairline between two that only touch): a
    // triangle with no height that has the region on both sides of its
    // line, across its own sides or across the triangles with no height
    // next to it (a gap is a row of them). One with the region on one side
    // only is no gap: points in a line along a side of the region or of
    // the hull, with nothing beyond; closed, a row of those would join
    // parts of the region that only stand along the same line, and frame
    // what lies between them.
    let slivers: Vec<bool> = {
        let mut slivers = vec![false; inside.len()];
        for f in cdt.inner_faces().filter(|f| inside.get(f.fix().index()) == Some(&false)) {
            let [a, b, c] = corners(&f);
            let longest = a.distance(b).max(b.distance(c)).max(c.distance(a));
            let height = (b - a).perp_dot(c - a).abs() / longest.max(1e-300);
            if let Some(slot) = slivers.get_mut(f.fix().index()) {
                *slot = height < THIN;
            }
        }
        slivers
    };
    let mut closed: Vec<usize> = Vec::new();
    let mut seen: Vec<usize> = Vec::new();
    for f in cdt.inner_faces().filter(|f| slivers.get(f.fix().index()) == Some(&true)) {
        // Its line: through the ends of its longest side.
        let [a, b, c] = corners(&f);
        let (p, q) = [(a, b), (b, c), (c, a)]
            .into_iter()
            .max_by(|x, y| x.0.distance_squared(x.1).total_cmp(&y.0.distance_squared(y.1)))
            .unwrap_or((a, b));
        let along = (q - p).normalize_or_zero();
        let (mut left, mut right) = (false, false);
        seen.clear();
        seen.push(f.fix().index());
        let mut row = vec![f];
        while let Some(sliver) = row.pop() {
            for edge in sliver.adjacent_edges() {
                let Some(next) = edge.rev().face().as_inner() else { continue };
                let i = next.fix().index();
                if inside.get(i) == Some(&true) {
                    // The side of the line its corner farthest from it stands on.
                    let off = corners(&next).map(|v| along.perp_dot(v - p));
                    let far = off.into_iter().fold(0.0f64, |m, d| if d.abs() > m.abs() { d } else { m });
                    left |= far > THIN;
                    right |= far < -THIN;
                } else if slivers.get(i) == Some(&true) && seen.len() < MAX_GAP_ROW && !seen.contains(&i) {
                    seen.push(i);
                    row.push(next);
                }
            }
            if left && right {
                closed.push(f.fix().index());
                break;
            }
        }
    }
    for i in closed {
        if let Some(slot) = inside.get_mut(i) {
            *slot = true;
        }
    }

    // The outline: the edges with the region on their left only.
    let mut from: Vec<usize> = Vec::new();
    let mut to: Vec<usize> = Vec::new();
    for f in cdt.inner_faces().filter(|f| inside.get(f.fix().index()) == Some(&true)) {
        for e in f.adjacent_edges() {
            if inside.get(e.rev().face().fix().index()) != Some(&true) {
                from.push(e.from().fix().index());
                to.push(e.to().fix().index());
            }
        }
    }
    let mut leaving: HashMap<usize, Vec<usize>> = HashMap::new();
    for (e, v) in from.iter().enumerate() {
        leaving.entry(*v).or_default().push(e);
    }
    let mut used = vec![false; from.len()];
    let mut loops: Vec<Vec<DVec2>> = Vec::new();
    for first in 0..from.len() {
        if used[first] {
            continue;
        }
        let mut ring = Vec::new();
        let mut e = first;
        // Each step takes an unused edge, so the walk ends.
        for _ in 0..=from.len() {
            used[e] = true;
            ring.push(at(from[e]) + center);
            let (u, v) = (at(from[e]), at(to[e]));
            // The region lies left of the edge: the next edge around the
            // same part of it is the first one clockwise from the way back.
            let back = u - v;
            let turn = |next: usize| {
                let d = at(to[next]) - v;
                let angle = d.perp_dot(back).atan2(d.dot(back));
                if angle <= 0.0 { angle + std::f64::consts::TAU } else { angle }
            };
            let next = leaving
                .get(&to[e])
                .into_iter()
                .flatten()
                .copied()
                .filter(|n| !used[*n] || *n == first)
                .min_by(|a, b| turn(*a).total_cmp(&turn(*b)));
            match next {
                Some(n) if n == first => break,
                Some(n) => e = n,
                // An outline is closed: an open one is a failure.
                None => return None,
            }
        }
        let ring = straightened(ring);
        if ring.len() >= 3 {
            loops.push(ring);
        }
    }

    pieces(loops)
}

/// The rings of a region from the loops of its outline: each outer loop
/// (counter-clockwise) with the holes (clockwise) inside it joined to it,
/// or cut in triangles when they cannot be. `None` when a hole lies inside
/// no outer loop: left out, the region would cover what it does not.
fn pieces(loops: Vec<Vec<DVec2>>) -> Option<Vec<Vec<DVec2>>> {
    let (outers, holes): (Vec<_>, Vec<_>) =
        loops.into_iter().map(|l| (polygon::signed_area(&l), l)).partition(|(a, _)| *a > 0.0);
    let mut pieces: Vec<(f64, Vec<DVec2>, Vec<Vec<DVec2>>)> =
        outers.into_iter().filter(|(a, _)| *a > MIN_AREA).map(|(a, l)| (a, l, Vec::new())).collect();
    for (_, hole) in holes.into_iter().filter(|(a, _)| *a < -MIN_AREA) {
        // A point inside the hole: the middle of its largest triangle (a
        // point of its outline may lie on the outline around it).
        let size = |t: &[usize; 3]| (hole[t[1]] - hole[t[0]]).perp_dot(hole[t[2]] - hole[t[0]]).abs();
        let largest = polygon::triangulate(&hole).into_iter().max_by(|a, b| size(a).total_cmp(&size(b)));
        let probe = match largest {
            Some([a, b, c]) => (hole[a] + hole[b] + hole[c]) / 3.0,
            None => (hole[0] + hole[1]) * 0.5,
        };
        let owner = pieces
            .iter_mut()
            .filter(|(_, outer, _)| polygon::contains(outer, probe))
            .min_by(|a, b| a.0.total_cmp(&b.0));
        let (_, _, holes) = owner?;
        holes.push(hole);
    }
    let mut rings = Vec::new();
    for (_, outer, holes) in pieces {
        if holes.is_empty() {
            rings.push(outer);
        } else if let Some(ring) = polygon::keyhole(&outer, &holes) {
            rings.push(ring);
        } else {
            rings.extend(polygon::triangulate_with_holes(&outer, &holes).into_iter().map(|t| t.to_vec()));
        }
    }
    Some(rings)
}

/// `ring` without the points on the line between their neighbors.
fn straightened(ring: Vec<DVec2>) -> Vec<DVec2> {
    let n = ring.len();
    if n < 4 {
        return ring;
    }
    let mut out: Vec<DVec2> = Vec::with_capacity(n);
    for i in 0..n {
        let (prev, p, next) = (ring[(i + n - 1) % n], ring[i], ring[(i + 1) % n]);
        let (a, b) = (p - prev, next - p);
        let straight = a.perp_dot(b).abs() <= 1e-9 * a.length() * b.length() && a.dot(b) > 0.0;
        if !straight {
            out.push(p);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<DVec2> {
        vec![DVec2::new(x0, y0), DVec2::new(x1, y0), DVec2::new(x1, y1), DVec2::new(x0, y1)]
    }

    fn area(rings: &[Vec<DVec2>]) -> f64 {
        rings.iter().map(|r| polygon::signed_area(r)).sum()
    }

    /// Two overlapping rectangles make one ring of their outline: eight
    /// corners, the area of both less the overlap.
    #[test]
    fn overlapping_polygons_make_one_ring() {
        let rings = covered(&[rect(0.0, 0.0, 2.0, 2.0), rect(1.0, 1.0, 3.0, 3.0)], None, 1000).unwrap();
        assert_eq!(rings.len(), 1, "{rings:?}");
        assert_eq!(rings[0].len(), 8, "{rings:?}");
        assert!((area(&rings) - 7.0).abs() < 1e-9);
        // The clockwise one and one repeated change nothing.
        let mut cw = rect(1.0, 1.0, 3.0, 3.0);
        cw.reverse();
        let again = covered(&[rect(0.0, 0.0, 2.0, 2.0), cw.clone(), cw], None, 1000).unwrap();
        assert!((area(&again) - 7.0).abs() < 1e-9 && again.len() == 1);
    }

    /// Polygons that share a side, whole or in part, join into one ring with
    /// no trace of the side: painters leave no hairline between them.
    #[test]
    fn polygons_sharing_a_side_join() {
        let rings = covered(&[rect(0.0, 0.0, 1.0, 1.0), rect(1.0, 0.0, 2.0, 1.0)], None, 1000).unwrap();
        assert_eq!(rings, vec![rect(0.0, 0.0, 2.0, 1.0)]);
        // A side lying on part of another, a hair off (rounding).
        let rings = covered(&[rect(0.0, 0.0, 1.0, 3.0), rect(1.0 + 3e-9, 1.0, 2.0, 2.0)], None, 1000).unwrap();
        assert_eq!(rings.len(), 1, "{rings:?}");
        assert!((area(&rings) - 4.0).abs() < 1e-6);
    }

    /// Polygons apart make a ring each; a clip keeps what lies inside it.
    #[test]
    fn a_clip_keeps_what_is_inside_it() {
        let polys = [rect(0.0, 0.0, 1.0, 1.0), rect(3.0, 0.0, 5.0, 1.0)];
        let rings = covered(&polys, None, 1000).unwrap();
        assert_eq!(rings.len(), 2);
        assert!((area(&rings) - 3.0).abs() < 1e-9);
        let clip = rect(0.5, -1.0, 4.0, 0.5);
        let rings = covered(&polys, Some(&clip), 1000).unwrap();
        assert_eq!(rings.len(), 2);
        assert!((area(&rings) - 0.75).abs() < 1e-9, "{rings:?}");
        // A concave clip: an L around the first square's corner.
        let l = vec![
            DVec2::new(-1.0, -1.0),
            DVec2::new(2.0, -1.0),
            DVec2::new(2.0, 0.5),
            DVec2::new(0.5, 0.5),
            DVec2::new(0.5, 2.0),
            DVec2::new(-1.0, 2.0),
        ];
        let rings = covered(&polys[..1], Some(&l), 1000).unwrap();
        assert!((area(&rings) - 0.75).abs() < 1e-9, "{rings:?}");
        // Nothing inside the clip, and nothing at all.
        assert_eq!(covered(&polys, Some(&rect(10.0, 10.0, 11.0, 11.0)), 1000), Some(Vec::new()));
        assert_eq!(covered(&[], None, 1000), Some(Vec::new()));
    }

    /// What other polygons cover is left out: a square less a strip across
    /// it is two rings, less a square inside it one ring around a hole, less
    /// one over all of it nothing; the left out polygons count as points.
    #[test]
    fn polygons_are_left_out_of_the_region() {
        let square = [rect(0.0, 0.0, 4.0, 4.0)];
        let rings = covered_except(&square, &[rect(-1.0, 1.0, 5.0, 2.0)], None, 1000).unwrap();
        assert_eq!(rings.len(), 2, "{rings:?}");
        assert!((area(&rings) - 12.0).abs() < 1e-9);
        assert!(rings.iter().all(|r| r.len() == 4), "{rings:?}");
        // A hole, and what lies outside the region left out too: nothing more.
        let inside = [rect(1.0, 1.0, 2.0, 2.0), rect(10.0, 10.0, 11.0, 11.0)];
        let rings = covered_except(&square, &inside, None, 1000).unwrap();
        assert_eq!(rings.len(), 1, "{rings:?}");
        assert!((area(&rings) - 15.0).abs() < 1e-9);
        assert!(!polygon::contains(&rings[0], DVec2::new(1.5, 1.5)));
        // Sharing a side with the region, overlapping one another, within a clip.
        let two = [rect(0.0, 0.0, 2.0, 2.0), rect(2.0, 0.0, 4.0, 2.0)];
        let out = [rect(1.0, -1.0, 3.0, 1.0), rect(2.0, 0.5, 5.0, 1.5)];
        let rings = covered_except(&two, &out, Some(&rect(0.0, 0.0, 4.0, 1.75)), 1000).unwrap();
        // 4 x 1.75, less 2 x 1 and the strip's 2 x 1, which share 1 x 0.5.
        assert!((area(&rings) - 3.5).abs() < 1e-9, "{rings:?}");
        assert_eq!(covered_except(&square, &[rect(-1.0, -1.0, 5.0, 5.0)], None, 1000), Some(Vec::new()));
        assert_eq!(covered_except(&square, &[], None, 1000), covered(&square, None, 1000));
        assert_eq!(covered_except(&square, &[rect(1.0, 1.0, 2.0, 2.0)], None, 7), None);
        let bad = vec![DVec2::ZERO, DVec2::X, DVec2::new(f64::INFINITY, 1.0)];
        assert_eq!(covered_except(&square, &[bad], None, 1000), None);
    }

    /// Four bars around a square leave it as a hole: one ring, its area the
    /// bars', and a point of the hole outside it.
    #[test]
    fn a_frame_keeps_its_hole() {
        let bars =
            [rect(0.0, 0.0, 3.0, 1.0), rect(0.0, 2.0, 3.0, 3.0), rect(0.0, 0.0, 1.0, 3.0), rect(2.0, 0.0, 3.0, 3.0)];
        let rings = covered(&bars, None, 1000).unwrap();
        assert_eq!(rings.len(), 1, "{rings:?}");
        assert!((area(&rings) - 8.0).abs() < 1e-9, "{}", area(&rings));
        assert!(!polygon::contains(&rings[0], DVec2::new(1.5, 1.5)));
        assert!(polygon::contains(&rings[0], DVec2::new(0.5, 1.5)));
    }

    /// Shadows cast on a triangular face, cut to it, as a view made them:
    /// four pieces apart from one another, whose points along the face's
    /// sides stand in a line to within a rounding error. The triangles of
    /// no height between those points, along the sides, are no gaps between
    /// parts of the region (nothing lies beyond them): closed, they framed
    /// the face, and the lit middle, a hole no outline held, was filled.
    #[test]
    fn points_in_a_line_along_the_hull_close_nothing() {
        let polys: Vec<Vec<DVec2>> = [
            vec![
                (-0.8962693548787477, -0.7170154839029981),
                (-1.3590920585351973, 1.1102230246251565e-16),
                (-5.0, 1.7384954415228017e-17),
                (-4.901361488574789, -2.0253774345976927),
                (-1.0662213527404827, -0.8529770821923861),
            ],
            vec![
                (-4.880846246389449, -2.4466237408033518),
                (-3.058279676004191, -2.4466237408033518),
                (-2.82772257003142, -2.262178056025135),
                (-4.889828990777998, -2.262178056025135),
            ],
            vec![
                (-4.9487577519516845, -1.0521741599254357),
                (-4.212017473662425, 0.0),
                (-4.332017473662423, 4.440892098500626e-16),
                (-4.956561313020961, -0.8919410393029626),
            ],
            vec![
                (-4.815595709322152, -3.7864347685852215),
                (-4.730817409049434, -3.784653927239546),
                (-4.727504156926133, -3.782003325540905),
                (-4.815631657454811, -3.785696633594619),
            ],
            vec![
                (-4.812502039673509, -3.849958118704018),
                (-4.812447648380024, -3.849958118704018),
                (-4.745049414154074, -3.796039531323258),
                (-4.815414614210567, -3.790153254876421),
            ],
        ]
        .into_iter()
        .map(|p| p.into_iter().map(|(x, y)| DVec2::new(x, y)).collect())
        .collect();
        let rings = covered(&polys, None, 1000).unwrap();
        // The first holds the third; the others lie apart.
        let apart: f64 = [0, 1, 3, 4].iter().map(|i| polygon::area(&polys[*i])).sum();
        assert!((area(&rings) - apart).abs() < 1e-9, "{} against {apart}", area(&rings));
        assert!((apart - 5.947_317_305_852_886).abs() < 1e-9);
        assert_eq!(rings.len(), 4, "{rings:?}");
        for i in 0..100 {
            for j in 0..100 {
                let p = DVec2::new(-5.0 + (i as f64 + 0.37) * 0.05, -3.85 + (j as f64 + 0.61) * 0.0385);
                let inside = polys.iter().any(|t| polygon::contains(t, p));
                assert_eq!(rings.iter().filter(|r| polygon::contains(r, p)).count(), usize::from(inside), "{p}");
            }
        }
    }

    /// A hole belongs to the smallest outline around it; one that no
    /// outline holds makes no region (rather than one without the hole).
    #[test]
    fn a_hole_inside_no_outline_is_a_failure() {
        let mut hole = rect(1.0, 1.0, 2.0, 2.0);
        hole.reverse();
        let rings = pieces(vec![rect(0.0, 0.0, 4.0, 4.0), hole.clone(), rect(10.0, 0.0, 11.0, 1.0)]).unwrap();
        assert_eq!(rings.len(), 2);
        assert!((area(&rings) - 16.0).abs() < 1e-9, "{}", area(&rings));
        let mut away = rect(6.0, 1.0, 7.0, 2.0);
        away.reverse();
        assert_eq!(pieces(vec![rect(0.0, 0.0, 4.0, 4.0), hole, away]), None);
        assert_eq!(pieces(Vec::new()), Some(Vec::new()));
    }

    /// Too many points, and numbers that are not finite, are refused; a
    /// polygon with no area covers nothing.
    #[test]
    fn bad_input_is_refused() {
        let polys = [rect(0.0, 0.0, 1.0, 1.0), rect(0.5, 0.5, 2.0, 2.0)];
        assert_eq!(covered(&polys, None, 7), None);
        assert!(covered(&polys, None, 8).is_some());
        assert_eq!(covered(&[vec![DVec2::ZERO, DVec2::X, DVec2::new(f64::NAN, 1.0)]], None, 100), None);
        let flat = vec![DVec2::ZERO, DVec2::X, DVec2::new(2.0, 0.0)];
        assert_eq!(covered(&[flat], None, 100), Some(Vec::new()));
    }

    /// Polygons that cross one another too many times are refused, not
    /// worked on without end: strips across strips, a crossing at every corner.
    #[test]
    fn too_many_crossings_are_refused() {
        let mut polys: Vec<Vec<DVec2>> = Vec::new();
        for i in 0..12 {
            let at = i as f64 * 0.5;
            polys.push(rect(0.0, at, 10.0, at + 0.2));
            polys.push(rect(at, 0.0, at + 0.2, 10.0));
        }
        // 24 bars cross at 576 points, making some 1500 edges.
        assert_eq!(covered_within(&polys, &[], None, 1000, 500), None);
        let rings = covered_within(&polys, &[], None, 1000, 5000).unwrap();
        let bars = 24.0 * 10.0 * 0.2 - 144.0 * 0.2 * 0.2;
        assert!((area(&rings) - bars).abs() < 1e-6, "{} against {bars}", area(&rings));
        assert_eq!(rings.len(), 1, "one ring, its 121 holes joined to it");
    }

    /// Many polygons crossing one another: the area is that of their union
    /// (a row of squares each half over the next), found through the grid.
    #[test]
    fn many_polygons_are_found_through_the_grid() {
        let n = 200;
        let polys: Vec<Vec<DVec2>> = (0..n).map(|i| rect(i as f64 * 0.5, 0.0, i as f64 * 0.5 + 1.0, 1.0)).collect();
        let rings = covered(&polys, None, 10_000).unwrap();
        assert_eq!(rings.len(), 1);
        assert_eq!(rings[0].len(), 4, "the points along the sides are left out");
        assert!((area(&rings) - (n as f64 * 0.5 + 0.5)).abs() < 1e-9);
    }

    /// Points a hair off an edge lie on it: a corner that should touch an
    /// edge and passes it by a rounding error (two crossings almost at one
    /// point), edges almost along one another and many edges through almost
    /// one point all weld into clean outlines.
    #[test]
    fn points_a_hair_off_an_edge_lie_on_it() {
        // A triangle whose tip just passes the rectangle's lower side: it
        // touches it at a point, a ring each.
        let tip = vec![DVec2::new(0.5, -1.0), DVec2::new(1.5, -1.0), DVec2::new(1.0, 1e-10)];
        let rings = covered(&[rect(0.0, 0.0, 2.0, 1.0), tip], None, 1000).unwrap();
        assert_eq!(rings.len(), 2, "{rings:?}");
        assert!(rings.iter().all(|r| r.len() <= 5), "{rings:?}");
        assert!((area(&rings) - 2.5).abs() < 1e-6, "{}", area(&rings));
        // Sides almost along one another, their ends in between each other's.
        let a = rect(0.0, 0.0, 3.0, 1.0);
        let b = vec![
            DVec2::new(1.0, 1.0 + 2e-10),
            DVec2::new(4.0, 1.0 - 3e-10),
            DVec2::new(4.0, 2.0),
            DVec2::new(1.0, 2.0),
        ];
        let rings = covered(&[a, b], None, 1000).unwrap();
        assert_eq!(rings.len(), 1, "{rings:?}");
        assert!((area(&rings) - 6.0).abs() < 1e-6);
        // Twelve thin blades through (almost) one point, like a fan.
        let fan: Vec<Vec<DVec2>> = (0..12)
            .map(|i| {
                let (s, c) = (i as f64 * 0.26).sin_cos();
                let (d, n) = (DVec2::new(c, s), DVec2::new(-s, c) * 0.05);
                let o = DVec2::new(1e-10 * i as f64, -1e-10 * i as f64);
                vec![o - d * 2.0 - n, o + d * 2.0 - n, o + d * 2.0 + n, o - d * 2.0 + n]
            })
            .collect();
        let rings = covered(&fan, None, 10_000).unwrap();
        assert!(!rings.is_empty());
        let total: f64 = fan.iter().map(|p| polygon::area(p)).sum();
        assert!(area(&rings) > 0.4 && area(&rings) < total, "{} of {total}", area(&rings));
    }

    /// Many triangles between the points of a lattice, each a rounding
    /// error off (sides along one another, points on sides, many sides
    /// through a point): the union has the area a count of sample points
    /// gives, whatever the seed.
    #[test]
    fn unions_of_jittered_lattice_triangles_have_the_sampled_area() {
        let mut seed = 0x2545_F491_4F6C_DD1Du64;
        let mut rand = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            (seed >> 11) as f64 / (1u64 << 53) as f64
        };
        for round in 0..20 {
            let mut point = || {
                let lattice = DVec2::new((rand() * 6.0).floor(), (rand() * 6.0).floor());
                lattice + DVec2::new(rand() - 0.5, rand() - 0.5) * 2e-10
            };
            let polys: Vec<Vec<DVec2>> = (0..40).map(|_| vec![point(), point(), point()]).collect();
            let clip = (round % 2 == 1).then(|| rect(0.7, 0.7, 4.3, 4.3));
            let rings = covered(&polys, clip.as_deref(), 10_000).unwrap_or_else(|| panic!("round {round}"));
            // Sample points off the lattice's lines.
            let n = 120;
            let mut hits = 0;
            for i in 0..n {
                for j in 0..n {
                    let p = DVec2::new((i as f64 + 0.37) * 5.0 / n as f64, (j as f64 + 0.61) * 5.0 / n as f64);
                    let inside = polys.iter().any(|t| polygon::contains(t, p))
                        && clip.as_ref().is_none_or(|c| polygon::contains(c, p));
                    hits += usize::from(inside);
                    let ringed = rings.iter().filter(|r| polygon::contains(r, p)).count();
                    assert_eq!(ringed, usize::from(inside), "round {round}: {p} is in {ringed} rings");
                }
            }
            let sampled = hits as f64 * 25.0 / (n * n) as f64;
            assert!((area(&rings) - sampled).abs() < 0.35, "round {round}: {} against {sampled}", area(&rings));
        }
    }
}
