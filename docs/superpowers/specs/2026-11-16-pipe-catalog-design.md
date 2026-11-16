# Pipe and fitting catalog (issue #383)

The user asked for "um catálogo de tubos e conexões para MEP hidráulico".
Issue #383 asks for European pipe series (PVC-U pressure EN ISO 1452, LDPE
irrigation, PVC-U drainage EN 1329 / EN 1401) and their inside diameters in
sizing. Scope chosen by the user (2026-11-16):

- **Pipes:** a built-in catalog plus importing series from a file.
- **Fittings:** a catalog with dimensions per series. The fittings the
  network derives match a catalog row, which gives them a real name, real
  geometry and a warning when the fitting does not exist.
- **Series:** Brazil, Europe and the US.

## Today

- `TypeData::Pipe { diameter, material }` stores only the outside
  diameter. The template makes 7 types from `template_spec::pipes`.
- `cc_geom::sizing` reads inside diameters from `WATER_PIPES`, a fixed
  Brazilian table. Outside it, the inside diameter is the outside one × 0.88.
  `mep_size_apply` picks the smallest pipe type of the whole project that
  is wide enough, mixing supply and drain types.
- Fittings are derived (`mep_net::FittingKind`: elbow, coupling, reducer,
  tee, cross). They have no type, name or dimensions.
  - Each socket is drawn as a cylinder of 0.6·d radius and 0.8·d length.
  - `mep_fittings` counts them by kind and diameter.
  - IFC export writes `IfcPipeFitting` with one type per kind.

## Goals

### Series in the project

`cc_core::pipe_series` adds `PipeSeries` to `Project::pipe_series`
(`BTreeMap<PipeSeriesId, PipeSeries>`). It is tracked as settings and uses
`EditCtx::pipe_series_mut`.

A series holds:

- id, `Uid`, a stable `code` (unique in the project), `name` (pipe type
  name prefix), `short` (used in fitting names), `standard`;
- `material` (`PipeMaterial`: PVC, CPVC, PPR, PE, PEX, Copper, Steel,
  CastIron), `use` (`PipeUse`: Supply, Drain, Any), `joint` (`PipeJoint`),
  `pressure` text, `sources`;
- `pipes: Vec<PipeSize>`, each `{ size, de, e, di }`. Lengths are meters in
  the model and mm in files.
- `fittings: Vec<CatalogFitting>`, each `{ kind, angle, size, size2,
  socket, socket2, z, z2, z3, od }`. `z3` is the laying length of the
  second end of the main run when it differs from `z` (sanitary tees and
  wyes; `z` is the outlet end). The dimensions are optional. `derived`
  says when a value came from a rule.

Series are copied into the project when they are first used, the same way
catalog profiles become `ProfileDef`s. A project therefore keeps working
when the built-in catalog changes, and files carry the series they use.
`repair_pipe_series` bounds sizes and counts (`MAX_SIZES`, `MAX_FITTINGS`,
`MAX_SERIES`). It drops rows whose `size` or `size2` does not exist and
resets dangling type references.

### Pipe types

`TypeData::Pipe` gains two fields, both with serde defaults:

- `series: Option<PipeSeriesId>`
- `size: String`

A type of a series has its `diameter` (and the new derived
`inside_diameter()`) from the series row. `EditCtx::finish` →
`sync_pipe_series` keeps the cached `diameter` in step inside the same
transaction, the way `sync_profiles` does. Editing a series' row resizes
its types in one undo step.

A type with no series keeps today's behavior. It may get an
`inside: Option<f64>` override.

`ElementType::inside_diameter(project)` returns:

1. the series row's `di`, else
2. `inside`, else
3. the existing table fallback.

### Built-in catalog (`cc_core::pipe_catalog`)

- The series are JSON files in `crates/cc_core/assets/pipes/`, compiled
  with `include_str!` and parsed once (`OnceLock`).
- Built-in files and imported files share one format, `pipe_file.rs`:
  format 1, mm, the schema of `FORMAT.md` in the research notes. Reading
  is bounded and errors are values with the row.
- Each built-in file is registered in Rust with an i18n key for its name
  (`pipe-series-<code>`), so the browser and dialogs show it translated.
- Pipe type and fitting names stay in the market's language (data, like
  profile designations), except the fitting kind words, which are i18n.

Series:

| region | series |
|---|---|
| Brazil | PVC soldável NBR 5648; PVC esgoto NBR 5688 normal and reforçada; CPVC NBR 15884; PPR NBR 15813; copper NBR 13206; PEAD where public data allows |
| Europe | PVC-U EN ISO 1452 PN10 and PN16; LDPE PN4 and PN6; PVC-U EN 1329-1; PVC-U EN 1401-1 SN4 and SN8; optionally PE100 SDR11 EN 12201 and copper EN 1057 |
| US | PVC Sch 40 and Sch 80 ASTM D1785; PVC DWV ASTM D2665; copper ASTM B88 type L and M; PEX ASTM F876; CPVC CTS ASTM D2846 |

Rules for the data:

- Every value comes from cited public documents (clean room). Sources
  stay in the file's `sources`.
- A fitting dimension no source gives stays `null`. Geometry then uses
  rules: socket = de·0.8, z = de·0.5 for elbows and tees, body = de·1.2.
- A test checks every built-in file:
  - it parses;
  - `di ≈ de − 2e` within 0.15 mm;
  - fitting sizes exist in the series;
  - no fitting row repeats.

### Template

- **Metric template:** pipe types of the BR series. They keep today's
  names and keys. Supply = PVC soldável 20/25/32, drain = esgoto normal
  40/50/75/100, with their series attached.
- **Imperial template:** the US series (Sch 40 1/2", 3/4", 1"; DWV
  1-1/2", 2", 3", 4").
- The series are copied into the template's project.
- **Format 29:** older files get the template's series. Their template
  pipe types are linked to a series row when the name and diameter match a
  template type. Other types stay free.

### Fittings matched to the catalog (`cc_geom::mep_net`)

- `FittingKind` gains `Wye`: three branches where the branch meets the
  run at 45° ± 7.5°.
- A fitting's runs give their types' series. When every run of a fitting
  is of the same series, `catalog::match_fitting` returns `Matched`, a
  `Composite` or `Missing`:
  - **Matched:** one row. An elbow takes the nearest catalog angle within
    7.5°. A tee or wye matches by main size and branch size. A coupling,
    a reducer and a cross match by sizes.
  - **Composite:** a reducing tee that is not sold is a tee plus a
    reducer on the branch. A reducer that skips sizes is a chain of
    reducers, at most 3. Each piece is its own row.
  - **Missing:** no row matches, for example a 60° elbow or a size the
    series lacks. The fitting keeps its kind with `missing: true` and a
    reason key.
- Runs of different series, or with no series, give a fitting with no
  catalog. That is today's behavior.
- `Fitting` gains `catalog: Option<FittingMatch>`: series id, rows and
  status. The name comes from `fitting_name(kind, angle, series.short,
  sizes)`, for example "Joelho 90° PVC soldável 25 mm" or
  "Elbow 90° PVC Sch 40 1/2"".

### Geometry

For a matched fitting, each branch is a cylinder from the center to
z + socket, of radius od/2 (or rules). Missing and uncatalogued fittings
keep today's sockets.

- **Pipe cut length:** `ModelGeometry::pipe_cut_lengths` gives each pipe
  run its run length minus the z of the matched fittings at its ends.
  Schedules show it as `CutLength`. Pipe bodies keep running to the node,
  which is the usual BIM representation; only the takeoff changes.

### Sizing

`size_pipes` uses `inside_diameter` of the run's type.

The required size is the smallest row of the run's series that keeps the
speed ≤ `MAX_SPEED`:

- Supply runs pick by di.
- Drain runs pick by de ≥ the required DN.

Runs without a series keep the `WATER_PIPES` table. `PipeSizing` gains
`size: Option<String>`.

`mep_size_apply` picks among the project's types of the run's series. It
creates a missing type of that series' row in the same undo step,
reported as `created`. It no longer mixes series.

### Engine commands (`cc_engine::commands::pipes`)

| command | does |
|---|---|
| `pipe_catalog` | lists the built-in series (code, name, region, material, use, standard), or one series' sizes and fittings with `series` |
| `pipe_series_list` / `pipe_series_get` | the project's series |
| `pipe_series_import` | from the built-in catalog (`code`) or a file (`path` or `text`). `on_clash`: skip, replace or rename. `sizes`: also make pipe types of those sizes (`"all"`). One undo step, returns `{series, types}` |
| `pipe_series_export` | writes a project series in the file format |
| `pipe_series_delete` | refused while types use it unless `detach` is set, which keeps the types free |
| `pipe_type_create` | `series` (code or name) plus `size`: makes the type (named `name size`) and imports the series when it is not in the project |
| `type_set` | takes `series` and `size` for pipe types |
| `mep_fittings` | each item gains `name`, `series`, `size`/`size2`, `missing` and a reason. Composites list their pieces |
| `mep_sizing` | gains the size |

