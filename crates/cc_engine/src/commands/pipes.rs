//! Pipe series (see [`cc_core::pipe_series`]): the built-in catalog, the
//! project's series, their files and the pipe types that are sizes of a
//! series. Series are project settings; each edit is one undo step.
//!
//! A pipe type that is a size of a series keeps its outside diameter at its
//! row's (`EditCtx::finish` re-syncs it), so `type_set` takes `series` (a
//! code or name, null for a free type) and `size` (a row's label) in place
//! of a diameter.

use super::edit;
use crate::error::{EngineError, Result};
use crate::refs::{self, Ref};
use crate::registry::{Command, CommandKind};
use crate::session::Session;
use crate::t;
use cc_core::pipe_series::{MAX_FILE_BYTES, MAX_SERIES};
use cc_core::{Category, EditCtx, ElementType, ElementTypeId, PipeSeries, PipeSeriesId, Project, TypeData, Uid};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::PathBuf;

/// How [`series_type`] found a pipe type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Found {
    /// A type of that series and size.
    Existing,
    /// A free pipe type of that name, now a size of the series.
    Linked,
    /// A new type.
    Created,
}

/// The name of the type of row `size` of `series`: `"{name} {size}"`.
pub(crate) fn type_name(series: &PipeSeries, size: &str) -> String {
    format!("{} {}", series.name, size)
}

/// The project's pipe type of series `sid` and row `size`.
pub(crate) fn type_of_row(p: &Project, sid: PipeSeriesId, size: &str) -> Option<ElementTypeId> {
    p.types_of(Category::Pipe)
        .find(|t| matches!(&t.data, TypeData::Pipe { series: Some(s), size: z, .. } if *s == sid && z == size))
        .map(|t| t.id)
}

/// The material a new type of series `sid` gets: another type's of that
/// series, else any pipe type's, else the project's first material.
pub(crate) fn default_material(p: &Project, sid: PipeSeriesId) -> String {
    let of = |t: &ElementType| match &t.data {
        TypeData::Pipe { material, .. } => Some(material.clone()),
        _ => None,
    };
    let pipes = || p.types_of(Category::Pipe);
    pipes()
        .filter(|t| matches!(t.data, TypeData::Pipe { series: Some(s), .. } if s == sid))
        .find_map(of)
        .or_else(|| pipes().find_map(of))
        .or_else(|| p.materials.first().map(|m| m.name.clone()))
        .unwrap_or_default()
}

/// `base`, or `base 2`, `base 3`… when a pipe type has that name.
fn free_type_name(p: &Project, base: &str) -> String {
    let taken = |n: &str| p.types_of(Category::Pipe).any(|t| t.name.trim() == n.trim());
    let mut name = base.to_string();
    let mut k = 2;
    while taken(&name) && k < 10_000 {
        name = format!("{base} {k}");
        k += 1;
    }
    name
}

/// Add a pipe type of row `size` of series `sid` named `name` (made free of
/// other pipe types' names) in `material`. `None` when the row is missing.
fn new_series_type(
    e: &mut EditCtx,
    sid: PipeSeriesId,
    size: &str,
    name: &str,
    material: String,
) -> Option<ElementTypeId> {
    let p = e.project();
    let row = p.pipe_series_by_id(sid)?.row(size)?.clone();
    let name = free_type_name(p, name);
    let mut data = TypeData::pipe(row.de, material);
    if let TypeData::Pipe { series, size: z, .. } = &mut data {
        *series = Some(sid);
        *z = row.size;
    }
    let id = ElementTypeId(e.alloc_id());
    e.add_type(ElementType { id, name, data, info: Default::default(), uid: Default::default() });
    Some(id)
}

