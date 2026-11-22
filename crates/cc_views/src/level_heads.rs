//! The levels of an elevation or a section: a line across the drawing per
//! level with a head (a triangle down to the line, the level's name above
//! and its elevation below) at the ends the view asks for. The view's
//! graphics hide them by category ([`Category::Level`]) or one by one
//! ([`cc_core::level_head::LevelHead::hidden`]) and give them the category's
//! projection lines; [`Drawing2D::levels`] keeps where each was drawn, so
//! the app picks and moves them.

use crate::projected::Frame;
use crate::{Anchor, Drawing2D, LineStyle, Pen, format};
use crate::{LevelHandle, LevelHeadHandle};
use cc_core::{Category, Project, View};
use glam::{DVec2, DVec3};

/// Height of a level's name on paper, millimeters.
const NAME_MM: f64 = 2.5;
/// Height of a level's elevation on paper, millimeters.
const ELEVATION_MM: f64 = 2.0;
/// Half the width and the height of the head's triangle, paper millimeters.
const SYMBOL_MM: (f64, f64) = (1.5, 2.0);
/// How far the line runs past the drawing on each side, paper millimeters.
const OVERRUN_MM: f64 = 15.0;
/// The run of an elbow's leader along the line, paper millimeters.
const ELBOW_RUN_MM: f64 = 6.0;

fn mm(scale: u32) -> f64 {
    f64::from(scale.max(1)) / 1000.0
}

/// Draw the levels of `view` (an elevation or a section seen through
/// `frame`) across what is drawn so far; before anything is modelled they
/// span the grids, or a few meters from the origin.
pub fn draw(d: &mut Drawing2D, project: &Project, view: &View, frame: &Frame) {
    let category = view.graphics.category(Category::Level);
    if !category.visible {
        return;
    }
    let mm = mm(view.scale);
    let grid_xs: Vec<f64> = project
        .elements
        .values()
        .filter_map(|e| match &e.kind {
            cc_core::ElementKind::Grid(g) => Some([g.start, g.end]),
            _ => None,
        })
        .flatten()
        .map(|p| frame.project(DVec3::new(p.x, p.y, 0.0)).x)
        .filter(|x| x.is_finite())
        .collect();
    let span = match d.bounds() {
        Some((lo, hi)) => (lo.x, hi.x),
        None if !grid_xs.is_empty() => grid_xs.iter().fold((f64::MAX, f64::MIN), |(a, b), x| (a.min(*x), b.max(*x))),
        None => (0.0, 10.0),
    };
    let (x0, x1) = (span.0 - OVERRUN_MM * mm, span.1 + OVERRUN_MM * mm);
    if !(x0.is_finite() && x1.is_finite()) {
        return;
    }
    let o = &category.overrides;
    let halftone = o.halftone == Some(true);
    let lines = &o.projection_lines;
    let pen = (!lines.is_empty() || halftone).then_some(Pen {
        color: lines.color,
        weight: lines.weight,
        pattern: lines.pattern,
        halftone,
        layer: None,
    });
    // Texts take the lines' color, faint with them.
    let ink = match (lines.color, halftone) {
        (Some(c), true) => Some(crate::drawing::halftone(c)),
        (None, true) => Some(crate::drawing::halftone([0, 0, 0])),
        (c, false) => c,
    };
    for level in &project.levels {
        let head = view.level_heads.get(&level.id).copied().unwrap_or_default();
        if head.hidden {
            continue;
        }
        let y = frame.project(DVec3::new(0.0, 0.0, level.elevation)).y;
        if !y.is_finite() {
            continue;
        }
        let (a, b) = (DVec2::new(x0, y), DVec2::new(x1, y));
        d.line_pen(a, b, LineStyle::Annotation, pen, None);
        let mut handle = LevelHandle { level: level.id, a, b, heads: Vec::new() };
        let ends = [
            (true, head.heads.at_start(), a, -1.0, head.start_elbow),
            (false, head.heads.at_end(), b, 1.0, head.elbow),
        ];
        for (start, shown, tip, out, elbow) in ends {
            if !shown {
                continue;
            }
            let texts = (level.name.as_str(), format::elevation(level.elevation));
            handle.heads.push(draw_head(d, (tip, out, elbow), texts, (pen, ink), (start, mm)));
        }
        d.levels.push(handle);
    }
}

