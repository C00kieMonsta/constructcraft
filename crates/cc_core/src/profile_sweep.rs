//! Swept profiles: a closed profile, held by the element's type, carried
//! along a path drawn in plan at a height above the element's level. They
//! model what runs along walls, ceilings and roof edges: baseboards, crown
//! moldings, fascia boards, gutters, copings.
//!
//! The profile is drawn in its own plane, square to the path: `x` across,
//! positive to the left of the path walking along it (`flip` mirrors it to
//! the right), and `y` up from the path's height.

use crate::arc::{ARC_STEP, Arc, MAX_BULGE, segments};
use crate::sketch::{JOIN_TOL, SketchCurve};
use glam::DVec2;
use serde::{Deserialize, Serialize};

/// Most points of a profile.
pub const MAX_PROFILE_POINTS: usize = 512;
/// Largest extent of a profile, meters.
pub const MAX_PROFILE_SIZE: f64 = 10.0;
/// Most points of a sweep's path.
pub const MAX_PATH_POINTS: usize = 4096;
/// Most stations of a sweep (the points of its path where the solid has a
/// section: the points, and those that split its arcs every [`ARC_STEP`]).
pub const MAX_STATIONS: usize = 8192;

/// Bulges closer to 0 than this are straight segments (as in [`crate::arc`]).
const MIN_BULGE: f64 = 1e-6;

/// A swept profile element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sweep {
    /// The path in plan, in order.
    pub path: Vec<DVec2>,
    /// The path runs back from its last point to its first (around a room).
    #[serde(default)]
    pub closed: bool,
    /// Height of the path above the level.
    pub base_offset: f64,
    /// The profile on the right of the path instead of the left.
    #[serde(default)]
    pub flip: bool,
    /// Arcs of the path: `bulges[i]` is the bulge (see [`crate::arc`]) of the
    /// segment from point `i` to the next (for the last point of a closed
    /// path, back to the first). Missing ones are straight; the closing
    /// segment's is kept, unused, while the path is open. A circle is a
    /// closed path of two points and two half circles.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bulges: Vec<f64>,
}

impl Sweep {
    /// Number of segments: one between each two points, and the way back of a closed path.
    pub fn segment_count(&self) -> usize {
        let n = self.path.len();
        if self.closed && n >= 2 { n } else { n.saturating_sub(1) }
    }

    /// The bulge of segment `i` (0 when straight).
    pub fn bulge(&self, i: usize) -> f64 {
        self.bulges.get(i).copied().filter(|b| b.is_finite()).unwrap_or(0.0)
    }

    /// The segments in order: start, end, and the arc of a curved one.
    pub fn segments(&self) -> Vec<(DVec2, DVec2, Option<Arc>)> {
        let n = self.path.len();
        (0..self.segment_count())
            .filter_map(|i| {
                let (a, b) = (*self.path.get(i)?, *self.path.get((i + 1) % n)?);
                Some((a, b, Arc::from_bulge(a, b, self.bulge(i))))
            })
            .collect()
    }

    /// Whether any segment is an arc.
    pub fn is_curved(&self) -> bool {
        self.segments().iter().any(|(_, _, arc)| arc.is_some())
    }

    /// Plan length of the path, arcs as arcs.
    pub fn length(&self) -> f64 {
        self.segments().iter().map(|(a, b, arc)| arc.map_or_else(|| a.distance(*b), |arc| arc.length())).sum()
    }

    /// Number of stations: the points, and those that split the arcs every [`ARC_STEP`].
    pub fn stations(&self) -> usize {
        let inner: usize =
            self.segments().iter().map(|(_, _, arc)| arc.map_or(1, |arc| segments(arc.sweep, ARC_STEP))).sum();
        inner + usize::from(!self.closed)
    }

    pub fn is_valid(&self) -> bool {
        // Two points close into a loop when one of the two segments between them is an arc.
        let needed = match (self.closed, self.is_curved()) {
            (true, false) => 3,
            _ => 2,
        };
        self.base_offset.is_finite()
            && self.path.len() >= needed
            && self.path.len() <= MAX_PATH_POINTS
            && self.path.iter().all(|p| p.is_finite())
            && self.bulges.len() <= self.path.len()
            && self.bulges.iter().all(|b| b.is_finite() && b.abs() <= MAX_BULGE)
            && self.stations() <= MAX_STATIONS
            && self.length() > 1e-6
    }

