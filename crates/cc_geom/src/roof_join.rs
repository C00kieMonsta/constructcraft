//! Roof joins (issue #381): a roof joined to another ([`cc_core::Roof::join_to`],
//! a cross wing or a bay against a main roof) runs on into it, or is cut,
//! until it meets the other roof's surface, forming valleys, and the other
//! roof is cut away under it. Nothing of this is stored: [`resolve`] works it
//! out from the roofs as drawn whenever the model is built.
//!
//! Each roof's slab is a set of [`Piece`]s, parts of planes in plan (the
//! faces of its skeleton, see [`crate::roofs::RoofBody::pieces`]), with its
//! vertical thickness. The joined roof `L` meets its target `M` along a run
//! of footprint edges whose middles lie on `M`'s pieces (its domain); one
//! standing apart from `M` meets it across the gap its nearest edge looks
//! out over. When both edges beside the run are sloped eaves (the end of a
//! cross wing), the run is moved far past `M` as a gable with no overhang,
//! the edges beside it running on: `L` is built over that longer footprint
//! (a hip wing gets a gable at its joined end). A wing whose edges beside
//! the run are gables is only cut.
//!
//! Of each piece of `L`, it keeps what lies outside `M` within its own
//! outline (the run moved across the gap), and over `M` where its top is
//! above `M`'s top; of those parts, the ones joined to the part outside `M`
//! (drops the patches that come up again past `M`'s ridge). Where the kept
//! pieces' edges run over `M` with both tops at one height are the valleys;
//! with none, the join does nothing (`L` wholly over or under `M`, or
//! tucked under its overhang). `M` is then cut away under the kept pieces.
//! Roofs joined in a chain are resolved from the free end; several wings on
//! one roof each meet it uncut. Loops of joins and missing targets leave the
//! roofs as drawn. Region operations are bounded ([`region`]), and every
//! failure leaves both roofs as drawn.

use crate::{Face, Solid, polygon, region};
use cc_core::{ElementId, ElementKind, Project, Roof, RoofShape};
use glam::{DVec2, DVec3};
use std::collections::{BTreeMap, BTreeSet};

/// Most points the region operations of one join take.
const MAX_JOIN_POINTS: usize = 50_000;
/// Most edges the outline of a roof's pieces is worked out from.
const MAX_EDGES: usize = 20_000;
/// A point this close to a roof's pieces, meters, lies on them.
const NEAR: f64 = 1e-6;
/// Tops closer than this, meters, meet (valleys).
const HEIGHT_TOL: f64 = 1e-6;
/// An edge whose ends are this close to another's line, meters, runs along it.
const LINE_TOL: f64 = 1e-6;
/// Pieces smaller than this, square meters, are left out.
const MIN_PIECE: f64 = 1e-9;

/// How a roof's join turned out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinState {
    /// The roof meets the other along valleys: both are built joined.
    Joined,
    /// The roofs do not meet (apart, or one wholly over or under the other):
    /// both are built as drawn.
    Apart,
    /// The join could not be worked out: both are built as drawn.
    Failed,
    /// The roof is on a loop of joins.
    Cycle,
    /// The roof it is joined to is not in the model (or not shown).
    Missing,
}

impl JoinState {
    /// The state's name in commands' results.
    pub fn name(self) -> &'static str {
        match self {
            JoinState::Joined => "joined",
            JoinState::Apart => "apart",
            JoinState::Failed => "failed",
            JoinState::Cycle => "cycle",
            JoinState::Missing => "missing",
        }
    }
}

/// A part of a roof's slab over one plane: its outline in plan
/// (counter-clockwise; openings joined to it by bridges walked there and
/// back, see [`polygon::keyhole`]) and its underside `z = c + g · p`.
#[derive(Debug, Clone, PartialEq)]
pub struct Piece {
    pub plan: Vec<DVec2>,
    pub c: f64,
    pub g: DVec2,
}

impl Piece {
    /// The underside's height at `p`.
    pub fn z(&self, p: DVec2) -> f64 {
        self.c + self.g.dot(p)
    }

    /// Plan area.
    pub fn area(&self) -> f64 {
        polygon::area(&self.plan)
    }

    /// True area of its plane.
    pub fn sloped_area(&self) -> f64 {
        self.area() * (1.0 + self.g.length_squared()).sqrt()
    }
}

/// A roof's slab as pieces of planes, with its vertical thickness and its
/// ridges, hips and valleys in plan.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RoofSurface {
    pub pieces: Vec<Piece>,
    pub thickness: f64,
    pub lines: Vec<(DVec2, DVec2)>,
}

/// What joins made of a roof: one joined to another, or one that joined
/// roofs cut ([`crate::ModelGeometry::roof_joins`]).
#[derive(Debug, Clone, PartialEq)]
pub struct RoofJoinGeom {
    /// The roof it is joined to.
    pub to: Option<ElementId>,
    /// How its own join turned out (`None` when it is joined to none).
    pub state: Option<JoinState>,
    /// The roofs joined to it that cut it.
    pub cut_by: Vec<ElementId>,
    /// Its outline in plan as the joins left it, the valleys left out (in
    /// `valleys`); empty when no join changed it.
    pub outline_edges: Vec<(DVec2, DVec2)>,
    /// Where it meets the roof it is joined to and the roofs that cut it,
    /// along their tops.
    pub valleys: Vec<(DVec3, DVec3)>,
    /// Plan area within the footprints of it and the roofs it is joined
    /// with (the schedules' Area).
    pub area: f64,
    /// Plan area covered.
    pub plan_area: f64,
    /// True area of its planes.
    pub sloped_area: f64,
}

impl RoofJoinGeom {
    /// Whether the roof is built from its joins: joined itself, or cut by a
    /// roof joined to it.
    pub fn active(&self) -> bool {
        self.state == Some(JoinState::Joined) || !self.cut_by.is_empty()
    }

    /// Length of the valleys.
    pub fn valley_length(&self) -> f64 {
        self.valleys.iter().map(|(a, b)| a.distance(*b)).sum()
    }
}

/// Every roof join of a model: the slabs of the roofs they change, and what
/// they made of each roof joined or cut.
#[derive(Debug, Clone, Default)]
pub struct Resolved {
    /// The roofs built from their joins.
    pub surfaces: BTreeMap<ElementId, RoofSurface>,
    pub joins: BTreeMap<ElementId, RoofJoinGeom>,
}

/// Roof `id` and the roofs joined with it by joins that are built,
/// directly or through others, either way.
pub fn joined_group(joins: &BTreeMap<ElementId, RoofJoinGeom>, id: ElementId) -> BTreeSet<ElementId> {
    let mut group = BTreeSet::from([id]);
    let mut todo = vec![id];
    while let Some(at) = todo.pop() {
        let Some(j) = joins.get(&at) else { continue };
        let to = j.to.filter(|_| j.state == Some(JoinState::Joined));
        for next in to.into_iter().chain(j.cut_by.iter().copied()) {
            if group.insert(next) {
                todo.push(next);
            }
        }
    }
    group
}

/// The roof joins of `project`.
pub fn resolve(project: &Project) -> Resolved {
    resolve_with(project, &|id| project.roof_join_target(id), None)
}

/// How joining roof `roof` to roof `to` would turn out, the model otherwise
/// as it is.
pub fn preview(project: &Project, roof: ElementId, to: ElementId) -> JoinState {
    let link = |id: ElementId| if id == roof { Some(to) } else { project.roof_join_target(id) };
    // The roof and those its target is joined to, one after another: what the join depends on.
    let mut only = BTreeSet::from([roof]);
    let mut at = to;
    while only.insert(at) {
        match link(at) {
            Some(next) => at = next,
            None => break,
        }
    }
    let resolved = resolve_with(project, &link, Some(&only));
    resolved.joins.get(&roof).and_then(|j| j.state).unwrap_or(JoinState::Missing)
}

