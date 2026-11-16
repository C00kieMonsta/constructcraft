# `cc_ifc` API

{% raw %}
[← API reference](index.md) · Generated from the source by `cargo run -p api_doc`; edit the doc comments, not this page.

IFC export and import for ConstructCraft.
`export` maps a `cc_core::Project` to an IFC4 STEP physical file
(ISO 10303-21): spatial structure (project, site, building, storeys),
walls with openings, doors, windows, slabs, ceilings and roofs as swept
solids, element types, material layer sets, surface styles and base
quantities. The STEP writer and the mapping are implemented here without
third-party IFC libraries. `import` reads IFC2X3/IFC4 files back into a
project (see the `import` module).

## Modules

| Module | Items | Summary |
|---|---|---|
| [`cc_ifc`](#mod-cc_ifc) | 0 | IFC export and import for ConstructCraft. |
| [`cc_ifc::export`](#mod-cc_ifc-export) | 19 | Mapping of the ConstructCraft model to IFC4 entities. |
| [`cc_ifc::geometry`](#mod-cc_ifc-geometry) | 12 | The IFC geometry resource turned into polygon meshes: placements, curves (polylines, indexed curves with arcs, conics, trimmed and composite curves, B-splines), parametric and arbitrary profiles (with voids), swept solids (extrusions, revolutions, swept disks, directrix sweeps), CSG primitives and trees, half-space clipping, boundary representations, face sets and mapped items. |
| [`cc_ifc::guid`](#mod-cc_ifc-guid) | 3 | IFC GlobalId: a 128-bit identifier in the 22-character IFC base64 form. |
| [`cc_ifc::import`](#mod-cc_ifc-import) | 5 | IFC import: maps the spatial structure and building elements of an IFC file (IFC2X3 or IFC4) onto a ConstructCraft project. |
| [`cc_ifc::parse`](#mod-cc_ifc-parse) | 5 | Reader for STEP physical files (ISO 10303-21): the header schema and the entity instances of the data section. |
| [`cc_ifc::point_list`](#mod-cc_ifc-point_list) | 2 | Point lists as text, for the property sets that carry a terrain's points and boundary (`export::TOPOGRAPHY_PSET`): coordinates separated by spaces, points by `;`. |
| [`cc_ifc::schema`](#mod-cc_ifc-schema) | 4 | The part of the IFC4 schema needed to write any physical element class back out: the attribute count of each `IfcElement` subtype, where its `PredefinedType` sits and which type object goes with it. |
| [`cc_ifc::step`](#mod-cc_ifc-step) | 8 | Minimal writer for STEP physical files (ISO 10303-21). |

<a id="mod-cc_ifc"></a>

## `cc_ifc`

<sub>`crates/cc_ifc/src/lib.rs`</sub>

**Re-exports**

- `pub use export::{ExportOptions, export, export_to_file, ifc_status}`
- `pub use guid::guid`
- `pub use import::{ImportError, ImportReport, import, import_file}`
- `pub use schema::display_name`

<a id="mod-cc_ifc-export"></a>

## `cc_ifc::export`

<sub>`crates/cc_ifc/src/export.rs`</sub>

Mapping of the ConstructCraft model to IFC4 entities.

**Items:** [`NICHE_PSET`](#cc_ifc-export-niche_pset) · [`ExportOptions`](#cc_ifc-export-exportoptions) · [`export`](#cc_ifc-export-export) · [`export_to_file`](#cc_ifc-export-export_to_file) · [`COMPONENT_PSET`](#cc_ifc-export-component_pset) · [`DESIGN_OPTION_PSET`](#cc_ifc-export-design_option_pset) · [`PHASING_PSET`](#cc_ifc-export-phasing_pset) · [`CURTAIN_PSET`](#cc_ifc-export-curtain_pset) · [`ROOFING_PSET`](#cc_ifc-export-roofing_pset) · [`COLD_FORMED_CATEGORY`](#cc_ifc-export-cold_formed_category) · [`TOPOGRAPHY_PSET`](#cc_ifc-export-topography_pset) · [`PADS_PSET`](#cc_ifc-export-pads_pset) · [`PILE_CAP_PSET`](#cc_ifc-export-pile_cap_pset) · [`PILE_PSET`](#cc_ifc-export-pile_pset) · [`pile_ifc`](#cc_ifc-export-pile_ifc) · [`REBAR_PSET`](#cc_ifc-export-rebar_pset) · [`ROOFING_SIZES`](#cc_ifc-export-roofing_sizes) · [`PHASES_PSET`](#cc_ifc-export-phases_pset) · [`ifc_status`](#cc_ifc-export-ifc_status)

<a id="cc_ifc-export-niche_pset"></a>

### const `NICHE_PSET`

```rust
pub const NICHE_PSET: &str
```

Property set of a niche's opening element: its sizes, the face of the
wall it is in and the volume it takes out.

<a id="cc_ifc-export-exportoptions"></a>

### struct `ExportOptions`

```rust
pub struct ExportOptions
```

**Traits:** `Debug`, `Clone`, `Default`, `PartialEq`

**Fields**

- `time_stamp: Option<String>`

  `FILE_NAME` time stamp (ISO 8601). The current UTC time when `None`.

- `file_name: String`

  `FILE_NAME` name, usually the target file name.


<a id="cc_ifc-export-export"></a>

### fn `export`

```rust
pub fn export(project: &Project, options: &ExportOptions) -> String
```

Serialize `project` as an IFC4 STEP physical file: the main model and
the primary option of each design option set (whatever option is being
edited), the option of each element in `DESIGN_OPTION_PSET`.

<a id="cc_ifc-export-export_to_file"></a>

### fn `export_to_file`

```rust
pub fn export_to_file(project: &Project, path: &std::path::Path) -> std::io::Result<()>
```

Export `project` to an `.ifc` file at `path`.

<a id="cc_ifc-export-component_pset"></a>

### const `COMPONENT_PSET`

```rust
pub const COMPONENT_PSET: &str
```

Property set with the instance parameter values of components.

<a id="cc_ifc-export-design_option_pset"></a>

### const `DESIGN_OPTION_PSET`

```rust
pub const DESIGN_OPTION_PSET: &str
```

Property set of the design option an element belongs to (`OptionSet`,
`Option`, `Primary`).

<a id="cc_ifc-export-phasing_pset"></a>

### const `PHASING_PSET`

```rust
pub const PHASING_PSET: &str
```

Property set of an element's construction phases (`PhaseCreated`, `PhaseDemolished`).

<a id="cc_ifc-export-curtain_pset"></a>

### const `CURTAIN_PSET`

```rust
pub const CURTAIN_PSET: &str
```

Property set of a curtain wall's grid: the type's `System` and the wall's
`Edits` as text (see `cc_core::curtain::to_text`), its `Height` and `Bulge`.

<a id="cc_ifc-export-roofing_pset"></a>

### const `ROOFING_PSET`

```rust
pub const ROOFING_PSET: &str
```

Property set of a roof's covering and framing (see `write_roof_framing`).

<a id="cc_ifc-export-cold_formed_category"></a>

### const `COLD_FORMED_CATEGORY`

```rust
pub const COLD_FORMED_CATEGORY: &str
```

`Category` of the material profile of a cold-formed section (rolled and
cold-formed steel share their materials).

<a id="cc_ifc-export-topography_pset"></a>

### const `TOPOGRAPHY_PSET`

```rust
pub const TOPOGRAPHY_PSET: &str
```

A terrain's points, boundary, base depth and type settings, for importers
to rebuild it as terrain (points as `x y z` and the boundary as `x y`,
separated by `;`).

<a id="cc_ifc-export-pads_pset"></a>

### const `PADS_PSET`

```rust
pub const PADS_PSET: &str
```

The building pads of a terrain (see `Exporter::pads_set`).

<a id="cc_ifc-export-pile_cap_pset"></a>

### const `PILE_CAP_PSET`

```rust
pub const PILE_CAP_PSET: &str
```

Property set of a pile cap: its `Placement` (position in plan and
rotation in degrees), its `Cap` (thickness and `cc_core::pile::PileCap`
without the pile type) as text (see `cc_core::curtain::to_text`), and the
name of its piles' type (`PileType`).

<a id="cc_ifc-export-pile_pset"></a>

### const `PILE_PSET`

```rust
pub const PILE_PSET: &str
```

Property set of a pile: its `Kind` (`cc_core::pile::PileKind::code`), its
`Section` and `Embedment` as text, and for a pile of a cap the cap's
GlobalId (`Cap`) and the pile's place in its layout (`CapIndex`).

<a id="cc_ifc-export-pile_ifc"></a>

### fn `pile_ifc`

```rust
pub fn pile_ifc(kind: cc_core::pile::PileKind) -> (&'static str, &'static str, Option<&'static str>)
```

The `IfcPileTypeEnum`, `IfcPileConstructionEnum` and, for a user-defined
type, the object type of a pile kind: bored piles are bored and cast in
place, precast and steel piles driven; augered and micro piles, cast in
place, have no predefined type of their own.

<a id="cc_ifc-export-rebar_pset"></a>

### const `REBAR_PSET`

```rust
pub const REBAR_PSET: &str
```

What a reinforcement set says of its bars: `Partition`, `BarMark` (a mark
given by hand), `Mark` (the mark its bars have, when they all have one)
and `Grade` (see `Exporter::write_rebar`).

<a id="cc_ifc-export-roofing_sizes"></a>

### const `ROOFING_SIZES`

```rust
pub const ROOFING_SIZES: [(&str, &str, &str); 4]
```

The framing's member sizes in `ROOFING_PSET`: member, width and height properties.

<a id="cc_ifc-export-phases_pset"></a>

### const `PHASES_PSET`

```rust
pub const PHASES_PSET: &str
```

Property set of the project's phases, in time order (properties `1`, `2`…).

<a id="cc_ifc-export-ifc_status"></a>

### fn `ifc_status`

```rust
pub fn ifc_status(project: &Project, e: &Element) -> Option<cc_core::PhaseStatus>
```

An element's IFC `Status` relative to the project's last phase (an
element demolished before it reads as demolished); `None` for
annotations and projects without phases.

<a id="mod-cc_ifc-geometry"></a>

## `cc_ifc::geometry`

<sub>`crates/cc_ifc/src/geometry.rs`</sub>

The IFC geometry resource turned into polygon meshes: placements,
curves (polylines, indexed curves with arcs, conics, trimmed and
composite curves, B-splines), parametric and arbitrary profiles (with
voids), swept solids (extrusions, revolutions, swept disks, directrix
sweeps), CSG primitives and trees, half-space clipping, boundary
representations, face sets and mapped items.
Everything is read from public schema definitions; curved surfaces are
approximated by facets. Every walk over references is depth-bounded and
malformed values are skipped, so a hostile file yields less geometry,
never a panic.

**Items:** [`Polygon`](#cc_ifc-geometry-polygon) · [`Profile`](#cc_ifc-geometry-profile) · [`ProfileKind`](#cc_ifc-geometry-profilekind) · [`Extrusion`](#cc_ifc-geometry-extrusion) · [`Solid`](#cc_ifc-geometry-solid) · [`subtract`](#cc_ifc-geometry-subtract) · [`bounds2`](#cc_ifc-geometry-bounds2) · [`affine`](#cc_ifc-geometry-affine) · [`frame`](#cc_ifc-geometry-frame) · [`clean_loop`](#cc_ifc-geometry-clean_loop) · [`clip_mesh`](#cc_ifc-geometry-clip_mesh) · [`Geometry`](#cc_ifc-geometry-geometry)

<a id="cc_ifc-geometry-polygon"></a>

### type alias `Polygon`

```rust
pub type Polygon = Vec<DVec3>
```

A planar polygon in world coordinates (counter-clockwise seen from outside).

<a id="cc_ifc-geometry-profile"></a>

### struct `Profile`

```rust
pub struct Profile
```

A profile in its plane: outer loop, inner loops and, for parametric
rectangles and circles, their exact parameters.

**Traits:** `Debug`, `Clone`

**Fields**

- `pts: Vec<DVec2>`
- `holes: Vec<Vec<DVec2>>`
- `kind: ProfileKind`
- `closed: bool`

  False for open profiles (a curve swept into a surface).

- `curves: Option<Vec<SketchCurve>>`

  The outline and voids as lines, arcs and circles in the profile's
  plane, when every one of them reads so and some are curved (see
  `plan_curves`): `pts` and `holes` split the curves.


**Methods**

- `pub fn rect(&self) -> (DVec2, DVec2, f64, f64)`

  Exact rectangle or circle, else the bounding rectangle of the outline.

- `pub fn is_simple(&self) -> bool`

  A solid rectangle or circle (what native columns and beams can hold).


<a id="cc_ifc-geometry-profilekind"></a>

### enum `ProfileKind`

```rust
pub enum ProfileKind
```

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Variants**

- `Rect { center: DVec2, x: DVec2, w: f64, h: f64, }`

  Rectangle `w` x `h` centered at `center`, with its width along `x`.

- `Circle { center: DVec2, r: f64, }`
- `Poly`

<a id="cc_ifc-geometry-extrusion"></a>

### struct `Extrusion`

```rust
pub struct Extrusion
```

An extruded profile: its plane and extrusion vector in world coordinates.

**Traits:** `Debug`, `Clone`

**Fields**

- `profile: Profile`
- `xf: DAffine3`

  Profile plane to world.

- `vector: DVec3`

  Extrusion vector in world coordinates (direction times depth).

- `source: Option<u32>`

  The profile definition entity extruded.


**Methods**

- `pub fn is_vertical(&self) -> bool`
- `pub fn footprint(&self) -> Vec<DVec2>`

  World XY of the profile outline (the bottom cap of a vertical extrusion).

- `pub fn z_range(&self) -> (f64, f64)`

  Lowest and highest z of the extruded profile.

- `pub fn footprint_curves(&self) -> Option<Vec<SketchCurve>>`

  `Extrusion::footprint` and its voids as lines, arcs and circles,
  when the profile has them and lies level.

- `pub fn void_footprints(&self) -> Vec<Vec<DVec2>>`

  World XY of the profile's voids, like `Extrusion::footprint`.

- `pub fn plane_point(&self, p: DVec2) -> DVec3`
- `pub fn plane_vector(&self, v: DVec2) -> DVec3`

<a id="cc_ifc-geometry-solid"></a>

### struct `Solid`

```rust
pub struct Solid
```

A representation item resolved to world coordinates.

**Traits:** `Debug`, `Clone`, `Default`

**Fields**

- `extrusion: Option<Extrusion>`

  The extruded profile, when the item is a plain extrusion.

- `mesh: Vec<Polygon>`

  Faces in world coordinates.

- `points: Vec<DVec3>`

  Every vertex of the faces.

- `item: u32`

  Representation item (for styles).

- `clipped: bool`

  A half space or boolean cut the item (its extrusion is the uncut one).

- `cuts: Vec<Extrusion>`

  The plain extrusions a boolean difference subtracted from it (a slab
  with its shafts cut out of its body).

- `other_cut: bool`

  Something other than those extrusions changed it (half spaces,
  intersections, cutters of other shapes).

- `disk: Option<(Vec<DVec3>, f64)>`

  The center line (world) and radius of a disk swept along it (bars).


**Methods**

- `pub fn replace_mesh(&mut self, mesh: Vec<Polygon>)`

  Replace the faces by a cut version of them; a changed solid is
  marked clipped (its extrusion stays the uncut one).


<a id="cc_ifc-geometry-subtract"></a>

### fn `subtract`

```rust
pub fn subtract(a: &[Polygon], b: &[Polygon]) -> Option<Vec<Polygon>>
```

`a` minus `b` when their bounds overlap (`None`: nothing to cut or too large to cut).

<a id="cc_ifc-geometry-bounds2"></a>

### fn `bounds2`

```rust
pub fn bounds2(pts: &[DVec2]) -> (DVec2, DVec2)
```

<a id="cc_ifc-geometry-affine"></a>

### fn `affine`

```rust
pub fn affine(x: DVec3, y: DVec3, z: DVec3, origin: DVec3) -> DAffine3
```

<a id="cc_ifc-geometry-frame"></a>

### fn `frame`

```rust
pub fn frame(z: Option<DVec3>, x: Option<DVec3>, origin: DVec3) -> DAffine3
```

A right-handed frame from a Z axis and an approximate X axis.

<a id="cc_ifc-geometry-clean_loop"></a>

### fn `clean_loop`

```rust
pub fn clean_loop<T: Copy + PartialEq>(mut pts: Vec<T>, close: impl Fn(&T, &T) -> bool) -> Vec<T>
```

Drop a repeated closing point and consecutive duplicates.

<a id="cc_ifc-geometry-clip_mesh"></a>

### fn `clip_mesh`

```rust
pub fn clip_mesh(mesh: &[Polygon], point: DVec3, normal: DVec3) -> Vec<Polygon>
```

Keep the part of `mesh` on the side of the plane where `(p - point)·normal >= 0`,
closing the cut with cap faces.

<a id="cc_ifc-geometry-geometry"></a>

### struct `Geometry`

```rust
pub struct Geometry<'a>
```

Reads geometry from a parsed IFC file.

**Fields**

- `f: &'a StepFile`
- `scale: f64`

  Meters per file length unit.

- `angle: f64`

  Radians per file plane angle unit.


*Some fields are private.*

**Methods**

- `pub fn new(f: &'a StepFile, scale: f64, angle: f64) -> Self`
- `pub fn measured_curve(&self, param: &Param) -> Vec<DVec3>`

  A curve whose length is measured (a bar's center line): its arcs are
  sampled at 2° rather than at the facets of meshes.

- `pub fn point(&self, param: &Param) -> Option<DVec3>`
- `pub fn direction(&self, param: &Param) -> Option<DVec3>`
- `pub fn axis_placement(&self, param: &Param) -> DAffine3`

  `IfcAxis2Placement3D` / `2D` / `IfcAxis1Placement` as a transform (identity if unset).

- `pub fn placement(&self, param: &Param) -> DAffine3`

  World transform of an object placement chain.

- `pub fn operator(&self, param: &Param) -> DAffine3`

  `IfcCartesianTransformationOperator2D/3D` (uniform or not) as a transform.

- `pub fn curve(&self, param: &Param, depth: usize) -> Vec<DVec3>`

  Points of a curve in its own coordinates (2D curves at z = 0).

- `pub fn plan_curves(&self, param: &Param, depth: usize) -> Option<Vec<SketchCurve>>`

  A 2D curve as lines, arcs and circles in its own coordinates (polylines,
  indexed poly curves, composite curves of those, circles and trimmed
  circles and lines); `None` when any part is something else (ellipses,
  splines, offsets) or the curves are too many for a sketch.

- `pub fn profiles(&self, param: &Param, depth: usize) -> Vec<Profile>`

  The profiles of a profile definition (several for composite profiles).

- `pub fn parametric_shape(&self, e: &Entity) -> Option<ProfileShape>`

  The structural profile shape of a parametric profile definition,
  when the shape holds it exactly (in meters, before its position);
  `None` for other definitions and for what the shapes lack (edge
  radii, slopes, a hollow rectangle's own inner radius).

- `pub fn items(&self, param: &Param, xf: DAffine3, out: &mut Vec<Solid>)`

  Resolve a representation item into world solids (`xf`: item coordinates to world).


<a id="mod-cc_ifc-guid"></a>

## `cc_ifc::guid`

<sub>`crates/cc_ifc/src/guid.rs`</sub>

IFC GlobalId: a 128-bit identifier in the 22-character IFC base64 form.

**Items:** [`encode`](#cc_ifc-guid-encode) · [`decode`](#cc_ifc-guid-decode) · [`guid`](#cc_ifc-guid-guid)

<a id="cc_ifc-guid-encode"></a>

### fn `encode`

```rust
pub fn encode(n: u128) -> String
```

Encode a 128-bit value: 2 bits in the first character, 6 in each of the other 21.

<a id="cc_ifc-guid-decode"></a>

### fn `decode`

```rust
pub fn decode(s: &str) -> Option<u128>
```

Decode a 22-character GlobalId (`None` if malformed).

<a id="cc_ifc-guid-guid"></a>

### fn `guid`

```rust
pub fn guid(seed: &str, tag: &str) -> String
```

Deterministic GlobalId for the object named `tag` within the project
identified by `seed`, so repeated exports keep the same ids.

<a id="mod-cc_ifc-import"></a>

## `cc_ifc::import`

<sub>`crates/cc_ifc/src/import.rs`</sub>

IFC import: maps the spatial structure and building elements of an IFC
file (IFC2X3 or IFC4) onto a ConstructCraft project.
Geometry is read from extruded area solids (the usual representation of
walls, slabs, columns and beams), with faceted B-reps, face sets and
bounding boxes used for extents. Storeys become levels; walls, doors,
windows, slabs, ceilings, roofs, spaces, columns and beams become native
elements with matching or new types. Products that cannot be interpreted
are counted in the report instead of failing the import.

**Items:** [`MAX_IFC_BYTES`](#cc_ifc-import-max_ifc_bytes) · [`ImportReport`](#cc_ifc-import-importreport) · [`ImportError`](#cc_ifc-import-importerror) · [`import_file`](#cc_ifc-import-import_file) · [`import`](#cc_ifc-import-import)

<a id="cc_ifc-import-max_ifc_bytes"></a>

### const `MAX_IFC_BYTES`

```rust
pub const MAX_IFC_BYTES: u64
```

Largest IFC file imported, bytes: the parsed instances take many times
the file's size in memory.

<a id="cc_ifc-import-importreport"></a>

### struct `ImportReport`

```rust
pub struct ImportReport
```

What an import brought in.

**Traits:** `Debug`, `Clone`, `Default`, `PartialEq`

**Fields**

- `schema: String`

  IFC schema of the file (`IFC4`, `IFC2X3`…).

- `levels: usize`
- `imported: BTreeMap<Category, usize>`
- `generic: BTreeMap<String, usize>`

  Objects brought in as generic components keeping their IFC class, by class.

- `skipped: BTreeMap<String, usize>`

  Products that were not imported, by IFC entity name.


**Methods**

- `pub fn imported_total(&self) -> usize`
- `pub fn skipped_total(&self) -> usize`

<a id="cc_ifc-import-importerror"></a>

### enum `ImportError`

```rust
pub enum ImportError
```

**Traits:** `Debug`, `std::fmt::Display`, `std::error::Error`

**Variants**

- `Io(std::io::Error)`
- `Parse(ParseError)`
- `NotIfc`

  The file has no `IFCPROJECT`.


<a id="cc_ifc-import-import_file"></a>

### fn `import_file`

```rust
pub fn import_file(path: &std::path::Path, base: Project) -> Result<(Project, ImportReport), ImportError>
```

Import the IFC file at `path` into `base` (see `import`).
Files larger than `MAX_IFC_BYTES` are refused before they are read
(an `ImportError::Io` of kind `FileTooLarge`).

<a id="cc_ifc-import-import"></a>

### fn `import`

```rust
pub fn import(text: &str, base: Project) -> Result<(Project, ImportReport), ImportError>
```

Import IFC `text`. `base` supplies materials, types and project-wide
views (usually the default template); its levels, elements, plans and
sheets are replaced by the imported building.

<a id="mod-cc_ifc-parse"></a>

## `cc_ifc::parse`

<sub>`crates/cc_ifc/src/parse.rs`</sub>

Reader for STEP physical files (ISO 10303-21): the header schema and the
entity instances of the data section. Malformed input yields an error,
never a panic; nesting depth is bounded.

**Items:** [`Param`](#cc_ifc-parse-param) · [`Entity`](#cc_ifc-parse-entity) · [`StepFile`](#cc_ifc-parse-stepfile) · [`ParseError`](#cc_ifc-parse-parseerror) · [`parse`](#cc_ifc-parse-parse)

<a id="cc_ifc-parse-param"></a>

### enum `Param`

```rust
pub enum Param
```

A parsed attribute value.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Variants**

- `Null`
- `Derived`
- `Int(i64)`
- `Real(f64)`
- `Str(String)`
- `Enum(String)`

  Enumeration literal without the dots (`T`, `F`, `ELEMENT`…).

- `Ref(u32)`
- `List(Vec<Param>)`
- `Typed(String, Box<Param>)`

  Typed value of a SELECT, such as `IFCLABEL('x')`.


**Methods**

- `pub fn as_ref(&self) -> Option<u32>`
- `pub fn as_f64(&self) -> Option<f64>`

  Numeric value (integers widen; typed measures unwrap).

- `pub fn as_str(&self) -> Option<&str>`
- `pub fn as_enum(&self) -> Option<&str>`
- `pub fn as_bool(&self) -> Option<bool>`
- `pub fn as_list(&self) -> &[Param]`

<a id="cc_ifc-parse-entity"></a>

### struct `Entity`

```rust
pub struct Entity
```

An entity instance: upper-case type name and attributes. Complex
(multi-type) instances are kept with an empty name.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `name: String`
- `args: Vec<Param>`

**Methods**

- `pub fn arg(&self, i: usize) -> &Param`

  Attribute `i`, or `Null` when missing.


<a id="cc_ifc-parse-stepfile"></a>

### struct `StepFile`

```rust
pub struct StepFile
```

A parsed exchange structure.

**Traits:** `Debug`, `Clone`, `Default`

**Fields**

- `schema: String`

  First schema of `FILE_SCHEMA` (e.g. `IFC4`, `IFC2X3`).

- `entities: BTreeMap<u32, Entity>`

**Methods**

- `pub fn get(&self, id: u32) -> Option<&Entity>`
- `pub fn deref(&self, param: &Param) -> Option<&Entity>`

  The instance `param` refers to.

- `pub fn of_type<'a>(&'a self, name: &'a str) -> impl Iterator<Item = (u32, &'a Entity)> + 'a`

  Instances of entity `name` (upper case), in file order.


<a id="cc_ifc-parse-parseerror"></a>

### struct `ParseError`

```rust
pub struct ParseError
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Eq`, `std::fmt::Display`, `std::error::Error`

**Fields**

- `offset: usize`

  Byte offset of the problem.

- `message: String`

<a id="cc_ifc-parse-parse"></a>

### fn `parse`

```rust
pub fn parse(text: &str) -> Result<StepFile, ParseError>
```

Parse a STEP physical file.

<a id="mod-cc_ifc-point_list"></a>

## `cc_ifc::point_list`

<sub>`crates/cc_ifc/src/point_list.rs`</sub>

Point lists as text, for the property sets that carry a terrain's points
and boundary (`export::TOPOGRAPHY_PSET`): coordinates separated by
spaces, points by `;`.

**Items:** [`encode`](#cc_ifc-point_list-encode) · [`decode`](#cc_ifc-point_list-decode)

<a id="cc_ifc-point_list-encode"></a>

### fn `encode`

```rust
pub(crate) fn encode(points: impl Iterator<Item = Vec<f64>>) -> String
```

Points as text: coordinates separated by spaces, points by `;`.

<a id="cc_ifc-point_list-decode"></a>

### fn `decode`

```rust
pub(crate) fn decode(text: &str, dims: usize, max: usize) -> Vec<Vec<f64>>
```

Points read from `encode` text, `dims` numbers each: at most `max` of
them; malformed or non-finite entries are skipped.

<a id="mod-cc_ifc-schema"></a>

## `cc_ifc::schema`

<sub>`crates/cc_ifc/src/schema.rs`</sub>

The part of the IFC4 schema needed to write any physical element class
back out: the attribute count of each `IfcElement` subtype, where its
`PredefinedType` sits and which type object goes with it. Taken from the
published IFC4 ADD2 TC1 schema; all the attributes after the common eight
are optional, so the writer fills them with `$`.

**Items:** [`TypeLayout`](#cc_ifc-schema-typelayout) · [`ElementClass`](#cc_ifc-schema-elementclass) · [`element_class`](#cc_ifc-schema-element_class) · [`display_name`](#cc_ifc-schema-display_name)

<a id="cc_ifc-schema-typelayout"></a>

### enum `TypeLayout`

```rust
pub enum TypeLayout
```

Extra attributes of a type object after the nine of `IfcElementType`.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Variants**

- `Predefined`

  `PredefinedType`.

- `Plain`

  No further attributes.

- `Furniture`

  `AssemblyPlace`, `PredefinedType` (furniture).

- `Door`

  `PredefinedType`, `OperationType`, `ParameterTakesPrecedence`, `UserDefinedOperationType`.

- `Window`

  `PredefinedType`, `PartitioningType`, `ParameterTakesPrecedence`, `UserDefinedPartitioningType`.


<a id="cc_ifc-schema-elementclass"></a>

### struct `ElementClass`

```rust
pub struct ElementClass
```

How an element class is written.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Fields**

- `entity: &'static str`
- `attrs: usize`

  Number of attributes of an occurrence.

- `predefined: Option<usize>`

  Index of `PredefinedType` in an occurrence.

- `type_entity: Option<(&'static str, TypeLayout)>`

  Type object entity (none when the schema only has an abstract one).


<a id="cc_ifc-schema-element_class"></a>

### fn `element_class`

```rust
pub fn element_class(entity: &str) -> Option<&'static ElementClass>
```

How to write occurrences of `entity` (upper case) in IFC4, if it is a
physical element class there.

<a id="cc_ifc-schema-display_name"></a>

### fn `display_name`

```rust
pub fn display_name(entity: &str) -> String
```

Readable class name: `IFCPIPESEGMENT` → `IfcPipeSegment` for the classes
in the table, else the entity name with only its first letters capitalized.

<a id="mod-cc_ifc-step"></a>

## `cc_ifc::step`

<sub>`crates/cc_ifc/src/step.rs`</sub>

Minimal writer for STEP physical files (ISO 10303-21).

**Items:** [`Id`](#cc_ifc-step-id) · [`Value`](#cc_ifc-step-value) · [`opt_str`](#cc_ifc-step-opt_str) · [`reals`](#cc_ifc-step-reals) · [`args`](#cc_ifc-step-args) · [`Header`](#cc_ifc-step-header) · [`StepWriter`](#cc_ifc-step-stepwriter) · [`is_type_name`](#cc_ifc-step-is_type_name)

<a id="cc_ifc-step-id"></a>

### struct `Id`

```rust
pub struct Id(pub u32)
```

Instance name of an entity in the data section (`#n`).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`

<a id="cc_ifc-step-value"></a>

### enum `Value`

```rust
pub enum Value
```

An attribute value.

**Traits:** `Debug`, `Clone`, `PartialEq`, `From<f64>`, `From<i64>`, `From<bool>`, `From<&str>`, `From<String>`, `From<Id>`, `From<Option<T>>`, `From<Vec<T>>`

**Variants**

- `Null`

  Unset optional attribute (`$`).

- `Derived`

  Attribute redeclared as derived in a subtype (`*`).

- `Bool(bool)`
- `Int(i64)`
- `Real(f64)`
- `Str(String)`
- `Enum(&'static str)`

  Enumeration literal, written as `.NAME.`.

- `EnumName(String)`

  Enumeration literal read from data; anything but `A-Z`, `0-9` and `_`
  is written as `NOTDEFINED`.

- `Ref(Id)`
- `List(Vec<Value>)`
- `Typed(&'static str, Box<Value>)`

  Typed value for SELECT attributes, such as `IFCLABEL('x')`.

- `TypedName(String, Box<Value>)`

  Typed value whose type name comes from data; an invalid name (not
  `IFC` followed by `A-Z`, `0-9`, `_`) is written as `$`.


<a id="cc_ifc-step-opt_str"></a>

### fn `opt_str`

```rust
pub fn opt_str(s: &str) -> Value
```

A string attribute, or `$` when empty.

<a id="cc_ifc-step-reals"></a>

### fn `reals`

```rust
pub fn reals(xs: &[f64]) -> Value
```

A list of reals, for coordinates and direction ratios.

<a id="cc_ifc-step-args"></a>

### macro `args`

```rust
macro_rules! args
```

Builds the attribute list of an entity instance.

<a id="cc_ifc-step-header"></a>

### struct `Header`

```rust
pub struct Header
```

File header fields (`FILE_DESCRIPTION`, `FILE_NAME`, `FILE_SCHEMA`).

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `description: Vec<String>`
- `name: String`
- `time_stamp: String`
- `author: Vec<String>`
- `organization: Vec<String>`
- `preprocessor_version: String`
- `originating_system: String`
- `authorization: String`
- `schema: String`

<a id="cc_ifc-step-stepwriter"></a>

### struct `StepWriter`

```rust
pub struct StepWriter
```

Accumulates entity instances and serializes the exchange structure.

**Traits:** `Debug`, `Default`

*Some fields are private.*

**Methods**

- `pub fn new() -> Self`
- `pub fn add(&mut self, entity: &str, args: Vec<Value>) -> Id`

  Append an instance of `entity` (upper case, e.g. `IFCWALL`) and return its name.

- `pub fn len(&self) -> usize`
- `pub fn is_empty(&self) -> bool`
- `pub fn finish(self, header: &Header) -> String`

<a id="cc_ifc-step-is_type_name"></a>

### fn `is_type_name`

```rust
pub fn is_type_name(name: &str) -> bool
```

An IFC type name: `IFC` followed by upper-case letters, digits and `_`.

{% endraw %}