/// The pipe type of row `size` of series `sid`: the project's type of that
/// row; else a free pipe type named `"{series} {size}"`, which becomes that
/// row; else a new type of that name in `material` (default
/// [`default_material`]). The diameter follows the row when the edit
/// finishes. `None` when the series or the row is missing.
pub(crate) fn series_type(
    e: &mut EditCtx,
    sid: PipeSeriesId,
    size: &str,
    material: Option<String>,
) -> Option<(ElementTypeId, Found)> {
    let p = e.project();
    let series = p.pipe_series_by_id(sid)?;
    let row = series.row(size)?.clone();
    if let Some(id) = type_of_row(p, sid, size) {
        return Some((id, Found::Existing));
    }
    let name = type_name(series, size);
    let material = material.unwrap_or_else(|| default_material(p, sid));
    let free = p
        .types_of(Category::Pipe)
        .find(|t| t.name.trim() == name.trim() && matches!(t.data, TypeData::Pipe { series: None, .. }))
        .map(|t| t.id);
    if let Some(id) = free {
        if let Some(TypeData::Pipe { diameter, series, size: z, inside, .. }) = e.type_mut(id).map(|t| &mut t.data) {
            *diameter = row.de;
            *series = Some(sid);
            *z = row.size;
            *inside = None;
        }
        return Some((id, Found::Linked));
    }
    Some((new_series_type(e, sid, size, &name, material)?, Found::Created))
}

/// What `element_get` and `type_get` add for a pipe type: its
/// `inside_diameter` and `wall` (m; null when unknown), and the code of its
/// series and its size (null for a free type).
pub(crate) fn type_derived(p: &Project, t: &ElementType, out: &mut serde_json::Map<String, Value>) {
    let TypeData::Pipe { diameter, .. } = &t.data else { return };
    let row = p.pipe_row(&t.data);
    let inside = t.inside_diameter(p);
    let wall = row.map(|(_, r)| r.e).or_else(|| inside.map(|di| (diameter - di) / 2.0));
    out.insert("inside_diameter".into(), json!(inside));
    out.insert("wall".into(), json!(wall));
    out.insert("series_code".into(), json!(row.map(|(s, _)| s.code.clone())));
    out.insert("size".into(), json!(row.map(|(_, r)| r.size.clone())));
}

/// A pipe type's params patch with its `series` (a code, name or id, as
/// `type_get` shows it; null for a free type) turned into the series' id;
/// a type with no series loses its size.
pub(crate) fn pipe_patch(p: &Project, data: &TypeData, patch: Value) -> Result<Value> {
    let mut patch = patch;
    if !matches!(data, TypeData::Pipe { .. }) {
        return Ok(patch);
    }
    let Some(o) = patch.as_object_mut() else { return Ok(patch) };
    let Some(slot) = o.get_mut("series") else { return Ok(patch) };
    let r = match slot {
        Value::Null => {
            o.insert("size".into(), json!(""));
            return Ok(patch);
        }
        Value::String(n) => Ref::Name(n.clone()),
        Value::Number(n) => Ref::Id(n.as_u64().ok_or_else(|| EngineError::invalid("series: not a series id"))?),
        _ => return Err(EngineError::invalid("series: a pipe series' code, name or id")),
    };
    let id = refs::pipe_series(p, &r).map_err(|e| EngineError::invalid(format!("series: {}", e.message)))?;
    *slot = json!(id.0);
    Ok(patch)
}

/// Refuse a pipe type whose series has no row of its size.
pub(crate) fn check_size(p: &Project, data: &TypeData) -> Result<()> {
    let TypeData::Pipe { series: Some(sid), size, .. } = data else { return Ok(()) };
    let series = p.pipe_series_by_id(*sid).ok_or_else(|| EngineError::not_found("pipe series"))?;
    if series.row(size).is_some() {
        return Ok(());
    }
    let sizes: Vec<&str> = series.pipes.iter().map(|r| r.size.as_str()).collect();
    Err(EngineError::invalid(format!(
        "unknown_size: series {} has no size '{size}' ({})",
        series.code,
        sizes.join(", ")
    )))
}

/// A series in the file form (millimeters), as JSON.
fn file_json(series: &PipeSeries) -> Value {
    serde_json::from_str(&cc_core::pipe_file::write_series(series)).unwrap_or(Value::Null)
}

/// The name of built-in series `code` in the active language.
fn built_in_name(code: &str, fallback: &str) -> String {
    cc_core::pipe_catalog::built_in_of(code)
        .and_then(|b| crate::i18n::try_tr(b.name_key))
        .unwrap_or_else(|| fallback.to_string())
}