/// A roof as joins work on it.
struct Input<'a> {
    r: &'a Roof,
    level_z: f64,
    thk: f64,
}

impl Input<'_> {
    /// The roof's slab as drawn.
    fn surface(&self) -> RoofSurface {
        surface_of(self.r, self.level_z, self.thk)
    }
}

/// The slab of `r` as pieces.
fn surface_of(r: &Roof, level_z: f64, thk: f64) -> RoofSurface {
    let body = crate::roofs::body(r, level_z, thk);
    RoofSurface { pieces: body.pieces(), thickness: body.thickness(), lines: body.lines().unwrap_or_default() }
}

/// What a joined roof keeps.
struct Kept {
    surface: RoofSurface,
    valleys: Vec<(DVec3, DVec3)>,
}

/// The joins of `project` with each roof joined to `link(roof)`, among the
/// roofs `only` holds (all when `None`).
fn resolve_with(
    project: &Project,
    link: &dyn Fn(ElementId) -> Option<ElementId>,
    only: Option<&BTreeSet<ElementId>>,
) -> Resolved {
    let wanted = |id: ElementId| only.is_none_or(|o| o.contains(&id));
    let roofs: BTreeMap<ElementId, Input> = project
        .elements
        .values()
        .filter(|e| wanted(e.id))
        .filter_map(|e| {
            let ElementKind::Roof(r) = &e.kind else { return None };
            let ty = project.element_type(e.type_id)?;
            Some((e.id, Input { r, level_z: project.level_elevation(e.level), thk: ty.thickness() }))
        })
        .collect();
    let links: BTreeMap<ElementId, ElementId> = roofs.keys().filter_map(|id| Some((*id, link(*id)?))).collect();
    if links.is_empty() {
        return Resolved::default();
    }
    // Joins to no roof, or to itself.
    let mut state: BTreeMap<ElementId, JoinState> = BTreeMap::new();
    for (&id, &to) in &links {
        if to == id {
            state.insert(id, JoinState::Cycle);
        } else if !roofs.contains_key(&to) {
            state.insert(id, JoinState::Missing);
        }
    }
    let valid = |id: ElementId| links.get(&id).copied().filter(|to| *to != id && roofs.contains_key(to));
    // Loops: each roof joins one other at most, so a walk from each finds a loop once.
    let mut done = BTreeSet::new();
    for &start in links.keys() {
        let (mut path, mut on_path) = (Vec::new(), BTreeSet::new());
        let mut at = Some(start);
        while let Some(id) = at {
            if done.contains(&id) {
                break;
            }
            if !on_path.insert(id) {
                let from = path.iter().position(|p| *p == id).unwrap_or(0);
                for c in path.get(from..).unwrap_or_default() {
                    state.insert(*c, JoinState::Cycle);
                }
                break;
            }
            path.push(id);
            at = valid(id);
        }
        done.extend(path);
    }
    // How many joins lie between each roof and a roof joined to none: the order they are resolved in.
    let free = |id: ElementId| !links.contains_key(&id) || state.contains_key(&id);
    let mut depth: BTreeMap<ElementId, usize> = BTreeMap::new();
    for &id in links.keys().filter(|id| !state.contains_key(id)) {
        let mut path = Vec::new();
        let mut at = id;
        let mut from = 0;
        for _ in 0..=roofs.len() {
            if let Some(d) = depth.get(&at) {
                from = *d;
                break;
            }
            if free(at) {
                break;
            }
            path.push(at);
            match valid(at) {
                Some(next) => at = next,
                None => break,
            }
        }
        for (k, p) in path.iter().rev().enumerate() {
            depth.insert(*p, from + k + 1);
        }
    }
    let mut order: Vec<(usize, ElementId)> = depth.iter().map(|(id, d)| (*d, *id)).collect();
    order.sort();

    // Each roof after its own join, from the free ends on.
    let mut after: BTreeMap<ElementId, RoofSurface> = BTreeMap::new();
    let mut kept: BTreeMap<ElementId, Kept> = BTreeMap::new();
    for (_, id) in order {
        let (Some(input), Some(to)) = (roofs.get(&id), valid(id)) else { continue };
        let target = match after.get(&to) {
            Some(s) => s.clone(),
            None => {
                let Some(t) = roofs.get(&to) else { continue };
                let s = t.surface();
                after.insert(to, s.clone());
                s
            }
        };
        match join(input, &target) {
            Ok(k) => {
                state.insert(id, JoinState::Joined);
                after.insert(id, k.surface.clone());
                kept.insert(id, k);
            }
            Err(s) => {
                state.insert(id, s);
            }
        }
    }

    // The roofs joined ones cut, and the slabs of every roof the joins change.
    let mut cutters: BTreeMap<ElementId, Vec<ElementId>> = BTreeMap::new();
    for id in kept.keys() {
        if let Some(to) = valid(*id) {
            cutters.entry(to).or_default().push(*id);
        }
    }
    let mut out = Resolved::default();
    let mut valleys: BTreeMap<ElementId, Vec<(DVec3, DVec3)>> = BTreeMap::new();
    let changed: BTreeSet<ElementId> = kept.keys().chain(cutters.keys()).copied().collect();
    for id in changed {
        let Some(input) = roofs.get(&id) else { continue };
        let mut surface = after.get(&id).cloned().unwrap_or_else(|| input.surface());
        let mut own = kept.get(&id).map(|k| k.valleys.clone()).unwrap_or_default();
        if let Some(wings) = cutters.get(&id) {
            let by: Vec<&Kept> = wings.iter().filter_map(|w| kept.get(w)).collect();
            let plans: Vec<Vec<DVec2>> =
                by.iter().flat_map(|k| k.surface.pieces.iter().map(|p| p.plan.clone())).collect();
            let wing_valleys: Vec<(DVec3, DVec3)> = by.iter().flat_map(|k| k.valleys.iter().copied()).collect();
            surface.pieces = cut(&surface.pieces, &plans);
            let mut lines = clip_lines(&surface.lines, &surface.pieces);
            lines.extend(wing_valleys.iter().map(|(a, b)| (a.truncate(), b.truncate())));
            surface.lines = lines;
            own.extend(wing_valleys);
        }
        valleys.insert(id, own);
        out.surfaces.insert(id, surface);
    }

    // What became of each roof joined or cut.
    let partners = |id: ElementId| -> Vec<ElementId> {
        let mut p = vec![id];
        if state.get(&id) == Some(&JoinState::Joined)
            && let Some(to) = valid(id)
        {
            p.push(to);
        }
        p.extend(cutters.get(&id).into_iter().flatten().copied());
        p
    };
    let roofs_with: BTreeSet<ElementId> = links.keys().chain(cutters.keys()).copied().collect();
    for id in roofs_with {
        let Some(input) = roofs.get(&id) else { continue };
        let r = input.r;
        let own_valleys = valleys.remove(&id).unwrap_or_default();
        let geom = match out.surfaces.get(&id) {
            Some(s) => {
                let footprints: Vec<&Roof> = partners(id).iter().filter_map(|p| roofs.get(p).map(|i| i.r)).collect();
                RoofJoinGeom {
                    to: links.get(&id).copied(),
                    state: links.contains_key(&id).then(|| state.get(&id).copied().unwrap_or(JoinState::Missing)),
                    cut_by: cutters.get(&id).cloned().unwrap_or_default(),
                    outline_edges: outline_edges(&s.pieces, &own_valleys),
                    area: area_within(&s.pieces, &footprints),
                    plan_area: s.pieces.iter().map(Piece::area).sum(),
                    sloped_area: s.pieces.iter().map(Piece::sloped_area).sum(),
                    valleys: own_valleys,
                }
            }
            None => {
                let plan_area = crate::roofs::plan_area(r);
                let slope = match r.shape {
                    RoofShape::Flat => 0.0,
                    RoofShape::Shed { slope, .. } | RoofShape::Gable { slope, .. } | RoofShape::Hip { slope } => slope,
                };
                RoofJoinGeom {
                    to: links.get(&id).copied(),
                    state: links.contains_key(&id).then(|| state.get(&id).copied().unwrap_or(JoinState::Missing)),
                    cut_by: Vec::new(),
                    outline_edges: Vec::new(),
                    valleys: Vec::new(),
                    area: r.area(),
                    plan_area,
                    sloped_area: plan_area * (1.0 + slope * slope).sqrt(),
                }
            }
        };
        out.joins.insert(id, geom);
    }
    out
}

