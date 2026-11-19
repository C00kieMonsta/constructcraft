//! Pile reinforcement: longitudinal bars around a pile's section from a
//! length below its head up into its cap, and a spiral or closed ties
//! around them over that length, laid out from the pile when the geometry
//! is built, so they follow it when its cap's type changes.
//!
//! The bars stand around the section through [`crate::faces`] as a
//! column's do (a round pile's evenly around a circle, a square one's at
//! its corners and between them), inside the spiral or the ties, the cover
//! inside the face. They run from `length` below the head (0: the whole
//! pile, to the cover above its toe) to `projection` above it; under a cap
//! they reach at most the cap's top less the cover. A spiral is a helix
//! from the cover below the head down the length, drawn in chords with its
//! exact length (`sqrt((2πr)² + pitch²)` a turn, for as many turns, not
//! necessarily whole, as the height takes; no closing turns at its ends);
//! in a square pile it winds on the circle inside the sides. Ties are
//! closed loops every `spacing` over the length, closed by their hooks.

use crate::faces;
use crate::pile::PileGeom;
use crate::rebar::{Bar, Helix, LoopHooks, MAX_BARS, from_convex_corner, lap_of, stations};
use cc_core::pile::PileSection;
use cc_core::{FaceBars, Rebar, RebarLayout};
use glam::{DVec2, DVec3};
use std::f64::consts::TAU;

/// Chords of a round tie.
const TIE_CHORDS: usize = 32;
/// Chords of a spiral's turn.
const SPIRAL_CHORDS: f64 = 24.0;
/// Most points of a spiral's center line (as a shaped bar keeps).
const MAX_SPIRAL_POINTS: usize = cc_core::ShapedRebar::MAX_POINTS;

/// A pile as its bars see it.
pub(crate) struct PileInput<'a> {
    pub pile: &'a PileGeom,
    /// The top of the pile's cap (none for a free pile): the bars reach at
    /// most its cover below it.
    pub cap_top: Option<f64>,
    /// The largest diameter of the pile's spiral and ties (0: none): the
    /// longitudinal bars stand inside them.
    pub tie: f64,
}

/// The bottom of the stretch `length` below the head (0: the whole pile),
/// at least `cover` above the toe.
fn zone_bottom(p: &PileGeom, length: f64, cover: f64) -> f64 {
    let (toe, head) = p.z_range;
    let lowest = toe + cover;
    if length > 0.0 { (head - length).max(lowest) } else { lowest }
}

