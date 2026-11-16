# Pipe Catalog, Stage 1 (Core and Data) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Pipe series (sizes with outside/inside diameters and walls, plus the fittings sold for them with their dimensions) live in the project. A built-in catalog of 22 series (Brazil, Europe, US) and series files can be imported. Pipe types point to a series row, the templates use series, older files are linked on load (format 29), and sizing uses real inside diameters within the run's series.

**Spec:** `docs/superpowers/specs/2026-11-16-pipe-catalog-design.md` (this plan covers its "Plan shape" item 1; fittings matching, IFC and the app are stages 2 and 3).

**Data:** already in the worktree. `crates/cc_core/assets/pipes/*.json` is 22 series in file format 1 (mm), and `assets/pipes/notes/{br,eu,us}.md` holds the research notes. The data is not edited by this plan except to fix what the validation test finds; any fix is recorded in the notes.

## Global constraints

- **Where to work.** Only in the worktree `D:\constructcad-pipes` (branch `feat/pipe-catalog`, from `origin/main` at `1300ebe5`). Never touch `D:\constructcad` or other worktrees.
- **Git.**
  - Never push, never run `gh`, never merge.
  - Commit with `git commit -s`, with no Claude attribution and no Co-Authored-By.
  - One commit per task, which also includes the spec, plan and data in Task 1.
- **No panics in non-test code.** No `unwrap`, `expect`, `panic!`, `unreachable!`, `todo!` or `unimplemented!`, and no `unsafe`. Files are hostile input, so reading is bounded and errors are values.
- **i18n.** Every interface or transaction text is a key in all 12 `assets/i18n/*.ftl` files (`pt-BR` is the reference).
- **Commands.**
  - Every command parameter has a doc comment.
  - Commands are registered in `crates/cc_engine/src/registry.rs`.
  - Regenerate the reference with `cargo run -q -p cc_cli -- reference > docs/commands.md`.
  - A new file in `crates/cc_engine/tests/` needs `mod x;` in `tests/main.rs`.
- **Checks.**
  - Tests: `cargo nextest run -p <crate> <filter>`, with `--test main` for integration tests.
  - Type errors: `cargo check --tests -p <crate>`.
  - Format: `cargo fmt --all` (max_width 120).
  - Lints: `cargo clippy --workspace --all-targets -- -D warnings`.
- **Units.** Model values are SI: meters in `PipeSeries`. File values are millimeters, converted on read and write.

## Names (new, must not clash)

- `cc_core::pipe_series`:
  - `PipeSeries`, `PipeSize`, `CatalogFitting`, `FittingRowKind` (`Elbow`, `Tee`, `Wye`, `Cross`, `Coupling`, `Reducer`), `PipeMaterial`, `PipeUse`, `PipeJoint`, `SeriesSource`;
  - `PipeSeriesId` (`id_type!(PipeSeriesId, "PS")` in `ids.rs`);
  - `Project::pipe_series: BTreeMap<PipeSeriesId, PipeSeries>` (serde default, `skip_serializing_if` empty);
  - `EditCtx::pipe_series_mut`.
- `cc_core::pipe_file`: `SeriesFile` (the serde form, mm), `read_series(text) -> Result<PipeSeries, PipeFileError>`, `write_series(&PipeSeries) -> String`, `FILE_FORMAT = 1`.
- `cc_core::pipe_catalog`:
  - `BuiltIn { code, name_key, region: Region (Brazil, Europe, Us), file }`, `BUILT_IN: &[BuiltIn]`;
  - `series(code) -> Option<&'static PipeSeries>` (parsed once, `OnceLock`), `all() -> &'static [(…)]`.
- `cc_engine::commands::pipes`: `PipeCatalog`, `PipeSeriesList`, `PipeSeriesGet`, `PipeSeriesImport`, `PipeSeriesExport`, `PipeSeriesDelete`, `PipeTypeCreate`.

## Decisions (where the spec is open)

### Model

- `PipeSeries { id, uid: Uid, code, name, short, standard, material, use_: PipeUse (serde rename "use"), joint, pressure: String, sources: Vec<SeriesSource>, pipes: Vec<PipeSize>, fittings: Vec<CatalogFitting> }`.
- `PipeSize { size: String, de, e, di }`.
- `CatalogFitting { kind, angle: Option<f64>, size, size2: Option<String>, socket, socket2, z, z2, z3, od: Option<f64>, derived: String }`.
- Lengths are meters. `PipeSeries::row(size) -> Option<&PipeSize>`.