/// Join roof `input` to `target` (the slab of the roof it is joined to,
/// after that roof's own join): what it keeps, or why it keeps nothing.
fn join(input: &Input, target: &RoofSurface) -> Result<Kept, JoinState> {
    let r = input.r;
    let domain: Vec<Vec<DVec2>> = target.pieces.iter().map(|p| p.plan.clone()).collect();
    if domain.is_empty() {
        return Err(JoinState::Apart);
    }
    let fp = polygon::ccw(&r.footprint);
    let n = fp.len();
    if n < 3 || fp.iter().any(|p| !p.is_finite()) {
        return Err(JoinState::Failed);
    }
    let on_target = |p: DVec2| domain.iter().any(|d| polygon::contains(d, p) || ring_distance(d, p) <= NEAR);
    let meets: Vec<bool> = (0..n).map(|i| on_target((fp[i] + fp[(i + 1) % n]) * 0.5)).collect();
    let own = crate::roofs::ccw_edge_slopes(r);
    let overhangs = crate::roofs::ccw_edge_overhangs(r);
    let outline = crate::roofs::outline(r).points;
    let size = diagonal(&fp);
    // The run of edges that meets the target, and the gap between them.
    let (run, gap) = if meets.iter().all(|m| *m) {
        (None, 0.0)
    } else if meets.iter().any(|m| *m) {
        (longest_run(&fp, &meets), 0.0)
    } else if overlaps(&outline, &domain).ok_or(JoinState::Failed)? {
        (None, 0.0)
    } else {
        match nearest_hit(&fp, &domain) {
            Some((i, t)) if t <= size => (Some((i, 1)), t),
            _ => return Err(JoinState::Apart),
        }
    };
    // What the roof may keep outside the target: its own outline, the run moved across the gap.
    let within = match run {
        Some((s, _)) if gap > 0.0 => crate::roofs::outline(&moved(r, &fp, (&own, &overhangs), s, gap)).points,
        _ => outline,
    };
    // The roof as built for the join: run on past the target when its end is a cross wing's.
    let reach = diagonal(&domain.iter().flatten().copied().collect::<Vec<_>>()) + size;
    let built = match run {
        Some(run) if extends(r, &fp, &own, run) => {
            extend(r, &fp, (&own, &overhangs), run, reach).ok_or(JoinState::Failed)?
        }
        _ => Roof { join_to: None, sketch: None, ..r.clone() },
    };
    let body = crate::roofs::body(&built, input.level_z, input.thk);
    let pieces = body.pieces();
    let (t_l, t_m) = (body.thickness(), target.thickness);

    // Of each piece: the part outside the target within the roof's own
    // outline, and over each piece of the target the part whose top is above it.
    let mut parts: Vec<(usize, Vec<DVec2>, bool)> = Vec::new();
    for (f, pf) in pieces.iter().enumerate() {
        let outside = region::covered_except(std::slice::from_ref(&pf.plan), &domain, Some(&within), MAX_JOIN_POINTS)
            .ok_or(JoinState::Failed)?;
        parts.extend(outside.into_iter().map(|ring| (f, ring, true)));
        let bf = Bounds::of(&pf.plan);
        for pg in target.pieces.iter().filter(|pg| bf.meets(&Bounds::of(&pg.plan))) {
            // top_L >= top_M  <=>  p · (g_M - g_L) <= c_L + t_L - c_M - t_M.
            let above = crate::roofs::clip_half_plane(&pf.plan, pg.g - pf.g, pf.c + t_l - pg.c - t_m);
            if above.len() < 3 || polygon::area(&above) <= MIN_PIECE {
                continue;
            }
            let over = region::covered(&[above], Some(&pg.plan), MAX_JOIN_POINTS).ok_or(JoinState::Failed)?;
            parts.extend(over.into_iter().map(|ring| (f, ring, false)));
        }
    }
    parts.retain(|(_, ring, _)| polygon::area(ring) > MIN_PIECE);
    // The parts joined to the part outside the target (else to the part within the roof's own outline).
    let plans: Vec<&[DVec2]> = parts.iter().map(|(_, ring, _)| ring.as_slice()).collect();
    let edges = Edges::of(&plans).ok_or(JoinState::Failed)?;
    let groups = edges.groups(plans.len());
    let mut anchors: Vec<usize> = (0..parts.len()).filter(|i| parts[*i].2).collect();
    if anchors.is_empty() {
        anchors = (0..parts.len())
            .filter(|i| interior_point(&parts[*i].1).is_some_and(|p| polygon::contains(&within, p)))
            .collect();
    }
    let held: BTreeSet<usize> = anchors.iter().filter_map(|i| groups.get(*i).copied()).collect();
    let kept: Vec<Piece> = parts
        .into_iter()
        .enumerate()
        .filter(|(i, _)| groups.get(*i).is_some_and(|g| held.contains(g)))
        .filter_map(|(_, (f, plan, _))| pieces.get(f).map(|p| Piece { plan, c: p.c, g: p.g }))
        .collect();
    if kept.is_empty() {
        return Err(JoinState::Apart);
    }
    // Valleys: the kept pieces' edges over the target where both tops meet.
    let plans: Vec<&[DVec2]> = kept.iter().map(|p| p.plan.as_slice()).collect();
    let mut valleys = Vec::new();
    for (k, a, b) in Edges::of(&plans).ok_or(JoinState::Failed)?.boundary() {
        let (Some(pf), Some(pg)) = (kept.get(k), piece_at(&target.pieces, (a + b) * 0.5)) else { continue };
        let gap = |p: DVec2| (pf.z(p) + t_l) - (pg.z(p) + t_m);
        if a.distance(b) > LINE_TOL && gap(a).abs() < HEIGHT_TOL && gap(b).abs() < HEIGHT_TOL {
            valleys.push((a.extend(pf.z(a) + t_l), b.extend(pf.z(b) + t_l)));
        }
    }
    if valleys.is_empty() {
        return Err(JoinState::Apart);
    }
    let mut lines = clip_lines(&body.lines().unwrap_or_default(), &kept);
    lines.extend(valleys.iter().map(|(a, b)| (a.truncate(), b.truncate())));
    Ok(Kept { surface: RoofSurface { pieces: kept, thickness: t_l, lines }, valleys })
}

/// The longest run (by length) of consecutive edges that `meets`: its
/// first edge and how many edges it has.
fn longest_run(fp: &[DVec2], meets: &[bool]) -> Option<(usize, usize)> {
    let n = meets.len();
    let at = |i: usize| meets.get(i % n).copied().unwrap_or(false);
    let mut best: Option<(usize, usize, f64)> = None;
    for s in (0..n).filter(|&s| at(s) && !at(s + n - 1)) {
        let mut len = 0;
        let mut total = 0.0;
        while len < n && at(s + len) {
            total += fp[(s + len) % n].distance(fp[(s + len + 1) % n]);
            len += 1;
        }
        if best.is_none_or(|b| total > b.2) {
            best = Some((s, len, total));
        }
    }
    best.map(|(s, len, _)| (s, len))
}

