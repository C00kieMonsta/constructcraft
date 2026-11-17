//! Walls whose layers have vertical extents of their own (see
//! [`cc_core::layer_extent`]): the body is the union of a band per run of
//! neighbouring layers that share one extent, each with its own bottom and
//! top, stepped across the wall where neighbouring bands differ.
//!
//! Within a column every height is a line along the wall ([`Lin`]): the
//! wall's bottom and top (its nominal base and top, its elevation profile's
//! chains or the top it is attached to), each layer's bottom and own top
//! raised from the wall's bottom, and the edges of the openings through it.
//! [`WallGeom::split_bands`] splits the columns where any two of these cross,
//! so which line bounds a band, and which bands are solid where, holds across
//! a column (but in the mitered stretch of an end column, which is not split:
//! there a crossing is drawn as if it fell at the stretch's edge).

use super::{Column, Recess, WallGeom, chain_at, split_column, subtract};
use crate::{Face, Solid};
use cc_core::Category;
use cc_core::layer_extent::{EPS, Extent};
use glam::{DVec2, DVec3};

/// A height that runs straight along a wall: `z0 + k u` at `u` along its
/// centerline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lin {
    pub z0: f64,
    pub k: f64,
}

impl Lin {
    /// A level height.
    pub fn level(z: f64) -> Self {
        Self { z0: z, k: 0.0 }
    }

    /// The height at `u`.
    pub fn at(self, u: f64) -> f64 {
        self.z0 + self.k * u
    }

    fn raised(self, dz: f64) -> Self {
        Self { z0: self.z0 + dz, k: self.k }
    }

    /// Where along the wall it meets `other`, if they are not parallel.
    fn meets(self, other: Lin) -> Option<f64> {
        let dk = self.k - other.k;
        (dk.abs() > 1e-12).then(|| (other.z0 - self.z0) / dk).filter(|u| u.is_finite())
    }
}

/// Neighbouring layers of a wall that share one vertical extent: the body
/// across them, from `v_hi` (nearer the left face) to `v_lo`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Band {
    pub v_hi: f64,
    pub v_lo: f64,
    pub extent: Extent,
    /// The first and last of its layers (indices into [`WallGeom::layers`]).
    pub layers: (usize, usize),
}

/// The middle of the stretch of a column where both its faces run.
fn column_mid(c: &Column) -> f64 {
    (c.left.0.max(c.right.0) + c.left.1.min(c.right.1)) * 0.5
}

/// The line of the chain segment (`(u0, u1, z0, z1)` pieces) that holds
/// `mid`, as [`chain_at`] finds it; `None` for an empty chain.
fn chain_lin(chain: &[(f64, f64, f64, f64)], mid: f64) -> Option<Lin> {
    let (u0, u1, z0, z1) = *chain
        .iter()
        .find(|s| s.0 - 1e-9 <= mid && mid <= s.1 + 1e-9)
        .or_else(|| if mid < chain.first()?.0 { chain.first() } else { chain.last() })?;
    if (u1 - u0).abs() > 1e-12 {
        let k = (z1 - z0) / (u1 - u0);
        Some(Lin { z0: z0 - k * u0, k })
    } else {
        // A vertical step: `chain_at` reads its first height there.
        chain_at(chain, mid, u0).map(Lin::level)
    }
}

/// `intervals` less `cut`, the heights compared at `at` (no line crosses
/// another inside the stretch they span).
fn subtract_lines(intervals: &[(Lin, Lin)], cut: (Lin, Lin), at: f64) -> Vec<(Lin, Lin)> {
    let (c0, c1) = (cut.0.at(at), cut.1.at(at));
    let mut out = Vec::with_capacity(intervals.len() + 1);
    for &(a, b) in intervals {
        let (za, zb) = (a.at(at), b.at(at));
        if c1 <= za + EPS || c0 >= zb - EPS {
            out.push((a, b));
            continue;
        }
        if c0 > za + EPS {
            out.push((a, cut.0));
        }
        if c1 < zb - EPS {
            out.push((cut.1, b));
        }
    }
    out
}

