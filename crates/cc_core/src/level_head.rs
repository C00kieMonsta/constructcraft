//! How an elevation or a section shows each level: the level's line across
//! the view and its heads (a symbol with the level's name and elevation) at
//! the line's ends. Each view keeps its own [`LevelHead`] per level in
//! [`crate::View::level_heads`]; a level with none is drawn as by default,
//! with its head at the end (the right) of its line.

use crate::{LevelId, View, ViewKind};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Farthest a head's elbow moves it from its line, meters in the view.
pub const MAX_ELBOW: f64 = 1000.0;

/// Which ends of a level's line carry a head.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum LevelHeads {
    /// At the end (the right) only.
    #[default]
    End,
    /// At the start (the left) only.
    Start,
    /// At both ends.
    Both,
    /// None: the line alone.
    None,
}

impl LevelHeads {
    pub const ALL: [LevelHeads; 4] = [LevelHeads::End, LevelHeads::Start, LevelHeads::Both, LevelHeads::None];

    pub fn at_start(self) -> bool {
        matches!(self, LevelHeads::Start | LevelHeads::Both)
    }

    pub fn at_end(self) -> bool {
        matches!(self, LevelHeads::End | LevelHeads::Both)
    }

    /// The name commands read and write.
    pub fn code(self) -> &'static str {
        match self {
            LevelHeads::End => "end",
            LevelHeads::Start => "start",
            LevelHeads::Both => "both",
            LevelHeads::None => "none",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|h| h.code() == code)
    }

    /// Translation key of the choice's name.
    pub fn key(self) -> &'static str {
        match self {
            LevelHeads::End => "level-heads-end",
            LevelHeads::Start => "level-heads-start",
            LevelHeads::Both => "level-heads-both",
            LevelHeads::None => "level-heads-none",
        }
    }
}

/// How one elevation or section shows one level.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct LevelHead {
    /// The level is not drawn in this view (line and heads).
    #[serde(default, skip_serializing_if = "is_false")]
    pub hidden: bool,
    /// Which ends carry a head.
    #[serde(default, skip_serializing_if = "is_end")]
    pub heads: LevelHeads,
    /// How far the end head is moved up (negative: down) from the line,
    /// meters in the view, with an elbow leader back to it: so the heads of
    /// levels close together do not overlap.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub elbow: f64,
    /// The same for the start head.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub start_elbow: f64,
}

impl LevelHead {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

fn is_false(v: &bool) -> bool {
    !*v
}

fn is_end(h: &LevelHeads) -> bool {
    *h == LevelHeads::End
}

fn is_zero(v: &f64) -> bool {
    *v == 0.0
}

/// Whether views of this kind draw the levels (elevations and sections).
pub fn shows_levels(kind: &ViewKind) -> bool {
    matches!(kind, ViewKind::Elevation { .. } | ViewKind::Section { .. })
}

/// Repair a view's level heads against the project's `levels`: heads of
/// missing levels or kept by views that draw no levels go, elbows out of
/// reach or not finite are reset, and heads left as the default are
/// removed. Returns how many problems were fixed.
pub fn repair_view_level_heads(view: &mut View, levels: &BTreeSet<LevelId>) -> usize {
    let mut n = 0;
    let shown = shows_levels(&view.kind);
    let heads: &mut BTreeMap<LevelId, LevelHead> = &mut view.level_heads;
    heads.retain(|id, h| {
        if !(shown && levels.contains(id)) {
            n += 1;
            return false;
        }
        for e in [&mut h.elbow, &mut h.start_elbow] {
            if !(e.is_finite() && e.abs() <= MAX_ELBOW) {
                *e = 0.0;
                n += 1;
            }
        }
        !h.is_default()
    });
    n
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Direction, ViewId};

    #[test]
    fn repair_drops_missing_levels_and_bad_elbows() {
        let mut v = View::new(ViewId(1), "South", 100, ViewKind::Elevation { direction: Direction::South });
        v.level_heads.insert(LevelId(1), LevelHead { elbow: f64::NAN, hidden: true, ..Default::default() });
        v.level_heads.insert(LevelId(2), LevelHead { elbow: 0.5, ..Default::default() });
        v.level_heads.insert(LevelId(3), LevelHead { start_elbow: f64::INFINITY, ..Default::default() });
        let levels = BTreeSet::from([LevelId(1), LevelId(3)]);
        assert_eq!(repair_view_level_heads(&mut v, &levels), 3);
        assert_eq!(v.level_heads.len(), 1, "the level gone and the head left as the default go");
        assert_eq!(v.level_heads.get(&LevelId(1)), Some(&LevelHead { hidden: true, ..Default::default() }));
    }

    #[test]
    fn views_without_levels_keep_no_level_heads() {
        let mut v = View::new(ViewId(1), "Drafting", 10, ViewKind::Drafting {});
        v.level_heads.insert(LevelId(1), LevelHead { hidden: true, ..Default::default() });
        assert_eq!(repair_view_level_heads(&mut v, &BTreeSet::from([LevelId(1)])), 1);
        assert!(v.level_heads.is_empty());
    }

    #[test]
    fn heads_codes_round_trip() {
        for h in LevelHeads::ALL {
            assert_eq!(LevelHeads::from_code(h.code()), Some(h));
        }
    }
}
