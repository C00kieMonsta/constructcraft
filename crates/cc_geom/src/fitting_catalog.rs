//! Fittings matched to the catalog of their pipes' series.
//!
//! A derived fitting ([`crate::mep_net::Fitting`]) whose branches are all
//! pipes of one [`PipeSeries`] is matched to the fitting rows sold for that
//! series ([`CatalogFitting`]):
//!
//! - **Matched:** one row. An elbow takes the nearest catalog angle within
//!   [`ANGLE_TOL`]; a tee or wye matches by main and branch size; a
//!   coupling, reducer or cross by its sizes.
//! - **Composite:** a reducing tee that is not sold is a tee plus reducers on
//!   the branch, a tee whose run ends differ gets reducers on the smaller
//!   end, and a reducer that skips sizes is a chain of reducers (at most
//!   [`MAX_CHAIN`] on one branch). Each piece is its own row.
//! - **Missing:** no row fits (a 60° elbow, a size the series lacks…), with a
//!   [`MissingReason`].
//!
//! The pieces carry their bodies (a cylinder per branch from the center to
//! the socket's mouth) and each branch the distance from the center to where
//! its pipe ends (the laying length), which shortens the pipe's cut length.
//! Dimensions no source gives come from rules on the outside diameter
//! ([`rule_socket`], [`rule_z`], [`RULE_BODY`]).

use cc_core::{CatalogFitting, ElementId, FittingRowKind, PipeSeries, PipeSeriesId, PipeSize, Project};
use glam::DVec3;

/// How far an elbow's (or a wye branch's) angle may be from a catalog angle, degrees.
pub const ANGLE_TOL: f64 = 7.5;
/// Most reducers chained on one branch.
pub const MAX_CHAIN: usize = 3;
/// A body's diameter by rule: the outside diameter times this.
pub const RULE_BODY: f64 = 1.2;
/// Two outside diameters this close are the same size (m).
const SAME: f64 = 1e-6;

/// Socket depth by rule.
pub fn rule_socket(de: f64) -> f64 {
    de * 0.8
}

/// Laying length by rule: half the outside diameter at a tee, a wye or a
/// cross, and at an elbow of `angle` the same scaled by tan(angle/2) (a
/// 90° elbow gives de/2); a coupling's ends nearly touch, a reducer is a
/// quarter diameter long on each side.
pub fn rule_z(kind: FittingRowKind, angle: Option<f64>, de: f64) -> f64 {
    match kind {
        FittingRowKind::Elbow => {
            let a = angle.unwrap_or(90.0).clamp(1.0, 179.0).to_radians();
            (de * 0.5 * (a * 0.5).tan()).clamp(de * 0.1, de * 2.0)
        }
        FittingRowKind::Tee | FittingRowKind::Wye | FittingRowKind::Cross => de * 0.5,
        FittingRowKind::Coupling => de * 0.02,
        FittingRowKind::Reducer => de * 0.25,
    }
}

/// Why a fitting has no catalog row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissingReason {
    /// The series sells this kind of fitting in this size, but not at this angle.
    Angle,
    /// The series sells no fitting of this kind in this size.
    Size,
    /// The series has no fitting (or chain of reducers) between these sizes.
    Sizes,
}

impl MissingReason {
    /// Translation key of the reason.
    pub fn key(self) -> &'static str {
        match self {
            MissingReason::Angle => "fitting-missing-angle",
            MissingReason::Size => "fitting-missing-size",
            MissingReason::Sizes => "fitting-missing-sizes",
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            MissingReason::Angle => "angle",
            MissingReason::Size => "size",
            MissingReason::Sizes => "sizes",
        }
    }
}

/// How a fitting matched its series' catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchStatus {
    Matched,
    Composite,
    Missing(MissingReason),
}

impl MatchStatus {
    pub fn code(self) -> &'static str {
        match self {
            MatchStatus::Matched => "matched",
            MatchStatus::Composite => "composite",
            MatchStatus::Missing(_) => "missing",
        }
    }
}

/// A cylinder of a fitting's body, in world coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Body {
    pub a: DVec3,
    pub b: DVec3,
    pub radius: f64,
}

