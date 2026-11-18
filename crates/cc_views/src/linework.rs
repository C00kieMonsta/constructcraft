//! Clean linework from a painted drawing, for formats that draw every line
//! (DXF). A [`Drawing2D`] is painted first to last: in projected views a
//! face's fill covers the edges of the faces behind it. [`clean`] keeps
//! what shows: each line less the parts strictly inside an opaque fill
//! painted after it, the cut fills and the fills of view-owned elements
//! (the surface fills of the model were only shading and occluders), and
//! the remaining lines of one layer, color, dash pattern and style merged
//! where they overlap and chained into polylines where two ends meet.
//!
//! Lengths are drawing units (meters on the view). Points closer than the
//! tolerance [`tolerance`] gives are one point: a line within it of a fill's
//! boundary stays visible, pieces shorter are dropped, ends within it meet.

use crate::{Drawing2D, LineStyle, Pen, Prim};
use cc_core::ElementId;
use glam::DVec2;
use std::collections::{BTreeMap, BTreeSet};

/// The smallest tolerance, drawing units: a micrometer on a view in meters,
/// far below what any plot shows and far above the rounding of projected points.
pub const MIN_TOLERANCE: f64 = 1e-6;
/// The tolerance relative to the drawing's extent (its diagonal), so huge
/// coordinates keep a tolerance above their rounding.
pub const RELATIVE_TOLERANCE: f64 = 1e-9;
/// Most a direction may turn, radians, for two segments to lie on one line.
pub const ANGLE_TOLERANCE: f64 = 1e-9;
/// Most unit steps of work (a fill's edge met by a line, a grid cell
/// visited) before [`clean`] gives up.
pub const MAX_WORK: u64 = 400_000_000;
/// Most visible pieces of lines [`clean`] makes before it gives up.
pub const MAX_SEGMENTS: usize = 2_000_000;
/// Most cells along each side of the grid that indexes the fills.
const MAX_GRID_SIDE: usize = 512;

/// A run of connected segments of one layer, color, dash pattern and style.
#[derive(Debug, Clone, PartialEq)]
pub struct Polyline {
    /// At least two points; a closed one returns from the last to the first.
    pub pts: Vec<DVec2>,
    pub closed: bool,
    /// The layer the caller named for its lines.
    pub layer: String,
    pub style: LineStyle,
    pub pen: Option<Pen>,
    /// The element of its first segment.
    pub element: Option<ElementId>,
}

/// A drawing cleaned by [`clean`]: the primitives other than lines that it
/// keeps (in drawing order, with any line whose points are not finite) and
/// the visible lines.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Linework {
    pub prims: Vec<Prim>,
    pub lines: Vec<Polyline>,
}

/// Why a drawing could not be cleaned: it is too large for the limits
/// ([`MAX_WORK`], [`MAX_SEGMENTS`]); the caller writes it as it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineworkError {
    TooMuchWork,
    TooManySegments,
}

/// What [`clean`] asks of the caller.
pub struct Options<'a> {
    /// The layer of a line: lines of different layers never merge.
    pub layer: &'a dyn Fn(&Prim) -> String,
    /// Whether an element belongs to a view (regions, notes): its fills are kept.
    pub view_owned: &'a dyn Fn(ElementId) -> bool,
    /// Elements written some other way (blocks): their primitives are left
    /// out, but their fills still hide what they cover.
    pub skip: &'a BTreeSet<ElementId>,
}

/// The tolerance for a drawing whose points span `diagonal`.
pub fn tolerance(diagonal: f64) -> f64 {
    let t = diagonal * RELATIVE_TOLERANCE;
    if t.is_finite() { t.max(MIN_TOLERANCE) } else { MIN_TOLERANCE }
}

/// The visible linework of `drawing` (see the module's documentation), or
/// an error when it passes a limit: never a drawing cleaned in part.
pub fn clean(drawing: &Drawing2D, options: &Options<'_>) -> Result<Linework, LineworkError> {
    clean_within(drawing, options, MAX_WORK, MAX_SEGMENTS)
}