/// A copy of built-in series `code`, named in the active language.
fn built_in_copy(code: &str) -> Option<PipeSeries> {
    let mut series = cc_core::pipe_catalog::series(code)?.clone();
    series.name = built_in_name(&series.code, &series.name);
    Some(series)
}

/// The built-in pipe series (Brazilian, European and US product lines):
/// each one's `code`, translated `name`, `region`, `material`, `use`
/// (supply, drain or any), `joint`, `pressure`, `standard`, its `sizes`
/// (labels) and its number of `fittings`. With `series`, that series alone,
/// with its `pipes` and `fittings` rows in millimeters.
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct PipeCatalog {
    /// A built-in series' code (ignoring case): list its rows.
    #[serde(default)]
    pub series: Option<String>,
}

fn catalog_json(b: &cc_core::pipe_catalog::BuiltIn, s: &PipeSeries) -> Value {
    json!({
        "code": s.code,
        "name": built_in_name(b.code, &s.name),
        "region": b.region.code(),
        "material": s.material.code(),
        "use": s.use_.code(),
        "joint": s.joint.code(),
        "pressure": s.pressure,
        "standard": s.standard,
        "sizes": s.pipes.iter().map(|r| r.size.clone()).collect::<Vec<_>>(),
        "fittings": s.fittings.len(),
    })
}

impl Command for PipeCatalog {
    const NAME: &'static str = "pipe_catalog";
    const KIND: CommandKind = CommandKind::Query;
    fn run(self, _s: &mut Session) -> Result<Value> {
        let all = cc_core::pipe_catalog::all();
        let Some(code) = &self.series else {
            let series: Vec<Value> = all.iter().map(|(b, s)| catalog_json(b, s)).collect();
            return Ok(json!({ "series": series }));
        };
        let (b, s) = all
            .iter()
            .find(|(b, _)| b.code.eq_ignore_ascii_case(code.trim()))
            .ok_or_else(|| EngineError::not_found(format!("no built-in pipe series '{code}'")))?;
        let mut out = catalog_json(b, s);
        let file = file_json(s);
        if let Some(o) = out.as_object_mut() {
            o.insert("pipes".into(), file.get("pipes").cloned().unwrap_or(json!([])));
            o.insert("fittings".into(), file.get("fittings").cloned().unwrap_or(json!([])));
            o.insert("sources".into(), file.get("sources").cloned().unwrap_or(json!([])));
        }
        Ok(json!({ "series": [out] }))
    }
}

/// The project's pipe series: id, code, name, standard, material, use,
/// joint, pressure, the size labels, the number of fittings and the pipe
/// types that are sizes of it.
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct PipeSeriesList {}

impl Command for PipeSeriesList {
    const NAME: &'static str = "pipe_series_list";
    const KIND: CommandKind = CommandKind::Query;
    fn run(self, s: &mut Session) -> Result<Value> {
        let p = s.project();
        let series: Vec<Value> = p
            .pipe_series
            .values()
            .map(|x| {
                json!({
                    "id": x.id.0,
                    "code": x.code,
                    "name": x.name,
                    "standard": x.standard,
                    "material": x.material.code(),
                    "use": x.use_.code(),
                    "joint": x.joint.code(),
                    "pressure": x.pressure,
                    "sizes": x.pipes.iter().map(|r| r.size.clone()).collect::<Vec<_>>(),
                    "fittings": x.fittings.len(),
                    "types": x_types(p, x.id),
                })
            })
            .collect();
        Ok(json!({ "series": series }))
    }
}

fn x_types(p: &Project, id: PipeSeriesId) -> Vec<u64> {
    p.pipe_series_users(id).into_iter().map(|t| t.0).collect()
}

/// One pipe series of the project in the file form (lengths in
/// millimeters: `pipes` with `de`, `e`, `di`; `fittings` with their
/// sockets and laying lengths), plus its `id` and the pipe `types` that are
/// sizes of it.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PipeSeriesGet {
    /// The series' id, code or name.
    pub series: Ref,
}