/// One catalog fitting of a matched fitting.
#[derive(Debug, Clone, PartialEq)]
pub struct Piece {
    /// Index of the row in the series' `fittings`.
    pub row: usize,
    /// Where the piece is: the fitting's center (the main piece) or, for a
    /// reducer on a branch, its large end on the branch.
    pub at: DVec3,
    /// The piece's body.
    pub bodies: Vec<Body>,
}

/// A fitting's catalog match.
#[derive(Debug, Clone, PartialEq)]
pub struct FittingMatch {
    pub series: PipeSeriesId,
    pub status: MatchStatus,
    /// The main piece first, then the reducers; empty when missing.
    pub pieces: Vec<Piece>,
    /// Per branch of the fitting: the distance from its center to where the
    /// branch's pipe ends (0 when missing).
    pub laying: Vec<f64>,
}

impl FittingMatch {
    pub fn is_missing(&self) -> bool {
        matches!(self.status, MatchStatus::Missing(_))
    }

    /// The main row (the first piece's).
    pub fn main_row<'a>(&self, series: &'a PipeSeries) -> Option<&'a CatalogFitting> {
        self.pieces.first().and_then(|p| series.fittings.get(p.row))
    }
}

/// English name of a fitting row kind (IFC and other exports without a language).
pub fn english_word(kind: FittingRowKind) -> &'static str {
    match kind {
        FittingRowKind::Elbow => "Elbow",
        FittingRowKind::Tee => "Tee",
        FittingRowKind::Wye => "Wye",
        FittingRowKind::Cross => "Cross",
        FittingRowKind::Coupling => "Coupling",
        FittingRowKind::Reducer => "Reducer",
    }
}

/// Translation key of a fitting row kind (`fitting-elbow`…).
pub fn kind_key(kind: FittingRowKind) -> &'static str {
    match kind {
        FittingRowKind::Elbow => "fitting-elbow",
        FittingRowKind::Tee => "fitting-tee",
        FittingRowKind::Wye => "fitting-wye",
        FittingRowKind::Cross => "fitting-cross",
        FittingRowKind::Coupling => "fitting-coupling",
        FittingRowKind::Reducer => "fitting-reducer",
    }
}

fn angle_text(a: f64) -> String {
    if (a - a.round()).abs() < 1e-6 { format!("{}°", a.round() as i64) } else { format!("{a:.1}°") }
}

/// The name of a catalog row: `word`, the angle (elbows and wyes), the
/// series' short name (else its name) and the sizes, as
/// "Joelho 90° PVC soldável 25 mm" or "Tee PVC Sch 40 1\" × 3/4\"".
pub fn fitting_name(word: &str, row: &CatalogFitting, series: &PipeSeries) -> String {
    let mut parts = vec![word.to_string()];
    if let Some(a) = row.angle.filter(|_| matches!(row.kind, FittingRowKind::Elbow | FittingRowKind::Wye)) {
        parts.push(angle_text(a));
    }
    let short = if series.short.trim().is_empty() { &series.name } else { &series.short };
    parts.push(short.clone());
    match &row.size2 {
        Some(s2) if *s2 != row.size => parts.push(format!("{} × {s2}", row.size)),
        _ => parts.push(row.size.clone()),
    }
    parts.join(" ")
}

/// Which end of a row a branch takes.
#[derive(Debug, Clone, Copy, PartialEq)]
enum End {
    /// An end of the main size (`z`, `socket`).
    Main,
    /// The main run's second end (`z3`, else `z`).
    Main2,
    /// The branch, or a reducer's `size2` end (`z2`, `socket2`).
    Second,
}

/// Laying length, socket depth and body radius of a row's end, of outside
/// diameter `de` (rules where the row has no value).
fn dims(row: &CatalogFitting, end: End, de: f64, de_main: f64) -> (f64, f64, f64) {
    let rule = rule_z(row.kind, row.angle, de_main.max(de));
    let (z, socket) = match end {
        End::Main => (row.z, row.socket),
        End::Main2 => (row.z3.or(row.z), row.socket),
        End::Second => (row.z2.or(row.z), row.socket2.or(row.socket.filter(|_| (de - de_main).abs() < SAME))),
    };
    // A reducer's `z` is the gap between its pipe ends: half on each side
    // unless `z2` gives the small side's.
    let z = match (row.kind, row.z2) {
        (FittingRowKind::Reducer, None) => row.z.map(|z| z * 0.5),
        (FittingRowKind::Reducer, Some(z2)) if end == End::Second => Some(z2),
        _ => z,
    };
    let radius = match row.od {
        Some(od) if de_main > 0.0 => od * 0.5 * (de / de_main).min(1.0),
        _ => de * RULE_BODY * 0.5,
    };
    (z.unwrap_or(rule), socket.unwrap_or_else(|| rule_socket(de)), radius.max(de * 0.5))
}