/// [`clean`] within other limits.
fn clean_within(
    drawing: &Drawing2D,
    options: &Options<'_>,
    max_work: u64,
    max_segments: usize,
) -> Result<Linework, LineworkError> {
    let prims = &drawing.prims;
    let tol = tolerance(extent(prims));
    let skipped = |p: &Prim| p.element().is_some_and(|e| options.skip.contains(&e));
    let mut work = Work { done: 0, max: max_work };
    let occluders: Vec<Occluder<'_>> = prims
        .iter()
        .enumerate()
        .filter_map(|(index, p)| match p {
            Prim::Fill { pts, color, .. } if color[3] == 255 && pts.len() >= 3 && pts.iter().all(|p| p.is_finite()) => {
                let (lo, hi) = bounds(pts);
                Some(Occluder { index, pts, lo: lo - DVec2::splat(tol), hi: hi + DVec2::splat(tol) })
            }
            _ => None,
        })
        .collect();
    let grid = Grid::new(&occluders, &mut work)?;
    let mut out = Linework::default();
    let mut groups: BTreeMap<Key, Group> = BTreeMap::new();
    let mut stamp = vec![usize::MAX; occluders.len()];
    let mut candidates = Vec::new();
    let mut hidden = Vec::new();
    let mut ts = Vec::new();
    let mut pieces = 0usize;
    for (i, prim) in prims.iter().enumerate() {
        if skipped(prim) {
            continue;
        }
        let Prim::Line { a, b, style, element, pen } = prim else {
            let keep = match prim {
                Prim::Fill { cut, element, .. } => *cut || element.is_some_and(|e| (options.view_owned)(e)),
                _ => true,
            };
            if keep {
                out.prims.push(prim.clone());
            }
            continue;
        };
        let (a, b) = (*a, *b);
        if !(a.is_finite() && b.is_finite()) {
            out.prims.push(prim.clone());
            continue;
        }
        let len = a.distance(b);
        if len < tol {
            continue;
        }
        // The fills painted later whose boxes the line meets.
        candidates.clear();
        grid.visit(a, b, tol, &mut work, &mut candidates, |k| {
            let o = occluders.get(k)?;
            let seen = stamp.get_mut(k)?;
            if *seen == i || o.index <= i {
                return None;
            }
            *seen = i;
            cc_geom::polygon::segment_meets_rect(a, b, o.lo, o.hi).then_some(k)
        })?;
        hidden.clear();
        for &k in &candidates {
            if let Some(o) = occluders.get(k) {
                hidden_by(a, b, o.pts, tol, &mut ts, &mut hidden, &mut work)?;
            }
        }
        let visible = complement(&mut hidden, tol / len);
        let layer = (options.layer)(prim);
        let group = groups.entry(Key::of(prim, layer.clone())).or_insert_with(|| Group {
            layer,
            style: *style,
            pen: *pen,
            segs: Vec::new(),
        });
        for (t0, t1) in visible {
            pieces += 1;
            if pieces > max_segments {
                return Err(LineworkError::TooManySegments);
            }
            let order = group.segs.len();
            group.segs.push(Seg { a: a.lerp(b, t0), b: a.lerp(b, t1), order, element: *element });
        }
    }
    for group in groups.into_values() {
        let merged = merge_collinear(group.segs, tol);
        for (pts, closed, element) in chain(&merged, tol) {
            out.lines.push(Polyline {
                pts,
                closed,
                layer: group.layer.clone(),
                style: group.style,
                pen: group.pen,
                element,
            });
        }
    }
    Ok(out)
}

/// The diagonal of the box of the drawing's finite points.
fn extent(prims: &[Prim]) -> f64 {
    let mut lo = DVec2::splat(f64::INFINITY);
    let mut hi = DVec2::splat(f64::NEG_INFINITY);
    let mut grow = |p: DVec2| {
        if p.is_finite() {
            lo = lo.min(p);
            hi = hi.max(p);
        }
    };
    for p in prims {
        match p {
            Prim::Line { a, b, .. } => {
                grow(*a);
                grow(*b);
            }
            Prim::Fill { pts, .. } => pts.iter().for_each(|p| grow(*p)),
            Prim::Text { pos, .. } => grow(*pos),
            Prim::Image { origin, u, v, .. } => Prim::image_corners(*origin, *u, *v).iter().for_each(|p| grow(*p)),
        }
    }
    if lo.is_finite() && hi.is_finite() { lo.distance(hi) } else { 0.0 }
}

fn bounds(pts: &[DVec2]) -> (DVec2, DVec2) {
    pts.iter()
        .fold((DVec2::splat(f64::INFINITY), DVec2::splat(f64::NEG_INFINITY)), |(lo, hi), p| (lo.min(*p), hi.max(*p)))
}

