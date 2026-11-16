# `cc_render` API

{% raw %}
[← API reference](index.md) · Generated from the source by `cargo run -p api_doc`; edit the doc comments, not this page.

Offscreen wgpu renderer for the 3D view.
The app builds a `SceneMesh` from the model geometry, uploads it with
`Renderer3D::upload` and calls `Renderer3D::render` each frame; the
resulting texture is shown as an egui image. `Renderer3D::set_light`
lights it by a sun fixed in the world, with cast shadows.

## Modules

| Module | Items | Summary |
|---|---|---|
| [`cc_render`](#mod-cc_render) | 0 | Offscreen wgpu renderer for the 3D view. |
| [`cc_render::camera`](#mod-cc_render-camera) | 2 |  |
| [`cc_render::mesh`](#mod-cc_render-mesh) | 9 |  |
| [`cc_render::renderer`](#mod-cc_render-renderer) | 8 |  |
| [`cc_render::select`](#mod-cc_render-select) | 1 | Window and crossing selection in a 3D view: which elements a screen rectangle takes, as the plans do with their linework. |
| [`cc_render::shadow`](#mod-cc_render-shadow) | 4 | The sun's GPU side: its pipelines and uniforms, made the first time a frame is lit by a sun, and its shadow map, made the first time it casts shadows (see `sun.rs` for the math). |
| [`cc_render::sun`](#mod-cc_render-sun) | 10 | A sun fixed in the world: the light it casts, the orthographic shadow map fitted to the scene and the ground that catches its shadows. |
| [`cc_render::textures`](#mod-cc_render-textures) | 4 | Material textures for the 3D view: each material's image decoded once, resized to one square layer of a texture array (with its mipmaps), so the GPU memory they take is bounded (`LAYER_SIZE`, `MAX_LAYERS`). |

<a id="mod-cc_render"></a>

## `cc_render`

<sub>`crates/cc_render/src/lib.rs`</sub>

**Re-exports**

- `pub use camera::{pick_ray, view_proj}`
- `pub use mesh::{ Highlight, MAX_HIDDEN_RUNS, SceneMesh, Shown, SolidStyle, Span, Tessellation, Vertex, hidden_runs, plane_axes, }`
- `pub use renderer::Renderer3D`
- `pub use select::elements_in_rect`
- `pub use sun::Light`
- `pub use textures::TextureSet`

<a id="mod-cc_render-camera"></a>

## `cc_render::camera`

<sub>`crates/cc_render/src/camera.rs`</sub>

**Items:** [`view_proj`](#cc_render-camera-view_proj) · [`pick_ray`](#cc_render-camera-pick_ray)

<a id="cc_render-camera-view_proj"></a>

### fn `view_proj`

```rust
pub fn view_proj(cam: &CameraState, aspect: f32) -> Mat4
```

Perspective view-projection matrix (Z up, wgpu depth range 0..1).

<a id="cc_render-camera-pick_ray"></a>

### fn `pick_ray`

```rust
pub fn pick_ray(cam: &CameraState, aspect: f32, ndc_x: f32, ndc_y: f32) -> Ray
```

World-space ray through normalized device coordinates (`x`, `y` in -1..1, y up).

<a id="mod-cc_render-mesh"></a>

## `cc_render::mesh`

<sub>`crates/cc_render/src/mesh.rs`</sub>

**Re-exports**

- `pub use cc_geom::plane_axes`

**Items:** [`Vertex`](#cc_render-mesh-vertex) · [`SceneMesh`](#cc_render-mesh-scenemesh) · [`Span`](#cc_render-mesh-span) · [`Shown`](#cc_render-mesh-shown) · [`Highlight`](#cc_render-mesh-highlight) · [`MAX_HIDDEN_RUNS`](#cc_render-mesh-max_hidden_runs) · [`hidden_runs`](#cc_render-mesh-hidden_runs) · [`Tessellation`](#cc_render-mesh-tessellation) · [`SolidStyle`](#cc_render-mesh-solidstyle)

<a id="cc_render-mesh-vertex"></a>

### struct `Vertex`

```rust
pub struct Vertex
```

**Traits:** `Debug`, `Clone`, `Copy`, `Pod`, `Zeroable`

**Fields**

- `pos: [f32; 3]`
- `normal: [f32; 3]`
- `color: [f32; 4]`
- `fg: [f32; 4]`

  The surface pattern drawn over a face (textures style): its color
  (linear) and, in `w`, its kind (the order of `cc_core::Pattern`, 0 for none).

- `bg: [f32; 4]`
- `tex: [f32; 4]`

  The material's texture (textures style): coordinates in repeats in
  `xy` and, in `z`, its layer plus one (0 for none).

- `plane: [f32; 2]`

  Where the vertex is on its face, in meters (see `plane_axes`),
  running on across the facets of a curved surface: what the surface
  patterns are laid along.


**Methods**

- `pub fn plain(pos: [f32; 3], normal: [f32; 3], color: [f32; 4]) -> Self`

  A vertex with no surface pattern or texture.


<a id="cc_render-mesh-scenemesh"></a>

### struct `SceneMesh`

```rust
pub struct SceneMesh
```

GPU-ready triangles and edge lines.

**Traits:** `Debug`, `Clone`, `Default`

**Fields**

- `vertices: Vec<Vertex>`
- `indices: Vec<u32>`
- `transparent_indices: Vec<u32>`

  Triangles of transparent solids, drawn last with blending.

- `edges: Vec<Vertex>`
- `unlit: bool`

  Faces drawn without light (hidden line, consistent colors).

- `grey: bool`

  Faces shaded grey by the light, as displayed (the shaded style: see
  `cc_core::VisualStyle::shaded_grey`), a cast shadow's grey where
  the sun does not reach them; else lit in their colors.

- `spans: Vec<Span>`

  Where each solid part of the model sits in the buffers, in the order
  it was added (see `SceneMesh::highlight`); empty for meshes whose
  solids the selection does not change (analytical, rebar).


**Methods**

- `pub fn build(tess: &Tessellation, selected: &BTreeSet<ElementId>, style: &mut dyn FnMut(ElementId) -> Option<SolidStyle>) -> Self`

  The model's solids in their materials' colors, lit (outside any
  view's style: thumbnails); `style` gives each element's view
  graphics (`None` hides it).

- `pub fn build_at(tess: &Tessellation, selected: &BTreeSet<ElementId>, style: &mut dyn FnMut(ElementId) -> Option<SolidStyle>, detail: cc_core::DetailLevel) -> Self`

  `SceneMesh::build` at `detail`: component pieces show at the
  levels they are set for.

- `pub fn build_styled(tess: &Tessellation, selected: &BTreeSet<ElementId>, style: &mut dyn FnMut(ElementId) -> Option<SolidStyle>, detail: cc_core::DetailLevel, visual: cc_core::VisualStyle) -> Self`

  The model's solids in a visual style, at `detail`: the wireframe has
  only edges, hidden line white unlit faces, shaded white faces
  greyed by the light (see `SceneMesh::grey`), consistent colors the
  faces' colors unlit, textures the materials' appearance colors with
  their surface patterns, lit (view graphics' color overrides win over
  all of them).

- `pub fn build_textured(tess: &Tessellation, selected: &BTreeSet<ElementId>, style: &mut dyn FnMut(ElementId) -> Option<SolidStyle>, detail: cc_core::DetailLevel, visual: cc_core::VisualStyle, textures: &TextureSet) -> Self`

  `SceneMesh::build_styled` with the materials' textures: in the textures
  style, faces of a material with a texture in `textures` show it (not
  where view graphics override the color, under halftone or selected).

- `pub fn highlight(tess: &Tessellation, spans: &[Span], selected: &BTreeSet<ElementId>) -> Highlight`

  The selection over a mesh built from `tess` with no selection, whose
  spans are `spans`: the selected elements' parts as
  `SceneMesh::build_textured` shows them selected, and the rest of
  the base mesh to draw. Together they draw the triangles and edges of
  a mesh built with the selection.

- `pub fn build_plain(tess: &Tessellation, selected: &BTreeSet<ElementId>) -> Self`

  Every solid as its material shows it.

- `pub fn build_rebar(geom: &ModelGeometry, tess: &Tessellation, selected: &BTreeSet<ElementId>) -> Self`

  The reinforcing bars (those of the selected hosts or sets highlighted)
  over the model's solids drawn faint.

- `pub fn rebar_highlight(geom: &ModelGeometry, selected: &BTreeSet<ElementId>) -> Highlight`

  The bars `SceneMesh::build_rebar` highlights, drawn over a mesh
  it built with no selection (the same lines, so they win the depth test).

- `pub fn build_analytical(tess: &Tessellation, model: &cc_geom::analytical::AnalyticalModel, selected: &BTreeSet<ElementId>) -> Self`

  The analytical model: member lines (columns blue, beams red, the
  selection highlighted) and node crosses (supports green) over the
  model's solids drawn faint.

- `pub fn analytical_highlight(model: &cc_geom::analytical::AnalyticalModel, selected: &BTreeSet<ElementId>) -> Highlight`

  The members `SceneMesh::build_analytical` highlights, drawn over a
  mesh it built with no selection (the same lines, so they win the depth test).


<a id="cc_render-mesh-span"></a>

### struct `Span`

```rust
pub struct Span
```

The ranges of one solid part (see `Tessellation`) in a `SceneMesh`'s
buffers. A mesh's spans follow one another: each range starts where the
previous span's ended.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Eq`

**Fields**

- `element: ElementId`
- `vertices: Range<u32>`
- `indices: Range<u32>`
- `transparent: Range<u32>`
- `edges: Range<u32>`

*Some fields are private.*

<a id="cc_render-mesh-shown"></a>

### struct `Shown`

```rust
pub struct Shown
```

The ranges of a base mesh's buffers to draw (the others are covered by a
`Highlight`): index ranges of its opaque and transparent triangles and
vertex ranges of its edges. Ranges may run past the buffers' ends.

**Traits:** `Debug`, `Clone`, `Default`, `PartialEq`, `Eq`

**Fields**

- `indices: Vec<Range<u32>>`
- `transparent: Vec<Range<u32>>`
- `edges: Vec<Range<u32>>`

<a id="cc_render-mesh-highlight"></a>

### struct `Highlight`

```rust
pub struct Highlight
```

The selection drawn over a base mesh built without it: the selected
solids' faces (opaque, untextured, in the selection color) and edges, or
the selected analytical members or bars, plus the parts of the base mesh
to keep (`None`: all of it).

**Traits:** `Debug`, `Clone`, `Default`

**Fields**

- `mesh: SceneMesh`

  Opaque triangles (`indices`) and edges; nothing transparent.

- `shown: Option<Shown>`

<a id="cc_render-mesh-max_hidden_runs"></a>

### const `MAX_HIDDEN_RUNS`

```rust
pub const MAX_HIDDEN_RUNS: usize
```

The most runs of selected spans `SceneMesh::highlight` takes: each run
splits the base mesh's draws in two, so a selection scattered wider than
this is better built into the base mesh (see `hidden_runs`).

<a id="cc_render-mesh-hidden_runs"></a>

### fn `hidden_runs`

```rust
pub fn hidden_runs(spans: &[Span], selected: &BTreeSet<ElementId>) -> usize
```

How many runs of consecutive spans of selected elements `spans` holds:
the draws `SceneMesh::highlight` would add to each of the base mesh's buffers.

<a id="cc_render-mesh-tessellation"></a>

### struct `Tessellation`

```rust
pub struct Tessellation
```

The triangles and outline edges of a model's solids, without colors:
the costly part of a `SceneMesh`, kept while the geometry stays, so a
new selection or view graphics only repaint it.

**Traits:** `Debug`, `Clone`, `Default`

*Some fields are private.*

**Methods**

- `pub fn new(geom: &ModelGeometry) -> Self`

<a id="cc_render-mesh-solidstyle"></a>

### struct `SolidStyle`

```rust
pub struct SolidStyle
```

View graphics of an element's solids in 3D; `None` keeps the material's.

**Traits:** `Debug`, `Clone`, `Copy`, `Default`, `PartialEq`

**Fields**

- `color: Option<[u8; 3]>`
- `transparency: Option<f64>`

  0 (opaque) to 1.

- `halftone: bool`

  Faces and edges mixed with white.

- `edges: Option<[u8; 3]>`

  Edge color.


<a id="mod-cc_render-renderer"></a>

## `cc_render::renderer`

<sub>`crates/cc_render/src/renderer.rs`</sub>

**Items:** [`DEPTH_FORMAT`](#cc_render-renderer-depth_format) · [`FACE_BIAS`](#cc_render-renderer-face_bias) · [`Uniforms`](#cc_render-renderer-uniforms) · [`shading`](#cc_render-renderer-shading) · [`unlit_shadow`](#cc_render-renderer-unlit_shadow) · [`PipelineSpec`](#cc_render-renderer-pipelinespec) · [`frame_pipeline`](#cc_render-renderer-frame_pipeline) · [`Renderer3D`](#cc_render-renderer-renderer3d)

<a id="cc_render-renderer-depth_format"></a>

### const `DEPTH_FORMAT`

```rust
pub(crate) const DEPTH_FORMAT: wgpu::TextureFormat
```

<a id="cc_render-renderer-face_bias"></a>

### const `FACE_BIAS`

```rust
pub(crate) const FACE_BIAS: wgpu::DepthBiasState
```

Pushes faces slightly back so coplanar edges win the depth test.

<a id="cc_render-renderer-uniforms"></a>

### struct `Uniforms`

```rust
pub(crate) struct Uniforms
```

**Traits:** `Clone`, `Copy`, `Pod`, `Zeroable`

**Fields**

- `view_proj: [[f32; 4]; 4]`
- `light_dir: [f32; 4]`
- `style: [f32; 4]`

  x: 1 for unlit faces; y: 1 for faces shaded grey (the shaded style);
  z: what the unlit styles keep of a color in a cast shadow (linear light).

- `shading: [f32; 4]`

  The shaded style's greys, as displayed: x ambient, y diffuse, z a
  cast shadow's (never lighter than the face; see
  `cc_core::ShadingSettings`); w the opacity of the ground's shadows.


<a id="cc_render-renderer-shading"></a>

### fn `shading`

```rust
pub(crate) fn shading(shading: &cc_core::ShadingSettings) -> [f32; 4]
```

The greys of `shading`, as `Uniforms::shading` holds them.

<a id="cc_render-renderer-unlit_shadow"></a>

### fn `unlit_shadow`

```rust
pub(crate) fn unlit_shadow(shading: &cc_core::ShadingSettings) -> f32
```

What the unlit styles keep of a color in a cast shadow under `shading`
(`Uniforms::style`'s z).

<a id="cc_render-renderer-pipelinespec"></a>

### struct `PipelineSpec`

```rust
pub(crate) struct PipelineSpec<'a>
```

What sets a pipeline drawing into the frame's targets apart.

**Fields**

- `label: &'a str`
- `topology: wgpu::PrimitiveTopology`
- `vs: &'a str`
- `fs: &'a str`
- `vertices: bool`

  It reads the mesh's vertices (else it makes its own).

- `cull: Option<wgpu::Face>`
- `bias: wgpu::DepthBiasState`
- `blend: Option<wgpu::BlendState>`

  Blended without writing depth; opaque with `None`.


<a id="cc_render-renderer-frame_pipeline"></a>

### fn `frame_pipeline`

```rust
pub(crate) fn frame_pipeline(device: &wgpu::Device, layout: &wgpu::PipelineLayout, shader: &wgpu::ShaderModule, spec: PipelineSpec) -> wgpu::RenderPipeline
```

A pipeline drawing into the frame's multisampled color and depth targets.

<a id="cc_render-renderer-renderer3d"></a>

### struct `Renderer3D`

```rust
pub struct Renderer3D
```

Draws a `SceneMesh` into an offscreen, multisampled texture, with an
optional `Highlight` (the selection) over it, lit from the camera or by
a sun fixed in the world (see `Renderer3D::set_light`).

*Some fields are private.*

**Methods**

- `pub fn new(device: &wgpu::Device) -> Self`
- `pub fn set_light(&mut self, light: Option<Light>)`

  Light the frames by `light`, a sun fixed in the world, or from the
  camera with `None` (the default). A direction that is zero or not a
  number counts as `None`. The frame is drawn again, and the shadow map
  made again, only when the light changes.

- `pub fn set_shading(&mut self, shading: cc_core::ShadingSettings)`

  Shade the faces and cast the shadows as `shading` says (a view's
  `cc_core::View::shading`; the defaults until set). The frame is
  drawn again only when it changes.

- `pub(crate) fn texture_array(device: &wgpu::Device, size: u32, layers: u32, mips: u32) -> wgpu::Texture`

  A texture array of `layers` square layers of `size` pixels with `mips` levels.

- `pub(crate) fn array_view() -> wgpu::TextureViewDescriptor<'static>`

  Views of the texture array are arrays even with one layer.

- `pub(crate) fn bind(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, uniforms: &wgpu::Buffer, layers: &wgpu::TextureView, sampler: &wgpu::Sampler) -> wgpu::BindGroup`
- `pub fn set_textures(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, set: &TextureSet)`

  Upload the materials' textures (when they changed since the last
  call); meshes built with `SceneMesh::build_textured` and the same
  set refer to their layers.

- `pub fn upload(&mut self, device: &wgpu::Device, mesh: &SceneMesh)`

  Upload a new base mesh; the highlight over the old one is dropped
  (its ranges refer to that mesh).

- `pub fn set_highlight(&mut self, device: &wgpu::Device, highlight: Option<&Highlight>)`

  Draw `highlight` (made for the mesh last uploaded) over that mesh, or
  nothing with `None`; the base mesh's buffers stay as they are.

- `pub fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, size: [u32; 2], camera: &CameraState) -> Option<(&wgpu::TextureView, bool)>`

  Render a frame (unless the last one, with the same mesh, light, camera
  and size, is still in the targets). Returns the output view and whether it
  was recreated (callers must then re-register it with egui); `None` if
  there are no targets.

- `pub fn render_to_rgba(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, size: [u32; 2], camera: &CameraState) -> Option<Vec<u8>>`

  Render one frame of `size` pixels and read it back as tightly packed
  RGBA bytes (already sRGB encoded), e.g. for thumbnails.


<a id="mod-cc_render-select"></a>

## `cc_render::select`

<sub>`crates/cc_render/src/select.rs`</sub>

Window and crossing selection in a 3D view: which elements a screen
rectangle takes, as the plans do with their linework. A window takes the
elements whose solids lie wholly inside it (and in front of the camera);
a crossing also takes those that any face of reaches into it. Hidden
faces count too, as in plans.

**Items:** [`elements_in_rect`](#cc_render-select-elements_in_rect)

<a id="cc_render-select-elements_in_rect"></a>

### fn `elements_in_rect`

```rust
pub fn elements_in_rect(solids: &[Solid], cam: &CameraState, aspect: f32, lo: DVec2, hi: DVec2, window: bool, shown: &mut dyn FnMut(ElementId) -> bool) -> Vec<ElementId>
```

Elements of `solids` (those `shown` accepts) taken by the rectangle from
`lo` to `hi` in normalized device coordinates (-1..1, y up): wholly
inside it when `window`, or reaching into it otherwise.

<a id="mod-cc_render-shadow"></a>

## `cc_render::shadow`

<sub>`crates/cc_render/src/shadow.rs`</sub>

The sun's GPU side: its pipelines and uniforms, made the first time a
frame is lit by a sun, and its shadow map, made the first time it casts
shadows (see `sun.rs` for the math).

**Items:** [`SUN_SHADER`](#cc_render-shadow-sun_shader) · [`Casters`](#cc_render-shadow-casters) · [`SunPass`](#cc_render-shadow-sunpass) · [`caught`](#cc_render-shadow-caught)

<a id="cc_render-shadow-sun_shader"></a>

### const `SUN_SHADER`

```rust
pub(crate) const SUN_SHADER: &str
```

The sun's shader module: the renderer's with the sun's entry points.

<a id="cc_render-shadow-casters"></a>

### struct `Casters`

```rust
pub(crate) struct Casters<'a>
```

The opaque triangles that cast shadows.

**Fields**

- `vertices: &'a wgpu::Buffer`
- `indices: &'a wgpu::Buffer`
- `count: u32`

<a id="cc_render-shadow-sunpass"></a>

### struct `SunPass`

```rust
pub(crate) struct SunPass
```

**Fields**

- `faces: wgpu::RenderPipeline`

  Faces lit by the sun, opaque and blended, and the edges, as the
  renderer's own pipelines draw them.

- `blended: wgpu::RenderPipeline`
- `edges: wgpu::RenderPipeline`
- `ground: wgpu::RenderPipeline`

  The ground that catches shadows.

- `bind_group: wgpu::BindGroup`

  Group 1 of the sun's pipelines: its uniforms and shadow map.

- `draws_ground: bool`

  The ground is drawn (the sun is up and casts shadows on it).


*Some fields are private.*

**Methods**

- `pub fn new(device: &wgpu::Device, bind_layout: &wgpu::BindGroupLayout, sampler: &wgpu::Sampler) -> Self`

  The sun's pipelines over the renderer's `bind_layout` (group 0) and
  its texture `sampler`.

- `pub fn map_size(&self) -> u32`

  The side of the shadow map, texels (1 until the sun casts shadows).

- `pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, light: Light, bounds: Option<Bounds>, casters: Option<Casters>)`

  Set the sun's uniforms for `light` (usable) over a mesh within
  `bounds`, and draw the shadow map of its opaque triangles when it
  casts shadows.


<a id="cc_render-shadow-caught"></a>

### fn `caught`

```rust
pub(crate) fn caught<T>(device: &wgpu::Device, make: impl FnOnce() -> T) -> Option<T>
```

Run `make` catching the device errors it causes (wgpu aborts on those
left uncaught): `None` if there were any. Native devices report them at
once; a report still pending counts as none.

<a id="mod-cc_render-sun"></a>

## `cc_render::sun`

<sub>`crates/cc_render/src/sun.rs`</sub>

A sun fixed in the world: the light it casts, the orthographic shadow map
fitted to the scene and the ground that catches its shadows. Pure math;
the renderer draws them (see `shadow.rs`).

**Items:** [`Light`](#cc_render-sun-light) · [`replace`](#cc_render-sun-replace) · [`SHADOW_MAP_SIZE`](#cc_render-sun-shadow_map_size) · [`Bounds`](#cc_render-sun-bounds) · [`ShadowFit`](#cc_render-sun-shadowfit) · [`fit`](#cc_render-sun-fit) · [`ground_quad`](#cc_render-sun-ground_quad) · [`SunUniforms`](#cc_render-sun-sununiforms) · [`SunSetup`](#cc_render-sun-sunsetup) · [`setup`](#cc_render-sun-setup)

<a id="cc_render-sun-light"></a>

### struct `Light`

```rust
pub struct Light
```

A sun fixed in the world.

**Traits:** `Clone`, `Copy`, `Debug`, `PartialEq`

**Fields**

- `direction: Vec3`

  Unit vector from the model towards the sun (Z up).

- `shadows: bool`

  Cast shadows.

- `ground: Option<f32>`

  Height of a horizontal ground plane that receives shadows, if any.


**Methods**

- `pub(crate) fn usable(self) -> Option<Self>`

  The light as the renderer uses it: its direction made a unit vector,
  `None` when it is zero or not a number (the camera's light is used
  then); a ground height that is not a number is left out.


<a id="cc_render-sun-replace"></a>

### fn `replace`

```rust
pub(crate) fn replace(current: &mut Option<Light>, light: Option<Light>) -> bool
```

Sets `current` to the usable form of `light`; returns whether it changed
(what has been drawn with the old one is then out of date).

<a id="cc_render-sun-shadow_map_size"></a>

### const `SHADOW_MAP_SIZE`

```rust
pub(crate) const SHADOW_MAP_SIZE: u32
```

Side of the square shadow map, texels.

<a id="cc_render-sun-bounds"></a>

### struct `Bounds`

```rust
pub(crate) struct Bounds
```

An axis-aligned box.

**Traits:** `Clone`, `Copy`, `Debug`, `PartialEq`

**Fields**

- `min: Vec3`
- `max: Vec3`

**Methods**

- `pub fn of(points: impl IntoIterator<Item = [f32; 3]>) -> Option<Self>`

  The box around the finite `points`; `None` when there are none.

- `pub fn corners(&self) -> [Vec3; 8]`

<a id="cc_render-sun-shadowfit"></a>

### struct `ShadowFit`

```rust
pub(crate) struct ShadowFit
```

The light's view of a scene: world to its clip space, where x and y run
-1 to 1 across the shadow map and depth 0 (nearest the sun) to 1.

**Traits:** `Clone`, `Copy`, `Debug`, `PartialEq`

**Fields**

- `view_proj: Mat4`
- `texel: f32`

  The longer side of a map texel, meters.

- `depth: f32`

  The depth range, meters.


<a id="cc_render-sun-fit"></a>

### fn `fit`

```rust
pub(crate) fn fit(bounds: &Bounds, d: Vec3, ground: Option<&[Vec3; 4]>, map: u32) -> Option<ShadowFit>
```

The orthographic view from a sun towards unit `d` (above the horizon or
not) that holds every point of `bounds` (the casters, which set the depth
range) and, across the map, the `ground` corners (receivers behind or in
front of every caster are compared at the range's ends, where they are
lit). It depends on the scene and the sun only, so shadows stay put while
the camera moves. `None` for input that is not a number.

<a id="cc_render-sun-ground_quad"></a>

### fn `ground_quad`

```rust
pub(crate) fn ground_quad(bounds: &Bounds, d: Vec3, z: f32) -> Option<[Vec3; 4]>
```

The ground at height `z` that catches the shadows of `bounds` from a sun
towards unit `d` (above the horizon): a level rectangle along the shadows
from the scene to the tip of the longest one, cut at the length a sun
`MIN_GROUND_ALTITUDE_DEG` up casts. Its corners, counter-clockwise seen
from above, are those of the scene slid along the sun's rays onto the
ground, so the light sees them within the scene: they fall inside its
shadow map. `None` when nothing stands above the ground, the sun is
(almost) level or the input is not a number.

<a id="cc_render-sun-sununiforms"></a>

### struct `SunUniforms`

```rust
pub(crate) struct SunUniforms
```

The sun's uniforms (`Sun` in `sun.wgsl`).

**Traits:** `Clone`, `Copy`, `Debug`, `PartialEq`, `Pod`, `Zeroable`

**Fields**

- `light_view_proj: [[f32; 4]; 4]`
- `dir: [f32; 4]`

  xyz: unit vector towards the sun, zero below the horizon.

- `shadow: [f32; 4]`

  x: 1 to look shadows up; y: a texel, meters; z: depth bias, in map
  depth; w: a texel, in map coordinates.

- `ground: [[f32; 4]; 4]`

  The ground's corners (xyz), counter-clockwise seen from above.


<a id="cc_render-sun-sunsetup"></a>

### struct `SunSetup`

```rust
pub(crate) struct SunSetup
```

What a frame lit by a sun draws.

**Traits:** `Clone`, `Copy`, `Debug`, `PartialEq`

**Fields**

- `uniforms: SunUniforms`
- `shadow: Option<Mat4>`

  The light's view-projection, when the shadow map is drawn.

- `ground: bool`

  The ground catches shadows.


<a id="cc_render-sun-setup"></a>

### fn `setup`

```rust
pub(crate) fn setup(light: Light, bounds: Option<Bounds>, map: u32) -> SunSetup
```

The sun's setup for `light` (usable) over a scene within `bounds`: no
diffuse light, shadows or ground below the horizon; shadows only when
asked for and the scene has bounds.

<a id="mod-cc_render-textures"></a>

## `cc_render::textures`

<sub>`crates/cc_render/src/textures.rs`</sub>

Material textures for the 3D view: each material's image decoded once,
resized to one square layer of a texture array (with its mipmaps), so
the GPU memory they take is bounded (`LAYER_SIZE`, `MAX_LAYERS`).
A material whose image cannot be decoded, or past the last layer, shows
its appearance color instead.

**Items:** [`LAYER_SIZE`](#cc_render-textures-layer_size) · [`MAX_LAYERS`](#cc_render-textures-max_layers) · [`TextureSet`](#cc_render-textures-textureset) · [`mip_count`](#cc_render-textures-mip_count)

<a id="cc_render-textures-layer_size"></a>

### const `LAYER_SIZE`

```rust
pub const LAYER_SIZE: u32
```

Side of a layer, pixels.

<a id="cc_render-textures-max_layers"></a>

### const `MAX_LAYERS`

```rust
pub const MAX_LAYERS: usize
```

Most textures a view holds at once.

<a id="cc_render-textures-textureset"></a>

### struct `TextureSet`

```rust
pub struct TextureSet
```

The textures of a project's materials, ready to upload.

**Traits:** `Debug`, `Clone`, `Default`

*Some fields are private.*

**Methods**

- `pub fn update(&mut self, project: &cc_core::Project)`

  Follow the materials of `project`: images already decoded are kept,
  new ones decoded, those no longer used dropped.

- `pub fn of(&self, name: &str) -> Option<(u32, &MaterialTexture)>`

  The layer of material `name`'s texture and its layout.

- `pub fn len(&self) -> usize`

  How many layers there are.

- `pub fn is_empty(&self) -> bool`
- `pub fn generation(&self) -> u64`
- `pub(crate) fn level(&self, level: usize) -> impl Iterator<Item = &[u8]>`

  Mip level `level` of each layer, in layer order.


<a id="cc_render-textures-mip_count"></a>

### fn `mip_count`

```rust
pub(crate) fn mip_count() -> u32
```

Mip levels of a layer: `LAYER_SIZE` down to 1 pixel.

{% endraw %}
