//! The solid of a swept profile: its type's profile carried along its path
//! in plan, at the level's elevation plus its base offset, mitered at the
//! corners (and around a closed path). Along an arc the profile stands
//! square to the path (radial) at stations every [`ARC_STEP`], the sides
//! between them flat facets marked smooth where the surface they make is
//! curved; the volume of such a sweep is exact ([`exact_volume`]).

use cc_core::arc::{ARC_STEP, segments};
use cc_core::component::SketchPlane;
use cc_core::profile_sweep::{MAX_STATIONS, Sweep};
use cc_core::sweep::MAX_MITER;
use cc_core::{Element, Project, TypeData};
use glam::{DVec2, DVec3};

/// The profile of a sweep's type, on the side the sweep puts it.
pub fn sweep_profile(project: &Project, element: &Element, sweep: &Sweep) -> Option<Vec<DVec2>> {
    let TypeData::Sweep { profile, .. } = &project.element_type(element.type_id)?.data else { return None };
    cc_core::profile_sweep::check_profile(profile).ok()?;
    let side = if sweep.flip { -1.0 } else { 1.0 };
    let placed: Vec<DVec2> = profile.iter().map(|p| DVec2::new(p.x * side, p.y)).collect();
    Some(cc_core::sweep::counter_clockwise(&placed))
}

/// The faces of a sweep (outward) and, one per face, whether it is a facet
/// of a curved surface.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SweepMesh {
    pub faces: Vec<Vec<DVec3>>,
    pub smooth: Vec<bool>,
}

/// The faces of a sweep (outward), or `None` when its path or profile cannot make one.
pub fn sweep_faces(project: &Project, element: &Element, sweep: &Sweep) -> Option<Vec<Vec<DVec3>>> {
    sweep_mesh(project, element, sweep).map(|m| m.faces)
}

/// The faces of a sweep with the facets of its curved surfaces marked, or
/// `None` when its path or profile cannot make one.
pub fn sweep_mesh(project: &Project, element: &Element, sweep: &Sweep) -> Option<SweepMesh> {
    if !sweep.is_valid() {
        return None;
    }
    let profile = sweep_profile(project, element, sweep)?;
    let z = project.level_elevation(element.level) + sweep.base_offset;
    if sweep.is_curved() {
        // A profile drawn with arcs: the sides along its arcs are smooth too.
        let rounded = matches!(
            project.element_type(element.type_id).map(|t| &t.data),
            Some(TypeData::Sweep { sketch: Some(sketch), .. }) if sketch.is_curved()
        );
        let rounded = if rounded { crate::section::smooth_sides(&profile) } else { vec![false; profile.len()] };
        return curved_mesh(sweep, z, &profile, &rounded);
    }
    let faces = if sweep.closed {
        cc_core::sweep::sweep_closed(SketchPlane::Plan, &sweep.path, z, &profile)
    } else {
        let mut path = sweep.path.clone();
        path.dedup_by(|a, b| a.distance(*b) < 1e-9);
        cc_core::sweep::sweep(SketchPlane::Plan, &path, z, &profile)
    };
    let faces = faces.ok()?;
    Some(SweepMesh { smooth: vec![false; faces.len()], faces })
}

/// The exact volume of a sweep with arcs: the profile's area times the
/// length of the path its centroid runs (Pappus), each arc at the
/// centroid's own radius, each mitered corner taking `2 c tan(θ / 2)` off
/// it (the centroid `c` to the left, the path turning `θ` to the left).
/// `None` for a straight path (its mitered solid is exact already) and for
/// one with a corner so sharp that the solid's miter is cut short.
pub fn exact_volume(project: &Project, element: &Element, sweep: &Sweep) -> Option<f64> {
    if !sweep.is_valid() || !sweep.is_curved() {
        return None;
    }
    let profile = sweep_profile(project, element, sweep)?;
    let (area, centroid) = area_and_centroid(&profile)?;
    let c = centroid.x;
    let segs = sweep.segments();
    let tangents = tangents(sweep)?;
    let mut length: f64 = segs
        .iter()
        .map(|(a, b, arc)| arc.map_or_else(|| a.distance(*b), |arc| arc.sweep.abs() * arc.radius_at(c)))
        .sum();
    let joints = if sweep.closed { tangents.len() } else { tangents.len().saturating_sub(1) };
    for i in 0..joints {
        let (Some(before), Some(after)) = (tangents.get(i), tangents.get((i + 1) % tangents.len())) else { break };
        let (t_in, t_out) = (before.1, after.0);
        let half = t_in.perp_dot(t_out).atan2(t_in.dot(t_out)) * 0.5;
        if half.cos() < 1.0 / MAX_MITER {
            return None;
        }
        length -= 2.0 * c * half.tan();
    }
    let volume = area * length;
    (volume.is_finite() && volume > 0.0).then_some(volume)
}

/// Area and centroid of a counter-clockwise outline.
fn area_and_centroid(outline: &[DVec2]) -> Option<(f64, DVec2)> {
    let n = outline.len();
    let (mut twice, mut sum) = (0.0, DVec2::ZERO);
    for i in 0..n {
        let (a, b) = (outline[i], outline[(i + 1) % n]);
        let cross = a.perp_dot(b);
        twice += cross;
        sum += (a + b) * cross;
    }
    let area = twice * 0.5;
    let centroid = sum / (3.0 * twice);
    (area.is_finite() && area > 1e-12 && centroid.is_finite()).then_some((area, centroid))
}

/// The direction of travel at the start and end of each segment.
fn tangents(sweep: &Sweep) -> Option<Vec<(DVec2, DVec2)>> {
    sweep
        .segments()
        .iter()
        .map(|(a, b, arc)| match arc {
            Some(arc) => Some((arc.tangent(0.0), arc.tangent(arc.length()))),
            None => (*b - *a).try_normalize().map(|d| (d, d)),
        })
        .collect()
}