impl Command for PipeSeriesGet {
    const NAME: &'static str = "pipe_series_get";
    const KIND: CommandKind = CommandKind::Query;
    fn run(self, s: &mut Session) -> Result<Value> {
        let p = s.project();
        let id = refs::pipe_series(p, &self.series)?;
        let series = p.pipe_series_by_id(id).ok_or_else(|| EngineError::not_found("pipe series"))?;
        let mut out = file_json(series);
        if let Some(o) = out.as_object_mut() {
            o.remove("format");
            o.insert("id".into(), json!(id.0));
            o.insert("types".into(), json!(x_types(p, id)));
        }
        Ok(out)
    }
}

/// What to do with an imported series whose code the project already has.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SeriesClash {
    /// Keep the project's series (types are still made from it).
    #[default]
    Skip,
    /// Put the imported rows in place of the project's, keeping its id: its
    /// types follow their rows (a type whose size is gone becomes free).
    Replace,
    /// Import it under the code with a number (`-2`, `-3`…).
    Rename,
}

/// Import a pipe series into the project: a built-in one by `code` (see
/// `pipe_catalog`), or a series file (JSON, millimeters, the format
/// `pipe_series_export` writes) by `path` or `text`. With `sizes`, also
/// make the pipe types of those sizes, named `"{series} {size}"`: a type of
/// that row is reused, and a free pipe type of that name becomes the row.
/// One undo step; returns the `series` id and the types `created` and
/// `linked`.
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct PipeSeriesImport {
    /// A built-in series' code (or give `path` or `text`).
    #[serde(default)]
    pub code: Option<String>,
    /// Path of a series file.
    #[serde(default)]
    pub path: Option<PathBuf>,
    /// A series file's text.
    #[serde(default)]
    pub text: Option<String>,
    /// When the project has a series of the same code: `skip` (default),
    /// `replace` or `rename`.
    #[serde(default)]
    pub on_clash: SeriesClash,
    /// Sizes to make pipe types of: labels of the series' rows, or `["all"]`
    /// (default: none).
    #[serde(default)]
    pub sizes: Option<Vec<String>>,
}

/// What an import does to the project's series.
enum Plan {
    /// Keep the existing series.
    Keep(PipeSeriesId),
    /// Put new rows in an existing series.
    Replace(PipeSeriesId, Box<PipeSeries>),
    /// Add a series.
    Add(Box<PipeSeries>),
}

