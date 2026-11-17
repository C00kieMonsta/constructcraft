//! *Edit Profile* of a wall (issue #306): the outline of a straight wall's
//! face in its own elevation frame (`u` meters along the wall from its start,
//! at the left; `z` meters above its base), drawn in the profile sketch's
//! canvas (see [`crate::profile_sketch`]) with the sketch's own tools, lines
//! and arcs. The wall's ends, base and nominal top and the levels near it are
//! reference lines the sketch snaps to; the doors, windows and niches the
//! wall hosts are drawn faintly, as the outline must hold them. *Finish* runs
//! `sketch_set` on the wall (one undo step), and warns when hosted elements
//! no longer fit; *Reset Profile* runs `wall_profile_remove`.

use crate::paint::Xf;
use crate::sketch_mode::{Draw, Session, Target};
use crate::state::AppState;
use crate::t;
use crate::table::{readonly_row, section};
use crate::tools::{Snap, Tool};
use cc_core::sketch::{Sketch, SketchCurve};
use cc_core::wall_profile::{ProfileError, WallProfile};
use cc_core::{Element, ElementId, Project, Wall};
use cc_engine::commands::sketch::{CurveArg, SketchGet, SketchSet};
use egui::{Color32, Painter, Stroke, Ui};
use glam::DVec2;
use std::collections::BTreeSet;

/// Text on the paper, which stays white in both themes.
const INK: Color32 = Color32::from_gray(40);
/// Levels this far (meters) below or above the wall and its profile are shown.
const LEVEL_MARGIN: f64 = 10.0;
/// Points snap to this grid (meters) when nothing nearer catches them.
const GRID: f64 = 0.001;

/// What the profile sketch shows around the curves, in the wall's frame.
#[derive(Debug, Clone, PartialEq)]
pub struct Context {
    /// The wall's length (its end at `u = length`).
    pub length: f64,
    /// The wall's nominal height (its top at `z = height`).
    pub height: f64,
    /// The levels near the wall: name and height above the wall's base.
    pub levels: Vec<(String, f64)>,
    /// The doors, windows and niches it hosts, as `[u0, u1, z0, z1]`.
    pub openings: Vec<[f64; 4]>,
}

/// The wall selected alone, if any.
pub fn selected_wall(state: &AppState) -> Option<ElementId> {
    let mut sel = state.selection.iter();
    let (Some(id), None) = (sel.next(), sel.next()) else { return None };
    state.project().element(*id).filter(|e| e.as_wall().is_some()).map(|e| e.id)
}

/// Why wall `e` takes no profile (the engine's reasons), or `None` when it does.
pub fn why_not(project: &Project, e: &Element, w: &Wall) -> Option<String> {
    if w.is_curved() {
        Some(t!("why-wall-profile-arc"))
    } else if project.element_type(e.type_id).is_some_and(|t| t.curtain().is_some()) {
        Some(t!("why-wall-profile-curtain"))
    } else if w.attach_top.is_some() {
        Some(t!("why-wall-profile-attached"))
    } else if project.wall_framing_of(e.id).is_some() {
        Some(t!("why-wall-profile-framed"))
    } else {
        None
    }
}

/// Why *Edit Profile* cannot run now, or `None` when it can.
pub fn refusal(state: &AppState) -> Option<String> {
    let p = state.project();
    let Some(e) = selected_wall(state).and_then(|id| p.element(id)) else { return Some(t!("why-select-wall")) };
    let Some(w) = e.as_wall() else { return Some(t!("why-select-wall")) };
    why_not(p, e, w)
}

/// Why *Reset Profile* cannot run now, or `None` when it can.
pub fn reset_refusal(state: &AppState) -> Option<String> {
    let p = state.project();
    let Some(w) = selected_wall(state).and_then(|id| p.element(id)).and_then(Element::as_wall) else {
        return Some(t!("why-select-wall"));
    };
    w.profile.is_none().then(|| t!("why-wall-no-profile"))
}

/// Whether the open sketch is a wall's profile.
pub fn is_open(state: &AppState) -> bool {
    state.sketch.as_ref().is_some_and(|s| matches!(s.target, Target::WallProfile { .. }))
}

/// The curves `sketch_get` gives for the wall: its profile's, or its nominal rectangle.
fn initial_curves(state: &mut AppState, wall: ElementId) -> Option<Vec<SketchCurve>> {
    let out = state.session.run(SketchGet { id: wall.0 }).ok()?;
    let args: Vec<CurveArg> = serde_json::from_value(out.get("curves")?.clone()).ok()?;
    cc_engine::commands::sketch::sketch_of(&args).ok().map(|s| s.curves)
}

