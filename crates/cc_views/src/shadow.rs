//! The shadows a view's sun casts in projected views and plans, as exact
//! polygons (see [`crate::projected::render`]).
//!
//! Every opaque face of the model that is not edge-on to the sun casts; a
//! face turned to the sun catches. The shadow on a face is the union of the
//! casters standing in front of its plane, slid along the sun's rays onto
//! that plane and kept within the face ([`cc_geom::region::covered`]); the
//! ground catches them the same way on a level plane. A closed convex
//! solid (a box, a round column) casts as one polygon, the hull of its
//! points. What a view cuts away (a plan's model above its cut plane, a
//! section's in front of its line) casts nothing: the rest casts as it is
//! drawn, closed by its cut.
//!
//! Bounded: a grid over the casters seen from the sun finds those over a
//! face, and the faces that cast, the casters looked at, the points of one
//! union and the points of a whole view are limited; past a limit the view
//! gets no shadows at all.

use crate::projected::Frame;
use cc_core::{ElementId, VisualStyle};
use cc_geom::{ModelGeometry, Solid, clip_polygon, face_plane_segments, polygon, region};
use glam::{DVec2, DVec3};
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::ops::Range;

/// A face catches the sun's shadows when the cosine of its angle to the
/// sun is at least this (edge-on, shadows run without end).
const MIN_FACING: f64 = 0.01;
/// A face casts when the cosine of its angle to the sun is at least this
/// (edge-on, it covers nothing).
const MIN_CASTING: f64 = 1e-3;
/// What stands less than this in front of a face, meters, casts nothing on
/// it (faces touching it, its own solid's faces around it).
const LIFT: f64 = 1e-4;
/// A face less opaque than this casts nothing, as in the 3D view.
const OPAQUE: f64 = 0.999;
/// Faces smaller than this, square meters, catch nothing.
const MIN_AREA: f64 = 1e-8;
/// Most faces that cast in one view.
pub const MAX_CASTERS: usize = 200_000;
/// Most points of the casters over one face (or the ground).
pub const MAX_UNION_POINTS: usize = 60_000;
/// Most points of the casters over all the faces of one view.
pub const MAX_POINTS: usize = 600_000;
/// Most casters looked at over all the faces of one view.
pub const MAX_LOOKS: usize = 15_000_000;
/// Most cells along a side of the grid over the casters.
const GRID_SIDE: usize = 128;
/// A caster over more cells than this is kept apart, compared with every face.
const LARGE: usize = 256;
/// A solid of more faces than this is not tested for being convex.
const MAX_CONVEX_FACES: usize = 96;
/// Most comparisons made over one face to leave out polygons inside others.
const MAX_PRUNING: usize = 4_000_000;

/// The ground's shadows: a black veil this opaque (the 3D view's, `GROUND_SHADOW` in `cc_render`'s `sun.wgsl`).
const GROUND_SHADOW: f64 = 0.35;
/// What is left of an unlit face's color in a shadow, as displayed (the
/// consistent colors style; the 3D view's `UNLIT_SHADOW`, of linear light).
const UNLIT_SHADOW: f64 = 0.85;
/// A sun lower than this casts shadows on the ground no longer than it
/// would from this altitude, degrees (as the 3D view's ground).
const MIN_GROUND_ALTITUDE_DEG: f64 = 5.0;
/// Farthest the ground's shadows reach past the model, meters.
const MAX_GROUND_REACH: f64 = 5000.0;
/// The light a face of the textures style keeps turned away from the sun,
/// and what a face square to it adds.
pub(crate) const TEXTURES_AMBIENT: f64 = 0.62;
pub(crate) const TEXTURES_DIFFUSE: f64 = 0.38;

/// The elements a view's graphics keep out of its shadows.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Skip {
    /// Hidden in the view (by category, filter, element, phase, design
    /// option or temporarily): they neither cast nor catch shadows.
    pub hidden: BTreeSet<ElementId>,
    /// Made see-through by the view's graphics: they cast none.
    pub clear: BTreeSet<ElementId>,
}

/// The shadows the sun of a view casts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shadows<'a> {
    /// The height of the level plane that catches them around the model
    /// (see [`cc_core::Project::sun_ground_height`]); `None`: no ground.
    pub ground: Option<f64>,
    /// The elements left out of them.
    pub skip: Option<&'a Skip>,
}

/// The opacity (0 to 255) of the black veil that a cast shadow lays on a
/// face of `style` at `facing` (the cosine of its angle to the sun), so
/// that 2D views match the 3D view: in the shaded style the face shows
/// [`VisualStyle::SHADED_SHADOW`] under it, in the textures style it loses
/// the sun's light and as much again as the shaded style's shadow is darker
/// than its shade, in consistent colors it darkens as the 3D view's unlit
/// faces do. The shadow fades out where the face turns edge-on to the sun,
/// as in the 3D view. No shadows in the other styles.
pub fn veil(style: VisualStyle, facing: f64) -> u8 {
    let cos = if facing.is_nan() { 0.0 } else { facing.clamp(0.0, 1.0) };
    let (lit, dark) = match style {
        VisualStyle::Shaded => (VisualStyle::shaded_grey(cos), VisualStyle::SHADED_SHADOW),
        VisualStyle::Textures => (
            TEXTURES_AMBIENT + TEXTURES_DIFFUSE * cos,
            TEXTURES_AMBIENT * VisualStyle::SHADED_SHADOW / VisualStyle::SHADED_AMBIENT,
        ),
        VisualStyle::ConsistentColors => (1.0, UNLIT_SHADOW),
        VisualStyle::Wireframe | VisualStyle::HiddenLine => return 0,
    };
    let t = (cos / 0.05).clamp(0.0, 1.0);
    let fade = t * t * (3.0 - 2.0 * t);
    ((1.0 - dark / lit) * fade * 255.0).round().clamp(0.0, 255.0) as u8
}