/// One head at the end `tip` of a level's line, outward along `out` (±1 in
/// x), moved `elbow` meters up from the line with a leader back to it.
fn draw_head(
    d: &mut Drawing2D,
    (tip, out, elbow): (DVec2, f64, f64),
    (name, elevation): (&str, String),
    (pen, ink): (Option<Pen>, Option<[u8; 3]>),
    (start, mm): (bool, f64),
) -> LevelHeadHandle {
    let elbow = Some(elbow).filter(|e| e.is_finite()).unwrap_or(0.0);
    let mut point = tip;
    if elbow != 0.0 {
        // Along the line, across to the head's height, along again.
        let run = ELBOW_RUN_MM * mm / 3.0;
        let pts = [
            tip,
            tip + DVec2::new(out * run, 0.0),
            tip + DVec2::new(out * 2.0 * run, elbow),
            tip + DVec2::new(out * 3.0 * run, elbow),
        ];
        d.polyline_pen(&pts, false, LineStyle::Annotation, pen, None);
        point = pts[3];
    }
    let (w, h) = (SYMBOL_MM.0 * mm, SYMBOL_MM.1 * mm);
    let triangle = [point, point + DVec2::new(w, h), point + DVec2::new(-w, h)];
    d.polyline_pen(&triangle, true, LineStyle::Annotation, pen, None);
    let anchor = if start { Anchor::RightBottom } else { Anchor::LeftBottom };
    let gap = out * (w + 1.5 * mm);
    let name_at = point + DVec2::new(gap, 1.2 * mm);
    let elevation_at = point + DVec2::new(gap, -1.8 * mm - ELEVATION_MM * mm);
    d.colored_text(name_at, name, NAME_MM, (anchor, 0.0), None, ink);
    d.colored_text(elevation_at, elevation.clone(), ELEVATION_MM, (anchor, 0.0), None, ink);
    // The box the head takes on paper: symbol and texts.
    let chars = name.chars().count().max(elevation.chars().count()) as f64;
    let width = w + 1.5 * mm + chars * NAME_MM * 0.6 * mm;
    let (lo_y, hi_y) = (elevation_at.y - 0.5 * mm, name_at.y + NAME_MM * mm);
    let far = point.x + out * width;
    let (lo_x, hi_x) = ((point.x - out * w).min(far), (point.x - out * w).max(far));
    let lo = DVec2::new(lo_x, lo_y);
    let hi = DVec2::new(hi_x, hi_y);
    LevelHeadHandle { start, tip, point, center: (lo + hi) / 2.0, half: (hi - lo) / 2.0, elbow }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Prim;
    use cc_core::graphics::{CategoryGraphics, GraphicOverride, LineGraphics};
    use cc_core::level_head::{LevelHead, LevelHeads};
    use cc_core::{Direction, ViewId, ViewKind};
    use cc_geom::ModelGeometry;

    fn south() -> View {
        View::new(ViewId(900), "South", 100, ViewKind::Elevation { direction: Direction::South })
    }

    fn drawn(project: &Project, view: &View) -> Drawing2D {
        crate::generate(project, &ModelGeometry::build(project), view).unwrap()
    }

    fn texts(d: &Drawing2D) -> Vec<&str> {
        d.prims
            .iter()
            .filter_map(|p| match p {
                Prim::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn an_empty_elevation_shows_the_levels() {
        let project = Project::from_template();
        let d = drawn(&project, &south());
        for level in &project.levels {
            assert!(texts(&d).contains(&level.name.as_str()), "{}", level.name);
        }
        assert_eq!(d.levels.len(), project.levels.len());
        assert!(d.levels.iter().all(|l| l.heads.len() == 1 && !l.heads[0].start), "one head, at the end");
    }

    /// Switching the category off in the view's graphics hides every level.
    #[test]
    fn the_category_hides_the_levels() {
        let project = Project::from_template();
        let mut view = south();
        view.graphics.set_category(Category::Level, CategoryGraphics { visible: false, ..Default::default() });
        let d = drawn(&project, &view);
        assert!(d.levels.is_empty());
        assert!(project.levels.iter().all(|l| !texts(&d).contains(&l.name.as_str())));
    }

    /// A level hidden in the view goes, line and head; the others stay.
    #[test]
    fn one_level_hides_in_one_view() {
        let project = Project::from_template();
        let [first, rest @ ..] = project.levels.as_slice() else { panic!("levels") };
        let mut view = south();
        view.level_heads.insert(first.id, LevelHead { hidden: true, ..Default::default() });
        let d = drawn(&project, &view);
        assert!(!texts(&d).contains(&first.name.as_str()));
        assert!(rest.iter().all(|l| texts(&d).contains(&l.name.as_str())));
        assert!(d.levels.iter().all(|l| l.level != first.id));
        // Other views still show it.
        assert!(texts(&drawn(&project, &south())).contains(&first.name.as_str()));
    }

    /// Heads at both ends, at the start only or none; an elbow moves the
    /// head up with a leader, and the handles find level and head.
    #[test]
    fn heads_go_where_the_view_says() {
        let project = Project::from_template();
        let level = project.levels[0].id;
        let mut view = south();
        let mut with = |head: LevelHead| {
            view.level_heads.insert(level, head);
            let d = drawn(&project, &view);
            d.levels.iter().find(|l| l.level == level).cloned().unwrap()
        };
        let both = with(LevelHead { heads: LevelHeads::Both, ..Default::default() });
        assert_eq!(both.heads.iter().map(|h| h.start).collect::<Vec<_>>(), [true, false]);
        assert!(with(LevelHead { heads: LevelHeads::None, ..Default::default() }).heads.is_empty());
        let start = with(LevelHead { heads: LevelHeads::Start, ..Default::default() });
        assert!(start.heads[0].start && start.heads[0].tip == start.a);
        let moved = with(LevelHead { elbow: 0.8, ..Default::default() });
        let h = moved.heads[0];
        assert!((h.point.y - h.tip.y - 0.8).abs() < 1e-9 && h.point.x > h.tip.x, "{h:?}");
        let d = drawn(&project, &view);
        assert_eq!(d.level_at(h.point + DVec2::new(0.1, 0.1), 0.05), Some((level, Some(false))));
        let mid = (moved.a + moved.b) / 2.0;
        assert_eq!(d.level_at(mid + DVec2::new(0.0, 0.01), 0.05), Some((level, None)));
    }

    /// The category's projection lines draw the levels' lines, and their
    /// color the texts.
    #[test]
    fn the_category_overrides_the_pen() {
        let project = Project::from_template();
        let mut view = south();
        let lines = LineGraphics { color: Some([200, 0, 0]), weight: Some(0.5), ..Default::default() };
        let overrides = GraphicOverride { projection_lines: lines, ..Default::default() };
        view.graphics
            .set_category(Category::Level, CategoryGraphics { visible: true, overrides, ..Default::default() });
        let d = drawn(&project, &view);
        let red_lines = d
            .prims
            .iter()
            .filter(|p| matches!(p, Prim::Line { pen: Some(pen), .. } if pen.color == Some([200, 0, 0])))
            .count();
        assert!(red_lines >= project.levels.len() * 4, "line and triangle of each level: {red_lines}");
        let name = project.levels[0].name.as_str();
        assert!(d.prims.iter().any(|p| matches!(p, Prim::Text { text, color: Some([200, 0, 0]), .. } if text == name)));
    }

    /// The head and its texts grow with the scale: the same on paper.
    #[test]
    fn heads_keep_their_size_on_paper() {
        let project = Project::from_template();
        let mut view = south();
        let size = |view: &View| drawn(&project, view).levels[0].heads[0].half;
        let at_100 = size(&view);
        view.scale = 50;
        let at_50 = size(&view);
        assert!((at_100 - at_50 * 2.0).length() < 1e-9, "{at_100} vs {at_50}");
    }
}
