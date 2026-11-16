//! Pipe series (see [`cc_core::pipe_series`]): the built-in catalog, the
//! project's series, their files and the pipe types that are sizes of a
//! series. Series are project settings; each edit is one undo step.
//!
//! A pipe type that is a size of a series keeps its outside diameter at its
//! row's (`EditCtx::finish` re-syncs it), so `type_set` takes `series` (a
//! code or name, null for a free type) and `size` (a row's label) in place
//! of a diameter.

use cc_core::{Category, EditCtx, ElementType, ElementTypeId, PipeSeries, PipeSeriesId, Project, TypeData};

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
