//! Spline walls: a wall whose location line is a smooth free-form curve, a
//! chain of cubic Bézier segments kept as their control points (the form of
//! [`crate::spline`], which sketches use too).
//!
//! The model keeps the control points ([`WallSpline`]); everything that
//! needs the curve's shape works on a [`WallPath`], the curve flattened
//! into chords with the true normal at each point. On it wall-local
//! coordinates are `u`, the distance along the chords from the start, and
//! `v`, the distance to the left: the point `(u, v)` lies between the
//! points `v` along the normals of the two stations around `u`, so a line
//! at a constant `v` (a face, a layer boundary) is the polyline through the
//! exact offsets of the stations.
//!
//! Tolerances: a chord strays at most [`SAGITTA`] from the location line
//! and turns at most [`MAX_TURN`]; lengths are the chords', short of the
//! true curve by about one part in ten thousand at most.

use crate::arc::ARC_STEP;
use crate::spline;
use glam::DVec2;
use serde::{Deserialize, Serialize};

/// Most Bézier segments a wall's spline holds.
pub const MAX_SEGMENTS: usize = spline::MAX_SEGMENTS;
/// The shortest spline wall, and the least its ends stand apart, meters.
pub const MIN_LENGTH: f64 = 0.05;
/// The farthest a control point lies from the origin, meters.
pub const MAX_COORD: f64 = 1.0e6;
/// The farthest a chord strays from the location line, meters.
pub const SAGITTA: f64 = 0.0005;
/// The most the curve turns over one chord, radians.
pub const MAX_TURN: f64 = 2.0 * ARC_STEP;
/// Most chords of a flattened spline (past them the tolerances loosen).
pub const MAX_CHORDS: usize = 2048;
/// The least radius left to the face on the inside of a bend, meters: a
/// spline bends no tighter than half the wall (the distance from the
/// location line to that face) plus this.
pub const MIN_FACE_RADIUS: f64 = 0.05;
/// The most the direction changes where two segments join, degrees: more
/// is a corner, which takes two walls.
pub const MAX_KINK_DEG: f64 = 1.0;

/// How deep a segment is halved while flattening.
const MAX_DEPTH: u32 = 10;

/// The path of a spline wall: the control points `p0 c1 c2 p1 c1 c2 p2 …`
/// of a chain of cubic Bézier segments (`3n + 1` points for `n` segments),
/// the wall's location line from its start to its end.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WallSpline {
    pub control: Vec<DVec2>,
}

/// Why a spline makes no wall.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SplineError {
    /// Not the control points of a spline (count, numbers, size).
    Points,
    /// Too short, or its ends meet.
    Short,
    /// The direction breaks where two segments join.
    Corner(DVec2),
    /// It bends tighter than the wall's thickness allows.
    Tight(DVec2),
    /// The wall would cross itself.
    Crossing(DVec2),
}

impl SplineError {
    /// Translation key of the message.
    pub fn key(&self) -> &'static str {
        match self {
            SplineError::Points => "wall-spline-points",
            SplineError::Short => "wall-spline-short",
            SplineError::Corner(_) => "wall-spline-corner",
            SplineError::Tight(_) => "wall-spline-tight",
            SplineError::Crossing(_) => "wall-spline-crossing",
        }
    }

    /// Where it happens, to show it.
    pub fn point(&self) -> Option<DVec2> {
        match *self {
            SplineError::Corner(p) | SplineError::Tight(p) | SplineError::Crossing(p) => Some(p),
            _ => None,
        }
    }
}

impl std::fmt::Display for SplineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let at = |p: &DVec2| format!(" at [{:.3}, {:.3}]", p.x, p.y);
        match self {
            SplineError::Points => write!(
                f,
                "a spline wall takes the control points of 1 to {MAX_SEGMENTS} cubic segments (3n + 1 finite points)"
            ),
            SplineError::Short => {
                write!(f, "the spline is too short, or its ends meet (a spline wall is open, {MIN_LENGTH} m at least)")
            }
            SplineError::Corner(p) => {
                write!(f, "the spline has a corner{}: its direction must carry on where segments join", at(p))
            }
            SplineError::Tight(p) => write!(
                f,
                "the spline bends too tightly for the wall's thickness{}: the face inside a bend needs a radius of {MIN_FACE_RADIUS} m at least",
                at(p)
            ),
            SplineError::Crossing(p) => write!(f, "the wall would cross itself{}", at(p)),
        }
    }
}

impl WallSpline {
    /// The spline through the `fit` points (see [`spline::interpolate`]);
    /// `None` with fewer than two distinct points.
    pub fn through(fit: &[DVec2]) -> Option<Self> {
        if fit.len() > MAX_SEGMENTS + 1 || !fit.iter().all(|p| p.is_finite()) {
            return None;
        }
        let control = spline::interpolate(fit, false);
        (!control.is_empty()).then_some(Self { control })
    }

    /// The control points of a spline: finite, within [`MAX_COORD`], a
    /// whole number of segments up to [`MAX_SEGMENTS`].
    pub fn is_valid(&self) -> bool {
        self.control.len() <= 3 * MAX_SEGMENTS + 1
            && spline::is_valid(&self.control)
            && self.control.iter().all(|p| p.x.abs() <= MAX_COORD && p.y.abs() <= MAX_COORD)
    }