/// What of `a` lies outside every interval of `b`, compared at `at`.
fn exposed(a: &[(Lin, Lin)], b: &[(Lin, Lin)], at: f64) -> Vec<(Lin, Lin)> {
    b.iter().fold(a.to_vec(), |acc, &cut| subtract_lines(&acc, cut, at))
}

/// Intervals evaluated at `u`.
fn heights(lines: &[(Lin, Lin)], u: f64) -> Vec<(f64, f64)> {
    lines.iter().map(|(a, b)| (a.at(u), b.at(u).max(a.at(u)))).collect()
}

/// Sorted intervals merged where they overlap or touch.
fn merged(mut all: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    all.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut out: Vec<(f64, f64)> = Vec::with_capacity(all.len());
    for (a, b) in all {
        match out.last_mut() {
            Some(last) if a <= last.1 + EPS => last.1 = last.1.max(b),
            _ => out.push((a, b)),
        }
    }
    out
}

/// The polygon's area and the `x` of its centroid (zero area: `None`).
fn area_and_centroid_x(poly: &[DVec2]) -> Option<(f64, f64)> {
    let (mut area, mut moment) = (0.0, 0.0);
    for (i, &a) in poly.iter().enumerate() {
        let b = poly[(i + 1) % poly.len()];
        let cross = a.perp_dot(b);
        area += cross;
        moment += (a.x + b.x) * cross;
    }
    (area.abs() > 1e-15).then(|| (area * 0.5, moment / (3.0 * area)))
}

impl WallGeom {
    /// Whether its layers have extents of their own (see
    /// [`cc_core::layer_extent`]): its body is then stepped across the wall.
    pub fn is_banded(&self) -> bool {
        !self.layer_extents.is_empty() && self.layer_extents.len() == self.layers.len()
    }

    /// The layers' extents grouped into bands of neighbours that share one
    /// (none unless [`Self::is_banded`]). The first band starts at the left
    /// face and the last ends at the right one.
    pub fn bands(&self) -> Vec<Band> {
        if !self.is_banded() {
            return Vec::new();
        }
        let mut out: Vec<Band> = Vec::new();
        for (i, ((v_hi, v_lo, _), extent)) in self.layers.iter().zip(&self.layer_extents).enumerate() {
            match out.last_mut() {
                Some(b) if b.extent == *extent => {
                    b.v_lo = *v_lo;
                    b.layers.1 = i;
                }
                _ => out.push(Band { v_hi: *v_hi, v_lo: *v_lo, extent: *extent, layers: (i, i) }),
            }
        }
        if let Some(b) = out.first_mut() {
            b.v_hi = self.half;
        }
        if let Some(b) = out.last_mut() {
            b.v_lo = -self.half;
        }
        out
    }

    /// The ends along the wall of the line `v` across column `c`: its edges
    /// interpolated across the wall (the faces' own at `±half`).
    pub fn span_at(&self, c: &Column, v: f64) -> (f64, f64) {
        let h = self.half;
        if v >= h {
            return c.left;
        }
        if v <= -h {
            return c.right;
        }
        let s = if h > 1e-12 { (v + h) / (2.0 * h) } else { 0.5 };
        (c.right.0 + (c.left.0 - c.right.0) * s, c.right.1 + (c.left.1 - c.right.1) * s)
    }

    /// The wall's bottom and top through column `c`, as lines along it.
    fn column_lines(&self, c: &Column) -> (Lin, Lin) {
        let mid = column_mid(c);
        let bottom = chain_lin(&self.bottom_profile, mid).unwrap_or(Lin::level(self.base_z));
        let top = chain_lin(&self.top_profile, mid).unwrap_or(Lin::level(self.top_z));
        (bottom, top)
    }

    /// The solid heights of a layer of `extent` in column `c`, as lines
    /// along the wall: from its own bottom to its top (its own height's or
    /// the wall's, the lower one), less the openings through the column.
    /// Which lines bound it is decided at the column's middle (see the module).
    pub fn extent_lines(&self, c: &Column, extent: Extent) -> Vec<(Lin, Lin)> {
        let mid = column_mid(c);
        let (bottom, top) = self.column_lines(c);
        let lo = bottom.raised(extent.base_offset);
        let hi = match extent.height {
            Some(h) if lo.raised(h).at(mid) < top.at(mid) => lo.raised(h),
            _ => top,
        };
        if hi.at(mid) <= lo.at(mid) + EPS {
            return Vec::new();
        }
        let mut out = vec![(lo, hi)];
        for o in self.openings.iter().filter(|o| o.u0 <= mid && mid <= o.u1) {
            out = subtract_lines(&out, (Lin::level(o.z0), Lin::level(o.z1)), mid);
        }
        out
    }