/// A point of the path where the solid has a section: where it is, and
/// the directions the path arrives and leaves in.
struct Station {
    at: DVec2,
    before: Option<DVec2>,
    after: Option<DVec2>,
}

/// The stations of a path, and for each span between two of them (from
/// the last back to the first on a closed path) whether it is on an arc.
fn stations(sweep: &Sweep) -> Option<(Vec<Station>, Vec<bool>)> {
    let segs = sweep.segments();
    let tangents = tangents(sweep)?;
    let mut stations = Vec::new();
    let mut on_arc = Vec::new();
    for (i, ((a, _, arc), (start, _))) in segs.iter().zip(&tangents).enumerate() {
        let before = match i.checked_sub(1) {
            Some(j) => tangents.get(j).map(|t| t.1),
            None if sweep.closed => tangents.last().map(|t| t.1),
            None => None,
        };
        stations.push(Station { at: *a, before, after: Some(*start) });
        match arc {
            Some(arc) => {
                let n = segments(arc.sweep, ARC_STEP);
                for k in 1..n {
                    let u = arc.length() * k as f64 / n as f64;
                    let t = arc.tangent(u);
                    stations.push(Station { at: arc.point(u), before: Some(t), after: Some(t) });
                }
                on_arc.extend(std::iter::repeat_n(true, n));
            }
            None => on_arc.push(false),
        }
        if stations.len() > MAX_STATIONS {
            return None;
        }
    }
    if !sweep.closed {
        let (_, end, _) = segs.last()?;
        stations.push(Station { at: *end, before: tangents.last().map(|t| t.1), after: None });
    }
    Some((stations, on_arc))
}

/// The profile placed at a station: square to the path, or in the plane
/// bisecting a corner (stretched across to meet both sides, as far as
/// [`MAX_MITER`]).
fn ring(s: &Station, z: f64, profile: &[DVec2]) -> Vec<DVec3> {
    let (d, scale) = match (s.before, s.after) {
        // Along an arc, or where a line runs on tangent to it: square.
        (Some(a), Some(b)) if a.dot(b) > 1.0 - 1e-12 => (b, 1.0),
        (Some(a), Some(b)) => {
            let d = (a + b).try_normalize().unwrap_or(b);
            (d, (1.0 / d.dot(b).max(1.0 / MAX_MITER)).min(MAX_MITER))
        }
        (Some(d), None) | (None, Some(d)) => (d, 1.0),
        (None, None) => (DVec2::X, 1.0),
    };
    let left = d.perp();
    profile
        .iter()
        .map(|q| {
            let p = s.at + left * (q.x * scale);
            DVec3::new(p.x, p.y, z + q.y)
        })
        .collect()
}

/// Whether four points lie in one plane.
fn planar(q: &[DVec3; 4]) -> bool {
    let n = (q[1] - q[0]).cross(q[2] - q[0]) + (q[2] - q[0]).cross(q[3] - q[0]);
    let Some(n) = n.try_normalize() else { return true };
    q.iter().all(|p| (*p - q[0]).dot(n).abs() < 1e-9)
}

/// The solid of a counter-clockwise `profile` carried along a path with
/// arcs, at height `z`: a section at every station, sides between them
/// (two triangles where a side is not flat), smooth on the arcs except
/// those of the profile's level edges, and all along those of the
/// profile's edges flagged in `rounded` (the facets of its own arcs); end
/// caps on an open path.
fn curved_mesh(sweep: &Sweep, z: f64, profile: &[DVec2], rounded: &[bool]) -> Option<SweepMesh> {
    let (stations, on_arc) = stations(sweep)?;
    let rings: Vec<Vec<DVec3>> = stations.iter().map(|s| ring(s, z, profile)).collect();
    let m = profile.len();
    let count = rings.len();
    if m < 3 || count < 2 {
        return None;
    }
    let mut mesh = SweepMesh::default();
    if !sweep.closed
        && let (Some(first), Some(last)) = (rings.first(), rings.last())
    {
        mesh.faces.push(first.clone());
        mesh.faces.push(last.iter().rev().copied().collect());
        mesh.smooth.extend([false, false]);
    }
    for (i, &curved) in on_arc.iter().enumerate() {
        let (Some(r0), Some(r1)) = (rings.get(i), rings.get((i + 1) % count)) else { continue };
        for j in 0..m {
            let k = (j + 1) % m;
            let (Some(pj), Some(pk)) = (profile.get(j), profile.get(k)) else { continue };
            // The level edges of the profile sweep flat rings; the others curved surfaces.
            let smooth = (curved && (pj.y - pk.y).abs() > 1e-9) || rounded.get(j).copied().unwrap_or(false);
            let (Some(&a), Some(&b), Some(&c), Some(&d)) = (r0.get(j), r1.get(j), r1.get(k), r0.get(k)) else {
                continue;
            };
            let quad = [a, b, c, d];
            if planar(&quad) {
                mesh.faces.push(quad.to_vec());
                mesh.smooth.push(smooth);
            } else {
                mesh.faces.push(vec![a, b, c]);
                mesh.faces.push(vec![a, c, d]);
                mesh.smooth.extend([smooth, smooth]);
            }
        }
    }
    // Outward faces.
    let volume = cc_core::sweep::volume(&mesh.faces);
    if !volume.is_finite() || volume.abs() < 1e-12 {
        return None;
    }
    if volume < 0.0 {
        mesh.faces.iter_mut().for_each(|f| f.reverse());
    }
    Some(mesh)
}
