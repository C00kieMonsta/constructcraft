//! Wall elevation profiles: the outline of a wall's face drawn in its own
//! elevation plane, so its base and top need not be level (a basement wall
//! whose top follows the ground, a stepped base, a gable end, an arched top).
//!
//! The frame: `u`, meters along the wall's drawn location line from its
//! start (for a straight wall the same as along its body centerline, so
//! doors and windows share it), and `z`, meters above the wall's base
//! ([`crate::Project::wall_base_z`]). A profile keeps its outline (arcs split
//! into chords every [`crate::arc::ARC_STEP`]) and, as floors do, the curves
//! it was drawn with ([`Sketch`]); the outline is what the geometry is built
//! from, and a sketch that no longer makes it is dropped.
//!
//! The outline is one closed loop that does not cross itself and is
//! monotone along the wall: every vertical line inside its extent crosses it
//! twice, so it is a bottom chain and a top chain between its start and its
//! end ([`Chains`]; vertical ends and steps are allowed). It has no holes
//! (doors and windows open a wall), lies within the wall's length, and
//! reaches no further than [`MAX_BELOW`] below and [`MAX_ABOVE`] above the
//! base. Only straight walls that are not curtain walls take one, and a wall
//! with a profile is not attached to a roof (repair keeps the profile and
//! drops the attachment).
//!
//! With a profile the wall's body is the outline extruded across its
//! thickness: the profile replaces `height` and the top constraint for the
//! body's shape, which stay its nominal box. When the wall's length changes
//! the profile stays where it was along the wall (a moved start shifts it
//! back), what falls past the wall's ends is cut off at them, and a profile
//! that reached an end follows that end when the wall grows there
//! ([`WallProfile::refit`]); a profile left with nothing is dropped.

use crate::sketch::{JOIN_TOL, Sketch, SketchCurve, SketchError};
use crate::transform::PlanTransform;
use glam::DVec2;
use serde::{Deserialize, Serialize};

/// Most points an outline holds.
pub const MAX_POINTS: usize = 4096;

/// The lowest a profile reaches below the wall's base, meters.
pub const MAX_BELOW: f64 = 100.0;

/// The highest a profile reaches above the wall's base, meters.
pub const MAX_ABOVE: f64 = 1000.0;

/// The least extent of a profile along the wall, meters.
pub const MIN_LENGTH: f64 = 0.01;

/// The least area a profile encloses, square meters.
pub const MIN_AREA: f64 = 1e-4;

/// Points closer than this to each other or to the wall's ends meet them.
pub const TOL: f64 = JOIN_TOL;

/// The outline of a wall's face in its elevation frame (see the module).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WallProfile {
    /// The outline as `[u, z]` points, counter-clockwise, without the
    /// closing repeat.
    pub outline: Vec<DVec2>,
    /// The curves the outline was drawn with, in the same frame (none: its sides).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sketch: Option<Sketch>,
}

/// Why an outline makes no wall profile.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProfileError {
    /// The curves make no single closed loop.
    Sketch(SketchError),
    /// Fewer than three points, or numbers that are not.
    Invalid,
    /// Too many points.
    TooLarge,
    /// A vertical line crosses the outline more than twice near here.
    NotMonotone(DVec2),
    /// The outline runs past the wall's ends or the height bounds here.
    OutOfBounds(DVec2),
    /// The outline encloses (almost) nothing.
    NoArea,
}

impl ProfileError {
    /// Translation key of the message.
    pub fn key(&self) -> &'static str {
        match self {
            ProfileError::Sketch(e) => e.key(),
            ProfileError::Invalid => "wall-profile-invalid",
            ProfileError::TooLarge => "wall-profile-too-large",
            ProfileError::NotMonotone(_) => "wall-profile-not-monotone",
            ProfileError::OutOfBounds(_) => "wall-profile-out-of-bounds",
            ProfileError::NoArea => "wall-profile-no-area",
        }
    }

    /// Where it happens, to show it.
    pub fn point(&self) -> Option<DVec2> {
        match *self {
            ProfileError::Sketch(e) => e.point(),
            ProfileError::NotMonotone(p) | ProfileError::OutOfBounds(p) => Some(p),
            _ => None,
        }
    }
}

impl std::fmt::Display for ProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let at = |p: &DVec2| format!(" at u = {:.3}, z = {:.3}", p.x, p.y);
        match self {
            ProfileError::Sketch(SketchError::Openings(p)) => {
                write!(f, "a wall profile has no holes (doors and windows open walls){}", at(p))
            }
            ProfileError::Sketch(e) => e.fmt(f),
            ProfileError::Invalid => write!(f, "the profile needs at least three points with finite coordinates"),
            ProfileError::TooLarge => write!(f, "the profile has more than {MAX_POINTS} points"),
            ProfileError::NotMonotone(p) => write!(
                f,
                "a vertical line crosses the profile more than twice{}: it must be one bottom and one top line along the wall",
                at(p)
            ),
            ProfileError::OutOfBounds(p) => write!(
                f,
                "the profile runs past the wall's ends or more than {MAX_BELOW} m below or {MAX_ABOVE} m above its base{}",
                at(p)
            ),
            ProfileError::NoArea => write!(f, "the profile encloses no area"),
        }
    }
}

/// A profile's outline as its two chains, each by increasing `u` from the
/// start of the profile to its end; consecutive points at one `u` are a
/// vertical step.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Chains {
    pub bottom: Vec<DVec2>,
    pub top: Vec<DVec2>,
}

impl Chains {
    /// Where the profile starts and ends along the wall.
    pub fn span(&self) -> (f64, f64) {
        let first = self.bottom.first().map_or(0.0, |p| p.x);
        let last = self.bottom.last().map_or(0.0, |p| p.x);
        (first, last)
    }