    /// Where it starts and ends.
    pub fn ends(&self) -> Option<(DVec2, DVec2)> {
        Some((*self.control.first()?, *self.control.last()?))
    }

    /// The points the segments join at, ends included.
    pub fn knots(&self) -> impl Iterator<Item = DVec2> + '_ {
        self.control.iter().step_by(3).copied()
    }

    /// The same curve from its end to its start.
    pub fn reversed(&self) -> Self {
        Self { control: spline::reversed(&self.control) }
    }

    /// Every control point through `f`.
    pub fn map(&mut self, f: impl Fn(DVec2) -> DVec2) {
        for p in &mut self.control {
            *p = f(*p);
        }
    }

    /// Its end knot (`at_start`: its first) moved by `delta` with its handle.
    pub fn move_end(&mut self, at_start: bool, delta: DVec2) {
        let n = self.control.len();
        let pair = if at_start { [0, 1] } else { [n.saturating_sub(1), n.saturating_sub(2)] };
        for i in pair {
            if let Some(p) = self.control.get_mut(i) {
                *p += delta;
            }
        }
    }

    /// The parts before and after the point of the curve nearest `at`;
    /// `None` when that is (nearly) an end.
    pub fn split(&self, at: DVec2) -> Option<(Self, Self)> {
        let (s, _, _) = spline::nearest(&self.control, at)?;
        let (a, b) = spline::split_at(&self.control, s)?;
        let part = |control: Vec<DVec2>| {
            let part = Self { control };
            (part.is_valid() && part.path().is_some_and(|p| p.length() >= MIN_LENGTH)).then_some(part)
        };
        Some((part(a)?, part(b)?))
    }

    /// A spline `distance` to the left of this one, as near as
    /// [`spline::offset`] gets (exact where it is straight, close on gentle
    /// bends).
    pub fn offset(&self, distance: f64) -> Option<Self> {
        let moved = Self { control: spline::offset(&self.control, distance) };
        moved.is_valid().then_some(moved)
    }

    /// The location line flattened; `None` for an invalid spline.
    pub fn path(&self) -> Option<WallPath> {
        if !self.is_valid() {
            return None;
        }
        let mut tolerance = (SAGITTA, MAX_TURN);
        // Bounded: each round doubles the sagitta, and 2¹⁶ times it is past any wall.
        for _ in 0..16 {
            let stations = flatten(&self.control, tolerance.0, tolerance.1);
            if stations.len() <= MAX_CHORDS + 1 {
                return WallPath::from_stations(stations);
            }
            tolerance = (tolerance.0 * 2.0, (tolerance.1 * 1.5).min(0.5));
        }
        // The plain split of sketches: at most `MAX_SEGMENTS * STEPS` chords.
        let n = spline::segment_count(&self.control)?;
        let mut stations = Vec::with_capacity(n * spline::STEPS + 1);
        for (i, seg) in spline::segments(&self.control).enumerate() {
            for k in 0..=spline::STEPS {
                if k == 0 && i > 0 {
                    continue;
                }
                stations.push(station(seg, i, k as f64 / spline::STEPS as f64));
            }
        }
        WallPath::from_stations(stations)
    }

    /// The wall this spline makes, with faces `left` and `right` of the
    /// location line (signed distances to its left, `left > right`): its
    /// flattened location line, or why it makes none.
    pub fn check(&self, left: f64, right: f64) -> Result<WallPath, SplineError> {
        if !self.is_valid() || !(left.is_finite() && right.is_finite()) {
            return Err(SplineError::Points);
        }
        let (a, b) = self.ends().ok_or(SplineError::Points)?;
        if a.distance(b) < MIN_LENGTH {
            return Err(SplineError::Short);
        }
        let path = self.path().ok_or(SplineError::Points)?;
        if path.length() < MIN_LENGTH {
            return Err(SplineError::Short);
        }
        if let Some(p) = path.corner() {
            return Err(SplineError::Corner(p));
        }
        if let Some(p) = path.tight(left, right) {
            return Err(SplineError::Tight(p));
        }
        if let Some(p) = path.crossing(left, right) {
            return Err(SplineError::Crossing(p));
        }
        Ok(path)
    }
}

impl crate::Project {
    /// The flattened body centerline of a spline wall: its location line
    /// moved across by the location line setting (`None` for other walls).
    pub fn wall_path(&self, element: &crate::Element, wall: &crate::Wall) -> Option<WallPath> {
        wall.spline_path()?.offset(-self.wall_line_offset(element, wall))
    }

    /// The faces of a wall of type `type_id` drawn on `line`, as signed
    /// distances to the left of its location line: `(left, right)`.
    pub fn wall_faces(&self, type_id: crate::ElementTypeId, line: crate::LocationLine) -> (f64, f64) {
        let (half, offset) =
            self.element_type(type_id).map_or((0.0, 0.0), |t| (t.thickness() * 0.5, t.location_offset(line)));
        (half - offset, -half - offset)
    }

