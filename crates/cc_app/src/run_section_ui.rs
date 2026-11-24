//! Drawing pipes, conduits and cable trays in sections and elevations. The
//! runs lie in a plane parallel to the view: at the depth of the run the
//! chain starts on, else (in a section) on the cut plane, so vertical runs
//! and the runs they join are drawn as they are seen. Clicks chain runs
//! like the plan tool; a click on a run seen in the view joins it there.

use crate::paint::{PREVIEW, Xf};
use crate::state::AppState;
use crate::t;
use crate::tools::{Run, Snap, draw_snap, snap_at, temp_dimension};
use cc_core::{Category, ElementId, ElementTypeId, LevelId, Project, View, ViewKind};
use cc_engine::Ref;
use cc_views::{Drawing2D, Frame};
use egui::{Color32, Painter, Response, Stroke, Ui};
use glam::{DVec2, DVec3};
use std::collections::BTreeSet;

/// Screen pixels within which the cursor takes a run's end or center line.
const REACH_PX: f64 = 8.0;
/// Shorter runs are not drawn, meters.
const MIN_RUN: f64 = 0.01;

/// Whether runs are drawn in `view`: sections and elevations.
pub fn hosts(view: &View) -> bool {
    matches!(view.kind, ViewKind::Section { .. } | ViewKind::Elevation { .. })
}

/// A run of the tool's network seen in the view: its center line in the
/// drawing (`a2`, `b2`) and in the model (`a`, `b`).
struct Seen {
    a2: DVec2,
    b2: DVec2,
    a: DVec3,
    b: DVec3,
}

/// The category of the runs a tool draws and joins, and the pipes' system.
fn network(state: &AppState, run: Run) -> (Category, Option<cc_core::SystemId>) {
    match run {
        Run::Pipe => (Category::Pipe, state.mep.system),
        Run::Conduit => (Category::Conduit, None),
        Run::Tray => (Category::CableTray, None),
        Run::Conductor => (Category::LightningConductor, None),
    }
}

/// Whether `p` lies within the depth the view shows.
fn visible(frame: &Frame, p: DVec3) -> bool {
    let tol = cc_geom::mep_net::TOL;
    let d = frame.depth(p);
    frame.near.is_none_or(|n| d >= n - tol) && frame.far.is_none_or(|f| d <= f + tol)
}

/// The runs the tool may join that the view shows.
fn seen_runs(state: &AppState, frame: &Frame, run: Run) -> Vec<Seen> {
    let (category, system) = network(state, run);
    let project = state.project();
    state
        .session
        .geom()
        .segments
        .values()
        .filter(|g| {
            project.element(g.element).is_some_and(|e| {
                e.category() == category
                    && (category != Category::Pipe || e.as_segment().and_then(|s| s.system()) == system)
            })
        })
        .filter(|g| visible(frame, g.a) || visible(frame, g.b))
        .map(|g| Seen { a2: frame.project(g.a), b2: frame.project(g.b), a: g.a, b: g.b })
        .collect()
}

/// The point of a seen run under `raw` (an end first, else the nearest
/// point of a center line) in the drawing and in the model.
fn on_run(runs: &[Seen], raw: DVec2, reach: f64) -> Option<(DVec2, DVec3)> {
    let closest = |list: Vec<(DVec2, DVec3)>| {
        list.into_iter()
            .filter(|(p, _)| p.distance(raw) <= reach)
            .min_by(|x, y| x.0.distance(raw).total_cmp(&y.0.distance(raw)))
    };
    let ends = runs.iter().flat_map(|r| [(r.a2, r.a), (r.b2, r.b)]).collect();
    closest(ends).or_else(|| {
        let lines = runs
            .iter()
            .map(|r| {
                let (p, t) = cc_geom::polygon::closest_on_segment(r.a2, r.b2, raw);
                (p, r.a.lerp(r.b, t))
            })
            .collect();
        closest(lines)
    })
}

/// The level a run whose lower end is at `z` belongs to: the highest level at
/// or below it, else the lowest.
pub fn level_under(project: &Project, z: f64) -> Option<LevelId> {
    let mut levels: Vec<(f64, LevelId)> =
        project.levels.iter().map(|l| (project.level_elevation(l.id), l.id)).collect();
    levels.sort_by(|a, b| a.0.total_cmp(&b.0));
    levels.iter().rev().find(|(e, _)| *e <= z + 1e-6).or(levels.first()).map(|x| x.1)
}

