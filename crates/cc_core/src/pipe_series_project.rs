//! Pipe series in the project: lookups, the pipe types that are sizes of a
//! series, the template's series and the repair of loaded files.

use crate::pipe_series::{MAX_SERIES, PipeSeries, PipeSeriesId, PipeSize};
use crate::{ElementType, ElementTypeId, MeasurementSystem, Project, TypeData, Uid};
use std::collections::BTreeSet;

/// How far an old template pipe type's diameter may be from the template's
/// for a file before format 30 to link it to its series row, m.
const LINK_TOLERANCE: f64 = 0.001;

/// What a pipe type's diameter should become, and whether it keeps its
/// series reference: `Some(Some(de))` keeps it at the row's outside
/// diameter, `Some(None)` drops a reference whose series or size is gone.
pub(crate) fn pipe_fix(project: &Project, data: &TypeData) -> Option<Option<f64>> {
    let TypeData::Pipe { series: Some(_), .. } = data else { return None };
    Some(project.pipe_row(data).map(|(_, row)| row.de))
}

/// Bring a pipe type in step with its series: the diameter of its row, or
/// no series when the row is gone (the diameter kept). Returns whether it changed.
pub(crate) fn apply_pipe_fix(data: &mut TypeData, fix: Option<f64>) -> bool {
    let TypeData::Pipe { diameter, series, size, .. } = data else { return false };
    match fix {
        Some(de) if *diameter != de => {
            *diameter = de;
            true
        }
        Some(_) => false,
        None => {
            *series = None;
            size.clear();
            true
        }
    }
}

impl Project {
    /// A series of the project.
    pub fn pipe_series_by_id(&self, id: PipeSeriesId) -> Option<&PipeSeries> {
        self.pipe_series.get(&id)
    }

    /// A series of the project by code, ignoring case and surrounding spaces.
    pub fn pipe_series_by_code(&self, code: &str) -> Option<&PipeSeries> {
        let code = code.trim();
        self.pipe_series.values().find(|s| s.code.eq_ignore_ascii_case(code))
    }

    /// The series and row a pipe type is a size of; `None` for other types,
    /// free pipe types and references whose series or size is gone.
    pub fn pipe_row(&self, data: &TypeData) -> Option<(&PipeSeries, &PipeSize)> {
        let TypeData::Pipe { series: Some(id), size, .. } = data else { return None };
        let series = self.pipe_series.get(id)?;
        Some((series, series.row(size)?))
    }

    /// The pipe types that are sizes of series `id`.
    pub fn pipe_series_users(&self, id: PipeSeriesId) -> Vec<ElementTypeId> {
        self.types
            .values()
            .filter(|t| matches!(t.data, TypeData::Pipe { series: Some(s), .. } if s == id))
            .map(|t| t.id)
            .collect()
    }

    /// Add a copy of `series` with a new id and uid (outside an edit: the
    /// template and loading use it; edits allocate through `EditCtx`).
    /// Returns its id.
    pub(crate) fn insert_pipe_series(&mut self, mut series: PipeSeries) -> PipeSeriesId {
        let id = PipeSeriesId(self.alloc_id());
        series.id = id;
        series.uid = Uid::generate();
        self.pipe_series.insert(id, series);
        id
    }

    /// The project's series with the code of built-in series `code`, else a
    /// copy of the built-in one added; `None` when there is no such built-in.
    pub(crate) fn built_in_pipe_series(&mut self, code: &str) -> Option<PipeSeriesId> {
        if let Some(s) = self.pipe_series_by_code(code) {
            return Some(s.id);
        }
        let series = crate::pipe_catalog::series(code)?.clone();
        Some(self.insert_pipe_series(series))
    }

    /// Link the free pipe types named like a pipe type of the template of
    /// `system` (`names` gives the names a key may have) to that type's
    /// series row, when their diameter is within [`LINK_TOLERANCE`] of the
    /// template's old one; the series is copied in on first use and the
    /// diameter becomes the row's. Returns the number of types linked.
    pub(crate) fn link_template_pipes(
        &mut self,
        system: MeasurementSystem,
        names: &dyn Fn(&str) -> Vec<String>,
    ) -> usize {
        let spec = crate::template_spec::TemplateSpec::of(system);
        let mut linked = 0;
        for (key, old, code, label) in spec.pipes {
            let names = names(key);
            let ids: Vec<ElementTypeId> = self
                .types
                .values()
                .filter(|t| names.contains(&t.name))
                .filter(|t| {
                    matches!(t.data, TypeData::Pipe { series: None, diameter, .. }
                        if (diameter - old).abs() <= LINK_TOLERANCE)
                })
                .map(|t| t.id)
                .collect();
            if ids.is_empty() {
                continue;
            }
            let Some(sid) = self.built_in_pipe_series(code) else { continue };
            let Some(de) = self.pipe_series.get(&sid).and_then(|s| s.row(label)).map(|r| r.de) else { continue };
            for id in ids {
                if let Some(TypeData::Pipe { diameter, series, size, .. }) =
                    self.types.get_mut(&id).map(|t| &mut t.data)
                {
                    *diameter = de;
                    *series = Some(sid);
                    *size = label.to_string();
                    linked += 1;
                }
            }
        }
        linked
    }