    /// The flattened location line of the wall `spline` makes with type
    /// `type_id` drawn on `line`, or why it makes none (see
    /// [`WallSpline::check`]).
    pub fn check_wall_spline(
        &self,
        type_id: crate::ElementTypeId,
        line: crate::LocationLine,
        spline: &WallSpline,
    ) -> Result<WallPath, SplineError> {
        let (left, right) = self.wall_faces(type_id, line);
        spline.check(left, right)
    }
}

impl crate::Project {
    /// Spline walls keep only curves that make a wall of their type (see
    /// [`WallSpline::check`]) that is no curtain wall: another goes, the
    /// wall staying straight between its ends. A spline wall starts and
    /// ends where its curve does, has no bulge, no elevation profile and no
    /// attached top. Returns how many things changed.
    pub(crate) fn repair_wall_splines(&mut self) -> usize {
        let verdicts: Vec<(crate::ElementId, Option<(DVec2, DVec2)>)> = self
            .elements
            .values()
            .filter_map(|e| {
                let w = e.as_wall()?;
                let spline = w.spline.as_ref()?;
                let curtain = self.element_type(e.type_id).is_some_and(|t| t.curtain().is_some());
                let fits = !curtain && self.check_wall_spline(e.type_id, w.location_line, spline).is_ok();
                Some((e.id, if fits { spline.ends() } else { None }))
            })
            .collect();
        let mut repairs = 0;
        for (id, ends) in verdicts {
            let Some(crate::ElementKind::Wall(w)) = self.elements.get_mut(&id).map(|e| &mut e.kind) else { continue };
            let Some((start, end)) = ends else {
                w.spline = None;
                repairs += 1;
                continue;
            };
            if w.start != start || w.end != end {
                (w.start, w.end) = (start, end);
                repairs += 1;
            }
            if w.bulge != 0.0 {
                w.bulge = 0.0;
                repairs += 1;
            }
            repairs += usize::from(w.profile.take().is_some());
            repairs += usize::from(w.attach_top.take().is_some());
        }
        repairs
    }
}

/// One point of a flattened spline.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Station {
    point: DVec2,
    /// Unit direction of travel.
    tangent: DVec2,
    /// Signed curvature (1 / radius), positive turning left.
    curvature: f64,
    /// The spline's parameter (segment index plus `t`).
    param: f64,
    /// Degrees the direction breaks here (where two segments join).
    kink: f64,
}

/// The second derivative of a segment at `t`.
fn second([p0, p1, p2, p3]: [DVec2; 4], t: f64) -> DVec2 {
    (p2 - 2.0 * p1 + p0) * (6.0 * (1.0 - t)) + (p3 - 2.0 * p2 + p1) * (6.0 * t)
}

/// The unit direction of a segment at `t` (along the chord around `t`
/// where a handle on its knot leaves no derivative).
fn direction(seg: [DVec2; 4], t: f64) -> DVec2 {
    let d = spline::derivative(seg, t);
    let d = if d.length() > 1e-9 {
        d
    } else {
        spline::point(seg, (t + 0.01).min(1.0)) - spline::point(seg, (t - 0.01).max(0.0))
    };
    d.try_normalize().unwrap_or(DVec2::X)
}

fn station(seg: [DVec2; 4], index: usize, t: f64) -> Station {
    let d1 = spline::derivative(seg, t);
    let speed = d1.length();
    // No speed: a cusp, or a handle on its knot; either way no radius to rely on.
    let curvature = if speed > 1e-9 { d1.perp_dot(second(seg, t)) / (speed * speed * speed) } else { f64::INFINITY };
    Station {
        point: spline::point(seg, t),
        tangent: direction(seg, t),
        curvature: if curvature.is_nan() { f64::INFINITY } else { curvature },
        param: index as f64 + t,
        kink: 0.0,
    }
}

/// The angle between two unit directions, radians.
fn turn(a: DVec2, b: DVec2) -> f64 {
    a.perp_dot(b).atan2(a.dot(b)).abs()
}

/// The distance of `p` from the segment `a`–`b`.
fn off_chord(p: DVec2, a: DVec2, b: DVec2) -> f64 {
    let d = b - a;
    let len2 = d.length_squared();
    if len2 < 1e-24 {
        return p.distance(a);
    }
    let t = ((p - a).dot(d) / len2).clamp(0.0, 1.0);
    p.distance(a + d * t)
}