/// A branch of the fitting being matched.
#[derive(Debug, Clone, Copy)]
struct Branch<'a> {
    dir: DVec3,
    row: &'a PipeSize,
}

/// The plan of a match before geometry: the main row and, per branch, the
/// end it takes and the reducers after it.
struct Plan {
    main: usize,
    ends: Vec<(End, f64)>,
    chains: Vec<Vec<usize>>,
}

fn de_of(series: &PipeSeries, label: &str) -> Option<f64> {
    series.row(label).map(|p| p.de)
}

/// The shortest chain of reducer rows (at most [`MAX_CHAIN`]) from size
/// `from` down to `to`, each step to a smaller size not below `to`.
/// `Some(empty)` when they are the same size.
fn reducer_chain(series: &PipeSeries, from: &str, to: &str) -> Option<Vec<usize>> {
    if from == to {
        return Some(Vec::new());
    }
    let (de_from, de_to) = (de_of(series, from)?, de_of(series, to)?);
    if de_to > de_from {
        return None;
    }
    // Reducer rows as (large, small, row), within the range.
    let steps: Vec<(&str, &str, usize)> = series
        .fittings
        .iter()
        .enumerate()
        .filter(|(_, f)| f.kind == FittingRowKind::Reducer)
        .filter_map(|(i, f)| {
            let (a, b) = (f.size.as_str(), f.size2.as_deref()?);
            let (da, db) = (de_of(series, a)?, de_of(series, b)?);
            let (large, small, dl, ds) = if da >= db { (a, b, da, db) } else { (b, a, db, da) };
            (dl <= de_from + SAME && ds >= de_to - SAME && dl - ds > SAME).then_some((large, small, i))
        })
        .collect();
    // Breadth first: the fewest pieces, the first rows on ties.
    let mut frontier: Vec<(&str, Vec<usize>)> = vec![(from, Vec::new())];
    for _ in 0..MAX_CHAIN {
        let mut next = Vec::new();
        for (at, path) in &frontier {
            for &(large, small, i) in &steps {
                if large != *at {
                    continue;
                }
                let mut p = path.clone();
                p.push(i);
                if small == to {
                    return Some(p);
                }
                if !next.iter().any(|(s, _): &(&str, Vec<usize>)| *s == small) {
                    next.push((small, p));
                }
            }
        }
        if next.is_empty() {
            return None;
        }
        frontier = next;
    }
    None
}

/// The nearest row of `kind` and main size `size` whose angle is within
/// [`ANGLE_TOL`] of `angle` (rows without an angle match any), with a
/// branch of `size2` (None: equal). Err tells whether the size exists at all.
fn find_row(
    series: &PipeSeries,
    kind: FittingRowKind,
    size: &str,
    size2: Option<&str>,
    angle: Option<f64>,
) -> Result<usize, MissingReason> {
    let mut any_size = false;
    let mut best: Option<(f64, usize)> = None;
    for (i, f) in series.fittings.iter().enumerate() {
        if f.kind != kind || f.size != size {
            continue;
        }
        let branch = f.size2.as_deref().filter(|s| *s != f.size);
        if branch != size2.filter(|s| *s != size) {
            continue;
        }
        any_size = true;
        let off = match (angle, f.angle) {
            (Some(a), Some(b)) => (a - b).abs(),
            _ => 0.0,
        };
        if off <= ANGLE_TOL + 1e-9 && best.is_none_or(|(o, _)| off < o) {
            best = Some((off, i));
        }
    }
    match best {
        Some((_, i)) => Ok(i),
        None if any_size => Err(MissingReason::Angle),
        None => Err(MissingReason::Size),
    }
}

/// Whether the series sells `kind` in main size `size` at all.
fn sells(series: &PipeSeries, kind: FittingRowKind, size: &str) -> bool {
    series.fittings.iter().any(|f| f.kind == kind && f.size == size)
}