    /// The solid heights of layer `layer` in column `c` at `u` along the
    /// centerline: its own extent's, or the whole wall's (see
    /// [`Self::intervals_at`]) when the layers have none.
    pub fn layer_intervals(&self, c: &Column, layer: usize, u: f64) -> Vec<(f64, f64)> {
        match self.layer_extents.get(layer).filter(|_| self.is_banded()) {
            Some(e) => heights(&self.extent_lines(c, *e), u),
            None => self.intervals_at(c, u),
        }
    }

    /// The solid heights at `u` in column `c` of the layer through `v`
    /// across the wall (its left face at `half`).
    pub fn intervals_at_v(&self, c: &Column, u: f64, v: f64) -> Vec<(f64, f64)> {
        let layer = self.layers.iter().position(|(v0, v1, _)| *v1 - 1e-9 <= v && v <= *v0 + 1e-9);
        match layer {
            Some(k) => self.layer_intervals(c, k, u),
            None => self.intervals_at(c, u),
        }
    }

    /// The union of the layers' heights at `u` in column `c`: the wall as a
    /// whole (what joins, rooms and plans' states read).
    pub(super) fn union_at(&self, c: &Column, u: f64) -> Vec<(f64, f64)> {
        let all = self.bands().iter().flat_map(|b| heights(&self.extent_lines(c, b.extent), u)).collect();
        merged(all)
    }

    /// Split the columns where the lines of the layers' extents cross each
    /// other, the wall's bottom and top, or the edges of the openings through
    /// them, and set each column's `solid` to the layers' union at its
    /// middle. Call once the columns are final (after the openings and the
    /// elevation profile or attached top); nothing without extents.
    pub(crate) fn split_bands(&mut self) {
        if !self.is_banded() {
            return;
        }
        let mut extents: Vec<Extent> = Vec::new();
        for e in &self.layer_extents {
            if !extents.contains(e) {
                extents.push(*e);
            }
        }
        let mut columns = Vec::with_capacity(self.columns.len());
        for c in std::mem::take(&mut self.columns) {
            let (lo, hi) = (c.left.0.max(c.right.0), c.left.1.min(c.right.1));
            let mid = (lo + hi) * 0.5;
            let (bottom, top) = self.column_lines(&c);
            let mut lines = vec![bottom, top];
            for e in &extents {
                lines.push(bottom.raised(e.base_offset));
                if let Some(h) = e.height {
                    lines.push(bottom.raised(e.base_offset + h));
                }
            }
            for o in self.openings.iter().filter(|o| o.u0 <= mid && mid <= o.u1) {
                lines.extend([Lin::level(o.z0), Lin::level(o.z1)]);
            }
            let mut cuts: Vec<f64> = Vec::new();
            for (i, a) in lines.iter().enumerate() {
                for b in &lines[i + 1..] {
                    if let Some(u) = a.meets(*b).filter(|u| *u > lo + 1e-6 && *u < hi - 1e-6) {
                        cuts.push(u);
                    }
                }
            }
            cuts.sort_by(f64::total_cmp);
            cuts.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
            columns.extend(split_column(c, &cuts));
        }
        self.columns = columns;
        let solids: Vec<Vec<(f64, f64)>> = self.columns.iter().map(|c| self.union_at(c, column_mid(c))).collect();
        for (c, solid) in self.columns.iter_mut().zip(solids) {
            c.solid = solid;
        }
    }

