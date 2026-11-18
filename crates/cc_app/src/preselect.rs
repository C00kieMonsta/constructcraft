//! Tab cycles the pre-selection in 2D views (issue #406). With the Select
//! tool, the elements under the pointer are candidates in pick order (what a
//! click takes first, then the others), plus the chain of walls joined to the
//! first wall among them; Tab and Shift+Tab step through them, the highlight
//! and the status bar show the current one, and a click selects it.
//!
//! The raw input hook takes Tab over the canvas ([`take_tab`]), before egui
//! moves the keyboard focus with it; the canvas applies the steps in
//! [`frame`] the same frame.

use crate::paint::Xf;
use crate::state::AppState;
use crate::t;
use cc_core::{ElementId, Project, ViewId};
use cc_views::Drawing2D;
use egui::{Event, Key, Pos2, Rect};

/// How far (screen points) the pointer may move from where Tab was first
/// pressed before the cycle starts over.
pub const ANCHOR_SLACK: f32 = 3.0;

/// What a click would select.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Candidate {
    One(ElementId),
    /// The walls joined end to end with a wall under the pointer.
    Chain(Vec<ElementId>),
}

impl Candidate {
    pub fn ids(&self) -> &[ElementId] {
        match self {
            Candidate::One(id) => std::slice::from_ref(id),
            Candidate::Chain(ids) => ids,
        }
    }
}

/// The candidates from the elements under the pointer in pick order
/// ([`Drawing2D::hits_at`]): each of them, and right after the first one
/// that `chain_of` gives a chain for (a wall), that chain when it has more
/// than one wall.
pub fn candidates(hits: &[ElementId], mut chain_of: impl FnMut(ElementId) -> Option<Vec<ElementId>>) -> Vec<Candidate> {
    let mut out = Vec::with_capacity(hits.len() + 1);
    let mut chained = false;
    for id in hits {
        out.push(Candidate::One(*id));
        if !chained && let Some(chain) = chain_of(*id) {
            chained = true;
            if chain.len() > 1 {
                out.push(Candidate::Chain(chain));
            }
        }
    }
    out
}

/// `index` moved `delta` places among `len` candidates, wrapping round.
pub fn step(index: usize, len: usize, delta: i64) -> usize {
    if len == 0 {
        return 0;
    }
    let len = len as i64;
    (index as i64 + delta).rem_euclid(len) as usize
}

/// A cycle started by Tab: the canvas, where the pointer was, the current
/// candidate and the document revision.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cycle {
    pub view: ViewId,
    pub anchor: Pos2,
    pub index: usize,
    pub revision: u64,
}

impl Cycle {
    /// Whether the cycle goes on in the canvas of `view` with the pointer
    /// at `pointer` (none when it is off the canvas): the same view, the
    /// pointer within [`ANCHOR_SLACK`] of the anchor and the document
    /// unchanged.
    pub fn holds(&self, view: ViewId, pointer: Option<Pos2>, revision: u64) -> bool {
        self.view == view
            && self.revision == revision
            && pointer.is_some_and(|p| p.distance(self.anchor) <= ANCHOR_SLACK)
    }
}

/// The interaction state of Tab cycling (in [`crate::tools::Interaction`],
/// so a new tool starts it over).
#[derive(Debug, Clone, Default)]
pub struct TabCycle {
    /// The canvas the Select tool pre-selects in, set each frame the pointer
    /// is over it: the raw input hook of the next frame takes Tab there.
    pub armed: Option<Rect>,
    /// Tab presses taken by the hook, not yet applied (Shift+Tab counts -1).
    pub steps: i64,
    pub cycle: Option<Cycle>,
    /// The chain of the last wall asked, with the revision and the view it was found in.
    chain: Option<(u64, ViewId, ElementId, Vec<ElementId>)>,
}

