//! Spline walls (see [`cc_core::wall_spline`]): a wall whose location line
//! is a smooth free-form curve, a chain of cubic Bézier segments.
//! `wall_create` draws one from `start` `through` points to `end`, or by its `control` points,
//! `element_get` shows the control points in `params.spline` (and the
//! curve's figures in `derived.spline`) and `element_set` patches them.
//!
//! First stage: a spline wall takes no doors, windows or niches, no
//! attached top, no elevation profile, no curtain wall type, no
//! reinforcement and no framing, and is not trimmed, extended or aligned;
//! the commands that would give it one say so, through the checks here.

use crate::error::{EngineError, Result};
use crate::refs::{Point, points};
use cc_core::wall_spline::{SplineError, WallSpline};
use cc_core::{Element, ElementId, ElementTypeId, Project, Wall};
use glam::DVec2;
use serde_json::{Value, json};

/// A spline's error as a command error.
pub(crate) fn spline_error(e: SplineError) -> EngineError {
    EngineError::invalid(e.to_string())
}

/// The spline from `start` to `end` that the `through` points between them
/// or the `control` points draw, when either is given. The ends may be
/// repeated among the points through (points repeated in a row count
/// once); the control points start at `start` and end at `end`.
pub(crate) fn spline_arg(
    start: DVec2,
    end: DVec2,
    through: Option<&[Point]>,
    control: Option<&[Point]>,
) -> Result<Option<WallSpline>> {
    match (through, control) {
        (None, None) => Ok(None),
        (Some(between), None) => {
            let mut fit = vec![start];
            fit.extend(points(between)?);
            fit.push(end);
            WallSpline::through(&fit).map(Some).ok_or_else(|| {
                EngineError::invalid(format!(
                    "a spline wall passes `through` at most {} points between its ends",
                    cc_core::wall_spline::MAX_SEGMENTS - 1
                ))
            })
        }
        (None, Some(control)) => {
            let control = points(control)?;
            let ends = control.first().zip(control.last());
            if cc_core::spline::segment_count(&control).is_none() {
                return Err(spline_error(SplineError::Points));
            }
            if !ends.is_some_and(|(a, b)| a.distance(start) < 1e-6 && b.distance(end) < 1e-6) {
                return Err(EngineError::invalid(
                    "a spline wall's `control` points run from `start` to `end`: its first and last points are those",
                ));
            }
            Ok(Some(WallSpline { control }))
        }
        (Some(_), Some(_)) => {
            Err(EngineError::invalid("give a spline wall's `through` points or its `control` points, not both"))
        }
    }
}

/// Whether `wall` of type `type_id` may be the spline wall it says (any
/// other wall passes): its type is no curtain wall, it has no bulge,
/// profile or attached top, its curve fits its thickness, and (`id`: the
/// wall it already is) it hosts nothing spline walls do not take yet.
pub(crate) fn check_wall(p: &Project, id: Option<ElementId>, type_id: ElementTypeId, wall: &Wall) -> Result<()> {
    let Some(spline) = &wall.spline else { return Ok(()) };
    if p.element_type(type_id).is_some_and(|t| t.curtain().is_some()) {
        return Err(EngineError::invalid("a curtain wall follows lines and arcs: its type makes no spline wall yet"));
    }
    if wall.bulge != 0.0 {
        return Err(EngineError::invalid(
            "a wall is an arc (a bulge) or a spline, not both: take the spline off (\"spline\": null) to make it an arc",
        ));
    }
    if wall.profile.is_some() {
        return Err(EngineError::invalid("a spline wall takes no elevation profile yet"));
    }
    if wall.attach_top.is_some() {
        return Err(EngineError::invalid("the top of a spline wall is not attached to a roof, ceiling or floor yet"));
    }
    p.check_wall_spline(type_id, wall.location_line, spline).map_err(spline_error)?;
    let Some(id) = id else { return Ok(()) };
    if p.hosted_by(id).next().is_some() {
        return Err(EngineError::invalid(format!(
            "wall {} hosts doors, windows or niches: a spline wall takes none yet",
            id.0
        )));
    }
    if p.rebar_of(id).next().is_some() {
        return Err(EngineError::invalid(format!(
            "wall {} is reinforced: a spline wall takes no reinforcement yet",
            id.0
        )));
    }
    if p.wall_framing_of(id).is_some() {
        return Err(EngineError::invalid(format!("wall {} is framed: a spline wall takes no framing yet", id.0)));
    }
    Ok(())
}

/// Refuse `what` (a sentence fragment: what spline walls do not take yet)
/// when `id` is a spline wall.
pub(crate) fn refuse(p: &Project, id: ElementId, what: &str) -> Result<()> {
    match p.element(id).and_then(|e| e.as_wall()) {
        Some(w) if w.spline.is_some() => Err(EngineError::invalid(format!("wall {} is a spline wall: {what}", id.0))),
        _ => Ok(()),
    }
}

/// `derived.spline` of `element_get`: the number of segments, the least
/// radius of curvature of the location line (`min_radius`) and the least
/// radius left to a face inside a bend (`face_radius`; both null for a
/// straight curve), and the points the segments join at (`knots`).
pub(crate) fn derived(p: &Project, e: &Element, w: &Wall) -> Option<Value> {
    let spline = w.spline.as_ref()?;
    let path = spline.path()?;
    let (left, right) = p.wall_faces(e.type_id, w.location_line);
    let radius = |r: f64| r.is_finite().then_some(r);
    Some(json!({
        "segments": cc_core::spline::segment_count(&spline.control),
        "min_radius": radius(path.min_radius()),
        "face_radius": radius(path.face_radius(left, right)),
        "knots": spline.knots().map(|k| k.to_array()).collect::<Vec<_>>(),
    }))
}