/// The stations of the spline: each segment halved until its chords stray
/// no more than `sagitta` and turn no more than `max_turn` (to
/// [`MAX_DEPTH`]). Where two segments join there is one station, its
/// direction the mean of theirs.
fn flatten(control: &[DVec2], sagitta: f64, max_turn: f64) -> Vec<Station> {
    let mut out: Vec<Station> = Vec::new();
    for (i, seg) in spline::segments(control).enumerate() {
        let first = station(seg, i, 0.0);
        match out.last_mut() {
            Some(last) => {
                // The joint: one point, the mean direction, the tighter curvature.
                last.kink = turn(last.tangent, first.tangent).to_degrees();
                last.tangent = (last.tangent + first.tangent).try_normalize().unwrap_or(first.tangent);
                if first.curvature.abs() > last.curvature.abs() {
                    last.curvature = first.curvature;
                }
            }
            None => out.push(first),
        }
        // Intervals still to do, the nearest on top.
        let mut stack: Vec<(f64, f64, u32)> = vec![(0.0, 1.0, 0)];
        while let Some((t0, t1, depth)) = stack.pop() {
            let tm = (t0 + t1) * 0.5;
            let (a, b) = (spline::point(seg, t0), spline::point(seg, t1));
            let (da, dm, db) = (direction(seg, t0), direction(seg, tm), direction(seg, t1));
            let strays = [0.25, 0.5, 0.75]
                .iter()
                .any(|k| off_chord(spline::point(seg, t0 + (t1 - t0) * k), a, b) > sagitta);
            let turns = turn(da, dm) + turn(dm, db) > max_turn;
            if depth < MAX_DEPTH && (strays || turns) {
                stack.push((tm, t1, depth + 1));
                stack.push((t0, tm, depth + 1));
            } else {
                out.push(station(seg, i, t1));
            }
        }
    }
    out
}

/// A spline flattened: its points from start to end, the unit left normal
/// of the true curve at each, the distance along the chords to each and the
/// curve's signed curvature there. See the module for its `(u, v)` frame.
#[derive(Debug, Clone, PartialEq)]
pub struct WallPath {
    points: Vec<DVec2>,
    normals: Vec<DVec2>,
    along: Vec<f64>,
    curvature: Vec<f64>,
    params: Vec<f64>,
    kinks: Vec<f64>,
}

impl WallPath {
    fn from_stations(stations: Vec<Station>) -> Option<Self> {
        // Stations on one point (a segment of no length) count once.
        let mut kept: Vec<Station> = Vec::with_capacity(stations.len());
        for s in stations {
            if !(s.point.is_finite() && s.tangent.is_finite()) {
                return None;
            }
            match kept.last_mut() {
                Some(last) if last.point.distance(s.point) < 1e-9 => {
                    last.param = s.param;
                    last.kink = last.kink.max(s.kink);
                }
                _ => kept.push(s),
            }
        }
        if kept.len() < 2 {
            return None;
        }
        let mut along = Vec::with_capacity(kept.len());
        let mut total = 0.0;
        for (i, s) in kept.iter().enumerate() {
            if let Some(prev) = i.checked_sub(1).and_then(|j| kept.get(j)) {
                total += prev.point.distance(s.point);
            }
            along.push(total);
        }
        Some(Self {
            points: kept.iter().map(|s| s.point).collect(),
            normals: kept.iter().map(|s| s.tangent.perp()).collect(),
            along,
            curvature: kept.iter().map(|s| s.curvature).collect(),
            params: kept.iter().map(|s| s.param).collect(),
            kinks: kept.iter().map(|s| s.kink).collect(),
        })
    }

    /// The path `distance` to the left of this one: the same stations moved
    /// along their normals (the offset of the true curve at each).
    pub fn offset(&self, distance: f64) -> Option<Self> {
        if distance == 0.0 {
            return Some(self.clone());
        }
        let stations: Vec<Station> = (0..self.points.len())
            .filter_map(|i| {
                let (p, n, k) = (*self.points.get(i)?, *self.normals.get(i)?, *self.curvature.get(i)?);
                let scale = 1.0 - distance * k;
                Some(Station {
                    point: p + n * distance,
                    tangent: -n.perp(),
                    curvature: if scale > 1e-9 { k / scale } else { f64::INFINITY },
                    param: *self.params.get(i)?,
                    kink: *self.kinks.get(i)?,
                })
            })
            .collect();
        Self::from_stations(stations)
    }

    /// Length along the chords.
    pub fn length(&self) -> f64 {
        self.along.last().copied().unwrap_or(0.0)
    }

    /// The flattened points, from start to end.
    pub fn points(&self) -> &[DVec2] {
        &self.points
    }

    /// The distance along the path to each of [`Self::points`].
    pub fn stations(&self) -> &[f64] {
        &self.along
    }

    /// The least radius of curvature along the path (infinite when straight).
    pub fn min_radius(&self) -> f64 {
        let k = self.curvature.iter().fold(0.0_f64, |m, k| m.max(k.abs()));
        if k > 1e-12 { 1.0 / k } else { f64::INFINITY }
    }

    /// The index of the chord that holds `u` (the first or last beyond the ends).
    fn chord(&self, u: f64) -> usize {
        let last = self.along.len().saturating_sub(2);
        self.along.partition_point(|a| *a <= u).saturating_sub(1).min(last)
    }

    /// The chord around `u` and how far along it `u` is (0 to 1); `None`
    /// before the start (`Err(true)`) or past the end.
    fn span(&self, u: f64) -> Result<(usize, f64), bool> {
        if u < 0.0 || u.is_nan() {
            return Err(true);
        }
        if u > self.length() {
            return Err(false);
        }
        let i = self.chord(u);
        let (a, b) = (self.along.get(i).copied().unwrap_or(0.0), self.along.get(i + 1).copied().unwrap_or(0.0));
        Ok((i, if b - a > 1e-12 { ((u - a) / (b - a)).clamp(0.0, 1.0) } else { 0.0 }))
    }