    /// Link the pipe types of a file older than format 30 that are the
    /// template's (by name, in either system, through `translate` or the
    /// Portuguese defaults, and by diameter). Returns the number linked.
    pub(crate) fn link_old_template_pipes(&mut self, translate: &dyn Fn(&str) -> Option<String>) -> usize {
        let names = |key: &str| {
            let fallback = crate::project::template_default(key).to_string();
            let mut names = vec![fallback.clone()];
            if let Some(t) = translate(key)
                && t != fallback
            {
                names.push(t);
            }
            names
        };
        MeasurementSystem::ALL.into_iter().map(|system| self.link_template_pipes(system, &names)).sum()
    }

    /// Bound the project's series and fix them (see [`PipeSeries::repair`]),
    /// make codes and uids unique, drop series with no sizes, and bring pipe
    /// types in step: references to missing series or sizes dropped (the
    /// diameter kept), diameters of linked types their row's, bad inside
    /// diameters of free types dropped. Returns the number of repairs.
    pub(crate) fn repair_pipe_series(&mut self) -> usize {
        let mut repairs = 0;
        if self.pipe_series.len() > MAX_SERIES {
            repairs += self.pipe_series.len() - MAX_SERIES;
            let keep: BTreeSet<PipeSeriesId> = self.pipe_series.keys().take(MAX_SERIES).copied().collect();
            self.pipe_series.retain(|k, _| keep.contains(k));
        }
        let mut codes: BTreeSet<String> = BTreeSet::new();
        let mut uids: BTreeSet<Uid> = BTreeSet::new();
        for (id, s) in self.pipe_series.iter_mut() {
            if s.id != *id {
                s.id = *id;
                repairs += 1;
            }
            repairs += s.repair();
            if !codes.insert(s.code.to_lowercase()) {
                let base: String = s.code.chars().take(crate::pipe_series::MAX_TEXT - 12).collect();
                let mut n = 2u64;
                while codes.contains(&format!("{base}-{n}").to_lowercase()) {
                    n += 1;
                }
                s.code = format!("{base}-{n}");
                codes.insert(s.code.to_lowercase());
                repairs += 1;
            }
            if s.uid.is_nil() || !uids.insert(s.uid) {
                s.uid = Uid::generate();
                uids.insert(s.uid);
                repairs += 1;
            }
        }
        let before = self.pipe_series.len();
        self.pipe_series.retain(|_, s| !s.pipes.is_empty());
        repairs += before - self.pipe_series.len();

        let fixes: Vec<(ElementTypeId, Option<f64>)> =
            self.types.values().filter_map(|t| Some((t.id, pipe_fix(self, &t.data)?))).collect();
        for (id, fix) in fixes {
            if let Some(t) = self.types.get_mut(&id) {
                repairs += usize::from(apply_pipe_fix(&mut t.data, fix));
            }
        }
        for t in self.types.values_mut() {
            if let TypeData::Pipe { series, size, inside, diameter, .. } = &mut t.data {
                if series.is_none() && !size.is_empty() {
                    size.clear();
                    repairs += 1;
                }
                if inside.is_some_and(|d| !(d.is_finite() && d > 0.0 && d < *diameter)) {
                    *inside = None;
                    repairs += 1;
                }
            }
        }
        repairs
    }
}