/// Open wall `wall`'s profile in a sketch; whether it opened.
pub fn open(state: &mut AppState, wall: ElementId) -> bool {
    let Some(curves) = initial_curves(state, wall) else { return false };
    state.set_tool(Tool::Select);
    let mut session = Session::new(Target::WallProfile { wall }, curves);
    session.draw = Draw::Select;
    state.sketch = Some(session);
    state.message = t!("hint-wall-profile-sketch");
    true
}

/// *Edit Profile*: the selected wall's.
pub fn edit_selected(state: &mut AppState) {
    if refusal(state).is_none()
        && let Some(id) = selected_wall(state)
    {
        open(state, id);
    }
}

/// *Reset Profile*: the selected wall's profile removed (one undo step).
pub fn reset_selected(state: &mut AppState) {
    if reset_refusal(state).is_none()
        && let Some(id) = selected_wall(state)
    {
        crate::actions::run_command(state, cc_engine::commands::wall_profile::WallProfileRemove { wall: id.0 });
    }
}

/// The profile the curves make for the wall, or why not.
fn checked(state: &AppState, wall: ElementId, curves: &[SketchCurve]) -> Option<Result<WallProfile, ProfileError>> {
    let w = state.project().element(wall).and_then(Element::as_wall)?;
    Some(WallProfile::from_sketch(Sketch { curves: curves.to_vec() }, w.length()))
}

/// *Finish*: the wall takes the sketch as its profile with `sketch_set`
/// (nothing when the curves did not change). A sketch that makes no
/// profile stays open with the reason, marked in the canvas; hosted
/// elements the new profile leaves out are counted in the status bar.
pub fn finish(state: &mut AppState) {
    let Some(Target::WallProfile { wall }) = state.sketch.as_ref().map(|s| s.target) else { return };
    let curves = state.sketch.as_ref().map(|s| s.curves.clone()).unwrap_or_default();
    match checked(state, wall, &curves) {
        // The wall is gone: nothing to edit.
        None => {
            crate::sketch_mode::cancel(state);
            return;
        }
        Some(Err(e)) => {
            state.message = crate::i18n::tr(e.key());
            return;
        }
        Some(Ok(_)) => {}
    }
    if initial_curves(state, wall).as_deref() != Some(&curves[..]) {
        let command = SketchSet { id: wall.0, curves: curves.iter().map(CurveArg::of).collect(), points: None };
        if crate::actions::run_command(state, command).is_none() {
            return;
        }
    }
    state.sketch = None;
    state.set_tool(Tool::Select);
    state.selection = BTreeSet::from([wall]);
    let left_out = misfits(state.project(), wall);
    if left_out > 0 {
        state.message = t!("wall-profile-openings-out", "count" => left_out);
    }
}

/// The rectangle `[u0, u1, z0, z1]` of hosted element `h` in its wall's frame.
fn opening_rect(project: &Project, h: &Element) -> Option<[f64; 4]> {
    let host = h.as_hosted()?;
    let (width, height) = project.element_type(h.type_id)?.opening_size()?;
    let (u, z) = (host.offset, host.sill_height);
    Some([u - width * 0.5, u + width * 0.5, z, z + height])
}

/// How many doors, windows and niches wall `wall` hosts lie outside its
/// profile (they cut nothing: the geometry leaves out what the profile does
/// not hold, see `cc_geom::build`).
pub fn misfits(project: &Project, wall: ElementId) -> usize {
    let Some(profile) = project.element(wall).and_then(Element::as_wall).and_then(|w| w.profile.as_ref()) else {
        return 0;
    };
    project
        .hosted_by(wall)
        .filter_map(|h| opening_rect(project, h))
        .filter(|[u0, u1, z0, z1]| !profile.holds(*u0, *u1, *z0, *z1))
        .count()
}

/// What the canvas shows around the sketch of wall `wall`'s profile.
pub fn context(project: &Project, wall: ElementId, curves: &[SketchCurve]) -> Option<Context> {
    let e = project.element(wall)?;
    let w = e.as_wall()?;
    let (length, height) = (w.length(), project.wall_height(e, w));
    let base = project.wall_base_z(e, w);
    let (lo, hi) =
        curves.iter().flat_map(|c| c.points()).fold((0.0_f64, height), |(lo, hi), p| (lo.min(p.y), hi.max(p.y)));
    let levels = project
        .levels
        .iter()
        .map(|l| (l.name.clone(), l.elevation - base))
        .filter(|(_, z)| z.is_finite() && *z >= lo - LEVEL_MARGIN && *z <= hi + LEVEL_MARGIN)
        .collect();
    let openings = project.hosted_by(wall).filter_map(|h| opening_rect(project, h)).collect();
    Some(Context { length, height, levels, openings })
}