### Bounds

| constant | value |
|---|---|
| `MAX_SERIES` | 200 |
| `MAX_SIZES` | 100 |
| `MAX_FITTINGS` | 2000 |
| `MAX_TEXT` | 200 chars |
| `MAX_SOURCES` | 50 |
| `MAX_FILE_BYTES` | 2 MB |
| `de` | 1 mm to 2 m, and `0 < e < de/2` |
| fitting lengths | 0 to 2 m |
| angles | 1–180 |

- `di` must be within 1 mm of `de − 2e` (the reader refuses otherwise); the built-in test is tighter, 0.15 mm.
- Unknown material, use, joint or kind in a file is an error, with the row.

### Pipe type fields

`TypeData::Pipe` gains:

- `#[serde(default, skip_serializing_if = "Option::is_none")] series: Option<PipeSeriesId>`
- `#[serde(default, skip_serializing_if = "String::is_empty")] size: String`
- `#[serde(default, skip_serializing_if = "Option::is_none")] inside: Option<f64>`

Two methods read it:

- `Project::pipe_row(&TypeData) -> Option<(&PipeSeries, &PipeSize)>`.
- `ElementType::inside_diameter(&Project) -> Option<f64>`: the row's `di`, else `inside`, else `None`. Sizing then falls back to its table.

### Sync and repair

- **Sync.** `EditCtx::finish` → `sync_pipe_series` runs first, before `sync_profiles`. It runs when settings changed or a pipe type changed:
  - a type whose row exists gets `diameter = de` (written only when it differs);
  - a type whose series is gone keeps `diameter` and drops `series` and `size`;
  - a type whose size is gone from its series also drops `series` and `size`.
- **Repair.** `repair_pipe_series` (in `Project::repair`):
  - bounds and drops bad rows;
  - makes codes unique by appending `-2`;
  - fixes uids;
  - drops fitting rows whose size or size2 is not a row;
  - drops dangling type references, keeping the diameter;
  - resyncs `diameter` from the rows.

### Built-in catalog

- Each file's name key is `pipe-series-<code lowercased>`, for example `pipe-series-br-pvc-sold`.
- Its value in each language is the series' `name`, translated. Translate the generic words, keep the standard codes.

### Import

`pipe_series_import`:

- **Source.** Exactly one of `code` (built-in), `path` or `text`.
- **`on_clash`** (default `skip`), when a series with the same code exists:
  - `skip` keeps the project's series;
  - `replace` overwrites its rows while keeping its id and uid, so the types re-sync;
  - `rename` imports it with the code suffixed.
- **`sizes`**: absent means no types; `["all"]` or a list of labels means types too.
- **Type names.** A type is named `"{name} {size}"`; an existing type of that name is reused and linked, not duplicated.
- **Result.** One undo step, returning `{series: id, created: [type ids], linked: [type ids]}`.

### Templates

- **Metric template.** Its pipe types attach to series copied from the catalog:
  - BR-PVC-SOLD sizes `20 mm`, `25 mm`, `32 mm`;
  - BR-PVC-ESG-N `DN 40`, `DN 50`, `DN 75`, `DN 100`.
- **Imperial template.** US-PVC-SCH40 `1/2"`, `3/4"`, `1"`; US-PVC-DWV `1-1/2"`, `2"`, `3"`, `4"`.
- **Names and diameters.** Type names keep their `tpl-*` keys. The cached diameter becomes the row's `de`, for example DN 100 → 101.6 mm and 1/2" → 21.34 mm. Tests pinned to the old diameters are updated, with the reason noted in the commit.
- **`template_spec`.** The pipe tuples gain the series code and size label. The `(key, diameter)` pairs stay, for the projects that predate building services (`project.rs:653`).
- **`template_spec` tests.** The tests that scan imperial sizes for round numbers skip pipe types of a series: their sizes are the standard's, not rounded.

### Format 29

`FORMAT_VERSION = 29`. On load of a file before 29:

- for each pipe type with no series whose name equals a template pipe type's localized name (through `translate`, the same one used for title blocks), for either system;
- and whose diameter is within 1 mm of that template type's old diameter;
- copy the template's series into the project (once) and link the type.

### Sizing (`cc_geom::sizing`)

- **Inside diameter.** `inside(outside)` becomes `inside_of(project, type)`: the type's `inside_diameter`, else the table, else ×0.88.
- **Supply runs of a series.** `required` is the `de` of the smallest row of the run's series whose `di` keeps speed ≤ `MAX_SPEED` (else the largest row). Runs with no series keep the `WATER_PIPES` table.
- **Drains of a series.** The smallest row with `de ≥ DN − 2 mm`.
- **Size label.** `PipeSizing` gains `size: Option<String>`, the row's label.

### `mep_size_apply`

For a run of a series:

- it takes the project's type of that series and row;
- else it creates `"{name} {size}"` in the same undo step;
- the result has `created`.

Runs without a series keep `pipe_type_for`, restricted to types without a series.

### Engine commands

- `pipe_catalog`:
  - with no `series`: `{series: [{code, name (translated), region, material, use, joint, pressure, standard, sizes, fittings}]}`;
  - with `series`: also `pipes` (mm) and `fittings` (mm).
- `pipe_series_list`: the project's series.
- `pipe_series_get`: one, by id, code or name, in the file form plus `id` and `types` (type ids using it).
- `pipe_series_export`: `series` plus `path` writes the file; with no `path`, returns `text`.
- `pipe_series_delete`: `series`, `detach` (bool). Refused with `in_use` while types use it, unless `detach`.
- `pipe_type_create`:
  - `series` (project series, else built-in code), `size`, optional `name`, `material`;
  - imports a built-in series when needed, all in one undo step;
  - returns `{id, series}`.
- `type_set` on a pipe type:
  - `series` takes a code or name, `null` to detach, resolved before the merge;
  - `size` is checked against the series (`unknown_size`).
- `element_get`/`type_get` of a pipe type: `derived` gains `inside_diameter`, `wall`, `series_code` and `size`.

## i18n keys (all 12 languages)

- `pipe-series-<code>` for the 22 built-in codes:
  - br-cobre-a, br-cobre-e, br-cpvc, br-ppr-pn20, br-pvc-esg-n, br-pvc-esg-r, br-pvc-sold;
  - eu-pe100-sdr11, eu-peld-pn4, eu-peld-pn6, eu-pvcu-1329, eu-pvcu-1401-sn4, eu-pvcu-1401-sn8, eu-pvcu-1452-pn10, eu-pvcu-1452-pn16;
  - us-cpvc-cts, us-cu-l, us-cu-m, us-pex, us-pvc-dwv, us-pvc-sch40, us-pvc-sch80.
- `pipe-region-brazil`, `pipe-region-europe`, `pipe-region-us`.
- Transactions: `tx-pipe-series-import`, `tx-pipe-series-delete`, `tx-pipe-type-create`.

Check before adding error messages: if the engine's existing error messages are English literals rather than keys, follow that.

## Tasks

### Task 1: Model, file format and catalog (cc_core)

Files:

- new `crates/cc_core/src/pipe_series.rs`, `pipe_file.rs`, `pipe_catalog.rs`;
- `lib.rs`, `ids.rs`, `project.rs` (field), `document.rs` (`pipe_series_mut`);
- `Cargo.toml` (add `serde_json` as a workspace dependency if it is not in cc_core);
- the spec, this plan and `assets/pipes/**` committed.

- [ ] Types and bounds as above. `PipeSeries::check() -> Result<(), PipeFileError>` is shared by the reader and repair.
- [ ] `pipe_file`:
  - serde structs in mm;
  - `read_series` refuses `format != 1`, too large, unknown enums and bad numbers, giving errors with a row index;
  - it converts to meters;
  - `write_series` writes the same JSON style as the assets (one row per line).
- [ ] `pipe_catalog`: the 22 `include_str!` entries, `OnceLock<Vec<PipeSeries>>`, skipping (and in debug, test-failing) a file that does not read.
- [ ] Tests:
  - every built-in reads;
  - codes are unique and match the file names;
  - `|di − (de − 2e)| ≤ 0.15 mm`;
  - fitting sizes exist, and size2 for reducers, reducing tees and wyes;
  - no duplicate fitting row (kind, angle, size, size2);
  - write → read round trip equals;
  - bad files: format 2, a 3 MB text, `de` NaN as a string, an unknown kind, size2 missing, `e ≥ de/2` are each refused with the right row.