/// Whether a roof's end at the run `(s, len)` of its counter-clockwise
/// footprint `fp` runs on into the roof it joins: a gable or hip roof whose
/// edges beside the run are sloped eaves (a cross wing's end).
fn extends(r: &Roof, fp: &[DVec2], own: &[Option<f64>], (s, len): (usize, usize)) -> bool {
    let n = fp.len();
    let (slope, ridge) = match r.shape {
        RoofShape::Gable { slope, ridge_deg } => (slope, Some(DVec2::from_angle(ridge_deg.to_radians()))),
        RoofShape::Hip { slope } => (slope, None),
        _ => return false,
    };
    if n < len + 2 {
        return false;
    }
    let slopes = crate::roofs::edge_slopes(fp, slope, ridge, own);
    let sloped = |i: usize| slopes.get(i % n).is_some_and(|s| *s > 0.0);
    sloped(s + n - 1) && sloped(s + len)
}

/// Roof `r` over the counter-clockwise footprint `pts`, with the edge
/// slopes and overhangs `data` in its order.
fn roof_over(r: &Roof, pts: Vec<DVec2>, data: (Vec<Option<f64>>, Vec<Option<f64>>)) -> Roof {
    Roof { footprint: pts, edge_slopes: data.0, edge_overhangs: data.1, sketch: None, join_to: None, ..r.clone() }
}

/// Roof `r` (counter-clockwise footprint `fp`, its edge slopes and
/// overhangs in that order) with its run `(s, len)` replaced by one gable
/// edge with no overhang, `reach` further on along the edges beside it (or
/// less, should that cross them over); else the run moved `reach` out.
/// `None` when no such footprint is simple.
fn extend(
    r: &Roof,
    fp: &[DVec2],
    (own, overhangs): (&[Option<f64>], &[Option<f64>]),
    (s, len): (usize, usize),
    reach: f64,
) -> Option<Roof> {
    let n = fp.len();
    let e = (s + len) % n;
    let p = (s + n - 1) % n;
    let before = (fp[s] - fp[p]).try_normalize()?;
    let after = (fp[(e + 1) % n] - fp[e]).try_normalize()?;
    // The points off the run: after its end round to before its start.
    let rest: Vec<DVec2> = (1..n - len).map(|k| fp[(e + k) % n]).collect();
    // Edges: the one after the run round to the one before it, then the new gable.
    let data = |v: &[Option<f64>]| -> Vec<Option<f64>> {
        (0..n - len).map(|k| v.get((e + k) % n).copied().flatten()).chain([Some(0.0)]).collect()
    };
    for d in [reach, reach / 2.0, reach / 4.0, reach / 8.0] {
        let (a, b) = (fp[s] + before * d, fp[e] - after * d);
        let pts: Vec<DVec2> = std::iter::once(b).chain(rest.iter().copied()).chain([a]).collect();
        if simple(&pts) {
            return Some(roof_over(r, pts, (data(own), data(overhangs))));
        }
    }
    // The run moved out square to its chord, as gables with no overhang.
    let out = -(fp[e] - fp[s]).try_normalize()?.perp();
    let on_run = |i: usize| (0..=len).any(|k| (s + k) % n == i);
    let pts: Vec<DVec2> = (0..n).map(|i| if on_run(i) { fp[i] + out * reach } else { fp[i] }).collect();
    let gable = |v: &[Option<f64>]| -> Vec<Option<f64>> {
        (0..n).map(|i| if on_run(i) && i != e { Some(0.0) } else { v.get(i).copied().flatten() }).collect()
    };
    simple(&pts).then(|| roof_over(r, pts, (gable(own), gable(overhangs))))
}

/// Roof `r` (counter-clockwise footprint `fp`, its edge data in that order)
/// with edge `s` moved out by `gap`, the edges beside it running on to it.
fn moved(r: &Roof, fp: &[DVec2], (own, overhangs): (&[Option<f64>], &[Option<f64>]), s: usize, gap: f64) -> Roof {
    let n = fp.len();
    let (e, p) = ((s + 1) % n, (s + n - 1) % n);
    let out = -(fp[e] - fp[s]).normalize_or_zero().perp();
    let before = (fp[s] - fp[p]).normalize_or_zero();
    let after = (fp[(e + 1) % n] - fp[e]).normalize_or_zero();
    let a = if before.dot(out) > 0.2 { fp[s] + before * (gap / before.dot(out)) } else { fp[s] + out * gap };
    let b = if -after.dot(out) > 0.2 { fp[e] - after * (gap / -after.dot(out)) } else { fp[e] + out * gap };
    let mut pts = fp.to_vec();
    pts[s] = a;
    pts[e] = b;
    roof_over(r, pts, (own.to_vec(), overhangs.to_vec()))
}

/// Whether `pts` is a simple counter-clockwise polygon.
fn simple(pts: &[DVec2]) -> bool {
    let n = pts.len();
    if n < 3 || polygon::signed_area(pts) <= 0.0 || pts.iter().any(|p| !p.is_finite()) {
        return false;
    }
    for i in 0..n {
        for j in i + 1..n {
            // Edges beside each other share a point only.
            if j == i + 1 || (i == 0 && j == n - 1) {
                continue;
            }
            if segments_meet(pts[i], pts[(i + 1) % n], pts[j], pts[(j + 1) % n]) {
                return false;
            }
        }
    }
    true
}

/// Whether segments `p`-`q` and `a`-`b` meet (crossing or touching).
fn segments_meet(p: DVec2, q: DVec2, a: DVec2, b: DVec2) -> bool {
    let o = |x: DVec2, y: DVec2, z: DVec2| (y - x).perp_dot(z - x);
    let (d1, d2, d3, d4) = (o(p, q, a), o(p, q, b), o(a, b, p), o(a, b, q));
    if ((d1 > 0.0) != (d2 > 0.0)) && ((d3 > 0.0) != (d4 > 0.0)) && d1 != 0.0 && d2 != 0.0 {
        return true;
    }
    let on = |x: DVec2, y: DVec2, z: DVec2| {
        let len = (y - x).length().max(1e-300);
        o(x, y, z).abs() / len <= 1e-12 * len.max(1.0) && (z - x).dot(z - y) <= 0.0
    };
    on(p, q, a) || on(p, q, b) || on(a, b, p) || on(a, b, q)
}

/// Whether `outline` overlaps any of `domain`; `None` when that could not be worked out.
fn overlaps(outline: &[DVec2], domain: &[Vec<DVec2>]) -> Option<bool> {
    let common = region::covered(domain, Some(outline), MAX_JOIN_POINTS)?;
    Some(common.iter().map(|r| polygon::area(r)).sum::<f64>() > MIN_PIECE)
}

/// The edge of the counter-clockwise footprint `fp` whose middle, looking
/// straight out, sees `domain` nearest, with how far it is.
fn nearest_hit(fp: &[DVec2], domain: &[Vec<DVec2>]) -> Option<(usize, f64)> {
    let n = fp.len();
    let mut best: Option<(usize, f64)> = None;
    for i in 0..n {
        let (a, b) = (fp[i], fp[(i + 1) % n]);
        let Some(dir) = (b - a).try_normalize() else { continue };
        let (m, out) = ((a + b) * 0.5, -dir.perp());
        for ring in domain {
            let k = ring.len();
            for j in 0..k {
                let (c, d) = (ring[j], ring[(j + 1) % k]);
                let e = d - c;
                let den = out.perp_dot(e);
                if den.abs() < 1e-12 {
                    continue;
                }
                let w = c - m;
                let (t, u) = (w.perp_dot(e) / den, w.perp_dot(out) / den);
                if t > NEAR && (-1e-9..=1.0 + 1e-9).contains(&u) && best.is_none_or(|(_, bt)| t < bt) {
                    best = Some((i, t));
                }
            }
        }
    }
    best
}