impl ElementType {
    /// The inside diameter of a pipe type: its series row's, else its own
    /// `inside`; `None` for free types without one (callers fall back to
    /// their tables) and for other types.
    pub fn inside_diameter(&self, project: &Project) -> Option<f64> {
        if let Some((_, row)) = project.pipe_row(&self.data) {
            return Some(row.di);
        }
        match self.data {
            TypeData::Pipe { inside, .. } => inside,
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Document, MeasurementSystem, PipeSeriesId, Project, TypeData};

    fn pipe_types(p: &Project) -> Vec<&crate::ElementType> {
        p.types.values().filter(|t| matches!(t.data, TypeData::Pipe { .. })).collect()
    }

    #[test]
    fn every_template_pipe_type_is_a_size_of_a_series() {
        for system in MeasurementSystem::ALL {
            let p = Project::from_template_system(system, &|_| None);
            let pipes = pipe_types(&p);
            assert_eq!(pipes.len(), 7);
            for t in pipes {
                let (series, row) = p.pipe_row(&t.data).expect("a series row");
                let TypeData::Pipe { diameter, .. } = t.data else { panic!("a pipe type") };
                assert_eq!(diameter, row.de, "{}", t.name);
                assert_eq!(t.inside_diameter(&p), Some(row.di));
                assert!(!series.uid.is_nil());
            }
            assert_eq!(p.pipe_series.len(), 2);
            assert_eq!(p.repairs_needed(), 0);
        }
        let p = Project::from_template();
        let dn100 = p.types.values().find(|t| t.name == "Tubo PVC esgoto 100 mm").expect("DN 100");
        assert!(matches!(dn100.data, TypeData::Pipe { diameter, .. } if (diameter - 0.1016).abs() < 1e-12));
        let p = Project::from_template_system(MeasurementSystem::Imperial, &|_| None);
        let half = p.types.values().find(|t| t.name == "Tubo PVC soldável 1/2\"").expect("1/2\"");
        assert!(matches!(half.data, TypeData::Pipe { diameter, .. } if (diameter - 0.02134).abs() < 1e-12));
    }

    #[test]
    fn editing_a_row_resizes_its_types_in_one_step() {
        let mut doc = Document::new(Project::from_template());
        let (sid, tid) = {
            let p = &doc.project;
            let t = pipe_types(p).into_iter().find(|t| t.name == "Tubo PVC soldável 25 mm").expect("25 mm");
            let TypeData::Pipe { series: Some(sid), .. } = t.data else { panic!("a pipe type") };
            (sid, t.id)
        };
        doc.edit("test", |ctx| {
            if let Some(row) =
                ctx.pipe_series_mut().get_mut(&sid).and_then(|s| s.pipes.iter_mut().find(|r| r.size == "25 mm"))
            {
                row.de = 0.026;
                row.di = 0.026 - 2.0 * row.e;
            }
        });
        let diameter = |p: &Project| match p.types[&tid].data {
            TypeData::Pipe { diameter, .. } => diameter,
            _ => 0.0,
        };
        assert_eq!(diameter(&doc.project), 0.026);
        assert!(doc.undo());
        assert_eq!(diameter(&doc.project), 0.025);
        assert_eq!(doc.project.pipe_series[&sid].row("25 mm").map(|r| r.de), Some(0.025));
    }

    #[test]
    fn a_deleted_series_detaches_its_types() {
        let mut doc = Document::new(Project::from_template());
        let sid = doc.project.pipe_series_by_code("BR-PVC-SOLD").expect("series").id;
        let users = doc.project.pipe_series_users(sid);
        assert_eq!(users.len(), 3);
        doc.edit("test", |ctx| {
            ctx.pipe_series_mut().remove(&sid);
        });
        for id in users {
            let TypeData::Pipe { series, size, diameter, .. } = &doc.project.types[&id].data else {
                panic!("a pipe type")
            };
            assert_eq!((*series, size.as_str()), (None, ""));
            assert!(*diameter > 0.0);
        }
    }

    #[test]
    fn a_type_set_to_another_size_takes_its_diameter() {
        let mut doc = Document::new(Project::from_template());
        let t = pipe_types(&doc.project).into_iter().find(|t| t.name == "Tubo PVC soldável 20 mm").expect("20").id;
        doc.edit("test", |ctx| {
            if let Some(TypeData::Pipe { size, .. }) = ctx.type_mut(t).map(|t| &mut t.data) {
                *size = "50 mm".into();
            }
        });
        assert!(matches!(doc.project.types[&t].data, TypeData::Pipe { diameter, .. } if diameter == 0.05));
    }

    #[test]
    fn repair_drops_dangling_rows_and_references() {
        let mut p = Project::from_template();
        let sid = p.pipe_series_by_code("BR-PVC-ESG-N").expect("series").id;
        let users = p.pipe_series_users(sid);
        let s = p.pipe_series.get_mut(&sid).expect("series");
        s.pipes.retain(|r| r.size != "DN 100");
        s.fittings.push(s.fittings[0].clone());
        let other = p.pipe_series_by_code("BR-PVC-SOLD").expect("series").id;
        p.pipe_series.get_mut(&other).expect("series").code = "br-pvc-esg-n".into();
        if let Some(TypeData::Pipe { series, .. }) = p.types.get_mut(&users[0]).map(|t| &mut t.data) {
            *series = Some(PipeSeriesId(999_999));
        }
        let fittings = p.pipe_series[&sid].fittings.len();
        assert!(p.repair() >= 4);
        let s = &p.pipe_series[&sid];
        assert!(s.fittings.len() < fittings);
        assert!(s.fittings.iter().all(|f| s.row(&f.size).is_some()));
        // The older series keeps the code; the newer one gets a suffix.
        assert_eq!(p.pipe_series[&other].code, "br-pvc-esg-n");
        assert_eq!(p.pipe_series[&sid].code, "BR-PVC-ESG-N-2");
        let free = users.iter().filter(|id| p.pipe_row(&p.types[id].data).is_none()).count();
        assert_eq!(free, 2, "the dangling reference and the DN 100 type are free");
        assert!(matches!(p.types[&users[0]].data, TypeData::Pipe { series: None, .. }));
        assert_eq!(p.repair(), 0);
    }
}
