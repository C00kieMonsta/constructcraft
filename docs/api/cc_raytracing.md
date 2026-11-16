# `cc_raytracing` API

{% raw %}
[← API reference](index.md) · Generated from the source by `cargo run -p api_doc`; edit the doc comments, not this page.

Photorealistic rendering of a 3D view: path tracing on the pure Rust
`sort-raytracing` library, with no GPU.
`RenderScene::build` turns the solids a view shows (its visibility,
phase and design options through `cc_views`' graphics resolver, its
detail level, painted faces, material textures laid as the 3D view's
textures style lays them, smooth normals across curved facets) into the
path tracer's triangles and materials (`materials`), lights them with a
small generated sky, ground and sun (`environment`) and places the
camera the viewport has (`camera`). `run` then refines the picture a
few samples at a time, handing each step to a callback that may stop it
(an application shows it while it converges); `render_image` renders it
to the end. An `Image` holds linear radiance, which
`Image::to_rgba8` exposes, tone maps and encodes for display and
`Image::save` writes as PNG or JPEG (or linear as EXR or HDR).
The model is Z up; the path tracer is Y up (its sky's poles are on its Y
axis), so every point and direction goes through `space`: `(x, y, z)`
becomes `(x, z, -y)`, a rotation, so windings and normals are kept. The
scene is also moved so the model's center is at the origin, which keeps
the path tracer's `f32` precise for models far from the project origin.
The library may panic on input it does not expect; every call into it is
made inside `catch_unwind`, and a panic comes back as
`RenderError::Internal`. Input is checked before that: non-finite or
degenerate triangles are dropped, cameras and settings validated, and
scenes past `MAX_TRIANGLES` refused.

## Modules