impl Context {
    /// The points the canvas fits: the wall's nominal rectangle.
    pub fn extent(&self) -> [DVec2; 2] {
        [DVec2::ZERO, DVec2::new(self.length, self.height)]
    }

    /// Reference lines across `u` (the wall's ends) and along it (its base,
    /// its top and the levels).
    fn lines(&self) -> (Vec<f64>, Vec<f64>) {
        let across = vec![0.0, self.length];
        let along = [0.0, self.height].into_iter().chain(self.levels.iter().map(|(_, z)| *z)).collect();
        (across, along)
    }
}

/// Snap `raw`: to the sketch's points, the wall's corners and its hosted
/// elements' corners, then to the reference lines (where two cross, or the
/// nearest), then the millimeter grid; Shift keeps the line from the last
/// pick horizontal or vertical.
pub fn snap(s: &Session, c: &Context, raw: DVec2, tol: f64, shift: bool) -> Snap {
    let corners = c.openings.iter().flat_map(|[u0, u1, z0, z1]| {
        [DVec2::new(*u0, *z0), DVec2::new(*u1, *z0), DVec2::new(*u1, *z1), DVec2::new(*u0, *z1)]
    });
    let [a, b] = c.extent();
    let near = s
        .snap_points()
        .into_iter()
        .chain([a, DVec2::new(b.x, a.y), b, DVec2::new(a.x, b.y)])
        .chain(corners)
        .filter(|q| q.distance(raw) <= tol)
        .min_by(|p, q| p.distance(raw).total_cmp(&q.distance(raw)));
    if let Some(q) = near {
        return Snap::plain(q, true);
    }
    let mut p = raw;
    if shift && let Some(a) = s.picks.last() {
        let d = raw - *a;
        if d.x.abs() >= d.y.abs() {
            p.y = a.y;
        } else {
            p.x = a.x;
        }
    }
    let (across, along) = c.lines();
    let nearest = |list: &[f64], v: f64| {
        list.iter().copied().filter(|x| (x - v).abs() <= tol).min_by(|x, y| (x - v).abs().total_cmp(&(y - v).abs()))
    };
    let (u, z) = (nearest(&across, p.x), nearest(&along, p.y));
    let grid = |v: f64| (v / GRID).round() * GRID;
    match (u, z) {
        (Some(u), Some(z)) => Snap::plain(DVec2::new(u, z), true),
        (Some(u), None) => Snap::nearest(DVec2::new(u, grid(p.y))),
        (None, Some(z)) => Snap::nearest(DVec2::new(grid(p.x), z)),
        (None, None) => Snap::plain(DVec2::new(grid(p.x), grid(p.y)), false),
    }
}

/// Dashes along a screen segment.
fn dashed(painter: &Painter, a: egui::Pos2, b: egui::Pos2, stroke: Stroke, on: f32, off: f32) {
    painter.extend(egui::Shape::dashed_line(&[a, b], stroke, on, off));
}

/// The wall's reference lines, the levels near it and its hosted elements.
pub fn paint_context(painter: &Painter, rect: egui::Rect, xf: &Xf, c: &Context) {
    let font = egui::FontId::proportional(11.0);
    // Hosted elements: faint, under everything.
    let faint = crate::paint::stroke_for(cc_views::LineStyle::Faint, false);
    for [u0, u1, z0, z1] in &c.openings {
        let r = egui::Rect::from_two_pos(xf.to_screen(DVec2::new(*u0, *z0)), xf.to_screen(DVec2::new(*u1, *z1)));
        painter.rect_filled(r, 0.0, Color32::from_black_alpha(14));
        painter.rect_stroke(r, 0.0, faint, egui::StrokeKind::Middle);
    }
    // Levels: dashed and named at the left.
    let level = Stroke::new(1.0, Color32::from_gray(150));
    for (name, z) in &c.levels {
        let y = xf.to_screen(DVec2::new(0.0, *z)).y;
        if !(rect.top()..=rect.bottom()).contains(&y) {
            continue;
        }
        dashed(painter, egui::pos2(rect.left(), y), egui::pos2(rect.right(), y), level, 8.0, 5.0);
        let at = egui::pos2(rect.left() + 8.0, y - 2.0);
        painter.text(at, egui::Align2::LEFT_BOTTOM, name, font.clone(), INK.gamma_multiply(0.7));
    }
    // The wall's ends, base and top, as reference planes are drawn.
    let reference = crate::paint::stroke_for(cc_views::LineStyle::Center, false);
    let color = reference.color;
    let ends = [(0.0, t!("wall-profile-ref-start")), (c.length, t!("wall-profile-ref-end"))];
    for (u, label) in ends {
        let x = xf.to_screen(DVec2::new(u, 0.0)).x;
        dashed(painter, egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom()), reference, 12.0, 4.0);
        painter.text(egui::pos2(x + 4.0, rect.top() + 26.0), egui::Align2::LEFT_TOP, label, font.clone(), color);
    }
    let sides = [(0.0, t!("wall-profile-ref-base")), (c.height, t!("wall-profile-ref-top"))];
    for (z, label) in sides {
        let y = xf.to_screen(DVec2::new(0.0, z)).y;
        dashed(painter, egui::pos2(rect.left(), y), egui::pos2(rect.right(), y), reference, 12.0, 4.0);
        painter.text(egui::pos2(rect.right() - 8.0, y - 2.0), egui::Align2::RIGHT_BOTTOM, label, font.clone(), color);
    }
}

