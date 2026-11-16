# `cc_skp` API

{% raw %}
[← API reference](index.md) · Generated from the source by `cargo run -p api_doc`; edit the doc comments, not this page.

SketchUp (.skp) files: reader (current and legacy formats, scenes and
per-definition instanced scenes) and writer (`create`, `edit`).
Derived from the MIT-licensed OpenSKP toolkit (see `LICENSE`); no external
dependencies. Lengths in the parsed model are inches, as in the file.
```rust
use cc_skp::{ParseOptions, SkpFile};
let model = SkpFile::open("building.skp")?.parse(&ParseOptions::default())?;
println!("{} {} definitions", model.version, model.definitions.len());
```

## Modules

| Module | Items | Summary |
|---|---|---|
| [`cc_skp`](#mod-cc_skp) | 0 | SketchUp (.skp) files: reader (current and legacy formats, scenes and per-definition instanced scenes) and writer (`create`, `edit`). |
| [`cc_skp::create`](#mod-cc_skp-create) | 27 | Create new legacy-format (v17) `.skp` files from scratch. |
| [`cc_skp::create::archive`](#mod-cc_skp-create-archive) | 16 | `ArchiveWriter` - the write-side mirror of the legacy reader's MFC `CArchive` slot/class-ref bookkeeping, plus every entity record encoder the builder uses. |
| [`cc_skp::create::geometry`](#mod-cc_skp-create-geometry) | 13 | Small geometry/linear-algebra helpers for the writer. |
| [`cc_skp::create::scaffold_blank_v17`](#mod-cc_skp-create-scaffold_blank_v17) | 23 | The blank-document scaffold every new file is spliced into, and the fixed positions inside it the writer patches. |
| [`cc_skp::curves`](#mod-cc_skp-curves) | 7 | Loose-edge chaining and analytic-arc recovery, shared by the baked and the instanced scene builders (Python port extension: `openskp/_curves.py`; the C++ port has no equivalent). |
| [`cc_skp::deflate`](#mod-cc_skp-deflate) | 2 | A small pure-Rust DEFLATE (RFC 1951) / zlib (RFC 1950) compressor: hash-chain LZ77 with optional one-step lazy matching, and per block the cheapest of stored / fixed-Huffman / dynamic-Huffman coding. |
| [`cc_skp::earcut`](#mod-cc_skp-earcut) | 4 | 2D ear-clipping with holes (`earcut.cpp`): `EarPoint` and `earcut_2d`. |
| [`cc_skp::edit`](#mod-cc_skp-edit) | 9 | Load an existing legacy-format `.skp` file and rebuild it as a new, independent `SkpBuilder` (`edit.hpp` / `edit.cpp`, itself a port of Python's `openskp.edit`). |
| [`cc_skp::errors`](#mod-cc_skp-errors) | 3 | Structured parse errors (`errors.hpp` / `errors.cpp`). |
| [`cc_skp::face_groups`](#mod-cc_skp-face_groups) | 11 | Local-space face grouping shared by the baked (`scene_build.rs`) and instanced (`instanced_scene_build.rs`) scene builders (`face_groups.hpp` / `face_groups.cpp`). |
| [`cc_skp::inflate`](#mod-cc_skp-inflate) | 7 | Pure-Rust DEFLATE (RFC 1951) and zlib (RFC 1950) decoder, replacing the parts of miniz's `tinfl` / `mz_uncompress` the reader uses. |
| [`cc_skp::instanced_scene`](#mod-cc_skp-instanced_scene) | 8 | The instanced scene graph (`instanced_scene.hpp`). |
| [`cc_skp::instanced_scene_build`](#mod-cc_skp-instanced_scene_build) | 1 | The instanced scene builder (`instanced_scene.cpp`'s `build_instanced_scene_raw`), plus the Python port's loose-edge curve resources (`instanced_scene.py`'s `curve_resources`). |
| [`cc_skp::internal`](#mod-cc_skp-internal) | 19 | Reader infrastructure shared by the VFF reader, the legacy reader and the scene builders (`internal.hpp`). |
| [`cc_skp::internal::build_model`](#mod-cc_skp-internal-build_model) | 1 | Shapes a `RawParsed` into the public `SkpModel` (`model.cpp`'s `build_model`). |
| [`cc_skp::internal::bytes`](#mod-cc_skp-internal-bytes) | 10 | Bounds-checked little-endian readers, var-ints and the TLV walkers (`tlv.cpp`). |
| [`cc_skp::internal::core`](#mod-cc_skp-internal-core) | 7 | The VFF (2021+) reader: ZIP container, material/style XML, and the model.dat record walk (`core.cpp`). |
| [`cc_skp::internal::geometry`](#mod-cc_skp-internal-geometry) | 8 | VFF geometry, layer, material-id and definition collection (`geometry.cpp`). |
| [`cc_skp::internal::pages_dimensions`](#mod-cc_skp-internal-pages_dimensions) | 6 | VFF (2021+) scenes ("pages") and linear dimensions (`pages_dimensions.cpp`, itself a port of Python's `_core.py`). |
| [`cc_skp::internal::strconv`](#mod-cc_skp-internal-strconv) | 4 | C library string-to-number conversions with the C++ `std::stoi` / `std::stoll` / `std::stod` semantics the reader relies on: leading whitespace is skipped, the longest valid prefix is converted, trailing text is ignored, and "no conversion" or "out of range" is `None` (where the C++ would throw). |
| [`cc_skp::internal::transforms`](#mod-cc_skp-internal-transforms) | 5 | Instance transform helpers (`transforms.cpp`). |
| [`cc_skp::internal::vff`](#mod-cc_skp-internal-vff) | 4 | Header, version and meta.dat helpers (`vff.cpp`). |
| [`cc_skp::internal::xml`](#mod-cc_skp-internal-xml) | 7 | Hand-written equivalents of the `std::regex` patterns core.cpp uses to pull attributes out of material.xml / style.xml. |
| [`cc_skp::legacy`](#mod-cc_skp-legacy) | 22 | Legacy (pre-2021 MFC `CArchive`) reader: SketchUp 3 through 2020 (`legacy.cpp`). |
| [`cc_skp::legacy::archive`](#mod-cc_skp-legacy-archive) | 7 | The MFC object-graph walk (`legacy.cpp`'s `struct Archive`, the per-class readers and `walk_model`). |
| [`cc_skp::legacy::pages`](#mod-cc_skp-legacy-pages) | 2 | Scenes (CViewPage records) in a classic file's document tail (`legacy.cpp`'s `scan_pages_for_layers` and `read_page_list`). |
| [`cc_skp::legacy::reader`](#mod-cc_skp-legacy-reader) | 3 | The bounds-checked cursor the legacy archive reads through (`legacy.cpp`'s `struct R`). |
| [`cc_skp::model`](#mod-cc_skp-model) | 38 | The parsed SketchUp model (`model.hpp` / `model.cpp`). |
| [`cc_skp::observability`](#mod-cc_skp-observability) | 10 | Parse progress and log callbacks (`observability.hpp`). |
| [`cc_skp::parser`](#mod-cc_skp-parser) | 4 | Public entry points (`parser.hpp` / `parser.cpp`). |
| [`cc_skp::scene`](#mod-cc_skp-scene) | 9 | The baked, world-space scene (`scene.hpp`). |
| [`cc_skp::scene_build`](#mod-cc_skp-scene_build) | 17 | The baked scene builder (`scene.cpp`'s `build_scene_raw`), plus the Python port's loose-edge curve sets (`scene.py`'s `curve_sets`). |
| [`cc_skp::triangulator`](#mod-cc_skp-triangulator) | 3 | Face normals and planar-face triangulation (`triangulator.hpp` / `triangulator.cpp`). |
| [`cc_skp::zip`](#mod-cc_skp-zip) | 5 | Minimal in-memory ZIP reader replacing the miniz `mz_zip_reader_*` calls core.cpp makes: init from memory, file count/stat, locate by name, and extract to memory (stored or deflated, CRC-checked). |

<a id="mod-cc_skp"></a>

## `cc_skp`

<sub>`crates/cc_skp/src/lib.rs`</sub>

**Re-exports**

- `pub use edit::{OpenExistingError, OpenExistingResult, open_existing}`
- `pub use errors::SkpParseError`
- `pub use instanced_scene::{ InstancedCurveResource, InstancedMeshResource, InstancedNode, InstancedScene, LocalCurve, LocalPrimitive, SceneBounds, }`
- `pub use model::{ ArcCurve, AttributeKind, ByteBuffer, CoEdge, Color3, Color4, ConstructionLine, ConstructionPoint, Definition, Dimension, Edge, EntityId, Face, Instance, Layer, LooseEdgeRun, METRES_TO_INCHES, Material, Page, ParsedAttrDict, ParsedAttrDictionaries, ParsedAttribute, SectionPlane, SkpModel, StringDictionaries, Style, StyleItem, StyleWatermark, TextEntity, Texture, Vec3, Vertex, ViewCamera, loose_edge_runs, stringify_attr_dict, stringify_attr_dictionaries, }`
- `pub use observability::{LogLevel, PROGRESS_INTERVAL, ParseOptions, ParseProgress, ParseStage, stage_name}`
- `pub use parser::{SkpFile, build_instanced_scene, build_scene, parse_skp}`
- `pub use scene::{ CurveArc, CurveSetMetadata, GlbPrimitive, GltfMaterial, InstanceNode, MeshMetadata, PbrMetallicRoughness, Scene, SceneTexture, }`
- `pub use triangulator::{compute_face_normal, triangulate_face_3d}`
- `pub use create::{ ArcOptions, AttributeDict, AttributeValue, CircleOptions, ComponentDefinitionBuilder, DefinitionHandle, DefinitionOptions, DocumentOptions, FaceOptions, GroupInstanceOptions, GroupOptions, GuideOptions, ImageOptions, InstanceOptions, LayerOptions, PolylineOptions, SkpBuilder, SkpWriteError, create, }`

<a id="mod-cc_skp-create"></a>

## `cc_skp::create`

<sub>`crates/cc_skp/src/create/mod.rs`</sub>

Create new legacy-format (v17) `.skp` files from scratch.
A from-scratch binary writer for the same MFC `CArchive` object-stream format the legacy
reader decodes, built by inverting that reader's own proven logic (the class-ref/back-ref
protocol, entity preambles, drawbase records). Ported from OpenSKP's C++ `create.cpp` (itself a
port of the SketchUp-SDK-cross-checked Python writer) and kept byte-for-byte identical to it.
No SketchUp SDK is involved at any point.
**Scope:** planar faces from vertex coordinates (vertices/edges shared automatically wherever
coordinates coincide exactly), faces with holes, `auto_triangulate` fan-splitting of
non-planar polygons; solid-color and PNG/JPEG-textured materials with optional explicit UV
positioning; named layers; reusable component definitions with positioned instances; groups;
nested definitions and nested group instances; per-instance rotation and hidden state; custom
attribute dictionaries; true `CArcCurve` circles/arcs and `CCurve` polylines; Image entities;
free linear dimensions, leader texts, construction lines/points and section planes.
Coordinates are in **inches**, SketchUp's native internal unit. Every file opens to the
standard "Iso" view (parallel projection, from the (1, -1, 1) octant).
**The blank scaffold, and why it's there.** Every legacy `.skp` carries a header/material-
manager/style-and-font-manager region that is only understood well enough to preserve it
byte-for-byte and renumber the few internal references that shift when geometry is inserted.
So rather than synthesize it, new files are built by splicing genuinely-written records into
a bundled minimal empty document (see `scaffold_blank_v17`). Those template bytes came from
Trimble's own SDK (`SUModelCreate` + a bare save) - SketchUp's built-in empty-document
boilerplate (default style, Layer0, system font references), the same bytes any new
SketchUp document contains. Everything else - the entity encoding, the object-graph protocol,
the flag bytes real SketchUp silently requires, the tail renumbering - is independently
reverse-engineered.
```rust
use cc_skp::create::{create, FaceOptions};
let mut builder = create();
let red = builder.add_material("Red", [255, 0, 0], None)?;
let roof = builder.add_layer("Roof", &Default::default())?;
let options = FaceOptions { material: Some(red), layer: Some(roof), ..Default::default() };
builder.add_face(&[[0.0, 0.0, 0.0], [100.0, 0.0, 0.0], [100.0, 100.0, 0.0], [0.0, 100.0, 0.0]], &options)?;
builder.save("output.skp")?;
```

**Re-exports**

- `pub use crate::model::{Color3, Color4}`

**Items:** [`Point3`](#cc_skp-create-point3) · [`Matrix3x3`](#cc_skp-create-matrix3x3) · [`Rotation`](#cc_skp-create-rotation) · [`UvPoint`](#cc_skp-create-uvpoint) · [`UvCorrespondence`](#cc_skp-create-uvcorrespondence) · [`DEFAULT_DIMENSION_OFFSET`](#cc_skp-create-default_dimension_offset) · [`DEFAULT_TEXT_LEADER`](#cc_skp-create-default_text_leader) · [`SkpWriteError`](#cc_skp-create-skpwriteerror) · [`AttributeValue`](#cc_skp-create-attributevalue) · [`AttributeDict`](#cc_skp-create-attributedict) · [`MaterialColor`](#cc_skp-create-materialcolor) · [`DocumentOptions`](#cc_skp-create-documentoptions) · [`GuideOptions`](#cc_skp-create-guideoptions) · [`FaceOptions`](#cc_skp-create-faceoptions) · [`CircleOptions`](#cc_skp-create-circleoptions) · [`ArcOptions`](#cc_skp-create-arcoptions) · [`PolylineOptions`](#cc_skp-create-polylineoptions) · [`DefinitionOptions`](#cc_skp-create-definitionoptions) · [`LayerOptions`](#cc_skp-create-layeroptions) · [`InstanceOptions`](#cc_skp-create-instanceoptions) · [`GroupOptions`](#cc_skp-create-groupoptions) · [`GroupInstanceOptions`](#cc_skp-create-groupinstanceoptions) · [`ImageOptions`](#cc_skp-create-imageoptions) · [`DefinitionHandle`](#cc_skp-create-definitionhandle) · [`SkpBuilder`](#cc_skp-create-skpbuilder) · [`create`](#cc_skp-create-create) · [`ComponentDefinitionBuilder`](#cc_skp-create-componentdefinitionbuilder)

<a id="cc_skp-create-point3"></a>

### type alias `Point3`

```rust
pub type Point3 = [f64; 3]
```

A 3D point (x, y, z) in inches.

<a id="cc_skp-create-matrix3x3"></a>

### type alias `Matrix3x3`

```rust
pub type Matrix3x3 = [f64; 9]
```

Row-major 3x3 rotation/scale matrix.

<a id="cc_skp-create-rotation"></a>

### type alias `Rotation`

```rust
pub type Rotation = (Point3, f64)
```

A rotation as (axis, angle in radians), right-hand rule; the axis need not be unit length.

<a id="cc_skp-create-uvpoint"></a>

### type alias `UvPoint`

```rust
pub type UvPoint = (Point3, [f64; 2])
```

One (world point, (u, v)) correspondence for explicit texture positioning.

<a id="cc_skp-create-uvcorrespondence"></a>

### type alias `UvCorrespondence`

```rust
pub type UvCorrespondence = Vec<UvPoint>
```

Exactly 3 `UvPoint`s - the minimum that fully determines an affine UV mapping.

<a id="cc_skp-create-default_dimension_offset"></a>

### const `DEFAULT_DIMENSION_OFFSET`

```rust
pub const DEFAULT_DIMENSION_OFFSET: f64
```

The C++ default for `SkpBuilder::add_dimension`'s `offset`, in inches.

<a id="cc_skp-create-default_text_leader"></a>

### const `DEFAULT_TEXT_LEADER`

```rust
pub const DEFAULT_TEXT_LEADER: Point3
```

The C++ default for `SkpBuilder::add_text`'s `leader`, in inches.

<a id="cc_skp-create-skpwriteerror"></a>

### struct `SkpWriteError`

```rust
pub struct SkpWriteError
```

Raised when a `.skp` file cannot be constructed.

**Traits:** `Clone`, `Debug`, `PartialEq`, `Eq`, `fmt::Display`, `std::error::Error`

*Some fields are private.*

**Methods**

- `pub(crate) fn new(message: impl Into<String>) -> Self`
- `pub fn message(&self) -> &str`

  The human-readable reason.


<a id="cc_skp-create-attributevalue"></a>

### enum `AttributeValue`

```rust
pub enum AttributeValue
```

A custom attribute value, one variant per SketchUp attribute type this format supports.

**Traits:** `Clone`, `Debug`, `PartialEq`, `From<&str>`, `From<String>`, `From<i32>`, `From<f64>`, `From<bool>`

**Variants**

- `String(String)`

  A string (type 10).

- `Int32(i32)`

  An int32 (type 4).

- `Double(f64)`

  A plain double (type 6, "Float").

- `Bool(bool)`

  A boolean (type 7).

- `Length(f64)`

  A SketchUp `Length` in inches (type 12).


<a id="cc_skp-create-attributedict"></a>

### type alias `AttributeDict`

```rust
pub type AttributeDict = BTreeMap<String, AttributeValue>
```

A named dictionary of custom key/value metadata - the same mechanism SketchUp's own "dynamic
component" attributes use. Keys are written in sorted order.

<a id="cc_skp-create-materialcolor"></a>

### trait `MaterialColor`

```rust
pub trait MaterialColor
```

A material color: an opaque `Color3` (alpha 255) or a full `Color4`.

**Implemented by (2):** [`cc_skp::model::Color3`](cc_skp.md#cc_skp-model-color3), [`cc_skp::model::Color4`](cc_skp.md#cc_skp-model-color4)

**Required and provided items**

- `fn to_rgba(self) -> Color4`

  The color as RGBA.


<a id="cc_skp-create-documentoptions"></a>

### struct `DocumentOptions`

```rust
pub struct DocumentOptions
```

Display settings supported by the legacy document scaffold, for
`SkpBuilder::set_document_options`.

**Traits:** `Clone`, `Debug`, `PartialEq`, `Default`

**Fields**

- `length_unit: i32`

  Native UnitsOptions values: inches=0, feet=1, mm=2, cm=3, metres=4, yards=5.

- `length_precision: i32`
- `style_items: BTreeMap<i32, f64>`

  Native style item ids and scalar values (colours are packed ABGR u32 values). Only items
  present in the bundled legacy style are updated; newer items are ignored.


<a id="cc_skp-create-guideoptions"></a>

### struct `GuideOptions`

```rust
pub struct GuideOptions
```

Options for `add_construction_line` / `add_construction_point` on `SkpBuilder` and
`ComponentDefinitionBuilder`.

**Traits:** `Clone`, `Debug`, `Default`, `PartialEq`, `Eq`

**Fields**

- `layer: Option<i32>`

  Layer handle (from `add_layer`); `None` = default (Layer0).

- `hidden: bool`

  Hides the guide itself (SketchUp's "Hide" on this specific guide).


<a id="cc_skp-create-faceoptions"></a>

### struct `FaceOptions`

```rust
pub struct FaceOptions
```

Options for `add_face`.

**Traits:** `Clone`, `Debug`, `PartialEq`, `Default`

**Fields**

- `material: Option<i32>`

  Front-side material handle (from `add_material`/`add_texture_material`); `None` = default.

- `layer: Option<i32>`

  Layer handle (from `add_layer`); `None` = Layer0.

- `back_material: Option<i32>`

  Back-side material handle; `None` = default.

- `hidden: bool`

  Hides the face itself.

- `soft_edges: bool`

  Edge flags, applied only to edges NEWLY declared by this call (a shared edge keeps the
  flags it was first declared with).

- `smooth_edges: bool`
- `hidden_edges: bool`
- `front_uv: Option<UvCorrespondence>`

  Explicit front-side texture positioning: exactly 3 (point, (u, v)) pairs, (u, v) in tiles
  of the image. Works on a face of any orientation.

- `back_uv: Option<UvCorrespondence>`
- `attributes: AttributeDict`

  Custom metadata, written under a dictionary named `attribute_dict_name`.

- `attribute_dict_name: String`
- `auto_triangulate: bool`

  Fan-triangulate a non-coplanar polygon from `points[0]` into real triangles instead of
  failing. Already-planar input is still written as one face. Not compatible with
  `front_uv`/`back_uv`; ignored when `holes` is non-empty.

- `holes: Vec<Vec<Point3>>`

  Closed polygons cut out of the face, on the outer boundary's plane. Winding is irrelevant.


<a id="cc_skp-create-circleoptions"></a>

### struct `CircleOptions`

```rust
pub struct CircleOptions
```

Options for `add_circle`.

**Traits:** `Clone`, `Debug`, `PartialEq`, `Default`

**Fields**

- `num_segments: u32`

  Tessellation segment count, 3-255 (SketchUp's own circle tool defaults to 24).

- `material: Option<i32>`
- `layer: Option<i32>`
- `back_material: Option<i32>`
- `hidden: bool`
- `front_uv: Option<UvCorrespondence>`

  Explicit texture positioning. Unlike `FaceOptions`, these pins are NOT scaled by the
  material's applied size (matching the C++ writer).

- `back_uv: Option<UvCorrespondence>`
- `attributes: AttributeDict`
- `attribute_dict_name: String`

<a id="cc_skp-create-arcoptions"></a>

### struct `ArcOptions`

```rust
pub struct ArcOptions
```

Options for `add_arc`.

**Traits:** `Clone`, `Copy`, `Debug`, `PartialEq`, `Default`

**Fields**

- `num_segments: u32`

  Segment count, 3-255.

- `hidden_edges: bool`
- `soft_edges: bool`
- `smooth_edges: bool`

<a id="cc_skp-create-polylineoptions"></a>

### struct `PolylineOptions`

```rust
pub struct PolylineOptions
```

Options for `add_polyline`.

**Traits:** `Clone`, `Copy`, `Debug`, `Default`, `PartialEq`, `Eq`

**Fields**

- `closed: bool`

  Also connects the last point back to the first.

- `hidden_edges: bool`
- `soft_edges: bool`
- `smooth_edges: bool`

<a id="cc_skp-create-definitionoptions"></a>

### struct `DefinitionOptions`

```rust
pub struct DefinitionOptions
```

Options for `SkpBuilder::add_component_definition`.

**Traits:** `Clone`, `Debug`, `PartialEq`, `Default`

**Fields**

- `attributes: AttributeDict`

  Custom metadata on the definition itself, under `attribute_dict_name`.

- `attribute_dict_name: String`

<a id="cc_skp-create-layeroptions"></a>

### struct `LayerOptions`

```rust
pub struct LayerOptions
```

Options for `SkpBuilder::add_layer`.

**Traits:** `Clone`, `Debug`, `Default`, `PartialEq`

**Fields**

- `color: Option<Color4>`

  The layer's display color; `None` writes all-zero bytes.

- `hidden: bool`

  The layer's own visibility (the layer panel checkbox).

- `extra_dictionaries: BTreeMap<String, AttributeDict>`

  Named attribute dictionaries written onto the layer; the writer reserves no names.


<a id="cc_skp-create-instanceoptions"></a>

### struct `InstanceOptions`

```rust
pub struct InstanceOptions
```

Options for `add_instance`.

**Traits:** `Clone`, `Debug`, `PartialEq`, `Default`

**Fields**

- `name: Option<String>`

  Defaults to the definition's name.

- `translation: Point3`

  Applied after the matrix, in inches.

- `matrix3x3: Option<Matrix3x3>`

  Rotation/scale (identity if unset). At most one of `matrix3x3`/`rotation`.

- `rotation: Option<Rotation>`
- `material: Option<i32>`

  Applied to the instance itself, not its contents.

- `layer: Option<i32>`
- `attributes: AttributeDict`

  Custom metadata on this placement (not its definition), under `attribute_dict_name`.

- `attribute_dict_name: String`
- `hidden: bool`

  Hides this placement; its contents still exist in the file.


<a id="cc_skp-create-groupoptions"></a>

### struct `GroupOptions`

```rust
pub struct GroupOptions
```

Options for `SkpBuilder::add_group`.

**Traits:** `Clone`, `Debug`, `PartialEq`, `Default`

**Fields**

- `name: Option<String>`

  Defaults to "Group".

- `translation: Point3`
- `matrix3x3: Option<Matrix3x3>`
- `rotation: Option<Rotation>`
- `material: Option<i32>`
- `layer: Option<i32>`
- `attributes: AttributeDict`

  Custom metadata; a group only gets a real attribute container when this is non-empty
  (openskp#261).

- `attribute_dict_name: String`
- `hidden: bool`

<a id="cc_skp-create-groupinstanceoptions"></a>

### struct `GroupInstanceOptions`

```rust
pub struct GroupInstanceOptions
```

Options for `ComponentDefinitionBuilder::add_group_instance`; same shape as
`GroupOptions`, but the name defaults to the placed definition's name.

**Traits:** `Clone`, `Debug`, `PartialEq`, `Default`

**Fields**

- `name: Option<String>`
- `translation: Point3`
- `matrix3x3: Option<Matrix3x3>`
- `rotation: Option<Rotation>`
- `material: Option<i32>`
- `layer: Option<i32>`
- `attributes: AttributeDict`
- `attribute_dict_name: String`
- `hidden: bool`

<a id="cc_skp-create-imageoptions"></a>

### struct `ImageOptions`

```rust
pub struct ImageOptions
```

Options for `SkpBuilder::add_image`.

**Traits:** `Clone`, `Copy`, `Debug`, `Default`, `PartialEq`

**Fields**

- `translation: Point3`
- `matrix3x3: Option<Matrix3x3>`

  Rotation/scale (identity if unset). At most one of `matrix3x3`/`rotation`.

- `rotation: Option<Rotation>`
- `layer: Option<i32>`
- `hidden: bool`

<a id="cc_skp-create-definitionhandle"></a>

### struct `DefinitionHandle`

```rust
pub struct DefinitionHandle
```

A reference to a component/group definition of one `SkpBuilder`, used to place it
(`add_instance`/`add_group_instance`) or to reopen its `ComponentDefinitionBuilder`.

**Traits:** `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`, `Hash`

*Some fields are private.*

<a id="cc_skp-create-skpbuilder"></a>

### struct `SkpBuilder`

```rust
pub struct SkpBuilder
```

Accumulates geometry and writes it into a new legacy-format (v17) `.skp` file.
Sections are spliced into the scaffold in a fixed order, so calls must follow it: materials,
then layers, then component definitions/groups, then root-level geometry and placements.
Each step's archive slot numbering depends on the final size of every earlier one.

**Traits:** `Debug`, `Default`, `ReplayTarget`

*Some fields are private.*

**Methods**

- `pub fn new() -> Self`

  An empty builder. See also `create`.

- `pub fn materials_by_name(&self) -> &BTreeMap<String, i32>`

  Every material registered so far, by name, with its handle.

- `pub fn layers_by_name(&self) -> &BTreeMap<String, i32>`

  Every layer registered so far, by name, with its handle.

- `pub fn use_deterministic_ids(&mut self, timestamp: u32)`

  Replace random GUIDs and the current time with a fixed, reproducible sequence (GUID `n`
  is bytes `n*16 + i` with the v4 bits applied; every timestamp is `timestamp`). Only for
  byte-exact tests: real files need unique GUIDs.

- `pub fn set_document_options(&mut self, options: &DocumentOptions) -> Result<(), SkpWriteError>`

  Set decimal display units and basic working/saved style values, without scaling geometry.

- `pub fn debug_geometry_next_slot(&self) -> Option<i32>`

  The root geometry writer's next archive slot, once root geometry exists (test hook).

- `pub fn add_material(&mut self, name: &str, color: impl MaterialColor, opacity: Option<f64>) -> Result<i32, SkpWriteError>`

  Register a solid-color material and return its handle for `material` options. `color` is
  a `Color3` (opaque) or `Color4`. Registering an existing name returns its handle.
  Materials must all come before any layer, definition or geometry.

- `pub fn add_texture_material(&mut self, name: &str, image_path: impl AsRef<Path>, applied_height: Option<f64>, applied_width: Option<f64>, opacity: Option<f64>) -> Result<i32, SkpWriteError>`

  Register an image-textured material from a PNG or JPEG file (detected from its magic
  bytes) and return its handle. Same ordering rules as [`add_material`](Self::add_material).
  `applied_height`/`applied_width` (default 1.0) are the inches of model space one tile of
  the image covers. A face textured without positioning carries no UV record, so this pair
  IS its mapping. The path string itself is embedded in the file.

- `pub fn add_layer(&mut self, name: &str, options: &LayerOptions) -> Result<i32, SkpWriteError>`

  Register a layer and return its handle for `layer` options. Registering an existing name
  returns its handle (and ignores `options`). Must come before any definition or geometry.

- `pub fn add_component_definition(&mut self, name: &str, options: &DefinitionOptions) -> Result<ComponentDefinitionBuilder<'_>, SkpWriteError>`

  Start a reusable component definition: add its geometry through the returned builder,
  `close()` it, then place it with [`add_instance`](Self::add_instance). Must come before
  any root-level geometry, and only one definition may be open at a time.

- `pub fn add_group(&mut self, options: &GroupOptions) -> Result<ComponentDefinitionBuilder<'_>, SkpWriteError>`

  Start a group: add its geometry through the returned builder; `close()` places it at
  `options.translation`/`matrix3x3` automatically. Same ordering rule as
  [`add_component_definition`](Self::add_component_definition).

- `pub fn definition(&mut self, handle: DefinitionHandle) -> Result<ComponentDefinitionBuilder<'_>, SkpWriteError>`

  Reopen the builder of one of this builder's definitions (e.g. to `close()` it later).

- `pub fn add_instance(&mut self, definition: DefinitionHandle, options: &InstanceOptions) -> Result<(), SkpWriteError>`

  Place one instance of a closed definition in the model.

- `pub fn add_image(&mut self, image_path: impl AsRef<Path>, width: f64, height: f64, options: &ImageOptions) -> Result<(), SkpWriteError>`

  Place a SketchUp Image entity (File > Import > Image): a picture as its own object, with
  its own Outliner classification and explode behavior, unlike a textured face.
  `width`/`height` size its quad in inches (in the XY plane before placement); the image
  covers it edge to edge regardless of the file's own pixel aspect ratio. It registers a
  texture material internally, so it must come before any layer, definition or geometry.
  The quad's UVs are pinned explicitly with the default 1.0 applied size, which makes the
  reader's divide-by-applied-height a no-op. CImage's schema version is an unverified guess
  (no real SketchUp-authored Image sample was available).

- `pub fn add_face(&mut self, points: &[Point3], options: &FaceOptions) -> Result<(), SkpWriteError>`

  Add one planar face: 3+ coplanar points (inches) in order, without repeating the first.
  Vertices/edges are shared with earlier faces wherever coordinates match exactly.

- `pub fn add_circle(&mut self, center: Point3, normal: Point3, radius: f64, options: &CircleOptions) -> Result<(), SkpWriteError>`

  Add one circular face - a true SketchUp circle (editable by radius, one "Curve" entity),
  not loose edges tracing it. `normal` need not be unit length.

- `pub fn add_arc(&mut self, center: Point3, normal: Point3, radius: f64, start_angle: f64, end_angle: f64, options: &ArcOptions) -> Result<(), SkpWriteError>`

  Add one partial (open) arc - a genuine arc entity, edges only, no face. Angles (radians)
  sweep from a fixed 0-angle reference direction in the arc's plane chosen from `normal`;
  either direction and sweeps beyond a full turn are valid.

- `pub fn add_polyline(&mut self, points: &[Point3], options: &PolylineOptions) -> Result<(), SkpWriteError>`

  Add one freeform polyline (at least 2 points) - a chain of edges grouped into one genuine
  "Curve" entity. No face.

- `pub fn add_dimension(&mut self, p1: Point3, p2: Point3, offset: f64) -> Result<(), SkpWriteError>`

  Add a FREE linear dimension between two world-space points. `offset` is the dimension
  line's signed offset from the measured segment, in inches (C++ default:
  `DEFAULT_DIMENSION_OFFSET`).

- `pub fn add_dimension_with_offset_vector(&mut self, p1: Point3, p2: Point3, offset_vector: Point3) -> Result<(), SkpWriteError>`

  Add a linear dimension placed by an offset vector (inches, in the same space as the
  endpoints): its component perpendicular to `p1`->`p2` sets the plane and side of the
  dimension line. Coordinates must be finite and the endpoints distinct; a zero offset is
  supported. (C++: the `add_dimension` overload taking a `Point3` offset.)

- `pub fn add_text(&mut self, text: &str, point: Point3, leader: Point3) -> Result<(), SkpWriteError>`

  Add a leader text anchored at `point`, its label at `point + leader` (C++ default:
  `DEFAULT_TEXT_LEADER`).

- `pub fn add_construction_line(&mut self, point: Point3, point2: Option<Point3>, direction: Option<Point3>, options: &GuideOptions) -> Result<(), SkpWriteError>`

  Add a construction (guide) line through `point`: pass exactly one of `point2` (a bounded
  segment, like `Entities#add_cline(p1, p2)`) or `direction` (unbounded, like
  `Entities#add_cline(point, vector)`). `options` places the guide on a layer and/or hides
  it.

- `pub fn add_construction_point(&mut self, position: Point3, options: &GuideOptions) -> Result<(), SkpWriteError>`

  Add a construction (guide) point. `options` places the guide on a layer and/or hides it.

- `pub fn add_section_plane(&mut self, point: Point3, normal: Point3) -> Result<(), SkpWriteError>`

  Add a section plane through `point` with `normal` (any length), like
  `Entities#add_section_plane([point, normal])`.

- `pub fn to_bytes(&mut self) -> Result<Vec<u8>, SkpWriteError>`

  The finished file's bytes. Fails if nothing was added at the root level (a closed group
  counts) or a definition is still open while groups wait to be placed.

- `pub fn save(&mut self, path: impl AsRef<Path>) -> Result<(), SkpWriteError>`

  Write the finished file to `path`.


<a id="cc_skp-create-create"></a>

### fn `create`

```rust
pub fn create() -> SkpBuilder
```

Start building a new legacy-format (v17) `.skp` file. Same as `SkpBuilder::new`.

<a id="cc_skp-create-componentdefinitionbuilder"></a>

### struct `ComponentDefinitionBuilder`

```rust
pub struct ComponentDefinitionBuilder<'a>
```

Accumulates one component/group definition's geometry. Obtained from
`SkpBuilder::add_component_definition`, `SkpBuilder::add_group` or
`SkpBuilder::definition`; it borrows the builder, so finish the definition (`close()`)
before going back to the builder - only one definition can be open at a time anyway.
```rust
let mut builder = create();
let mut chair = builder.add_component_definition("Chair", &DefinitionOptions::default())?;
chair.add_face(&[[0.0, 0.0, 0.0], [20.0, 0.0, 0.0], [20.0, 20.0, 0.0], [0.0, 20.0, 0.0]], &FaceOptions::default())?;
let chair = chair.close()?;
builder.add_instance(chair, &InstanceOptions { translation: [100.0, 0.0, 0.0], ..Default::default() })?;
```

**Traits:** `Debug`, `ReplayTarget`

*Some fields are private.*

**Methods**

- `pub fn name(&self) -> &str`

  This definition's name.

- `pub fn closed(&self) -> bool`

  True once `close()` has succeeded; no more geometry may be added.

- `pub fn slot(&self) -> i32`

  This definition's archive slot number.

- `pub fn handle(&self) -> DefinitionHandle`

  A handle for placing this definition (valid even before it closes).

- `pub fn add_face(&mut self, points: &[Point3], options: &FaceOptions) -> Result<(), SkpWriteError>`

  Add one planar face - like `SkpBuilder::add_face`, but vertices/edges are shared only
  within this definition.

- `pub fn add_circle(&mut self, center: Point3, normal: Point3, radius: f64, options: &CircleOptions) -> Result<(), SkpWriteError>`

  Add one circular face - like `SkpBuilder::add_circle`.

- `pub fn add_arc(&mut self, center: Point3, normal: Point3, radius: f64, start_angle: f64, end_angle: f64, options: &ArcOptions) -> Result<(), SkpWriteError>`

  Add one partial (open) arc - like `SkpBuilder::add_arc`.

- `pub fn add_polyline(&mut self, points: &[Point3], options: &PolylineOptions) -> Result<(), SkpWriteError>`

  Add one freeform polyline - like `SkpBuilder::add_polyline`.

- `pub fn add_dimension(&mut self, p1: Point3, p2: Point3, offset: f64) -> Result<(), SkpWriteError>`

  Add a free linear dimension - like `SkpBuilder::add_dimension`, in definition-local
  inches.

- `pub fn add_dimension_with_offset_vector(&mut self, p1: Point3, p2: Point3, offset_vector: Point3) -> Result<(), SkpWriteError>`

  Add a linear dimension placed by an offset vector - like
  `SkpBuilder::add_dimension_with_offset_vector`, in definition-local inches.

- `pub fn add_construction_line(&mut self, point: Point3, point2: Option<Point3>, direction: Option<Point3>, options: &GuideOptions) -> Result<(), SkpWriteError>`

  Add a construction line - like `SkpBuilder::add_construction_line`, in
  definition-local inches.

- `pub fn add_construction_point(&mut self, position: Point3, options: &GuideOptions) -> Result<(), SkpWriteError>`

  Add a construction point - like `SkpBuilder::add_construction_point`, in
  definition-local inches.

- `pub fn add_instance(&mut self, definition: DefinitionHandle, options: &InstanceOptions) -> Result<(), SkpWriteError>`

  Place an instance of another (closed) definition of the same builder inside this one.

- `pub fn add_group_instance(&mut self, definition: DefinitionHandle, options: &GroupInstanceOptions) -> Result<(), SkpWriteError>`

  Place another definition inside this one as a *group* (`CGroup`) rather than a component
  instance. A nested group can't be declared inline (this format can't embed one
  definition's declaration inside another's): build it with `add_component_definition`
  first, then place it here.

- `pub fn close(&mut self) -> Result<DefinitionHandle, SkpWriteError>`

  Finalize this definition: patch its entity count, write its tail (GUID/name/thumbnail)
  and, for a group, queue its placement. Fails if it has no geometry or is already closed.
  Returns the definition's handle.


<a id="mod-cc_skp-create-archive"></a>

## `cc_skp::create::archive`

<sub>`crates/cc_skp/src/create/archive.rs`</sub>

`ArchiveWriter` - the write-side mirror of the legacy reader's MFC `CArchive` slot/class-ref
bookkeeping, plus every entity record encoder the builder uses.

**Items:** [`PutLe`](#cc_skp-create-archive-putle) · [`read_u16_at`](#cc_skp-create-archive-read_u16_at) · [`write_u16_at`](#cc_skp-create-archive-write_u16_at) · [`write_u32_at`](#cc_skp-create-archive-write_u32_at) · [`shift_ref`](#cc_skp-create-archive-shift_ref) · [`detect_image_subtype`](#cc_skp-create-archive-detect_image_subtype) · [`IdSource`](#cc_skp-create-archive-idsource) · [`AttributeDictList`](#cc_skp-create-archive-attributedictlist) · [`Topology`](#cc_skp-create-archive-topology) · [`EdgeFlags`](#cc_skp-create-archive-edgeflags) · [`ArcCurveParams`](#cc_skp-create-archive-arccurveparams) · [`ChainCurve`](#cc_skp-create-archive-chaincurve) · [`FaceSpec`](#cc_skp-create-archive-facespec) · [`TextureSource`](#cc_skp-create-archive-texturesource) · [`Placement`](#cc_skp-create-archive-placement) · [`ArchiveWriter`](#cc_skp-create-archive-archivewriter)

<a id="cc_skp-create-archive-putle"></a>

### trait `PutLe`

```rust
pub(super) trait PutLe
```

Little-endian appends onto a byte buffer.

**Implemented by (1):** `cc_skp::create::archive::Vec`

**Required and provided items**

- `fn put_u16(&mut self, v: u16)`
- `fn put_u32(&mut self, v: u32)`
- `fn put_f64(&mut self, v: f64)`
- `fn put_zeros(&mut self, n: usize)`

<a id="cc_skp-create-archive-read_u16_at"></a>

### fn `read_u16_at`

```rust
pub(super) fn read_u16_at(buf: &[u8], pos: usize) -> u16
```

<a id="cc_skp-create-archive-write_u16_at"></a>

### fn `write_u16_at`

```rust
pub(super) fn write_u16_at(buf: &mut [u8], pos: usize, v: u16)
```

<a id="cc_skp-create-archive-write_u32_at"></a>

### fn `write_u32_at`

```rust
pub(super) fn write_u32_at(buf: &mut [u8], pos: usize, v: u32)
```

<a id="cc_skp-create-archive-shift_ref"></a>

### fn `shift_ref`

```rust
pub(super) fn shift_ref(buf: &mut Vec<u8>, pos: usize, shift: i32) -> usize
```

Renumber the u16 slot reference at `pos` by `shift`, keeping its 0x8000 class-ref tag bit.
Widens to the 6-byte escape form if the new slot lands at or past 0x7FFF: 0x7FFF is the
archive's big-tag escape marker and 0x8000|0x7FFF == 0xFFFF its new-class marker, so neither
can be an ordinary slot number. Returns how many bytes the field grew by (0 or 4).

<a id="cc_skp-create-archive-detect_image_subtype"></a>

### fn `detect_image_subtype`

```rust
pub(super) fn detect_image_subtype(image_bytes: &[u8]) -> Result<u32, SkpWriteError>
```

CDib's image format tag from the file's own magic bytes: 4 for PNG, 1 for JPEG.

<a id="cc_skp-create-archive-idsource"></a>

### enum `IdSource`

```rust
pub(super) enum IdSource
```

Where definition/instance GUIDs and definition timestamps come from.

**Traits:** `Clone`, `Debug`

**Variants**

- `Random`

  Random v4 GUIDs and the current time - what real files get.

- `Deterministic { next: u32, timestamp: u32 }`

  Reproducible output for byte-exact tests: GUID `n` is bytes `n*16 + i` (with the v4
  version/variant bits applied) and every timestamp is fixed.


**Methods**

- `pub fn guid(&mut self) -> [u8; 16]`
- `pub fn timestamp(&self) -> u32`

<a id="cc_skp-create-archive-attributedictlist"></a>

### type alias `AttributeDictList`

```rust
pub(super) type AttributeDictList = Vec<(String, AttributeDict)>
```

A named attribute dictionary, in write order.

<a id="cc_skp-create-archive-topology"></a>

### struct `Topology`

```rust
pub(super) struct Topology
```

The vertex/edge sharing state of one entity list (the root model or one definition).

**Traits:** `Debug`, `Default`

*Some fields are private.*

<a id="cc_skp-create-archive-edgeflags"></a>

### struct `EdgeFlags`

```rust
pub(super) struct EdgeFlags
```

**Traits:** `Clone`, `Copy`, `Debug`, `Default`

**Fields**

- `hidden: bool`
- `soft: bool`
- `smooth: bool`

<a id="cc_skp-create-archive-arccurveparams"></a>

### struct `ArcCurveParams`

```rust
pub(super) struct ArcCurveParams
```

Curve parameters shared by every edge of a circle/arc - see `write_arc_curve`.

**Traits:** `Clone`, `Copy`, `Debug`

**Fields**

- `center: Point3`
- `normal: Point3`
- `xaxis: Point3`

  The arc's fixed 0-angle reference direction (unit vector times radius).

- `start_angle: f64`
- `end_angle: f64`
- `radius: f64`
- `num_segments: u32`

<a id="cc_skp-create-archive-chaincurve"></a>

### enum `ChainCurve`

```rust
pub(super) enum ChainCurve<'a>
```

The curve object an edge chain's edges share, declared inline by its first new edge.

**Traits:** `Clone`, `Copy`, `Debug`

**Variants**

- `None`
- `Arc(&'a ArcCurveParams)`
- `Polyline { num_edges: u32 }`

<a id="cc_skp-create-archive-facespec"></a>

### struct `FaceSpec`

```rust
pub(super) struct FaceSpec<'a>
```

Everything about one face except its outer boundary points.

**Traits:** `Clone`, `Copy`, `Debug`

**Fields**

- `material: i32`
- `layer: i32`
- `back_material: i32`
- `hidden: bool`
- `edges: EdgeFlags`
- `front_uv: Option<&'a [UvPoint]>`
- `back_uv: Option<&'a [UvPoint]>`
- `attribute_dicts: &'a [(String, AttributeDict)]`
- `curve: Option<&'a ArcCurveParams>`
- `holes: &'a [Vec<Point3>]`

<a id="cc_skp-create-archive-texturesource"></a>

### struct `TextureSource`

```rust
pub(super) struct TextureSource<'a>
```

A texture image to embed: its bytes, the path string stored with it, and its CDib subtype.

**Traits:** `Clone`, `Copy`, `Debug`

**Fields**

- `bytes: &'a [u8]`
- `path: &'a str`
- `subtype: u32`

<a id="cc_skp-create-archive-placement"></a>

### struct `Placement`

```rust
pub(super) struct Placement<'a>
```

One placement of a definition (component instance, group or image).

**Traits:** `Clone`, `Copy`, `Debug`

**Fields**

- `definition_slot: i32`
- `name: &'a str`
- `translation: Point3`
- `matrix3x3: Option<Matrix3x3>`
- `material: i32`
- `layer: i32`
- `attribute_dicts: &'a [(String, AttributeDict)]`
- `hidden: bool`

<a id="cc_skp-create-archive-archivewriter"></a>

### struct `ArchiveWriter`

```rust
pub(super) struct ArchiveWriter
```

**Traits:** `Clone`, `Debug`

**Fields**

- `next_slot: i32`
- `class_slot: HashMap<&'static str, i32>`
- `next_pid: u64`
- `buf: Vec<u8>`

*Some fields are private.*

**Methods**

- `pub fn new(next_slot: i32, class_slot: HashMap<&'static str, i32>, next_pid: u64) -> Self`
- `pub fn alloc_pid(&mut self) -> u64`
- `pub fn write_str(&mut self, s: &str) -> Result<(), SkpWriteError>`

  An MFC CString: `FF FE FF`, a u8 UTF-16 code-unit count, then UTF-16LE code units.

- `pub fn write_dimension(&mut self, p1: Point3, p2: Point3, offset: f64, offset_vector: Option<Point3>) -> Result<(), SkpWriteError>`

  A FREE linear dimension between two explicit points. The layout is the byte-exact one the
  real SDK writes for free dimensions (docs/dimension-record-notes.md): connection type 1
  with the point stored inline in each connection block and null object refs.

- `pub fn write_text(&mut self, text: &str, point: Point3, leader: Point3) -> Result<(), SkpWriteError>`

  A leader text anchored at `point`, label at `point + leader`. Mirrors human-drawn leader
  texts from real files (the SDK's own create only makes SCREEN texts): screen slot zeroed,
  the free-connection block dimensions use, the label's world position in the placement
  tail, and leader type 2 (pushpin) before the arrow delimiter.

- `pub fn write_construction_line(&mut self, point: Point3, point2: Option<Point3>, direction: Option<Point3>, options: &GuideOptions) -> Result<(), SkpWriteError>`

  A construction line: point + unit direction + signed start/end distance parameters along
  it (ground truth: SketchUp 2025, SDK/Ruby cross-checked). Exactly one of `point2` (bounded
  segment) or `direction` (unbounded, written with the real +/-1e30 sentinel) is required.

- `pub fn write_construction_point(&mut self, position: Point3, options: &GuideOptions)`

  A construction point. A second, always-zero 3-double block and a trailing zero byte follow
  the position - reserved, written as zero to match every real sample.

- `pub fn write_section_plane(&mut self, point: Point3, normal: Point3) -> Result<(), SkpWriteError>`

  A section plane: preamble + drawbase + (a, b, c, d) with a*x + b*y + c*z + d = 0 (unit
  normal, d = -(normal . point)), the form `SectionPlane#get_plane` returns. The v18+
  name/label pair is omitted: this writer only produces v17 files.

- `pub fn write_material(&mut self, name: &str, rgba: Color4, opacity: Option<f64>) -> Result<i32, SkpWriteError>`

  A solid-color material. The stored f64 is TRANSPARENCY (0 = opaque), gated by the u8
  after it - see `write_textured_material`.

- `pub fn write_textured_material(&mut self, name: &str, image: &TextureSource<'_>, applied_height: Option<f64>, applied_width: Option<f64>, opacity: Option<f64>) -> Result<i32, SkpWriteError>`

  An image-textured material. `subtype` is CDib's format tag (see `detect_image_subtype`).
  The applied size (both default 1.0) is how much model space one tile covers, in inches.
  A face textured without positioning carries no per-face UV record, so for it this pair IS
  the mapping - and the reader divides a positioned face's UV by it too. (It once defaulted
  to a garbage sentinel pattern copied from one SDK sample, which real SketchUp renders as a
  vertically-smeared texture.)

- `pub fn write_layer(&mut self, name: &str, with_pids: bool, hidden: bool, rgba: Option<Color4>, dicts: &[(String, AttributeDict)]) -> Result<i32, SkpWriteError>`

  A CLayer record. Ground truth shows each top-level layer embeds a second pid after its
  name, so it consumes 2 pids. `with_pids = false` (only for the Layer0 copy a definition
  embeds) omits both.

- `pub fn write_definition_header(&mut self, attribute_dicts: &[(String, AttributeDict)]) -> Result<(i32, usize), SkpWriteError>`

  Begin a CComponentDefinition record, up to (not including) its entity list. Returns
  (definition slot, offset of the u32 entity count the caller patches on close).

- `pub fn write_definition_tail(&mut self, name: &str, is_group: bool, ids: &mut IdSource) -> Result<usize, SkpWriteError>`

  Close a CComponentDefinition record: relationship count, GUID, name, timestamp, behavior
  flags and a default thumbnail. Returns the offset of the u32 group flag, so a definition
  first closed as a component can be flagged later when it is placed as a group.

- `pub fn write_instance(&mut self, placement: &Placement<'_>, ids: &mut IdSource) -> Result<(), SkpWriteError>`

  A component instance - which, unlike a group, always carries a real (if empty)
  attribute container.

- `pub fn write_group(&mut self, placement: &Placement<'_>, ids: &mut IdSource) -> Result<(), SkpWriteError>`

  A group: structurally a component instance except for its class/schema and attribute
  pointer - a group only gets a real container when it actually has attributes (a real
  SketchUp 2020 Group with attributes carries one at this exact schema; openskp#261).

- `pub fn write_image(&mut self, definition_slot: i32, translation: Point3, matrix3x3: Option<Matrix3x3>, layer: i32, hidden: bool, ids: &mut IdSource) -> Result<(), SkpWriteError>`

  A CImage, which the reader treats as instance-shaped: field-for-field the same as
  `write_instance` (13 f64s, name, GUID), with an always-empty source-path string as the
  name (real SketchUp writes it empty too) and no material - an Image's look comes entirely
  from its definition's textured face.

- `pub fn write_arc(&mut self, points: &[Point3], topo: &mut Topology, params: &ArcCurveParams, flags: EdgeFlags) -> i32`

  A partial (open) arc as a chain of straight edges - no face. Returns the number of new
  entity-list entries.

- `pub fn write_polyline(&mut self, points: &[Point3], topo: &mut Topology, closed: bool, flags: EdgeFlags) -> i32`

  A freeform polyline: straight edges sharing one CCurve, no face. Returns the number of new
  entity-list entries.

- `pub fn write_face(&mut self, points: &[Point3], topo: &mut Topology, spec: &FaceSpec<'_>) -> Result<i32, SkpWriteError>`

  One planar face. Returns the number of new entity-list entries (newly declared edges of
  the boundary and every hole, plus the face itself).


<a id="mod-cc_skp-create-geometry"></a>

## `cc_skp::create::geometry`

<sub>`crates/cc_skp/src/create/geometry.rs`</sub>

Small geometry/linear-algebra helpers for the writer.
Every formula keeps the exact operation order of the C++ writer: these values are written
straight into the file as f64s, so any reassociation would change output bytes.

**Items:** [`cross3`](#cc_skp-create-geometry-cross3) · [`normalize3`](#cc_skp-create-geometry-normalize3) · [`resolve_matrix3x3`](#cc_skp-create-geometry-resolve_matrix3x3) · [`circle_basis`](#cc_skp-create-geometry-circle_basis) · [`circle_points`](#cc_skp-create-geometry-circle_points) · [`arc_points`](#cc_skp-create-geometry-arc_points) · [`uv_matrix_for_face`](#cc_skp-create-geometry-uv_matrix_for_face) · [`polygon_span`](#cc_skp-create-geometry-polygon_span) · [`plane_tolerance`](#cc_skp-create-geometry-plane_tolerance) · [`Plane`](#cc_skp-create-geometry-plane) · [`newell_plane`](#cc_skp-create-geometry-newell_plane) · [`plane_from_polygon`](#cc_skp-create-geometry-plane_from_polygon) · [`is_coplanar`](#cc_skp-create-geometry-is_coplanar)

<a id="cc_skp-create-geometry-cross3"></a>

### fn `cross3`

```rust
pub(super) fn cross3(a: Point3, b: Point3) -> Point3
```

<a id="cc_skp-create-geometry-normalize3"></a>

### fn `normalize3`

```rust
pub(super) fn normalize3(v: Point3) -> Result<Point3, SkpWriteError>
```

<a id="cc_skp-create-geometry-resolve_matrix3x3"></a>

### fn `resolve_matrix3x3`

```rust
pub(super) fn resolve_matrix3x3(matrix3x3: Option<Matrix3x3>, rotation: Option<Rotation>) -> Result<Option<Matrix3x3>, SkpWriteError>
```

`matrix3x3` and `rotation` are two spellings of the same transform field, so at most one may
be given (neither means identity).

<a id="cc_skp-create-geometry-circle_basis"></a>

### fn `circle_basis`

```rust
pub(super) fn circle_basis(normal: Point3) -> Result<(Point3, Point3), SkpWriteError>
```

An arbitrary orthonormal in-plane basis (U, W) for a circle/arc plane: Gram-Schmidt whichever
of world +Z/+X is less parallel to `normal`, then W = normal x U.

<a id="cc_skp-create-geometry-circle_points"></a>

### fn `circle_points`

```rust
pub(super) fn circle_points(center: Point3, radius: f64, num_segments: u32, u: Point3, w: Point3) -> Vec<Point3>
```

<a id="cc_skp-create-geometry-arc_points"></a>

### fn `arc_points`

```rust
pub(super) fn arc_points(center: Point3, radius: f64, num_segments: u32, (u, w): (Point3, Point3), start_angle: f64, end_angle: f64) -> Vec<Point3>
```

The `num_segments + 1` points (both endpoints included) of a partial arc.

<a id="cc_skp-create-geometry-uv_matrix_for_face"></a>

### fn `uv_matrix_for_face`

```rust
pub(super) fn uv_matrix_for_face(pairs: &[UvPoint], normal: Point3) -> Result<Matrix3x3, SkpWriteError>
```

Fit the 3x3 UV-to-world affine matrix real SketchUp stores for a positioned texture, from
exactly 3 (world point, (u, v)) correspondences, in the face's own UV basis.

<a id="cc_skp-create-geometry-polygon_span"></a>

### fn `polygon_span`

```rust
pub(super) fn polygon_span(points: &[Point3]) -> f64
```

The largest axis-aligned extent of `points`.

<a id="cc_skp-create-geometry-plane_tolerance"></a>

### fn `plane_tolerance`

```rust
pub(super) fn plane_tolerance(points: &[Point3]) -> f64
```

The coplanarity tolerance for a polygon: scales with the face's own size.

<a id="cc_skp-create-geometry-plane"></a>

### struct `Plane`

```rust
pub(super) struct Plane
```

A plane `nx*x + ny*y + nz*z = d` with a unit normal.

**Traits:** `Clone`, `Copy`, `Debug`

**Fields**

- `nx: f64`
- `ny: f64`
- `nz: f64`
- `d: f64`

**Methods**

- `pub fn normal(&self) -> Point3`
- `pub fn distance(&self, p: Point3) -> f64`

<a id="cc_skp-create-geometry-newell_plane"></a>

### fn `newell_plane`

```rust
pub(super) fn newell_plane(points: &[Point3]) -> Result<Plane, SkpWriteError>
```

Newell's method: sums a cross-product-like term over every edge rather than reading the
normal off the first 3 points - correct for concave polygons too, as long as the polygon is
planar and simple.

<a id="cc_skp-create-geometry-plane_from_polygon"></a>

### fn `plane_from_polygon`

```rust
pub(super) fn plane_from_polygon(points: &[Point3]) -> Result<Plane, SkpWriteError>
```

The polygon's plane; every point must actually lie on it.

<a id="cc_skp-create-geometry-is_coplanar"></a>

### fn `is_coplanar`

```rust
pub(super) fn is_coplanar(points: &[Point3]) -> Result<bool, SkpWriteError>
```

Same fit/tolerance as `plane_from_polygon`, but answers instead of failing on a non-planar
polygon (still fails on a degenerate one).

<a id="mod-cc_skp-create-scaffold_blank_v17"></a>

## `cc_skp::create::scaffold_blank_v17`

<sub>`crates/cc_skp/src/create/scaffold_blank_v17.rs`</sub>

The blank-document scaffold every new file is spliced into, and the fixed positions inside it
the writer patches.
`scaffold_blank_v17.skp` is byte-for-byte the SDK-authored empty v17 document every OpenSKP
writer port bundles (`packages/python/src/openskp/_scaffold/blank_v17.skp`, sha256
`809a1ab73a20a192ab13aaff197afb1c67d0e9352f6a353a9cd8030919f8a6c3`; also
`tests/fixtures/blank_v17.skp`). See the `crate::create` module docs for what it is and why
it is there.
The Python writer computes the offsets below at runtime by walking the scaffold with the
generic MFC archive reader; since the scaffold is a fixed blob, the C++ port (and this one)
bake them in instead. They were produced against the exact bundled file with:
```text
import openskp, sys
b = sys.modules['openskp.create'].SkpBuilder()
# then print each of: b._material_insert_pos, b._base, b._layer_count_pos,
# b._orig_layer_count, b._layer_insert_pos, b._layer_writer_base, b._def_count_pos,
# b._orig_def_count, b._root_count_pos, b._orig_root_count, b._tail_pos,
# b._scaffold_next_slot, b._scaffold_class_slot
```rust
If the scaffold is ever swapped, re-run that recipe and update the file AND every constant
here together.

**Items:** [`SCAFFOLD_BLANK_V17`](#cc_skp-create-scaffold_blank_v17-scaffold_blank_v17) · [`MATERIAL_INSERT_POS`](#cc_skp-create-scaffold_blank_v17-material_insert_pos) · [`BASE`](#cc_skp-create-scaffold_blank_v17-base) · [`LAYER_COUNT_POS`](#cc_skp-create-scaffold_blank_v17-layer_count_pos) · [`ORIG_LAYER_COUNT`](#cc_skp-create-scaffold_blank_v17-orig_layer_count) · [`LAYER_INSERT_POS`](#cc_skp-create-scaffold_blank_v17-layer_insert_pos) · [`LAYER_WRITER_BASE`](#cc_skp-create-scaffold_blank_v17-layer_writer_base) · [`DEF_COUNT_POS`](#cc_skp-create-scaffold_blank_v17-def_count_pos) · [`ORIG_DEF_COUNT`](#cc_skp-create-scaffold_blank_v17-orig_def_count) · [`ROOT_COUNT_POS`](#cc_skp-create-scaffold_blank_v17-root_count_pos) · [`ORIG_ROOT_COUNT`](#cc_skp-create-scaffold_blank_v17-orig_root_count) · [`TAIL_POS`](#cc_skp-create-scaffold_blank_v17-tail_pos) · [`SCAFFOLD_NEXT_SLOT`](#cc_skp-create-scaffold_blank_v17-scaffold_next_slot) · [`PID_COUNTER_POS`](#cc_skp-create-scaffold_blank_v17-pid_counter_pos) · [`FIRST_NEW_PID`](#cc_skp-create-scaffold_blank_v17-first_new_pid) · [`TAIL_REF_POSITIONS`](#cc_skp-create-scaffold_blank_v17-tail_ref_positions) · [`ACTIVE_LAYER_ANCHOR_REL`](#cc_skp-create-scaffold_blank_v17-active_layer_anchor_rel) · [`CCAMERA_SLOT`](#cc_skp-create-scaffold_blank_v17-ccamera_slot) · [`ATTR_CONTAINER_SLOT`](#cc_skp-create-scaffold_blank_v17-attr_container_slot) · [`ATTRIBUTE_NAMED_SLOT`](#cc_skp-create-scaffold_blank_v17-attribute_named_slot) · [`ISO_CAMERA_PREFIX_OFFSET`](#cc_skp-create-scaffold_blank_v17-iso_camera_prefix_offset) · [`ISO_CAMERA_PREFIX_PATCH`](#cc_skp-create-scaffold_blank_v17-iso_camera_prefix_patch) · [`ISO_CAMERA_TAIL_PATCHES`](#cc_skp-create-scaffold_blank_v17-iso_camera_tail_patches)

<a id="cc_skp-create-scaffold_blank_v17-scaffold_blank_v17"></a>

### static `SCAFFOLD_BLANK_V17`

```rust
pub(super) static SCAFFOLD_BLANK_V17: &[u8; 5796]
```

The bundled blank v17 document.

<a id="cc_skp-create-scaffold_blank_v17-material_insert_pos"></a>

### const `MATERIAL_INSERT_POS`

```rust
pub(super) const MATERIAL_INSERT_POS: usize
```

Absolute offset of the material-manager insertion point (right before the "layer list
marker" a zero-material scaffold starts with) - Python's `_material_insert_pos`.

<a id="cc_skp-create-scaffold_blank_v17-base"></a>

### const `BASE`

```rust
pub(super) const BASE: i32
```

Starting archive slot for anything spliced in after the (empty) material section - `_base`.

<a id="cc_skp-create-scaffold_blank_v17-layer_count_pos"></a>

### const `LAYER_COUNT_POS`

```rust
pub(super) const LAYER_COUNT_POS: usize
```

Absolute offset of the u32 layer-count field that precedes the layer list.

<a id="cc_skp-create-scaffold_blank_v17-orig_layer_count"></a>

### const `ORIG_LAYER_COUNT`

```rust
pub(super) const ORIG_LAYER_COUNT: i32
```

<a id="cc_skp-create-scaffold_blank_v17-layer_insert_pos"></a>

### const `LAYER_INSERT_POS`

```rust
pub(super) const LAYER_INSERT_POS: usize
```

Absolute offset right after the scaffold's own layer records (Layer0) - where new layers go.

<a id="cc_skp-create-scaffold_blank_v17-layer_writer_base"></a>

### const `LAYER_WRITER_BASE`

```rust
pub(super) const LAYER_WRITER_BASE: i32
```

Starting archive slot for a fresh layer writer, before any material shift.

<a id="cc_skp-create-scaffold_blank_v17-def_count_pos"></a>

### const `DEF_COUNT_POS`

```rust
pub(super) const DEF_COUNT_POS: usize
```

Absolute offset of the u32 component-definition-count field.

<a id="cc_skp-create-scaffold_blank_v17-orig_def_count"></a>

### const `ORIG_DEF_COUNT`

```rust
pub(super) const ORIG_DEF_COUNT: i32
```

<a id="cc_skp-create-scaffold_blank_v17-root_count_pos"></a>

### const `ROOT_COUNT_POS`

```rust
pub(super) const ROOT_COUNT_POS: usize
```

Absolute offset of the u32 root-entity-count field.

<a id="cc_skp-create-scaffold_blank_v17-orig_root_count"></a>

### const `ORIG_ROOT_COUNT`

```rust
pub(super) const ORIG_ROOT_COUNT: i32
```

<a id="cc_skp-create-scaffold_blank_v17-tail_pos"></a>

### const `TAIL_POS`

```rust
pub(super) const TAIL_POS: usize
```

Absolute offset where the document "tail" (the undecoded style/font-manager region) begins.

<a id="cc_skp-create-scaffold_blank_v17-scaffold_next_slot"></a>

### const `SCAFFOLD_NEXT_SLOT`

```rust
pub(super) const SCAFFOLD_NEXT_SLOT: i32
```

Starting slot for anything written after the byte-for-byte-copied layer/definition/root
region - `_scaffold_next_slot`.

<a id="cc_skp-create-scaffold_blank_v17-pid_counter_pos"></a>

### const `PID_COUNTER_POS`

```rust
pub(super) const PID_COUNTER_POS: usize
```

Offset of the u64 next persistent entity id. Ids are document-wide, not per archive section;
the scaffold has already consumed the ids below `FIRST_NEW_PID`.

<a id="cc_skp-create-scaffold_blank_v17-first_new_pid"></a>

### const `FIRST_NEW_PID`

```rust
pub(super) const FIRST_NEW_PID: u64
```

The first persistent id free for new objects in the scaffold.

<a id="cc_skp-create-scaffold_blank_v17-tail_ref_positions"></a>

### const `TAIL_REF_POSITIONS`

```rust
pub(super) const TAIL_REF_POSITIONS: [usize; 6]
```

Offsets, relative to the tail start, of internal slot references that must be renumbered by
the number of archive slots inserted before them.

<a id="cc_skp-create-scaffold_blank_v17-active_layer_anchor_rel"></a>

### const `ACTIVE_LAYER_ANCHOR_REL`

```rust
pub(super) const ACTIVE_LAYER_ANCHOR_REL: usize
```

Offset (relative to the layer insertion point) of the active-layer anchor - a back-reference
to Layer0. Moves only when materials shift Layer0's own slot.

<a id="cc_skp-create-scaffold_blank_v17-ccamera_slot"></a>

### const `CCAMERA_SLOT`

```rust
pub(super) const CCAMERA_SLOT: i32
```

Classes the scaffold's own prefix already declares, so the writer only ever short-refs them.

<a id="cc_skp-create-scaffold_blank_v17-attr_container_slot"></a>

### const `ATTR_CONTAINER_SLOT`

```rust
pub(super) const ATTR_CONTAINER_SLOT: i32
```

<a id="cc_skp-create-scaffold_blank_v17-attribute_named_slot"></a>

### const `ATTRIBUTE_NAMED_SLOT`

```rust
pub(super) const ATTRIBUTE_NAMED_SLOT: i32
```

<a id="cc_skp-create-scaffold_blank_v17-iso_camera_prefix_offset"></a>

### const `ISO_CAMERA_PREFIX_OFFSET`

```rust
pub(super) const ISO_CAMERA_PREFIX_OFFSET: usize
```

The scaffold ships SketchUp's arbitrary default camera; every written file is patched to the
standard "Iso" view instead (parallel projection, looking at the origin from the (1, -1, 1)
octant). Found by diffing two SDK-authored blank documents that differ only in an explicit
`SUCameraSetOrientation` + `SUCameraSetPerspective(false)` - copied verbatim, not decoded.

<a id="cc_skp-create-scaffold_blank_v17-iso_camera_prefix_patch"></a>

### const `ISO_CAMERA_PREFIX_PATCH`

```rust
pub(super) const ISO_CAMERA_PREFIX_PATCH: [u8; 98]
```

<a id="cc_skp-create-scaffold_blank_v17-iso_camera_tail_patches"></a>

### const `ISO_CAMERA_TAIL_PATCHES`

```rust
pub(super) const ISO_CAMERA_TAIL_PATCHES: [(usize, &[u8]); 2]
```

Iso-camera patches inside the tail, as (offset relative to the tail start, bytes).

<a id="mod-cc_skp-curves"></a>

## `cc_skp::curves`

<sub>`crates/cc_skp/src/curves.rs`</sub>

Loose-edge chaining and analytic-arc recovery, shared by the baked and the
instanced scene builders (Python port extension: `openskp/_curves.py`; the
C++ port has no equivalent).
Loose edges - edges no face uses - are how SketchUp stores drawing
geometry (a facade elevation, a section outline). Edges bounding a face are
already represented by that face's mesh and are not emitted again.
The run order follows the Python reference exactly, including the
iteration order of the CPython `set` it walks a curve's edges from (see
`py_set_order`), so a run comes out in the same direction there and here.

**Items:** [`INCHES_TO_M`](#cc_skp-curves-inches_to_m) · [`LooseRun`](#cc_skp-curves-looserun) · [`round6`](#cc_skp-curves-round6) · [`py_set_order`](#cc_skp-curves-py_set_order) · [`majority_layer`](#cc_skp-curves-majority_layer) · [`chain_loose_edges`](#cc_skp-curves-chain_loose_edges) · [`solve_run_arc`](#cc_skp-curves-solve_run_arc)

<a id="cc_skp-curves-inches_to_m"></a>

### const `INCHES_TO_M`

```rust
pub const INCHES_TO_M: f64
```

Inches to metres.

<a id="cc_skp-curves-looserun"></a>

### struct `LooseRun`

```rust
pub struct LooseRun
```

One chained run of loose edges: its edges in walk order, the vertex ids
along it, and whether it returns to its first vertex (which is then NOT
repeated at the end).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Eq`, `Default`

**Fields**

- `edge_ids: Vec<EntityId>`
- `vertex_ids: Vec<EntityId>`
- `closed: bool`

<a id="cc_skp-curves-round6"></a>

### fn `round6`

```rust
pub fn round6(x: f64) -> f64
```

Python's `round(x, 6)`: the double nearest the decimal rounding of `x`'s
exact value to 6 places, ties to even.

<a id="cc_skp-curves-py_set_order"></a>

### fn `py_set_order`

```rust
pub fn py_set_order(keys: &[EntityId]) -> Vec<EntityId>
```

The iteration order of CPython's `set(keys)` for integer keys
(`Objects/setobject.c`: open addressing with 9 linear probes, then
perturbed probing; the table grows 4x once 3/5 full). Duplicates keep their
first occurrence.

<a id="cc_skp-curves-majority_layer"></a>

### fn `majority_layer`

```rust
pub(crate) fn majority_layer(ids: impl Iterator<Item = Option<EntityId>>) -> Option<EntityId>
```

The majority layer id (first wins a tie) among `ids`, ignoring absent and
zero ids; `None` when there is none.

<a id="cc_skp-curves-chain_loose_edges"></a>

### fn `chain_loose_edges`

```rust
pub fn chain_loose_edges(b: &GeometryBuilder) -> Vec<LooseRun>
```

Turns a definition's *loose* edges into polyline runs.
Grouping comes from the file (`edge_curves`, SketchUp's own `Edge#curve`),
not from vertex adjacency: an edge with no curve is its own two-point run,
and two curves touching at a vertex stay two runs. Ungrouped edges come
first (in edge order), then each curve in first-seen order.

<a id="cc_skp-curves-solve_run_arc"></a>

### fn `solve_run_arc`

```rust
pub fn solve_run_arc(b: &GeometryBuilder, eids: &[EntityId], matrix: Option<&[f64]>, pts: &[Vec3], closed: bool) -> Option<CurveArc>
```

Recovers the analytic arc behind one loose-edge run, or `None` (the
answer for almost every run, and the safe one: the caller then keeps the
chords the file stores).
The frame (`arc_curves`) is used only if (1) the run is the whole curve,
(2) the frame is a circle (`y_axis == normal x x_axis`, not the affine
image of one), and (3) every point lies on that circle. `pts` are the
run's points as the caller produced them (metres, glTF Y-up, rounded to 6
places) after transforming by `matrix` (`None` = local space); the result
is in the same frame. A full turn comes back as two half-turn segments.

<a id="mod-cc_skp-deflate"></a>

## `cc_skp::deflate`

<sub>`crates/cc_skp/src/deflate.rs`</sub>

A small pure-Rust DEFLATE (RFC 1951) / zlib (RFC 1950) compressor:
hash-chain LZ77 with optional one-step lazy matching, and per block the
cheapest of stored / fixed-Huffman / dynamic-Huffman coding.
Output is valid DEFLATE but not byte-identical to miniz's `tdefl`.

**Items:** [`deflate_raw`](#cc_skp-deflate-deflate_raw) · [`zlib_compress`](#cc_skp-deflate-zlib_compress)

<a id="cc_skp-deflate-deflate_raw"></a>

### fn `deflate_raw`

```rust
pub fn deflate_raw(data: &[u8], level: u32) -> Vec<u8>
```

Compresses `data` into a raw DEFLATE stream. `level` 0 stores; 1-10 trade
speed for ratio (longer hash chains, lazy matching from 4).

<a id="cc_skp-deflate-zlib_compress"></a>

### fn `zlib_compress`

```rust
pub fn zlib_compress(data: &[u8], level: u32) -> Vec<u8>
```

Compresses `data` into a zlib stream (2-byte header, DEFLATE, Adler-32).

<a id="mod-cc_skp-earcut"></a>

## `cc_skp::earcut`

<sub>`crates/cc_skp/src/earcut.rs`</sub>

2D ear-clipping with holes (`earcut.cpp`): `EarPoint` and `earcut_2d`.
Every holeless loop is clipped directly; each hole is bridged into the
outer boundary from its right-most vertex to the nearest boundary vertex
on its right. When two or more holes genuinely overlap, bridging them
independently can produce a self-intersecting ring, so the outer boundary
is clipped alone and triangles whose centroid falls inside any hole are
dropped instead (openskp#285).

**Items:** [`EarPoint`](#cc_skp-earcut-earpoint) · [`MAX_EAR_VERTICES`](#cc_skp-earcut-max_ear_vertices) · [`MAX_CLIP_WORK`](#cc_skp-earcut-max_clip_work) · [`earcut_2d`](#cc_skp-earcut-earcut_2d)

<a id="cc_skp-earcut-earpoint"></a>

### struct `EarPoint`

```rust
pub struct EarPoint
```

One projected 2D vertex, tagged with the source vertex id.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`

**Fields**

- `x: f64`
- `y: f64`
- `id: EntityId`

<a id="cc_skp-earcut-max_ear_vertices"></a>

### const `MAX_EAR_VERTICES`

```rust
pub const MAX_EAR_VERTICES: usize
```

Most vertices (all loops of a face) ear clipped; a larger face is fanned
from the first vertex of its outer loop instead, its holes left out
(exact for convex outlines, and bounded for any): checking holes for
overlaps and clipping cost the square of the vertices or more.

<a id="cc_skp-earcut-max_clip_work"></a>

### const `MAX_CLIP_WORK`

```rust
pub const MAX_CLIP_WORK: usize
```

Most steps (vertices looked at, ear tests, vertices shifted) one ring is
clipped with; what is left past them is fanned.

<a id="cc_skp-earcut-earcut_2d"></a>

### fn `earcut_2d`

```rust
pub fn earcut_2d(mut loops: Vec<Vec<EarPoint>>) -> Vec<[EntityId; 3]>
```

Triangulates `loops[0]` (outer boundary) minus `loops[1..]` (holes),
returning vertex-id triples.

<a id="mod-cc_skp-edit"></a>

## `cc_skp::edit`

<sub>`crates/cc_skp/src/edit.rs`</sub>

Load an existing legacy-format `.skp` file and rebuild it as a new, independent
`SkpBuilder` (`edit.hpp` / `edit.cpp`, itself a port of Python's `openskp.edit`).
[`create`](mod@crate::create) only ever builds a brand-new file by splicing new geometry into its
own bundled blank scaffold - there is no way to append to or patch an arbitrary existing file's
bytes in place, because real SketchUp itself doesn't do that either: it fully re-serializes the
whole document on every save, so there is no stable "original bytes + appended bytes"
structure to target for a file this project didn't create.
This module takes the other viable approach instead: fully parse the existing file with this
crate's own reader, then *replay* everything it understood back through the
[`create`](mod@crate::create) public API (materials, layers, every component definition, every
face/instance, construction points and lines) to produce a brand-new file - not a byte-patched
copy of the original, but a freshly-built one with equivalent content, to which the caller can
add more geometry before saving.
**Adding more geometry after the fact.** The returned builder can take more `add_face`/
`add_circle`/`add_instance`/etc. calls, and every material/layer the source had is already
reachable via `SkpBuilder::materials_by_name`/`SkpBuilder::layers_by_name`;
`OpenExistingResult::definitions` maps each component definition's name to its handle, for
placing more instances of something the source already defined. What the returned builder can
no longer do is register a genuinely NEW material, layer, or component definition/group - the
writer's own file-format ordering requirement (materials/layers/definitions must all be
finalized before any geometry is written) is already satisfied by the time replay finishes
writing the source's own root-level geometry (which happens for any source file with root-level
content - in practice, almost always), so `add_material`/`add_layer`/
`add_component_definition`/`add_group` all fail with `SkpWriteError` on the returned builder
in that case. Build anything new into a separate [`create()`](crate::create::create) call
instead.
**Scope and known fidelity gaps** (every gap here is a genuine, deliberately-scoped limitation
carried over from the Python port, not an oversight):

* Only a **legacy-format** (SketchUp 2013-2020) source file is accepted - the writer never
  writes any other format, so a modern VFF (2021+) source can't be faithfully round-tripped
  through it.
* Per-edge `hidden`/`soft`/`smooth` flags are applied per-FACE, not per-edge (an "any edge in
  this boundary has the flag" approximation) - `add_face` can only set these uniformly for
  every edge it newly declares in one call, the same limitation any user of that API has.
* A positioned texture is replayed via 3 sample-point correspondences fitted to an affine map
  (see `FaceOptions::front_uv`/`FaceOptions::back_uv`) - exact at those 3 points, but a
  genuinely projective (4-pin/distorted) source mapping won't interpolate identically between
  them. A *projected* (draped) texture has no equivalent at all and falls back to the default
  projection.
* A colorized (tinted) material variant is replayed as its plain source texture, losing the
  tint.
* Per-face material/layer painting: only a face's front/back *material* is replayed - the
  reader doesn't expose a per-face layer assignment for legacy files (only instances carry an
  explicit layer).
* Every placed thing (originally a group or a component instance alike) is replayed as a plain
  component instance - structurally simpler, and visually identical, but no longer shows as a
  "Group" in SketchUp's Outliner afterward.
* Section planes, text entities, and dimensions aren't carried over at all.
* A circle/arc/polyline's original `CArcCurve`/`CCurve` grouping is lost - a round-tripped
  circle becomes an ordinary straight-edged face.
* Definition-level and face-level custom attributes aren't reproduced - only an instance's own
  `properties` are.
Every one of these gaps is surfaced in the returned `warnings` list wherever it actually
affected the specific source file being replayed, not just documented here.

**Items:** [`OpenExistingResult`](#cc_skp-edit-openexistingresult) · [`OpenExistingError`](#cc_skp-edit-openexistingerror) · [`EdgeMap`](#cc_skp-edit-edgemap) · [`build_edge_map`](#cc_skp-edit-build_edge_map) · [`reconstruct_loop_vertices`](#cc_skp-edit-reconstruct_loop_vertices) · [`face_uv_basis`](#cc_skp-edit-face_uv_basis) · [`texture_suffix`](#cc_skp-edit-texture_suffix) · [`non_collinear_triple`](#cc_skp-edit-non_collinear_triple) · [`open_existing`](#cc_skp-edit-open_existing)

<a id="cc_skp-edit-openexistingresult"></a>

### struct `OpenExistingResult`

```rust
pub struct OpenExistingResult
```

Result of `open_existing`.

**Traits:** `Debug`

**Fields**

- `builder: SkpBuilder`

  Ready for more `add_face`/`add_circle`/`add_instance`/etc. calls before
  `SkpBuilder::save`. Every material and layer the source file had is already reachable via
  `SkpBuilder::materials_by_name`/`SkpBuilder::layers_by_name`.

- `warnings: Vec<String>`

  Anything from the source file that couldn't be faithfully reproduced - see the module docs
  for the exact, deliberately-scoped gaps this draws from.

- `definitions: BTreeMap<String, DefinitionHandle>`

  Maps each replayed component definition's own name to its (already-closed) definition, so
  the caller can place additional instances of something the source file already defined via
  `builder.add_instance(definitions["Wheel"], &options)`. If two source definitions share a
  name, the later one (by entity id) wins - real SketchUp allows duplicate component names;
  only this convenience lookup needs them unique. Definitions with an empty name are not
  reachable through this map (but are still replayed and instantiable through their
  instances).


<a id="cc_skp-edit-openexistingerror"></a>

### enum `OpenExistingError`

```rust
pub enum OpenExistingError
```

Why `open_existing` failed. The C++ port throws one of two exception types; this keeps them
apart.

**Traits:** `Debug`, `fmt::Display`, `Error`, `From<SkpWriteError>`, `From<SkpParseError>`

**Variants**

- `Write(SkpWriteError)`

  The file can't be read, isn't a legacy-format file, or replaying it through the writer
  failed (C++ `SkpWriteError`).

- `Parse(SkpParseError)`

  The reader rejected the file (C++ `SkpParseError`).


<a id="cc_skp-edit-edgemap"></a>

### type alias `EdgeMap`

```rust
pub(crate) type EdgeMap = BTreeMap<EntityId, (EntityId, EntityId)>
```

<a id="cc_skp-edit-build_edge_map"></a>

### fn `build_edge_map`

```rust
pub(crate) fn build_edge_map(defn: &Definition) -> EdgeMap
```

<a id="cc_skp-edit-reconstruct_loop_vertices"></a>

### fn `reconstruct_loop_vertices`

```rust
pub(crate) fn reconstruct_loop_vertices(lp: &[CoEdge], edges: &EdgeMap) -> Vec<EntityId>
```

<a id="cc_skp-edit-face_uv_basis"></a>

### fn `face_uv_basis`

```rust
pub(crate) fn face_uv_basis(n: Point3) -> (Point3, Point3)
```

<a id="cc_skp-edit-texture_suffix"></a>

### fn `texture_suffix`

```rust
pub(crate) fn texture_suffix(filename: &str) -> &str
```

The image's file-name suffix (from the last `.`), `.png` if there is none.

<a id="cc_skp-edit-non_collinear_triple"></a>

### fn `non_collinear_triple`

```rust
pub(crate) fn non_collinear_triple(points: &[Point3]) -> Option<[Point3; 3]>
```

`front_uv`/`back_uv` need exactly 3 correspondences whose (u, v) values are NOT collinear (an
affine fit is impossible otherwise) - real faces can have a "flat" vertex (three consecutive
vertices genuinely collinear in 3D), which `points[0..2]` alone isn't guaranteed to avoid.
Search for the first non-collinear triple - an affine map preserves collinearity, so a
non-collinear triple in 3D is also non-collinear in (u, v).

<a id="cc_skp-edit-open_existing"></a>

### fn `open_existing`

```rust
pub fn open_existing(path: impl AsRef<Path>) -> Result<OpenExistingResult, OpenExistingError>
```

Parse `path` (a legacy-format `.skp` file) and rebuild it as a new `SkpBuilder`, replaying
materials, layers, every component definition, and all root-level geometry/instances. See the
module docs for exactly what is and isn't reproduced.
Fails with `OpenExistingError::Write` if `path` can't be read or isn't a legacy-format file
(or replay fails), and with `OpenExistingError::Parse` if the reader rejects it.

<a id="mod-cc_skp-errors"></a>

## `cc_skp::errors`

<sub>`crates/cc_skp/src/errors.rs`</sub>

Structured parse errors (`errors.hpp` / `errors.cpp`).

**Items:** [`ErrorCause`](#cc_skp-errors-errorcause) · [`SkpParseError`](#cc_skp-errors-skpparseerror) · [`Result`](#cc_skp-errors-result)

<a id="cc_skp-errors-errorcause"></a>

### type alias `ErrorCause`

```rust
pub type ErrorCause = Arc<dyn Error + Send + Sync + 'static>
```

The underlying error an `SkpParseError` wraps (C++'s `std::exception_ptr`).

<a id="cc_skp-errors-skpparseerror"></a>

### struct `SkpParseError`

```rust
pub struct SkpParseError(Box<Inner>)
```

A parse failure with optional context: the pipeline stage, the record being
processed, its tag/offset, the definition, and the underlying cause.
`Display` renders the same text as the C++ `what()`:
`message | stage=... | record=r/t | tag=... | offset=0x... | definitionId=...`.

**Traits:** `Clone`, `fmt::Display`, `fmt::Debug`, `Error`

**Methods**

- `pub fn new(message: impl Into<String>) -> Self`

  An error with just a message.

- `pub fn at_stage(message: impl Into<String>, stage: ParseStage) -> Self`

  An error with a message and the stage it happened in.

- `pub fn with_context(message: impl Into<String>, stage: Option<ParseStage>, record_index: Option<u64>, total_records: Option<u64>, tag: Option<String>, offset: Option<u64>, definition_id: Option<i64>, cause: Option<ErrorCause>) -> Self`

  Every field at once, in the C++ constructor's order.

- `pub fn with_stage(mut self, stage: ParseStage) -> Self`
- `pub fn with_record(mut self, index: u64, total: u64) -> Self`
- `pub fn with_tag(mut self, tag: impl Into<String>) -> Self`
- `pub fn with_offset(mut self, offset: u64) -> Self`
- `pub fn with_definition_id(mut self, id: i64) -> Self`
- `pub fn with_cause<E: Error + Send + Sync + 'static>(mut self, cause: E) -> Self`
- `pub fn with_cause_arc(mut self, cause: ErrorCause) -> Self`
- `pub fn message(&self) -> &str`

  The bare message, without the formatted context.

- `pub fn stage(&self) -> Option<ParseStage>`
- `pub fn record_index(&self) -> Option<u64>`
- `pub fn total_records(&self) -> Option<u64>`
- `pub fn tag(&self) -> Option<&str>`
- `pub fn offset(&self) -> Option<u64>`
- `pub fn definition_id(&self) -> Option<i64>`
- `pub fn cause(&self) -> Option<&ErrorCause>`

  The wrapped underlying error, if any.

- `pub fn what(&self) -> String`

  The full formatted text (C++ `what()`).


<a id="cc_skp-errors-result"></a>

### type alias `Result`

```rust
pub type Result<T, E = SkpParseError> = std::result::Result<T, E>
```

Crate-wide result alias.

<a id="mod-cc_skp-face_groups"></a>

## `cc_skp::face_groups`

<sub>`crates/cc_skp/src/face_groups.rs`</sub>

Local-space face grouping shared by the baked (`scene_build.rs`) and
instanced (`instanced_scene_build.rs`) scene builders (`face_groups.hpp` /
`face_groups.cpp`).
A definition's faces are grouped by resolved (color, double_sided, texture)
identity in DEFINITION-LOCAL space (inches, SketchUp Z-up): exactly what
the baked builder assembles just before applying an instance's world
matrix, and exactly what the instanced builder keeps local and puts on the
node instead. One implementation keeps the two paths agreeing on
triangulation, UV seams, normals and front/back handling.
An unpainted face falls back to the caller's `fallback_color` for color,
but its material (and so its texture tile size) comes from the face's OWN
`material_id` / `back_material_id` only. The color key carries a 4th
(alpha) component.

**Items:** [`FaceColorKey`](#cc_skp-face_groups-facecolorkey) · [`GroupKey`](#cc_skp-face_groups-groupkey) · [`FaceVKey`](#cc_skp-face_groups-facevkey) · [`Group`](#cc_skp-face_groups-group) · [`find_material`](#cc_skp-face_groups-find_material) · [`material_color`](#cc_skp-face_groups-material_color) · [`default_color`](#cc_skp-face_groups-default_color) · [`sniff_image_mime`](#cc_skp-face_groups-sniff_image_mime) · [`TextureRegistry`](#cc_skp-face_groups-textureregistry) · [`FaceGroupContext`](#cc_skp-face_groups-facegroupcontext) · [`build_local_face_groups`](#cc_skp-face_groups-build_local_face_groups)

<a id="cc_skp-face_groups-facecolorkey"></a>

### type alias `FaceColorKey`

```rust
pub type FaceColorKey = [i32; 4]
```

Color key: (r, g, b, alpha-from-transparency).

<a id="cc_skp-face_groups-groupkey"></a>

### struct `GroupKey`

```rust
pub struct GroupKey
```

A face group's identity. Ordered like the C++ `std::tie(color,
double_sided, texture_index)` (an absent texture sorts first).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Default`

**Fields**

- `color: FaceColorKey`
- `double_sided: bool`
- `texture_index: Option<usize>`

  The texture is part of the identity, not just the color: two different
  images can average to the same RGB.


<a id="cc_skp-face_groups-facevkey"></a>

### type alias `FaceVKey`

```rust
pub type FaceVKey = (EntityId, u64, u64)
```

A vertex is keyed by (source vertex id, u, v): UVs are per-face, so a
position shared by two faces that disagree on texture mapping becomes two
output vertices. The doubles are stored as bits with `-0.0` folded into
`0.0`, matching the C++ `std::map`'s `<` equivalence.

<a id="cc_skp-face_groups-group"></a>

### struct `Group`

```rust
pub struct Group
```

One group's local-space mesh. `verts`, `uvs` and `normals` (accumulated,
unnormalised) are aligned by output vertex index.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `verts: Vec<Vec3>`
- `uvs: Vec<[f32; 2]>`
- `tris: Vec<[usize; 3]>`
- `map: HashMap<FaceVKey, usize>`
- `normals: Vec<Vec3>`

<a id="cc_skp-face_groups-find_material"></a>

### fn `find_material`

```rust
pub fn find_material(parsed: &RawParsed, id: Option<EntityId>) -> Option<&RawMaterial>
```

The material a face (or instance) material id names: by name first, then
by material folder.

<a id="cc_skp-face_groups-material_color"></a>

### fn `material_color`

```rust
pub fn material_color(material: Option<&RawMaterial>) -> Option<FaceColorKey>
```

A material's color key. Opacity combines the RGBA color record's alpha
byte and the XML material's transparency; the untouched one defaults to
fully opaque.

<a id="cc_skp-face_groups-default_color"></a>

### fn `default_color`

```rust
pub fn default_color(parsed: &RawParsed, layer: &str) -> FaceColorKey
```

The layer's color (or SketchUp's default grey), fully opaque.

<a id="cc_skp-face_groups-sniff_image_mime"></a>

### fn `sniff_image_mime`

```rust
pub fn sniff_image_mime(data: &[u8]) -> Option<&'static str>
```

An image's MIME type from its magic bytes; `None` for anything glTF cannot
carry (glTF only allows PNG and JPEG).

<a id="cc_skp-face_groups-textureregistry"></a>

### struct `TextureRegistry`

```rust
pub struct TextureRegistry
```

The scene's texture table, deduplicated by bytes (the builders'
`texture_index_for`): the same image routinely backs several materials.

**Traits:** `Debug`, `Clone`, `Default`

**Fields**

- `textures: Vec<SceneTexture>`

*Some fields are private.*

**Methods**

- `pub fn index_for(&mut self, material: Option<&RawMaterial>) -> Option<usize>`

  The texture index for a material's image, registering it on first use;
  `None` when the material has no image glTF can carry.


<a id="cc_skp-face_groups-facegroupcontext"></a>

### struct `FaceGroupContext`

```rust
pub struct FaceGroupContext<'a>
```

What `build_local_face_groups` needs from its caller besides the builder.

**Fields**

- `texture_index_for: &'a mut dyn FnMut(Option<&RawMaterial>) -> Option<usize>`
- `fallback_color: FaceColorKey`

  Color an unpainted face falls back to (the instance's inherited paint
  color, or the effective layer's color when nothing is inherited).


<a id="cc_skp-face_groups-build_local_face_groups"></a>

### fn `build_local_face_groups`

```rust
pub fn build_local_face_groups(parsed: &RawParsed, b: &GeometryBuilder, ctx: &mut FaceGroupContext<'_>) -> Result<BTreeMap<GroupKey, Group>, Error>
```

Groups a definition's faces by resolved material identity, in local space.
A face whose front and back resolve to the SAME color is emitted once with
`double_sided` set; a face whose sides differ is emitted as two
single-sided triangle sets (one normal-wound front, one reverse-wound back)
so each side keeps its own color.
Fails (`OutOfRange`, the C++ `map::at`) when a triangle references a
vertex the builder does not have.

<a id="mod-cc_skp-inflate"></a>

## `cc_skp::inflate`

<sub>`crates/cc_skp/src/inflate.rs`</sub>

Pure-Rust DEFLATE (RFC 1951) and zlib (RFC 1950) decoder, replacing the
parts of miniz's `tinfl` / `mz_uncompress` the reader uses.
Validation follows tinfl: a Huffman code must be complete unless it uses at
most one symbol, a length-repeat code may not start the code-length list,
a distance may not reach before the start of the output, and a stored
block's LEN must match ~NLEN. Output can be capped; exceeding the cap is an
error (tinfl's non-wrapping output buffer).

**Items:** [`InflateError`](#cc_skp-inflate-inflateerror) · [`inflate_raw`](#cc_skp-inflate-inflate_raw) · [`adler32`](#cc_skp-inflate-adler32) · [`zlib_decompress`](#cc_skp-inflate-zlib_decompress) · [`zlib_decompress_capped`](#cc_skp-inflate-zlib_decompress_capped) · [`UncompressError`](#cc_skp-inflate-uncompresserror) · [`uncompress`](#cc_skp-inflate-uncompress)

<a id="cc_skp-inflate-inflateerror"></a>

### enum `InflateError`

```rust
pub enum InflateError
```

Why a stream failed to decode.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `fmt::Display`, `std::error::Error`

**Variants**

- `Truncated`

  The input ended before the stream did.

- `InvalidBlockType`

  Block type 3.

- `StoredLengthMismatch`

  A stored block's LEN is not the complement of NLEN.

- `InvalidCodeLengths`

  Over-subscribed / incomplete Huffman code, or a bad repeat code.

- `InvalidSymbol`

  A bit pattern or symbol that no valid code produces.

- `DistanceTooFar`

  A back-reference before the start of the output.

- `OutputLimit`

  The output would exceed the caller's cap.

- `BadZlibHeader`

  Bad zlib CMF/FLG header (or a preset dictionary).

- `Adler32Mismatch`

  zlib Adler-32 trailer mismatch.


<a id="cc_skp-inflate-inflate_raw"></a>

### fn `inflate_raw`

```rust
pub fn inflate_raw(input: &[u8], limit: usize) -> Result<Vec<u8>, InflateError>
```

Decodes a raw DEFLATE stream (no zlib wrapper). Fails with
`InflateError::OutputLimit` when the output would exceed `limit` bytes.

<a id="cc_skp-inflate-adler32"></a>

### fn `adler32`

```rust
pub fn adler32(data: &[u8]) -> u32
```

Adler-32 checksum.

<a id="cc_skp-inflate-zlib_decompress"></a>

### fn `zlib_decompress`

```rust
pub fn zlib_decompress(input: &[u8]) -> Result<Vec<u8>, InflateError>
```

Decodes a complete zlib stream (RFC 1950), verifying the Adler-32 trailer.
The output size is **not** limited: a small hostile stream can expand to
gigabytes. Use this only on input you trust (for example, data you just
compressed yourself); for untrusted input use `zlib_decompress_capped`.
The library itself never calls this on file data.

<a id="cc_skp-inflate-zlib_decompress_capped"></a>

### fn `zlib_decompress_capped`

```rust
pub fn zlib_decompress_capped(input: &[u8], limit: usize) -> Result<Vec<u8>, InflateError>
```

Decodes a complete zlib stream (RFC 1950) up to `limit` uncompressed bytes,
verifying the Adler-32 trailer.

<a id="cc_skp-inflate-uncompresserror"></a>

### enum `UncompressError`

```rust
pub enum UncompressError
```

Result codes of `uncompress`, mirroring `mz_uncompress`.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `fmt::Display`, `std::error::Error`

**Variants**

- `Buf`

  `MZ_BUF_ERROR`: the destination capacity is too small.

- `Data`

  `MZ_DATA_ERROR`: the stream is malformed or truncated.


<a id="cc_skp-inflate-uncompress"></a>

### fn `uncompress`

```rust
pub fn uncompress(src: &[u8], dest_cap: usize) -> Result<Vec<u8>, UncompressError>
```

`mz_uncompress` equivalent: decodes a zlib stream into at most `dest_cap`
bytes. A too-small destination is `UncompressError::Buf` (callers grow
and retry); anything else that fails is `UncompressError::Data`.

<a id="mod-cc_skp-instanced_scene"></a>

## `cc_skp::instanced_scene`

<sub>`crates/cc_skp/src/instanced_scene.rs`</sub>

The instanced scene graph (`instanced_scene.hpp`). Types only; the builder
lives in `instanced_scene_build.rs`.

**Items:** [`LocalPrimitive`](#cc_skp-instanced_scene-localprimitive) · [`InstancedMeshResource`](#cc_skp-instanced_scene-instancedmeshresource) · [`LocalCurve`](#cc_skp-instanced_scene-localcurve) · [`InstancedCurveResource`](#cc_skp-instanced_scene-instancedcurveresource) · [`IDENTITY_GLTF`](#cc_skp-instanced_scene-identity_gltf) · [`InstancedNode`](#cc_skp-instanced_scene-instancednode) · [`SceneBounds`](#cc_skp-instanced_scene-scenebounds) · [`InstancedScene`](#cc_skp-instanced_scene-instancedscene)

<a id="cc_skp-instanced_scene-localprimitive"></a>

### struct `LocalPrimitive`

```rust
pub struct LocalPrimitive
```

One reusable, DEFINITION-LOCAL triangulated mesh: the instanced
counterpart of `GlbPrimitive`, minus the world transform (metres, glTF
Y-up). Normal transformation is deferred to the consumer's node transform.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `positions: Vec<f32>`
- `normals: Vec<f32>`
- `uvs: Vec<f32>`
- `indices: Vec<u32>`
- `material_index: usize`

<a id="cc_skp-instanced_scene-instancedmeshresource"></a>

### struct `InstancedMeshResource`

```rust
pub struct InstancedMeshResource
```

A definition's geometry resolved for one rendering context. One definition
can yield more than one resource (e.g. painted two colors) - see
`variant_key`.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `id: String`
- `definition_id: Option<EntityId>`

  `None` for the root definition (loose, non-component geometry).

- `definition_name: String`
- `variant_key: String`
- `primitives: Vec<LocalPrimitive>`

<a id="cc_skp-instanced_scene-localcurve"></a>

### struct `LocalCurve`

```rust
pub struct LocalCurve
```

One loose-edge run in definition-local space (Python port extension).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `points_m: Vec<[f64; 3]>`
- `closed: bool`
- `layer: String`

  This run's own resolved layer.

- `arc: Option<CurveArc>`

<a id="cc_skp-instanced_scene-instancedcurveresource"></a>

### struct `InstancedCurveResource`

```rust
pub struct InstancedCurveResource
```

A definition's loose edges, ready to be referenced by any number of
nodes - the loose-edge counterpart of `InstancedMeshResource` (Python
port extension).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `id: String`
- `definition_id: Option<EntityId>`
- `definition_name: String`
- `curves: Vec<LocalCurve>`

<a id="cc_skp-instanced_scene-identity_gltf"></a>

### const `IDENTITY_GLTF`

```rust
pub const IDENTITY_GLTF: [f64; 16]
```

The glTF identity matrix.

<a id="cc_skp-instanced_scene-instancednode"></a>

### struct `InstancedNode`

```rust
pub struct InstancedNode
```

One placed node in the instanced scene graph.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `name: String`
- `name_is_generated: bool`

  True when `name` is a generated fallback rather than a real name.

- `definition_name: String`
- `layer: String`
- `guid: String`

  This instance's own persistent GUID (VFF), or `""`.

- `matrix: [f64; 16]`

  Transform RELATIVE TO THE PARENT, column-major glTF (metres, Y-up).

- `position_mm: [f64; 3]`
- `properties: BTreeMap<String, String>`
- `attribute_dictionaries: StringDictionaries`

  Every attribute dictionary other than `dynamic_attributes` and
  `SU_InstanceSet`, stringified.

- `mesh_resource_id: Option<String>`
- `curve_resource_id: Option<String>`

  Python port extension: the loose-edge resource this node places.

- `children: Vec<InstancedNode>`

<a id="cc_skp-instanced_scene-scenebounds"></a>

### struct `SceneBounds`

```rust
pub struct SceneBounds
```

Axis-aligned bounds of the scene as PLACED, metres and Y-up.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`

**Fields**

- `min: [f64; 3]`
- `max: [f64; 3]`
- `size: [f64; 3]`
- `center: [f64; 3]`

<a id="cc_skp-instanced_scene-instancedscene"></a>

### struct `InstancedScene`

```rust
pub struct InstancedScene
```

The result of `build_instanced_scene()`: the placed scene graph with
SketchUp's instancing PRESERVED.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `bounds: Option<SceneBounds>`
- `scene_hierarchy: InstancedNode`
- `mesh_resources: Vec<InstancedMeshResource>`
- `gltf_materials: Vec<GltfMaterial>`
- `textures: Vec<SceneTexture>`
- `layer_hidden: BTreeMap<String, bool>`

  The file's own per-layer visibility, by layer name.

- `curve_resources: Vec<InstancedCurveResource>`

  Loose-edge resources, one per (definition, effective layer) (Python
  port extension). Empty for a model whose geometry is all faces.


<a id="mod-cc_skp-instanced_scene_build"></a>

## `cc_skp::instanced_scene_build`

<sub>`crates/cc_skp/src/instanced_scene_build.rs`</sub>

The instanced scene builder (`instanced_scene.cpp`'s
`build_instanced_scene_raw`), plus the Python port's loose-edge curve
resources (`instanced_scene.py`'s `curve_resources`).

**Items:** [`build_instanced_scene_raw`](#cc_skp-instanced_scene_build-build_instanced_scene_raw)

<a id="cc_skp-instanced_scene_build-build_instanced_scene_raw"></a>

### fn `build_instanced_scene_raw`

```rust
pub fn build_instanced_scene_raw(mut p: RawParsed, o: &ParseOptions) -> Result<InstancedScene, SkpParseError>
```

Builds the instanced scene graph from a reader's raw output.

<a id="mod-cc_skp-internal"></a>

## `cc_skp::internal`

<sub>`crates/cc_skp/src/internal/mod.rs`</sub>

Reader infrastructure shared by the VFF reader, the legacy reader and the
scene builders (`internal.hpp`).
Public (but `#[doc(hidden)]` from the crate root) so integration tests and
sibling modules can reach the same internals the C++ tests include.

**Re-exports**

- `pub use self::build_model::build_model`
- `pub use self::bytes::{parse_flat, parse_tlv_recursive, parse_varint, read_f64, read_i32, read_u16, read_u32, tag_at}`
- `pub use self::core::{decode_xml_entities, full_parse}`
- `pub use self::geometry::{ collect_definitions, collect_geometry, collect_layers, collect_material_ids, collect_model_attribute_dictionaries, extract_attribute_dictionaries, }`
- `pub use self::pages_dimensions::{ find_page_node, parse_camera, parse_dimensions, parse_pages, scan_instance_transforms, scan_vertex_positions, }`
- `pub use self::transforms::{fold_scale, multiply_matrices, transform_determinant, transform_normal, transform_point}`
- `pub use self::vff::{extract_version, is_legacy, read_meta_units, valid_header}`
- `pub use crate::legacy::{ LegacyEntityListResult, LegacyKind, LegacyObject, LegacyPathResult, LegacySlotEntry, LegacySlotTable, LegacyValue, V20FillerHit, find_count_after_v20_filler, find_empty_root_after_v20_filler, legacy_connection_paths, legacy_instance_has_guid, parse_legacy, read_legacy_attribute_value, read_legacy_entity_list, scan_pages_for_layers, }`
- `pub use crate::observability::{emit_log, emit_progress}`

**Items:** [`ErrorKind`](#cc_skp-internal-errorkind) · [`Error`](#cc_skp-internal-error) · [`Result`](#cc_skp-internal-result) · [`Tag`](#cc_skp-internal-tag) · [`TlvNode`](#cc_skp-internal-tlvnode) · [`RawFace`](#cc_skp-internal-rawface) · [`RawInstance`](#cc_skp-internal-rawinstance) · [`RawArcFrame`](#cc_skp-internal-rawarcframe) · [`GeometryBuilder`](#cc_skp-internal-geometrybuilder) · [`RawDefinition`](#cc_skp-internal-rawdefinition) · [`RawTexture`](#cc_skp-internal-rawtexture) · [`RawMaterial`](#cc_skp-internal-rawmaterial) · [`RawStyle`](#cc_skp-internal-rawstyle) · [`RawPage`](#cc_skp-internal-rawpage) · [`RawDimension`](#cc_skp-internal-rawdimension) · [`RawCamera`](#cc_skp-internal-rawcamera) · [`RawParsed`](#cc_skp-internal-rawparsed) · [`text`](#cc_skp-internal-text) · [`hex_upper`](#cc_skp-internal-hex_upper)

<a id="cc_skp-internal-errorkind"></a>

### enum `ErrorKind`

```rust
pub enum ErrorKind
```

Which standard exception the C++ code would have thrown.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`

**Variants**

- `OutOfRange`

  `std::out_of_range`

- `Overflow`

  `std::overflow_error`

- `InvalidArgument`

  `std::invalid_argument`

- `Runtime`

  `std::runtime_error` (and anything else)


<a id="cc_skp-internal-error"></a>

### struct `Error`

```rust
pub struct Error
```

An internal failure: a bounds-checked read past the end, a varint wider
than 64 bits, or a reader giving up (`std::runtime_error`). Readers return
it; the parse entry points wrap it into an `SkpParseError` cause.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Eq`, `fmt::Display`, `std::error::Error`

**Fields**

- `kind: ErrorKind`
- `message: Cow<'static, str>`

**Methods**

- `pub fn new(kind: ErrorKind, message: impl Into<Cow<'static, str>>) -> Self`
- `pub fn out_of_range(message: impl Into<Cow<'static, str>>) -> Self`
- `pub fn overflow(message: impl Into<Cow<'static, str>>) -> Self`
- `pub fn invalid_argument(message: impl Into<Cow<'static, str>>) -> Self`
- `pub fn runtime(message: impl Into<Cow<'static, str>>) -> Self`

<a id="cc_skp-internal-result"></a>

### type alias `Result`

```rust
pub type Result<T, E = Error> = std::result::Result<T, E>
```

Result of an internal reader step.

<a id="cc_skp-internal-tag"></a>

### struct `Tag`

```rust
pub struct Tag([u8; 4])
```

A TLV tag as its 4 upper-case hex digits in byte order (bytes `F4 01` are
`"F401"`), stored inline. Compares directly against `&str`.

**Traits:** `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Default`, `From<&str>`, `PartialEq<str>`, `PartialEq<&str>`, `fmt::Display`, `fmt::Debug`

**Methods**

- `pub const fn from_bytes(b0: u8, b1: u8) -> Self`

  The tag of the two raw bytes `b0 b1`.

- `pub fn as_str(&self) -> &str`

  The tag text (`""` for a default tag).


<a id="cc_skp-internal-tlvnode"></a>

### struct `TlvNode`

```rust
pub struct TlvNode
```

One TLV record: container tags carry `children`, everything else (and a
container whose payload does not parse) carries `payload`.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Eq`, `Default`

**Fields**

- `offset: u64`
- `size: u64`
- `tag: Tag`
- `children: Vec<TlvNode>`
- `payload: Vec<u8>`

**Methods**

- `pub fn leaf(tag: &str, payload: Vec<u8>) -> Self`

  A leaf with a payload (test / builder convenience).

- `pub fn container(tag: &str, children: Vec<TlvNode>) -> Self`

  A container with children (test / builder convenience).


<a id="cc_skp-internal-rawface"></a>

### struct `RawFace`

```rust
pub struct RawFace
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `loops: Vec<Vec<CoEdge>>`
- `normal: Vec3`
- `material_id: Option<EntityId>`
- `back_material_id: Option<EntityId>`
- `uv_transform: Option<[f64; 9]>`
- `uv_transform_back: Option<[f64; 9]>`
- `uv_projected: bool`
- `uv_projected_back: bool`
- `hidden: bool`
- `layer: String`

  Layer entity id as text until build_model resolves it to a name, same
  as `RawInstance::layer`.

- `layer_id: Option<EntityId>`

  The raw layer id (Python port's `Face.layer`); `None` when the record
  carries none.


<a id="cc_skp-internal-rawinstance"></a>

### struct `RawInstance`

```rust
pub struct RawInstance
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `offset: u64`
- `id: Option<EntityId>`
- `ref_guid: String`
- `name: String`
- `ref_idx: Option<EntityId>`
- `material_id: Option<EntityId>`
- `matrix: Vec<f64>`
- `layer: String`
- `properties: BTreeMap<String, String>`

  The "dynamic_attributes" dictionary only, stringified.

- `attribute_dicts: ParsedAttrDictionaries`

  Every named dictionary, values typed.

- `hidden: bool`

<a id="cc_skp-internal-rawarcframe"></a>

### struct `RawArcFrame`

```rust
pub struct RawArcFrame
```

The analytic frame of a Curve that is one arc.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`

**Fields**

- `center: Vec3`
- `normal: Vec3`
- `x_axis: Vec3`
- `y_axis: Vec3`
- `start_angle: f64`
- `end_angle: f64`

<a id="cc_skp-internal-geometrybuilder"></a>

### struct `GeometryBuilder`

```rust
pub struct GeometryBuilder
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `vertices: BTreeMap<EntityId, Vec3>`
- `edges: BTreeMap<EntityId, (Option<EntityId>, Option<EntityId>)>`
- `edge_flags: BTreeMap<EntityId, i32>`
- `edge_curves: BTreeMap<EntityId, EntityId>`

  Edge id -> entity id of the Curve the edge belongs to (Edge#curve).

- `edge_layers: BTreeMap<EntityId, EntityId>`

  Edge id -> raw layer id (VFF `D007 > D207`; Python's `edge_layers`).

- `arc_curves: BTreeMap<EntityId, RawArcFrame>`

  Curve id -> its analytic arc, for the curves that are arcs.

- `faces: BTreeMap<EntityId, RawFace>`
- `instances: Vec<RawInstance>`
- `section_planes: Vec<SectionPlane>`
- `texts: Vec<TextEntity>`
- `dimensions: Vec<Dimension>`
- `construction_lines: Vec<ConstructionLine>`
- `construction_points: Vec<ConstructionPoint>`

<a id="cc_skp-internal-rawdefinition"></a>

### struct `RawDefinition`

```rust
pub struct RawDefinition
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `guid: String`
- `name: String`
- `always_faces_camera: bool`
- `shadows_face_sun: bool`
- `is_image: bool`
- `is_group: bool`
- `builder: GeometryBuilder`

**Methods**

- `pub fn root() -> Self`

  The implicit root definition (`{"ROOT", "ROOT_MODEL"}`).


<a id="cc_skp-internal-rawtexture"></a>

### struct `RawTexture`

```rust
pub struct RawTexture
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `filename: String`
- `x_scale: f64`
- `y_scale: f64`
- `data: Option<Vec<u8>>`

<a id="cc_skp-internal-rawmaterial"></a>

### struct `RawMaterial`

```rust
pub struct RawMaterial
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `name: String`
- `r: i32`
- `g: i32`
- `b: i32`
- `a: i32`
- `transparency: f64`
- `colorized: bool`
- `colorize_type: i32`
- `texture: Option<RawTexture>`

<a id="cc_skp-internal-rawstyle"></a>

### struct `RawStyle`

```rust
pub struct RawStyle
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Eq`, `Default`

**Fields**

- `name: String`
- `description: String`
- `front_color: Option<Color3>`
- `back_color: Option<Color3>`
- `items: BTreeMap<i32, StyleItem>`
- `watermarks: Vec<StyleWatermark>`
- `folder: String`
- `active: bool`
- `working_copy: bool`
- `modified: bool`

<a id="cc_skp-internal-rawpage"></a>

### struct `RawPage`

```rust
pub struct RawPage
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `name: String`
- `eye: Option<Vec3>`
- `target: Option<Vec3>`
- `up: Option<Vec3>`
- `fov: f64`
- `parallel: bool`
- `ortho_height: f64`
- `hidden_layer_ids: Vec<EntityId>`
- `selected: bool`

<a id="cc_skp-internal-rawdimension"></a>

### struct `RawDimension`

```rust
pub struct RawDimension
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `a: Vec3`
- `b: Vec3`
- `offset: f64`
- `plane_x: Option<Vec3>`
- `normal: Option<Vec3>`
- `offset_vector: Option<Vec3>`
- `text: String`

<a id="cc_skp-internal-rawcamera"></a>

### struct `RawCamera`

```rust
pub struct RawCamera
```

A view camera record (34BC), as scenes and the model's current view store
it.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`

**Fields**

- `eye: Option<Vec3>`
- `target: Option<Vec3>`
- `up: Option<Vec3>`
- `fov: f64`
- `parallel: bool`
- `ortho_height: f64`

<a id="cc_skp-internal-rawparsed"></a>

### struct `RawParsed`

```rust
pub struct RawParsed
```

Everything a reader (VFF or legacy) extracts, before build_model /
build_scene_raw / build_instanced_scene_raw shape it.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `version: String`
- `units: Option<String>`

  The model's unit-system string from meta/meta.dat (VFF only).

- `layer_colors: BTreeMap<String, Color3>`
- `layer_order: Vec<String>`

  Layer names in the order first encountered in the file -
  `layer_colors` is sorted by name, so this is the only place file order
  survives.

- `layer_hidden: BTreeMap<String, bool>`

  Layer visibility: VFF from the layer manager's 8E3C byte, legacy from
  the CLayer record.

- `layer_attribute_dictionaries: BTreeMap<String, StringDictionaries>`
- `layer_id_to_name: BTreeMap<EntityId, String>`
- `pages: Vec<RawPage>`
- `camera: Option<ViewCamera>`
- `dimensions: Vec<RawDimension>`
- `material_id_to_name: BTreeMap<EntityId, String>`
- `materials: BTreeMap<String, Arc<RawMaterial>>`

  Materials by name. The same `Arc` may also sit in
  `materials_by_folder`; identity (`Arc::ptr_eq`) matters to build_model.

- `materials_by_folder: BTreeMap<String, Arc<RawMaterial>>`
- `styles: Vec<RawStyle>`
- `definitions: BTreeMap<EntityId, RawDefinition>`
- `root: RawDefinition`
- `model_attribute_dicts: ParsedAttrDictionaries`

  Dictionaries attached to the model entity itself (F601 > 8813 > D007 >
  DC05).


<a id="cc_skp-internal-text"></a>

### fn `text`

```rust
pub fn text(p: &[u8]) -> String
```

Lossy UTF-8 text of raw bytes (the C++ keeps them as a byte string).

<a id="cc_skp-internal-hex_upper"></a>

### fn `hex_upper`

```rust
pub fn hex_upper(p: &[u8]) -> String
```

Upper-case hex of raw bytes.

<a id="mod-cc_skp-internal-build_model"></a>

## `cc_skp::internal::build_model`

<sub>`crates/cc_skp/src/internal/build_model.rs`</sub>

Shapes a `RawParsed` into the public `SkpModel` (`model.cpp`'s
`build_model`).

**Items:** [`build_model`](#cc_skp-internal-build_model-build_model)

<a id="cc_skp-internal-build_model-build_model"></a>

### fn `build_model`

```rust
pub fn build_model(p: RawParsed, o: &ParseOptions) -> SkpModel
```

Builds the public model from a reader's raw output.

<a id="mod-cc_skp-internal-bytes"></a>

## `cc_skp::internal::bytes`

<sub>`crates/cc_skp/src/internal/bytes.rs`</sub>

Bounds-checked little-endian readers, var-ints and the TLV walkers
(`tlv.cpp`).

**Items:** [`read_u16`](#cc_skp-internal-bytes-read_u16) · [`read_u32`](#cc_skp-internal-bytes-read_u32) · [`read_i32`](#cc_skp-internal-bytes-read_i32) · [`read_f64`](#cc_skp-internal-bytes-read_f64) · [`parse_varint`](#cc_skp-internal-bytes-parse_varint) · [`tag_at`](#cc_skp-internal-bytes-tag_at) · [`u32_at`](#cc_skp-internal-bytes-u32_at) · [`MAX_TLV_DEPTH`](#cc_skp-internal-bytes-max_tlv_depth) · [`parse_tlv_recursive`](#cc_skp-internal-bytes-parse_tlv_recursive) · [`parse_flat`](#cc_skp-internal-bytes-parse_flat)

<a id="cc_skp-internal-bytes-read_u16"></a>

### fn `read_u16`

```rust
pub fn read_u16(d: &[u8], o: usize) -> Result<u16>
```

<a id="cc_skp-internal-bytes-read_u32"></a>

### fn `read_u32`

```rust
pub fn read_u32(d: &[u8], o: usize) -> Result<u32>
```

<a id="cc_skp-internal-bytes-read_i32"></a>

### fn `read_i32`

```rust
pub fn read_i32(d: &[u8], o: usize) -> Result<i32>
```

<a id="cc_skp-internal-bytes-read_f64"></a>

### fn `read_f64`

```rust
pub fn read_f64(d: &[u8], o: usize) -> Result<f64>
```

<a id="cc_skp-internal-bytes-parse_varint"></a>

### fn `parse_varint`

```rust
pub fn parse_varint(d: &[u8], o: usize, n: usize) -> Result<u64>
```

Little-endian unsigned integer of `n` bytes (at most 8).

<a id="cc_skp-internal-bytes-tag_at"></a>

### fn `tag_at`

```rust
pub fn tag_at(d: &[u8], o: usize) -> Result<Tag>
```

The tag of the two bytes at `o`, as upper-case hex in byte order.

<a id="cc_skp-internal-bytes-u32_at"></a>

### fn `u32_at`

```rust
pub(crate) fn u32_at(d: &[u8], o: usize) -> u32
```

Infallible little-endian u32 for offsets already range-checked.

<a id="cc_skp-internal-bytes-max_tlv_depth"></a>

### const `MAX_TLV_DEPTH`

```rust
pub const MAX_TLV_DEPTH: usize
```

Maximum allowed nesting depth for recursive container TLV nodes.

<a id="cc_skp-internal-bytes-parse_tlv_recursive"></a>

### fn `parse_tlv_recursive`

```rust
pub fn parse_tlv_recursive(d: &[u8], start: usize, end: usize) -> Result<Vec<TlvNode>>
```

Parses `d[start..end]` into TLV nodes, recursing into container tags.

<a id="cc_skp-internal-bytes-parse_flat"></a>

### fn `parse_flat`

```rust
pub fn parse_flat(d: &[u8]) -> Vec<(Tag, &[u8])>
```

One flat level of TLV records over the whole buffer, stopping at the first
record that would run past the end. Payloads borrow from `d`.

<a id="mod-cc_skp-internal-core"></a>

## `cc_skp::internal::core`

<sub>`crates/cc_skp/src/internal/core.rs`</sub>

The VFF (2021+) reader: ZIP container, material/style XML, and the
model.dat record walk (`core.cpp`).

**Items:** [`MAX_MODEL_ENTRY_BYTES`](#cc_skp-internal-core-max_model_entry_bytes) · [`MAX_OTHER_ENTRY_BYTES`](#cc_skp-internal-core-max_other_entry_bytes) · [`MAX_TOTAL_EXTRACTED_BYTES`](#cc_skp-internal-core-max_total_extracted_bytes) · [`decode_xml_entities`](#cc_skp-internal-core-decode_xml_entities) · [`decode_xml_entities_bytes`](#cc_skp-internal-core-decode_xml_entities_bytes) · [`PARSE_STACK_BYTES`](#cc_skp-internal-core-parse_stack_bytes) · [`full_parse`](#cc_skp-internal-core-full_parse)

<a id="cc_skp-internal-core-max_model_entry_bytes"></a>

### const `MAX_MODEL_ENTRY_BYTES`

```rust
pub const MAX_MODEL_ENTRY_BYTES: u64
```

Largest `model.dat`.

<a id="cc_skp-internal-core-max_other_entry_bytes"></a>

### const `MAX_OTHER_ENTRY_BYTES`

```rust
pub const MAX_OTHER_ENTRY_BYTES: u64
```

Largest other entry (a texture, a style, the metadata).

<a id="cc_skp-internal-core-max_total_extracted_bytes"></a>

### const `MAX_TOTAL_EXTRACTED_BYTES`

```rust
pub const MAX_TOTAL_EXTRACTED_BYTES: u64
```

Most bytes extracted from one file's entries in all.

<a id="cc_skp-internal-core-decode_xml_entities"></a>

### fn `decode_xml_entities`

```rust
pub fn decode_xml_entities(value: &str) -> String
```

Decodes the 5 predefined XML entities plus numeric character references
(`&#39;`, `&#x27;`) in a raw attribute value extracted without a real XML
parser. A real material name legitimately contains "<"/">" - SketchUp's own
"&lt;auto&gt;" default-material naming - so without this such names come
through still escaped. A single left-to-right pass, not five sequential
replacements: sequential replacement would double-decode "&amp;lt;" into
"<" instead of the correct "&lt;".

<a id="cc_skp-internal-core-decode_xml_entities_bytes"></a>

### fn `decode_xml_entities_bytes`

```rust
pub fn decode_xml_entities_bytes(v: &[u8]) -> Vec<u8>
```

`decode_xml_entities` on raw bytes. Numeric references are UTF-8 encoded
with the C++'s byte arithmetic (a 64-bit `unsigned long`); one that
overflows is left as-is.

<a id="cc_skp-internal-core-parse_stack_bytes"></a>

### const `PARSE_STACK_BYTES`

```rust
pub const PARSE_STACK_BYTES: usize
```

Stack of the thread a parse runs on. The readers recurse up to their depth limits
(legacy objects nest 256 deep), which a hostile file reaches: an optimized build needs
more than the 1 MiB of the Windows main thread there and a debug build more than 8 MiB.
The stack is reserved, not committed, so a large one costs only address space.

<a id="cc_skp-internal-core-full_parse"></a>

### fn `full_parse`

```rust
pub fn full_parse(data: &[u8], o: &ParseOptions) -> Result<RawParsed, SkpParseError>
```

Reads a whole .skp buffer: legacy (pre-2021) files go to the legacy reader,
VFF files are read here. The parse runs on its own thread with a
`PARSE_STACK_BYTES` stack, so the caller's stack size does not matter.

<a id="mod-cc_skp-internal-geometry"></a>

## `cc_skp::internal::geometry`

<sub>`crates/cc_skp/src/internal/geometry.rs`</sub>

VFF geometry, layer, material-id and definition collection
(`geometry.cpp`).

**Items:** [`find_node`](#cc_skp-internal-geometry-find_node) · [`entity_id`](#cc_skp-internal-geometry-entity_id) · [`extract_attribute_dictionaries`](#cc_skp-internal-geometry-extract_attribute_dictionaries) · [`collect_geometry`](#cc_skp-internal-geometry-collect_geometry) · [`collect_layers`](#cc_skp-internal-geometry-collect_layers) · [`collect_model_attribute_dictionaries`](#cc_skp-internal-geometry-collect_model_attribute_dictionaries) · [`collect_material_ids`](#cc_skp-internal-geometry-collect_material_ids) · [`collect_definitions`](#cc_skp-internal-geometry-collect_definitions)

<a id="cc_skp-internal-geometry-find_node"></a>

### fn `find_node`

```rust
pub(crate) fn find_node<'a>(ns: &'a [TlvNode], t: &str) -> Option<&'a TlvNode>
```

Depth-first search for the first node tagged `t`.

<a id="cc_skp-internal-geometry-entity_id"></a>

### fn `entity_id`

```rust
pub(crate) fn entity_id(n: &TlvNode) -> Result<Option<EntityId>>
```

The node's entity id: a direct DE05 child, a DC05 child wrapping a DE05,
or (failing both) the first one found deeper down.

<a id="cc_skp-internal-geometry-extract_attribute_dictionaries"></a>

### fn `extract_attribute_dictionaries`

```rust
pub fn extract_attribute_dictionaries(p: &[u8], out: &mut ParsedAttrDictionaries)
```

Every attribute dictionary in a DC05 payload, keyed by the dictionary's own
declared name (B436). Each dictionary is a B436 name node immediately
followed by a sibling entries container (typically B536) holding B636 key /
A438 value pairs; B436 can occur at any depth reachable through container
tags, so both walks recurse into every container-tagged node.

<a id="cc_skp-internal-geometry-collect_geometry"></a>

### fn `collect_geometry`

```rust
pub fn collect_geometry(es: &[TlvNode], b: &mut GeometryBuilder) -> Result<()>
```

Collects vertices, edges, curves, faces, instances and construction
geometry from a list of nodes into `b`, descending into any other node with
children.

<a id="cc_skp-internal-geometry-collect_layers"></a>

### fn `collect_layers`

```rust
pub fn collect_layers(ns: &[TlvNode], out: &mut BTreeMap<EntityId, String>, hidden: &mut BTreeMap<String, bool>) -> Result<()>
```

Layer ids -> names and visibility from every 993A layer-manager list.

<a id="cc_skp-internal-geometry-collect_model_attribute_dictionaries"></a>

### fn `collect_model_attribute_dictionaries`

```rust
pub fn collect_model_attribute_dictionaries(model: &TlvNode, out: &mut ParsedAttrDictionaries)
```

The model record (F601) carries the model entity's own attributes the same
way an instance does: an 8813 entity base holding a D007 container whose
DC05 payload lists the named dictionaries.

<a id="cc_skp-internal-geometry-collect_material_ids"></a>

### fn `collect_material_ids`

```rust
pub fn collect_material_ids(ns: &[TlvNode], out: &mut BTreeMap<EntityId, String>) -> Result<()>
```

Material ids -> names (C832 records: DC05 id, CC32 name).

<a id="cc_skp-internal-geometry-collect_definitions"></a>

### fn `collect_definitions`

```rust
pub fn collect_definitions(ns: &[TlvNode], out: &mut BTreeMap<EntityId, RawDefinition>) -> Result<()>
```

Component/group definitions (7C15 records) with their geometry.

<a id="mod-cc_skp-internal-pages_dimensions"></a>

## `cc_skp::internal::pages_dimensions`

<sub>`crates/cc_skp/src/internal/pages_dimensions.rs`</sub>

VFF (2021+) scenes ("pages") and linear dimensions (`pages_dimensions.cpp`,
itself a port of Python's `_core.py`).

**Items:** [`scan_vertex_positions`](#cc_skp-internal-pages_dimensions-scan_vertex_positions) · [`scan_instance_transforms`](#cc_skp-internal-pages_dimensions-scan_instance_transforms) · [`parse_dimensions`](#cc_skp-internal-pages_dimensions-parse_dimensions) · [`find_page_node`](#cc_skp-internal-pages_dimensions-find_page_node) · [`parse_camera`](#cc_skp-internal-pages_dimensions-parse_camera) · [`parse_pages`](#cc_skp-internal-pages_dimensions-parse_pages)

<a id="cc_skp-internal-pages_dimensions-scan_vertex_positions"></a>

### fn `scan_vertex_positions`

```rust
pub fn scan_vertex_positions(top: &TlvNode, id2pos: &mut BTreeMap<String, Vec3>)
```

Accumulates every vertex's persistent id (hex) -> (x, y, z) inches. A vertex
is a C409 record: DC05 holds its persistent id (the DE05 var-int payload),
C509 its 3xf64 position. Dimension connection points reference geometry by
this id. Called once per top-level record.

<a id="cc_skp-internal-pages_dimensions-scan_instance_transforms"></a>

### fn `scan_instance_transforms`

```rust
pub fn scan_instance_transforms(top: &TlvNode, world: &mut BTreeMap<String, Vec<f64>>)
```

Accumulates each instance's persistent id (hex) -> its WORLD transform (a
13-double matrix, or empty for the identity), composing parent x local at
every 6419. A dimension connects to geometry INSIDE a placed component; the
vertex position is definition-local, so it must be lifted to world by the
instance's transform.

<a id="cc_skp-internal-pages_dimensions-parse_dimensions"></a>

### fn `parse_dimensions`

```rust
pub fn parse_dimensions(model_dat: &[u8], id2pos: &BTreeMap<String, Vec3>, inst_world: &BTreeMap<String, Vec<f64>>) -> Vec<RawDimension>
```

Linear dimensions (SketchUp's Dimension tool).
A dimension entity is a 5BCC record (raw bytes `cc 5b`) holding:

* 5BCD / 5BCE - the two connection points. Each wraps a 5208 whose 5209 is
  the connection TYPE (1 = a free explicit point in 520A, already world
  space; 2 = connected to geometry: 520B > 53FC names the target, 53FD the
  vertex by persistent id, 53FE a length-prefixed persistent id of the
  INSTANCE holding it - lifted to world by that instance's transform).
* 5BCF - the dimension plane normal; 5BD0 - its X axis; 5BD1 - the
  constrained axis.
* 5BD2 - the offset distance (inches).
Endpoints come out in WORLD space (inches). A connection point that cannot
be resolved drops the whole dimension (fail-safe).

<a id="cc_skp-internal-pages_dimensions-find_page_node"></a>

### fn `find_page_node`

```rust
pub fn find_page_node(top: &TlvNode) -> Option<&TlvNode>
```

The 0702 scenes node inside `top`'s subtree (including `top` itself).

<a id="cc_skp-internal-pages_dimensions-parse_camera"></a>

### fn `parse_camera`

```rust
pub fn parse_camera(record: &[u8]) -> RawCamera
```

A view camera (34BC): 34BD eye, 34BE target, 34BF up (3xf64, inches), 34C4
field of view (degrees), 34C2 u8 = PERSPECTIVE flag (00 = parallel
projection - calibrated against the bundled scene thumbnails), 34C3 f64 =
visible height when parallel (inches). Scenes and the model's current view
(FA01) use the same record.

<a id="cc_skp-internal-pages_dimensions-parse_pages"></a>

### fn `parse_pages`

```rust
pub fn parse_pages(node: Option<&TlvNode>) -> Result<Vec<RawPage>>
```

Scenes ("pages"). The 0702 node's payload nests 6D60 > 6D61 > one 7148
record per page:

* 6F54 > 6F55 - page name (UTF-8)
* 714A > 34BC - camera (see `parse_camera`)
* 7150 - layers hidden in this page: (u8 length, var-int layer id) runs
Each page's 6F54 also carries its entity id (DC05 > DE05), and 6D62 next to
6D61 holds the id of the scene selected when the model was saved.

<a id="mod-cc_skp-internal-strconv"></a>

## `cc_skp::internal::strconv`

<sub>`crates/cc_skp/src/internal/strconv.rs`</sub>

C library string-to-number conversions with the C++ `std::stoi` /
`std::stoll` / `std::stod` semantics the reader relies on: leading
whitespace is skipped, the longest valid prefix is converted, trailing text
is ignored, and "no conversion" or "out of range" is `None` (where the C++
would throw).

**Items:** [`c_stoll`](#cc_skp-internal-strconv-c_stoll) · [`c_stoi`](#cc_skp-internal-strconv-c_stoi) · [`c_stoul_digits`](#cc_skp-internal-strconv-c_stoul_digits) · [`c_stod`](#cc_skp-internal-strconv-c_stod)

<a id="cc_skp-internal-strconv-c_stoll"></a>

### fn `c_stoll`

```rust
pub fn c_stoll(s: &str) -> Option<i64>
```

`std::stoll`.

<a id="cc_skp-internal-strconv-c_stoi"></a>

### fn `c_stoi`

```rust
pub fn c_stoi(s: &str) -> Option<i32>
```

`std::stoi`.

<a id="cc_skp-internal-strconv-c_stoul_digits"></a>

### fn `c_stoul_digits`

```rust
pub fn c_stoul_digits(digits: &[u8], base: u32) -> Option<u64>
```

`std::stoul` on a digit string in `base` (10 or 16), with a 64-bit
`unsigned long`.

<a id="cc_skp-internal-strconv-c_stod"></a>

### fn `c_stod`

```rust
pub fn c_stod(s: &str) -> Option<f64>
```

`std::stod`: decimal, hexadecimal (`0x1.8p3`), `inf`/`infinity` and `nan`
prefixes; overflow and underflow (a non-zero value that rounds to zero or
to a subnormal) are range errors.

<a id="mod-cc_skp-internal-transforms"></a>

## `cc_skp::internal::transforms`

<sub>`crates/cc_skp/src/internal/transforms.rs`</sub>

Instance transform helpers (`transforms.cpp`). Matrices are the 13-value
form: 0-8 the row-major 3x3, 9-11 the translation, 12 the scale term.

**Items:** [`transform_determinant`](#cc_skp-internal-transforms-transform_determinant) · [`fold_scale`](#cc_skp-internal-transforms-fold_scale) · [`transform_point`](#cc_skp-internal-transforms-transform_point) · [`transform_normal`](#cc_skp-internal-transforms-transform_normal) · [`multiply_matrices`](#cc_skp-internal-transforms-multiply_matrices)

<a id="cc_skp-internal-transforms-transform_determinant"></a>

### fn `transform_determinant`

```rust
pub fn transform_determinant(m: &[f64]) -> f64
```

<a id="cc_skp-internal-transforms-fold_scale"></a>

### fn `fold_scale`

```rust
pub fn fold_scale(m: &[f64]) -> Vec<f64>
```

Folds a 13-value matrix's scale term (index 12) into its rotation and
translation, so index 12 is 1.
SketchUp can keep a uniform scale in the transformation's homogeneous term:
the 3x3 stays as it is, the translation is in the scaled units, and the 16th
value `w` is the divisor. The 13-value matrix stores `1/w` at index 12, so a
point transforms as `s * (R p + t)` with `s = m[12]`; folding gives the
equivalent `(s R) p + s t` with `s = 1`, which every consumer that reads
only indices 0-11 handles correctly (openskp#444). A matrix whose scale is
already 1 is returned unchanged.

<a id="cc_skp-internal-transforms-transform_point"></a>

### fn `transform_point`

```rust
pub fn transform_point(matrix: &[f64], p: Vec3) -> Vec3
```

<a id="cc_skp-internal-transforms-transform_normal"></a>

### fn `transform_normal`

```rust
pub fn transform_normal(m: &[f64], n: Vec3) -> Vec3
```

Transforms a normal by the inverse transpose (the cofactor matrix, with the
determinant's sign so mirrors keep normals pointing outward).

<a id="cc_skp-internal-transforms-multiply_matrices"></a>

### fn `multiply_matrices`

```rust
pub fn multiply_matrices(a: &[f64], b: &[f64]) -> Vec<f64>
```

`a x b` (apply `b`, then `a`), with both scale terms folded in first so the
product's own scale term is 1.

<a id="mod-cc_skp-internal-vff"></a>

## `cc_skp::internal::vff`

<sub>`crates/cc_skp/src/internal/vff.rs`</sub>

Header, version and meta.dat helpers (`vff.cpp`).

**Items:** [`valid_header`](#cc_skp-internal-vff-valid_header) · [`extract_version`](#cc_skp-internal-vff-extract_version) · [`read_meta_units`](#cc_skp-internal-vff-read_meta_units) · [`is_legacy`](#cc_skp-internal-vff-is_legacy)

<a id="cc_skp-internal-vff-valid_header"></a>

### fn `valid_header`

```rust
pub fn valid_header(d: &[u8]) -> bool
```

The SKP magic: `FF FE FF 0E`.

<a id="cc_skp-internal-vff-extract_version"></a>

### fn `extract_version`

```rust
pub fn extract_version(d: &[u8]) -> String
```

The `{major.minor.build}` version string from the UTF-16 header, or
`"unknown"`.

<a id="cc_skp-internal-vff-read_meta_units"></a>

### fn `read_meta_units`

```rust
pub fn read_meta_units(meta_bytes: &[u8]) -> Option<String>
```

The model's unit-system string from a VFF file's meta/meta.dat contents, or
`None` if the expected tags aren't found.

<a id="cc_skp-internal-vff-is_legacy"></a>

### fn `is_legacy`

```rust
pub fn is_legacy(d: &[u8]) -> bool
```

True for a classic (pre-2021, MFC) file: major version <= 20.

<a id="mod-cc_skp-internal-xml"></a>

## `cc_skp::internal::xml`

<sub>`crates/cc_skp/src/internal/xml.rs`</sub>

Hand-written equivalents of the `std::regex` patterns core.cpp uses to pull
attributes out of material.xml / style.xml. Each function documents the
pattern it reproduces and follows ECMAScript leftmost-first backtracking
semantics (`\s` = space, \t, \n, \v, \f, \r; `\w` = [A-Za-z0-9_]).
Everything works on bytes, like the C++ `std::string`s.

**Items:** [`OpenTag`](#cc_skp-internal-xml-opentag) · [`find_open_tag`](#cc_skp-internal-xml-find_open_tag) · [`Element`](#cc_skp-internal-xml-element) · [`find_element`](#cc_skp-internal-xml-find_element) · [`find_screenimage`](#cc_skp-internal-xml-find_screenimage) · [`attr`](#cc_skp-internal-xml-attr) · [`attributes`](#cc_skp-internal-xml-attributes)

<a id="cc_skp-internal-xml-opentag"></a>

### struct `OpenTag`

```rust
pub struct OpenTag
```

A match of an opening-tag pattern.

**Fields**

- `start: usize`
- `attrs: Range<usize>`
- `end: usize`

<a id="cc_skp-internal-xml-find_open_tag"></a>

### fn `find_open_tag`

```rust
pub fn find_open_tag(s: &[u8], from: usize, name: &str, require_gt: bool) -> Option<OpenTag>
```

`<(?:P:)?NAME\b([^>]*)>` (icase) searched from `from`. With
`require_gt = false` the pattern is `<(?:P:)?NAME\b([^>]*)` (no closing
`>`), as material_xml's image lookup uses.

<a id="cc_skp-internal-xml-element"></a>

### struct `Element`

```rust
pub struct Element
```

A match of an element pattern.

**Fields**

- `attrs: Range<usize>`
- `body: Option<Range<usize>>`
- `end: usize`

<a id="cc_skp-internal-xml-find_element"></a>

### fn `find_element`

```rust
pub fn find_element(s: &[u8], from: usize, name: &str) -> Option<Element>
```

`<(?:P:)?NAME\b([^>]*)>([\s\S]*?)</(?:P:)?NAME>` (icase) from `from`.

<a id="cc_skp-internal-xml-find_screenimage"></a>

### fn `find_screenimage`

```rust
pub fn find_screenimage(s: &[u8], from: usize) -> Option<Element>
```

`<(?:P:)?screenimage\b([^>]*?)(?:/>|>([\s\S]*?)</(?:P:)?screenimage>)`
(icase) from `from`: a self-closing tag or one with a body.

<a id="cc_skp-internal-xml-attr"></a>

### fn `attr`

```rust
pub fn attr(s: &[u8], key: &str) -> String
```

`(?:^|\s)KEY\s*=\s*["']([^"']*)["']` (icase), entity-decoded; `""` when
absent (core.cpp's `attr`).

<a id="cc_skp-internal-xml-attributes"></a>

### fn `attributes`

```rust
pub fn attributes(s: &[u8]) -> BTreeMap<String, String>
```

Every `name="value"` / `name='value'` attribute
(`([A-Za-z_][\w.:-]*)\s*=\s*(?:"([^"]*)"|'([^']*)')`), entity-decoded;
later duplicates win.

<a id="mod-cc_skp-legacy"></a>

## `cc_skp::legacy`

<sub>`crates/cc_skp/src/legacy/mod.rs`</sub>

Legacy (pre-2021 MFC `CArchive`) reader: SketchUp 3 through 2020
(`legacy.cpp`).
A classic .skp is one MFC object graph: every object is written in full
the first time anything points at it and back-referenced by its store-map
slot afterwards. The walk starts at the material manager (or, with no
materials, at the layer list), recovers the archive's absolute slot base,
then reads materials, layers, component definitions and the root entity
list in file order. Scenes are read from the document tail afterwards.
Beyond the C++ reader this port also surfaces what the Python reader does
(`legacy.py`): each edge's own layer (`GeometryBuilder::edge_layers`),
each face's raw layer id (`RawFace::layer_id`) and attribute values with
their full SketchUp type (`Time`, `Length`, `Point3d`, `Vector3d`, nested
`Array`).

**Re-exports**

- `pub use self::pages::scan_pages_for_layers`

**Items:** [`legacy_instance_has_guid`](#cc_skp-legacy-legacy_instance_has_guid) · [`FIRST_V4`](#cc_skp-legacy-first_v4) · [`FIRST_DIB_OBJECT`](#cc_skp-legacy-first_dib_object) · [`FIRST_NAMED_INSTANCE`](#cc_skp-legacy-first_named_instance) · [`LegacyKind`](#cc_skp-legacy-legacykind) · [`LegacyDimension`](#cc_skp-legacy-legacydimension) · [`LegacyValue`](#cc_skp-legacy-legacyvalue) · [`LegacyObject`](#cc_skp-legacy-legacyobject) · [`LegacySlotEntry`](#cc_skp-legacy-legacyslotentry) · [`LegacySlotTable`](#cc_skp-legacy-legacyslottable) · [`LegacyPathResult`](#cc_skp-legacy-legacypathresult) · [`anchored_text_connection`](#cc_skp-legacy-anchored_text_connection) · [`LegacyEntityListResult`](#cc_skp-legacy-legacyentitylistresult) · [`read_legacy_entity_list`](#cc_skp-legacy-read_legacy_entity_list) · [`legacy_connection_paths`](#cc_skp-legacy-legacy_connection_paths) · [`MAX_V20_FILLER_PAD`](#cc_skp-legacy-max_v20_filler_pad) · [`V20FillerHit`](#cc_skp-legacy-v20fillerhit) · [`find_count_after_v20_filler`](#cc_skp-legacy-find_count_after_v20_filler) · [`find_empty_root_after_v20_filler`](#cc_skp-legacy-find_empty_root_after_v20_filler) · [`find_v20_filler_marker`](#cc_skp-legacy-find_v20_filler_marker) · [`read_legacy_attribute_value`](#cc_skp-legacy-read_legacy_attribute_value) · [`parse_legacy`](#cc_skp-legacy-parse_legacy)

<a id="cc_skp-legacy-legacy_instance_has_guid"></a>

### fn `legacy_instance_has_guid`

```rust
pub fn legacy_instance_has_guid(ver: i32) -> bool
```

<a id="cc_skp-legacy-first_v4"></a>

### const `FIRST_V4`

```rust
pub(crate) const FIRST_V4: i32
```

<a id="cc_skp-legacy-first_dib_object"></a>

### const `FIRST_DIB_OBJECT`

```rust
pub(crate) const FIRST_DIB_OBJECT: i32
```

<a id="cc_skp-legacy-first_named_instance"></a>

### const `FIRST_NAMED_INSTANCE`

```rust
pub(crate) const FIRST_NAMED_INSTANCE: i32
```

<a id="cc_skp-legacy-legacykind"></a>

### enum `LegacyKind`

```rust
pub enum LegacyKind
```

Which entity a decoded `LegacyValue` is (C++ `V::k`).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`

**Variants**

- `None`

  An object this port reads only to stay aligned (camera, thumbnail,
  font).

- `Vertex`
- `Edge`
- `Curve`
- `ArcCurve`
- `EdgeUse`
- `Loop`
- `Face`
- `Attrs`
- `Dict`
- `Layer`
- `Material`
- `Dib`
- `Ftc`
- `Image`
- `Relationship`
- `ConstructionLine`
- `ConstructionPoint`
- `SectionPlane`
- `Dimension`
- `Text`
- `Definition`
- `Instance`

<a id="cc_skp-legacy-legacydimension"></a>

### struct `LegacyDimension`

```rust
pub struct LegacyDimension
```

One decoded entity of the MFC object graph (C++ `V`): material, layer,
component, edge, face, ... - `k` says which, and only the fields that kind
uses are filled.
A legacy dimension's decoded geometry, and the raw refs of its connection
points, resolved to vertex positions once the whole archive is read.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `geometry: Dimension`
- `connect1: Option<u64>`
- `connect2: Option<u64>`

<a id="cc_skp-legacy-legacyvalue"></a>

### struct `LegacyValue`

```rust
pub struct LegacyValue
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `k: LegacyKind`
- `name: String`
- `label: String`
- `text: String`
- `dimension: Option<Box<LegacyDimension>>`

  Dimension only (boxed, so mesh vertices do not carry it).

- `guid: String`
- `xyz: Vec3`
- `direction: Vec3`

  Construction line only: its normalized direction, and its bounded
  segment's start/end (unset when unbounded in that direction).

- `start: Option<Vec3>`
- `end: Option<Vec3>`
- `plane: Vec<f64>`
- `xf: Vec<f64>`
- `uvf: Vec<f64>`
- `uvb: Vec<f64>`
- `front_projected: bool`
- `back_projected: bool`
- `v1: u64`
- `v2: u64`
- `curve: Option<u64>`

  Edge only: the slot of its parent Curve, and the curve itself on the
  edge that first points at it (a back-ref to a curve already read
  carries none).

- `curve_value: Option<Rc<LegacyValue>>`
- `arc: RawArcFrame`

  Arc curve only.

- `edge: u64`
- `def: u64`
- `attrs: Option<u64>`

  Slot of this entity's CAttributeContainer, or `None` when it has none
  (slot 0 is a legitimate slot, so it cannot double as "absent").

- `entries: ParsedAttrDict`

  Dictionary (CAttributeNamed) only: its own typed key/value pairs.

- `tex_dib: u64`
- `inline_tex: bool`

  SketchUp 3-5: the texture image sits inline (bytes in `blob`), no
  store slot.

- `sense: bool`
- `faces_camera: bool`
- `shadows_face_sun: bool`
- `is_group: bool`
- `colorized: bool`
- `colorize_type: i32`
- `mat: i32`
- `back_mat: i32`
- `layer: i32`
- `hidden: i32`
- `soft: i32`
- `smooth: i32`
- `r: i32`
- `g: i32`
- `b: i32`
- `a: i32`
- `opacity: f64`
- `tw: f64`
- `th: f64`
- `tex_file: String`
- `blob: Vec<u8>`
- `loops: Vec<Rc<LegacyValue>>`
- `uses: Vec<Rc<LegacyValue>>`
- `ents: Vec<LegacyObject>`

<a id="cc_skp-legacy-legacyobject"></a>

### struct `LegacyObject`

```rust
pub struct LegacyObject
```

One object read off the archive (C++ `tuple<slot, class name, V>`): a null
reference has an empty `name`; a back-ref into the unwalked pre-model
region is named `"premodel"`, one into a burned connection-point band
`"reserved"`.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `slot: u64`
- `name: Rc<str>`
- `v: Option<Rc<LegacyValue>>`

**Methods**

- `pub(crate) fn null() -> Self`

<a id="cc_skp-legacy-legacyslotentry"></a>

### struct `LegacySlotEntry`

```rust
pub struct LegacySlotEntry
```

One legacy MFC archive slot-table entry: either a class declaration
(`cls`, `name`/`schema` identify the class, `v` unused) or a read object
(`v` holds its decoded value, possibly `None` for object kinds this port
doesn't decode). Public so `scan_pages_for_layers` is unit-testable
without the full archive type.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `cls: bool`
- `name: Rc<str>`
- `schema: i32`
- `v: Option<Rc<LegacyValue>>`

<a id="cc_skp-legacy-legacyslottable"></a>

### struct `LegacySlotTable`

```rust
pub struct LegacySlotTable<'a>
```

A minimal view into an archive's slot-table bookkeeping - the only pieces
the page (scene) scanner needs. The real reader wraps its own archive's
tables; a test can hand-build just the entries a case needs.

**Fields**

- `slots: &'a mut HashMap<u64, LegacySlotEntry>`
- `class_slot: &'a mut HashMap<String, u64>`
- `next: &'a mut u64`

**Methods**

- `pub fn alloc(&mut self, e: LegacySlotEntry) -> u64`

<a id="cc_skp-legacy-legacypathresult"></a>

### struct `LegacyPathResult`

```rust
pub struct LegacyPathResult
```

What follows a legacy linear dimension's connection ref (openskp#384 /
#412): `[entity ref][u32 n1][n1 refs][u32 n2][n2 refs]` - the instance
paths of the anchored entity.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Eq`, `Default`

**Fields**

- `extra: Option<u64>`
- `first: Vec<Option<u64>>`
- `second: Vec<Option<u64>>`
- `first_new: Vec<bool>`

  Whether each entry of `first` / `second` was an object written in full.

- `second_new: Vec<bool>`
- `next: usize`

<a id="cc_skp-legacy-anchored_text_connection"></a>

### fn `anchored_text_connection`

```rust
pub(crate) fn anchored_text_connection(d: &[u8], pos: usize) -> bool
```

Whether the text whose font ends at `pos` is anchored to an entity: after
the two screen-fraction f64s comes the connection block, `[u32 type][u32 4]
[point3d]`, and a type other than 0 or 1 (2, 4, 5 and 7 seen; see
docs/dimension-record-notes.md) anchors the label. Matches Python's
`_anchored_connection`.

<a id="cc_skp-legacy-legacyentitylistresult"></a>

### struct `LegacyEntityListResult`

```rust
pub struct LegacyEntityListResult
```

A legacy definition's entity list, read by `read_legacy_entity_list`:
each item's slot and class name, and the offset just past the list.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Eq`, `Default`

**Fields**

- `items: Vec<(u64, String)>`
- `next: usize`

<a id="cc_skp-legacy-read_legacy_entity_list"></a>

### fn `read_legacy_entity_list`

```rust
pub fn read_legacy_entity_list(data: &[u8], pos: usize, ver: i32, count: u32, table: LegacySlotTable) -> Result<LegacyEntityListResult>
```

Reads a legacy definition's entity list of `count` items at `pos`, with an
archive of SketchUp version `ver` over `slots`. Exposed, like
`scan_pages_for_layers`, so the list's bookkeeping is unit-testable: an
instance a text label's connection path writes in full is one of the
list's items whether or not the list names it again.

<a id="cc_skp-legacy-legacy_connection_paths"></a>

### fn `legacy_connection_paths`

```rust
pub fn legacy_connection_paths<F>(data: &[u8], pos: usize, mut read_new_object: F) -> Result<LegacyPathResult> where F: FnMut(usize) -> Result<(u64, usize)>
```

Reads a dimension connection's instance paths from `pos`.
`read_new_object(at)` is called for a path entry MFC wrote in full (the
first time anything pointed at it) and returns `(slot, position just past
the object)`.
A dimension on loose geometry has a null ref and two empty lists (ten zero
bytes, which the old fixed 42/82-byte blocks skipped); one anchored to a
vertex inside nested components carries a ref per component, and a
fixed-size read slid off the record by exactly those refs - silently
truncating the ROOT entity list after it. Matches Python's
`_connection_paths` (openskp#384; found again on a real SketchUp 2017
cabinet, #412).

<a id="cc_skp-legacy-max_v20_filler_pad"></a>

### const `MAX_V20_FILLER_PAD`

```rust
pub(crate) const MAX_V20_FILLER_PAD: usize
```

<a id="cc_skp-legacy-v20fillerhit"></a>

### struct `V20FillerHit`

```rust
pub struct V20FillerHit
```

Result of `find_count_after_v20_filler`: the recovered count and the
offset just past it.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Fields**

- `count: u32`
- `next: usize`

<a id="cc_skp-legacy-find_count_after_v20_filler"></a>

### fn `find_count_after_v20_filler`

```rust
pub fn find_count_after_v20_filler(d: &[u8], count_pos: usize, limit: u32) -> Option<V20FillerHit>
```

Locates the count that follows a v20 filler record, given the offset the
bad count was read from. Pure byte logic, exposed for tests.
SketchUp 2020 (v20) writes an extra, undocumented record ahead of some
counts that v17 does not have, which leaves the reader a few bytes early
and makes it read garbage as the count. The filler is an empty UTF-16
string record followed by zero padding:
```text
  <ff fe ff> <u8 0>        empty string
  <zero padding>           runs up to the real count
```rust
Rather than hard-code an offset (the number of bytes before the marker
differs per call site), locate the marker in the short window ahead, then
take the first plausible u32 that follows the padding. Only the EMPTY-
string form counts as filler: a real string here would mean genuine data,
and moving the cursor past it would corrupt the parse.
Returns `None` when the bytes do not match the filler layout.

<a id="cc_skp-legacy-find_empty_root_after_v20_filler"></a>

### fn `find_empty_root_after_v20_filler`

```rust
pub fn find_empty_root_after_v20_filler(d: &[u8], count_pos: usize) -> Option<usize>
```

The SketchUp 2020 layout of a model with NO component definitions AND no
root entities. `find_count_after_v20_filler` cannot see it: behind the
filler (an empty string record, then 13 bytes of padding) the only count is
the ROOT entity count, and it is 0, which that search skips as "no count".
Returns the offset of that root count (so the caller can put the cursor
there and read the zero), or `None` when the bytes are not that layout: the
empty string record has to sit within 12 bytes of `count_pos`, and the 13
padding bytes and the count behind it have to be all zero.

<a id="cc_skp-legacy-find_v20_filler_marker"></a>

### fn `find_v20_filler_marker`

```rust
pub(crate) fn find_v20_filler_marker(d: &[u8], count_pos: usize) -> Option<usize>
```

The empty-string filler marker in the 12 bytes from `count_pos`, or `None`
(also when the string there is not empty: that is real data).

<a id="cc_skp-legacy-read_legacy_attribute_value"></a>

### fn `read_legacy_attribute_value`

```rust
pub fn read_legacy_attribute_value(data: &[u8], pos: usize, type_tag: u8) -> Result<(ParsedAttribute, usize)>
```

Decodes one typed `CAttributeNamed` value of type `type_tag` starting at
`pos`; returns the value and the offset just past it. Exposed for tests.

<a id="cc_skp-legacy-parse_legacy"></a>

### fn `parse_legacy`

```rust
pub fn parse_legacy(data: &[u8], o: &ParseOptions) -> Result<RawParsed, SkpParseError>
```

Reads a classic (pre-2021) MFC-archive .skp buffer.

<a id="mod-cc_skp-legacy-archive"></a>

## `cc_skp::legacy::archive`

<sub>`crates/cc_skp/src/legacy/archive.rs`</sub>

The MFC object-graph walk (`legacy.cpp`'s `struct Archive`, the per-class
readers and `walk_model`).

**Items:** [`Value`](#cc_skp-legacy-archive-value) · [`is_class_ref`](#cc_skp-legacy-archive-is_class_ref) · [`Archive`](#cc_skp-legacy-archive-archive) · [`WalkResult`](#cc_skp-legacy-archive-walkresult) · [`bootstrap_two_materials`](#cc_skp-legacy-archive-bootstrap_two_materials) · [`probe_layer_anchor_bases`](#cc_skp-legacy-archive-probe_layer_anchor_bases) · [`walk_with_bases`](#cc_skp-legacy-archive-walk_with_bases)

<a id="cc_skp-legacy-archive-value"></a>

### type alias `Value`

```rust
pub(crate) type Value = Rc<LegacyValue>
```

A decoded value (always present for an object read in full).

<a id="cc_skp-legacy-archive-is_class_ref"></a>

### fn `is_class_ref`

```rust
pub(crate) fn is_class_ref(d: &[u8], p: usize, slot: u64) -> bool
```

<a id="cc_skp-legacy-archive-archive"></a>

### struct `Archive`

```rust
pub(crate) struct Archive<'a>
```

**Fields**

- `r: R<'a>`
- `ver: i32`
- `next: u64`
- `base: u64`
- `slots: HashMap<u64, LegacySlotEntry>`
- `class_slot: HashMap<String, u64>`

*Some fields are private.*

**Methods**

- `pub fn new(d: &'a [u8], ver: i32) -> Self`
- `pub fn object(&mut self, expect: Option<&str>) -> Result<LegacyObject>`
- `pub fn collect_trailing_layers(&mut self, mut slots_out: Option<&mut Vec<u64>>, mut layers_out: Option<&mut Vec<(u64, Value)>>, unmatched_class_ref: bool) -> Result<()>`
- `pub fn entity_list(&mut self, n: u32, root: bool) -> Result<Vec<LegacyObject>>`

<a id="cc_skp-legacy-archive-walkresult"></a>

### struct `WalkResult`

```rust
pub(crate) struct WalkResult<'a>
```

The finished walk: the archive (its slot table holds every object), the
root entity list, and the layers and materials in file order.

**Fields**

- `ar: Archive<'a>`
- `root: Vec<LegacyObject>`
- `layers: Vec<(u64, Value)>`
- `materials: Vec<(u64, Option<Value>)>`

<a id="cc_skp-legacy-archive-bootstrap_two_materials"></a>

### fn `bootstrap_two_materials`

```rust
pub(crate) fn bootstrap_two_materials(data: &[u8], ver: i32, mat_hdr: usize) -> Result<u64>
```

<a id="cc_skp-legacy-archive-probe_layer_anchor_bases"></a>

### fn `probe_layer_anchor_bases`

```rust
pub(crate) fn probe_layer_anchor_bases(data: &[u8], ver: i32, start: usize, mat_count: u32) -> Result<Vec<u64>>
```

<a id="cc_skp-legacy-archive-walk_with_bases"></a>

### fn `walk_with_bases`

```rust
pub(crate) fn walk_with_bases<'a>(data: &'a [u8], ver: i32, start: usize, mat_count: u32, bases: &[u64]) -> Result<WalkResult<'a>>
```

<a id="mod-cc_skp-legacy-pages"></a>

## `cc_skp::legacy::pages`

<sub>`crates/cc_skp/src/legacy/pages.rs`</sub>

Scenes (CViewPage records) in a classic file's document tail
(`legacy.cpp`'s `scan_pages_for_layers` and `read_page_list`).

**Items:** [`scan_pages_for_layers`](#cc_skp-legacy-pages-scan_pages_for_layers) · [`read_page_list`](#cc_skp-legacy-pages-read_page_list)

<a id="cc_skp-legacy-pages-scan_pages_for_layers"></a>

### fn `scan_pages_for_layers`

```rust
pub fn scan_pages_for_layers(data: &[u8], start_pos: usize, mut slots: LegacySlotTable) -> Result<Vec<RawPage>>
```

Best-effort scan for CViewPage (scene) records in a legacy (pre-2021 MFC)
file: name + hidden-layer slot ids only, for the narrow capture-flag
combinations this reader understands. `start_pos` is the byte offset to
begin scanning from (where the main entity walk stopped, in a real parse).
Runs as an independent scan over the file tail rather than a sequential
archive read: each candidate is validated by its own local structure (a
name string immediately followed by an empty second string and a
recognized flags word), and any candidate that doesn't fully validate is
silently skipped, so a page this reader can't fully understand is just
absent instead of wrong. Mirrors Python's `legacy._scan_pages`.
Errors only where the C++ throws out of the scan (a flags word cut off by
the end of the buffer, an unreadable header version).

<a id="cc_skp-legacy-pages-read_page_list"></a>

### fn `read_page_list`

```rust
pub(crate) fn read_page_list(data: &[u8], ar: &Archive) -> Option<Vec<RawPage>>
```

Every scene, with its camera and hidden layers, read as a sequence from
the scene list in the file tail. `None` when there is a scene list this
reader can't fully read; an empty list when there is none.

<a id="mod-cc_skp-legacy-reader"></a>

## `cc_skp::legacy::reader`

<sub>`crates/cc_skp/src/legacy/reader.rs`</sub>

The bounds-checked cursor the legacy archive reads through (`legacy.cpp`'s
`struct R`).

**Items:** [`R`](#cc_skp-legacy-reader-r) · [`decode_utf16le`](#cc_skp-legacy-reader-decode_utf16le) · [`header_major`](#cc_skp-legacy-reader-header_major)

<a id="cc_skp-legacy-reader-r"></a>

### struct `R`

```rust
pub(crate) struct R<'a>
```

A read cursor over the whole file: every read is bounds-checked and fails
with `out_of_range` ("legacy archive truncated") past the end.

**Traits:** `Debug`, `Clone`, `Copy`

**Fields**

- `d: &'a [u8]`
- `p: usize`

**Methods**

- `pub fn new(d: &'a [u8], p: usize) -> Self`
- `pub fn need(&self, n: usize) -> Result<()>`
- `pub fn u8(&mut self) -> Result<u8>`
- `pub fn u16(&mut self) -> Result<u16>`
- `pub fn u32(&mut self) -> Result<u32>`
- `pub fn f64(&mut self) -> Result<f64>`
- `pub fn f64s(&mut self, n: usize) -> Result<Vec<f64>>`
- `pub fn vec3(&mut self) -> Result<[f64; 3]>`

  Three doubles.

- `pub fn raw(&mut self, n: usize) -> Result<&'a [u8]>`
- `pub fn skip(&mut self, n: usize) -> Result<()>`
- `pub fn marker(&self) -> bool`

  True when the string-record marker `FF FE FF` starts at the cursor.

- `pub fn peek_u16(&self) -> Result<u16>`
- `pub fn utf16(&mut self) -> Result<String>`

  One MFC `CString` record: `FF FE FF`, a u8 length (escaped to u16 /
  u32), then that many UTF-16LE code units.

- `pub fn attr_value(&mut self, t: u8) -> Result<ParsedAttribute>`

  One typed `CAttributeNamed` value (`Archive::typed`). Every SketchUp
  value type keeps its own kind here (the Python reader's
  `read_typed`): `Time` (9), `Length` (12), `Point3d` (17), `Vector3d`
  (18), nested `Array` (11).


<a id="cc_skp-legacy-reader-decode_utf16le"></a>

### fn `decode_utf16le`

```rust
pub(crate) fn decode_utf16le(units: &[u8]) -> String
```

UTF-16LE code units to UTF-8. The C++ converts unit by unit (no surrogate
pairing); a Rust `String` must be valid UTF-8, so surrogate pairs are
combined and a lone surrogate becomes U+FFFD. BMP text is identical.

<a id="cc_skp-legacy-reader-header_major"></a>

### fn `header_major`

```rust
pub(crate) fn header_major(data: &[u8]) -> Option<std::result::Result<i32, ()>>
```

The major version from the `{N.` in the header's first 96 bytes (NULs
dropped), as the C++ `std::regex("\\{(\\d+)\\.")` + `std::stoi` read it:
`None` when there is no such text, `Some(Err)` when the digits overflow.

<a id="mod-cc_skp-model"></a>

## `cc_skp::model`

<sub>`crates/cc_skp/src/model.rs`</sub>

The parsed SketchUp model (`model.hpp` / `model.cpp`).

**Items:** [`EntityId`](#cc_skp-model-entityid) · [`ByteBuffer`](#cc_skp-model-bytebuffer) · [`Vec3`](#cc_skp-model-vec3) · [`Color3`](#cc_skp-model-color3) · [`Color4`](#cc_skp-model-color4) · [`METRES_TO_INCHES`](#cc_skp-model-metres_to_inches) · [`AttributeKind`](#cc_skp-model-attributekind) · [`ParsedAttribute`](#cc_skp-model-parsedattribute) · [`format_double`](#cc_skp-model-format_double) · [`ParsedAttrDict`](#cc_skp-model-parsedattrdict) · [`ParsedAttrDictionaries`](#cc_skp-model-parsedattrdictionaries) · [`StringDictionaries`](#cc_skp-model-stringdictionaries) · [`stringify_attr_dict`](#cc_skp-model-stringify_attr_dict) · [`stringify_attr_dictionaries`](#cc_skp-model-stringify_attr_dictionaries) · [`Vertex`](#cc_skp-model-vertex) · [`Edge`](#cc_skp-model-edge) · [`ArcCurve`](#cc_skp-model-arccurve) · [`CoEdge`](#cc_skp-model-coedge) · [`Face`](#cc_skp-model-face) · [`Layer`](#cc_skp-model-layer) · [`Texture`](#cc_skp-model-texture) · [`Material`](#cc_skp-model-material) · [`StyleItem`](#cc_skp-model-styleitem) · [`StyleWatermark`](#cc_skp-model-stylewatermark) · [`Style`](#cc_skp-model-style) · [`Instance`](#cc_skp-model-instance) · [`SectionPlane`](#cc_skp-model-sectionplane) · [`TextEntity`](#cc_skp-model-textentity) · [`Dimension`](#cc_skp-model-dimension) · [`constrained_offset_vector`](#cc_skp-model-constrained_offset_vector) · [`ConstructionLine`](#cc_skp-model-constructionline) · [`ConstructionPoint`](#cc_skp-model-constructionpoint) · [`Page`](#cc_skp-model-page) · [`ViewCamera`](#cc_skp-model-viewcamera) · [`LooseEdgeRun`](#cc_skp-model-looseedgerun) · [`Definition`](#cc_skp-model-definition) · [`loose_edge_runs`](#cc_skp-model-loose_edge_runs) · [`SkpModel`](#cc_skp-model-skpmodel)

<a id="cc_skp-model-entityid"></a>

### type alias `EntityId`

```rust
pub type EntityId = i64
```

Unique entity id from the file.

<a id="cc_skp-model-bytebuffer"></a>

### type alias `ByteBuffer`

```rust
pub type ByteBuffer = Vec<u8>
```

Raw bytes.

<a id="cc_skp-model-vec3"></a>

### type alias `Vec3`

```rust
pub type Vec3 = [f64; 3]
```

`(x, y, z)`.

<a id="cc_skp-model-color3"></a>

### type alias `Color3`

```rust
pub type Color3 = [u8; 3]
```

RGB 0..255.

**Traits:** `MaterialColor`

<a id="cc_skp-model-color4"></a>

### type alias `Color4`

```rust
pub type Color4 = [u8; 4]
```

RGBA 0..255.

**Traits:** `MaterialColor`

<a id="cc_skp-model-metres_to_inches"></a>

### const `METRES_TO_INCHES`

```rust
pub const METRES_TO_INCHES: f64
```

1 metre = 39.37007874015748 inches (SketchUp native unit). Shared by every
exporter that takes a coordinate scale factor.

<a id="cc_skp-model-attributekind"></a>

### enum `AttributeKind`

```rust
pub enum AttributeKind
```

The C++ `ParsedAttribute::Kind` an attribute value maps to. The Rust enum
`ParsedAttribute` distinguishes a few more SketchUp types (Point3d vs
Vector3d, Time) that the C++ port folds into `Vec` / `Integer`.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`

**Variants**

- `Null`
- `String`
- `Integer`
- `Float`
- `Vec`
- `Array`
- `Boolean`
- `Length`

<a id="cc_skp-model-parsedattribute"></a>

### enum `ParsedAttribute`

```rust
pub enum ParsedAttribute
```

One attribute-dictionary value, covering every value type SketchUp stores
(legacy `CAttributeNamed` types 0/4/6/7/9/10/11/12/17/18 and the VFF `A438`
wrapper's AD38/A738/A938/AF38/AA38/B438/B538/AE38 tags):

* `Null` - an explicitly present key with no value (and any value tag this
  reader does not recognise, rather than a wrong guess).
* `Length` keeps its own kind (VFF `AF38`, legacy type 12) so it can be
  written back as a Length; it stringifies exactly like `Float`.
* `Boolean` (VFF `AA38`, legacy type 7) stringifies as `1` / `0`.
* `Time` (legacy type 9, a `time_t`) stringifies like `Integer`.
`Instance::properties` stays a `map<string,string>` view of a dictionary
named `dynamic_attributes`; scene / JSON / IFC stringify every named
dictionary with `ParsedAttribute::to_string`. The model keeps types.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`, `fmt::Display`, `PartialEq<str>`, `PartialEq<&str>`, `PartialEq<String>`, `From<&str>`, `From<String>`

**Variants**

- `Null`
- `String(String)`
- `Integer(i64)`
- `Float(f64)`
- `Length(f64)`

  SketchUp `Length` (inches).

- `Boolean(bool)`
- `Time(i64)`

  `time_t` seconds.

- `Point3d(Vec3)`
- `Vector3d(Vec3)`
- `Array(Vec<ParsedAttribute>)`

**Methods**

- `pub fn kind(&self) -> AttributeKind`

  The C++ kind this value maps to.

- `pub fn text(&self) -> &str`

  The string, or `""` (C++ `text`).

- `pub fn integer(&self) -> i64`

  Integer / Time / Boolean (1/0) value, or 0 (C++ `integer`).

- `pub fn number(&self) -> f64`

  Float / Length value, or 0 (C++ `number`).

- `pub fn vec(&self) -> Vec3`

  Point3d / Vector3d value, or zeros (C++ `vec`).

- `pub fn array(&self) -> &[ParsedAttribute]`

  Array items, or an empty slice (C++ `array`).

- `pub fn is_null(&self) -> bool`

<a id="cc_skp-model-format_double"></a>

### fn `format_double`

```rust
pub fn format_double(v: f64) -> String
```

Shortest round-trip text of a double, as C++ `std::to_chars(double)` writes
it: the shortest digits, in fixed or scientific notation, whichever is
shorter (fixed on a tie); scientific uses a signed, two-digit-minimum
exponent (`1e+20`, `1e-07`).

<a id="cc_skp-model-parsedattrdict"></a>

### type alias `ParsedAttrDict`

```rust
pub type ParsedAttrDict = BTreeMap<String, ParsedAttribute>
```

One named dictionary's entries.

<a id="cc_skp-model-parsedattrdictionaries"></a>

### type alias `ParsedAttrDictionaries`

```rust
pub type ParsedAttrDictionaries = BTreeMap<String, ParsedAttrDict>
```

Dictionaries keyed by their declared name.

<a id="cc_skp-model-stringdictionaries"></a>

### type alias `StringDictionaries`

```rust
pub type StringDictionaries = BTreeMap<String, BTreeMap<String, String>>
```

A stringified dictionary set (scene / JSON / IFC view).

<a id="cc_skp-model-stringify_attr_dict"></a>

### fn `stringify_attr_dict`

```rust
pub fn stringify_attr_dict(src: &ParsedAttrDict) -> BTreeMap<String, String>
```

Stringifies every value of one dictionary.

<a id="cc_skp-model-stringify_attr_dictionaries"></a>

### fn `stringify_attr_dictionaries`

```rust
pub fn stringify_attr_dictionaries(src: &ParsedAttrDictionaries) -> StringDictionaries
```

Stringifies every value of every dictionary.

<a id="cc_skp-model-vertex"></a>

### struct `Vertex`

```rust
pub struct Vertex
```

3D point in raw SketchUp space (Z-up, inches).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`

**Fields**

- `id: EntityId`
- `x: f64`
- `y: f64`
- `z: f64`

<a id="cc_skp-model-edge"></a>

### struct `Edge`

```rust
pub struct Edge
```

Straight line segment connecting two vertices.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `id: EntityId`
- `v1_id: EntityId`
- `v2_id: EntityId`
- `soft: bool`

  Soft edge flag (suppresses edge rendering across coplanar faces).

- `smooth: bool`

  Smooth edge flag (enables vertex normal interpolation).

- `hidden: bool`

  Hidden edge flag (explicit "Hide" bit on this edge).

- `curve_id: Option<EntityId>`

  SketchUp's own `Edge#curve` grouping: the id of the Curve this edge
  belongs to, or `None`. Edges sharing a curve_id were drawn as one
  continuous multi-segment line or arc; if the curve is an arc,
  `Definition::arcs` has it under the same id.

- `layer: String`

  This edge's own layer, resolved to a name (Python port's `Edge.layer`,
  VFF `D007 > D207`, the legacy drawbase's layer field); `""` when the
  file carries no explicit override (legacy: the default id 0), the raw id
  as text when it names no known layer. Only meaningful for a *loose*
  edge - a face-bounded edge's layer is cosmetic.

- `layer_id: Option<EntityId>`

  The raw layer id behind `Edge::layer`, or `None`.


<a id="cc_skp-model-arccurve"></a>

### struct `ArcCurve`

```rust
pub struct ArcCurve
```

The analytic arc behind a curve: SketchUp's own ArcCurve.
The arc is the set of points `center + cos(t) * x_axis + sin(t) * y_axis`
for t from `start_angle` to `end_angle`. For a circle `x_axis` and `y_axis`
are perpendicular and both of length `ArcCurve::radius`; an arc scaled
non-uniformly stores an affine image of a circle. Lengths are in inches, in
the definition's local space.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`

**Fields**

- `center: Vec3`
- `normal: Vec3`

  Unit normal of the arc's plane.

- `x_axis: Vec3`

  Vector from the centre to the arc's start (length = radius).

- `y_axis: Vec3`

  normal x x_axis.

- `start_angle: f64`

  Start parameter in radians.

- `end_angle: f64`

  End parameter in radians. A full circle has `end_angle == start_angle`
  in a classic (pre-2021) file and `start_angle + 2 pi` in a 2021+ file;
  `ArcCurve::is_full_circle` covers both.


**Methods**

- `pub fn radius(&self) -> f64`

  Length of `x_axis`.

- `pub fn is_full_circle(&self) -> bool`

<a id="cc_skp-model-coedge"></a>

### struct `CoEdge`

```rust
pub struct CoEdge
```

Oriented edge reference within a face loop.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Default`

**Fields**

- `edge_id: EntityId`
- `orientation: i64`

  +1 = same direction as the edge, -1 = reversed.


<a id="cc_skp-model-face"></a>

### struct `Face`

```rust
pub struct Face
```

Planar polygon bounded by outer and inner (hole) edge loops.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `id: EntityId`
- `loops: Vec<Vec<CoEdge>>`

  `loops[0]` = outer boundary, `loops[1..]` = holes.

- `normal: Option<Vec3>`
- `material_id: Option<EntityId>`

  Front-face material id (references `SkpModel::materials`).

- `back_material_id: Option<EntityId>`

  Back-face material id.

- `uv_transform: Option<[f64; 9]>`

  Front-face 3x3 row-major UV transform.

- `uv_transform_back: Option<[f64; 9]>`

  Back-face 3x3 row-major UV transform.

- `uv_projected: bool`

  The texture is PROJECTED (e.g. the Add Location terrain drape): its UVs
  run in the projection plane's frame, not the face frame.

- `uv_projected_back: bool`
- `hidden: bool`

  SketchUp's "Hide" on this face (not a layer visibility toggle).

- `layer: String`

  Name of the face's layer. `""` for faces on the default layer, which
  VFF files don't record per face, and for every legacy face.

- `layer_id: Option<EntityId>`

  The raw layer id behind `Face::layer` (Python port's `Face.layer`), or
  `None`. A legacy face always carries one (its drawbase layer field, `0`
  for the default layer), even though its `layer` name stays `""`.


<a id="cc_skp-model-layer"></a>

### struct `Layer`

```rust
pub struct Layer
```

Organization layer (tag).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `name: String`
- `color: Color3`
- `hidden: bool`

  Whether the layer's visibility is switched off.

- `attribute_dictionaries: StringDictionaries`

  Named `CAttributeNamed` dictionaries on this `CLayer` (legacy only).


<a id="cc_skp-model-texture"></a>

### struct `Texture`

```rust
pub struct Texture
```

Embedded texture image data and metadata.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `filename: String`

  Original image filename inside the SKP archive.

- `width: f64`

  Horizontal tile size (inches).

- `height: f64`

  Vertical tile size (inches).

- `data: Option<ByteBuffer>`

  Raw image file bytes (PNG/JPG/BMP).


**Methods**

- `pub fn save(&self, path: impl AsRef<Path>) -> io::Result<()>`

  Saves the image bytes to `path`.


<a id="cc_skp-model-material"></a>

### struct `Material`

```rust
pub struct Material
```

Surface material properties.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `name: String`
- `color: Color4`
- `transparency: f64`

  Opacity factor [0 = fully transparent, 1 = fully opaque].

- `id: Option<EntityId>`

  First TLV entity id that names this material.

- `texture: Option<Texture>`
- `colorized: bool`
- `colorize_type: i32`

<a id="cc_skp-model-styleitem"></a>

### struct `StyleItem`

```rust
pub struct StyleItem
```

One `<sty:item>` of a style.xml: the raw `<t:variant>` of a style setting.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Eq`, `Default`

**Fields**

- `r#type: i32`

  Variant type attribute: 1 bool, 3/4 int32, 5 ABGR color in older files,
  6 float, 7 double, 13 nested XML such as the watermark list.

- `value: String`

  Variant text, trimmed; nested XML is kept verbatim.


<a id="cc_skp-model-stylewatermark"></a>

### struct `StyleWatermark`

```rust
pub struct StyleWatermark
```

One watermark of a style (style.xml item 5001, a `<screenimage>`), in file
order. The "<MODEL SPACE>" separator entry is not included.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Eq`, `Default`

**Fields**

- `name: String`
- `attributes: BTreeMap<String, String>`

  Raw `<screenimage>` attributes, entity-decoded (`name` excluded).

- `image_path: String`

  Image path as the style names it.

- `file_name: String`

  Original image file name.

- `image: Option<ByteBuffer>`

  Raw image bytes, when the ZIP has the entry.


<a id="cc_skp-model-style"></a>

### struct `Style`

```rust
pub struct Style
```

Bundled rendering style settings.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Eq`, `Default`

**Fields**

- `name: String`
- `description: String`
- `front_color: Option<Color3>`

  Default front face color (item 2002).

- `back_color: Option<Color3>`

  Default back face color (item 2003).

- `items: BTreeMap<i32, StyleItem>`

  Every item of the style.xml, keyed by SketchUp's item id.

- `watermarks: Vec<StyleWatermark>`
- `folder: String`

  Folder of this style in the SKP ZIP ("styles/<folder>/style.xml").

- `active: bool`

  The model's current style.

- `working_copy: bool`

  Working copy of the current style (folder "<folder>_1").

- `modified: bool`

  On the current style: edited without updating the style.


<a id="cc_skp-model-instance"></a>

### struct `Instance`

```rust
pub struct Instance
```

Component or group placement within a definition.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `name: String`
- `ref_idx: Option<EntityId>`

  Id of the placed definition.

- `guid: String`

  GUID of the placed definition.

- `matrix: Vec<f64>`

  13-value transform: 0-8 the row-major 3x3, 9-11 translation, 12 the
  scale term (`1/w`); empty when the file stores none.

- `layer: String`

  This instance's own explicit layer override, or `""`.

- `properties: BTreeMap<String, String>`

  View of `attribute_dictionaries["dynamic_attributes"]`, stringified.

- `attribute_dictionaries: ParsedAttrDictionaries`

  Every attribute dictionary on this instance, values typed.

- `material_id: Option<EntityId>`

  Instance material override id.

- `hidden: bool`

  SketchUp's "Hide" on this placement.

- `id: Option<EntityId>`

  The placement's own persistent entity id (VFF only).


<a id="cc_skp-model-sectionplane"></a>

### struct `SectionPlane`

```rust
pub struct SectionPlane
```

Section plane entity.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `plane: [f64; 4]`
- `name: String`
- `label: String`
- `hidden: bool`

<a id="cc_skp-model-textentity"></a>

### struct `TextEntity`

```rust
pub struct TextEntity
```

Text annotation entity.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `text: String`
- `hidden: bool`
- `point: Option<Vec3>`

  Anchor point in inches (Python port extension), or `None` when the
  record anchors to geometry or the reader does not recover it.

- `label_point: Option<Vec3>`

  Where the label floats (Python port extension), or `None`.


<a id="cc_skp-model-dimension"></a>

### struct `Dimension`

```rust
pub struct Dimension
```

A linear dimension (SketchUp's Dimension tool).
The legacy reader recovers free endpoints and vertex attachments in the
owning entities; unresolved attachments remain unset. See
`SkpModel::dimensions` for the VFF model-level, world-space list.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `text: String`

  Displayed text; empty when it shows the auto-computed value.

- `hidden: bool`
- `a: Option<Vec3>`

  First measured point in inches in the owning entities' space
  (definition-local, or world at the model root), or unset when
  unresolved.

- `b: Option<Vec3>`

  Second measured point.

- `offset: f64`

  Offset distance (inches) of the dimension line from the a-b segment.

- `plane_x: Option<Vec3>`

  Historical frame field: VFF `5BCF` (plane normal), legacy X axis, or
  unset.

- `normal: Option<Vec3>`

  Historical frame field: VFF `5BD0` (X axis), legacy plane normal, or
  unset.

- `offset_vector: Option<Vec3>`

  Explicit offset for legacy or VFF dimensions constrained to a plane
  axis. Preserves the side of the line independently of endpoint order.


<a id="cc_skp-model-constrained_offset_vector"></a>

### fn `constrained_offset_vector`

```rust
pub(crate) fn constrained_offset_vector(axis: u32, normal: Vec3, x: Vec3, offset: f64) -> Option<Vec3>
```

The offset vector of a dimension constrained to a plane axis (`axis` 1 or
2), from its plane `normal`, X axis `x` and signed `offset`; `None` for
any other mode, which keeps only the scalar offset.

<a id="cc_skp-model-constructionline"></a>

### struct `ConstructionLine`

```rust
pub struct ConstructionLine
```

A construction/guide line: point + normalized direction; `start`/`end` are
unset when the line is unbounded in that direction.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `point: Vec3`
- `direction: Vec3`
- `start: Option<Vec3>`
- `end: Option<Vec3>`
- `layer: String`

  Name of the layer (tag) the guide is assigned to, or `""` for the
  default layer (Layer0).

- `hidden: bool`

  Whether the guide is hidden (SketchUp's "Hide" on this specific guide,
  not a layer/tag visibility toggle).


<a id="cc_skp-model-constructionpoint"></a>

### struct `ConstructionPoint`

```rust
pub struct ConstructionPoint
```

A construction/guide point.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `position: Vec3`
- `layer: String`

  Name of the layer (tag) the guide is assigned to, or `""` for the
  default layer (Layer0).

- `hidden: bool`

  Whether the guide is hidden (SketchUp's "Hide" on this specific guide,
  not a layer/tag visibility toggle).


<a id="cc_skp-model-page"></a>

### struct `Page`

```rust
pub struct Page
```

A saved scene ("page").

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `name: String`
- `eye: Option<Vec3>`
- `target: Option<Vec3>`
- `up: Option<Vec3>`
- `fov: f64`

  Field of view in degrees.

- `parallel: bool`

  Parallel (orthographic) projection; `fov` still holds the stored angle.

- `ortho_height: f64`

  Visible height in inches when `parallel`.

- `hidden_layers: Vec<String>`

  Names of the layers this scene hides.

- `selected: bool`

  The scene selected when the model was saved.


<a id="cc_skp-model-viewcamera"></a>

### struct `ViewCamera`

```rust
pub struct ViewCamera
```

A view camera (VFF `34BC` record).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`

**Fields**

- `eye: Vec3`
- `target: Vec3`
- `up: Vec3`
- `fov: f64`
- `parallel: bool`
- `ortho_height: f64`

<a id="cc_skp-model-looseedgerun"></a>

### struct `LooseEdgeRun`

```rust
pub struct LooseEdgeRun
```

One ordered run of loose edges (see `Definition::loose_edge_runs`).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Eq`, `Default`

**Fields**

- `edge_ids: Vec<EntityId>`
- `vertex_ids: Vec<EntityId>`

  Vertex ids in walk order; a closed run does not repeat its first vertex.

- `closed: bool`

<a id="cc_skp-model-definition"></a>

### struct `Definition`

```rust
pub struct Definition
```

Reusable geometry container (component definition or group).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `id: EntityId`
- `guid: String`
- `name: String`
- `vertices: BTreeMap<EntityId, Vertex>`
- `edges: BTreeMap<EntityId, Edge>`
- `faces: BTreeMap<EntityId, Face>`
- `instances: Vec<Instance>`
- `section_planes: Vec<SectionPlane>`
- `texts: Vec<TextEntity>`
- `dimensions: Vec<Dimension>`
- `construction_lines: Vec<ConstructionLine>`
- `construction_points: Vec<ConstructionPoint>`
- `always_faces_camera: bool`
- `shadows_face_sun: bool`
- `arcs: BTreeMap<EntityId, ArcCurve>`

  The analytic arc of every curve that is one, keyed by curve id.

- `is_image: bool`

  Backs an Image entity.

- `is_group: bool`

  SketchUp made this definition for a group rather than a component.


**Methods**

- `pub fn loose_edge_runs(&self) -> Vec<LooseEdgeRun>`

  Groups this definition's LOOSE edges (edges no face uses) into ordered
  polyline runs (Python's `loose_edge_runs`).
  Grouping comes from the file's own `Edge#curve` (`Edge::curve_id`),
  not a vertex-adjacency guess. An edge with no curve id is its own
  two-point run - joining it to neighbours by adjacency alone would
  invent a curve the author never drew. Each curve's edges are walked in
  order (a SketchUp Curve is a path or a single cycle); a branched group
  emits its longest walk and orders the remainder again.


<a id="cc_skp-model-loose_edge_runs"></a>

### fn `loose_edge_runs`

```rust
pub fn loose_edge_runs(definition: &Definition) -> Vec<LooseEdgeRun>
```

Free-function form of `Definition::loose_edge_runs` (Python API name).

<a id="cc_skp-model-skpmodel"></a>

### struct `SkpModel`

```rust
pub struct SkpModel
```

Top-level model: definitions, layers, materials, styles and scenes.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `version: String`

  SketchUp version string (e.g. "{25.0.575}").

- `units: Option<String>`

  Unit-system string from meta/meta.dat (VFF only).

- `definitions: BTreeMap<EntityId, Definition>`

  Component/group definitions keyed by entity id.

- `layers: Vec<Layer>`

  Layers in file order.

- `pages: Vec<Page>`

  Saved scenes.

- `camera: Option<ViewCamera>`

  The view the model was saved with (VFF only).

- `dimensions: Vec<Dimension>`

  Model-level linear dimensions with world-space endpoints (VFF).

- `materials: Vec<Material>`
- `styles: Vec<Style>`
- `attribute_dictionaries: ParsedAttrDictionaries`

  Dictionaries attached to the model itself (VFF only), values typed.

- `root: Definition`
- `material_indices: HashMap<EntityId, usize>`

**Methods**

- `pub fn root(&self) -> &Definition`

  The implicit root model definition.

- `pub fn root_mut(&mut self) -> &mut Definition`
- `pub fn material_by_id(&self, id: EntityId) -> Option<&Material>`

  Looks up a material by any TLV entity id that names it.

- `pub fn material_by_id_mut(&mut self, id: EntityId) -> Option<&mut Material>`
- `pub fn material_index(&self, id: EntityId) -> Option<usize>`

  Index into `SkpModel::materials` of the material an id names.

- `pub fn materials_by_id(&self) -> BTreeMap<EntityId, &Material>`

  Every material keyed by each TLV entity id that names it (several ids
  may alias one material).


<a id="mod-cc_skp-observability"></a>

## `cc_skp::observability`

<sub>`crates/cc_skp/src/observability.rs`</sub>

Parse progress and log callbacks (`observability.hpp`).

**Items:** [`LogLevel`](#cc_skp-observability-loglevel) · [`ParseStage`](#cc_skp-observability-parsestage) · [`stage_name`](#cc_skp-observability-stage_name) · [`ParseProgress`](#cc_skp-observability-parseprogress) · [`ProgressFn`](#cc_skp-observability-progressfn) · [`LogFn`](#cc_skp-observability-logfn) · [`ParseOptions`](#cc_skp-observability-parseoptions) · [`PROGRESS_INTERVAL`](#cc_skp-observability-progress_interval) · [`emit_log`](#cc_skp-observability-emit_log) · [`emit_progress`](#cc_skp-observability-emit_progress)

<a id="cc_skp-observability-loglevel"></a>

### enum `LogLevel`

```rust
pub enum LogLevel
```

Severity of a log message emitted through `ParseOptions::log`.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`

**Variants**

- `Debug`
- `Information`

<a id="cc_skp-observability-parsestage"></a>

### enum `ParseStage`

```rust
pub enum ParseStage
```

The stage of the parse pipeline a progress report or error refers to.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `fmt::Display`

**Variants**

- `Header`
- `ZipExtract`
- `TlvWalk`
- `LegacyWalk`
- `LegacyDefs`
- `BuildScene`

**Methods**

- `pub const fn name(self) -> &'static str`

  The stage's snake_case name, as it appears in error messages.


<a id="cc_skp-observability-stage_name"></a>

### fn `stage_name`

```rust
pub const fn stage_name(s: ParseStage) -> &'static str
```

The stage's snake_case name (`"header"`, `"zip_extract"`, ...).

<a id="cc_skp-observability-parseprogress"></a>

### struct `ParseProgress`

```rust
pub struct ParseProgress
```

One progress report: `current` of `total` units done in `stage`.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Fields**

- `stage: ParseStage`
- `current: u64`
- `total: u64`

<a id="cc_skp-observability-progressfn"></a>

### type alias `ProgressFn`

```rust
pub type ProgressFn = dyn Fn(&ParseProgress) + Send + Sync
```

Progress callback type.

<a id="cc_skp-observability-logfn"></a>

### type alias `LogFn`

```rust
pub type LogFn = dyn Fn(LogLevel, &str) + Send + Sync
```

Log callback type.

<a id="cc_skp-observability-parseoptions"></a>

### struct `ParseOptions`

```rust
pub struct ParseOptions
```

Optional progress and logging callbacks. Both are invoked synchronously on
the calling thread.

**Traits:** `Clone`, `Default`, `fmt::Debug`

**Fields**

- `progress: Option<Arc<ProgressFn>>`
- `log: Option<Arc<LogFn>>`

**Methods**

- `pub fn new() -> Self`

  Options with no callbacks.

- `pub fn with_progress(mut self, f: impl Fn(&ParseProgress) + Send + Sync + 'static) -> Self`

  Sets the progress callback.

- `pub fn with_log(mut self, f: impl Fn(LogLevel, &str) + Send + Sync + 'static) -> Self`

  Sets the log callback.


<a id="cc_skp-observability-progress_interval"></a>

### const `PROGRESS_INTERVAL`

```rust
pub const PROGRESS_INTERVAL: u64
```

Progress is reported every this many records (and on the last one).

<a id="cc_skp-observability-emit_log"></a>

### fn `emit_log`

```rust
pub fn emit_log(o: &ParseOptions, level: LogLevel, message: &str)
```

Invokes the log callback, if any (`emit_log`).

<a id="cc_skp-observability-emit_progress"></a>

### fn `emit_progress`

```rust
pub fn emit_progress(o: &ParseOptions, stage: ParseStage, current: u64, total: u64)
```

Invokes the progress callback, if any (`emit_progress`).

<a id="mod-cc_skp-parser"></a>

## `cc_skp::parser`

<sub>`crates/cc_skp/src/parser.rs`</sub>

Public entry points (`parser.hpp` / `parser.cpp`).

**Items:** [`parse_skp`](#cc_skp-parser-parse_skp) · [`build_scene`](#cc_skp-parser-build_scene) · [`build_instanced_scene`](#cc_skp-parser-build_instanced_scene) · [`SkpFile`](#cc_skp-parser-skpfile)

<a id="cc_skp-parser-parse_skp"></a>

### fn `parse_skp`

```rust
pub fn parse_skp(buffer: &[u8], options: &ParseOptions) -> Result<SkpModel, SkpParseError>
```

Parses an SKP buffer into its definitions, geometry, materials, layers,
and styles. Callbacks in `options` run synchronously on this thread.

<a id="cc_skp-parser-build_scene"></a>

### fn `build_scene`

```rust
pub fn build_scene(buffer: &[u8], options: &ParseOptions) -> Result<Scene, SkpParseError>
```

Parses an SKP buffer and bakes it into world-space, GLB-ready scene data.

<a id="cc_skp-parser-build_instanced_scene"></a>

### fn `build_instanced_scene`

```rust
pub fn build_instanced_scene(buffer: &[u8], options: &ParseOptions) -> Result<InstancedScene, SkpParseError>
```

Parses an SKP buffer and builds the placed scene graph with SketchUp's
component/group instancing PRESERVED instead of baked into world-space
vertex data: output grows with unique geometry + instance transforms
rather than definition geometry x placement count.

<a id="cc_skp-parser-skpfile"></a>

### struct `SkpFile`

```rust
pub struct SkpFile
```

In-memory handle for repeatedly parsing or baking one SKP source. Each call
starts from the original bytes and performs an independent parse.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Eq`

*Some fields are private.*

**Methods**

- `pub fn open(path: impl AsRef<Path>) -> io::Result<SkpFile>`

  Reads an SKP file into memory, checking the (case-insensitive) `.skp`
  extension; format validation is deferred to `parse()`.
  Errors: `NotFound` when the path does not exist, `InvalidInput` for
  another extension, and the underlying error when reading fails.

- `pub fn from_buffer(buffer: ByteBuffer) -> SkpFile`

  A handle owning bytes already in memory.

- `pub fn data(&self) -> &[u8]`

  The source bytes.

- `pub fn parse(&self, options: &ParseOptions) -> Result<SkpModel, SkpParseError>`

  Parses the stored source into a new model.

- `pub fn build_scene(&self, options: &ParseOptions) -> Result<Scene, SkpParseError>`

  Parses the stored source and bakes a new world-space scene.

- `pub fn build_instanced_scene(&self, options: &ParseOptions) -> Result<InstancedScene, SkpParseError>`

  Parses the stored source and builds a new instanced scene.


<a id="mod-cc_skp-scene"></a>

## `cc_skp::scene`

<sub>`crates/cc_skp/src/scene.rs`</sub>

The baked, world-space scene (`scene.hpp`). Types only; the builder lives
in `scene_build.rs`.

**Items:** [`InstanceNode`](#cc_skp-scene-instancenode) · [`MeshMetadata`](#cc_skp-scene-meshmetadata) · [`GlbPrimitive`](#cc_skp-scene-glbprimitive) · [`PbrMetallicRoughness`](#cc_skp-scene-pbrmetallicroughness) · [`GltfMaterial`](#cc_skp-scene-gltfmaterial) · [`SceneTexture`](#cc_skp-scene-scenetexture) · [`CurveArc`](#cc_skp-scene-curvearc) · [`CurveSetMetadata`](#cc_skp-scene-curvesetmetadata) · [`Scene`](#cc_skp-scene-scene)

<a id="cc_skp-scene-instancenode"></a>

### struct `InstanceNode`

```rust
pub struct InstanceNode
```

One node of the baked instance tree.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `name: String`
- `name_is_generated: bool`

  True when `name` is a fallback this project generated rather than a
  real name from the file (Python port extension; see
  `InstancedNode::name_is_generated`).

- `definition_name: String`
- `layer: String`
- `position_mm: [f64; 3]`
- `properties: BTreeMap<String, String>`
- `children: Vec<InstanceNode>`
- `attribute_dictionaries: StringDictionaries`

  Every attribute dictionary other than SketchUp's own
  `dynamic_attributes` (already on `properties`) and `SU_InstanceSet`
  (always-present Owner/Status boilerplate). Values are stringified.

- `path: String`

  The full-path string this node's meshes carry as `MeshMetadata::path`
  (Python port extension), or `""`.


<a id="cc_skp-scene-meshmetadata"></a>

### struct `MeshMetadata`

```rust
pub struct MeshMetadata
```

Metadata for one baked mesh, keyed like its primitive's `geom_name`.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `name: String`
- `definition_name: String`
- `layer: String`
- `position_mm: [f64; 3]`
- `properties: BTreeMap<String, String>`
- `path: String`
- `attribute_dictionaries: StringDictionaries`

  Same as `InstanceNode::attribute_dictionaries`, backfilled from the
  instance that placed this mesh's definition.


<a id="cc_skp-scene-glbprimitive"></a>

### struct `GlbPrimitive`

```rust
pub struct GlbPrimitive
```

One triangulated, world-space mesh (metres, glTF Y-up).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `positions: Vec<f32>`
- `normals: Vec<f32>`
- `uvs: Vec<f32>`

  Flat `[u, v, ...]` texture coordinates matching positions 1:1.

- `indices: Vec<u32>`
- `material_index: usize`
- `geom_name: String`

<a id="cc_skp-scene-pbrmetallicroughness"></a>

### struct `PbrMetallicRoughness`

```rust
pub struct PbrMetallicRoughness
```

glTF `pbrMetallicRoughness`.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `base_color_factor: [f64; 4]`
- `metallic_factor: f64`
- `roughness_factor: f64`
- `base_color_texture: Option<usize>`

  Index into `Scene::textures`, or `None` for an untextured material.
  `base_color_factor` stays the resolved color even with a texture set:
  glTF multiplies the two, and colorized materials rely on that tint.


<a id="cc_skp-scene-gltfmaterial"></a>

### struct `GltfMaterial`

```rust
pub struct GltfMaterial
```

One glTF material.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `name: String`
- `texture_path: String`
- `pbr_metallic_roughness: PbrMetallicRoughness`
- `double_sided: bool`

<a id="cc_skp-scene-scenetexture"></a>

### struct `SceneTexture`

```rust
pub struct SceneTexture
```

One texture image referenced by the scene's glTF materials.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Eq`, `Default`

**Fields**

- `data: ByteBuffer`

  The image file's raw bytes, exactly as stored in the .skp.

- `mime_type: String`

  Sniffed from the bytes, not from `filename`.

- `filename: String`

<a id="cc_skp-scene-curvearc"></a>

### struct `CurveArc`

```rust
pub struct CurveArc
```

An analytic arc recovered for a loose-edge run (Python port extension):
points in the run's own frame plus 0-based `(start, mid, end)` index
triples into them, ready to become IFC4 `IfcArcIndex` entries.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `points: Vec<[f64; 3]>`
- `segments: Vec<[usize; 3]>`

<a id="cc_skp-scene-curvesetmetadata"></a>

### struct `CurveSetMetadata`

```rust
pub struct CurveSetMetadata
```

Metadata for one run of *loose* edges - edges no face uses (Python port
extension; the C++ port does not emit these). Points are world-space
metres in glTF's Y-up frame, like `GlbPrimitive::positions`.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `name: String`
- `definition_name: String`
- `layer: String`
- `path: String`
- `name_is_generated: bool`

  `name` was synthesised rather than read from the file.

- `points_m: Vec<[f64; 3]>`
- `closed: bool`

  The run returns to its own first point.

- `arc: Option<CurveArc>`

  Set only when the run provably is one whole circular arc.

- `attribute_dictionaries: StringDictionaries`

<a id="cc_skp-scene-scene"></a>

### struct `Scene`

```rust
pub struct Scene
```

The result of `build_scene()`: the placed instances baked into a flat,
world-space scene.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `scene_hierarchy: InstanceNode`
- `mesh_index: BTreeMap<String, MeshMetadata>`
- `glb_primitives: Vec<GlbPrimitive>`
- `gltf_materials: Vec<GltfMaterial>`
- `textures: Vec<SceneTexture>`

  Distinct texture images the placed materials use, deduplicated by
  source bytes.

- `layer_hidden: BTreeMap<String, bool>`

  Each layer's own hidden/visible state, by layer name.

- `curve_sets: Vec<CurveSetMetadata>`

  Loose-edge runs, one per chained polyline (Python port extension; the
  meshes above are unaffected by them). Empty for a model whose geometry
  is all faces.


<a id="mod-cc_skp-scene_build"></a>

## `cc_skp::scene_build`

<sub>`crates/cc_skp/src/scene_build.rs`</sub>

The baked scene builder (`scene.cpp`'s `build_scene_raw`), plus the Python
port's loose-edge curve sets (`scene.py`'s `curve_sets`).

**Items:** [`IDENTITY`](#cc_skp-scene_build-identity) · [`is_generic_definition_name`](#cc_skp-scene_build-is_generic_definition_name) · [`omitted_from_scene`](#cc_skp-scene_build-omitted_from_scene) · [`name_override`](#cc_skp-scene_build-name_override) · [`scene_dictionaries`](#cc_skp-scene_build-scene_dictionaries) · [`instance_layer`](#cc_skp-scene_build-instance_layer) · [`definition_name`](#cc_skp-scene_build-definition_name) · [`curve_fallback_layer`](#cc_skp-scene_build-curve_fallback_layer) · [`run_layer`](#cc_skp-scene_build-run_layer) · [`build_error`](#cc_skp-scene_build-build_error) · [`MAX_COMPONENT_DEPTH`](#cc_skp-scene_build-max_component_depth) · [`MAX_EXPANDED_INSTANCES`](#cc_skp-scene_build-max_expanded_instances) · [`validate_hierarchy`](#cc_skp-scene_build-validate_hierarchy) · [`recursive_definition`](#cc_skp-scene_build-recursive_definition) · [`max_depth_exceeded`](#cc_skp-scene_build-max_depth_exceeded) · [`max_instances_exceeded`](#cc_skp-scene_build-max_instances_exceeded) · [`build_scene_raw`](#cc_skp-scene_build-build_scene_raw)

<a id="cc_skp-scene_build-identity"></a>

### const `IDENTITY`

```rust
pub(crate) const IDENTITY: [f64; 13]
```

The 13-value identity matrix.

<a id="cc_skp-scene_build-is_generic_definition_name"></a>

### fn `is_generic_definition_name`

```rust
pub(crate) fn is_generic_definition_name(name: &str) -> bool
```

SketchUp's own auto-generated placeholder definition names
(`^(?:Group|Component)\d*#\d+$`: "Group#1", "Component#12", "Group86#2"),
which carry no more meaning than the internal index.

<a id="cc_skp-scene_build-omitted_from_scene"></a>

### fn `omitted_from_scene`

```rust
pub(crate) fn omitted_from_scene(dict_name: &str) -> bool
```

Scene nodes omit SketchUp's own dictionaries: `dynamic_attributes` is
already on `properties`, `SU_InstanceSet` is Owner/Status boilerplate.

<a id="cc_skp-scene_build-name_override"></a>

### fn `name_override`

```rust
pub(crate) fn name_override(i: &RawInstance) -> Option<String>
```

The attribute-dictionary display-name override: "name", "label" or "code"
on any dictionary except the omitted ones; first dictionary and first key
with a non-empty value wins.

<a id="cc_skp-scene_build-scene_dictionaries"></a>

### fn `scene_dictionaries`

```rust
pub(crate) fn scene_dictionaries(i: &RawInstance) -> StringDictionaries
```

The instance's dictionaries minus the omitted ones, stringified.

<a id="cc_skp-scene_build-instance_layer"></a>

### fn `instance_layer`

```rust
pub(crate) fn instance_layer(p: &RawParsed, o: &ParseOptions, i: &RawInstance, parent_layer: &str) -> String
```

An instance's layer: its own id resolved to a name, the parent's layer
when the id names no layer, or the raw text when it is not a number.

<a id="cc_skp-scene_build-definition_name"></a>

### fn `definition_name`

```rust
pub(crate) fn definition_name(p: &RawParsed, i: &RawInstance) -> String
```

The definition's name, or `""` when the instance names none / an unknown
one.

<a id="cc_skp-scene_build-curve_fallback_layer"></a>

### fn `curve_fallback_layer`

```rust
pub(crate) fn curve_fallback_layer(p: &RawParsed, b: &GeometryBuilder, layer: &str) -> String
```

The fallback layer a definition's loose edges resolve to (Python port):
its own faces' majority layer when they carry one, else `layer`.

<a id="cc_skp-scene_build-run_layer"></a>

### fn `run_layer`

```rust
pub(crate) fn run_layer(p: &RawParsed, b: &GeometryBuilder, edge_ids: &[EntityId], fallback: &str) -> String
```

A run's own layer: its edges' majority layer, else `fallback`.

<a id="cc_skp-scene_build-build_error"></a>

### fn `build_error`

```rust
pub(crate) fn build_error(e: Error) -> SkpParseError
```

A missing vertex (the C++ `std::out_of_range` from `map::at`).

<a id="cc_skp-scene_build-max_component_depth"></a>

### const `MAX_COMPONENT_DEPTH`

```rust
pub(crate) const MAX_COMPONENT_DEPTH: usize
```

Maximum component nesting depth before rejecting the hierarchy.

<a id="cc_skp-scene_build-max_expanded_instances"></a>

### const `MAX_EXPANDED_INSTANCES`

```rust
pub(crate) const MAX_EXPANDED_INSTANCES: u64
```

Maximum total expanded component instances in a scene build (guards against
exponential DAG expansion / "billion laughs" component graphs).

<a id="cc_skp-scene_build-validate_hierarchy"></a>

### fn `validate_hierarchy`

```rust
pub(crate) fn validate_hierarchy(p: &RawParsed) -> Result<(), SkpParseError>
```

Validates component hierarchy depth, cycles, and expanded instance bounds
before initiating expensive scene allocation. Uses memoization so DAGs with
exponential expansions are detected in linear time O(V + E) without allocating.

<a id="cc_skp-scene_build-recursive_definition"></a>

### fn `recursive_definition`

```rust
pub(crate) fn recursive_definition(id: EntityId) -> SkpParseError
```

<a id="cc_skp-scene_build-max_depth_exceeded"></a>

### fn `max_depth_exceeded`

```rust
pub(crate) fn max_depth_exceeded(id: EntityId) -> SkpParseError
```

<a id="cc_skp-scene_build-max_instances_exceeded"></a>

### fn `max_instances_exceeded`

```rust
pub(crate) fn max_instances_exceeded(id: EntityId) -> SkpParseError
```

<a id="cc_skp-scene_build-build_scene_raw"></a>

### fn `build_scene_raw`

```rust
pub fn build_scene_raw(mut p: RawParsed, o: &ParseOptions) -> Result<Scene, SkpParseError>
```

Bakes a reader's raw output into a world-space `Scene`.

<a id="mod-cc_skp-triangulator"></a>

## `cc_skp::triangulator`

<sub>`crates/cc_skp/src/triangulator.rs`</sub>

Face normals and planar-face triangulation (`triangulator.hpp` /
`triangulator.cpp`).

**Items:** [`compute_face_normal`](#cc_skp-triangulator-compute_face_normal) · [`triangulate_face_3d`](#cc_skp-triangulator-triangulate_face_3d) · [`triangulate_with`](#cc_skp-triangulator-triangulate_with)

<a id="cc_skp-triangulator-compute_face_normal"></a>

### fn `compute_face_normal`

```rust
pub fn compute_face_normal(points: &[Vec3]) -> Option<Vec3>
```

Newell's-method unit normal of a polygon, or `None` for fewer than three
points or a degenerate (zero-area) polygon.

<a id="cc_skp-triangulator-triangulate_face_3d"></a>

### fn `triangulate_face_3d`

```rust
pub fn triangulate_face_3d(vertices: &BTreeMap<EntityId, Vertex>, loops: &[Vec<EntityId>], normal: Vec3) -> Vec<[EntityId; 3]>
```

Triangulates a planar face given as vertex-id loops (`loops[0]` the outer
boundary, the rest holes) in the plane of `normal`. Returns vertex-id
triples; empty when a general-path loop references a vertex missing from
`vertices`.

<a id="cc_skp-triangulator-triangulate_with"></a>

### fn `triangulate_with`

```rust
pub(crate) fn triangulate_with(lookup: impl Fn(EntityId) -> Option<Vec3>, loops: &[Vec<EntityId>], mut n: Vec3) -> Vec<[EntityId; 3]>
```

`triangulate_face_3d` over any vertex lookup (the scene builders read the
builder's own `id -> position` map instead of copying it into `Vertex`es).

<a id="mod-cc_skp-zip"></a>

## `cc_skp::zip`

<sub>`crates/cc_skp/src/zip.rs`</sub>

Minimal in-memory ZIP reader replacing the miniz `mz_zip_reader_*` calls
core.cpp makes: init from memory, file count/stat, locate by name, and
extract to memory (stored or deflated, CRC-checked).
The central-directory validation, the case-insensitive heap-sorted name
index used by `locate`, and the extraction checks mirror miniz so the same
archives are accepted and rejected.

**Items:** [`ZipError`](#cc_skp-zip-ziperror) · [`crc32_update`](#cc_skp-zip-crc32_update) · [`crc32`](#cc_skp-zip-crc32) · [`ZipFileStat`](#cc_skp-zip-zipfilestat) · [`ZipArchive`](#cc_skp-zip-ziparchive)

<a id="cc_skp-zip-ziperror"></a>

### enum `ZipError`

```rust
pub enum ZipError
```

Why an archive or entry was rejected (miniz's `mz_zip_error`).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `fmt::Display`, `std::error::Error`

**Variants**

- `NotAnArchive`
- `FailedFindingCentralDir`
- `InvalidHeaderOrCorrupted`
- `UnsupportedMultidisk`
- `TooManyFiles`
- `UnsupportedCdirSize`
- `UnsupportedEncryption`
- `UnsupportedMethod`
- `FileReadFailed`
- `InvalidParameter`
- `DecompressionFailed`
- `UnexpectedDecompressedSize`
- `CrcCheckFailed`

<a id="cc_skp-zip-crc32_update"></a>

### fn `crc32_update`

```rust
pub fn crc32_update(crc: u32, data: &[u8]) -> u32
```

Continues a CRC-32 (IEEE) from `crc` (0 to start).

<a id="cc_skp-zip-crc32"></a>

### fn `crc32`

```rust
pub fn crc32(data: &[u8]) -> u32
```

CRC-32 (IEEE) of `data`.

<a id="cc_skp-zip-zipfilestat"></a>

### struct `ZipFileStat`

```rust
pub struct ZipFileStat
```

One entry's central-directory metadata (`mz_zip_archive_file_stat`).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Eq`

**Fields**

- `index: u32`
- `filename: Vec<u8>`

  Name bytes as miniz exposes them: at most 511 bytes, cut at the first
  NUL (it is a C string).

- `comp_size: u64`
- `uncomp_size: u64`
- `crc32: u32`
- `method: u16`
- `bit_flag: u16`
- `local_header_ofs: u64`
- `external_attr: u32`
- `is_directory: bool`

**Methods**

- `pub fn filename_str(&self) -> String`

  The name as (lossy) UTF-8.


<a id="cc_skp-zip-ziparchive"></a>

### struct `ZipArchive`

```rust
pub struct ZipArchive<'a>
```

A ZIP archive held in memory (`mz_zip_reader_init_mem`).

*Some fields are private.*

**Methods**

- `pub fn new(data: &'a [u8]) -> Result<Self, ZipError>`

  Reads and validates the central directory.

- `pub fn len(&self) -> usize`

  Number of entries (`mz_zip_reader_get_num_files`).

- `pub fn is_empty(&self) -> bool`
- `pub fn stat(&self, index: u32) -> Result<ZipFileStat, ZipError>`

  Entry metadata (`mz_zip_reader_file_stat`), resolving zip64 extended
  sizes/offsets.

- `pub fn locate(&self, name: &[u8]) -> Option<u32>`

  Index of the entry named `name` (`mz_zip_reader_locate_file` with no
  flags: an ASCII case-insensitive binary search). The name is a C
  string, so it ends at its first NUL.

- `pub fn extract(&self, index: u32) -> Result<Vec<u8>, ZipError>`

  Extracts an entry into a buffer of exactly its declared uncompressed
  size (`mz_zip_reader_extract_to_mem` with a buffer that size).


{% endraw %}