impl Command for PipeSeriesImport {
    const NAME: &'static str = "pipe_series_import";
    const KIND: CommandKind = CommandKind::Edit;
    const READS_FILES: bool = true;
    fn run(self, s: &mut Session) -> Result<Value> {
        let mut series = match (&self.code, &self.path, self.text) {
            (Some(code), None, None) => built_in_copy(code)
                .ok_or_else(|| EngineError::not_found(format!("no built-in pipe series '{code}'")))?,
            (None, Some(path), None) => {
                s.require_read("path", path)?;
                let len = std::fs::metadata(path)?.len();
                if len > MAX_FILE_BYTES as u64 {
                    return Err(EngineError::invalid(format!("the file is larger than {} MB", MAX_FILE_BYTES >> 20)));
                }
                let text = String::from_utf8_lossy(&std::fs::read(path)?).into_owned();
                read(&text)?
            }
            (None, None, Some(text)) => read(&text)?,
            _ => return Err(EngineError::invalid("give exactly one of code, path or text")),
        };
        let p = s.project();
        let clash = p.pipe_series_by_code(&series.code).map(|x| x.id);
        let plan = match (clash, self.on_clash) {
            (Some(id), SeriesClash::Skip) => Plan::Keep(id),
            (Some(id), SeriesClash::Replace) => Plan::Replace(id, Box::new(series)),
            (Some(_), SeriesClash::Rename) => {
                let base: String = series.code.chars().take(cc_core::pipe_series::MAX_TEXT - 8).collect();
                let mut k = 2;
                while p.pipe_series_by_code(&format!("{base}-{k}")).is_some() && k < 10_000 {
                    k += 1;
                }
                series.code = format!("{base}-{k}");
                Plan::Add(Box::new(series))
            }
            (None, _) => Plan::Add(Box::new(series)),
        };
        if matches!(plan, Plan::Add(_)) && p.pipe_series.len() >= MAX_SERIES {
            return Err(EngineError::rejected(format!("a project has at most {MAX_SERIES} pipe series")));
        }
        // The sizes asked for, checked against the rows the series will have.
        let rows: Vec<String> = match &plan {
            Plan::Keep(id) => p.pipe_series_by_id(*id).map(|x| x.pipes.clone()).unwrap_or_default(),
            Plan::Replace(_, x) | Plan::Add(x) => x.pipes.clone(),
        }
        .into_iter()
        .map(|r| r.size)
        .collect();
        let sizes: Vec<String> = match &self.sizes {
            None => Vec::new(),
            Some(list) if list.iter().any(|x| x.trim().eq_ignore_ascii_case("all")) => rows,
            Some(list) => list
                .iter()
                .map(|x| {
                    rows.iter().find(|r| r.as_str() == x.trim()).cloned().ok_or_else(|| {
                        EngineError::not_found(format!(
                            "unknown_size: the series has no size '{x}' ({})",
                            rows.join(", ")
                        ))
                    })
                })
                .collect::<Result<_>>()?,
        };
        if let Plan::Keep(id) = &plan
            && sizes.iter().all(|z| type_of_row(p, *id, z).is_some())
        {
            return Ok(json!({ "series": id.0, "created": [], "linked": [], "skipped": true }));
        }
        let skipped = matches!(plan, Plan::Keep(_));
        let (sid, created, linked) = edit(s, t!("tx-pipe-series-import"), |e| {
            let sid = match plan {
                Plan::Keep(id) => id,
                Plan::Replace(id, mut x) => {
                    let uid = e.project().pipe_series_by_id(id).map_or_else(Uid::generate, |old| old.uid);
                    x.id = id;
                    x.uid = uid;
                    e.pipe_series_mut().insert(id, *x);
                    id
                }
                Plan::Add(mut x) => {
                    let id = PipeSeriesId(e.alloc_id());
                    x.id = id;
                    x.uid = Uid::generate();
                    e.pipe_series_mut().insert(id, *x);
                    id
                }
            };
            let (mut created, mut linked) = (Vec::new(), Vec::new());
            for size in &sizes {
                match series_type(e, sid, size, None) {
                    Some((t, Found::Created)) => created.push(t.0),
                    Some((t, Found::Linked)) => linked.push(t.0),
                    _ => {}
                }
            }
            Ok((sid, created, linked))
        })?;
        Ok(json!({ "series": sid.0, "created": created, "linked": linked, "skipped": skipped }))
    }
}

/// Read a series file, its problem as an invalid argument.
fn read(text: &str) -> Result<PipeSeries> {
    cc_core::pipe_file::read_series(text).map_err(|e| EngineError::invalid(format!("not a valid series file: {e}")))
}

/// Write a project pipe series as a series file (JSON, millimeters): to
/// `path` when given; the `text` is returned either way.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PipeSeriesExport {
    /// The series' id, code or name.
    pub series: Ref,
    /// Path of the file to write.
    #[serde(default)]
    pub path: Option<PathBuf>,
}

impl Command for PipeSeriesExport {
    const NAME: &'static str = "pipe_series_export";
    const KIND: CommandKind = CommandKind::File;
    const CHECKS_PATHS: bool = true;
    fn run(self, s: &mut Session) -> Result<Value> {
        if let Some(path) = &self.path {
            s.require_write("path", path)?;
        }
        let p = s.project();
        let id = refs::pipe_series(p, &self.series)?;
        let series = p.pipe_series_by_id(id).ok_or_else(|| EngineError::not_found("pipe series"))?;
        let text = cc_core::pipe_file::write_series(series);
        if let Some(path) = &self.path {
            std::fs::write(path, &text)?;
        }
        Ok(json!({ "code": series.code, "text": text }))
    }
}

/// Delete a pipe series from the project. Refused while pipe types are
/// sizes of it, unless `detach`: they then become free types keeping their
/// outside and inside diameters. One undo step.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PipeSeriesDelete {
    /// The series' id, code or name.
    pub series: Ref,
    /// Turn the pipe types of the series into free types instead of refusing.
    #[serde(default)]
    pub detach: bool,
}