/// Counts the work done against its limit ([`MAX_WORK`]).
struct Work {
    done: u64,
    max: u64,
}

impl Work {
    fn add(&mut self, steps: usize) -> Result<(), LineworkError> {
        self.done = self.done.saturating_add(steps as u64);
        if self.done > self.max { Err(LineworkError::TooMuchWork) } else { Ok(()) }
    }
}

/// An opaque fill: where it is painted, its outline and its box grown by
/// the tolerance.
struct Occluder<'a> {
    index: usize,
    pts: &'a [DVec2],
    lo: DVec2,
    hi: DVec2,
}

/// A uniform grid over the fills' boxes: each cell lists the fills whose
/// boxes meet it, in drawing order.
struct Grid {
    lo: DVec2,
    cell: DVec2,
    nx: usize,
    ny: usize,
    cells: Vec<Vec<usize>>,
}

impl Grid {
    fn new(occluders: &[Occluder<'_>], work: &mut Work) -> Result<Self, LineworkError> {
        let mut lo = DVec2::splat(f64::INFINITY);
        let mut hi = DVec2::splat(f64::NEG_INFINITY);
        for o in occluders {
            lo = lo.min(o.lo);
            hi = hi.max(o.hi);
        }
        let side = ((occluders.len() as f64).sqrt().ceil() as usize).clamp(1, MAX_GRID_SIDE);
        let (lo, size) = if lo.is_finite() && hi.is_finite() { (lo, hi - lo) } else { (DVec2::ZERO, DVec2::ONE) };
        let cell = DVec2::new(size.x.max(1e-300) / side as f64, size.y.max(1e-300) / side as f64);
        let mut grid = Grid { lo, cell, nx: side, ny: side, cells: vec![Vec::new(); side * side] };
        for (k, o) in occluders.iter().enumerate() {
            let ([x0, y0], [x1, y1]) = (grid.at(o.lo), grid.at(o.hi));
            work.add((x1 - x0 + 1) * (y1 - y0 + 1))?;
            for y in y0..=y1 {
                for x in x0..=x1 {
                    if let Some(c) = grid.cells.get_mut(y * grid.nx + x) {
                        c.push(k);
                    }
                }
            }
        }
        Ok(grid)
    }

    /// The cell of `p`, clamped to the grid.
    fn at(&self, p: DVec2) -> [usize; 2] {
        let f = (p - self.lo) / self.cell;
        let clamp = |v: f64, n: usize| if v.is_finite() { (v.floor().max(0.0) as usize).min(n - 1) } else { 0 };
        [clamp(f.x, self.nx), clamp(f.y, self.ny)]
    }

    /// Calls `pick` on the fills listed in the cells the box of `a`–`b`
    /// (grown by `tol`) meets, gathering what it returns in `found`, sorted.
    fn visit(
        &self,
        a: DVec2,
        b: DVec2,
        tol: f64,
        work: &mut Work,
        found: &mut Vec<usize>,
        mut pick: impl FnMut(usize) -> Option<usize>,
    ) -> Result<(), LineworkError> {
        found.clear();
        let ([x0, y0], [x1, y1]) = (self.at(a.min(b) - DVec2::splat(tol)), self.at(a.max(b) + DVec2::splat(tol)));
        for y in y0..=y1 {
            for x in x0..=x1 {
                let Some(cell) = self.cells.get(y * self.nx + x) else { continue };
                work.add(cell.len() + 1)?;
                found.extend(cell.iter().filter_map(|&k| pick(k)));
            }
        }
        found.sort_unstable();
        Ok(())
    }
}

/// The parts of `a`–`b` (as parameters along it) strictly inside `poly`
/// and farther than `tol` from its outline, added to `hidden`.
fn hidden_by(
    a: DVec2,
    b: DVec2,
    poly: &[DVec2],
    tol: f64,
    ts: &mut Vec<f64>,
    hidden: &mut Vec<(f64, f64)>,
    work: &mut Work,
) -> Result<(), LineworkError> {
    let d = b - a;
    let len2 = d.length_squared();
    if len2 <= 0.0 {
        return Ok(());
    }
    let n = poly.len();
    work.add(n)?;
    // Where the line crosses the outline, and where a corner lies on it.
    ts.clear();
    ts.extend([0.0, 1.0]);
    for k in 0..n {
        let (Some(&p), Some(&q)) = (poly.get(k), poly.get((k + 1) % n)) else { continue };
        let tp = (p - a).dot(d) / len2;
        if tp > 0.0 && tp < 1.0 && p.distance(a + d * tp) <= tol {
            ts.push(tp);
        }
        let e = q - p;
        let denom = d.perp_dot(e);
        if denom != 0.0 {
            let w = p - a;
            let (t, s) = (w.perp_dot(e) / denom, w.perp_dot(d) / denom);
            if t.is_finite() && s.is_finite() && t > 0.0 && t < 1.0 && (0.0..=1.0).contains(&s) {
                ts.push(t);
            }
        }
    }
    ts.sort_by(f64::total_cmp);
    ts.dedup();
    work.add(2 * n * ts.len())?;
    // Each piece between those points is wholly in or out: its middle tells.
    for w in ts.windows(2) {
        let (t0, t1) = (w[0], w[1]);
        if t1 <= t0 {
            continue;
        }
        let m = a + d * ((t0 + t1) * 0.5);
        if cc_geom::polygon::contains(poly, m) && boundary_distance(poly, m) > tol {
            hidden.push((t0, t1));
        }
    }
    Ok(())
}

/// The distance from `p` to the outline of `poly`.
fn boundary_distance(poly: &[DVec2], p: DVec2) -> f64 {
    let n = poly.len();
    (0..n)
        .filter_map(|k| Some((*poly.get(k)?, *poly.get((k + 1) % n)?)))
        .map(|(a, b)| cc_geom::polygon::closest_on_segment(a, b, p).0.distance(p))
        .fold(f64::INFINITY, f64::min)
}

/// The parts of 0..1 outside the `hidden` intervals, those apart by less
/// than `gap` joined and those shorter than it dropped.
fn complement(hidden: &mut [(f64, f64)], gap: f64) -> Vec<(f64, f64)> {
    hidden.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.1.total_cmp(&y.1)));
    let mut visible = Vec::new();
    let mut from = 0.0;
    for &(t0, t1) in hidden.iter() {
        if t0 - from > gap {
            visible.push((from, t0));
        }
        if t1 > from {
            from = t1;
        }
    }
    if 1.0 - from > gap {
        visible.push((from, 1.0));
    }
    visible
}