/// Elbows: one size on both branches, the nearest catalog angle.
fn plan_elbow(series: &PipeSeries, b: &[Branch], angle: f64) -> Result<Plan, MissingReason> {
    if (b[0].row.de - b[1].row.de).abs() > SAME {
        return Err(MissingReason::Sizes);
    }
    let main = find_row(series, FittingRowKind::Elbow, &b[0].row.size, None, Some(angle))?;
    Ok(Plan { main, ends: vec![(End::Main, b[0].row.de); 2], chains: vec![Vec::new(), Vec::new()] })
}

/// Straight runs: a coupling of one size, else a reducer (or a chain of them).
fn plan_straight(series: &PipeSeries, b: &[Branch]) -> Result<Plan, MissingReason> {
    if (b[0].row.de - b[1].row.de).abs() <= SAME {
        let main = find_row(series, FittingRowKind::Coupling, &b[0].row.size, None, None)?;
        return Ok(Plan { main, ends: vec![(End::Main, b[0].row.de); 2], chains: vec![Vec::new(), Vec::new()] });
    }
    let (large, small) = if b[0].row.de > b[1].row.de { (0, 1) } else { (1, 0) };
    let mut chain = reducer_chain(series, &b[large].row.size, &b[small].row.size).ok_or(MissingReason::Sizes)?;
    if chain.is_empty() {
        return Err(MissingReason::Sizes);
    }
    let main = chain.remove(0);
    let row = &series.fittings[main];
    // The first reducer's ends: the large one on the large branch.
    let (large_label, small_label) =
        match de_of(series, &row.size).zip(row.size2.as_deref().and_then(|s| de_of(series, s))) {
            Some((d1, d2)) if d1 < d2 => (row.size2.clone().unwrap_or_default(), row.size.clone()),
            _ => (row.size.clone(), row.size2.clone().unwrap_or_default()),
        };
    let end_of = |label: &str| if *label == row.size { End::Main } else { End::Second };
    let mut ends = vec![(End::Main, 0.0); 2];
    ends[large] = (end_of(&large_label), de_of(series, &large_label).unwrap_or(b[large].row.de));
    ends[small] = (end_of(&small_label), de_of(series, &small_label).unwrap_or(b[small].row.de));
    let mut chains = vec![Vec::new(), Vec::new()];
    chains[small] = chain;
    Ok(Plan { main, ends, chains })
}

/// Tees and wyes: the main run is the most opposite pair of branches, the
/// third is the branch. The main row has the larger run size and a branch
/// size from the actual one up; reducers make up the differences.
fn plan_tee(series: &PipeSeries, b: &[Branch], kind: FittingRowKind) -> Result<Plan, MissingReason> {
    let pairs = [(0, 1, 2), (0, 2, 1), (1, 2, 0)];
    let Some(&(i, j, k)) =
        pairs.iter().min_by(|x, y| b[x.0].dir.dot(b[x.1].dir).total_cmp(&b[y.0].dir.dot(b[y.1].dir)))
    else {
        return Err(MissingReason::Size);
    };
    // The outlet end of the run: the one the branch points away from.
    let (outlet, inlet) = if b[k].dir.dot(b[i].dir) <= b[k].dir.dot(b[j].dir) { (i, j) } else { (j, i) };
    let angle = b[k].dir.angle_between(b[inlet].dir).to_degrees();
    let angle = (kind == FittingRowKind::Wye).then_some(angle);
    let (big, other) = if b[outlet].row.de + SAME >= b[inlet].row.de { (outlet, inlet) } else { (inlet, outlet) };
    let main_size = &b[big].row.size;
    if b[k].row.de > b[big].row.de + SAME {
        return Err(MissingReason::Sizes);
    }
    if !sells(series, kind, main_size) {
        return Err(MissingReason::Size);
    }
    let run_chain = reducer_chain(series, main_size, &b[other].row.size).ok_or(MissingReason::Sizes)?;
    // Branch sizes of the rows of this main size, from the actual one up.
    let mut candidates: Vec<&PipeSize> = series.pipes.iter().filter(|p| p.de + SAME >= b[k].row.de).collect();
    candidates.sort_by(|x, y| x.de.total_cmp(&y.de));
    let mut best: Option<(usize, usize, Vec<usize>)> = None;
    let mut angle_missing = false;
    for c in candidates.into_iter().filter(|c| c.de <= b[big].row.de + SAME) {
        let size2 = (c.size != *main_size).then_some(c.size.as_str());
        let row = match find_row(series, kind, main_size, size2, angle) {
            Ok(r) => r,
            Err(MissingReason::Angle) => {
                angle_missing = true;
                continue;
            }
            Err(_) => continue,
        };
        let Some(chain) = reducer_chain(series, &c.size, &b[k].row.size) else { continue };
        if best.as_ref().is_none_or(|(_, n, _)| chain.len() < *n) {
            best = Some((row, chain.len(), chain));
        }
    }
    let Some((main, _, branch_chain)) = best else {
        return Err(if angle_missing { MissingReason::Angle } else { MissingReason::Sizes });
    };
    let row = &series.fittings[main];
    let main_de = b[big].row.de;
    let branch_de = row.size2.as_deref().and_then(|s| de_of(series, s)).unwrap_or(main_de);
    let mut ends = vec![(End::Main, main_de); 3];
    ends[inlet] = (End::Main2, main_de);
    ends[k] = (End::Second, branch_de);
    let mut chains = vec![Vec::new(); 3];
    chains[other] = run_chain;
    chains[k] = branch_chain;
    Ok(Plan { main, ends, chains })
}