/// The raw input hook's part: when the Select tool pre-selected over a
/// canvas last frame (`armed`), the pointer is still over it and nothing
/// else wants the keys (`blocked`: a focused widget or a modal dialog),
/// Tab presses become `steps` and every Tab event is taken out of the
/// input, so egui does not move the focus. Returns whether it took them.
pub fn take_tab(armed: Option<Rect>, steps: &mut i64, events: &mut Vec<Event>, blocked: bool) -> bool {
    let Some(rect) = armed else { return false };
    if blocked {
        return false;
    }
    // Where this frame's events leave the pointer.
    let mut inside = true;
    for e in events.iter() {
        match e {
            Event::PointerMoved(p) => inside = rect.contains(*p),
            Event::PointerGone => inside = false,
            _ => {}
        }
    }
    if !inside {
        return false;
    }
    for e in events.iter() {
        if let Event::Key { key: Key::Tab, pressed: true, modifiers, .. } = e {
            *steps += if modifiers.shift { -1 } else { 1 };
        }
    }
    events.retain(|e| !matches!(e, Event::Key { key: Key::Tab, .. }));
    true
}

/// This frame's candidates in the canvas of `view` and the index of the
/// current one: the Tab steps applied, the cycle started over where the
/// rules say, and the canvas armed for the next frame's Tab while the
/// pointer is over it. While a cycle holds, the candidates are those under
/// its anchor.
pub fn frame(
    state: &mut AppState,
    view: ViewId,
    rect: Rect,
    pointer: Option<Pos2>,
    drawing: &Drawing2D,
    xf: &Xf,
    tol: f64,
) -> (Vec<Candidate>, usize) {
    let revision = state.session.doc.revision();
    let dragging = state.interaction.drag.is_some();
    let project = &state.session.doc.project;
    let tab = &mut state.interaction.tab;
    let steps = if pointer.is_some() { std::mem::take(&mut tab.steps) } else { 0 };
    // Another canvas leaves the cycle alone unless the pointer is over it.
    if let Some(c) = tab.cycle
        && (c.view == view || pointer.is_some())
        && !c.holds(view, pointer, revision)
    {
        tab.cycle = None;
    }
    if dragging {
        tab.cycle = None;
        return (Vec::new(), 0);
    }
    if pointer.is_some() {
        tab.armed = Some(rect);
    }
    let Some(at) = tab.cycle.map(|c| c.anchor).or(pointer) else { return (Vec::new(), 0) };
    let hits = drawing.hits_at(xf.to_world(at), tol);
    let cache = &mut tab.chain;
    let list = candidates(&hits, |id| {
        project.element(id)?.as_wall()?;
        if !matches!(cache, Some((r, v, w, _)) if *r == revision && *v == view && *w == id) {
            // Only the walls this view draws: hidden ones are not selected with the chain.
            let walls: std::collections::BTreeSet<ElementId> = project.walls().map(|(e, _)| e.id).collect();
            let drawn: std::collections::BTreeSet<ElementId> = (drawing.prims_of(&walls).into_iter())
                .filter_map(|i| drawing.prims.get(i).and_then(cc_views::Prim::element))
                .collect();
            let chain = cc_core::wall_ops::wall_chain_among(project, id, &mut |w| drawn.contains(&w));
            *cache = Some((revision, view, id, chain));
        }
        cache.as_ref().map(|(.., chain)| chain.clone())
    });
    if steps != 0 && list.len() > 1 {
        let cycle = tab.cycle.get_or_insert(Cycle { view, anchor: at, index: 0, revision });
        cycle.index = step(cycle.index, list.len(), steps);
    }
    let index = tab.cycle.map_or(0, |c| c.index);
    let index = if index < list.len() { index } else { 0 };
    (list, index)
}