    /// [`Self::cut_columns`] of a banded wall: split where any layer's
    /// bottom or top crosses `z`, so each layer's state at `z` holds across
    /// a piece.
    pub(super) fn banded_cut_columns(&self, z: f64) -> Vec<Column> {
        let bands = self.bands();
        let mut out = Vec::with_capacity(self.columns.len());
        for c in &self.columns {
            let (lo, hi) = (c.left.0.max(c.right.0), c.left.1.min(c.right.1));
            let mut cuts = Vec::new();
            for b in &bands {
                for (a, t) in self.extent_lines(c, b.extent) {
                    for line in [a, t] {
                        if let Some(u) = line.meets(Lin::level(z)).filter(|u| *u > lo + 1e-6 && *u < hi - 1e-6) {
                            cuts.push(u);
                        }
                    }
                }
            }
            cuts.sort_by(f64::total_cmp);
            cuts.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
            for piece in split_column(c.clone(), &cuts) {
                let solid = self.union_at(c, column_mid(&piece));
                out.push(Column { solid, ..piece });
            }
        }
        out
    }

    /// Exact plan area of the strip of column `c` between `v_hi` and `v_lo`.
    pub fn strip_area(&self, c: &Column, v_hi: f64, v_lo: f64) -> f64 {
        let (hi, lo) = (self.span_at(c, v_hi), self.span_at(c, v_lo));
        match &self.arc {
            Some(a) => {
                // Green's theorem, as `arc_band_area`: two arcs and the straight ends.
                let arc_part = |u0: f64, u1: f64, v: f64| {
                    let r = a.radius_at(v);
                    let (p0, p1) = (a.at(u0, v), a.at(u1, v));
                    (r * r * (a.angle_at(u1) - a.angle_at(u0)) + a.center.perp_dot(p1 - p0)) * 0.5
                };
                let seg = |p: DVec2, q: DVec2| p.perp_dot(q) * 0.5;
                let sum = arc_part(lo.0, lo.1, v_lo)
                    + seg(a.at(lo.1, v_lo), a.at(hi.1, v_hi))
                    + arc_part(hi.1, hi.0, v_hi)
                    + seg(a.at(hi.0, v_hi), a.at(lo.0, v_lo));
                sum.abs()
            }
            None => {
                let quad = [self.world(lo.0, v_lo), self.world(lo.1, v_lo), self.world(hi.1, v_hi), self.world(hi.0, v_hi)];
                crate::polygon::area(&quad)
            }
        }
    }

    /// Where along the centerline the centroid of a strip of column `c`
    /// lies: its middle in an arc wall or a wall of level heights, where it
    /// makes no difference.
    fn strip_centroid_u(&self, c: &Column, v_hi: f64, v_lo: f64) -> f64 {
        let mid = column_mid(c);
        if self.arc.is_some() || (self.top_profile.is_empty() && self.bottom_profile.is_empty()) {
            return mid;
        }
        let (hi, lo) = (self.span_at(c, v_hi), self.span_at(c, v_lo));
        let quad = [DVec2::new(lo.0, v_lo), DVec2::new(lo.1, v_lo), DVec2::new(hi.1, v_hi), DVec2::new(hi.0, v_hi)];
        area_and_centroid_x(&quad).map_or(mid, |(_, x)| x)
    }

    /// The volume a recess takes out of layer `layer`: its part across the
    /// layer's thickness, over the heights the layer has there.
    fn recess_layer_volume(&self, r: &Recess, layer: usize) -> f64 {
        let Some(&(v_from, v_to, _)) = self.layers.get(layer) else { return 0.0 };
        let (face, back) = r.across(self.half);
        let (lo, hi) = (face.min(back).max(v_to), face.max(back).min(v_from));
        if hi <= lo {
            return 0.0;
        }
        let area = match &self.arc {
            Some(a) if a.radius > 1e-9 => {
                let turn = (r.u1 - r.u0) / a.radius;
                ((a.radius_at(hi).powi(2) - a.radius_at(lo).powi(2)) * 0.5 * turn).abs()
            }
            _ => (r.u1 - r.u0) * (hi - lo),
        };
        let mid = (r.u0 + r.u1) * 0.5;
        let Some(c) = self.column_at(mid) else { return 0.0 };
        let height: f64 = self
            .layer_intervals(c, layer, mid)
            .iter()
            .map(|&(z0, z1)| (z1.min(r.z1) - z0.max(r.z0)).max(0.0))
            .sum();
        area * height
    }