/// The Pipe, Conduit and Cable Tray tools in a section or elevation.
#[allow(clippy::too_many_arguments)]
pub fn tool(
    ui: &Ui,
    resp: &Response,
    painter: &Painter,
    state: &mut AppState,
    view: &View,
    type_id: ElementTypeId,
    run: Run,
    xf: &Xf,
    drawing: &Drawing2D,
) {
    let Some(frame) = cc_views::view_frame(view).filter(|_| hosts(view)) else { return };
    // Conductors follow the roofs, which plans place them on.
    if run == Run::Conductor {
        if resp.clicked() {
            state.message = t!("msg-plan-only");
        }
        return;
    }
    let Some(raw) = resp.hover_pos().map(|p| xf.to_world(p)) else { return };
    let runs = seen_runs(state, &frame, run);
    let hit = on_run(&runs, raw, REACH_PX / xf.scale);
    let anchor = state.interaction.anchor;
    let s = match hit {
        Some((p, _)) => Snap::plain(p, true),
        None => {
            let shift = ui.input(|i| i.modifiers.shift);
            let ends: Vec<DVec2> = drawing.snap_lines().iter().flat_map(|(a, b)| [*a, *b]).collect();
            snap_at(state, None, raw, anchor, &ends, xf.scale, shift)
        }
    };
    draw_snap(painter, xf, s);
    let project = state.project();
    let ty = project.element_type(type_id);
    let diameter = ty.and_then(|t| t.diameter().or(t.tray_section().map(|s| s.0))).unwrap_or(0.025);
    let system = if run == Run::Pipe { state.mep.system } else { state.mep.circuit };
    let color = system.and_then(|id| project.system(id)).map_or(PREVIEW, |sys| {
        let [r, g, b] = sys.color();
        Color32::from_rgb(r, g, b)
    });
    if let Some(a) = anchor {
        let half = (diameter / 2.0).max(1.0 / xf.scale);
        let n = (s.point - a).normalize_or_zero().perp() * half;
        let pts = vec![xf.to_screen(a - n), xf.to_screen(s.point - n), xf.to_screen(s.point + n), xf.to_screen(a + n)];
        painter.add(egui::Shape::closed_line(pts, Stroke::new(1.5, color)));
        temp_dimension(painter, xf, a, s.point, None, None, half);
    }
    if !resp.clicked() {
        return;
    }
    let target = hit.map(|h| h.1);
    let (Some(a), Some(depth)) = (anchor, state.mep.depth) else {
        // The chain starts on a run seen here, else on a section's cut plane.
        let Some(start) = target.or_else(|| frame.near.map(|near| frame.unproject(s.point, near))) else {
            state.message = t!("msg-run-start-on-run");
            return;
        };
        state.interaction.anchor = Some(s.point);
        state.mep.depth = Some(frame.depth(start));
        return;
    };
    let start = frame.unproject(a, depth);
    let end = target.unwrap_or_else(|| frame.unproject(s.point, depth));
    if start.distance(end) < MIN_RUN {
        return;
    }
    if let Some(reached) = create(state, run, type_id, start, end) {
        state.interaction.anchor = Some(frame.project(reached));
        state.mep.depth = Some(frame.depth(reached));
    }
}

/// Create a run from `start` to `end` (model points) on the level under its
/// lower end; a level run of a drain falls by the tool's slope. Returns where
/// the run ends.
fn create(state: &mut AppState, run: Run, type_id: ElementTypeId, start: DVec3, end: DVec3) -> Option<DVec3> {
    let project = state.project();
    let level = level_under(project, start.z.min(end.z))?;
    let base = project.level_elevation(level);
    let vertical = start.truncate().distance(end.truncate()) < 1e-3;
    let level_run = !vertical && (end.z - start.z).abs() < 1e-3;
    let offset = Some(start.z - base);
    let end_offset = (!level_run).then_some(end.z - base);
    let plan_end = (!vertical).then_some(end.truncate().to_array());
    let level_ref = Some(Ref::Id(level.0));
    let type_ref = Some(Ref::Id(type_id.0));
    let circuits: Vec<Ref> = state.mep.circuit.map(|id| Ref::Id(id.0)).into_iter().collect();
    let out = match run {
        Run::Pipe => {
            let command = cc_engine::commands::mep::PipeCreate {
                start: start.truncate().to_array(),
                end: plan_end,
                level: level_ref,
                type_ref,
                system: state.mep.system.map(|id| Ref::Id(id.0)),
                offset,
                end_offset,
                slope: state.mep.slope,
            };
            crate::actions::run_command(state, command)
        }
        Run::Conduit => {
            let command = cc_engine::commands::mep::ConduitCreate {
                start: start.truncate().to_array(),
                end: plan_end,
                level: level_ref,
                type_ref,
                circuits,
                offset,
                end_offset,
            };
            crate::actions::run_command(state, command)
        }
        Run::Tray => {
            let command = cc_engine::commands::cable_tray::CableTrayCreate {
                start: start.truncate().to_array(),
                end: plan_end,
                level: level_ref,
                type_ref,
                circuits,
                offset,
                end_offset,
            };
            crate::actions::run_command(state, command)
        }
        Run::Conductor => None,
    }?;
    let id = ElementId(out["id"].as_u64()?);
    state.selection = BTreeSet::from([id]);
    if run == Run::Pipe {
        crate::pipe_catalog_dialog::report_fittings(state, id);
    }
    let z = out["params"]["end_offset"].as_f64().map_or(end.z, |e| base + e);
    Some(end.truncate().extend(z))
}
