//! Reinforcing bars in floor plans, elevations and sections: the bars of a
//! set as the view sees them, within its range. A bar the view's cut plane
//! crosses (a plan's cut height, a section's line) is drawn where it is
//! cut, as a dot as wide as the bar; the parts of bars between the cut and
//! the view's depth (a plan down to the level below, a section to its
//! depth, within its line's ends) are drawn as thin lines, and a bar seen
//! end-on there as a dot. Nothing in front of the cut is drawn. Views with
//! [`cc_core::View::rebar`] draw every set this way over the model; bar
//! tags draw the sets they tag ([`crate::bar_tags`]).

use crate::projected::{Frame, clip_segment};
use crate::{Drawing2D, LineStyle};
use cc_core::{ElementId, Project, View, ViewKind};
use cc_geom::ModelGeometry;
use cc_geom::rebar::RebarGeom;
use glam::{DVec2, DVec3};

/// Fill of the dots of cut bars and bars seen end-on.
pub(crate) const DOT_FILL: [u8; 4] = [20, 20, 20, 255];
/// Least radius of a dot on paper, millimeters, so a thin bar stays visible.
const MIN_DOT: f64 = 0.35;
/// Most points drawn for one set's bars.
const MAX_POINTS: usize = 200_000;
/// A bar is drawn as cut where it crosses the cut plane at least this
/// steeply (the cosine of its angle to the view direction): bars lying
/// nearly in the plane are drawn as lines.
const CUT_STEEPNESS: f64 = 0.5;
/// Projected runs shorter than this are a bar seen end-on, meters.
const END_ON: f64 = 1e-6;

/// How a view sees bars: the frame it projects them through, with its cut
/// plane (`near`) and depth (`far`) and, in a section, its line's ends.
pub(crate) struct BarRange {
    frame: Frame,
}

impl BarRange {
    /// The range of `view`: a floor plan from its cut height down to the
    /// level below (no limit under the lowest), a section from its line to
    /// its depth, an elevation everything; none for other views.
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
            ViewKind::Elevation { .. } | ViewKind::Section { .. } => crate::projected::view_frame(view)?,
            _ => return None,
        };
        Some(Self { frame })
    }

    /// The distance of `p` past the cut plane (none without one).
    fn past_cut(&self, p: DVec3) -> Option<f64> {
        let near = self.frame.near?;
        Some((p - self.frame.origin).dot(self.frame.forward) - near)
    }

    /// Whether `p` lies within the view's other limits (depth and line ends).
    fn within(&self, p: DVec3) -> bool {
        self.frame
            .planes()
            .iter()
            .skip(usize::from(self.frame.near.is_some()))
            .all(|(at, n)| (p - *at).dot(*n) >= -1e-9)
    }

    /// The part of segment `a`-`b` within the range.
    fn clip(&self, a: DVec3, b: DVec3) -> Option<(DVec3, DVec3)> {
        self.frame.planes().iter().try_fold((a, b), |(a, b), &(p, n)| clip_segment(a, b, p, n))
    }
}

/// A filled dot of radius `r` at `at`.
fn dot(d: &mut Drawing2D, at: DVec2, r: f64, id: ElementId) {
    let pts = (0..16).map(|i| at + DVec2::from_angle(i as f64 / 16.0 * std::f64::consts::TAU) * r).collect();
    d.fill(pts, DOT_FILL, Some(id));
}

/// Draw the bars of `set` as `range` sees them, at `k` meters per paper
/// millimeter, tagged with `id`.
pub(crate) fn draw_set(d: &mut Drawing2D, set: &RebarGeom, range: &BarRange, k: f64, id: ElementId) {
    let frame = &range.frame;
    let r = (set.diameter / 2.0).max(MIN_DOT * k);
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
        let mut cut = false;
        for i in 0..count {
            let (Some(&a), Some(&b)) = (bar.points.get(i), bar.points.get((i + 1) % n)) else { continue };
            // Where the bar passes through the cut plane, steeply enough: a dot.
            if let (Some(da), Some(db)) = (range.past_cut(a), range.past_cut(b))
                && (da < 0.0) != (db < 0.0)
                && (b - a).normalize_or_zero().dot(frame.forward).abs() >= CUT_STEEPNESS
            {
                let at = a.lerp(b, da / (da - db));
                if range.within(at) {
                    dot(d, frame.project(at), r, id);
                    cut = true;
                }
            }
            match range.clip(a, b) {
                Some((p, q)) => {
                    let (p, q) = (frame.project(p), frame.project(q));
                    // A run goes on while the bar stays within the range.
                    if run.last().is_none_or(|l| l.distance(p) > 1e-9) {
                        if run.len() > 1 {
                            runs.push(std::mem::take(&mut run));
                        }
                        run = vec![p];
                    }
                    run.push(q);
                }
                None if run.len() > 1 => runs.push(std::mem::take(&mut run)),
                None => run.clear(),
            }
        }
        if run.len() > 1 {
            runs.push(run);
        }
        let length: f64 = runs.iter().flat_map(|r| r.windows(2)).map(|w| w[0].distance(w[1])).sum();
        if length < END_ON {
            // Seen end-on within the range (and not cut): a dot.
            if !cut && let Some(&at) = runs.first().and_then(|r| r.first()) {
                dot(d, at, r, id);
            }
            continue;
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
