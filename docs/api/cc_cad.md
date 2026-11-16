# `cc_cad` API

{% raw %}
[← API reference](index.md) · Generated from the source by `cargo run -p api_doc`; edit the doc comments, not this page.

Reading DWG and DXF drawings into ConstructCraft's neutral
`cc_core::cad::CadDrawing` (an underlay placed in a 2D view).
DXF (ASCII and binary) is read by an in-house reader: group-code pairs,
the HEADER (units, version, code page), the LAYER table, BLOCKS and
ENTITIES; blocks are inserted (nested, arrayed, cycle-safe) and 2D
entities taken from their object coordinate systems to the plan. DWG is
read by the `acadrust` crate, whose document is written to binary DXF in
memory and read by the same reader, so both formats share one path.
Everything is bounded (file size, primitives, nesting, work) and errors
are values.

## Modules

| Module | Items | Summary |
|---|---|---|
| [`cc_cad`](#mod-cc_cad) | 10 | Reading DWG and DXF drawings into ConstructCraft's neutral `cc_core::cad::CadDrawing` (an underlay placed in a 2D view). |
| [`cc_cad::dwg`](#mod-cc_cad-dwg) | 2 | DWG files, read by the `acadrust` crate (MPL-2.0, pure Rust). |
| [`cc_cad::dxf`](#mod-cc_cad-dxf) | 3 | DXF files: sections, tables, blocks and entities, read into raw entities (their group pairs) that `crate::flatten` turns into primitives. |
| [`cc_cad::flatten`](#mod-cc_cad-flatten) | 2 | Turning raw entities into flat primitives: object coordinate systems, block insertion (nested, arrayed, cycle-safe and bounded), colors and layers by block, and curve tessellation where a primitive cannot stay an arc. |
| [`cc_cad::geom`](#mod-cc_cad-geom) | 9 | Coordinate systems and curve tessellation for flattening drawings. |
| [`cc_cad::svg`](#mod-cc_cad-svg) | 4 | SVG drawings into the neutral CAD drawing: paths (curves flattened to polylines) in their stroke color, single-outline filled shapes as fills, and embedded pictures as images. |
| [`cc_cad::text`](#mod-cc_cad-text) | 2 | Text strings of drawings: control codes and MTEXT formatting. |
| [`cc_cad::tokens`](#mod-cc_cad-tokens) | 4 | DXF group-code pairs, from ASCII or binary files. |

<a id="mod-cc_cad"></a>

## `cc_cad`

<sub>`crates/cc_cad/src/lib.rs`</sub>

**Items:** [`CadError`](#cc_cad-caderror) · [`MAX_INSERTS`](#cc_cad-max_inserts) · [`Limits`](#cc_cad-limits) · [`Format`](#cc_cad-format) · [`CadRead`](#cc_cad-cadread) · [`CadBlock`](#cc_cad-cadblock) · [`CadInsert`](#cc_cad-cadinsert) · [`detect`](#cc_cad-detect) · [`read`](#cc_cad-read) · [`read_file`](#cc_cad-read_file)

<a id="cc_cad-caderror"></a>

### enum `CadError`

```rust
pub enum CadError
```

Why a drawing could not be read.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Error`

**Variants**

- `TooLarge(usize)`
- `UnknownFormat`
- `Format(String)`
- `TooComplex(String)`
- `Dwg(String)`
- `Io(String)`

<a id="cc_cad-max_inserts"></a>

### const `MAX_INSERTS`

```rust
pub const MAX_INSERTS: usize
```

Most block references one read keeps (`Limits::keep_blocks`).

<a id="cc_cad-limits"></a>

### struct `Limits`

```rust
pub struct Limits
```

Bounds on what one read may do.

**Traits:** `Debug`, `Clone`, `Default`

**Fields**

- `max_dxf_bytes: usize`

  Largest DXF file, bytes.

- `max_dwg_bytes: usize`

  Largest DWG file, bytes.

- `max_prims: usize`

  Most primitives produced (blocks inserted).

- `max_work: usize`

  Most entity visits while inserting blocks (arrays of nested blocks).

- `max_points: usize`

  Most points and text characters handled in all: those of every
  entity read (each time a block inserts it) and of every primitive
  produced, so one large entity in an arrayed block cannot ask for
  more memory or time than this.

- `max_depth: usize`

  Deepest block nesting followed.

- `max_block_entities: usize`

  Most entities kept in block definitions.

- `keep_blocks: bool`

  Keep block inserts as references (`CadRead::inserts`) and the
  blocks they use as their own drawings (`CadRead::kept`) instead of
  flattening them into the drawing, wherever an insert places its
  block by a similarity of the plan (move, turn, uniform scale,
  mirror); other inserts, anonymous blocks and dimensions are still
  flattened. DXF and DWG only.


<a id="cc_cad-format"></a>

### enum `Format`

```rust
pub enum Format
```

File formats.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Default`

**Variants**

- `Dxf`
- `Dwg`
- `Svg`

<a id="cc_cad-cadread"></a>

### struct `CadRead`

```rust
pub struct CadRead
```

A drawing read, with what the reader saw.

**Traits:** `Debug`, `Clone`, `Default`

**Fields**

- `drawing: CadDrawing`
- `format: Format`
- `version: String`

  `$ACADVER` (e.g. `AC1032`).

- `binary: bool`

  A binary DXF file (or a DWG, read through binary DXF).

- `entities: usize`

  Model space entities read.

- `blocks: usize`

  Block definitions read.

- `skipped: BTreeMap<String, usize>`

  Entities not drawn, by type.

- `truncated: bool`

  A limit stopped the reader; the drawing is incomplete.

- `kept: Vec<CadBlock>`

  Blocks kept (with `Limits::keep_blocks`): those the drawing uses.

- `inserts: Vec<CadInsert>`

  Where the drawing inserts kept blocks.


**Methods**

- `pub fn skipped_total(&self) -> usize`

<a id="cc_cad-cadblock"></a>

### struct `CadBlock`

```rust
pub struct CadBlock
```

A block kept as its own drawing: primitives in its frame (drawing
units, its base point at the origin, on the drawing's layers) and the
kept blocks it inserts.

**Traits:** `Debug`, `Clone`, `Default`

**Fields**

- `name: String`

  The block's name, upper case (as DXF compares them).

- `prims: Vec<cc_core::cad::CadPrim>`
- `inserts: Vec<CadInsert>`

<a id="cc_cad-cadinsert"></a>

### struct `CadInsert`

```rust
pub struct CadInsert
```

An insert of a kept block: its frame scaled by `scale`, flipped across
its X axis first when `mirrored`, turned `rotation` radians and moved
to `position` (drawing units, in the frame of what holds it).

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `block: String`
- `position: glam::DVec2`
- `rotation: f64`
- `scale: f64`
- `mirrored: bool`
- `layer: u32`

  The insert's own layer and color.

- `color: Option<[u8; 3]>`

<a id="cc_cad-detect"></a>

### fn `detect`

```rust
pub fn detect(data: &[u8]) -> Option<Format>
```

The format of a file's bytes: DWG files start with their version
(`AC1015`…), binary DXF with its sentinel, ASCII DXF with group pairs.

<a id="cc_cad-read"></a>

### fn `read`

```rust
pub fn read(data: &[u8], limits: &Limits) -> Result<CadRead, CadError>
```

Read a drawing from its bytes (DXF, DWG or SVG).

<a id="cc_cad-read_file"></a>

### fn `read_file`

```rust
pub fn read_file(path: &std::path::Path, limits: &Limits) -> Result<CadRead, CadError>
```

Read a drawing file.

<a id="mod-cc_cad-dwg"></a>

## `cc_cad::dwg`

<sub>`crates/cc_cad/src/dwg.rs`</sub>

DWG files, read by the `acadrust` crate (MPL-2.0, pure Rust).
The decoded document is written to binary DXF in memory and read by our
DXF reader, so DWG and DXF drawings take the same path (blocks, object
coordinate systems, colors, limits). `acadrust` is not written to our
robustness rules (it may panic on damaged files), so it runs on its own
thread with a large stack, inside `catch_unwind`; a panic becomes an
error and the program goes on. Its memory-mapped reading (`unsafe`) is
not used: the file is read from a byte buffer.

**Items:** [`panic_message`](#cc_cad-dwg-panic_message) · [`read`](#cc_cad-dwg-read)

<a id="cc_cad-dwg-panic_message"></a>

### fn `panic_message`

```rust
pub(crate) fn panic_message(payload: &(dyn std::any::Any + Send)) -> String
```

<a id="cc_cad-dwg-read"></a>

### fn `read`

```rust
pub fn read(data: &[u8], limits: &Limits) -> Result<CadRead, CadError>
```

<a id="mod-cc_cad-dxf"></a>

## `cc_cad::dxf`

<sub>`crates/cc_cad/src/dxf.rs`</sub>

DXF files: sections, tables, blocks and entities, read into raw
entities (their group pairs) that `crate::flatten` turns into
primitives. Model space entities are flattened as they are read, so the
largest section is never held whole.

**Items:** [`Entity`](#cc_cad-dxf-entity) · [`read`](#cc_cad-dxf-read) · [`rgb`](#cc_cad-dxf-rgb)

<a id="cc_cad-dxf-entity"></a>

### struct `Entity`

```rust
pub struct Entity
```

An entity as read: its type, its groups in order, and the entities that
belong to it (a polyline's vertices, an insert's attributes).

**Traits:** `Debug`, `Clone`, `Default`

**Fields**

- `name: String`
- `groups: Vec<(i32, Val)>`
- `children: Vec<Entity>`

**Methods**

- `pub fn get(&self, code: i32) -> Option<&Val>`
- `pub fn real(&self, code: i32, default: f64) -> f64`
- `pub fn int(&self, code: i32, default: i64) -> i64`
- `pub fn str(&self, code: i32) -> &str`
- `pub fn has(&self, code: i32) -> bool`
- `pub fn point(&self, code: i32) -> DVec3`

  The point of groups `code`, `code + 10`, `code + 20` (first occurrence).

- `pub fn normal(&self) -> DVec3`

  The extrusion direction (210/220/230), +Z by default.

- `pub fn points(&self, code: i32) -> Vec<DVec3>`

  Every point of groups `code`/`code + 10`/`code + 20`, in order.

- `pub fn reals(&self, code: i32) -> Vec<f64>`

  Every value of `code`, in order.


<a id="cc_cad-dxf-read"></a>

### fn `read`

```rust
pub fn read(data: &[u8], limits: &Limits) -> Result<CadRead, CadError>
```

Reads a whole DXF file.

<a id="cc_cad-dxf-rgb"></a>

### fn `rgb`

```rust
pub fn rgb(v: i64) -> [u8; 3]
```

The color of a 24-bit true color value.

<a id="mod-cc_cad-flatten"></a>

## `cc_cad::flatten`

<sub>`crates/cc_cad/src/flatten.rs`</sub>

Turning raw entities into flat primitives: object coordinate systems,
block insertion (nested, arrayed, cycle-safe and bounded), colors and
layers by block, and curve tessellation where a primitive cannot stay
an arc.

**Items:** [`Block`](#cc_cad-flatten-block) · [`Flattener`](#cc_cad-flatten-flattener)

<a id="cc_cad-flatten-block"></a>

### struct `Block`

```rust
pub struct Block
```

A block definition: its base point and entities.

**Traits:** `Debug`, `Clone`, `Default`

**Fields**

- `base: DVec3`
- `entities: Vec<Entity>`

<a id="cc_cad-flatten-flattener"></a>

### struct `Flattener`

```rust
pub struct Flattener
```

*Some fields are private.*

**Methods**

- `pub fn new(limits: &Limits) -> Self`
- `pub fn take_inserts(&mut self) -> Vec<CadInsert>`

  The block references of the drawing's entities (blocks kept).

- `pub fn kept_blocks(&mut self, blocks: &HashMap<String, Block>) -> Vec<CadBlock>`

  The blocks the drawing references, each read on its own (base point
  at the origin, layer 0 and BYBLOCK left as they are), with the
  references they make to others; a reference that would close a
  cycle is dropped. Call after the entities and `Self::take_inserts`.

- `pub fn set_layers(&mut self, layers: &[CadLayer])`

  Register the layers of the LAYER table (kept in file order).

- `pub fn finish(mut self) -> (Vec<CadPrim>, Vec<CadLayer>, bool)`

  The primitives, the layers and whether a limit cut the drawing short.

- `pub fn top_level(&mut self, e: &Entity, blocks: &HashMap<String, Block>)`

  Flatten a model space entity.


<a id="mod-cc_cad-geom"></a>

## `cc_cad::geom`

<sub>`crates/cc_cad/src/geom.rs`</sub>

Coordinate systems and curve tessellation for flattening drawings.

**Items:** [`MAX_CURVE_SEGMENTS`](#cc_cad-geom-max_curve_segments) · [`ocs_axes`](#cc_cad-geom-ocs_axes) · [`ocs`](#cc_cad-geom-ocs) · [`Affine2`](#cc_cad-geom-affine2) · [`arc_segments`](#cc_cad-geom-arc_segments) · [`arc_points`](#cc_cad-geom-arc_points) · [`bulge_arc`](#cc_cad-geom-bulge_arc) · [`ellipse_points`](#cc_cad-geom-ellipse_points) · [`spline_points`](#cc_cad-geom-spline_points)

<a id="cc_cad-geom-max_curve_segments"></a>

### const `MAX_CURVE_SEGMENTS`

```rust
pub const MAX_CURVE_SEGMENTS: usize
```

Most segments one curve (ellipse, spline) is tessellated into.

<a id="cc_cad-geom-ocs_axes"></a>

### fn `ocs_axes`

```rust
pub fn ocs_axes(normal: DVec3) -> (DVec3, DVec3, DVec3)
```

The object coordinate system of an extrusion direction (the DXF
"arbitrary axis algorithm"): its X and Y axes in world coordinates.

<a id="cc_cad-geom-ocs"></a>

### fn `ocs`

```rust
pub fn ocs(normal: DVec3) -> DAffine3
```

The transform from an OCS (of `normal`) to world coordinates.

<a id="cc_cad-geom-affine2"></a>

### struct `Affine2`

```rust
pub struct Affine2
```

A 2D affine map `p ↦ m·p + t` (the plan projection of a 3D transform of a plane).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `m: DMat2`
- `t: DVec2`

**Methods**

- `pub fn of_plane(t: &DAffine3, elevation: f64) -> Self`

  Points `(x, y)` of the plane `z = elevation` of the frame `t`, projected on the world XY plane.

- `pub fn apply(&self, p: DVec2) -> DVec2`
- `pub fn vector(&self, v: DVec2) -> DVec2`
- `pub fn det(&self) -> f64`
- `pub fn similarity(&self) -> Option<f64>`

  The uniform scale when the map is a similarity (rotation, uniform
  scale, maybe a mirror), else `None`.


<a id="cc_cad-geom-arc_segments"></a>

### fn `arc_segments`

```rust
pub fn arc_segments(sweep: f64) -> usize
```

Number of segments for `sweep` radians, about 5° each, within bounds.

<a id="cc_cad-geom-arc_points"></a>

### fn `arc_points`

```rust
pub fn arc_points(center: DVec2, radius: f64, start: f64, sweep: f64) -> Vec<DVec2>
```

Points of a circular arc.

<a id="cc_cad-geom-bulge_arc"></a>

### fn `bulge_arc`

```rust
pub fn bulge_arc(a: DVec2, b: DVec2, bulge: f64) -> Option<(DVec2, f64, f64, f64)>
```

The arc of a polyline segment from `a` to `b` with `bulge`: center,
radius, start angle and sweep; `None` for a straight segment.

<a id="cc_cad-geom-ellipse_points"></a>

### fn `ellipse_points`

```rust
pub fn ellipse_points(center: DVec3, major: DVec3, ratio: f64, normal: DVec3, t0: f64, t1: f64) -> Vec<DVec3>
```

Points of an elliptical arc (world coordinates) from parameter `t0` to
`t1`, counter-clockwise about `normal`.

<a id="cc_cad-geom-spline_points"></a>

### fn `spline_points`

```rust
pub fn spline_points(degree: usize, knots: &[f64], control: &[DVec3], weights: &[f64]) -> Option<Vec<DVec3>>
```

Points along a NURBS curve of `degree` with `knots` and `control` points
(weights optional), by de Boor's algorithm; `None` when the data is
inconsistent.

<a id="mod-cc_cad-svg"></a>

## `cc_cad::svg`

<sub>`crates/cc_cad/src/svg.rs`</sub>

SVG drawings into the neutral CAD drawing: paths (curves flattened to
polylines) in their stroke color, single-outline filled shapes as fills,
and embedded pictures as images. Units are millimeters at the CSS 96
pixels per inch, y up, the page's bottom-left corner at the origin.
Texts are not read (they need fonts): SVGs from CAD programs usually
carry them as outlines already.

**Items:** [`MAX_SVG_BYTES`](#cc_cad-svg-max_svg_bytes) · [`is_svg`](#cc_cad-svg-is_svg) · [`options`](#cc_cad-svg-options) · [`read`](#cc_cad-svg-read)

<a id="cc_cad-svg-max_svg_bytes"></a>

### const `MAX_SVG_BYTES`

```rust
pub const MAX_SVG_BYTES: usize
```

Largest SVG file read, bytes.

<a id="cc_cad-svg-is_svg"></a>

### fn `is_svg`

```rust
pub fn is_svg(data: &[u8]) -> bool
```

Whether `data` looks like SVG (an `<svg` element near the start).

<a id="cc_cad-svg-options"></a>

### fn `options`

```rust
pub fn options() -> usvg::Options<'static>
```

Parsing options that read nothing but the file itself: an `<image>`
whose href is not a `data:` URL (a path on this machine, say) comes in
empty.

<a id="cc_cad-svg-read"></a>

### fn `read`

```rust
pub fn read(data: &[u8], limits: &Limits) -> Result<CadRead, CadError>
```

Read an SVG file's bytes.
usvg parses and converts nested elements recursively (it allows over a
thousand levels), which can overflow a small stack, and a stack overflow
aborts the process; so the file is read on a thread of its own with a
large stack, inside `catch_unwind`, like DWG files.

<a id="mod-cc_cad-text"></a>

## `cc_cad::text`

<sub>`crates/cc_cad/src/text.rs`</sub>

Text strings of drawings: control codes and MTEXT formatting.

**Items:** [`plain`](#cc_cad-text-plain) · [`mtext_lines`](#cc_cad-text-mtext_lines)

<a id="cc_cad-text-plain"></a>

### fn `plain`

```rust
pub fn plain(s: &str) -> String
```

Replace `\U+XXXX` escapes and the `%%` control codes of single-line
text (`%%c` diameter, `%%d` degree, `%%p` plus-minus, `%%nnn` a
character code; `%%u`/`%%o` underline and overline toggles dropped).

<a id="cc_cad-text-mtext_lines"></a>

### fn `mtext_lines`

```rust
pub fn mtext_lines(s: &str) -> Vec<String>
```

The lines of an MTEXT string without its formatting: `\P` breaks lines,
`\~` is a space, `\\ \{ \}` are literal, stacked fractions `\Sa^b;`
become `a/b`, other codes (`\A1;`, `\H2x;`, `\fArial|b0;`, `\C1;`, `\L`…)
and grouping braces are removed.

<a id="mod-cc_cad-tokens"></a>

## `cc_cad::tokens`

<sub>`crates/cc_cad/src/tokens.rs`</sub>

DXF group-code pairs, from ASCII or binary files.
A DXF file is a sequence of (group code, value) pairs; the code decides
the value's type. ASCII files put each on its own line; binary files
(after the sentinel `AutoCAD Binary DXF\r\n\x1a\0`) store the code as a
little-endian 16-bit integer and the value in its binary form.

**Items:** [`BINARY_SENTINEL`](#cc_cad-tokens-binary_sentinel) · [`Val`](#cc_cad-tokens-val) · [`Decoder`](#cc_cad-tokens-decoder) · [`Pairs`](#cc_cad-tokens-pairs)

<a id="cc_cad-tokens-binary_sentinel"></a>

### const `BINARY_SENTINEL`

```rust
pub const BINARY_SENTINEL: &[u8]
```

The sentinel that starts a binary DXF file.

<a id="cc_cad-tokens-val"></a>

### enum `Val`

```rust
pub enum Val
```

A group value.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Variants**

- `Str(String)`
- `Real(f64)`
- `Int(i64)`
- `Bin`

  Binary chunks (thumbnails, proxy data): ignored.


**Methods**

- `pub fn real(&self) -> f64`
- `pub fn int(&self) -> i64`
- `pub fn str(&self) -> &str`

<a id="cc_cad-tokens-decoder"></a>

### struct `Decoder`

```rust
pub struct Decoder
```

Decodes strings: UTF-8 when valid, else the drawing's code page.

**Traits:** `Default`

*Some fields are private.*

**Methods**

- `pub fn set_code_page(&mut self, name: &str)`

  Use the code page named by `$DWGCODEPAGE` (e.g. `ANSI_1252`).

- `pub fn decode(&self, bytes: &[u8]) -> String`

<a id="cc_cad-tokens-pairs"></a>

### struct `Pairs`

```rust
pub struct Pairs<'a>
```

Reads pairs one at a time, with one pair of look-ahead.

**Fields**

- `decoder: Decoder`

*Some fields are private.*

**Methods**

- `pub fn new(data: &'a [u8]) -> Self`
- `pub fn is_binary(&self) -> bool`
- `pub fn next_pair(&mut self) -> Result<Option<(i32, Val)>, CadError>`

  The next pair, or `None` at the end of the data.

- `pub fn peek(&mut self) -> Result<Option<&(i32, Val)>, CadError>`

  Look at the next pair without consuming it.


{% endraw %}