/// What lines must share to merge: the layer, the color, the dash pattern
/// (as bits) and the style.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    layer: String,
    color: [u8; 3],
    dashes: Vec<u64>,
    style: u8,
}

impl Key {
    fn of(prim: &Prim, layer: String) -> Self {
        let (style, pen) = match prim {
            Prim::Line { style, pen, .. } => (*style, pen.as_ref()),
            _ => (LineStyle::Thin, None),
        };
        let resolved = style.resolve(pen);
        Key {
            layer,
            color: resolved.color,
            dashes: resolved.dashes.iter().map(|d| d.to_bits()).collect(),
            style: style as u8,
        }
    }
}

struct Group {
    layer: String,
    style: LineStyle,
    pen: Option<Pen>,
    segs: Vec<Seg>,
}

/// A visible piece of a line; `order` is its place among its group's.
#[derive(Debug, Clone, Copy)]
struct Seg {
    a: DVec2,
    b: DVec2,
    order: usize,
    element: Option<ElementId>,
}

/// The segments with duplicates and collinear pieces that overlap or touch
/// joined into one (ends taken from the pieces), in a stable order.
fn merge_collinear(segs: Vec<Seg>, tol: f64) -> Vec<Seg> {
    // Each segment's direction, turned to point right (or up), and its angle.
    let mut lines: Vec<(f64, DVec2, Seg)> = segs
        .into_iter()
        .filter_map(|s| {
            let d = s.b - s.a;
            let len = d.length();
            if len < tol || !len.is_finite() {
                return None;
            }
            let mut u = d / len;
            if u.x < 0.0 || (u.x == 0.0 && u.y < 0.0) {
                u = -u;
            }
            Some((u.y.atan2(u.x), u, s))
        })
        .collect();
    lines.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.2.order.cmp(&y.2.order)));
    // Runs of the same direction (the last run joins the first across a half turn).
    let mut runs: Vec<Vec<usize>> = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        match runs.last_mut() {
            Some(run) if run.last().and_then(|&j| lines.get(j)).is_some_and(|p| l.0 - p.0 <= ANGLE_TOLERANCE) => {
                run.push(i)
            }
            _ => runs.push(vec![i]),
        }
    }
    if runs.len() > 1
        && let (Some(first), Some(last)) = (lines.first(), lines.last())
        && first.0 + std::f64::consts::PI - last.0 <= ANGLE_TOLERANCE
        && let Some(tail) = runs.pop()
        && let Some(head) = runs.first_mut()
    {
        head.extend(tail);
    }
    let mut out = Vec::new();
    for run in runs {
        let Some(r) = run.first().and_then(|&i| lines.get(i)).map(|l| l.1) else { continue };
        // Offset of each segment's line from the origin, and its span along `r`.
        let mut placed: Vec<(f64, Seg)> =
            run.iter().filter_map(|&i| lines.get(i)).map(|(_, _, s)| (r.perp_dot((s.a + s.b) * 0.5), *s)).collect();
        placed.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.1.order.cmp(&y.1.order)));
        let mut start = 0;
        while start < placed.len() {
            let mut end = start + 1;
            while end < placed.len() && placed[end].0 - placed[end - 1].0 <= tol {
                end += 1;
            }
            let mut spans: Vec<(f64, DVec2, f64, DVec2, Seg)> = placed[start..end]
                .iter()
                .map(|(_, s)| {
                    let (ta, tb) = (r.dot(s.a), r.dot(s.b));
                    if ta <= tb { (ta, s.a, tb, s.b, *s) } else { (tb, s.b, ta, s.a, *s) }
                })
                .collect();
            spans.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.4.order.cmp(&y.4.order)));
            let mut current: Option<(f64, DVec2, f64, DVec2, Seg)> = None;
            for span in spans {
                current = match current {
                    Some(mut c) if span.0 <= c.2 + tol => {
                        if span.2 > c.2 {
                            (c.2, c.3) = (span.2, span.3);
                        }
                        if span.4.order < c.4.order {
                            c.4 = span.4;
                        }
                        Some(c)
                    }
                    other => {
                        if let Some(c) = other {
                            out.push(Seg { a: c.1, b: c.3, ..c.4 });
                        }
                        Some(span)
                    }
                };
            }
            if let Some(c) = current {
                out.push(Seg { a: c.1, b: c.3, ..c.4 });
            }
            start = end;
        }
    }
    out.sort_by_key(|s| s.order);
    out
}