/// The bars of the set `r` (bars `d` thick) in the pile of `input`.
pub(crate) fn pile_bars(input: &PileInput, r: &Rebar, d: f64) -> Vec<Bar> {
    let p = input.pile;
    let cover = r.cover;
    if !(cover.is_finite() && d.is_finite() && d > 0.0 && p.position.is_finite()) {
        return Vec::new();
    }
    let rot = DVec2::from_angle(p.rotation_deg.to_radians());
    let place = |q: DVec2, z: f64| (p.position + rot.rotate(q)).extend(z);
    let head = p.z_range.1;
    let size = p.section.size();
    match r.layout {
        RebarLayout::PileLongitudinal { count, spacing, length, projection } => {
            let inset = cover + input.tie + d / 2.0;
            let around = match p.section {
                PileSection::Round { diameter } => {
                    let radius = diameter / 2.0 - inset;
                    if !(radius.is_finite() && radius > 1e-6) {
                        return Vec::new();
                    }
                    // A count, or as many as keep them at most `spacing` apart around the circle.
                    let n = if count > 0 {
                        count as usize
                    } else if spacing.is_finite() && spacing > 0.0 {
                        ((TAU * radius / spacing) - 1e-9).ceil().max(1.0) as usize
                    } else {
                        0
                    };
                    let n = n.min(cc_core::pile_bars::MAX_AROUND as usize);
                    (0..n).map(|i| DVec2::from_angle(TAU * i as f64 / n as f64) * radius).collect::<Vec<_>>()
                }
                PileSection::Square { side } => {
                    let h = side / 2.0;
                    let outline = [DVec2::new(-h, -h), DVec2::new(h, -h), DVec2::new(h, h), DVec2::new(-h, h)];
                    let faces = faces::outline_faces(&outline, inset);
                    let spec = if count > 0 { Vec::new() } else { vec![FaceBars { count: 0, spacing }; faces.len()] };
                    faces::face_bars(&faces, &spec, count as usize).into_iter().map(|b| b.at).collect()
                }
            };
            // From the bottom of the stretch up into the cap.
            let bottom = zone_bottom(p, length, cover);
            let mut top = head + projection.max(0.0);
            if let Some(cap) = input.cap_top {
                top = top.min(cap - cover);
            }
            let top = top.max(head);
            let height = top - bottom;
            if !(height.is_finite() && height > 1e-6) {
                return Vec::new();
            }
            let pieces = r.column_pieces(height, lap_of(r, d), false);
            let mut bars = Vec::new();
            for q in around {
                for (from, to) in &pieces {
                    if bars.len() >= MAX_BARS {
                        return bars;
                    }
                    bars.push(Bar::new(vec![place(q, bottom + from), place(q, bottom + to)], false));
                }
            }
            bars
        }
        RebarLayout::PileTies { spacing, length, hooks } => {
            let inset = cover + d / 2.0;
            let half = size / 2.0 - inset;
            if !(half.is_finite() && half > 1e-6) {
                return Vec::new();
            }
            let (outline, true_length) = match p.section {
                PileSection::Round { .. } => {
                    let points = (0..TIE_CHORDS)
                        .map(|i| DVec2::from_angle(TAU * i as f64 / TIE_CHORDS as f64) * half)
                        .collect::<Vec<_>>();
                    (points, Some(TAU * half))
                }
                PileSection::Square { .. } => {
                    let h = half;
                    (vec![DVec2::new(-h, -h), DVec2::new(h, -h), DVec2::new(h, h), DVec2::new(-h, h)], None)
                }
            };
            let hooks = hooks.and_then(|h| LoopHooks::of(&h, d));
            let outline = if hooks.is_some() { from_convex_corner(outline) } else { outline };
            stations(zone_bottom(p, length, cover), head - cover, spacing)
                .into_iter()
                .take(MAX_BARS)
                .map(|z| Bar { true_length, hooks, ..Bar::new(outline.iter().map(|q| place(*q, z)).collect(), true) })
                .collect()
        }
        RebarLayout::PileSpiral { pitch, length } => {
            let radius = size / 2.0 - cover - d / 2.0;
            let (z1, z0) = (head - cover, zone_bottom(p, length, cover));
            let height = z1 - z0;
            if !(radius > 1e-6 && height > 1e-6 && pitch.is_finite() && pitch > 0.0) {
                return Vec::new();
            }
            let helix = Helix { radius, pitch, turns: height / pitch };
            if !helix.is_valid() {
                return Vec::new();
            }
            // Chords along the coil, from the top down.
            let n = (helix.turns * SPIRAL_CHORDS).ceil();
            let n = if n.is_finite() { (n as usize).clamp(1, MAX_SPIRAL_POINTS - 1) } else { 1 };
            let points: Vec<DVec3> = (0..=n)
                .map(|k| {
                    let t = k as f64 / n as f64;
                    place(DVec2::from_angle(TAU * helix.turns * t) * radius, z1 - height * t)
                })
                .collect();
            vec![Bar { true_length: Some(helix.length()), helix: Some(helix), ..Bar::new(points, false) }]
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cc_core::ElementId;

    fn pile(section: PileSection) -> PileGeom {
        PileGeom {
            element: ElementId(1),
            position: DVec2::new(2.0, 3.0),
            rotation_deg: 0.0,
            section,
            z_range: (-12.0, 0.0),
            cap: None,
        }
    }

    #[test]
    fn a_spiral_has_the_exact_length_of_its_coil() {
        let p = pile(PileSection::Round { diameter: 0.4 });
        let r = Rebar::new(ElementId(2), RebarLayout::PileSpiral { pitch: 0.1, length: 3.0 }, 0.05);
        let bars = pile_bars(&PileInput { pile: &p, cap_top: None, tie: 0.0 }, &r, 0.01);
        assert_eq!(bars.len(), 1);
        // From 0.05 below the head down to 3 m below it: 2.95 m, 29.5 turns of radius 0.145.
        let (radius, turns) = (0.2 - 0.05 - 0.005, 2.95 / 0.1);
        let want = turns * ((TAU * radius).powi(2) + 0.01).sqrt();
        assert!((bars[0].length() - want).abs() < 1e-9, "{} vs {want}", bars[0].length());
        let h = bars[0].helix.unwrap();
        assert!((h.turns - turns).abs() < 1e-9 && (h.radius - radius).abs() < 1e-12);
        // Every point on the circle, the top at the cover below the head.
        assert!(bars[0].points.iter().all(|q| ((q.truncate() - p.position).length() - radius).abs() < 1e-9));
        assert!((bars[0].points[0].z + 0.05).abs() < 1e-12);
        // Values a file may hold: nothing, and no panic.
        for bad in [f64::NAN, -1.0, 0.0] {
            let r = Rebar::new(ElementId(2), RebarLayout::PileSpiral { pitch: bad, length: 3.0 }, 0.05);
            assert!(pile_bars(&PileInput { pile: &p, cap_top: None, tie: 0.0 }, &r, 0.01).is_empty());
        }
    }
}
