//! Hidden lines of plans. A plan sees what lies below its cut plane from
//! above: an edge shows only where no face lies over it, so the faces of
//! the pieces turned up below the cut plane, and their sections at it,
//! hide the parts of the edges they cover from above.
//!
//! The test is exact for planar faces: an edge is cut where it crosses the
//! outline of a face seen from above, and along each stretch inside the
//! outline the height of the face over the edge varies linearly, so the
//! stretch is hidden from where the face rises above the edge. A margin
//! (a line's width on paper) also hides what runs that close along the
//! outline of a face over it, which would only thicken that outline.

use crate::mesh::UP;
use crate::solid::Face;
use glam::{DVec2, DVec3};

/// A face must be this much higher (m) than an edge to hide it: faces
/// through the edge (those it bounds) and level with it do not.
const DEPTH_TOL: f64 = 1e-5;
/// A face whose points lie this close (m) to its plane is flat.
const PLANAR_TOL: f64 = 1e-7;
/// Most points of a stretch of an edge tested against the margin.
const MAX_MARGIN_STEPS: usize = 16;
/// Most edge crossings a cover tests in all; past it the edges it is
/// asked about show whole (as without a cover).
const MAX_TESTS: usize = 8_000_000;
/// Most cells along each side of the grid that finds the faces near an edge.
const MAX_CELLS: usize = 128;
/// A face over more cells than this is tested against every edge.
const MAX_SPAN: usize = 64;
/// Most faces a cover holds; later ones hide nothing.
const MAX_FACES: usize = 200_000;

/// A region seen from above at a height: a face below the cut plane, a
/// prism's top or a section at the cut plane.
#[derive(Debug, Clone)]
struct Occluder {
    /// Its outline in plan, as segments (holes too: inside is where a ray
    /// crosses them an odd number of times).
    edges: Vec<(DVec2, DVec2)>,
    lo: DVec2,
    hi: DVec2,
    /// Height at `origin`, rising by `slope` per meter in plan.
    origin: DVec2,
    z: f64,
    slope: DVec2,
    /// Its highest point.
    top: f64,
    /// The piece it belongs to (see [`PlanCover::begin_piece`]).
    piece: usize,
}

impl Occluder {
    fn new(edges: Vec<(DVec2, DVec2)>, origin: DVec2, z: f64, slope: DVec2, top: f64) -> Option<Self> {
        let first = edges.first()?.0;
        let (lo, hi) = edges.iter().fold((first, first), |(lo, hi), (a, b)| (lo.min(*a).min(*b), hi.max(*a).max(*b)));
        let finite = lo.is_finite() && hi.is_finite() && origin.is_finite() && slope.is_finite();
        (finite && z.is_finite() && top.is_finite()).then_some(Self { edges, lo, hi, origin, z, slope, top, piece: 0 })
    }

    fn height(&self, p: DVec2) -> f64 {
        self.z + self.slope.dot(p - self.origin)
    }