/// Distance from `p` to the outline of `ring`.
fn ring_distance(ring: &[DVec2], p: DVec2) -> f64 {
    let n = ring.len();
    (0..n)
        .map(|i| polygon::closest_on_segment(ring[i], ring[(i + 1) % n], p).0.distance(p))
        .fold(f64::INFINITY, f64::min)
}

/// The diagonal of the box around `pts`.
fn diagonal(pts: &[DVec2]) -> f64 {
    let b = Bounds::of(pts);
    if b.lo.x > b.hi.x { 0.0 } else { (b.hi - b.lo).length() }
}

/// The piece of `pieces` at `p` (or the nearest within [`NEAR`] of it).
fn piece_at(pieces: &[Piece], p: DVec2) -> Option<&Piece> {
    pieces.iter().find(|q| polygon::contains(&q.plan, p)).or_else(|| {
        pieces
            .iter()
            .map(|q| (ring_distance(&q.plan, p), q))
            .filter(|(d, _)| *d <= NEAR)
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, q)| q)
    })
}

/// A point inside `ring`: the middle of its largest triangle.
fn interior_point(ring: &[DVec2]) -> Option<DVec2> {
    let size = |t: &[usize; 3]| (ring[t[1]] - ring[t[0]]).perp_dot(ring[t[2]] - ring[t[0]]).abs();
    let t = polygon::triangulate(ring).into_iter().max_by(|a, b| size(a).total_cmp(&size(b)))?;
    Some((ring[t[0]] + ring[t[1]] + ring[t[2]]) / 3.0)
}

/// `pieces` less what `cutters` cover (a piece whose cut fails stays whole).
fn cut(pieces: &[Piece], cutters: &[Vec<DVec2>]) -> Vec<Piece> {
    let mut out = Vec::new();
    for p in pieces {
        let b = Bounds::of(&p.plan);
        let near: Vec<Vec<DVec2>> = cutters.iter().filter(|c| b.meets(&Bounds::of(c))).cloned().collect();
        if near.is_empty() {
            out.push(p.clone());
            continue;
        }
        match region::covered_except(std::slice::from_ref(&p.plan), &near, None, MAX_JOIN_POINTS) {
            Some(rings) => out.extend(rings.into_iter().filter(|r| polygon::area(r) > MIN_PIECE).map(|plan| Piece {
                plan,
                c: p.c,
                g: p.g,
            })),
            None => out.push(p.clone()),
        }
    }
    out
}

/// The parts of `lines` over `pieces` (on their edges too).
fn clip_lines(lines: &[(DVec2, DVec2)], pieces: &[Piece]) -> Vec<(DVec2, DVec2)> {
    let mut out = Vec::new();
    for &(a, b) in lines {
        let d = b - a;
        let Some(side) = d.try_normalize().map(|u| u.perp() * NEAR) else { continue };
        // Where the line crosses the pieces' edges.
        let mut ts = vec![0.0, 1.0];
        for p in pieces {
            let k = p.plan.len();
            for j in 0..k {
                let (c, e) = (p.plan[j], p.plan[(j + 1) % k]);
                let den = d.perp_dot(e - c);
                if den.abs() < 1e-12 {
                    continue;
                }
                let w = c - a;
                let (t, u) = (w.perp_dot(e - c) / den, w.perp_dot(d) / den);
                if t > 0.0 && t < 1.0 && (0.0..=1.0).contains(&u) {
                    ts.push(t);
                }
            }
        }
        ts.sort_by(f64::total_cmp);
        let over = |p: DVec2| pieces.iter().any(|q| polygon::contains(&q.plan, p));
        let mut start: Option<f64> = None;
        for w in ts.windows(2) {
            // Crossings at one point (a corner of the pieces) leave nothing between them.
            if (w[1] - w[0]) * d.length() <= LINE_TOL {
                continue;
            }
            let mid = a + d * ((w[0] + w[1]) * 0.5);
            let inside = over(mid + side) || over(mid - side);
            match (inside, start) {
                (true, None) => start = Some(w[0]),
                (false, Some(t0)) => {
                    out.push((a + d * t0, a + d * w[0]));
                    start = None;
                }
                _ => {}
            }
        }
        if let Some(t0) = start {
            out.push((a + d * t0, b));
        }
    }
    out.retain(|(a, b)| a.distance(*b) > LINE_TOL);
    out
}

/// The outline of `pieces` in plan, less the parts along `valleys`.
fn outline_edges(pieces: &[Piece], valleys: &[(DVec3, DVec3)]) -> Vec<(DVec2, DVec2)> {
    let plans: Vec<&[DVec2]> = pieces.iter().map(|p| p.plan.as_slice()).collect();
    let Some(edges) = Edges::of(&plans) else { return Vec::new() };
    let on_valley = |p: DVec2| {
        valleys
            .iter()
            .any(|(a, b)| polygon::closest_on_segment(a.truncate(), b.truncate(), p).0.distance(p) <= LINE_TOL)
    };
    edges.boundary().into_iter().filter(|(_, a, b)| !(on_valley(*a) && on_valley(*b))).map(|(_, a, b)| (a, b)).collect()
}

/// Plan area of `pieces` within the footprints of `roofs` (less their openings).
fn area_within(pieces: &[Piece], roofs: &[&Roof]) -> f64 {
    let footprints: Vec<Vec<DVec2>> = roofs.iter().map(|r| r.footprint.clone()).collect();
    let openings: Vec<Vec<DVec2>> = roofs.iter().flat_map(|r| r.holes.iter().chain(&r.cuts).cloned()).collect();
    pieces
        .iter()
        .map(|p| match region::covered_except(&footprints, &openings, Some(&p.plan), MAX_JOIN_POINTS) {
            Some(rings) => rings.iter().map(|r| polygon::area(r)).sum(),
            None => p.area(),
        })
        .sum()
}

/// Add the closed slab of `pieces`, `thickness` thick (vertically), to
/// `solid`: each piece's top and underside, and a vertical face along every
/// part of their outline no other piece runs back along.
pub fn add_pieces(solid: &mut Solid, pieces: &[Piece], thickness: f64) {
    let pieces: Vec<&Piece> = pieces.iter().filter(|p| p.plan.len() >= 3 && p.area() > MIN_PIECE).collect();
    let up = DVec3::Z * thickness;
    for p in &pieces {
        let normal = DVec3::new(-p.g.x, -p.g.y, 1.0).normalize_or_zero();
        let under: Vec<DVec3> = p.plan.iter().map(|q| q.extend(p.z(*q))).collect();
        let top: Vec<DVec3> = under.iter().map(|v| *v + up).collect();
        solid.faces.push(Face::new(top, normal));
        solid.faces.push(Face::new(under.into_iter().rev().collect(), -normal));
    }
    let plans: Vec<&[DVec2]> = pieces.iter().map(|p| p.plan.as_slice()).collect();
    let Some(edges) = Edges::of(&plans) else { return };
    for (k, a, b) in edges.boundary() {
        let Some(p) = pieces.get(k) else { continue };
        let out = -(b - a).normalize_or_zero().perp();
        let (pa, pb) = (a.extend(p.z(a)), b.extend(p.z(b)));
        solid.faces.push(Face::new(vec![pa, pb, pb + up, pa + up], out.extend(0.0)));
    }
}

/// A part of an edge: from and to, 0 at its start and 1 at its end.
type Span = (f64, f64);

/// A box in plan.
#[derive(Debug, Clone, Copy)]
struct Bounds {
    lo: DVec2,
    hi: DVec2,
}