    /// The sloped and level pieces `(u0, u1, z0, z1)` of a chain (vertical
    /// steps left out), by increasing `u`.
    pub fn segments(chain: &[DVec2]) -> Vec<(f64, f64, f64, f64)> {
        chain.windows(2).filter(|w| w[1].x - w[0].x > TOL).map(|w| (w[0].x, w[1].x, w[0].y, w[1].y)).collect()
    }

    /// Whether the rectangle from `u0` to `u1` along the wall and `z0` to
    /// `z1` above its base lies inside the profile.
    pub fn holds(&self, u0: f64, u1: f64, z0: f64, z1: f64) -> bool {
        let (a, b) = self.span();
        if !(u0.is_finite() && u1.is_finite() && z0.is_finite() && z1.is_finite()) {
            return false;
        }
        if u0 < a - TOL || u1 > b + TOL {
            return false;
        }
        // The chains are straight between their points: their extremes over
        // the rectangle lie at its sides or at their points.
        let over = |chain: &[DVec2], test: &dyn Fn(f64) -> bool| {
            Self::segments(chain).into_iter().all(|(s0, s1, za, zb)| {
                let (lo, hi) = (s0.max(u0), s1.min(u1));
                if hi - lo <= TOL {
                    return true;
                }
                let z = |u: f64| za + (zb - za) * (u - s0) / (s1 - s0);
                test(z(lo)) && test(z(hi))
            })
        };
        over(&self.bottom, &|z| z <= z0 + TOL) && over(&self.top, &|z| z >= z1 - TOL)
    }

    /// The lowest and highest points of the profile above the wall's base.
    pub fn z_range(&self) -> (f64, f64) {
        let lo = self.bottom.iter().chain(&self.top).map(|p| p.y).fold(f64::INFINITY, f64::min);
        let hi = self.bottom.iter().chain(&self.top).map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
        (lo, hi)
    }
}

/// Signed area of a closed polygon (positive counter-clockwise).
fn signed_area(pts: &[DVec2]) -> f64 {
    let n = pts.len();
    (0..n).map(|i| pts[i].perp_dot(pts[(i + 1) % n])).sum::<f64>() / 2.0
}

/// The chains of a counter-clockwise outline, or why it has none: it must
/// rise along the wall from its leftmost lowest point to its end, then come
/// back along the top.
fn chains_of(pts: &[DVec2]) -> Result<Chains, ProfileError> {
    let n = pts.len();
    let umin = pts.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
    let umax = pts.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
    if !(umax - umin >= MIN_LENGTH) {
        return Err(ProfileError::NoArea);
    }
    let start = (0..n)
        .filter(|&i| pts[i].x <= umin + TOL)
        .min_by(|&a, &b| pts[a].y.total_cmp(&pts[b].y))
        .ok_or(ProfileError::Invalid)?;
    let at = |k: usize| pts[(start + k) % n];
    let mut bottom = vec![at(0)];
    let mut top: Vec<DVec2> = Vec::new();
    let mut rising = true;
    for k in 1..=n {
        let (prev, cur) = (at(k - 1), at(k));
        if rising {
            if cur.x >= prev.x - TOL {
                bottom.push(cur);
                continue;
            }
            // Turning back along the wall is only allowed at its end.
            if prev.x < umax - TOL {
                return Err(ProfileError::NotMonotone(prev));
            }
            rising = false;
            top.push(prev);
            top.push(cur);
        } else {
            if cur.x > prev.x + TOL {
                return Err(ProfileError::NotMonotone(prev));
            }
            top.push(cur);
        }
    }
    if rising {
        return Err(ProfileError::NoArea);
    }
    // The bottom chain ends at its first point at the end (the rest climbs
    // the end side); the top chain at its first point at the start.
    let b_end = bottom.iter().position(|p| p.x >= umax - TOL).ok_or(ProfileError::Invalid)?;
    bottom.truncate(b_end + 1);
    let t_end = top.iter().position(|p| p.x <= umin + TOL).ok_or(ProfileError::Invalid)?;
    top.truncate(t_end + 1);
    top.reverse();
    Ok(Chains { bottom, top })
}

/// `poly` clipped to the half plane where `inside` holds, the edges cut at
/// `cut` (Sutherland–Hodgman).
fn clip_half(poly: &[DVec2], inside: impl Fn(DVec2) -> bool, cut: impl Fn(DVec2, DVec2) -> DVec2) -> Vec<DVec2> {
    let n = poly.len();
    let mut out = Vec::with_capacity(n + 2);
    for i in 0..n {
        let (a, b) = (poly[i], poly[(i + 1) % n]);
        match (inside(a), inside(b)) {
            (true, true) => out.push(b),
            (true, false) => out.push(cut(a, b)),
            (false, true) => {
                out.push(cut(a, b));
                out.push(b);
            }
            (false, false) => {}
        }
    }
    out.dedup_by(|a, b| a.distance(*b) < 1e-9);
    if out.len() > 1 && out.first().zip(out.last()).is_some_and(|(a, b)| a.distance(*b) < 1e-9) {
        out.pop();
    }
    out
}

/// The polygon between the vertical lines `u = lo` and `u = hi`.
fn clip_polygon(poly: &[DVec2], lo: f64, hi: f64) -> Vec<DVec2> {
    let at_u = |u: f64| {
        move |a: DVec2, b: DVec2| {
            let t = if (b.x - a.x).abs() > 1e-15 { (u - a.x) / (b.x - a.x) } else { 0.0 };
            DVec2::new(u, a.y + (b.y - a.y) * t)
        }
    };
    let left = clip_half(poly, |p| p.x >= lo, at_u(lo));
    clip_half(&left, |p| p.x <= hi, at_u(hi))
}