### Schedules

- Pipes gain `InsideDiameter`, `Series`, `Size` and `CutLength`.
- A **fitting schedule**: `ScheduleDef::source` gets `Fittings`. Fittings
  are not elements, so the schedule reads `ModelGeometry::mep.fittings`.
  Fields: `FittingName`, `FittingKind`, `Size`, `System`, `Count`. It is
  grouped by name by default, which gives the takeoff.
  - If `schedule.rs`'s element loop cannot take a second source cheaply,
    this moves to a follow-up and `mep_fittings` plus CSV export cover the
    takeoff.

### IFC

- Pipe segment types write `ConstructCAD_PipeSeries`: Code, Size, Standard,
  OutsideDiameter, InsideDiameter, WallThickness, Pressure.
- Matched fittings:
  - Each is typed by an `IfcPipeFittingType` per catalog row, named with
    the fitting name. Composites write one fitting per piece, placed along
    the branch.
  - They use the catalog geometry, and `PredefinedType` comes from the
    kind (BEND, JUNCTION, CONNECTOR, TRANSITION).
- Import: IFC import makes no native pipes today, so nothing changes.

### App

- **Type Properties of a pipe type:**
  - a *Series* picker: the project's series, plus *Load from catalog…*;
  - a *Size* picker of that series' rows;
  - read-only de/e/di/pressure;
  - a free type keeps its diameter field and gets *Inside diameter*.
- **The Pipe Catalog dialog** (`pipe_catalog_dialog.rs`, under
  *Systems → Plumbing → Pipe Catalog* and *Manage → Pipe Series*):
  - a list of built-in and project series, filtered by region and use;
  - the sizes and fittings table of the selected series;
  - *Load series*, *Create types* (sizes checked), *Import file…* and
    *Export…*;
  - OK/Cancel through `dialog::session`.
- The pipe tool's type list shows the series as groups.
- The fittings found by the tool are named in the status and palette;
  missing ones get a warning badge.

## Out of scope (follow-ups)

- Placed fittings as component elements (`ComponentKind::PipeFitting`,
  `IfcPipeFitting` with `IfcDistributionPort`s) and free connector
  directions. The issue's first and second notes. Derived fittings cover
  the takeoff and IFC now.
- The `constructcraft-components` source names that clash with the
  template (*Janela de abrir* / *de correr*). That is another repository.
- Pipe insulation, hangers, and pressure-loss sizing (Hazen-Williams /
  Fair-Whipple-Hsiao).
- IFC import of native pipes.

## Tests

- **cc_core:**
  - file parse (bad files, bounds);
  - every built-in series' checks;
  - `sync_pipe_series` resizes types in one step;
  - repair of dangling series and rows;
  - format 29 linking of old template types.
- **cc_geom:**
  - wye detection;
  - matched, composite and missing fittings (elbow 60° missing, reducing
    tee composite);
  - catalog geometry volume;
  - cut lengths;
  - sizing by a series' inside diameters (the issue's d63/d90 case).
- **cc_engine:**
  - every command;
  - `mep_size_apply` keeps the series and creates a missing type;
  - `mep_fittings` names;
  - the regenerated `docs/commands.md`.
- **cc_ifc:**
  - the property set;
  - fitting types per row;
  - an IFC validation fixture `pipes.ccad` added to CI.
- **cc_app:**
  - the dialog in `every_dialog_keeps_its_body_inside`;
  - a UI test that loads a series and creates types.

## Plan shape

Three PRs, each with its own plan section:

1. **Core and data.**
   - `pipe_series`, the file format, the catalog with the researched
     JSON, the type fields, sync, repair, the template and format 29.
   - The `pipe_catalog`, `pipe_series_*` and `pipe_type_create` commands.
   - Sizing by the inside diameter.
2. **Fittings.**
   - Wye detection, matching, geometry and cut lengths.
   - `mep_fittings`, `mep_size_apply`, schedules and IFC.
3. **App.** Type Properties, the Pipe Catalog dialog, the tool's grouping,
   the docs and i18n in every language.