    /// Bring the bulges in range: at most one per point, numbers (else
    /// straight), short of a full circle. Returns how many changed.
    pub fn repair(&mut self) -> usize {
        let mut repairs = 0;
        if self.bulges.len() > self.path.len() {
            self.bulges.truncate(self.path.len());
            repairs += 1;
        }
        for b in &mut self.bulges {
            let fixed = if !b.is_finite() || b.abs() < MIN_BULGE { 0.0 } else { b.clamp(-MAX_BULGE, MAX_BULGE) };
            // By value: a mirrored straight segment's -0 is fine.
            if fixed != *b {
                *b = fixed;
                repairs += 1;
            }
        }
        repairs
    }
}

/// A sweep's path made from curves: its points, the bulge of each segment
/// and whether it closes.
#[derive(Debug, Clone, PartialEq)]
pub struct SweepPath {
    pub points: Vec<DVec2>,
    pub bulges: Vec<f64>,
    pub closed: bool,
}

/// Why curves make no sweep path.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathProblem {
    Empty,
    TooMany,
    /// Curve `i` is an ellipse or a spline.
    Unsupported(usize),
    /// Curve `i` has invalid numbers, no size, or turns too far.
    Invalid(usize),
    CircleWithOthers,
    /// More than two curves meet here.
    Branch(DVec2),
    /// A curve lies apart from the others, here.
    Apart(DVec2),
    /// The closed path encloses nothing.
    NoArea,
}

impl PathProblem {
    pub fn message(self) -> String {
        let at = |p: DVec2| format!("({:.3}, {:.3})", p.x, p.y);
        match self {
            PathProblem::Empty => "the path has no curves".into(),
            PathProblem::TooMany => format!("the path has more than {MAX_PATH_POINTS} curves"),
            PathProblem::Unsupported(i) => format!("curve {i}: a path is made of lines, arcs and circles"),
            PathProblem::Invalid(i) => format!(
                "curve {i} has invalid numbers or no size, or turns too far (an arc's bulge is at most {MAX_BULGE})"
            ),
            PathProblem::CircleWithOthers => "a circle is a whole closed path: give it alone".into(),
            PathProblem::Branch(p) => format!("more than two curves meet at {}: the path must not touch itself", at(p)),
            PathProblem::Apart(p) => format!("the curves do not make one path: one lies apart at {}", at(p)),
            PathProblem::NoArea => "the closed path encloses no area".into(),
        }
    }
}