/// `c` with the ends at `u = from` moved to `u = to` (lines and arcs; `None`
/// for other curves that touch that line).
fn stretch_curve(c: &SketchCurve, from: f64, to: f64) -> Option<SketchCurve> {
    let moved = |p: DVec2| if (p.x - from).abs() <= TOL { DVec2::new(to, p.y) } else { p };
    match *c {
        SketchCurve::Line { a, b } => Some(SketchCurve::Line { a: moved(a), b: moved(b) }),
        SketchCurve::Arc { a, b, bulge } => Some(SketchCurve::Arc { a: moved(a), b: moved(b), bulge }),
        ref other => other.points().iter().all(|p| (p.x - from).abs() > TOL).then(|| other.clone()),
    }
}

/// The parts of `c` between `u = lo` and `u = hi`; `None` when a curve that
/// crosses those lines cannot be cut (a whole circle or ellipse).
fn clip_curve(c: &SketchCurve, lo: f64, hi: f64) -> Option<Vec<SketchCurve>> {
    use crate::sketch_edit::{between, crossings, param, span};
    let pts = c.points();
    let umin = pts.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
    let umax = pts.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
    if umin >= lo - TOL && umax <= hi + TOL {
        return Some(vec![c.clone()]);
    }
    if umax <= lo + TOL || umin >= hi - TOL {
        return Some(Vec::new());
    }
    let end = span(c)?;
    let mut ts: Vec<f64> = Vec::new();
    for u in [lo, hi] {
        let line = SketchCurve::Line { a: DVec2::new(u, 0.0), b: DVec2::new(u, 1.0) };
        for x in crossings(c, &line) {
            if let Some(t) = param(c, x).filter(|t| *t > TOL && *t < end - TOL) {
                ts.push(t);
            }
        }
    }
    ts.sort_by(f64::total_cmp);
    ts.dedup_by(|a, b| (*a - *b).abs() <= TOL);
    let mut bounds = vec![0.0];
    bounds.extend(ts);
    bounds.push(end);
    let mut out = Vec::new();
    for w in bounds.windows(2) {
        let piece = between(c, w[0], w[1])?;
        // The piece's middle: the end of its first half.
        let (_, m) = between(c, w[0], (w[0] + w[1]) * 0.5)?.ends()?;
        if m.x >= lo - TOL && m.x <= hi + TOL {
            out.push(piece);
        }
    }
    Some(out)
}

/// The curves cut to between `u = lo` and `u = hi`, closed again along
/// those lines; `None` when that makes no closed outline.
fn clip_sketch(sketch: &Sketch, lo: f64, hi: f64) -> Option<Sketch> {
    let mut curves = Vec::new();
    for c in &sketch.curves {
        curves.extend(clip_curve(c, lo, hi)?);
    }
    // The ends left open lie on the cut lines: join them in pairs, bottom up.
    let ends: Vec<DVec2> = curves.iter().filter_map(SketchCurve::ends).flat_map(|(a, b)| [a, b]).collect();
    let open: Vec<DVec2> =
        ends.iter().copied().filter(|p| ends.iter().filter(|q| q.distance(*p) <= JOIN_TOL).count() == 1).collect();
    let mut closing = Vec::new();
    let mut used = 0;
    for u in [lo, hi] {
        let mut on: Vec<DVec2> = open.iter().copied().filter(|p| (p.x - u).abs() <= 10.0 * TOL).collect();
        on.sort_by(|a, b| a.y.total_cmp(&b.y));
        if on.len() % 2 != 0 {
            return None;
        }
        used += on.len();
        for pair in on.chunks(2) {
            if let [a, b] = pair
                && a.distance(*b) > JOIN_TOL
            {
                closing.push(SketchCurve::Line { a: *a, b: *b });
            }
        }
    }
    if used != open.len() {
        return None;
    }
    curves.extend(closing);
    Some(Sketch { curves })
}

impl WallProfile {
    /// The profile the curves draw for a wall `length` long (its outline is
    /// the curves' loop, arcs split into chords).
    pub fn from_sketch(sketch: Sketch, length: f64) -> Result<Self, ProfileError> {
        let outline = sketch.outline().map_err(ProfileError::Sketch)?;
        Self { outline, sketch: Some(sketch) }.fitted(length)
    }

    /// The profile of `outline` (points `[u, z]`, either way round) for a
    /// wall `length` long.
    pub fn from_outline(outline: Vec<DVec2>, length: f64) -> Result<Self, ProfileError> {
        Self { outline, sketch: None }.fitted(length)
    }

    /// A wall `length` long whose base runs straight from `base.0` at its
    /// start to `base.1` at its end and whose top from `top.0` to `top.1`
    /// (heights above the wall's base).
    pub fn sloped(length: f64, base: (f64, f64), top: (f64, f64)) -> Result<Self, ProfileError> {
        let outline = vec![
            DVec2::new(0.0, base.0),
            DVec2::new(length, base.1),
            DVec2::new(length, top.1),
            DVec2::new(0.0, top.0),
        ];
        let sketch = Sketch::from_polygons(&outline, &[]);
        Self::from_sketch(sketch, length)
    }