    fn end(&self, at_start: bool) -> (DVec2, DVec2, f64) {
        let i = if at_start { 0 } else { self.points.len().saturating_sub(1) };
        let n = self.normals.get(i).copied().unwrap_or(DVec2::Y);
        (self.points.get(i).copied().unwrap_or(DVec2::ZERO), n, if at_start { 0.0 } else { self.length() })
    }

    /// The point `u` along the path and `v` to its left. Beyond the ends
    /// the path carries on straight along its direction there.
    pub fn at(&self, u: f64, v: f64) -> DVec2 {
        match self.span(u) {
            Ok((i, t)) => {
                let get = |k: usize| {
                    self.points.get(k).copied().unwrap_or(DVec2::ZERO)
                        + self.normals.get(k).copied().unwrap_or(DVec2::ZERO) * v
                };
                get(i).lerp(get(i + 1), t)
            }
            Err(at_start) => {
                let (p, n, u0) = self.end(at_start);
                p - n.perp() * (u - u0) + n * v
            }
        }
    }

    /// Unit direction of travel at `u`.
    pub fn tangent(&self, u: f64) -> DVec2 {
        -self.normal(u).perp()
    }

    /// Unit left normal at `u`.
    pub fn normal(&self, u: f64) -> DVec2 {
        match self.span(u) {
            Ok((i, t)) => {
                let (a, b) = (
                    self.normals.get(i).copied().unwrap_or(DVec2::Y),
                    self.normals.get(i + 1).copied().unwrap_or(DVec2::Y),
                );
                a.lerp(b, t).try_normalize().unwrap_or(a)
            }
            Err(at_start) => self.end(at_start).1,
        }
    }

    /// The spline's parameter (segment index plus `t`) at `u`.
    pub fn param_at(&self, u: f64) -> f64 {
        match self.span(u) {
            Ok((i, t)) => {
                let (a, b) =
                    (self.params.get(i).copied().unwrap_or(0.0), self.params.get(i + 1).copied().unwrap_or(0.0));
                a + (b - a) * t
            }
            Err(true) => self.params.first().copied().unwrap_or(0.0),
            Err(false) => self.params.last().copied().unwrap_or(0.0),
        }
    }

    /// `(u, v)` of the plan point `p`: the inverse of [`Self::at`], the
    /// nearest when several stretches reach it.
    pub fn locate(&self, p: DVec2) -> (f64, f64) {
        let mut best: Option<(f64, f64)> = None;
        let mut take = |u: f64, v: f64| {
            if u.is_finite() && v.is_finite() && best.is_none_or(|(_, b)| v.abs() < b.abs()) {
                best = Some((u, v));
            }
        };
        const EPS: f64 = 1e-9;
        for i in 0..self.points.len().saturating_sub(1) {
            let (Some(&a), Some(&b), Some(&na), Some(&nb)) =
                (self.points.get(i), self.points.get(i + 1), self.normals.get(i), self.normals.get(i + 1))
            else {
                continue;
            };
            let (d, dn, q) = (b - a, nb - na, p - a);
            // (q - t d) × (na + t dn) = 0.
            let (qa, qb, qc) = (-d.perp_dot(dn), q.perp_dot(dn) - d.perp_dot(na), q.perp_dot(na));
            let mut roots = [f64::NAN; 2];
            if qa.abs() < 1e-12 {
                if qb.abs() > 1e-15 {
                    roots[0] = -qc / qb;
                }
            } else {
                let disc = qb * qb - 4.0 * qa * qc;
                if disc >= 0.0 {
                    let s = disc.sqrt();
                    // The stable pair of roots.
                    let w = -0.5 * (qb + if qb >= 0.0 { s } else { -s });
                    roots = [w / qa, if w.abs() > 1e-300 { qc / w } else { f64::NAN }];
                }
            }
            let (u0, len) = (self.along.get(i).copied().unwrap_or(0.0), d.length());
            for t in roots {
                if !(-EPS..=1.0 + EPS).contains(&t) {
                    continue;
                }
                let t = t.clamp(0.0, 1.0);
                let n = na + dn * t;
                let n2 = n.length_squared();
                if n2 > 1e-12 {
                    take(u0 + len * t, (q - d * t).dot(n) / n2);
                }
            }
        }
        // Beyond the ends: the straight frame there.
        for at_start in [true, false] {
            let (e, n, u0) = self.end(at_start);
            let along = (p - e).dot(-n.perp());
            if (at_start && along < 0.0) || (!at_start && along > 0.0) {
                take(u0 + along, (p - e).dot(n));
            }
        }
        best.unwrap_or_else(|| self.nearest(p))
    }

