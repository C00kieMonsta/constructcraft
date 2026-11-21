//! Levels in elevations and sections on the canvas (issue #453): a level's
//! head is dragged up or down off its line with an elbow, so the heads of
//! levels close together do not overlap (dropped near the line it goes back
//! onto it), and a right click on a level's line or head opens its menu:
//! hide it in the view, choose the ends that carry a head, or put it back
//! as by default. Every change is one `level_head_set`.

use crate::paint::{PREVIEW, Xf};
use crate::state::AppState;
use crate::t;
use cc_core::level_head::LevelHeads;
use cc_core::{LevelId, ViewId};
use cc_engine::Ref;
use cc_engine::commands::level_heads::LevelHeadSet;
use cc_views::{Drawing2D, LevelHeadHandle};
use egui::{Painter, Response, Stroke};
use glam::DVec2;

/// Screen points within which a dragged head goes back onto its line.
const SNAP_PX: f64 = 6.0;
/// Screen points within which a right click finds a level's line.
const PICK_PX: f64 = 5.0;

fn set(view: ViewId, level: LevelId) -> LevelHeadSet {
    LevelHeadSet { view: Ref::Id(view.0), levels: vec![Ref::Id(level.0)], ..Default::default() }
}

/// The level head under `w` (the last drawn on top), with its level.
pub fn head_at(drawing: &Drawing2D, w: DVec2) -> Option<(LevelId, LevelHeadHandle)> {
    drawing.levels.iter().rev().find_map(|l| l.heads.iter().find(|h| h.contains(w)).map(|h| (l.level, *h)))
}

/// The elbow of a head dragged from `start` to `end`: back on the line
/// when dropped within a few points of it.
fn elbow(head: &LevelHeadHandle, start: DVec2, end: DVec2, xf: &Xf) -> f64 {
    let e = head.elbow + (end.y - start.y);
    if e.abs() * xf.scale <= SNAP_PX { 0.0 } else { e }
}

/// A head being dragged: its preview while `raw` is under the pointer, and
/// its new elbow once the drag `stopped` at `end`.
#[allow(clippy::too_many_arguments)]
pub fn drag(
    painter: &Painter,
    xf: &Xf,
    state: &mut AppState,
    (view, level, head): (ViewId, LevelId, LevelHeadHandle),
    start: DVec2,
    raw: Option<DVec2>,
    end: Option<DVec2>,
    stopped: bool,
) {
    if let Some(raw) = raw {
        let up = DVec2::new(0.0, elbow(&head, start, raw, xf) - head.elbow);
        let c = head.center + up;
        let rect = egui::Rect::from_two_pos(xf.to_screen(c - head.half), xf.to_screen(c + head.half));
        painter.rect_stroke(rect, 2.0, Stroke::new(1.5, PREVIEW), egui::StrokeKind::Middle);
        painter.line_segment([xf.to_screen(head.tip), xf.to_screen(head.point + up)], Stroke::new(1.0, PREVIEW));
    }
    if !stopped {
        return;
    }
    state.interaction.drag = None;
    if let Some(p) = end
        && (p.y - start.y).abs() > 1e-6
    {
        let e = Some(elbow(&head, start, p, xf));
        let command = if head.start {
            LevelHeadSet { start_elbow: e, ..set(view, level) }
        } else {
            LevelHeadSet { elbow: e, ..set(view, level) }
        };
        crate::actions::run_command(state, command);
    }
}

/// The level a right click at `w` falls on: on its head or near its line.
pub fn picked(drawing: &Drawing2D, w: DVec2, xf: &Xf) -> Option<LevelId> {
    drawing.level_at(w, PICK_PX / xf.scale.max(1e-9)).map(|(level, _)| level)
}

/// The context menu of the level a right click picked
/// (`Interaction::level_menu`), on the canvas's response.
pub fn menu(resp: &Response, state: &mut AppState) {
    let Some((view, level)) = state.interaction.level_menu else { return };
    let current = state.project().view(view).and_then(|v| v.level_heads.get(&level).copied()).unwrap_or_default();
    let mut command = None;
    let open = resp.context_menu(|ui| {
        if ui.button(t!("menu-hide-level")).clicked() {
            command = Some(LevelHeadSet { hidden: Some(true), ..set(view, level) });
            ui.close();
        }
        ui.separator();
        ui.label(t!("level-heads"));
        for heads in LevelHeads::ALL {
            if ui.radio(current.heads == heads, crate::i18n::tr(heads.key())).clicked() {
                command = Some(LevelHeadSet { heads: Some(heads.code().to_string()), ..set(view, level) });
                ui.close();
            }
        }
        ui.separator();
        if ui.add_enabled(!current.is_default(), egui::Button::new(t!("menu-reset-level"))).clicked() {
            command = Some(LevelHeadSet { reset: true, ..set(view, level) });
            ui.close();
        }
    });
    if open.is_none() && !resp.secondary_clicked() {
        state.interaction.level_menu = None;
    }
    if let Some(c) = command {
        crate::actions::run_command(state, c);
    }
}

/// The levels hidden in `view`, for the view's graphics dialog to show again.
pub fn hidden_levels(state: &AppState, view: ViewId) -> Vec<LevelId> {
    let Some(v) = state.project().view(view) else { return Vec::new() };
    v.level_heads.iter().filter(|(_, h)| h.hidden).map(|(level, _)| *level).collect()
}

/// Show the levels hidden in `view` again.
pub fn show_hidden(state: &mut AppState, view: ViewId) {
    let levels: Vec<Ref> = hidden_levels(state, view).into_iter().map(|l| Ref::Id(l.0)).collect();
    if !levels.is_empty() {
        let command = LevelHeadSet { view: Ref::Id(view.0), levels, hidden: Some(false), ..Default::default() };
        crate::actions::run_command(state, command);
    }
}
