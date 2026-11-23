//! Reinforcing bars in floor plans, elevations and sections: the bars of a
//! set as the view sees them, within its range. A bar the view's cut plane
//! crosses (a plan's cut height, a section's line) at least
//! [`CUT_STEEPNESS`] steeply is drawn where it is cut, as a dot as wide as
//! the bar; a bar crossing it more obliquely, nearly in the plane, is drawn
//! as a line only. The parts of bars between the cut and the view's depth
//! (a plan down to the level below, a section to its depth, within its
//! line's ends) are drawn as thin lines, and a leg of an open bar seen
//! end-on there (the vertical leg of a starter bar below a plan's cut,
//! whatever its hook) as a dot; a closed loop (a stirrup) is drawn by its
//! other legs. Nothing in front of the cut is drawn. Views with
//! [`cc_core::View::rebar`] draw every set this way over the model; bar tags
//! draw the sets they tag ([`crate::bar_tags`]), and a ceiling plan, which
//! has no range here, draws a tagged set whole, seen from above.

use crate::projected::{Frame, clip_segment};
use crate::{Drawing2D, LineStyle};
use cc_core::{ElementId, Project, View, ViewKind};
use cc_geom::ModelGeometry;
use cc_geom::rebar::RebarGeom;
use glam::{DVec2, DVec3};

/// Fill of the dots of cut bars and bars seen end-on.
pub const DOT_FILL: [u8; 4] = [20, 20, 20, 255];
/// Least radius of a dot on paper, millimeters, so a thin bar stays visible.
pub const MIN_DOT: f64 = 0.35;
/// Most points drawn for one set's bars.
const MAX_POINTS: usize = 200_000;
/// A bar is drawn as cut where it crosses the cut plane at least this
/// steeply (the cosine of its angle to the view direction, 60° from the
/// plane): bars lying nearly in the plane are drawn as lines.
pub const CUT_STEEPNESS: f64 = 0.5;
/// A segment whose projection is shorter than this is seen end-on, meters.
pub const END_ON: f64 = 1e-6;

/// The radius a bar `diameter` thick is drawn with as a dot at `k` meters
/// per paper millimeter: half the bar, at least [`MIN_DOT`] on paper.
pub fn dot_radius(diameter: f64, k: f64) -> f64 {
    (diameter / 2.0).max(MIN_DOT * k)
}

/// A filled dot of radius `r` at `at`, tagged with `id`.
pub fn dot(d: &mut Drawing2D, at: DVec2, r: f64, id: ElementId) {
    let pts = (0..16).map(|i| at + DVec2::from_angle(i as f64 / 16.0 * std::f64::consts::TAU) * r).collect();
    d.fill(pts, DOT_FILL, Some(id));
}

/// How a view sees bars: the frame it projects them through, with its cut
/// plane (`near`) and depth (`far`) and, in a section, its line's ends; the
/// clipping planes computed once.
pub(crate) struct BarRange {
    frame: Frame,
    /// Every clipping plane, the cut plane first when there is one.
    planes: Vec<(DVec3, DVec3)>,
}

impl BarRange {
    /// The range of `view`: a floor plan from its cut height down to the
    /// level below (no limit under the lowest), a section from its line to
    /// its depth, an elevation and a ceiling plan everything (a ceiling plan
    /// as seen from above, as bar tags drew it); none for other views.
    pub(crate) fn of(project: &Project, view: &View) -> Option<Self> {
        let frame = match &view.kind {
            ViewKind::FloorPlan { level, cut_height, .. } => {
                let elevation = project.level_elevation(*level);
                let cut = elevation + cut_height;
                let below = project
                    .levels
                    .iter()
                    .map(|l| l.elevation)
                    .filter(|e| *e < elevation - 1e-9)
                    .fold(f64::NEG_INFINITY, f64::max);
                let mut frame = Frame::looking(DVec3::new(0.0, 0.0, cut), DVec3::NEG_Z);
                frame.near = Some(0.0);
                frame.far = below.is_finite().then_some(cut - below);
                frame
            }
            ViewKind::CeilingPlan { .. } => Frame::looking(DVec3::ZERO, DVec3::NEG_Z),
            ViewKind::Elevation { .. } | ViewKind::Section { .. } => crate::projected::view_frame(view)?,
            _ => return None,
        };
        let planes = frame.planes();
        Some(Self { frame, planes })
    }

    /// The distance of `p` past the cut plane (none without one).
    fn past_cut(&self, p: DVec3) -> Option<f64> {
        let near = self.frame.near?;
        Some((p - self.frame.origin).dot(self.frame.forward) - near)
    }

    /// Whether `p` lies within the view's other limits (depth and line ends).
    fn within(&self, p: DVec3) -> bool {
        let skip = usize::from(self.frame.near.is_some());
        self.planes.iter().skip(skip).all(|(at, n)| (p - *at).dot(*n) >= -1e-9)
    }