- [ ] Fix data the tests reject, noting each fix in `assets/pipes/notes/*.md`.
- [ ] Commit: "Add pipe series: the model, the file format and the built-in catalog".

### Task 2: Pipe types, sync, repair, templates, format 29 (cc_core)

Files: `types.rs` (`TypeData::Pipe`), `project.rs` (template, `pipe_row`), `template_spec.rs`, `document.rs` (`sync_pipe_series` in `finish`), `repair.rs`, `persist.rs`, new `pipe_series_project.rs` (the `impl Project` helpers and tests).

- [ ] Fields, `pipe_row`, `inside_diameter`. Update the two `TypeData::Pipe { diameter, material }` constructors.
- [ ] `sync_pipe_series` and `repair_pipe_series` as decided.
  - Tests: editing a row's `de` in one `edit` changes its types' diameter in the same transaction, and undo restores both; a deleted series detaches its types; repair drops dangling rows and references.
- [ ] Templates (metric and imperial) with series.
  - Tests: every template pipe type has a series and its diameter equals the row's `de`; imperial template checks still pass with the decided exemption.
- [ ] Format 29 linking.
  - Test: a project text at format 28 with the old "Tubo PVC soldável 25 mm" type (diameter 0.025) and a free "Custom" pipe type → after load the first is linked to BR-PVC-SOLD `25 mm`, the second is free, and the series is copied once.
- [ ] Run `cargo nextest run -p cc_core` and fix fallout across the workspace (`cargo check --tests --workspace`), including tests pinned to old diameters.
- [ ] Commit.

### Task 3: Sizing by inside diameter (cc_geom, cc_engine `mep_size_apply`)

- [ ] `sizing.rs` as decided, `PipeSizing::size`.
  - Tests: a d63 EU-PVCU-1452-PN10 supply run uses `di` 57.0 (no 0.88 fallback). For a flow where the BR table would choose 60 mm, an EU series run chooses d63. Drain DN 100 (de 101.6) satisfies 100.
- [ ] `mep_size_apply` keeps the series and creates a missing type, in one step with `created`.
  - Engine test in `crates/cc_engine/tests/pipes.rs`.
- [ ] Commit.

### Task 4: Engine commands

Files: new `crates/cc_engine/src/commands/pipes.rs`, `commands/mod.rs`, `registry.rs`, `commands/types.rs` (`type_set` series and size; `derived`), `refs.rs` (`pipe_series` by id, code or name), i18n, `docs/commands.md`.

- [ ] The seven commands as decided.
- [ ] Tests in `tests/pipes.rs`:
  - the catalog lists 22 series, and one series' rows are in mm;
  - import by code with `sizes: ["all"]` creates the types, and a second import with `skip` creates nothing;
  - `replace` keeps ids;
  - import from `text` (a written series round trip) and from a bad text gives an error with no undo step;
  - export then import with `rename`;
  - delete is refused while in use, and `detach` keeps the types;
  - `pipe_type_create` from a built-in code imports the series once;
  - `type_set` series/size resizes, and an unknown size is refused;
  - `mep_sizing` shows `size`.
- [ ] Regenerate `docs/commands.md`. Add the i18n keys in all 12 languages.
- [ ] Commit.

### Task 5: Docs and full check

- [ ] `AGENTS.md`: a "Pipe series" bullet next to MEP (model, file format, catalog, sync, format 29, sizing, commands). `docs/automation.md` and `docs/automacao.md`: a short section with an import example.
- [ ] `cargo fmt --all`, clippy, `cargo nextest run --workspace`, `cargo test --workspace --doc`. All green.
- [ ] Commit.

## Self-review checklist

- The spec's stage-1 items are covered:
  - series in the project, file format, built-in catalog;
  - type fields and sync;
  - templates;
  - format 29;
  - sizing;
  - commands.
- Left to stage 2: `FittingKind::Wye`, matching, fitting geometry, cut lengths, `mep_fittings` names, schedules and IFC.