impl Bounds {
    fn of(pts: &[DVec2]) -> Self {
        let (lo, hi) = pts.iter().fold((DVec2::INFINITY, DVec2::NEG_INFINITY), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
        Self { lo, hi }
    }

    fn meets(&self, o: &Bounds) -> bool {
        self.lo.x <= o.hi.x + NEAR
            && o.lo.x <= self.hi.x + NEAR
            && self.lo.y <= o.hi.y + NEAR
            && o.lo.y <= self.hi.y + NEAR
    }
}

/// The edges of several rings (closed, counter-clockwise), each with the
/// parts of it another edge runs back along: shared by two pieces that
/// meet, walked there and back along a bridge, or meeting another piece's
/// edge in part (a point of one lying on the other's edge).
struct Edges {
    edges: Vec<Edge>,
}

struct Edge {
    ring: usize,
    a: DVec2,
    b: DVec2,
    /// Parts other edges run back along: from and to (0 at `a`, 1 at `b`) and the other's ring.
    covered: Vec<(f64, f64, usize)>,
}

impl Edges {
    /// `None` past [`MAX_EDGES`] edges.
    fn of(rings: &[&[DVec2]]) -> Option<Self> {
        let mut edges = Vec::new();
        for (ring, pts) in rings.iter().enumerate() {
            let n = pts.len();
            for i in 0..n {
                let (a, b) = (pts[i], pts[(i + 1) % n]);
                if a.distance(b) > 1e-12 && a.is_finite() && b.is_finite() {
                    edges.push(Edge { ring, a, b, covered: Vec::new() });
                }
                if edges.len() > MAX_EDGES {
                    return None;
                }
            }
        }
        // Edges by the left of their boxes: each meets only those whose boxes start before it ends.
        let lo = |e: &Edge| e.a.min(e.b);
        let hi = |e: &Edge| e.a.max(e.b);
        let mut order: Vec<usize> = (0..edges.len()).collect();
        order.sort_by(|i, j| lo(&edges[*i]).x.total_cmp(&lo(&edges[*j]).x));
        let mut found: Vec<(usize, Span, usize, Span)> = Vec::new();
        for (k, &i) in order.iter().enumerate() {
            let (li, hi_i) = (lo(&edges[i]), hi(&edges[i]));
            for &j in &order[k + 1..] {
                let (lj, hj) = (lo(&edges[j]), hi(&edges[j]));
                if lj.x > hi_i.x + LINE_TOL {
                    break;
                }
                if lj.y > hi_i.y + LINE_TOL || hj.y < li.y - LINE_TOL {
                    continue;
                }
                if let Some((ti, tj)) = back_along(&edges[i], &edges[j]) {
                    found.push((i, ti, j, tj));
                }
            }
        }
        for (i, ti, j, tj) in found {
            let (ri, rj) = (edges[i].ring, edges[j].ring);
            edges[i].covered.push((ti.0, ti.1, rj));
            edges[j].covered.push((tj.0, tj.1, ri));
        }
        Some(Self { edges })
    }

    /// The parts of the edges no other edge runs back along: the outline
    /// of what the rings cover together, as `(ring, from, to)`.
    fn boundary(&self) -> Vec<(usize, DVec2, DVec2)> {
        let mut out = Vec::new();
        for e in &self.edges {
            let d = e.b - e.a;
            let len = d.length();
            let mut covered: Vec<(f64, f64)> = e.covered.iter().map(|(a, b, _)| (*a, *b)).collect();
            covered.sort_by(|x, y| x.0.total_cmp(&y.0));
            let mut t = 0.0;
            for (lo, hi) in covered {
                if (lo - t) * len > LINE_TOL {
                    out.push((e.ring, e.a + d * t, e.a + d * lo));
                }
                t = f64::max(t, hi);
            }
            if (1.0 - t) * len > LINE_TOL {
                out.push((e.ring, e.a + d * t, e.b));
            }
        }
        out
    }