    /// The part of segment `a`-`b` within the range.
    fn clip(&self, a: DVec3, b: DVec3) -> Option<(DVec3, DVec3)> {
        self.planes.iter().try_fold((a, b), |(a, b), &(p, n)| clip_segment(a, b, p, n))
    }

    /// Whether a box from `lo` to `hi` lies wholly outside one of the planes.
    fn misses(&self, lo: DVec3, hi: DVec3) -> bool {
        self.planes.iter().any(|(at, n)| {
            // The corner of the box farthest along the plane's normal.
            let far = DVec3::new(
                if n.x >= 0.0 { hi.x } else { lo.x },
                if n.y >= 0.0 { hi.y } else { lo.y },
                if n.z >= 0.0 { hi.z } else { lo.z },
            );
            (far - *at).dot(*n) < -1e-9
        })
    }
}

/// Draw the bars of `set` as `range` sees them, at `k` meters per paper
/// millimeter, tagged with `id`.
pub(crate) fn draw_set(d: &mut Drawing2D, set: &RebarGeom, range: &BarRange, k: f64, id: ElementId) {
    let frame = &range.frame;
    // A set wholly outside the range draws nothing.
    let points = set.bars.iter().flat_map(|b| b.points.iter()).filter(|p| p.is_finite());
    let (lo, hi) = points.fold((DVec3::INFINITY, DVec3::NEG_INFINITY), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
    if lo.cmple(hi).all() && range.misses(lo, hi) {
        return;
    }
    let r = dot_radius(set.diameter, k);
    let mut budget = MAX_POINTS;
    for bar in &set.bars {
        if bar.points.len() > budget {
            break;
        }
        budget -= bar.points.len();
        let n = bar.points.len();
        let count = if bar.closed && n > 2 { n } else { n.saturating_sub(1) };
        let mut run: Vec<DVec2> = Vec::new();
        let mut runs: Vec<Vec<DVec2>> = Vec::new();
        let mut dots: Vec<DVec2> = Vec::new();
        for i in 0..count {
            let (Some(&a), Some(&b)) = (bar.points.get(i), bar.points.get((i + 1) % n)) else { continue };
            // Where the bar passes through the cut plane, steeply enough: a dot.
            let mut cut = false;
            if let (Some(da), Some(db)) = (range.past_cut(a), range.past_cut(b))
                && (da < 0.0) != (db < 0.0)
                && (b - a).normalize_or_zero().dot(frame.forward).abs() >= CUT_STEEPNESS
            {
                let at = a.lerp(b, da / (da - db));
                if range.within(at) {
                    dots.push(frame.project(at));
                    cut = true;
                }
            }
            match range.clip(a, b) {
                Some((p, q)) => {
                    let (p2, q2) = (frame.project(p), frame.project(q));
                    if p2.distance(q2) < END_ON {
                        // A leg of an open bar seen end-on within the range (and not cut
                        // there): a dot. A closed loop (a stirrup) is drawn by its other
                        // legs, its corners not dotted.
                        if !cut && !bar.closed && p.distance(q) > END_ON {
                            dots.push(p2);
                        }
                        continue;
                    }
                    // A run goes on while the bar stays within the range.
                    if run.last().is_none_or(|l| l.distance(p2) > 1e-9) {
                        if run.len() > 1 {
                            runs.push(std::mem::take(&mut run));
                        }
                        run = vec![p2];
                    }
                    run.push(q2);
                }
                None if run.len() > 1 => runs.push(std::mem::take(&mut run)),
                None => run.clear(),
            }
        }
        if run.len() > 1 {
            runs.push(run);
        }
        let mut drawn: Vec<DVec2> = Vec::new();
        for at in dots {
            if !drawn.iter().any(|p| p.distance(at) < r) {
                dot(d, at, r, id);
                drawn.push(at);
            }
        }
        let whole = bar.closed && runs.len() == 1 && runs[0].len() == count + 1;
        for pts in runs {
            if whole {
                d.polyline(&pts[..pts.len() - 1], true, LineStyle::Thin, Some(id));
            } else {
                d.polyline(&pts, false, LineStyle::Thin, Some(id));
            }
        }
    }
}

/// Draw every reinforcement set of the model in `view`, when the view
/// shows reinforcement ([`View::rebar`]).
pub(crate) fn draw_all(d: &mut Drawing2D, project: &Project, geom: &ModelGeometry, view: &View) {
    if !view.rebar || !view.kind.takes_rebar() {
        return;
    }
    let Some(range) = BarRange::of(project, view) else { return };
    let k = view.scale.max(1) as f64 / 1000.0;
    for (id, set) in &geom.rebar {
        draw_set(d, set, &range, k, *id);
    }
}