    /// `(u, v)` of the point of the chords nearest `p`.
    fn nearest(&self, p: DVec2) -> (f64, f64) {
        let mut best = (0.0, 0.0, f64::INFINITY);
        for i in 0..self.points.len().saturating_sub(1) {
            let (Some(&a), Some(&b)) = (self.points.get(i), self.points.get(i + 1)) else { continue };
            let d = b - a;
            let len2 = d.length_squared();
            let t = if len2 > 1e-24 { ((p - a).dot(d) / len2).clamp(0.0, 1.0) } else { 0.0 };
            let foot = a + d * t;
            let dist = p.distance(foot);
            if dist < best.2 {
                let side = if d.perp_dot(p - a) >= 0.0 { 1.0 } else { -1.0 };
                best = (self.along.get(i).copied().unwrap_or(0.0) + len2.sqrt() * t, side * dist, dist);
            }
        }
        (best.0, best.1)
    }

    /// The stations strictly between `u0` and `u1` (`u0 < u1`).
    pub fn stations_between(&self, u0: f64, u1: f64) -> impl Iterator<Item = f64> + '_ {
        let from = self.along.partition_point(|a| *a <= u0 + 1e-9);
        self.along.iter().skip(from).copied().take_while(move |a| *a < u1 - 1e-9)
    }

    /// The line `v` to the left of the path from `u0` to `u1` (either
    /// order), through every station between them.
    pub fn line(&self, u0: f64, u1: f64, v: f64) -> Vec<DVec2> {
        let (lo, hi) = (u0.min(u1), u0.max(u1));
        let mut pts = vec![self.at(lo, v)];
        pts.extend(self.stations_between(lo, hi).map(|u| self.at(u, v)));
        pts.push(self.at(hi, v));
        if u0 > u1 {
            pts.reverse();
        }
        pts
    }

    /// The whole line `v` to the left of the path.
    pub fn face(&self, v: f64) -> Vec<DVec2> {
        self.points.iter().zip(&self.normals).map(|(p, n)| *p + *n * v).collect()
    }

    /// Where two segments join in a corner (see [`MAX_KINK_DEG`]).
    pub fn corner(&self) -> Option<DVec2> {
        let i = self.kinks.iter().position(|k| *k > MAX_KINK_DEG)?;
        self.points.get(i).copied()
    }

    /// Where the path bends too tightly for faces `left` and `right` of it
    /// (signed distances to the left): the face inside the bend would keep
    /// less than [`MIN_FACE_RADIUS`], or a chord still turns too far.
    pub fn tight(&self, left: f64, right: f64) -> Option<DVec2> {
        for (i, k) in self.curvature.iter().enumerate() {
            let room = |v: f64| 1.0 - v * k >= MIN_FACE_RADIUS * k.abs();
            if !k.is_finite() || !room(left) || !room(right) {
                return self.points.get(i).copied();
            }
        }
        // A bend the flattening could not resolve (a cusp between stations).
        for (i, w) in self.normals.windows(2).enumerate() {
            if turn(w[0], w[1]) > 4.0 * MAX_TURN {
                return self.points.get(i).copied();
            }
        }
        None
    }

    /// Where a wall with faces `left` and `right` of the path would cross
    /// itself: where one of its faces crosses itself or the other.
    pub fn crossing(&self, left: f64, right: f64) -> Option<DVec2> {
        let faces = [self.face(left), self.face(right)];
        // Every chord of both faces as (face, index, a, b), swept along x.
        let mut chords: Vec<(usize, usize, DVec2, DVec2)> = Vec::new();
        for (f, pts) in faces.iter().enumerate() {
            chords.extend(pts.windows(2).enumerate().map(|(i, w)| (f, i, w[0], w[1])));
        }
        chords.sort_by(|a, b| a.2.x.min(a.3.x).total_cmp(&b.2.x.min(b.3.x)));
        for (i, &(fa, ia, a0, a1)) in chords.iter().enumerate() {
            let reach = a0.x.max(a1.x);
            for &(fb, ib, b0, b1) in chords.iter().skip(i + 1) {
                if b0.x.min(b1.x) > reach {
                    break;
                }
                if fa == fb && ia.abs_diff(ib) <= 1 {
                    continue;
                }
                if let Some(p) = cross(a0, a1, b0, b1) {
                    return Some(p);
                }
            }
        }
        None
    }
}