    /// The same profile, its outline counter-clockwise and its points at the
    /// wall's ends exactly there, checked for a wall `length` long.
    pub fn fitted(mut self, length: f64) -> Result<Self, ProfileError> {
        let n = self.outline.len();
        if n < 3 || !self.outline.iter().all(|p| p.is_finite()) || !length.is_finite() {
            return Err(ProfileError::Invalid);
        }
        if n > MAX_POINTS {
            return Err(ProfileError::TooLarge);
        }
        self.outline.dedup_by(|a, b| a.distance(*b) < 1e-9);
        if self.outline.len() > 1
            && self.outline.first().zip(self.outline.last()).is_some_and(|(a, b)| a.distance(*b) < 1e-9)
        {
            self.outline.pop();
        }
        if signed_area(&self.outline) < 0.0 {
            self.outline.reverse();
        }
        for p in &mut self.outline {
            if p.x.abs() <= TOL / 2.0 {
                p.x = 0.0;
            } else if (p.x - length).abs() <= TOL / 2.0 {
                p.x = length;
            }
        }
        self.check(length)?;
        if self.sketch.as_ref().is_some_and(|s| !s.is_valid() || !s.makes(&self.outline, &[])) {
            self.sketch = None;
        }
        Ok(self)
    }

    /// Whether the outline is a profile for a wall `length` long: its chains.
    pub fn check(&self, length: f64) -> Result<Chains, ProfileError> {
        let n = self.outline.len();
        if n < 3 || !self.outline.iter().all(|p| p.is_finite()) {
            return Err(ProfileError::Invalid);
        }
        if n > MAX_POINTS {
            return Err(ProfileError::TooLarge);
        }
        if let Some(p) =
            self.outline.iter().find(|p| p.x < -TOL || p.x > length + TOL || p.y < -MAX_BELOW || p.y > MAX_ABOVE)
        {
            return Err(ProfileError::OutOfBounds(*p));
        }
        // One loop that does not cross itself.
        Sketch::from_polygons(&self.outline, &[]).outline().map_err(ProfileError::Sketch)?;
        if signed_area(&self.outline).abs() < MIN_AREA {
            return Err(ProfileError::NoArea);
        }
        let ccw: Vec<DVec2> = if signed_area(&self.outline) < 0.0 {
            self.outline.iter().rev().copied().collect()
        } else {
            self.outline.clone()
        };
        chains_of(&ccw)
    }

    /// The bottom and top chains (`None` for an outline that is no profile).
    pub fn chains(&self) -> Option<Chains> {
        let ccw: Vec<DVec2> = if signed_area(&self.outline) < 0.0 {
            self.outline.iter().rev().copied().collect()
        } else {
            self.outline.clone()
        };
        chains_of(&ccw).ok()
    }

    /// Where the profile starts and ends along the wall.
    pub fn span(&self) -> (f64, f64) {
        let lo = self.outline.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
        let hi = self.outline.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
        (lo, hi)
    }

    /// The lowest and highest points above the wall's base.
    pub fn z_range(&self) -> (f64, f64) {
        let lo = self.outline.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
        let hi = self.outline.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
        (lo, hi)
    }

    /// The area of the wall's face: exact along the curves it was drawn with.
    pub fn area(&self) -> f64 {
        match self.sketch.as_ref().filter(|s| s.is_curved()).map(Sketch::area) {
            Some(Ok(a)) => a,
            _ => signed_area(&self.outline).abs(),
        }
    }

    /// The curves of the outline: its sketch, or lines along its sides.
    pub fn curves(&self) -> Sketch {
        self.sketch.clone().unwrap_or_else(|| Sketch::from_polygons(&self.outline, &[]))
    }

    /// Whether the rectangle from `u0` to `u1` along the wall and `z0` to
    /// `z1` above its base (a door or window) lies inside the profile.
    pub fn holds(&self, u0: f64, u1: f64, z0: f64, z1: f64) -> bool {
        self.chains().is_some_and(|c| c.holds(u0, u1, z0, z1))
    }

    /// The same profile for the wall walked the other way (`length` long).
    pub fn reversed(&self, length: f64) -> Self {
        let mut outline: Vec<DVec2> = self.outline.iter().map(|p| DVec2::new(length - p.x, p.y)).collect();
        // A mirror turns the loop the other way round.
        outline.reverse();
        let sketch = self.sketch.clone().map(|mut s| {
            s.transform(&PlanTransform::mirror_across(DVec2::new(length / 2.0, 0.0), DVec2::Y));
            s
        });
        Self { outline, sketch }
    }

    /// The profile of a wall `old_length` long whose start moved `shift`
    /// along it (forward positive) and which is now `new_length` long: it
    /// stays where it was along the wall; where it reached an end and the
    /// wall now runs on past it, it follows that end; what lies past the
    /// wall's ends is cut off at them. `None` when nothing is left.
    pub fn refit(&self, old_length: f64, shift: f64, new_length: f64) -> Option<Self> {
        if !(old_length.is_finite() && shift.is_finite() && new_length.is_finite()) || new_length < MIN_LENGTH {
            return None;
        }
        let (umin, umax) = self.span();
        let (old_start, old_end) = (-shift, old_length - shift);
        let (lo, hi) = (0.0, new_length);
        let grew_start = umin <= TOL && old_start > lo + TOL;
        let grew_end = umax >= old_length - TOL && old_end < hi - TOL;
        let fit_points = |pts: &[DVec2]| -> Vec<DVec2> {
            let moved: Vec<DVec2> = pts
                .iter()
                .map(|p| {
                    let u = p.x - shift;
                    let u = if grew_start && (u - old_start).abs() <= TOL {
                        lo
                    } else if grew_end && (u - old_end).abs() <= TOL {
                        hi
                    } else {
                        u
                    };
                    DVec2::new(u, p.y)
                })
                .collect();
            clip_polygon(&moved, lo, hi)
        };
        // Along the curves when they can follow, else along the outline.
        let along_sketch = self.sketch.as_ref().and_then(|s| {
            let mut s = s.clone();
            s.transform(&PlanTransform::translate(DVec2::new(-shift, 0.0)));
            let mut curves = Vec::with_capacity(s.curves.len());
            for c in &s.curves {
                let mut c = c.clone();
                if grew_start {
                    c = stretch_curve(&c, old_start, lo)?;
                }
                if grew_end {
                    c = stretch_curve(&c, old_end, hi)?;
                }
                curves.push(c);
            }
            let s = clip_sketch(&Sketch { curves }, lo, hi)?;
            Self::from_sketch(s, new_length).ok()
        });
        along_sketch.or_else(|| Self::from_outline(fit_points(&self.outline), new_length).ok())
    }

