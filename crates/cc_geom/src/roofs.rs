//! Pitched roofs (gable and hip) over any footprint, courtyards included:
//! every sloped eave raises a plane and every gable edge a vertical plane;
//! the weighted straight skeleton ([`crate::skeleton`]) gives the part of
//! each plane over the footprint, with its hips, valleys and ridges.
//!
//! The older construction, kept as a fallback when the skeleton fails,
//! takes the lowest of the planes rising from each sloped eave: right for
//! convex footprints only.
//!
//! The roof's base is the underside at the tips of its eaves when every
//! edge has the roof's `overhang`: each sloped plane passes through its
//! wall line (the footprint edge) at `base + slope * overhang`. An edge
//! with an overhang of its own (`Roof::edge_overhangs`) keeps that plane
//! and moves its eave along it: `slope * (overhang - own)` above the base
//! (below it when it reaches further out). The skeleton starts such an edge
//! late (see [`crate::skeleton::skeleton_with_starts`]) and the base drops
//! to the lowest eave ([`eave_starts`]).

use crate::polygon;
use crate::{Face, Solid};
use glam::{DVec2, DVec3};

/// One plane of a pitched roof: rises with `slope` from the eave through
/// `a` (counter-clockwise footprint, so the inside is on the left).
#[derive(Debug, Clone, Copy)]
pub struct Eave {
    pub a: DVec2,
    pub normal: DVec2,
    pub slope: f64,
}

impl Eave {
    /// Height above the eave of the plane at `p`.
    pub fn height(&self, p: DVec2) -> f64 {
        self.slope * (p - self.a).dot(self.normal)
    }
}

/// Keep the part of `poly` where `p · w <= c`.
pub(crate) fn clip_half_plane(poly: &[DVec2], w: DVec2, c: f64) -> Vec<DVec2> {
    let mut out = Vec::new();
    let n = poly.len();
    for i in 0..n {
        let (p, q) = (poly[i], poly[(i + 1) % n]);
        let (dp, dq) = (p.dot(w) - c, q.dot(w) - c);
        if dp <= 0.0 {
            out.push(p);
        }
        if (dp <= 0.0) != (dq <= 0.0) {
            out.push(p + (q - p) * (dp / (dp - dq)));
        }
    }
    out
}

/// The slope of each edge of a counter-clockwise loop: its own slope when
/// `own` sets one (0 = a gable end), else `slope` on a hip roof (no
/// `ridge`) or on edges within 45° of the ridge direction, 0 on the others.
pub fn edge_slopes(pts: &[DVec2], slope: f64, ridge: Option<DVec2>, own: &[Option<f64>]) -> Vec<f64> {
    let n = pts.len();
    (0..n)
        .map(|i| {
            let s = own.get(i).copied().flatten().unwrap_or_else(|| {
                let dir = (pts[(i + 1) % n] - pts[i]).normalize_or_zero();
                let sloped = ridge.is_none_or(|r| dir.dot(r).abs() >= std::f64::consts::FRAC_1_SQRT_2);
                if sloped { slope } else { 0.0 }
            });
            if s.is_finite() { s.max(0.0) } else { 0.0 }
        })
        .collect()
}

/// A roof's own edge slopes in the order of its counter-clockwise outline
/// (`polygon::ccw` reverses a clockwise footprint, and with it the edges).
pub fn ccw_edge_slopes(r: &cc_core::Roof) -> Vec<Option<f64>> {
    let n = r.footprint.len();
    if n == 0 || polygon::signed_area(&r.footprint) >= 0.0 {
        return (0..n).map(|i| r.edge_slope(i)).collect();
    }
    // Reversed, edge `j` runs from old point `n-1-j` to `n-2-j`: old edge `n-2-j`.
    (0..n).map(|j| r.edge_slope((2 * n - 2 - j) % n)).collect()
}

/// A roof's own edge overhangs in the order of its counter-clockwise
/// outline, as [`ccw_edge_slopes`].
pub fn ccw_edge_overhangs(r: &cc_core::Roof) -> Vec<Option<f64>> {
    (0..r.footprint.len()).map(|j| r.edge_overhang(ccw_source(r, j))).collect()
}

/// The footprint edge (index into `Roof::footprint`) that edge `j` of its
/// counter-clockwise copy comes from.
fn ccw_source(r: &cc_core::Roof, j: usize) -> usize {
    let n = r.footprint.len();
    if n == 0 || polygon::signed_area(&r.footprint) >= 0.0 { j } else { (2 * n - 2 - j) % n }
}

/// A roof's outline, the footprint grown by the overhang of each edge,
/// counter-clockwise, with what belongs to each of its edges.
#[derive(Debug, Clone, PartialEq)]
pub struct RoofOutline {
    pub points: Vec<DVec2>,
    /// Per edge (from point `i` to `i + 1`): the slope the roof sets on it,
    /// like [`ccw_edge_slopes`] (a step between collinear edges of
    /// different overhangs is a gable, `Some(0.0)`).
    pub own: Vec<Option<f64>>,
    /// Per edge: its overhang.
    pub overhangs: Vec<f64>,
    /// Corners where an eave of no length starts late (see
    /// [`crate::skeleton::CornerSeed`]), for the skeleton.
    pub seeds: Vec<crate::skeleton::CornerSeed>,
}

/// Where two edge lines through a corner meet once moved out by `d1` and
/// `d2` along their outward normals `n1` and `n2` (`v · n1 = d1`,
/// `v · n2 = d2`), from the corner; `None` when they are parallel.
fn meet(n1: DVec2, d1: f64, n2: DVec2, d2: f64) -> Option<DVec2> {
    let det = n1.perp_dot(n2);
    (det.abs() > 1e-9).then(|| DVec2::new(d1 * n2.y - d2 * n1.y, n1.x * d2 - n2.x * d1) / det)
}