    /// The net volume of each layer (in the order of [`Self::layers`]): its
    /// strip of each column times its heights there, less what recesses take
    /// out of it. For a banded wall [`Self::volume`] is their sum; for any
    /// other they add up to it.
    pub fn layer_volumes(&self) -> Vec<f64> {
        let last = self.layers.len().saturating_sub(1);
        (0..self.layers.len())
            .map(|k| {
                let Some(&(v_from, v_to, _)) = self.layers.get(k) else { return 0.0 };
                // The outer layers reach the faces exactly.
                let v_hi = if k == 0 { self.half } else { v_from };
                let v_lo = if k == last { -self.half } else { v_to };
                let body: f64 = self
                    .columns
                    .iter()
                    .map(|c| {
                        let at = self.strip_centroid_u(c, v_hi, v_lo);
                        let height: f64 = self.layer_intervals(c, k, at).iter().map(|(z0, z1)| z1 - z0).sum();
                        self.strip_area(c, v_hi, v_lo) * height
                    })
                    .sum();
                let recessed: f64 = self.recesses.iter().map(|r| self.recess_layer_volume(r, k)).sum();
                (body - recessed).max(0.0)
            })
            .collect()
    }

    /// The layers' volumes added up by material, in the order the
    /// materials first appear.
    pub fn volumes_by_material(&self) -> Vec<(String, f64)> {
        let mut out: Vec<(String, f64)> = Vec::new();
        for ((_, _, material), v) in self.layers.iter().zip(self.layer_volumes()) {
            match out.iter_mut().find(|(m, _)| m == material) {
                Some(slot) => slot.1 += v,
                None => out.push((material.clone(), v)),
            }
        }
        out
    }

    /// Each layer's lowest and highest point, absolute (`None` for a layer
    /// that has no height anywhere).
    pub fn layer_range(&self, layer: usize) -> Option<(f64, f64)> {
        let mut range: Option<(f64, f64)> = None;
        for c in &self.columns {
            let (lo, hi) = (c.left.0.min(c.right.0), c.left.1.max(c.right.1));
            for u in [lo, column_mid(c), hi] {
                for (z0, z1) in self.layer_intervals(c, layer, u) {
                    range = Some(range.map_or((z0, z1), |(a, b)| (a.min(z0), b.max(z1))));
                }
            }
        }
        range
    }

    /// The outlines, in the wall's elevation frame (`u` along the
    /// centerline, absolute `z`), of where layer `layer` stands, openings
    /// left in: one polygon (counter-clockwise) per stretch where it has
    /// height, carried on past the ends to the corners of mitered ends as
    /// the body is. Nothing unless [`Self::is_banded`].
    pub fn layer_outlines(&self, layer: usize) -> Vec<Vec<DVec2>> {
        let Some(extent) = self.layer_extents.get(layer).copied().filter(|_| self.is_banded()) else {
            return Vec::new();
        };
        let mut plain = self.clone();
        plain.recesses.clear();
        plain.set_openings(Vec::new());
        match self.profile.clone() {
            Some(chains) => plain.set_elevation(chains),
            None if !self.top_profile.is_empty() => plain.set_top_profile(self.top_profile.clone()),
            None => {}
        }
        plain.split_bands();
        let first_u = self.start_ext.0.min(self.start_ext.1);
        let last_u = self.end_ext.0.max(self.end_ext.1);
        let count = plain.columns.len();
        let mut out = Vec::new();
        let (mut bottom, mut top): (Vec<DVec2>, Vec<DVec2>) = (Vec::new(), Vec::new());
        let mut flush = |bottom: &mut Vec<DVec2>, top: &mut Vec<DVec2>| {
            let mut poly: Vec<DVec2> = std::mem::take(bottom);
            poly.extend(std::mem::take(top).into_iter().rev());
            poly.dedup_by(|a, b| a.distance(*b) < 1e-9);
            if poly.first().zip(poly.last()).is_some_and(|(a, b)| a.distance(*b) < 1e-9) {
                poly.pop();
            }
            if poly.len() >= 3 && area_and_centroid_x(&poly).is_some_and(|(a, _)| a > 1e-12) {
                out.push(poly);
            }
        };
        for (i, c) in plain.columns.iter().enumerate() {
            let ua = if i == 0 { first_u } else { c.left.0.max(c.right.0) };
            let ub = if i + 1 == count { last_u } else { c.left.1.min(c.right.1) };
            match plain.extent_lines(c, extent).first() {
                Some(&(lo, hi)) => {
                    bottom.extend([DVec2::new(ua, lo.at(ua)), DVec2::new(ub, lo.at(ub))]);
                    top.extend([DVec2::new(ua, hi.at(ua)), DVec2::new(ub, hi.at(ub))]);
                }
                None => flush(&mut bottom, &mut top),
            }
        }
        flush(&mut bottom, &mut top);
        out
    }