impl Command for PipeSeriesDelete {
    const NAME: &'static str = "pipe_series_delete";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let p = s.project();
        let id = refs::pipe_series(p, &self.series)?;
        let users = p.pipe_series_users(id);
        if !users.is_empty() && !self.detach {
            return Err(EngineError::rejected(format!(
                "in_use: {} pipe types are sizes of this series; set detach to keep them as free types",
                users.len()
            )));
        }
        edit(s, t!("tx-pipe-series-delete"), |e| {
            for t in &users {
                let inside = e.project().element_type(*t).and_then(|x| x.inside_diameter(e.project()));
                if let Some(TypeData::Pipe { series, size, inside: own, .. }) = e.type_mut(*t).map(|x| &mut x.data) {
                    *series = None;
                    size.clear();
                    *own = inside;
                }
            }
            e.pipe_series_mut().remove(&id);
            Ok(())
        })?;
        Ok(json!({ "deleted": id.0, "detached": users.iter().map(|t| t.0).collect::<Vec<_>>() }))
    }
}

/// Make a pipe type that is a size of a pipe series: `series` is a project
/// series (id, code or name) or a built-in code, which is then imported in
/// the same undo step. The type is named `name` (default `"{series}
/// {size}"`, numbered when taken) and its outside diameter is the row's.
/// Returns the type's `id` and the `series` id.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PipeTypeCreate {
    /// A project series' id, code or name, or a built-in series' code.
    pub series: Ref,
    /// The size: a row's label (`25 mm`, `DN 100`, `d63`, `3/4"`…).
    pub size: String,
    /// The type's name (default: the series' name and the size).
    #[serde(default)]
    pub name: Option<String>,
    /// Material name or index (default: another pipe type's).
    #[serde(default)]
    pub material: Option<Ref>,
}

impl Command for PipeTypeCreate {
    const NAME: &'static str = "pipe_type_create";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let p = s.project();
        let (existing, imported) = match refs::pipe_series(p, &self.series) {
            Ok(id) => (Some(id), None),
            Err(err) => match &self.series {
                Ref::Name(code) => match built_in_copy(code) {
                    Some(x) => (None, Some(x)),
                    None => return Err(err),
                },
                Ref::Id(_) => return Err(err),
            },
        };
        let rows = match (existing.and_then(|id| p.pipe_series_by_id(id)), &imported) {
            (Some(x), _) | (None, Some(x)) => x,
            (None, None) => return Err(EngineError::not_found("pipe series")),
        };
        let Some(row) = rows.row(self.size.trim()) else {
            let sizes: Vec<&str> = rows.pipes.iter().map(|r| r.size.as_str()).collect();
            return Err(EngineError::invalid(format!(
                "unknown_size: series {} has no size '{}' ({})",
                rows.code,
                self.size,
                sizes.join(", ")
            )));
        };
        let size = row.size.clone();
        let base = match &self.name {
            Some(n) if n.trim().is_empty() => return Err(EngineError::invalid("the name is empty")),
            Some(n) => {
                if p.types_of(Category::Pipe).any(|t| t.name.trim() == n.trim()) {
                    return Err(EngineError::invalid(format!("the name '{n}' is already used")));
                }
                n.trim().to_string()
            }
            None => type_name(rows, &size),
        };
        let material = self.material.as_ref().map(|m| refs::material(p, m)).transpose()?;
        if imported.is_some() && p.pipe_series.len() >= MAX_SERIES {
            return Err(EngineError::rejected(format!("a project has at most {MAX_SERIES} pipe series")));
        }
        let (id, sid) = edit(s, t!("tx-pipe-type-create"), |e| {
            let sid = match (existing, imported) {
                (Some(id), _) => id,
                (None, Some(mut x)) => {
                    let id = PipeSeriesId(e.alloc_id());
                    x.id = id;
                    x.uid = Uid::generate();
                    e.pipe_series_mut().insert(id, x);
                    id
                }
                (None, None) => return Err(EngineError::not_found("pipe series")),
            };
            let material = material.unwrap_or_else(|| default_material(e.project(), sid));
            let id = new_series_type(e, sid, &size, &base, material)
                .ok_or_else(|| EngineError::not_found("pipe series row"))?;
            Ok((id, sid))
        })?;
        Ok(json!({ "id": id.0, "series": sid.0 }))
    }
}