    /// The height of the region at its point nearest to `p`, when that is
    /// within `margin` of it (`p` itself when inside).
    fn height_near(&self, p: DVec2, margin: f64) -> Option<f64> {
        if self.contains(p) {
            return Some(self.height(p));
        }
        let nearest = self.edges.iter().map(|(a, b)| {
            let ab = *b - *a;
            let t = ((p - *a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0);
            *a + ab * if t.is_finite() { t } else { 0.0 }
        });
        let q = nearest.min_by(|x, y| x.distance_squared(p).total_cmp(&y.distance_squared(p)))?;
        (q.distance(p) <= margin).then(|| self.height(q))
    }

    /// Whether `p` is inside the outline (even-odd).
    fn contains(&self, p: DVec2) -> bool {
        let mut inside = false;
        for (a, b) in &self.edges {
            if (a.y > p.y) != (b.y > p.y) {
                let x = a.x + (p.y - a.y) * (b.x - a.x) / (b.y - a.y);
                if p.x < x {
                    inside = !inside;
                }
            }
        }
        inside
    }

    /// The stretches of `a..b` (parameters along it) this region covers
    /// from above, `za`/`zb` being the heights of its ends.
    fn covers(&self, (a, b): (DVec2, DVec2), (za, zb): (f64, f64), out: &mut Vec<(f64, f64)>) {
        let d = b - a;
        let mut ts = vec![0.0, 1.0];
        for (p, q) in &self.edges {
            let e = *q - *p;
            let denom = d.perp_dot(e);
            if denom.abs() < 1e-18 {
                continue;
            }
            let w = *p - a;
            let (t, u) = (w.perp_dot(e) / denom, w.perp_dot(d) / denom);
            if t > 0.0 && t < 1.0 && (-1e-12..=1.0 + 1e-12).contains(&u) {
                ts.push(t);
            }
        }
        ts.sort_by(f64::total_cmp);
        let above = |t: f64| self.height(a + d * t) - (za + (zb - za) * t) - DEPTH_TOL;
        for w in ts.windows(2) {
            let (t0, t1) = (w[0], w[1]);
            if t1 - t0 < 1e-12 || !self.contains(a + d * ((t0 + t1) * 0.5)) {
                continue;
            }
            // The face's height over the edge varies linearly along it.
            let (h0, h1) = (above(t0), above(t1));
            let stretch = match (h0 > 0.0, h1 > 0.0) {
                (true, true) => (t0, t1),
                (false, false) => continue,
                (true, false) => (t0, t0 + (t1 - t0) * h0 / (h0 - h1)),
                (false, true) => (t0 + (t1 - t0) * h0 / (h0 - h1), t1),
            };
            if stretch.0.is_finite() && stretch.1.is_finite() {
                out.push(stretch);
            }
        }
    }
}

/// A uniform grid over the faces' boxes, to find those near an edge.
#[derive(Debug, Clone, Default)]
struct Grid {
    lo: DVec2,
    cell: DVec2,
    side: usize,
    cells: Vec<Vec<u32>>,
    /// Faces over too many cells, tested against every edge.
    wide: Vec<u32>,
}

impl Grid {
    fn new(faces: &[Occluder]) -> Self {
        let Some(first) = faces.first() else { return Self::default() };
        let (lo, hi) = faces.iter().fold((first.lo, first.hi), |(lo, hi), f| (lo.min(f.lo), hi.max(f.hi)));
        let side = ((faces.len() as f64).sqrt().ceil() as usize).clamp(1, MAX_CELLS);
        let size = (hi - lo) / side as f64;
        let cell = DVec2::new(size.x.max(1e-6), size.y.max(1e-6));
        let mut grid = Self { lo, cell, side, cells: vec![Vec::new(); side * side], wide: Vec::new() };
        for (i, f) in faces.iter().enumerate() {
            let ((x0, y0), (x1, y1)) = (grid.index(f.lo), grid.index(f.hi));
            if (x1 - x0 + 1) * (y1 - y0 + 1) > MAX_SPAN {
                grid.wide.push(i as u32);
                continue;
            }
            for y in y0..=y1 {
                for x in x0..=x1 {
                    if let Some(c) = grid.cells.get_mut(y * side + x) {
                        c.push(i as u32);
                    }
                }
            }
        }
        grid
    }

    /// The cell holding `p`, clamped to the grid.
    fn index(&self, p: DVec2) -> (usize, usize) {
        let at = |v: f64, lo: f64, cell: f64| {
            let k = ((v - lo) / cell).floor();
            if k.is_finite() { (k.max(0.0) as usize).min(self.side.saturating_sub(1)) } else { 0 }
        };
        (at(p.x, self.lo.x, self.cell.x), at(p.y, self.lo.y, self.cell.y))
    }
}

/// What hides edges in a plan cut at `cut_z` (see the module): faces and
/// sections added, then edges asked about with [`PlanCover::visible`].
#[derive(Debug, Clone)]
pub struct PlanCover {
    cut_z: f64,
    faces: Vec<Occluder>,
    grid: Option<Grid>,
    /// Edge crossings tested so far (see [`MAX_TESTS`]).
    tests: usize,
    /// The query that last met each face (each is tested once per edge).
    seen: Vec<u32>,
    query: u32,
    /// See [`PlanCover::with_margin`].
    margin: f64,
    /// The piece faces are added to (see [`PlanCover::begin_piece`]).
    piece: usize,
}

impl PlanCover {
    pub fn new(cut_z: f64) -> Self {
        Self { cut_z, faces: Vec::new(), grid: None, tests: 0, seen: Vec::new(), query: 0, margin: 0.0, piece: 0 }
    }