/// Crosses: four branches of one size.
fn plan_cross(series: &PipeSeries, b: &[Branch]) -> Result<Plan, MissingReason> {
    if b.iter().any(|x| (x.row.de - b[0].row.de).abs() > SAME) {
        return Err(MissingReason::Sizes);
    }
    let main = find_row(series, FittingRowKind::Cross, &b[0].row.size, None, None)?;
    // The branch opposite the first is the run; the other two the branch pair.
    let mut ends = vec![(End::Second, b[0].row.de); 4];
    ends[0] = (End::Main, b[0].row.de);
    if let Some(o) = (1..4).min_by(|&x, &y| b[0].dir.dot(b[x].dir).total_cmp(&b[0].dir.dot(b[y].dir))) {
        ends[o] = (End::Main, b[0].row.de);
    }
    Ok(Plan { main, ends, chains: vec![Vec::new(); 4] })
}

/// The kind of a derived fitting as a catalog row kind (None for cable trays and conduits).
fn row_kind(kind: crate::mep_net::FittingKind) -> FittingRowKind {
    use crate::mep_net::FittingKind as K;
    match kind {
        K::Elbow { .. } => FittingRowKind::Elbow,
        K::Coupling => FittingRowKind::Coupling,
        K::Reducer => FittingRowKind::Reducer,
        K::Tee => FittingRowKind::Tee,
        K::Wye => FittingRowKind::Wye,
        K::Cross => FittingRowKind::Cross,
    }
}

/// Match a pipe fitting at `point` of `kind`, whose branches leave along
/// `dirs` from the runs `runs` (one per branch), to the catalog of the runs'
/// series. None when a run is not a pipe of a series, or the runs are of
/// different series.
pub fn match_fitting(
    project: &Project,
    kind: crate::mep_net::FittingKind,
    point: DVec3,
    dirs: &[DVec3],
    runs: &[ElementId],
) -> Option<FittingMatch> {
    if dirs.len() != runs.len() || dirs.len() < 2 {
        return None;
    }
    let mut series: Option<&PipeSeries> = None;
    let mut branches = Vec::with_capacity(runs.len());
    for (dir, run) in dirs.iter().zip(runs) {
        let e = project.element(*run)?;
        if !matches!(e.kind, cc_core::ElementKind::Pipe(_)) {
            return None;
        }
        let t = project.element_type(e.type_id)?;
        let (s, row) = project.pipe_row(&t.data)?;
        if series.is_some_and(|x| x.id != s.id) {
            return None;
        }
        series = Some(s);
        branches.push(Branch { dir: *dir, row });
    }
    let series = series?;
    let row_kind = row_kind(kind);
    let plan = match kind {
        crate::mep_net::FittingKind::Elbow { angle } if branches.len() == 2 => plan_elbow(series, &branches, angle),
        crate::mep_net::FittingKind::Coupling | crate::mep_net::FittingKind::Reducer if branches.len() == 2 => {
            plan_straight(series, &branches)
        }
        crate::mep_net::FittingKind::Tee | crate::mep_net::FittingKind::Wye if branches.len() == 3 => {
            plan_tee(series, &branches, row_kind)
        }
        crate::mep_net::FittingKind::Cross if branches.len() == 4 => plan_cross(series, &branches),
        _ => Err(MissingReason::Size),
    };
    let plan = match plan {
        Ok(p) => p,
        Err(reason) => {
            return Some(FittingMatch {
                series: series.id,
                status: MatchStatus::Missing(reason),
                pieces: Vec::new(),
                laying: vec![0.0; branches.len()],
            });
        }
    };
    Some(build(series, point, &branches, &plan))
}