/// The segments chained into polylines through the points where exactly two
/// ends meet (within `tol`): open runs from their ends, then closed loops;
/// each with its points, whether it is closed, and its first segment's element.
fn chain(segs: &[Seg], tol: f64) -> Vec<(Vec<DVec2>, bool, Option<ElementId>)> {
    // The ends, 2k and 2k + 1 for segment k, gathered into points.
    let ends: Vec<DVec2> = segs.iter().flat_map(|s| [s.a, s.b]).collect();
    let mut parent: Vec<usize> = (0..ends.len()).collect();
    fn root(parent: &mut [usize], mut i: usize) -> usize {
        let mut steps = 0;
        while let Some(&p) = parent.get(i) {
            if p == i || steps > parent.len() {
                break;
            }
            i = p;
            steps += 1;
        }
        i
    }
    let cell = |p: DVec2| ((p.x / tol).floor() as i64, (p.y / tol).floor() as i64);
    let mut cells: BTreeMap<(i64, i64), Vec<usize>> = BTreeMap::new();
    for (i, p) in ends.iter().enumerate() {
        let (cx, cy) = cell(*p);
        for dx in -1..=1i64 {
            for dy in -1..=1i64 {
                let Some(near) = cells.get(&(cx.saturating_add(dx), cy.saturating_add(dy))) else { continue };
                for &j in near {
                    if ends.get(j).is_some_and(|q| q.distance(*p) <= tol) {
                        // The lower end is the root: its point stands for all.
                        let (ri, rj) = (root(&mut parent, i), root(&mut parent, j));
                        let (lo, hi) = (ri.min(rj), ri.max(rj));
                        if let Some(slot) = parent.get_mut(hi) {
                            *slot = lo;
                        }
                    }
                }
            }
        }
        cells.entry((cx, cy)).or_default().push(i);
    }
    let node: Vec<usize> = (0..ends.len()).map(|i| root(&mut parent, i)).collect();
    // The ends at each point (a segment from a point to itself takes no part).
    let mut at: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (k, _) in segs.iter().enumerate() {
        let (Some(&n0), Some(&n1)) = (node.get(2 * k), node.get(2 * k + 1)) else { continue };
        if n0 != n1 {
            at.entry(n0).or_default().push(2 * k);
            at.entry(n1).or_default().push(2 * k + 1);
        }
    }
    let through = |n: usize| at.get(&n).is_some_and(|e| e.len() == 2);
    let point = |n: usize| ends.get(n).copied().unwrap_or_default();
    let mut used = vec![false; segs.len()];
    let mut out = Vec::new();
    // Walks from end `e` of its segment along the chain.
    let walk = |e: usize, used: &mut Vec<bool>| -> (Vec<DVec2>, bool) {
        let start = node.get(e).copied().unwrap_or(e);
        let mut pts = vec![point(start)];
        let mut e = e;
        loop {
            let k = e / 2;
            if let Some(u) = used.get_mut(k) {
                *u = true;
            }
            let far = e ^ 1;
            let n = node.get(far).copied().unwrap_or(far);
            pts.push(point(n));
            if n == start && pts.len() > 3 {
                pts.pop();
                return (pts, true);
            }
            if !through(n) {
                return (pts, false);
            }
            let Some(next) = at.get(&n).and_then(|list| list.iter().copied().find(|&x| x != far)) else {
                return (pts, false);
            };
            if used.get(next / 2).copied().unwrap_or(true) {
                return (pts, false);
            }
            e = next;
        }
    };
    let element = |e: usize| segs.get(e / 2).and_then(|s| s.element);
    // Open runs, from a point that is not passed through.
    for k in 0..segs.len() {
        if used.get(k).copied().unwrap_or(true) {
            continue;
        }
        if node.get(2 * k) == node.get(2 * k + 1) {
            // A segment from a point to itself (ends gathered through others): alone.
            if let (Some(s), Some(u)) = (segs.get(k), used.get_mut(k)) {
                *u = true;
                out.push((vec![s.a, s.b], false, s.element));
            }
            continue;
        }
        for e in [2 * k, 2 * k + 1] {
            let n = node.get(e).copied().unwrap_or(e);
            if !used.get(k).copied().unwrap_or(true) && !through(n) {
                let (pts, closed) = walk(e, &mut used);
                out.push((pts, closed, element(e)));
            }
        }
    }
    // What is left is closed loops.
    for k in 0..segs.len() {
        if !used.get(k).copied().unwrap_or(true) {
            let (pts, closed) = walk(2 * k, &mut used);
            out.push((pts, closed, element(2 * k)));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(a: [f64; 2], b: [f64; 2]) -> Prim {
        Prim::Line { a: DVec2::from(a), b: DVec2::from(b), style: LineStyle::Projection, element: None, pen: None }
    }

    fn fill(pts: &[[f64; 2]], alpha: u8) -> Prim {
        Prim::Fill {
            pts: pts.iter().map(|p| DVec2::from(*p)).collect(),
            color: [255, 255, 255, alpha],
            element: None,
            cut: false,
        }
    }

    fn square(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<[f64; 2]> {
        vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]]
    }

    fn run(prims: Vec<Prim>) -> Linework {
        let d = Drawing2D { prims, ..Default::default() };
        clean_with(&d, &|_| "0".into())
    }

    fn clean_with(d: &Drawing2D, layer: &dyn Fn(&Prim) -> String) -> Linework {
        let skip = BTreeSet::new();
        clean(d, &Options { layer, view_owned: &|_| false, skip: &skip }).unwrap()
    }

    /// The lines as segments, each from its lower end, sorted.
    fn segments(l: &Linework) -> Vec<[f64; 4]> {
        let mut out = Vec::new();
        for p in &l.lines {
            let n = p.pts.len();
            let count = if p.closed { n } else { n - 1 };
            for i in 0..count {
                let (a, b) = (p.pts[i], p.pts[(i + 1) % n]);
                let (a, b) = if (a.x, a.y) <= (b.x, b.y) { (a, b) } else { (b, a) };
                out.push([a.x, a.y, b.x, b.y].map(|v| (v * 1e6).round() / 1e6));
            }
        }
        out.sort_by(|x, y| x.partial_cmp(y).unwrap());
        out
    }

    #[test]
    fn a_later_opaque_fill_cuts_a_line_at_its_edge() {
        let l = run(vec![line([0.0, 0.5], [2.0, 0.5]), fill(&square(1.0, 0.0, 3.0, 1.0), 255)]);
        assert_eq!(segments(&l), vec![[0.0, 0.5, 1.0, 0.5]]);
        assert!(l.prims.is_empty(), "the surface fill is left out");
    }

    #[test]
    fn earlier_and_see_through_fills_hide_nothing() {
        let earlier = run(vec![fill(&square(1.0, 0.0, 3.0, 1.0), 255), line([0.0, 0.5], [2.0, 0.5])]);
        assert_eq!(segments(&earlier), vec![[0.0, 0.5, 2.0, 0.5]]);
        let glass = run(vec![line([0.0, 0.5], [2.0, 0.5]), fill(&square(1.0, 0.0, 3.0, 1.0), 254)]);
        assert_eq!(segments(&glass), vec![[0.0, 0.5, 2.0, 0.5]]);
    }

    #[test]
    fn lines_on_a_fills_outline_stay() {
        // Along the bottom edge, and the edge of a face drawn after the next face.
        let l = run(vec![
            line([0.0, 0.0], [2.0, 0.0]),
            line([1.0, -1.0], [1.0, 1.0]),
            fill(&square(1.0, 0.0, 3.0, 1.0), 255),
        ]);
        assert_eq!(segments(&l), vec![[0.0, 0.0, 2.0, 0.0], [1.0, -1.0, 1.0, 1.0]]);
    }

    #[test]
    fn a_concave_fill_leaves_two_pieces() {
        // A U open at the top: the line crosses both arms and shows in the notch.
        let u = [[0.0, 0.0], [3.0, 0.0], [3.0, 2.0], [2.0, 2.0], [2.0, 1.0], [1.0, 1.0], [1.0, 2.0], [0.0, 2.0]];
        let l = run(vec![line([-1.0, 1.5], [4.0, 1.5]), fill(&u, 255)]);
        assert_eq!(segments(&l), vec![[-1.0, 1.5, 0.0, 1.5], [1.0, 1.5, 2.0, 1.5], [3.0, 1.5, 4.0, 1.5]]);
    }

    #[test]
    fn cut_and_view_owned_fills_are_kept() {
        let owned = ElementId(7);
        let mut cut = fill(&square(0.0, 0.0, 1.0, 1.0), 255);
        if let Prim::Fill { cut: c, .. } = &mut cut {
            *c = true;
        }
        let mut region = fill(&square(2.0, 0.0, 3.0, 1.0), 255);
        if let Prim::Fill { element, .. } = &mut region {
            *element = Some(owned);
        }
        let surface = fill(&square(4.0, 0.0, 5.0, 1.0), 255);
        let d = Drawing2D { prims: vec![cut.clone(), region.clone(), surface], ..Default::default() };
        let skip = BTreeSet::new();
        let options = Options { layer: &|_| "0".into(), view_owned: &|e| e == owned, skip: &skip };
        assert_eq!(clean(&d, &options).unwrap().prims, vec![cut, region]);
    }

    #[test]
    fn skipped_elements_hide_but_are_left_out() {
        let block = ElementId(3);
        let mut cover = fill(&square(1.0, 0.0, 3.0, 1.0), 255);
        if let Prim::Fill { element, .. } = &mut cover {
            *element = Some(block);
        }
        let mut own_line = line([1.0, 0.0], [1.0, 1.0]);
        if let Prim::Line { element, .. } = &mut own_line {
            *element = Some(block);
        }
        let d = Drawing2D { prims: vec![line([0.0, 0.5], [2.0, 0.5]), cover, own_line], ..Default::default() };
        let skip = BTreeSet::from([block]);
        let l = clean(&d, &Options { layer: &|_| "0".into(), view_owned: &|_| true, skip: &skip }).unwrap();
        assert_eq!(segments(&l), vec![[0.0, 0.5, 1.0, 0.5]]);
        assert!(l.prims.is_empty());
    }

    #[test]
    fn duplicates_and_collinear_pieces_merge() {
        let l = run(vec![
            line([0.0, 0.0], [1.0, 0.0]),
            line([1.0, 0.0], [0.0, 0.0]),
            line([0.5, 0.0], [2.0, 0.0]),
            line([2.0, 0.0], [3.0, 0.0]),
            line([5.0, 0.0], [6.0, 0.0]),
            line([0.0, 1.0], [0.0, 1.0 + 1e-9]),
        ]);
        assert_eq!(segments(&l), vec![[0.0, 0.0, 3.0, 0.0], [5.0, 0.0, 6.0, 0.0]], "the tiny one is dropped");
        assert_eq!(l.lines.len(), 2);
        assert!(l.lines.iter().all(|p| p.pts.len() == 2 && !p.closed));
        // Vertical lines, either way up, merge too.
        let l = run(vec![line([0.0, 0.0], [0.0, 1.0]), line([0.0, 2.0], [0.0, 1.0])]);
        assert_eq!(segments(&l), vec![[0.0, 0.0, 0.0, 2.0]]);
    }

    #[test]
    fn a_square_of_four_segments_is_one_closed_polyline() {
        let l = run(vec![
            line([0.0, 0.0], [1.0, 0.0]),
            line([1.0, 1.0], [1.0, 0.0]),
            line([1.0, 1.0], [0.0, 1.0]),
            line([0.0, 0.0], [0.0, 1.0]),
        ]);
        assert_eq!(l.lines.len(), 1);
        assert!(l.lines[0].closed);
        assert_eq!(l.lines[0].pts.len(), 4);
        // An open L is one polyline of three points.
        let l = run(vec![line([0.0, 0.0], [1.0, 0.0]), line([1.0, 1.0], [1.0, 0.0])]);
        assert_eq!((l.lines.len(), l.lines[0].pts.len(), l.lines[0].closed), (1, 3, false));
    }

    #[test]
    fn layers_and_styles_do_not_merge() {
        let mut dashed = line([1.0, 0.0], [2.0, 0.0]);
        if let Prim::Line { style, .. } = &mut dashed {
            *style = LineStyle::Hidden;
        }
        let l = run(vec![line([0.0, 0.0], [1.0, 0.0]), dashed]);
        assert_eq!(l.lines.len(), 2);
        let d =
            Drawing2D { prims: vec![line([0.0, 0.0], [1.0, 0.0]), line([0.0, 0.0], [1.0, 0.0])], ..Default::default() };
        let first = std::cell::Cell::new(true);
        let l = clean_with(&d, &|_| if first.replace(false) { "A".into() } else { "B".into() });
        assert_eq!(l.lines.iter().map(|p| p.layer.as_str()).collect::<Vec<_>>(), vec!["A", "B"]);
    }

    #[test]
    fn three_ends_at_a_point_do_not_chain_through() {
        let l = run(vec![line([0.0, 0.0], [1.0, 0.0]), line([1.0, 0.0], [2.0, 1.0]), line([1.0, 0.0], [1.0, 1.0])]);
        assert_eq!(l.lines.len(), 3);
        assert!(l.lines.iter().all(|p| p.pts.len() == 2));
    }

    #[test]
    fn passing_a_limit_is_an_error() {
        let mut prims: Vec<Prim> =
            (0..100).map(|i| line([-2.0, f64::from(i) * 0.01], [2.0, f64::from(i) * 0.01])).collect();
        prims.push(fill(&square(-1.0, -1.0, 1.0, 2.0), 255));
        let d = Drawing2D { prims, ..Default::default() };
        let skip = BTreeSet::new();
        let options = Options { layer: &|_| "0".into(), view_owned: &|_| false, skip: &skip };
        assert_eq!(clean_within(&d, &options, 500, MAX_SEGMENTS), Err(LineworkError::TooMuchWork));
        assert_eq!(clean_within(&d, &options, MAX_WORK, 150), Err(LineworkError::TooManySegments));
        let l = clean(&d, &options).unwrap();
        assert_eq!(l.lines.len(), 200, "each line in two pieces, either side of the fill");
    }

    #[test]
    fn the_same_drawing_cleans_the_same_way() {
        let mut prims = Vec::new();
        for i in 0..40 {
            let x = f64::from(i) * 0.3;
            prims.push(line([x, 0.0], [x + 1.0, 2.0]));
            prims.push(line([x, 1.0], [x + 2.0, 1.0]));
            if i % 3 == 0 {
                prims.push(fill(&square(x, 0.2, x + 0.5, 1.5), 255));
            }
        }
        let a = run(prims.clone());
        let b = run(prims);
        assert_eq!(a, b);
        assert!(!a.lines.is_empty());
    }

    #[test]
    fn lines_that_are_not_finite_pass_through() {
        let l = run(vec![line([f64::NAN, 0.0], [1.0, 0.0]), fill(&square(-1.0, -1.0, 2.0, 1.0), 255)]);
        assert_eq!(l.prims.len(), 1);
        assert!(l.lines.is_empty());
    }
}