    /// Hide too the stretches of edges that run within `margin` (in plan)
    /// of a face of another piece lying over them all along: a line that
    /// close to the outline of what covers it merges with it on paper, and
    /// two outlines of one curve drawn with different chords would
    /// otherwise leave the slivers of the lower one between the chords of
    /// the upper one. A piece's own faces never hide its edges so (its
    /// surface rising from its outline is no other outline).
    pub fn with_margin(mut self, margin: f64) -> Self {
        self.margin = if margin.is_finite() { margin.max(0.0) } else { 0.0 };
        self
    }

    /// Start a new piece: the faces added from now on belong to it. Returns
    /// its number, to give [`Self::visible`] with the piece's own edges.
    pub fn begin_piece(&mut self) -> usize {
        self.piece = self.piece.wrapping_add(1);
        self.piece
    }

    pub fn is_empty(&self) -> bool {
        self.faces.is_empty()
    }

    fn push(&mut self, face: Option<Occluder>) {
        if let Some(mut face) = face
            && self.faces.len() < MAX_FACES
        {
            face.piece = self.piece;
            self.faces.push(face);
            self.grid = None;
        }
    }

    /// The faces of a closed piece: those turned up hide what is under
    /// them, their parts below the cut plane. A face that is not flat (a
    /// blend's twisted side) hides as its triangles, each in its own plane,
    /// so it never hides its own edges.
    pub fn add_faces(&mut self, faces: &[Vec<DVec3>]) {
        for verts in faces {
            let Some(face) = Face::of_polygon(verts.clone()) else { continue };
            if face.normal.z <= UP {
                continue;
            }
            let Some(&first) = verts.first() else { continue };
            if verts.iter().all(|v| (*v - first).dot(face.normal).abs() < PLANAR_TOL) {
                self.add_flat(face);
                continue;
            }
            for [i, j, k] in face.triangulate() {
                let (Some(&a), Some(&b), Some(&c)) = (verts.get(i), verts.get(j), verts.get(k)) else { continue };
                if let Some(t) = Face::of_polygon(vec![a, b, c]).filter(|t| t.normal.z > UP) {
                    self.add_flat(t);
                }
            }
        }
    }

    /// A flat face turned up: its part below the cut plane.
    fn add_flat(&mut self, face: Face) {
        let n = face.normal;
        let below = crate::clip_polygon(&face.verts, DVec3::new(0.0, 0.0, self.cut_z), -DVec3::Z);
        let Some(&p) = below.first() else { return };
        let flat: Vec<DVec2> = below.iter().map(|v| v.truncate()).collect();
        if below.len() < 3 || crate::polygon::signed_area(&flat).abs() < 1e-12 {
            return;
        }
        let top = below.iter().map(|v| v.z).fold(f64::NEG_INFINITY, f64::max);
        let slope = -DVec2::new(n.x, n.y) / n.z;
        self.push(Occluder::new(ring(&flat), p.truncate(), p.z, slope, top));
    }

    /// A piece's section at the cut plane (its segments, in any order):
    /// it hides everything under it. Left out unless the segments close.
    pub fn add_section(&mut self, segments: &[(DVec2, DVec2)]) {
        let mut ends: std::collections::HashMap<(i64, i64), usize> = std::collections::HashMap::new();
        let key = |p: DVec2| ((p.x * 1e6).round() as i64, (p.y * 1e6).round() as i64);
        for (a, b) in segments {
            *ends.entry(key(*a)).or_default() += 1;
            *ends.entry(key(*b)).or_default() += 1;
        }
        if segments.is_empty() || ends.values().any(|n| n % 2 != 0) {
            return;
        }
        let z = self.cut_z;
        self.push(Occluder::new(segments.to_vec(), DVec2::ZERO, z, DVec2::ZERO, z));
    }