/// The path that lines, arcs and a circle make: chained end to end (ends
/// within [`JOIN_TOL`] meet, in any order and direction), running the way
/// the first curve runs; closed when the chain closes. A circle alone is a
/// closed path running counter-clockwise (the profile, on its left, inside).
pub fn path_from_curves(curves: &[SketchCurve]) -> Result<SweepPath, PathProblem> {
    if curves.is_empty() {
        return Err(PathProblem::Empty);
    }
    if curves.len() > MAX_PATH_POINTS {
        return Err(PathProblem::TooMany);
    }
    let mut segs: Vec<(DVec2, DVec2, f64)> = Vec::with_capacity(curves.len());
    for (i, c) in curves.iter().enumerate() {
        if !c.is_valid() {
            return Err(PathProblem::Invalid(i));
        }
        match *c {
            SketchCurve::Line { a, b } => segs.push((a, b, 0.0)),
            SketchCurve::Arc { a, b, bulge } if bulge.abs() <= MAX_BULGE => {
                segs.push((a, b, if bulge.abs() < MIN_BULGE { 0.0 } else { bulge }));
            }
            SketchCurve::Arc { .. } => return Err(PathProblem::Invalid(i)),
            SketchCurve::Circle { center, radius } => {
                if curves.len() > 1 {
                    return Err(PathProblem::CircleWithOthers);
                }
                let (a, b) = (center + DVec2::X * radius, center - DVec2::X * radius);
                return Ok(SweepPath { points: vec![a, b], bulges: vec![1.0, 1.0], closed: true });
            }
            _ => return Err(PathProblem::Unsupported(i)),
        }
    }
    // Ends within the tolerance are one node.
    let mut nodes: Vec<DVec2> = Vec::new();
    let mut node_of = |p: DVec2| match nodes.iter().position(|q| q.distance(p) <= JOIN_TOL) {
        Some(i) => i,
        None => {
            nodes.push(p);
            nodes.len() - 1
        }
    };
    let ends: Vec<(usize, usize)> = segs.iter().map(|(a, b, _)| (node_of(*a), node_of(*b))).collect();
    if let Some(i) = ends.iter().position(|(s, e)| s == e) {
        return Err(PathProblem::Invalid(i));
    }
    let mut at: Vec<Vec<usize>> = vec![Vec::new(); nodes.len()];
    for (i, (s, e)) in ends.iter().enumerate() {
        at[*s].push(i);
        at[*e].push(i);
    }
    if let Some(n) = at.iter().position(|c| c.len() > 2) {
        return Err(PathProblem::Branch(nodes[n]));
    }
    let loose: Vec<usize> = (0..nodes.len()).filter(|n| at[*n].len() == 1).collect();
    let (closed, start) = match loose.as_slice() {
        [] => (true, ends[0].0),
        [first, _] => (false, *first),
        [.., third] => return Err(PathProblem::Apart(nodes[*third])),
    };
    // Walk the chain from its start, each curve once.
    let mut walked: Vec<(DVec2, DVec2, f64)> = Vec::with_capacity(segs.len());
    let mut used = vec![false; segs.len()];
    let mut first_forward = true;
    let mut from = start;
    for _ in 0..segs.len() {
        let Some(&k) = at[from].iter().find(|&&k| !used[k]) else { break };
        used[k] = true;
        let (s, e) = ends[k];
        let forward = s == from;
        if k == 0 {
            first_forward = forward;
        }
        let to = if forward { e } else { s };
        let bulge = if forward { segs[k].2 } else { -segs[k].2 };
        walked.push((nodes[from], nodes[to], bulge));
        from = to;
    }
    if let Some(k) = used.iter().position(|u| !u) {
        return Err(PathProblem::Apart(segs[k].0));
    }
    // Run the way the first curve was drawn.
    if !first_forward {
        walked = walked.into_iter().rev().map(|(a, b, bulge)| (b, a, -bulge)).collect();
    }
    if closed {
        let area: f64 = walked.iter().map(|&(a, b, bulge)| SketchCurve::Arc { a, b, bulge }.swept_area()).sum();
        if !area.is_finite() || area.abs() < 1e-9 {
            return Err(PathProblem::NoArea);
        }
    }
    let mut points: Vec<DVec2> = walked.iter().map(|(a, _, _)| *a).collect();
    if !closed && let Some((_, b, _)) = walked.last() {
        points.push(*b);
    }
    Ok(SweepPath { points, bulges: walked.iter().map(|(_, _, b)| *b).collect(), closed })
}

/// Why a profile is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileProblem {
    TooFewPoints,
    TooManyPoints,
    NotFinite,
    TooLarge,
    NoArea,
    Crossing,
}

impl ProfileProblem {
    pub fn message(self) -> &'static str {
        match self {
            ProfileProblem::TooFewPoints => "a profile needs at least 3 points",
            ProfileProblem::TooManyPoints => "a profile has at most 512 points",
            ProfileProblem::NotFinite => "a profile point is not a finite number",
            ProfileProblem::TooLarge => "a profile is at most 10 m across",
            ProfileProblem::NoArea => "a profile must enclose an area",
            ProfileProblem::Crossing => "a profile must not cross itself",
        }
    }
}

/// Check a profile: a simple closed outline of a sensible size.
pub fn check_profile(points: &[DVec2]) -> Result<(), ProfileProblem> {
    let n = points.len();
    if n < 3 {
        return Err(ProfileProblem::TooFewPoints);
    }
    if n > MAX_PROFILE_POINTS {
        return Err(ProfileProblem::TooManyPoints);
    }
    if points.iter().any(|p| !p.is_finite()) {
        return Err(ProfileProblem::NotFinite);
    }
    if points.iter().any(|p| p.abs().max_element() > MAX_PROFILE_SIZE) {
        return Err(ProfileProblem::TooLarge);
    }
    // Edges that are not neighbors must not meet.
    for i in 0..n {
        let (a, b) = (points[i], points[(i + 1) % n]);
        for j in i + 2..n {
            if i == 0 && j == n - 1 {
                continue;
            }
            let (c, d) = (points[j], points[(j + 1) % n]);
            if segments_meet(a, b, c, d) {
                return Err(ProfileProblem::Crossing);
            }
        }
    }
    let area: f64 = (0..n).map(|i| points[i].perp_dot(points[(i + 1) % n])).sum::<f64>() / 2.0;
    if area.abs() < 1e-8 {
        return Err(ProfileProblem::NoArea);
    }
    Ok(())
}