    /// A quad from `(u0, v0)` to `(u1, v1)` in plan, its bottom running from
    /// `ba` to `bb` and its top from `za` to `zb`; corners where they meet
    /// are kept once, and one with no area is left out.
    #[allow(clippy::too_many_arguments)]
    fn slanted_face(
        &self,
        (u0, v0): (f64, f64),
        (u1, v1): (f64, f64),
        (ba, bb): (f64, f64),
        (za, zb): (f64, f64),
        n: DVec3,
        smooth: bool,
    ) -> Option<Face> {
        let p = |u: f64, v: f64, z: f64| self.world(u, v).extend(z);
        let mut verts = vec![p(u0, v0, ba), p(u1, v1, bb), p(u1, v1, zb), p(u0, v0, za)];
        verts.dedup_by(|a, b| a.distance(*b) < 1e-12);
        if verts.len() > 1 && verts.first().zip(verts.last()).is_some_and(|(a, b)| a.distance(*b) < 1e-12) {
            verts.pop();
        }
        (verts.len() >= 3).then(|| {
            let mut f = Face::new(verts, n);
            f.smooth = smooth;
            f
        })
    }

    /// The body of a banded wall: each band's top and bottom over its strip
    /// of each column, the faces where it is exposed (the wall's faces, its
    /// steps against the next band, its ends against the next column), the
    /// recesses and the end caps.
    pub(super) fn banded_solid(&self, color: [u8; 3]) -> Solid {
        let mut s = Solid::new(self.element, Category::Wall, color);
        let h = self.half;
        let curved = self.arc.is_some();
        let flat = |v: DVec2| DVec3::new(v.x, v.y, 0.0);
        let bands = self.bands();
        let last_band = bands.len().saturating_sub(1);
        for (ci, c) in self.columns.iter().enumerate() {
            let n = self.segments(c);
            let at = |range: (f64, f64), i: usize| range.0 + (range.1 - range.0) * i as f64 / n as f64;
            let lines: Vec<Vec<(Lin, Lin)>> = bands.iter().map(|b| self.extent_lines(c, b.extent)).collect();
            for (j, band) in bands.iter().enumerate() {
                let (hi_span, lo_span) = (self.span_at(c, band.v_hi), self.span_at(c, band.v_lo));
                let (v_hi, v_lo) = (band.v_hi, band.v_lo);
                for i in 0..n {
                    let (a0, a1) = (at(hi_span, i), at(hi_span, i + 1));
                    let (b0, b1) = (at(lo_span, i), at(lo_span, i + 1));
                    let poly = [self.world(b0, v_lo), self.world(b1, v_lo), self.world(a1, v_hi), self.world(a0, v_hi)];
                    let us = [b0, b1, a1, a0];
                    let left_n = flat(self.normal_at((a0 + a1) * 0.5));
                    let right_n = -flat(self.normal_at((b0 + b1) * 0.5));
                    for &(lo, hi) in &lines[j] {
                        let upper: Vec<DVec3> = poly.iter().zip(us).map(|(p, u)| p.extend(hi.at(u))).collect();
                        let up = (upper[1] - upper[0]).cross(upper[2] - upper[0]).try_normalize().unwrap_or(DVec3::Z);
                        s.faces.push(Face::new(upper, up));
                        let lower: Vec<DVec3> = poly.iter().zip(us).rev().map(|(p, u)| p.extend(lo.at(u))).collect();
                        let down =
                            (lower[1] - lower[0]).cross(lower[2] - lower[0]).try_normalize().unwrap_or(-DVec3::Z);
                        s.faces.push(Face::new(lower, down));
                        // The wall's faces, open where a recess is.
                        if j == 0 {
                            let pieces =
                                self.face_pieces(1.0, (a0, a1), (lo.at(a0), lo.at(a1)), (hi.at(a0), hi.at(a1)));
                            for [a, b, ba, bb, ta, tb] in pieces {
                                s.faces.extend(self.slanted_face((b, h), (a, h), (bb, ba), (tb, ta), left_n, curved));
                            }
                        }
                        if j == last_band {
                            let pieces =
                                self.face_pieces(-1.0, (b0, b1), (lo.at(b0), lo.at(b1)), (hi.at(b0), hi.at(b1)));
                            for [a, b, ba, bb, ta, tb] in pieces {
                                s.faces.extend(self.slanted_face((a, -h), (b, -h), (ba, bb), (ta, tb), right_n, curved));
                            }
                        }
                    }
                    // The step to the next band, where only one of them is solid.
                    if let Some(next) = lines.get(j + 1) {
                        let um = (b0 + b1) * 0.5;
                        let (down_n, up_n) = (-flat(self.normal_at(um)), flat(self.normal_at(um)));
                        for (lo, hi) in exposed(&lines[j], next, um) {
                            let (bot, top) = ((lo.at(b0), lo.at(b1)), (hi.at(b0), hi.at(b1)));
                            s.faces.extend(self.slanted_face((b0, v_lo), (b1, v_lo), bot, top, down_n, curved));
                        }
                        for (lo, hi) in exposed(next, &lines[j], um) {
                            let (bot, top) = ((lo.at(b1), lo.at(b0)), (hi.at(b1), hi.at(b0)));
                            s.faces.extend(self.slanted_face((b1, v_lo), (b0, v_lo), bot, top, up_n, curved));
                        }
                    }
                }
            }
            // Each band's faces on the boundary with the next column where only one side is solid.
            if let Some(next) = self.columns.get(ci + 1) {
                let u = c.left.1;
                let along = flat(self.tangent(u));
                for (j, band) in bands.iter().enumerate() {
                    let mine = heights(&lines[j], u);
                    let theirs = heights(&self.extent_lines(next, band.extent), u);
                    let minus = |a: &[(f64, f64)], b: &[(f64, f64)]| -> Vec<(f64, f64)> {
                        b.iter().fold(a.to_vec(), |acc, &cut| acc.iter().flat_map(|&i| subtract(&[i], cut)).collect())
                    };
                    for (z0, z1) in minus(&mine, &theirs) {
                        let face = self.slanted_face((u, band.v_lo), (u, band.v_hi), (z0, z0), (z1, z1), along, false);
                        s.faces.extend(face);
                    }
                    for (z0, z1) in minus(&theirs, &mine) {
                        let face = self.slanted_face((u, band.v_hi), (u, band.v_lo), (z0, z0), (z1, z1), -along, false);
                        s.faces.extend(face);
                    }
                }
            }
        }
        self.recess_faces(&mut s);
        // End caps, omitted where another wall closes the joint: each band's piece of the cut.
        let cap = |s: &mut Solid, col: &Column, (ul, ur): (f64, f64), outward: f64| {
            let e = self.world(ul, h) - self.world(ur, -h);
            let mut n = e.perp().normalize_or_zero();
            if n.dot(self.tangent((ul + ur) * 0.5) * outward) < 0.0 {
                n = -n;
            }
            let n3 = flat(n);
            let across = |v: f64| if h > 1e-12 { ur + (ul - ur) * (v + h) / (2.0 * h) } else { ul };
            for band in &bands {
                let u_lo = if band.v_lo <= -h { ur } else { across(band.v_lo) };
                let u_hi = if band.v_hi >= h { ul } else { across(band.v_hi) };
                for (lo, hi) in self.extent_lines(col, band.extent) {
                    let (bot, top) = ((lo.at(u_lo), lo.at(u_hi)), (hi.at(u_lo), hi.at(u_hi)));
                    s.faces.extend(self.slanted_face((u_lo, band.v_lo), (u_hi, band.v_hi), bot, top, n3, false));
                }
            }
        };
        if !self.start_joined
            && let Some(c) = self.columns.first()
        {
            cap(&mut s, c, self.start_ext, -1.0);
        }
        if !self.end_joined
            && let Some(c) = self.columns.last()
        {
            cap(&mut s, c, self.end_ext, 1.0);
        }
        s
    }
}