/// The opacity (0 to 255) of the black veil of a shadow on the ground.
pub fn ground_veil() -> u8 {
    (GROUND_SHADOW * 255.0).round() as u8
}

/// A face that casts: its outline (what the view's cut leaves of it).
struct Caster<'a> {
    verts: Cow<'a, [DVec3]>,
    /// Its solid and face in the geometry (a cut's cap has no face).
    id: (usize, usize),
}

/// What casts as one: a face, or a closed convex solid, whose shadow is
/// the hull of its points.
struct Unit {
    /// Its faces among the casters (one, or a convex solid's).
    faces: Range<usize>,
    /// A convex solid's points (empty for a face).
    points: Vec<DVec3>,
    /// Its box seen from the sun, and its point nearest the sun (along it).
    lo: DVec2,
    hi: DVec2,
    top: f64,
}

/// What casts shadows in a view, seen from its sun.
pub(crate) struct Scene<'a> {
    /// Unit vector towards the sun.
    sun: DVec3,
    /// Two unit vectors square to it and to each other.
    axes: (DVec3, DVec3),
    casters: Vec<Caster<'a>>,
    units: Vec<Unit>,
    /// The grid over the units' boxes: origin, cell size, cells along a
    /// side, the units over each cell and those over many.
    origin: DVec2,
    cell: DVec2,
    side: usize,
    cells: Vec<Vec<u32>>,
    large: Vec<u32>,
    /// Units met by the current lookup carry its number.
    seen: Vec<u32>,
    lookup: u32,
    /// Points given to unions, and casters looked at over faces, so far.
    points: usize,
    looks: usize,
    /// A limit was passed: no shadows.
    failed: bool,
}

/// Two unit vectors square to unit `n` and to each other, `n` their cross product.
fn basis(n: DVec3) -> (DVec3, DVec3) {
    let helper = if n.z.abs() < 0.9 { DVec3::Z } else { DVec3::X };
    let u = n.cross(helper).normalize_or(DVec3::X);
    (u, n.cross(u))
}

fn box2(points: impl Iterator<Item = DVec2>) -> (DVec2, DVec2) {
    points.fold((DVec2::INFINITY, DVec2::NEG_INFINITY), |(lo, hi), p| (lo.min(p), hi.max(p)))
}

fn apart(a: (DVec2, DVec2), b: (DVec2, DVec2)) -> bool {
    a.0.x > b.1.x || b.0.x > a.1.x || a.0.y > b.1.y || b.0.y > a.1.y
}

/// Whether `solid` is a closed convex body: every edge of a face is walked
/// the other way by another face, and no point lies in front of a face.
fn convex(solid: &Solid) -> bool {
    let faces = &solid.faces;
    if faces.len() < 4 || faces.len() > MAX_CONVEX_FACES {
        return false;
    }
    let key = |p: DVec3| ((p.x * 1e6).round() as i64, (p.y * 1e6).round() as i64, (p.z * 1e6).round() as i64);
    let mut edges = std::collections::HashMap::new();
    for face in faces {
        let n = face.verts.len();
        for i in 0..n {
            let (a, b) = (key(face.verts[i]), key(face.verts[(i + 1) % n]));
            if a != b {
                *edges.entry((a, b)).or_insert(0usize) += 1;
            }
        }
    }
    if !edges.iter().all(|((a, b), n)| *n == 1 && edges.get(&(*b, *a)) == Some(&1)) {
        return false;
    }
    faces.iter().all(|f| {
        let Some(p0) = f.verts.first() else { return false };
        faces.iter().flat_map(|g| g.verts.iter()).all(|p| (*p - *p0).dot(f.normal) <= 1e-6)
    })
}

/// Whether the convex polygon `poly` (counter-clockwise) holds every point of `points`.
fn holds_all(poly: &[DVec2], points: &[DVec2]) -> bool {
    let n = poly.len();
    n >= 3 && points.iter().all(|p| (0..n).all(|i| (poly[(i + 1) % n] - poly[i]).perp_dot(*p - poly[i]) >= 0.0))
}

/// `poly` counter-clockwise when it is convex (else `None`).
fn as_convex(poly: &[DVec2]) -> Option<Vec<DVec2>> {
    let n = poly.len();
    let turn = |i: usize| (poly[(i + 1) % n] - poly[i]).perp_dot(poly[(i + 2) % n] - poly[(i + 1) % n]);
    if n >= 3 && (0..n).all(|i| turn(i) >= 0.0) {
        Some(poly.to_vec())
    } else if n >= 3 && (0..n).all(|i| turn(i) <= 0.0) {
        Some(poly.iter().rev().copied().collect())
    } else {
        None
    }
}

/// A caster slid onto a plane: its outline there, whether it is convex
/// (then counter-clockwise) and its box.
struct Polygon {
    pts: Vec<DVec2>,
    convex: bool,
    lo: DVec2,
    hi: DVec2,
}

/// What the casters lay on a plane.
enum Cast {
    /// These polygons (to unite).
    Polygons(Vec<Polygon>),
    /// One of them covers the whole face.
    All,
}

