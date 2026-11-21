//! How elevations and sections show the levels: hide a level in one view,
//! choose which ends of its line carry a head, move a head up or down with
//! an elbow so the heads of levels close together do not overlap.

use super::edit;
use crate::error::{EngineError, Result};
use crate::refs::{self, Ref, finite};
use crate::registry::{Command, CommandKind};
use crate::session::Session;
use crate::t;
use cc_core::level_head::{LevelHead, LevelHeads, MAX_ELBOW, shows_levels};
use cc_core::{LevelId, Project, View};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Adjust how an elevation or a section shows some levels, in one undo
/// step: hide them in this view (`hidden`) or show them again, put their
/// heads at the `end` of the line (the right, the default), at its
/// `start`, at `both` ends or at `none`, and move a head up (or down,
/// negative) from its line by an elbow (`elbow` for the end head,
/// `start_elbow` for the start head, meters in the view; `0` puts it back
/// on the line). `reset` puts the levels back as the view draws them by
/// default before the other fields apply. Each view keeps its own
/// settings; the category *Levels* of the view's graphics hides them all.
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct LevelHeadSet {
    /// Elevation or section id or name.
    pub view: Ref,
    /// The levels (ids or names).
    pub levels: Vec<Ref>,
    /// Hide the levels in this view, or show them again.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    /// Which ends carry a head: `end`, `start`, `both` or `none`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heads: Option<String>,
    /// How far the end head is moved up from the line, meters in the view (negative: down).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elbow: Option<f64>,
    /// How far the start head is moved up from the line, meters in the view (negative: down).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_elbow: Option<f64>,
    /// Put the levels back as the view draws them by default, before the other fields apply.
    #[serde(default)]
    pub reset: bool,
}

fn elbow(v: f64, what: &str) -> Result<f64> {
    let v = finite(v, what)?;
    if v.abs() > MAX_ELBOW {
        return Err(EngineError::invalid(format!("{what} stays within {MAX_ELBOW} m of the line")));
    }
    Ok(v)
}

impl Command for LevelHeadSet {
    const NAME: &'static str = "level_head_set";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let p = s.project();
        let view = refs::view(p, &self.view)?;
        let current = p.view(view).ok_or_else(|| EngineError::not_found("view"))?;
        if !shows_levels(&current.kind) {
            return Err(EngineError::invalid("levels are drawn in elevations and sections"));
        }
        if self.levels.is_empty() {
            return Err(EngineError::invalid("give one level or more"));
        }
        let heads = match &self.heads {
            Some(code) => Some(
                LevelHeads::from_code(code)
                    .ok_or_else(|| EngineError::invalid(format!("heads is end, start, both or none, not {code:?}")))?,
            ),
            None => None,
        };
        let end_elbow = self.elbow.map(|v| elbow(v, "elbow")).transpose()?;
        let start_elbow = self.start_elbow.map(|v| elbow(v, "start_elbow")).transpose()?;
        let mut changed: Vec<(LevelId, LevelHead)> = Vec::new();
        for r in &self.levels {
            let level = refs::level(p, r)?;
            let mut h = if self.reset { LevelHead::default() } else { head(current, level) };
            if let Some(hidden) = self.hidden {
                h.hidden = hidden;
            }
            if let Some(heads) = heads {
                h.heads = heads;
            }
            if let Some(e) = end_elbow {
                h.elbow = e;
            }
            if let Some(e) = start_elbow {
                h.start_elbow = e;
            }
            changed.push((level, h));
        }
        edit(s, t!("tx-set-level-head"), |e| {
            let v = e.view_mut(view).ok_or_else(|| EngineError::not_found("view"))?;
            for (level, h) in &changed {
                if h.is_default() {
                    v.level_heads.remove(level);
                } else {
                    v.level_heads.insert(*level, *h);
                }
            }
            Ok(())
        })?;
        let p = s.project();
        let levels: Vec<Value> =
            changed.iter().filter_map(|(level, _)| p.view(view).map(|v| head_json(p, v, *level))).collect();
        Ok(json!({ "view": view.0, "levels": levels }))
    }
}

/// How `view` shows `level`.
pub fn head(view: &View, level: LevelId) -> LevelHead {
    view.level_heads.get(&level).copied().unwrap_or_default()
}

fn head_json(p: &Project, view: &View, level: LevelId) -> Value {
    let h = head(view, level);
    json!({
        "id": level.0,
        "name": p.level(level).map(|l| l.name.clone()),
        "hidden": h.hidden,
        "heads": h.heads.code(),
        "elbow": h.elbow,
        "start_elbow": h.start_elbow,
    })
}

/// How an elevation or a section shows each level, for `view_get` (`null`
/// for views that draw no levels).
pub fn view_level_heads_json(p: &Project, view: &View) -> Value {
    if !shows_levels(&view.kind) {
        return Value::Null;
    }
    p.levels.iter().map(|l| head_json(p, view, l.id)).collect()
}