    /// A level region at height `z` (a prism's top; at the cut plane when
    /// it rises through it).
    pub fn add_outline(&mut self, outline: &[DVec2], z: f64) {
        if outline.len() < 3 || crate::polygon::signed_area(outline).abs() < 1e-12 {
            return;
        }
        let z = z.min(self.cut_z);
        self.push(Occluder::new(ring(outline), DVec2::ZERO, z, DVec2::ZERO, z));
    }

    /// The parts of the edge `a..b` of piece `own` (see
    /// [`Self::begin_piece`]) seen from above (in plan), in order from `a`:
    /// none where a face lies over it (or, of another piece, within the
    /// margin of it all along, see [`Self::with_margin`]).
    pub fn visible(&mut self, a: DVec3, b: DVec3, own: Option<usize>) -> Vec<(DVec2, DVec2)> {
        let (a2, b2) = (a.truncate(), b.truncate());
        let whole = vec![(a2, b2)];
        if self.faces.is_empty() || !a.is_finite() || !b.is_finite() || a2.distance_squared(b2) < 1e-18 {
            return whole;
        }
        if self.tests >= MAX_TESTS {
            return whole;
        }
        let grid = match self.grid.take() {
            Some(grid) => grid,
            None => Grid::new(&self.faces),
        };
        self.query = self.query.wrapping_add(1);
        if self.query == 0 || self.seen.len() != self.faces.len() {
            self.seen = vec![0; self.faces.len()];
            self.query = 1;
        }
        let reach = DVec2::splat(self.margin);
        let (lo, hi) = (a2.min(b2) - reach, a2.max(b2) + reach);
        let ((x0, y0), (x1, y1)) = (grid.index(lo), grid.index(hi));
        let mut near: Vec<u32> = grid.wide.clone();
        for y in y0..=y1 {
            for x in x0..=x1 {
                near.extend(grid.cells.get(y * grid.side + x).into_iter().flatten());
            }
        }
        self.grid = Some(grid);
        // The faces that may lie over the edge.
        let low = a.z.min(b.z) + DEPTH_TOL;
        let mut over = Vec::new();
        for i in near {
            let Some(seen) = self.seen.get_mut(i as usize) else { continue };
            if *seen == self.query {
                continue;
            }
            *seen = self.query;
            let Some(face) = self.faces.get(i as usize) else { continue };
            let apart = face.lo.x > hi.x || face.hi.x < lo.x || face.lo.y > hi.y || face.hi.y < lo.y;
            if !apart && face.top > low {
                over.push(i as usize);
            }
        }
        let mut covered = Vec::new();
        for face in over.iter().filter_map(|&i| self.faces.get(i)) {
            self.tests += face.edges.len();
            if self.tests >= MAX_TESTS {
                return whole;
            }
            face.covers((a2, b2), (a.z, b.z), &mut covered);
        }
        covered.sort_by(|x, y| x.0.total_cmp(&y.0));
        let mut shown = Vec::new();
        let mut t = 0.0;
        for (s0, s1) in covered {
            if s0 > t + 1e-9 {
                shown.push((t, s0));
            }
            t = t.max(s1);
        }
        if t < 1.0 - 1e-9 {
            shown.push((t, 1.0));
        }
        if self.margin > 0.0 && !over.is_empty() {
            over.retain(|&i| self.faces.get(i).is_some_and(|f| Some(f.piece) != own));
            shown.retain(|&stretch| !self.hugs(&over, (a, b), stretch));
        }
        shown.into_iter().map(|(t0, t1)| (a2.lerp(b2, t0), a2.lerp(b2, t1))).collect()
    }