impl<'a> Scene<'a> {
    /// The casters of `geom` under a sun towards unit `sun`: the faces of
    /// the solids `shown` that `skip` leaves in, opaque and not edge-on to
    /// the sun, those of a closed convex solid together as one unit; with
    /// a `cut` plane (a point and the direction kept), only what lies
    /// beyond it, closed by the cut's caps. `None` when the sun is not a
    /// finite direction or more than [`MAX_CASTERS`] faces cast.
    pub fn new(
        geom: &'a ModelGeometry,
        sun: DVec3,
        skip: Option<&Skip>,
        shown: &dyn Fn(&Solid) -> bool,
        cut: Option<(DVec3, DVec3)>,
    ) -> Option<Self> {
        let sun = sun.try_normalize().filter(|s| s.is_finite())?;
        let axes = basis(sun);
        let mut casters: Vec<Caster<'a>> = Vec::new();
        let mut units: Vec<Unit> = Vec::new();
        let seen_from = |p: &DVec3| DVec2::new(p.dot(axes.0), p.dot(axes.1));
        let unit = |faces: Range<usize>, points: Vec<DVec3>, verts: &mut dyn Iterator<Item = DVec3>| {
            let (mut lo, mut hi, mut top) = (DVec2::INFINITY, DVec2::NEG_INFINITY, f64::NEG_INFINITY);
            for p in verts {
                let s = seen_from(&p);
                (lo, hi, top) = (lo.min(s), hi.max(s), top.max(p.dot(sun)));
            }
            Unit { faces, points, lo, hi, top }
        };
        let good = |verts: &[DVec3]| verts.len() >= 3 && verts.iter().all(|p| p.is_finite());
        let casts = |s: &Solid| {
            shown(s) && !skip.is_some_and(|k| k.hidden.contains(&s.element) || k.clear.contains(&s.element))
        };
        for (index, solid) in geom.solids.iter().enumerate().filter(|(_, s)| casts(s)) {
            let opaque = |f: usize| 1.0 - solid.paint_of(f).map_or(solid.transparency, |p| p.transparency) >= OPAQUE;
            let cut_face = |verts: &'a [DVec3]| match cut {
                None => Cow::Borrowed(verts),
                Some((point, keep)) => Cow::Owned(clip_polygon(verts, point, keep)),
            };
            let start = casters.len();
            if (0..solid.faces.len()).all(opaque) && convex(solid) {
                // One unit: the hull of what the cut leaves of it (its cap's corners are among the points).
                for (f, face) in solid.faces.iter().enumerate() {
                    let verts = cut_face(&face.verts);
                    if good(&verts) {
                        casters.push(Caster { verts, id: (index, f) });
                    }
                }
                let mut points: Vec<DVec3> = casters[start..].iter().flat_map(|c| c.verts.iter().copied()).collect();
                points.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)).then(a.z.total_cmp(&b.z)));
                points.dedup_by(|a, b| a.distance_squared(*b) < 1e-18);
                if points.len() >= 4 {
                    let made = unit(start..casters.len(), points.clone(), &mut points.iter().copied());
                    units.push(made);
                } else {
                    casters.truncate(start);
                }
            } else {
                for (f, face) in solid.faces.iter().enumerate() {
                    if face.normal.dot(sun).abs() < MIN_CASTING || !opaque(f) {
                        continue;
                    }
                    let verts = cut_face(&face.verts);
                    if good(&verts) {
                        casters.push(Caster { verts, id: (index, f) });
                    }
                }
                if let Some((point, keep)) = cut.filter(|(_, keep)| keep.dot(sun).abs() >= MIN_CASTING)
                    && solid.transparency <= 1.0 - OPAQUE
                {
                    let caps = caps(solid, point, keep).into_iter().filter(|c| good(c));
                    casters.extend(caps.map(|cap| Caster { verts: Cow::Owned(cap), id: (index, usize::MAX) }));
                }
                for (i, caster) in casters.iter().enumerate().skip(start) {
                    units.push(unit(i..i + 1, Vec::new(), &mut caster.verts.iter().copied()));
                }
            }
            if casters.len() > MAX_CASTERS {
                return None;
            }
        }
        // The grid: about a unit a cell, over their boxes.
        let (lo, hi) =
            units.iter().fold((DVec2::INFINITY, DVec2::NEG_INFINITY), |(lo, hi), c| (lo.min(c.lo), hi.max(c.hi)));
        let side = ((units.len() as f64).sqrt().ceil() as usize).clamp(1, GRID_SIDE);
        let size = if units.is_empty() { DVec2::ONE } else { (hi - lo).max(DVec2::splat(1e-6)) };
        let origin = if units.is_empty() { DVec2::ZERO } else { lo };
        let mut scene = Self {
            sun,
            axes,
            casters,
            seen: vec![0; units.len()],
            units,
            origin,
            cell: size / side as f64,
            side,
            cells: vec![Vec::new(); side * side],
            large: Vec::new(),
            lookup: 0,
            points: 0,
            looks: 0,
            failed: false,
        };
        for i in 0..scene.units.len() {
            let (a, z) = scene.span(scene.units[i].lo, scene.units[i].hi);
            if (z.0 - a.0 + 1) * (z.1 - a.1 + 1) > LARGE {
                scene.large.push(i as u32);
                continue;
            }
            for y in a.1..=z.1 {
                for x in a.0..=z.0 {
                    if let Some(c) = scene.cells.get_mut(y * side + x) {
                        c.push(i as u32);
                    }
                }
            }
        }
        Some(scene)
    }

    /// Whether a limit was passed: the view then draws no shadows.
    pub fn failed(&self) -> bool {
        self.failed
    }

    /// The cells from the one holding `lo` to the one holding `hi` (within the grid).
    fn span(&self, lo: DVec2, hi: DVec2) -> ((usize, usize), (usize, usize)) {
        let at = |p: DVec2| {
            let c = ((p - self.origin) / self.cell).floor();
            let clamp = |v: f64| if v.is_nan() { 0 } else { v.clamp(0.0, (self.side - 1) as f64) as usize };
            (clamp(c.x), clamp(c.y))
        };
        (at(lo), at(hi))
    }

    /// The units whose boxes, seen from the sun, may reach `lo`..`hi`: the
    /// large ones first.
    fn around(&mut self, lo: DVec2, hi: DVec2) -> Vec<usize> {
        self.lookup = self.lookup.wrapping_add(1);
        if self.lookup == 0 {
            self.seen.fill(0);
            self.lookup = 1;
        }
        let mut out: Vec<usize> = self.large.iter().map(|i| *i as usize).collect();
        let (a, z) = self.span(lo, hi);
        for y in a.1..=z.1 {
            for x in a.0..=z.0 {
                for &i in self.cells.get(y * self.side + x).into_iter().flatten() {
                    if let Some(s) = self.seen.get_mut(i as usize)
                        && *s != self.lookup
                    {
                        *s = self.lookup;
                        out.push(i as usize);
                    }
                }
            }
        }
        out
    }

    /// Count `points` given to a union; `false` (and no more shadows) past a limit.
    fn spend(&mut self, points: usize) -> bool {
        self.points = self.points.saturating_add(points);
        if points > MAX_UNION_POINTS || self.points > MAX_POINTS {
            self.failed = true;
        }
        !self.failed
    }

    /// What the units `around` lay on the plane through `p0` with unit
    /// `normal` (at `facing` to the sun), as `flat` maps it, of what stands
    /// in front of it and within the convex `window` (counter-clockwise),
    /// leaving out the face `own`; [`Cast::All`] as soon as one covers the
    /// whole `outline`. Polygons inside another are left out.
    fn cast(
        &self,
        around: &[usize],
        (p0, normal, facing): (DVec3, DVec3, f64),
        flat: &dyn Fn(DVec3) -> DVec2,
        own: (usize, usize),
        seen: Option<((DVec2, DVec2), f64)>,
        (window, outline): (&[DVec2], &[DVec2]),
    ) -> Cast {
        let sun = self.sun;
        let front = p0 + normal * LIFT;
        let bounds = box2(window.iter().copied());
        // Slid along the sun's rays onto the plane.
        let slid = |p: DVec3| flat(p - sun * ((p - p0).dot(normal) / facing));
        let mut polys: Vec<Polygon> = Vec::new();
        for unit in around.iter().filter_map(|i| self.units.get(*i)) {
            if seen.is_some_and(|(seen, low)| unit.top <= low + LIFT || apart((unit.lo, unit.hi), seen)) {
                continue;
            }
            let faces = self.casters.get(unit.faces.clone()).unwrap_or(&[]);
            let poly = if unit.points.is_empty() {
                // A face: what of it stands in front of the plane.
                let Some(c) = faces.first().filter(|c| c.id != own) else { continue };
                let part = clip_polygon(&c.verts, front, normal);
                if part.len() < 3 {
                    continue;
                }
                part.iter().map(|p| slid(*p)).collect::<Vec<DVec2>>()
            } else {
                // A convex solid: the hull of what stands in front of the plane.
                let height = |p: &DVec3| (*p - p0).dot(normal);
                let (lowest, highest) = unit
                    .points
                    .iter()
                    .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), p| (lo.min(height(p)), hi.max(height(p))));
                if highest <= LIFT {
                    continue;
                }
                let points: Vec<DVec2> = if lowest >= LIFT {
                    unit.points.iter().map(|p| slid(*p)).collect()
                } else {
                    faces.iter().flat_map(|c| clip_polygon(&c.verts, front, normal)).map(slid).collect()
                };
                polygon::convex_hull(&points)
            };
            if poly.len() < 3 || !poly.iter().all(|p| p.is_finite()) || apart(box2(poly.iter().copied()), bounds) {
                continue;
            }
            let convex = as_convex(&poly);
            if !outline.is_empty() && convex.as_ref().is_some_and(|c| holds_all(c, outline)) {
                return Cast::All;
            }
            // Only what lies in the window matters (and crosses the other polygons).
            let pts = crate::projected::clip_convex(convex.as_deref().unwrap_or(&poly), window);
            if pts.len() >= 3 && polygon::area(&pts) > MIN_AREA {
                let (lo, hi) = box2(pts.iter().copied());
                polys.push(Polygon { pts, convex: convex.is_some(), lo, hi });
            }
        }
        // A polygon inside a convex one adds nothing: from the largest down,
        // each is compared with the convex ones kept so far (of two alike,
        // the first stays), while the work allows.
        let mut order: Vec<usize> = (0..polys.len()).collect();
        let areas: Vec<f64> = polys.iter().map(|p| polygon::area(&p.pts)).collect();
        order.sort_by(|a, b| areas[*b].total_cmp(&areas[*a]).then(a.cmp(b)));
        let mut kept: Vec<usize> = Vec::new();
        let mut convex: Vec<usize> = Vec::new();
        let mut work = 0usize;
        for i in order {
            let a = &polys[i];
            let inside = |j: &usize| {
                let b = &polys[*j];
                b.lo.cmple(a.lo).all() && b.hi.cmpge(a.hi).all() && holds_all(&b.pts, &a.pts)
            };
            work = work.saturating_add(convex.len());
            if work <= MAX_PRUNING && convex.iter().any(inside) {
                continue;
            }
            kept.push(i);
            if a.convex {
                convex.push(i);
            }
        }
        if kept.len() < polys.len() {
            kept.sort_unstable();
            let mut slots: Vec<Option<Polygon>> = polys.into_iter().map(Some).collect();
            polys = kept.into_iter().filter_map(|i| slots.get_mut(i).and_then(Option::take)).collect();
        }
        Cast::Polygons(polys)
    }

    /// The region `polys` cover within `clip` (when they were not all cut
    /// to it already): rings. Convex polygons apart from one another are
    /// their own rings; the others are united ([`region::covered`]; a union
    /// that fails leaves no shadow here).
    fn rings(&mut self, polys: Vec<Polygon>, clip: Option<&[DVec2]>) -> Vec<Vec<DVec2>> {
        if polys.is_empty() {
            return Vec::new();
        }
        let simple = clip.is_none()
            && polys.iter().all(|p| p.convex)
            && (0..polys.len()).all(|i| (0..i).all(|j| apart((polys[i].lo, polys[i].hi), (polys[j].lo, polys[j].hi))));
        if simple {
            return polys.into_iter().map(|p| p.pts).collect();
        }
        let polys: Vec<Vec<DVec2>> = polys.into_iter().map(|p| p.pts).collect();
        if !self.spend(polys.iter().map(Vec::len).sum::<usize>() + clip.map_or(0, <[DVec2]>::len)) {
            return Vec::new();
        }
        region::covered(&polys, clip, MAX_UNION_POINTS).unwrap_or_default()
    }

    /// The shadows cast on the face `own` (its solid and face in the
    /// geometry) with outline `verts` and unit `normal`: rings in its
    /// plane (see [`region::covered`]). None on a face turned away from the
    /// sun or edge-on to it.
    pub fn on_face(&mut self, verts: &[DVec3], normal: DVec3, own: (usize, usize)) -> Vec<Vec<DVec3>> {
        let facing = normal.dot(self.sun);
        if self.failed || verts.len() < 3 || facing.is_nan() || facing < MIN_FACING {
            return Vec::new();
        }
        let p0 = verts[0];
        let (u, v) = basis(normal);
        let flat = |p: DVec3| DVec2::new((p - p0).dot(u), (p - p0).dot(v));
        let outline: Vec<DVec2> = verts.iter().map(|p| flat(*p)).collect();
        if !outline.iter().all(|p| p.is_finite()) || polygon::area(&outline) < MIN_AREA {
            return Vec::new();
        }
        // The casters are cut to the face when it is convex, else to its box (and the union to the face).
        let (lo, hi) = box2(outline.iter().copied());
        let window = as_convex(&outline);
        let cut_to_face = window.is_some();
        let window = window.unwrap_or_else(|| vec![lo, DVec2::new(hi.x, lo.y), hi, DVec2::new(lo.x, hi.y)]);
        let (e1, e2) = self.axes;
        let seen = box2(verts.iter().map(|p| DVec2::new(p.dot(e1), p.dot(e2))));
        let low = verts.iter().map(|p| p.dot(self.sun)).fold(f64::INFINITY, f64::min);
        let around = self.around(seen.0, seen.1);
        self.looks = self.looks.saturating_add(around.len());
        if self.looks > MAX_LOOKS {
            self.failed = true;
            return Vec::new();
        }
        let plane = (p0, normal, facing);
        let polys = match self.cast(&around, plane, &flat, own, Some((seen, low)), (&window, &outline)) {
            Cast::All => return vec![verts.to_vec()],
            Cast::Polygons(polys) => polys,
        };
        let rings = self.rings(polys, (!cut_to_face).then_some(&outline));
        rings.into_iter().map(|r| r.into_iter().map(|q| p0 + u * q.x + v * q.y).collect()).collect()
    }

    /// The shadows cast on the level plane at height `z` around a model
    /// within `bounds`: rings at that height, no farther from the model
    /// than the shadows of a sun [`MIN_GROUND_ALTITUDE_DEG`] up reach.
    pub fn on_ground(&mut self, z: f64, bounds: (DVec3, DVec3)) -> Vec<Vec<DVec3>> {
        let height = bounds.1.z - z;
        let finite = z.is_finite() && bounds.0.is_finite() && bounds.1.is_finite();
        if self.failed || !finite || self.sun.z < MIN_FACING || height <= LIFT {
            return Vec::new();
        }
        let reach = (height / MIN_GROUND_ALTITUDE_DEG.to_radians().tan()).min(MAX_GROUND_REACH);
        let (lo, hi) = (bounds.0.truncate() - reach, bounds.1.truncate() + reach);
        let window = [lo, DVec2::new(hi.x, lo.y), hi, DVec2::new(lo.x, hi.y)];
        let all: Vec<usize> = (0..self.units.len()).collect();
        let none = (usize::MAX, usize::MAX);
        let plane = (DVec3::new(0.0, 0.0, z), DVec3::Z, self.sun.z);
        let Cast::Polygons(polys) = self.cast(&all, plane, &|p| p.truncate(), none, None, (&window, &[])) else {
            return Vec::new();
        };
        let rings = self.rings(polys, None);
        rings.into_iter().map(|r| r.into_iter().map(|q| q.extend(z)).collect()).collect()
    }
}