    /// The parts of the profile of a wall `length` long split `at` along
    /// it: before (from the wall's start) and after (from the split).
    pub fn split(&self, length: f64, at: f64) -> (Option<Self>, Option<Self>) {
        (self.refit(length, 0.0, at), self.refit(length, at, length - at))
    }

    /// Drop a sketch that no longer makes the outline (edited without it), or
    /// is invalid; returns whether it did.
    pub fn drop_stale_sketch(&mut self) -> bool {
        let stale = self.sketch.as_ref().is_some_and(|s| !s.is_valid() || !s.makes(&self.outline, &[]));
        if stale {
            self.sketch = None;
        }
        stale
    }

    /// Repair a profile read from a file for a wall `length` long: an
    /// invalid sketch goes, an outline past the wall's end is cut there.
    /// Returns the number of repairs, or `None` when the profile must go.
    pub fn repair(&mut self, length: f64) -> Option<usize> {
        let mut repairs = usize::from(self.drop_stale_sketch());
        if self.check(length).is_ok() {
            return Some(repairs);
        }
        // Points a little past the ends, or past the end of a wall made shorter outside this program.
        let in_bounds = self.outline.iter().all(|p| p.is_finite() && p.y >= -MAX_BELOW && p.y <= MAX_ABOVE);
        if in_bounds
            && self.outline.len() <= MAX_POINTS
            && let Some(fitted) = self.refit(length, 0.0, length)
        {
            *self = fitted;
            repairs += 1;
            return Some(repairs);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(x: f64, y: f64) -> DVec2 {
        DVec2::new(x, y)
    }

    fn line(a: DVec2, b: DVec2) -> SketchCurve {
        SketchCurve::Line { a, b }
    }

    fn poly(points: &[(f64, f64)]) -> Vec<DVec2> {
        points.iter().map(|&(x, y)| v(x, y)).collect()
    }

    /// The pieces of a chain, compared within a micrometer.
    fn same(chain: &[DVec2], want: &[(f64, f64, f64, f64)]) -> bool {
        let got = Chains::segments(chain);
        let near = |a: f64, b: f64| (a - b).abs() < 1e-6;
        got.len() == want.len()
            && got.iter().zip(want).all(|(a, b)| near(a.0, b.0) && near(a.1, b.1) && near(a.2, b.2) && near(a.3, b.3))
    }

    #[test]
    fn monotone_outlines_are_profiles() {
        // A sloped top, a stepped base, a gable and an arched top.
        let sloped = WallProfile::sloped(6.0, (0.0, 0.0), (3.0, 2.0)).unwrap();
        assert!((sloped.area() - 15.0).abs() < 1e-9);
        let c = sloped.chains().unwrap();
        assert_eq!(c.span(), (0.0, 6.0));
        assert!(same(&c.top, &[(0.0, 6.0, 3.0, 2.0)]));
        let stepped = WallProfile::from_outline(
            poly(&[(0.0, 0.0), (2.0, 0.0), (2.0, 0.5), (6.0, 0.5), (6.0, 3.0), (0.0, 3.0)]),
            6.0,
        )
        .unwrap();
        let c = stepped.chains().unwrap();
        assert!(same(&c.bottom, &[(0.0, 2.0, 0.0, 0.0), (2.0, 6.0, 0.5, 0.5)]));
        assert!((stepped.area() - 16.0).abs() < 1e-9);
        // Clockwise points are turned round.
        let gable = WallProfile::from_outline(poly(&[(0.0, 0.0), (0.0, 2.5), (3.0, 4.0), (6.0, 2.5), (6.0, 0.0)]), 6.0)
            .unwrap();
        assert!(signed_area(&gable.outline) > 0.0);
        let arch = Sketch {
            curves: vec![
                line(v(0.0, 0.0), v(4.0, 0.0)),
                line(v(4.0, 0.0), v(4.0, 2.0)),
                SketchCurve::Arc { a: v(4.0, 2.0), b: v(0.0, 2.0), bulge: 1.0 },
                line(v(0.0, 2.0), v(0.0, 0.0)),
            ],
        };
        let arch = WallProfile::from_sketch(arch, 4.0).unwrap();
        // The half circle on top is exact in the area.
        let exact = 8.0 + std::f64::consts::PI * 2.0;
        assert!((arch.area() - exact).abs() < 1e-9, "{}", arch.area());
        assert!(arch.sketch.is_some());
        // A triangle whose ends meet in points (no vertical sides).
        assert!(WallProfile::from_outline(poly(&[(0.0, 1.0), (3.0, 0.0), (6.0, 1.0), (3.0, 4.0)]), 6.0).is_ok());
    }

    #[test]
    fn other_outlines_are_refused_with_the_point() {
        // A C open to the end: a vertical line crosses it four times.
        let c = poly(&[(0.0, 0.0), (4.0, 0.0), (4.0, 1.0), (1.0, 1.0), (1.0, 2.0), (4.0, 2.0), (4.0, 3.0), (0.0, 3.0)]);
        assert!(
            matches!(WallProfile::from_outline(c, 4.0), Err(ProfileError::NotMonotone(p)) if p == v(1.0, 2.0) || p == v(1.0, 1.0))
        );
        // An overhang turning back along the top.
        let hook = poly(&[(0.0, 0.0), (4.0, 0.0), (4.0, 3.0), (2.0, 3.0), (3.0, 4.0), (0.0, 4.0)]);
        assert!(matches!(WallProfile::from_outline(hook, 4.0), Err(ProfileError::NotMonotone(_))));
        // A bow tie crosses itself.
        let bow = poly(&[(0.0, 0.0), (4.0, 3.0), (4.0, 0.0), (0.0, 3.0)]);
        assert!(matches!(WallProfile::from_outline(bow, 4.0), Err(ProfileError::Sketch(SketchError::Crossing(_)))));
        // A hole.
        let mut holed = Sketch::from_polygons(&poly(&[(0.0, 0.0), (4.0, 0.0), (4.0, 3.0), (0.0, 3.0)]), &[]);
        holed
            .curves
            .extend(Sketch::from_polygons(&poly(&[(1.0, 1.0), (2.0, 1.0), (2.0, 2.0), (1.0, 2.0)]), &[]).curves);
        let err = WallProfile::from_sketch(holed, 4.0).unwrap_err();
        assert!(matches!(err, ProfileError::Sketch(SketchError::Openings(_))), "{err:?}");
        assert!(err.to_string().contains("no holes"));
        // An open outline.
        let open = Sketch { curves: vec![line(v(0.0, 0.0), v(4.0, 0.0)), line(v(4.0, 0.0), v(4.0, 3.0))] };
        assert!(matches!(WallProfile::from_sketch(open, 4.0), Err(ProfileError::Sketch(SketchError::OpenEnd(_)))));
        // Past the wall's end, or far below its base.
        let long = poly(&[(0.0, 0.0), (5.0, 0.0), (5.0, 3.0), (0.0, 3.0)]);
        assert!(matches!(WallProfile::from_outline(long, 4.0), Err(ProfileError::OutOfBounds(p)) if p.x == 5.0));
        let deep = poly(&[(0.0, -200.0), (4.0, 0.0), (4.0, 3.0), (0.0, 3.0)]);
        assert!(matches!(WallProfile::from_outline(deep, 4.0), Err(ProfileError::OutOfBounds(_))));
        // Nothing to it.
        assert_eq!(WallProfile::from_outline(poly(&[(0.0, 0.0), (1.0, 0.0)]), 4.0), Err(ProfileError::Invalid));
        let flat = poly(&[(0.0, 0.0), (0.01, 0.0), (0.01, 0.005), (0.0, 0.005)]);
        assert_eq!(WallProfile::from_outline(flat, 4.0), Err(ProfileError::NoArea));
        let nan = poly(&[(0.0, 0.0), (4.0, f64::NAN), (0.0, 3.0)]);
        assert_eq!(WallProfile::from_outline(nan, 4.0), Err(ProfileError::Invalid));
    }

    #[test]
    fn rectangles_inside_the_profile_fit() {
        let p = WallProfile::sloped(6.0, (0.0, 0.0), (3.0, 2.0)).unwrap();
        // A door 2.1 high near the high end fits; near the low end it does not.
        assert!(p.holds(0.5, 1.5, 0.0, 2.1));
        assert!(!p.holds(4.5, 5.5, 0.0, 2.1));
        assert!(!p.holds(5.5, 6.5, 0.0, 1.0), "past the end");
        let stepped = WallProfile::from_outline(
            poly(&[(0.0, 0.0), (2.0, 0.0), (2.0, 0.5), (6.0, 0.5), (6.0, 3.0), (0.0, 3.0)]),
            6.0,
        )
        .unwrap();
        assert!(stepped.holds(0.5, 2.0, 0.0, 2.0), "up to the step");
        assert!(!stepped.holds(1.5, 2.5, 0.2, 2.0), "across the step");
        assert!(stepped.holds(2.5, 3.5, 0.5, 2.5));
    }

    #[test]
    fn mirrored_and_moved_profiles() {
        let p = WallProfile::sloped(6.0, (0.0, 0.5), (3.0, 2.0)).unwrap();
        let r = p.reversed(6.0);
        let c = r.chains().unwrap();
        assert!(same(&c.top, &[(0.0, 6.0, 2.0, 3.0)]));
        assert!(same(&c.bottom, &[(0.0, 6.0, 0.5, 0.0)]));
        assert!(r.sketch.as_ref().is_some_and(|s| s.makes(&r.outline, &[])));
        assert_eq!(r.reversed(6.0).chains(), p.chains());
    }

    #[test]
    fn length_changes_cut_and_follow_the_ends() {
        let p = WallProfile::sloped(6.0, (0.0, 0.0), (3.0, 2.0)).unwrap();
        // Shorter: cut at the new end, the slope kept.
        let short = p.refit(6.0, 0.0, 3.0).unwrap();
        let c = short.chains().unwrap();
        assert_eq!(c.span(), (0.0, 3.0));
        assert!(same(&c.top, &[(0.0, 3.0, 3.0, 2.5)]), "{:?}", c.top);
        assert!(short.sketch.as_ref().is_some_and(|s| s.makes(&short.outline, &[])), "the curves are cut too");
        // Longer: the end follows the wall's end.
        let long = p.refit(6.0, 0.0, 8.0).unwrap();
        assert!(same(&long.chains().unwrap().top, &[(0.0, 8.0, 3.0, 2.0)]));
        // The start moved forward 1 m: the profile stays where it was along the wall.
        let trimmed = p.refit(6.0, 1.0, 5.0).unwrap();
        assert!(same(&trimmed.chains().unwrap().top, &[(0.0, 5.0, 3.0 - 1.0 / 6.0, 2.0)]));
        // The start moved back 1 m: the profile's start follows it.
        let extended = p.refit(6.0, -1.0, 7.0).unwrap();
        assert!(same(&extended.chains().unwrap().top, &[(0.0, 7.0, 3.0, 2.0)]));
        // A profile short of the end stays short when the wall grows.
        let part = WallProfile::from_outline(poly(&[(1.0, 0.0), (4.0, 0.0), (4.0, 3.0), (1.0, 3.0)]), 6.0).unwrap();
        assert_eq!(part.refit(6.0, 0.0, 9.0).unwrap().span(), (1.0, 4.0));
        // Nothing left.
        assert_eq!(part.refit(6.0, 0.0, 0.5), None);
        // Split: each part from its own start.
        let (a, b) = p.split(6.0, 2.0);
        assert_eq!(a.unwrap().span(), (0.0, 2.0));
        let b = b.unwrap();
        assert_eq!(b.span(), (0.0, 4.0));
        assert!(same(&b.chains().unwrap().top, &[(0.0, 4.0, 3.0 - 2.0 / 6.0, 2.0)]));
    }

    #[test]
    fn arcs_are_cut_on_their_circle() {
        let arch = Sketch {
            curves: vec![
                line(v(0.0, 0.0), v(4.0, 0.0)),
                line(v(4.0, 0.0), v(4.0, 2.0)),
                SketchCurve::Arc { a: v(4.0, 2.0), b: v(0.0, 2.0), bulge: 1.0 },
                line(v(0.0, 2.0), v(0.0, 0.0)),
            ],
        };
        let arch = WallProfile::from_sketch(arch, 4.0).unwrap();
        let half = arch.refit(4.0, 0.0, 2.0).unwrap();
        let s = half.sketch.as_ref().expect("the curves are kept");
        assert!(s.curves.iter().any(|c| matches!(c, SketchCurve::Arc { .. })));
        // A quarter circle of radius 2 over a 2 x 2 square.
        let exact = 4.0 + std::f64::consts::PI;
        assert!((half.area() - exact).abs() < 1e-9, "{}", half.area());
    }

    #[test]
    fn hostile_profiles_are_repaired_or_dropped() {
        let mut nan = WallProfile { outline: poly(&[(0.0, 0.0), (4.0, f64::NAN), (0.0, 3.0)]), sketch: None };
        assert_eq!(nan.repair(4.0), None);
        let mut huge = WallProfile { outline: poly(&[(0.0, 0.0), (4.0, 0.0), (4.0, 1e9), (0.0, 3.0)]), sketch: None };
        assert_eq!(huge.repair(4.0), None);
        let mut many = WallProfile { outline: vec![v(0.0, 0.0); MAX_POINTS + 1], sketch: None };
        assert_eq!(many.repair(4.0), None);
        // A wall made shorter: cut at its end.
        let mut long = WallProfile { outline: poly(&[(0.0, 0.0), (6.0, 0.0), (6.0, 3.0), (0.0, 3.0)]), sketch: None };
        assert_eq!(long.repair(4.0), Some(1));
        assert_eq!(long.span(), (0.0, 4.0));
        // A sketch that draws something else goes; the outline stays.
        let mut stale = WallProfile::sloped(4.0, (0.0, 0.0), (3.0, 3.0)).unwrap();
        stale.outline[2].y = 2.0;
        assert_eq!(stale.repair(4.0), Some(1));
        assert!(stale.sketch.is_none());
        assert_eq!(stale.repair(4.0), Some(0));
    }
}

/// Profiles on walls of a project: edits, mirrors, splits and repair.
#[cfg(test)]
mod project_tests {
    use super::*;
    use crate::{Category, Document, Element, ElementId, ElementKind, Project, Wall};

    fn doc_with_wall(wall: Wall) -> (Document, ElementId) {
        let mut doc = Document::new(Project::from_template());
        let level = doc.project.levels[0].id;
        let type_id = doc.project.default_type(Category::Wall).unwrap();
        let id = doc.edit("wall", |e| {
            let id = ElementId(e.alloc_id());
            e.add_element(Element {
                id,
                level,
                type_id,
                mark: String::new(),
                comments: String::new(),
                kind: ElementKind::Wall(wall),
                uid: Default::default(),
                phase_created: Default::default(),
                phase_demolished: None,
                design_option: None,
                paints: Vec::new(),
            })
        });
        (doc, id)
    }

    fn sloped_wall() -> Wall {
        Wall {
            start: DVec2::ZERO,
            end: DVec2::new(6.0, 0.0),
            profile: Some(WallProfile::sloped(6.0, (0.0, 0.0), (3.0, 2.0)).unwrap()),
            ..Default::default()
        }
    }

    fn profile(doc: &Document, id: ElementId) -> Option<WallProfile> {
        doc.project.element(id).and_then(|e| e.as_wall()).and_then(|w| w.profile.clone())
    }

    fn top(doc: &Document, id: ElementId) -> Vec<(f64, f64, f64, f64)> {
        Chains::segments(&profile(doc, id).unwrap().chains().unwrap().top)
    }

    fn near(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn moving_an_end_cuts_or_stretches_the_profile_in_the_same_step() {
        let (mut doc, id) = doc_with_wall(sloped_wall());
        // The end pulled back to 3 m: cut there, the slope kept.
        doc.edit("end", |e| e.move_endpoint(id, false, DVec2::new(3.0, 0.0))).unwrap();
        let t = top(&doc, id);
        assert!(t.len() == 1 && near(t[0].1, 3.0) && near(t[0].3, 2.5), "{t:?}");
        // The start pushed forward 1 m along the wall: the profile stays where it was.
        doc.edit("start", |e| e.move_endpoint(id, true, DVec2::new(1.0, 0.0))).unwrap();
        let t = top(&doc, id);
        assert!(t.len() == 1 && near(t[0].0, 0.0) && near(t[0].1, 2.0), "{t:?}");
        assert!(near(t[0].2, 3.0 - 1.0 / 6.0) && near(t[0].3, 2.5), "{t:?}");
        // The end pulled out again: the profile reached it, so it follows.
        doc.edit("end", |e| e.move_endpoint(id, false, DVec2::new(7.0, 0.0))).unwrap();
        let t = top(&doc, id);
        assert!(t.len() == 1 && near(t[0].1, 6.0) && near(t[0].3, 2.5), "{t:?}");
        // One undo step each.
        doc.undo();
        doc.undo();
        doc.undo();
        assert_eq!(profile(&doc, id), sloped_wall().profile);
        // Moving the whole wall keeps the profile as it was.
        doc.edit("move", |e| e.transform_elements(&[id], &PlanTransform::translate(DVec2::new(2.0, 0.0))));
        assert_eq!(profile(&doc, id), sloped_wall().profile);
    }

    #[test]
    fn a_mirrored_wall_walks_its_profile_the_other_way() {
        let (mut doc, id) = doc_with_wall(sloped_wall());
        doc.edit("mirror", |e| e.transform_elements(&[id], &PlanTransform::mirror_across(DVec2::ZERO, DVec2::Y)));
        let w = doc.project.element(id).and_then(|e| e.as_wall()).unwrap().clone();
        // The wall now runs from (-6, 0) to (0, 0): its high end, still at x = 0, is its far end.
        assert!(w.start.distance(DVec2::new(-6.0, 0.0)) < 1e-9 && w.end.distance(DVec2::ZERO) < 1e-9, "{w:?}");
        let t = top(&doc, id);
        assert!(t.len() == 1 && near(t[0].2, 2.0) && near(t[0].3, 3.0), "{t:?}");
    }

    #[test]
    fn a_split_wall_splits_its_profile() {
        let (mut doc, id) = doc_with_wall(sloped_wall());
        let new = doc.edit("split", |e| e.split(id, DVec2::new(2.0, 0.0))).unwrap();
        let (a, b) = (top(&doc, id), top(&doc, new));
        assert!(a.len() == 1 && near(a[0].1, 2.0) && near(a[0].3, 3.0 - 2.0 / 6.0), "{a:?}");
        assert!(b.len() == 1 && near(b[0].1, 4.0) && near(b[0].2, 3.0 - 2.0 / 6.0) && near(b[0].3, 2.0), "{b:?}");
        assert_eq!(doc.project.repairs_needed(), 0);
    }

    #[test]
    fn walls_made_curved_or_curtain_walls_lose_their_profile() {
        let (mut doc, id) = doc_with_wall(sloped_wall());
        doc.edit("arc", |e| {
            if let Some(ElementKind::Wall(w)) = e.element_mut(id).map(|x| &mut x.kind) {
                w.bulge = 0.3;
            }
        });
        assert_eq!(profile(&doc, id), None);
        doc.undo();
        let curtain = doc.project.types.values().find(|t| t.curtain().is_some()).map(|t| t.id);
        if let Some(curtain) = curtain {
            doc.edit("type", |e| {
                if let Some(x) = e.element_mut(id) {
                    x.type_id = curtain;
                }
            });
            assert_eq!(profile(&doc, id), None);
        }
    }

    #[test]
    fn hostile_profiles_in_a_file_are_repaired() {
        let (mut doc, id) = doc_with_wall(sloped_wall());
        let arc = doc.edit("more", |e| {
            let mut x = e.project().element(id).unwrap().clone();
            x.id = ElementId(e.alloc_id());
            x.uid = Default::default();
            if let ElementKind::Wall(w) = &mut x.kind {
                w.bulge = 0.5;
            }
            e.add_element(x)
        });
        let roof = doc.edit("floor", |e| {
            let mut x = e.project().element(id).unwrap().clone();
            x.id = ElementId(e.alloc_id());
            x.uid = Default::default();
            x.type_id = e.project().default_type(Category::Floor).unwrap();
            let square = vec![DVec2::ZERO, DVec2::new(6.0, 0.0), DVec2::new(6.0, 6.0), DVec2::new(0.0, 6.0)];
            x.kind = ElementKind::Floor(crate::Floor::new(square, 3.0));
            Some(e.add_element(x))
        });
        // The edit dropped that one; put hostile profiles back as a file would hold them.
        let mut p = doc.project.clone();
        let set = |p: &mut Project, id: ElementId, f: &dyn Fn(&mut Wall)| {
            if let Some(ElementKind::Wall(w)) = p.elements.get_mut(&id).map(|e| &mut e.kind) {
                f(w);
            }
        };
        set(&mut p, arc, &|w| w.profile = sloped_wall().profile);
        set(&mut p, id, &|w| {
            w.attach_top = roof;
            if let Some(pr) = &mut w.profile {
                pr.outline[1].y = f64::NAN;
            }
        });
        let text = crate::to_ron(&p).unwrap();
        let (loaded, repairs) = crate::from_ron_with_repairs(&text).unwrap();
        assert!(repairs >= 2, "{repairs}");
        let wall = |id: ElementId| loaded.element(id).and_then(|e| e.as_wall()).unwrap().clone();
        assert_eq!(wall(arc).profile, None, "arc walls take none");
        assert_eq!(wall(id).profile, None, "not a number");
        // A good profile on an attached wall keeps the profile.
        set(&mut p, id, &|w| w.profile = sloped_wall().profile);
        set(&mut p, arc, &|w| w.profile = None);
        let (loaded, repairs) = crate::from_ron_with_repairs(&crate::to_ron(&p).unwrap()).unwrap();
        assert_eq!(repairs, 1);
        let w = loaded.element(id).and_then(|e| e.as_wall()).unwrap();
        assert!(w.profile.is_some() && w.attach_top.is_none());
        assert_eq!(loaded.repairs_needed(), 0);
    }
}