/// The corners of a pitched roof's counter-clockwise footprint `pts`
/// where a notch cut into a sloped eave runs right up to the corner: a
/// convex corner between two gables lying on the line of a sloped eave
/// (the rule of the skeleton's own corner seeds). Per corner, the edge of
/// that eave: of those that fit, the lowest at its wall line (the most
/// overhang `d`), then the least steep along the corner's bisector.
fn notch_corners(r: &cc_core::Roof, pts: &[DVec2], own: &[Option<f64>], d: &[f64]) -> Vec<Option<usize>> {
    let n = pts.len();
    let (slope, ridge) = match r.shape {
        cc_core::RoofShape::Gable { slope, ridge_deg } => (slope, Some(DVec2::from_angle(ridge_deg.to_radians()))),
        cc_core::RoofShape::Hip { slope } => (slope, None),
        _ => return vec![None; n],
    };
    let slopes = edge_slopes(pts, slope, ridge, own);
    let (lo, hi) = pts
        .iter()
        .fold((DVec2::splat(f64::INFINITY), DVec2::splat(f64::NEG_INFINITY)), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
    let tol = (hi - lo).length() * 1e-9;
    let dir = |j: usize| (pts[(j + 1) % n] - pts[j]).normalize_or_zero();
    (0..n)
        .map(|i| {
            let (a, b) = ((i + n - 1) % n, i);
            let gables = slopes.get(a).is_some_and(|s| *s <= 0.0) && slopes.get(b).is_some_and(|s| *s <= 0.0);
            let (da, db) = (dir(a), dir(b));
            let inward = (da.perp() + db.perp()).normalize_or_zero();
            if !gables || da.perp_dot(db) <= 1e-9 || inward == DVec2::ZERO {
                return None;
            }
            let cur = pts[i];
            let key = |j: usize| {
                let s = slopes.get(j).copied().unwrap_or(0.0);
                (-d.get(j).copied().unwrap_or(0.0), inward.dot(dir(j).perp()) * s)
            };
            (0..n)
                .filter(|&j| {
                    let dj = dir(j);
                    slopes.get(j).is_some_and(|s| *s > 0.0)
                        && dj != DVec2::ZERO
                        && (cur - pts[j]).dot(dj.perp()).abs() <= tol
                        && da.perp_dot(dj) > 1e-9
                        && dj.perp_dot(db) > 1e-9
                        && inward.dot(dj.perp()) > 0.0
                })
                .min_by(|&x, &y| {
                    let (kx, ky) = (key(x), key(y));
                    kx.0.total_cmp(&ky.0).then(kx.1.total_cmp(&ky.1))
                })
        })
        .collect()
}

/// The outline of a roof's slab in plan: its footprint grown by the
/// overhang of each edge (mitered corners, as [`polygon::offset`], which
/// gives it exactly when no edge has an overhang of its own and no corner
/// is cut).
///
/// A corner where a notch cut into a sloped eave runs right up to it (a
/// convex corner between two gables on the eave's line) is grown as if an
/// eave of no length were there, moved out by that eave's overhang: the
/// outline is cut along the eave's moved line, with two points on it where
/// the moved gable lines cross it, and the piece between is a piece of
/// that eave. Where those points come out the other way round (the eave
/// reaches further out than the gables' miter), that piece has no length
/// left: the corner keeps its miter, and the eave's plane starts there
/// late, once its wavefront gets there (a [`crate::skeleton::CornerSeed`]).
pub fn outline(r: &cc_core::Roof) -> RoofOutline {
    let n = r.footprint.len();
    if n < 3 {
        let points = polygon::ccw(&polygon::offset(&r.footprint, r.overhang));
        let m = points.len();
        return RoofOutline { own: ccw_edge_slopes(r), overhangs: vec![r.overhang; m], points, seeds: Vec::new() };
    }
    let pts = polygon::ccw(&r.footprint);
    let d: Vec<f64> = (0..n).map(|j| r.overhang_of(ccw_source(r, j)).max(0.0)).collect();
    let own: Vec<Option<f64>> = (0..n).map(|j| r.edge_slope(ccw_source(r, j))).collect();
    let cuts = notch_corners(r, &pts, &own, &d);
    if !r.has_edge_overhangs() && cuts.iter().all(Option::is_none) {
        let points = polygon::ccw(&polygon::offset(&r.footprint, r.overhang));
        let m = points.len();
        return RoofOutline { own: ccw_edge_slopes(r), overhangs: vec![r.overhang; m], points, seeds: Vec::new() };
    }
    let (lo, hi) = pts
        .iter()
        .fold((DVec2::splat(f64::INFINITY), DVec2::splat(f64::NEG_INFINITY)), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
    let tol = (hi - lo).length() * 1e-9;
    let mut out = RoofOutline { points: Vec::new(), own: Vec::new(), overhangs: Vec::new(), seeds: Vec::new() };
    for i in 0..n {
        let (prev, cur, next) = (pts[(i + n - 1) % n], pts[i], pts[(i + 1) % n]);
        let (d1, d2) = (d[(i + n - 1) % n], d[i]);
        // Outward normals of a counter-clockwise outline point to the right of each edge.
        let n1 = -(cur - prev).normalize_or_zero().perp();
        let n2 = -(next - cur).normalize_or_zero().perp();
        // The eave a notch runs up to this corner along, and where the moved
        // gable lines cross its moved line.
        let cut = cuts.get(i).copied().flatten().and_then(|e| {
            let dir = (pts[(e + 1) % n] - pts[e]).normalize_or_zero();
            let (ne, de) = (-dir.perp(), d.get(e).copied().unwrap_or(0.0));
            Some((e, dir, ne, de, meet(n1, d1, ne, de)?, meet(ne, de, n2, d2)?))
        });
        if let Some(v) = meet(n1, d1, n2, d2) {
            // Where the two moved edge lines meet, pulled in at sharp corners
            // as `polygon::offset` does.
            let limit = 5.0 * d1.max(d2);
            let v = if v.length() > limit && limit > 0.0 { v * (limit / v.length()) } else { v };
            match cut {
                Some((e, dir, _, _, a, b)) if (b - a).dot(dir) > tol => {
                    // A piece of the eave between the two gables.
                    out.points.push(cur + a);
                    out.own.push(own.get(e).copied().flatten());
                    out.overhangs.push(d.get(e).copied().unwrap_or(0.0));
                    out.points.push(cur + b);
                }
                // No length: the corner on the eave's line.
                Some((_, dir, _, _, a, b)) if (b - a).dot(dir) >= -tol => out.points.push(cur + (a + b) * 0.5),
                Some((_, dir, ne, de, _, _)) => {
                    out.points.push(cur + v);
                    out.seeds.push(crate::skeleton::CornerSeed { corner: cur + v, eave: cur + ne * de, dir });
                }
                None => out.points.push(cur + v),
            }
        } else if n1.dot(n2) > 0.0 && (d1 - d2).abs() > 1e-12 {
            // Straight on with another overhang: a step, square to the edge.
            out.points.push(cur + n1 * d1);
            out.own.push(Some(0.0));
            out.overhangs.push(d2);
            out.points.push(cur + n2 * d2);
        } else {
            out.points.push(cur + (n1 * d1 + n2 * d2) * 0.5);
        }
        out.own.push(own.get(i).copied().flatten());
        out.overhangs.push(d2);
    }
    out
}

/// Where each sloped plane of a roof outline starts (see
/// [`crate::skeleton::skeleton_with_starts`]): the height of its eave above
/// the lowest eave, and that lowest eave's height above the roof's base.
/// `slopes` holds the slope of each outline edge (see [`edge_slopes`]) and
/// `overhangs` its overhang; `overhang`, the roof's own, fixes the planes;
/// `openings` says whether opening edges, whose eaves lie at the base, count.
pub fn eave_starts(slopes: &[f64], overhangs: &[f64], overhang: f64, openings: bool) -> (Vec<f64>, f64) {
    let rise: Vec<f64> = slopes
        .iter()
        .enumerate()
        .map(|(i, &s)| if s > 0.0 { s * (overhang - overhangs.get(i).copied().unwrap_or(overhang)) } else { 0.0 })
        .collect();
    let mut low = rise.iter().zip(slopes).filter(|(_, s)| **s > 0.0).map(|(r, _)| *r).fold(f64::INFINITY, f64::min);
    if openings {
        low = low.min(0.0);
    }
    if !low.is_finite() {
        low = 0.0;
    }
    let starts = rise.iter().zip(slopes).map(|(r, s)| if *s > 0.0 { (r - low).max(0.0) } else { 0.0 }).collect();
    (starts, low)
}

/// Where a shed roof's plane starts, along its rise direction `d`: the low
/// eave of the footprint grown by the roof's own overhang (edges with an
/// overhang of their own leave the plane where it is).
pub fn shed_low(r: &cc_core::Roof, d: DVec2) -> f64 {
    polygon::offset(&r.footprint, r.overhang).iter().map(|p| p.dot(d)).fold(f64::INFINITY, f64::min)
}

/// Move the eaves [`sloped_eaves`] gave for `footprint` and `slopes` out to
/// where their planes reach the base, given the height each starts at
/// (see [`eave_starts`]).
pub fn raise_eaves(eaves: &mut [Eave], footprint: &[DVec2], slopes: &[f64], starts: &[f64]) {
    let n = footprint.len();
    let sloped = (0..n).filter(|&i| {
        (footprint[(i + 1) % n] - footprint[i]).try_normalize().is_some() && slopes.get(i).is_some_and(|s| *s > 0.0)
    });
    for (eave, i) in eaves.iter_mut().zip(sloped) {
        let start = starts.get(i).copied().unwrap_or(0.0);
        if eave.slope > 0.0 {
            eave.a -= eave.normal * (start / eave.slope);
        }
    }
}

/// The sloped eaves of a counter-clockwise footprint, given the slope of
/// each edge (see [`edge_slopes`]); edges of slope 0 are gables.
pub fn sloped_eaves(footprint: &[DVec2], slopes: &[f64]) -> Vec<Eave> {
    let n = footprint.len();
    (0..n)
        .filter_map(|i| {
            let (a, b) = (footprint[i], footprint[(i + 1) % n]);
            let dir = (b - a).try_normalize()?;
            let slope = slopes.get(i).copied().unwrap_or(0.0);
            (slope > 0.0).then(|| Eave { a, normal: dir.perp(), slope })
        })
        .collect()
}

/// Part of the footprint owned by each eave's plane (empty parts are skipped).
pub fn roof_regions(footprint: &[DVec2], eaves: &[Eave]) -> Vec<(usize, Vec<DVec2>)> {
    eaves
        .iter()
        .enumerate()
        .filter_map(|(i, e)| {
            let mut region = footprint.to_vec();
            for (j, other) in eaves.iter().enumerate() {
                if i != j {
                    // height_i <= height_j  <=>  p·(s_i n_i - s_j n_j) <= s_i a_i·n_i - s_j a_j·n_j
                    let (ni, nj) = (e.normal * e.slope, other.normal * other.slope);
                    region = clip_half_plane(&region, ni - nj, e.a.dot(ni) - other.a.dot(nj));
                }
            }
            (polygon::area(&region) > 1e-9).then_some((i, region))
        })
        .collect()
}

/// Ridge and hip lines in plan: region edges that are not on the footprint boundary.
pub fn ridge_lines(footprint: &[DVec2], regions: &[(usize, Vec<DVec2>)]) -> Vec<(DVec2, DVec2)> {
    let on_boundary = |p: DVec2, q: DVec2| on_footprint_edge(footprint, (p + q) * 0.5);
    let mut lines: Vec<(DVec2, DVec2)> = Vec::new();
    for (_, region) in regions {
        let n = region.len();
        for i in 0..n {
            let (p, q) = (region[i], region[(i + 1) % n]);
            if p.distance(q) < 1e-6 || on_boundary(p, q) {
                continue;
            }
            // Shared edges appear in both neighbouring regions; keep one copy.
            let duplicate = lines.iter().any(|&(a, b)| {
                (a.distance(p) < 1e-6 && b.distance(q) < 1e-6) || (a.distance(q) < 1e-6 && b.distance(p) < 1e-6)
            });
            if !duplicate {
                lines.push((p, q));
            }
        }
    }
    lines
}

fn on_footprint_edge(footprint: &[DVec2], m: DVec2) -> bool {
    let n = footprint.len();
    (0..n).any(|i| polygon::closest_on_segment(footprint[i], footprint[(i + 1) % n], m).0.distance(m) < 1e-7)
}

fn plane_normal(pts: &[DVec3]) -> DVec3 {
    let n = pts.len();
    let sum: DVec3 = (0..n).map(|i| pts[i].cross(pts[(i + 1) % n])).sum();
    sum.normalize_or_zero()
}

/// Add the closed solid of a pitched roof of vertical thickness `thickness`
/// whose underside is at `base` along the eaves.
pub fn add_pitched_roof(solid: &mut Solid, footprint: &[DVec2], eaves: &[Eave], base: f64, thickness: f64) {
    let regions = roof_regions(footprint, eaves);
    for (i, region) in &regions {
        let Some(eave) = eaves.get(*i) else { continue };
        let bottom = |p: DVec2| DVec3::new(p.x, p.y, base + eave.height(p));
        let top: Vec<DVec3> = region.iter().map(|&p| bottom(p) + DVec3::Z * thickness).collect();
        let mut under: Vec<DVec3> = region.iter().map(|&p| bottom(p)).collect();
        under.reverse();
        let (tn, un) = (plane_normal(&top), plane_normal(&under));
        solid.faces.push(Face::new(top, tn));
        solid.faces.push(Face::new(under, un));
        // Vertical faces where the region meets the footprint outline (eaves and gable ends).
        let n = region.len();
        for k in 0..n {
            let (p, q) = (region[k], region[(k + 1) % n]);
            if p.distance(q) < 1e-9 || !on_footprint_edge(footprint, (p + q) * 0.5) {
                continue;
            }
            let out = -(q - p).normalize_or_zero().perp();
            let quad = vec![bottom(p), bottom(q), bottom(q) + DVec3::Z * thickness, bottom(p) + DVec3::Z * thickness];
            solid.faces.push(Face::new(quad, DVec3::new(out.x, out.y, 0.0)));
        }
    }
}

/// How a roof's slab is built: its outline, openings and planes, from the
/// roof, its level's elevation and its type's thickness (see [`body`]).
#[derive(Debug, Clone)]
pub enum RoofBody {
    /// A slab of `thickness` over the outline less its openings, its
    /// underside at `base` (flat roofs, and pitched ones with no sloped edge
    /// or whose skeleton failed round openings).
    Prism { outline: Vec<DVec2>, holes: Vec<Vec<DVec2>>, base: f64, thickness: f64 },
    /// A shed roof's plane, its underside `base + slope * (p · dir - low)`,
    /// `thickness` measured vertically.
    Shed { outline: Vec<DVec2>, holes: Vec<Vec<DVec2>>, base: f64, slope: f64, dir: DVec2, low: f64, thickness: f64 },
    /// A pitched roof from its skeleton (see [`add_skeleton_roof`]).
    Skeleton { sk: crate::skeleton::Skeleton, base: f64, thickness: f64 },
    /// A pitched roof from the lowest planes of its outline, should the
    /// skeleton fail (see [`add_pitched_roof`]).
    Planes { outline: Vec<DVec2>, eaves: Vec<Eave>, base: f64, thickness: f64 },
}

/// How roof `r` on a level at `level_z`, its type `thk` thick, is built:
/// the outline grown by each edge's overhang ([`outline`]), openings shrunk
/// by the overhang and shafts' cuts, and its planes.
pub fn body(r: &cc_core::Roof, level_z: f64, thk: f64) -> RoofBody {
    use cc_core::RoofShape;
    // The footprint grown by each edge's overhang.
    let outline = outline(r);
    let fp = outline.points.clone();
    // Openings shrink by the overhang (the eaves reach over them), clockwise.
    let holes: Vec<Vec<DVec2>> = r
        .holes
        .iter()
        .map(|h| {
            let mut h = polygon::offset(h, -r.overhang);
            h.reverse();
            h
        })
        .filter(|h| h.len() >= 3 && polygon::signed_area(h) < 0.0)
        // Shafts cut their own outline: no eaves reach over them.
        .chain(r.cuts.iter().map(|c| {
            let mut c = polygon::ccw(c);
            c.reverse();
            c
        }))
        .collect();
    let base = level_z + r.base_offset;
    match r.shape {
        RoofShape::Flat => RoofBody::Prism { outline: fp, holes, base, thickness: thk },
        RoofShape::Shed { slope, direction_deg } => {
            let a = direction_deg.to_radians();
            let dir = DVec2::new(a.cos(), a.sin());
            // The plane starts at the low eave of the roof's own overhang.
            let low = shed_low(r, dir);
            let thickness = thk * (1.0 + slope * slope).sqrt();
            RoofBody::Shed { outline: fp, holes, base, slope, dir, low, thickness }
        }
        RoofShape::Gable { slope, .. } | RoofShape::Hip { slope } => {
            let ridge = match r.shape {
                RoofShape::Gable { ridge_deg, .. } => Some(DVec2::from_angle(ridge_deg.to_radians())),
                _ => None,
            };
            let vertical_thk = thk * (1.0 + slope * slope).sqrt();
            let mut loops = vec![fp.clone()];
            loops.extend(holes.iter().cloned());
            let own = &outline.own;
            let weights = eave_weights(&loops, slope, ridge, own);
            let sloped = weights.iter().flatten().any(|w| *w > 0.0);
            // Eaves with an overhang of their own lie higher or lower on
            // their planes: they start late, from the lowest eave.
            let slopes = edge_slopes(&fp, slope, ridge, own);
            let (starts, low) = eave_starts(&slopes, &outline.overhangs, r.overhang, loops.len() > 1);
            let base = base + low;
            // Openings keep their eaves at the roof's base.
            let mut all_starts = vec![starts.clone()];
            all_starts.extend(holes.iter().map(|h| vec![-low; h.len()]));
            match crate::skeleton::skeleton_with_seeds(&loops, &weights, &all_starts, &outline.seeds) {
                Ok(sk) if sloped => RoofBody::Skeleton { sk, base, thickness: vertical_thk },
                // Should the skeleton fail, the lowest planes of the outline.
                Err(_) if sloped && holes.is_empty() => {
                    let mut eaves = sloped_eaves(&fp, &slopes);
                    raise_eaves(&mut eaves, &fp, &slopes, &starts);
                    RoofBody::Planes { outline: fp, eaves, base, thickness: vertical_thk }
                }
                _ => RoofBody::Prism { outline: fp, holes, base, thickness: thk },
            }
        }
    }
}

impl RoofBody {
    /// Add the roof's closed slab to `solid`.
    pub fn add_to(&self, solid: &mut Solid) {
        match self {
            RoofBody::Prism { outline, holes, base, thickness } => {
                let (base, thk) = (*base, *thickness);
                solid.add_prism_with_holes(outline, holes, |_| base, |_| base + thk);
            }
            RoofBody::Shed { outline, holes, base, slope, dir, low, thickness } => {
                let (base, slope, d, min, vertical_thk) = (*base, *slope, *dir, *low, *thickness);
                let bottom = move |p: DVec2| base + slope * (p.dot(d) - min);
                solid.add_prism_with_holes(outline, holes, bottom, move |p| bottom(p) + vertical_thk);
            }
            RoofBody::Skeleton { sk, base, thickness } => add_skeleton_roof(solid, sk, *base, *thickness),
            RoofBody::Planes { outline, eaves, base, thickness } => {
                add_pitched_roof(solid, outline, eaves, *base, *thickness);
            }
        }
    }

    /// The ridges, hips and valleys in plan, of pitched roofs.
    pub fn lines(&self) -> Option<Vec<(DVec2, DVec2)>> {
        match self {
            RoofBody::Skeleton { sk, .. } => Some(sk.lines.clone()),
            RoofBody::Planes { outline, eaves, .. } => Some(ridge_lines(outline, &roof_regions(outline, eaves))),
            RoofBody::Prism { .. } | RoofBody::Shed { .. } => None,
        }
    }

    /// The slab's thickness, measured vertically.
    pub fn thickness(&self) -> f64 {
        match self {
            RoofBody::Prism { thickness, .. }
            | RoofBody::Shed { thickness, .. }
            | RoofBody::Skeleton { thickness, .. }
            | RoofBody::Planes { thickness, .. } => *thickness,
        }
    }

    /// The slab's underside as pieces of planes (see
    /// [`crate::roof_join::Piece`]): each face of the skeleton (a face round
    /// openings as one ring, the openings joined to its outline, or in
    /// triangles), each flat top, each region of the fallback's planes, or
    /// the outline less its openings.
    pub fn pieces(&self) -> Vec<crate::roof_join::Piece> {
        use crate::roof_join::Piece;
        let ring = |outer: &[DVec2], holes: &[Vec<DVec2>]| -> Vec<Vec<DVec2>> {
            let outer = polygon::ccw(outer);
            if holes.is_empty() {
                return vec![outer];
            }
            match polygon::keyhole(&outer, holes) {
                Some(r) => vec![r],
                None => polygon::triangulate_with_holes(&outer, holes).into_iter().map(|t| polygon::ccw(&t)).collect(),
            }
        };
        let flat =
            |c: f64, rings: Vec<Vec<DVec2>>| rings.into_iter().map(move |plan| Piece { plan, c, g: DVec2::ZERO });
        match self {
            RoofBody::Prism { outline, holes, base, .. } => flat(*base, ring(outline, holes)).collect(),
            RoofBody::Shed { outline, holes, base, slope, dir, low, .. } => ring(outline, holes)
                .into_iter()
                .map(|plan| Piece { plan, c: base - slope * low, g: *dir * *slope })
                .collect(),
            RoofBody::Skeleton { sk, base, .. } => {
                let mut out = Vec::new();
                for (i, (plane, pts)) in sk.faces.iter().enumerate() {
                    let holes: Vec<Vec<DVec2>> =
                        sk.face_holes.iter().filter(|(f, _)| *f == i).map(|(_, h)| h.clone()).collect();
                    // The plane's height is affine in plan: through the origin and along each axis.
                    let h0 = sk.face_height(*plane, DVec2::ZERO);
                    let g = DVec2::new(sk.face_height(*plane, DVec2::X) - h0, sk.face_height(*plane, DVec2::Y) - h0);
                    out.extend(ring(pts, &holes).into_iter().map(|plan| Piece { plan, c: base + h0, g }));
                }
                for (height, pts) in &sk.flats {
                    out.extend(flat(base + height, ring(pts, &[])));
                }
                out
            }
            RoofBody::Planes { outline, eaves, base, .. } => roof_regions(outline, eaves)
                .into_iter()
                .filter_map(|(i, plan)| {
                    let e = eaves.get(i)?;
                    let g = e.normal * e.slope;
                    Some(Piece { plan: polygon::ccw(&plan), c: base - e.a.dot(g), g })
                })
                .collect(),
        }
    }
}

/// Plan area covered by a roof: its outline grown by the overhangs, less
/// its openings shrunk by the roof's.
pub fn plan_area(r: &cc_core::Roof) -> f64 {
    let holes: f64 = r
        .holes
        .iter()
        .map(|h| polygon::offset(h, -r.overhang))
        .filter(|h| h.len() >= 3 && polygon::signed_area(h) > 0.0)
        .map(|h| polygon::area(&h))
        .sum();
    (polygon::area(&outline(r).points) - holes).max(0.0)
}

/// Edge weights of a footprint's loops for the skeleton: sloped eaves move
/// in by `1 / slope` per unit of height, gables (see [`edge_slopes`]) stay.
/// `own` holds the outline's own edge slopes; openings follow the shape.
pub fn eave_weights(loops: &[Vec<DVec2>], slope: f64, ridge: Option<DVec2>, own: &[Option<f64>]) -> Vec<Vec<f64>> {
    loops
        .iter()
        .enumerate()
        .map(|(l, pts)| {
            let own = if l == 0 { own } else { &[] };
            edge_slopes(pts, slope, ridge, own).into_iter().map(|s| if s > 0.0 { 1.0 / s } else { 0.0 }).collect()
        })
        .collect()
}

/// Add the closed solid of a roof from its skeleton: each plane's part
/// lifted from `base` (its underside) with `thickness` measured vertically,
/// flat tops where the planes never close, and vertical faces along the
/// eaves and the gables.
pub fn add_skeleton_roof(solid: &mut Solid, sk: &crate::skeleton::Skeleton, base: f64, thickness: f64) {
    let up = DVec3::Z * thickness;
    let mut surface = |pts: &[DVec2], height: &dyn Fn(DVec2) -> f64| {
        let under: Vec<DVec3> = pts.iter().map(|&p| DVec3::new(p.x, p.y, base + height(p))).collect();
        let top: Vec<DVec3> = under.iter().map(|&p| p + up).collect();
        let mut under = under;
        under.reverse();
        let (tn, un) = (plane_normal(&top), plane_normal(&under));
        if tn != DVec3::ZERO && un != DVec3::ZERO {
            solid.faces.push(Face::new(top, tn));
            solid.faces.push(Face::new(under, un));
        }
    };
    for (i, (plane, pts)) in sk.faces.iter().enumerate() {
        let holes: Vec<Vec<DVec2>> = sk.face_holes.iter().filter(|(f, _)| *f == i).map(|(_, h)| h.clone()).collect();
        if holes.is_empty() {
            surface(pts, &|p| sk.face_height(*plane, p));
        } else {
            // A plane all the way round an opening, in triangles (a face has one outline).
            for t in polygon::triangulate_with_holes(pts, &holes) {
                surface(&t, &|p| sk.face_height(*plane, p));
            }
        }
    }
    for (height, pts) in &sk.flats {
        surface(pts, &|_| *height);
    }
    let mut strip = |a: DVec3, b: DVec3, out: DVec2| {
        let mut quad = vec![a, b, b + up, a + up];
        let out = DVec3::new(out.x, out.y, 0.0);
        if plane_normal(&quad).dot(out) < 0.0 {
            quad.reverse();
        }
        solid.faces.push(Face::new(quad, out));
    };
    for plane in sk.planes.iter().filter(|p| p.weight > 0.0 && p.start <= 0.0) {
        let a = plane.origin;
        let b = a + plane.dir * plane.length;
        strip(DVec3::new(a.x, a.y, base), DVec3::new(b.x, b.y, base), -plane.dir.perp());
    }
    // Eaves that start higher: the part of their edge left when they do.
    for &(i, a, b, h) in &sk.late_eaves {
        let Some(plane) = sk.planes.get(i) else { continue };
        strip(DVec3::new(a.x, a.y, base + h), DVec3::new(b.x, b.y, base + h), -plane.dir.perp());
    }
    for &(p, t0, q, t1, out) in &sk.gables {
        strip(DVec3::new(p.x, p.y, base + t0), DVec3::new(q.x, q.y, base + t1), out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cc_core::{Category, ElementId};

    fn rect(w: f64, d: f64) -> Vec<DVec2> {
        vec![DVec2::ZERO, DVec2::new(w, 0.0), DVec2::new(w, d), DVec2::new(0.0, d)]
    }

    #[test]
    fn hip_roof_on_a_rectangle_has_a_ridge_and_four_hips() {
        let fp = rect(10.0, 6.0);
        let eaves = sloped_eaves(&fp, &edge_slopes(&fp, 1.0, None, &[]));
        let regions = roof_regions(&fp, &eaves);
        assert_eq!(regions.len(), 4);
        let total: f64 = regions.iter().map(|(_, r)| polygon::area(r)).sum();
        assert!((total - 60.0).abs() < 1e-9);
        // Ridge from (3,3) to (7,3) plus four hips.
        let lines = ridge_lines(&fp, &regions);
        assert_eq!(lines.len(), 5, "{lines:?}");
        assert!(
            lines.iter().any(|&(a, b)| (a.y - 3.0).abs() < 1e-9 && (b.y - 3.0).abs() < 1e-9 && (a.x - b.x).abs() > 3.9)
        );
    }

    #[test]
    fn gable_roof_volume_and_ridge_height() {
        let fp = rect(10.0, 6.0);
        let (slope, thickness) = (0.5, 0.2);
        let eaves = sloped_eaves(&fp, &edge_slopes(&fp, slope, Some(DVec2::X), &[]));
        assert_eq!(eaves.len(), 2);
        let mut s = Solid::new(ElementId(1), Category::Roof, [0; 3]);
        add_pitched_roof(&mut s, &fp, &eaves, 3.0, thickness);
        // Volume of a constant vertical thickness: plan area x thickness.
        assert!((s.volume() - 60.0 * thickness).abs() < 1e-9, "{}", s.volume());
        let (_, hi) = s.bounds().unwrap();
        assert!((hi.z - (3.0 + slope * 3.0 + thickness)).abs() < 1e-9);
    }

    fn skeleton_solid(loops: &[Vec<DVec2>], slope: f64, ridge: Option<DVec2>, thickness: f64) -> Solid {
        let weights = eave_weights(loops, slope, ridge, &[]);
        let sk = crate::skeleton::skeleton(loops, &weights).unwrap();
        let mut s = Solid::new(ElementId(1), Category::Roof, [0; 3]);
        add_skeleton_roof(&mut s, &sk, 3.0, thickness);
        s
    }

    #[test]
    fn skeleton_roofs_are_closed_slabs() {
        let l = vec![
            DVec2::new(0.0, 0.0),
            DVec2::new(10.0, 0.0),
            DVec2::new(10.0, 4.0),
            DVec2::new(4.0, 4.0),
            DVec2::new(4.0, 10.0),
            DVec2::new(0.0, 10.0),
        ];
        let courtyard = [
            rect(12.0, 12.0),
            vec![DVec2::new(4.0, 4.0), DVec2::new(4.0, 8.0), DVec2::new(8.0, 8.0), DVec2::new(8.0, 4.0)],
        ];
        for (loops, area) in [(vec![rect(10.0, 6.0)], 60.0), (vec![l], 64.0), (courtyard.to_vec(), 128.0)] {
            for ridge in [None, Some(DVec2::X)] {
                let s = skeleton_solid(&loops, 0.5, ridge, 0.2);
                // A constant vertical thickness: plan area x thickness.
                assert!((s.volume() - area * 0.2).abs() < 1e-6, "{ridge:?}: {} vs {}", s.volume(), area * 0.2);
            }
        }
        // A square hip roof peaks at half its width times the slope.
        let s = skeleton_solid(&[rect(8.0, 8.0)], 0.6, None, 0.1);
        let (_, hi) = s.bounds().unwrap();
        assert!((hi.z - (3.0 + 4.0 * 0.6 + 0.1)).abs() < 1e-9);
        // Equal slopes: the sloped faces' true areas sum to the footprint over cos(slope).
        let sk =
            crate::skeleton::skeleton(&[rect(10.0, 6.0)], &eave_weights(&[rect(10.0, 6.0)], 0.75, None, &[])).unwrap();
        let true_area: f64 = sk.faces.iter().map(|(_, f)| polygon::area(f) * (1.0f64 + 0.75 * 0.75).sqrt()).sum();
        assert!((true_area - 60.0 * 1.25).abs() < 1e-9);
    }

    /// Issue #380: the slab of a gable roof with a notch cut into its north
    /// eave right up to a corner tops out at the ridge, and one whose north
    /// plane runs round an opening leaves the opening out: the volumes are
    /// the plan area times the (vertical) thickness.
    #[test]
    fn notched_and_holed_gable_roofs_are_closed_slabs() {
        let pts = |c: &[(f64, f64)]| c.iter().map(|&(x, y)| DVec2::new(x, y)).collect::<Vec<_>>();
        let notched = pts(&[(0.0, 0.0), (13.716, 0.0), (13.716, 7.01), (11.125, 4.419), (8.534, 7.01), (0.0, 7.01)]);
        let own = [None, Some(0.0), Some(0.0), Some(0.0), None, Some(0.0)];
        let weights = eave_weights(std::slice::from_ref(&notched), 0.42, Some(DVec2::X), &own);
        let sk = crate::skeleton::skeleton(std::slice::from_ref(&notched), &weights).unwrap();
        let mut s = Solid::new(ElementId(1), Category::Roof, [0; 3]);
        add_skeleton_roof(&mut s, &sk, 3.0, 0.2);
        assert!((s.volume() - polygon::area(&notched) * 0.2).abs() < 1e-6, "{}", s.volume());
        let (_, hi) = s.bounds().unwrap();
        assert!((hi.z - (3.0 + 3.505 * 0.42 + 0.2)).abs() < 1e-9, "the ridge is the top: {}", hi.z);
        // An opening with gable sides (steeper than 45° to the ridge) and a sloped top.
        let outer = rect(13.716, 7.01);
        let hole = pts(&[(9.5, 6.5), (12.5, 6.5), (11.0, 4.4)]);
        let s = skeleton_solid(&[outer.clone(), hole.clone()], 0.42, Some(DVec2::X), 0.2);
        let area = polygon::area(&outer) - polygon::area(&hole);
        assert!((s.volume() - area * 0.2).abs() < 1e-6, "{} vs {}", s.volume(), area * 0.2);
    }

    #[test]
    fn hip_roof_solid_is_closed() {
        let fp = rect(8.0, 8.0);
        let eaves = sloped_eaves(&fp, &edge_slopes(&fp, 1.0, None, &[]));
        let mut s = Solid::new(ElementId(1), Category::Roof, [0; 3]);
        add_pitched_roof(&mut s, &fp, &eaves, 0.0, 0.1);
        assert!((s.volume() - 6.4).abs() < 1e-9);
    }

    #[test]
    fn edge_slopes_make_gables_and_steeper_eaves() {
        let fp = rect(10.0, 6.0);
        // A hip roof with its east edge (edge 1) turned into a gable end.
        let own = [None, Some(0.0)];
        let weights = eave_weights(std::slice::from_ref(&fp), 0.5, None, &own);
        assert_eq!(weights[0], vec![2.0, 0.0, 2.0, 2.0]);
        let sk = crate::skeleton::skeleton(std::slice::from_ref(&fp), &weights).unwrap();
        assert!(sk.faces.iter().all(|(p, _)| *p != 1), "a gable edge has no sloped face");
        let mut s = Solid::new(ElementId(1), Category::Roof, [0; 3]);
        add_skeleton_roof(&mut s, &sk, 0.0, 0.2);
        assert!((s.volume() - 60.0 * 0.2).abs() < 1e-6, "{}", s.volume());
        // A gable roof whose south eave is steeper: the planes meet where 1.0 y = 0.5 (6 - y).
        let slopes = edge_slopes(&fp, 0.5, Some(DVec2::X), &[Some(1.0)]);
        assert_eq!(slopes, vec![1.0, 0.0, 0.5, 0.0]);
        let mut s = Solid::new(ElementId(1), Category::Roof, [0; 3]);
        add_pitched_roof(&mut s, &fp, &sloped_eaves(&fp, &slopes), 0.0, 0.1);
        let (_, hi) = s.bounds().unwrap();
        assert!((hi.z - 2.1).abs() < 1e-9, "{}", hi.z);
        assert!((s.volume() - 6.0).abs() < 1e-9, "{}", s.volume());
    }

    #[test]
    fn edge_slopes_follow_a_clockwise_footprint() {
        let mut fp = rect(10.0, 6.0);
        fp.reverse();
        let mut r = cc_core::Roof::new(fp, 0.0, 0.0, cc_core::RoofShape::Hip { slope: 0.5 });
        r.edge_slopes = vec![None, Some(0.9)];
        let ccw = polygon::ccw(&r.footprint);
        let own = ccw_edge_slopes(&r);
        assert_eq!(own.iter().filter(|s| s.is_some()).count(), 1);
        let i = own.iter().position(Option::is_some).unwrap();
        // The same segment as the footprint's edge 1, walked the other way.
        let (a, b) = (ccw[i], ccw[(i + 1) % 4]);
        assert!(a.distance(r.footprint[2]) < 1e-9 && b.distance(r.footprint[1]) < 1e-9, "{a} {b}");
    }

    /// Overhangs per edge: each edge moves out by its own; a straight wall
    /// split in two with different overhangs steps square between them; no
    /// edge of its own gives the uniform offset exactly.
    #[test]
    fn outline_grows_each_edge_by_its_overhang() {
        let fp =
            vec![DVec2::ZERO, DVec2::new(5.0, 0.0), DVec2::new(10.0, 0.0), DVec2::new(10.0, 6.0), DVec2::new(0.0, 6.0)];
        let mut r = cc_core::Roof::new(fp, 0.0, 0.5, cc_core::RoofShape::Hip { slope: 0.5 });
        assert_eq!(outline(&r).points, polygon::ccw(&polygon::offset(&r.footprint, 0.5)));
        r.edge_overhangs = vec![Some(0.0), Some(1.0)];
        let o = outline(&r);
        let want = [(-0.5, 0.0), (5.0, 0.0), (5.0, -1.0), (10.5, -1.0), (10.5, 6.5), (-0.5, 6.5)];
        let got: Vec<(f64, f64)> = o.points.iter().map(|p| (p.x, p.y)).collect();
        assert_eq!(got.len(), want.len(), "{got:?}");
        for (p, w) in o.points.iter().zip(want) {
            assert!(p.distance(DVec2::new(w.0, w.1)) < 1e-12, "{got:?}");
        }
        assert_eq!(o.own, vec![None, Some(0.0), None, None, None, None]);
        assert_eq!(o.overhangs, vec![0.0, 1.0, 1.0, 0.5, 0.5, 0.5]);
        // The edge with no overhang starts a quarter meter up; the one reaching 1 m
        // out is the lowest eave, 0.25 m below the base; the step is a gable.
        let slopes = edge_slopes(&o.points, 0.5, None, &o.own);
        let (starts, low) = eave_starts(&slopes, &o.overhangs, 0.5, false);
        assert!((low + 0.25).abs() < 1e-12, "{low}");
        assert_eq!(starts, vec![0.5, 0.0, 0.0, 0.25, 0.25, 0.25]);
        // Its slab is closed: as much volume as plan area times its thickness.
        let weights = eave_weights(std::slice::from_ref(&o.points), 0.5, None, &o.own);
        let sk = crate::skeleton::skeleton_with_starts(std::slice::from_ref(&o.points), &weights, &[starts]).unwrap();
        assert!(!sk.late_eaves.is_empty());
        let mut s = Solid::new(ElementId(1), Category::Roof, [0; 3]);
        add_skeleton_roof(&mut s, &sk, low, 0.2);
        let area = polygon::area(&o.points);
        assert!((s.volume() - area * 0.2).abs() < 1e-6, "{} {}", s.volume(), area * 0.2);
        // Every plane passes its wall line half a meter's rise (0.25 m) over the base.
        for (plane, pts) in &sk.faces {
            for p in pts {
                let walls = [p.y, 10.0 - p.x, 6.0 - p.y, p.x];
                let want = 0.25 + 0.5 * walls.iter().copied().fold(f64::INFINITY, f64::min);
                assert!((low + sk.face_height(*plane, *p) - want).abs() < 1e-9, "{p}");
            }
        }
    }

    /// The notched roof of issue #380 (13.716 × 7.01, a V notch cut into
    /// its north eave right up to the north-east corner, the notch's sides
    /// and the ends gables): a gable roof along X at 42 % with `overhang`,
    /// and `edges` the overhang of each footprint edge.
    fn notched_roof(overhang: f64, edges: &[Option<f64>]) -> cc_core::Roof {
        let fp = vec![
            DVec2::new(0.0, 0.0),
            DVec2::new(13.716, 0.0),
            DVec2::new(13.716, 7.01),
            DVec2::new(11.125, 4.419),
            DVec2::new(8.534, 7.01),
            DVec2::new(0.0, 7.01),
        ];
        let shape = cc_core::RoofShape::Gable { slope: 0.42, ridge_deg: 0.0 };
        let mut r = cc_core::Roof::new(fp, 0.0, overhang, shape);
        r.edge_slopes = vec![None, Some(0.0), Some(0.0), Some(0.0), None, Some(0.0)];
        r.edge_overhangs = edges.to_vec();
        r
    }

    /// A roof's slab as the model builds it (0.2 thick, vertically): the
    /// skeleton, the solid and the lowest eave's height.
    fn slab(r: &cc_core::Roof) -> (crate::skeleton::Skeleton, Solid, f64) {
        slab_seeded(r, true)
    }

    /// [`slab`], with or without the outline's corner seeds.
    fn slab_seeded(r: &cc_core::Roof, seeded: bool) -> (crate::skeleton::Skeleton, Solid, f64) {
        let o = outline(r);
        let (slope, ridge) = (0.42, Some(DVec2::X));
        let weights = eave_weights(std::slice::from_ref(&o.points), slope, ridge, &o.own);
        let slopes = edge_slopes(&o.points, slope, ridge, &o.own);
        let (starts, low) = eave_starts(&slopes, &o.overhangs, r.overhang, false);
        let seeds = if seeded { o.seeds.clone() } else { Vec::new() };
        let sk =
            crate::skeleton::skeleton_with_seeds(std::slice::from_ref(&o.points), &weights, &[starts], &seeds).unwrap();
        let mut s = Solid::new(ElementId(1), Category::Roof, [0; 3]);
        add_skeleton_roof(&mut s, &sk, low, 0.2);
        (sk, s, low)
    }

    /// One ridge at y = 3.505 from `x0` to `x1`; the slab's top over the
    /// ridge, which lies `3.505 + reach` in from the lowest eave tip (`reach`
    /// the most overhang of an eave); its volume the plan area times its
    /// thickness.
    fn full_slab(r: &cc_core::Roof, x0: f64, x1: f64, reach: f64) {
        let (sk, s, low) = slab(r);
        assert_eq!(sk.lines.len(), 1, "one ridge: {:?}", sk.lines);
        let (a, b) = sk.lines[0];
        assert!((a.y - 3.505).abs() < 1e-9 && (b.y - 3.505).abs() < 1e-9, "{:?}", sk.lines);
        assert!((a.x.min(b.x) - x0).abs() < 1e-9 && (a.x.max(b.x) - x1).abs() < 1e-9, "{:?}", sk.lines);
        let vertical = 0.2;
        let (lo, hi) = s.bounds().unwrap();
        assert!((lo.z - low).abs() < 1e-9, "{} {low}", lo.z);
        assert!((hi.z - lo.z - ((3.505 + reach) * 0.42 + vertical)).abs() < 1e-9, "the ridge is the top: {lo} {hi}");
        assert!((s.volume() - plan_area(r) * vertical).abs() < 1e-6, "{} vs {}", s.volume(), plan_area(r) * vertical);
    }

    /// Issue #380 with an overhang: the corner where the notch meets the
    /// east gable is cut along the north eave's moved line, as if a piece
    /// of eave of no length were there; the ridge runs the whole length,
    /// out over both gables' overhangs.
    #[test]
    fn a_notch_up_to_a_corner_is_cut_along_the_eave() {
        let r = notched_roof(0.5, &[]);
        let o = outline(&r);
        let h = std::f64::consts::SQRT_2;
        let want = [
            (-0.5, -0.5),
            (14.216, -0.5),
            (14.216, 7.51),
            (13.716 + 0.5 - 0.5 * h, 7.51),
            (11.125, 4.419 + 0.5 * h),
            (8.534 + 0.5 * h - 0.5, 7.51),
            (-0.5, 7.51),
        ];
        assert_eq!(o.points.len(), want.len(), "{:?}", o.points);
        for (p, w) in o.points.iter().zip(want) {
            assert!(p.distance(DVec2::new(w.0, w.1)) < 1e-9, "{:?}", o.points);
        }
        // The piece between the two cut points is a piece of the north eave.
        assert_eq!(o.own, vec![None, Some(0.0), None, Some(0.0), Some(0.0), None, Some(0.0)]);
        assert!(o.seeds.is_empty());
        full_slab(&r, -0.5, 14.216, 0.5);

        // Overhangs per edge: the north eave 0.762, the end gables 0.3.
        let r = notched_roof(0.5, &[None, Some(0.3), None, None, Some(0.762), Some(0.3)]);
        let o = outline(&r);
        let (a, b) = (o.points[2], o.points[3]);
        assert!(a.distance(DVec2::new(14.016, 7.772)) < 1e-9, "{a}");
        assert!(b.distance(DVec2::new(13.716 + 0.762 - 0.5 * h, 7.772)) < 1e-9, "{b}");
        full_slab(&r, -0.3, 14.016, 0.762);

        // Ending 1 cm short of the corner, a piece of eave left there: the
        // same roof, with no corner to cut.
        let mut r = notched_roof(0.5, &[]);
        r.footprint.insert(3, DVec2::new(13.706, 7.01));
        r.edge_slopes = vec![None, Some(0.0), None, Some(0.0), Some(0.0), None, Some(0.0)];
        assert_eq!(outline(&r).points, polygon::ccw(&polygon::offset(&r.footprint, 0.5)));
        full_slab(&r, -0.5, 14.216, 0.5);
    }

    /// Where the eave reaches further out than the gables' miter, the piece
    /// between the cut points has no length left: the corner keeps its
    /// miter and the eave's plane starts there once its wavefront arrives.
    #[test]
    fn a_notch_corner_inside_the_eave_starts_its_plane_late() {
        let r = notched_roof(0.5, &[None, Some(0.3), Some(0.3), Some(0.3), Some(0.762), Some(0.3)]);
        let o = outline(&r);
        let tip = DVec2::new(14.016, 7.01 + 0.3 + 0.3 * std::f64::consts::SQRT_2);
        assert_eq!(o.points.len(), 6, "{:?}", o.points);
        assert!(o.points[2].distance(tip) < 1e-9, "{:?}", o.points);
        assert_eq!(o.seeds.len(), 1, "{:?}", o.seeds);
        assert!(o.seeds[0].corner.distance(tip) < 1e-9 && (o.seeds[0].eave.y - 7.772).abs() < 1e-9);
        full_slab(&r, -0.3, 14.016, 0.762);
        // Without the seed the corner is left to the south plane: above the ridge.
        let (_, s, low) = slab_seeded(&r, false);
        let (_, hi) = s.bounds().unwrap();
        assert!(hi.z - low > (3.505 + 0.762) * 0.42 + 0.2 + 0.1, "{}", hi.z);
    }
}