/// Where the sketch makes no profile for the wall (past its ends, or a
/// vertical line crossing it more than twice), marked as the sketch marks
/// where it does not close; the sketch's own marks are left to it.
pub fn paint_error(painter: &Painter, xf: &Xf, s: &Session, c: &Context) {
    if !s.picks.is_empty() || s.open_path() {
        return;
    }
    let sketch = Sketch { curves: s.curves.clone() };
    if sketch.loops().is_err() {
        return;
    }
    if let Err(e) = WallProfile::from_sketch(sketch, c.length)
        && let Some(p) = e.point()
    {
        let danger = crate::theme::PALETTE.danger;
        painter.circle_stroke(xf.to_screen(p), 6.0, Stroke::new(1.5, danger));
    }
}

/// The *Profile* section of a straight wall's properties: whether it has a
/// profile, and the buttons that edit and remove it.
pub fn palette_rows(ui: &mut Ui, state: &mut AppState, el: &Element, w: &Wall) {
    let curtain = state.project().element_type(el.type_id).is_some_and(|t| t.curtain().is_some());
    if w.is_curved() || curtain {
        return;
    }
    section(ui, "wall-profile", t!("section-wall-profile"), |ui| {
        let kind = if w.profile.is_some() { t!("wall-profile-custom") } else { t!("wall-profile-rectangular") };
        readonly_row(ui, &t!("prop-wall-profile"), kind);
        use crate::commands::UiCommand;
        let pressed = crate::table::button_row(ui, |ui| {
            let mut pressed = None;
            for command in [UiCommand::EditWallProfile, UiCommand::ResetWallProfile] {
                let why = command.unavailable(state);
                let button = egui::Button::new(command.label(state));
                let mut r = ui.add_enabled(why.is_none(), button).on_hover_text(command.description());
                if let Some(why) = why {
                    r = r.on_disabled_hover_text(why);
                }
                if r.clicked() {
                    pressed = Some(command);
                }
            }
            pressed
        });
        if let Some(command) = pressed {
            crate::commands::run(state, &ui.ctx().clone(), command);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> Context {
        Context { length: 4.0, height: 3.0, levels: vec![("L2".into(), 3.2)], openings: vec![[1.0, 2.0, 0.0, 2.1]] }
    }

    #[test]
    fn points_snap_to_the_walls_lines_levels_and_openings() {
        let s = Session::new(Target::WallProfile { wall: ElementId(1) }, Vec::new());
        let c = context();
        // The wall's corner.
        assert_eq!(snap(&s, &c, DVec2::new(4.003, 2.998), 0.01, false).point, DVec2::new(4.0, 3.0));
        // A door's head corner.
        assert_eq!(snap(&s, &c, DVec2::new(2.004, 2.104), 0.01, false).point, DVec2::new(2.0, 2.1));
        // The level line, across the wall.
        let p = snap(&s, &c, DVec2::new(2.5004, 3.205), 0.01, false).point;
        assert!(p.distance(DVec2::new(2.5, 3.2)) < 1e-12, "{p}");
        // The wall's end, where the level crosses it.
        assert_eq!(snap(&s, &c, DVec2::new(3.995, 3.195), 0.01, false).point, DVec2::new(4.0, 3.2));
        // Nothing near: the millimeter grid.
        let p = snap(&s, &c, DVec2::new(1.23456, 1.65432), 0.01, false).point;
        assert!(p.distance(DVec2::new(1.235, 1.654)) < 1e-12, "{p}");
    }
}