/// What the status bar says of a candidate: `Category : Type` for an
/// element, or a chain of walls; with ` (n/m)` while Tab cycles.
pub fn label(project: &Project, candidate: &Candidate, cycled: Option<(usize, usize)>) -> Option<String> {
    let text = match candidate {
        Candidate::One(id) => {
            let e = project.element(*id)?;
            t!(
                "status-preselect",
                "category" => crate::actions::t_category(e.category()),
                "type" => project.type_name(e.type_id)
            )
        }
        Candidate::Chain(_) => t!("status-wall-chain"),
    };
    Some(match cycled {
        Some((index, count)) => t!("status-preselect-cycle", "label" => text, "index" => index + 1, "count" => count),
        None => text,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(list: &[u64]) -> Vec<ElementId> {
        list.iter().map(|i| ElementId(*i)).collect()
    }

    #[test]
    fn the_chain_follows_the_first_wall() {
        // Elements 1 and 3 are walls; 1's chain is 1, 5, 6; 3's would be 3, 7.
        let chain_of = |id: ElementId| match id.0 {
            1 => Some(ids(&[5, 1, 6])),
            3 => Some(ids(&[3, 7])),
            _ => None,
        };
        let list = candidates(&ids(&[2, 1, 3]), chain_of);
        assert_eq!(
            list,
            vec![
                Candidate::One(ElementId(2)),
                Candidate::One(ElementId(1)),
                Candidate::Chain(ids(&[5, 1, 6])),
                Candidate::One(ElementId(3)),
            ]
        );
        // A lone wall makes no chain, and the next wall is not asked.
        let mut asked = Vec::new();
        let list = candidates(&ids(&[1, 3]), |id| {
            asked.push(id.0);
            Some(vec![id])
        });
        assert_eq!(list, vec![Candidate::One(ElementId(1)), Candidate::One(ElementId(3))]);
        assert_eq!(asked, [1]);
        assert!(candidates(&[], chain_of).is_empty());
        assert_eq!(Candidate::Chain(ids(&[5, 1])).ids(), ids(&[5, 1]).as_slice());
    }

    #[test]
    fn steps_wrap_both_ways() {
        assert_eq!(step(0, 3, 1), 1);
        assert_eq!(step(2, 3, 1), 0);
        assert_eq!(step(0, 3, -1), 2);
        assert_eq!(step(1, 3, 5), 0);
        assert_eq!(step(0, 1, 1), 0);
        assert_eq!(step(4, 0, 1), 0);
    }

    #[test]
    fn a_cycle_holds_near_its_anchor_in_its_view_and_revision() {
        let c = Cycle { view: ViewId(1), anchor: Pos2::new(100.0, 100.0), index: 1, revision: 7 };
        assert!(c.holds(ViewId(1), Some(Pos2::new(102.0, 101.0)), 7));
        assert!(!c.holds(ViewId(1), Some(Pos2::new(104.0, 100.0)), 7), "moved away");
        assert!(!c.holds(ViewId(1), None, 7), "off the canvas");
        assert!(!c.holds(ViewId(2), Some(Pos2::new(100.0, 100.0)), 7), "another view");
        assert!(!c.holds(ViewId(1), Some(Pos2::new(100.0, 100.0)), 8), "the document changed");
    }

    fn tab(shift: bool, pressed: bool) -> Event {
        let modifiers = if shift { egui::Modifiers::SHIFT } else { egui::Modifiers::NONE };
        Event::Key { key: Key::Tab, physical_key: None, pressed, repeat: false, modifiers }
    }

    #[test]
    fn tab_is_taken_only_over_an_armed_canvas() {
        let rect = Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 100.0));
        let mut steps = 0;
        // Tab, Tab, Shift+Tab: one step on; every Tab event goes.
        let mut events = vec![tab(false, true), tab(false, false), tab(false, true), tab(true, true), tab(true, false)];
        assert!(take_tab(Some(rect), &mut steps, &mut events, false));
        assert_eq!(steps, 1);
        assert!(events.is_empty());
        // Not armed (another tool, no canvas under the pointer), or a widget
        // has the focus or a dialog is open: Tab is left alone.
        for (armed, blocked) in [(None, false), (Some(rect), true)] {
            let mut events = vec![tab(false, true)];
            assert!(!take_tab(armed, &mut steps, &mut events, blocked));
            assert_eq!(events.len(), 1);
        }
        // The pointer left the canvas in this frame.
        let mut events = vec![Event::PointerMoved(Pos2::new(150.0, 50.0)), tab(false, true)];
        assert!(!take_tab(Some(rect), &mut steps, &mut events, false));
        let mut events = vec![Event::PointerGone, tab(false, true)];
        assert!(!take_tab(Some(rect), &mut steps, &mut events, false));
        // Moved within it: taken, and nothing else is.
        let mut events = vec![Event::PointerMoved(Pos2::new(50.0, 50.0)), tab(false, true)];
        assert!(take_tab(Some(rect), &mut steps, &mut events, false));
        assert_eq!((steps, events.len()), (2, 1));
    }
}