/// The pieces, bodies and laying lengths of a plan.
fn build(series: &PipeSeries, point: DVec3, branches: &[Branch], plan: &Plan) -> FittingMatch {
    let Some(main_row) = series.fittings.get(plan.main) else {
        return FittingMatch {
            series: series.id,
            status: MatchStatus::Missing(MissingReason::Size),
            pieces: Vec::new(),
            laying: vec![0.0; branches.len()],
        };
    };
    let main_de = de_of(series, &main_row.size).unwrap_or(0.0);
    let mut main = Piece { row: plan.main, at: point, bodies: Vec::new() };
    let mut reducers = Vec::new();
    let mut laying = Vec::with_capacity(branches.len());
    for ((b, &(end, de)), chain) in branches.iter().zip(&plan.ends).zip(&plan.chains) {
        let (z, socket, radius) = dims(main_row, end, de, main_de);
        main.bodies.push(Body { a: point, b: point + b.dir * (z + socket), radius });
        let mut at = z;
        let mut size_de = de;
        for &r in chain {
            let Some(row) = series.fittings.get(r) else { continue };
            let d1 = de_of(series, &row.size).unwrap_or(size_de);
            let d2 = row.size2.as_deref().and_then(|s| de_of(series, s)).unwrap_or(d1);
            // The reducer's small end, where the next piece (or the pipe) goes.
            let (small_end, small_de, large_de) = if d1 <= d2 { (End::Main, d1, d2) } else { (End::Second, d2, d1) };
            let large_end = if small_end == End::Main { End::Second } else { End::Main };
            let (zl, _, large_radius) = dims(row, large_end, large_de, d1);
            let (zs, ss, _) = dims(row, small_end, small_de, d1);
            let start = point + b.dir * at;
            reducers.push(Piece {
                row: r,
                at: start,
                bodies: vec![Body { a: start, b: start + b.dir * (zl + zs + ss), radius: large_radius }],
            });
            at += zl + zs;
            size_de = small_de;
        }
        laying.push(at);
    }
    let status = if reducers.is_empty() { MatchStatus::Matched } else { MatchStatus::Composite };
    let mut pieces = vec![main];
    pieces.extend(reducers);
    FittingMatch { series: series.id, status, pieces, laying }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mep_net::FittingKind as K;
    use cc_core::{Document, Element, ElementKind, ElementType, ElementTypeId, MepSegment, TypeData};
    use glam::DVec2;

    /// A project with the BR-PVC-SOLD series and a pipe type per size label.
    fn project_with(sizes: &[&str]) -> (Document, Vec<ElementTypeId>) {
        let mut doc = Document::new(Project::from_template());
        let series = cc_core::pipe_catalog::series("BR-PVC-SOLD").unwrap().clone();
        let mut ids = Vec::new();
        doc.edit("t", |e| {
            let sid = cc_core::PipeSeriesId(e.alloc_id());
            let mut s = series.clone();
            s.id = sid;
            s.code = "T-SOLD".into();
            e.pipe_series_mut().insert(sid, s);
            for size in sizes {
                let id = ElementTypeId(e.alloc_id());
                let de = series.row(size).unwrap().de;
                e.add_type(ElementType {
                    id,
                    name: format!("t {size}"),
                    data: TypeData::Pipe {
                        diameter: de,
                        material: String::new(),
                        series: Some(sid),
                        size: (*size).into(),
                        inside: None,
                    },
                    info: Default::default(),
                    uid: Default::default(),
                });
                ids.push(id);
            }
        });
        (doc, ids)
    }

    fn pipe(doc: &mut Document, t: ElementTypeId, a: [f64; 2], b: [f64; 2]) -> ElementId {
        doc.edit("t", |e| {
            let level = e.project().levels[0].id;
            let id = ElementId(e.alloc_id());
            e.add_element(Element {
                id,
                level,
                type_id: t,
                mark: String::new(),
                comments: String::new(),
                uid: Default::default(),
                kind: ElementKind::Pipe(MepSegment::new(DVec2::from_array(a), DVec2::from_array(b), 0.5)),
                phase_created: Default::default(),
                phase_demolished: None,
                design_option: None,
                paints: Vec::new(),
                plugin_data: Default::default(),
            });
            id
        })
    }

    fn only(doc: &Document) -> crate::mep_net::Fitting {
        let geom = crate::ModelGeometry::build(&doc.project);
        assert_eq!(geom.mep.fittings.len(), 1, "{:?}", geom.mep.fittings);
        geom.mep.fittings[0].clone()
    }

    fn row<'a>(doc: &'a Document, m: &FittingMatch, piece: usize) -> &'a CatalogFitting {
        let s = &doc.project.pipe_series[&m.series];
        &s.fittings[m.pieces[piece].row]
    }

    #[test]
    fn a_right_angle_elbow_is_the_catalog_row_with_its_laying_length() {
        let (mut doc, t) = project_with(&["25 mm"]);
        pipe(&mut doc, t[0], [0.0, 0.0], [2.0, 0.0]);
        pipe(&mut doc, t[0], [2.0, 0.0], [2.0, 2.0]);
        let f = only(&doc);
        let m = f.catalog.as_ref().unwrap();
        assert_eq!(m.status, MatchStatus::Matched);
        let r = row(&doc, m, 0);
        assert_eq!((r.kind, r.angle, r.size.as_str()), (FittingRowKind::Elbow, Some(90.0), "25 mm"));
        // Krona's elbow 25: Z 14.4 mm, socket 19 mm, body 33.8 mm.
        assert!(m.laying.iter().all(|l| (l - 0.0144).abs() < 1e-9), "{:?}", m.laying);
        let body = m.pieces[0].bodies[0];
        assert!((body.a.distance(body.b) - (0.0144 + 0.019)).abs() < 1e-9);
        assert!((body.radius - 0.0169).abs() < 1e-9);
        let s = &doc.project.pipe_series[&m.series];
        assert_eq!(fitting_name("Joelho", r, s), "Joelho 90° PVC soldável 25 mm");
    }

    #[test]
    fn a_sixty_degree_elbow_is_missing() {
        let (mut doc, t) = project_with(&["25 mm"]);
        pipe(&mut doc, t[0], [0.0, 0.0], [2.0, 0.0]);
        let a = 60f64.to_radians();
        pipe(&mut doc, t[0], [2.0, 0.0], [2.0 + a.cos(), a.sin()]);
        let f = only(&doc);
        let m = f.catalog.as_ref().unwrap();
        assert_eq!(m.status, MatchStatus::Missing(MissingReason::Angle));
        assert!(m.pieces.is_empty() && m.laying.iter().all(|l| *l == 0.0));
    }

    #[test]
    fn a_reducing_tee_that_is_not_sold_is_a_tee_and_a_reducer() {
        // Krona sells 40 × 25 and 32 × 25 reducing tees, no 40 × 20.
        let (mut doc, t) = project_with(&["40 mm", "20 mm"]);
        let series = &doc.project.pipe_series.values().next().unwrap().clone();
        assert!(find_row(series, FittingRowKind::Tee, "40 mm", Some("20 mm"), None).is_err());
        pipe(&mut doc, t[0], [0.0, 0.0], [4.0, 0.0]);
        pipe(&mut doc, t[1], [2.0, 0.0], [2.0, 2.0]);
        let f = only(&doc);
        assert_eq!(f.kind, K::Tee);
        let m = f.catalog.as_ref().unwrap();
        assert_eq!(m.status, MatchStatus::Composite, "{m:?}");
        assert!(m.pieces.len() >= 2);
        assert_eq!(row(&doc, m, 0).kind, FittingRowKind::Tee);
        assert!(m.pieces[1..].iter().all(|p| series.fittings[p.row].kind == FittingRowKind::Reducer));
        // The branch's pipe ends beyond the tee's branch socket.
        let main = row(&doc, m, 0);
        let branch = f.branches.iter().position(|b| b.0.y > 0.5).unwrap();
        assert!(m.laying[branch] > main.z2.unwrap_or(0.0), "{:?}", m.laying);
    }

    #[test]
    fn a_reducer_chain_skips_sizes_and_stays_short() {
        let (mut doc, t) = project_with(&["50 mm", "20 mm"]);
        pipe(&mut doc, t[0], [0.0, 0.0], [2.0, 0.0]);
        pipe(&mut doc, t[1], [2.0, 0.0], [4.0, 0.0]);
        let f = only(&doc);
        assert_eq!(f.kind, K::Reducer);
        let m = f.catalog.as_ref().unwrap();
        // Krona: 50 × 25 then 25 × 20.
        assert_eq!(m.status, MatchStatus::Composite, "{m:?}");
        let sizes: Vec<(&str, Option<&str>)> =
            (0..m.pieces.len()).map(|i| row(&doc, m, i)).map(|r| (r.size.as_str(), r.size2.as_deref())).collect();
        assert_eq!(sizes, [("50 mm", Some("25 mm")), ("25 mm", Some("20 mm"))]);
        // The gaps of both reducers (16.8 and 1.7 mm) between the pipe ends.
        let total: f64 = m.laying.iter().sum();
        assert!((total - 0.0185).abs() < 1e-9, "{:?}", m.laying);
        // The cut lengths lose them.
        let geom = crate::ModelGeometry::build(&doc.project);
        let cut: f64 = geom.pipe_cuts.values().sum();
        assert!((cut - (4.0 - 0.0185)).abs() < 1e-9, "{cut}");
    }

    #[test]
    fn cut_lengths_lose_the_laying_lengths_at_both_ends() {
        let (mut doc, t) = project_with(&["25 mm"]);
        pipe(&mut doc, t[0], [0.0, 0.0], [2.0, 0.0]);
        let middle = pipe(&mut doc, t[0], [2.0, 0.0], [2.0, 2.0]);
        pipe(&mut doc, t[0], [2.0, 2.0], [4.0, 2.0]);
        let geom = crate::ModelGeometry::build(&doc.project);
        // Two 90° elbows of Z 14.4 mm.
        let cut = geom.pipe_cut_length(middle).unwrap();
        assert!((cut - (2.0 - 2.0 * 0.0144)).abs() < 1e-9, "{cut}");
        // The 3D fittings are the catalog's bodies: two cylinders of 33.8 mm, 33.4 mm long, per elbow.
        let elbows = geom.mep.fittings.iter().filter_map(|f| f.catalog.as_ref());
        for m in elbows {
            assert!(m.pieces[0].bodies.iter().all(|b| (b.radius - 0.0169).abs() < 1e-9));
            assert!(m.pieces[0].bodies.iter().all(|b| (b.a.distance(b.b) - 0.0334).abs() < 1e-9));
        }
    }

    #[test]
    fn runs_of_different_series_or_free_types_have_no_catalog() {
        let (mut doc, t) = project_with(&["25 mm"]);
        let free = doc.project.default_type(cc_core::Category::Pipe).unwrap();
        doc.edit("t", |e| {
            if let Some(TypeData::Pipe { series, .. }) = e.type_mut(free).map(|t| &mut t.data) {
                *series = None;
            }
        });
        pipe(&mut doc, t[0], [0.0, 0.0], [2.0, 0.0]);
        pipe(&mut doc, free, [2.0, 0.0], [2.0, 2.0]);
        assert!(only(&doc).catalog.is_none());
    }

    #[test]
    fn a_forty_five_degree_branch_is_a_wye() {
        let mut doc = Document::new(Project::from_template());
        let drain = doc
            .project
            .types
            .values()
            .find(|t| {
                doc.project.pipe_row(&t.data).is_some_and(|(s, r)| s.code == "BR-PVC-ESG-N" && r.size == "DN 100")
            })
            .map(|t| t.id)
            .unwrap();
        pipe(&mut doc, drain, [0.0, 0.0], [4.0, 0.0]);
        let a = 45f64.to_radians();
        pipe(&mut doc, drain, [2.0 - 2.0 * a.cos(), 2.0 * a.sin()], [2.0, 0.0]);
        let f = only(&doc);
        assert_eq!(f.kind, K::Wye);
        let m = f.catalog.as_ref().unwrap();
        assert_eq!(m.status, MatchStatus::Matched, "{m:?}");
        assert_eq!(row(&doc, m, 0).kind, FittingRowKind::Wye);
    }
}
