//! Roof joins: a cross wing or bay joined to the roof it meets (see
//! `cc_geom::roof_join`, which works out what each roof keeps).

use super::{created, edit};
use crate::error::{EngineError, Result};
use crate::refs;
use crate::registry::{Command, CommandKind};
use crate::session::Session;
use crate::t;
use cc_core::{ElementId, ElementKind, Project};
use cc_geom::roof_join::{JoinState, RoofJoinGeom};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Whether roof `roof` may be joined to `to`: both roofs, distinct, and no
/// loop of joins (`to` is not joined to `roof`, directly or through others).
pub(crate) fn check(p: &Project, roof: ElementId, to: ElementId) -> Result<()> {
    let is_roof = |id: ElementId| p.element(id).is_some_and(|e| matches!(e.kind, ElementKind::Roof(_)));
    if !is_roof(roof) || !is_roof(to) {
        return Err(EngineError::invalid(t!("msg-roof-join-not-roof")));
    }
    if roof == to {
        return Err(EngineError::invalid(t!("msg-roof-join-same")));
    }
    if p.roof_join_loops(roof, to) {
        return Err(EngineError::invalid(t!("msg-roof-join-cycle", "to" => to.0)));
    }
    Ok(())
}

/// Join a roof to another it meets (a cross wing or bay against a main
/// roof): its end runs on into the other roof, or is cut, until it meets
/// that roof's surface, forming valleys, and the other roof is cut away
/// under it. They stay joined when either changes; walls attached to either
/// follow; areas, tiles, framing, schedules and IFC count each part once.
/// Joining it again replaces the join.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RoofJoin {
    /// The roof whose end runs into the other (the wing or bay).
    pub roof: u64,
    /// The roof it meets (the main roof), cut away under it.
    pub to: u64,
}

impl Command for RoofJoin {
    const NAME: &'static str = "roof_join";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let p = s.project();
        let (roof, to) = (refs::element(p, self.roof)?, refs::element(p, self.to)?);
        check(p, roof, to)?;
        match cc_geom::roof_join::preview(p, roof, to) {
            JoinState::Joined => {}
            JoinState::Apart => return Err(EngineError::invalid(t!("msg-roof-join-apart"))),
            _ => return Err(EngineError::invalid(t!("msg-roof-join-failed"))),
        }
        edit(s, t!("tx-roof-join"), |e| {
            let Some(ElementKind::Roof(r)) = e.element_mut(roof).map(|x| &mut x.kind) else {
                return Err(EngineError::invalid(t!("msg-roof-join-not-roof")));
            };
            r.join_to = Some(to);
            Ok(())
        })?;
        Ok(created(s, roof))
    }
}

/// Unjoin a roof from the roof it was joined to: both are built as drawn again.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RoofUnjoin {
    /// Roof id.
    pub roof: u64,
}

impl Command for RoofUnjoin {
    const NAME: &'static str = "roof_unjoin";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let roof = refs::element(s.project(), self.roof)?;
        match s.project().element(roof).map(|e| &e.kind) {
            Some(ElementKind::Roof(r)) if r.join_to.is_some() => {}
            Some(ElementKind::Roof(_)) => return Err(EngineError::not_found(t!("msg-roof-join-none"))),
            _ => return Err(EngineError::invalid(t!("msg-roof-join-not-roof"))),
        }
        edit(s, t!("tx-roof-unjoin"), |e| {
            if let Some(ElementKind::Roof(r)) = e.element_mut(roof).map(|x| &mut x.kind) {
                r.join_to = None;
            }
            Ok(())
        })?;
        Ok(created(s, roof))
    }
}

/// A roof's joins as JSON (`element_get`'s `derived.join`): the roof it is
/// joined to, whether the join is built (`active`), why not (`issue`: the
/// state), the valleys (3D, on the tops) and their length, and the roofs
/// joined to it that cut it.
pub(crate) fn join_json(j: &RoofJoinGeom) -> Value {
    let issue = j.state.filter(|s| *s != JoinState::Joined).map(JoinState::name);
    json!({
        "to": j.to.map(|t| t.0),
        "active": j.active(),
        "issue": issue,
        "valleys": j.valleys.iter().map(|(a, b)| [a.to_array(), b.to_array()]).collect::<Vec<_>>(),
        "valley_length": j.valley_length(),
        "cut_by": j.cut_by.iter().map(|c| c.0).collect::<Vec<_>>(),
    })
}