fn segments_meet(a: DVec2, b: DVec2, c: DVec2, d: DVec2) -> bool {
    let side = |p: DVec2, q: DVec2, r: DVec2| (q - p).perp_dot(r - p);
    let (d1, d2) = (side(c, d, a), side(c, d, b));
    let (d3, d4) = (side(a, b, c), side(a, b, d));
    let eps = 1e-12;
    if ((d1 > eps && d2 < -eps) || (d1 < -eps && d2 > eps)) && ((d3 > eps && d4 < -eps) || (d3 < -eps && d4 > eps)) {
        return true;
    }
    // Touching or overlapping.
    let on = |p: DVec2, q: DVec2, r: DVec2| {
        side(p, q, r).abs() <= eps
            && r.x >= p.x.min(q.x) - eps
            && r.x <= p.x.max(q.x) + eps
            && r.y >= p.y.min(q.y) - eps
            && r.y <= p.y.max(q.y) + eps
    };
    on(c, d, a) || on(c, d, b) || on(a, b, c) || on(a, b, d)
}

/// The profile of a new project's sweep type: a baseboard 1.5 cm thick and
/// 10 cm tall, on the left of the path.
pub fn baseboard() -> Vec<DVec2> {
    vec![DVec2::new(0.0, 0.0), DVec2::new(0.015, 0.0), DVec2::new(0.015, 0.1), DVec2::new(0.0, 0.1)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiles_are_checked() {
        assert_eq!(check_profile(&baseboard()), Ok(()));
        assert_eq!(check_profile(&baseboard()[..2]), Err(ProfileProblem::TooFewPoints));
        let bow = [DVec2::ZERO, DVec2::new(1.0, 1.0), DVec2::new(1.0, 0.0), DVec2::new(0.0, 1.0)];
        assert_eq!(check_profile(&bow), Err(ProfileProblem::Crossing));
        let flat = [DVec2::ZERO, DVec2::X, DVec2::new(2.0, 0.0)];
        assert_eq!(check_profile(&flat), Err(ProfileProblem::NoArea));
        let mut nan = baseboard();
        nan[1].x = f64::NAN;
        assert_eq!(check_profile(&nan), Err(ProfileProblem::NotFinite));
        let huge = [DVec2::ZERO, DVec2::new(20.0, 0.0), DVec2::new(0.0, 1.0)];
        assert_eq!(check_profile(&huge), Err(ProfileProblem::TooLarge));
    }

    #[test]
    fn closed_paths_count_the_way_back() {
        let square = vec![DVec2::ZERO, DVec2::X, DVec2::ONE, DVec2::Y];
        let open = Sweep { path: square.clone(), closed: false, base_offset: 0.0, flip: false, bulges: Vec::new() };
        assert!((open.length() - 3.0).abs() < 1e-12 && open.is_valid());
        let closed = Sweep { closed: true, ..open };
        assert!((closed.length() - 4.0).abs() < 1e-12 && closed.is_valid());
        let short = Sweep { path: square[..2].to_vec(), ..closed };
        assert!(!short.is_valid());
    }

    #[test]
    fn a_circle_is_two_half_circles_and_arcs_count_as_arcs() {
        let curves = [SketchCurve::Circle { center: DVec2::new(1.0, 2.0), radius: 3.0 }];
        let path = path_from_curves(&curves).unwrap();
        assert!(path.closed);
        let circle = Sweep { path: path.points, closed: true, base_offset: 0.0, flip: false, bulges: path.bulges };
        assert!(circle.is_valid() && circle.is_curved());
        assert!((circle.length() - std::f64::consts::TAU * 3.0).abs() < 1e-12);
        // 3° steps: 60 stations per half circle, none repeated at the ends of a loop.
        assert_eq!(circle.stations(), 120);
        // Open, the closing half circle is kept but unused.
        let open = Sweep { closed: false, ..circle.clone() };
        assert!(open.is_valid());
        assert!((open.length() - std::f64::consts::PI * 3.0).abs() < 1e-12);
        // Two points close only around an arc.
        let flat = Sweep { bulges: vec![0.0, 0.0], ..circle };
        assert!(!flat.is_valid());
    }

    #[test]
    fn curves_chain_into_one_path_the_way_the_first_runs() {
        let line = |a: (f64, f64), b: (f64, f64)| SketchCurve::Line { a: a.into(), b: b.into() };
        // A half circle from (2, 0) round to (-2, 0), then lines given backwards and out of order.
        let arc = SketchCurve::Arc { a: DVec2::new(2.0, 0.0), b: DVec2::new(-2.0, 0.0), bulge: 1.0 };
        let curves = [arc.clone(), line((-2.0, -3.0), (-2.0, 0.0)), line((2.0, -3.0), (2.0, 0.0))];
        let path = path_from_curves(&curves).unwrap();
        assert!(!path.closed);
        let want = [(2.0, -3.0), (2.0, 0.0), (-2.0, 0.0), (-2.0, -3.0)].map(DVec2::from);
        assert_eq!(path.points, want);
        assert_eq!(path.bulges, [0.0, 1.0, 0.0]);
        // The same curves with the arc reversed run the other way.
        let back = path_from_curves(&[arc.reversed(), curves[1].clone(), curves[2].clone()]).unwrap();
        assert_eq!(back.points, want.iter().rev().copied().collect::<Vec<_>>());
        assert_eq!(back.bulges, [0.0, -1.0, 0.0]);
        // Closed by a line across: a D.
        let d = path_from_curves(&[arc.clone(), line((-2.0, 0.0), (2.0, 0.0))]).unwrap();
        assert!(d.closed && d.points.len() == 2);
        // Refused: a branch, apart curves, a circle with others, an ellipse, a retraced loop.
        let branch = [arc.clone(), line((2.0, 0.0), (2.0, -3.0)), line((2.0, 0.0), (5.0, 0.0))];
        assert!(matches!(path_from_curves(&branch), Err(PathProblem::Branch(_))));
        let apart = [arc.clone(), line((5.0, 5.0), (6.0, 5.0))];
        assert!(matches!(path_from_curves(&apart), Err(PathProblem::Apart(_))));
        let circle = SketchCurve::Circle { center: DVec2::ZERO, radius: 1.0 };
        assert_eq!(path_from_curves(&[circle, arc.clone()]), Err(PathProblem::CircleWithOthers));
        let ellipse = SketchCurve::ellipse(DVec2::ZERO, DVec2::new(2.0, 1.0), 0.0);
        assert_eq!(path_from_curves(&[ellipse]), Err(PathProblem::Unsupported(0)));
        let retraced = [line((0.0, 0.0), (1.0, 0.0)), line((1.0, 0.0), (0.0, 0.0))];
        assert_eq!(path_from_curves(&retraced), Err(PathProblem::NoArea));
        let nan = SketchCurve::Arc { a: DVec2::ZERO, b: DVec2::X, bulge: f64::NAN };
        assert_eq!(path_from_curves(&[nan]), Err(PathProblem::Invalid(0)));
        assert_eq!(path_from_curves(&[]), Err(PathProblem::Empty));
    }

    #[test]
    fn bad_bulges_are_repaired() {
        let path = vec![DVec2::ZERO, DVec2::new(4.0, 0.0)];
        let mut s = Sweep { path, closed: false, base_offset: 0.0, flip: false, bulges: vec![f64::NAN, 50.0, 1.0] };
        assert!(!s.is_valid());
        assert_eq!(s.repair(), 3);
        assert_eq!(s.bulges, [0.0, MAX_BULGE]);
        assert!(s.is_valid());
        assert_eq!(s.repair(), 0);
        // A mirrored straight segment's -0 needs no repair.
        s.bulges = vec![-0.0, -1.0];
        assert_eq!(s.repair(), 0);
    }
}