    /// Whether the stretch `t0..t1` of the edge `a..b` runs within the
    /// margin of faces of `over` higher than it, all along.
    fn hugs(&mut self, over: &[usize], (a, b): (DVec3, DVec3), (t0, t1): (f64, f64)) -> bool {
        let length = a.truncate().distance(b.truncate()) * (t1 - t0);
        let steps = ((length / self.margin).ceil() as usize).clamp(1, MAX_MARGIN_STEPS);
        let sides: usize = over.iter().filter_map(|&i| self.faces.get(i)).map(|f| f.edges.len()).sum();
        self.tests += (steps + 1) * sides;
        if self.tests >= MAX_TESTS {
            return false;
        }
        (0..=steps).all(|k| {
            let t = t0 + (t1 - t0) * k as f64 / steps as f64;
            let (p, z) = (a.truncate().lerp(b.truncate(), t), a.z + (b.z - a.z) * t);
            over.iter()
                .filter_map(|&i| self.faces.get(i))
                .any(|f| f.height_near(p, self.margin).is_some_and(|h| h > z + DEPTH_TOL))
        })
    }
}

/// The sides of a closed outline.
fn ring(outline: &[DVec2]) -> Vec<(DVec2, DVec2)> {
    outline.iter().copied().zip(outline.iter().copied().cycle().skip(1)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Solid;
    use cc_core::{Category, ElementId};

    fn boxed(x: (f64, f64), y: (f64, f64), z: (f64, f64)) -> Vec<Vec<DVec3>> {
        let mut s = Solid::new(ElementId(1), Category::Component, [0; 3]);
        s.add_box(DVec2::ZERO, DVec2::X, DVec2::Y, x.0, x.1, y.0, y.1, z.0, z.1);
        s.faces.into_iter().map(|f| f.verts).collect()
    }

    fn length(parts: &[(DVec2, DVec2)]) -> f64 {
        parts.iter().map(|(a, b)| a.distance(*b)).sum()
    }

    /// A slab over a lower box hides the box's top edges where it lies over
    /// them, wholly and in part; an edge level with or above the slab shows.
    #[test]
    fn faces_above_hide_what_they_cover() {
        let mut cover = PlanCover::new(1.2);
        cover.add_faces(&boxed((0.0, 2.0), (0.0, 1.0), (0.5, 0.6)));
        // Under the slab: hidden.
        let v = |x: f64, y: f64, z: f64| DVec3::new(x, y, z);
        assert!(cover.visible(v(0.5, 0.5, 0.3), v(1.5, 0.5, 0.3), None).is_empty());
        // Half under it: 1 m out of 2 shows, from x = 2 to 3.
        let half = cover.visible(v(1.0, 0.5, 0.3), v(3.0, 0.5, 0.3), None);
        assert!((length(&half) - 1.0).abs() < 1e-9, "{half:?}");
        assert!(half.iter().all(|(a, b)| a.x >= 2.0 - 1e-9 && b.x >= 2.0 - 1e-9));
        // The slab's own edges, and edges over it, show whole.
        assert_eq!(cover.visible(v(0.0, 0.0, 0.6), v(2.0, 0.0, 0.6), None).len(), 1);
        assert!((length(&cover.visible(v(0.5, 0.5, 0.6), v(1.5, 0.5, 0.6), None)) - 1.0).abs() < 1e-9);
        assert!((length(&cover.visible(v(0.5, 0.5, 0.9), v(1.5, 0.5, 0.9), None)) - 1.0).abs() < 1e-9);
    }

    /// An edge rising through a face shows from where it passes over it.
    #[test]
    fn an_edge_through_a_face_shows_above_it() {
        let mut cover = PlanCover::new(5.0);
        cover.add_faces(&boxed((0.0, 4.0), (0.0, 4.0), (0.0, 1.0)));
        // From 0 to 2 m high across the top at 1 m: shown from halfway.
        let parts = cover.visible(DVec3::new(1.0, 2.0, 0.0), DVec3::new(3.0, 2.0, 2.0), None);
        assert_eq!(parts.len(), 1);
        assert!((parts[0].0.x - 2.0).abs() < 1e-4 && (parts[0].1.x - 3.0).abs() < 1e-9, "{parts:?}");
    }

    /// A section at the cut plane hides what is under it, within its
    /// openings nothing; a section that does not close hides nothing.
    #[test]
    fn sections_hide_what_is_under_them() {
        let square = |r: f64| {
            let p = [DVec2::new(-r, -r), DVec2::new(r, -r), DVec2::new(r, r), DVec2::new(-r, r)];
            ring(&p)
        };
        let mut cover = PlanCover::new(1.0);
        let mut ring_cut = square(2.0);
        ring_cut.extend(square(1.0));
        cover.add_section(&ring_cut);
        let parts = cover.visible(DVec3::new(-3.0, 0.0, 0.2), DVec3::new(3.0, 0.0, 0.2), None);
        // Outside (1 m each side) and inside the opening (2 m) show.
        assert!((length(&parts) - 4.0).abs() < 1e-9, "{parts:?}");
        let mut open = PlanCover::new(1.0);
        open.add_section(&square(2.0)[..3]);
        assert!(open.is_empty());
    }

    /// One circle drawn by two pieces with different chords: the lower one
    /// leaves slivers between the chords of the upper one, which a margin
    /// as wide as a line on paper hides; a circle 1 cm outside still shows.
    #[test]
    fn a_margin_merges_lines_with_the_outline_over_them() {
        let circle = |r: f64, n: usize| -> Vec<DVec2> {
            (0..n).map(|i| DVec2::from_angle(std::f64::consts::TAU * i as f64 / n as f64) * r).collect()
        };
        let seen_of = |cover: &mut PlanCover, outline: &[DVec2], own: Option<usize>| -> f64 {
            let at = |p: DVec2| p.extend(0.5);
            (0..outline.len()).map(|i| length(&cover.visible(at(outline[i]), at(outline[(i + 1) % 48]), own))).sum()
        };
        let seen = |cover: &mut PlanCover, outline: &[DVec2]| seen_of(cover, outline, None);
        let lower = circle(0.2, 48);
        let mut bare = PlanCover::new(1.2);
        bare.add_outline(&circle(0.2, 24), 0.8);
        let slivers = seen(&mut bare, &lower);
        assert!(slivers > 0.05, "half the lower chords peek out: {slivers}");
        // At 1:10, a 0.25 mm line is 2.5 mm wide: more than the 1.7 mm the chords leave.
        let mut merged = PlanCover::new(1.2).with_margin(0.0025);
        merged.add_outline(&circle(0.2, 24), 0.8);
        assert_eq!(seen(&mut merged, &lower), 0.0);
        // The margin leaves a piece's own faces out (its surface rising from its outline).
        let own = Some(0);
        assert!((seen_of(&mut merged, &lower, own) - slivers).abs() < 1e-9);
        let outside = circle(0.21, 48);
        let perimeter: f64 = (0..48).map(|i| outside[i].distance(outside[(i + 1) % 48])).sum();
        assert!((seen(&mut merged, &outside) - perimeter).abs() < 1e-9);
    }

    /// A twisted face (not flat) never hides its own edges, and hides what is under it.
    #[test]
    fn twisted_faces_do_not_hide_their_edges() {
        let quad = [
            DVec3::new(0.0, 0.0, 0.0),
            DVec3::new(1.0, 0.0, 0.0),
            DVec3::new(1.0, 1.0, 0.3),
            DVec3::new(0.0, 1.0, 0.0),
        ];
        let mut cover = PlanCover::new(1.0);
        cover.add_faces(&[quad.to_vec()]);
        for i in 0..4 {
            let (a, b) = (quad[i], quad[(i + 1) % 4]);
            let seen = cover.visible(a, b, None);
            assert!((length(&seen) - a.truncate().distance(b.truncate())).abs() < 1e-9, "{i}: {seen:?}");
        }
        assert!(cover.visible(DVec3::new(0.4, 0.5, -0.1), DVec3::new(0.6, 0.5, -0.1), None).is_empty());
    }

    /// Faces turned down, and the parts of faces above the cut plane, hide nothing.
    #[test]
    fn only_faces_turned_up_below_the_cut_hide() {
        let mut cover = PlanCover::new(1.0);
        // A box from 0.5 to 3 m: its top is above the cut plane.
        cover.add_faces(&boxed((0.0, 2.0), (0.0, 2.0), (0.5, 3.0)));
        let parts = cover.visible(DVec3::new(0.5, 1.0, 0.2), DVec3::new(1.5, 1.0, 0.2), None);
        assert!((length(&parts) - 1.0).abs() < 1e-9);
    }
}