| Module | Items | Summary |
|---|---|---|
| [`cc_raytracing`](#mod-cc_raytracing) | 2 | Photorealistic rendering of a 3D view: path tracing on the pure Rust `sort-raytracing` library, with no GPU. |
| [`cc_raytracing::camera`](#mod-cc_raytracing-camera) | 4 | The render's camera: the 3D view's orbit camera (`CameraState`) as a pinhole camera of the path tracer, framing the picture the viewport frames (its perspective has a vertical field of view of 45°, the default of `cc_core::RenderSettings::fov_deg`) at the render's aspect. |
| [`cc_raytracing::environment`](#mod-cc_raytracing-environment) | 15 | The light around the model: a generated sky map with the ground below the horizon, a sun, and an optional ground plane that catches shadows. |
| [`cc_raytracing::materials`](#mod-cc_raytracing-materials) | 12 | The path tracer's materials, made from the model's. |
| [`cc_raytracing::render`](#mod-cc_raytracing-render) | 7 | Rendering a `RenderScene`: progressively with a callback that sees each step and may stop the render (`run`), or to the end (`render_image`), and the image it gives (`Image`). |
| [`cc_raytracing::scene`](#mod-cc_raytracing-scene) | 2 | The scene of a render: the triangles, materials, lights and camera of a view, owned so a worker thread can render it. |
| [`cc_raytracing::space`](#mod-cc_raytracing-space) | 3 | The path tracer's frame. |

<a id="mod-cc_raytracing"></a>

## `cc_raytracing`

<sub>`crates/cc_raytracing/src/lib.rs`</sub>

**Re-exports**

- `pub use render::{Control, Image, Outcome, Progress, Region, render_image, run}`
- `pub use scene::{MAX_TRIANGLES, RenderScene}`

**Items:** [`RenderError`](#cc_raytracing-rendererror) · [`guarded`](#cc_raytracing-guarded)

<a id="cc_raytracing-rendererror"></a>

### enum `RenderError`

```rust
pub enum RenderError
```

What went wrong building or rendering a scene, or writing its image.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Error`

**Variants**

- `TooManyTriangles { limit: usize }`

  The view shows more triangles than a render takes.

- `InvalidCamera(String)`

  The camera has values that are not numbers.

- `InvalidImage(String)`

  The image or region asked for cannot be rendered or written.

- `UnsupportedFormat(String)`

  An image file of a kind that is not written.

- `Write(String)`

  Writing the image file failed.

- `Internal(String)`

  The path tracer failed (it panicked); the message says where.


<a id="cc_raytracing-guarded"></a>

### fn `guarded`

```rust
pub(crate) fn guarded<T>(what: &str, f: impl FnOnce() -> T) -> Result<T, RenderError>
```

Run `f`, turning a panic into `RenderError::Internal`.

<a id="mod-cc_raytracing-camera"></a>

## `cc_raytracing::camera`

<sub>`crates/cc_raytracing/src/camera.rs`</sub>

The render's camera: the 3D view's orbit camera (`CameraState`) as a
pinhole camera of the path tracer, framing the picture the viewport
frames (its perspective has a vertical field of view of 45°, the default
of `cc_core::RenderSettings::fov_deg`) at the render's aspect.

**Items:** [`VIEWPORT_FOV_DEG`](#cc_raytracing-camera-viewport_fov_deg) · [`eye_and_forward`](#cc_raytracing-camera-eye_and_forward) · [`tracer_camera`](#cc_raytracing-camera-tracer_camera) · [`fit`](#cc_raytracing-camera-fit)

<a id="cc_raytracing-camera-viewport_fov_deg"></a>

### const `VIEWPORT_FOV_DEG`

```rust
pub const VIEWPORT_FOV_DEG: f64
```

The field of view of the 3D viewport, degrees (vertical).

<a id="cc_raytracing-camera-eye_and_forward"></a>

### fn `eye_and_forward`

```rust
pub fn eye_and_forward(camera: &CameraState) -> Result<(DVec3, DVec3), RenderError>
```

The eye and the unit direction the camera looks along, in model
coordinates. A camera with no distance (or a negative one) stands at its
target, still looking along its yaw and pitch.

<a id="cc_raytracing-camera-tracer_camera"></a>

### fn `tracer_camera`

```rust
pub(crate) fn tracer_camera(camera: &CameraState, fov_deg: f64, width: u32, height: u32, space: &Space) -> Result<Camera, RenderError>
```

The path tracer's camera for `camera` with a vertical field of view of
`fov_deg` and an image of `width` × `height` pixels, in `space`.

<a id="cc_raytracing-camera-fit"></a>

### fn `fit`

```rust
pub fn fit(camera: &CameraState, geom: &ModelGeometry, fov_deg: f64, aspect: f64) -> CameraState
```

`camera` moved back along its direction so the whole of `geom` fits a
picture of aspect `aspect` (width over height) with a vertical field of
view of `fov_deg`, around the model's center (what the 3D view does to a
view whose camera was never moved); the camera itself when there is no
geometry.

<a id="mod-cc_raytracing-environment"></a>

## `cc_raytracing::environment`

<sub>`crates/cc_raytracing/src/environment.rs`</sub>

The light around the model: a generated sky map with the ground
below the horizon, a sun, and an optional ground plane that catches
shadows.
The sky is an equirectangular map of `ENV_WIDTH` × `ENV_HEIGHT`
pixels in linear light (about 0.35° a pixel: the path tracer reads the
nearest pixel, and coarser maps show the sky's gradient as bands), made
from the sun's altitude: deep blue at the zenith, paler towards the
horizon, warmer there towards a low sun, with a soft glow around the
sun; once the sun is below the horizon it fades through a blue-gray dusk
to night. Below
the horizon the map holds the ground color lit by the sun and the sky (a
matte ground's radiance), blended into the sky over a degree or two, so
the horizon past the ground plane reads right. The sky map is scaled by
`cc_core::RenderEnvironment::sky_strength`.
The sun is a distant light of `SUN_IRRADIANCE` at the zenith, dimmed
and warmed by the air it crosses (an air mass formula and a per-channel
extinction), none below the horizon, scaled by
`cc_core::RenderEnvironment::sun_strength`. Its disk is not drawn in
the sky map: rays bouncing off surfaces would find it and count the sun
a second time, as bright isolated pixels; the glow around it is broad
and dim enough not to.
A view with no sun is lit by `default_sun`, a sun that follows the
view: `DEFAULT_SUN_ALTITUDE` high, behind the viewer and to the left
(where the 3D view's own light comes from), so the faces seen are lit
and the shadows fall to the right.
At the default strengths, exposure 0 and the filmic curve, a white wall
facing a high sun comes out bright but not clipped, the sky light blue
and shaded faces mid gray.

**Items:** [`ENV_WIDTH`](#cc_raytracing-environment-env_width) · [`ENV_HEIGHT`](#cc_raytracing-environment-env_height) · [`SUN_IRRADIANCE`](#cc_raytracing-environment-sun_irradiance) · [`DEFAULT_SUN_ALTITUDE`](#cc_raytracing-environment-default_sun_altitude) · [`DEFAULT_SUN_AZIMUTH`](#cc_raytracing-environment-default_sun_azimuth) · [`GROUND_NUDGE`](#cc_raytracing-environment-ground_nudge) · [`MIN_GROUND_HALF_SIDE`](#cc_raytracing-environment-min_ground_half_side) · [`GROUND_RADII`](#cc_raytracing-environment-ground_radii) · [`sun_state`](#cc_raytracing-environment-sun_state) · [`default_sun`](#cc_raytracing-environment-default_sun) · [`sun_irradiance`](#cc_raytracing-environment-sun_irradiance) · [`sky_radiance`](#cc_raytracing-environment-sky_radiance) · [`sky_map`](#cc_raytracing-environment-sky_map) · [`lights`](#cc_raytracing-environment-lights) · [`ground_height`](#cc_raytracing-environment-ground_height)

<a id="cc_raytracing-environment-env_width"></a>

### const `ENV_WIDTH`

```rust
pub const ENV_WIDTH: usize
```

Width of the sky map, pixels (longitude).

<a id="cc_raytracing-environment-env_height"></a>

### const `ENV_HEIGHT`

```rust
pub const ENV_HEIGHT: usize
```

Height of the sky map, pixels (latitude, zenith to nadir).

<a id="cc_raytracing-environment-sun_irradiance"></a>

### const `SUN_IRRADIANCE`

```rust
pub const SUN_IRRADIANCE: f64
```

Irradiance of the sun at the zenith on a surface facing it, before the
air dims it (in the units of the sky map's radiance).

<a id="cc_raytracing-environment-default_sun_altitude"></a>

### const `DEFAULT_SUN_ALTITUDE`

```rust
pub const DEFAULT_SUN_ALTITUDE: f64
```

Altitude of the sun of a view with none, degrees.

<a id="cc_raytracing-environment-default_sun_azimuth"></a>

### const `DEFAULT_SUN_AZIMUTH`

```rust
pub const DEFAULT_SUN_AZIMUTH: f64
```

Where the sun of a view with none stands, degrees clockwise from the
direction the viewer looks: behind and to the left.

<a id="cc_raytracing-environment-ground_nudge"></a>

### const `GROUND_NUDGE`

```rust
pub const GROUND_NUDGE: f64
```

The ground plane is this far below the ground height, meters, so faces
lying at that height (floor slabs) cover it.

<a id="cc_raytracing-environment-min_ground_half_side"></a>

### const `MIN_GROUND_HALF_SIDE`

```rust
pub const MIN_GROUND_HALF_SIDE: f64
```

Smallest half side of the ground plane, meters.

<a id="cc_raytracing-environment-ground_radii"></a>

### const `GROUND_RADII`

```rust
pub const GROUND_RADII: f64
```

Half side of the ground plane in model radii (around the model's center).

<a id="cc_raytracing-environment-sun_state"></a>

### fn `sun_state`

```rust
pub fn sun_state(project: &Project, view: &View, camera: &CameraState) -> SunState
```

The sun lighting `view` through `camera`: the view's own sun (a view
sun turning with the view follows `camera`), else `default_sun`.

<a id="cc_raytracing-environment-default_sun"></a>

### fn `default_sun`

```rust
pub fn default_sun() -> SunSettings
```

The sun of a view that has none: `DEFAULT_SUN_ALTITUDE` high, at
`DEFAULT_SUN_AZIMUTH` from the direction the viewer looks.

<a id="cc_raytracing-environment-sun_irradiance"></a>

### fn `sun_irradiance`

```rust
pub fn sun_irradiance(sun: &SunState) -> DVec3
```

The sun's irradiance (linear RGB) at strength 1, on a surface facing it.

<a id="cc_raytracing-environment-sky_radiance"></a>

### fn `sky_radiance`

```rust
pub fn sky_radiance(d: DVec3, sun: &SunState) -> DVec3
```

The sky's radiance (linear RGB) at strength 1 along unit direction `d`
(model coordinates, at or above the horizon) under `sun`.

<a id="cc_raytracing-environment-sky_map"></a>

### fn `sky_map`

```rust
pub fn sky_map(sun: &SunState, environment: &RenderEnvironment) -> EnvMap
```

The sky map of `sun` and `environment` (in the tracer's frame), with the
ground of `environment`'s color below the horizon.

<a id="cc_raytracing-environment-lights"></a>

### fn `lights`

```rust
pub fn lights(sun: &SunState, environment: &RenderEnvironment) -> Vec<Light>
```

The lights of a render under `sun`: the sun (when it is above the horizon
and has strength) and the sky.

<a id="cc_raytracing-environment-ground_height"></a>

### fn `ground_height`

```rust
pub fn ground_height(project: &Project, view: &View, bottom: Option<f64>) -> f64
```

The height of the ground plane: the view's sun's ground level, else the
lowest level, else the bottom of the model (`bottom`), else 0 (the rule
the 3D view and the 2D views' shadows share,
`Project::sun_ground_height`).

<a id="mod-cc_raytracing-materials"></a>

## `cc_raytracing::materials`

<sub>`crates/cc_raytracing/src/materials.rs`</sub>

The path tracer's materials, made from the model's.
Each face is shaded by a `Surface`: what the 3D view's textures style
shows of its material (or of the paint on it), after the view's graphics
(a color override, halftone, transparency). Its appearance color is the
base color, its `Finish` sets how rough and shiny it is (table in
`Surface::material`), metals (material class `Metal`) are metallic, and
its texture, decoded once per image (`Textures`), replaces the color.
Glass is a material of class `Glass`, or one at least
`GLASS_TRANSPARENCY` transparent: it becomes thin glass (light goes
straight through, the pane reflects like a dielectric and shadow rays
cross it), so the sun enters rooms through windows. Masses, which the
model always draws see-through, and any other transparent material keep
their transparency as partial coverage instead.

**Items:** [`GLASS_TRANSPARENCY`](#cc_raytracing-materials-glass_transparency) · [`GLASS_TINT`](#cc_raytracing-materials-glass_tint) · [`MAX_TEXTURES`](#cc_raytracing-materials-max_textures) · [`MAX_TEXTURE_SIDE`](#cc_raytracing-materials-max_texture_side) · [`CLAY_COLOR`](#cc_raytracing-materials-clay_color) · [`Surface`](#cc_raytracing-materials-surface) · [`Style`](#cc_raytracing-materials-style) · [`Textures`](#cc_raytracing-materials-textures) · [`srgb_to_linear`](#cc_raytracing-materials-srgb_to_linear) · [`linear`](#cc_raytracing-materials-linear) · [`SurfaceInput`](#cc_raytracing-materials-surfaceinput) · [`texture_uv`](#cc_raytracing-materials-texture_uv)

<a id="cc_raytracing-materials-glass_transparency"></a>

### const `GLASS_TRANSPARENCY`

```rust
pub const GLASS_TRANSPARENCY: f64
```

A material at least this transparent (0 to 1) that is not of class glass
renders as glass all the same.

<a id="cc_raytracing-materials-glass_tint"></a>

### const `GLASS_TINT`

```rust
pub const GLASS_TINT: f32
```

How much of a glass material's color tints the light through it: clear
panes read as their color in the views, but a pane that absorbed that
much would darken a room.

<a id="cc_raytracing-materials-max_textures"></a>

### const `MAX_TEXTURES`

```rust
pub const MAX_TEXTURES: usize
```

The most textures a render decodes; the materials past them show their color.

<a id="cc_raytracing-materials-max_texture_side"></a>

### const `MAX_TEXTURE_SIDE`

```rust
pub const MAX_TEXTURE_SIDE: u32
```

The longest side of a decoded texture, pixels (larger images are reduced).

<a id="cc_raytracing-materials-clay_color"></a>

### const `CLAY_COLOR`

```rust
pub const CLAY_COLOR: [u8; 3]
```

The base color of clay renders (sRGB): a white matte of about 0.8 in
linear light, lit by the sky and the sun like any other surface.

<a id="cc_raytracing-materials-surface"></a>

### struct `Surface`

```rust
pub struct Surface
```

How a face is shaded: the recipe of one path tracer material.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`

**Fields**

- `color: [u8; 3]`

  Base color, sRGB.

- `finish: Finish`
- `metal: bool`
- `glass: bool`

  Thin glass.

- `opacity: u16`

  How much of each point is there, in thousandths (1000: opaque).

- `texture: Option<usize>`

  The texture replacing the color, an index into `Textures`.


**Methods**

- `pub fn of(project: &cc_core::Project, input: &SurfaceInput, style: &Style, clay: bool, textures: &mut Textures) -> Self`

  The surface of a face with `input`, in an element of view graphics
  `style`; `clay` makes every opaque surface `CLAY_COLOR` and matte,
  and glass clear.

- `pub fn matte(color: [u8; 3]) -> Self`

  A matte surface of `color` (the ground).

- `pub fn material(&self, textures: &Textures) -> Material`

  The path tracer's material. Finishes, as roughness / specular
  (0.5 is a reflectance of 4 % head on) / clear coat:
  | finish   | dielectric          | metal (metallic 1) |
  |----------|---------------------|--------------------|
  | matte    | 0.90 / 0.25 / none  | roughness 0.65     |
  | satin    | 0.55 / 0.50 / none  | roughness 0.40     |
  | glossy   | 0.30 / 0.50 / 0.4   | roughness 0.22     |
  | polished | 0.12 / 0.50 / 1.0   | roughness 0.06     |
  Glass is thin, fully transmissive, smooth, of index 1.5, tinted by
  `GLASS_TINT` of its color. A texture multiplies a white base.


<a id="cc_raytracing-materials-style"></a>

### struct `Style`

```rust
pub struct Style
```

The view graphics of an element's solids, as the 3D view applies them.

**Traits:** `Debug`, `Clone`, `Copy`, `Default`, `PartialEq`

**Fields**

- `color: Option<[u8; 3]>`
- `transparency: Option<f64>`

  0 (opaque) to 1.

- `halftone: bool`

  Faces mixed with white.


**Methods**

- `pub fn of(g: &cc_core::GraphicOverride) -> Self`

  How a resolved graphic override shows on surfaces (as the 3D view
  reads it): the surface fill's color (white for a fill of no pattern),
  transparency and halftone.


<a id="cc_raytracing-materials-textures"></a>

### struct `Textures`

```rust
pub struct Textures
```

The decoded textures of a render, one per image.

**Traits:** `Default`

*Some fields are private.*

**Methods**

- `pub fn index(&mut self, texture: &MaterialTexture) -> Option<usize>`

  The index of `texture`'s image, decoded on first use; `None` when it
  cannot be decoded or `MAX_TEXTURES` are taken.

- `pub fn get(&self, index: usize) -> Option<&Arc<Texture>>`

  The decoded texture `index`.

- `pub fn len(&self) -> usize`
- `pub fn is_empty(&self) -> bool`

<a id="cc_raytracing-materials-srgb_to_linear"></a>

### fn `srgb_to_linear`

```rust
pub fn srgb_to_linear(c: u8) -> f32
```

An sRGB channel as linear light.

<a id="cc_raytracing-materials-linear"></a>

### fn `linear`

```rust
pub fn linear(c: [u8; 3]) -> Vec3
```

An sRGB color as linear light.

<a id="cc_raytracing-materials-surfaceinput"></a>

### struct `SurfaceInput`

```rust
pub struct SurfaceInput<'a>
```

What a face of a material shows, as the 3D view's textures style shows it.

**Fields**

- `material: Option<&'a str>`

  The material's name (the solid's own, or the paint's).

- `appearance: [u8; 3]`

  The appearance color (when the project has no such material).

- `transparency: f64`

  The transparency the model gives the face (0 to 1).

- `mass: bool`

  A mass: kept see-through, never glass.


<a id="cc_raytracing-materials-texture_uv"></a>

### fn `texture_uv`

```rust
pub fn texture_uv(texture: &MaterialTexture, uv: [f64; 2]) -> [f64; 2]
```

The texture coordinates of a point with plane coordinates `uv` (meters,
see `cc_geom::plane_axes`) for `texture`, as the 3D view lays them: in
repeats, rows running down the image so its up is the face's up.

<a id="mod-cc_raytracing-render"></a>

## `cc_raytracing::render`

<sub>`crates/cc_raytracing/src/render.rs`</sub>

Rendering a `RenderScene`: progressively with a callback that sees
each step and may stop the render (`run`), or to the end
(`render_image`), and the image it gives (`Image`).

**Items:** [`Region`](#cc_raytracing-render-region) · [`Control`](#cc_raytracing-render-control) · [`Progress`](#cc_raytracing-render-progress) · [`Outcome`](#cc_raytracing-render-outcome) · [`run`](#cc_raytracing-render-run) · [`render_image`](#cc_raytracing-render-render_image) · [`Image`](#cc_raytracing-render-image)

<a id="cc_raytracing-render-region"></a>

### struct `Region`

```rust
pub struct Region
```

A rectangle of the image, in pixels from its top left corner.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Fields**

- `x: u32`
- `y: u32`
- `width: u32`
- `height: u32`

<a id="cc_raytracing-render-control"></a>

### enum `Control`

```rust
pub enum Control
```

What the callback of `run` says after a step.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Variants**

- `Continue`

  Render more samples.

- `Stop`

  Stop: `run` returns the image so far.


<a id="cc_raytracing-render-progress"></a>

### struct `Progress`

```rust
pub struct Progress<'a>
```

A render in progress, as the callback of `run` sees it after a step.

*Some fields are private.*

**Methods**

- `pub fn samples(&self) -> u32`

  Samples per pixel rendered so far.

- `pub fn total_samples(&self) -> u32`

  Samples per pixel of the finished render.

- `pub fn is_done(&self) -> bool`
- `pub fn elapsed(&self) -> Duration`

  Time spent rendering so far.

- `pub fn region(&self) -> Region`

  The part of the image being rendered (all of it unless a region was given).

- `pub fn image(&self) -> Image`

  The whole image so far, black outside the region.

- `pub fn region_image(&self) -> Image`

  The region alone so far, as an image of its size.


<a id="cc_raytracing-render-outcome"></a>

### struct `Outcome`

```rust
pub struct Outcome
```

How a `run` ended.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `image: Image`

  The whole image (black outside the region, if one was given).

- `samples: u32`

  Samples per pixel rendered.

- `stopped: bool`

  The callback stopped the render before the last sample.


<a id="cc_raytracing-render-run"></a>

### fn `run`

```rust
pub fn run(scene: &RenderScene, region: Option<Region>, mut on_step: impl FnMut(&Progress<'_>) -> Control) -> Result<Outcome, RenderError>
```

Render `scene` (the `region` of its image only, when given: the rest is
never traced and stays black), a few samples per pixel at a time, calling
`on_step` after each step until the render is done or the callback says
`Control::Stop`. The first steps add one sample, so the first picture
comes fast; later ones grow so a step takes about a quarter of a second.
Blocking: an application runs it on a thread of its own and stops it
from the callback (an atomic flag the callback reads). A panic of the
path tracer comes back as `RenderError::Internal`.

<a id="cc_raytracing-render-render_image"></a>

### fn `render_image`

```rust
pub fn render_image(scene: &RenderScene) -> Result<Image, RenderError>
```

Render all of `scene` to the last sample.

<a id="cc_raytracing-render-image"></a>

### struct `Image`

```rust
pub struct Image
```

An image of linear radiance (RGB), rows from the top.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `width: u32`
- `height: u32`
- `pixels: Vec<[f32; 3]>`

**Methods**

- `pub fn is_valid(&self) -> bool`

  Whether the pixels are as many as the size says (and there are some).

- `pub fn to_rgba8(&self, exposure_stops: f64, tone_map: ToneMap) -> Vec<u8>`

  The pixels multiplied by `2^exposure_stops`, tone mapped and sRGB
  encoded, as RGBA bytes (alpha 255), 4 per pixel. Values that are not
  numbers come out black.

- `pub fn save(&self, path: &Path, exposure_stops: f64, tone_map: ToneMap) -> Result<(), RenderError>`

  Write the image to `path`: `.png`, `.jpg` and `.jpeg` exposed, tone
  mapped and sRGB encoded (as `Image::to_rgba8`); `.exr` and `.hdr`
  in linear light, exposed but not tone mapped.


<a id="mod-cc_raytracing-scene"></a>

## `cc_raytracing::scene`

<sub>`crates/cc_raytracing/src/scene.rs`</sub>

The scene of a render: the triangles, materials, lights and camera of a
view, owned so a worker thread can render it.

**Items:** [`MAX_TRIANGLES`](#cc_raytracing-scene-max_triangles) · [`RenderScene`](#cc_raytracing-scene-renderscene)

<a id="cc_raytracing-scene-max_triangles"></a>

### const `MAX_TRIANGLES`

```rust
pub const MAX_TRIANGLES: usize
```

The most triangles a render takes (about a gigabyte of scene); a view
showing more is refused.

<a id="cc_raytracing-scene-renderscene"></a>

### struct `RenderScene`

```rust
pub struct RenderScene
```

A view ready to render: the path tracer's scene (geometry, materials,
lights), its camera and how it is sampled, plus the exposure and tone
mapping its image is shown with. It owns everything (`Send` and `Sync`),
so a worker thread can take it (or an `Arc` of it) and render it with
`crate::run`.

**Fields**

- `scene: Scene`
- `camera: Camera`
- `tracer: TracerSettings`

*Some fields are private.*

**Methods**

- `pub fn build(project: &Project, geom: &ModelGeometry, view: &View, camera: &CameraState, settings: &RenderSettings, temporary: Option<&TemporaryVisibility>) -> Result<Self, RenderError>`

  The scene of `view` seen through `camera` (the 3D view's camera: the
  one in the view, or the viewport's live one) with `settings` (brought
  into range first). `geom` is the geometry the view shows (its phase
  and design options, `Session::view_geometry`), and `temporary` what is
  temporarily hidden or isolated in it: the elements the view's graphics
  hide are left out, its overrides of color, transparency and halftone
  shown as in its textures style. Errors for a camera that is not a
  number and a scene of more than `MAX_TRIANGLES`; an empty view
  gives the sky and the ground.

- `pub fn width(&self) -> u32`

  Width of the image, pixels.

- `pub fn height(&self) -> u32`

  Height of the image, pixels.

- `pub fn samples(&self) -> u32`

  Samples per pixel of the finished render.

- `pub fn exposure(&self) -> f64`

  The exposure the image is shown with, stops.

- `pub fn tone_map(&self) -> ToneMap`

  The tone mapping the image is shown with.

- `pub fn triangles(&self) -> usize`

  Triangles of the model in the scene (the ground plane not counted).

- `pub fn textures(&self) -> usize`

  Material textures decoded for the scene.


<a id="mod-cc_raytracing-space"></a>

## `cc_raytracing::space`

<sub>`crates/cc_raytracing/src/space.rs`</sub>

The path tracer's frame. The model is Z up, in meters, in `f64`; the
path tracer is Y up (its sky map has its poles on Y) and works in `f32`.
A model point `p` becomes `p - origin` turned by `(x, y, z) → (x, z, -y)`:
a rotation of -90° about X, so the handedness, triangle windings and
normals are kept, model up (+Z) is the tracer's up (+Y) and model north
(+Y) is the tracer's -Z.

**Items:** [`Space`](#cc_raytracing-space-space) · [`dir`](#cc_raytracing-space-dir) · [`model_dir`](#cc_raytracing-space-model_dir)

<a id="cc_raytracing-space-space"></a>

### struct `Space`

```rust
pub struct Space
```

Model coordinates to the path tracer's, around a chosen origin.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `origin: DVec3`

  The model point that is the tracer's origin.


**Methods**

- `pub fn new(origin: DVec3) -> Self`
- `pub fn point(&self, p: DVec3) -> Vec3`

  A model point in the tracer's frame.

- `pub fn model_point(&self, p: Vec3) -> DVec3`

  A tracer point back in model coordinates.


<a id="cc_raytracing-space-dir"></a>

### fn `dir`

```rust
pub fn dir(d: DVec3) -> Vec3
```

A model direction (or any vector) in the tracer's frame.

<a id="cc_raytracing-space-model_dir"></a>

### fn `model_dir`

```rust
pub fn model_dir(d: Vec3) -> DVec3
```

A tracer direction back in model coordinates.

{% endraw %}