/// Where the segments `a0`–`a1` and `b0`–`b1` cross (touching ends count).
fn cross(a0: DVec2, a1: DVec2, b0: DVec2, b1: DVec2) -> Option<DVec2> {
    let (d, e) = (a1 - a0, b1 - b0);
    let den = d.perp_dot(e);
    if den.abs() < 1e-15 {
        return None;
    }
    let w = b0 - a0;
    let (t, s) = (w.perp_dot(e) / den, w.perp_dot(d) / den);
    ((0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&s)).then(|| a0 + d * t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::FRAC_PI_2;

    fn wave() -> WallSpline {
        let fit = [DVec2::new(0.0, 0.0), DVec2::new(3.0, 1.5), DVec2::new(6.0, -1.0), DVec2::new(9.0, 0.5)];
        WallSpline::through(&fit).unwrap()
    }

    /// A quarter circle of radius `r` around the origin, as one Bézier segment.
    fn quarter(r: f64) -> WallSpline {
        let k = 0.552_284_749_8 * r;
        WallSpline {
            control: vec![DVec2::new(r, 0.0), DVec2::new(r, k), DVec2::new(k, r), DVec2::new(0.0, r)],
        }
    }

    #[test]
    fn fit_points_and_control_points_make_the_same_wall() {
        let fit = [DVec2::new(0.0, 0.0), DVec2::new(3.0, 1.5), DVec2::new(6.0, -1.0), DVec2::new(9.0, 0.5)];
        let s = WallSpline::through(&fit).unwrap();
        assert_eq!(s.control.len(), 10);
        // It passes through the points, which are its knots.
        assert_eq!(s.knots().collect::<Vec<_>>(), fit.to_vec());
        // Its control points say the same.
        let again = WallSpline { control: s.control.clone() };
        assert_eq!(again, s);
        assert_eq!(again.path(), s.path());
        assert!(s.check(0.1, -0.1).is_ok());
        // Text and back.
        let text = ron::to_string(&s).unwrap();
        assert_eq!(ron::from_str::<WallSpline>(&text).unwrap(), s);
    }

    #[test]
    fn a_straight_spline_is_a_line() {
        let s = WallSpline::through(&[DVec2::ZERO, DVec2::new(2.0, 0.0), DVec2::new(5.0, 0.0)]).unwrap();
        let p = s.path().unwrap();
        assert!((p.length() - 5.0).abs() < 1e-12);
        assert!(p.min_radius().is_infinite());
        assert!(p.at(1.25, 0.1).distance(DVec2::new(1.25, 0.1)) < 1e-12);
        assert!(p.at(-0.5, -0.1).distance(DVec2::new(-0.5, -0.1)) < 1e-12);
        assert!(p.at(5.5, 0.0).distance(DVec2::new(5.5, 0.0)) < 1e-12);
        let (u, v) = p.locate(DVec2::new(4.0, 0.3));
        assert!((u - 4.0).abs() < 1e-9 && (v - 0.3).abs() < 1e-9);
        let (u, v) = p.locate(DVec2::new(-1.0, -0.2));
        assert!((u + 1.0).abs() < 1e-9 && (v + 0.2).abs() < 1e-9);
    }

    #[test]
    fn a_quarter_circle_is_flattened_within_tolerance() {
        let r = 3.0;
        let p = quarter(r).path().unwrap();
        // The Bézier quarter is a circle to 0.03 %; the chords stay within the sagitta of it.
        assert!((p.length() / (FRAC_PI_2 * r) - 1.0).abs() < 5e-4, "{}", p.length());
        assert!((p.min_radius() / r - 1.0).abs() < 0.01, "{}", p.min_radius());
        for w in p.points().windows(2) {
            let mid = (w[0] + w[1]) * 0.5;
            assert!((mid.length() - r).abs() < SAGITTA + 1e-3 * r * 0.3);
        }
        // Turning left: the left is towards the center, and offsets are concentric.
        assert!((p.at(p.length() * 0.5, 0.5).length() - (r - 0.5)).abs() < 2e-3);
        let inner = p.offset(0.5).unwrap();
        assert!((inner.length() / (FRAC_PI_2 * (r - 0.5)) - 1.0).abs() < 1e-3);
        assert!((inner.min_radius() / (r - 0.5) - 1.0).abs() < 0.01);
        // Tangents: up at the start, to the left at the end.
        assert!(p.tangent(0.0).distance(DVec2::Y) < 1e-9);
        assert!(p.tangent(p.length()).distance(-DVec2::X) < 1e-9);
    }

    #[test]
    fn locate_inverts_at() {
        let p = wave().path().unwrap();
        let len = p.length();
        for k in 0..=40 {
            let u = len * k as f64 / 40.0;
            for v in [-0.15, 0.0, 0.2] {
                let (lu, lv) = p.locate(p.at(u, v));
                assert!((lu - u).abs() < 1e-6 && (lv - v).abs() < 1e-6, "{u} {v}: {lu} {lv}");
            }
        }
        // Beyond the ends too.
        for (u, v) in [(-0.3, 0.1), (len + 0.2, -0.1)] {
            let (lu, lv) = p.locate(p.at(u, v));
            assert!((lu - u).abs() < 1e-6 && (lv - v).abs() < 1e-6, "{u} {v}: {lu} {lv}");
        }
        // Lines between stations run through them.
        let line = p.line(0.5, len - 0.5, 0.1);
        assert!(line.len() > 4);
        assert!(line[0].distance(p.at(0.5, 0.1)) < 1e-12);
        let back = p.line(len - 0.5, 0.5, 0.1);
        assert!(back[0].distance(*line.last().unwrap()) < 1e-12 && back.len() == line.len());
    }

    #[test]
    fn hostile_splines_make_no_wall() {
        let bad = |control: Vec<DVec2>| WallSpline { control }.check(0.1, -0.1);
        assert_eq!(bad(vec![DVec2::ZERO]), Err(SplineError::Points));
        assert_eq!(bad(vec![DVec2::ZERO, DVec2::X, DVec2::NAN, DVec2::Y]), Err(SplineError::Points));
        assert_eq!(bad(vec![DVec2::ZERO, DVec2::X, DVec2::new(1e9, 0.0), DVec2::Y]), Err(SplineError::Points));
        assert_eq!(bad(vec![DVec2::X; 300_001]), Err(SplineError::Points));
        assert_eq!(bad(vec![DVec2::ZERO; 4]), Err(SplineError::Points));
        // Closed, and too short.
        let closed = spline::interpolate(&[DVec2::ZERO, DVec2::new(4.0, 0.0), DVec2::new(2.0, 3.0)], true);
        assert_eq!(bad(closed), Err(SplineError::Short));
        assert_eq!(WallSpline::through(&[DVec2::ZERO, DVec2::new(0.01, 0.0)]).unwrap().check(0.1, -0.1), Err(SplineError::Short));
        // A corner between two straight segments.
        let corner = vec![
            DVec2::new(0.0, 0.0),
            DVec2::new(1.0, 0.0),
            DVec2::new(2.0, 0.0),
            DVec2::new(3.0, 0.0),
            DVec2::new(3.0, 1.0),
            DVec2::new(3.0, 2.0),
            DVec2::new(3.0, 3.0),
        ];
        assert_eq!(bad(corner), Err(SplineError::Corner(DVec2::new(3.0, 0.0))));
        // A bend of 15 cm radius under a 30 cm wall.
        assert!(matches!(quarter(0.15).check(0.15, -0.15), Err(SplineError::Tight(_))));
        assert!(quarter(0.15).check(0.05, -0.05).is_ok());
        // The location line on the outer face leaves the bend more room.
        assert!(matches!(quarter(0.3).check(0.3, 0.0), Err(SplineError::Tight(_))));
        assert!(quarter(0.3).check(0.0, -0.3).is_ok());
        // A loop that crosses itself.
        let looped =
            WallSpline { control: vec![DVec2::ZERO, DVec2::new(6.0, 4.0), DVec2::new(-2.0, 4.0), DVec2::new(4.0, 0.0)] };
        assert!(matches!(looped.check(0.1, -0.1), Err(SplineError::Crossing(_))), "{:?}", looped.check(0.1, -0.1));
        // A hairpin whose passes lie closer than the wall is thick.
        let hairpin = WallSpline::through(&[
            DVec2::new(0.0, 0.0),
            DVec2::new(4.0, 0.0),
            DVec2::new(5.5, 1.5),
            DVec2::new(4.0, 3.0),
            DVec2::new(0.0, 3.0),
        ])
        .unwrap();
        assert!(hairpin.check(0.1, -0.1).is_ok(), "{:?}", hairpin.check(0.1, -0.1));
        assert!(matches!(hairpin.check(1.6, -1.6), Err(SplineError::Tight(_) | SplineError::Crossing(_))));
        // A cusp: both handles past each other.
        let cusp = WallSpline {
            control: vec![DVec2::ZERO, DVec2::new(3.0, 0.0), DVec2::new(-2.0, 0.0), DVec2::new(1.0, 0.0)],
        };
        assert!(cusp.check(0.1, -0.1).is_err());
    }

    #[test]
    fn flattening_is_bounded() {
        // 64 wild segments: still no more chords than the cap.
        let mut fit = Vec::new();
        for i in 0..=MAX_SEGMENTS {
            let a = i as f64 * 2.4;
            fit.push(DVec2::new(a.cos() * (500.0 + i as f64 * 37.0), a.sin() * (500.0 + i as f64 * 11.0)));
        }
        let s = WallSpline::through(&fit).unwrap();
        let p = s.path().unwrap();
        assert!(p.points().len() <= MAX_CHORDS + 1, "{}", p.points().len());
        assert!(p.length().is_finite() && p.length() > 1000.0);
        assert!(WallSpline::through(&vec![DVec2::X; MAX_SEGMENTS + 2]).is_none());
    }

    #[test]
    fn splitting_and_reversing_keep_the_curve() {
        let s = wave();
        let len = s.path().unwrap().length();
        let (a, b) = s.split(DVec2::new(4.5, 0.4)).unwrap();
        assert!((a.path().unwrap().length() + b.path().unwrap().length() - len).abs() < 1e-3);
        assert_eq!(a.ends().unwrap().1, b.ends().unwrap().0);
        assert!(s.split(DVec2::new(-1.0, 0.0)).is_none());
        let r = s.reversed();
        assert_eq!(r.ends().unwrap(), (s.ends().unwrap().1, s.ends().unwrap().0));
        assert!((r.path().unwrap().length() - len).abs() < 1e-9);
        // The left of the reversed curve is the right of the curve.
        let (p, q) = (s.path().unwrap(), r.path().unwrap());
        assert!(p.at(1.0, 0.1).distance(q.at(len - 1.0, -0.1)) < 1e-9);
        // An offset keeps its distance (to a few millimeters on these bends).
        let o = s.offset(0.3).unwrap();
        for k in o.knots() {
            assert!((p.locate(k).1 - 0.3).abs() < 5e-3);
        }
        // Moving an end takes its handle along.
        let mut m = s.clone();
        m.move_end(false, DVec2::new(1.0, 1.0));
        assert_eq!(m.control[9], s.control[9] + DVec2::ONE);
        assert_eq!(m.control[8], s.control[8] + DVec2::ONE);
        assert_eq!(m.control[7], s.control[7]);
    }
}