    /// The group of rings each ring belongs to: rings sharing a part of an
    /// edge are in one group (an index of one of them).
    fn groups(&self, rings: usize) -> Vec<usize> {
        let mut parent: Vec<usize> = (0..rings).collect();
        fn root(parent: &mut [usize], mut i: usize) -> usize {
            // Halving the path on the way: each step moves closer to the root.
            while let Some(&p) = parent.get(i) {
                if p == i {
                    break;
                }
                let up = parent.get(p).copied().unwrap_or(p);
                if let Some(slot) = parent.get_mut(i) {
                    *slot = up;
                }
                i = p;
            }
            i
        }
        for e in &self.edges {
            for &(_, _, other) in &e.covered {
                let (a, b) = (root(&mut parent, e.ring), root(&mut parent, other));
                if a != b
                    && let Some(slot) = parent.get_mut(a)
                {
                    *slot = b;
                }
            }
        }
        (0..rings).map(|i| root(&mut parent, i)).collect()
    }
}

/// Where edge `f` runs back along edge `e` (collinear, the other way):
/// that part as parameters along each, `None` when it does not (or only
/// at a point).
fn back_along(e: &Edge, f: &Edge) -> Option<(Span, Span)> {
    let (d, df) = (e.b - e.a, f.b - f.a);
    let (len, len_f) = (d.length(), df.length());
    let u = d / len;
    if u.dot(df) >= 0.0
        || u.perp_dot(f.a - e.a).abs() > LINE_TOL
        || u.perp_dot(f.b - e.a).abs() > LINE_TOL
        || (df / len_f).perp_dot(e.a - f.a).abs() > LINE_TOL
        || (df / len_f).perp_dot(e.b - f.a).abs() > LINE_TOL
    {
        return None;
    }
    let along = |p: DVec2| (p - e.a).dot(u) / len;
    let (ta, tb) = (along(f.a), along(f.b));
    let (lo, hi) = (ta.min(tb).max(0.0), ta.max(tb).min(1.0));
    if (hi - lo) * len <= LINE_TOL {
        return None;
    }
    let on_f = |t: f64| ((e.a + d * t) - f.a).dot(df) / (len_f * len_f);
    let (fa, fb) = (on_f(lo), on_f(hi));
    Some(((lo, hi), (fa.min(fb).clamp(0.0, 1.0), fa.max(fb).clamp(0.0, 1.0))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ModelGeometry;
    use cc_core::{Category, Element};

    fn pts(c: &[(f64, f64)]) -> Vec<DVec2> {
        c.iter().map(|&(x, y)| DVec2::new(x, y)).collect()
    }

    fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<DVec2> {
        pts(&[(x0, y0), (x1, y0), (x1, y1), (x0, y1)])
    }

    /// A project with roofs of the template's roof type on its first level.
    struct Model {
        p: Project,
    }

    impl Model {
        fn new() -> Self {
            Self { p: Project::from_template() }
        }

        fn add(&mut self, roof: Roof) -> ElementId {
            let id = ElementId(self.p.alloc_id());
            let e = Element {
                id,
                level: self.p.levels[0].id,
                type_id: self.p.default_type(Category::Roof).unwrap(),
                mark: String::new(),
                comments: String::new(),
                kind: ElementKind::Roof(roof),
                uid: Default::default(),
                phase_created: Default::default(),
                phase_demolished: None,
                design_option: None,
                paints: Vec::new(),
            };
            self.p.elements.insert(id, e);
            id
        }

        /// A gable roof (slope 0.5) at `base`, its ridge along `ridge_deg`.
        fn gable(
            &mut self,
            fp: Vec<DVec2>,
            base: f64,
            ridge_deg: f64,
            overhang: f64,
            to: Option<ElementId>,
        ) -> ElementId {
            let mut r = Roof::new(fp, base, overhang, RoofShape::Gable { slope: 0.5, ridge_deg });
            r.join_to = to;
            self.add(r)
        }

        fn hip(&mut self, fp: Vec<DVec2>, base: f64, to: Option<ElementId>) -> ElementId {
            let mut r = Roof::new(fp, base, 0.0, RoofShape::Hip { slope: 0.5 });
            r.join_to = to;
            self.add(r)
        }

        /// The vertical thickness of the type's slab at slope 0.5.
        fn t_v(&self) -> f64 {
            let t = self.p.element_type(self.p.default_type(Category::Roof).unwrap()).unwrap().thickness();
            t * 1.25f64.sqrt()
        }

        fn build(&self) -> ModelGeometry {
            ModelGeometry::build(&self.p)
        }
    }

    fn join(g: &ModelGeometry, id: ElementId) -> &RoofJoinGeom {
        g.roof_joins.get(&id).unwrap_or_else(|| panic!("no join for {id:?}: {:?}", g.roof_joins))
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    /// The valleys in plan, each as its two ends.
    fn valley_ends(j: &RoofJoinGeom) -> Vec<(DVec2, DVec2)> {
        j.valleys.iter().map(|(a, b)| (a.truncate(), b.truncate())).collect()
    }

    /// Whether `j` has a valley from `a` to `b` (either way).
    fn has_valley(j: &RoofJoinGeom, a: (f64, f64), b: (f64, f64)) -> bool {
        let (a, b) = (DVec2::new(a.0, a.1), DVec2::new(b.0, b.1));
        valley_ends(j).iter().any(|(p, q)| {
            (p.distance(a) < 1e-6 && q.distance(b) < 1e-6) || (p.distance(b) < 1e-6 && q.distance(a) < 1e-6)
        })
    }

    /// The plan areas, and the solids' volumes the plan areas times the vertical thickness.
    fn check_areas(m: &Model, g: &ModelGeometry, want: &[(ElementId, f64)]) {
        for &(id, area) in want {
            let j = join(g, id);
            assert!(close(j.plan_area, area), "{id:?}: plan area {} against {area}", j.plan_area);
            let v = g.volume(id);
            assert!((v - area * m.t_v()).abs() < 1e-6, "{id:?}: volume {v} against {}", area * m.t_v());
        }
    }

    /// The main roof of the tests: a gable roof 10 × 8, ridge along X at y = 4.
    fn main_roof(m: &mut Model, overhang: f64) -> ElementId {
        m.gable(rect(0.0, 0.0, 10.0, 8.0), 3.0, 0.0, overhang, None)
    }

    /// A gable cross wing 4 wide from x 3 to 7, meeting the main roof's
    /// south eave: valleys from the wing's eaves up to (5, 2), 3 m long; the
    /// wing's ridge runs on to (5, 2); the main roof loses the triangle
    /// between them. Its framing covers the wing's true area.
    #[test]
    fn a_gable_wing_meets_the_main_roof_in_two_valleys() {
        let mut m = Model::new();
        let main = main_roof(&mut m, 0.0);
        let wing = m.gable(rect(3.0, -6.0, 7.0, 0.0), 3.0, 90.0, 0.0, Some(main));
        let g = m.build();
        let j = join(&g, wing);
        assert_eq!(j.state, Some(JoinState::Joined));
        assert_eq!(j.valleys.len(), 2, "{:?}", j.valleys);
        assert!(has_valley(j, (3.0, 0.0), (5.0, 2.0)) && has_valley(j, (5.0, 2.0), (7.0, 0.0)), "{:?}", j.valleys);
        for (a, b) in &j.valleys {
            assert!(close(a.distance(*b), 3.0), "{a} {b}");
        }
        assert!(close(j.valley_length(), 6.0));
        // The wing's ridge, at its underside 4, ends on the valleys.
        let ridges = &g.roof_ridges[&wing];
        let ridge = ridges.iter().find(|(a, b)| close(a.x, 5.0) && close(b.x, 5.0)).expect("a ridge along x = 5");
        let (lo, hi) = (ridge.0.y.min(ridge.1.y), ridge.0.y.max(ridge.1.y));
        assert!(close(lo, -6.0) && close(hi, 2.0), "{ridges:?}");
        let underside = g
            .body_of(wing)
            .flat_map(|s| s.faces.iter().filter(|f| f.normal.z < -0.1).flat_map(|f| f.verts.iter().map(|v| v.z)))
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(close(underside, 4.0), "{underside}");
        check_areas(&m, &g, &[(wing, 28.0), (main, 76.0)]);
        assert_eq!(join(&g, main).cut_by, vec![wing]);
        assert!(close(join(&g, main).area, 76.0) && close(j.area, 28.0));
        let framing = &g.roof_framing[&wing];
        assert!(close(framing.sloped_area, 28.0 * 1.25f64.sqrt()), "{}", framing.sloped_area);
        assert!(close(j.sloped_area, 28.0 * 1.25f64.sqrt()));
        // The main roof's outline loses the triangle, its valleys drawn as ridges.
        let mr = join(&g, main);
        assert!(mr.outline_edges.iter().all(|(a, b)| !(a.y > 1e-9 && b.y > 1e-9 && a.y < 4.0 && b.y < 4.0)));
        assert!(g.roof_ridges[&main].len() >= 3);
    }

    /// With overhangs of 0.5 on both roofs the valleys start at the wing's
    /// eave tips, on the main roof's eave line.
    #[test]
    fn overhangs_move_the_valleys_out() {
        let mut m = Model::new();
        let main = main_roof(&mut m, 0.5);
        let wing = m.gable(rect(3.0, -6.0, 7.0, 0.0), 3.0, 90.0, 0.5, Some(main));
        let g = m.build();
        let j = join(&g, wing);
        assert_eq!(j.state, Some(JoinState::Joined));
        assert!(has_valley(j, (2.5, -0.5), (5.0, 2.0)) && has_valley(j, (5.0, 2.0), (7.5, -0.5)), "{:?}", j.valleys);
        check_areas(&m, &g, &[(wing, 36.25), (main, 92.75)]);
    }

    /// A wing lower than the main roof meets it higher up its slope (its
    /// eaves against the main roof's fascia); one higher runs on level
    /// until the main roof rises to its eaves.
    #[test]
    fn a_wing_lower_or_higher_than_the_main_roof() {
        let mut m = Model::new();
        let main = main_roof(&mut m, 0.0);
        let wing = m.gable(rect(3.0, -6.0, 7.0, 0.0), 2.5, 90.0, 0.0, Some(main));
        let g = m.build();
        let j = join(&g, wing);
        assert!(has_valley(j, (4.0, 0.0), (5.0, 1.0)) && has_valley(j, (5.0, 1.0), (6.0, 0.0)), "{:?}", j.valleys);
        check_areas(&m, &g, &[(wing, 25.0), (main, 79.0)]);

        let mut m = Model::new();
        let main = main_roof(&mut m, 0.0);
        let wing = m.gable(rect(3.0, -6.0, 7.0, 0.0), 3.5, 90.0, 0.0, Some(main));
        let g = m.build();
        let j = join(&g, wing);
        assert!(has_valley(j, (3.0, 1.0), (5.0, 3.0)) && has_valley(j, (5.0, 3.0), (7.0, 1.0)), "{:?}", j.valleys);
        check_areas(&m, &g, &[(wing, 32.0), (main, 72.0)]);
        let ridge = g.roof_ridges[&wing].iter().find(|(a, b)| close(a.x, 5.0) && close(b.x, 5.0)).copied().unwrap();
        assert!(close(ridge.0.y.max(ridge.1.y), 3.0), "{ridge:?}");
    }

    /// A hip main roof meets the wing on its south plane as the gable did;
    /// a hip wing gets a gable at its joined end, its hips at the other.
    #[test]
    fn hip_roofs_join_like_gables() {
        let mut m = Model::new();
        let main = m.hip(rect(0.0, 0.0, 10.0, 8.0), 3.0, None);
        let wing = m.gable(rect(3.0, -6.0, 7.0, 0.0), 3.0, 90.0, 0.0, Some(main));
        let g = m.build();
        let j = join(&g, wing);
        assert!(has_valley(j, (3.0, 0.0), (5.0, 2.0)) && has_valley(j, (5.0, 2.0), (7.0, 0.0)), "{:?}", j.valleys);
        check_areas(&m, &g, &[(wing, 28.0), (main, 76.0)]);

        let mut m = Model::new();
        let main = main_roof(&mut m, 0.0);
        let wing = m.hip(rect(3.0, -6.0, 7.0, 0.0), 3.0, Some(main));
        let g = m.build();
        let j = join(&g, wing);
        assert!(has_valley(j, (3.0, 0.0), (5.0, 2.0)) && has_valley(j, (5.0, 2.0), (7.0, 0.0)), "{:?}", j.valleys);
        check_areas(&m, &g, &[(wing, 28.0), (main, 76.0)]);
        let ridge = g.roof_ridges[&wing].iter().find(|(a, b)| close(a.x, 5.0) && close(b.x, 5.0)).copied();
        let ridge = ridge.unwrap_or_else(|| panic!("{:?}", g.roof_ridges[&wing]));
        let (lo, hi) = (ridge.0.y.min(ridge.1.y), ridge.0.y.max(ridge.1.y));
        assert!(close(lo, -4.0) && close(hi, 2.0), "{ridge:?}");
    }

    /// Two wings on one main roof, a bay joined to a wing (a chain), and a
    /// wing standing a meter off the main roof that reaches over the gap.
    #[test]
    fn several_wings_chains_and_gaps() {
        let mut m = Model::new();
        let main = main_roof(&mut m, 0.0);
        let south = m.gable(rect(3.0, -6.0, 7.0, 0.0), 3.0, 90.0, 0.0, Some(main));
        let north = m.gable(rect(3.0, 8.0, 7.0, 14.0), 3.0, 90.0, 0.0, Some(main));
        let g = m.build();
        assert!(has_valley(join(&g, north), (3.0, 8.0), (5.0, 6.0)), "{:?}", join(&g, north).valleys);
        check_areas(&m, &g, &[(south, 28.0), (north, 28.0), (main, 72.0)]);
        let mut cut_by = join(&g, main).cut_by.clone();
        cut_by.sort();
        assert_eq!(cut_by, vec![south, north]);

        // A bay on the wing's west side, its ridge along X.
        let mut m = Model::new();
        let main = main_roof(&mut m, 0.0);
        let wing = m.gable(rect(3.0, -6.0, 7.0, 0.0), 3.0, 90.0, 0.0, Some(main));
        let bay = m.gable(rect(0.0, -4.0, 3.0, -2.0), 3.0, 0.0, 0.0, Some(wing));
        let g = m.build();
        let j = join(&g, bay);
        assert_eq!(j.state, Some(JoinState::Joined));
        assert!(has_valley(j, (3.0, -4.0), (4.0, -3.0)) && has_valley(j, (4.0, -3.0), (3.0, -2.0)), "{:?}", j.valleys);
        check_areas(&m, &g, &[(bay, 7.0), (wing, 27.0), (main, 76.0)]);
        assert_eq!(join(&g, wing).cut_by, vec![bay]);

        // A wing a meter short of the main roof reaches over the gap.
        let mut m = Model::new();
        let main = main_roof(&mut m, 0.0);
        let wing = m.gable(rect(3.0, -7.0, 7.0, -1.0), 3.0, 90.0, 0.0, Some(main));
        let g = m.build();
        assert_eq!(join(&g, wing).state, Some(JoinState::Joined));
        check_areas(&m, &g, &[(wing, 32.0), (main, 76.0)]);
    }

    /// Roofs that do not meet: a wing far off, one too low to reach the
    /// main roof's surface. Both are built as drawn.
    #[test]
    fn roofs_apart_are_built_as_drawn() {
        for (y0, base) in [(-20.0, 3.0), (-6.0, -1.0)] {
            let mut m = Model::new();
            let main = main_roof(&mut m, 0.0);
            let wing = m.gable(rect(3.0, y0, 7.0, y0 + 6.0), base, 90.0, 0.0, Some(main));
            let g = m.build();
            let j = join(&g, wing);
            assert_eq!(j.state, Some(JoinState::Apart), "{y0}");
            assert!(!j.active() && !g.roof_joins.get(&main).is_some_and(|j| j.active()));
            assert!((g.volume(wing) - 24.0 * m.t_v()).abs() < 1e-6);
            assert!((g.volume(main) - 80.0 * m.t_v()).abs() < 1e-6);
        }
    }

    /// Missing targets and loops leave the roofs as drawn; the preview
    /// tells a join that would work from one that would not.
    #[test]
    fn missing_targets_loops_and_previews() {
        let mut m = Model::new();
        let main = main_roof(&mut m, 0.0);
        let wing = m.gable(rect(3.0, -6.0, 7.0, 0.0), 3.0, 90.0, 0.0, Some(ElementId(987_654)));
        let g = m.build();
        assert_eq!(join(&g, wing).state, Some(JoinState::Missing));
        assert!((g.volume(wing) - 24.0 * m.t_v()).abs() < 1e-6);
        assert_eq!(preview(&m.p, wing, main), JoinState::Joined);
        let far = m.gable(rect(30.0, 0.0, 34.0, 6.0), 3.0, 90.0, 0.0, None);
        assert_eq!(preview(&m.p, wing, far), JoinState::Apart);
        // A loop (as a crafted file might hold it).
        if let Some(ElementKind::Roof(r)) = m.p.elements.get_mut(&wing).map(|e| &mut e.kind) {
            r.join_to = Some(main);
        }
        if let Some(ElementKind::Roof(r)) = m.p.elements.get_mut(&main).map(|e| &mut e.kind) {
            r.join_to = Some(wing);
        }
        let g = m.build();
        assert_eq!(join(&g, wing).state, Some(JoinState::Cycle));
        assert_eq!(join(&g, main).state, Some(JoinState::Cycle));
        assert!((g.volume(main) - 80.0 * m.t_v()).abs() < 1e-6);
    }

    /// Unjoined roofs build as before: the shared builder gives the same
    /// slab as the pieces do.
    #[test]
    fn pieces_make_the_same_slab_as_the_roof() {
        let shapes = [
            RoofShape::Gable { slope: 0.5, ridge_deg: 0.0 },
            RoofShape::Hip { slope: 0.7 },
            RoofShape::Shed { slope: 0.2, direction_deg: 30.0 },
            RoofShape::Flat,
        ];
        let l = pts(&[(0.0, 0.0), (10.0, 0.0), (10.0, 4.0), (4.0, 4.0), (4.0, 10.0), (0.0, 10.0)]);
        for shape in shapes {
            let mut r = Roof::new(l.clone(), 3.0, 0.4, shape);
            r.holes = vec![rect(1.0, 6.0, 2.5, 8.0)];
            let body = crate::roofs::body(&r, 0.0, 0.2);
            let mut a = Solid::new(ElementId(1), Category::Roof, [0; 3]);
            body.add_to(&mut a);
            let mut b = Solid::new(ElementId(1), Category::Roof, [0; 3]);
            add_pieces(&mut b, &body.pieces(), body.thickness());
            assert!((a.volume() - b.volume()).abs() < 1e-6, "{shape:?}: {} against {}", a.volume(), b.volume());
            let area: f64 = body.pieces().iter().map(Piece::area).sum();
            assert!((area - crate::roofs::plan_area(&r)).abs() < 1e-6, "{shape:?}: {area}");
        }
    }
}