/// The caps that close `solid` where the plane through `point` cuts it,
/// keeping the side `keep` points to: the loops of its section, each outer
/// one with the loops inside it as holes ([`polygon::keyhole`]).
fn caps(solid: &Solid, point: DVec3, keep: DVec3) -> Vec<Vec<DVec3>> {
    let frame = Frame::looking(point, keep);
    let segs: Vec<(DVec2, DVec2)> = solid
        .faces
        .iter()
        .flat_map(|f| face_plane_segments(f, point, keep))
        .map(|(a, b)| (frame.project(a), frame.project(b)))
        .collect();
    if segs.is_empty() {
        return Vec::new();
    }
    let loops: Vec<Vec<DVec2>> =
        crate::projected::chain_loops(&segs).into_iter().filter(|l| polygon::area(l) > MIN_AREA).collect();
    // A loop inside an odd number of others is a hole of the smallest of them.
    let depth: Vec<Vec<usize>> = (0..loops.len())
        .map(|i| (0..loops.len()).filter(|&j| j != i && polygon::contains(&loops[j], loops[i][0])).collect())
        .collect();
    let mut out = Vec::new();
    for (i, outer) in loops.iter().enumerate().filter(|(i, _)| depth[*i].len().is_multiple_of(2)) {
        let holes: Vec<Vec<DVec2>> = (0..loops.len())
            .filter(|&h| depth[h].len() == depth[i].len() + 1 && depth[h].contains(&i))
            .map(|h| loops[h].clone())
            .collect();
        let ring = if holes.is_empty() { Some(outer.clone()) } else { polygon::keyhole(outer, &holes) };
        // Holes that cannot be joined: the outline alone (a shadow where light passes).
        let ring = ring.unwrap_or_else(|| outer.clone());
        out.push(ring.into_iter().map(|p| frame.origin + frame.right * p.x + frame.up * p.y).collect());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Look, Prim};
    use cc_core::Category;

    fn solid(id: u64, lo: DVec3, hi: DVec3) -> Solid {
        let mut s = Solid::new(ElementId(id), Category::Wall, [200; 3]);
        s.add_box(DVec2::ZERO, DVec2::X, DVec2::Y, lo.x, hi.x, lo.y, hi.y, lo.z, hi.z);
        s
    }

    fn model(solids: Vec<Solid>) -> ModelGeometry {
        ModelGeometry { solids, ..Default::default() }
    }

    fn look(sun: DVec3, ground: Option<f64>) -> Look<'static> {
        Look { light: Some(sun.normalize()), shadows: Some(Shadows { ground, skip: None }), ..Look::plain() }
    }

    /// The shadows of a drawing: the black veils that belong to no element.
    fn veils(prims: &[Prim]) -> Vec<(Vec<DVec2>, u8)> {
        prims
            .iter()
            .filter_map(|p| match p {
                Prim::Fill { pts, color: [0, 0, 0, a], element: None, cut: false } => Some((pts.clone(), *a)),
                _ => None,
            })
            .collect()
    }

    fn bounds(pts: &[DVec2]) -> (DVec2, DVec2) {
        box2(pts.iter().copied())
    }

    fn plan(geom: &ModelGeometry, cut_z: f64, look: Look<'_>) -> Vec<Prim> {
        crate::plan::style_underlay(geom, cut_z, false, look, &Default::default())
    }

    fn south(geom: &ModelGeometry, look: Look<'_>) -> Vec<Prim> {
        let frame = Frame::looking(DVec3::ZERO, DVec3::Y);
        crate::projected::render(geom, &frame, None, look, &Default::default()).prims
    }

    /// The fill of the face of `element` holding `p`, as painted (the last one).
    fn face_at(prims: &[Prim], element: u64, p: DVec2) -> [u8; 4] {
        prims
            .iter()
            .rev()
            .find_map(|prim| match prim {
                Prim::Fill { pts, color, element: Some(e), .. } if e.0 == element && polygon::contains(pts, p) => {
                    Some(*color)
                }
                _ => None,
            })
            .unwrap()
    }

    /// A box 2 m high on the ground under a sun 45° up in the west: in a
    /// plan its shadow on the ground runs 2 m east of it, one ring with the
    /// box's width, in the ground's veil; under everything else.
    #[test]
    fn a_box_casts_its_shadow_on_the_ground_of_a_plan() {
        let geom = model(vec![solid(1, DVec3::ZERO, DVec3::new(1.0, 1.0, 2.0))]);
        let prims = plan(&geom, 10.0, look(DVec3::new(-1.0, 0.0, 1.0), Some(0.0)));
        let shadows = veils(&prims);
        assert_eq!(shadows.len(), 1, "{shadows:?}");
        let (ring, alpha) = &shadows[0];
        assert_eq!(*alpha, ground_veil());
        assert_eq!(ring.len(), 4, "{ring:?}");
        let (lo, hi) = bounds(ring);
        assert!(lo.distance(DVec2::new(LIFT, 0.0)) < 1e-6 && hi.distance(DVec2::new(3.0, 1.0)) < 1e-6, "{lo} {hi}");
        assert!((polygon::area(ring) - 3.0).abs() < 1e-3);
        assert!(matches!(&prims[0], Prim::Fill { element: None, .. }), "the ground's shadow comes first");
        // A lower ground: the shadow is longer and starts farther away.
        let low = veils(&plan(&geom, 10.0, look(DVec3::new(-1.0, 0.0, 1.0), Some(-1.0))));
        let (lo, hi) = bounds(&low[0].0);
        assert!(lo.distance(DVec2::new(1.0, 0.0)) < 1e-6 && hi.distance(DVec2::new(4.0, 1.0)) < 1e-6, "{lo} {hi}");
        // No ground plane, a ground above the plan's cut: no ground shadow.
        assert!(veils(&plan(&geom, 10.0, look(DVec3::new(-1.0, 0.0, 1.0), None))).is_empty());
        assert!(veils(&plan(&geom, 1.0, look(DVec3::new(-1.0, 0.0, 1.0), Some(1.5)))).is_empty());
    }

    /// A plan casts from what it shows: a box 3 m high cut at 1.2 m casts
    /// the shadow of a box 1.2 m high, closed by its cut.
    #[test]
    fn a_plan_casts_from_the_model_below_its_cut() {
        let geom = model(vec![solid(1, DVec3::ZERO, DVec3::new(1.0, 1.0, 3.0))]);
        let shadows = veils(&plan(&geom, 1.2, look(DVec3::new(-1.0, 0.0, 1.0), Some(0.0))));
        assert_eq!(shadows.len(), 1, "{shadows:?}");
        let (lo, hi) = bounds(&shadows[0].0);
        assert!(lo.distance(DVec2::new(LIFT, 0.0)) < 1e-6 && hi.distance(DVec2::new(2.2, 1.0)) < 1e-6, "{lo} {hi}");
        // The box is one caster, the hull of what is left of it: its corners at the cut among its points.
        let cut = Some((DVec3::new(0.0, 0.0, 1.2), -DVec3::Z));
        let scene = Scene::new(&geom, DVec3::Z, None, &|_| true, cut).unwrap();
        assert_eq!(scene.units.len(), 1);
        let top = scene.units[0].points.iter().filter(|p| (p.z - 1.2).abs() < 1e-9).count();
        assert_eq!((scene.units[0].points.len(), top), (8, 4));
        // A solid that is not convex (an L of two boxes) casts face by face, closed by the cut's cap.
        let mut l = solid(1, DVec3::ZERO, DVec3::new(1.0, 1.0, 3.0));
        l.faces.extend(solid(1, DVec3::new(1.0, 0.0, 0.0), DVec3::new(2.0, 0.5, 3.0)).faces);
        let geom = model(vec![l]);
        let scene = Scene::new(&geom, DVec3::new(-1.0, 0.0, 1.0), None, &|_| true, cut).unwrap();
        let caps: Vec<_> = scene.casters.iter().filter(|c| c.id.1 == usize::MAX).collect();
        assert!(!caps.is_empty() && scene.units.iter().all(|u| u.points.is_empty()));
        assert!(caps.iter().flat_map(|c| c.verts.iter()).all(|p| (p.z - 1.2).abs() < 1e-9));
        let flat = |c: &&Caster| polygon::area(&c.verts.iter().map(|p| p.truncate()).collect::<Vec<_>>());
        assert!((caps.iter().map(flat).sum::<f64>() - 1.5).abs() < 1e-9);
        let shadows = veils(&plan(&geom, 1.2, look(DVec3::new(-1.0, 0.0, 1.0), Some(0.0))));
        assert_eq!(shadows.len(), 1, "{shadows:?}");
        // The L's outline swept 1.2 m east: 3.2 m long along its foot, 2.2 m along its stem.
        assert!((polygon::area(&shadows[0].0) - 2.7).abs() < 1e-3, "{}", polygon::area(&shadows[0].0));
    }

    /// A slab under a box: the shadow on the slab's top is a veil that
    /// leaves the shaded style's shadow grey of the lit face, the 3D
    /// view's; the ground's shadow lies under the slab.
    #[test]
    fn a_shadow_on_a_face_shows_the_shadow_grey() {
        let slab = solid(1, DVec3::new(-5.0, -5.0, -0.2), DVec3::new(5.0, 5.0, 0.0));
        let geom = model(vec![slab, solid(2, DVec3::ZERO, DVec3::new(1.0, 1.0, 2.0))]);
        let sun = DVec3::new(-1.0, 0.0, 1.0).normalize();
        let prims = plan(&geom, 10.0, look(sun, Some(-0.2)));
        let lit = face_at(&prims, 1, DVec2::new(-3.0, -3.0));
        let grey = (VisualStyle::shaded_grey(sun.z) * 255.0).round();
        assert_eq!(lit, [grey as u8, grey as u8, grey as u8, 255]);
        let on_slab: Vec<_> =
            veils(&prims).into_iter().filter(|(_, a)| *a == veil(VisualStyle::Shaded, sun.z)).collect();
        assert_eq!(on_slab.len(), 1, "{:?}", veils(&prims));
        let (lo, hi) = bounds(&on_slab[0].0);
        assert!(lo.distance(DVec2::new(LIFT, 0.0)) < 1e-6 && hi.distance(DVec2::new(3.0, 1.0)) < 1e-6, "{lo} {hi}");
        let shown = grey * (1.0 - f64::from(on_slab[0].1) / 255.0);
        assert!((shown - VisualStyle::SHADED_SHADOW * 255.0).abs() <= 1.0, "{shown}");
        // After the slab's fill and before the box's faces, which cover its start.
        let at = |f: &dyn Fn(&Prim) -> bool| prims.iter().position(f).unwrap();
        let veil_at = at(&|p| matches!(p, Prim::Fill { pts, element: None, .. } if *pts == on_slab[0].0));
        let slab_at = at(&|p| matches!(p, Prim::Fill { element: Some(e), .. } if e.0 == 1));
        let box_at = at(&|p| matches!(p, Prim::Fill { element: Some(e), .. } if e.0 == 2));
        assert!(slab_at < veil_at && veil_at < box_at, "{slab_at} {veil_at} {box_at}");
        // The other styles: darker faces, each by its own veil; none in hidden line and wireframe.
        for (style, some) in [
            (VisualStyle::Textures, true),
            (VisualStyle::ConsistentColors, true),
            (VisualStyle::HiddenLine, false),
            (VisualStyle::Wireframe, false),
        ] {
            let found = veils(&plan(&geom, 10.0, Look { style, ..look(sun, Some(-0.2)) }));
            assert_eq!(!found.is_empty(), some, "{style:?}");
            assert!(found.iter().all(|(_, a)| *a == veil(style, sun.z) || *a == ground_veil()), "{style:?}");
            assert_eq!(veil(style, sun.z) > 0, some);
        }
    }

    /// A box 3 m high, 1 m in front of a wall, under a sun in the
    /// south-west as high as it is far along each axis, seen in the south
    /// elevation: its shadow on the wall is the pentagon the box's faces
    /// cast together, 3.5 m².
    #[test]
    fn a_box_casts_its_shadow_on_a_wall_in_an_elevation() {
        let wall = solid(1, DVec3::new(0.0, 1.0, 0.0), DVec3::new(10.0, 1.2, 5.0));
        let geom = model(vec![wall, solid(2, DVec3::new(4.0, -1.0, 0.0), DVec3::new(5.0, 0.0, 3.0))]);
        let sun = DVec3::new(-1.0, -1.0, 1.0).normalize();
        let prims = south(&geom, look(sun, Some(0.0)));
        let shadows = veils(&prims);
        assert_eq!(shadows.len(), 1, "{shadows:?}");
        let (ring, alpha) = &shadows[0];
        assert_eq!(*alpha, veil(VisualStyle::Shaded, sun.dot(-DVec3::Y)));
        assert_eq!(ring.len(), 5, "{ring:?}");
        assert!((polygon::area(ring) - 3.5).abs() < 1e-3, "{}", polygon::area(ring));
        for corner in [(5.0, 0.0), (7.0, 0.0), (7.0, 1.0), (6.0, 2.0), (5.0, 2.0)] {
            let c = DVec2::new(corner.0, corner.1);
            assert!(ring.iter().any(|p| p.distance(c) < 1e-3), "{c} in {ring:?}");
        }
        // The veil belongs to no element: the wall is picked through it.
        let d = crate::Drawing2D { prims, ..Default::default() };
        assert_eq!(d.hit_test(DVec2::new(6.5, 0.5), 0.01), Some(ElementId(1)));
    }

    /// Faces turned away from the sun keep their shade and get no shadow;
    /// no sun, a sun below the horizon and shadows off cast none.
    #[test]
    fn no_shadows_without_a_sun_that_casts_them() {
        let wall = solid(1, DVec3::new(0.0, 1.0, 0.0), DVec3::new(10.0, 1.2, 5.0));
        let geom = model(vec![wall, solid(2, DVec3::new(4.0, -1.0, 0.0), DVec3::new(5.0, 0.0, 3.0))]);
        // From the north: the south faces the elevation shows are turned away.
        let north = south(&geom, look(DVec3::new(1.0, 1.0, 1.0), Some(0.0)));
        assert!(veils(&north).is_empty());
        let ambient = (VisualStyle::SHADED_AMBIENT * 255.0).round() as u8;
        assert_eq!(face_at(&north, 1, DVec2::new(1.0, 1.0)), [ambient, ambient, ambient, 255]);
        let sun = DVec3::new(-1.0, -1.0, 1.0).normalize();
        let cases = [
            Look { shadows: None, ..look(sun, Some(0.0)) },
            Look { light: None, ..look(sun, Some(0.0)) },
            Look { light: Some(DVec3::ZERO), ..look(sun, Some(0.0)) },
        ];
        for case in cases {
            assert!(veils(&south(&geom, case)).is_empty(), "{case:?}");
            assert!(veils(&plan(&geom, 10.0, case)).is_empty(), "{case:?}");
        }
        assert_eq!(veil(VisualStyle::Shaded, 0.0), 0);
        assert_eq!(veil(VisualStyle::Shaded, -0.5), 0);
        assert_eq!(veil(VisualStyle::Shaded, f64::NAN), 0);
    }

    /// What the view hides neither casts nor catches; what it makes
    /// see-through, and glass, cast nothing.
    #[test]
    fn hidden_and_see_through_elements_cast_no_shadow() {
        let wall = solid(1, DVec3::new(0.0, 1.0, 0.0), DVec3::new(10.0, 1.2, 5.0));
        let front = solid(2, DVec3::new(4.0, -1.0, 0.0), DVec3::new(5.0, 0.0, 3.0));
        let geom = model(vec![wall.clone(), front.clone()]);
        let sun = DVec3::new(-1.0, -1.0, 1.0);
        let with = |skip: &Skip| {
            let base = look(sun, Some(0.0));
            let shadows = Some(Shadows { ground: Some(0.0), skip: Some(skip) });
            veils(&south(&geom, Look { shadows, ..base })).len()
        };
        assert_eq!(with(&Skip::default()), 1);
        let only = |id: u64| BTreeSet::from([ElementId(id)]);
        assert_eq!(with(&Skip { hidden: only(2), ..Default::default() }), 0, "the caster is hidden");
        assert_eq!(with(&Skip { clear: only(2), ..Default::default() }), 0, "the caster is see-through");
        assert_eq!(with(&Skip { hidden: only(1), ..Default::default() }), 0, "the face that catches is hidden");
        let glass = Solid { transparency: 0.6, ..front };
        assert!(veils(&south(&model(vec![wall, glass]), look(sun, Some(0.0)))).is_empty());
    }

    /// Past a limit there are no shadows at all, and nothing panics: too
    /// many faces make no scene, too many points stop it.
    #[test]
    fn past_a_limit_there_are_no_shadows() {
        let mut many = Solid::new(ElementId(1), Category::Wall, [200; 3]);
        let face = cc_geom::Face::new(vec![DVec3::ZERO, DVec3::X, DVec3::Y], DVec3::Z);
        many.faces = vec![face; MAX_CASTERS + 1];
        let geom = model(vec![many]);
        assert!(Scene::new(&geom, DVec3::Z, None, &|_| true, None).is_none());
        assert!(Scene::new(&geom, DVec3::new(f64::NAN, 0.0, 1.0), None, &|_| true, None).is_none());
        assert!(Scene::new(&geom, DVec3::ZERO, None, &|_| true, None).is_none());

        let slab = solid(1, DVec3::new(-5.0, -5.0, -0.2), DVec3::new(5.0, 5.0, 0.0));
        let geom = model(vec![slab, solid(2, DVec3::ZERO, DVec3::new(1.0, 1.0, 2.0))]);
        let top: Vec<DVec3> = geom.solids[0].faces[0].verts.clone();
        let mut scene = Scene::new(&geom, DVec3::new(-1.0, 0.0, 1.0), None, &|_| true, None).unwrap();
        assert_eq!(scene.on_face(&top, DVec3::Z, (0, 0)).len(), 1);
        assert!(!scene.failed());
        assert!(!scene.spend(MAX_UNION_POINTS + 1));
        assert!(scene.failed());
        assert!(scene.on_face(&top, DVec3::Z, (0, 0)).is_empty());
        assert!(scene.on_ground(-0.2, geom.bounds().unwrap()).is_empty());
        // Too many casters looked at over the faces of a view.
        let mut scene = Scene::new(&geom, DVec3::new(-1.0, 0.0, 1.0), None, &|_| true, None).unwrap();
        scene.looks = MAX_LOOKS;
        assert!(scene.on_face(&top, DVec3::Z, (0, 0)).is_empty() && scene.failed());
        // Faces that are no polygons, and an empty model, catch and cast nothing.
        let mut scene = Scene::new(&geom, DVec3::new(-1.0, 0.0, 1.0), None, &|_| true, None).unwrap();
        assert!(scene.on_face(&top[..2], DVec3::Z, (0, 0)).is_empty());
        assert!(scene.on_face(&[DVec3::ZERO, DVec3::X, DVec3::new(f64::NAN, 0.0, 0.0)], DVec3::Z, (9, 9)).is_empty());
        let empty = model(Vec::new());
        let mut scene = Scene::new(&empty, DVec3::Z, None, &|_| true, None).unwrap();
        assert!(scene.on_face(&top, DVec3::Z, (0, 0)).is_empty());
    }
}
