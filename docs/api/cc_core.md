# `cc_core` API

{% raw %}
[← API reference](index.md) · Generated from the source by `cargo run -p api_doc`; edit the doc comments, not this page.

Core BIM data model for ConstructCraft.
The `Project` is the single source of truth. Every view, schedule and
sheet is a *query* over it; all geometry is derived. Edits go through
`Document::edit`, which records a `Transaction` for undo/redo.

## Modules

| Module | Items | Summary |
|---|---|---|
| [`cc_core`](#mod-cc_core) | 0 | Core BIM data model for ConstructCraft. |
| [`cc_core::arc`](#mod-cc_core-arc) | 4 | Circular arcs in plan, described like polyline arcs by their two ends and a *bulge*: `tan(sweep / 4)`, positive when the arc turns counter-clockwise from the start to the end, 0 for a straight line, ±1 for a half circle. |
| [`cc_core::bar_marks`](#mod-cc_core-bar_marks) | 20 | Bar marks: the names (N1, N2…) bending schedules and bar tags give the bars of reinforcement, one per bar shape: the same diameter, steel grade and shape (legs, radii, bends and hooks, within a length tolerance and `ANGLE_TOLERANCE_DEG`, read from either end or mirrored). |
| [`cc_core::bar_tag`](#mod-cc_core-bar_tag) | 5 | Bar tags: annotations that write a reinforcement set's bars in a plan, an elevation or a section — `25 N1 Ø12.5 c/12 C=500`: the count, the mark, the diameter in millimeters, the spacing in centimeters (sets without one leave it out) and the length of one bar in centimeters (`C=var` when the set's bars differ). |
| [`cc_core::bracing`](#mod-cc_core-bracing) | 11 | Parametric bracing: a host element that holds the parameters of the braces of one bay of a frame (the bay line between two column axes, the storey height, the number of storeys, the pattern and the brace type) and owns the ordinary beams made from them, kept in step like a truss's (see `crate::member_host`). |
| [`cc_core::browser`](#mod-cc_core-browser) | 13 | Project browser organization: named schemes that group the views of the project browser by view type, level, discipline or phase, in that order of nesting. |
| [`cc_core::cad`](#mod-cc_core-cad) | 20 | Imported CAD drawings (DWG and DXF underlays). |
| [`cc_core::callout`](#mod-cc_core-callout) | 5 | Crop regions and callouts. |
| [`cc_core::component`](#mod-cc_core-component) | 25 | Parametric components: user-defined objects (furniture, fixtures, casework…) whose solids are sized by named parameters. |
| [`cc_core::component_file`](#mod-cc_core-component_file) | 14 | `.ccomp` component files: a component definition with all its types and the materials it uses, to share components between projects. |
| [`cc_core::component_ref`](#mod-cc_core-component_ref) | 13 | Reference planes, reference lines and labeled dimensions of a component: the skeleton that parameters drive and pieces follow. |
| [`cc_core::component_script`](#mod-cc_core-component_script) | 15 | Scripted component pieces: the model side. |
| [`cc_core::connection`](#mod-cc_core-connection) | 7 | Steel and timber connections: an element at one end of a column or beam (`member`, `end`) of a standard kind, whose plates and bolts `cc_geom::connection` derives from the member's geometry on every build. |
| [`cc_core::curtain`](#mod-cc_core-curtain) | 20 | Curtain walls: walls made of a grid of panels and mullions. |
| [`cc_core::design_option`](#mod-cc_core-design_option) | 4 | Design options: alternative sets of elements within one project. |
| [`cc_core::detail`](#mod-cc_core-detail) | 9 | Detail lines: view-specific 2D curves (lines, arcs, circles and whole or partial ellipses, the `SketchCurve` geometry) drawn in one view with a line style. |
| [`cc_core::detail_component`](#mod-cc_core-detail_component) | 14 | Detail components: reusable 2D components for construction details. |
| [`cc_core::dim_line`](#mod-cc_core-dim_line) | 7 | The dimension line of linear and aligned dimensions: lining it up with the lines of other dimensions, and chains of dimensions that share one line (continuous dimensioning). |
| [`cc_core::dim_refs`](#mod-cc_core-dim_refs) | 5 | Associative dimensions: each end of a dimension may reference a point of an element (stored in that element's own frame, plus a height in elevations and sections) or a level. |
| [`cc_core::dim_style`](#mod-cc_core-dim_style) | 4 | How a dimension type draws its dimensions: the marks at the ends of the dimension line, the witness lines, the pens and the text around the value. |
| [`cc_core::document`](#mod-cc_core-document) | 3 |  |
| [`cc_core::element`](#mod-cc_core-element) | 36 |  |
| [`cc_core::element::slab_bars`](#mod-cc_core-element-slab_bars) | 3 | Limits of slab reinforcement. |
| [`cc_core::element::wall_bars`](#mod-cc_core-element-wall_bars) | 5 | Limits of wall reinforcement. |
| [`cc_core::expr`](#mod-cc_core-expr) | 6 | Arithmetic expressions over named parameters, used by component geometry: numbers, parameter names, `+ - * / ^`, parentheses, unary minus and the functions `min`, `max`, `abs`, `sqrt`, `round`, `floor`, `ceil`, `sin`, `cos`, `tan` (degrees). |
| [`cc_core::fill_pattern`](#mod-cc_core-fill_pattern) | 22 | The project's own fill patterns (hatches): families of parallel, possibly dashed lines, as in the plain-text `.pat` format CAD programs share (see `crate::pat`). |
| [`cc_core::graph_binding`](#mod-cc_core-graph_binding) | 2 | Elements made by visual programming graphs. |
| [`cc_core::graphics`](#mod-cc_core-graphics) | 14 | View graphics: what a view shows and how it draws it. |
| [`cc_core::grid`](#mod-cc_core-grid) | 8 | Grid naming and alignment. |
| [`cc_core::ids`](#mod-cc_core-ids) | 1 |  |
| [`cc_core::level_mark`](#mod-cc_core-level_mark) | 5 | Level marks: annotations that write the elevation of a point in a plan, an elevation or a section — the finished level above a shelf line and the raw (structural) level under it. |
| [`cc_core::lightning`](#mod-cc_core-lightning) | 8 | Lightning protection: the conductors of a lightning protection system (on the roof, down the walls and in the ground) and the protection classes the coverage check uses. |
| [`cc_core::mass`](#mod-cc_core-mass) | 7 | Conceptual masses: volumes for massing studies. |
| [`cc_core::material`](#mod-cc_core-material) | 16 | Materials: identity data, graphics in views (shading, transparency, surface and cut patterns), render appearance and physical properties. |
| [`cc_core::member_host`](#mod-cc_core-member_host) | 5 | Member hosts: elements with no geometry of their own (steel buildings, trusses, bracings, wall framings) that own ordinary columns and beams laid out from their parameters. |
| [`cc_core::mep`](#mod-cc_core-mep) | 16 | Building services (MEP): plumbing, electrical, structured cabling and lightning protection systems, the pipe, conduit and cable tray segments that run them, the valves on pipes, and the data that makes a component a plumbing fixture or an electrical, communications or lightning protection device. |
| [`cc_core::openings`](#mod-cc_core-openings) | 11 | Doors and windows as components: the built-in definitions that the template's door and window types use and that older files' door and window types become. |
| [`cc_core::pad`](#mod-cc_core-pad) | 8 | Building pads: a flat area at an elevation, graded into the terrain that hosts it, with cut and fill slopes running out from its edges to the existing ground. |
| [`cc_core::paint`](#mod-cc_core-paint) | 4 | Painted faces: a material laid on one surface of an element, over the material its type gives it (a painted wall face, a tiled floor top). |
| [`cc_core::pat`](#mod-cc_core-pat) | 10 | The plain-text `.pat` hatch pattern format many CAD programs read and write, as publicly described: ```text ;%UNITS=MM *NAME, optional description ;%TYPE=MODEL angle, x-origin, y-origin, delta-x, delta-y [, dash1, dash2, ...] ```rust A header line starting with `*` opens a pattern; each following line is a family of parallel lines through the origin at the angle (degrees), the next line shifted by delta-x along it and delta-y across it, with optional dashes (positive drawn, negative gaps, zero dots). |
| [`cc_core::persist`](#mod-cc_core-persist) | 12 | `.ccad` project files: a versioned RON document. |
| [`cc_core::phase`](#mod-cc_core-phase) | 10 | Construction phases (renovation work). |
| [`cc_core::pile`](#mod-cc_core-pile) | 29 | Piles and pile caps. |
| [`cc_core::profile`](#mod-cc_core-profile) | 14 | Profiles of component pieces: closed outlines of straight segments, arcs and parts of ellipses drawn on one of the component's planes (plan, front or side), with every coordinate an `crate::expr` expression, and the solids made from them: an extrusion across the plane or a revolution about an axis in it. |
| [`cc_core::profile_catalog`](#mod-cc_core-profile_catalog) | 6 | Built-in profile catalogs: the standard series of rolled, cold-formed, timber and hollow sections, as tables (cells split by `;`, since the designations write decimals with a comma) compiled into the binary and parsed once. |
| [`cc_core::profile_sweep`](#mod-cc_core-profile_sweep) | 7 | Swept profiles: a closed profile, held by the element's type, carried along a path drawn in plan at a height above the element's level. |
| [`cc_core::profiles_project`](#mod-cc_core-profiles_project) | 7 | Structural profiles in the project: lookups and repair of loaded files. |
| [`cc_core::project`](#mod-cc_core-project) | 9 |  |
| [`cc_core::properties`](#mod-cc_core-properties) | 6 | Property and quantity sets carried over from IFC files: named sets of named values, each value keeping its IFC measure type. |
| [`cc_core::ramp`](#mod-cc_core-ramp) | 7 | Ramps: the plan shapes of a ramp element and the accessibility rules of its type (steepest slope, highest rise of one flight, shortest landing). |
| [`cc_core::raster`](#mod-cc_core-raster) | 6 | Raster images kept in a project (imported pictures, scanned drawings): the file's own bytes, unchanged, with the format and pixel size. |
| [`cc_core::recovery`](#mod-cc_core-recovery) | 9 | Crash recovery: while a project has unsaved changes, a copy of it is written now and then to a recovery folder, so a crash (or a power cut) loses at most the last few minutes of work. |
| [`cc_core::region`](#mod-cc_core-region) | 3 | Regions: 2D areas drawn in one view, owned by it like detail lines and text notes (`Element::owner_view`), in the view's drawing coordinates (meters). |
| [`cc_core::render_settings`](#mod-cc_core-render_settings) | 28 | Per-view settings of photorealistic rendering (path tracing). |
| [`cc_core::repair`](#mod-cc_core-repair) | 0 | Repair of projects read from files, which may be damaged or crafted: invalid numbers and dangling references are removed or reset so the rest of the program can rely on a consistent model. |
| [`cc_core::roofing`](#mod-cc_core-roofing) | 5 | Roof coverings and framing. |
| [`cc_core::scale`](#mod-cc_core-scale) | 4 | Drawing scales as people read them: `1:50` in metric projects, the customary `1/4" = 1'-0"` in imperial ones. |
| [`cc_core::schedule`](#mod-cc_core-schedule) | 15 | Schedule definitions: which fields a schedule shows, how rows are filtered, sorted and grouped, how each column is formatted and how the table looks on paper. |
| [`cc_core::section_props`](#mod-cc_core-section_props) | 1 | Geometric properties of a section outline (area, centroid, second moments, moduli, radii of gyration, torsion constant, mass per length), computed exactly for the polygon with Green's theorem. |
| [`cc_core::shaft`](#mod-cc_core-shaft) | 3 | Shaft openings: a vertical opening with an outline in plan, from its level (plus `base_offset`) up to a level or a height, that cuts every floor, ceiling and roof it passes through. |
| [`cc_core::site`](#mod-cc_core-site) | 7 | Where the project stands on Earth: the geographic position of the project origin, its height above sea level, the direction of true north and the parts of its postal address, plus the conversion between latitude/longitude and the project's own coordinates (meters) used to bring map data in. |
| [`cc_core::sketch`](#mod-cc_core-sketch) | 7 | Sketches: the curves an outline is drawn with (lines, arcs, circles and ellipses; rectangles and polygons are drawn as lines). |
| [`cc_core::sketch_edit`](#mod-cc_core-sketch_edit) | 13 | Editing the curves of a sketch: where two curves (or the lines, circles and ellipses they lie on) cross, a curve trimmed or extended to a point, split in two, and offset alone or along the chain of curves joined to it. |
| [`cc_core::spline`](#mod-cc_core-spline) | 21 | Splines of sketches: a chain of cubic Bézier segments, kept as their control points `p0 c1 c2 p1 c1 c2 p2 …` (`3n + 1` points for `n` segments). |
| [`cc_core::stair`](#mod-cc_core-stair) | 19 | Stairs: the plan shapes of a stair element, how its type is built (solid, cast in place, assembled) and the rules that size its steps (riser limits and Blondel's formula `2 × riser + tread`). |
| [`cc_core::steel_building`](#mod-cc_core-steel_building) | 22 | Parametric steel buildings: a host element that holds the parameters of a steel shed (span, eave height, roof slope and shape, bays, the types of its members, purlins and girts) and owns ordinary columns and beams made from them. |
| [`cc_core::structural_material`](#mod-cc_core-structural_material) | 1 | The structural material of a column or beam type: concrete, steel (rolled or cold-formed) or timber. |
| [`cc_core::structural_profile`](#mod-cc_core-structural_profile) | 16 | Structural profiles: the cross-sections of columns, beams and framing members as named project objects (`Project::profiles`). |
| [`cc_core::sun`](#mod-cc_core-sun) | 32 | The sun of a view: where it stands in the sky for a date, a time and the project's site, and the settings each view keeps to place it. |
| [`cc_core::sweep`](#mod-cc_core-sweep) | 8 | Sweeps and blends of component pieces. |
| [`cc_core::template_spec`](#mod-cc_core-template_spec) | 6 | The sizes and names that differ between the metric and the imperial templates. |
| [`cc_core::title_block`](#mod-cc_core-title_block) | 28 | Title blocks: components of kind `ComponentKind::TitleBlock` whose 2D pieces (`ComponentDef::sheet`) draw a whole sheet (border, title block, logo) in paper millimeters, origin at the paper's bottom-left corner. |
| [`cc_core::topography`](#mod-cc_core-topography) | 11 | Terrain: a topography surface from measured points (x, y, z) inside a closed boundary drawn in plan. |
| [`cc_core::transform`](#mod-cc_core-transform) | 1 | Rigid transforms in plan (move, rotate, mirror) applied to elements. |
| [`cc_core::truss`](#mod-cc_core-truss) | 21 | Parametric trusses: a host element that holds the parameters of a plane truss (its span line, shape, height, web pattern, panels, member types and copies) and owns the ordinary beams (chords, diagonals) and columns (verticals) made from them, kept in step like a steel building's (see `crate::member_host`). |
| [`cc_core::types`](#mod-cc_core-types) | 16 |  |
| [`cc_core::units`](#mod-cc_core-units) | 13 | Project units: how lengths, areas, volumes, angles, slopes and money are displayed and typed. |
| [`cc_core::view`](#mod-cc_core-view) | 18 |  |
| [`cc_core::wall_framing`](#mod-cc_core-wall_framing) | 12 | Light steel and wood framing of walls: a host element attached to one straight wall that owns ordinary columns (studs, king and jack studs, cripples) and beams (tracks or plates, headers, sills, blocking) laid out from its parameters, its wall and the doors and windows the wall hosts. |
| [`cc_core::wall_ops`](#mod-cc_core-wall_ops) | 7 | Geometry edits of linear elements (walls, beams, grids): trim or extend to a corner or to a reference line, split, offset and align. |

<a id="mod-cc_core"></a>

## `cc_core`

<sub>`crates/cc_core/src/lib.rs`</sub>

**Re-exports**

- `pub use template_spec::Defaults`
- `pub use cad::{CadColorMode, CadDrawing, CadImport, CadLayer, CadPrim, CadUnit}`
- `pub use callout::{Callout, CropBox}`
- `pub use component::{ ComponentDef, ComponentItem, ComponentKind, ComponentParam, ComponentShape, DoorOperation, ParamKind, ResolveCtx, SymbolLine, WindowOperation, }`
- `pub use component_ref::{PlaneAxis, RefDimension, RefEqual, RefLine, RefPlane}`
- `pub use component_script::{Hotspot, HotspotShape, ParamChoice, ScriptEval, ScriptInput, ScriptOwner, ScriptResult}`
- `pub use connection::{Connection, ConnectionKind, ConnectionParams, MemberEnd}`
- `pub use design_option::{DesignOption, OptionSet}`
- `pub use detail::{DetailLine, DetailStyle, LineStyleDef, LineStyleRef}`
- `pub use detail_component::DetailComponent`
- `pub use dim_refs::{DimRef, Projection, RefAnchor, RefHeight, RefTarget}`
- `pub use document::{Document, EditCtx, Transaction}`
- `pub use element::{ BarEnd, BarFace, Beam, BeamLateral, BeamVertical, Ceiling, Column, Component, Dimension, DimensionKind, Element, ElementKind, FaceBars, Floor, Foundation, FoundationShape, Grid, Hosted, Leader, LeaderSymbol, LocationLine, Railing, Ramp, Rebar, RebarLayout, Roof, RoofShape, Room, ShapedRebar, SpacingZone, Splice, Stair, TextNote, Wall, WallSide, WallTop, slab_bars, wall_bars, }`
- `pub use fill_pattern::{FillPattern, FillPatternKind, PatternLine}`
- `pub use graphics::{ CategoryGraphics, FillGraphics, FilterOp, FilterParam, FilterRule, FilterUse, GraphicOverride, LineGraphics, LinePattern, TemplateIncludes, TemporaryVisibility, ViewFilter, ViewGraphics, ViewTemplate, }`
- `pub use ids::{ BrowserSchemeId, ElementId, ElementTypeId, FillPatternId, FilterId, GroupId, LevelId, LineStyleId, OptionId, OptionSetId, PhaseFilterId, PhaseId, ProfileId, SystemId, TemplateId, Uid, ViewId, }`
- `pub use lightning::{ConductorRole, LightningConductor, LpsClass}`
- `pub use material::{Material, MaterialClass, Pattern, PatternFill}`
- `pub use mep::{DeviceKind, MepData, MepSegment, MepSystem, SystemClass, SystemKind, TrayStyle, Valve, ValveKind}`
- `pub use persist::{ FORMAT_VERSION, PersistError, from_ron, from_ron_with_repairs, from_ron_with_repairs_localized, load, load_with_repairs, load_with_repairs_localized, read_to_string_limited, save, to_ron, }`
- `pub use phase::{Phase, PhaseDisplay, PhaseFilter, PhaseGraphics, PhaseState, PhaseStatus}`
- `pub use profile::{ProfilePoint, SketchPlane}`
- `pub use project::{ Group, Level, Project, ProjectField, ProjectInfo, can_join_geometry, join_priority, template_default, }`
- `pub use ramp::{RampCheck, RampIssue, RampLayout, RampRules, RampShape, ramp_flights}`
- `pub use region::Region`
- `pub use render_settings::{RenderEnvironment, RenderSettings, ToneMap}`
- `pub use schedule::{ BendingOptions, ColumnAlign, ColumnFormat, ScheduleAppearance, ScheduleDef, ScheduleField, ScheduleFilter, SortLevel, }`
- `pub use section_props::SectionProperties`
- `pub use stair::{ LandingBeams, LandingPosts, PostAt, StairBuild, StairCheck, StairConstruction, StairIssue, StairLayout, StairRailing, StairRules, StairShape, StairSide, Stringers, TreadRule, Turn, stair_steps, }`
- `pub use structural_material::StructuralMaterial`
- `pub use structural_profile::{ProfileDef, ProfileMaterial, ProfileShape, SectionOutline}`
- `pub use sun::{SunDate, SunInterval, SunMode, SunPreset, SunSettings, SunSite, SunState}`
- `pub use transform::PlanTransform`
- `pub use types::{ Category, ColumnSection, ElementType, FoundationKind, Layer, LayerFunction, SectionShape, StripHaunch, TypeData, TypeInfo, WallFunction, stair_layout, }`
- `pub use units::{AreaUnit, LengthUnit, MeasurementSystem, NumberStyle, SlopeUnit, Units, VolumeUnit}`
- `pub use view::{ CameraState, DetailLevel, Direction, Looking, MAX_PAPER_MM, MAX_SECTION_SEGMENT, MIN_PAPER_MM, MepDiscipline, ShadingSettings, SheetSize, Underlay, UnderlayLevel, UnderlayPlan, View, ViewCategory, ViewKind, Viewport, VisualStyle, }`
- `pub use glam::{DVec2, DVec3}`

<a id="mod-cc_core-arc"></a>

## `cc_core::arc`

<sub>`crates/cc_core/src/arc.rs`</sub>

Circular arcs in plan, described like polyline arcs by their two ends and
a *bulge*: `tan(sweep / 4)`, positive when the arc turns counter-clockwise
from the start to the end, 0 for a straight line, ±1 for a half circle.
A bulge is unchanged by moving, rotating or scaling the ends, and by a
mirror that also swaps them.

**Items:** [`MAX_BULGE`](#cc_core-arc-max_bulge) · [`Arc`](#cc_core-arc-arc) · [`segments`](#cc_core-arc-segments) · [`ARC_STEP`](#cc_core-arc-arc_step)

<a id="cc_core-arc-max_bulge"></a>

### const `MAX_BULGE`

```rust
pub const MAX_BULGE: f64
```

Largest bulge accepted: a sweep of about 337°, short of a full circle.

<a id="cc_core-arc-arc"></a>

### struct `Arc`

```rust
pub struct Arc
```

A circular arc from `start_angle` turning by `sweep` radians
(counter-clockwise when positive) around `center`.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `center: DVec2`
- `radius: f64`
- `start_angle: f64`
- `sweep: f64`

**Methods**

- `pub fn from_bulge(a: DVec2, b: DVec2, bulge: f64) -> Option<Arc>`

  The arc from `a` to `b` with `bulge`; `None` when it is a straight
  line (no bulge), or the ends coincide or are not finite.

- `pub fn bulge_through(a: DVec2, m: DVec2, b: DVec2) -> Option<f64>`

  Bulge of the arc from `a` through `m` to `b`; `None` when the points
  are (nearly) collinear, coincide, or `m` is not between the ends.

- `pub fn bulge_tangent(a: DVec2, tangent: DVec2, b: DVec2) -> Option<f64>`

  The arc tangent to `tangent` at `a` ending at `b` (a tangent arc),
  as a bulge; `None` when `b` lies on the tangent line.

- `pub fn bulge_radius(a: DVec2, b: DVec2, radius: f64, ccw: bool, major: bool) -> Option<f64>`

  The arc of `radius` from `a` to `b`, as a bulge: the shorter arc, or the
  longer one when `major`, turning counter-clockwise when `ccw`. `None` when
  the radius is less than half the distance between the ends.

- `pub fn sign(&self) -> f64`

  Turning direction: 1 counter-clockwise, -1 clockwise.

- `pub fn length(&self) -> f64`
- `pub fn angle_at(&self, u: f64) -> f64`

  Angle of the point `u` along the arc from its start.

- `pub fn point(&self, u: f64) -> DVec2`

  Point `u` along the arc from its start (beyond the ends it continues on the circle).

- `pub fn tangent(&self, u: f64) -> DVec2`

  Unit direction of travel at `u`.

- `pub fn at(&self, u: f64, v: f64) -> DVec2`

  Point `u` along the arc and `v` to the left of it (on a concentric circle).

- `pub fn radius_at(&self, v: f64) -> f64`

  Radius of the concentric circle `v` to the left of the arc.

- `pub fn locate(&self, p: DVec2) -> (f64, f64)`

  Position of `p` in the arc's frame: distance along the arc from its
  start (negative before it, past `length` after it; measured to the
  nearer side of the gap of the circle) and distance to the left.

- `pub fn points(&self, u0: f64, u1: f64, v: f64, max_step: f64) -> Vec<DVec2>`

  The arc between `u0` and `u1` along it (either order), as points at
  most `max_step` radians apart (at least two points).

- `pub fn bulge_between(&self, u0: f64, u1: f64) -> f64`

  The bulge of the part of this arc between `u0` and `u1`.


<a id="cc_core-arc-segments"></a>

### fn `segments`

```rust
pub fn segments(angle: f64, max_step: f64) -> usize
```

Number of segments for a span of `angle` radians, each at most `max_step`.

<a id="cc_core-arc-arc_step"></a>

### const `ARC_STEP`

```rust
pub const ARC_STEP: f64
```

Default angular step for drawing and meshing arcs: 3°.

<a id="mod-cc_core-bar_marks"></a>

## `cc_core::bar_marks`

<sub>`crates/cc_core/src/bar_marks.rs`</sub>

Bar marks: the names (N1, N2…) bending schedules and bar tags give the
bars of reinforcement, one per bar shape: the same diameter, steel grade
and shape (legs, radii, bends and hooks, within a length tolerance and
`ANGLE_TOLERANCE_DEG`, read from either end or mirrored).
Marks are derived (`cc_geom::bar_marks` resolves them, a pure
function): a set's `bar_mark` override names all its bars; otherwise a
bar whose shape matches an entry of the frozen marks table
(`BarMarking::frozen`, in table order) takes its mark; the shapes left
are numbered on the fly with the `prefix`, after the highest frozen number
(and from `start`), in a stable order: partition, then diameter
descending, then length descending, then shape. So every bar has a mark,
and frozen marks never shift when the model changes. Freezing writes the
marks shown into the table; renumbering clears it and freezes a fresh
numbering.
The settings live in `crate::Project::bar_marking` (tracked as project
settings) with the `stock_length` bars are bought in (bars longer than it
need splices).

**Items:** [`DEFAULT_PREFIX`](#cc_core-bar_marks-default_prefix) · [`DEFAULT_START`](#cc_core-bar_marks-default_start) · [`MAX_START`](#cc_core-bar_marks-max_start) · [`DEFAULT_LENGTH_TOLERANCE`](#cc_core-bar_marks-default_length_tolerance) · [`MIN_LENGTH_TOLERANCE`](#cc_core-bar_marks-min_length_tolerance) · [`MAX_LENGTH_TOLERANCE`](#cc_core-bar_marks-max_length_tolerance) · [`ANGLE_TOLERANCE_DEG`](#cc_core-bar_marks-angle_tolerance_deg) · [`DEFAULT_STOCK_LENGTH`](#cc_core-bar_marks-default_stock_length) · [`MAX_FROZEN`](#cc_core-bar_marks-max_frozen) · [`MAX_MARK_LEGS`](#cc_core-bar_marks-max_mark_legs) · [`MAX_MARK_LENGTH`](#cc_core-bar_marks-max_mark_length) · [`MAX_MARK_TEXT`](#cc_core-bar_marks-max_mark_text) · [`MIN_STOCK_LENGTH`](#cc_core-bar_marks-min_stock_length) · [`MAX_STOCK_LENGTH`](#cc_core-bar_marks-max_stock_length) · [`BarMarking`](#cc_core-bar_marks-barmarking) · [`FrozenMark`](#cc_core-bar_marks-frozenmark) · [`MarkShape`](#cc_core-bar_marks-markshape) · [`MarkLeg`](#cc_core-bar_marks-markleg) · [`tolerance_ok`](#cc_core-bar_marks-tolerance_ok) · [`stock_length_ok`](#cc_core-bar_marks-stock_length_ok)

<a id="cc_core-bar_marks-default_prefix"></a>

### const `DEFAULT_PREFIX`

```rust
pub const DEFAULT_PREFIX: &str
```

Prefix of numbered marks when none is set.

<a id="cc_core-bar_marks-default_start"></a>

### const `DEFAULT_START`

```rust
pub const DEFAULT_START: u32
```

First number when none is set.

<a id="cc_core-bar_marks-max_start"></a>

### const `MAX_START`

```rust
pub const MAX_START: u32
```

Largest first number.

<a id="cc_core-bar_marks-default_length_tolerance"></a>

### const `DEFAULT_LENGTH_TOLERANCE`

```rust
pub const DEFAULT_LENGTH_TOLERANCE: f64
```

Legs of two bars of one mark differ by at most this when none is set, meters.

<a id="cc_core-bar_marks-min_length_tolerance"></a>

### const `MIN_LENGTH_TOLERANCE`

```rust
pub const MIN_LENGTH_TOLERANCE: f64
```

Smallest and largest length tolerance, meters.

<a id="cc_core-bar_marks-max_length_tolerance"></a>

### const `MAX_LENGTH_TOLERANCE`

```rust
pub const MAX_LENGTH_TOLERANCE: f64
```

<a id="cc_core-bar_marks-angle_tolerance_deg"></a>

### const `ANGLE_TOLERANCE_DEG`

```rust
pub const ANGLE_TOLERANCE_DEG: f64
```

Turns of two bars of one mark differ by at most this, degrees.

<a id="cc_core-bar_marks-default_stock_length"></a>

### const `DEFAULT_STOCK_LENGTH`

```rust
pub const DEFAULT_STOCK_LENGTH: f64
```

Length bars are bought in when none is set, meters.

<a id="cc_core-bar_marks-max_frozen"></a>

### const `MAX_FROZEN`

```rust
pub const MAX_FROZEN: usize
```

Most entries of the frozen marks table.

<a id="cc_core-bar_marks-max_mark_legs"></a>

### const `MAX_MARK_LEGS`

```rust
pub const MAX_MARK_LEGS: usize
```

Most legs of a frozen shape (bars of more legs are numbered, not frozen).

<a id="cc_core-bar_marks-max_mark_length"></a>

### const `MAX_MARK_LENGTH`

```rust
pub const MAX_MARK_LENGTH: f64
```

Longest leg or radius a frozen shape keeps, meters.

<a id="cc_core-bar_marks-max_mark_text"></a>

### const `MAX_MARK_TEXT`

```rust
pub const MAX_MARK_TEXT: usize
```

Longest prefix or mark, in characters.

<a id="cc_core-bar_marks-min_stock_length"></a>

### const `MIN_STOCK_LENGTH`

```rust
pub const MIN_STOCK_LENGTH: f64
```

Shortest and longest stock length, meters.

<a id="cc_core-bar_marks-max_stock_length"></a>

### const `MAX_STOCK_LENGTH`

```rust
pub const MAX_STOCK_LENGTH: f64
```

<a id="cc_core-bar_marks-barmarking"></a>

### struct `BarMarking`

```rust
pub struct BarMarking
```

How bars are marked: the numbering and the frozen marks table.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `prefix: String`

  Text before the number of numbered marks (e.g. "N" for N1, N2…).

- `start: u32`

  The first number given.

- `length_tolerance: f64`

  Legs (and radii) of bars of one mark differ by at most this, meters.

- `stock_length: f64`

  Length bars are bought in, meters: longer bars need splices.

- `frozen: Vec<FrozenMark>`

  Marks kept for their shapes, in order: a bar takes the first that
  matches it.


**Methods**

- `pub fn is_default(&self) -> bool`

  Whether nothing differs from the defaults (files leave it out).

- `pub fn number_of(&self, mark: &str) -> Option<u32>`

  The number of a mark written with the prefix (`N12` → 12), if it is one.

- `pub fn mark_of(&self, n: u32) -> String`

  The mark of number `n`.

- `pub fn check(&self) -> Option<String>`

  Why the values cannot be kept (`None`: they can): the checks
  `Self::repair` applies, as a message.

- `pub fn repair(&mut self) -> usize`

  Bring the values into range (a long prefix cut, bad numbers reset to
  their defaults, unusable or extra frozen entries dropped); returns
  how many changed.


<a id="cc_core-bar_marks-frozenmark"></a>

### struct `FrozenMark`

```rust
pub struct FrozenMark
```

A mark kept for one bar shape.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `mark: String`
- `diameter: f64`

  Bar diameter, meters.

- `grade: String`

  Steel grade (empty: none).

- `shape: MarkShape`

**Methods**

- `pub fn is_valid(&self) -> bool`

  Whether the entry is usable: a mark of 1 to `MAX_MARK_TEXT`
  characters, a finite diameter above 0, a usable shape.


<a id="cc_core-bar_marks-markshape"></a>

### struct `MarkShape`

```rust
pub struct MarkShape
```

A bar shape in a compact canonical form: open or closed, its legs in
order (the orientation `cc_geom::bar_marks` picks among the readings
from either end, mirrored and, for closed bars, from any corner).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `closed: bool`
- `legs: Vec<MarkLeg>`

**Methods**

- `pub fn is_valid(&self) -> bool`

  Whether the shape is usable: one to `MAX_MARK_LEGS` legs of finite
  lengths above 0 and at most `MAX_MARK_LENGTH`, radii likewise,
  turns within a whole turn either way.

- `pub fn length(&self) -> f64`

  Length of the bar, meters.


<a id="cc_core-bar_marks-markleg"></a>

### struct `MarkLeg`

```rust
pub struct MarkLeg
```

One leg of a `MarkShape`: its length (meters), the radius of an arc
leg, the turn along an arc and the bend after the leg (degrees, signed
the way the bar first turns), and whether it is part of a hook.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `length: f64`
- `radius: Option<f64>`
- `sweep: f64`
- `bend: f64`
- `hook: bool`

<a id="cc_core-bar_marks-tolerance_ok"></a>

### fn `tolerance_ok`

```rust
pub fn tolerance_ok(v: f64) -> bool
```

Whether `v` is a usable length tolerance.

<a id="cc_core-bar_marks-stock_length_ok"></a>

### fn `stock_length_ok`

```rust
pub fn stock_length_ok(v: f64) -> bool
```

Whether `v` is a usable stock length.

<a id="mod-cc_core-bar_tag"></a>

## `cc_core::bar_tag`

<sub>`crates/cc_core/src/bar_tag.rs`</sub>

Bar tags: annotations that write a reinforcement set's bars in a plan,
an elevation or a section — `25 N1 Ø12.5 c/12 C=500`: the count, the
mark, the diameter in millimeters, the spacing in centimeters (sets
without one leave it out) and the length of one bar in centimeters
(`C=var` when the set's bars differ). A `multiplier` writes `2x` first,
for sets repeated on both faces. The text is derived each time
(`cc_views::bar_tags`), from the set's bars and marks.
Like text notes, a tag lives in one view (`view`); it goes with that view
and with its set.

**Items:** [`MAX_MULTIPLIER`](#cc_core-bar_tag-max_multiplier) · [`DEFAULT_TEXT_SIZE`](#cc_core-bar_tag-default_text_size) · [`BarTagParts`](#cc_core-bar_tag-bartagparts) · [`BarTag`](#cc_core-bar_tag-bartag) · [`repair_type`](#cc_core-bar_tag-repair_type)

<a id="cc_core-bar_tag-max_multiplier"></a>

### const `MAX_MULTIPLIER`

```rust
pub const MAX_MULTIPLIER: u32
```

Largest multiplier.

<a id="cc_core-bar_tag-default_text_size"></a>

### const `DEFAULT_TEXT_SIZE`

```rust
pub const DEFAULT_TEXT_SIZE: f64
```

Text size of the default type, paper millimeters.

<a id="cc_core-bar_tag-bartagparts"></a>

### struct `BarTagParts`

```rust
pub struct BarTagParts
```

The parts of a tag's text that are written.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `count: bool`
- `mark: bool`
- `diameter: bool`
- `spacing: bool`
- `length: bool`

<a id="cc_core-bar_tag-bartag"></a>

### struct `BarTag`

```rust
pub struct BarTag
```

A bar tag in a view.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `view: ViewId`
- `set: ElementId`

  The reinforcement set (or shaped bars) tagged.

- `position: DVec2`

  Where the text starts (its left end, on the baseline), in the view's
  drawing coordinates.

- `leader: Option<DVec2>`

  Where a leader from the text points (none: no leader).

- `rotation_deg: f64`

  The text's angle, degrees counter-clockwise.

- `multiplier: u32`

  How many times the set is repeated (2 writes `2x` first; 1: nothing).

- `parts: BarTagParts`

**Methods**

- `pub fn new(view: ViewId, set: ElementId, position: DVec2) -> Self`
- `pub fn is_valid(&self) -> bool`

  Whether its points are finite (a tag without one is dropped).

- `pub fn repair(&mut self) -> usize`

  Bring the values into range (points far away reset, a bad angle or
  multiplier reset); returns how many changed.

- `pub fn check(&self) -> Option<String>`

  Why the values cannot be kept (`None`: they can), as a message.


<a id="cc_core-bar_tag-repair_type"></a>

### fn `repair_type`

```rust
pub fn repair_type(text_size: &mut f64) -> usize
```

Replace an invalid type text size; returns how many changed.

<a id="mod-cc_core-bracing"></a>

## `cc_core::bracing`

<sub>`crates/cc_core/src/bracing.rs`</sub>

Parametric bracing: a host element that holds the parameters of the
braces of one bay of a frame (the bay line between two column axes, the
storey height, the number of storeys, the pattern and the brace type) and
owns the ordinary beams made from them, kept in step like a truss's (see
`crate::member_host`).
Local coordinates: x along start → end (the bay, `L` wide), heights above
the level from `base_offset`, storey `s` between `s · height` and
`(s + 1) · height`.

**Items:** [`LENGTH`](#cc_core-bracing-length) · [`HEIGHT`](#cc_core-bracing-height) · [`STORIES`](#cc_core-bracing-stories) · [`BASE_OFFSET`](#cc_core-bracing-base_offset) · [`DEFAULT_LENGTH`](#cc_core-bracing-default_length) · [`DEFAULT_HEIGHT`](#cc_core-bracing-default_height) · [`BracingPattern`](#cc_core-bracing-bracingpattern) · [`BracingRole`](#cc_core-bracing-bracingrole) · [`BracingMember`](#cc_core-bracing-bracingmember) · [`Bracing`](#cc_core-bracing-bracing) · [`host_type`](#cc_core-bracing-host_type)

<a id="cc_core-bracing-length"></a>

### const `LENGTH`

```rust
pub const LENGTH: RangeInclusive<f64>
```

The bay width, column axis to column axis, meters.

<a id="cc_core-bracing-height"></a>

### const `HEIGHT`

```rust
pub const HEIGHT: RangeInclusive<f64>
```

One storey, meters.

<a id="cc_core-bracing-stories"></a>

### const `STORIES`

```rust
pub const STORIES: RangeInclusive<u32>
```

<a id="cc_core-bracing-base_offset"></a>

### const `BASE_OFFSET`

```rust
pub const BASE_OFFSET: RangeInclusive<f64>
```

The foot of the lowest storey above the level, meters.

<a id="cc_core-bracing-default_length"></a>

### const `DEFAULT_LENGTH`

```rust
pub const DEFAULT_LENGTH: f64
```

<a id="cc_core-bracing-default_height"></a>

### const `DEFAULT_HEIGHT`

```rust
pub const DEFAULT_HEIGHT: f64
```

<a id="cc_core-bracing-bracingpattern"></a>

### enum `BracingPattern`

```rust
pub enum BracingPattern
```

How the braces cross a storey of the bay.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `V`

  Two braces from the top corners down to the middle of the bay.

- `Chevron`

  Two braces from the bottom corners up to the middle of the bay (an inverted V).

- `Diagonal`

  One brace from a bottom corner to the opposite top corner.

- `X`

  Both diagonals.

- `K`

  Four braces from the mid-height of each column to the middle of the
  bay at the top and at the bottom of the storey.


**Methods**

- `pub const ALL: [BracingPattern; 5]`
- `pub fn code(self) -> &'static str`
- `pub fn from_code(code: &str) -> Option<Self>`
- `pub fn key(self) -> &'static str`

  The i18n key of its name.

- `pub fn per_story(self) -> usize`

  How many braces it puts in a storey.


<a id="cc_core-bracing-bracingrole"></a>

### struct `BracingRole`

```rust
pub struct BracingRole
```

A brace's place: its storey (from the bottom) and its index in the pattern there.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Serialize`, `Deserialize`

**Fields**

- `story: u16`
- `index: u16`

**Methods**

- `pub fn key(self) -> RoleKey`
- `pub fn from_key(key: RoleKey) -> Option<Self>`

<a id="cc_core-bracing-bracingmember"></a>

### struct `BracingMember`

```rust
pub struct BracingMember
```

A brace the bracing manages.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Serialize`, `Deserialize`

**Fields**

- `role: BracingRole`
- `id: ElementId`

<a id="cc_core-bracing-bracing"></a>

### struct `Bracing`

```rust
pub struct Bracing
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`, `MemberHost`

**Fields**

- `start: DVec2`

  The bay in plan: the axes of its two columns.

- `end: DVec2`
- `base_offset: f64`

  The foot of the lowest storey above the level, meters.

- `height: f64`

  One storey, meters.

- `stories: u32`

  Storeys braced, one above the other.

- `pattern: BracingPattern`
- `flip: bool`

  A diagonal rises toward the start instead of the end.

- `alternate: bool`

  Diagonals change direction storey by storey (a zigzag).

- `brace_type: ElementTypeId`

  The beam type of the braces.

- `members: Vec<BracingMember>`
- `detached: Vec<BracingRole>`

**Methods**

- `pub fn new(brace_type: ElementTypeId) -> Self`

  An X bracing of one 3 m storey over a 5 m bay on the x axis, at the level.

- `pub fn length(&self) -> f64`

  The bay width.

- `pub fn total_height(&self) -> f64`

  Foot to top of all its storeys, meters.

- `pub fn check(&self) -> Result<(), String>`

  Whether every number is in its range (the first one that is not, named).

- `pub fn check_types(&self, project: &Project) -> Result<(), String>`

  Whether the brace type is a beam type.

- `pub fn reset_out_of_range(&mut self) -> usize`

  Put every out-of-range number back to its default; returns how many.

- `pub fn transform(&mut self, t: &PlanTransform)`

  Move, turn or mirror it (a mirrored diagonal rises toward the mirrored end).

- `pub fn layout(&self) -> Vec<(BracingRole, ElementTypeId, ElementKind)>`

  Its braces: role, type and beam, storey by storey from the bottom.
  Empty when a number is out of range (see `Bracing::check`).


<a id="cc_core-bracing-host_type"></a>

### fn `host_type`

```rust
pub fn host_type(id: ElementTypeId, name: String) -> ElementType
```

The type a project gets with its first bracing.

<a id="mod-cc_core-browser"></a>

## `cc_core::browser`

<sub>`crates/cc_core/src/browser.rs`</sub>

Project browser organization: named schemes that group the views of the
project browser by view type, level, discipline or phase, in that order
of nesting. The schemes and the one in use are project settings; with no
scheme in use the browser lists the views by type alone.
`tree` builds what the browser shows: at each depth the views that have
a value for the criterion go into a group per value, and the others (a
section has no level) go on to the next criterion beside those groups.

**Items:** [`MAX_SCHEMES`](#cc_core-browser-max_schemes) · [`MAX_CRITERIA`](#cc_core-browser-max_criteria) · [`MAX_NAME`](#cc_core-browser-max_name) · [`DEFAULT_GROUPING`](#cc_core-browser-default_grouping) · [`GroupBy`](#cc_core-browser-groupby) · [`Discipline`](#cc_core-browser-discipline) · [`BrowserScheme`](#cc_core-browser-browserscheme) · [`BrowserOrganization`](#cc_core-browser-browserorganization) · [`distinct`](#cc_core-browser-distinct) · [`GroupKey`](#cc_core-browser-groupkey) · [`Node`](#cc_core-browser-node) · [`label`](#cc_core-browser-label) · [`tree`](#cc_core-browser-tree)

<a id="cc_core-browser-max_schemes"></a>

### const `MAX_SCHEMES`

```rust
pub const MAX_SCHEMES: usize
```

Most schemes a project keeps.

<a id="cc_core-browser-max_criteria"></a>

### const `MAX_CRITERIA`

```rust
pub const MAX_CRITERIA: usize
```

Most criteria a scheme nests.

<a id="cc_core-browser-max_name"></a>

### const `MAX_NAME`

```rust
pub const MAX_NAME: usize
```

Longest scheme name, characters.

<a id="cc_core-browser-default_grouping"></a>

### const `DEFAULT_GROUPING`

```rust
pub const DEFAULT_GROUPING: [GroupBy; 1]
```

The grouping of a project without a scheme in use: by view type.

<a id="cc_core-browser-groupby"></a>

### enum `GroupBy`

```rust
pub enum GroupBy
```

What a level of the browser tree groups the views by.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Serialize`, `Deserialize`

**Variants**

- `ViewType`

  Floor plans, ceiling plans, 3D views, sections, sheets…

- `Level`

  The level of a floor or ceiling plan, lowest first.

- `Discipline`

  Architecture, structure or a building service (plans only).

- `Phase`

  The phase a view shows, earliest first.


**Methods**

- `pub const ALL: [GroupBy; 4]`
- `pub fn name(self) -> &'static str`

  Name in commands (`view_type`, `level`, `discipline`, `phase`).

- `pub fn from_name(name: &str) -> Option<Self>`
- `pub fn key(self) -> &'static str`

  Translation key of the criterion's name.


<a id="cc_core-browser-discipline"></a>

### enum `Discipline`

```rust
pub enum Discipline
```

The discipline a plan belongs to.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`

**Variants**

- `Architecture`
- `Structure`
- `Services(MepDiscipline)`

**Methods**

- `pub fn name(self) -> &'static str`

  Name in command results (`architecture`, `structure`, `plumbing`…).

- `pub fn key(self) -> &'static str`

  Translation key of the discipline's name.

- `pub fn of(view: &View) -> Option<Self>`

  The discipline of a view: plans have one, other views none.


<a id="cc_core-browser-browserscheme"></a>

### struct `BrowserScheme`

```rust
pub struct BrowserScheme
```

A named way to group the browser's views.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `id: crate::BrowserSchemeId`
- `name: String`
- `group_by: Vec<GroupBy>`

  The criteria, outermost first (no repeats, at most `MAX_CRITERIA`).


<a id="cc_core-browser-browserorganization"></a>

### struct `BrowserOrganization`

```rust
pub struct BrowserOrganization
```

The project's browser schemes and the one in use.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `schemes: Vec<BrowserScheme>`
- `active: Option<crate::BrowserSchemeId>`

  The scheme the browser follows (`None`: by view type alone).


**Methods**

- `pub fn is_default(&self) -> bool`

  Nothing to save: no schemes, the default in use.

- `pub fn scheme(&self, id: crate::BrowserSchemeId) -> Option<&BrowserScheme>`
- `pub fn in_use(&self) -> Option<&BrowserScheme>`

  The scheme in use, if any.

- `pub fn grouping(&self) -> &[GroupBy]`

  The criteria the browser groups by now.

- `pub fn repair(&mut self) -> usize`

  Fix what a crafted file may hold: too many schemes, repeated or zero
  ids, empty or overlong names, repeated or too many criteria, a scheme
  in use that does not exist. Returns how many things changed.


<a id="cc_core-browser-distinct"></a>

### fn `distinct`

```rust
pub fn distinct(criteria: &[GroupBy]) -> Vec<GroupBy>
```

`criteria` without repeats, cut at `MAX_CRITERIA`.

<a id="cc_core-browser-groupkey"></a>

### enum `GroupKey`

```rust
pub enum GroupKey
```

What a group of the browser tree stands for.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`

**Variants**

- `ViewType(ViewCategory)`
- `Level(LevelId)`
- `Discipline(Discipline)`
- `Phase(PhaseId)`

**Methods**

- `pub fn criterion(self) -> GroupBy`

  The criterion the group comes from.


<a id="cc_core-browser-node"></a>

### enum `Node`

```rust
pub enum Node
```

A row of the browser tree.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Variants**

- `Group { key: GroupKey, children: Vec<Node> }`
- `View(ViewId)`

**Methods**

- `pub fn view_count(&self) -> usize`

  How many views the row holds (a view row holds itself).


<a id="cc_core-browser-label"></a>

### fn `label`

```rust
pub fn label(view: &View) -> String
```

A view's name in the browser: sheets go by number and name.

<a id="cc_core-browser-tree"></a>

### fn `tree`

```rust
pub fn tree(project: &Project, criteria: &[GroupBy]) -> Vec<Node>
```

The browser tree of `project` grouped by `criteria`, outermost first.

<a id="mod-cc_core-cad"></a>

## `cc_core::cad`

<sub>`crates/cc_core/src/cad.rs`</sub>

Imported CAD drawings (DWG and DXF underlays).
A `CadDrawing` is the neutral form `cc_cad` reads both formats into:
layers and flat primitives (blocks already inserted) in drawing units.
Arcs stay arcs (centers and ends can be snapped to) and are tessellated
only when drawn. A `CadImport` element places one drawing in one 2D
view: units to meters, an extra scale, a rotation and an insertion point.
The drawing is shared (`Arc`), so undo steps that change the placement or
the layers do not copy it.

**Items:** [`MAX_PRIMS`](#cc_core-cad-max_prims) · [`MAX_POLY_POINTS`](#cc_core-cad-max_poly_points) · [`MAX_TEXT_CHARS`](#cc_core-cad-max_text_chars) · [`MAX_COORD`](#cc_core-cad-max_coord) · [`CadUnit`](#cc_core-cad-cadunit) · [`CadColorMode`](#cc_core-cad-cadcolormode) · [`CadLayer`](#cc_core-cad-cadlayer) · [`CadHAlign`](#cc_core-cad-cadhalign) · [`CadVAlign`](#cc_core-cad-cadvalign) · [`CadPrim`](#cc_core-cad-cadprim) · [`on_paper`](#cc_core-cad-on_paper) · [`text_width`](#cc_core-cad-text_width) · [`union`](#cc_core-cad-union) · [`CadImage`](#cc_core-cad-cadimage) · [`MAX_IMAGES`](#cc_core-cad-max_images) · [`CadDrawing`](#cc_core-cad-caddrawing) · [`CadImport`](#cc_core-cad-cadimport) · [`CadSnapIndex`](#cc_core-cad-cadsnapindex) · [`CadSnap`](#cc_core-cad-cadsnap) · [`aci_color`](#cc_core-cad-aci_color)

<a id="cc_core-cad-max_prims"></a>

### const `MAX_PRIMS`

```rust
pub const MAX_PRIMS: usize
```

Most primitives one drawing may hold (after blocks are inserted).

<a id="cc_core-cad-max_poly_points"></a>

### const `MAX_POLY_POINTS`

```rust
pub const MAX_POLY_POINTS: usize
```

Most points of one polyline or fill.

<a id="cc_core-cad-max_text_chars"></a>

### const `MAX_TEXT_CHARS`

```rust
pub const MAX_TEXT_CHARS: usize
```

Longest text kept, in characters.

<a id="cc_core-cad-max_coord"></a>

### const `MAX_COORD`

```rust
pub const MAX_COORD: f64
```

Largest coordinate magnitude accepted, drawing units.

<a id="cc_core-cad-cadunit"></a>

### enum `CadUnit`

```rust
pub enum CadUnit
```

Units a drawing is drawn in.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Millimeters`
- `Centimeters`
- `Decimeters`
- `Meters`
- `Kilometers`
- `Inches`
- `Feet`
- `Yards`
- `Miles`

**Methods**

- `pub const ALL: [CadUnit; 9]`
- `pub fn meters(self) -> f64`

  Meters in one unit.

- `pub fn from_insunits(code: i64) -> Option<CadUnit>`

  The unit of a DXF `$INSUNITS` code (DWG `INSUNITS` uses the same
  codes); `None` for unitless (0) and units not listed here.

- `pub fn code(self) -> &'static str`

  Short name used by commands (`mm`, `cm`, `dm`, `m`, `km`, `in`, `ft`, `yd`, `mi`).

- `pub fn from_code(code: &str) -> Option<CadUnit>`
- `pub fn key(self) -> &'static str`

  Translation key of the unit name.


<a id="cc_core-cad-cadcolormode"></a>

### enum `CadColorMode`

```rust
pub enum CadColorMode
```

How an imported drawing is colored.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Grey`

  One grey for everything: a background to trace over.

- `Layers`

  The colors of the layers and entities (white and very light colors darkened for white paper).

- `Black`

  Black, like the model's linework.


**Methods**

- `pub const ALL: [CadColorMode; 3]`
- `pub fn key(self) -> &'static str`
- `pub fn code(self) -> &'static str`
- `pub fn from_code(code: &str) -> Option<CadColorMode>`

<a id="cc_core-cad-cadlayer"></a>

### struct `CadLayer`

```rust
pub struct CadLayer
```

A layer of the drawing.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `name: String`
- `color: [u8; 3]`

  The layer color (from its color index or true color).

- `visible: bool`

  On and thawed in the file.


<a id="cc_core-cad-cadhalign"></a>

### enum `CadHAlign`

```rust
pub enum CadHAlign
```

Horizontal alignment of a text at its position.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Left`
- `Center`
- `Right`

<a id="cc_core-cad-cadvalign"></a>

### enum `CadVAlign`

```rust
pub enum CadVAlign
```

Vertical alignment of a text at its position.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Baseline`
- `Bottom`
- `Middle`
- `Top`

<a id="cc_core-cad-cadprim"></a>

### enum `CadPrim`

```rust
pub enum CadPrim
```

A primitive of a drawing, in drawing units. `layer` indexes
`CadDrawing::layers`; `color` is the entity's own color (`None`: the
layer's).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Variants**

- `Polyline { layer: u32, #[serde(default, skip_serializing_if = "Option::is_none")] color: Option<[u8; 3]>, #[serde(default, skip_serializing_if = "is_false")] closed: bool, pts: Vec<DVec2>, #[serde(default, skip_serializing_if = "Vec::is_empty")] bulges: Vec<f64>, }`

  Polyline through `pts`; `bulges[i]` bends the segment from point `i`
  (`tan(sweep / 4)`, positive counter-clockwise; empty: all straight).

- `Arc { layer: u32, #[serde(default, skip_serializing_if = "Option::is_none")] color: Option<[u8; 3]>, center: DVec2, radius: f64, start: f64, sweep: f64, }`

  Circular arc from `start` (radians) turning by `sweep` (positive
  counter-clockwise; ±τ for a circle).

- `Text { layer: u32, #[serde(default, skip_serializing_if = "Option::is_none")] color: Option<[u8; 3]>, pos: DVec2, height: f64, #[serde(default)] rotation: f64, text: String, #[serde(default)] h: CadHAlign, #[serde(default)] v: CadVAlign, }`

  One line of text `height` tall, rotated by `rotation` radians.

- `Fill { layer: u32, #[serde(default, skip_serializing_if = "Option::is_none")] color: Option<[u8; 3]>, pts: Vec<DVec2>, }`

  Solid fill of a simple outline.


**Methods**

- `pub fn layer(&self) -> u32`
- `pub fn color(&self) -> Option<[u8; 3]>`
- `pub fn bounds(&self) -> Option<(DVec2, DVec2)>`

  Bounding box in drawing units (arcs by their full circle when that is simpler).

- `pub fn is_valid(&self) -> bool`

  Whether every number is finite and within `MAX_COORD`, with enough points.


<a id="cc_core-cad-on_paper"></a>

### fn `on_paper`

```rust
pub fn on_paper(rgb: [u8; 3]) -> [u8; 3]
```

A file color as drawn on white paper: colors meant for a dark screen
(white, yellow, cyan) darkened.

<a id="cc_core-cad-text_width"></a>

### fn `text_width`

```rust
pub fn text_width(text: &str, height: f64) -> f64
```

Approximate width of a text line (the proportion used for notes).

<a id="cc_core-cad-union"></a>

### fn `union`

```rust
pub fn union(a: Option<(DVec2, DVec2)>, b: Option<(DVec2, DVec2)>) -> Option<(DVec2, DVec2)>
```

Merge two optional boxes.

<a id="cc_core-cad-cadimage"></a>

### struct `CadImage`

```rust
pub struct CadImage
```

A raster image laid on the drawing: pixel (0, 0) is the top-left corner
of the parallelogram `origin + s·u + t·v` (`s`, `t` in 0..1), whose
bottom-left corner is `origin`, `u` running along the image's width and
`v` up its height (drawing units).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `layer: u32`
- `origin: DVec2`
- `u: DVec2`
- `v: DVec2`
- `image: std::sync::Arc<crate::raster::RasterImage>`

**Methods**

- `pub fn corners(&self) -> [DVec2; 4]`

  Corners in order: bottom-left, bottom-right, top-right, top-left.

- `pub fn is_valid(&self) -> bool`

  Whether the placement is finite, not flat, and the image usable.

- `pub fn bounds(&self) -> Option<(DVec2, DVec2)>`

<a id="cc_core-cad-max_images"></a>

### const `MAX_IMAGES`

```rust
pub const MAX_IMAGES: usize
```

Most images one drawing may hold.

<a id="cc_core-cad-caddrawing"></a>

### struct `CadDrawing`

```rust
pub struct CadDrawing
```

A drawing read from a DWG, DXF, SVG or PDF file, or an imported picture.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `layers: Vec<CadLayer>`
- `prims: Vec<CadPrim>`
- `images: Vec<CadImage>`

  Pictures, drawn under the primitives.

- `extents: Option<(DVec2, DVec2)>`

  Extents of the primitives (min, max), drawing units.

- `units: Option<CadUnit>`

  Units the file declares (`None`: unitless).


**Methods**

- `pub fn compute_extents(&mut self)`

  Recompute `CadDrawing::extents` from the primitives.

- `pub fn layer_of(&self, prim: &CadPrim) -> Option<&CadLayer>`

  The layer of a primitive.

- `pub fn needs_repair(&self) -> bool`

  Whether `CadDrawing::repair` would change anything.

- `pub fn repair(&mut self) -> usize`

  Remove invalid primitives and dangling layer references; returns how many were fixed.


<a id="cc_core-cad-cadimport"></a>

### struct `CadImport`

```rust
pub struct CadImport
```

A DWG or DXF drawing placed in one 2D view, in that view's drawing
coordinates (like text notes): drawing point `p` lands at
`position + R(rotation)·M·(p · units · scale)`, `M` mirroring across
the drawing's X axis when `mirrored`.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `view: ViewId`
- `name: String`

  Name of the file it was read from (for display).

- `drawing: std::sync::Arc<CadDrawing>`
- `units: CadUnit`

  Units of the drawing.

- `scale: f64`

  Extra scale on top of the units.

- `position: DVec2`

  Where the drawing's origin lands, view coordinates (meters).

- `rotation_deg: f64`

  Rotation in degrees counter-clockwise.

- `mirrored: bool`
- `hidden_layers: BTreeSet<String>`

  Layers hidden in this view (by name).

- `color_mode: CadColorMode`
- `halftone: bool`

  Drawn faded (mixed with white).


**Methods**

- `pub fn factor(&self) -> f64`

  Meters per drawing unit, scale included.

- `pub fn placement(&self) -> PlanTransform`

  The rigid part of the placement (rotation, mirror, position).

- `pub fn to_view(&self, p: DVec2) -> DVec2`

  A drawing point in view coordinates.

- `pub fn to_drawing(&self, q: DVec2) -> DVec2`

  A view point in drawing units (inverse of `CadImport::to_view`).

- `pub fn layer_visible(&self, index: u32) -> bool`

  Whether layer `index` is shown.

- `pub fn transform(&mut self, t: &PlanTransform)`

  Move, rotate or mirror the placement.

- `pub fn is_valid(&self) -> bool`

  Whether the placement's numbers are usable.

- `pub fn snap_features(&self, q: DVec2, r: f64) -> CadSnap`

  Snap features of the visible primitives near `q` (view coordinates)
  within `r`: endpoints and vertices, midpoints, arc centers, and
  segments, all in view coordinates.

- `pub fn snap_features_in(&self, index: &CadSnapIndex, q: DVec2, r: f64) -> CadSnap`

  `CadImport::snap_features` with the drawing's primitives found
  through `index`, built from this element's drawing (callers keep it
  while the drawing is the same, so a pointer move visits only the
  primitives near it). Long polylines give only their segments near `q`.


<a id="cc_core-cad-cadsnapindex"></a>

### struct `CadSnapIndex`

```rust
pub struct CadSnapIndex
```

Where a drawing's primitives lie, to find those near a point without
visiting them all: snapping asks on every pointer move, and imported
drawings hold up to a million primitives. A uniform grid over the
drawing's extents; primitives spanning many cells are kept apart and
checked on every query.

**Traits:** `Debug`, `Clone`, `Default`

*Some fields are private.*

**Methods**

- `pub fn new(drawing: &CadDrawing) -> Self`
- `pub fn near(&self, p: DVec2, r: f64) -> Vec<usize>`

  Indices, in ascending order, of the primitives whose bounds come
  within `r` of `p` (drawing units).


<a id="cc_core-cad-cadsnap"></a>

### struct `CadSnap`

```rust
pub struct CadSnap
```

Snap features of an imported drawing near a point, view coordinates.

**Traits:** `Debug`, `Clone`, `Default`, `PartialEq`

**Fields**

- `ends: Vec<DVec2>`
- `mids: Vec<DVec2>`
- `centers: Vec<DVec2>`
- `lines: Vec<(DVec2, DVec2)>`
- `arcs: Vec<crate::arc::Arc>`

<a id="cc_core-cad-aci_color"></a>

### fn `aci_color`

```rust
pub fn aci_color(index: i64) -> [u8; 3]
```

RGB of an AutoCAD Color Index (the standard 256-entry palette: 1-9 the
named colors, 10-249 hue/saturation steps, 250-255 greys); 0 and 256
(by block / by layer) and 7 come out black for white paper.

<a id="mod-cc_core-callout"></a>

## `cc_core::callout`

<sub>`crates/cc_core/src/callout.rs`</sub>

Crop regions and callouts.
A view of the model drawn in 2D (a floor or ceiling plan, a section or
an elevation) may have a `crop`: a rectangle in its own drawing
coordinates (meters on the view plane) outside which nothing of the
model is drawn. A *callout* is a mark in a parent view standing for a
detail view of a region of it: the detail view keeps the parent's kind
(so it stays live with the model) and its crop is the callout's
rectangle, or the callout points to a drafting view and keeps its own
rectangle. The callout lives on the detail view (`crate::View::callout`),
so deleting that view removes its mark, and moving the mark is editing
the detail view's crop.

**Items:** [`MIN_CROP`](#cc_core-callout-min_crop) · [`MAX_CROP`](#cc_core-callout-max_crop) · [`MAX_CHAIN`](#cc_core-callout-max_chain) · [`CropBox`](#cc_core-callout-cropbox) · [`Callout`](#cc_core-callout-callout)

<a id="cc_core-callout-min_crop"></a>

### const `MIN_CROP`

```rust
pub const MIN_CROP: f64
```

Smallest side of a crop rectangle, meters.

<a id="cc_core-callout-max_crop"></a>

### const `MAX_CROP`

```rust
pub const MAX_CROP: f64
```

Largest coordinate of a crop rectangle, meters from the view's origin.

<a id="cc_core-callout-max_chain"></a>

### const `MAX_CHAIN`

```rust
pub const MAX_CHAIN: usize
```

How many parent links `Project::sync_callouts` follows (details of details).

<a id="cc_core-callout-cropbox"></a>

### struct `CropBox`

```rust
pub struct CropBox
```

A rectangle in a view's drawing coordinates, `min` below and left of `max`.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `min: DVec2`
- `max: DVec2`

**Methods**

- `pub fn new(a: DVec2, b: DVec2) -> Self`

  The rectangle with corners `a` and `b` (any two opposite corners).

- `pub fn is_valid(&self) -> bool`

  Finite, ordered, at least `MIN_CROP` on each side and within `MAX_CROP`.

- `pub fn contains(&self, p: DVec2) -> bool`
- `pub fn size(&self) -> DVec2`
- `pub fn center(&self) -> DVec2`
- `pub fn corners(&self) -> [DVec2; 4]`

  The corners, counter-clockwise from `min`.

- `pub fn translated(&self, d: DVec2) -> Self`

<a id="cc_core-callout-callout"></a>

### struct `Callout`

```rust
pub struct Callout
```

A callout: the mark in `parent` standing for the view that holds it.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `parent: ViewId`

  The view the mark is drawn in.

- `rect: Option<CropBox>`

  The mark's rectangle in the parent, for a callout pointing to a
  drafting view (which has no crop). A detail view of the model has
  none: its crop is the rectangle.


<a id="mod-cc_core-component"></a>

## `cc_core::component`

<sub>`crates/cc_core/src/component.rs`</sub>

Parametric components: user-defined objects (furniture, fixtures,
casework…) whose solids are sized by named parameters.
A `ComponentDef` lists numeric parameters and solid pieces. Each piece
is a box, a cylinder, a plan outline extruded upwards, or a profile of
lines and arcs drawn on the plan, front or side plane and extruded across
it or revolved about an axis (`crate::profile`); its coordinates and
sizes are `crate::expr` expressions over the parameters, in the
component's local frame: origin at the insertion point, X to the right,
Y to the back, Z up, meters. A piece marked `void` is cut out of the
solid pieces it overlaps. Types of the same component share the definition and
differ in parameter values; parameters marked `instance` can be
overridden per element.
Pieces brought in from other tools (IFC import) may be fixed polyhedral
meshes instead, and the definition then remembers the IFC class the
object had, so it is written back as the same class.
A script piece (`ComponentShape::Script`) is Lua source that returns
other pieces; see `crate::component_script`.

**Re-exports**

- `pub use crate::profile::{ProfilePoint, SketchPlane}`

**Items:** [`ComponentKind`](#cc_core-component-componentkind) · [`ParamKind`](#cc_core-component-paramkind) · [`ComponentParam`](#cc_core-component-componentparam) · [`ComponentShape`](#cc_core-component-componentshape) · [`ComponentItem`](#cc_core-component-componentitem) · [`DetailLevels`](#cc_core-component-detaillevels) · [`ConnectorDirection`](#cc_core-component-connectordirection) · [`ComponentConnector`](#cc_core-component-componentconnector) · [`ResolvedConnector`](#cc_core-component-resolvedconnector) · [`WALL_THICKNESS`](#cc_core-component-wall_thickness) · [`HostOpening`](#cc_core-component-hostopening) · [`DoorOperation`](#cc_core-component-dooroperation) · [`WindowOperation`](#cc_core-component-windowoperation) · [`SymbolSegment`](#cc_core-component-symbolsegment) · [`SymbolLine`](#cc_core-component-symbolline) · [`ComponentDef`](#cc_core-component-componentdef) · [`IfcClass`](#cc_core-component-ifcclass) · [`ComponentPart`](#cc_core-component-componentpart) · [`PartError`](#cc_core-component-parterror) · [`ResolveCtx`](#cc_core-component-resolvectx) · [`replace_name`](#cc_core-component-replace_name) · [`convex_hull`](#cc_core-component-convex_hull) · [`library`](#cc_core-component-library) · [`mep_library`](#cc_core-component-mep_library) · [`device_library`](#cc_core-component-device_library)

<a id="cc_core-component-componentkind"></a>

### enum `ComponentKind`

```rust
pub enum ComponentKind
```

What a component represents (used for IFC mapping and grouping).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Furniture`
- `Casework`
- `Plumbing`
- `Lighting`
- `Electrical`

  Outlets, switches and distribution boards.

- `Generic`
- `Door`

  Doors: with a `HostOpening`, their types are of the door category.

- `Window`

  Windows: with a `HostOpening`, their types are of the window category.

- `Niche`

  Niches: with a `HostOpening` that has a `depth`, their types are of
  the niche category and recess one face of their wall (see `crate::openings`).

- `TitleBlock`

  A sheet layout (border, title block): 2D pieces in paper millimeters, see `crate::title_block`.

- `Detail`

  A detail component: 2D pieces in meters placed in one view, see `crate::detail_component`.


**Methods**

- `pub const ALL: [ComponentKind; 11]`
- `pub fn key(self) -> &'static str`
- `pub fn is_2d(self) -> bool`

  Kinds whose definitions hold only 2D pieces (`ComponentDef::sheet`).


<a id="cc_core-component-paramkind"></a>

### enum `ParamKind`

```rust
pub enum ParamKind
```

Unit or kind of a parameter value.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Length`

  Meters.

- `Angle`

  Degrees.

- `Number`
- `Text`

  Text (in `ComponentParam::text`).

- `Boolean`

  Yes or no: `value` 1 or 0, so expressions can read it.

- `Choice`

  One of `ComponentParam::choices`, by key (in `text`).

- `Material`

  A project material, by name (in `text`).


**Methods**

- `pub const ALL: [ParamKind; 3]`

  The numeric kinds, the ones a hand-made parameter can take.

- `pub const EVERY: [ParamKind; 7]`

  Every kind.

- `pub fn key(self) -> &'static str`
- `pub fn is_text(self) -> bool`

  The value is text (`ComponentParam::text`), not a number.

- `pub fn name(self) -> &'static str`

  The name used by commands and scripts.

- `pub fn from_name(name: &str) -> Option<ParamKind>`

<a id="cc_core-component-componentparam"></a>

### struct `ComponentParam`

```rust
pub struct ComponentParam
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `name: String`

  Name used in expressions (see `expr::is_valid_name`).

- `value: f64`

  Value of this type (1 or 0 for booleans; unused by text kinds).

- `kind: ParamKind`
- `instance: bool`

  Elements may override the value.

- `label: String`

  Shown instead of the name when set.

- `text: String`

  Value of this type for text, choice (the key) and material (the name) parameters.

- `choices: Vec<crate::component_script::ParamChoice>`

  The options of a choice parameter.

- `from_script: bool`

  Declared by a script piece: kept in step with it, not renamed by hand.


**Methods**

- `pub fn number(name: impl Into<String>, value: f64, kind: ParamKind) -> ComponentParam`

  A numeric parameter of this type.

- `pub fn display(&self) -> &str`

  The label shown: the label, or the name.

- `pub fn accepts_text(&self, text: &str) -> bool`

  Whether `text` is a value this parameter can take (a choice's key).


<a id="cc_core-component-componentshape"></a>

### enum `ComponentShape`

```rust
pub enum ComponentShape
```

Geometry of a piece; every field is an expression.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Variants**

- `Box { x: String, y: String, z: String, width: String, depth: String, height: String }`

  Box from the corner (`x`, `y`, `z`) with the given sizes.

- `Cylinder { x: String, y: String, z: String, radius: String, height: String }`

  Vertical cylinder with its base centered at (`x`, `y`, `z`).

- `Extrusion { points: Vec<(String, String)>, z: String, height: String }`

  Closed profile in plan extruded upwards from `z`.

- `Profile { plane: SketchPlane, points: Vec<ProfilePoint>, offset: String, depth: String }`

  Closed profile of lines and arcs on `plane`, placed at `offset` on the
  axis normal to the plane (Z, Y or X) and extruded `depth` along it.

- `Revolution { plane: SketchPlane, points: Vec<ProfilePoint>, offset: String, axis: String, angle: String }`

  Closed profile on `plane` (placed at `offset` on its normal axis) turned
  `angle` degrees about the line `u = axis` of the plane (vertical in
  the front and side planes).

- `Sweep { plane: SketchPlane, path: Vec<ProfilePoint>, offset: String, profile: Vec<ProfilePoint> }`

  Closed `profile` (u to the left of the path, v along the plane's
  normal) carried along the open `path` drawn on `plane`, which stands
  at `offset` on its normal axis; mitered at the path's corners.

- `Blend { plane: SketchPlane, bottom: Vec<ProfilePoint>, top: Vec<ProfilePoint>, offset: String, depth: String }`

  The solid joining the closed `bottom` outline on `plane` at `offset`
  to the closed `top` outline `depth` farther along its normal.

- `Mesh { vertices: std::sync::Arc<Vec<[f64; 3]>>, faces: std::sync::Arc<Vec<Vec<u32>>> }`

  Fixed polyhedral surface (not parametric): vertices in the component
  frame and faces as counter-clockwise (seen from outside) index loops.

- `Script { source: String }`

  Lua source whose `geometry` function returns the pieces (see
  `crate::component_script`); they take this piece's material and
  flags where they set none.


**Methods**

- `pub fn expressions(&self) -> Vec<(&'static str, &String)>`

  Every expression of the shape, with a field label.

- `pub fn expressions_mut(&mut self) -> Vec<&mut String>`
- `pub fn point_count(&self) -> usize`

  Number of outline points of a profile (0 for boxes, cylinders and meshes).


<a id="cc_core-component-componentitem"></a>

### struct `ComponentItem`

```rust
pub struct ComponentItem
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `name: String`
- `shape: ComponentShape`
- `material: String`

  Project material; empty for the default color.

- `in_plan: bool`

  The piece's outline is drawn in plans.

- `void: bool`

  The piece is cut out of the solid pieces it overlaps instead of adding to them.

- `plan_only: bool`

  A plan symbol only (a door's swing): drawn in plans, not a solid.

- `detail: DetailLevels`

  The detail levels of the views that show the piece (all by default).


<a id="cc_core-component-detaillevels"></a>

### struct `DetailLevels`

```rust
pub struct DetailLevels
```

The detail levels of the views a piece shows in.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `coarse: bool`
- `medium: bool`
- `fine: bool`

**Methods**

- `pub const ALL: Self`
- `pub fn is_all(&self) -> bool`
- `pub fn shows(&self, level: crate::DetailLevel) -> bool`

  Whether a view at `level` shows the piece.

- `pub fn set(&mut self, level: crate::DetailLevel, on: bool)`

<a id="cc_core-component-connectordirection"></a>

### enum `ConnectorDirection`

```rust
pub enum ConnectorDirection
```

Which way a run leaves a connector, in the component's frame.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Down`
- `Up`
- `Front`

  Towards -Y.

- `Back`

  Towards +Y (usually the wall).

- `Left`

  Towards -X.

- `Right`

  Towards +X.


**Methods**

- `pub const ALL: [ConnectorDirection; 6]`
- `pub fn key(self) -> &'static str`
- `pub fn vector(self) -> DVec3`

  Unit vector in the component's frame.

- `pub fn is_vertical(self) -> bool`

  Whether runs leave it vertically.


<a id="cc_core-component-componentconnector"></a>

### struct `ComponentConnector`

```rust
pub struct ComponentConnector
```

A point where a pipe or conduit connects to the component: a toilet has
a water inlet and a sewer outlet, an outlet its electrical connection.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `name: String`
- `system: crate::SystemKind`

  The kind of system it connects to.

- `x: String`

  Position in the component's frame (expressions).

- `y: String`
- `z: String`
- `direction: ConnectorDirection`
- `diameter: String`

  Outside diameter of the run it takes, meters (an expression); empty for unset.


**Methods**

- `pub fn expressions_mut(&mut self) -> [&mut String; 4]`

<a id="cc_core-component-resolvedconnector"></a>

### struct `ResolvedConnector`

```rust
pub struct ResolvedConnector
```

A connector resolved in the component's frame.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `index: usize`
- `name: String`
- `system: crate::SystemKind`
- `point: DVec3`
- `direction: ConnectorDirection`
- `diameter: Option<f64>`

<a id="cc_core-component-wall_thickness"></a>

### const `WALL_THICKNESS`

```rust
pub const WALL_THICKNESS: &str
```

Name of the host wall's thickness in the expressions of door and window
components (see `crate::openings`).

<a id="cc_core-component-hostopening"></a>

### struct `HostOpening`

```rust
pub struct HostOpening
```

The opening a door or window component cuts in its host wall: `width`
wide, centered on the component, `height` tall from the sill.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `width: String`

  Expression of the parameters.

- `height: String`

  Expression of the parameters.

- `sill: String`

  Sill height of new elements (windows), an expression; empty for 0.

- `own_symbol: bool`

  Plans draw the pieces marked `in_plan` instead of the category's
  symbol (a door's swing, a window's sash lines).

- `operation: WindowOperation`

  How a window's panels open (doors ignore it): the plan symbol and
  the IFC panel properties.

- `door_operation: DoorOperation`

  How a door's leaves swing (windows ignore it): the plan swings,
  the elevation symbol and the IFC operation type.

- `depth: String`

  Niches: how far the opening goes into the wall from the face it is
  on, an expression of the parameters; empty for an opening through
  the wall (doors and windows).


**Methods**

- `pub fn expressions_mut(&mut self) -> [&mut String; 3]`

<a id="cc_core-component-dooroperation"></a>

### enum `DoorOperation`

```rust
pub enum DoorOperation
```

How the leaves of a swing door open, after IFC's door operation types:
each leaf swings about a vertical hinge towards the side the door opens
to, with the hand of `WindowOperation` (left is the start of the
opening along the wall, swapped by the element's hand flip).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `SingleSwingLeft`

  One leaf hinged on the left.

- `SingleSwingRight`

  One leaf hinged on the right.

- `DoubleDoor`

  Two leaves hinged on either jamb, meeting in the middle (a double door).


**Methods**

- `pub const ALL: [Self; 3]`
- `pub fn is_single_left(&self) -> bool`
- `pub fn key(self) -> &'static str`

  The interface key of its name.

- `pub fn leaves(self) -> usize`

  How many leaves the door has.

- `pub fn mirrored(self) -> Self`

  The same door with its hand swapped (a flipped element).

- `pub fn ifc_operation(self) -> &'static str`

  Its `IfcDoorTypeOperationEnum` value.

- `pub fn from_ifc(operation: &str) -> Option<Self>`

  The operation of an IFC door operation type; `None` for those it has no value for.

- `pub fn ifc_panels(self) -> &'static [(&'static str, f64)]`

  The leaves from the start of the opening: each one's IFC panel
  position (`IfcDoorPanelPositionEnum`) and share of the width.

- `pub fn hinges(self) -> &'static [(f64, f64)]`

  The hinges in plan, as `WindowOperation::hinges`.

- `pub fn elevation_lines(self, width: f64, height: f64) -> Vec<SymbolSegment>`

  The lines of its elevation symbol, as `WindowOperation::elevation_lines`.


<a id="cc_core-component-windowoperation"></a>

### enum `WindowOperation`

```rust
pub enum WindowOperation
```

How the panels of a window open, after IFC's window panel operations:
side-hung panels swing about a vertical hinge towards the side the
window opens to (`y` of its frame). The hand is the hinge's side seen
looking that way (a door's convention): left is the start of the
opening along the wall (`-x`), swapped by the element's hand flip.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Fixed`

  One panel that does not open.

- `SideHungLeft`

  One panel hinged on the left.

- `SideHungRight`

  One panel hinged on the right.

- `DoubleSideHung`

  Two panels hinged on either jamb, meeting in the middle.

- `TopHung`

  One panel hinged at its head, projecting out at the bottom.

- `PivotHorizontal`

  One panel turning about a horizontal axis at mid height.

- `PivotVertical`

  One panel turning about a vertical axis in the middle.

- `Sliding`

  Two panels sliding past each other along the wall.


**Methods**

- `pub const ALL: [Self; 8]`
- `pub fn is_fixed(&self) -> bool`
- `pub fn key(self) -> &'static str`

  The interface key of its name.

- `pub fn ifc_panels(self) -> &'static [(&'static str, &'static str)]`

  The panels from the start of the opening: each one's IFC operation
  and position (`IfcWindowPanelOperationEnum`, `IfcWindowPanelPositionEnum`).

- `pub fn ifc_partitioning(self) -> &'static str`

  The IFC partitioning of a window type with these panels.

- `pub fn from_ifc_panels(panels: &[&str]) -> Option<Self>`

  The operation of IFC panels (operations in order along the window);
  `None` for combinations it has no value for.

- `pub fn hinges(self) -> &'static [(f64, f64)]`

  The hinges in plan: for each side-hung panel, where its hinge stands
  as a fraction of the opening's width from its start, and which way
  along the wall the panel closes (+1 towards the end). Others: none.

- `pub fn elevation_lines(self, width: f64, height: f64) -> Vec<SymbolSegment>`

  The lines of its elevation symbol on an opening `width` wide and
  `height` tall, as `(x, z)` from its bottom left (the start of the
  opening along the wall): the opening direction as a triangle whose
  apex points at the hinge or the axis, pivot axes running past the
  frame, and a sliding window's arrows.


<a id="cc_core-component-symbolsegment"></a>

### type alias `SymbolSegment`

```rust
pub type SymbolSegment = ((f64, f64), (f64, f64), SymbolLine)
```

A line of a window's elevation symbol: its ends `(x, z)` and how it is drawn.

<a id="cc_core-component-symbolline"></a>

### enum `SymbolLine`

```rust
pub enum SymbolLine
```

How a line of a window's elevation symbol is drawn.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Variants**

- `Opening`

  The opening direction (dashed).

- `Axis`

  A pivot axis (dash-dot).

- `Plain`

  A plain thin line (a sliding panel's arrow).


<a id="cc_core-component-componentdef"></a>

### struct `ComponentDef`

```rust
pub struct ComponentDef
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `kind: ComponentKind`
- `params: Vec<ComponentParam>`
- `items: Vec<ComponentItem>`
- `ifc: Option<IfcClass>`

  IFC class of objects imported from IFC, written back on export.

- `mep: Option<crate::MepData>`

  Plumbing fixture or electrical device data.

- `opening: Option<Box<HostOpening>>`

  The opening a door or window cuts in its wall.

- `connectors: Vec<ComponentConnector>`

  Where pipes and conduits connect to it.

- `ref_planes: Vec<crate::component_ref::RefPlane>`

  Named planes whose positions expressions read (see `crate::component_ref`).

- `ref_lines: Vec<crate::component_ref::RefLine>`

  Named lines on the sketch planes.

- `dimensions: Vec<crate::component_ref::RefDimension>`

  Dimensions between reference planes, labeled by the parameters that drive them.

- `equals: Vec<crate::component_ref::RefEqual>`

  Planes kept halfway between two others (EQ).

- `hotspots: Vec<crate::component_script::Hotspot>`

  Points elements move with grips, declared by script pieces.

- `sheet: Vec<crate::title_block::SheetItem>`

  The 2D pieces of a title block (see `crate::title_block`) or of a
  detail component (see `crate::detail_component`).


**Methods**

- `pub fn param(&self, name: &str) -> Option<&ComponentParam>`
- `pub fn values(&self, overrides: &BTreeMap<String, f64>) -> BTreeMap<String, f64>`

  Numeric parameter values (booleans as 1 or 0) with instance
  `overrides` applied (only to instance parameters; non-finite
  overrides are ignored). Text parameters are left out.

- `pub fn texts(&self, overrides: &BTreeMap<String, String>) -> BTreeMap<String, String>`

  Text, choice and material values with instance `overrides` applied
  (only to instance parameters, and only values the parameter accepts).

- `pub fn hotspot_positions(&self, moved: &BTreeMap<String, [f64; 3]>) -> BTreeMap<String, DVec3>`

  Hotspot positions with the element's `moved` ones applied (finite only).

- `pub fn hotspot(&self, name: &str) -> Option<&crate::component_script::Hotspot>`
- `pub fn has_script(&self) -> bool`

  Whether any piece is a script.

- `pub fn opening_size(&self) -> Option<(f64, f64)>`

  Width and height of the opening a door or window cuts (`None` for
  other components, or when they do not evaluate to positive sizes).

- `pub fn opening_depth(&self) -> Option<f64>`

  The depth of a niche's recess (see `HostOpening::depth`): `None`
  for an opening through the wall, or a depth that does not evaluate
  to a positive length.

- `pub fn resolve_connectors(&self, overrides: &BTreeMap<String, f64>) -> Vec<ResolvedConnector>`

  The connectors whose expressions evaluate, in the component's frame.

- `pub fn default_sill(&self) -> f64`

  Sill height of new doors and windows of this definition (0 when unset or invalid).

- `pub fn category(&self) -> crate::Category`

  The category of the types of this definition: doors and windows
  (with an opening) have their own.

- `pub fn resolve(&self, overrides: &BTreeMap<String, f64>) -> (Vec<ComponentPart>, Vec<(usize, PartError)>)`

  Resolve every piece; pieces that fail are reported with their index.

- `pub fn resolve_in(&self, overrides: &BTreeMap<String, f64>, extra: &[(&str, f64)]) -> (Vec<ComponentPart>, Vec<(usize, PartError)>)`

  Resolve every piece with `extra` names defined too (a door's
  `WALL_THICKNESS`); parameters of the same name win. Pieces also
  read the reference planes. Script pieces fail here: they need
  `ComponentDef::resolve_with`.

- `pub fn resolve_with(&self, ctx: &ResolveCtx) -> (Vec<ComponentPart>, Vec<(usize, PartError)>)`

  Resolve every piece, script pieces through `ctx.scripts`.

- `pub fn main_material(&self) -> &str`

  Material of the first piece (for coloring and type legends).

- `pub fn rename_param(&mut self, old: &str, new: &str)`

  Rename a parameter and every reference to it in the pieces.

- `pub fn rename(&mut self, old: &str, new: &str)`

  Rename a parameter, reference plane or line and every expression,
  dimension and label that reads it.

- `pub fn scope(&self, overrides: &BTreeMap<String, f64>, extra: &[(&str, f64)]) -> Scope` *(in `cc_core::component_ref`)*

  The values every expression of the component reads (see `Scope`).
  Planes may read parameters and other planes in any order; a plane
  left unresolved (a missing name or a cycle) is reported once.

- `pub fn ref_plane(&self, name: &str) -> Option<RefPlane>` *(in `cc_core::component_ref`)*

  A reference plane by name, origin planes included (as `at: "0"`).

- `pub fn all_ref_planes(&self) -> Vec<RefPlane>` *(in `cc_core::component_ref`)*

  Every plane, the origin ones first.

- `pub fn name_taken(&self, name: &str) -> bool` *(in `cc_core::component_ref`)*

  Whether `name` is already used by a parameter, plane or line (or reserved).

- `pub fn dimension_value(&self, scope: &Scope, index: usize) -> Result<f64, RefError>` *(in `cc_core::component_ref`)*

  The value a dimension shows: the distance between its planes, or why not.

- `pub fn equal_holds(&self, index: usize) -> Result<(), RefError>` *(in `cc_core::component_ref`)*

  Whether equality `index` holds: three parallel planes seen edge-on
  in its view, the middle one written halfway between the others.

- `pub fn ref_problems(&self) -> Vec<String>` *(in `cc_core::component_ref`)*

  The names an expression of the component reads, other than parameters
  and planes, that do not exist; and repeated or invalid names. Empty
  when the references are sound.

- `pub fn resolve_sheet(&self, paper: DVec2) -> SheetPieces` *(in `cc_core::title_block`)*

  The 2D pieces on a `paper` sheet (millimeters); pieces that fail are
  reported with their index and left out.

- `pub fn resolve_2d(&self, lookup: &dyn Fn(&str) -> Option<f64>) -> SheetPieces` *(in `cc_core::title_block`)*

  The 2D pieces resolved with `lookup` (title blocks and detail
  components); pieces that fail are reported with their index and left out.


<a id="cc_core-component-ifcclass"></a>

### struct `IfcClass`

```rust
pub struct IfcClass
```

An IFC entity class with its predefined and object types.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `entity: String`

  Entity name as written in IFC files, e.g. `IFCPIPESEGMENT`.

- `predefined: String`

  `PredefinedType` enumeration value (without dots); empty if unset.

- `object_type: String`

  `ObjectType` (the user-defined type when `predefined` is `USERDEFINED`).

- `properties: Vec<crate::properties::PropertySet>`

  Property and quantity sets of the IFC type object.


<a id="cc_core-component-componentpart"></a>

### struct `ComponentPart`

```rust
pub struct ComponentPart
```

A resolved piece: a prism of `footprint` between `z0` and `z1`, in the
component's local frame. Meshes, profiles on the front and side planes
and revolutions carry their faces, with the convex hull of the vertices
as footprint and their height range.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `item: usize`
- `footprint: Vec<DVec2>`
- `z0: f64`
- `z1: f64`
- `material: String`
- `in_plan: bool`
- `void: bool`

  Cut out of the other pieces (see `cc_geom::resolve_component`).

- `plan_only: bool`

  Drawn in plans only (see `ComponentItem::plan_only`).

- `detail: DetailLevels`

  The detail levels of the views that show it.

- `faces: Vec<Vec<DVec3>>`

  Faces of a piece that is not a vertical prism (empty for prisms).

- `smooth: Vec<bool>`

  Per face of `faces`, whether it is a facet of a curved surface (see
  `cc_geom::Solid::smooth_curved`); filled in when a placed piece is
  resolved, empty otherwise.


<a id="cc_core-component-parterror"></a>

### enum `PartError`

```rust
pub enum PartError
```

Why a piece could not be resolved.

**Traits:** `Debug`, `Clone`, `PartialEq`, `std::fmt::Display`

**Variants**

- `Expr { field: &'static str, error: ExprError, }`
- `Degenerate`

  A size is zero or negative, or a profile has fewer than 3 points.

- `Script(String)`

  A script piece failed (its last pieces that worked may still be shown).


<a id="cc_core-component-resolvectx"></a>

### struct `ResolveCtx`

```rust
pub struct ResolveCtx<'a>
```

What resolving reads besides the definition.

**Traits:** `Default`

**Fields**

- `overrides: &'a BTreeMap<String, f64>`

  Numeric (and boolean) instance values.

- `texts: &'a BTreeMap<String, String>`

  Text, choice and material instance values.

- `hotspots: &'a BTreeMap<String, [f64; 3]>`

  Moved hotspots.

- `extra: &'a [(&'a str, f64)]`

  Names defined besides the parameters (a door's wall thickness).

- `scripts: &'a dyn crate::component_script::ScriptEval`
- `owner: crate::component_script::ScriptOwner`

  Whose pieces these are (for the last script result that worked).


<a id="cc_core-component-replace_name"></a>

### fn `replace_name`

```rust
pub fn replace_name(text: &str, old: &str, new: &str) -> String
```

Replace whole-identifier occurrences of `old` in `text` (numbers such as
`2e3` are left alone).

<a id="cc_core-component-convex_hull"></a>

### fn `convex_hull`

```rust
pub fn convex_hull(points: &[DVec2]) -> Vec<DVec2>
```

Convex hull of plan points (counter-clockwise; fewer than 3 points when degenerate).

<a id="cc_core-component-library"></a>

### fn `library`

```rust
pub fn library(n: &dyn Fn(&str) -> String) -> Vec<(String, String, ComponentDef)>
```

The default component library: (component name, type name, definition).
Parameter names come from `n` (keys `tpl-param-*`), so expressions use
the translated names; materials are the template's.

<a id="cc_core-component-mep_library"></a>

### fn `mep_library`

```rust
pub fn mep_library(n: &dyn Fn(&str) -> String) -> Vec<(String, String, ComponentDef)>
```

The building-services part of the default library: plumbing fixtures
and electrical devices. Wall-mounted devices have their back on the
insertion point and stand out towards -Y; their elevation is the
instance's base offset (the mounting height).

<a id="cc_core-component-device_library"></a>

### fn `device_library`

```rust
pub fn device_library(device: crate::DeviceKind, n: &dyn Fn(&str) -> String) -> Option<(String, String, ComponentDef)>
```

The built-in definition of a communications or lightning protection
device, which a project gets with the first one placed: (component,
type name, definition). Data outlets and racks hang on walls like the
electrical devices (their back on the insertion point); an air terminal
is a rod standing `Height` (an instance parameter) on a base plate; an
earth electrode a rod driven `Length` down from its top. `None` for the
other device kinds (see `mep_library`).

<a id="mod-cc_core-component_file"></a>

## `cc_core::component_file`

<sub>`crates/cc_core/src/component_file.rs`</sub>

`.ccomp` component files: a component definition with all its types and
the materials it uses, to share components between projects.

**Items:** [`COMPONENT_FORMAT_VERSION`](#cc_core-component_file-component_format_version) · [`ComponentFile`](#cc_core-component_file-componentfile) · [`NestedComponent`](#cc_core-component_file-nestedcomponent) · [`ComponentFileType`](#cc_core-component_file-componentfiletype) · [`ComponentFileError`](#cc_core-component_file-componentfileerror) · [`extract`](#cc_core-component_file-extract) · [`to_ron`](#cc_core-component_file-to_ron) · [`from_ron`](#cc_core-component_file-from_ron) · [`save`](#cc_core-component_file-save) · [`MAX_COMPONENT_BYTES`](#cc_core-component_file-max_component_bytes) · [`load`](#cc_core-component_file-load) · [`MAX_LIBRARY_FILES`](#cc_core-component_file-max_library_files) · [`scan`](#cc_core-component_file-scan) · [`types_for`](#cc_core-component_file-types_for)

<a id="cc_core-component_file-component_format_version"></a>

### const `COMPONENT_FORMAT_VERSION`

```rust
pub const COMPONENT_FORMAT_VERSION: u32
```

Current `.ccomp` format version.

<a id="cc_core-component_file-componentfile"></a>

### struct `ComponentFile`

```rust
pub struct ComponentFile
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `format_version: u32`
- `generator: String`
- `component: String`
- `def: ComponentDef`

  The definition, with the first type's parameter values.

- `types: Vec<ComponentFileType>`
- `materials: Vec<Material>`
- `nested: Vec<NestedComponent>`

  The detail components a detail component nests (directly or deeper),
  so the file stands alone; loading adds those the project lacks.


<a id="cc_core-component_file-nestedcomponent"></a>

### struct `NestedComponent`

```rust
pub struct NestedComponent
```

A detail component another one in the file nests.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `component: String`
- `def: ComponentDef`

  The definition, with its first type's parameter values.


<a id="cc_core-component_file-componentfiletype"></a>

### struct `ComponentFileType`

```rust
pub struct ComponentFileType
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `name: String`
- `values: BTreeMap<String, f64>`

  Parameter values of this type.

- `info: TypeInfo`

<a id="cc_core-component_file-componentfileerror"></a>

### enum `ComponentFileError`

```rust
pub enum ComponentFileError
```

**Traits:** `Debug`, `Error`

**Variants**

- `Io(#[from] std::io::Error)`
- `Parse(#[from] ron::error::SpannedError)`
- `Serialize(#[from] ron::Error)`
- `Version(u32)`
- `Invalid(String)`
- `NotFound(String)`

<a id="cc_core-component_file-extract"></a>

### fn `extract`

```rust
pub fn extract(project: &Project, component: &str) -> Result<ComponentFile, ComponentFileError>
```

Collect the component named `component` from `project`.

<a id="cc_core-component_file-to_ron"></a>

### fn `to_ron`

```rust
pub fn to_ron(file: &ComponentFile) -> Result<String, ComponentFileError>
```

<a id="cc_core-component_file-from_ron"></a>

### fn `from_ron`

```rust
pub fn from_ron(text: &str) -> Result<ComponentFile, ComponentFileError>
```

Parse and validate a component file.

<a id="cc_core-component_file-save"></a>

### fn `save`

```rust
pub fn save(file: &ComponentFile, path: &Path) -> Result<(), ComponentFileError>
```

<a id="cc_core-component_file-max_component_bytes"></a>

### const `MAX_COMPONENT_BYTES`

```rust
pub const MAX_COMPONENT_BYTES: u64
```

Largest `.ccomp` file read: imported meshes make the largest ones.

<a id="cc_core-component_file-load"></a>

### fn `load`

```rust
pub fn load(path: &Path) -> Result<ComponentFile, ComponentFileError>
```

<a id="cc_core-component_file-max_library_files"></a>

### const `MAX_LIBRARY_FILES`

```rust
pub const MAX_LIBRARY_FILES: usize
```

Most component files a library scan reads.

<a id="cc_core-component_file-scan"></a>

### fn `scan`

```rust
pub fn scan(folders: &[std::path::PathBuf]) -> (Vec<(std::path::PathBuf, ComponentFile)>, Vec<(std::path::PathBuf, String)>)
```

The component files (`.ccomp`) in `folders` and their subfolders, sorted
by path: the ones that load, and the paths of those that do not, with
why. Bounded in depth and number of files; links to folders are not followed.

<a id="cc_core-component_file-types_for"></a>

### fn `types_for`

```rust
pub fn types_for(project: &Project, file: &ComponentFile, mut alloc: impl FnMut() -> u64) -> (Vec<ElementType>, Vec<Material>)
```

Types to add to `project` for `file`: the component and type names are
made unique and missing materials are listed. Ids are allocated from
`alloc`, so callers can add them inside a transaction.

<a id="mod-cc_core-component_ref"></a>

## `cc_core::component_ref`

<sub>`crates/cc_core/src/component_ref.rs`</sub>

Reference planes, reference lines and labeled dimensions of a
component: the skeleton that parameters drive and pieces follow.
A reference plane is a named plane square to one axis of the component's
frame, at a position given by an `crate::expr` expression. Its name can
be used in any expression of the component (a piece's coordinates, other
planes, connectors), so a piece "locked" to a plane simply reads it:
`x = Right`. Three origin planes are always there, at zero:
`CENTER_X` (the plane x = 0, center left/right), `CENTER_Y` (y = 0,
center front/back) and `BASE` (z = 0).
A dimension between two parallel planes shows their distance in one of
the component's views. Labeling it with a parameter writes the second
plane's position as the first plus (or minus) the parameter, so changing
the parameter moves the plane and whatever reads it. Everything stays an
expression; there is no constraint solver.

**Items:** [`CENTER_X`](#cc_core-component_ref-center_x) · [`CENTER_Y`](#cc_core-component_ref-center_y) · [`BASE`](#cc_core-component_ref-base) · [`ORIGIN_PLANES`](#cc_core-component_ref-origin_planes) · [`MAX_REFS`](#cc_core-component_ref-max_refs) · [`PlaneAxis`](#cc_core-component_ref-planeaxis) · [`RefPlane`](#cc_core-component_ref-refplane) · [`RefLine`](#cc_core-component_ref-refline) · [`RefDimension`](#cc_core-component_ref-refdimension) · [`RefEqual`](#cc_core-component_ref-refequal) · [`RefError`](#cc_core-component_ref-referror) · [`Scope`](#cc_core-component_ref-scope) · [`is_reserved_for`](#cc_core-component_ref-is_reserved_for)

<a id="cc_core-component_ref-center_x"></a>

### const `CENTER_X`

```rust
pub const CENTER_X: &str
```

The origin plane x = 0 (center left/right).

<a id="cc_core-component_ref-center_y"></a>

### const `CENTER_Y`

```rust
pub const CENTER_Y: &str
```

The origin plane y = 0 (center front/back).

<a id="cc_core-component_ref-base"></a>

### const `BASE`

```rust
pub const BASE: &str
```

The origin plane z = 0 (the base).

<a id="cc_core-component_ref-origin_planes"></a>

### const `ORIGIN_PLANES`

```rust
pub const ORIGIN_PLANES: [(&str, PlaneAxis); 3]
```

The origin planes: name and axis.

<a id="cc_core-component_ref-max_refs"></a>

### const `MAX_REFS`

```rust
pub const MAX_REFS: usize
```

Most reference planes, lines and dimensions of a component.

<a id="cc_core-component_ref-planeaxis"></a>

### enum `PlaneAxis`

```rust
pub enum PlaneAxis
```

The axis a reference plane is square to.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `X`

  A plane x = constant (left/right): seen edge-on in the plan and front views.

- `Y`

  A plane y = constant (front/back): seen edge-on in the plan and side views.

- `Z`

  A plane z = constant (a height): seen edge-on in the front and side views.


**Methods**

- `pub const ALL: [PlaneAxis; 3]`
- `pub fn in_view(self, view: SketchPlane) -> Option<bool>`

  Where a plane square to this axis is seen edge-on in `view`: `Some(true)`
  as a line of constant u (vertical), `Some(false)` as a line of constant
  v (horizontal), `None` when the view looks along the plane's face.

- `pub fn key(self) -> &'static str`

<a id="cc_core-component_ref-refplane"></a>

### struct `RefPlane`

```rust
pub struct RefPlane
```

A named plane square to `axis` at the position `at` (an expression).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `name: String`
- `axis: PlaneAxis`
- `at: String`

<a id="cc_core-component_ref-refline"></a>

### struct `RefLine`

```rust
pub struct RefLine
```

A named line drawn on one of the sketch planes: the plane stands at
`offset` along its normal axis, and the ends are (u, v) expressions in
it (see `SketchPlane`). Revolution axes and angled references use it.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `name: String`
- `plane: SketchPlane`
- `offset: String`
- `start: (String, String)`
- `end: (String, String)`

<a id="cc_core-component_ref-refdimension"></a>

### struct `RefDimension`

```rust
pub struct RefDimension
```

A dimension between the parallel reference planes `from` and `to`, shown
in `view`; `label` is the parameter that drives it, if any. `place` is
where its line sits across the planes (meters, in the view's other
coordinate).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `view: SketchPlane`
- `from: String`
- `to: String`
- `label: Option<String>`
- `place: f64`

<a id="cc_core-component_ref-refequal"></a>

### struct `RefEqual`

```rust
pub struct RefEqual
```

An equality (EQ): the plane `middle` stays halfway between the parallel
planes `a` and `b`, shown in `view` as two equal dimensions at `place`.
Its position is written `(a + b) / 2` when the equality is made.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `view: SketchPlane`
- `a: String`
- `middle: String`
- `b: String`
- `place: f64`

**Methods**

- `pub fn expression(&self) -> String`

  The expression that keeps `middle` halfway.


<a id="cc_core-component_ref-referror"></a>

### enum `RefError`

```rust
pub enum RefError
```

Why a reference plane, line or dimension does not evaluate.

**Traits:** `Debug`, `Clone`, `PartialEq`, `std::fmt::Display`

**Variants**

- `Expr { name: String, error: ExprError }`

  Its expression fails (an unknown name also covers planes that
  depend on each other in a cycle).

- `Dimension { index: usize }`

  A dimension names a missing plane, or planes that are not parallel.

- `Equal { index: usize }`

  An equality names a missing plane, planes that are not parallel, or
  its middle plane is no longer halfway.


<a id="cc_core-component_ref-scope"></a>

### struct `Scope`

```rust
pub struct Scope
```

Every name that expressions of the component read, with its value: the
parameters (with overrides), `extra` names, the origin planes and the
reference planes that evaluate, plus the planes that do not.

**Traits:** `Debug`, `Clone`, `Default`, `PartialEq`

**Fields**

- `values: BTreeMap<String, f64>`
- `errors: Vec<RefError>`

**Methods**

- `pub fn get(&self, name: &str) -> Option<f64>`

<a id="cc_core-component_ref-is_reserved_for"></a>

### fn `is_reserved_for`

```rust
pub fn is_reserved_for(kind: crate::component::ComponentKind, name: &str) -> bool
```

Whether `name` is reserved in a component of `kind`: the origin planes
and a door's wall thickness everywhere, the paper size
(`SheetWidth`, `SheetHeight`) in title blocks.

<a id="mod-cc_core-component_script"></a>

## `cc_core::component_script`

<sub>`crates/cc_core/src/component_script.rs`</sub>

Scripted component pieces: the model side.
A `crate::ComponentShape::Script` piece holds Lua source that declares
parameters and hotspots and returns ordinary pieces from its `geometry`
function. Running Lua is not this crate's business: resolving goes
through a `ScriptEval` (the `cc_script` crate implements it), which
gets the parameter values and returns the pieces, or an error and the
last pieces that worked for the same owner.
The parameters a script declares are copied into the definition's
`params` (marked `from_script`) and its hotspots into `hotspots`, so
schedules, the properties palette, engine commands and files see them
without running Lua.

**Items:** [`MAX_SCRIPT_LEN`](#cc_core-component_script-max_script_len) · [`MAX_CHOICES`](#cc_core-component_script-max_choices) · [`MAX_TEXT_LEN`](#cc_core-component_script-max_text_len) · [`MAX_HOTSPOTS`](#cc_core-component_script-max_hotspots) · [`ParamChoice`](#cc_core-component_script-paramchoice) · [`HotspotShape`](#cc_core-component_script-hotspotshape) · [`Hotspot`](#cc_core-component_script-hotspot) · [`parse_color`](#cc_core-component_script-parse_color) · [`color_text`](#cc_core-component_script-color_text) · [`ScriptOwner`](#cc_core-component_script-scriptowner) · [`ScriptInput`](#cc_core-component_script-scriptinput) · [`ScriptResult`](#cc_core-component_script-scriptresult) · [`ScriptEval`](#cc_core-component_script-scripteval) · [`NoScripts`](#cc_core-component_script-noscripts) · [`truncate`](#cc_core-component_script-truncate)

<a id="cc_core-component_script-max_script_len"></a>

### const `MAX_SCRIPT_LEN`

```rust
pub const MAX_SCRIPT_LEN: usize
```

Longest script source kept (bytes); longer ones are emptied on load.

<a id="cc_core-component_script-max_choices"></a>

### const `MAX_CHOICES`

```rust
pub const MAX_CHOICES: usize
```

Most choices of a choice parameter.

<a id="cc_core-component_script-max_text_len"></a>

### const `MAX_TEXT_LEN`

```rust
pub const MAX_TEXT_LEN: usize
```

Longest text value, label or choice key kept (bytes).

<a id="cc_core-component_script-max_hotspots"></a>

### const `MAX_HOTSPOTS`

```rust
pub const MAX_HOTSPOTS: usize
```

Most hotspots of a definition.

<a id="cc_core-component_script-paramchoice"></a>

### struct `ParamChoice`

```rust
pub struct ParamChoice
```

One option of a choice parameter: a stable key and the label shown.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Eq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `key: String`
- `label: String`

<a id="cc_core-component_script-hotspotshape"></a>

### enum `HotspotShape`

```rust
pub enum HotspotShape
```

The marker drawn for a hotspot.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Circle`
- `Square`
- `Triangle`
- `Diamond`
- `TriangleDown`

**Methods**

- `pub const ALL: [HotspotShape; 5]`
- `pub fn name(self) -> &'static str`

  The name scripts use.

- `pub fn from_name(name: &str) -> Option<HotspotShape>`

<a id="cc_core-component_script-hotspot"></a>

### struct `Hotspot`

```rust
pub struct Hotspot
```

A point of a component that its elements move with a grip; scripts read
where it is. Positions are in the component's frame (meters).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `name: String`
- `label: String`
- `position: [f64; 3]`

  Where new elements have it.

- `shape: HotspotShape`
- `color: [u8; 4]`

  RGBA.


<a id="cc_core-component_script-parse_color"></a>

### fn `parse_color`

```rust
pub fn parse_color(text: &str) -> Option<[u8; 4]>
```

Parse `#RRGGBB` or `#RRGGBBAA` (alpha last).

<a id="cc_core-component_script-color_text"></a>

### fn `color_text`

```rust
pub fn color_text(c: [u8; 4]) -> String
```

`#RRGGBBAA`.

<a id="cc_core-component_script-scriptowner"></a>

### struct `ScriptOwner`

```rust
pub struct ScriptOwner
```

Whose script result this is, for the last pieces that worked: the
element's type and the element (none for a type's own preview).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Default`

**Fields**

- `type_uid: Uid`
- `element: Option<Uid>`
- `piece: usize`

  The piece of the definition.


<a id="cc_core-component_script-scriptinput"></a>

### struct `ScriptInput`

```rust
pub struct ScriptInput
```

What a script receives.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `numbers: BTreeMap<String, f64>`

  Numeric parameters (booleans as 0 or 1), reference planes and extra
  names such as a door's wall thickness.

- `texts: BTreeMap<String, String>`

  Text, choice (the key) and material (the name) parameters.

- `hotspots: BTreeMap<String, DVec3>`

  Hotspot positions.


<a id="cc_core-component_script-scriptresult"></a>

### struct `ScriptResult`

```rust
pub struct ScriptResult
```

A script's pieces, or why there are none this time.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `items: Option<Arc<Vec<ComponentItem>>>`

  The pieces, or the last that worked for the same owner when the
  script failed (none when it never worked).

- `error: Option<String>`

<a id="cc_core-component_script-scripteval"></a>

### trait `ScriptEval`

```rust
pub trait ScriptEval
```

Runs a piece's script.

**Implemented by (2):** [`cc_core::component_script::NoScripts`](cc_core.md#cc_core-component_script-noscripts), [`cc_script::object::ScriptCache`](cc_script.md#cc_script-object-scriptcache)

**Required and provided items**

- `fn eval(&self, source: &str, input: &ScriptInput, owner: ScriptOwner) -> ScriptResult`

<a id="cc_core-component_script-noscripts"></a>

### struct `NoScripts`

```rust
pub struct NoScripts
```

No runtime: every script fails.

**Traits:** `ScriptEval`

<a id="cc_core-component_script-truncate"></a>

### fn `truncate`

```rust
pub fn truncate(text: &mut String, max: usize) -> bool
```

Cut `text` to at most `max` bytes on a character boundary; whether it was cut.

<a id="mod-cc_core-connection"></a>

## `cc_core::connection`

<sub>`crates/cc_core/src/connection.rs`</sub>

Steel and timber connections: an element at one end of a column or beam
(`member`, `end`) of a standard kind, whose plates and bolts
`cc_geom::connection` derives from the member's geometry on every build.
It has no placement of its own: it is removed with its member
(`crate::EditCtx::remove_element`), follows its phases and design option
(`crate::phase::phase_host`), is never copied (`Project::copy_set`),
and a member end carries at most one (`Project::repair_connections`
keeps the lowest id).

**Items:** [`LENGTH`](#cc_core-connection-length) · [`COUNT`](#cc_core-connection-count) · [`ConnectionKind`](#cc_core-connection-connectionkind) · [`MemberEnd`](#cc_core-connection-memberend) · [`ConnectionParams`](#cc_core-connection-connectionparams) · [`Connection`](#cc_core-connection-connection) · [`host_type`](#cc_core-connection-host_type)

<a id="cc_core-connection-length"></a>

### const `LENGTH`

```rust
pub const LENGTH: RangeInclusive<f64>
```

Every length of a connection, meters. A plate's `width` and `length`
may also be 0: taken from the member's section plus `edge`.

<a id="cc_core-connection-count"></a>

### const `COUNT`

```rust
pub const COUNT: RangeInclusive<u32>
```

Bolt rows and columns.

<a id="cc_core-connection-connectionkind"></a>

### enum `ConnectionKind`

```rust
pub enum ConnectionKind
```

The standard joints.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `BasePlate`

  A steel plate under a column with anchor bolts.

- `EndPlate`

  A steel plate on a beam end that stops at a column face, bolted through.

- `FinPlate`

  A steel plate along the web of a beam end at a column or a beam.

- `TimberBolts`

  A group of bolts with washers through a timber beam end at a column.

- `Hanger`

  A bent steel plate under a timber beam end at a beam.


**Methods**

- `pub const ALL: [ConnectionKind; 5]`
- `pub fn code(self) -> &'static str`
- `pub fn from_code(code: &str) -> Option<Self>`
- `pub fn key(self) -> &'static str`

  The i18n key of its name.

- `pub fn at_column(self) -> bool`

  Whether it sits at a column's base (else at a beam's start or end).

- `pub fn material(self) -> StructuralMaterial`

  The Structure tab material whose *Connections* group offers it.

- `pub fn defaults(self) -> ConnectionParams`

  The parameters a new connection of this kind gets.


<a id="cc_core-connection-memberend"></a>

### enum `MemberEnd`

```rust
pub enum MemberEnd
```

Which end of its member a connection sits at.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Serialize`, `Deserialize`

**Variants**

- `Base`

  A column's base.

- `Start`

  A beam's start.

- `End`

  A beam's end.


**Methods**

- `pub const ALL: [MemberEnd; 3]`
- `pub fn code(self) -> &'static str`
- `pub fn key(self) -> &'static str`

<a id="cc_core-connection-connectionparams"></a>

### struct `ConnectionParams`

```rust
pub struct ConnectionParams
```

The sizes of a connection, meters (see the kind for what each means).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `thickness: f64`

  Plate thickness (a bolted joint's washers).

- `width: f64`

  Plate width; 0: from the member's section plus `edge`.

- `length: f64`

  Plate length; 0: from the member's section (or bolts) plus `edge`.

- `bolt_diameter: f64`
- `rows: u32`
- `columns: u32`
- `spacing: f64`

  Bolt to bolt.

- `edge: f64`

  Bolt to the plate's edge (or to the member's end).

- `anchor_length: f64`

  Anchor bolts below a base plate.


**Methods**

- `pub fn check(&self) -> Result<(), String>`

  Whether every value is in range.

- `pub fn reset_out_of_range(&mut self, kind: ConnectionKind) -> usize`

  Values out of range back to `kind`'s defaults (a bad width or length
  to 0); returns how many changed.


<a id="cc_core-connection-connection"></a>

### struct `Connection`

```rust
pub struct Connection
```

A connection at one end of a column or beam.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `member: ElementId`

  The column or beam.

- `end: MemberEnd`
- `kind: ConnectionKind`
- `params: ConnectionParams`

**Methods**

- `pub fn new(member: ElementId, end: MemberEnd, kind: ConnectionKind) -> Self`

  A connection with the kind's default parameters.

- `pub fn fits(&self, member: &ElementKind) -> bool`

  Whether it can sit on `member` at its end: a base plate on a column's
  base, the other kinds on a beam's start or end.


<a id="cc_core-connection-host_type"></a>

### fn `host_type`

```rust
pub fn host_type(id: ElementTypeId, name: String) -> ElementType
```

The type a project gets with its first connection.

<a id="mod-cc_core-curtain"></a>

## `cc_core::curtain`

<sub>`crates/cc_core/src/curtain.rs`</sub>

Curtain walls: walls made of a grid of panels and mullions.
A wall type with a `CurtainSystem` (`TypeData::Wall::curtain`) is a
curtain wall type: its walls are not layered bodies but a grid laid over
the wall's face, vertical lines spaced along the wall and horizontal lines
up from its base, with a panel in each cell and mullions on the lines and
around the border. Each wall may adjust its grid (`CurtainEdits`): its
own lines instead of the type's, segments of lines removed (the cells on
both sides become one panel) and panels swapped.
`layout` works out the grid of one wall in wall-local coordinates
(`u` along the wall's body centerline from its start, `z` absolute
elevations); `cc_geom::curtain` places it in the world.

**Items:** [`MAX_LINES`](#cc_core-curtain-max_lines) · [`MIN_SPACING`](#cc_core-curtain-min_spacing) · [`LINE_TOL`](#cc_core-curtain-line_tol) · [`GridLayout`](#cc_core-curtain-gridlayout) · [`GridJustification`](#cc_core-curtain-gridjustification) · [`CurtainGrid`](#cc_core-curtain-curtaingrid) · [`CurtainPanel`](#cc_core-curtain-curtainpanel) · [`Mullion`](#cc_core-curtain-mullion) · [`CurtainSystem`](#cc_core-curtain-curtainsystem) · [`GridAxis`](#cc_core-curtain-gridaxis) · [`GridSegment`](#cc_core-curtain-gridsegment) · [`PanelSwap`](#cc_core-curtain-panelswap) · [`CurtainEdits`](#cc_core-curtain-curtainedits) · [`MAX_TEXT`](#cc_core-curtain-max_text) · [`to_text`](#cc_core-curtain-to_text) · [`from_text`](#cc_core-curtain-from_text) · [`PanelCell`](#cc_core-curtain-panelcell) · [`MullionPiece`](#cc_core-curtain-mullionpiece) · [`Layout`](#cc_core-curtain-layout) · [`layout`](#cc_core-curtain-layout)

<a id="cc_core-curtain-max_lines"></a>

### const `MAX_LINES`

```rust
pub const MAX_LINES: usize
```

Most grid lines on one axis of a wall (files are hostile input).

<a id="cc_core-curtain-min_spacing"></a>

### const `MIN_SPACING`

```rust
pub const MIN_SPACING: f64
```

Smallest spacing a layout uses, in meters.

<a id="cc_core-curtain-line_tol"></a>

### const `LINE_TOL`

```rust
pub const LINE_TOL: f64
```

Positions within this distance are the same grid line.

<a id="cc_core-curtain-gridlayout"></a>

### enum `GridLayout`

```rust
pub enum GridLayout
```

How a grid spaces its lines over the length (or height) of a wall.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `None`

  No lines: one cell across.

- `FixedDistance`

  A line every `spacing`, from where `justification` says.

- `FixedNumber`

  `count` equal divisions.

- `MaxSpacing`

  As few equal divisions as keep them at most `spacing` wide.


**Methods**

- `pub const ALL: [GridLayout; 4]`
- `pub fn key(self) -> &'static str`

<a id="cc_core-curtain-gridjustification"></a>

### enum `GridJustification`

```rust
pub enum GridJustification
```

Where a fixed-distance grid starts: the remainder goes to the other end
(or half to each end, centered).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Start`
- `Center`
- `End`

**Methods**

- `pub const ALL: [GridJustification; 3]`
- `pub fn key(self) -> &'static str`

<a id="cc_core-curtain-curtaingrid"></a>

### struct `CurtainGrid`

```rust
pub struct CurtainGrid
```

The lines of one direction of a curtain grid.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `layout: GridLayout`
- `spacing: f64`

  Distance between lines (fixed distance) or the widest cell (maximum spacing).

- `count: u32`

  Number of divisions (fixed number).

- `justification: GridJustification`

**Methods**

- `pub fn lines(&self, lo: f64, hi: f64) -> Vec<f64>`

  The interior lines between `lo` and `hi`, ascending (at most `MAX_LINES`).

- `pub fn repair(&mut self) -> usize`

  Fix non-finite or too small values; returns the number of repairs.


<a id="cc_core-curtain-curtainpanel"></a>

### enum `CurtainPanel`

```rust
pub enum CurtainPanel
```

What fills a cell of the grid.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`, `Default`

**Variants**

- `Glass { thickness: f64, material: String }`

  A glazed panel `thickness` thick.

- `Solid { thickness: f64, material: String }`

  An opaque panel `thickness` thick (spandrel, infill).

- `Empty`

  No panel: the cell is open.

- `Component { component: ElementTypeId }`

  A component placed in the cell (a door or window component in its
  opening, other components from the cell's lower left corner), its
  `Width` and `Height` parameters set to the cell's size.


**Methods**

- `pub fn key(&self) -> &'static str`
- `pub fn material(&self) -> Option<&str>`
- `pub fn thickness(&self) -> f64`
- `pub fn is_glazed(&self) -> bool`
- `pub fn repair(&mut self, exists: &dyn Fn(ElementTypeId) -> bool) -> usize`

  Fix a bad thickness, and a component that is no longer a type
  (`exists` says which are); returns the number of repairs.


<a id="cc_core-curtain-mullion"></a>

### struct `Mullion`

```rust
pub struct Mullion
```

A rectangular mullion section: `width` seen on the face of the wall,
`depth` across it.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `width: f64`
- `depth: f64`
- `material: String`

<a id="cc_core-curtain-curtainsystem"></a>

### struct `CurtainSystem`

```rust
pub struct CurtainSystem
```

The curtain grid of a wall type.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `vertical: CurtainGrid`

  Vertical lines, spaced along the wall.

- `horizontal: CurtainGrid`

  Horizontal lines, spaced up from the wall's base.

- `panel: CurtainPanel`

  What fills the cells.

- `vertical_mullion: Option<Mullion>`

  Mullions on the interior vertical lines.

- `horizontal_mullion: Option<Mullion>`

  Mullions on the interior horizontal lines.

- `border_mullion: Option<Mullion>`

  Mullions around the border.


**Methods**

- `pub fn with_materials(glass: &str, frame: &str) -> Self`

  A system whose panels and mullions are of the given materials.

- `pub fn thickness(&self) -> f64`

  Thickness of walls of this type: the deepest mullion or the panel (at least 1 cm).

- `pub fn main_material(&self) -> &str`

  The material of the frame (the first mullion with one), else the panel's.

- `pub fn materials(&self) -> Vec<&str>`

  Material names referenced.

- `pub fn rename_material(&mut self, old: &str, new: &str)`
- `pub fn components(&self) -> Vec<ElementTypeId>`

  The component types used as panels.

- `pub fn repair(&mut self, exists: &dyn Fn(ElementTypeId) -> bool) -> usize`

<a id="cc_core-curtain-gridaxis"></a>

### enum `GridAxis`

```rust
pub enum GridAxis
```

A direction of the grid: vertical lines (spaced along the wall) or
horizontal ones (spaced up it).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Serialize`, `Deserialize`

**Variants**

- `Vertical`
- `Horizontal`

<a id="cc_core-curtain-gridsegment"></a>

### struct `GridSegment`

```rust
pub struct GridSegment
```

A removed piece of an interior grid line: the line at `at` (`u` of a
vertical line, height above the base of a horizontal one), between the
two lines across it on both sides of `along` (a height for a vertical
line, a `u` for a horizontal one).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `axis: GridAxis`
- `at: f64`
- `along: f64`

<a id="cc_core-curtain-panelswap"></a>

### struct `PanelSwap`

```rust
pub struct PanelSwap
```

A panel other than the type's, in the cell (or merged cells) holding the
point `u` along the wall and `z` above its base.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `u: f64`
- `z: f64`
- `panel: CurtainPanel`

<a id="cc_core-curtain-curtainedits"></a>

### struct `CurtainEdits`

```rust
pub struct CurtainEdits
```

One wall's adjustments to its type's grid. Positions are wall-local:
`u` along the body centerline from its start, heights above its base.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `vertical: Option<Vec<f64>>`

  The wall's own vertical lines instead of the type's.

- `horizontal: Option<Vec<f64>>`

  The wall's own horizontal lines instead of the type's.

- `removed: Vec<GridSegment>`
- `panels: Vec<PanelSwap>`

**Methods**

- `pub fn is_empty(&self) -> bool`
- `pub fn reverse(&mut self, length: f64)`

  The same edits for the wall walked the other way (`length` long).

- `pub fn materials(&self) -> impl Iterator<Item = &str>`
- `pub fn rename_material(&mut self, old: &str, new: &str)`
- `pub fn components(&self) -> impl Iterator<Item = ElementTypeId> + '_`

  The component types used as panels.

- `pub fn repair(&mut self, exists: &dyn Fn(ElementTypeId) -> bool) -> usize`

  Drop non-finite values and keep the lists bounded; returns the number of repairs.


<a id="cc_core-curtain-max_text"></a>

### const `MAX_TEXT`

```rust
pub const MAX_TEXT: usize
```

Longest text `from_text` reads (files are hostile input).

<a id="cc_core-curtain-to_text"></a>

### fn `to_text`

```rust
pub fn to_text<T: Serialize>(value: &T) -> String
```

A curtain system or a wall's edits as text (for exchange files, such as
an IFC property). Component panels keep their type id, which only means
something in the same project.

<a id="cc_core-curtain-from_text"></a>

### fn `from_text`

```rust
pub fn from_text<T: serde::de::DeserializeOwned>(text: &str) -> Option<T>
```

Read what `to_text` wrote; `None` when it is too long or not valid.

<a id="cc_core-curtain-panelcell"></a>

### struct `PanelCell`

```rust
pub struct PanelCell
```

A cell, or cells merged where segments were removed, and its panel.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `column: usize`

  Its lowest column and row (cells are numbered from the start and from the base).

- `row: usize`
- `cell_u: (f64, f64)`

  The span between the grid lines around it (absolute elevations).

- `cell_z: (f64, f64)`
- `u: (f64, f64)`

  The panel's own extent: the cell less the mullions around it.

- `z: (f64, f64)`
- `panel: CurtainPanel`
- `swapped: bool`

  Swapped on this wall rather than the type's.


**Methods**

- `pub fn area(&self) -> f64`

<a id="cc_core-curtain-mullionpiece"></a>

### struct `MullionPiece`

```rust
pub struct MullionPiece
```

A straight piece of mullion: along a vertical line at `at` from `span.0`
to `span.1` (heights), or along a horizontal line at height `at` from
`span.0` to `span.1` (along the wall); `offset` is where its section's
center lies from the line, along the wall or up.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `axis: GridAxis`
- `border: bool`
- `at: f64`
- `offset: f64`
- `span: (f64, f64)`
- `mullion: Mullion`

**Methods**

- `pub fn length(&self) -> f64`

<a id="cc_core-curtain-layout"></a>

### struct `Layout`

```rust
pub struct Layout
```

The grid of one curtain wall.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `u: Vec<f64>`

  Vertical lines along the wall, both ends included.

- `z: Vec<f64>`

  Horizontal lines (absolute elevations), base and top included.

- `panels: Vec<PanelCell>`
- `mullions: Vec<MullionPiece>`

**Methods**

- `pub fn columns(&self) -> usize`
- `pub fn rows(&self) -> usize`
- `pub fn panel_at(&self, column: usize, row: usize) -> Option<&PanelCell>`

  The panel covering cell `(column, row)`.

- `pub fn cell_center(&self, column: usize, row: usize) -> Option<(f64, f64)>`

  The center `(u, z)` of cell `(column, row)`.

- `pub fn interior(&self, axis: GridAxis) -> &[f64]`

  Interior lines of an axis (without the borders).

- `pub fn glazed_area(&self) -> f64`
- `pub fn mullion_length(&self) -> f64`
- `pub fn panel_count(&self) -> usize`

  Panels that are there (not empty cells).


<a id="cc_core-curtain-layout"></a>

### fn `layout`

```rust
pub fn layout(system: &CurtainSystem, edits: &CurtainEdits, u: (f64, f64), base: f64, top: f64) -> Layout
```

The grid of a curtain wall of `system`, adjusted by `edits`, over `u`
(along the body centerline, between the ends after joins) and from
`base` to `top` (absolute elevations).

<a id="mod-cc_core-design_option"></a>

## `cc_core::design_option`

<sub>`crates/cc_core/src/design_option.rs`</sub>

Design options: alternative sets of elements within one project.
The project holds `OptionSet`s (say "Entrance" or "Kitchen layout"),
each with an ordered list of `DesignOption`s and one *primary* option.
An element belongs to the main model (`Element::design_option` is
`None`), shown with every option, or to exactly one option. A view shows
the main model plus one option of each set: the one it chooses
(`View::design_options`), else the option being edited when it belongs to
that set (`Project::active_option`), else the set's primary option
("automatic"). Elements of the other options are left out of the view
altogether: drawings, schedules, joins and room boundaries.
Hosted elements follow their hosts (a door is in its wall's option, see
`Project::option_fixes`); annotations belong to no option. Deleting an
option deletes its elements; accepting a set's primary option moves its
elements into the main model and deletes the set with its other options
and their elements. IFC export writes the main model and the primary
options only.

**Items:** [`MAX_OPTION_SETS`](#cc_core-design_option-max_option_sets) · [`MAX_OPTIONS`](#cc_core-design_option-max_options) · [`DesignOption`](#cc_core-design_option-designoption) · [`OptionSet`](#cc_core-design_option-optionset)

<a id="cc_core-design_option-max_option_sets"></a>

### const `MAX_OPTION_SETS`

```rust
pub const MAX_OPTION_SETS: usize
```

Most option sets in a project.

<a id="cc_core-design_option-max_options"></a>

### const `MAX_OPTIONS`

```rust
pub const MAX_OPTIONS: usize
```

Most options in a set.

<a id="cc_core-design_option-designoption"></a>

### struct `DesignOption`

```rust
pub struct DesignOption
```

One alternative of an option set.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `id: OptionId`
- `name: String`

<a id="cc_core-design_option-optionset"></a>

### struct `OptionSet`

```rust
pub struct OptionSet
```

A named group of mutually exclusive design options.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `id: OptionSetId`
- `name: String`
- `options: Vec<DesignOption>`

  The alternatives, in display order (never empty in a repaired project).

- `primary: OptionId`

  The option views show by default and exports write.


**Methods**

- `pub fn option(&self, id: OptionId) -> Option<&DesignOption>`
- `pub fn contains(&self, id: OptionId) -> bool`
- `pub fn primary_option(&self) -> Option<OptionId>`

  The primary option (the first one when the primary is unknown).


<a id="mod-cc_core-detail"></a>

## `cc_core::detail`

<sub>`crates/cc_core/src/detail.rs`</sub>

Detail lines: view-specific 2D curves (lines, arcs, circles and whole or
partial ellipses, the `SketchCurve` geometry) drawn in one view with a
line style. Like text notes they live in their view's drawing
coordinates (meters), are owned by it (`Element::owner_view`) and go
with it; they belong to no phase-dependent geometry, take no part in the
model's 3D or IFC, and are drawn in drafting views and on top of plans,
ceiling plans, elevations, sections and legends.
A polyline is drawn as one element per segment, so each segment is an
ordinary curve that the sketch edits (`crate::sketch_edit`: trim,
extend, split, offset) and grips change on its own.
The line style is a `LineStyleRef`: one of the built-in
`DetailStyle`s or one of the project's own line styles
(`LineStyleDef`: a name, weight, color and dash pattern, kept in
`Project::line_styles` and tracked as project settings). Region
boundaries (`crate::region`) take the same references. A reference to
a project style that no longer exists reads as the thin built-in style,
and repair resets it.

**Items:** [`DetailLine`](#cc_core-detail-detailline) · [`MAX_COORD`](#cc_core-detail-max_coord) · [`LineStyles`](#cc_core-detail-linestyles) · [`LineStyleRef`](#cc_core-detail-linestyleref) · [`MAX_LINE_STYLES`](#cc_core-detail-max_line_styles) · [`MIN_WEIGHT_MM`](#cc_core-detail-min_weight_mm) · [`MAX_WEIGHT_MM`](#cc_core-detail-max_weight_mm) · [`LineStyleDef`](#cc_core-detail-linestyledef) · [`DetailStyle`](#cc_core-detail-detailstyle)

<a id="cc_core-detail-detailline"></a>

### struct `DetailLine`

```rust
pub struct DetailLine
```

A detail line of one view.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `view: ViewId`

  The view that owns (and alone shows) the line.

- `curve: SketchCurve`

  The curve, in the view's drawing coordinates (meters).

- `style: LineStyleRef`

  How the line is drawn.


**Methods**

- `pub fn is_valid(&self) -> bool`

  Finite numbers and a size (see `SketchCurve::is_valid`), within bounds.

- `pub fn snap_features(&self, out: &mut crate::cad::CadSnap)`

  What drawing tools snap to on the line (in the shape of an imported
  drawing's snap features): ends, middles, centers, quadrant and axis
  points, and the lines and arcs themselves (ellipses by their chords).


<a id="cc_core-detail-max_coord"></a>

### const `MAX_COORD`

```rust
pub const MAX_COORD: f64
```

Coordinates and sizes past this (meters) are refused (a crafted file).

<a id="cc_core-detail-linestyles"></a>

### type alias `LineStyles`

```rust
pub type LineStyles = BTreeMap<LineStyleId, LineStyleDef>
```

The project's line styles, by id.

<a id="cc_core-detail-linestyleref"></a>

### enum `LineStyleRef`

```rust
pub enum LineStyleRef
```

The style a detail line or a region's boundary is drawn with.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Serialize`, `Deserialize`, `Default`

**Variants**

- `BuiltIn(DetailStyle)`

  One of the built-in styles.

- `Project(LineStyleId)`

  One of the project's line styles (`Project::line_styles`).


**Methods**

- `pub fn pen(self, styles: &LineStyles) -> (f64, [u8; 3], LinePattern)`

  Width on paper (millimeters), color and dash pattern; a project
  style that is gone reads as the default built-in style.

- `pub fn code(self, styles: &LineStyles) -> String`

  The style's name as a code (DXF layers, commands): a built-in
  style's code, a project style's name.

- `pub fn exists(self, styles: &LineStyles) -> bool`

  Whether the reference is to a style that exists.


<a id="cc_core-detail-max_line_styles"></a>

### const `MAX_LINE_STYLES`

```rust
pub const MAX_LINE_STYLES: usize
```

Most project line styles (a crafted file).

<a id="cc_core-detail-min_weight_mm"></a>

### const `MIN_WEIGHT_MM`

```rust
pub const MIN_WEIGHT_MM: f64
```

Range of a line style's weight, paper millimeters.

<a id="cc_core-detail-max_weight_mm"></a>

### const `MAX_WEIGHT_MM`

```rust
pub const MAX_WEIGHT_MM: f64
```

<a id="cc_core-detail-linestyledef"></a>

### struct `LineStyleDef`

```rust
pub struct LineStyleDef
```

One of the project's line styles: a name, a weight on paper, a color and
a dash pattern. Names are unique and differ from the built-in codes, so
commands can name either.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `id: LineStyleId`
- `name: String`
- `weight_mm: f64`

  Width on paper, millimeters.

- `color: [u8; 3]`
- `pattern: LinePattern`

**Methods**

- `pub fn repair(&mut self) -> bool`

  Bring the weight into range (a crafted file); returns whether it changed.


<a id="cc_core-detail-detailstyle"></a>

### enum `DetailStyle`

```rust
pub enum DetailStyle
```

The built-in line styles of detail lines.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Thin`

  Fine continuous line.

- `Medium`
- `Wide`
- `Hidden`

  Short dashes.

- `Dashed`

  Long dashes.

- `Center`

  Dash-dot (center lines, axes).

- `Dotted`

**Methods**

- `pub const ALL: [DetailStyle; 7]`
- `pub fn weight_mm(self) -> f64`

  Width on paper, millimeters.

- `pub fn pattern(self) -> LinePattern`
- `pub fn code(self) -> &'static str`

  The name used by commands and DXF layers.

- `pub fn from_code(code: &str) -> Option<Self>`
- `pub fn key(self) -> &'static str`

  Translation key of the name.


<a id="mod-cc_core-detail_component"></a>

## `cc_core::detail_component`

<sub>`crates/cc_core/src/detail_component.rs`</sub>

Detail components: reusable 2D components for construction details.
A detail component is a component of kind `ComponentKind::Detail`
(category `DetailComponent`, `TypeData::Component`) made only of 2D
pieces, the same `SheetItem`s title blocks are drawn with
(`ComponentDef::sheet`): strokes of lines and arcs (open or closed,
with a pen weight in paper millimeters, a color, a dash pattern, and a
solid or pattern fill when closed), texts and pictures, plus
`SheetShape::Nested` pieces that place another detail component
inside this one. Unlike a title block's, every length is in meters in
the component's own frame (text heights too), and expressions read the
component's parameters.
An element of kind `ElementKind::DetailComponent` places a type in one
view (`view`, like detail lines and text notes; removed with it): its
origin at `position` in the view's drawing coordinates, turned
`rotation_deg`, sized by `scale` and, when `mirrored`, flipped across
its own X axis before turning. It has no phase, no 3D and no IFC.
Nested components are named by their component name and read the
parameter values of that component's first type; resolving them
(`walk`) is bounded in depth, in visits and in drawn points, and a
component met again inside itself is left out (a cycle).

**Items:** [`MIN_SCALE`](#cc_core-detail_component-min_scale) · [`MAX_SCALE`](#cc_core-detail_component-max_scale) · [`MAX_DEPTH`](#cc_core-detail_component-max_depth) · [`MAX_VISITS`](#cc_core-detail_component-max_visits) · [`MAX_POINTS`](#cc_core-detail_component-max_points) · [`scale_ok`](#cc_core-detail_component-scale_ok) · [`DetailComponent`](#cc_core-detail_component-detailcomponent) · [`Place`](#cc_core-detail_component-place) · [`nested_names`](#cc_core-detail_component-nested_names) · [`Visit`](#cc_core-detail_component-visit) · [`walk`](#cc_core-detail_component-walk) · [`stroke_of`](#cc_core-detail_component-stroke_of) · [`ring_stroke`](#cc_core-detail_component-ring_stroke) · [`def_of`](#cc_core-detail_component-def_of)

<a id="cc_core-detail_component-min_scale"></a>

### const `MIN_SCALE`

```rust
pub const MIN_SCALE: f64
```

Range of a placement's scale.

<a id="cc_core-detail_component-max_scale"></a>

### const `MAX_SCALE`

```rust
pub const MAX_SCALE: f64
```

<a id="cc_core-detail_component-max_depth"></a>

### const `MAX_DEPTH`

```rust
pub const MAX_DEPTH: usize
```

Deepest nesting followed (the element's own component is depth 0).

<a id="cc_core-detail_component-max_visits"></a>

### const `MAX_VISITS`

```rust
pub const MAX_VISITS: usize
```

Most component resolutions one element's drawing makes.

<a id="cc_core-detail_component-max_points"></a>

### const `MAX_POINTS`

```rust
pub const MAX_POINTS: usize
```

Most points of strokes one element's drawing draws in all.

<a id="cc_core-detail_component-scale_ok"></a>

### fn `scale_ok`

```rust
pub fn scale_ok(s: f64) -> bool
```

Whether `s` is a usable scale.

<a id="cc_core-detail_component-detailcomponent"></a>

### struct `DetailComponent`

```rust
pub struct DetailComponent
```

A detail component placed in one view.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `view: ViewId`

  The view that owns (and alone shows) it.

- `position: DVec2`

  Where the component's origin lands, view coordinates (meters).

- `rotation_deg: f64`

  Degrees counter-clockwise.

- `scale: f64`

  Size factor of the whole component.

- `mirrored: bool`

  Flipped across its own X axis (before turning).

- `overrides: BTreeMap<String, f64>`

  Values of instance parameters that differ from the type.


**Methods**

- `pub fn new(view: ViewId, position: DVec2) -> Self`
- `pub fn is_valid(&self) -> bool`

  Finite numbers within bounds and a usable scale.

- `pub fn place(&self) -> Place`

  Where the component's frame goes in the view.

- `pub fn transform(&mut self, t: &PlanTransform)`

  Move, rotate or mirror the placement with the view's contents.


<a id="cc_core-detail_component-place"></a>

### struct `Place`

```rust
pub struct Place
```

A similarity of the plane: scale by `scale`, then the rigid `t`.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`

**Fields**

- `t: PlanTransform`
- `scale: f64`

**Methods**

- `pub fn apply(&self, p: DVec2) -> DVec2`
- `pub fn apply_vector(&self, v: DVec2) -> DVec2`
- `pub fn then(&self, next: &Place) -> Place`

  `self` followed by `next`.

- `pub fn nested(&self, n: &crate::title_block::SheetNested) -> Place`

  The placement of a nested piece, resolved in the frame `self` maps.

- `pub fn apply_angle(&self, angle: f64) -> f64`

  A direction given in radians, through the placement.


<a id="cc_core-detail_component-nested_names"></a>

### fn `nested_names`

```rust
pub fn nested_names(def: &ComponentDef) -> impl Iterator<Item = &str>
```

The names of the components `def` nests directly.

<a id="cc_core-detail_component-visit"></a>

### struct `Visit`

```rust
pub struct Visit<'a>
```

What `walk` visits: each resolution's pieces, where they go, and the
nesting depth (0 for the element's own component).

**Fields**

- `pieces: &'a SheetPieces`
- `place: Place`
- `depth: usize`

<a id="cc_core-detail_component-walk"></a>

### fn `walk`

```rust
pub fn walk<'d>(lookup: &dyn Fn(&str) -> Option<&'d ComponentDef>, name: &str, def: &ComponentDef, overrides: &BTreeMap<String, f64>, place: Place, visit: &mut dyn FnMut(Visit)) -> bool
```

Resolve `def` with `overrides` and visit its pieces through `place`,
then each nested component's (found by `lookup`), depth first. A
component already being resolved (a cycle), nesting past `MAX_DEPTH`,
more than `MAX_VISITS` resolutions or strokes past `MAX_POINTS`
points are left out; returns whether everything was visited.

<a id="cc_core-detail_component-stroke_of"></a>

### fn `stroke_of`

```rust
pub fn stroke_of(curve: &SketchCurve) -> Option<SheetShape>
```

A stroke piece that draws `curve` (in the component's frame, meters):
lines and arcs as two points (a longer arc three), circles and whole
ellipses as closed strokes.

<a id="cc_core-detail_component-ring_stroke"></a>

### fn `ring_stroke`

```rust
pub fn ring_stroke(ring: &[DVec2]) -> Option<SheetShape>
```

A closed stroke along `ring` (straight sides).

<a id="cc_core-detail_component-def_of"></a>

### fn `def_of`

```rust
pub fn def_of(pieces: Vec<SheetItem>) -> ComponentDef
```

A new detail component definition from `pieces`.

<a id="mod-cc_core-dim_line"></a>

## `cc_core::dim_line`

<sub>`crates/cc_core/src/dim_line.rs`</sub>

The dimension line of linear and aligned dimensions: lining it up with
the lines of other dimensions, and chains of dimensions that share one
line (continuous dimensioning).
The line of a dimension with direction `d` (unit) is the set of points
`x` with `x · n = a · n + offset`, where `n = d.perp()` (the left of `d`).

**Items:** [`PARALLEL_SIN`](#cc_core-dim_line-parallel_sin) · [`extent`](#cc_core-dim_line-extent) · [`offset_through`](#cc_core-dim_line-offset_through) · [`LineAlignment`](#cc_core-dim_line-linealignment) · [`align`](#cc_core-dim_line-align) · [`chain`](#cc_core-dim_line-chain) · [`ordinate`](#cc_core-dim_line-ordinate)

<a id="cc_core-dim_line-parallel_sin"></a>

### const `PARALLEL_SIN`

```rust
pub const PARALLEL_SIN: f64
```

Largest sine of the angle between two dimension lines that still counts
as parallel (about 0.06°).

<a id="cc_core-dim_line-extent"></a>

### fn `extent`

```rust
pub fn extent(dim: &Dimension) -> Option<(DVec2, DVec2)>
```

The drawn extent of a linear or aligned dimension's line: where the
witness lines of `a` and `b` meet it. `None` for other kinds or a
degenerate direction.

<a id="cc_core-dim_line-offset_through"></a>

### fn `offset_through`

```rust
pub fn offset_through(dim: &Dimension, p: DVec2) -> Option<f64>
```

The offset (from `dim.a`, left of its direction positive) that puts the
line of `dim` through `p`.

<a id="cc_core-dim_line-linealignment"></a>

### struct `LineAlignment`

```rust
pub struct LineAlignment
```

The line of `dim` lined up with another dimension's.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `offset: f64`

  The offset of `dim` that puts its line on the other line.

- `reference: DVec2`

  The end of the other dimension's line nearest to `dim` (where a guide starts).

- `point: DVec2`

  The point of the aligned line of `dim` nearest to `reference` (where the guide ends).


<a id="cc_core-dim_line-align"></a>

### fn `align`

```rust
pub fn align<'a>(dim: &Dimension, others: impl IntoIterator<Item = &'a Dimension>, tol: f64) -> Option<LineAlignment>
```

Snap the line of `dim` (a linear or aligned dimension being placed, at
its current offset) onto the line of a parallel dimension of `others`
when it lies within `tol` (model units) of it. The nearest line wins;
`None` when no parallel line is close enough.

<a id="cc_core-dim_line-chain"></a>

### fn `chain`

```rust
pub fn chain(view: ViewId, points: &[DVec2], dir: DVec2, offset: f64, aligned: bool) -> Option<Vec<Dimension>>
```

The dimensions of a chain through `points` measured along `dir`, all on
the line at `offset` from the first point (left of `dir` positive): one
dimension from each point to the next. With `aligned`, a segment along
`dir` (or against it) is an aligned dimension, so it keeps following the
direction of what it measures; any other segment (and every segment
without `aligned`) is a linear dimension along `dir`. `None` when `dir`
is degenerate, a value is not finite, or a segment measures nothing
along `dir` (shorter than 1e-6).

<a id="cc_core-dim_line-ordinate"></a>

### fn `ordinate`

```rust
pub fn ordinate(view: ViewId, points: &[DVec2], dir: DVec2, offset: f64) -> Option<Vec<Dimension>>
```

The running (ordinate) dimensions from the origin `points[0]` to each
other point, measured along `dir`, all on the line at `offset` from the
origin (left of `dir` positive). `None` when `dir` is degenerate, a value
is not finite, or a point measures nothing from the origin along `dir`
(closer than 1e-6).

<a id="mod-cc_core-dim_refs"></a>

## `cc_core::dim_refs`

<sub>`crates/cc_core/src/dim_refs.rs`</sub>

Associative dimensions: each end of a dimension may reference a point
of an element (stored in that element's own frame, plus a height in
elevations and sections) or a level. Every edit ends by recomputing
referenced ends (`crate::EditCtx`), inside the same transaction, so
dimensions follow moved, rotated, stretched or raised elements and
levels, and undo restores both together.

**Items:** [`DimRef`](#cc_core-dim_refs-dimref) · [`RefTarget`](#cc_core-dim_refs-reftarget) · [`RefAnchor`](#cc_core-dim_refs-refanchor) · [`RefHeight`](#cc_core-dim_refs-refheight) · [`Projection`](#cc_core-dim_refs-projection)

<a id="cc_core-dim_refs-dimref"></a>

### struct `DimRef`

```rust
pub struct DimRef
```

What a dimension end follows.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `target: RefTarget`
- `anchor: RefAnchor`
- `z: Option<RefHeight>`

  Height in elevations and sections (`None` in plans).


**Methods**

- `pub fn element(&self) -> Option<ElementId>`

  The referenced element, if the target is one.

- `pub(crate) fn is_finite(&self) -> bool`
- `pub(crate) fn mirrored(self, target: &ElementKind) -> DimRef`

  The reference to the mirror image of this point once `target` (the
  referenced element, already mirrored) went through a mirror: a wall
  is walked the other way, beams, grids, columns and components keep
  their direction, and an opening's frame follows its wall.

- `pub(crate) fn target_exists(&self, project: &Project) -> bool`

  Whether the target still exists in `project`.


<a id="cc_core-dim_refs-reftarget"></a>

### enum `RefTarget`

```rust
pub enum RefTarget
```

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Serialize`, `Deserialize`

**Variants**

- `Element(ElementId)`
- `Level(LevelId)`

  A level line of an elevation or section (only its height counts).


<a id="cc_core-dim_refs-refanchor"></a>

### enum `RefAnchor`

```rust
pub enum RefAnchor
```

Where the point sits in the element's frame.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`

**Variants**

- `Line { from_end: bool, along: f64, across: f64 }`

  On a wall, beam or grid: `along` meters from its start (or from its
  end when `from_end`, measured back toward the start), `across`
  meters to the left of its line.

- `Local { x: f64, y: f64 }`

  In the frame of a column or component (its position and rotation).

- `Hosted { along: f64, across: f64 }`

  On a door or window: `along` its wall from the opening center,
  `across` to the left of the wall's line.

- `Level`

  A level: the end keeps its position along the view.


<a id="cc_core-dim_refs-refheight"></a>

### struct `RefHeight`

```rust
pub struct RefHeight
```

Height of a referenced point: `dz` above the element's base, or above
its top when `from_top` (walls), so it follows height changes.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `from_top: bool`
- `dz: f64`

<a id="cc_core-dim_refs-projection"></a>

### struct `Projection`

```rust
pub struct Projection
```

How elevations and sections map the model: x along `right` from
`origin` (plan), y the height.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `origin: DVec2`
- `right: DVec2`

**Methods**

- `pub fn project(&self, p: DVec2, z: f64) -> DVec2`

<a id="mod-cc_core-dim_style"></a>

## `cc_core::dim_style`

<sub>`crates/cc_core/src/dim_style.rs`</sub>

How a dimension type draws its dimensions: the marks at the ends of the
dimension line, the witness lines, the pens and the text around the
value. Sizes are paper millimeters, so they read the same at any scale.

**Items:** [`MAX_LENGTH_MM`](#cc_core-dim_style-max_length_mm) · [`MIN_MARK_MM`](#cc_core-dim_style-min_mark_mm) · [`MAX_AFFIX`](#cc_core-dim_style-max_affix) · [`DimensionStyle`](#cc_core-dim_style-dimensionstyle)

<a id="cc_core-dim_style-max_length_mm"></a>

### const `MAX_LENGTH_MM`

```rust
pub const MAX_LENGTH_MM: f64
```

Most paper millimeters any length of the style may take.

<a id="cc_core-dim_style-min_mark_mm"></a>

### const `MIN_MARK_MM`

```rust
pub const MIN_MARK_MM: f64
```

Smallest mark.

<a id="cc_core-dim_style-max_affix"></a>

### const `MAX_AFFIX`

```rust
pub const MAX_AFFIX: usize
```

Most characters of the prefix or the suffix.

<a id="cc_core-dim_style-dimensionstyle"></a>

### struct `DimensionStyle`

```rust
pub struct DimensionStyle
```

The drawing parameters of a dimension type. Every field has a default,
so types saved before them draw as they did.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `mark: LeaderSymbol`

  The mark at each end of the dimension line (linear and angular
  dimensions; radial ones keep their arrows).

- `mark_size: f64`

  Length of the mark: the stroke of a tick, the depth of an arrow.

- `mark_weight: Option<f64>`

  Pen width of the mark; `None` draws ticks heavy and other marks like the lines.

- `line_weight: Option<f64>`

  Pen width of the dimension and witness lines; `None` is the annotation pen.

- `color: Option<[u8; 3]>`

  Color of the lines and marks; `None` is the annotation color.

- `witness_gap: f64`

  Gap between the measured point and the start of its witness line.

- `witness_extension: f64`

  How far witness lines run past the dimension line.

- `line_extension: f64`

  How far the dimension line runs past the witness lines.

- `text_offset: f64`

  Distance from the dimension line to the bottom of the text.

- `prefix: String`

  Written before the value.

- `suffix: String`

  Written after the value.

- `opaque_text: bool`

  The text hides what lies under it (a white box behind it).


**Methods**

- `pub fn of(system: crate::MeasurementSystem) -> Self`

  The default style of a project of `system`: in imperial projects
  round inches on paper (1/8" marks and extensions, 1/16" gaps, the
  text 1/32" off the line).

- `pub fn label(&self, value: &str) -> String`

  `value` written with the prefix and the suffix.

- `pub fn repair(&mut self) -> usize`

  Reset what is out of range; returns the number of repairs.


<a id="mod-cc_core-document"></a>

## `cc_core::document`

<sub>`crates/cc_core/src/document.rs`</sub>

**Items:** [`Transaction`](#cc_core-document-transaction) · [`EditCtx`](#cc_core-document-editctx) · [`Document`](#cc_core-document-document)

<a id="cc_core-document-transaction"></a>

### struct `Transaction`

```rust
pub struct Transaction
```

A recorded, reversible set of changes.

**Traits:** `Debug`, `Clone`, `Default`

**Fields**

- `name: String`

*Some fields are private.*

**Methods**

- `pub fn is_empty(&self) -> bool`
- `pub fn changed_elements(&self) -> impl Iterator<Item = ElementId> + '_`

  Element ids touched by this transaction.


<a id="cc_core-document-editctx"></a>

### struct `EditCtx`

```rust
pub struct EditCtx<'a>
```

Mutable access to a project inside `Document::edit`; records the
original state of everything it touches.

*Some fields are private.*

**Methods**

- `pub(crate) fn sync_dimensions(&mut self)` *(in `cc_core::dim_refs`)*

  Move the referenced ends of every dimension to their targets'
  current points (and an angle's vertex to the intersection of its
  lines); references to deleted targets are dropped (the end stays
  where it was). Only changed dimensions are touched.

- `pub fn project(&self) -> &Project`
- `pub fn alloc_id(&mut self) -> u64`
- `pub(crate) fn element_changes(&self) -> Vec<(ElementId, Option<&Element>)>`

  The elements this edit has changed so far, each with its state before
  the edit (`None`: it was added). Elements touched but left as they
  were are skipped.

- `pub(crate) fn changed_types(&self) -> std::collections::BTreeSet<ElementTypeId>`

  The types this edit has added, removed or changed so far.

- `pub fn add_element(&mut self, mut element: Element) -> ElementId`

  Insert an element. Its `id` must come from `EditCtx::alloc_id`.

- `pub fn element_mut(&mut self, id: ElementId) -> Option<&mut Element>`
- `pub fn remove_element(&mut self, id: ElementId)`

  Remove an element and everything it hosts. Iterative with a visited set,
  so cyclic hosting in a crafted file cannot recurse forever.

- `pub fn add_view(&mut self, view: View) -> ViewId`
- `pub fn view_mut(&mut self, id: ViewId) -> Option<&mut View>`
- `pub fn remove_view(&mut self, id: ViewId)`

  Remove a view, the elements it owns (dimensions, text notes,
  imported drawings) and any sheet viewports showing it.

- `pub fn add_type(&mut self, mut element_type: ElementType) -> ElementTypeId`
- `pub fn type_mut(&mut self, id: ElementTypeId) -> Option<&mut ElementType>`
- `pub fn remove_type(&mut self, id: ElementTypeId)`

  Remove a type (callers make sure no element uses it).

- `pub fn info_mut(&mut self) -> &mut ProjectInfo`
- `pub fn levels_mut(&mut self) -> &mut Vec<Level>`

  Levels; callers should keep them sorted by elevation.

- `pub fn materials_mut(&mut self) -> &mut Vec<Material>`
- `pub fn rename_material(&mut self, old: &str, new: &str) -> bool`

  Rename a material and every type reference to it. Returns false
  (and changes nothing) when `new` is empty or already taken.

- `pub fn set_pinned(&mut self, ids: &[ElementId], pinned: bool) -> usize`

  Pin or unpin elements; returns how many changed.

- `pub fn set_joined(&mut self, cutting: ElementId, cut: ElementId, joined: bool) -> bool`

  Join the geometry of two elements, `cutting` keeping their common
  volume (an existing join between them is replaced), or unjoin them.
  Returns whether anything changed.

- `pub fn add_group(&mut self, name: String, members: std::collections::BTreeSet<ElementId>) -> crate::GroupId`

  Group `members` (taking them out of any group they were in) under
  `name`; returns the new group's id.

- `pub fn remove_group(&mut self, id: crate::GroupId) -> bool`

  Dissolve a group (its elements stay); returns whether it existed.

- `pub fn systems_mut(&mut self) -> &mut BTreeMap<crate::SystemId, crate::MepSystem>`

  Plumbing systems and electrical circuits.

- `pub fn filters_mut(&mut self) -> &mut BTreeMap<crate::FilterId, crate::ViewFilter>`

  View filters.

- `pub fn templates_mut(&mut self) -> &mut BTreeMap<crate::TemplateId, crate::ViewTemplate>`

  View templates (views that follow one are updated when the edit ends).

- `pub fn browser_mut(&mut self) -> &mut crate::browser::BrowserOrganization`

  The project browser's schemes and the one in use.

- `pub fn bar_marking_mut(&mut self) -> &mut crate::bar_marks::BarMarking`

  How the project's reinforcing bars are marked.

- `pub fn line_styles_mut(&mut self) -> &mut crate::detail::LineStyles`

  The project's line styles.

- `pub fn sun_presets_mut(&mut self) -> &mut Vec<crate::sun::SunPreset>`

  The project's sun presets (see `crate::sun`).

- `pub fn fill_patterns_mut(&mut self) -> &mut crate::fill_pattern::FillPatterns`

  The project's fill patterns.

- `pub fn profiles_mut(&mut self) -> &mut BTreeMap<crate::ProfileId, crate::structural_profile::ProfileDef>`

  The project's structural profiles.

- `pub fn phases_mut(&mut self) -> &mut Vec<crate::Phase>`

  Construction phases, in time order.

- `pub fn phase_filters_mut(&mut self) -> &mut BTreeMap<crate::PhaseFilterId, crate::PhaseFilter>`

  Phase filters.

- `pub fn set_graph_binding(&mut self, id: ElementId, binding: Option<crate::graph_binding::GraphBinding>)`

  Graphics of each phase status.
  Bind an element to the graph call that made it, or unbind it with `None`.

- `pub fn phase_graphics_mut(&mut self) -> &mut crate::PhaseGraphics`
- `pub fn option_sets_mut(&mut self) -> &mut Vec<crate::OptionSet>`

  Design option sets (see `crate::design_option`).

- `pub fn units_mut(&mut self) -> &mut crate::Units`
- `pub(crate) fn sync_member_hosts(&mut self)` *(in `cc_core::member_host`)*

  Keep member hosts and their members in step, from what this edit
  itself changed (it runs before the syncs that touch elements):

  1. a member the edit deleted, or whose kind, type or level it changed
     while leaving its host alone, is detached (its role leaves the
     member list for the detached roles; the element stays as the user
     left it);
  2. a host the edit added or changed, or whose member types it
     changed, or touched one of its inputs (see `MemberHost::inputs`),
     gets the members of its layout (except detached roles):
     existing ones rewritten in place (same ids), missing ones added,
     those whose role is gone deleted. A host the edit added (a copy, a
     paste) keeps none of the members it lists and gets its own: the
     listed elements may be another host's or, after a paste, unrelated
     elements the user owns;
  3. a host whose phases or design option the edit changed passes them
     on to the members it manages (a member's own phase or option edit
     is not a detach, and stays until the host's next change).

- `pub(crate) fn sync_member_phases(&mut self)` *(in `cc_core::member_host`)*

  After the phase and design option syncs: the members of a host whose
  phases or option this edit changed (a wall framing follows its wall
  there, after `Self::sync_member_hosts` ran), and the members this
  edit added (which `sync_options` may have put in the active option),
  take the host's phases and option.

- `pub fn transform_elements(&mut self, ids: &[ElementId], t: &PlanTransform) -> Vec<ElementId>` *(in `cc_core::transform`)*

  Transform `ids` in one step. Doors and windows of transformed walls
  go with them (their offset follows a mirrored wall's new direction);
  hosted elements whose wall is not transformed slide along it by the
  translation's component on the wall. Returns the ids changed.

- `pub fn copy_elements(&mut self, ids: &[ElementId], transforms: &[PlanTransform]) -> Vec<ElementId>` *(in `cc_core::transform`)*

  Copy `ids` once per transform (arrays pass several). Copied walls take
  copies of their doors and windows along; a hosted element copied
  without its wall stays on that wall, shifted along it. Returns the
  new ids.

- `pub fn paste_elements(&mut self, set: &[Element], transforms: &[PlanTransform], level: Option<crate::LevelId>) -> Vec<ElementId>` *(in `cc_core::transform`)*

  Add copies of `set` (from `crate::Project::copy_set`) once per
  transform, on `level` when given (hosted elements whose wall was not
  copied stay on that wall's level). Elements whose type, level, view
  or wall no longer exist are skipped, and walls attached to a missing
  level become unconnected. Copies are built in the creation phase
  (see `crate::phase`). A copied dimension measures the copies of
  what it measured; pasted in place (no move, no level) it keeps
  measuring what was not copied. Returns the new ids.

- `pub fn trim_detail(&mut self, id: ElementId, pick: DVec2, boundary: &SketchCurve) -> Result<(), EditError>` *(in `cc_core::wall_ops`)*

  Trim or extend detail line `id` to the curve `boundary` (where its
  line, circle or ellipse runs), keeping the picked part.

- `pub fn trim_corner(&mut self, a: ElementId, pick_a: DVec2, b: ElementId, pick_b: DVec2) -> Result<(), EditError>` *(in `cc_core::wall_ops`)*

  Trim or extend two elements so they meet at a corner, keeping the
  picked parts. Lines meet where they cross; an arc wall meets a line
  or another arc where its circle crosses it (of two crossings, the one
  on the elements themselves, else the one that changes them least).

- `pub fn trim_to(&mut self, id: ElementId, pick: DVec2, reference: Path) -> Result<(), EditError>` *(in `cc_core::wall_ops`)*

  Trim or extend `id` to the `reference` path (a line, or an arc's
  circle), keeping the picked part.

- `pub fn split(&mut self, id: ElementId, at: DVec2) -> Result<ElementId, EditError>` *(in `cc_core::wall_ops`)*

  Split a wall, beam or grid at the projection of `at`; returns the
  new element (the part after the split). Openings past the split go
  to the new wall.

- `pub fn move_endpoint(&mut self, id: ElementId, at_start: bool, to: DVec2) -> Result<(), EditError>` *(in `cc_core::wall_ops`)*

  Move one end of a wall, beam or grid to `to` (lengthening or
  shortening it). Walls whose ends met that end move with it, so
  corners stay closed; openings keep their place.

- `pub fn offset_copy(&mut self, id: ElementId, distance: f64) -> Result<ElementId, EditError>` *(in `cc_core::wall_ops`)*

  A copy of `id` moved `distance` perpendicular to its line (positive
  to the left of start→end). Openings are not copied.

- `pub fn align_to(&mut self, id: ElementId, reference: Path) -> Result<(), EditError>` *(in `cc_core::wall_ops`)*

  Move `id` (with its openings) onto the `reference` path: a line onto
  a parallel line, an arc wall onto the circle of a concentric arc (its
  radius changes, its sweep stays; openings keep their angles).


<a id="cc_core-document-document"></a>

### struct `Document`

```rust
pub struct Document
```

An open project plus its undo history and file state. Cloned to run a
long command (an import) on another thread while the app keeps drawing.

**Traits:** `Clone`

**Fields**

- `project: Project`
- `path: Option<PathBuf>`
- `creation_phase: Option<crate::PhaseId>`

  The phase new elements are built in (the active view's; `None`:
  the last phase). Display state: not saved, not undone.


*Some fields are private.*

**Methods**

- `pub fn new(mut project: Project) -> Self`
- `pub fn set_view_state(&mut self, f: impl FnOnce(&mut Project))`

  Apply a change that is saved but not undoable, and that no derived
  geometry depends on (a 3D view's camera). The revision stays, so caches
  keyed on it stay valid: callers drop what does depend on the change
  (see `Session::set_view_camera`).

- `pub fn set_active_option(&mut self, option: Option<crate::OptionId>)`

  Set the design option new elements go into, which views without a
  choice of their own show for its set (`None`: the main model). Not
  undone nor saved, but views change: the revision moves on (without
  marking the document as changed).

- `pub fn revision(&self) -> u64`

  Monotonic counter bumped on every model change; use it to invalidate caches.

- `pub fn is_dirty(&self) -> bool`
- `pub fn mark_saved(&mut self)`
- `pub fn mark_unsaved(&mut self)`

  Treat the project as not saved (one restored from a recovery copy,
  whose changes never reached its file).

- `pub fn edit<R>(&mut self, name: impl Into<String>, f: impl FnOnce(&mut EditCtx) -> R) -> R`

  Run `f` as one undoable transaction named `name`.

- `pub fn edit_merged<R>(&mut self, name: impl Into<String>, merge_key: u64, f: impl FnOnce(&mut EditCtx) -> R) -> R`

  Like `Document::edit`, but consecutive edits with the same `merge_key`
  become a single undo step (e.g. while dragging a value).

- `pub fn undo_depth(&self) -> usize`

  Number of undoable transactions.

- `pub fn squash_since(&mut self, depth: usize, name: impl Into<String>) -> bool`

  Fold the undo steps recorded since undo depth `depth` into one step
  named `name` (a command made of other commands, undone at once).
  Changes that cancel out are dropped, and so is the step if nothing is
  left; returns whether a step remains.

- `pub fn break_merge(&mut self)`

  End coalescing so the next merged edit starts a new undo step.

- `pub fn try_edit<R, E>(&mut self, name: impl Into<String>, f: impl FnOnce(&mut EditCtx) -> Result<R, E>, check: impl FnOnce(&Project) -> Result<(), E>) -> Result<R, E>`

  Like `Document::edit`, but the change is kept only if `f` succeeds
  and `check` accepts the resulting project; otherwise the project is
  restored and nothing is recorded (no undo step, redo kept).

- `pub fn try_edit_merged<R, E>(&mut self, name: impl Into<String>, merge_key: Option<u64>, f: impl FnOnce(&mut EditCtx) -> Result<R, E>, check: impl FnOnce(&Project) -> Result<(), E>) -> Result<R, E>`

  `Document::try_edit` that merges into the previous step when
  `merge_key` matches its key (like `Document::edit_merged`).

- `pub fn can_undo(&self) -> bool`
- `pub fn can_redo(&self) -> bool`
- `pub fn undo_name(&self) -> Option<&str>`
- `pub fn redo_name(&self) -> Option<&str>`
- `pub fn undo(&mut self) -> bool`
- `pub fn redo(&mut self) -> bool`

<a id="mod-cc_core-element"></a>

## `cc_core::element`

<sub>`crates/cc_core/src/element.rs`</sub>

**Items:** [`Element`](#cc_core-element-element) · [`ElementKind`](#cc_core-element-elementkind) · [`WallTop`](#cc_core-element-walltop) · [`LocationLine`](#cc_core-element-locationline) · [`Wall`](#cc_core-element-wall) · [`Hosted`](#cc_core-element-hosted) · [`Floor`](#cc_core-element-floor) · [`Ceiling`](#cc_core-element-ceiling) · [`RoofShape`](#cc_core-element-roofshape) · [`Roof`](#cc_core-element-roof) · [`Room`](#cc_core-element-room) · [`Grid`](#cc_core-element-grid) · [`TextNote`](#cc_core-element-textnote) · [`Leader`](#cc_core-element-leader) · [`LeaderSymbol`](#cc_core-element-leadersymbol) · [`Dimension`](#cc_core-element-dimension) · [`DimensionKind`](#cc_core-element-dimensionkind) · [`Column`](#cc_core-element-column) · [`BeamLateral`](#cc_core-element-beamlateral) · [`BeamVertical`](#cc_core-element-beamvertical) · [`Beam`](#cc_core-element-beam) · [`FoundationShape`](#cc_core-element-foundationshape) · [`Foundation`](#cc_core-element-foundation) · [`BarFace`](#cc_core-element-barface) · [`WallSide`](#cc_core-element-wallside) · [`SpacingZone`](#cc_core-element-spacingzone) · [`BarEnd`](#cc_core-element-barend) · [`RebarLayout`](#cc_core-element-rebarlayout) · [`FaceBars`](#cc_core-element-facebars) · [`Splice`](#cc_core-element-splice) · [`Rebar`](#cc_core-element-rebar) · [`ShapedRebar`](#cc_core-element-shapedrebar) · [`Stair`](#cc_core-element-stair) · [`Ramp`](#cc_core-element-ramp) · [`Railing`](#cc_core-element-railing) · [`Component`](#cc_core-element-component)

<a id="cc_core-element-element"></a>

### struct `Element`

```rust
pub struct Element
```

A model element instance. Coordinates are in meters; X east, Y north, Z up.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `id: ElementId`
- `level: LevelId`
- `type_id: ElementTypeId`
- `mark: String`

  "Mark" instance parameter, shown in schedules and tags.

- `comments: String`
- `kind: ElementKind`
- `uid: Uid`

  Persistent unique id (IFC GlobalId and similar).

- `phase_created: crate::PhaseId`

  The phase the element is built in (see `crate::phase`).

- `phase_demolished: Option<crate::PhaseId>`

  The phase the element is demolished in (not before it is built).

- `design_option: Option<crate::OptionId>`

  The design option the element belongs to (`None`: the main model,
  shown with every option; see `crate::design_option`).

- `paints: Vec<crate::paint::FacePaint>`

  Faces painted with a material of their own (see `crate::paint`).


**Methods**

- `pub fn category(&self) -> Category`
- `pub fn rebar_partition(&self) -> Option<&str>`

  The partition of a reinforcement set or of shaped bars.

- `pub fn rebar_bar_mark(&self) -> Option<&str>`

  The mark given by hand to a reinforcement set's or shaped bars' bars.

- `pub fn owner_view(&self) -> Option<ViewId>`

  The view that owns the element (dimensions, text notes, detail lines
  and imported drawings live in one view and go with it).

- `pub fn set_owner_view(&mut self, view: ViewId) -> bool`

  Give a view-owned element (see `Self::owner_view`) another view.
  Returns false, changing nothing, for elements of the model.

- `pub fn as_segment(&self) -> Option<&crate::MepSegment>`

  The run of a pipe, conduit, cable tray or lightning conductor.

- `pub fn as_segment_mut(&mut self) -> Option<&mut crate::MepSegment>`
- `pub fn systems(&self) -> Vec<crate::SystemId>`

  Systems (or circuits) the element belongs to.

- `pub fn as_wall(&self) -> Option<&Wall>`
- `pub fn as_hosted(&self) -> Option<&Hosted>`
- `pub fn as_hosted_mut(&mut self) -> Option<&mut Hosted>`
- `pub fn drawn_sketch_mut(&mut self) -> Option<&mut Option<crate::sketch::Sketch>>`

  The sketch an outline was drawn with (floors, ceilings, roofs and
  masses), to read or set.

- `pub fn drawn_sketch(&self) -> Option<&crate::sketch::Sketch>`

  The sketch an outline was drawn with, if it has one.

- `pub fn sketched_profile(&self) -> Option<(&[DVec2], &[Vec<DVec2>])>`

  The outline and openings a sketch makes for this element: floors and
  roofs have openings, ceilings and masses an outline only.

- `pub fn drop_stale_sketch(&mut self) -> bool`

  Drop a sketch that no longer makes the outline (edited without it), or
  is invalid; returns whether it did.

- `pub fn sketch(&self) -> Option<&[DVec2]>`

  Sketch boundary for floors, ceilings and roofs.

- `pub fn translate(&mut self, delta: DVec2)`

  Translate the element in plan. Hosted elements move with their host.

- `pub fn member_host(&self) -> Option<&dyn MemberHost>` *(in `cc_core::member_host`)*
- `pub fn shaft_cuts_mut(&mut self) -> Option<&mut Vec<Vec<DVec2>>>` *(in `cc_core::shaft`)*

  The shaft cuts kept in a floor, ceiling or roof.

- `pub fn shaft_cuts(&self) -> &[Vec<DVec2>]` *(in `cc_core::shaft`)*

  The shaft cuts of a floor, ceiling or roof.

- `pub fn transform(&mut self, t: &PlanTransform)` *(in `cc_core::transform`)*

  Move, rotate or mirror the element in plan. Hosted elements are
  placed along their wall and are handled with it (see
  `crate::EditCtx::transform_elements`); here only their hand flips
  under a mirror.


<a id="cc_core-element-elementkind"></a>

### enum `ElementKind`

```rust
pub enum ElementKind
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Variants**

- `Wall(Wall)`
- `Door(Hosted)`
- `Window(Hosted)`
- `Niche(Hosted)`

  A recess in one face of its host wall (the left one; the right with
  `flip_facing`), as deep as its type says: see `crate::openings`.

- `Floor(Floor)`
- `Ceiling(Ceiling)`
- `Roof(Roof)`
- `Room(Room)`
- `Grid(Grid)`
- `Dimension(Box<Dimension>)`
- `Text(Box<TextNote>)`
- `Column(Column)`
- `Beam(Beam)`
- `Stair(Stair)`
- `Ramp(Ramp)`
- `Railing(Railing)`
- `Component(Component)`
- `Foundation(Foundation)`
- `Rebar(Rebar)`
- `ShapedRebar(ShapedRebar)`

  Reinforcing bars bent to shapes of their own, placed by their center
  lines (bars read from IFC files).

- `Pipe(crate::MepSegment)`
- `Conduit(crate::MepSegment)`
- `CableTray(crate::MepSegment)`

  A cable tray run (a rectangular channel carrying circuits).

- `Valve(crate::Valve)`

  A valve on a pipe.

- `LightningConductor(crate::lightning::LightningConductor)`

  A conductor of a lightning protection system (roof, down or earth conductor).

- `CadImport(Box<crate::cad::CadImport>)`

  A DWG or DXF drawing placed in a 2D view (an underlay to trace over).

- `Mass(crate::mass::Mass)`

  A conceptual mass (massing studies).

- `Sweep(crate::profile_sweep::Sweep)`

  A profile swept along a path (baseboards, moldings, fascias, gutters).

- `Shaft(crate::shaft::Shaft)`

  A vertical opening through floors, ceilings and roofs.

- `SteelBuilding(Box<crate::steel_building::SteelBuilding>)`

  A parametric steel building: it owns the columns and beams of its frames.

- `Truss(Box<crate::truss::Truss>)`

  A parametric truss: it owns the beams and columns of its chords and web.

- `Bracing(Box<crate::bracing::Bracing>)`

  A parametric bracing: it owns the braces of one bay of a frame.

- `WallFraming(Box<crate::wall_framing::WallFraming>)`

  Light steel or wood framing of a wall: it owns the studs and the beams of its tracks, headers and sills.

- `Connection(crate::connection::Connection)`

  A steel or timber joint at one end of a column or beam (placed by its member).

- `Topography(crate::topography::Topography)`

  A terrain surface from measured points inside a boundary.

- `Pad(crate::pad::Pad)`

  A building pad graded into its terrain.

- `DetailLine(crate::detail::DetailLine)`

  A 2D line, arc, circle or ellipse drawn in one view.

- `FilledRegion(crate::region::Region)`

  A 2D area of one view filled as its type says.

- `MaskingRegion(crate::region::Region)`

  A 2D area of one view that hides the model's linework under it.

- `LevelMark(Box<crate::level_mark::LevelMark>)`

  The elevation of a point, written in one view.

- `BarTag(Box<crate::bar_tag::BarTag>)`

  The count, mark, diameter, spacing and length of a reinforcement
  set's bars, written in one view.

- `Pile(crate::pile::Pile)`

  A pile: free, or one of a pile cap's (see `crate::pile`).

- `DetailComponent(Box<crate::detail_component::DetailComponent>)`

  A 2D component placed in one view (construction details).


**Methods**

- `pub fn member_host(&self) -> Option<&dyn MemberHost>` *(in `cc_core::member_host`)*

  The host behind this element, if it is one.

- `pub fn member_host_mut(&mut self) -> Option<&mut dyn MemberHost>` *(in `cc_core::member_host`)*

<a id="cc_core-element-walltop"></a>

### enum `WallTop`

```rust
pub enum WallTop
```

How the top of a wall is defined.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Unconnected`

  `height` above the base.

- `Level { level: LevelId, offset: f64 }`

  At a level's elevation plus `offset`; follows the level when it moves.


<a id="cc_core-element-locationline"></a>

### enum `LocationLine`

```rust
pub enum LocationLine
```

Which line of the wall the drawn `start`-`end` line follows.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Centerline`
- `CoreCenter`
- `FinishExterior`
- `FinishInterior`
- `CoreExterior`
- `CoreInterior`

**Methods**

- `pub const ALL: [LocationLine; 6]`
- `pub fn key(self) -> &'static str`

<a id="cc_core-element-wall"></a>

### struct `Wall`

```rust
pub struct Wall
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `start: DVec2`

  Location line start point (see `location_line`).

- `end: DVec2`
- `base_offset: f64`

  Offset of the wall base from its level elevation.

- `height: f64`

  Unconnected height (used when `top` is `WallTop::Unconnected`).

- `top: WallTop`
- `location_line: LocationLine`
- `room_bounding: bool`

  The wall bounds rooms.

- `structural: bool`

  The wall is load-bearing.

- `join_start: bool`

  The start (end) may join other walls; when off it stays square.

- `join_end: bool`
- `bulge: f64`

  An arc wall: `tan(sweep / 4)` of its location line, positive when it
  turns counter-clockwise from `start` to `end` (see `crate::arc`);
  0 for a straight wall.

- `attach_top: Option<ElementId>`

  The roof, ceiling or floor the top is attached to: under it the wall
  rises or drops to its underside, following its slope; elsewhere it
  keeps its own top.

- `curtain: crate::curtain::CurtainEdits`

  Adjustments to the grid of a curtain wall type (ignored by other types).


**Methods**

- `pub fn arc(&self) -> Option<crate::arc::Arc>`

  The arc of the location line (`None` for a straight wall).

- `pub fn length(&self) -> f64`

  Length of the location line (along the arc for arc walls).

- `pub fn direction(&self) -> DVec2`

  Unit direction from start to end (X axis if degenerate): the chord of an arc wall.

- `pub fn normal(&self) -> DVec2`

  Left-hand normal of the chord.

- `pub fn tangent_at(&self, distance: f64) -> DVec2`

  Direction of the location line `distance` along it.

- `pub fn normal_at(&self, distance: f64) -> DVec2`

  Left-hand normal of the location line `distance` along it.

- `pub fn point_at(&self, distance: f64) -> DVec2`
- `pub fn at(&self, along: f64, across: f64) -> DVec2`

  The point `along` the location line and `across` to its left.

- `pub fn project(&self, p: DVec2) -> (f64, f64)`

  Distance along the wall of the projection of `p`, and the signed
  perpendicular distance (positive on the left).

- `pub fn offset_line(&self, distance: f64) -> (DVec2, DVec2)`

  The location line moved `distance` to its left (concentric for arc
  walls, which keep their bulge).


<a id="cc_core-element-hosted"></a>

### struct `Hosted`

```rust
pub struct Hosted
```

A door or window hosted by a wall.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `host: ElementId`
- `offset: f64`

  Distance from the host wall start to the opening center, along the wall.

- `sill_height: f64`

  Sill height above the wall base (0 for doors).

- `flip_facing: bool`

  Swing/facing to the wall's right side instead of the left.

- `flip_hand: bool`

  Hinge at the opposite jamb.


<a id="cc_core-element-floor"></a>

### struct `Floor`

```rust
pub struct Floor
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `boundary: Vec<DVec2>`

  Boundary sketch (closed polygon, either winding).

- `height_offset: f64`

  Offset of the top face from the level elevation.

- `openings: Vec<Vec<DVec2>>`

  Openings through the slab (shafts, stairwells): closed outlines
  inside the boundary, apart from each other.

- `cuts: Vec<Vec<DVec2>>`

  Openings cut by shafts, kept in step with them by every edit (see
  `crate::shaft`).

- `sketch: Option<crate::sketch::Sketch>`

  The curves the outline was drawn with (see `crate::sketch`); dropped
  when the outline is edited without them.

- `structural: bool`

  A structural slab: one that takes reinforcement.


**Methods**

- `pub fn new(boundary: Vec<DVec2>, height_offset: f64) -> Self`

  A floor over `boundary` with no openings.

- `pub fn all_openings(&self) -> impl Iterator<Item = &Vec<DVec2>>`

  Its openings and the shafts' cuts.

- `pub fn area(&self) -> f64`

  Plan area: the boundary less its openings, exact along the arcs,
  circles and ellipses of its sketch (the boundary's points are chords).


<a id="cc_core-element-ceiling"></a>

### struct `Ceiling`

```rust
pub struct Ceiling
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `boundary: Vec<DVec2>`
- `height_offset: f64`

  Height of the ceiling's underside above the level.

- `cuts: Vec<Vec<DVec2>>`

  Openings cut by shafts, kept in step with them by every edit (see
  `crate::shaft`).

- `sketch: Option<crate::sketch::Sketch>`

  The curves the outline was drawn with (see `crate::sketch`); dropped
  when the outline is edited without them.


<a id="cc_core-element-roofshape"></a>

### enum `RoofShape`

```rust
pub enum RoofShape
```

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`

**Variants**

- `Flat`
- `Shed { slope: f64, direction_deg: f64, }`

  Single slope rising towards `direction_deg` (0 = +X), slope as rise/run.

- `Gable { slope: f64, ridge_deg: f64, }`

  Two slopes meeting at a ridge along `ridge_deg`; the eaves parallel to
  the ridge rise, the others become gable ends.

- `Hip { slope: f64, }`

  Every eave rises with the same slope (hips and, on long plans, a ridge).


<a id="cc_core-element-roof"></a>

### struct `Roof`

```rust
pub struct Roof
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `footprint: Vec<DVec2>`
- `base_offset: f64`

  Offset of the roof base above the level.

- `overhang: f64`
- `shape: RoofShape`
- `holes: Vec<Vec<DVec2>>`

  Openings through the roof (courtyards), closed outlines inside the
  footprint; pitched roofs slope down to their eaves too.

- `cuts: Vec<Vec<DVec2>>`

  Slope of each footprint edge (edge `i` from point `i` to `i + 1`) that
  differs from the shape's, for gable and hip roofs: `Some(0.0)` makes
  the edge a gable end, `Some(s)` an eave rising at `s`; missing or
  `None` entries follow the shape.
  Openings cut by shafts, kept in step with them by every edit (see
  `crate::shaft`).

- `edge_slopes: Vec<Option<f64>>`
- `sketch: Option<crate::sketch::Sketch>`

  The curves the outline was drawn with (see `crate::sketch`); dropped
  when the outline is edited without them.


**Methods**

- `pub fn new(footprint: Vec<DVec2>, base_offset: f64, overhang: f64, shape: RoofShape) -> Self`

  A roof over `footprint` with no openings.

- `pub fn edge_slope(&self, i: usize) -> Option<f64>`

  The slope set on footprint edge `i`, if any.

- `pub fn area(&self) -> f64`

  Plan area: the footprint less its openings.


<a id="cc_core-element-room"></a>

### struct `Room`

```rust
pub struct Room
```

A room: the space enclosed by room-bounding walls around `position`.
Its boundary is derived from the walls (see `cc_geom`).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `position: DVec2`

  A point inside the room; also where its tag is drawn.

- `number: String`
- `name: String`
- `height: f64`

  Height of the room above its level (upper limit).


<a id="cc_core-element-grid"></a>

### struct `Grid`

```rust
pub struct Grid
```

A named grid line (datum), shown in every plan, and in the elevations
and sections it runs square to.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `start: DVec2`
- `end: DVec2`
- `name: String`

<a id="cc_core-element-textnote"></a>

### struct `TextNote`

```rust
pub struct TextNote
```

A text note in one view, in that view's drawing coordinates (like
dimensions), optionally with leaders pointing at what it describes.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `view: ViewId`
- `position: DVec2`

  Insertion point: the top left corner of the text, before rotation.

- `text: String`

  The text; line breaks start new lines.

- `width: Option<f64>`

  Wrapping width in paper millimeters for multiline text; `None` for
  single-line text (only explicit line breaks break it).

- `rotation_deg: f64`

  Rotation in degrees counter-clockwise.

- `leaders: Vec<Leader>`

<a id="cc_core-element-leader"></a>

### struct `Leader`

```rust
pub struct Leader
```

A leader line from a text to what it points at: one segment, or two
when it has an elbow (text → elbow → end).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `end: DVec2`

  The point it points at, where the symbol is drawn.

- `elbow: Option<DVec2>`

  Bend point splitting the leader in two segments.

- `symbol: LeaderSymbol`

<a id="cc_core-element-leadersymbol"></a>

### enum `LeaderSymbol`

```rust
pub enum LeaderSymbol
```

The mark at the end of a leader.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Arrow`

  Filled arrowhead.

- `OpenArrow`

  Open arrowhead (two strokes).

- `Triangle`

  Outlined arrowhead.

- `Dot`

  Small filled circle.

- `Circle`
- `Square`
- `FilledSquare`
- `Tick`

  Slanted stroke (architectural tick).

- `None`

**Methods**

- `pub const ALL: [LeaderSymbol; 9]`
- `pub fn key(self) -> &'static str`

  Translation key of the symbol's name.


<a id="cc_core-element-dimension"></a>

### struct `Dimension`

```rust
pub struct Dimension
```

A dimension in one view, in that view's drawing coordinates. What `a`,
`b` and `offset` mean depends on the `DimensionKind`.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `view: ViewId`
- `a: DVec2`
- `b: DVec2`
- `offset: f64`

  Linear kinds: distance of the dimension line from `a`, positive on
  the left of the measured direction. Angular: radius of the arc.

- `kind: DimensionKind`
- `refs: [Option<crate::DimRef>; 2]`

  Elements the ends `a` and `b` follow (associative dimension).


**Methods**

- `pub fn length(&self) -> f64`
- `pub fn direction(&self) -> Option<DVec2>`

  Direction the dimension measures along (linear kinds), if defined.

- `pub fn all_refs_mut(&mut self) -> impl Iterator<Item = &mut Option<crate::DimRef>>`

  Every element reference: the ends and, for angles, the rays.

- `pub fn is_associated(&self) -> bool`

  Whether any end or ray follows an element.

- `pub fn value(&self) -> f64`

  The measured value: meters for lengths, radians for angles.


<a id="cc_core-element-dimensionkind"></a>

### enum `DimensionKind`

```rust
pub enum DimensionKind
```

How a dimension measures.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Aligned`

  Distance from `a` to `b`, along `a`-`b`.

- `Linear { dir: DVec2 }`

  Distance from `a` to `b` measured along `dir` (a unit vector,
  horizontal or vertical when placed).

- `Angular { vertex: DVec2, #[serde(default)] rays: [Option<crate::DimRef>; 2], }`

  Angle at `vertex` between the rays through `a` and through `b`.
  `rays` hold a second point of each measured line (the vertex, in the
  frame of that line's element), so the vertex follows the elements.

- `Radial`

  Radius of the arc centered at `a` through `b`.

- `Diameter`

  Diameter of the circle centered at `a` through `b`.

- `Ordinate { dir: DVec2 }`

  Distance from the origin `a` to `b` measured along `dir` (a unit
  vector), drawn as a running (ordinate) dimension: the dimensions of
  a set share the origin and the line, and each shows its running
  total from the origin.


<a id="cc_core-element-column"></a>

### struct `Column`

```rust
pub struct Column
```

A column placed at a point, from its base to its top constraint.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `position: DVec2`
- `rotation_deg: f64`

  Rotation of the section in plan, degrees counter-clockwise.

- `base_offset: f64`
- `height: f64`

  Height when `top` is unconnected.

- `top: WallTop`
- `structural: bool`

  A load-bearing column; an architectural one (off) is cut by the walls it is joined with.


<a id="cc_core-element-beamlateral"></a>

### enum `BeamLateral`

```rust
pub enum BeamLateral
```

Which line across a beam's section its `start`-`end` location line follows.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Center`

  The middle of the section.

- `Left`

  The left face (looking from the start to the end).

- `Right`

  The right face.


**Methods**

- `pub const ALL: [BeamLateral; 3]`
- `pub fn key(self) -> &'static str`
- `pub fn mirrored(self) -> Self`

  The other side, as seen in a mirror.


<a id="cc_core-element-beamvertical"></a>

### enum `BeamVertical`

```rust
pub enum BeamVertical
```

Which height of a beam's section sits at its location elevation.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Top`
- `Center`
- `Bottom`

**Methods**

- `pub const ALL: [BeamVertical; 3]`
- `pub fn key(self) -> &'static str`

<a id="cc_core-element-beam"></a>

### struct `Beam`

```rust
pub struct Beam
```

A horizontal beam between two points. Its location line is at the level
elevation plus `top_offset`; `vertical` picks which height of the section
sits there (the top by default) and `lateral` which line across it.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `start: DVec2`
- `end: DVec2`
- `top_offset: f64`
- `end_offset: Option<f64>`

  Location elevation of the end above the level when it differs from
  the start's (a sloped beam).

- `lateral: BeamLateral`
- `vertical: BeamVertical`
- `join_start: bool`

  The start (end) may join structural columns and other beams; when off it stays square.

- `join_end: bool`
- `bulge: f64`

  Bulge of an arc beam (`tan(sweep / 4)`, positive when it turns
  counter-clockwise from `start` to `end`; see `crate::arc`); 0 for a
  straight beam. An arc beam is level: it has no `end_offset`.


**Methods**

- `pub fn rise(&self) -> f64`

  Rise of the location line from the start to the end.

- `pub fn true_length(&self) -> f64`

  Length along the slope, or along the arc of an arc beam.

- `pub fn arc(&self) -> Option<crate::arc::Arc>`

  The arc of the location line (`None` for a straight beam).

- `pub fn new(start: DVec2, end: DVec2, top_offset: f64) -> Self`

  A beam whose ends may join.

- `pub fn length(&self) -> f64`

  Distance between the ends (the chord of an arc beam).


<a id="cc_core-element-foundationshape"></a>

### enum `FoundationShape`

```rust
pub enum FoundationShape
```

Where a foundation is in plan.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Variants**

- `Pad { position: DVec2, rotation_deg: f64 }`

  A pad centered at `position`, its width along the rotated X axis.

- `Strip { start: DVec2, end: DVec2, #[serde(default)] bulge: f64, }`

  A strip centered on the line `start`-`end`, or on an arc between them
  when `bulge` is not 0 (`tan(sweep / 4)`, positive when it turns
  counter-clockwise from `start` to `end`; see `crate::arc`).

- `Slab { boundary: Vec<DVec2> }`

  A slab with this boundary.


**Methods**

- `pub fn points_mut(&mut self) -> Vec<&mut DVec2>`
- `pub fn points(&self) -> Vec<DVec2>`
- `pub fn arc(&self) -> Option<crate::arc::Arc>`

  The centerline arc of an arc strip (`None` for straight strips and other shapes).

- `pub fn strip_length(&self) -> f64`

  Length of a strip's centerline (along the arc for arc strips); 0 for other shapes.


<a id="cc_core-element-foundation"></a>

### struct `Foundation`

```rust
pub struct Foundation
```

A foundation: its top is `top_offset` above the level; the type gives its
size and depth.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `shape: FoundationShape`
- `top_offset: f64`
- `sketch: Option<crate::sketch::Sketch>`

  The curves a slab's boundary was drawn with (see `crate::sketch`);
  dropped when the boundary is edited without them.


**Methods**

- `pub fn new(shape: FoundationShape, top_offset: f64) -> Self`

  A foundation of `shape`, its top `top_offset` above its level.

- `pub fn slab_area(&self) -> Option<f64>`

  Plan area of a slab, exact along the arcs and circles of its sketch
  (the boundary's points are chords); `None` for pads and strips.


<a id="cc_core-element-barface"></a>

### enum `BarFace`

```rust
pub enum BarFace
```

Where a set of longitudinal bars lies in its host's section.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Top`

  Along the top of a beam.

- `Bottom`

  Along the bottom of a beam.

- `Perimeter`

  Around a column's section (corners first).


**Methods**

- `pub fn key(self) -> &'static str`

<a id="cc_core-element-wallside"></a>

### enum `WallSide`

```rust
pub enum WallSide
```

The face of a wall a layer of bars lies near: its exterior (left) face
or its interior (right) one.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Exterior`
- `Interior`

**Methods**

- `pub const ALL: [WallSide; 2]`
- `pub fn key(self) -> &'static str`
- `pub fn sign(self) -> f64`

  1 towards the wall's left (exterior) face, -1 towards its right.


<a id="cc_core-element-spacingzone"></a>

### struct `SpacingZone`

```rust
pub struct SpacingZone
```

The spacing of a wall's horizontal bars up to a height above its base;
the last zone of a set runs to the top of the wall.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `up_to: f64`
- `spacing: f64`

<a id="cc_core-element-barend"></a>

### struct `BarEnd`

```rust
pub struct BarEnd
```

How an end of a wall's vertical bar is bent: `across` the wall (towards
the outside of its own face when positive, towards the other face when
negative; 0 for a straight end), then `along` (back beside the bar when
positive, on past its end when negative; 0 for none). An L hook has only
`across`; a bar turned into the slab and up again has both.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `across: f64`
- `along: f64`

**Methods**

- `pub const MAX_LEG: f64`

  Longest leg of a bent end, meters.

- `pub fn is_straight(&self) -> bool`
- `pub fn is_valid(&self) -> bool`
- `pub fn length(&self) -> f64`

  Length of the legs.


<a id="cc_core-element-rebarlayout"></a>

### enum `RebarLayout`

```rust
pub enum RebarLayout
```

How a set of bars is laid out in its host.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Variants**

- `Longitudinal { face: BarFace, count: u32 }`

  `count` bars running along the host, on `face`.

- `Stirrups { spacing: f64 }`

  Closed stirrups (ties) around the section every `spacing` along the host.

- `Sides { count: u32, spacing: f64 }`

  Side (skin) bars running along a beam on both side faces, between its
  top and bottom bars: `count` on each face, or with a count of 0 as
  many as keep them at most `spacing` apart over the height.

- `Ties { spacing: f64 }`

  Cross-ties of a column every `spacing` up it: a straight bar with a
  hook at each end from every bar of its longitudinal set that is not
  at a corner, across the section to the bar opposite.

- `WallHorizontal { side: WallSide, zones: Vec<SpacingZone>, #[serde(default)] inner: bool, }`

  Horizontal bars near one face of a wall (ring bars in an arc wall),
  spaced up the wall by `zones` from its base. `inner` puts them inside
  the vertical bars of that face instead of outside them.

- `WallVertical { side: WallSide, spacing: f64, #[serde(default)] offset: f64, #[serde(default)] from: f64, #[serde(default)] to: f64, #[serde(default)] bottom: BarEnd, #[serde(default)] top: BarEnd, #[serde(default)] inner: bool, }`

  Vertical bars near one face of a wall every `spacing` along it (from
  `offset` after its start: half a spacing interleaves a second set),
  from `from` to `to` above its base (`to` 0: the top), their ends bent
  as `bottom` and `top` say. `inner` as for horizontal bars.

- `WallTies { every_horizontal: u32, every_vertical: u32, #[serde(default = "wall_bars::default_hooks")] hooks: [f64; 2], }`

  Ties between the two faces of a wall: a hooked bar across it at
  every `every_horizontal`-th horizontal bar and `every_vertical`-th
  vertical bar of its exterior face. `hooks` are the angles its two
  ends are bent by, degrees (at the exterior face, then the interior).

- `SlabStraight { top: bool, angle_deg: f64, spacing: f64, #[serde(default)] inner: bool, #[serde(default)] hook: f64, }`

  Straight bars of a slab's top or bottom layer, `spacing` apart, in
  the direction `angle_deg` from the plan's X axis, each clipped to the
  slab's boundary and openings. `inner` puts them inside the bars of
  the other direction of that layer. `hook` bends the ends at the
  slab's edge by that length: towards the middle of the slab when
  positive (down for top bars), out of it when negative (into the wall
  above a top bar or below a bottom one); 0 for straight ends.

- `SlabRing { top: bool, spacing: f64, #[serde(default)] center: Option<DVec2>, #[serde(default)] inner: bool, }`

  Ring bars of a slab's layer around `center` (default: the middle of
  the slab), `spacing` apart, clipped to the slab.

- `SlabRadial { top: bool, count: u32, #[serde(default)] center: Option<DVec2>, #[serde(default)] start: f64, #[serde(default)] intermediate: f64, #[serde(default)] inner: bool, }`

  `count` radial bars of a slab's layer around `center`, from the
  radius `start` to the slab's edge; with an `intermediate` radius, a
  shorter bar from there to the edge between each two.

- `StripTransverse { top: bool, spacing: f64 }`

  Bars across a strip footing (radial in an arc strip) every `spacing`
  along it. Bottom bars are straight; a top bar follows the top of the
  footing (up its slopes and flat between the faces of its beam) and
  its ends turn down to the bottom layer, closing the edges.

- `StripLongitudinal { top: bool, spacing: f64 }`

  Bars along a strip footing (ring bars in an arc strip) `spacing`
  apart across it, in its bottom layer or under its top, following its
  slopes; top bars leave the width of the beam to the beam's own.

- `SlabTrim { top: bool, count: u32, spacing: f64, anchorage: f64, #[serde(default)] diagonals: bool, }`

  Trimming bars around each opening of a slab, in one layer: `count`
  bars `spacing` apart along each side of the opening, reaching
  `anchorage` past it, and with `diagonals` one bar across each corner.


**Methods**

- `pub fn runs_along(&self) -> bool`

  Whether the bars run along the host (and so may be spliced).

- `pub fn in_wall(&self) -> bool`

  Whether the set reinforces a wall.

- `pub fn in_slab(&self) -> bool`

  Whether the set reinforces a slab (a floor or a foundation slab).

- `pub fn in_strip(&self) -> bool`

  Whether the set is one of a strip footing's own (across or along its footing slab).

- `pub fn fits_strip(&self) -> bool`

  Whether a strip footing may host the set: its own layouts, and those
  of a beam (for the beam on top of it, or its whole section).

- `pub fn slab_top(&self) -> Option<bool>`

  Whether a slab's set lies in its top layer.

- `pub fn wall_side(&self) -> Option<WallSide>`

  The face of a wall the set lies near.

- `pub fn is_valid(&self) -> bool`

  Whether the values are usable (finite and within their limits).


<a id="cc_core-element-facebars"></a>

### struct `FaceBars`

```rust
pub struct FaceBars
```

The longitudinal bars on one face of a column's section: `count` of them
evenly spread, the two at the face's ends (the corners, shared with the
faces beside it) included, so at least 2; or with a count of 0 as many
as keep them at most `spacing` apart.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `count: u32`
- `spacing: f64`

**Methods**

- `pub const MAX_COUNT: u32`

  Most bars on one face.

- `pub const MAX_FACES: usize`

  Most faces a set lists.

- `pub fn is_valid(&self) -> bool`
- `pub fn between(&self, length: f64) -> usize`

  Bars strictly between the corners of a face `length` long.


<a id="cc_core-element-splice"></a>

### struct `Splice`

```rust
pub struct Splice
```

Lap splices: a bar longer than `max_length` (the stock length) is made of
pieces overlapping by `lap`, the splices of neighbouring bars staggered.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `max_length: f64`
- `lap: f64`

**Methods**

- `pub const MIN_LENGTH: f64`

  Shortest stock length and lap accepted, meters.

- `pub const MIN_LAP: f64`
- `pub fn is_valid(&self) -> bool`

  Usable: finite, a stock of at least `Self::MIN_LENGTH` and a lap
  from `Self::MIN_LAP` to half the stock.

- `pub fn pieces(&self, length: f64, index: usize) -> Vec<(f64, f64)>`

  The pieces of bar number `index` of a set, `length` long: `(from,
  to)` along it. A bar within the stock length is one piece; a longer
  one is cut in the fewest equal pieces, and odd bars are cut half a
  piece off the even ones so that neighbours never splice together.


<a id="cc_core-element-rebar"></a>

### struct `Rebar`

```rust
pub struct Rebar
```

A set of reinforcing bars in a beam, column, wall or slab (`host`), `cover`
from its faces; the type gives the bar diameter. The bars follow their host.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `host: ElementId`
- `layout: RebarLayout`
- `cover: f64`
- `splice: Option<Splice>`

  Lap splices of the bars running along a beam, or the lifts of a
  column's vertical bars (none: whole bars).

- `faces: Vec<FaceBars>`

  A column's longitudinal bars face by face, in the order of the faces
  of its section (empty: `count` bars spread around it).

- `lap_at_joins: bool`

  Bars along the host (a wall's horizontal bars, those along a strip
  footing or a beam) run half a lap past each end where another host
  carries on in the same line, as the walls of a cylinder or the
  strips of a ring footing do: the bars of the two then lap by a
  whole one at the joint, instead of meeting end to end.

- `overlapping: bool`

  A column's stirrups as several closed rectangles that overlap, one
  around each of the rectangles a section with only square corners is
  made of (two in an L or a T, three in a U), instead of one stirrup
  following the whole outline.

- `lifts: Vec<f64>`

  Heights above a column's base where its vertical bars are cut, rising:
  the bars of each lift start at one and run a lap past the next (empty:
  whole bars, or the equal lifts of `splice`).

- `partition: String`

  The part of the structure the set belongs to (e.g. "FOUNDATION",
  "WALL"), which schedules filter and group by; empty for none.

- `bar_mark: Option<String>`

  A mark given by hand to all the set's bars, instead of the one the
  project's marking gives their shape (see `crate::bar_marks`).


**Methods**

- `pub const MAX_LIFTS: usize`

  Most heights a set's bars are cut at.

- `pub fn new(host: ElementId, layout: RebarLayout, cover: f64) -> Rebar`

  A set in `host` with nothing but its layout and cover.

- `pub fn lifts_are_valid(lifts: &[f64]) -> bool`

  Whether the heights of `lifts` are usable: positive, finite and rising.

- `pub fn is_valid(&self) -> bool`

  Whether the values are usable (finite and within their limits).

- `pub fn lift_pieces(lifts: &[f64], height: f64, lap: f64) -> Vec<(f64, f64)>`

  The pieces `(from, to)` of a vertical bar of a column `height` high
  cut at `lifts`: each runs from a cut (or the base) to `lap` past the
  next one (or to the top). Cuts at or above the top are ignored.


<a id="cc_core-element-shapedrebar"></a>

### struct `ShapedRebar`

```rust
pub struct ShapedRebar
```

Reinforcing bars of any shape (bars bent to a schedule's shape codes,
read from IFC files): each bar is its center line, in plan x/y and z
above the element's level (meters), closed when its ends meet; the type
gives the diameter. Unlike `Rebar` they have no host and keep their
place.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `bars: Vec<Vec<DVec3>>`
- `properties: Vec<crate::properties::PropertySet>`

  Property and quantity sets read from an IFC file (shape code, bar
  marks, grade), written back on export.

- `partition: String`

  The part of the structure the bars belong to, as `Rebar::partition`.

- `bar_mark: Option<String>`

  A mark given by hand to all the bars, as `Rebar::bar_mark`.


**Methods**

- `pub const MAX_BARS: usize`

  At most this many bars in one element (the ties of a tank wall are
  a set of over a thousand).

- `pub const MAX_POINTS: usize`

  At most this many points along one bar.

- `pub const MAX_TOTAL_POINTS: usize`

  At most this many points in all the bars of one element.

- `pub fn points_mut(&mut self) -> impl Iterator<Item = &mut DVec3>`

  Every point of every bar.

- `pub fn is_valid(&self) -> bool`

  Whether the bars are usable: one to `Self::MAX_BARS` of them, each
  with two to `Self::MAX_POINTS` finite points and some length, and
  no more than `Self::MAX_TOTAL_POINTS` points in all.


<a id="cc_core-element-stair"></a>

### struct `Stair`

```rust
pub struct Stair
```

A stair starting at `start` (bottom of the first riser, center of the
width) going up along `direction_deg`; `layout` gives its plan shape.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `start: DVec2`
- `direction_deg: f64`
- `base_offset: f64`
- `height: f64`

  Height when `top` is unconnected.

- `top: WallTop`
- `layout: crate::StairLayout`

<a id="cc_core-element-ramp"></a>

### struct `Ramp`

```rust
pub struct Ramp
```

A ramp starting at `start` (the middle of its foot) going up along
`direction_deg`; `layout` gives its plan shape and slope.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `start: DVec2`
- `direction_deg: f64`
- `base_offset: f64`
- `height: f64`

  Height when `top` is unconnected.

- `top: WallTop`
- `layout: crate::RampLayout`

<a id="cc_core-element-railing"></a>

### struct `Railing`

```rust
pub struct Railing
```

A railing along an open path.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `path: Vec<DVec2>`
- `base_offset: f64`
- `stair: Option<crate::StairRailing>`

  The stair side the railing follows: its path and rise come from the
  stair (`path` keeps the last one, for when the stair is gone).


**Methods**

- `pub fn length(&self) -> f64`

<a id="cc_core-element-component"></a>

### struct `Component`

```rust
pub struct Component
```

A placed parametric component (see `crate::component`).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `position: DVec2`

  Insertion point in plan.

- `rotation_deg: f64`

  Rotation in plan, degrees counter-clockwise.

- `base_offset: f64`

  Elevation of the insertion point above the level.

- `overrides: std::collections::BTreeMap<String, f64>`

  Values of instance parameters that differ from the type.

- `properties: Vec<crate::properties::PropertySet>`

  Property and quantity sets of the object (from IFC import).

- `system: Option<crate::SystemId>`

  The system or circuit it is connected to (an electrical device's circuit).

- `text_overrides: std::collections::BTreeMap<String, String>`

  Values of instance text, choice and material parameters that differ from the type.

- `hotspots: std::collections::BTreeMap<String, [f64; 3]>`

  Hotspots moved away from where the type has them (component frame).


<a id="mod-cc_core-element-slab_bars"></a>

## `cc_core::element::slab_bars`

<sub>`crates/cc_core/src/element.rs`</sub>

Limits of slab reinforcement.

**Items:** [`MAX_RADIAL`](#cc_core-element-slab_bars-max_radial) · [`MAX_TRIM`](#cc_core-element-slab_bars-max_trim) · [`MAX_LENGTH`](#cc_core-element-slab_bars-max_length)

<a id="cc_core-element-slab_bars-max_radial"></a>

### const `MAX_RADIAL`

```rust
pub const MAX_RADIAL: u32
```

Most radial bars of one set.

<a id="cc_core-element-slab_bars-max_trim"></a>

### const `MAX_TRIM`

```rust
pub const MAX_TRIM: u32
```

Most trimming bars along one side of an opening.

<a id="cc_core-element-slab_bars-max_length"></a>

### const `MAX_LENGTH`

```rust
pub const MAX_LENGTH: f64
```

Longest anchorage, hook or start radius, meters.

<a id="mod-cc_core-element-wall_bars"></a>

## `cc_core::element::wall_bars`

<sub>`crates/cc_core/src/element.rs`</sub>

Limits of wall reinforcement.

**Items:** [`MIN_SPACING`](#cc_core-element-wall_bars-min_spacing) · [`MAX_ZONES`](#cc_core-element-wall_bars-max_zones) · [`MAX_EVERY`](#cc_core-element-wall_bars-max_every) · [`HOOK_ANGLES`](#cc_core-element-wall_bars-hook_angles) · [`default_hooks`](#cc_core-element-wall_bars-default_hooks)

<a id="cc_core-element-wall_bars-min_spacing"></a>

### const `MIN_SPACING`

```rust
pub const MIN_SPACING: f64
```

Least spacing of bars, meters.

<a id="cc_core-element-wall_bars-max_zones"></a>

### const `MAX_ZONES`

```rust
pub const MAX_ZONES: usize
```

Most spacing zones of one set of horizontal bars.

<a id="cc_core-element-wall_bars-max_every"></a>

### const `MAX_EVERY`

```rust
pub const MAX_EVERY: u32
```

Ties are placed at most every this many bars.

<a id="cc_core-element-wall_bars-hook_angles"></a>

### const `HOOK_ANGLES`

```rust
pub const HOOK_ANGLES: std::ops::RangeInclusive<f64>
```

The angles a tie's hooks may be bent by, degrees.

<a id="cc_core-element-wall_bars-default_hooks"></a>

### fn `default_hooks`

```rust
pub fn default_hooks() -> [f64; 2]
```

The hooks of a tie when none are given: 135 degrees at both ends.

<a id="mod-cc_core-expr"></a>

## `cc_core::expr`

<sub>`crates/cc_core/src/expr.rs`</sub>

Arithmetic expressions over named parameters, used by component
geometry: numbers, parameter names, `+ - * / ^`, parentheses, unary
minus and the functions `min`, `max`, `abs`, `sqrt`, `round`, `floor`,
`ceil`, `sin`, `cos`, `tan` (degrees). Errors are values, never panics;
nesting depth and input length are bounded.

**Items:** [`MAX_LEN`](#cc_core-expr-max_len) · [`ExprError`](#cc_core-expr-exprerror) · [`eval`](#cc_core-expr-eval) · [`names`](#cc_core-expr-names) · [`is_ident_char`](#cc_core-expr-is_ident_char) · [`is_valid_name`](#cc_core-expr-is_valid_name)

<a id="cc_core-expr-max_len"></a>

### const `MAX_LEN`

```rust
pub const MAX_LEN: usize
```

Longest expression text accepted, in bytes.

<a id="cc_core-expr-exprerror"></a>

### enum `ExprError`

```rust
pub enum ExprError
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Eq`, `std::fmt::Display`, `std::error::Error`

**Variants**

- `Empty`
- `TooLong`
- `TooDeep`
- `Syntax(usize)`

  Unexpected character or token at the byte offset.

- `UnknownName(String)`
- `Arity(String)`

  Wrong number of arguments for a function.

- `NotFinite`

  Division by zero, square root of a negative number, overflow…


<a id="cc_core-expr-eval"></a>

### fn `eval`

```rust
pub fn eval(text: &str, lookup: &dyn Fn(&str) -> Option<f64>) -> Result<f64, ExprError>
```

Evaluate `text`, resolving names with `lookup`.

<a id="cc_core-expr-names"></a>

### fn `names`

```rust
pub fn names(text: &str) -> Vec<String>
```

Names referenced by `text` (for dependency checks and renames).

<a id="cc_core-expr-is_ident_char"></a>

### fn `is_ident_char`

```rust
pub fn is_ident_char(c: char) -> bool
```

Characters allowed in parameter names (letters of any script, digits, `_`).

<a id="cc_core-expr-is_valid_name"></a>

### fn `is_valid_name`

```rust
pub fn is_valid_name(name: &str) -> bool
```

Whether `name` can be used as a parameter name.

<a id="mod-cc_core-fill_pattern"></a>

## `cc_core::fill_pattern`

<sub>`crates/cc_core/src/fill_pattern.rs`</sub>

The project's own fill patterns (hatches): families of parallel, possibly
dashed lines, as in the plain-text `.pat` format CAD programs share (see
`crate::pat`). Materials, filled region types and view overrides refer
to one with `Pattern::Custom`.
Lengths are millimeters: on paper for drafting patterns, which keep their
printed size at any view scale, and in the real world for model patterns
(tiles, brick courses), which scale with the view. A pattern's `scale`
multiplies every length.

**Items:** [`MAX_FILL_PATTERNS`](#cc_core-fill_pattern-max_fill_patterns) · [`MAX_FAMILIES`](#cc_core-fill_pattern-max_families) · [`MAX_DASHES`](#cc_core-fill_pattern-max_dashes) · [`MAX_NAME`](#cc_core-fill_pattern-max_name) · [`MAX_DESCRIPTION`](#cc_core-fill_pattern-max_description) · [`MIN_OFFSET`](#cc_core-fill_pattern-min_offset) · [`MIN_PERIOD`](#cc_core-fill_pattern-min_period) · [`MAX_LENGTH`](#cc_core-fill_pattern-max_length) · [`MIN_SCALE`](#cc_core-fill_pattern-min_scale) · [`MAX_SCALE`](#cc_core-fill_pattern-max_scale) · [`FillPatternKind`](#cc_core-fill_pattern-fillpatternkind) · [`PatternLine`](#cc_core-fill_pattern-patternline) · [`FillPattern`](#cc_core-fill_pattern-fillpattern) · [`FillPatterns`](#cc_core-fill_pattern-fillpatterns) · [`PatternSlot`](#cc_core-fill_pattern-patternslot) · [`visit_fill`](#cc_core-fill_pattern-visit_fill) · [`visit_material`](#cc_core-fill_pattern-visit_material) · [`visit_type`](#cc_core-fill_pattern-visit_type) · [`visit_override`](#cc_core-fill_pattern-visit_override) · [`visit_view_graphics`](#cc_core-fill_pattern-visit_view_graphics) · [`visit_phase_graphics`](#cc_core-fill_pattern-visit_phase_graphics) · [`PatternUsage`](#cc_core-fill_pattern-patternusage)

<a id="cc_core-fill_pattern-max_fill_patterns"></a>

### const `MAX_FILL_PATTERNS`

```rust
pub const MAX_FILL_PATTERNS: usize
```

Most custom patterns in a project (a crafted file).

<a id="cc_core-fill_pattern-max_families"></a>

### const `MAX_FAMILIES`

```rust
pub const MAX_FAMILIES: usize
```

Most line families in one pattern.

<a id="cc_core-fill_pattern-max_dashes"></a>

### const `MAX_DASHES`

```rust
pub const MAX_DASHES: usize
```

Most dash values in one family.

<a id="cc_core-fill_pattern-max_name"></a>

### const `MAX_NAME`

```rust
pub const MAX_NAME: usize
```

Longest pattern name, characters.

<a id="cc_core-fill_pattern-max_description"></a>

### const `MAX_DESCRIPTION`

```rust
pub const MAX_DESCRIPTION: usize
```

Longest description, characters.

<a id="cc_core-fill_pattern-min_offset"></a>

### const `MIN_OFFSET`

```rust
pub const MIN_OFFSET: f64
```

Smallest distance between the lines of a family, millimeters (before the scale).

<a id="cc_core-fill_pattern-min_period"></a>

### const `MIN_PERIOD`

```rust
pub const MIN_PERIOD: f64
```

Shortest repeat of a dash list, millimeters (before the scale).

<a id="cc_core-fill_pattern-max_length"></a>

### const `MAX_LENGTH`

```rust
pub const MAX_LENGTH: f64
```

Largest length or coordinate, millimeters.

<a id="cc_core-fill_pattern-min_scale"></a>

### const `MIN_SCALE`

```rust
pub const MIN_SCALE: f64
```

Range of a pattern's scale.

<a id="cc_core-fill_pattern-max_scale"></a>

### const `MAX_SCALE`

```rust
pub const MAX_SCALE: f64
```

<a id="cc_core-fill_pattern-fillpatternkind"></a>

### enum `FillPatternKind`

```rust
pub enum FillPatternKind
```

How a pattern's lengths are read.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Drafting`

  Millimeters on paper: the same printed size at any view scale.

- `Model`

  Millimeters in the real world: scales with the view.


**Methods**

- `pub fn key(self) -> &'static str`
- `pub fn code(self) -> &'static str`
- `pub fn from_code(code: &str) -> Option<Self>`

<a id="cc_core-fill_pattern-patternline"></a>

### struct `PatternLine`

```rust
pub struct PatternLine
```

One family of parallel lines.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `angle: f64`

  Direction of the lines, degrees counterclockwise from +x.

- `origin: [f64; 2]`

  A point the first line passes through (where its dashes start), mm.

- `offset: [f64; 2]`

  From one line to the next, in the line's own axes: along it (which
  staggers the dashes) and across it (the spacing), mm.

- `dashes: Vec<f64>`

  Dash lengths in turn, mm: positive drawn, negative a gap, zero a
  dot. Empty: continuous lines.


**Methods**

- `pub fn solid(angle: f64, spacing: f64) -> Self`

  Continuous lines at `angle`, `spacing` apart.

- `pub fn period(&self) -> f64`

  The length of one repeat of the dashes (0 for continuous lines).

- `pub fn problem(&self) -> Option<String>`

  Why the family cannot be drawn, if it cannot.


<a id="cc_core-fill_pattern-fillpattern"></a>

### struct `FillPattern`

```rust
pub struct FillPattern
```

A custom fill pattern.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `id: FillPatternId`
- `name: String`
- `description: String`
- `kind: FillPatternKind`
- `scale: f64`

  Multiplies every length of the families.

- `lines: Vec<PatternLine>`

**Methods**

- `pub fn problem(&self) -> Option<String>`

  Why the pattern cannot be kept as it is, if it cannot.

- `pub fn repair(&mut self) -> usize`

  Fix what a crafted file may hold: names and descriptions within
  their limits, a valid scale, families that can be drawn and not too
  many. Returns how many things it changed.

- `pub fn to_pat(&self) -> String`

  The pattern as a `.pat` file entry (see `crate::pat::write`).


<a id="cc_core-fill_pattern-fillpatterns"></a>

### type alias `FillPatterns`

```rust
pub type FillPatterns = BTreeMap<FillPatternId, FillPattern>
```

The project's fill patterns, by id.

<a id="cc_core-fill_pattern-patternslot"></a>

### enum `PatternSlot`

```rust
pub enum PatternSlot<'a>
```

A place that holds a pattern: a fill's own (where `Pattern::None` means
nothing is drawn) or a view override's (where `None` keeps what the view
draws).

**Variants**

- `Fill(&'a mut Pattern)`
- `Override(&'a mut Option<Pattern>)`

**Methods**

- `pub fn get(&self) -> Option<Pattern>`
- `pub fn set(&mut self, with: Pattern)`

  Put `with` in place of the pattern.

- `pub fn clear(&mut self)`

  Forget the pattern: no lines in a fill, no override in a view.


<a id="cc_core-fill_pattern-visit_fill"></a>

### fn `visit_fill`

```rust
pub fn visit_fill(f: &mut PatternFill, v: &mut dyn FnMut(PatternSlot))
```

Visits every pattern of a fill.

<a id="cc_core-fill_pattern-visit_material"></a>

### fn `visit_material`

```rust
pub fn visit_material(m: &mut Material, v: &mut dyn FnMut(PatternSlot))
```

Visits the patterns of a material (surface and cut).

<a id="cc_core-fill_pattern-visit_type"></a>

### fn `visit_type`

```rust
pub fn visit_type(t: &mut TypeData, v: &mut dyn FnMut(PatternSlot))
```

Visits the patterns of a type: a filled region's fill, the hatches of a
component's 2D pieces.

<a id="cc_core-fill_pattern-visit_override"></a>

### fn `visit_override`

```rust
pub fn visit_override(o: &mut GraphicOverride, v: &mut dyn FnMut(PatternSlot))
```

Visits the fill patterns of an override.

<a id="cc_core-fill_pattern-visit_view_graphics"></a>

### fn `visit_view_graphics`

```rust
pub fn visit_view_graphics(g: &mut ViewGraphics, v: &mut dyn FnMut(PatternSlot))
```

Visits the patterns of a view's (or template's) graphics.

<a id="cc_core-fill_pattern-visit_phase_graphics"></a>

### fn `visit_phase_graphics`

```rust
pub fn visit_phase_graphics(g: &mut PhaseGraphics, v: &mut dyn FnMut(PatternSlot))
```

Visits the patterns of the phase graphics.

<a id="cc_core-fill_pattern-patternusage"></a>

### struct `PatternUsage`

```rust
pub struct PatternUsage
```

What uses a custom pattern.

**Traits:** `Debug`, `Clone`, `Default`, `PartialEq`

**Fields**

- `materials: Vec<String>`
- `types: Vec<ElementTypeId>`
- `views: Vec<ViewId>`
- `templates: Vec<TemplateId>`
- `phase_graphics: bool`

**Methods**

- `pub fn count(&self) -> usize`
- `pub fn is_empty(&self) -> bool`

<a id="mod-cc_core-graph_binding"></a>

## `cc_core::graph_binding`

<sub>`crates/cc_core/src/graph_binding.rs`</sub>

Elements made by visual programming graphs.
A graph (`cc_engine::graph`) run creates elements through engine
commands; each one is bound to the call that made it, so the next run
changes it in place instead of adding another. Bindings live in
`crate::Project::graph_bindings`, keyed by element, and are tracked by
undo like the other project settings; removing an element drops its
binding, and copies of an element are never bound.

**Items:** [`MAX_PATH`](#cc_core-graph_binding-max_path) · [`GraphBinding`](#cc_core-graph_binding-graphbinding)

<a id="cc_core-graph_binding-max_path"></a>

### const `MAX_PATH`

```rust
pub const MAX_PATH: usize
```

Deepest lacing path kept (list nesting of a graph value).

<a id="cc_core-graph_binding-graphbinding"></a>

### struct `GraphBinding`

```rust
pub struct GraphBinding
```

The call of a graph node that made an element.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, `Serialize`, `Deserialize`

**Fields**

- `graph: Uid`

  The graph (its file's `uid`).

- `node: u64`

  The node's id inside the graph.

- `path: Vec<u32>`

  The indices of the call through the lists its inputs were mapped
  over (empty for a single call).

- `index: u32`

  Which of the elements that call made (most calls make one).


**Methods**

- `pub fn is_valid(&self) -> bool`

  Whether it can be kept (a file is hostile input).

- `pub fn of_node(&self, graph: Uid, node: u64) -> bool`

  Whether it was made by `node` of `graph`.


<a id="mod-cc_core-graphics"></a>

## `cc_core::graphics`

<sub>`crates/cc_core/src/graphics.rs`</sub>

View graphics: what a view shows and how it draws it.
Each view holds a `ViewGraphics`: per-category visibility and graphic
overrides, the view filters applied to it (in order) and per-element
overrides and hidden elements. The project holds the named
`ViewFilter`s (categories plus rules on parameters) and the
`ViewTemplate`s (sets of view properties a view can follow).
Overrides combine attribute by attribute: an element's own override
beats the filters, the first matching filter beats later ones, and the
filters beat the element's phase status overrides (see `crate::phase`),
which beat the category. An element is hidden when it is hidden in the
view, its category is hidden, or any matching filter hides it.

**Items:** [`LinePattern`](#cc_core-graphics-linepattern) · [`LineGraphics`](#cc_core-graphics-linegraphics) · [`FillGraphics`](#cc_core-graphics-fillgraphics) · [`GraphicOverride`](#cc_core-graphics-graphicoverride) · [`CategoryGraphics`](#cc_core-graphics-categorygraphics) · [`FilterUse`](#cc_core-graphics-filteruse) · [`ViewGraphics`](#cc_core-graphics-viewgraphics) · [`FilterOp`](#cc_core-graphics-filterop) · [`FilterParam`](#cc_core-graphics-filterparam) · [`FilterRule`](#cc_core-graphics-filterrule) · [`ViewFilter`](#cc_core-graphics-viewfilter) · [`TemplateIncludes`](#cc_core-graphics-templateincludes) · [`ViewTemplate`](#cc_core-graphics-viewtemplate) · [`TemporaryVisibility`](#cc_core-graphics-temporaryvisibility)

<a id="cc_core-graphics-linepattern"></a>

### enum `LinePattern`

```rust
pub enum LinePattern
```

Dash pattern of a line.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Solid`
- `Dash`
- `ShortDash`

  Short dashes (hidden lines).

- `Dot`
- `DashDot`

**Methods**

- `pub const ALL: [LinePattern; 5]`
- `pub fn key(self) -> &'static str`
- `pub fn dashes_mm(self) -> &'static [f64]`

  Dash and gap lengths in paper millimeters (empty: continuous).


<a id="cc_core-graphics-linegraphics"></a>

### struct `LineGraphics`

```rust
pub struct LineGraphics
```

Overrides of a kind of line; `None` keeps what the view draws.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `color: Option<[u8; 3]>`

  RGB color.

- `weight: Option<f64>`

  Pen width on paper, millimeters.

- `pattern: Option<LinePattern>`

**Methods**

- `pub fn is_empty(&self) -> bool`
- `pub fn or(&self, lower: &Self) -> Self`

  Attribute by attribute, these values or else `lower`'s.


<a id="cc_core-graphics-fillgraphics"></a>

### struct `FillGraphics`

```rust
pub struct FillGraphics
```

Overrides of a kind of fill; `None` keeps what the view draws.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `color: Option<[u8; 3]>`

  RGB color (of the pattern lines, or of a solid fill).

- `pattern: Option<Pattern>`

  `None` hides the fill, `Solid` fills it with the color.


**Methods**

- `pub fn is_empty(&self) -> bool`
- `pub fn or(&self, lower: &Self) -> Self`

<a id="cc_core-graphics-graphicoverride"></a>

### struct `GraphicOverride`

```rust
pub struct GraphicOverride
```

Graphic overrides of a category, filter or element in a view. Cut
graphics apply where the view plane cuts the element (plans, sections),
projection and surface graphics to what is seen beyond it.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `projection_lines: LineGraphics`
- `cut_lines: LineGraphics`
- `surface_fill: FillGraphics`
- `cut_fill: FillGraphics`
- `halftone: Option<bool>`

  Draw faint (lines and fills mixed with white).

- `transparency: Option<f64>`

  Surface transparency, 0 (opaque) to 1.


**Methods**

- `pub fn is_empty(&self) -> bool`
- `pub fn or(&self, lower: &Self) -> Self`

  Attribute by attribute, these values or else `lower`'s.

- `pub fn repair(&mut self) -> usize`

  Reset values that are not valid numbers or out of range; returns how many.


<a id="cc_core-graphics-categorygraphics"></a>

### struct `CategoryGraphics`

```rust
pub struct CategoryGraphics
```

Visibility and overrides of a category in a view.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `visible: bool`
- `overrides: GraphicOverride`

**Methods**

- `pub fn is_default(&self) -> bool`

<a id="cc_core-graphics-filteruse"></a>

### struct `FilterUse`

```rust
pub struct FilterUse
```

A filter applied to a view: elements it matches are hidden or drawn
with its overrides.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `filter: FilterId`
- `visible: bool`

  Matching elements are shown (`false` hides them).

- `enabled: bool`

  The filter is applied (`false` keeps it listed without effect).

- `overrides: GraphicOverride`

**Methods**

- `pub fn new(filter: FilterId) -> Self`

<a id="cc_core-graphics-viewgraphics"></a>

### struct `ViewGraphics`

```rust
pub struct ViewGraphics
```

What a view shows and how: category visibility and overrides, applied
filters, and element overrides and hidden elements.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `categories: BTreeMap<Category, CategoryGraphics>`

  Categories with a non-default visibility or overrides.

- `filters: Vec<FilterUse>`

  Filters in priority order (the first matching filter wins per attribute).

- `elements: BTreeMap<ElementId, GraphicOverride>`

  Overrides of single elements in this view.

- `hidden: BTreeSet<ElementId>`

  Elements hidden in this view.


**Methods**

- `pub fn is_empty(&self) -> bool`
- `pub fn category(&self, c: Category) -> CategoryGraphics`
- `pub fn set_category(&mut self, c: Category, g: CategoryGraphics)`

  Set a category's graphics (dropping the entry when it is the default).

- `pub fn resolve(&self, id: ElementId, category: Category, matches: &mut dyn FnMut(FilterId) -> bool) -> Option<GraphicOverride>`

  The graphics of an element of `category`, or `None` when it is
  hidden; `matches` says whether a filter matches the element.

- `pub fn resolve_phased(&self, id: ElementId, category: Category, matches: &mut dyn FnMut(FilterId) -> bool, phase: &GraphicOverride) -> Option<GraphicOverride>`

  `Self::resolve` with the overrides of the element's phase status,
  which rank below the filters and above the category.

- `pub fn repair(&mut self, filters: &BTreeMap<FilterId, ViewFilter>, elements: Option<&dyn Fn(ElementId) -> bool>) -> usize`

  Drop references to missing filters and elements and invalid values;
  returns how many.


<a id="cc_core-graphics-filterop"></a>

### enum `FilterOp`

```rust
pub enum FilterOp
```

How a rule compares an element's parameter with its value.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Serialize`, `Deserialize`

**Variants**

- `Equals`
- `NotEquals`
- `Contains`
- `NotContains`
- `BeginsWith`
- `NotBeginsWith`
- `Greater`
- `GreaterOrEqual`
- `Less`
- `LessOrEqual`
- `HasValue`
- `HasNoValue`

**Methods**

- `pub const ALL: [FilterOp; 12]`
- `pub fn key(self) -> &'static str`
- `pub fn needs_value(self) -> bool`

  Whether the operator takes a value.

- `pub fn test(self, actual: Option<(&str, Option<f64>)>, value: &str) -> bool`

  Whether `actual` (the element's value as text and, for quantities,
  as a number in SI units; `None` when it has none) satisfies the
  rule `self value`. Text compares without case; numbers compare
  numerically (equality within a millionth) when both sides are numbers.


<a id="cc_core-graphics-filterparam"></a>

### enum `FilterParam`

```rust
pub enum FilterParam
```

A parameter a filter rule tests: any schedule field, or the element's
base elevation.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Serialize`, `Deserialize`

**Variants**

- `Field(ScheduleField)`

  A field as schedules show it (`Mark`, `Type`, `Width`…).

- `Elevation`

  Elevation of the element's level, meters.


**Methods**

- `pub fn applies_to(self, category: Category) -> bool`

  Whether elements of `category` can have a value for this parameter.


<a id="cc_core-graphics-filterrule"></a>

### struct `FilterRule`

```rust
pub struct FilterRule
```

One test of a filter.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `param: FilterParam`
- `op: FilterOp`
- `value: String`

  Text, or a number in SI units (meters) for quantities.


<a id="cc_core-graphics-viewfilter"></a>

### struct `ViewFilter`

```rust
pub struct ViewFilter
```

A named, project-wide selection of elements by category and rules.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `id: FilterId`
- `name: String`
- `categories: BTreeSet<Category>`

  Categories the filter applies to (empty: all).

- `rules: Vec<FilterRule>`
- `any: bool`

  Any rule is enough (`false`: all rules must pass).


**Methods**

- `pub fn inapplicable_param(&self) -> Option<FilterParam>`

  The first rule parameter that applies to none of the filter's
  categories, so the rule never passes; `None` when every rule can pass
  or the filter covers all categories.

- `pub fn matches(&self, category: Category, rule: &mut dyn FnMut(&FilterRule) -> bool) -> bool`

  Whether the filter matches an element of `category`; `rule` tests
  one rule against the element.


<a id="cc_core-graphics-templateincludes"></a>

### struct `TemplateIncludes`

```rust
pub struct TemplateIncludes
```

The view properties a template controls.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `scale: bool`

  The view scale.

- `categories: bool`

  Category visibility and overrides.

- `filters: bool`

  The applied filters and their overrides.

- `cut_height: bool`

  Plans: the cut plane height.

- `discipline: bool`

  Floor plans: the building services shown over a faint architecture.

- `structural: bool`

  Floor plans: structural (formwork) display and structure tags.

- `phase_filter: bool`

  The phase filter (not the phase: views of one template can show
  different phases).

- `detail: bool`

  The detail level.

- `visual_style: bool`

  The visual style.

- `underlay: bool`

  Plans: the underlay (other levels' plans drawn under them).

- `design_options: bool`

  The design option shown of each set (off by default: views keep
  their own choice).


<a id="cc_core-graphics-viewtemplate"></a>

### struct `ViewTemplate`

```rust
pub struct ViewTemplate
```

Named view properties views can copy or follow.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `id: TemplateId`
- `name: String`
- `include: TemplateIncludes`
- `scale: u32`
- `graphics: ViewGraphics`

  Category visibility and overrides, and filters (element overrides are not kept).

- `cut_height: f64`
- `discipline: Option<MepDiscipline>`
- `structural: bool`
- `structure_tags: bool`
- `phase_filter: Option<crate::PhaseFilterId>`
- `detail: crate::DetailLevel`
- `visual_style: Option<crate::VisualStyle>`

  `None`: views keep the usual style of their kind.

- `underlay: crate::Underlay`

  Plans: other levels' plans drawn under them.

- `design_options: BTreeMap<crate::OptionSetId, crate::OptionId>`

  The option shown of each design option set (sets left out: automatic).


**Methods**

- `pub fn from_view(id: TemplateId, name: String, view: &View) -> Self`

  A template holding the properties of `view`.

- `pub fn applies_to(kind: &ViewKind) -> bool`

  Whether views of `kind` can use templates.

- `pub fn apply(&self, view: &mut View)`

  Copy the included properties into `view` (those that apply to its kind).

- `pub fn repair(&mut self, filters: &BTreeMap<FilterId, ViewFilter>, phase_filters: &BTreeMap<crate::PhaseFilterId, crate::PhaseFilter>, levels: &[crate::LevelId]) -> usize`

  Reset invalid values; returns how many.


<a id="cc_core-graphics-temporaryvisibility"></a>

### struct `TemporaryVisibility`

```rust
pub struct TemporaryVisibility
```

Elements hidden or isolated for a while in a view (not part of the
model: not saved and not undone).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `isolate: bool`

  `true`: only these elements and categories are shown; `false`: they are hidden.

- `elements: BTreeSet<ElementId>`
- `categories: BTreeSet<Category>`

**Methods**

- `pub fn is_empty(&self) -> bool`
- `pub fn shows(&self, id: ElementId, category: Category) -> bool`

  Whether an element of `category` is shown.


<a id="mod-cc_core-grid"></a>

## `cc_core::grid`

<sub>`crates/cc_core/src/grid.rs`</sub>

Grid naming and alignment.
**Naming.** A new grid continues the series of the grids parallel to it
(within `SERIES_DEG`): the name after that of the most recently placed
parallel grid (`successor`: `1` → `2`, `09` → `10`, `A` → `B`,
`Z` → `AA`, `A1` → `A2`, `1.1` → `1.2`). A grid with no parallel grid
starts a new series: numbers for the first series, letters for a series
across a numbered one (and numbers across a lettered one). The name is
then made unique among the existing grids by stepping the series on.
**Alignment.** The bubble lines of a grid are the lines through its ends
perpendicular to it. `align_end` snaps a grid end being drawn or
dragged onto the bubble line of a parallel grid's end (keeping the grid
parallel to it), and `aligned_ends` gives a grid the extents of a
parallel reference grid.

**Items:** [`SERIES_DEG`](#cc_core-grid-series_deg) · [`PARALLEL_DEG`](#cc_core-grid-parallel_deg) · [`is_parallel`](#cc_core-grid-is_parallel) · [`successor`](#cc_core-grid-successor) · [`next_name`](#cc_core-grid-next_name) · [`EndAlignment`](#cc_core-grid-endalignment) · [`align_end`](#cc_core-grid-align_end) · [`aligned_ends`](#cc_core-grid-aligned_ends)

<a id="cc_core-grid-series_deg"></a>

### const `SERIES_DEG`

```rust
pub const SERIES_DEG: f64
```

Grids whose directions differ by at most this many degrees (either way
along the line) belong to the same series when naming.

<a id="cc_core-grid-parallel_deg"></a>

### const `PARALLEL_DEG`

```rust
pub const PARALLEL_DEG: f64
```

Grids whose directions differ by at most this many degrees are parallel
when aligning ends.

<a id="cc_core-grid-is_parallel"></a>

### fn `is_parallel`

```rust
pub fn is_parallel(u: DVec2, v: DVec2, deg: f64) -> bool
```

Whether the lines along `u` and `v` are within `deg` degrees of each
other, either way along the line. Degenerate directions are not parallel.

<a id="cc_core-grid-successor"></a>

### fn `successor`

```rust
pub fn successor(name: &str) -> Option<String>
```

The name after `name` in its series: the trailing number counts up
keeping its zero padding (`A1` → `A2`, `1.9` → `1.10`, `09` → `10`), or
the trailing letters (up to three, all upper or all lower case) count
like spreadsheet columns (`A` → `B`, `Z` → `AA`, `AZ` → `BA`, `b` → `c`).
`None` when the name does not end in a counter (`North`, `Sul`, `A'`).

<a id="cc_core-grid-next_name"></a>

### fn `next_name`

```rust
pub fn next_name<'a>(grids: impl IntoIterator<Item = (ElementId, &'a Grid)>, start: DVec2, end: DVec2) -> String
```

Name for a new grid from `start` to `end` among `grids` (see the module
documentation): it continues the series of the most recently placed
(highest id) parallel grid and is unique among `grids`.

<a id="cc_core-grid-endalignment"></a>

### struct `EndAlignment`

```rust
pub struct EndAlignment
```

A grid end snapped onto the bubble line of another grid's end.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `point: DVec2`

  The snapped end.

- `reference: DVec2`

  The end of the other grid it lines up with.


<a id="cc_core-grid-align_end"></a>

### fn `align_end`

```rust
pub fn align_end(others: &[(DVec2, DVec2)], fixed: Option<DVec2>, raw: DVec2, tol: f64) -> Option<EndAlignment>
```

Snap a grid end near `raw` to the bubble lines of the grids `others`
(start and end points), within `tol` (model units):

- with the grid's other end `fixed` (drawing the second point, or
  dragging an end handle), the end goes where the line from `fixed`
  parallel to another grid meets the bubble line of that grid's start or
  end, so the grid comes out parallel with its end lined up;
- without it (the first point), the end goes onto the nearest bubble line.
The closest candidate wins; `None` when none is within `tol`.

<a id="cc_core-grid-aligned_ends"></a>

### fn `aligned_ends`

```rust
pub fn aligned_ends(grid: &Grid, reference: &Grid, start: bool, end: bool) -> Option<(DVec2, DVec2)>
```

New `(start, end)` of `grid` with the extents of the parallel grid
`reference`: each end moves along the grid's own line onto the bubble
line of the reference end on its side (`start`/`end` pick which ends
move). `None` when the grids are not parallel (within `PARALLEL_DEG`)
or the result would be shorter than 1 cm.

<a id="mod-cc_core-ids"></a>

## `cc_core::ids`

<sub>`crates/cc_core/src/ids.rs`</sub>

**Items:** [`Uid`](#cc_core-ids-uid)

<a id="cc_core-ids-uid"></a>

### struct `Uid`

```rust
pub struct Uid(pub u128)
```

Globally unique, persistent identifier of a model object (128 bits),
kept across saves so exports such as IFC keep stable ids. Zero means
"not assigned yet".

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, `Default`, `fmt::Display`, `Serialize`, `Deserialize<'de>`

**Methods**

- `pub fn generate() -> Self`

  A new random identifier (version 4 UUID layout).

- `pub fn is_nil(self) -> bool`

<a id="mod-cc_core-level_mark"></a>

## `cc_core::level_mark`

<sub>`crates/cc_core/src/level_mark.rs`</sub>

Level marks: annotations that write the elevation of a point in a plan,
an elevation or a section — the finished level above a shelf line and
the raw (structural) level under it. Plans draw a circle with two filled
quarters on the point; elevations and sections a triangle pointing down
to it. Like text notes, a mark lives in one view (`view`).
The levels are measured from the model (see `cc_geom::level_mark`) unless
the mark sets them; the type adds its `datum` (the value written for the
project's zero, e.g. 100.00) and the suffixes after each value.

**Items:** [`LevelMark`](#cc_core-level_mark-levelmark) · [`MAX_LEVEL`](#cc_core-level_mark-max_level) · [`DEFAULT_TEXT_SIZE`](#cc_core-level_mark-default_text_size) · [`default_type`](#cc_core-level_mark-default_type) · [`repair_type`](#cc_core-level_mark-repair_type)

<a id="cc_core-level_mark-levelmark"></a>

### struct `LevelMark`

```rust
pub struct LevelMark
```

A level mark in a view.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `view: ViewId`
- `position: DVec2`

  The point measured, in the view's drawing coordinates: a plan point,
  or in elevations and sections a point whose `y` is its elevation.

- `flip: bool`

  The values on the left of the symbol instead of the right.

- `finished: Option<f64>`

  Finished level written instead of the measured one (meters, an
  elevation of the project, before the type's datum is added).

- `raw: Option<f64>`

  Raw level written instead of the measured one.

- `show_raw: bool`

  Whether the raw level is written (when there is one: set, or measured
  on a floor with layers over its structure).


**Methods**

- `pub fn new(view: ViewId, position: DVec2) -> Self`
- `pub fn repair(&mut self) -> usize`

  Replace invalid values (non-finite, out of range); returns how many changed.


<a id="cc_core-level_mark-max_level"></a>

### const `MAX_LEVEL`

```rust
pub const MAX_LEVEL: f64
```

Largest level a mark writes either way from zero, meters (datum included).

<a id="cc_core-level_mark-default_text_size"></a>

### const `DEFAULT_TEXT_SIZE`

```rust
pub const DEFAULT_TEXT_SIZE: f64
```

Text size of the default type, paper millimeters.

<a id="cc_core-level_mark-default_type"></a>

### fn `default_type`

```rust
pub fn default_type(name: &dyn Fn(&str) -> String) -> crate::TypeData
```

The type a project gets with its first level mark, with the suffixes
`name` gives the keys `tpl-level-mark-finished` and `tpl-level-mark-raw`
(in the interface language).

<a id="cc_core-level_mark-repair_type"></a>

### fn `repair_type`

```rust
pub fn repair_type(text_size: &mut f64, datum: &mut f64) -> usize
```

Replace invalid type values; returns how many changed.

<a id="mod-cc_core-lightning"></a>

## `cc_core::lightning`

<sub>`crates/cc_core/src/lightning.rs`</sub>

Lightning protection: the conductors of a lightning protection system
(on the roof, down the walls and in the ground) and the protection
classes the coverage check uses.
Air terminals and earth electrodes are components (devices of the
[`DeviceKind::AirTerminal`](crate::DeviceKind::AirTerminal) and
[`DeviceKind::EarthElectrode`](crate::DeviceKind::EarthElectrode) kinds);
conductors are runs like pipes (`MepSegment`), of a conductor type
that gives their diameter and material.

**Items:** [`ConductorRole`](#cc_core-lightning-conductorrole) · [`LightningConductor`](#cc_core-lightning-lightningconductor) · [`EARTH_DEPTH`](#cc_core-lightning-earth_depth) · [`EARTH_RING_DISTANCE`](#cc_core-lightning-earth_ring_distance) · [`MIN_CONDUCTOR_DIAMETER`](#cc_core-lightning-min_conductor_diameter) · [`MAX_CONDUCTOR_DIAMETER`](#cc_core-lightning-max_conductor_diameter) · [`LpsClass`](#cc_core-lightning-lpsclass) · [`protected_reach`](#cc_core-lightning-protected_reach)

<a id="cc_core-lightning-conductorrole"></a>

### enum `ConductorRole`

```rust
pub enum ConductorRole
```

What a lightning protection conductor does.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Air`

  Air termination: a conductor on the roof that intercepts strikes.

- `Down`

  Down conductor: from the roof down the walls to the earthing.

- `Earth`

  Earthing: a buried conductor (an earth ring) or a bonding conductor.


**Methods**

- `pub const ALL: [ConductorRole; 3]`
- `pub fn key(self) -> &'static str`
- `pub fn name(self) -> &'static str`

  Name in commands (`air`, `down`, `earth`).

- `pub fn of_run(run: &MepSegment) -> Self`

  The usual role of `run`: vertical runs are down conductors, runs
  below the level are in the ground, the others on the roof.


<a id="cc_core-lightning-lightningconductor"></a>

### struct `LightningConductor`

```rust
pub struct LightningConductor
```

A conductor of a lightning protection system: a straight run (on a
roof, down a wall or in the ground) with its role.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `run: MepSegment`
- `role: ConductorRole`

<a id="cc_core-lightning-earth_depth"></a>

### const `EARTH_DEPTH`

```rust
pub const EARTH_DEPTH: f64
```

Depth of an earth ring and of the top of earth electrodes below the
level, meters (the usual minimum burial depth).

<a id="cc_core-lightning-earth_ring_distance"></a>

### const `EARTH_RING_DISTANCE`

```rust
pub const EARTH_RING_DISTANCE: f64
```

Usual distance of an earth ring from the building's walls, meters.

<a id="cc_core-lightning-min_conductor_diameter"></a>

### const `MIN_CONDUCTOR_DIAMETER`

```rust
pub const MIN_CONDUCTOR_DIAMETER: f64
```

Smallest and largest conductor diameter, meters (values outside are cut in files).

<a id="cc_core-lightning-max_conductor_diameter"></a>

### const `MAX_CONDUCTOR_DIAMETER`

```rust
pub const MAX_CONDUCTOR_DIAMETER: f64
```

<a id="cc_core-lightning-lpsclass"></a>

### enum `LpsClass`

```rust
pub enum LpsClass
```

Class of a lightning protection system (I is the most demanding). It
sets the rolling sphere radius and the mesh width of the air termination,
as in the public tables of the lightning protection standards.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `I`
- `II`
- `III`
- `IV`

**Methods**

- `pub const ALL: [LpsClass; 4]`
- `pub fn sphere_radius(self) -> f64`

  Radius of the rolling sphere, meters.

- `pub fn mesh_width(self) -> f64`

  Width of the air termination mesh, meters.

- `pub fn name(self) -> &'static str`

  Name in commands (`I` to `IV`).

- `pub fn parse(name: &str) -> Option<Self>`

  The class named `I` to `IV` (or `1` to `4`), in any case.


<a id="cc_core-lightning-protected_reach"></a>

### fn `protected_reach`

```rust
pub fn protected_reach(tip: f64, height: f64, radius: f64) -> Option<f64>
```

Horizontal reach of the protected volume of an air termination point at
`tip` above a reference plane, at `height` above that plane, for a
rolling sphere of `radius`: the protected volume of a single rod (the
sphere resting on the plane and touching the rod). `None` when the
point is above what the rod protects.

<a id="mod-cc_core-mass"></a>

## `cc_core::mass`

<sub>`crates/cc_core/src/mass.rs`</sub>

Conceptual masses: volumes for massing studies. A mass is a base outline
in plan extruded `height` up from its level (plus `base_offset`), or
lofted to a different top outline. Mass floors cut it at the levels it
lists: each gives the floor outline at that level, with its area and
perimeter, for gross floor areas before any wall or floor is drawn.

**Items:** [`MAX_OUTLINE`](#cc_core-mass-max_outline) · [`Mass`](#cc_core-mass-mass) · [`MassFloor`](#cc_core-mass-massfloor) · [`area`](#cc_core-mass-area) · [`perimeter`](#cc_core-mass-perimeter) · [`MassFace`](#cc_core-mass-massface) · [`push_edge`](#cc_core-mass-push_edge)

<a id="cc_core-mass-max_outline"></a>

### const `MAX_OUTLINE`

```rust
pub const MAX_OUTLINE: usize
```

Largest outline a mass keeps (points).

<a id="cc_core-mass-mass"></a>

### struct `Mass`

```rust
pub struct Mass
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `outline: Vec<DVec2>`

  Base outline in plan (closed, either winding).

- `base_offset: f64`

  Offset of the base from the level elevation.

- `height: f64`

  Height of the mass above its base.

- `top: Option<Vec<DVec2>>`

  Top outline of a loft; `None` extrudes the base outline.

- `floors: Vec<LevelId>`

  Levels where mass floors are taken (those the mass spans).

- `sketch: Option<crate::sketch::Sketch>`

  The curves the outline was drawn with (see `crate::sketch`); dropped
  when the outline is edited without them.


**Methods**

- `pub fn is_valid(&self) -> bool`

  Outlines of at least three finite points (bounded), a finite positive
  height and a finite base offset.

- `pub fn section(&self, t: f64) -> Vec<DVec2>`

  The outline at `t` of the height (0 at the base, 1 at the top),
  counter-clockwise: the base outline of an extrusion, or the loft's
  section between its outlines.

- `pub fn pushed(&self, face: MassFace, d: f64) -> Option<Mass>`

  The mass with `face` moved `d` outward (inward when negative): the top
  changes the height, the bottom the base offset and the height, and a
  side moves its edge of the outline, the edges beside it keeping their
  directions. Sides of a loft are not moved. `None` when the result is
  not a valid mass (no height left, a side pushed through the others).


<a id="cc_core-mass-massfloor"></a>

### struct `MassFloor`

```rust
pub struct MassFloor
```

A mass floor: the outline of a mass at a level, its area and perimeter.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `level: LevelId`
- `outline: Vec<DVec2>`
- `area: f64`
- `perimeter: f64`

<a id="cc_core-mass-area"></a>

### fn `area`

```rust
pub fn area(poly: &[DVec2]) -> f64
```

Area of a closed polygon (either winding).

<a id="cc_core-mass-perimeter"></a>

### fn `perimeter`

```rust
pub fn perimeter(poly: &[DVec2]) -> f64
```

Perimeter of a closed polygon.

<a id="cc_core-mass-massface"></a>

### enum `MassFace`

```rust
pub enum MassFace
```

A face of a mass, for push/pull.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Variants**

- `Top`
- `Bottom`
- `Side(usize)`

  The side over edge `i` of the base outline (from point `i` to the next).


<a id="cc_core-mass-push_edge"></a>

### fn `push_edge`

```rust
pub fn push_edge(outline: &[DVec2], i: usize, d: f64) -> Option<Vec<DVec2>>
```

`outline` with edge `i` (from point `i` to the next) moved `d` outward,
the edges beside it keeping their directions (they stretch or shrink).
`None` for a bad edge or when the outline would fold over.

<a id="mod-cc_core-material"></a>

## `cc_core::material`

<sub>`crates/cc_core/src/material.rs`</sub>

Materials: identity data, graphics in views (shading, transparency,
surface and cut patterns), render appearance and physical properties.
Types refer to materials by name.

**Items:** [`MaterialClass`](#cc_core-material-materialclass) · [`Pattern`](#cc_core-material-pattern) · [`PatternFill`](#cc_core-material-patternfill) · [`MaterialIdentity`](#cc_core-material-materialidentity) · [`MaterialGraphics`](#cc_core-material-materialgraphics) · [`Finish`](#cc_core-material-finish) · [`MAX_TEXTURE_BYTES`](#cc_core-material-max_texture_bytes) · [`MAX_TEXTURE_PIXELS`](#cc_core-material-max_texture_pixels) · [`MIN_TEXTURE_SIZE`](#cc_core-material-min_texture_size) · [`MAX_TEXTURE_SIZE`](#cc_core-material-max_texture_size) · [`MaterialTexture`](#cc_core-material-materialtexture) · [`MaterialAppearance`](#cc_core-material-materialappearance) · [`Behavior`](#cc_core-material-behavior) · [`MaterialPhysical`](#cc_core-material-materialphysical) · [`Material`](#cc_core-material-material) · [`template_properties`](#cc_core-material-template_properties)

<a id="cc_core-material-materialclass"></a>

### enum `MaterialClass`

```rust
pub enum MaterialClass
```

Broad family of a material (filters, defaults and IFC categories).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Generic`
- `Concrete`
- `Masonry`
- `Mortar`
- `Metal`
- `Wood`
- `Glass`
- `Plastic`
- `Stone`
- `Ceramic`
- `Gypsum`
- `Insulation`
- `Fabric`
- `Paint`
- `Earth`
- `Liquid`
- `Gas`

**Methods**

- `pub const ALL: [MaterialClass; 17]`
- `pub fn key(self) -> &'static str`

<a id="cc_core-material-pattern"></a>

### enum `Pattern`

```rust
pub enum Pattern
```

Fill pattern drawn on surfaces (elevations) or cuts (plans, sections).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `None`
- `Solid`
- `Diagonal`

  Lines at 45°.

- `DiagonalBack`

  Lines at 135°.

- `Crosshatch`

  Both diagonals.

- `Horizontal`
- `Vertical`
- `Grid`
- `Brick`

  Running bond courses.

- `Concrete`

  Scattered dots and small triangles.

- `Sand`

  Fine dots.

- `Insulation`

  Zig-zag batt insulation.

- `Earth`

  Earth: blocks of short strokes, alternately at 45° and 135° (a weave).

- `Custom(crate::FillPatternId)`

  One of the project's own patterns (`Project::fill_patterns`).


**Methods**

- `pub const ALL: [Pattern; 13]`

  The built-in patterns (custom ones live in the project).

- `pub fn key(self) -> &'static str`

<a id="cc_core-material-patternfill"></a>

### struct `PatternFill`

```rust
pub struct PatternFill
```

A pattern over an optional background pattern, each with its color.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `foreground: Pattern`
- `foreground_color: [u8; 3]`
- `background: Pattern`
- `background_color: [u8; 3]`

**Methods**

- `pub fn lines(pattern: Pattern) -> Self`
- `pub fn is_empty(&self) -> bool`

<a id="cc_core-material-materialidentity"></a>

### struct `MaterialIdentity`

```rust
pub struct MaterialIdentity
```

Descriptive and product data.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `description: String`
- `class: MaterialClass`
- `comments: String`
- `keywords: String`
- `manufacturer: String`
- `model: String`
- `cost: Option<f64>`

  Unit cost in the project currency.

- `url: String`
- `keynote: String`
- `mark: String`

<a id="cc_core-material-materialgraphics"></a>

### struct `MaterialGraphics`

```rust
pub struct MaterialGraphics
```

How the material looks in views.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `use_appearance: bool`

  Shade with the appearance color instead of the material color.

- `transparency: f64`

  0 (opaque) to 1 (invisible).

- `surface: PatternFill`

  Pattern on faces seen in elevation and 3D.

- `cut: PatternFill`

  Pattern where the material is cut (plans, sections).


<a id="cc_core-material-finish"></a>

### enum `Finish`

```rust
pub enum Finish
```

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Matte`
- `Satin`
- `Glossy`
- `Polished`

**Methods**

- `pub const ALL: [Finish; 4]`
- `pub fn key(self) -> &'static str`
- `pub fn shininess(self) -> f64`

  Strength of the specular highlight, 0 to 1.


<a id="cc_core-material-max_texture_bytes"></a>

### const `MAX_TEXTURE_BYTES`

```rust
pub const MAX_TEXTURE_BYTES: usize
```

Largest texture image file kept, bytes.

<a id="cc_core-material-max_texture_pixels"></a>

### const `MAX_TEXTURE_PIXELS`

```rust
pub const MAX_TEXTURE_PIXELS: u64
```

Most pixels a texture image may have (width × height).

<a id="cc_core-material-min_texture_size"></a>

### const `MIN_TEXTURE_SIZE`

```rust
pub const MIN_TEXTURE_SIZE: f64
```

Smallest and largest real-world size of one repeat of a texture, meters.

<a id="cc_core-material-max_texture_size"></a>

### const `MAX_TEXTURE_SIZE`

```rust
pub const MAX_TEXTURE_SIZE: f64
```

<a id="cc_core-material-materialtexture"></a>

### struct `MaterialTexture`

```rust
pub struct MaterialTexture
```

An image laid on a material's surfaces in the textures style: one repeat
covers `width` × `height` meters, moved by `offset` and turned by
`rotation_deg` in the face's plane coordinates (see `Self::map`).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `image: std::sync::Arc<crate::raster::RasterImage>`

  The image file (shared: undo steps and copies do not copy the bytes).

- `width: f64`
- `height: f64`
- `offset: [f64; 2]`

  Where a repeat starts, meters along the face's plane axes.

- `rotation_deg: f64`

**Methods**

- `pub fn new(image: crate::raster::RasterImage, width: f64, height: f64) -> Self`

  A texture of `image` repeating every `width` × `height` meters.

- `pub fn image_ok(&self) -> bool`

  Whether the image is one a texture may hold (valid, within the size limits).

- `pub fn repair(&mut self) -> usize`

  Replace sizes, offsets and angles that are not finite or out of
  range; returns the number of fixes (the image is checked by the caller).

- `pub fn map(&self, uv: [f64; 2]) -> [f64; 2]`

  Texture coordinates (in repeats, `v` up the image) of a point with
  plane coordinates `uv` in meters: moved by the offset, turned by the
  rotation and scaled by the size.


<a id="cc_core-material-materialappearance"></a>

### struct `MaterialAppearance`

```rust
pub struct MaterialAppearance
```

Render appearance.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `color: [u8; 3]`
- `finish: Finish`
- `description: String`
- `texture: Option<MaterialTexture>`

  An image mapped on the faces in the textures style (the color shows
  where it cannot be drawn).


<a id="cc_core-material-behavior"></a>

### enum `Behavior`

```rust
pub enum Behavior
```

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Isotropic`
- `Orthotropic`

**Methods**

- `pub const ALL: [Behavior; 2]`
- `pub fn key(self) -> &'static str`

<a id="cc_core-material-materialphysical"></a>

### struct `MaterialPhysical`

```rust
pub struct MaterialPhysical
```

Physical properties (SI units); `None` when unknown.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `behavior: Behavior`
- `density: Option<f64>`

  kg/m³.

- `thermal_conductivity: Option<f64>`

  W/(m·K).

- `specific_heat: Option<f64>`

  J/(kg·K).

- `thermal_expansion: Option<f64>`

  1/K.

- `youngs_modulus: Option<f64>`

  MPa.

- `poisson: Option<f64>`

  Poisson's ratio.

- `compressive_strength: Option<f64>`

  MPa.

- `tensile_strength: Option<f64>`

  MPa.


**Methods**

- `pub fn values_mut(&mut self) -> [(&'static str, &mut Option<f64>); 8]`

  Values as (field key, value) pairs, in display order.


<a id="cc_core-material-material"></a>

### struct `Material`

```rust
pub struct Material
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `name: String`
- `color: [u8; 3]`

  sRGB shading color.

- `identity: MaterialIdentity`
- `graphics: MaterialGraphics`
- `appearance: MaterialAppearance`
- `physical: MaterialPhysical`

**Methods**

- `pub fn new(name: impl Into<String>, color: [u8; 3]) -> Self`

  A plain material: the appearance takes the shading color.

- `pub fn shading_color(&self) -> [u8; 3]`

  Color used to shade faces in views.

- `pub fn repair(&mut self) -> usize`

  Replace non-finite or out-of-range numbers; returns the number of fixes.


<a id="cc_core-material-template_properties"></a>

### fn `template_properties`

```rust
pub fn template_properties(m: &mut Material, class: MaterialClass)
```

Template data for a default material: class, cut pattern and physical values.

<a id="mod-cc_core-member_host"></a>

## `cc_core::member_host`

<sub>`crates/cc_core/src/member_host.rs`</sub>

Member hosts: elements with no geometry of their own (steel buildings,
trusses, bracings, wall framings) that own ordinary columns and beams laid out from their
parameters. Every edit keeps them in step (`crate::EditCtx` runs
`crate::EditCtx::sync_member_hosts`): a changed host rewrites its
members in place, keeping their ids while their roles exist; a member
the user edited or deleted is detached, and the host leaves its role
alone from then on. Each host keeps its own typed roles; the shared code
sees them as `RoleKey`s, whose order is the typed roles' order.

**Items:** [`RoleKey`](#cc_core-member_host-rolekey) · [`MemberHost`](#cc_core-member_host-memberhost) · [`within`](#cc_core-member_host-within) · [`reset`](#cc_core-member_host-reset) · [`fix_type`](#cc_core-member_host-fix_type)

<a id="cc_core-member_host-rolekey"></a>

### struct `RoleKey`

```rust
pub struct RoleKey(pub u64)
```

A member's role as the shared code sees it: kind, group (frame, copy)
and index packed so that the order is the typed role's.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`

**Methods**

- `pub fn new(kind: u8, group: u16, index: u16) -> Self`
- `pub fn parts(self) -> (u8, u16, u16)`

<a id="cc_core-member_host-memberhost"></a>

### trait `MemberHost`

```rust
pub trait MemberHost
```

A host of members: what the shared sync and repair need from it.

**Implemented by (4):** [`cc_core::bracing::Bracing`](cc_core.md#cc_core-bracing-bracing), [`cc_core::steel_building::SteelBuilding`](cc_core.md#cc_core-steel_building-steelbuilding), [`cc_core::truss::Truss`](cc_core.md#cc_core-truss-truss), [`cc_core::wall_framing::WallFraming`](cc_core.md#cc_core-wall_framing-wallframing)

**Required and provided items**

- `fn member_types(&self) -> Vec<ElementTypeId>`

  The member types it uses (an edit of one regenerates it).

- `fn member_layout(&self, project: &Project) -> Vec<(RoleKey, ElementTypeId, ElementKind)>`

  Its members as its parameters lay them out, in a fixed order.

- `fn member_list(&self) -> Vec<(RoleKey, ElementId)>`

  The members it manages, by role.

- `fn set_member_list(&mut self, members: &[(RoleKey, ElementId)])`

  Replace the managed members (keys it cannot read are left out).

- `fn detached_roles(&self) -> Vec<RoleKey>`

  Roles the user took over: not made again.

- `fn set_detached_roles(&mut self, roles: &[RoleKey])`
- `fn role_fits(&self, role: RoleKey, kind: &ElementKind) -> bool`

  Whether an element of `kind` can fill `role`.

- `fn check_parameters(&self, project: &Project) -> Result<(), String>`

  Whether its numbers are in range and its types of the right categories.

- `fn repair_parameters(&mut self, project: &Project) -> Option<usize>`

  From a file: numbers back to their defaults, types to the category's
  default; returns the repairs, `None` when the host must go.

- `fn inputs(&self, _project: &Project) -> Vec<ElementId>`

  Other elements whose change regenerates the host (a wall framing's
  wall and the wall's doors and windows). An edit regenerates the host
  when it changes, adds or removes one of them, edits one's type, or
  touches an element one of them hosted before the edit (a removed or
  re-hosted door or window). None by default.

- `fn fed_by(&self, _e: &Element) -> bool`

  Whether an element like `e` can be one of its inputs (a wall, a door
  or a window): an edit touching none, and no type, skips the scan.

- `fn member_level(&self, _project: &Project) -> Option<LevelId>`

  The level its members go on when it is not the host's own (a wall
  framing's members are on its wall's level).


<a id="cc_core-member_host-within"></a>

### fn `within`

```rust
pub(crate) fn within(name: &str, v: f64, range: &RangeInclusive<f64>) -> Result<(), String>
```

Whether `v` is a finite number in `range` (else an error naming it).

<a id="cc_core-member_host-reset"></a>

### fn `reset`

```rust
pub(crate) fn reset(v: &mut f64, range: &RangeInclusive<f64>, default: f64) -> usize
```

Reset `v` to `default` when it is out of `range`; returns 1 when it did.

<a id="cc_core-member_host-fix_type"></a>

### fn `fix_type`

```rust
pub(crate) fn fix_type(p: &Project, slot: &mut ElementTypeId, category: Category) -> Option<usize>
```

`slot` if it is a type of `category`, else the category's default type
(`Some(1)`: one repair); `None` when there is none.

<a id="mod-cc_core-mep"></a>

## `cc_core::mep`

<sub>`crates/cc_core/src/mep.rs`</sub>

Building services (MEP): plumbing, electrical, structured cabling and
lightning protection systems, the pipe, conduit and cable tray segments
that run them, the valves on pipes, and the data that makes a component
a plumbing fixture or an electrical, communications or lightning
protection device.
A system (a water supply, a drain, an electrical circuit, the cabling of
a rack, a lightning protection system) is a project setting; pipes,
conduits, conductors and components refer to it by `SystemId`.
Defaults follow common Brazilian practice (fixture units and supply
weights as in the public tables of the drainage and water supply
standards; outlet mounting heights as usual in electrical plans).

**Items:** [`SystemKind`](#cc_core-mep-systemkind) · [`SystemClass`](#cc_core-mep-systemclass) · [`MepSystem`](#cc_core-mep-mepsystem) · [`MepSegment`](#cc_core-mep-mepsegment) · [`TrayStyle`](#cc_core-mep-traystyle) · [`TRAY_RUNG_SPACING`](#cc_core-mep-tray_rung_spacing) · [`MAX_TRAY_WIDTH`](#cc_core-mep-max_tray_width) · [`MAX_TRAY_HEIGHT`](#cc_core-mep-max_tray_height) · [`ValveKind`](#cc_core-mep-valvekind) · [`Valve`](#cc_core-mep-valve) · [`DeviceKind`](#cc_core-mep-devicekind) · [`MAX_PORTS`](#cc_core-mep-max_ports) · [`MepData`](#cc_core-mep-mepdata) · [`CABLE_SLACK`](#cc_core-mep-cable_slack) · [`MAX_LINK_LENGTH`](#cc_core-mep-max_link_length) · [`cable_length`](#cc_core-mep-cable_length)

<a id="cc_core-mep-systemkind"></a>

### enum `SystemKind`

```rust
pub enum SystemKind
```

What a system carries.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `ColdWater`
- `HotWater`
- `Sanitary`

  Sanitary drainage (soil and waste).

- `Vent`

  Drainage vents.

- `Rainwater`
- `Power`

  An electrical circuit feeding outlets and equipment.

- `Lighting`

  An electrical circuit feeding lights.

- `Data`

  Structured cabling: the data and voice outlets cabled to a rack.

- `Lightning`

  A lightning protection system: air terminals, conductors and earth electrodes.


**Methods**

- `pub const ALL: [SystemKind; 9]`
- `pub fn key(self) -> &'static str`
- `pub fn name(self) -> &'static str`

  Name in commands and files exchanged with other programs (`cold_water`…).

- `pub fn class(self) -> SystemClass`

  The family of the system.

- `pub fn is_electrical(self) -> bool`

  An electrical circuit (carried by conduits, fed by a board).

- `pub fn is_cabled(self) -> bool`

  Carried by conduits and cable trays: circuits and structured cabling.

- `pub fn is_drainage(self) -> bool`

  Gravity drainage: its pipes slope down along the flow.

- `pub fn default_color(self) -> [u8; 3]`

  Color of the system's pipes and conduits in plans and 3D.


<a id="cc_core-mep-systemclass"></a>

### enum `SystemClass`

```rust
pub enum SystemClass
```

The families of systems: which elements each takes.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`

**Variants**

- `Plumbing`

  Pipes and plumbing fixtures.

- `Circuit`

  Electrical circuits: conduits, cable trays and electrical devices.

- `Data`

  Structured cabling: conduits, cable trays, data outlets and racks.

- `Lightning`

  Lightning protection: conductors, air terminals and earth electrodes.


**Methods**

- `pub const ALL: [SystemClass; 4]`

<a id="cc_core-mep-mepsystem"></a>

### struct `MepSystem`

```rust
pub struct MepSystem
```

A plumbing system or an electrical circuit.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `name: String`
- `code: String`

  Short code shown in plan tags (e.g. a system abbreviation or a circuit number).

- `kind: SystemKind`
- `color: Option<[u8; 3]>`

  Display color; `None` uses the kind's color.

- `panel: Option<ElementId>`

  Circuits: the distribution board (a panel component) feeding it;
  structured cabling: the rack its outlets are cabled to.

- `voltage: f64`

  Circuits: nominal voltage, volts.


**Methods**

- `pub fn new(name: impl Into<String>, code: impl Into<String>, kind: SystemKind) -> Self`
- `pub fn color(&self) -> [u8; 3]`
- `pub fn label(&self) -> &str`

  The code, or the name when it has none.


<a id="cc_core-mep-mepsegment"></a>

### struct `MepSegment`

```rust
pub struct MepSegment
```

A straight pipe or conduit run between two points, each at its own
elevation above the level: horizontal, sloped, or vertical (a riser,
when both ends share the plan position).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `start: DVec2`
- `end: DVec2`
- `start_offset: f64`

  Elevation of the center line above the level at the start.

- `end_offset: f64`

  Elevation of the center line above the level at the end.

- `systems: Vec<SystemId>`

  Systems it belongs to: one for a pipe, the circuits a conduit carries.


**Methods**

- `pub fn new(start: DVec2, end: DVec2, offset: f64) -> Self`
- `pub fn plan_length(&self) -> f64`

  Length in plan.

- `pub fn rise(&self) -> f64`

  Rise from the start to the end.

- `pub fn length(&self) -> f64`

  True length along the run.

- `pub fn is_vertical(&self) -> bool`

  A vertical run (less than 1 mm long in plan).

- `pub fn slope(&self) -> f64`

  Fall per unit of plan length (positive when the end is lower); 0 for risers.

- `pub fn points(&self, level_z: f64) -> (DVec3, DVec3)`

  The end points in the world, on a level at `level_z`.

- `pub fn system(&self) -> Option<SystemId>`

  The first system, for pipes.


<a id="cc_core-mep-traystyle"></a>

### enum `TrayStyle`

```rust
pub enum TrayStyle
```

How a cable tray is built; it picks the 3D shape and the plan drawing.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Ladder`

  Two side rails joined by rungs.

- `SolidBottom`

  A channel with a solid bottom.

- `Perforated`

  A channel with a perforated bottom.


**Methods**

- `pub const ALL: [TrayStyle; 3]`
- `pub fn key(self) -> &'static str`
- `pub fn name(self) -> &'static str`

  Name in commands (`ladder`, `solid_bottom`, `perforated`).


<a id="cc_core-mep-tray_rung_spacing"></a>

### const `TRAY_RUNG_SPACING`

```rust
pub const TRAY_RUNG_SPACING: f64
```

Distance between the rungs of a ladder tray, meters.

<a id="cc_core-mep-max_tray_width"></a>

### const `MAX_TRAY_WIDTH`

```rust
pub const MAX_TRAY_WIDTH: f64
```

Widest and tallest cable tray section, meters (larger values in files are cut down).

<a id="cc_core-mep-max_tray_height"></a>

### const `MAX_TRAY_HEIGHT`

```rust
pub const MAX_TRAY_HEIGHT: f64
```

<a id="cc_core-mep-valvekind"></a>

### enum `ValveKind`

```rust
pub enum ValveKind
```

What a valve does; it picks the plan symbol and the IFC predefined type.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Gate`

  Opens or shuts the flow with a sliding gate.

- `Ball`

  Opens or shuts the flow with a quarter turn.

- `Check`

  Lets the flow pass one way only.

- `Globe`

  Regulates the flow.


**Methods**

- `pub const ALL: [ValveKind; 4]`
- `pub fn key(self) -> &'static str`
- `pub fn name(self) -> &'static str`

  Name in commands (`gate`, `ball`, `check`, `globe`).

- `pub fn has_handle(self) -> bool`

  Whether it has a stem and handwheel (check valves work by themselves).


<a id="cc_core-mep-valve"></a>

### struct `Valve`

```rust
pub struct Valve
```

A valve on a pipe: an inline accessory `at` meters along its host's run
(true length from the start), of the pipe's diameter. It moves, is
copied and is deleted with its pipe.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `host: ElementId`
- `at: f64`

  Distance from the start of the host pipe along its run, meters
  (clamped to the run where it is read).


**Methods**

- `pub fn fraction(&self, run: &MepSegment) -> f64`

  Its position along `run` as a fraction (0 at the start, 1 at the end).

- `pub fn point(&self, run: &MepSegment, level_z: f64) -> DVec3`

  Its center in the world, on a host run whose level is at `level_z`.


<a id="cc_core-mep-devicekind"></a>

### enum `DeviceKind`

```rust
pub enum DeviceKind
```

What an electrical device is; it picks the plan symbol and the IFC class.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Outlet`

  General-purpose outlet.

- `DedicatedOutlet`

  Outlet dedicated to one appliance.

- `Switch`
- `CeilingLight`

  Light point on the ceiling.

- `WallLight`

  Light point on a wall.

- `Panel`

  Distribution board.

- `DataOutlet`

  Data or voice outlet of the structured cabling (one or more jacks).

- `Rack`

  Communications rack (with its patch panels) the data outlets are cabled to.

- `AirTerminal`

  Air terminal (a lightning rod) of a lightning protection system.

- `EarthElectrode`

  Earth electrode (a driven rod) of a lightning protection system.


**Methods**

- `pub const ALL: [DeviceKind; 10]`
- `pub fn key(self) -> &'static str`
- `pub fn name(self) -> &'static str`

  Name in commands (`outlet`, `data_outlet`, `air_terminal`…).

- `pub fn is_light(self) -> bool`

  A light point (on a lighting circuit).

- `pub fn class(self) -> SystemClass`

  The family of systems it belongs to.

- `pub fn is_board(self) -> bool`

  What feeds a system of its family: a distribution board or a rack.

- `pub fn circuit_kind(self) -> Option<SystemKind>`

  The kind of system that serves it (`None` for boards, racks and switches).


<a id="cc_core-mep-max_ports"></a>

### const `MAX_PORTS`

```rust
pub const MAX_PORTS: u32
```

Most jacks on one data outlet (larger values in files are cut down).

<a id="cc_core-mep-mepdata"></a>

### enum `MepData`

```rust
pub enum MepData
```

Building-services data of a component definition.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`

**Variants**

- `Fixture { drain_units: f64, supply_weight: f64, hot_water: bool }`

  A plumbing fixture: its drainage fixture units, its relative weight
  on the water supply and whether it takes hot water.

- `Device { device: DeviceKind, power: f64, #[serde(default = "one", skip_serializing_if = "is_one")] ports: u32, }`

  An electrical device with its apparent power in volt-amperes; a data
  outlet with its number of jacks (`ports`), each cabled to its rack.


**Methods**

- `pub fn device_of(device: DeviceKind, power: f64) -> Self`

  An electrical device of `power` VA with one port.

- `pub fn device(&self) -> Option<DeviceKind>`
- `pub fn power(&self) -> f64`

  Apparent power of a device, VA (0 for fixtures).

- `pub fn cables(&self) -> u32`

  Cables a data outlet takes back to its rack: one per jack (0 for
  anything else).


<a id="cc_core-mep-cable_slack"></a>

### const `CABLE_SLACK`

```rust
pub const CABLE_SLACK: f64
```

Allowance added to each horizontal cable for the slack left at the rack
and at the outlet, meters.

<a id="cc_core-mep-max_link_length"></a>

### const `MAX_LINK_LENGTH`

```rust
pub const MAX_LINK_LENGTH: f64
```

Longest horizontal cable (permanent link) of structured cabling, meters,
as in the usual cabling standards.

<a id="cc_core-mep-cable_length"></a>

### fn `cable_length`

```rust
pub fn cable_length(outlet: DVec3, rack: DVec3, ceiling: f64) -> f64
```

The estimated route of a horizontal cable from an outlet at `outlet` to
a rack at `rack` (world points): up from the outlet to `ceiling` (the
world height of the outlet's ceiling, where the pathways run), across in
plan along the two axes, and down (or up) to the rack, plus
`CABLE_SLACK`.

<a id="mod-cc_core-openings"></a>

## `cc_core::openings`

<sub>`crates/cc_core/src/openings.rs`</sub>

Doors and windows as components: the built-in definitions that the
template's door and window types use and that older files' door and
window types become.
A door or window component (`ComponentKind::Door` or `Window`, with a
`HostOpening`) is placed in a wall: its frame has `x` along the wall
from the opening's center, `y` across the wall from its location line
(positive on the side a door opens to) and `z` up from the sill; its
expressions may use `WALL_THICKNESS`, the host wall's thickness.

**Items:** [`FRAME`](#cc_core-openings-frame) · [`STILE`](#cc_core-openings-stile) · [`BOTTOM_RAIL`](#cc_core-openings-bottom_rail) · [`OpeningNames`](#cc_core-openings-openingnames) · [`door_def`](#cc_core-openings-door_def) · [`operable_door_def`](#cc_core-openings-operable_door_def) · [`french_door_def`](#cc_core-openings-french_door_def) · [`window_def`](#cc_core-openings-window_def) · [`niche_def`](#cc_core-openings-niche_def) · [`NICHE_MIN_BACK`](#cc_core-openings-niche_min_back) · [`operable_window_def`](#cc_core-openings-operable_window_def)

<a id="cc_core-openings-frame"></a>

### const `FRAME`

```rust
pub const FRAME: f64
```

Width of frame members (jambs, head, sash rails), meters.

<a id="cc_core-openings-stile"></a>

### const `STILE`

```rust
pub const STILE: f64
```

Width of the stiles and top rail of a glazed door's leaves, meters.

<a id="cc_core-openings-bottom_rail"></a>

### const `BOTTOM_RAIL`

```rust
pub const BOTTOM_RAIL: f64
```

Height of a French door's bottom rail (the solid lower part of each leaf), meters.

<a id="cc_core-openings-openingnames"></a>

### struct `OpeningNames`

```rust
pub struct OpeningNames
```

Names of the built-in parameters and pieces, in the project's language.

**Traits:** `Debug`, `Clone`, `Default`

**Fields**

- `door: String`
- `double_door: String`

  Swing doors of two leaves.

- `french_door: String`

  Glazed doors (see `french_door_def`), of one leaf and of two.

- `french_door_double: String`
- `window: String`
- `niche: String`

  The component of niches (see `niche_def`).

- `depth: String`

  The depth of a niche.

- `casement: String`

  The components of windows that open (see `Self::window_for`).

- `double_casement: String`
- `top_hung: String`
- `pivot_horizontal: String`
- `pivot_vertical: String`
- `sliding: String`
- `width: String`
- `height: String`
- `sill: String`
- `bottom_rail: String`

  The height of a French door's bottom rail.

- `frame: String`
- `leaf: String`
- `sash: String`
- `glass: String`
- `sill_board: String`

**Methods**

- `pub fn new(n: &dyn Fn(&str) -> String) -> Self`

  Names from `n(key)` (template keys `tpl-*`); parameter names that are
  not valid in expressions fall back to the Portuguese ones.

- `pub fn window_for(&self, operation: WindowOperation) -> &String`

  The component of windows that open as `operation` says.

- `pub fn door_for(&self, operation: DoorOperation, glazed: bool) -> &String`

  The component of doors whose leaves swing as `operation` says,
  glazed (French doors) or not.


<a id="cc_core-openings-door_def"></a>

### fn `door_def`

```rust
pub fn door_def(names: &OpeningNames, width: f64, height: f64, material: &str) -> ComponentDef
```

A swing door: two jambs and a head as deep as the wall, and a leaf of
`material` in the middle of the wall.

<a id="cc_core-openings-operable_door_def"></a>

### fn `operable_door_def`

```rust
pub fn operable_door_def(names: &OpeningNames, (width, height): (f64, f64), material: &str, operation: DoorOperation) -> ComponentDef
```

A swing door whose leaves open as `operation` says: `door_def` with,
for a double door, two leaves meeting in the middle.

<a id="cc_core-openings-french_door_def"></a>

### fn `french_door_def`

```rust
pub fn french_door_def(names: &OpeningNames, (width, height, bottom_rail): (f64, f64, f64), (leaf_material, glass): (&str, &str), operation: DoorOperation) -> ComponentDef
```

A French door: a glazed swing door from the floor, whose leaves (one or
two, as `operation` says) are stiles and a top rail of `leaf_material`
around a pane of `glass`, over a solid bottom rail `bottom_rail` tall
(the `OpeningNames::bottom_rail` parameter), in a frame as deep as the wall.

<a id="cc_core-openings-window_def"></a>

### fn `window_def`

```rust
pub fn window_def(names: &OpeningNames, width: f64, height: f64, sill: f64, material: &str) -> ComponentDef
```

A window: a sash frame in the middle of the wall around a pane of
`material`, and a sill board across the wall under it.

<a id="cc_core-openings-niche_def"></a>

### fn `niche_def`

```rust
pub fn niche_def(names: &OpeningNames, (width, height, depth, sill): (f64, f64, f64, f64)) -> ComponentDef
```

A niche: a recess `width` wide, `height` tall and `depth` deep in one
face of its wall, its bottom `sill` above the wall's base. It has no
pieces: the wall shows inside it (pieces of a niche component are not
placed yet).

<a id="cc_core-openings-niche_min_back"></a>

### const `NICHE_MIN_BACK`

```rust
pub const NICHE_MIN_BACK: f64
```

The least a niche leaves of its wall behind it, meters: a deeper one
stops there (an opening through the wall is a door or a window).

<a id="cc_core-openings-operable_window_def"></a>

### fn `operable_window_def`

```rust
pub fn operable_window_def(names: &OpeningNames, (width, height, sill): (f64, f64, f64), material: &str, operation: WindowOperation) -> ComponentDef
```

A window whose panels open as `operation` says: `window_def` with,
for two side-hung panels, a stile where they meet and a pane in each,
and for sliding ones two panels on either side of the wall's middle,
overlapping by a stile.

<a id="mod-cc_core-pad"></a>

## `cc_core::pad`

<sub>`crates/cc_core/src/pad.rs`</sub>

Building pads: a flat area at an elevation, graded into the terrain that
hosts it, with cut and fill slopes running out from its edges to the
existing ground. The terrain's surface becomes the graded one
(`cc_geom::pad`); the pad itself has no solid.
A pad lives on its terrain's level (kept equal by every edit, see
`crate::EditCtx`), is deleted with it and moves with it.

**Items:** [`MAX_OUTLINE`](#cc_core-pad-max_outline) · [`MIN_SLOPE`](#cc_core-pad-min_slope) · [`MAX_SLOPE`](#cc_core-pad-max_slope) · [`DEFAULT_CUT_SLOPE`](#cc_core-pad-default_cut_slope) · [`DEFAULT_FILL_SLOPE`](#cc_core-pad-default_fill_slope) · [`Pad`](#cc_core-pad-pad) · [`slope_ok`](#cc_core-pad-slope_ok) · [`clamp_slope`](#cc_core-pad-clamp_slope)

<a id="cc_core-pad-max_outline"></a>

### const `MAX_OUTLINE`

```rust
pub const MAX_OUTLINE: usize
```

Most points of a pad's outline.

<a id="cc_core-pad-min_slope"></a>

### const `MIN_SLOPE`

```rust
pub const MIN_SLOPE: f64
```

Gentlest and steepest slope ratios (horizontal run per unit of rise).

<a id="cc_core-pad-max_slope"></a>

### const `MAX_SLOPE`

```rust
pub const MAX_SLOPE: f64
```

<a id="cc_core-pad-default_cut_slope"></a>

### const `DEFAULT_CUT_SLOPE`

```rust
pub const DEFAULT_CUT_SLOPE: f64
```

Ratios of a new pad type's cut and fill slopes.

<a id="cc_core-pad-default_fill_slope"></a>

### const `DEFAULT_FILL_SLOPE`

```rust
pub const DEFAULT_FILL_SLOPE: f64
```

<a id="cc_core-pad-pad"></a>

### struct `Pad`

```rust
pub struct Pad
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `host: ElementId`

  The terrain it grades.

- `outline: Vec<DVec2>`

  Closed outline in plan (either winding).

- `elevation: f64`

  Elevation of the pad above the terrain's level.

- `cut_slope: Option<f64>`

  Horizontal run per unit of rise of the cut slope; `None` uses the type's.

- `fill_slope: Option<f64>`

  Horizontal run per unit of rise of the fill slope; `None` uses the type's.

- `sketch: Option<crate::sketch::Sketch>`

  The curves the outline was drawn with (see `crate::sketch`).


**Methods**

- `pub fn is_valid(&self) -> bool`

  An outline of three to `MAX_OUTLINE` finite points enclosing an area,
  and a finite elevation. Bad slope overrides are repaired instead.

- `pub fn repair(&mut self) -> usize`

  Drop slope overrides out of range; returns how many were dropped.

- `pub fn slopes(&self, project: &Project, type_id: crate::ElementTypeId) -> (f64, f64)`

  The cut and fill ratios in effect: the pad's own, else its type's.


<a id="cc_core-pad-slope_ok"></a>

### fn `slope_ok`

```rust
pub fn slope_ok(r: f64) -> bool
```

Whether `r` is a slope ratio a pad keeps.

<a id="cc_core-pad-clamp_slope"></a>

### fn `clamp_slope`

```rust
pub fn clamp_slope(r: f64, default: f64) -> f64
```

`r` brought into the slope range (`default` when not a number).

<a id="mod-cc_core-paint"></a>

## `cc_core::paint`

<sub>`crates/cc_core/src/paint.rs`</sub>

Painted faces: a material laid on one surface of an element, over the
material its type gives it (a painted wall face, a tiled floor top).
The geometry is derived and rebuilt after every edit, so a paint does not
hold a face index. It keeps what identifies the face from outside: a
point on it and its outward normal, in model coordinates. After a rebuild
the paint goes on the element's face (of its body, framing left out)
whose normal is within `NORMAL_TOLERANCE_DEG` of the stored one and
that passes nearest the stored point, and from there on the faces of the
element in the same plane (a wall face split by its openings) and, for a
curved surface, the smooth facets joined to it (see `cc_geom::paint`).
Moving, rotating or mirroring the element in plan moves the point and
turns the normal with it; other edits (a longer wall, a thicker layer)
keep the paint on the face that is still nearest. A face turned beyond
the tolerance leaves the paint unshown until it is painted again.

**Items:** [`MAX_PAINTS`](#cc_core-paint-max_paints) · [`NORMAL_TOLERANCE_DEG`](#cc_core-paint-normal_tolerance_deg) · [`FacePaint`](#cc_core-paint-facepaint) · [`repair`](#cc_core-paint-repair)

<a id="cc_core-paint-max_paints"></a>

### const `MAX_PAINTS`

```rust
pub const MAX_PAINTS: usize
```

Most paints an element keeps.

<a id="cc_core-paint-normal_tolerance_deg"></a>

### const `NORMAL_TOLERANCE_DEG`

```rust
pub const NORMAL_TOLERANCE_DEG: f64
```

How far a face may have turned and still take its paint, degrees.

<a id="cc_core-paint-facepaint"></a>

### struct `FacePaint`

```rust
pub struct FacePaint
```

A material painted on one surface of an element.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `point: DVec3`

  A point on the painted face, meters.

- `normal: DVec3`

  The face's outward unit normal when it was painted.

- `material: String`

  The material's name.


**Methods**

- `pub fn is_valid(&self) -> bool`

  Whether the numbers are usable (finite, a normal of unit length).

- `pub fn transform(&mut self, t: &crate::PlanTransform)`

  Move and turn with the element in plan.

- `pub fn translate(&mut self, delta: DVec2)`

<a id="cc_core-paint-repair"></a>

### fn `repair`

```rust
pub fn repair(paints: &mut Vec<FacePaint>, material_exists: &dyn Fn(&str) -> bool) -> usize
```

Drop paints with bad numbers, of materials that are gone, and past the
limit; returns how many went. Normals are made unit length first.

<a id="mod-cc_core-pat"></a>

## `cc_core::pat`

<sub>`crates/cc_core/src/pat.rs`</sub>

The plain-text `.pat` hatch pattern format many CAD programs read and
write, as publicly described:
```text
;%UNITS=MM
*NAME, optional description
;%TYPE=MODEL
angle, x-origin, y-origin, delta-x, delta-y [, dash1, dash2, ...]
```rust
A header line starting with `*` opens a pattern; each following line is
a family of parallel lines through the origin at the angle (degrees),
the next line shifted by delta-x along it and delta-y across it, with
optional dashes (positive drawn, negative gaps, zero dots). Text after
`;` is a comment; the comments `;%UNITS=MM|INCH` and
`;%TYPE=MODEL|DRAFTING` are read as markers (a type marker before the
first header sets the file's default, one inside a pattern that
pattern's kind). Parsing is bounded: file size, line length, patterns,
families and dashes.

**Items:** [`MAX_PAT_BYTES`](#cc_core-pat-max_pat_bytes) · [`MAX_PAT_PATTERNS`](#cc_core-pat-max_pat_patterns) · [`MAX_LINE_CHARS`](#cc_core-pat-max_line_chars) · [`MM_PER_INCH`](#cc_core-pat-mm_per_inch) · [`PatUnits`](#cc_core-pat-patunits) · [`PatOptions`](#cc_core-pat-patoptions) · [`PatError`](#cc_core-pat-paterror) · [`ParsedPat`](#cc_core-pat-parsedpat) · [`parse`](#cc_core-pat-parse) · [`write`](#cc_core-pat-write)

<a id="cc_core-pat-max_pat_bytes"></a>

### const `MAX_PAT_BYTES`

```rust
pub const MAX_PAT_BYTES: usize
```

Largest file read, bytes.

<a id="cc_core-pat-max_pat_patterns"></a>

### const `MAX_PAT_PATTERNS`

```rust
pub const MAX_PAT_PATTERNS: usize
```

Most patterns read from one file.

<a id="cc_core-pat-max_line_chars"></a>

### const `MAX_LINE_CHARS`

```rust
pub const MAX_LINE_CHARS: usize
```

Longest line read, characters.

<a id="cc_core-pat-mm_per_inch"></a>

### const `MM_PER_INCH`

```rust
pub const MM_PER_INCH: f64
```

Millimeters in an inch.

<a id="cc_core-pat-patunits"></a>

### enum `PatUnits`

```rust
pub enum PatUnits
```

The units a file's lengths are in.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Default`

**Variants**

- `Auto`

  The file's `;%UNITS=` marker, else millimeters.

- `Millimeters`
- `Inches`

<a id="cc_core-pat-patoptions"></a>

### struct `PatOptions`

```rust
pub struct PatOptions
```

How a file is read.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`

**Fields**

- `units: PatUnits`
- `kind: Option<FillPatternKind>`

  Every pattern's kind, in place of the file's markers.

- `scale: f64`

  The scale given to every pattern.


<a id="cc_core-pat-paterror"></a>

### struct `PatError`

```rust
pub struct PatError
```

A problem with a line of the file.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `line: usize`

  Line number, from 1 (0: the whole file).

- `pattern: Option<String>`

  The pattern being read, if any (it is left out).

- `message: String`

<a id="cc_core-pat-parsedpat"></a>

### struct `ParsedPat`

```rust
pub struct ParsedPat
```

What a file holds: the patterns read (ids 0, millimeters) and the
problems found; a pattern with a problem is left out.

**Traits:** `Debug`, `Clone`, `Default`, `PartialEq`

**Fields**

- `patterns: Vec<FillPattern>`
- `errors: Vec<PatError>`

<a id="cc_core-pat-parse"></a>

### fn `parse`

```rust
pub fn parse(text: &str, options: &PatOptions) -> ParsedPat
```

Read `.pat` text.

<a id="cc_core-pat-write"></a>

### fn `write`

```rust
pub fn write(patterns: &[FillPattern]) -> String
```

`patterns` as `.pat` text in millimeters, their scales applied.

<a id="mod-cc_core-persist"></a>

## `cc_core::persist`

<sub>`crates/cc_core/src/persist.rs`</sub>

`.ccad` project files: a versioned RON document.

**Items:** [`FORMAT_VERSION`](#cc_core-persist-format_version) · [`MAX_FILE_BYTES`](#cc_core-persist-max_file_bytes) · [`PersistError`](#cc_core-persist-persisterror) · [`to_ron`](#cc_core-persist-to_ron) · [`from_ron`](#cc_core-persist-from_ron) · [`from_ron_with_repairs`](#cc_core-persist-from_ron_with_repairs) · [`from_ron_with_repairs_localized`](#cc_core-persist-from_ron_with_repairs_localized) · [`save`](#cc_core-persist-save) · [`read_to_string_limited`](#cc_core-persist-read_to_string_limited) · [`load`](#cc_core-persist-load) · [`load_with_repairs`](#cc_core-persist-load_with_repairs) · [`load_with_repairs_localized`](#cc_core-persist-load_with_repairs_localized)

<a id="cc_core-persist-format_version"></a>

### const `FORMAT_VERSION`

```rust
pub const FORMAT_VERSION: u32
```

Current file format version. Bump when the schema changes incompatibly.
28: the ANSI and architectural sheet sizes, and the slope unit
`RisePer12` (`6:12`), which older builds cannot read.

<a id="cc_core-persist-max_file_bytes"></a>

### const `MAX_FILE_BYTES`

```rust
pub const MAX_FILE_BYTES: u64
```

Largest project file read, bytes (pictures and textures are kept in it).

<a id="cc_core-persist-persisterror"></a>

### enum `PersistError`

```rust
pub enum PersistError
```

**Traits:** `Debug`, `Error`

**Variants**

- `Io(#[from] std::io::Error)`
- `Parse(#[from] ron::error::SpannedError)`
- `Serialize(#[from] ron::Error)`
- `Version(u32)`
- `TooLarge(u64)`

<a id="cc_core-persist-to_ron"></a>

### fn `to_ron`

```rust
pub fn to_ron(project: &Project) -> Result<String, PersistError>
```

<a id="cc_core-persist-from_ron"></a>

### fn `from_ron`

```rust
pub fn from_ron(text: &str) -> Result<Project, PersistError>
```

<a id="cc_core-persist-from_ron_with_repairs"></a>

### fn `from_ron_with_repairs`

```rust
pub fn from_ron_with_repairs(text: &str) -> Result<(Project, usize), PersistError>
```

Parse a project and repair invalid data (see `Project::repair`); also
returns how many repairs were needed.

<a id="cc_core-persist-from_ron_with_repairs_localized"></a>

### fn `from_ron_with_repairs_localized`

```rust
pub fn from_ron_with_repairs_localized(text: &str, translate: &dyn Fn(&str) -> Option<String>) -> Result<(Project, usize), PersistError>
```

Like `from_ron_with_repairs`, naming what migration adds with
`translate` (template keys; Portuguese defaults when it gives none).

<a id="cc_core-persist-save"></a>

### fn `save`

```rust
pub fn save(project: &Project, path: &Path) -> Result<(), PersistError>
```

<a id="cc_core-persist-read_to_string_limited"></a>

### fn `read_to_string_limited`

```rust
pub fn read_to_string_limited(path: &Path, max: u64) -> std::io::Result<String>
```

The text of the file at `path`, refused past `max` bytes before it is
read (an I/O error of kind `FileTooLarge`): files are hostile input, and
one far larger than any real file would only exhaust memory.

<a id="cc_core-persist-load"></a>

### fn `load`

```rust
pub fn load(path: &Path) -> Result<Project, PersistError>
```

<a id="cc_core-persist-load_with_repairs"></a>

### fn `load_with_repairs`

```rust
pub fn load_with_repairs(path: &Path) -> Result<(Project, usize), PersistError>
```

<a id="cc_core-persist-load_with_repairs_localized"></a>

### fn `load_with_repairs_localized`

```rust
pub fn load_with_repairs_localized(path: &Path, translate: &dyn Fn(&str) -> Option<String>) -> Result<(Project, usize), PersistError>
```

Like `load_with_repairs`, naming what migration adds with `translate`
(see `from_ron_with_repairs_localized`).

<a id="mod-cc_core-phase"></a>

## `cc_core::phase`

<sub>`crates/cc_core/src/phase.rs`</sub>

Construction phases (renovation work).
The project holds an ordered list of `Phase`s (the template has
"Existing" and "New Construction"). Every model element is created in a
phase and may be demolished in the same or a later one. Each model view
shows the model as of one phase, through a `PhaseFilter`: relative to
the view's phase an element is

* **Existing** — created in an earlier phase and not demolished yet;
* **Demolished** — created earlier and demolished in this phase;
* **New** — created in this phase (and not demolished in it);
* **Temporary** — created and demolished in this phase.
Elements created in a later phase or demolished in an earlier one are
not in the view at all. The filter maps each status to not displayed,
displayed by category, or displayed with the project's
`PhaseGraphics` for that status (which combine with view graphics
below element overrides and view filters, above category overrides).
Hosted elements follow their hosts (see `Project::phase_fixes`): a door
or window (or a rebar set) is never created before its host nor survives
its host's demolition; it can be demolished earlier on its own.
Annotation categories (dimensions, text, grids) have no phase.

**Items:** [`Phase`](#cc_core-phase-phase) · [`PhaseStatus`](#cc_core-phase-phasestatus) · [`PhaseState`](#cc_core-phase-phasestate) · [`PhaseDisplay`](#cc_core-phase-phasedisplay) · [`PhaseFilter`](#cc_core-phase-phasefilter) · [`PhaseGraphics`](#cc_core-phase-phasegraphics) · [`phase_graphics_is_default`](#cc_core-phase-phase_graphics_is_default) · [`TEMPLATE_FILTERS`](#cc_core-phase-template_filters) · [`TEMPLATE_PHASES`](#cc_core-phase-template_phases) · [`phase_host`](#cc_core-phase-phase_host)

<a id="cc_core-phase-phase"></a>

### struct `Phase`

```rust
pub struct Phase
```

A construction phase. The project keeps them in time order.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `id: PhaseId`
- `name: String`
- `description: String`

<a id="cc_core-phase-phasestatus"></a>

### enum `PhaseStatus`

```rust
pub enum PhaseStatus
```

An element's status relative to a view's phase.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Serialize`, `Deserialize`

**Variants**

- `Existing`
- `Demolished`
- `New`
- `Temporary`

**Methods**

- `pub const ALL: [PhaseStatus; 4]`
- `pub fn key(self) -> &'static str`

  Translation key of the status name.

- `pub fn ifc(self) -> &'static str`

  The IFC4 `Status` enumeration value (`Pset_*Common`).

- `pub fn from_ifc(value: &str) -> Option<Self>`

  Read an IFC `Status` value (case-insensitive).


<a id="cc_core-phase-phasestate"></a>

### enum `PhaseState`

```rust
pub enum PhaseState
```

Where an element stands in a view's phase.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Variants**

- `Unphased`

  Annotations: shown in every phase, never overridden.

- `Absent`

  Created later or demolished earlier: not part of the view.

- `Status(PhaseStatus)`

<a id="cc_core-phase-phasedisplay"></a>

### enum `PhaseDisplay`

```rust
pub enum PhaseDisplay
```

How a phase filter shows elements of one status.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `NotDisplayed`
- `ByCategory`

  With the view's graphics only.

- `Overridden`

  With the project's graphics for the status.


**Methods**

- `pub const ALL: [PhaseDisplay; 3]`
- `pub fn key(self) -> &'static str`

<a id="cc_core-phase-phasefilter"></a>

### struct `PhaseFilter`

```rust
pub struct PhaseFilter
```

A named rule for which statuses a view shows, and how.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `id: PhaseFilterId`
- `name: String`
- `existing: PhaseDisplay`
- `demolished: PhaseDisplay`
- `new: PhaseDisplay`
- `temporary: PhaseDisplay`

**Methods**

- `pub fn display(&self, status: PhaseStatus) -> PhaseDisplay`
- `pub fn set_display(&mut self, status: PhaseStatus, display: PhaseDisplay)`
- `pub fn is_show_all(&self) -> bool`

  Whether the filter shows everything, overriding all but new elements
  (the default filter of new views).


<a id="cc_core-phase-phasegraphics"></a>

### struct `PhaseGraphics`

```rust
pub struct PhaseGraphics
```

Graphic overrides of each status (used where a filter says "Overridden").

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `existing: GraphicOverride`
- `demolished: GraphicOverride`
- `new: GraphicOverride`
- `temporary: GraphicOverride`

**Methods**

- `pub fn get(&self, status: PhaseStatus) -> &GraphicOverride`
- `pub fn get_mut(&mut self, status: PhaseStatus) -> &mut GraphicOverride`
- `pub fn repair(&mut self) -> usize`

  Reset invalid values; returns how many.


<a id="cc_core-phase-phase_graphics_is_default"></a>

### fn `phase_graphics_is_default`

```rust
pub(crate) fn phase_graphics_is_default(g: &PhaseGraphics) -> bool
```

<a id="cc_core-phase-template_filters"></a>

### const `TEMPLATE_FILTERS`

```rust
pub const TEMPLATE_FILTERS: [(&str, [PhaseDisplay; 4]); 7]
```

The template's phase filters: name key and displays of existing,
demolished, new and temporary elements.

<a id="cc_core-phase-template_phases"></a>

### const `TEMPLATE_PHASES`

```rust
pub const TEMPLATE_PHASES: [&str; 2]
```

The template's phases (name keys), in order.

<a id="cc_core-phase-phase_host"></a>

### fn `phase_host`

```rust
pub fn phase_host(e: &Element) -> Option<ElementId>
```

The element that decides an element's phases: the wall of a door or
window, the beam or column of a rebar set, the stair of a railing, the
wall of a wall framing, the member of a connection, the cap of a pile.

<a id="mod-cc_core-pile"></a>

## `cc_core::pile`

<sub>`crates/cc_core/src/pile.rs`</sub>

Piles and pile caps.
A pile (`Pile`, `ElementKind::Pile`, category `Pile`) is a vertical
shaft in the ground at a plan `position`: its head (`cut_off`) and its
toe (`toe`) are offsets from its level, its `embedment` how far the head
enters the cap above it; its type (`TypeData::Pile`) gives the section
(`PileSection`), the kind (`PileKind`) and the material. The length
is derived: `cut_off - toe`.
A pile cap is a foundation (`ElementKind::Foundation` placed by a
`FoundationShape::Pad`: position and rotation) whose type is of
`FoundationKind::PileCap`: the type's `PileCap` gives the layout of
its piles, their spacing and edge distance, their type, length and
embedment; its `thickness` is the cap's. The cap's outline follows from
the layout (`PileCap::outline`), in the cap's frame: the origin at its
position, `u` along its rotation and `v` to the left of it.
The piles of a cap are elements pointing at it (`Pile::cap`, with their
index in the layout). Every edit keeps them in step with the cap and its
type (see `crate::EditCtx`): piles are added or removed when the
layout's count changes, and each one stands where the layout puts it
through the cap's placement, on the cap's level, its head at the cap's
bottom plus the embedment, its toe the pile length below. They are
deleted, moved and copied with the cap; moved alone they go back. A pile
whose cap is gone (or is no pile cap) is dropped, like a building pad
without its terrain.

**Items:** [`MAX_PILES`](#cc_core-pile-max_piles) · [`MIN_PILE_SIZE`](#cc_core-pile-min_pile_size) · [`MAX_PILE_SIZE`](#cc_core-pile-max_pile_size) · [`MIN_PILE_LENGTH`](#cc_core-pile-min_pile_length) · [`MAX_PILE_LENGTH`](#cc_core-pile-max_pile_length) · [`MAX_OFFSET`](#cc_core-pile-max_offset) · [`MAX_EMBEDMENT`](#cc_core-pile-max_embedment) · [`MIN_SPACING`](#cc_core-pile-min_spacing) · [`MAX_SPACING`](#cc_core-pile-max_spacing) · [`MIN_EDGE_DISTANCE`](#cc_core-pile-min_edge_distance) · [`MAX_EDGE_DISTANCE`](#cc_core-pile-max_edge_distance) · [`MIN_CIRCLE_RADIUS`](#cc_core-pile-min_circle_radius) · [`MAX_CIRCLE_RADIUS`](#cc_core-pile-max_circle_radius) · [`DEFAULT_DIAMETER`](#cc_core-pile-default_diameter) · [`DEFAULT_PILE_LENGTH`](#cc_core-pile-default_pile_length) · [`DEFAULT_EMBEDMENT`](#cc_core-pile-default_embedment) · [`DEFAULT_CAP_THICKNESS`](#cc_core-pile-default_cap_thickness) · [`PileKind`](#cc_core-pile-pilekind) · [`PileSection`](#cc_core-pile-pilesection) · [`PileCapRef`](#cc_core-pile-pilecapref) · [`Pile`](#cc_core-pile-pile) · [`length_ok`](#cc_core-pile-length_ok) · [`PileLayout`](#cc_core-pile-pilelayout) · [`CapOutline`](#cc_core-pile-capoutline) · [`PileCap`](#cc_core-pile-pilecap) · [`cap_of_type`](#cc_core-pile-cap_of_type) · [`cap_placement`](#cc_core-pile-cap_placement) · [`CapPlacement`](#cc_core-pile-capplacement) · [`PileSync`](#cc_core-pile-pilesync)

<a id="cc_core-pile-max_piles"></a>

### const `MAX_PILES`

```rust
pub const MAX_PILES: u32
```

Most piles under one cap.

<a id="cc_core-pile-min_pile_size"></a>

### const `MIN_PILE_SIZE`

```rust
pub const MIN_PILE_SIZE: f64
```

Smallest and largest pile diameter (or square side), meters.

<a id="cc_core-pile-max_pile_size"></a>

### const `MAX_PILE_SIZE`

```rust
pub const MAX_PILE_SIZE: f64
```

<a id="cc_core-pile-min_pile_length"></a>

### const `MIN_PILE_LENGTH`

```rust
pub const MIN_PILE_LENGTH: f64
```

Shortest and longest pile, meters.

<a id="cc_core-pile-max_pile_length"></a>

### const `MAX_PILE_LENGTH`

```rust
pub const MAX_PILE_LENGTH: f64
```

<a id="cc_core-pile-max_offset"></a>

### const `MAX_OFFSET`

```rust
pub const MAX_OFFSET: f64
```

Largest offset of a pile's head or toe from its level, meters.

<a id="cc_core-pile-max_embedment"></a>

### const `MAX_EMBEDMENT`

```rust
pub const MAX_EMBEDMENT: f64
```

Deepest embedment of a pile's head into its cap, meters.

<a id="cc_core-pile-min_spacing"></a>

### const `MIN_SPACING`

```rust
pub const MIN_SPACING: f64
```

Closest and widest pile spacing in a cap (center to center), meters.

<a id="cc_core-pile-max_spacing"></a>

### const `MAX_SPACING`

```rust
pub const MAX_SPACING: f64
```

<a id="cc_core-pile-min_edge_distance"></a>

### const `MIN_EDGE_DISTANCE`

```rust
pub const MIN_EDGE_DISTANCE: f64
```

Smallest and largest edge distance (pile center to cap edge), meters.

<a id="cc_core-pile-max_edge_distance"></a>

### const `MAX_EDGE_DISTANCE`

```rust
pub const MAX_EDGE_DISTANCE: f64
```

<a id="cc_core-pile-min_circle_radius"></a>

### const `MIN_CIRCLE_RADIUS`

```rust
pub const MIN_CIRCLE_RADIUS: f64
```

Smallest and largest radius of a circle of piles, meters.

<a id="cc_core-pile-max_circle_radius"></a>

### const `MAX_CIRCLE_RADIUS`

```rust
pub const MAX_CIRCLE_RADIUS: f64
```

<a id="cc_core-pile-default_diameter"></a>

### const `DEFAULT_DIAMETER`

```rust
pub const DEFAULT_DIAMETER: f64
```

Defaults of a new pile type and pile cap type.

<a id="cc_core-pile-default_pile_length"></a>

### const `DEFAULT_PILE_LENGTH`

```rust
pub const DEFAULT_PILE_LENGTH: f64
```

<a id="cc_core-pile-default_embedment"></a>

### const `DEFAULT_EMBEDMENT`

```rust
pub const DEFAULT_EMBEDMENT: f64
```

<a id="cc_core-pile-default_cap_thickness"></a>

### const `DEFAULT_CAP_THICKNESS`

```rust
pub const DEFAULT_CAP_THICKNESS: f64
```

<a id="cc_core-pile-pilekind"></a>

### enum `PileKind`

```rust
pub enum PileKind
```

How a pile is made.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Bored`

  Drilled and cast in place.

- `ContinuousFlightAuger`

  Continuous flight auger: concreted through the hollow auger as it is withdrawn.

- `Micro`

  Micropiles and root piles: small drilled and grouted piles.

- `PrecastDriven`

  Precast concrete driven into the ground.

- `Steel`

  Steel sections or tubes driven into the ground.

- `Other`

**Methods**

- `pub const ALL: [PileKind; 6]`
- `pub fn key(self) -> &'static str`

  Translation key of the kind's name.

- `pub fn code(self) -> &'static str`

  Short name used by commands (`bored`, `cfa`, `micro`, `precast`, `steel`, `other`).

- `pub fn from_code(code: &str) -> Option<PileKind>`

  The kind named by `code` (see `Self::code`), ignoring case.


<a id="cc_core-pile-pilesection"></a>

### enum `PileSection`

```rust
pub enum PileSection
```

A pile's cross-section, centered on its axis.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`, `Default`

**Variants**

- `Round { diameter: f64, }`
- `Square { side: f64, }`

  A square `side` wide, turned with the pile's rotation.


**Methods**

- `pub fn size(&self) -> f64`

  The diameter, or the side of a square.

- `pub fn area(&self) -> f64`

  Exact area of the section.

- `pub fn is_valid(&self) -> bool`

  Whether the size is a finite number within the limits.

- `pub fn repair(&mut self) -> bool`

  Bring the size within the limits (the default when not a number);
  returns whether it changed.


<a id="cc_core-pile-pilecapref"></a>

### struct `PileCapRef`

```rust
pub struct PileCapRef
```

The cap a pile belongs to and its place in the cap's layout.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Serialize`, `Deserialize`

**Fields**

- `cap: ElementId`
- `index: u32`

<a id="cc_core-pile-pile"></a>

### struct `Pile`

```rust
pub struct Pile
```

A pile (see the module documentation).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `position: DVec2`

  Axis in plan.

- `rotation_deg: f64`

  Rotation of a square section in plan, degrees counter-clockwise.

- `cut_off: f64`

  Top of the pile (its cut-off) above the level, meters (negative below).

- `toe: f64`

  Bottom of the pile above the level, meters (below `cut_off`).

- `embedment: f64`

  How far the head enters the cap above it, meters.

- `cap: Option<PileCapRef>`

  The pile cap that owns the pile (`None`: a free pile).


**Methods**

- `pub fn new(position: DVec2, cut_off: f64, length: f64) -> Self`

  A free pile from its head down `length`.

- `pub fn length(&self) -> f64`

  Length from the head to the toe.

- `pub fn is_valid(&self) -> bool`

  Every number finite, the offsets within `MAX_OFFSET`.

- `pub fn repair(&mut self) -> usize`

  Bring the embedment and the length within their limits (a toe not
  below the head goes the default length down); returns how many values
  changed.


<a id="cc_core-pile-length_ok"></a>

### fn `length_ok`

```rust
pub fn length_ok(length: f64) -> bool
```

Whether `length` is a pile length within the limits.

<a id="cc_core-pile-pilelayout"></a>

### enum `PileLayout`

```rust
pub enum PileLayout
```

Where the piles of a cap stand, in the cap's frame (`spacing` is the
distance between neighbors, except on a circle).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`

**Variants**

- `Single`

  One pile at the center.

- `Pair`

  Two piles along `u`, `spacing` apart.

- `Triangle`

  Three piles at the corners of an equilateral triangle of side
  `spacing` centered on the origin, the first on `+u` (the cap points
  along its rotation).

- `Square`

  Four piles at the corners of a square of side `spacing`.

- `Line { count: u32 }`

  `count` piles along `u`, `spacing` apart, centered.

- `Circle { count: u32, radius: f64, #[serde(default, skip_serializing_if = "Option::is_none")] ring_width: Option<f64>, }`

  `count` piles on a circle of `radius`, the first on `+u`: a round
  cap (`radius` plus the edge distance), or a ring `ring_width` wide
  centered on the circle.


**Methods**

- `pub fn count(&self) -> u32`

  Number of piles.

- `pub fn key(&self) -> &'static str`

  Translation key of the layout's name.


<a id="cc_core-pile-capoutline"></a>

### enum `CapOutline`

```rust
pub enum CapOutline
```

The plan of a pile cap in its frame.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Variants**

- `Polygon(Vec<DVec2>)`

  A convex polygon, counter-clockwise.

- `Disk { radius: f64 }`

  A disk centered on the origin.

- `Ring { inner: f64, outer: f64 }`

  An annulus centered on the origin.


**Methods**

- `pub fn area(&self) -> f64`

  Exact plan area.

- `pub fn bounds(&self) -> (DVec2, DVec2)`

  The bounding box, `(min, max)`.

- `pub fn loops(&self, segments: usize) -> Vec<Vec<DVec2>>`

  The outer boundary and the holes as polygons (counter-clockwise; a
  circle as `segments` chords).


<a id="cc_core-pile-pilecap"></a>

### struct `PileCap`

```rust
pub struct PileCap
```

What a pile cap type holds besides its thickness and material: its piles.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `layout: PileLayout`
- `spacing: f64`

  Center-to-center distance between neighboring piles (unused on a circle).

- `edge_distance: f64`

  Distance from a pile's center to the cap's edge (on a circle: to the
  outer edge of a round cap; a ring has its width).

- `pile_type: Option<ElementTypeId>`

  The type of the cap's piles (`None`: the project's first pile type).

- `pile_length: f64`

  Length of each pile, from its head to its toe.

- `embedment: f64`

  How far each pile's head enters the cap.


**Methods**

- `pub fn positions(&self) -> Vec<DVec2>`

  Pile positions in the cap's frame, in layout order.

- `pub fn outline(&self) -> CapOutline`

  The cap's plan in its frame: a rectangle `edge_distance` around the
  piles of one, two, four and a line; for three, the triangle whose
  sides lie `edge_distance` off the piles with its corners cut square
  `edge_distance` past the corner piles; on a circle a disk, or the
  ring.

- `pub fn problem(&self, pile_size: f64) -> Option<String>`

  Why the layout cannot hold piles `pile_size` across (`None` when it
  can): piles apart, inside the cap, numbers within the limits.

- `pub fn repair(&mut self) -> usize`

  Bring every value within its limits (a number that is not one takes
  the default); returns how many changed. The pile type is checked
  against the project by `Project::repair`.


<a id="cc_core-pile-cap_of_type"></a>

### fn `cap_of_type`

```rust
pub fn cap_of_type(data: &TypeData) -> Option<(&PileCap, f64)>
```

The pile cap a foundation type describes, with the cap's thickness.

<a id="cc_core-pile-cap_placement"></a>

### fn `cap_placement`

```rust
pub fn cap_placement(e: &Element) -> Option<(DVec2, f64, f64)>
```

A pile cap element's placement: position, rotation (degrees) and top above its level.

<a id="cc_core-pile-capplacement"></a>

### struct `CapPlacement`

```rust
pub struct CapPlacement<'a>
```

A pile cap element as the sync and the geometry see it.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `cap: &'a PileCap`
- `thickness: f64`
- `position: DVec2`
- `rotation_deg: f64`
- `top_offset: f64`

**Methods**

- `pub fn world(&self, local: DVec2) -> DVec2`

  A point of the cap's frame in plan.

- `pub fn pile_positions(&self) -> Vec<DVec2>`

  The piles' axes in plan, in layout order.

- `pub fn cut_off(&self) -> f64`

  The head of the cap's piles above its level: the cap's bottom plus the embedment.


<a id="cc_core-pile-pilesync"></a>

### struct `PileSync`

```rust
pub(crate) struct PileSync
```

The changes that bring piles in step with their caps.

**Traits:** `Debug`, `Default`

**Fields**

- `types: Vec<(ElementTypeId, f64, f64)>`

  Cap types whose width and length differ from their outline's size.

- `remove: Vec<ElementId>`

  Piles of no cap (or of a place another pile takes).

- `update: Vec<(ElementId, crate::LevelId, ElementTypeId, Pile)>`

  Piles to put where their cap wants them: level, type and pile.

- `add: Vec<(crate::LevelId, ElementTypeId, Pile)>`

  Missing piles of caps.


<a id="mod-cc_core-profile"></a>

## `cc_core::profile`

<sub>`crates/cc_core/src/profile.rs`</sub>

Profiles of component pieces: closed outlines of straight segments,
arcs and parts of ellipses drawn on one of the component's planes (plan, front or side), with
every coordinate an `crate::expr` expression, and the solids made from
them: an extrusion across the plane or a revolution about an axis in it.
A profile's plane is placed at `offset` along the axis normal to it (Z
for the plan, Y for the front, X for the side), and an extrusion runs
`depth` from there along that axis. The solids come out as closed meshes
with outward faces, in the component's frame (X to the right, Y to the
back, Z up).

**Items:** [`SketchPlane`](#cc_core-profile-sketchplane) · [`ProfilePoint`](#cc_core-profile-profilepoint) · [`ProfileBezier`](#cc_core-profile-profilebezier) · [`ProfileEllipse`](#cc_core-profile-profileellipse) · [`Segment`](#cc_core-profile-segment) · [`vertices`](#cc_core-profile-vertices) · [`segment_points`](#cc_core-profile-segment_points) · [`ProfileError`](#cc_core-profile-profileerror) · [`outline`](#cc_core-profile-outline) · [`extrude`](#cc_core-profile-extrude) · [`revolve`](#cc_core-profile-revolve) · [`from_curves`](#cc_core-profile-from_curves) · [`to_curves`](#cc_core-profile-to_curves) · [`written`](#cc_core-profile-written)

<a id="cc_core-profile-sketchplane"></a>

### enum `SketchPlane`

```rust
pub enum SketchPlane
```

The plane a profile is drawn on, as seen in the matching view.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Plan`

  Seen from above: U along X, V along Y; placed and extruded along Z.

- `Front`

  Seen from the front (-Y): U along X, V along Z; placed and extruded along Y.

- `Side`

  Seen from the right (+X): U along Y, V along Z; placed and extruded along X.


**Methods**

- `pub const ALL: [SketchPlane; 3]`
- `pub fn key(self) -> &'static str`
- `pub fn point(self, u: f64, v: f64, w: f64) -> DVec3`

  The point at `u`, `v` on the plane placed at `w` on its normal axis.


<a id="cc_core-profile-profilepoint"></a>

### struct `ProfilePoint`

```rust
pub struct ProfilePoint
```

A vertex of a profile. The segment to the next vertex is straight, an
arc of `radius` when it is not empty (a positive radius bulges to the
right of the direction of travel, outwards in a counter-clockwise
profile; a negative one to the left), a part of `ellipse`, or a cubic
Bézier curve by its two `bezier` handles (a spline's segment).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `u: String`
- `v: String`
- `radius: String`
- `ellipse: Option<ProfileEllipse>`
- `bezier: Option<ProfileBezier>`

**Methods**

- `pub fn new(u: impl Into<String>, v: impl Into<String>) -> Self`
- `pub fn arc(u: impl Into<String>, v: impl Into<String>, radius: impl Into<String>) -> Self`
- `pub fn expressions(&self) -> Vec<(&'static str, &String)>`

  Every expression of the point, with a field label (an arc's radius
  and an ellipse's rotation only when set).

- `pub fn expressions_mut(&mut self) -> Vec<&mut String>`

<a id="cc_core-profile-profilebezier"></a>

### struct `ProfileBezier`

```rust
pub struct ProfileBezier
```

The two handles of a cubic Bézier segment from a profile's vertex to the
next: (`u1`, `v1`) leaving the vertex, (`u2`, `v2`) arriving at the next.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `u1: String`
- `v1: String`
- `u2: String`
- `v2: String`

<a id="cc_core-profile-profileellipse"></a>

### struct `ProfileEllipse`

```rust
pub struct ProfileEllipse
```

The ellipse a profile segment runs along, from its vertex to the next:
center (`u`, `v`), semi-axes `rx` and `ry` along its own axes, turned
`rotation` degrees; counter-clockwise unless `clockwise`. The vertices
should lie on it (each is taken at the ellipse's parameter of its
direction from the center).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `u: String`
- `v: String`
- `rx: String`
- `ry: String`
- `rotation: String`
- `clockwise: bool`

<a id="cc_core-profile-segment"></a>

### enum `Segment`

```rust
pub enum Segment
```

The segment from a profile's vertex to the next, in numbers.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Variants**

- `Line`
- `Arc { radius: f64, }`

  An arc of `radius` (signed like `ProfilePoint::radius`).

- `Ellipse { center: DVec2, radii: DVec2, rotation_deg: f64, clockwise: bool, }`
- `Bezier { c1: DVec2, c2: DVec2, }`

  A cubic Bézier curve with these two handles.


<a id="cc_core-profile-vertices"></a>

### fn `vertices`

```rust
pub fn vertices(points: &[ProfilePoint], lookup: &dyn Fn(&str) -> Option<f64>) -> Result<Vec<(DVec2, Segment)>, ProfileError>
```

The vertices of `points` and the segments from each to the next, in numbers.

<a id="cc_core-profile-segment_points"></a>

### fn `segment_points`

```rust
pub(crate) fn segment_points(a: DVec2, b: DVec2, segment: Segment, out: &mut Vec<DVec2>) -> Result<(), ProfileError>
```

The inner points of the segment from `a` to `b`.

<a id="cc_core-profile-profileerror"></a>

### enum `ProfileError`

```rust
pub enum ProfileError
```

Why a profile could not be resolved.

**Traits:** `Debug`, `Clone`, `PartialEq`, `std::fmt::Display`

**Variants**

- `Expr { field: &'static str, error: ExprError, }`
- `Degenerate`

  Fewer than 3 distinct points, no area, an arc shorter than its chord,
  a zero size or a profile crossing its revolution axis.


<a id="cc_core-profile-outline"></a>

### fn `outline`

```rust
pub fn outline(points: &[ProfilePoint], lookup: &dyn Fn(&str) -> Option<f64>) -> Result<Vec<DVec2>, ProfileError>
```

The outline of `points` in plane coordinates, arcs divided into segments,
counter-clockwise.

<a id="cc_core-profile-extrude"></a>

### fn `extrude`

```rust
pub fn extrude(plane: SketchPlane, outline: &[DVec2], offset: f64, depth: f64) -> Vec<Vec<DVec3>>
```

The closed solid of a counter-clockwise `outline` on `plane`, from `offset`
along the plane's normal to `offset + depth`.

<a id="cc_core-profile-revolve"></a>

### fn `revolve`

```rust
pub fn revolve(plane: SketchPlane, outline: &[DVec2], offset: f64, axis: f64, angle: f64) -> Result<Vec<Vec<DVec3>>, ProfileError>
```

The solid swept by a counter-clockwise `outline` on `plane` (placed
`offset` along its normal) turning `angle` degrees about the line
`u = axis` of the plane (parallel to V). The outline must lie on one side
of the axis; it may touch it.

<a id="cc_core-profile-from_curves"></a>

### fn `from_curves`

```rust
pub fn from_curves(curves: &[SketchCurve], closed: bool) -> Result<Vec<(DVec2, Segment)>, SketchError>
```

The vertices and segments that draw the sketch `curves`: their one
closed loop when `closed`, or else the open path they make (its last
vertex ends it, with a straight segment that is not drawn). Arcs over
half a turn split in two, circles and whole ellipses into halves.

<a id="cc_core-profile-to_curves"></a>

### fn `to_curves`

```rust
pub fn to_curves(vertices: &[(DVec2, Segment)], closed: bool) -> Vec<SketchCurve>
```

The sketch curves that draw `vertices` (the segment from the last vertex
back to the first only when `closed`).

<a id="cc_core-profile-written"></a>

### fn `written`

```rust
pub fn written(vertices: &[(DVec2, Segment)]) -> Vec<ProfilePoint>
```

Profile points that write `vertices` as plain numbers.

<a id="mod-cc_core-profile_catalog"></a>

## `cc_core::profile_catalog`

<sub>`crates/cc_core/src/profile_catalog.rs`</sub>

Built-in profile catalogs: the standard series of rolled, cold-formed,
timber and hollow sections, as tables (cells split by `;`, since the
designations write decimals with a comma) compiled into the binary and
parsed once. Each row is a designation (the full name the standard gives,
e.g. `W150x22,5 (H)`) and the dimensions of the series'
`ProfileShape` in millimeters (taken from public dimension tables), plus
the tabulated area in cm², kept only to check the computed outlines.
Catalogs are not stored in projects: importing a row copies it in as a
`ProfileDef` that remembers its `CatalogRef`.

**Items:** [`MAX_ROWS`](#cc_core-profile_catalog-max_rows) · [`Series`](#cc_core-profile_catalog-series) · [`CatalogRow`](#cc_core-profile_catalog-catalogrow) · [`catalog`](#cc_core-profile_catalog-catalog) · [`series`](#cc_core-profile_catalog-series) · [`row_to_def`](#cc_core-profile_catalog-row_to_def)

<a id="cc_core-profile_catalog-max_rows"></a>

### const `MAX_ROWS`

```rust
pub const MAX_ROWS: usize
```

Most rows read from one table.

<a id="cc_core-profile_catalog-series"></a>

### struct `Series`

```rust
pub struct Series
```

A catalog series: rows of one shape and material.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `code: &'static str`

  Stable code, e.g. `"W"`, `"NBR6355-Ue"`.

- `name_key: &'static str`

  The i18n key of the series' name.

- `material: ProfileMaterial`
- `shape: &'static str`

  The `ProfileShape::code` of the rows.

- `rows: Vec<CatalogRow>`

<a id="cc_core-profile_catalog-catalogrow"></a>

### struct `CatalogRow`

```rust
pub struct CatalogRow
```

One row of a series.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `designation: String`
- `values: Vec<(String, f64)>`

  The shape's values by field name: meters, `angle` in degrees.

- `area: f64`

  The tabulated area, m² (reference only; 0 when the table has none).


<a id="cc_core-profile_catalog-catalog"></a>

### fn `catalog`

```rust
pub fn catalog() -> &'static [Series]
```

Every built-in series, parsed on first use; rows that do not parse are left out.

<a id="cc_core-profile_catalog-series"></a>

### fn `series`

```rust
pub fn series(code: &str) -> Option<&'static Series>
```

The series of that code.

<a id="cc_core-profile_catalog-row_to_def"></a>

### fn `row_to_def`

```rust
pub fn row_to_def(series: &Series, row: &CatalogRow) -> Option<ProfileDef>
```

A profile for a row (id 0 and a new uid; the caller sets the id), named
by its designation, or `None` when the row does not make a valid outline.
Values the row lacks are 0, except a Z's lip angle, which is square (90°).

<a id="mod-cc_core-profile_sweep"></a>

## `cc_core::profile_sweep`

<sub>`crates/cc_core/src/profile_sweep.rs`</sub>

Swept profiles: a closed profile, held by the element's type, carried
along a path drawn in plan at a height above the element's level. They
model what runs along walls, ceilings and roof edges: baseboards, crown
moldings, fascia boards, gutters, copings.
The profile is drawn in its own plane, square to the path: `x` across,
positive to the left of the path walking along it (`flip` mirrors it to
the right), and `y` up from the path's height.

**Items:** [`MAX_PROFILE_POINTS`](#cc_core-profile_sweep-max_profile_points) · [`MAX_PROFILE_SIZE`](#cc_core-profile_sweep-max_profile_size) · [`MAX_PATH_POINTS`](#cc_core-profile_sweep-max_path_points) · [`Sweep`](#cc_core-profile_sweep-sweep) · [`ProfileProblem`](#cc_core-profile_sweep-profileproblem) · [`check_profile`](#cc_core-profile_sweep-check_profile) · [`baseboard`](#cc_core-profile_sweep-baseboard)

<a id="cc_core-profile_sweep-max_profile_points"></a>

### const `MAX_PROFILE_POINTS`

```rust
pub const MAX_PROFILE_POINTS: usize
```

Most points of a profile.

<a id="cc_core-profile_sweep-max_profile_size"></a>

### const `MAX_PROFILE_SIZE`

```rust
pub const MAX_PROFILE_SIZE: f64
```

Largest extent of a profile, meters.

<a id="cc_core-profile_sweep-max_path_points"></a>

### const `MAX_PATH_POINTS`

```rust
pub const MAX_PATH_POINTS: usize
```

Most points of a sweep's path.

<a id="cc_core-profile_sweep-sweep"></a>

### struct `Sweep`

```rust
pub struct Sweep
```

A swept profile element.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `path: Vec<DVec2>`

  The path in plan, in order.

- `closed: bool`

  The path runs back from its last point to its first (around a room).

- `base_offset: f64`

  Height of the path above the level.

- `flip: bool`

  The profile on the right of the path instead of the left.


**Methods**

- `pub fn length(&self) -> f64`

  Plan length of the path.

- `pub fn is_valid(&self) -> bool`

<a id="cc_core-profile_sweep-profileproblem"></a>

### enum `ProfileProblem`

```rust
pub enum ProfileProblem
```

Why a profile is refused.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Variants**

- `TooFewPoints`
- `TooManyPoints`
- `NotFinite`
- `TooLarge`
- `NoArea`
- `Crossing`

**Methods**

- `pub fn message(self) -> &'static str`

<a id="cc_core-profile_sweep-check_profile"></a>

### fn `check_profile`

```rust
pub fn check_profile(points: &[DVec2]) -> Result<(), ProfileProblem>
```

Check a profile: a simple closed outline of a sensible size.

<a id="cc_core-profile_sweep-baseboard"></a>

### fn `baseboard`

```rust
pub fn baseboard() -> Vec<DVec2>
```

The profile of a new project's sweep type: a baseboard 1.5 cm thick and
10 cm tall, on the left of the path.

<a id="mod-cc_core-profiles_project"></a>

## `cc_core::profiles_project`

<sub>`crates/cc_core/src/profiles_project.rs`</sub>

Structural profiles in the project: lookups and repair of loaded files.

**Items:** [`TypeSection`](#cc_core-profiles_project-typesection) · [`section_name`](#cc_core-profiles_project-section_name) · [`section_name_in`](#cc_core-profiles_project-section_name_in) · [`SectorPlacement`](#cc_core-profiles_project-sectorplacement) · [`type_profile`](#cc_core-profiles_project-type_profile) · [`same_legacy`](#cc_core-profiles_project-same_legacy) · [`legacy_size`](#cc_core-profiles_project-legacy_size)

<a id="cc_core-profiles_project-typesection"></a>

### struct `TypeSection`

```rust
pub struct TypeSection
```

The section a column or beam type draws: its profile's outline turned and
mirrored, or the legacy section's.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `outline: SectionOutline`
- `profile: Option<ProfileId>`

  The profile it comes from; `None` for a legacy section.

- `material: ProfileMaterial`
- `curved: bool`

  Whether the outline has arcs (drawn as short chords).


<a id="cc_core-profiles_project-section_name"></a>

### fn `section_name`

```rust
pub fn section_name(shape: &ProfileShape, material: ProfileMaterial) -> String
```

The name a converted section's profile gets: `I 200x200x9x15`,
`Rect 20x40` (centimeters for concrete and timber rectangles and rounds,
as section tags show them), `Ø30`, `Tube 150x150x6`.

<a id="cc_core-profiles_project-section_name_in"></a>

### fn `section_name_in`

```rust
pub fn section_name_in(shape: &ProfileShape, material: ProfileMaterial, system: crate::MeasurementSystem) -> String
```

`section_name`, or in an imperial project the same name with every
size in inches (`I 8"x8"x5/16"x7/16"`, `Rect 8"x8"`, `Ø12"`).

<a id="cc_core-profiles_project-sectorplacement"></a>

### struct `SectorPlacement`

```rust
pub struct SectorPlacement
```

An annular sector section in its type's frame: the ring's `center`, the
unit `axis` from it through the middle of the sector, the radii and the
angle it turns (radians).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `center: DVec2`
- `axis: DVec2`
- `r_in: f64`
- `r_out: f64`
- `sweep: f64`

<a id="cc_core-profiles_project-type_profile"></a>

### fn `type_profile`

```rust
pub(crate) fn type_profile(data: &TypeData) -> Option<ProfileId>
```

The profile a column or beam type points to.

<a id="cc_core-profiles_project-same_legacy"></a>

### fn `same_legacy`

```rust
pub(crate) fn same_legacy(a: &TypeData, b: &TypeData) -> bool
```

Whether two column or beam types have the same legacy section fields
(`section`, or `width`, `height` and `shape`).

<a id="cc_core-profiles_project-legacy_size"></a>

### fn `legacy_size`

```rust
pub(crate) fn legacy_size(data: &TypeData) -> Option<(f64, f64)>
```

The legacy box of a column or beam type, `(width, depth or height)`.

<a id="mod-cc_core-project"></a>

## `cc_core::project`

<sub>`crates/cc_core/src/project.rs`</sub>

**Items:** [`ProjectField`](#cc_core-project-projectfield) · [`ProjectInfo`](#cc_core-project-projectinfo) · [`Level`](#cc_core-project-level) · [`Project`](#cc_core-project-project) · [`Group`](#cc_core-project-group) · [`template_default`](#cc_core-project-template_default) · [`can_join_geometry`](#cc_core-project-can_join_geometry) · [`join_priority`](#cc_core-project-join_priority) · [`natural_cmp`](#cc_core-project-natural_cmp)

<a id="cc_core-project-projectfield"></a>

### struct `ProjectField`

```rust
pub struct ProjectField
```

A project value named by the user (shown by title blocks as `{project:Name}`).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `name: String`
- `value: String`

<a id="cc_core-project-projectinfo"></a>

### struct `ProjectInfo`

```rust
pub struct ProjectInfo
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `uid: crate::Uid`

  Persistent unique id of the project.

- `name: String`
- `number: String`
- `client: String`
- `address: String`
- `author: String`
- `fields: Vec<ProjectField>`

  Fields named by the user, in the order they were added.

- `site: Option<crate::site::SiteLocation>`

  Where the project stands on Earth (see `crate::site`).


**Methods**

- `pub fn field(&self, name: &str) -> Option<&str>`

  The value of the field called `name`.


<a id="cc_core-project-level"></a>

### struct `Level`

```rust
pub struct Level
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `id: LevelId`
- `name: String`
- `elevation: f64`

  Elevation in meters.


<a id="cc_core-project-project"></a>

### struct `Project`

```rust
pub struct Project
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `info: ProjectInfo`
- `levels: Vec<Level>`

  Sorted by elevation.

- `materials: Vec<Material>`
- `types: BTreeMap<ElementTypeId, ElementType>`
- `elements: BTreeMap<ElementId, Element>`
- `views: BTreeMap<ViewId, View>`
- `units: crate::Units`

  How quantities are displayed and typed.

- `pinned: std::collections::BTreeSet<ElementId>`

  Elements protected from moving and deleting.

- `joins: std::collections::BTreeSet<(ElementId, ElementId)>`

  Joined geometry as `(cutting, cut)` pairs: where the two overlap,
  the first keeps the volume and their cut outlines merge.

- `groups: BTreeMap<crate::GroupId, Group>`

  Named sets of elements selected, moved and copied together.

- `systems: BTreeMap<crate::SystemId, crate::MepSystem>`

  Plumbing systems and electrical circuits.

- `filters: BTreeMap<crate::FilterId, crate::ViewFilter>`

  Named view filters (categories and rules), applied by views.

- `templates: BTreeMap<crate::TemplateId, crate::ViewTemplate>`

  View templates.

- `phases: Vec<crate::Phase>`

  Construction phases, in time order (see `crate::phase`).

- `phase_filters: BTreeMap<crate::PhaseFilterId, crate::PhaseFilter>`

  Phase filters views choose from.

- `phase_graphics: crate::PhaseGraphics`

  Graphics of each phase status, where a phase filter overrides it.

- `graph_bindings: BTreeMap<ElementId, crate::graph_binding::GraphBinding>`

  Elements made by visual programming graphs, and the call that made
  each (see `crate::graph_binding`).

- `line_styles: crate::detail::LineStyles`

  The project's own line styles (see `crate::detail`), used by detail
  lines and region boundaries next to the built-in ones.

- `fill_patterns: crate::fill_pattern::FillPatterns`

  The project's own fill patterns (see `crate::fill_pattern`), used
  by materials, filled regions and view overrides next to the built-in ones.

- `profiles: BTreeMap<crate::ProfileId, crate::structural_profile::ProfileDef>`

  Named structural profiles (tracked as settings).

- `option_sets: Vec<crate::OptionSet>`

  Design option sets (see `crate::design_option`).

- `browser: crate::browser::BrowserOrganization`

  How the project browser groups the views (see `crate::browser`).

- `bar_marking: crate::bar_marks::BarMarking`

  How reinforcing bars are marked (see `crate::bar_marks`).

- `sun_presets: Vec<crate::sun::SunPreset>`

  Named sun settings views can take (see `crate::sun`).

- `active_option: Option<crate::OptionId>`

  The design option new elements go into and views without a choice of
  their own show for its set: session state, not saved nor undone (set
  through `crate::Document::set_active_option`).


*Some fields are private.*

**Methods**

- `pub fn browser_tree(&self) -> Vec<Node>` *(in `cc_core::browser`)*

  The browser tree under the scheme in use.

- `pub fn callouts_in(&self, parent: ViewId) -> impl Iterator<Item = &View>` *(in `cc_core::callout`)*

  The views whose callouts are drawn in `parent`, in id order.

- `pub fn callout_parent(&self, view: ViewId) -> Option<&View>` *(in `cc_core::callout`)*

  The parent view of the view's callout, if it has one.

- `pub fn detail_kind(&self, view: &View) -> Option<ViewKind>` *(in `cc_core::callout`)*

  The kind a detail view takes from its parent: a section's or
  elevation's whole kind (its drawing coordinates are the parent's),
  a plan's level (cut height, underlay and the rest stay its own).
  `None` when nothing changes.

- `pub fn repair_callouts(&mut self) -> usize` *(in `cc_core::callout`)*

  Repair crops and callouts: drop invalid crops and crops of views
  that cannot have one, and callouts whose parent is missing, is the
  view itself or cannot hold them, of another kind than a detail view,
  or with no valid rectangle. Returns how many were fixed.

- `pub fn connections_of(&self, member: ElementId) -> impl Iterator<Item = (&Element, &Connection)>` *(in `cc_core::connection`)*

  The connections on `member`.

- `pub fn connection_at(&self, member: ElementId, end: MemberEnd) -> Option<ElementId>` *(in `cc_core::connection`)*

  The connection at `member`'s `end`, if it has one.

- `pub(crate) fn repair_connections(&mut self) -> usize` *(in `cc_core::connection`)*

  From a file: one connection per member end (the lowest id stays),
  numbers back in range; returns the repairs.

- `pub fn option_set(&self, id: OptionSetId) -> Option<&OptionSet>` *(in `cc_core::design_option`)*
- `pub fn option_set_of(&self, option: OptionId) -> Option<&OptionSet>` *(in `cc_core::design_option`)*

  The set an option belongs to.

- `pub fn design_option(&self, id: OptionId) -> Option<(&OptionSet, &DesignOption)>` *(in `cc_core::design_option`)*

  An option and its set.

- `pub fn option_label(&self, id: OptionId) -> String` *(in `cc_core::design_option`)*

  "Set: Option", or empty for unknown options.

- `pub fn active_option(&self) -> Option<OptionId>` *(in `cc_core::design_option`)*

  The option being edited, when it still exists.

- `pub fn primary_options(&self) -> BTreeSet<OptionId>` *(in `cc_core::design_option`)*

  The primary option of every set (what exports write).

- `pub fn shown_options(&self, view: Option<&View>) -> BTreeSet<OptionId>` *(in `cc_core::design_option`)*

  The option of each set that `view` shows (see the module docs); with
  no view, what views without a choice of their own show.

- `pub fn option_shows(&self, e: &Element, shown: &BTreeSet<OptionId>) -> bool` *(in `cc_core::design_option`)*

  Whether `e` is part of a model showing the options `shown`.

- `pub fn options_hide_any(&self, shown: &BTreeSet<OptionId>) -> bool` *(in `cc_core::design_option`)*

  Whether showing the options `shown` leaves any element out.

- `pub fn with_options(&self, shown: &BTreeSet<OptionId>) -> Option<Project>` *(in `cc_core::design_option`)*

  A copy of the project with only the main model and the options
  `shown` (joins between what is left), or `None` when nothing would
  be left out.

- `pub fn option_elements(&self, option: OptionId) -> impl Iterator<Item = &Element>` *(in `cc_core::design_option`)*

  Elements of one option.

- `pub fn option_fixes(&self) -> Vec<(ElementId, Option<OptionId>)>` *(in `cc_core::design_option`)*

  The options that would make elements consistent: annotations in no
  option, unknown options reset to the main model, hosted elements in
  their host's option. Returns `(element, option)` for each element
  that needs a change.

- `pub(crate) fn stale_choices(&self, choices: &BTreeMap<OptionSetId, OptionId>) -> bool` *(in `cc_core::design_option`)*

  Choices of `choices` that name no set, or an option outside its set.

- `pub(crate) fn clean_choices(&self, choices: &mut BTreeMap<OptionSetId, OptionId>) -> usize` *(in `cc_core::design_option`)*

  Drop the choices of `choices` that name no set, or an option outside
  its set; returns how many.

- `pub(crate) fn repair_options(&mut self) -> usize` *(in `cc_core::design_option`)*

  Sets, options, element options and view choices; returns how many
  repairs were needed.

- `pub fn detail_def(&self, component: &str) -> Option<&ComponentDef>` *(in `cc_core::detail_component`)*

  The definition of the detail component named `component` (its
  first type's), if the project has one.

- `pub fn detail_nests(&self, component: &str, target: &str) -> bool` *(in `cc_core::detail_component`)*

  Whether the detail component `component` reaches `target` through
  its nested pieces (or is it), within `MAX_DEPTH`.

- `pub fn dim_ref_at(&self, id: ElementId, p: DVec2) -> Option<DimRef>` *(in `cc_core::dim_refs`)*

  The reference of point `p` (plan coordinates) on element `id`, when
  the element kind can carry one.

- `pub fn dim_ref_in_view(&self, view: &View, id: ElementId, p: DVec2) -> Option<DimRef>` *(in `cc_core::dim_refs`)*

  The reference of point `p` (in `view`'s drawing coordinates) on
  element `id`: in plans as `Project::dim_ref_at`; in elevations and
  sections a plan point of the element that projects to `p.x`, and a
  height from the element's base or top (whichever is nearer).

- `pub fn level_ref(&self, view: &View, level: LevelId) -> Option<DimRef>` *(in `cc_core::dim_refs`)*

  A level as a reference in an elevation or section.

- `pub fn resolve_dim_ref(&self, r: &DimRef) -> Option<DVec2>` *(in `cc_core::dim_refs`)*

  The current plan point of an element reference.

- `pub fn resolve_in_view(&self, r: &DimRef, view: &View, current: DVec2) -> Option<DVec2>` *(in `cc_core::dim_refs`)*

  The current point of a reference in `view`'s drawing coordinates;
  `current` is the end's present position (levels keep its x).

- `pub fn fill_pattern(&self, id: FillPatternId) -> Option<&FillPattern>` *(in `cc_core::fill_pattern`)*

  The custom pattern `id`.

- `pub fn fill_pattern_named(&self, name: &str) -> Option<&FillPattern>` *(in `cc_core::fill_pattern`)*

  The custom pattern called `name` (ignoring case).

- `pub fn has_pattern(&self, pattern: Pattern) -> bool` *(in `cc_core::fill_pattern`)*

  Whether `pattern` can be drawn here: a built-in, or a custom pattern
  the project has.

- `pub fn fill_pattern_usage(&self, id: FillPatternId) -> PatternUsage` *(in `cc_core::fill_pattern`)*

  Everything that uses the custom pattern `id`.

- `pub fn pattern_usage(&self, p: Pattern) -> PatternUsage` *(in `cc_core::fill_pattern`)*

  Everything that uses `p` (a built-in or custom pattern).

- `pub fn visit_patterns(&mut self, v: &mut dyn FnMut(PatternSlot))` *(in `cc_core::fill_pattern`)*

  Visits every pattern the project holds (materials, types, views,
  templates and phase graphics).

- `pub(crate) fn repair_fill_patterns(&mut self) -> usize` *(in `cc_core::fill_pattern`)*

  See `crate::fill_pattern`: ids that match their keys, patterns
  within their limits, a bounded count, and references to patterns
  that are gone dropped. Returns how many things it changed.

- `pub fn host_members_of(&self, host: ElementId) -> Vec<ElementId>` *(in `cc_core::member_host`)*

  The members a host manages (none for other elements).

- `pub fn member_host_of(&self, member: ElementId) -> Option<ElementId>` *(in `cc_core::member_host`)*

  The host that manages `member`, if any.

- `pub(crate) fn repair_member_hosts(&mut self) -> usize` *(in `cc_core::member_host`)*

  Member hosts from a file: numbers and types repaired by the host
  (it goes when a type has no fallback), members that are gone, do not
  fit their role, repeat a role or are another host's dropped, and
  repeated detached roles dropped. Returns the repairs.

- `pub fn component_types(&self) -> impl Iterator<Item = &crate::ElementType>` *(in `cc_core::openings`)*

  Types of parametric components, of any category (doors and windows too).

- `pub fn convert_opening_types(&mut self, n: &dyn Fn(&str) -> String) -> usize` *(in `cc_core::openings`)*

  Turn door and window types of the old kind (fixed width, height and
  material) into types of the built-in door and window components, with
  the same sizes; elements keep their types. Returns how many changed.

- `pub fn pads_of(&self, host: ElementId) -> impl Iterator<Item = (&crate::Element, &Pad)>` *(in `cc_core::pad`)*

  The pads of terrain `host`, in id (creation) order.

- `pub(crate) fn pad_levels(&self) -> Vec<(ElementId, crate::LevelId)>` *(in `cc_core::pad`)*

  Pads whose level differs from their terrain's and valves whose level
  differs from their pipe's, with the level they take.

- `pub fn phase_index(&self, id: PhaseId) -> Option<usize>` *(in `cc_core::phase`)*

  Position of a phase in time order.

- `pub fn phase(&self, id: PhaseId) -> Option<&Phase>` *(in `cc_core::phase`)*
- `pub fn phase_name(&self, id: PhaseId) -> String` *(in `cc_core::phase`)*
- `pub fn last_phase(&self) -> Option<PhaseId>` *(in `cc_core::phase`)*

  The latest phase (where new elements and views go by default).

- `pub fn phase_filter(&self, id: PhaseFilterId) -> Option<&PhaseFilter>` *(in `cc_core::phase`)*
- `pub fn default_phase_filter(&self) -> Option<PhaseFilterId>` *(in `cc_core::phase`)*

  The filter new views get: the first that shows everything.

- `pub fn created_index(&self, e: &Element) -> usize` *(in `cc_core::phase`)*

  The phase an element is created in, as an index (see `Self::resolved_index`).

- `pub fn demolished_index(&self, e: &Element) -> Option<usize>` *(in `cc_core::phase`)*

  The phase an element is demolished in, as an index (`None`: standing).

- `pub fn view_phase(&self, view: &crate::View) -> Option<PhaseId>` *(in `cc_core::phase`)*

  The phase a view shows (its own, or the last one).

- `pub fn phase_state_at(&self, e: &Element, phase: usize) -> PhaseState` *(in `cc_core::phase`)*

  Status of `e` in phase number `phase` (an index into `Project::phases`).

- `pub fn phase_state(&self, e: &Element, phase: PhaseId) -> PhaseState` *(in `cc_core::phase`)*

  Status of `e` in phase `phase` (unknown phases mean the last one).

- `pub fn stands_after(&self, e: &Element, phase: usize) -> bool` *(in `cc_core::phase`)*

  Whether `e` stands at the end of phase number `phase`: created then
  or before and not demolished by then (annotations always).

- `pub fn present_at(&self, e: &Element, phase: usize) -> bool` *(in `cc_core::phase`)*

  Whether `e` is part of phase number `phase` at all (standing at its
  start or built in it).

- `pub fn phase_differs(&self, phase: usize) -> bool` *(in `cc_core::phase`)*

  Whether any element is absent from phase number `phase` (its views
  then need geometry of their own).

- `pub fn phase_fixes(&self) -> Vec<(ElementId, PhaseId, Option<PhaseId>)>` *(in `cc_core::phase`)*

  The phases that would make elements consistent: a known created
  phase (unassigned stays: it means the last), a demolition not before
  creation (else none), hosted elements not created before their host
  nor demolished after it, annotations without phases. Returns
  `(element, created, demolished)` for each element that needs a
  change; empty when the project has no phases.

- `pub fn ensure_phases(&mut self, name: &dyn Fn(&str) -> String)` *(in `cc_core::phase`)*

  Give a project without phases the template's (named by `name(key)`),
  its phase filters, and put every element and view in the last phase
  with the "show all" filter. Used for files made before phases.

- `pub fn pile_cap(&self, e: &Element) -> Option<CapPlacement<'_>>` *(in `cc_core::pile`)*

  `e` as a pile cap: a foundation placed as a pad whose type is a pile cap type.

- `pub fn is_pile_cap(&self, id: ElementId) -> bool` *(in `cc_core::pile`)*

  Whether element `id` is a pile cap.

- `pub fn piles_of(&self, cap: ElementId) -> impl Iterator<Item = (&Element, &Pile)>` *(in `cc_core::pile`)*

  The piles of cap `cap`, in id order.

- `pub fn cap_pile_type(&self, cap: &PileCap) -> Option<ElementTypeId>` *(in `cc_core::pile`)*

  The type a cap's piles take: its own when it names a pile type, else
  the project's first pile type.

- `pub fn cap_piles(&self, e: &Element) -> Vec<(u32, ElementTypeId, Pile)>` *(in `cc_core::pile`)*

  The piles cap `e` should have: index, type and the pile itself.

- `pub(crate) fn pile_sync(&self) -> PileSync` *(in `cc_core::pile`)*

  What keeps the piles in step with their caps (see `PileSync`).

- `pub fn profile(&self, id: ProfileId) -> Option<&ProfileDef>` *(in `cc_core::profiles_project`)*

  A profile by id.

- `pub fn profile_named(&self, name: &str) -> Option<&ProfileDef>` *(in `cc_core::profiles_project`)*

  A profile by name, ignoring case and surrounding spaces.

- `pub fn type_profile_def(&self, data: &TypeData) -> Option<&ProfileDef>` *(in `cc_core::profiles_project`)*

  The profile a column or beam type points to, when it exists.

- `pub fn type_sector(&self, data: &TypeData) -> Option<SectorPlacement>` *(in `cc_core::profiles_project`)*

  The ring of a column or beam type whose section is an annular
  sector, as the type places it (turned and mirrored).

- `pub fn section_is_round(&self, data: &TypeData) -> bool` *(in `cc_core::profiles_project`)*

  Whether a column or beam type's section is round (a solid or hollow
  circle): its profile's shape, else its legacy section. The legacy
  fields of a profile-made type are only its box, so readers ask this.

- `pub fn exact_section_area(&self, data: &TypeData) -> Option<f64>` *(in `cc_core::profiles_project`)*

  The exact area of a column or beam type's section (its profile's,
  else its legacy section's): arcs count whole, where the outline only
  follows them with chords. `None` when the shape makes no outline.

- `pub fn type_section_properties(&self, data: &TypeData, density: f64) -> Option<crate::SectionProperties>` *(in `cc_core::profiles_project`)*

  The properties of a column or beam type's section for a material of
  `density` kg/m³. The area and the mass per meter are exact for arcs
  (`Self::exact_section_area`); the inertias are those of the outline,
  which follows arcs with chords.

- `pub fn profile_users(&self, id: ProfileId) -> Vec<ElementTypeId>` *(in `cc_core::profiles_project`)*

  The column and beam types that use a profile.

- `pub fn profile_material_for(&self, material: &str) -> ProfileMaterial` *(in `cc_core::profiles_project`)*

  The profile material of a type's material: steel for metals, timber
  for wood, concrete otherwise.

- `pub fn type_section(&self, data: &TypeData) -> Option<TypeSection>` *(in `cc_core::profiles_project`)*

  The section a column or beam type draws: its profile's outline turned
  and mirrored, or the legacy section's (a legacy section whose shape
  does not fit its box reads as the box). `None` for other types or a
  profile whose outline fails.

- `pub fn type_section_without_radii(&self, data: &TypeData) -> Option<TypeSection>` *(in `cc_core::profiles_project`)*

  `Self::type_section` with every radius of the shape (`r`, `ri`,
  `ro`, `chamfer`) at zero: arcs replaced by their corners, as medium
  detail draws it. Falls back to the rounded outline when the shape
  fails without its radii; sketch profiles are unchanged.

- `pub(crate) fn profile_box(&self, data: &TypeData) -> Option<TypeData>` *(in `cc_core::profiles_project`)*

  `data` with its legacy section fields rewritten as the bounding box of
  its profile's placed outline (`Rectangular`, or `Solid` `width` x
  `height`), when the type points to a valid profile and its fields
  differ from that box; `None` when nothing needs to change (no
  profile, the legacy section is exactly the profile, or the outline
  fails).

- `pub(crate) fn resync_profile_boxes(&mut self) -> usize` *(in `cc_core::profiles_project`)*

  Rewrite the legacy section fields of every type that points to a
  valid profile as its box (see `Self::profile_box`), so what reads
  them (beam depth, axes) cannot disagree with the profile in a loaded
  file. Returns the number of types changed.

- `pub(crate) fn legacy_is_profile(&self, data: &TypeData) -> bool` *(in `cc_core::profiles_project`)*

  Whether a type's legacy section is exactly its profile (unturned,
  unmirrored), so the legacy fields need no rewrite.

- `pub fn convert_sections_to_profiles(&mut self, _translate: &dyn Fn(&str) -> String) -> usize` *(in `cc_core::profiles_project`)*

  Point every column and beam type without a profile to one made from
  its legacy section; identical (material, shape) pairs share one
  profile, named by `section_name`. Types whose section makes no valid
  outline are left as they are. Returns the number of types changed.
  `translate` is the template's key lookup (the names are made of
  language-neutral symbols, so it is not needed yet).

- `pub(crate) fn legacy_profile(&self, data: &TypeData) -> Option<(ProfileMaterial, ProfileShape)>` *(in `cc_core::profiles_project`)*

  The (material, shape) of a column or beam type's legacy section, when
  it makes a valid outline.

- `pub(crate) fn section_profile(&mut self, material: ProfileMaterial, shape: ProfileShape) -> Option<ProfileId>` *(in `cc_core::profiles_project`)*

  The profile of `material` and `shape`: an existing one equal to it, or
  a new one named by `section_name` (a fresh id and uid). `None` when
  the project has no room for another.

- `pub(crate) fn repair_type_profiles(&mut self) -> usize` *(in `cc_core::profiles_project`)*

  Clear references to profiles that are gone and turns that are not
  numbers; returns the number of fixes.

- `pub fn free_profile_name(&self, base: &str) -> String` *(in `cc_core::profiles_project`)*

  `base`, or `base 2`, `base 3`… the first name no profile has.

- `pub(crate) fn repair_profiles(&mut self) -> usize` *(in `cc_core::profiles_project`)*

  Fix the project's profiles after loading; returns the number of fixes.

- `pub fn empty() -> Self`

  An empty project with no levels, types or views.

- `pub fn sheets_in_order(&self) -> Vec<ViewId>`

  The sheets by number (natural order), then id.

- `pub fn title_blocks(&self) -> Vec<ElementTypeId>`

  The title block types, by id.

- `pub fn from_template() -> Self`

  A new project from the default architectural template (Brazilian Portuguese names).

- `pub fn from_template_localized(translate: &dyn Fn(&str) -> Option<String>) -> Self`

  A new project from the metric template, naming levels, materials,
  types and views with `translate(key)` (keys `tpl-*`), falling back to Portuguese.

- `pub fn from_template_system(system: crate::MeasurementSystem, translate: &dyn Fn(&str) -> Option<String>) -> Self`

  A new project from the template of `system`, named like
  `Self::from_template_localized`. Both templates have the same
  levels, types and views in the same order; the imperial one shows feet
  and inches and sizes its types in round inches.

- `pub fn add_standard_title_block(&mut self, n: &dyn Fn(&str) -> String) -> ElementTypeId`

  Add the standard title block type (named by `n`) and return its id.

- `pub fn give_sheets_title_block(&mut self, id: ElementTypeId) -> usize`

  Point every sheet without a title block at `id`; returns how many.

- `pub fn ensure_uids(&mut self)`

  Assign persistent unique ids to the project, types and elements that
  lack one, and replace duplicates (e.g. from copied elements).

- `pub fn ensure_annotation_types(&mut self)`

  Add default room, grid and dimension types to projects created before
  those categories existed, sized and named (in Portuguese) like the
  template of the project's measurement system.

- `pub fn measurement_system(&self) -> crate::MeasurementSystem`

  The measurement system of the project: imperial when its lengths
  show in an imperial unit (see `crate::Units::system`).

- `pub fn defaults(&self) -> &'static crate::Defaults`

  The sizes and rules things made outside the template get in this
  project's measurement system.

- `pub fn system(&self, id: crate::SystemId) -> Option<&crate::MepSystem>`
- `pub fn system_color(&self, element: &Element) -> [u8; 3]`

  Display color of a pipe, conduit, cable tray or device: its (first)
  system's, or gray; a valve takes its pipe's.

- `pub fn mep_data(&self, element: &Element) -> Option<crate::MepData>`

  The MEP data of a component element's type.

- `pub fn feeder(&self, system: crate::SystemId) -> Option<&Element>`

  The board or rack feeding a system (its `panel`, when that is still a
  component element).

- `pub fn component_point(&self, element: &Element) -> Option<glam::DVec3>`

  Where a component element stands in the world: its insertion point at
  its base (`None` for other elements).

- `pub fn outlet_cable(&self, element: &Element) -> Option<(u32, f64)>`

  The estimated length of the horizontal cables of a data outlet: one
  per jack, each routed to the rack of its data system as
  `crate::mep::cable_length` says (up to the ceiling of the outlet's
  level, across and down to the rack). `None` for elements that are not
  data outlets cabled to a rack.

- `pub fn segment_at(&self, element: &Element, level: LevelId) -> Option<crate::MepSegment>`

  A pipe, conduit or cable tray run as seen from `level`, with its elevations above
  that level: the level's own runs, and the vertical runs of other levels
  that pass through it (risers between floors, reaching at least its floor
  and starting below its top). `None` for other elements and runs.

- `pub fn segments_at(&self, level: LevelId) -> impl Iterator<Item = (&Element, crate::MepSegment)>`

  The pipe, conduit and cable tray runs seen from `level` (see `Project::segment_at`).

- `pub fn system_members(&self, id: crate::SystemId) -> impl Iterator<Item = &Element>`

  Elements in system `id`: its pipes, the conduits carrying the circuit
  and the components connected to it.

- `pub fn level_height(&self, id: LevelId, fallback: f64) -> f64`

  The level above `id`, or `id` itself plus `fallback` meters: the default upper limit of rooms.

- `pub fn column_span(&self, element: &Element, column: &crate::Column) -> (f64, f64)`

  Base elevation and top elevation of a column (top at least 1 cm above).

- `pub fn stair_span(&self, element: &Element, stair: &crate::Stair) -> (f64, f64)`

  Base and top elevation of a stair (the top at least one riser above).

- `pub fn ramp_span(&self, element: &Element, ramp: &crate::Ramp) -> (f64, f64)`

  Base and top elevation of a ramp (the top at least 1 cm above).

- `pub fn wall_base_z(&self, element: &Element, wall: &crate::Wall) -> f64`

  Elevation of a wall's base.

- `pub fn wall_top_z(&self, element: &Element, wall: &crate::Wall) -> f64`

  Elevation of a wall's top, from its top constraint (at least 1 cm above the base).

- `pub fn wall_height(&self, element: &Element, wall: &crate::Wall) -> f64`

  Effective height of a wall.

- `pub fn wall_axis(&self, element: &Element, wall: &crate::Wall) -> (DVec2, DVec2)`

  Start and end of a wall's body centerline: the location line shifted
  across the wall according to `location_line` and the type's layers
  (listed from the exterior face, on the left of the line). An arc
  wall's centerline is concentric with its location line and has the
  same bulge.

- `pub fn wall_length(&self, element: &Element, wall: &crate::Wall) -> f64`

  Length of a wall's body centerline (the location line's length for
  straight walls; arc walls measure along the centerline arc).

- `pub fn beam_axis(&self, element: &Element, beam: &crate::Beam) -> (DVec2, DVec2)`

  The beam's body centerline: its location line moved across by the
  lateral justification.

- `pub fn beam_z(&self, element: &Element, beam: &crate::Beam) -> (f64, f64)`

  Bottom and top elevations of a beam at its start, from its location
  elevation and vertical justification. A sloped beam's section stays
  square to its axis, so its vertical depth is a little more than its height.

- `pub fn next_id(&self) -> u64`

  Allocate a new unique id (shared by all id kinds).
  The id the next new element, view or type gets (every id after it is
  newer).

- `pub(crate) fn copy_with_types(&self, types: BTreeMap<ElementTypeId, ElementType>) -> Project`

  A copy of the project with `types` in place of its own (which are not copied).

- `pub fn alloc_id(&mut self) -> u64`
- `pub fn max_id(&self) -> u64`

  The largest id any object of the project uses (0 if none).

- `pub fn ensure_next_id(&mut self) -> bool`

  Move the id counter past every id in use, so new objects never take
  the id of an existing one (a file may carry a wrong counter), and back
  from the end of the range, where it could not move on. Returns
  whether it moved.

- `pub fn level(&self, id: LevelId) -> Option<&Level>`
- `pub fn level_elevation(&self, id: LevelId) -> f64`
- `pub fn level_name(&self, id: LevelId) -> &str`
- `pub fn is_pinned(&self, id: ElementId) -> bool`
- `pub fn join_of(&self, a: ElementId, b: ElementId) -> Option<(ElementId, ElementId)>`

  The join between two elements as `(cutting, cut)`, if they are joined.

- `pub fn group_of(&self, id: ElementId) -> Option<crate::GroupId>`

  The group an element belongs to.

- `pub fn with_groups(&self, ids: impl IntoIterator<Item = ElementId>) -> std::collections::BTreeSet<ElementId>`

  `ids` plus every member of the groups they belong to.

- `pub fn joined_with(&self, id: ElementId) -> impl Iterator<Item = ElementId> + '_`

  Elements whose geometry is joined with `id`.

- `pub fn element(&self, id: ElementId) -> Option<&Element>`
- `pub fn element_type(&self, id: ElementTypeId) -> Option<&ElementType>`
- `pub fn type_name(&self, id: ElementTypeId) -> &str`
- `pub fn thickness_of(&self, element: &Element) -> f64`

  Wall thickness for an element's type (0 if not a layered type).

- `pub fn types_of(&self, category: Category) -> impl Iterator<Item = &ElementType>`
- `pub fn default_type(&self, category: Category) -> Option<ElementTypeId>`
- `pub fn material_color(&self, name: &str) -> [u8; 3]`
- `pub fn material_transparency(&self, name: &str) -> f64`

  Transparency of material `name` (0 opaque, 1 invisible).

- `pub fn material(&self, name: &str) -> Option<&Material>`
- `pub fn material_users(&self, name: &str) -> Vec<ElementTypeId>`

  Types that use the material `name`.

- `pub fn material_instance_users(&self, name: &str) -> Vec<ElementId>`

  Components whose own value of a material parameter is `name`.

- `pub fn material_paint_users(&self, name: &str) -> Vec<ElementId>`

  Elements with a face painted in material `name`.

- `pub fn hosted_by(&self, wall: ElementId) -> impl Iterator<Item = &Element>`

  Doors and windows hosted by `wall`.

- `pub fn railings_of(&self, host: ElementId) -> impl Iterator<Item = &Element>`

  The railings that follow stair `host`.

- `pub fn rebar_of(&self, host: ElementId) -> impl Iterator<Item = (&Element, &crate::Rebar)>`

  The reinforcement sets of beam or column `host`.

- `pub fn bar_tags_of(&self, set: ElementId) -> impl Iterator<Item = (&Element, &crate::bar_tag::BarTag)>`

  The bar tags of reinforcement set `set`.

- `pub fn valves_of(&self, host: ElementId) -> impl Iterator<Item = (&Element, &crate::Valve)>`

  The valves on pipe `host`.

- `pub fn valve_host(&self, valve: &crate::Valve) -> Option<(&Element, &crate::MepSegment)>`

  A valve's pipe element and its run.

- `pub fn elements_of(&self, category: Category) -> impl Iterator<Item = &Element>`
- `pub fn walls(&self) -> impl Iterator<Item = (&Element, &crate::Wall)>`
- `pub fn views_of(&self, category: ViewCategory) -> impl Iterator<Item = &View>`
- `pub fn view(&self, id: ViewId) -> Option<&View>`
- `pub fn filter(&self, id: crate::FilterId) -> Option<&crate::ViewFilter>`
- `pub fn line_style(&self, id: crate::LineStyleId) -> Option<&crate::LineStyleDef>`
- `pub fn line_style_users(&self, id: crate::LineStyleId) -> Vec<ElementId>`

  The detail lines and regions drawn with a project line style.

- `pub fn template(&self, id: crate::TemplateId) -> Option<&crate::ViewTemplate>`
- `pub fn next_mark(&self, category: Category) -> String`

  A fresh mark for a new element: the next number among its category.

- `pub fn level_above(&self, id: LevelId) -> Option<&Level>`

  The level directly above `id`, if any.

- `pub fn move_top_with_base(&self, e: &mut crate::Element, to: LevelId)`

  Before `e` moves from its level to `to`: a top attached to a level
  moves up or down as many levels as the base (a column from the ground
  floor to the first floor pasted on the first floor reaches the second).
  When there is no such level, or it is not above the new base, the top
  is unconnected at the element's height.

- `pub fn sort_levels(&mut self)`
- `pub fn repair(&mut self) -> usize` *(in `cc_core::repair`)*

  Remove or reset invalid data; returns the number of repairs made.

- `pub fn repairs_needed(&self) -> usize` *(in `cc_core::repair`)*

  How many repairs `Project::repair` would make, without changing the
  project: it repairs a copy. The copy leaves out the vertices and faces
  of fixed mesh pieces, which repair never reads, so that checking every
  edit of a model with imported meshes does not copy them.

- `pub fn shaft_span(&self, element: &Element, shaft: &Shaft) -> (f64, f64)` *(in `cc_core::shaft`)*

  Bottom and top elevation of a shaft (the top at least 1 cm above).

- `pub fn shaft_cuts(&self, element: &Element) -> Vec<Vec<DVec2>>` *(in `cc_core::shaft`)*

  The outlines of the shafts that cut `element` (a floor, ceiling or
  roof): those that pass through its thickness and lie inside its
  outline, clear of its own openings and of one another.

- `pub fn structural_material(&self, data: &TypeData) -> Option<StructuralMaterial>` *(in `cc_core::structural_material`)*

  The structural material of a column or beam type: its profile's,
  else its material's class (metal is steel, wood timber, the rest
  concrete). `None` for other types.

- `pub fn types_of_material(&self, category: Category, material: StructuralMaterial) -> impl Iterator<Item = &ElementType> + '_` *(in `cc_core::structural_material`)*

  The types of `category` made of `material`, in id order.

- `pub fn sun_site(&self) -> SunSite` *(in `cc_core::sun`)*

  The site as the sun needs it: the project's location, else the
  equator at Greenwich in UTC with north up (`known: false`).

- `pub fn sun_state(&self, view: &View) -> Option<SunState>` *(in `cc_core::sun`)*

  Where `view`'s sun stands (`None` when the view has none).

- `pub fn sun_ground_height(&self, view: &View, bottom: Option<f64>) -> f64` *(in `cc_core::sun`)*

  The height of the ground plane that catches `view`'s sun's shadows
  (and that the rendered image stands the model on): the sun's ground
  level, else the lowest level, else the bottom of the model
  (`bottom`), else 0.

- `pub fn sun_preset(&self, name: &str) -> Option<&SunPreset>` *(in `cc_core::sun`)*

  The preset called `name`.

- `pub fn copy_set(&self, ids: &[ElementId]) -> Vec<Element>` *(in `cc_core::transform`)*

  The elements a copy of `ids` takes: the non-hosted ones, plus the
  doors and windows of copied walls, the railings of copied stairs, the
  pads of copied terrains, the valves of copied pipes and any selected
  hosted element. Connections are never copied: they belong to their
  member's end.
  The snapshot stays valid after the originals are deleted (cut).

- `pub fn wall_at(&self, level: crate::LevelId, p: DVec2, prefer: ElementId) -> Option<(ElementId, f64)>` *(in `cc_core::transform`)*

  The wall on `level` whose body contains plan point `p`, with the
  distance of `p` along it; `prefer` wins when it contains `p`.

- `pub fn framings_of(&self, wall: ElementId) -> impl Iterator<Item = ElementId> + '_` *(in `cc_core::wall_framing`)*

  The framings of `wall` (one in a consistent model).

- `pub fn wall_framing_of(&self, wall: ElementId) -> Option<ElementId>` *(in `cc_core::wall_framing`)*

  The framing of `wall`, if it has one.

- `pub(crate) fn repair_wall_framings(&mut self) -> usize` *(in `cc_core::wall_framing`)*

  One framing per wall: the later ones (higher ids) go, their members
  staying as plain elements. Returns how many went.


<a id="cc_core-project-group"></a>

### struct `Group`

```rust
pub struct Group
```

Elements kept together: selecting one selects them all. An element
belongs to one group at most.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `name: String`
- `members: std::collections::BTreeSet<ElementId>`

<a id="cc_core-project-template_default"></a>

### fn `template_default`

```rust
pub fn template_default(key: &str) -> &'static str
```

Portuguese names of the default template, keyed like the translation files.

<a id="cc_core-project-can_join_geometry"></a>

### fn `can_join_geometry`

```rust
pub fn can_join_geometry(category: crate::Category) -> bool
```

Whether elements of `category` have a body whose geometry can be joined.

<a id="cc_core-project-join_priority"></a>

### fn `join_priority`

```rust
pub fn join_priority(element: &crate::Element) -> u8
```

How strongly an element cuts the elements it is joined with: of two
joined elements, the higher cuts the lower. Structural columns and beams
cut hosts (walls, roofs, ceilings, floors); floors, ceilings and roofs cut
walls; walls cut architectural columns.

<a id="cc_core-project-natural_cmp"></a>

### fn `natural_cmp`

```rust
pub fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering
```

Compare texts with their digit runs as numbers (A2 before A10).

<a id="mod-cc_core-properties"></a>

## `cc_core::properties`

<sub>`crates/cc_core/src/properties.rs`</sub>

Property and quantity sets carried over from IFC files: named sets of
named values, each value keeping its IFC measure type. Lengths, areas,
volumes and angles are stored in meters, square and cubic meters and
radians, whatever the units of the file they came from.

**Items:** [`PropertySet`](#cc_core-properties-propertyset) · [`Property`](#cc_core-properties-property) · [`PropertyValue`](#cc_core-properties-propertyvalue) · [`IfcValue`](#cc_core-properties-ifcvalue) · [`Atom`](#cc_core-properties-atom) · [`repair_sets`](#cc_core-properties-repair_sets)

<a id="cc_core-properties-propertyset"></a>

### struct `PropertySet`

```rust
pub struct PropertySet
```

A property set (`IfcPropertySet`) or quantity set (`IfcElementQuantity`).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `name: String`
- `quantities: bool`

  A quantity set rather than a property set.

- `properties: Vec<Property>`

<a id="cc_core-properties-property"></a>

### struct `Property`

```rust
pub struct Property
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `name: String`
- `value: PropertyValue`

<a id="cc_core-properties-propertyvalue"></a>

### enum `PropertyValue`

```rust
pub enum PropertyValue
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Variants**

- `Single(Option<IfcValue>)`

  One value (or none).

- `Enumerated(Vec<IfcValue>)`

  Values chosen from an enumeration.

- `List(Vec<IfcValue>)`
- `Bounded { lower: Option<IfcValue>, upper: Option<IfcValue>, set_point: Option<IfcValue>, }`
- `Table { defining: Vec<IfcValue>, defined: Vec<IfcValue>, }`
- `Complex { usage: String, properties: Vec<Property>, }`

  Nested properties with a usage name.

- `Quantity { kind: String, value: f64, }`

  A quantity: its entity (`IFCQUANTITYLENGTH`…) and value.


**Methods**

- `pub fn is_finite(&self) -> bool`

  Every number is finite (nested properties included).


<a id="cc_core-properties-ifcvalue"></a>

### struct `IfcValue`

```rust
pub struct IfcValue
```

A value with its IFC defined type (`IFCLABEL`, `IFCLENGTHMEASURE`…).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `measure: String`
- `value: Atom`

**Methods**

- `pub fn is_finite(&self) -> bool`

<a id="cc_core-properties-atom"></a>

### enum `Atom`

```rust
pub enum Atom
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Variants**

- `Text(String)`
- `Real(f64)`
- `Integer(i64)`
- `Bool(bool)`
- `Enum(String)`

  Other enumeration literals (such as an unknown logical, `U`).


**Methods**

- `pub fn is_finite(&self) -> bool`

<a id="cc_core-properties-repair_sets"></a>

### fn `repair_sets`

```rust
pub fn repair_sets(sets: &mut [PropertySet]) -> usize
```

Drop properties with non-finite numbers; returns how many went.

<a id="mod-cc_core-ramp"></a>

## `cc_core::ramp`

<sub>`crates/cc_core/src/ramp.rs`</sub>

Ramps: the plan shapes of a ramp element and the accessibility rules of
its type (steepest slope, highest rise of one flight, shortest landing).

**Items:** [`RampShape`](#cc_core-ramp-rampshape) · [`RampLayout`](#cc_core-ramp-ramplayout) · [`RampRules`](#cc_core-ramp-ramprules) · [`RampIssue`](#cc_core-ramp-rampissue) · [`RampCheck`](#cc_core-ramp-rampcheck) · [`MAX_FLIGHTS`](#cc_core-ramp-max_flights) · [`ramp_flights`](#cc_core-ramp-ramp_flights)

<a id="cc_core-ramp-rampshape"></a>

### enum `RampShape`

```rust
pub enum RampShape
```

Plan shape of a ramp.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Straight`

  Flights in line, with landings between them as the rules need.

- `LShaped`

  Two flights at a right angle around a corner landing.

- `UShaped`

  Flights going back and forth (switchback), joined by landings.

- `Spiral`

  One helical flight around a center.


**Methods**

- `pub const ALL: [RampShape; 4]`
- `pub fn key(self) -> &'static str`
- `pub fn turns(self) -> bool`

<a id="cc_core-ramp-ramplayout"></a>

### struct `RampLayout`

```rust
pub struct RampLayout
```

Plan layout of a ramp element.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `shape: RampShape`
- `turn: Turn`
- `slope: f64`

  Slope of the flights, rise over run (0: the steepest the type allows).

- `landing: f64`

  Landing length along the way (0: the type's shortest landing).

- `gap: f64`

  Clear gap between the flights of a U-shaped ramp.

- `inner_radius: f64`

  Radius of the inner edge of a spiral ramp.


**Methods**

- `pub fn repair(&mut self) -> usize`

  Replace invalid values by defaults; returns how many changed.


<a id="cc_core-ramp-ramprules"></a>

### struct `RampRules`

```rust
pub struct RampRules
```

Accessibility rules of a ramp type (defaults after common accessibility
codes: 8.33 % slope, 0.80 m rise per flight, 1.20 m landings).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `max_slope: f64`

  Steepest slope, rise over run.

- `max_flight_rise: f64`

  Highest rise of one flight between landings.

- `min_landing: f64`

  Shortest landing along the way.


**Methods**

- `pub fn repair(&mut self) -> usize`

  Replace invalid limits by defaults; returns how many changed.


<a id="cc_core-ramp-rampissue"></a>

### enum `RampIssue`

```rust
pub enum RampIssue
```

A broken rule of a ramp.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`

**Variants**

- `TooSteep { slope: f64, max: f64 }`
- `FlightTooHigh { rise: f64, max: f64 }`
- `LandingTooShort { landing: f64, min: f64 }`

**Methods**

- `pub fn key(&self) -> &'static str`

<a id="cc_core-ramp-rampcheck"></a>

### struct `RampCheck`

```rust
pub struct RampCheck
```

Slope, flights and the rules a ramp breaks.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`

**Fields**

- `slope: f64`
- `flights: usize`
- `flight_rise: f64`

  Rise of each flight.

- `flight_run: f64`

  Plan length of each flight.

- `issues: Vec<RampIssue>`

**Methods**

- `pub fn ok(&self) -> bool`

<a id="cc_core-ramp-max_flights"></a>

### const `MAX_FLIGHTS`

```rust
pub const MAX_FLIGHTS: usize
```

Most flights of one ramp.

<a id="cc_core-ramp-ramp_flights"></a>

### fn `ramp_flights`

```rust
pub fn ramp_flights(height: f64, rules: &RampRules, layout: &RampLayout) -> RampCheck
```

Slope, flight count, rise and run of each flight of a ramp rising
`height` laid out as `layout`, and what it breaks of `rules`. Straight
and U-shaped ramps get as many flights as the highest flight rise
needs; L-shaped ones two; spiral ones one.

<a id="mod-cc_core-raster"></a>

## `cc_core::raster`

<sub>`crates/cc_core/src/raster.rs`</sub>

Raster images kept in a project (imported pictures, scanned drawings):
the file's own bytes, unchanged, with the format and pixel size. Project
files hold the bytes as base64 text.

**Items:** [`MAX_IMAGE_BYTES`](#cc_core-raster-max_image_bytes) · [`MAX_IMAGE_PIXELS`](#cc_core-raster-max_image_pixels) · [`RasterFormat`](#cc_core-raster-rasterformat) · [`RasterImage`](#cc_core-raster-rasterimage) · [`base64_encode`](#cc_core-raster-base64_encode) · [`base64_decode`](#cc_core-raster-base64_decode)

<a id="cc_core-raster-max_image_bytes"></a>

### const `MAX_IMAGE_BYTES`

```rust
pub const MAX_IMAGE_BYTES: usize
```

Largest image file kept, bytes.

<a id="cc_core-raster-max_image_pixels"></a>

### const `MAX_IMAGE_PIXELS`

```rust
pub const MAX_IMAGE_PIXELS: u64
```

Most pixels an image may have (width × height): decoding one takes up to
8 bytes per pixel, plus 4 for its RGBA copy.

<a id="cc_core-raster-rasterformat"></a>

### enum `RasterFormat`

```rust
pub enum RasterFormat
```

Encoding of an image's bytes.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Serialize`, `Deserialize`

**Variants**

- `Png`
- `Jpeg`
- `Gif`
- `Bmp`
- `Webp`

**Methods**

- `pub fn sniff(bytes: &[u8]) -> Option<RasterFormat>`

  The format of `bytes`, from their signature.

- `pub fn mime(self) -> &'static str`

  Media type (for data URLs).


<a id="cc_core-raster-rasterimage"></a>

### struct `RasterImage`

```rust
pub struct RasterImage
```

An image file's bytes with its format and size in pixels.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `format: RasterFormat`
- `width: u32`
- `height: u32`
- `data: Vec<u8>`

**Methods**

- `pub fn is_valid(&self) -> bool`

  Whether the size is usable and the bytes start like the format says.

- `pub fn dpi(&self) -> Option<(f64, f64)>`

  Resolution the file declares (dots per inch, x and y), if any: a PNG's
  `pHYs` chunk in dots per meter, or a JPEG's JFIF density.


<a id="cc_core-raster-base64_encode"></a>

### fn `base64_encode`

```rust
pub fn base64_encode(bytes: &[u8]) -> String
```

Standard base64 with padding.

<a id="cc_core-raster-base64_decode"></a>

### fn `base64_decode`

```rust
pub fn base64_decode(text: &str) -> Option<Vec<u8>>
```

Decode standard base64 (whitespace ignored); `None` on any other character.

<a id="mod-cc_core-recovery"></a>

## `cc_core::recovery`

<sub>`crates/cc_core/src/recovery.rs`</sub>

Crash recovery: while a project has unsaved changes, a copy of it is
written now and then to a recovery folder, so a crash (or a power cut)
loses at most the last few minutes of work. Each running session writes
its own entry: a plain `.ccad` file (it opens like any project) and a
small `.meta` file naming the project and where it was saved. Both are
written to a temporary file first and renamed, so an entry is never half
written. A clean save or close removes the entry; at start-up, the
entries left by earlier sessions are offered for restoring.

**Items:** [`RecoveryMeta`](#cc_core-recovery-recoverymeta) · [`RecoveryEntry`](#cc_core-recovery-recoveryentry) · [`now_seconds`](#cc_core-recovery-now_seconds) · [`write`](#cc_core-recovery-write) · [`remove`](#cc_core-recovery-remove) · [`list`](#cc_core-recovery-list) · [`load`](#cc_core-recovery-load) · [`AutosaveAction`](#cc_core-recovery-autosaveaction) · [`Autosave`](#cc_core-recovery-autosave)

<a id="cc_core-recovery-recoverymeta"></a>

### struct `RecoveryMeta`

```rust
pub struct RecoveryMeta
```

What a recovery entry says about the project it holds.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Eq`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `name: String`

  The project's name.

- `original: Option<PathBuf>`

  Where the project was saved, if it ever was.

- `saved_at: u64`

  When the copy was written, in seconds since the Unix epoch.


<a id="cc_core-recovery-recoveryentry"></a>

### struct `RecoveryEntry`

```rust
pub struct RecoveryEntry
```

A recovery entry found on disk.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Eq`

**Fields**

- `id: String`

  The entry's id (the session that wrote it).

- `project: PathBuf`

  The copy of the project.

- `meta: RecoveryMeta`

<a id="cc_core-recovery-now_seconds"></a>

### fn `now_seconds`

```rust
pub fn now_seconds() -> u64
```

Seconds since the Unix epoch (0 if the clock is before it).

<a id="cc_core-recovery-write"></a>

### fn `write`

```rust
pub fn write(dir: &Path, id: &str, project: &Project, meta: &RecoveryMeta) -> Result<PathBuf, PersistError>
```

Write the recovery entry `id` in `dir` (created if needed): the project
first, then its meta file, each atomically. Returns the project's path.

<a id="cc_core-recovery-remove"></a>

### fn `remove`

```rust
pub fn remove(dir: &Path, id: &str)
```

Remove the recovery entry `id` from `dir` (nothing if there is none).

<a id="cc_core-recovery-list"></a>

### fn `list`

```rust
pub fn list(dir: &Path) -> Vec<RecoveryEntry>
```

The recovery entries in `dir`, the newest first: those whose project
file exists (a meta file that cannot be read still offers its project).

<a id="cc_core-recovery-load"></a>

### fn `load`

```rust
pub fn load(entry: &RecoveryEntry) -> Result<(Project, usize), PersistError>
```

Load the project of a recovery entry (repaired like any opened file).

<a id="cc_core-recovery-autosaveaction"></a>

### enum `AutosaveAction`

```rust
pub enum AutosaveAction
```

What `Autosave::poll` asks the caller to do.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Variants**

- `Nothing`
- `Write`

  Write the document (its revision is the argument of `Autosave::written`).

- `Remove`

  The document has no unsaved changes: remove the entry.


<a id="cc_core-recovery-autosave"></a>

### struct `Autosave`

```rust
pub struct Autosave
```

When to write a recovery copy: once the document has had unsaved changes
for `interval`, and only if its revision changed since the last copy.

**Traits:** `Debug`, `Clone`

**Fields**

- `interval: Duration`

*Some fields are private.*

**Methods**

- `pub fn new(interval: Duration) -> Self`
- `pub fn poll(&mut self, revision: u64, dirty: bool, now: Instant) -> AutosaveAction`

  Called often (once a frame) with the document's `revision` and
  whether it has unsaved changes.

- `pub fn written(&mut self, revision: u64)`

  A copy of `revision` was written.

- `pub fn retry_later(&mut self, now: Instant)`

  Writing failed: try again an interval after `now`.

- `pub fn has_copy(&self) -> bool`

  Whether a copy is on disk.

- `pub fn removed(&mut self)`

  The entry was removed (the document was saved, closed or discarded).


<a id="mod-cc_core-region"></a>

## `cc_core::region`

<sub>`crates/cc_core/src/region.rs`</sub>

Regions: 2D areas drawn in one view, owned by it like detail lines and
text notes (`Element::owner_view`), in the view's drawing coordinates
(meters). An outline with openings, usually drawn as a sketch
(`crate::sketch::Sketch::profile`), and an optional boundary line
style (none: no boundary is drawn).

- A *filled region* (`ElementKind::FilledRegion`) is filled as its type
  says (`TypeData::FilledRegion { fill }`: a foreground and a background
  pattern, each with its color, as for material cut patterns; a solid
  pattern fills with its color).
- A *masking region* (`ElementKind::MaskingRegion`, `TypeData::MaskingRegion {}`)
  is opaque white: it hides the model's linework of the view under it,
  while the view's annotations (detail lines, dimensions, texts, tags)
  stay on top.
Regions of a view are drawn in id order over the model and under its
annotations; they take no part in 3D views or IFC.

**Items:** [`MAX_POINTS`](#cc_core-region-max_points) · [`MAX_OPENINGS`](#cc_core-region-max_openings) · [`Region`](#cc_core-region-region)

<a id="cc_core-region-max_points"></a>

### const `MAX_POINTS`

```rust
pub const MAX_POINTS: usize
```

Most points of an outline or an opening, and most openings (a crafted file).

<a id="cc_core-region-max_openings"></a>

### const `MAX_OPENINGS`

```rust
pub const MAX_OPENINGS: usize
```

<a id="cc_core-region-region"></a>

### struct `Region`

```rust
pub struct Region
```

A filled or masking region of one view.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `view: ViewId`

  The view that owns (and alone shows) the region.

- `outline: Vec<DVec2>`

  The outline, in the view's drawing coordinates (meters).

- `openings: Vec<Vec<DVec2>>`

  Holes inside the outline.

- `sketch: Option<Sketch>`

  The curves the outline was drawn with (see `crate::sketch`).

- `boundary: Option<LineStyleRef>`

  The style of the boundary lines; `None` draws no boundary.


**Methods**

- `pub fn is_valid(&self) -> bool`

  An outline of three or more finite points within bounds, with some
  area, and openings of the same kind.

- `pub fn area(&self) -> f64`

  Area of the outline less its openings, square meters.

- `pub fn perimeter(&self) -> f64`

  Length of the outline and of the openings' edges, meters.

- `pub fn points_mut(&mut self) -> impl Iterator<Item = &mut DVec2>`

  Every point of the outline and openings, to move them.


<a id="mod-cc_core-render_settings"></a>

## `cc_core::render_settings`

<sub>`crates/cc_core/src/render_settings.rs`</sub>

Per-view settings of photorealistic rendering (path tracing).
A view that takes them (`crate::ViewKind::takes_render`: 3D views)
keeps a `RenderSettings` in `crate::View::render`: the samples per
pixel, the light bounces, the output size, the field of view, the
exposure and tone mapping, the firefly clamp, the clay mode and the
environment (sky, sun and ground). Every field has a serde default, so
settings written by older or newer builds load; values out of range are
brought back by `RenderSettings::repair`.

**Items:** [`MIN_SAMPLES`](#cc_core-render_settings-min_samples) · [`MAX_SAMPLES`](#cc_core-render_settings-max_samples) · [`DEFAULT_SAMPLES`](#cc_core-render_settings-default_samples) · [`MIN_DEPTH`](#cc_core-render_settings-min_depth) · [`MAX_DEPTH`](#cc_core-render_settings-max_depth) · [`DEFAULT_DEPTH`](#cc_core-render_settings-default_depth) · [`MIN_SIZE`](#cc_core-render_settings-min_size) · [`MAX_SIZE`](#cc_core-render_settings-max_size) · [`MAX_PIXELS`](#cc_core-render_settings-max_pixels) · [`DEFAULT_WIDTH`](#cc_core-render_settings-default_width) · [`DEFAULT_HEIGHT`](#cc_core-render_settings-default_height) · [`MIN_FOV_DEG`](#cc_core-render_settings-min_fov_deg) · [`MAX_FOV_DEG`](#cc_core-render_settings-max_fov_deg) · [`DEFAULT_FOV_DEG`](#cc_core-render_settings-default_fov_deg) · [`MIN_EXPOSURE`](#cc_core-render_settings-min_exposure) · [`MAX_EXPOSURE`](#cc_core-render_settings-max_exposure) · [`DEFAULT_EXPOSURE`](#cc_core-render_settings-default_exposure) · [`MIN_CLAMP`](#cc_core-render_settings-min_clamp) · [`MAX_CLAMP`](#cc_core-render_settings-max_clamp) · [`DEFAULT_CLAMP`](#cc_core-render_settings-default_clamp) · [`MIN_STRENGTH`](#cc_core-render_settings-min_strength) · [`MAX_STRENGTH`](#cc_core-render_settings-max_strength) · [`DEFAULT_SKY_STRENGTH`](#cc_core-render_settings-default_sky_strength) · [`DEFAULT_SUN_STRENGTH`](#cc_core-render_settings-default_sun_strength) · [`DEFAULT_GROUND_COLOR`](#cc_core-render_settings-default_ground_color) · [`ToneMap`](#cc_core-render_settings-tonemap) · [`RenderEnvironment`](#cc_core-render_settings-renderenvironment) · [`RenderSettings`](#cc_core-render_settings-rendersettings)

<a id="cc_core-render_settings-min_samples"></a>

### const `MIN_SAMPLES`

```rust
pub const MIN_SAMPLES: u32
```

Fewest samples per pixel.

<a id="cc_core-render_settings-max_samples"></a>

### const `MAX_SAMPLES`

```rust
pub const MAX_SAMPLES: u32
```

Most samples per pixel.

<a id="cc_core-render_settings-default_samples"></a>

### const `DEFAULT_SAMPLES`

```rust
pub const DEFAULT_SAMPLES: u32
```

Samples per pixel of new settings.

<a id="cc_core-render_settings-min_depth"></a>

### const `MIN_DEPTH`

```rust
pub const MIN_DEPTH: u32
```

Fewest light bounces.

<a id="cc_core-render_settings-max_depth"></a>

### const `MAX_DEPTH`

```rust
pub const MAX_DEPTH: u32
```

Most light bounces.

<a id="cc_core-render_settings-default_depth"></a>

### const `DEFAULT_DEPTH`

```rust
pub const DEFAULT_DEPTH: u32
```

Light bounces of new settings.

<a id="cc_core-render_settings-min_size"></a>

### const `MIN_SIZE`

```rust
pub const MIN_SIZE: u32
```

Smallest output width or height, pixels.

<a id="cc_core-render_settings-max_size"></a>

### const `MAX_SIZE`

```rust
pub const MAX_SIZE: u32
```

Largest output width or height, pixels.

<a id="cc_core-render_settings-max_pixels"></a>

### const `MAX_PIXELS`

```rust
pub const MAX_PIXELS: u64
```

Most output pixels (7680 x 4320).

<a id="cc_core-render_settings-default_width"></a>

### const `DEFAULT_WIDTH`

```rust
pub const DEFAULT_WIDTH: u32
```

Output width of new settings, pixels.

<a id="cc_core-render_settings-default_height"></a>

### const `DEFAULT_HEIGHT`

```rust
pub const DEFAULT_HEIGHT: u32
```

Output height of new settings, pixels.

<a id="cc_core-render_settings-min_fov_deg"></a>

### const `MIN_FOV_DEG`

```rust
pub const MIN_FOV_DEG: f64
```

Narrowest vertical field of view, degrees.

<a id="cc_core-render_settings-max_fov_deg"></a>

### const `MAX_FOV_DEG`

```rust
pub const MAX_FOV_DEG: f64
```

Widest vertical field of view, degrees.

<a id="cc_core-render_settings-default_fov_deg"></a>

### const `DEFAULT_FOV_DEG`

```rust
pub const DEFAULT_FOV_DEG: f64
```

Vertical field of view of new settings, degrees (the 3D viewport's).

<a id="cc_core-render_settings-min_exposure"></a>

### const `MIN_EXPOSURE`

```rust
pub const MIN_EXPOSURE: f64
```

Lowest exposure, stops.

<a id="cc_core-render_settings-max_exposure"></a>

### const `MAX_EXPOSURE`

```rust
pub const MAX_EXPOSURE: f64
```

Highest exposure, stops.

<a id="cc_core-render_settings-default_exposure"></a>

### const `DEFAULT_EXPOSURE`

```rust
pub const DEFAULT_EXPOSURE: f64
```

Exposure of new settings, stops.

<a id="cc_core-render_settings-min_clamp"></a>

### const `MIN_CLAMP`

```rust
pub const MIN_CLAMP: f64
```

Lowest sample clamp (0 turns it off).

<a id="cc_core-render_settings-max_clamp"></a>

### const `MAX_CLAMP`

```rust
pub const MAX_CLAMP: f64
```

Highest sample clamp.

<a id="cc_core-render_settings-default_clamp"></a>

### const `DEFAULT_CLAMP`

```rust
pub const DEFAULT_CLAMP: f64
```

Sample clamp of new settings.

<a id="cc_core-render_settings-min_strength"></a>

### const `MIN_STRENGTH`

```rust
pub const MIN_STRENGTH: f64
```

Lowest sky or sun strength.

<a id="cc_core-render_settings-max_strength"></a>

### const `MAX_STRENGTH`

```rust
pub const MAX_STRENGTH: f64
```

Highest sky or sun strength.

<a id="cc_core-render_settings-default_sky_strength"></a>

### const `DEFAULT_SKY_STRENGTH`

```rust
pub const DEFAULT_SKY_STRENGTH: f64
```

Sky strength of new settings.

<a id="cc_core-render_settings-default_sun_strength"></a>

### const `DEFAULT_SUN_STRENGTH`

```rust
pub const DEFAULT_SUN_STRENGTH: f64
```

Sun strength of new settings.

<a id="cc_core-render_settings-default_ground_color"></a>

### const `DEFAULT_GROUND_COLOR`

```rust
pub const DEFAULT_GROUND_COLOR: [u8; 3]
```

Ground color of new settings: a neutral warm gray.

<a id="cc_core-render_settings-tonemap"></a>

### enum `ToneMap`

```rust
pub enum ToneMap
```

How the rendered radiance is mapped to display colors.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Filmic`

  A filmic curve: soft highlights, no hard clipping.

- `Standard`

  Plain clipping to the display range.


**Methods**

- `pub const ALL: [ToneMap; 2]`
- `pub fn name(self) -> &'static str`

  The name commands use: `filmic` or `standard`.


<a id="cc_core-render_settings-renderenvironment"></a>

### struct `RenderEnvironment`

```rust
pub struct RenderEnvironment
```

The light around the model: sky, sun and ground.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `sky_strength: f64`

  Multiplier of the sky's light.

- `sun_strength: f64`

  Multiplier of the sun's light (the view's sun, see `crate::sun`).

- `ground_color: [u8; 3]`

  Color of the ground, sRGB.

- `ground_plane: bool`

  An endless-looking ground under the model that catches shadows.


**Methods**

- `pub fn is_valid(&self) -> bool`

  Every value finite and in range.

- `pub fn repair(&mut self) -> bool`

  Bring values back in range (non-finite ones to their defaults);
  whether anything changed.


<a id="cc_core-render_settings-rendersettings"></a>

### struct `RenderSettings`

```rust
pub struct RenderSettings
```

How a view is rendered photorealistically.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `samples: u32`

  Samples per pixel.

- `max_depth: u32`

  Light bounces.

- `width: u32`

  Output width, pixels.

- `height: u32`

  Output height, pixels.

- `fov_deg: f64`

  Vertical field of view, degrees.

- `exposure: f64`

  Exposure, stops (EV).

- `tone_map: ToneMap`

  How radiance maps to display colors.

- `clamp: f64`

  Largest radiance of a sample (removes fireflies); 0 turns it off.

- `clay: bool`

  Render every opaque surface in one white matte (glass stays clear
  glass), lit by the same sky, sun and ground as a normal render.

- `environment: RenderEnvironment`

  Sky, sun and ground.


**Methods**

- `pub fn is_valid(&self) -> bool`

  Every value finite and in range, and the output not over `MAX_PIXELS`.

- `pub fn pixels(&self) -> u64`

  Output pixels.

- `pub fn aspect(&self) -> f64`

  Output width over height.

- `pub fn repair(&mut self) -> bool`

  Bring values back in range (non-finite ones to their defaults) and
  scale an output over `MAX_PIXELS` down, keeping its aspect;
  whether anything changed.


<a id="mod-cc_core-repair"></a>

## `cc_core::repair`

<sub>`crates/cc_core/src/repair.rs`</sub>

Repair of projects read from files, which may be damaged or crafted:
invalid numbers and dangling references are removed or reset so the rest
of the program can rely on a consistent model.

<a id="mod-cc_core-roofing"></a>

## `cc_core::roofing`

<sub>`crates/cc_core/src/roofing.rs`</sub>

Roof coverings and framing. A roof type may name its tiles (a kind from
a built-in catalog, with reference values: the least slope they take,
their weight, how many make a square meter, the batten gauge and the
widest span between purlins) and its framing: purlins with rafters and
battens (timber), or purlins only (steel, for sheets). The framing is
derived from the roof's sloped faces (see `cc_geom::roof_framing`).
The catalog's values are typical figures from public product data, for
early design; manufacturers' figures prevail.

**Items:** [`TileKind`](#cc_core-roofing-tilekind) · [`TileSpec`](#cc_core-roofing-tilespec) · [`FramingSystem`](#cc_core-roofing-framingsystem) · [`MemberSize`](#cc_core-roofing-membersize) · [`RoofFraming`](#cc_core-roofing-roofframing)

<a id="cc_core-roofing-tilekind"></a>

### enum `TileKind`

```rust
pub enum TileKind
```

Kinds of roof covering.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Serialize`, `Deserialize`

**Variants**

- `CeramicColonial`

  Ceramic tiles laid as caps and channels (colonial).

- `CeramicFrench`

  Ceramic interlocking tiles (French).

- `CeramicRoman`

  Ceramic interlocking tiles (Roman).

- `CeramicPortuguese`

  Ceramic interlocking tiles (Portuguese).

- `Concrete`

  Concrete tiles.

- `FiberCement`

  Corrugated fiber cement sheets (6 mm).

- `MetalTrapezoidal`

  Trapezoidal metal sheets.

- `Sandwich`

  Insulated sandwich panels.


**Methods**

- `pub const ALL: [TileKind; 8]`
- `pub fn key(self) -> &'static str`

  Translation key of the name.

- `pub fn code(self) -> &'static str`

  The name written in files (IFC property sets).

- `pub fn from_code(code: &str) -> Option<Self>`

  The kind written as `code`.

- `pub fn spec(self) -> TileSpec`

  The catalog's figures for this kind.


<a id="cc_core-roofing-tilespec"></a>

### struct `TileSpec`

```rust
pub struct TileSpec
```

What a kind of covering takes and weighs.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `min_slope: f64`

  Least slope, rise over run.

- `weight: f64`

  Weight of the covering, kg per square meter of roof (wet tiles).

- `per_m2: Option<f64>`

  Tiles per square meter of roof; `None` for sheets and panels.

- `batten_gauge: Option<f64>`

  Distance between battens along the slope (the tile's gauge); `None`
  when the covering lies on purlins.

- `purlin_span: f64`

  Widest distance between purlins along the slope.

- `framing: FramingSystem`

  The framing the covering usually lies on.


**Methods**

- `pub fn accepts(&self, slope: f64) -> bool`

  Whether a roof at `slope` (rise over run) is steep enough.

- `pub fn tiles(&self, area: f64) -> Option<f64>`

  Tiles for `area` square meters of roof, rounded up; `None` for sheets.

- `pub fn weight_on(&self, area: f64) -> f64`

  Weight of the covering on `area` square meters of roof, kg.


<a id="cc_core-roofing-framingsystem"></a>

### enum `FramingSystem`

```rust
pub enum FramingSystem
```

How a roof is framed.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `None`

  No framing drawn (a concrete slab, or framing left out).

- `Timber`

  Timber purlins carrying rafters, the rafters carrying battens.

- `TimberTrusses`

  Timber trusses carrying the purlins, rafters and battens of `Timber`.

- `Steel`

  Steel purlins carrying sheets or panels directly.


**Methods**

- `pub const ALL: [FramingSystem; 4]`
- `pub fn code(self) -> &'static str`

  The name written in files (IFC property sets).

- `pub fn from_code(code: &str) -> Option<Self>`

  The system written as `code`.

- `pub fn key(self) -> &'static str`

<a id="cc_core-roofing-membersize"></a>

### struct `MemberSize`

```rust
pub struct MemberSize
```

A member's section: width and height, meters.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `width: f64`
- `height: f64`

**Methods**

- `pub const fn new(width: f64, height: f64) -> Self`

<a id="cc_core-roofing-roofframing"></a>

### struct `RoofFraming`

```rust
pub struct RoofFraming
```

The framing of a roof type: members under its sloped faces.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `system: FramingSystem`
- `material: String`

  Material of the members.

- `purlin: MemberSize`

  Purlins, along the eaves.

- `purlin_spacing: f64`

  Widest distance between purlins along the slope (the covering's
  span is used when smaller).

- `rafter: MemberSize`

  Rafters (timber), down the slope.

- `rafter_spacing: f64`
- `batten: MemberSize`

  Battens (timber), along the eaves at the tile's gauge.

- `truss: MemberSize`

  Trusses (`TimberTrusses`): the section of their members and the
  widest distance between them along the ridge.

- `truss_spacing: f64`

**Methods**

- `pub fn timber(material: String) -> Self`

  Timber framing of common sizes: 6 x 16 cm purlins at most 1.5 m
  apart, 5 x 6 cm rafters every 50 cm, 1.5 x 5 cm battens; `None` until
  a system is chosen.

- `pub fn repair(&mut self) -> usize`

  Sizes and spacings that are positive and finite; the others are reset
  to the defaults. Returns how many were reset.

- `pub fn purlin_step(&self, covering: Option<TileKind>) -> f64`

  Distance between purlins along the slope for `covering`.

- `pub fn batten_step(&self, covering: Option<TileKind>) -> f64`

  Distance between battens along the slope for `covering`.


<a id="mod-cc_core-scale"></a>

## `cc_core::scale`

<sub>`crates/cc_core/src/scale.rs`</sub>

Drawing scales as people read them: `1:50` in metric projects, the
customary `1/4" = 1'-0"` in imperial ones. A view keeps its scale as the
ratio `N` of 1:`N` (`View::scale`); this only names and reads it.

**Items:** [`IMPERIAL`](#cc_core-scale-imperial) · [`label`](#cc_core-scale-label) · [`choices`](#cc_core-scale-choices) · [`parse`](#cc_core-scale-parse)

<a id="cc_core-scale-imperial"></a>

### const `IMPERIAL`

```rust
pub const IMPERIAL: [(u32, &str); 19]
```

The customary imperial scales: ratio and label (architectural, then engineering).

<a id="cc_core-scale-label"></a>

### fn `label`

```rust
pub fn label(system: MeasurementSystem, n: u32) -> String
```

The scale 1:`n` as a project of `system` writes it: the customary label
of an imperial scale in the table, else `1:n`.

<a id="cc_core-scale-choices"></a>

### fn `choices`

```rust
pub fn choices(system: MeasurementSystem) -> &'static [u32]
```

The scales a scale picker offers in a project of `system`, ascending.

<a id="cc_core-scale-parse"></a>

### fn `parse`

```rust
pub fn parse(text: &str) -> Option<u32>
```

Reads a scale typed as `1:48`, `48` or a label of the imperial table
(`1/4" = 1'-0"`, spaces optional, `1'` for `1'-0"`). Zero is refused.

<a id="mod-cc_core-schedule"></a>

## `cc_core::schedule`

<sub>`crates/cc_core/src/schedule.rs`</sub>

Schedule definitions: which fields a schedule shows, how rows are
filtered, sorted and grouped, how each column is formatted and how the
table looks on paper. The values are computed by `cc_views::schedule`.

**Items:** [`ScheduleField`](#cc_core-schedule-schedulefield) · [`ScheduleDef`](#cc_core-schedule-scheduledef) · [`MAX_BENDING_PARTITIONS`](#cc_core-schedule-max_bending_partitions) · [`MAX_WASTE_PERCENT`](#cc_core-schedule-max_waste_percent) · [`BendingOptions`](#cc_core-schedule-bendingoptions) · [`MAX_FILTERS`](#cc_core-schedule-max_filters) · [`MAX_SORT_LEVELS`](#cc_core-schedule-max_sort_levels) · [`MAX_SCHEDULE_TEXT`](#cc_core-schedule-max_schedule_text) · [`MIN_SCHEDULE_TEXT_SIZE`](#cc_core-schedule-min_schedule_text_size) · [`MAX_SCHEDULE_TEXT_SIZE`](#cc_core-schedule-max_schedule_text_size) · [`ScheduleFilter`](#cc_core-schedule-schedulefilter) · [`SortLevel`](#cc_core-schedule-sortlevel) · [`ColumnAlign`](#cc_core-schedule-columnalign) · [`ColumnFormat`](#cc_core-schedule-columnformat) · [`ScheduleAppearance`](#cc_core-schedule-scheduleappearance)

<a id="cc_core-schedule-schedulefield"></a>

### enum `ScheduleField`

```rust
pub enum ScheduleField
```

A column a schedule can show.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Serialize`, `Deserialize`

**Variants**

- `Mark`
- `Type`
- `Level`
- `Comments`
- `TypeMark`
- `Description`
- `Manufacturer`
- `Model`
- `FireRating`
- `Cost`
- `Length`
- `Height`
- `Width`
- `Thickness`
- `Area`
- `Perimeter`
- `Volume`
- `Sill`
- `Host`
- `Function`
- `Structural`
- `Number`
- `Name`
- `Risers`
- `RiserHeight`
- `Run`
- `Depth`
- `ComponentName`
- `ComponentKind`
- `Section`

  Section size of a column or beam.

- `Profile`

  Name of the profile of a column or beam type, whatever its material (tags show it only for steel and
  cold-formed profiles).

- `LinearMass`

  Mass per meter of a column or beam, kg/m.

- `CutLength`

  Length of a beam's body after its joins.

- `Material`

  Main material of the type.

- `Diameter`

  Bar diameter of reinforcement.

- `BarCount`

  Number of bars in a reinforcement set.

- `BarLength`

  Length of one bar.

- `Weight`

  Steel weight, kilograms.

- `Partition`

  The part of the structure a reinforcement set belongs to.

- `CutOff`

  Elevation of a pile's head (its cut-off).

- `Toe`

  Elevation of a pile's toe.

- `PileKind`

  How a pile is made.

- `PileCount`

  Number of piles under a pile cap.

- `System`

  Plumbing system or electrical circuits.

- `Slope`

  Fall of a pipe per unit of length.

- `Power`

  Apparent power of an electrical device, VA.

- `DrainUnits`

  Drainage fixture units of a plumbing fixture.

- `PhaseCreated`

  The phase the element is built in.

- `PhaseDemolished`

  The phase the element is demolished in.

- `FloorArea`

  Gross floor area of a mass: the sum of its mass floors.

- `TileCount`

  Number of tiles on a roof.

- `FramingLength`

  Total length of a roof's framing members.

- `FramingVolume`

  Volume of a roof's framing members.

- `FloorCount`

  How many mass floors a mass has.

- `PanelCount`

  Number of panels of a curtain wall.

- `GlazedArea`

  Area of a curtain wall's glass panels.

- `MullionLength`

  Total length of a curtain wall's mullions.

- `Elevation`

  Elevation of a building pad above its level.

- `CutVolume`

  Volume of earth cut by a pad (a terrain: by its pads).

- `FillVolume`

  Volume of earth filled by a pad (a terrain: by its pads).

- `CableLength`

  Estimated length of the horizontal cables from a data outlet to its rack.

- `ConnectionKind`

  Kind of a connection (base plate, end plate…).

- `PlateMass`

  Steel mass of a connection's plates, kilograms.

- `BoltCount`

  Number of bolts of a connection.

- `BoltDiameter`

  Bolt diameter of a connection.

- `Count`

  How many elements the row stands for (1, or the group's when rows are not itemized).


**Methods**

- `pub fn key(self) -> &'static str`

  Translation key of the column header.

- `pub fn key_in(self, system: crate::MeasurementSystem) -> &'static str`

  Translation key of the column header in a project of `system`. The
  metric headers of lengths, areas and volumes name their unit
  (`Length (m)`); imperial values carry their own marks and symbols
  (`12'-6"`, `100.00 ft²`), so those headers name none.

- `pub fn available(category: Category) -> Vec<ScheduleField>`

  Fields that make sense for `category`, in a sensible order.

- `pub fn applies_to(self, category: Category) -> bool`

  Whether elements of `category` can have a value for this field: the
  fields [`available`](Self::available) to its schedules, plus the type,
  level, main material, volume and count, which most elements have. A filter
  rule on a field that applies to none of its categories never matches.

- `pub fn defaults(category: Category) -> Vec<ScheduleField>`

  Fields shown by a new schedule of `category`.


<a id="cc_core-schedule-scheduledef"></a>

### struct `ScheduleDef`

```rust
pub struct ScheduleDef
```

What a schedule shows. Empty `fields` means the category defaults (files
written before schedules were configurable).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `fields: Vec<ScheduleField>`
- `sort_by: Option<ScheduleField>`

  Sort rows by this field (the default order is by mark or number).
  The first level of sorting of files written before `Self::sorting`.

- `descending: bool`
- `filter: Option<(ScheduleField, String)>`

  Keep only rows whose value of the field contains the text
  (case-insensitive). The filter of files written before `Self::filters`.

- `hide_totals: bool`

  Hide the totals row.

- `filters: Vec<ScheduleFilter>`

  Keep only the rows that pass every one of these.

- `sorting: Vec<SortLevel>`

  Sort the rows by these, the first outermost; a level may head and
  foot the rows it groups.

- `collapse: bool`

  One row for the elements alike in every sorting level, instead of
  one per element: values that differ among them read as varying, and
  totaled columns add up.

- `columns: Vec<ColumnFormat>`

  How columns are written, for the fields that differ from the usual.

- `appearance: ScheduleAppearance`

  How the table looks on paper.

- `bending: Option<BendingOptions>`

  A reinforcement schedule as a bending schedule: one row per bar mark
  instead of one per set (the fields, sort and filter do not apply).


**Methods**

- `pub fn fields_for(&self, category: Category) -> Vec<ScheduleField>`

  The fields to show for `category`.

- `pub fn all_filters(&self) -> Vec<ScheduleFilter>`

  Every filter: the older single one (a text the value contains), then `Self::filters`.

- `pub fn all_sorting(&self) -> Vec<SortLevel>`

  Every level of sorting: `Self::sorting`, or the older single field.

- `pub fn format_of(&self, field: ScheduleField) -> ColumnFormat`

  How the column of `field` is written.

- `pub fn normalize(&mut self)`

  The older single filter and sort field moved into `Self::filters`
  and `Self::sorting`, which the dialogs edit.

- `pub fn check(&self) -> Option<String>`

  Why the definition cannot be kept (`None`: it can), as a message.

- `pub fn repair(&mut self) -> usize`

  Bring the definition within its limits; returns how many values changed.


<a id="cc_core-schedule-max_bending_partitions"></a>

### const `MAX_BENDING_PARTITIONS`

```rust
pub const MAX_BENDING_PARTITIONS: usize
```

Most partitions a bending schedule is limited to.

<a id="cc_core-schedule-max_waste_percent"></a>

### const `MAX_WASTE_PERCENT`

```rust
pub const MAX_WASTE_PERCENT: f64
```

Largest waste a bending schedule adds, percent.

<a id="cc_core-schedule-bendingoptions"></a>

### struct `BendingOptions`

```rust
pub struct BendingOptions
```

What a bending schedule shows: one row per bar mark (mark, diameter,
grade, shape sketch with its dimensions, length of one bar, count, total
length, weight; rows of bars longer than the stock length noted), then a
summary per diameter and grade (total length and weight, with the waste
added when given).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `partitions: Vec<String>`

  Only the sets of these partitions (none: every set; an empty text
  stands for the sets without one).

- `group_by_partition: bool`

  Rows under a heading per partition (each partition's marks counted
  on their own).

- `waste_percent: Option<f64>`

  Waste added in the summary, percent (none: no waste columns).


**Methods**

- `pub fn check(&self) -> Option<String>`

  Why the options cannot be kept (`None`: they can), as a message.

- `pub fn repair(&mut self) -> usize`

  Bring the options into range; returns how many changed.

- `pub fn shows(&self, partition: &str) -> bool`

  Whether a set of `partition` is shown.


<a id="cc_core-schedule-max_filters"></a>

### const `MAX_FILTERS`

```rust
pub const MAX_FILTERS: usize
```

Most filters of one schedule.

<a id="cc_core-schedule-max_sort_levels"></a>

### const `MAX_SORT_LEVELS`

```rust
pub const MAX_SORT_LEVELS: usize
```

Most levels of sorting of one schedule.

<a id="cc_core-schedule-max_schedule_text"></a>

### const `MAX_SCHEDULE_TEXT`

```rust
pub const MAX_SCHEDULE_TEXT: usize
```

Longest filter value or column heading, in characters.

<a id="cc_core-schedule-min_schedule_text_size"></a>

### const `MIN_SCHEDULE_TEXT_SIZE`

```rust
pub const MIN_SCHEDULE_TEXT_SIZE: f64
```

Smallest and largest text of a schedule on paper, millimeters.

<a id="cc_core-schedule-max_schedule_text_size"></a>

### const `MAX_SCHEDULE_TEXT_SIZE`

```rust
pub const MAX_SCHEDULE_TEXT_SIZE: f64
```

<a id="cc_core-schedule-schedulefilter"></a>

### struct `ScheduleFilter`

```rust
pub struct ScheduleFilter
```

One filter: rows whose `field` compares as `op` says with `value` stay.
A number is compared as a number when the row's value is one (lengths in
meters, areas in square meters, and so on), else as text without case.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `field: ScheduleField`
- `op: FilterOp`

  The comparison, as view filters have them (`FilterOp::test`).

- `value: String`

<a id="cc_core-schedule-sortlevel"></a>

### struct `SortLevel`

```rust
pub struct SortLevel
```

One level of sorting: rows in the order of `field`; a level may write a
heading before each group of rows alike in it, a footer after it (the
group's count and totals) and leave a blank line between groups.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Serialize`, `Deserialize`

**Fields**

- `field: ScheduleField`
- `descending: bool`
- `header: bool`
- `footer: bool`
- `blank_line: bool`

**Methods**

- `pub fn new(field: ScheduleField) -> Self`
- `pub fn groups(&self) -> bool`

  Whether the level groups its rows (a heading, a footer or a blank line).


<a id="cc_core-schedule-columnalign"></a>

### enum `ColumnAlign`

```rust
pub enum ColumnAlign
```

Where a column's texts stand in their cells.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Auto`

  Numbers to the right, texts to the left.

- `Left`
- `Center`
- `Right`

**Methods**

- `pub const ALL: [ColumnAlign; 4]`
- `pub fn key(self) -> &'static str`

  Translation key of the alignment's name.


<a id="cc_core-schedule-columnformat"></a>

### struct `ColumnFormat`

```rust
pub struct ColumnFormat
```

How the column of `field` is written: its own heading, its alignment,
whether it is left out of the table (a hidden field still sorts and
filters) and whether it is totaled (`None`: as the field usually is).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `field: ScheduleField`
- `heading: Option<String>`
- `align: ColumnAlign`
- `hidden: bool`
- `total: Option<bool>`

**Methods**

- `pub fn new(field: ScheduleField) -> Self`
- `pub fn is_plain(&self) -> bool`

  Whether nothing differs from the usual (the entry may be dropped).


<a id="cc_core-schedule-scheduleappearance"></a>

### struct `ScheduleAppearance`

```rust
pub struct ScheduleAppearance
```

How a schedule looks on paper (sheets and exports): its rules, its title
and headers, and the sizes of its texts in millimeters.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `grid_lines: bool`

  Rules between rows and columns.

- `outline: bool`

  The heavier rule around the table.

- `show_title: bool`

  The row with the schedule's name.

- `show_headers: bool`

  The row with the columns' headings.

- `title_size: f64`
- `header_size: f64`
- `text_size: f64`

**Methods**

- `pub fn is_default(&self) -> bool`
- `pub fn repair(&mut self) -> usize`

  Bring the text sizes within their limits; returns how many changed.


<a id="mod-cc_core-section_props"></a>

## `cc_core::section_props`

<sub>`crates/cc_core/src/section_props.rs`</sub>

Geometric properties of a section outline (area, centroid, second
moments, moduli, radii of gyration, torsion constant, mass per length),
computed exactly for the polygon with Green's theorem. SI units.

**Items:** [`SectionProperties`](#cc_core-section_props-sectionproperties)

<a id="cc_core-section_props-sectionproperties"></a>

### struct `SectionProperties`

```rust
pub struct SectionProperties
```

Properties of a section. Units: m, m², m⁴, m³ and kg/m.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`, `Serialize`

**Fields**

- `area: f64`

  Area, m².

- `centroid: [f64; 2]`

  Centroid in the outline's own coordinates, m.

- `ix: f64`

  Second moment about the centroidal x axis, m⁴.

- `iy: f64`

  Second moment about the centroidal y axis, m⁴.

- `ixy: f64`

  Product of inertia about the centroidal axes, m⁴.

- `wx: f64`

  Elastic modulus about x (`ix` over the farthest fiber in y), m³.

- `wy: f64`

  Elastic modulus about y, m³.

- `rx: f64`

  Radius of gyration about x, m.

- `ry: f64`

  Radius of gyration about y, m.

- `j: f64`

  Torsion constant, m⁴. Approximate: Bredt's 4A²t/s on the mid-wall for
  closed shapes, `perimeter * t³ / 6` (`t = 2 * area / perimeter`) for open ones.

- `perimeter: f64`

  Outer perimeter plus the perimeters of the holes, m.

- `linear_mass: f64`

  Mass per length, kg/m.


<a id="mod-cc_core-shaft"></a>

## `cc_core::shaft`

<sub>`crates/cc_core/src/shaft.rs`</sub>

Shaft openings: a vertical opening with an outline in plan, from its
level (plus `base_offset`) up to a level or a height, that cuts every
floor, ceiling and roof it passes through. The cuts are written into
those elements (`cuts`) by every edit (see `crate::EditCtx`), so plans,
schedules, 3D views and IFC export see the holes like any opening.
A shaft cuts an element only where its outline lies inside the element's
outline, clear of its own openings.

**Items:** [`MAX_OUTLINE`](#cc_core-shaft-max_outline) · [`Shaft`](#cc_core-shaft-shaft) · [`fits`](#cc_core-shaft-fits)

<a id="cc_core-shaft-max_outline"></a>

### const `MAX_OUTLINE`

```rust
pub const MAX_OUTLINE: usize
```

Most points of a shaft's outline.

<a id="cc_core-shaft-shaft"></a>

### struct `Shaft`

```rust
pub struct Shaft
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `outline: Vec<DVec2>`

  Closed outline in plan (either winding).

- `base_offset: f64`

  Bottom of the shaft above its level.

- `height: f64`

  Height when `top` is unconnected.

- `top: WallTop`
- `sketch: Option<crate::sketch::Sketch>`

  The curves the outline was drawn with (see `crate::sketch`).


**Methods**

- `pub fn is_valid(&self) -> bool`

<a id="cc_core-shaft-fits"></a>

### fn `fits`

```rust
pub fn fits(outline: &[DVec2], boundary: &[DVec2], holes: &[Vec<DVec2>]) -> bool
```

Whether `outline` lies inside `boundary` and clear of each of `holes`
(no shared edges or points either): a hole it can cut.

<a id="mod-cc_core-site"></a>

## `cc_core::site`

<sub>`crates/cc_core/src/site.rs`</sub>

Where the project stands on Earth: the geographic position of the
project origin, its height above sea level, the direction of true north
and the parts of its postal address, plus the conversion between
latitude/longitude and the project's own coordinates (meters) used to
bring map data in.

**Items:** [`ELEVATION_RANGE`](#cc_core-site-elevation_range) · [`MAX_TEXT`](#cc_core-site-max_text) · [`SiteLocation`](#cc_core-site-sitelocation) · [`SiteAddress`](#cc_core-site-siteaddress) · [`normalize_angle`](#cc_core-site-normalize_angle) · [`to_dms`](#cc_core-site-to_dms) · [`from_dms`](#cc_core-site-from_dms)

<a id="cc_core-site-elevation_range"></a>

### const `ELEVATION_RANGE`

```rust
pub const ELEVATION_RANGE: std::ops::RangeInclusive<f64>
```

Lowest and highest site elevation accepted, meters (the Dead Sea shore to above Everest).

<a id="cc_core-site-max_text"></a>

### const `MAX_TEXT`

```rust
pub const MAX_TEXT: usize
```

Longest text kept in one address part, bytes.

<a id="cc_core-site-sitelocation"></a>

### struct `SiteLocation`

```rust
pub struct SiteLocation
```

The geographic location of the project origin (`[0, 0]` at level
elevation 0) with its postal address.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `latitude: f64`

  Latitude in degrees (WGS 84), north positive.

- `longitude: f64`

  Longitude in degrees (WGS 84), east positive.

- `elevation: f64`

  Height of the project's elevation 0 above sea level, meters.

- `true_north: f64`

  Angle from the project's north (+Y) to true north, degrees counter-clockwise.

- `place: String`

  The place as the map search named it.

- `address: SiteAddress`

  Parts of the postal address.

- `utc_offset: Option<f64>`

  Local time's offset from UTC, hours east (within
  `crate::sun::UTC_OFFSET_RANGE`); `None`: the offset of the
  longitude's time zone (see `SiteLocation::offset_hours`).


**Methods**

- `pub fn at(latitude: f64, longitude: f64) -> SiteLocation`

  A location at `latitude`, `longitude` with nothing else set.

- `pub fn offset_hours(&self) -> f64`

  Local time's offset from UTC, hours east: the one set, else the
  longitude's nominal zone (`longitude / 15`, rounded).

- `pub fn is_valid(&self) -> bool`

  Finite coordinates on Earth, a finite elevation in range, a finite
  north angle and an offset from UTC in range, if set.

- `pub fn repair(&mut self) -> (bool, usize)`

  Fix what can be fixed (north angle into `(-180, 180]`, texts cut to
  `MAX_TEXT`, an offset from UTC out of range dropped); `false` when
  the location is beyond repair.

- `pub fn east_north(&self, latitude: f64, longitude: f64) -> DVec2`

  Meters east and north of the site for a point at `latitude`,
  `longitude`, on the plane tangent to the ellipsoid at the site
  (radii of curvature at its latitude; within a few centimeters over a
  kilometer).

- `pub fn lat_lon(&self, east_north: DVec2) -> (f64, f64)`

  Latitude and longitude of a point `east_north` meters from the site (the inverse of `Self::east_north`).

- `pub fn north(&self) -> DVec2`

  The project's direction of true north (unit vector in plan).

- `pub fn to_project(&self, latitude: f64, longitude: f64) -> DVec2`

  Project coordinates of a point at `latitude`, `longitude`.

- `pub fn from_project(&self, p: DVec2) -> (f64, f64)`

  Latitude and longitude of a point in project coordinates.


<a id="cc_core-site-siteaddress"></a>

### struct `SiteAddress`

```rust
pub struct SiteAddress
```

Parts of a postal address (any may be empty).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `street: String`
- `number: String`
- `district: String`
- `city: String`
- `region: String`
- `postcode: String`
- `country: String`

**Methods**

- `pub fn parts(&self) -> [(&'static str, &str); 7]`

  The parts with their keys, in display order.

- `pub fn part_mut(&mut self, key: &str) -> Option<&mut String>`

  The part called `key`, to change it.

- `pub fn one_line(&self) -> String`

  One line: "street, number - district, city - region, postcode, country", empty parts left out.

- `pub fn is_empty(&self) -> bool`

<a id="cc_core-site-normalize_angle"></a>

### fn `normalize_angle`

```rust
pub fn normalize_angle(a: f64) -> f64
```

`a` degrees in `(-180, 180]`.

<a id="cc_core-site-to_dms"></a>

### fn `to_dms`

```rust
pub fn to_dms(deg: f64) -> [i64; 4]
```

Degrees as IFC's compound plane angle: degrees, minutes, seconds and
millionths of a second, all with the sign of the angle.

<a id="cc_core-site-from_dms"></a>

### fn `from_dms`

```rust
pub fn from_dms(parts: &[i64]) -> Option<f64>
```

Degrees from IFC's compound plane angle (three or four parts; the sign of any part makes it negative).

<a id="mod-cc_core-sketch"></a>

## `cc_core::sketch`

<sub>`crates/cc_core/src/sketch.rs`</sub>

Sketches: the curves an outline is drawn with (lines, arcs, circles and
ellipses; rectangles and polygons are drawn as lines). A floor, ceiling,
roof, foundation slab or mass keeps the sketch it was drawn from, so its
boundary can be edited again curve by curve; its outline (the polygon the
geometry is built from) is the sketch's loops, arcs and ellipses split
into short chords.
`Sketch::profile` chains the curves into closed loops: the one around
all the others is the outline, the others are openings in it. Ends meet
within `JOIN_TOL`; open ends, curves branching at a point, loops that
cross themselves or each other, and loops apart from or nested in each
other are errors, with the point where they happen.

**Items:** [`JOIN_TOL`](#cc_core-sketch-join_tol) · [`MAX_CURVES`](#cc_core-sketch-max_curves) · [`MAX_POINTS`](#cc_core-sketch-max_points) · [`SketchCurve`](#cc_core-sketch-sketchcurve) · [`SketchError`](#cc_core-sketch-sketcherror) · [`Profile`](#cc_core-sketch-profile) · [`Sketch`](#cc_core-sketch-sketch)

<a id="cc_core-sketch-join_tol"></a>

### const `JOIN_TOL`

```rust
pub const JOIN_TOL: f64
```

Ends closer than this (meters) join.

<a id="cc_core-sketch-max_curves"></a>

### const `MAX_CURVES`

```rust
pub const MAX_CURVES: usize
```

Most curves a sketch holds (a guard against crafted files).

<a id="cc_core-sketch-max_points"></a>

### const `MAX_POINTS`

```rust
pub const MAX_POINTS: usize
```

Most points the loops of a sketch make.

<a id="cc_core-sketch-sketchcurve"></a>

### enum `SketchCurve`

```rust
pub enum SketchCurve
```

A curve of a sketch, in plan coordinates (meters).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Variants**

- `Line { a: DVec2, b: DVec2, }`
- `Arc { a: DVec2, b: DVec2, bulge: f64, }`

  A circular arc from `a` to `b`, by its bulge (see `crate::arc`).

- `Circle { center: DVec2, radius: f64, }`
- `Ellipse { center: DVec2, radii: DVec2, rotation_deg: f64, #[serde(default)] start_deg: f64, #[serde(default = "full_turn")] sweep_deg: f64, }`

  An ellipse with semi-axes `radii` (along its own x and y), its x axis
  turned `rotation_deg` from the plan's; a part of it from the angle
  `start_deg` (of its parameter) turning by `sweep_deg`, or all of it
  when the sweep is a full turn.

- `Spline { points: Vec<DVec2>, }`

  A chain of cubic Bézier segments through its control points
  (`3n + 1` of them, see `crate::spline`); closed when it ends where
  it starts.


**Methods**

- `pub fn ellipse(center: DVec2, radii: DVec2, rotation_deg: f64) -> Self`

  A whole ellipse.

- `pub fn is_valid(&self) -> bool`

  Finite numbers and a size: lines and arcs with distinct ends, positive radii.

- `pub fn is_closed(&self) -> bool`

  Whether the curve closes on itself (a circle, a whole ellipse).

- `pub fn ends(&self) -> Option<(DVec2, DVec2)>`

  The start and end of an open curve.

- `pub fn points(&self) -> Vec<DVec2>`

  The curve as points from its start to its end (closed curves end
  where they start), arcs and ellipses split every `ARC_STEP`.

- `pub fn swept_area(&self) -> f64`

  The signed area the curve sweeps around the origin, the integral of
  `(x dy - y dx) / 2` along it: summed around a closed loop, the loop's
  exact area (positive counter-clockwise), arcs and ellipses as the true
  curves rather than the points they are split into.

- `pub fn is_curved(&self) -> bool`

  Whether the curve is curved (an arc, a circle, an ellipse).

- `pub fn reversed(&self) -> Self`

  The same curve run the other way.

- `pub fn transform(&mut self, t: &PlanTransform)`

  Move, rotate or mirror the curve.


<a id="cc_core-sketch-sketcherror"></a>

### enum `SketchError`

```rust
pub enum SketchError
```

Why a sketch makes no outline.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `std::fmt::Display`

**Variants**

- `Empty`

  No curves.

- `InvalidCurve(usize)`

  A curve with invalid numbers or no size.

- `TooLarge`

  Too many curves or points.

- `OpenEnd(DVec2)`

  A curve ends here without meeting another.

- `Branch(DVec2)`

  More than two curves meet here.

- `Crossing(DVec2)`

  Loops cross or touch here (themselves or each other).

- `Apart(DVec2)`

  A loop lies outside the others (more than one outline), near here.

- `Nested(DVec2)`

  A loop lies inside an opening, near here.

- `Openings(DVec2)`

  The outline has openings where none are allowed, near here.

- `NoArea(DVec2)`

  A loop encloses no area, near here.


**Methods**

- `pub fn key(&self) -> &'static str`

  Translation key of the message.

- `pub fn point(&self) -> Option<DVec2>`

  Where it happens, to show it.


<a id="cc_core-sketch-profile"></a>

### struct `Profile`

```rust
pub struct Profile
```

An outline and the openings in it, from a sketch.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `outline: Vec<DVec2>`
- `openings: Vec<Vec<DVec2>>`

<a id="cc_core-sketch-sketch"></a>

### struct `Sketch`

```rust
pub struct Sketch
```

The curves an outline is drawn with.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `curves: Vec<SketchCurve>`

**Methods**

- `pub fn from_polygons(outline: &[DVec2], openings: &[Vec<DVec2>]) -> Self`

  Lines around `outline` and each of `openings`.

- `pub fn transform(&mut self, t: &PlanTransform)`

  Move, rotate or mirror every curve.

- `pub fn is_valid(&self) -> bool`

  Valid curves and not too many of them.

- `pub fn loops(&self) -> Result<Vec<Vec<DVec2>>, SketchError>`

  The closed loops the curves make, each as points without the closing
  repeat.

- `pub fn area(&self) -> Result<f64, SketchError>`

  The exact area the sketch encloses: its outline less its openings,
  arcs, circles and ellipses as the true curves (the outline's points
  split them into chords, which cut a little off).

- `pub fn is_curved(&self) -> bool`

  Whether any curve is curved: only then does `Self::area` differ
  from the area of the points.

- `pub fn profile(&self) -> Result<Profile, SketchError>`

  The outline (the loop around all the others) and the openings in it.

- `pub fn outline(&self) -> Result<Vec<DVec2>, SketchError>`

  `Self::profile` for outlines that take no openings.

- `pub fn makes(&self, outline: &[DVec2], openings: &[Vec<DVec2>]) -> bool`

  Whether the sketch still makes `outline` and `openings` (the same
  points, in order from any start and either way round): an outline
  edited without its sketch leaves it behind.


<a id="mod-cc_core-sketch_edit"></a>

## `cc_core::sketch_edit`

<sub>`crates/cc_core/src/sketch_edit.rs`</sub>

Editing the curves of a sketch: where two curves (or the lines, circles
and ellipses they lie on) cross, a curve trimmed or extended to a point,
split in two, and offset alone or along the chain of curves joined to it.
The sketch tools of the app use these; `None` means the edit makes no
valid curve (closed curves have no end to trim, ellipses no offset that
is an ellipse, curves that never meet).

**Items:** [`crossings`](#cc_core-sketch_edit-crossings) · [`nearest`](#cc_core-sketch_edit-nearest) · [`param`](#cc_core-sketch_edit-param) · [`span`](#cc_core-sketch_edit-span) · [`between`](#cc_core-sketch_edit-between) · [`trim_to`](#cc_core-sketch_edit-trim_to) · [`trim_corner`](#cc_core-sketch_edit-trim_corner) · [`trim_extend`](#cc_core-sketch_edit-trim_extend) · [`split`](#cc_core-sketch_edit-split) · [`offset_left`](#cc_core-sketch_edit-offset_left) · [`offset`](#cc_core-sketch_edit-offset) · [`chain`](#cc_core-sketch_edit-chain) · [`offset_chain`](#cc_core-sketch_edit-offset_chain)

<a id="cc_core-sketch_edit-crossings"></a>

### fn `crossings`

```rust
pub fn crossings(a: &SketchCurve, b: &SketchCurve) -> Vec<DVec2>
```

Where the lines, circles and ellipses curves `a` and `b` lie on cross
(lines run on past their ends, arcs round their circle, partial
ellipses round the whole ellipse). Tangent contacts may be missed.

<a id="cc_core-sketch_edit-nearest"></a>

### fn `nearest`

```rust
pub fn nearest(points: &[DVec2], to: DVec2) -> Option<DVec2>
```

The point of `points` nearest `to`.

<a id="cc_core-sketch_edit-param"></a>

### fn `param`

```rust
pub fn param(c: &SketchCurve, p: DVec2) -> Option<f64>
```

Where `p` lies along an open curve from its start: meters along lines
and arcs, radians of the parameter along ellipses; before the start it
is negative and past the end greater than `span`, where the curve runs
on. `None` for closed curves.

<a id="cc_core-sketch_edit-span"></a>

### fn `span`

```rust
pub fn span(c: &SketchCurve) -> Option<f64>
```

The parameter of an open curve's end (see `param`).

<a id="cc_core-sketch_edit-between"></a>

### fn `between`

```rust
pub fn between(c: &SketchCurve, u0: f64, u1: f64) -> Option<SketchCurve>
```

The part of an open curve between the parameters `u0 < u1` (see
`param`); beyond its ends the curve runs on, so this also extends it.

<a id="cc_core-sketch_edit-trim_to"></a>

### fn `trim_to`

```rust
pub fn trim_to(c: &SketchCurve, keep: DVec2, at: DVec2) -> Option<SketchCurve>
```

The open curve `c` ending at `at` (a point on it or where it runs on),
keeping the side of `at` where `keep` is: trimmed or extended.

<a id="cc_core-sketch_edit-trim_corner"></a>

### fn `trim_corner`

```rust
pub fn trim_corner(c1: &SketchCurve, k1: DVec2, c2: &SketchCurve, k2: DVec2) -> Option<(SketchCurve, SketchCurve)>
```

Two curves trimmed or extended to meet where they cross, each keeping
the side clicked (`k1`, `k2`); the crossing nearest the clicks.

<a id="cc_core-sketch_edit-trim_extend"></a>

### fn `trim_extend`

```rust
pub fn trim_extend(c: &SketchCurve, keep: DVec2, boundary: &SketchCurve) -> Option<SketchCurve>
```

Curve `c` trimmed or extended to the curve `boundary`, keeping the side
clicked (`keep`); the crossing nearest the click.

<a id="cc_core-sketch_edit-split"></a>

### fn `split`

```rust
pub fn split(c: &SketchCurve, p: DVec2) -> Option<(SketchCurve, SketchCurve)>
```

Curve `c` split in two at the point of it nearest `p`. A circle or a
whole ellipse splits into halves, from `p` and from the point opposite.

<a id="cc_core-sketch_edit-offset_left"></a>

### fn `offset_left`

```rust
pub fn offset_left(c: &SketchCurve, v: f64) -> Option<SketchCurve>
```

Curve `c` moved `v` to its left (to its right when negative); `None`
for ellipses and when an arc's or a circle's radius would vanish.

<a id="cc_core-sketch_edit-offset"></a>

### fn `offset`

```rust
pub fn offset(c: &SketchCurve, distance: f64, toward: DVec2) -> Option<SketchCurve>
```

A copy of curve `c` `distance` away from it, on the side of `toward`.

<a id="cc_core-sketch_edit-chain"></a>

### fn `chain`

```rust
pub fn chain(curves: &[SketchCurve], i: usize) -> (Vec<(usize, bool)>, bool)
```

The curves joined end to end with curve `i`, in order along the chain,
each with whether it runs backwards in it, and whether the chain
closes. The chain stops where more than two curves meet.

<a id="cc_core-sketch_edit-offset_chain"></a>

### fn `offset_chain`

```rust
pub fn offset_chain(curves: &[SketchCurve], i: usize, distance: f64, toward: DVec2) -> Option<Vec<SketchCurve>>
```

Copies of the chain of curve `i` (see `chain`) `distance` away on the
side of `toward`, neighbors meeting where their copies cross (or joined
by a line where they do not); `None` when a curve has no offset.

<a id="mod-cc_core-spline"></a>

## `cc_core::spline`

<sub>`crates/cc_core/src/spline.rs`</sub>

Splines of sketches: a chain of cubic Bézier segments, kept as their
control points `p0 c1 c2 p1 c1 c2 p2 …` (`3n + 1` points for `n`
segments). The drawing tool passes through the points clicked
(`interpolate`); splitting (`split_at`), reversing, moving and the
enclosed area (`swept_area`) stay exact on that form.

**Items:** [`MAX_SEGMENTS`](#cc_core-spline-max_segments) · [`STEPS`](#cc_core-spline-steps) · [`segment_count`](#cc_core-spline-segment_count) · [`segment`](#cc_core-spline-segment) · [`segments`](#cc_core-spline-segments) · [`is_valid`](#cc_core-spline-is_valid) · [`is_closed`](#cc_core-spline-is_closed) · [`point`](#cc_core-spline-point) · [`derivative`](#cc_core-spline-derivative) · [`point_at`](#cc_core-spline-point_at) · [`direction_at`](#cc_core-spline-direction_at) · [`flatten`](#cc_core-spline-flatten) · [`length`](#cc_core-spline-length) · [`swept_area`](#cc_core-spline-swept_area) · [`split`](#cc_core-spline-split) · [`split_at`](#cc_core-spline-split_at) · [`between`](#cc_core-spline-between) · [`nearest`](#cc_core-spline-nearest) · [`reversed`](#cc_core-spline-reversed) · [`interpolate`](#cc_core-spline-interpolate) · [`offset`](#cc_core-spline-offset)

<a id="cc_core-spline-max_segments"></a>

### const `MAX_SEGMENTS`

```rust
pub const MAX_SEGMENTS: usize
```

Most segments a spline holds (a guard against crafted files).

<a id="cc_core-spline-steps"></a>

### const `STEPS`

```rust
pub const STEPS: usize
```

Points each segment is split into to draw it.

<a id="cc_core-spline-segment_count"></a>

### fn `segment_count`

```rust
pub fn segment_count(points: &[DVec2]) -> Option<usize>
```

The number of segments of `points`, when they are a spline's control points.

<a id="cc_core-spline-segment"></a>

### fn `segment`

```rust
pub fn segment(points: &[DVec2], i: usize) -> Option<[DVec2; 4]>
```

The control points of segment `i`.

<a id="cc_core-spline-segments"></a>

### fn `segments`

```rust
pub fn segments(points: &[DVec2]) -> impl Iterator<Item = [DVec2; 4]> + '_
```

The segments of `points`.

<a id="cc_core-spline-is_valid"></a>

### fn `is_valid`

```rust
pub fn is_valid(points: &[DVec2]) -> bool
```

A whole number of segments, not too many, finite, and not all in one point.

<a id="cc_core-spline-is_closed"></a>

### fn `is_closed`

```rust
pub fn is_closed(points: &[DVec2]) -> bool
```

Whether the spline ends where it starts (with more than one segment).

<a id="cc_core-spline-point"></a>

### fn `point`

```rust
pub fn point([p0, p1, p2, p3]: [DVec2; 4], t: f64) -> DVec2
```

The point of a segment at `t` in `0..=1`.

<a id="cc_core-spline-derivative"></a>

### fn `derivative`

```rust
pub fn derivative([p0, p1, p2, p3]: [DVec2; 4], t: f64) -> DVec2
```

The derivative of a segment at `t`.

<a id="cc_core-spline-point_at"></a>

### fn `point_at`

```rust
pub fn point_at(points: &[DVec2], s: f64) -> Option<DVec2>
```

The point at the spline's parameter `s` in `0..=n` (segment `floor(s)`).

<a id="cc_core-spline-direction_at"></a>

### fn `direction_at`

```rust
pub fn direction_at(points: &[DVec2], s: f64) -> Option<DVec2>
```

The unit direction of travel at the parameter `s`.

<a id="cc_core-spline-flatten"></a>

### fn `flatten`

```rust
pub fn flatten(points: &[DVec2]) -> Vec<DVec2>
```

The spline as points from its start to its end, each segment split in
`STEPS`; the ends are exact.

<a id="cc_core-spline-length"></a>

### fn `length`

```rust
pub fn length(points: &[DVec2]) -> f64
```

The length of the drawn spline.

<a id="cc_core-spline-swept_area"></a>

### fn `swept_area`

```rust
pub fn swept_area(points: &[DVec2]) -> f64
```

The signed area the spline sweeps around the origin, the integral of
`(x dy - y dx) / 2` along it, exact: the integrand is a polynomial of
degree 5, which 3-point Gauss-Legendre integrates exactly.

<a id="cc_core-spline-split"></a>

### fn `split`

```rust
pub fn split([p0, p1, p2, p3]: [DVec2; 4], t: f64) -> ([DVec2; 4], [DVec2; 4])
```

A segment split at `t`: the part before and the part after.

<a id="cc_core-spline-split_at"></a>

### fn `split_at`

```rust
pub fn split_at(points: &[DVec2], s: f64) -> Option<(Vec<DVec2>, Vec<DVec2>)>
```

The spline split at its parameter `s`: the part before and the part
after (either may be empty at an end).

<a id="cc_core-spline-between"></a>

### fn `between`

```rust
pub fn between(points: &[DVec2], s0: f64, s1: f64) -> Option<Vec<DVec2>>
```

The part of the spline between the parameters `s0` and `s1` (`s0 < s1`).

<a id="cc_core-spline-nearest"></a>

### fn `nearest`

```rust
pub fn nearest(points: &[DVec2], q: DVec2) -> Option<(f64, DVec2, f64)>
```

The nearest point of the spline to `q`: its parameter, the point and the distance.

<a id="cc_core-spline-reversed"></a>

### fn `reversed`

```rust
pub fn reversed(points: &[DVec2]) -> Vec<DVec2>
```

The same spline run the other way.

<a id="cc_core-spline-interpolate"></a>

### fn `interpolate`

```rust
pub fn interpolate(fit: &[DVec2], closed: bool) -> Vec<DVec2>
```

A spline through the `fit` points (closed back to the first when
`closed`): tangents from the neighbors, spaced by the distances between
the points, so it neither overshoots short spans nor flattens long ones.
Points repeated in a row count once; fewer than two distinct points make
no spline.

<a id="cc_core-spline-offset"></a>

### fn `offset`

```rust
pub fn offset(points: &[DVec2], d: f64) -> Vec<DVec2>
```

A spline `d` to the left of this one (to the right when negative), as
near as offsetting each segment's control points along its end normals
allows (exact for straight segments, close for gentle curves).

<a id="mod-cc_core-stair"></a>

## `cc_core::stair`

<sub>`crates/cc_core/src/stair.rs`</sub>

Stairs: the plan shapes of a stair element, how its type is built (solid,
cast in place, assembled) and the rules that size its steps (riser limits
and Blondel's formula `2 × riser + tread`).

**Items:** [`StairShape`](#cc_core-stair-stairshape) · [`Turn`](#cc_core-stair-turn) · [`StairLayout`](#cc_core-stair-stairlayout) · [`MAX_STEP_NUMBER`](#cc_core-stair-max_step_number) · [`MAX_WINDERS`](#cc_core-stair-max_winders) · [`MAX_RISERS`](#cc_core-stair-max_risers) · [`StairConstruction`](#cc_core-stair-stairconstruction) · [`Stringers`](#cc_core-stair-stringers) · [`LandingBeams`](#cc_core-stair-landingbeams) · [`PostAt`](#cc_core-stair-postat) · [`LandingPosts`](#cc_core-stair-landingposts) · [`StairBuild`](#cc_core-stair-stairbuild) · [`TreadRule`](#cc_core-stair-treadrule) · [`StairRules`](#cc_core-stair-stairrules) · [`stair_steps`](#cc_core-stair-stair_steps) · [`StairIssue`](#cc_core-stair-stairissue) · [`StairCheck`](#cc_core-stair-staircheck) · [`StairSide`](#cc_core-stair-stairside) · [`StairRailing`](#cc_core-stair-stairrailing)

<a id="cc_core-stair-stairshape"></a>

### enum `StairShape`

```rust
pub enum StairShape
```

Plan shape of a stair.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Straight`

  One straight flight.

- `StraightLanding`

  Two straight flights in line with a landing between them.

- `LShaped`

  Two flights at a right angle, with a landing or winders at the corner.

- `UShaped`

  Two parallel flights going opposite ways, joined by a landing.

- `ThreeFlight`

  Three flights turning twice at right angles, with a landing at each
  corner: the third comes back beside the first.

- `Spiral`

  Treads around a center (spiral or helical stair).


**Methods**

- `pub const ALL: [StairShape; 6]`
- `pub fn key(self) -> &'static str`
- `pub fn turns(self) -> bool`

  Whether the stair turns (so the side it turns to matters).

- `pub fn has_split(self) -> bool`

  Whether the stair has flights split by landings (or winders).


<a id="cc_core-stair-turn"></a>

### enum `Turn`

```rust
pub enum Turn
```

Side a stair turns to, walking up.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Left`

  Counter-clockwise in plan.

- `Right`

  Clockwise in plan.


**Methods**

- `pub const ALL: [Turn; 2]`
- `pub fn key(self) -> &'static str`
- `pub fn sign(self) -> f64`

  +1 to the left, -1 to the right.

- `pub fn flipped(self) -> Turn`

<a id="cc_core-stair-stairlayout"></a>

### struct `StairLayout`

```rust
pub struct StairLayout
```

Plan layout of a stair element (all fields have defaults, so files from
before shaped stairs read as straight ones).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `shape: StairShape`
- `turn: Turn`
- `split: u32`

  Risers of the first flight (0: half of them; a third on a three-flight stair).

- `split2: u32`

  Risers of the second flight of a three-flight stair (0: half of those
  left after the first).

- `risers: u32`

  Riser count (0: the fewest within the type's maximum riser); the
  riser height is the stair's height over it.

- `tread: f64`

  Tread depth (0: the type's, or Blondel's formula when its rules say so).

- `landing: f64`

  Landing depth along the first flight (0: the stair width); straight
  and U-shaped stairs.

- `gap: f64`

  Clear gap between the flights of a U-shaped stair.

- `winders: u32`

  Winder treads in the corner of an L-shaped stair (0: a landing).

- `inner_radius: f64`

  Radius of the inner edge of a spiral stair (0: treads meet at the center).

- `posts: [LandingPosts; 2]`

  Posts under the landings of an assembled stair, down to its base: one
  set per landing, in order up the stair.

- `numbers: bool`

  Number the steps in plans: the treads (winders included, landings
  not) from `number_start` up, and the floor the stair arrives at.

- `number_start: i32`

  Number of the first tread (another value continues a count from a
  stair below, for instance).


**Methods**

- `pub fn repair(&mut self) -> usize`

  Replace invalid values by defaults; returns how many changed.

- `pub fn steps(&self, height: f64, max_riser: f64, tread: f64, rules: &StairRules) -> (usize, f64, f64)`

  Riser count, riser height and tread depth of a stair of `height` of a
  type with `max_riser`, `tread` and `rules`, with this layout's riser
  count and tread when it sets them.


<a id="cc_core-stair-max_step_number"></a>

### const `MAX_STEP_NUMBER`

```rust
pub const MAX_STEP_NUMBER: i32
```

Largest step number, either way from zero.

<a id="cc_core-stair-max_winders"></a>

### const `MAX_WINDERS`

```rust
pub const MAX_WINDERS: u32
```

Most winder treads in one corner.

<a id="cc_core-stair-max_risers"></a>

### const `MAX_RISERS`

```rust
pub const MAX_RISERS: u32
```

Most risers in one stair.

<a id="cc_core-stair-stairconstruction"></a>

### enum `StairConstruction`

```rust
pub enum StairConstruction
```

How a stair is built.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Solid`

  Steps filled down to the base (a solid block, e.g. on fill).

- `CastInPlace`

  Reinforced concrete cast in place: steps on an inclined waist slab
  `waist` thick (landings are slabs of the same thickness).

- `Assembled`

  Assembled (precast, steel or timber): treads `tread_thickness` thick
  on stringers, with or without riser boards.


**Methods**

- `pub const ALL: [StairConstruction; 3]`
- `pub fn key(self) -> &'static str`

<a id="cc_core-stair-stringers"></a>

### enum `Stringers`

```rust
pub enum Stringers
```

Where the stringers of an assembled stair go.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Sides`

  One under each side of the treads.

- `Center`

  One under the middle of the treads (a central column on spirals).


**Methods**

- `pub const ALL: [Stringers; 2]`
- `pub fn key(self) -> &'static str`

<a id="cc_core-stair-landingbeams"></a>

### enum `LandingBeams`

```rust
pub enum LandingBeams
```

How the beams under a landing of an assembled stair with a central
stringer run (with side stringers they frame the landing's edge).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Cross`

  The lines of the stringers of the flights the landing joins carried
  across it, crossing (a beam across the middle on straight and
  U-shaped landings).

- `Tee`

  One beam across the landing and the other springing from it: on a
  corner, the beam of the arriving flight's stringer, with the leaving
  flight's from its middle to the edge the flight leaves; on a
  U-shaped landing, a beam across the middle with the stringers'
  beams from the front up to it.


**Methods**

- `pub const ALL: [LandingBeams; 2]`
- `pub fn key(self) -> &'static str`

<a id="cc_core-stair-postat"></a>

### enum `PostAt`

```rust
pub enum PostAt
```

A place round the edge of a landing where a post can stand: a corner or
the middle of an edge, named walking onto the landing (front: the edge
the stair arrives at; inner: the side the stair turns to).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Serialize`, `Deserialize`

**Variants**

- `FrontOuter`
- `Front`
- `FrontInner`
- `Inner`
- `BackInner`
- `Back`
- `BackOuter`
- `Outer`

**Methods**

- `pub const ALL: [PostAt; 8]`

  Round the edge, from the front corner on the outer side.

- `pub fn name(self) -> &'static str`

  The name in commands and files.

- `pub fn from_name(name: &str) -> Option<PostAt>`
- `pub fn key(self) -> &'static str`

<a id="cc_core-stair-landingposts"></a>

### struct `LandingPosts`

```rust
pub struct LandingPosts
```

Where posts stand under one landing (only places where the landing's
beams reach its edge take one, see `cc_geom::StairLanding::post_spots`).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `front_outer: bool`
- `front: bool`
- `front_inner: bool`
- `inner: bool`
- `back_inner: bool`
- `back: bool`
- `back_outer: bool`
- `outer: bool`

**Methods**

- `pub fn get(mut self, at: PostAt) -> bool`
- `pub fn set(&mut self, at: PostAt, on: bool)`
- `pub fn any(self) -> bool`

<a id="cc_core-stair-stairbuild"></a>

### struct `StairBuild`

```rust
pub struct StairBuild
```

Construction of a stair type.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `construction: StairConstruction`
- `waist: f64`

  Thickness of the waist slab of a cast-in-place stair, square to its pitch.

- `stringers: Stringers`
- `stringer_width: f64`

  Width and depth of the stringers of an assembled stair.

- `stringer_depth: f64`
- `tread_thickness: f64`

  Thickness of the treads of an assembled stair.

- `closed_risers: bool`

  Riser boards closing the gap between treads (assembled stairs).

- `landing_beams: LandingBeams`

  Layout of the beams under the landings of an assembled stair with a
  central stringer (they are as wide and deep as the stringers, under
  a deck as thick as the treads).

- `post_width: f64`

  Side of the square posts under the landings of an assembled stair.


**Methods**

- `pub fn repair(&mut self) -> usize`

  Replace invalid sizes by defaults; returns how many changed.


<a id="cc_core-stair-treadrule"></a>

### enum `TreadRule`

```rust
pub enum TreadRule
```

How the tread depth of a stair type is set.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Fixed`

  The type's tread depth.

- `Blondel`

  From Blondel's formula: the middle of the step range minus twice the riser.


**Methods**

- `pub const ALL: [TreadRule; 2]`
- `pub fn key(self) -> &'static str`

<a id="cc_core-stair-stairrules"></a>

### struct `StairRules`

```rust
pub struct StairRules
```

Comfort and safety limits of a stair type (defaults after common
building codes: risers 16-18 cm, treads at least 28 cm and
`2 × riser + tread` between 63 and 65 cm).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `tread_rule: TreadRule`
- `min_riser: f64`
- `min_tread: f64`
- `blondel_min: f64`

  Range of Blondel's step `2 × riser + tread`.

- `blondel_max: f64`

**Methods**

- `pub fn repair(&mut self) -> usize`

  Replace invalid limits by defaults; returns how many changed.

- `pub fn tread_for(&self, riser: f64, tread: f64) -> f64`

  Tread depth for a riser height: `tread` for a fixed rule, else the
  middle of the Blondel range minus twice the riser (at least 10 cm).


<a id="cc_core-stair-stair_steps"></a>

### fn `stair_steps`

```rust
pub fn stair_steps(height: f64, max_riser: f64, tread: f64, rules: &StairRules) -> (usize, f64, f64)
```

Riser count, riser height and tread depth of a stair of `height`: the
fewest risers at most `max_riser` high; the tread from the rules.

<a id="cc_core-stair-stairissue"></a>

### enum `StairIssue`

```rust
pub enum StairIssue
```

A broken rule of a stair.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`

**Variants**

- `RiserTooHigh { riser: f64, max: f64 }`
- `RiserTooLow { riser: f64, min: f64 }`
- `TreadTooShort { tread: f64, min: f64 }`
- `BlondelOutOfRange { step: f64, min: f64, max: f64 }`
- `LandingTooShort { landing: f64, min: f64 }`

**Methods**

- `pub fn key(&self) -> &'static str`

<a id="cc_core-stair-staircheck"></a>

### struct `StairCheck`

```rust
pub struct StairCheck
```

Riser, tread (on the walking line), Blondel's step and the rules they break.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`

**Fields**

- `risers: usize`
- `riser: f64`
- `tread: f64`
- `blondel: f64`

  `2 × riser + tread`.

- `issues: Vec<StairIssue>`

**Methods**

- `pub fn new(risers: usize, riser: f64, tread: f64, max_riser: f64, rules: &StairRules, landing: Option<(f64, f64)>) -> Self`

  Check a riser and tread against `rules`, `max_riser` and, when the
  stair has one, a landing at least `width` deep.

- `pub fn ok(&self) -> bool`

<a id="cc_core-stair-stairside"></a>

### enum `StairSide`

```rust
pub enum StairSide
```

Side of a stair, walking up.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Left`
- `Right`

**Methods**

- `pub const ALL: [StairSide; 2]`
- `pub fn flipped(self) -> StairSide`

<a id="cc_core-stair-stairrailing"></a>

### struct `StairRailing`

```rust
pub struct StairRailing
```

A railing that follows a side of a stair (`host`), `inset` in from its edge.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `host: crate::ElementId`
- `side: StairSide`
- `inset: f64`

<a id="mod-cc_core-steel_building"></a>

## `cc_core::steel_building`

<sub>`crates/cc_core/src/steel_building.rs`</sub>

Parametric steel buildings: a host element that holds the parameters of
a steel shed (span, eave height, roof slope and shape, bays, the types of
its members, purlins and girts) and owns ordinary columns and beams made
from them. Every edit keeps the members in step (see
`crate::EditCtx`): a changed building rewrites its members in place,
keeping their ids while their roles exist; a member the user edited or
deleted is detached, and the building leaves its role alone from then on.
Local coordinates: x across the span (from the first column line), y
along the building (frame `k` at `k * bay_spacing`), mapped to plan
through `origin` and `rotation_deg`.

**Items:** [`MAX_MEMBERS`](#cc_core-steel_building-max_members) · [`MISSING_SECTION`](#cc_core-steel_building-missing_section) · [`SPAN`](#cc_core-steel_building-span) · [`EAVE_HEIGHT`](#cc_core-steel_building-eave_height) · [`SLOPE`](#cc_core-steel_building-slope) · [`BAYS`](#cc_core-steel_building-bays) · [`BAY_SPACING`](#cc_core-steel_building-bay_spacing) · [`SPACING`](#cc_core-steel_building-spacing) · [`DEFAULT_SPAN`](#cc_core-steel_building-default_span) · [`DEFAULT_EAVE_HEIGHT`](#cc_core-steel_building-default_eave_height) · [`DEFAULT_SLOPE`](#cc_core-steel_building-default_slope) · [`DEFAULT_BAYS`](#cc_core-steel_building-default_bays) · [`DEFAULT_BAY_SPACING`](#cc_core-steel_building-default_bay_spacing) · [`DEFAULT_SECONDARY_SPACING`](#cc_core-steel_building-default_secondary_spacing) · [`FrameRoof`](#cc_core-steel_building-frameroof) · [`RoleKind`](#cc_core-steel_building-rolekind) · [`MemberRole`](#cc_core-steel_building-memberrole) · [`Secondary`](#cc_core-steel_building-secondary) · [`FrameMember`](#cc_core-steel_building-framemember) · [`SteelBuilding`](#cc_core-steel_building-steelbuilding) · [`section_box`](#cc_core-steel_building-section_box) · [`host_type`](#cc_core-steel_building-host_type)

<a id="cc_core-steel_building-max_members"></a>

### const `MAX_MEMBERS`

```rust
pub const MAX_MEMBERS: usize
```

Most members one building makes; the layout stops there.

<a id="cc_core-steel_building-missing_section"></a>

### const `MISSING_SECTION`

```rust
pub const MISSING_SECTION: f64
```

The width and depth of a type whose section cannot be read, meters.

<a id="cc_core-steel_building-span"></a>

### const `SPAN`

```rust
pub const SPAN: RangeInclusive<f64>
```

<a id="cc_core-steel_building-eave_height"></a>

### const `EAVE_HEIGHT`

```rust
pub const EAVE_HEIGHT: RangeInclusive<f64>
```

<a id="cc_core-steel_building-slope"></a>

### const `SLOPE`

```rust
pub const SLOPE: RangeInclusive<f64>
```

<a id="cc_core-steel_building-bays"></a>

### const `BAYS`

```rust
pub const BAYS: RangeInclusive<u32>
```

<a id="cc_core-steel_building-bay_spacing"></a>

### const `BAY_SPACING`

```rust
pub const BAY_SPACING: RangeInclusive<f64>
```

<a id="cc_core-steel_building-spacing"></a>

### const `SPACING`

```rust
pub const SPACING: RangeInclusive<f64>
```

Purlin and girt spacing.

<a id="cc_core-steel_building-default_span"></a>

### const `DEFAULT_SPAN`

```rust
pub const DEFAULT_SPAN: f64
```

<a id="cc_core-steel_building-default_eave_height"></a>

### const `DEFAULT_EAVE_HEIGHT`

```rust
pub const DEFAULT_EAVE_HEIGHT: f64
```

<a id="cc_core-steel_building-default_slope"></a>

### const `DEFAULT_SLOPE`

```rust
pub const DEFAULT_SLOPE: f64
```

<a id="cc_core-steel_building-default_bays"></a>

### const `DEFAULT_BAYS`

```rust
pub const DEFAULT_BAYS: u32
```

<a id="cc_core-steel_building-default_bay_spacing"></a>

### const `DEFAULT_BAY_SPACING`

```rust
pub const DEFAULT_BAY_SPACING: f64
```

<a id="cc_core-steel_building-default_secondary_spacing"></a>

### const `DEFAULT_SECONDARY_SPACING`

```rust
pub const DEFAULT_SECONDARY_SPACING: f64
```

<a id="cc_core-steel_building-frameroof"></a>

### enum `FrameRoof`

```rust
pub enum FrameRoof
```

The roof of the frames: two rafters meeting at mid-span, or one rising
toward `x = span`.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Gable`
- `Mono`

**Methods**

- `pub const ALL: [FrameRoof; 2]`
- `pub fn code(self) -> &'static str`
- `pub fn from_code(code: &str) -> Option<Self>`

<a id="cc_core-steel_building-rolekind"></a>

### enum `RoleKind`

```rust
pub enum RoleKind
```

What a member is in its building.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Serialize`, `Deserialize`

**Variants**

- `Column`
- `Rafter`
- `Purlin`
- `Girt`

**Methods**

- `pub const ALL: [RoleKind; 4]`

<a id="cc_core-steel_building-memberrole"></a>

### struct `MemberRole`

```rust
pub struct MemberRole
```

A member's place: its kind, its frame (columns, rafters) or bay
(purlins, girts), and its index there (girts: `side * 1000 + row`).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Serialize`, `Deserialize`

**Fields**

- `kind: RoleKind`
- `frame: u16`
- `index: u16`

**Methods**

- `pub fn fits(self, kind: &ElementKind) -> bool`

  Whether an element of `kind` can fill this role: a column for
  columns, a beam for the rest.

- `pub fn key(self) -> RoleKey`

  Its key for the shared member-host code (same order as the role).

- `pub fn from_key(key: RoleKey) -> Option<Self>`

<a id="cc_core-steel_building-secondary"></a>

### struct `Secondary`

```rust
pub struct Secondary
```

Purlins or girts: their beam type and the distance between lines
(purlins: along the rafter's plan run; girts: up the wall).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `type_id: ElementTypeId`
- `spacing: f64`

<a id="cc_core-steel_building-framemember"></a>

### struct `FrameMember`

```rust
pub struct FrameMember
```

A member the building manages.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Serialize`, `Deserialize`

**Fields**

- `role: MemberRole`
- `id: ElementId`

<a id="cc_core-steel_building-steelbuilding"></a>

### struct `SteelBuilding`

```rust
pub struct SteelBuilding
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`, `MemberHost`

**Fields**

- `origin: DVec2`

  Plan point of the first frame's left column.

- `rotation_deg: f64`

  Turn of the local x axis (across the span), degrees counter-clockwise.

- `span: f64`

  Column axis to column axis, meters.

- `eave_height: f64`

  Column top above the level (the rafters' underside at the column axis), meters.

- `slope_percent: f64`

  Roof slope, percent.

- `roof: FrameRoof`
- `bays: u32`

  Bays between frames (frames = bays + 1).

- `bay_spacing: f64`

  Frame to frame, meters.

- `column_type: ElementTypeId`
- `rafter_type: ElementTypeId`
- `purlins: Option<Secondary>`
- `girts: Option<Secondary>`
- `members: Vec<FrameMember>`

  The members it manages, by role.

- `detached: Vec<MemberRole>`

  Roles the user took over (edited or deleted members): not made again.


**Methods**

- `pub fn new(column_type: ElementTypeId, rafter_type: ElementTypeId, secondary: Option<ElementTypeId>) -> Self`

  A building with the default parameters at the origin; `secondary`
  gives purlins and girts of that beam type.

- `pub fn check(&self) -> Result<(), String>`

  Whether every number is in its range (the first one that is not, named).

- `pub fn check_types(&self, project: &Project) -> Result<(), String>`

  Whether the column type is a column type and the others beam types.

- `pub fn reset_out_of_range(&mut self) -> usize`

  Put every out-of-range number back to its default; returns how many.

- `pub fn type_ids(&self) -> impl Iterator<Item = ElementTypeId> + '_`

  The member types it uses.

- `pub fn length(&self) -> f64`

  Frame line to frame line, meters.

- `pub fn to_plan(&self, local: DVec2) -> DVec2`

  A local point in plan.

- `pub fn footprint(&self) -> [DVec2; 4]`

  The column lines' rectangle in plan, counter-clockwise from the origin.

- `pub fn transform(&mut self, t: &PlanTransform)`

  Move, turn or mirror it. A mirror flips the local frame's handedness,
  so the far gable's corner becomes the origin (the frames run the
  other way and the footprint stays where the mirror puts it).

- `pub fn layout(&self, project: &Project) -> Vec<(MemberRole, ElementTypeId, ElementKind)>`

  Its members: role, type and element, in a fixed order (columns,
  rafters, purlins, girts), at most `MAX_MEMBERS`. Empty when a
  number is out of range.


<a id="cc_core-steel_building-section_box"></a>

### fn `section_box`

```rust
pub fn section_box(project: &Project, type_id: ElementTypeId) -> (f64, f64)
```

The width (across) and depth (up) of a column or beam type's section
box, `MISSING_SECTION` each when it cannot be read.

<a id="cc_core-steel_building-host_type"></a>

### fn `host_type`

```rust
pub fn host_type(id: ElementTypeId, name: String) -> ElementType
```

The type a project gets with its first steel building.

<a id="mod-cc_core-structural_material"></a>

## `cc_core::structural_material`

<sub>`crates/cc_core/src/structural_material.rs`</sub>

The structural material of a column or beam type: concrete, steel
(rolled or cold-formed) or timber. It comes from the type's profile,
else from the type's own material (a legacy section that makes no
outline).

**Items:** [`StructuralMaterial`](#cc_core-structural_material-structuralmaterial)

<a id="cc_core-structural_material-structuralmaterial"></a>

### enum `StructuralMaterial`

```rust
pub enum StructuralMaterial
```

What a column or beam is made of, as the Structure tab groups its tools.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Default`

**Variants**

- `Concrete`
- `Steel`
- `Timber`

**Methods**

- `pub const ALL: [StructuralMaterial; 3]`
- `pub fn code(self) -> &'static str`
- `pub fn from_code(c: &str) -> Option<Self>`
- `pub fn key(self) -> &'static str`

  The i18n key of its name (shared with the profile materials).

- `pub fn of_profile(m: ProfileMaterial) -> Self`

  The structural material of a profile's: cold-formed steel is steel.


<a id="mod-cc_core-structural_profile"></a>

## `cc_core::structural_profile`

<sub>`crates/cc_core/src/structural_profile.rs`</sub>

Structural profiles: the cross-sections of columns, beams and framing
members as named project objects (`Project::profiles`). A profile is a
standard parametric shape (rolled, welded, cold-formed, timber or
concrete) or a parametric sketch (`ParamSketch`). Lengths are meters.
Outlines are laid out as in steel tables: the depth along +y, the web of
an I or channel vertical, the flanges of a channel and the legs of an
angle pointing to +x; centered on the bounding box.

**Items:** [`ARC_SEGMENTS`](#cc_core-structural_profile-arc_segments) · [`MAX_POINTS`](#cc_core-structural_profile-max_points) · [`MAX_OPENINGS`](#cc_core-structural_profile-max_openings) · [`MIN_THICKNESS`](#cc_core-structural_profile-min_thickness) · [`MAX_SIZE`](#cc_core-structural_profile-max_size) · [`MAX_NAME`](#cc_core-structural_profile-max_name) · [`MAX_DESCRIPTION`](#cc_core-structural_profile-max_description) · [`MAX_PROFILES`](#cc_core-structural_profile-max_profiles) · [`ProfileMaterial`](#cc_core-structural_profile-profilematerial) · [`ProfileShape`](#cc_core-structural_profile-profileshape) · [`MAX_SKETCH_PARAMS`](#cc_core-structural_profile-max_sketch_params) · [`ParamSketch`](#cc_core-structural_profile-paramsketch) · [`SectionOutline`](#cc_core-structural_profile-sectionoutline) · [`ProfileError`](#cc_core-structural_profile-profileerror) · [`CatalogRef`](#cc_core-structural_profile-catalogref) · [`ProfileDef`](#cc_core-structural_profile-profiledef)

<a id="cc_core-structural_profile-arc_segments"></a>

### const `ARC_SEGMENTS`

```rust
pub const ARC_SEGMENTS: usize
```

Segments of an arc that turns a quarter circle (fillets use this many).

<a id="cc_core-structural_profile-max_points"></a>

### const `MAX_POINTS`

```rust
pub const MAX_POINTS: usize
```

Most points in an outline, holes included.

<a id="cc_core-structural_profile-max_openings"></a>

### const `MAX_OPENINGS`

```rust
pub const MAX_OPENINGS: usize
```

Most holes in an outline.

<a id="cc_core-structural_profile-min_thickness"></a>

### const `MIN_THICKNESS`

```rust
pub const MIN_THICKNESS: f64
```

Smallest dimension, meters.

<a id="cc_core-structural_profile-max_size"></a>

### const `MAX_SIZE`

```rust
pub const MAX_SIZE: f64
```

Largest dimension, meters.

<a id="cc_core-structural_profile-max_name"></a>

### const `MAX_NAME`

```rust
pub const MAX_NAME: usize
```

Longest profile name, characters.

<a id="cc_core-structural_profile-max_description"></a>

### const `MAX_DESCRIPTION`

```rust
pub const MAX_DESCRIPTION: usize
```

Longest description, characters.

<a id="cc_core-structural_profile-max_profiles"></a>

### const `MAX_PROFILES`

```rust
pub const MAX_PROFILES: usize
```

Most profiles in a project (a crafted file).

<a id="cc_core-structural_profile-profilematerial"></a>

### enum `ProfileMaterial`

```rust
pub enum ProfileMaterial
```

What a profile is made of; decides the material a type made for it gets.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Steel`
- `ColdFormed`
- `Timber`
- `Concrete`

**Methods**

- `pub const ALL: [ProfileMaterial; 4]`
- `pub fn code(self) -> &'static str`
- `pub fn from_code(c: &str) -> Option<Self>`
- `pub fn key(self) -> &'static str`

  The i18n key of its name.

- `pub fn material_class(self) -> crate::MaterialClass`

  The `MaterialClass` a type made for this profile gets its material from.


<a id="cc_core-structural_profile-profileshape"></a>

### enum `ProfileShape`

```rust
pub enum ProfileShape
```

A standard parametric cross-section; every field is a length in meters
except `ColdZ::angle` (degrees).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Variants**

- `I { d: f64, bf: f64, tw: f64, tf: f64, r: f64, }`

  Rolled or welded I/H: depth, flange width, web and flange thickness, root radius.

- `IAsym { d: f64, bf_top: f64, tf_top: f64, bf_bot: f64, tf_bot: f64, tw: f64, }`

  I with different flanges.

- `Channel { d: f64, bf: f64, tw: f64, tf: f64, r: f64, }`

  Channel, web on the left, flanges to +x.

- `T { d: f64, bf: f64, tw: f64, tf: f64, r: f64, }`

  T with the flange on top.

- `Angle { h: f64, b: f64, t: f64, r: f64, }`

  Angle, legs to +x (`b`) and +y (`h`).

- `ColdU { d: f64, bf: f64, t: f64, ri: f64, }`

  Cold-formed channel without lips; `ri` is the inner bend radius.

- `ColdLippedU { d: f64, bf: f64, lip: f64, t: f64, ri: f64, }`

  Cold-formed channel with lips.

- `ColdZ { d: f64, bf: f64, lip: f64, t: f64, ri: f64, angle: f64, }`

  Cold-formed Z; `angle` turns the lips from the flange (90 = square).

- `ColdHat { d: f64, bf: f64, lip: f64, t: f64, ri: f64, }`

  Cold-formed hat (omega): crown width, brims of length `lip`.

- `RectTube { h: f64, b: f64, t: f64, ro: f64, }`
- `RoundTube { d: f64, t: f64, }`
- `Rect { b: f64, h: f64, chamfer: f64, }`

  Solid rectangle (timber or concrete), optionally chamfered.

- `Round { d: f64, }`
- `ConcreteT { h: f64, bf: f64, hf: f64, bw: f64, }`

  Concrete T beam, flange on top.

- `ConcreteL { h: f64, bf: f64, hf: f64, bw: f64, }`

  Concrete L beam, flange on top to +x.

- `AnnularSector { r_in: f64, r_out: f64, sweep: f64, }`

  An annular sector: the part of a ring between the radii `r_in` (0
  for a circular sector) and `r_out` turning by `sweep` degrees,
  symmetric about the Y axis with its outer face up, as the piece of a
  cylindrical wall a column continues.

- `Sketch(ParamSketch)`

**Methods**

- `pub fn from_column_section(s: ColumnSection) -> ProfileShape` *(in `cc_core::profiles_project`)*

  The profile shape of a legacy column section (`width` along x,
  `depth` along y).

- `pub fn from_beam(width: f64, height: f64, shape: SectionShape) -> ProfileShape` *(in `cc_core::profiles_project`)*

  The profile shape of a legacy `width` x `height` section filled by `shape`.

- `pub fn close_to(&self, other: &ProfileShape, tol: f64) -> bool` *(in `cc_core::profiles_project`)*

  Whether two shapes are the same kind with every parameter within
  `tol` (meters; degrees for angles), so float noise from a file does
  not make a new profile. Sketches compare exactly.

- `pub fn is_cold_formed(&self) -> bool` *(in `cc_core::profiles_project`)*

  Whether this is a cold-formed shape (bent from sheet).

- `pub fn has_arcs(&self) -> bool` *(in `cc_core::profiles_project`)*

  Whether the outline has arcs: a round, a radius above zero, or a
  sketch with arc or ellipse segments.

- `pub const MAX_SECTOR_SWEEP: f64`

  Largest turn of an annular sector, degrees.

- `pub fn sector_center(&self) -> Option<DVec2>`

  The center of an annular sector's ring in the section's own frame
  (the outline is centered on its bounding box): below the origin on
  the Y axis. `None` for other shapes and invalid values.

- `pub const CODES: &'static [&'static str]`

  Codes of the shapes, in the order a picker lists them.

- `pub fn code(&self) -> &'static str`
- `pub fn default_for(code: &str) -> Option<ProfileShape>`

  A shape with sensible values for `code`.

- `pub fn params(&self) -> Vec<(String, f64)>`

  The named values of the shape, in declaration order.

- `pub fn set_param(&mut self, name: &str, v: f64) -> bool`

  Sets a value by name; false when the shape has none of that name. The
  value is checked when the outline is built.

- `pub fn outline(&self) -> Result<SectionOutline, ProfileError>`

  The outline, centered on its bounding box.

- `pub fn area(&self) -> Result<f64, ProfileError>`

  The section's area, exact for its arcs: the areas of outlines with
  arcs of `ARC_SEGMENTS` and twice as many chords, extrapolated (the
  chords' error falls with the square of their number), so a circle or a
  root radius is not counted short. Sketches give their outline's area.


<a id="cc_core-structural_profile-max_sketch_params"></a>

### const `MAX_SKETCH_PARAMS`

```rust
pub const MAX_SKETCH_PARAMS: usize
```

Most parameters of a sketch.

<a id="cc_core-structural_profile-paramsketch"></a>

### struct `ParamSketch`

```rust
pub struct ParamSketch
```

A parametric sketch: an outline and openings whose points are
expressions over named numeric parameters (meters).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `params: Vec<ComponentParam>`
- `outline: Vec<ProfilePoint>`
- `openings: Vec<Vec<ProfilePoint>>`

**Methods**

- `pub fn square(b: f64, h: f64) -> ParamSketch`

  A centered `b` by `h` rectangle with parameters `b` and `h`.

- `pub fn resolve(&self) -> Result<SectionOutline, ProfileError>`

  The outline, wound and centered on its bounding box.


<a id="cc_core-structural_profile-sectionoutline"></a>

### struct `SectionOutline`

```rust
pub struct SectionOutline
```

An outline: `outer` runs counter-clockwise, `holes` clockwise.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `outer: Vec<DVec2>`
- `holes: Vec<Vec<DVec2>>`

**Methods**

- `pub fn properties(&self, density: f64) -> SectionProperties` *(in `cc_core::section_props`)*

  The section's properties for a material of `density` kg/m³ (a
  non-finite or negative density counts as 0). A degenerate outline
  gives all zeros.

- `pub fn bbox(&self) -> (DVec2, DVec2)`

  Corners of the bounding box of the outer loop.


<a id="cc_core-structural_profile-profileerror"></a>

### enum `ProfileError`

```rust
pub enum ProfileError
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Error`

**Variants**

- `Value(&'static str)`
- `Range(&'static str)`
- `DoesNotFit(String)`
- `Sketch(String)`
- `TooBig`

<a id="cc_core-structural_profile-catalogref"></a>

### struct `CatalogRef`

```rust
pub struct CatalogRef
```

Where a profile comes from in a manufacturer's or standard's table.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `series: String`
- `designation: String`

<a id="cc_core-structural_profile-profiledef"></a>

### struct `ProfileDef`

```rust
pub struct ProfileDef
```

A named profile of the project.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `id: ProfileId`
- `uid: Uid`
- `name: String`
- `material: ProfileMaterial`
- `shape: ProfileShape`
- `catalog: Option<CatalogRef>`
- `description: String`

**Methods**

- `pub fn repair(&mut self, id: ProfileId) -> usize` *(in `cc_core::profiles_project`)*

  Fix what a hostile file can break without dropping the profile: the
  id (it must be the map key), the name and a missing uid. Returns the
  number of fixes.

- `pub fn outline(&self) -> Result<SectionOutline, ProfileError>`

<a id="mod-cc_core-sun"></a>

## `cc_core::sun`

<sub>`crates/cc_core/src/sun.rs`</sub>

The sun of a view: where it stands in the sky for a date, a time and the
project's site, and the settings each view keeps to place it.
A view that takes a sun (`ViewKind::takes_sun`: plans, ceiling plans,
3D views, elevations and sections) may keep `SunSettings` in
`View::sun`: a `SunMode` that is either a still moment, a study
over one day or several days (a list of `SunFrame`s, the current one
chosen by `SunSettings::frame`), or a lighting direction given by hand.
`SunSettings::state` (or `Project::sun_state` for a view) resolves
them into a `SunState`: the sun's azimuth and altitude and the unit
vector towards it in project coordinates, turned by the site's true
north (`SunSite`, from `crate::site::SiteLocation`).
The solar position uses the usual public low-precision formulas: the
sun's mean longitude and anomaly in Julian centuries from J2000 give its
apparent longitude, the declination and the equation of time (good to
about a hundredth of a degree and a few seconds for centuries around
2000), the true solar time gives the hour angle, and spherical
trigonometry the altitude and azimuth. No atmospheric refraction is applied to positions; sunrise and
sunset are when the sun's upper limb meets the horizon, its center at
`HORIZON_ALTITUDE` (refraction and the sun's radius together).
Named `SunPreset`s live in `Project::sun_presets`, tracked as
project settings (`EditCtx::sun_presets_mut`).
Dates and times are local to the site: a date is a `SunDate` and a
time of day is minutes after local midnight (`u16`, `0..=``MAX_MINUTE`
in settings; `f64` in frames and results). Everything is deterministic:
nothing reads the clock.

**Items:** [`MAX_MINUTE`](#cc_core-sun-max_minute) · [`MAX_FRAMES`](#cc_core-sun-max_frames) · [`MAX_DAYS`](#cc_core-sun-max_days) · [`MAX_PRESETS`](#cc_core-sun-max_presets) · [`MAX_PRESET_NAME`](#cc_core-sun-max_preset_name) · [`MIN_YEAR`](#cc_core-sun-min_year) · [`MAX_YEAR`](#cc_core-sun-max_year) · [`UTC_OFFSET_RANGE`](#cc_core-sun-utc_offset_range) · [`HORIZON_ALTITUDE`](#cc_core-sun-horizon_altitude) · [`CONVENTIONAL_AZIMUTH`](#cc_core-sun-conventional_azimuth) · [`conventional_altitude`](#cc_core-sun-conventional_altitude) · [`SunDate`](#cc_core-sun-sundate) · [`is_leap_year`](#cc_core-sun-is_leap_year) · [`days_in_month`](#cc_core-sun-days_in_month) · [`SunPosition`](#cc_core-sun-sunposition) · [`Daylight`](#cc_core-sun-daylight) · [`solar_position`](#cc_core-sun-solar_position) · [`solar_noon`](#cc_core-sun-solar_noon) · [`daylight`](#cc_core-sun-daylight) · [`day_path`](#cc_core-sun-day_path) · [`sun_direction`](#cc_core-sun-sun_direction) · [`SunSite`](#cc_core-sun-sunsite) · [`SunInterval`](#cc_core-sun-suninterval) · [`SunMode`](#cc_core-sun-sunmode) · [`SunFrame`](#cc_core-sun-sunframe) · [`SunSettings`](#cc_core-sun-sunsettings) · [`SunState`](#cc_core-sun-sunstate) · [`view_forward`](#cc_core-sun-view_forward) · [`camera_forward`](#cc_core-sun-camera_forward) · [`SunPreset`](#cc_core-sun-sunpreset) · [`clean_preset_name`](#cc_core-sun-clean_preset_name) · [`repair_presets`](#cc_core-sun-repair_presets)

<a id="cc_core-sun-max_minute"></a>

### const `MAX_MINUTE`

```rust
pub const MAX_MINUTE: u16
```

The last minute of a day (23:59).

<a id="cc_core-sun-max_frames"></a>

### const `MAX_FRAMES`

```rust
pub const MAX_FRAMES: usize
```

Most frames a study yields; longer ones are cut.

<a id="cc_core-sun-max_days"></a>

### const `MAX_DAYS`

```rust
pub const MAX_DAYS: usize
```

Most days a study walks through (a bound for studies spanning polar nights).

<a id="cc_core-sun-max_presets"></a>

### const `MAX_PRESETS`

```rust
pub const MAX_PRESETS: usize
```

Most presets a project keeps.

<a id="cc_core-sun-max_preset_name"></a>

### const `MAX_PRESET_NAME`

```rust
pub const MAX_PRESET_NAME: usize
```

Longest preset name, characters.

<a id="cc_core-sun-min_year"></a>

### const `MIN_YEAR`

```rust
pub const MIN_YEAR: i32
```

Earliest and latest year of a `SunDate`.

<a id="cc_core-sun-max_year"></a>

### const `MAX_YEAR`

```rust
pub const MAX_YEAR: i32
```

<a id="cc_core-sun-utc_offset_range"></a>

### const `UTC_OFFSET_RANGE`

```rust
pub const UTC_OFFSET_RANGE: std::ops::RangeInclusive<f64>
```

Accepted offsets from UTC, hours.

<a id="cc_core-sun-horizon_altitude"></a>

### const `HORIZON_ALTITUDE`

```rust
pub const HORIZON_ALTITUDE: f64
```

Altitude of the sun's center, degrees, when its upper limb touches the
horizon (34' of refraction plus 16' of radius).

<a id="cc_core-sun-conventional_azimuth"></a>

### const `CONVENTIONAL_AZIMUTH`

```rust
pub const CONVENTIONAL_AZIMUTH: f64
```

The [conventional sun](SunMode::conventional)'s azimuth relative to the
view: behind the viewer, to the left (clockwise from the look direction).

<a id="cc_core-sun-conventional_altitude"></a>

### fn `conventional_altitude`

```rust
pub fn conventional_altitude() -> f64
```

The [conventional sun](SunMode::conventional)'s altitude, degrees: that of
a cube's diagonal, `atan(1 / √2)` (about 35.26°).

<a id="cc_core-sun-sundate"></a>

### struct `SunDate`

```rust
pub struct SunDate
```

A calendar date (proleptic Gregorian), local to the site.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Serialize`, `Deserialize`, `Default`, `std::fmt::Display`

**Fields**

- `year: i32`
- `month: u8`

  1 to 12.

- `day: u8`

  1 to the month's length.


**Methods**

- `pub fn new(year: i32, month: u8, day: u8) -> Option<SunDate>`

  The date, when it is a real one within `MIN_YEAR`..=`MAX_YEAR`.

- `pub fn is_valid(self) -> bool`

  A real date within `MIN_YEAR`..=`MAX_YEAR`.

- `pub fn clamped(self) -> SunDate`

  The nearest valid date: year, month and day each brought into range.

- `pub fn parse(text: &str) -> Option<SunDate>`

  Reads an ISO `YYYY-MM-DD` date (surrounding spaces ignored).

- `pub fn iso(self) -> String`

  The ISO `YYYY-MM-DD` form (the same as `to_string`).

- `pub fn days_in_year(self) -> u16`

  Days in the date's year: 365 or 366.

- `pub fn day_of_year(self) -> u16`

  The day of the year, 1 for January 1 (of the [clamped](Self::clamped) date).

- `pub fn to_days(self) -> i64`

  Days from 1970-01-01 (negative before it).

- `pub fn from_days(days: i64) -> Option<SunDate>`

  The date `days` after 1970-01-01, when its year is in range.

- `pub fn next_day(self) -> Option<SunDate>`

  The next day (`None` past `MAX_YEAR`).

- `pub fn add_days(self, n: i64) -> Option<SunDate>`

  The date `n` days later (earlier when negative), when in range.

- `pub fn add_weeks(self, n: i64) -> Option<SunDate>`

  The date `n` weeks later, when in range.

- `pub fn add_months(self, n: i64) -> Option<SunDate>`

  The date `n` months later, its day cut to the new month's length
  (January 31 plus one month is February 28 or 29), when in range.


<a id="cc_core-sun-is_leap_year"></a>

### fn `is_leap_year`

```rust
pub fn is_leap_year(year: i32) -> bool
```

Whether `year` has a February 29.

<a id="cc_core-sun-days_in_month"></a>

### fn `days_in_month`

```rust
pub fn days_in_month(year: i32, month: u8) -> u8
```

Days in `month` (1 to 12) of `year`; 0 for a month out of range.

<a id="cc_core-sun-sunposition"></a>

### struct `SunPosition`

```rust
pub struct SunPosition
```

Where the sun stands in the sky, degrees.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`

**Fields**

- `azimuth: f64`

  Clockwise from true north seen from above (0 north, 90 east), in `[0, 360)`.

- `altitude: f64`

  Above the horizon (negative below it), in `[-90, 90]`; no refraction.


<a id="cc_core-sun-daylight"></a>

### enum `Daylight`

```rust
pub enum Daylight
```

When the sun rises and sets on a day.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Variants**

- `Normal { sunrise: f64, sunset: f64 }`

  Sunrise and sunset, minutes after local midnight (may fall outside
  `0..1440` when the time zone is far from the longitude).

- `PolarDay`

  The sun never sets.

- `PolarNight`

  The sun never rises.


<a id="cc_core-sun-solar_position"></a>

### fn `solar_position`

```rust
pub fn solar_position(latitude: f64, longitude: f64, utc_offset_hours: f64, date: SunDate, minutes: f64) -> SunPosition
```

The sun's position seen from `latitude`, `longitude` (degrees, WGS 84,
north and east positive) on `date` at `minutes` after local midnight,
local time being `utc_offset_hours` east of UTC. Azimuth clockwise from
true north, altitude without atmospheric refraction. Inputs that are not
numbers or out of range are brought into range; nothing returns NaN.

<a id="cc_core-sun-solar_noon"></a>

### fn `solar_noon`

```rust
pub fn solar_noon(longitude: f64, utc_offset_hours: f64, date: SunDate) -> f64
```

Local solar noon on `date` at `longitude`, minutes after local midnight.

<a id="cc_core-sun-daylight"></a>

### fn `daylight`

```rust
pub fn daylight(latitude: f64, longitude: f64, utc_offset_hours: f64, date: SunDate) -> Daylight
```

Sunrise and sunset on `date` at the site (see `Daylight`).

<a id="cc_core-sun-day_path"></a>

### fn `day_path`

```rust
pub fn day_path(site: &SunSite, date: SunDate, step_minutes: f64) -> Vec<(f64, SunPosition)>
```

The sun's path over `date` at the site, every `step_minutes` (1 to
1440; 15 when not a number) from 00:00 to 24:00, both included, for sun
path diagrams: callers keep the part above the horizon. At most 1441 points.

<a id="cc_core-sun-sun_direction"></a>

### fn `sun_direction`

```rust
pub fn sun_direction(north: DVec2, azimuth: f64, altitude: f64) -> DVec3
```

The unit vector from the model towards the sun, in project
coordinates, for a sun at `azimuth` (clockwise from true north) and
`altitude` degrees, true north being `north` in plan.

<a id="cc_core-sun-sunsite"></a>

### struct `SunSite`

```rust
pub struct SunSite
```

What the sun needs of the project's site.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`

**Fields**

- `latitude: f64`

  Degrees, north positive.

- `longitude: f64`

  Degrees, east positive.

- `utc_offset: f64`

  Local time's offset from UTC, hours east.

- `north: DVec2`

  True north in plan (unit vector).

- `known: bool`

  The project has a site location (otherwise the values above are
  the defaults: the equator at Greenwich, north up).


**Methods**

- `pub fn position(&self, date: SunDate, minutes: f64) -> SunPosition`

  The sun's position here on `date` at `minutes` after local midnight.

- `pub fn daylight(&self, date: SunDate) -> Daylight`

  Sunrise and sunset here on `date`.

- `pub fn north_dir(&self) -> DVec2`

  True north as a unit vector (+Y when not one).


<a id="cc_core-sun-suninterval"></a>

### enum `SunInterval`

```rust
pub enum SunInterval
```

The step between frames of a sun study.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Minutes15`
- `Minutes30`
- `Minutes45`
- `Hour`
- `Day`
- `Week`
- `Month`

**Methods**

- `pub const ALL: [SunInterval; 7]`
- `pub const SUB_DAY: [SunInterval; 4]`

  The steps within a day: the only ones a single-day study takes.

- `pub fn key(self) -> &'static str`

  Translation key of its name.

- `pub fn name(self) -> &'static str`

  The name commands use: `minutes_15`, `minutes_30`, `minutes_45`,
  `hour`, `day`, `week` or `month`.

- `pub fn from_name(name: &str) -> Option<SunInterval>`

  The interval called `name` (any case; spaces and `-` read as `_`).

- `pub fn minutes(self) -> Option<f64>`

  Minutes between frames for a step within a day; `None` for days, weeks and months.

- `pub fn is_sub_day(self) -> bool`

  A step within a day (what a single-day study accepts).


<a id="cc_core-sun-sunmode"></a>

### enum `SunMode`

```rust
pub enum SunMode
```

How a view places its sun.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`, `Default`

**Variants**

- `Still { #[serde(default)] date: SunDate, #[serde(default = "default_time")] time: u16, }`

  One moment.

- `SingleDay { #[serde(default)] date: SunDate, #[serde(default = "day_from")] from: u16, #[serde(default = "day_to")] to: u16, #[serde(default)] sunrise_to_sunset: bool, #[serde(default)] interval: SunInterval, }`

  Frames over one day, from `from` to `to` (or sunrise to sunset)
  every `interval` (a step within the day).

- `MultiDay { #[serde(default = "year_start")] from_date: SunDate, #[serde(default = "year_end")] to_date: SunDate, #[serde(default = "noon")] from: u16, #[serde(default = "noon")] to: u16, #[serde(default)] sunrise_to_sunset: bool, #[serde(default = "month_interval")] interval: SunInterval, }`

  Frames over several days: with a step within a day, every day from
  `from_date` to `to_date` as a single-day study; with a day, week or
  month, one frame per step at `from` (at 12:00 with `sunrise_to_sunset`).

- `Lighting { #[serde(default = "light_azimuth")] azimuth: f64, #[serde(default = "light_altitude")] altitude: f64, #[serde(default = "yes")] relative_to_view: bool, }`

  A light from a direction given by hand, with no date.


**Methods**

- `pub const NAMES: [&'static str; 4]`

  The names commands use, in display order.

- `pub fn name(&self) -> &'static str`

  The name commands use: `still`, `single_day`, `multi_day` or `lighting`.

- `pub fn key(&self) -> &'static str`

  Translation key of its name.

- `pub fn from_name(name: &str) -> Option<SunMode>`

  A mode of the kind called `name` (any case; spaces and `-` read as
  `_`) with its default values.

- `pub fn conventional() -> SunMode`

  The conventional sun of façade drawings: a light relative to the
  view whose rays run along the diagonal of a cube, coming from in
  front of the view, over the viewer's left shoulder (`CONVENTIONAL_AZIMUTH`
  from the look direction, `conventional_altitude` high), so a
  shadow falls down and to the right of what casts it, as far down
  and as far across as its depth in front of the face that catches it.

- `pub fn is_conventional(&self) -> bool`

  Whether this is the [conventional](Self::conventional) sun (within
  a thousandth of a degree).

- `pub fn interval(&self) -> Option<SunInterval>`

  The step of a study (`None` for a still sun or lighting).

- `pub fn is_dated(&self) -> bool`

  Whether the mode places the sun by date and time (all but lighting).

- `pub fn is_valid(&self) -> bool`

  Whether the mode's values are all in range (see `Self::repair`).

- `pub fn repair(&mut self) -> usize`

  Bring every value into range: dates valid and in order, times
  within the day and `from <= to`, a single day's step within the
  day (else hourly), a light's azimuth in `[0, 360)` and altitude in
  `0..=90` (defaults when not numbers). Returns how many it fixed.


<a id="cc_core-sun-sunframe"></a>

### struct `SunFrame`

```rust
pub struct SunFrame
```

One moment of a sun study.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `date: SunDate`
- `minutes: f64`

  Minutes after local midnight.


<a id="cc_core-sun-sunsettings"></a>

### struct `SunSettings`

```rust
pub struct SunSettings
```

A view's sun: how it is placed, the frame of a study shown, and what is drawn.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `mode: SunMode`
- `frame: u32`

  The frame of a study shown (clamped to the frames when read).

- `show_path: bool`

  Draw the sun path diagram.

- `shadows: bool`

  Cast shadows: in 3D views, and in plans, elevations and sections
  drawn in the shaded, consistent colors or textures style.

- `ground: Option<LevelId>`

  The level whose plane receives the shadows (`None`: the lowest level,
  as the rendered image stands the model on it).


**Methods**

- `pub fn default_for(kind: &ViewKind) -> SunSettings`

  The sun a view of `kind` starts with when it is given one: the
  [conventional sun](SunMode::conventional) casting shadows for
  elevations and sections (the façade drawing's convention), the
  [default](Self::default) for the others.

- `pub fn frames(&self, site: &SunSite) -> Vec<SunFrame>`

  The frames of the study at `site`, in time order, at most
  `MAX_FRAMES`: one for a still sun; a single day from `from` to
  `to` (or sunrise to sunset) every interval, the end included even
  off the step (a polar night falls back to one frame at 12:00);
  several days as described at `SunMode::MultiDay` (dates stepped
  from `from_date` up to `to_date`, which is not added when off the
  step; no frames at all fall back to 12:00 on `from_date`); none for
  lighting.

- `pub fn current_frame(&self, site: &SunSite) -> Option<SunFrame>`

  The frame shown: `Self::frame` clamped to the frames (`None` for lighting).

- `pub fn is_valid(&self) -> bool`

  Whether every value is in range (see `SunMode::repair`).

- `pub fn repair(&mut self) -> usize`

  Bring the mode's values into range; returns how many it fixed. (A
  missing ground level is the project's to check.)

- `pub fn state(&self, site: &SunSite, view_forward: Option<DVec2>) -> SunState`

  Where the sun stands for these settings at `site`. Dated modes take
  the current frame's solar position. Lighting takes its azimuth from
  true north, or, `relative_to_view`, clockwise (seen from above) from
  `view_forward`, the plan direction the viewer looks along (0: the
  sun ahead of the viewer, 135: behind and to the right), which is
  project +Y when `None` or not a direction.

- `pub fn follows_view(&self) -> bool`

  Whether the sun is placed relative to the direction the viewer
  looks along (lighting `relative_to_view`): it turns with the view.


<a id="cc_core-sun-sunstate"></a>

### struct `SunState`

```rust
pub struct SunState
```

Where a view's sun stands, resolved.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `azimuth: f64`

  Degrees clockwise from true north, in `[0, 360)`.

- `altitude: f64`

  Degrees above the horizon.

- `direction: DVec3`

  Unit vector from the model towards the sun, project coordinates.

- `frame: Option<SunFrame>`

  The study frame shown (`None` for lighting).

- `above_horizon: bool`

  The sun is above the horizon (it lights and casts shadows).


<a id="cc_core-sun-view_forward"></a>

### fn `view_forward`

```rust
pub fn view_forward(view: &View) -> Option<DVec2>
```

The plan direction a view looks along, for a sun placed relative to
it: a 3D view's camera heading, an elevation's or section's look
direction; `None` for plans and other views.

<a id="cc_core-sun-camera_forward"></a>

### fn `camera_forward`

```rust
pub fn camera_forward(camera: &crate::CameraState) -> Option<DVec2>
```

The plan direction a 3D camera looks along (its heading); `None` when
its yaw is not a number. A live camera the viewer turns places a sun
relative to the view as the stored one does in `view_forward`.

<a id="cc_core-sun-sunpreset"></a>

### struct `SunPreset`

```rust
pub struct SunPreset
```

A named sun setting the views can take.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `name: String`
- `mode: SunMode`

<a id="cc_core-sun-clean_preset_name"></a>

### fn `clean_preset_name`

```rust
pub fn clean_preset_name(name: &str) -> Option<String>
```

A preset name: trimmed, cut to `MAX_PRESET_NAME` characters; `None` when empty.

<a id="cc_core-sun-repair_presets"></a>

### fn `repair_presets`

```rust
pub fn repair_presets(presets: &mut Vec<SunPreset>) -> usize
```

Clean names, drop presets with no name or a name already taken, past
`MAX_PRESETS`, and repair their modes; returns how many it fixed.

<a id="mod-cc_core-sweep"></a>

## `cc_core::sweep`

<sub>`crates/cc_core/src/sweep.rs`</sub>

Sweeps and blends of component pieces.
A sweep carries a closed profile along an open path drawn on one of the
sketch planes: the profile's `u` runs to the left of the path (in the
path's plane) and its `v` along the plane's normal, and at each corner
of the path the section is mitered (it lies in the plane that bisects
the corner). A blend joins two closed outlines drawn on the same sketch
plane at two positions along its normal (a bottom and a top), the faces
between them ruled from points spaced evenly around each outline.
Both come out as closed meshes with outward faces, in the component's
frame.

**Items:** [`path`](#cc_core-sweep-path) · [`sweep_closed`](#cc_core-sweep-sweep_closed) · [`sweep`](#cc_core-sweep-sweep) · [`counter_clockwise`](#cc_core-sweep-counter_clockwise) · [`blend_section`](#cc_core-sweep-blend_section) · [`blend`](#cc_core-sweep-blend) · [`volume`](#cc_core-sweep-volume) · [`eval_field`](#cc_core-sweep-eval_field)

<a id="cc_core-sweep-path"></a>

### fn `path`

```rust
pub fn path(points: &[ProfilePoint], lookup: &dyn Fn(&str) -> Option<f64>) -> Result<Vec<DVec2>, ProfileError>
```

An open path of lines, arcs and parts of ellipses (a point's `radius`
makes the segment to the next point an arc, positive bulging to the
right of travel; its `ellipse` a part of that ellipse).

<a id="cc_core-sweep-sweep_closed"></a>

### fn `sweep_closed`

```rust
pub fn sweep_closed(plane: SketchPlane, path: &[DVec2], offset: f64, profile: &[DVec2]) -> Result<Vec<Vec<DVec3>>, ProfileError>
```

The solid of a counter-clockwise `profile` carried around the closed
`path` (on `plane`, placed `offset` along its normal): mitered at every
corner, the last point joined back to the first, with no end caps.

<a id="cc_core-sweep-sweep"></a>

### fn `sweep`

```rust
pub fn sweep(plane: SketchPlane, path: &[DVec2], offset: f64, profile: &[DVec2]) -> Result<Vec<Vec<DVec3>>, ProfileError>
```

The solid of a counter-clockwise `profile` carried along `path` (on
`plane`, placed `offset` along its normal), mitered at the corners.

<a id="cc_core-sweep-counter_clockwise"></a>

### fn `counter_clockwise`

```rust
pub fn counter_clockwise(outline: &[DVec2]) -> Vec<DVec2>
```

`outline` in counter-clockwise order.

<a id="cc_core-sweep-blend_section"></a>

### fn `blend_section`

```rust
pub fn blend_section(bottom: &[DVec2], top: &[DVec2], t: f64) -> Vec<DVec2>
```

The section of `blend` between counter-clockwise `bottom` and `top` at
`t` (0 at the bottom, 1 at the top): its outlines resampled alike and
joined point by point, as the blend's faces are.

<a id="cc_core-sweep-blend"></a>

### fn `blend`

```rust
pub fn blend(plane: SketchPlane, bottom: &[DVec2], top: &[DVec2], offset: f64, depth: f64) -> Result<Vec<Vec<DVec3>>, ProfileError>
```

The solid between a counter-clockwise `bottom` outline on `plane` at
`offset` and a `top` outline `depth` farther along its normal.

<a id="cc_core-sweep-volume"></a>

### fn `volume`

```rust
pub fn volume(faces: &[Vec<DVec3>]) -> f64
```

Signed volume of a closed mesh (for tests and checks).

<a id="cc_core-sweep-eval_field"></a>

### fn `eval_field`

```rust
pub fn eval_field(field: &'static str, text: &str, lookup: &dyn Fn(&str) -> Option<f64>) -> Result<f64, ProfileError>
```

Evaluate an expression of a piece, naming the field on failure.

<a id="mod-cc_core-template_spec"></a>

## `cc_core::template_spec`

<sub>`crates/cc_core/src/template_spec.rs`</sub>

The sizes and names that differ between the metric and the imperial
templates. `crate::Project::from_template_system` builds both from the
same code and reads only these tables, so the two templates have the same
types, views and ids, in the same order.
Lengths are meters, except annotation sizes (paper millimeters); imperial
values are written in inches or feet and kept exact multiples of 1/16".

**Items:** [`Sized1`](#cc_core-template_spec-sized1) · [`Sized2`](#cc_core-template_spec-sized2) · [`Sized3`](#cc_core-template_spec-sized3) · [`SizedLibraryType`](#cc_core-template_spec-sizedlibrarytype) · [`TemplateSpec`](#cc_core-template_spec-templatespec) · [`Defaults`](#cc_core-template_spec-defaults)

<a id="cc_core-template_spec-sized1"></a>

### type alias `Sized1`

```rust
pub(crate) type Sized1 = (&'static str, f64)
```

A name key and a value.

<a id="cc_core-template_spec-sized2"></a>

### type alias `Sized2`

```rust
pub(crate) type Sized2 = (&'static str, f64, f64)
```

A name key and two values.

<a id="cc_core-template_spec-sized3"></a>

### type alias `Sized3`

```rust
pub(crate) type Sized3 = (&'static str, f64, f64, f64)
```

A name key and three values.

<a id="cc_core-template_spec-sizedlibrarytype"></a>

### type alias `SizedLibraryType`

```rust
pub(crate) type SizedLibraryType = (&'static str, &'static str, &'static [Sized1])
```

A library type whose name states a size: the library's name key, the
key used instead, and the parameters (keys) set to that size.

<a id="cc_core-template_spec-templatespec"></a>

### struct `TemplateSpec`

```rust
pub(crate) struct TemplateSpec
```

**Fields**

- `system: MeasurementSystem`

  The system the template measures in.

- `first_level: f64`

  Elevation of the upper level.

- `masonry: [Sized2; 2]`

  The two masonry walls: key, plaster on each side, masonry core.

- `partition: Sized1`

  The gypsum partition: key, thickness.

- `doors: [Sized2; 2]`

  The swing doors: key, width, height.

- `windows: [Sized3; 2]`

  The fixed windows: key, width, height, sill.

- `operable: [Sized3; 4]`

  The operable windows (side-hung, double side-hung, top-hung,
  sliding): key, width, height, sill.

- `slab: Sized2`

  The floor slab: key, screed, concrete.

- `ceiling: f64`

  Thickness of the gypsum ceiling.

- `column_rect: Sized1`

  Square concrete column: key, side.

- `column_round: Sized1`

  Round concrete column: key, diameter.

- `beam: Sized2`

  Concrete beam: key, width, height.

- `column_steel_h: Sized3`

  Steel H column: key, width and depth, web, flange.

- `column_steel_tube: Sized2`

  Steel square tube column: key, side, wall.

- `beam_steel_i: (&'static str, [f64; 4])`

  Steel I beam: key, (width, height, web, flange).

- `rebars: [Sized1; 5]`

  Reinforcing bars: key, diameter.

- `footings: [Sized3; 3]`

  Pad, strip and slab footings: key, width, length, thickness.

- `stair: Sized3`

  Stair: key, width, maximum riser, tread.

- `railing: Sized2`

  Railing: key, height, post spacing.

- `pipes: [Sized1; 7]`

  Supply pipes, then drain pipes: key, diameter.

- `conduits: [Sized1; 3]`

  Conduits: key, diameter.

- `grid: Sized1`

  Grid: key, bubble size (paper mm).

- `dimension: Sized1`

  Linear dimension: key, text size (paper mm).

- `texts: [Sized1; 2]`

  Texts: key, size (paper mm).

- `roof: (f64, f64)`

  Roof layers: tiles, wood.

- `french_doors: [Sized2; 2]`

  French doors (one leaf, two leaves): key, width, height.

- `plan_scale: u32`

  Scale of plans, ceiling plans and the legend.

- `model_scale: u32`

  Scale of the 3D view and the elevations.

- `sheet: (SheetSize, [DVec2; 2])`

  The sheet's paper and where the plan and the 3D view sit on it.

- `library: &'static [SizedLibraryType]`

  Library types whose name states a size: the library's key, the key
  used instead, and the parameters (keys) set to that size.


**Methods**

- `pub(crate) fn of(system: MeasurementSystem) -> &'static TemplateSpec`
- `pub(crate) fn library_key<'a>(&self, key: &'a str) -> &'a str`

  The key a library name is read from in this system.

- `pub(crate) fn sized_library(&self, n: &dyn Fn(&str) -> String, mut library: Vec<(String, String, crate::ComponentDef)>) -> Vec<(String, String, crate::ComponentDef)>`

  `library` (component, type name, definition, as the component
  library makes it with names from `n`) with the types whose name
  states a size given that size in this system; in imperial projects
  the other sizes rounded to whole inches (feet and half feet above a
  foot, see `crate::units::round_imperial`).


<a id="cc_core-template_spec-defaults"></a>

### struct `Defaults`

```rust
pub struct Defaults
```

Sizes and rules that commands and tools give what they create outside
the template, per measurement system (meters; see `Defaults::of` and
`crate::Project::defaults`). The metric values are the ones these
places always used; the imperial ones are round customary sizes.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `story: f64`

  Story height when there is no level above (walls, columns, stairs,
  ramps, masses, shafts, roofs).

- `ceiling: f64`

  Ceiling height when there is no level above (conduits, cable trays,
  ceiling lights, air terminals).

- `ceiling_offset: f64`

  Height of new ceilings above their level.

- `ramp: (&'static str, f64, f64)`

  The ramp type made with the first ramp: name key, width, slab.

- `ramp_rules: RampRules`

  The rules of new ramp types.

- `stair_rules: StairRules`

  The rules of the template's stair type.

- `flights: (f64, f64, f64)`

  New stairs and ramps: the gap between U-shaped flights, the inner
  radius of a spiral stair and of a spiral ramp.

- `baseboard: (&'static str, f64, f64)`

  The baseboard made with the first sweep: name key, thickness, height.

- `tray: (f64, f64)`

  The cable tray type made with the first tray: width, height.

- `tray_clearance: f64`

  How far a new cable tray runs under the level above.

- `conductor: (&'static str, f64)`

  The conductor type made with the first conductor: name key (with a
  `diameter` placeholder), diameter.

- `door: [f64; 5]`

  New door types: width of one leaf, of two leaves, height, French
  door height, French door bottom rail.

- `pipes: (f64, f64)`

  Elevation of new supply pipes and of new drains above their level.

- `drain_fall: (f64, f64, f64)`

  Least fall of a horizontal drain: up to the first diameter the
  second slope, larger ones the third.

- `devices: [f64; 6]`

  Mounting height of new outlets, switches, wall lights, boards, data
  outlets and racks.

- `terrain: (f64, f64)`

  New terrain types: contour interval; new terrains: base depth.

- `curtain: (f64, f64, f64, f64)`

  New curtain wall types: grid spacing, mullion width and depth, glass.

- `rebar: [f64; 5]`

  Rebar: cover, stirrup spacing, and the diameters of the bar types
  sought for main bars, top bars and stirrups.


**Methods**

- `pub fn of(system: MeasurementSystem) -> &'static Defaults`

<a id="mod-cc_core-title_block"></a>

## `cc_core::title_block`

<sub>`crates/cc_core/src/title_block.rs`</sub>

Title blocks: components of kind `ComponentKind::TitleBlock` whose 2D
pieces (`ComponentDef::sheet`) draw a whole sheet (border, title
block, logo) in paper millimeters, origin at the paper's bottom-left
corner. Expressions read the parameters and `SHEET_WIDTH` and
`SHEET_HEIGHT`; texts keep `{fields}` for the sheet to fill in.

**Items:** [`SHEET_WIDTH`](#cc_core-title_block-sheet_width) · [`SHEET_HEIGHT`](#cc_core-title_block-sheet_height) · [`MAX_SHEET_PIECES`](#cc_core-title_block-max_sheet_pieces) · [`MAX_STROKE_POINTS`](#cc_core-title_block-max_stroke_points) · [`MAX_SHEET_TEXT`](#cc_core-title_block-max_sheet_text) · [`MAX_SHEET_POINTS`](#cc_core-title_block-max_sheet_points) · [`DEFAULT_WEIGHT`](#cc_core-title_block-default_weight) · [`MAX_TEXT_SIZE`](#cc_core-title_block-max_text_size) · [`STANDARD_KEY`](#cc_core-title_block-standard_key) · [`standard`](#cc_core-title_block-standard) · [`SheetItem`](#cc_core-title_block-sheetitem) · [`TextAnchor`](#cc_core-title_block-textanchor) · [`SheetShape`](#cc_core-title_block-sheetshape) · [`MAX_NAME`](#cc_core-title_block-max_name) · [`SheetStroke`](#cc_core-title_block-sheetstroke) · [`SheetText`](#cc_core-title_block-sheettext) · [`SheetImage`](#cc_core-title_block-sheetimage) · [`SheetPieces`](#cc_core-title_block-sheetpieces) · [`SheetNested`](#cc_core-title_block-sheetnested) · [`repair_def`](#cc_core-title_block-repair_def) · [`MAX_FIELD_KEY`](#cc_core-title_block-max_field_key) · [`PROJECT_FIELD_PREFIX`](#cc_core-title_block-project_field_prefix) · [`project_field_name_ok`](#cc_core-title_block-project_field_name_ok) · [`FieldPart`](#cc_core-title_block-fieldpart) · [`field_parts`](#cc_core-title_block-field_parts) · [`rename_field_param`](#cc_core-title_block-rename_field_param) · [`is_iso_date`](#cc_core-title_block-is_iso_date) · [`parse_iso_date`](#cc_core-title_block-parse_iso_date)

<a id="cc_core-title_block-sheet_width"></a>

### const `SHEET_WIDTH`

```rust
pub const SHEET_WIDTH: &str
```

Name of the paper width (millimeters) in expressions.

<a id="cc_core-title_block-sheet_height"></a>

### const `SHEET_HEIGHT`

```rust
pub const SHEET_HEIGHT: &str
```

Name of the paper height (millimeters) in expressions.

<a id="cc_core-title_block-max_sheet_pieces"></a>

### const `MAX_SHEET_PIECES`

```rust
pub const MAX_SHEET_PIECES: usize
```

<a id="cc_core-title_block-max_stroke_points"></a>

### const `MAX_STROKE_POINTS`

```rust
pub const MAX_STROKE_POINTS: usize
```

<a id="cc_core-title_block-max_sheet_text"></a>

### const `MAX_SHEET_TEXT`

```rust
pub const MAX_SHEET_TEXT: usize
```

<a id="cc_core-title_block-max_sheet_points"></a>

### const `MAX_SHEET_POINTS`

```rust
pub const MAX_SHEET_POINTS: usize
```

Most points a title block's strokes may have in all, once arcs are flattened.

<a id="cc_core-title_block-default_weight"></a>

### const `DEFAULT_WEIGHT`

```rust
pub const DEFAULT_WEIGHT: f64
```

Pen width of a stroke without one, millimeters.

<a id="cc_core-title_block-max_text_size"></a>

### const `MAX_TEXT_SIZE`

```rust
pub const MAX_TEXT_SIZE: f64
```

Largest text height in millimeters (larger sizes are drawn at this one).

<a id="cc_core-title_block-standard_key"></a>

### const `STANDARD_KEY`

```rust
pub const STANDARD_KEY: &str
```

Name key of the standard title block (component and type).

<a id="cc_core-title_block-standard"></a>

### fn `standard`

```rust
pub fn standard(n: &dyn Fn(&str) -> String) -> ComponentDef
```

The title block every sheet had before title blocks were components:
a border 10 mm in, and a 170 × 42 mm block in the bottom-right corner
with the project, sheet and client rows and the sheet number cell.
Captions are looked up by `n` (template keys `tpl-tb-*`).

<a id="cc_core-title_block-sheetitem"></a>

### struct `SheetItem`

```rust
pub struct SheetItem
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `name: String`
- `shape: SheetShape`

<a id="cc_core-title_block-textanchor"></a>

### enum `TextAnchor`

```rust
pub enum TextAnchor
```

Where a text's position sits: on the baseline at its start, middle or
end, or at its center.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Left`
- `Center`
- `Right`
- `Middle`

<a id="cc_core-title_block-sheetshape"></a>

### enum `SheetShape`

```rust
pub enum SheetShape
```

A 2D piece; every `String` is an expression in millimeters (degrees for rotations).

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Variants**

- `Stroke { points: Vec<ProfilePoint>, #[serde(default)] closed: bool, #[serde(default, skip_serializing_if = "String::is_empty")] weight: String, #[serde(default, skip_serializing_if = "Option::is_none")] color: Option<[u8; 3]>, #[serde(default, skip_serializing_if = "Option::is_none")] fill: Option<[u8; 3]>, #[serde(default, skip_serializing_if = "is_solid")] pattern: crate::LinePattern, #[serde(default, skip_serializing_if = "Option::is_none")] hatch: Option<crate::PatternFill>, }`

  Lines and arcs: `points` run straight, or along an arc (`radius`) to the next.

- `Text { x: String, y: String, text: String, size: String, #[serde(default)] anchor: TextAnchor, #[serde(default, skip_serializing_if = "String::is_empty")] rotation: String, }`

  Fixed words and `{fields}`; line breaks start new lines.

- `Image { x: String, y: String, width: String, height: String, image: Arc<RasterImage> }`

  A picture filling the rectangle from (`x`, `y`) (its bottom-left corner).

- `Nested { component: String, x: String, y: String, #[serde(default, skip_serializing_if = "String::is_empty")] rotation: String, #[serde(default, skip_serializing_if = "String::is_empty")] scale: String, #[serde(default, skip_serializing_if = "std::ops::Not::not")] mirrored: bool, }`

  Another detail component placed inside this one (detail components
  only): the component named `component` (with the parameter values
  of its first type), its origin at (`x`, `y`), turned `rotation`
  degrees counter-clockwise, sized by `scale` (empty for 1) and, when
  `mirrored`, flipped across its own X axis first.


**Methods**

- `pub fn expressions_mut(&mut self) -> Vec<&mut String>`

  Every expression of the piece, for renaming parameters.

- `pub fn stroke(points: Vec<ProfilePoint>, closed: bool) -> SheetShape`

  A stroke through `points` with the default pen.


<a id="cc_core-title_block-max_name"></a>

### const `MAX_NAME`

```rust
pub const MAX_NAME: usize
```

Most characters of a nested component's name.

<a id="cc_core-title_block-sheetstroke"></a>

### struct `SheetStroke`

```rust
pub struct SheetStroke
```

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `item: usize`
- `points: Vec<DVec2>`
- `closed: bool`
- `weight: f64`
- `color: [u8; 3]`
- `fill: Option<[u8; 3]>`
- `pattern: crate::LinePattern`
- `hatch: Option<crate::PatternFill>`

<a id="cc_core-title_block-sheettext"></a>

### struct `SheetText`

```rust
pub struct SheetText
```

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `item: usize`
- `position: DVec2`
- `text: String`

  As written: `{fields}` are filled in by the sheet.

- `size: f64`
- `anchor: TextAnchor`
- `rotation_deg: f64`

<a id="cc_core-title_block-sheetimage"></a>

### struct `SheetImage`

```rust
pub struct SheetImage
```

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `item: usize`
- `corner: DVec2`

  Bottom-left corner.

- `size: DVec2`
- `image: Arc<RasterImage>`

<a id="cc_core-title_block-sheetpieces"></a>

### struct `SheetPieces`

```rust
pub struct SheetPieces
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `strokes: Vec<SheetStroke>`
- `texts: Vec<SheetText>`
- `images: Vec<SheetImage>`
- `nested: Vec<SheetNested>`

  Nested detail components (detail components only).

- `errors: Vec<(usize, PartError)>`

<a id="cc_core-title_block-sheetnested"></a>

### struct `SheetNested`

```rust
pub struct SheetNested
```

A nested detail component, resolved: where its frame goes.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `item: usize`
- `component: String`
- `position: DVec2`
- `rotation_deg: f64`
- `scale: f64`
- `mirrored: bool`

<a id="cc_core-title_block-repair_def"></a>

### fn `repair_def`

```rust
pub fn repair_def(def: &mut ComponentDef) -> usize
```

Keep a definition's pieces to its kind and within the limits: title
blocks keep only sheet pieces, other kinds none. Returns the fixes.

<a id="cc_core-title_block-max_field_key"></a>

### const `MAX_FIELD_KEY`

```rust
pub const MAX_FIELD_KEY: usize
```

Most bytes a field key may have (a longer `{…}` is not a field).

<a id="cc_core-title_block-project_field_prefix"></a>

### const `PROJECT_FIELD_PREFIX`

```rust
pub const PROJECT_FIELD_PREFIX: &str
```

What a project field's key starts with (`{project:Name}`).

<a id="cc_core-title_block-project_field_name_ok"></a>

### fn `project_field_name_ok`

```rust
pub fn project_field_name_ok(name: &str) -> bool
```

Whether `name` can name a project field a text reads as
`{project:Name}`: not blank, without braces, and short enough for the
key to stay within `MAX_FIELD_KEY`.

<a id="cc_core-title_block-fieldpart"></a>

### enum `FieldPart`

```rust
pub enum FieldPart<'a>
```

A run of a title block text, as `field_parts` splits it.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Variants**

- `Text { raw: &'a str, shown: &'a str }`

  Words as written (`raw`) and as shown (`shown`: `{{` and `}}` as one brace).

- `Field { key: &'a str, raw: &'a str }`

  `{key}`: the key between the braces and the whole field as written.


<a id="cc_core-title_block-field_parts"></a>

### fn `field_parts`

```rust
pub fn field_parts<'a>(text: &'a str, mut f: impl FnMut(FieldPart<'a>))
```

Splits `text` into words and `{key}` fields, in order: `{{` and `}}`
are literal braces, and a `{` closed by no `}` within `MAX_FIELD_KEY`
bytes (before another `{`) is a brace like any other character.

<a id="cc_core-title_block-rename_field_param"></a>

### fn `rename_field_param`

```rust
pub fn rename_field_param(text: &str, old: &str, new: &str) -> String
```

`text` with every `{param:old}` field reading `{param:new}`.

<a id="cc_core-title_block-is_iso_date"></a>

### fn `is_iso_date`

```rust
pub fn is_iso_date(s: &str) -> bool
```

Whether `s` is a real `YYYY-MM-DD` date.

<a id="cc_core-title_block-parse_iso_date"></a>

### fn `parse_iso_date`

```rust
pub fn parse_iso_date(s: &str) -> Option<(i32, u32, u32)>
```

Year, month and day of a `YYYY-MM-DD` date: exactly four, two and two
ASCII digits.

<a id="mod-cc_core-topography"></a>

## `cc_core::topography`

<sub>`crates/cc_core/src/topography.rs`</sub>

Terrain: a topography surface from measured points (x, y, z) inside a
closed boundary drawn in plan. The geometry (`cc_geom::topography`)
triangulates the points with the boundary, whose vertices take the
elevation the points give them, and closes the surface down to a flat base
`base_depth` below its lowest point, so sections cut the earth as a solid.
Points outside the boundary are kept: they give the boundary its
elevations but are not vertices of the surface.

**Items:** [`MAX_POINTS`](#cc_core-topography-max_points) · [`MAX_OUTLINE`](#cc_core-topography-max_outline) · [`POINT_TOL`](#cc_core-topography-point_tol) · [`DEFAULT_BASE_DEPTH`](#cc_core-topography-default_base_depth) · [`DEFAULT_CONTOUR_INTERVAL`](#cc_core-topography-default_contour_interval) · [`DEFAULT_MAJOR_EVERY`](#cc_core-topography-default_major_every) · [`MIN_CONTOUR_INTERVAL`](#cc_core-topography-min_contour_interval) · [`EARTH_COLOR`](#cc_core-topography-earth_color) · [`earth_material`](#cc_core-topography-earth_material) · [`Topography`](#cc_core-topography-topography) · [`clean_points`](#cc_core-topography-clean_points)

<a id="cc_core-topography-max_points"></a>

### const `MAX_POINTS`

```rust
pub const MAX_POINTS: usize
```

Most measured points a topography keeps.

<a id="cc_core-topography-max_outline"></a>

### const `MAX_OUTLINE`

```rust
pub const MAX_OUTLINE: usize
```

Most points of a topography's boundary.

<a id="cc_core-topography-point_tol"></a>

### const `POINT_TOL`

```rust
pub const POINT_TOL: f64
```

Points closer than this in plan are one point (the first is kept).

<a id="cc_core-topography-default_base_depth"></a>

### const `DEFAULT_BASE_DEPTH`

```rust
pub const DEFAULT_BASE_DEPTH: f64
```

Depth of the base below the lowest point, meters, unless given.

<a id="cc_core-topography-default_contour_interval"></a>

### const `DEFAULT_CONTOUR_INTERVAL`

```rust
pub const DEFAULT_CONTOUR_INTERVAL: f64
```

Spacing of the contour lines of a new terrain type, meters.

<a id="cc_core-topography-default_major_every"></a>

### const `DEFAULT_MAJOR_EVERY`

```rust
pub const DEFAULT_MAJOR_EVERY: u32
```

Every this many contours of a new terrain type, a major one.

<a id="cc_core-topography-min_contour_interval"></a>

### const `MIN_CONTOUR_INTERVAL`

```rust
pub const MIN_CONTOUR_INTERVAL: f64
```

Smallest contour interval a terrain type keeps, meters.

<a id="cc_core-topography-earth_color"></a>

### const `EARTH_COLOR`

```rust
pub const EARTH_COLOR: [u8; 3]
```

Color of an earth material added for terrain.

<a id="cc_core-topography-earth_material"></a>

### fn `earth_material`

```rust
pub fn earth_material(p: &crate::Project) -> Option<&str>
```

The project's earth material: the first of the earth class.

<a id="cc_core-topography-topography"></a>

### struct `Topography`

```rust
pub struct Topography
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `points: Vec<DVec3>`

  Measured points; z is relative to the element's level elevation.

- `boundary: Vec<DVec2>`

  Closed boundary in plan (either winding).

- `base_depth: f64`

  Depth of the flat base below the lowest point of the surface.

- `sketch: Option<crate::sketch::Sketch>`

  The curves the boundary was drawn with (see `crate::sketch`).


**Methods**

- `pub fn is_valid(&self) -> bool`

  A boundary of three to `MAX_OUTLINE` finite points enclosing an area.
  Points and the base depth are repaired instead (see `Self::repair`).

- `pub fn repair(&mut self) -> usize`

  Drop non-finite points, points over `MAX_POINTS` and points that
  repeat another in plan, and reset a bad base depth; returns how many
  repairs were made.


<a id="cc_core-topography-clean_points"></a>

### fn `clean_points`

```rust
pub fn clean_points(points: &[DVec3]) -> Vec<DVec3>
```

The finite points of `points`, the first of those that coincide in plan
(within `POINT_TOL`), at most `MAX_POINTS`.

<a id="mod-cc_core-transform"></a>

## `cc_core::transform`

<sub>`crates/cc_core/src/transform.rs`</sub>

Rigid transforms in plan (move, rotate, mirror) applied to elements.

**Items:** [`PlanTransform`](#cc_core-transform-plantransform)

<a id="cc_core-transform-plantransform"></a>

### struct `PlanTransform`

```rust
pub struct PlanTransform
```

`p ↦ R·M·p + t`: an optional reflection across the X axis (`M`), a
rotation by `angle` radians (`R`) and a translation `t`.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`

**Fields**

- `angle: f64`
- `mirror: bool`
- `translation: DVec2`

**Methods**

- `pub const IDENTITY: Self`
- `pub fn translate(delta: DVec2) -> Self`
- `pub fn rotate_about(center: DVec2, angle: f64) -> Self`

  Rotation by `angle` radians about `center`.

- `pub fn mirror_across(a: DVec2, dir: DVec2) -> Self`

  Reflection across the line through `a` with direction `dir`.

- `pub fn then(self, next: Self) -> Self`

  `self` after `first` (apply `first`, then `self`).

- `pub fn apply_vector(&self, v: DVec2) -> DVec2`
- `pub fn apply(&self, p: DVec2) -> DVec2`
- `pub fn apply_angle_deg(&self, deg: f64) -> f64`

  A plan direction given in degrees, transformed.

- `pub fn is_identity(&self) -> bool`

<a id="mod-cc_core-truss"></a>

## `cc_core::truss`

<sub>`crates/cc_core/src/truss.rs`</sub>

Parametric trusses: a host element that holds the parameters of a plane
truss (its span line, shape, height, web pattern, panels, member types
and copies) and owns the ordinary beams (chords, diagonals) and columns
(verticals) made from them, kept in step like a steel building's (see
`crate::member_host`).
Members are laid out as they are built: each chord is one continuous
beam per straight run (a gable's top chord is two), verticals stand
between the chords where their sections let them seat (inside a channel
that opens toward the web, else on the chord's face; see `Sections`),
and where a truss ends on a vertical the chords run on over it and the
one at the far end is turned around, so both ends are closed.
Local coordinates: x along start → end (`L` long), the copies to the
left of it every `spacing`, heights above the level from `base_offset`
(the bottom chord's axis).

**Items:** [`MAX_MEMBERS`](#cc_core-truss-max_members) · [`MIN_VERTICAL`](#cc_core-truss-min_vertical) · [`LENGTH`](#cc_core-truss-length) · [`HEIGHT`](#cc_core-truss-height) · [`PANELS`](#cc_core-truss-panels) · [`COPIES`](#cc_core-truss-copies) · [`SPACING`](#cc_core-truss-spacing) · [`BASE_OFFSET`](#cc_core-truss-base_offset) · [`DEFAULT_LENGTH`](#cc_core-truss-default_length) · [`DEFAULT_HEIGHT`](#cc_core-truss-default_height) · [`DEFAULT_SPACING`](#cc_core-truss-default_spacing) · [`PANEL_LENGTH`](#cc_core-truss-panel_length) · [`TrussShape`](#cc_core-truss-trussshape) · [`TrussPattern`](#cc_core-truss-trusspattern) · [`TrussRoleKind`](#cc_core-truss-trussrolekind) · [`TrussRole`](#cc_core-truss-trussrole) · [`TrussMember`](#cc_core-truss-trussmember) · [`Truss`](#cc_core-truss-truss) · [`default_panels`](#cc_core-truss-default_panels) · [`host_type`](#cc_core-truss-host_type) · [`Sections`](#cc_core-truss-sections)

<a id="cc_core-truss-max_members"></a>

### const `MAX_MEMBERS`

```rust
pub const MAX_MEMBERS: usize
```

Most members one truss (all copies) may make; `Truss::check` refuses
a truss whose layout would make more.

<a id="cc_core-truss-min_vertical"></a>

### const `MIN_VERTICAL`

```rust
pub const MIN_VERTICAL: f64
```

A vertical shorter than this is skipped, and so is a diagonal whose ends
differ less in height (it would lie on the bottom chord), meters.

<a id="cc_core-truss-length"></a>

### const `LENGTH`

```rust
pub const LENGTH: RangeInclusive<f64>
```

<a id="cc_core-truss-height"></a>

### const `HEIGHT`

```rust
pub const HEIGHT: RangeInclusive<f64>
```

<a id="cc_core-truss-panels"></a>

### const `PANELS`

```rust
pub const PANELS: RangeInclusive<u32>
```

<a id="cc_core-truss-copies"></a>

### const `COPIES`

```rust
pub const COPIES: RangeInclusive<u32>
```

<a id="cc_core-truss-spacing"></a>

### const `SPACING`

```rust
pub const SPACING: RangeInclusive<f64>
```

<a id="cc_core-truss-base_offset"></a>

### const `BASE_OFFSET`

```rust
pub const BASE_OFFSET: RangeInclusive<f64>
```

The bottom chord's axis above the level, meters.

<a id="cc_core-truss-default_length"></a>

### const `DEFAULT_LENGTH`

```rust
pub const DEFAULT_LENGTH: f64
```

<a id="cc_core-truss-default_height"></a>

### const `DEFAULT_HEIGHT`

```rust
pub const DEFAULT_HEIGHT: f64
```

<a id="cc_core-truss-default_spacing"></a>

### const `DEFAULT_SPACING`

```rust
pub const DEFAULT_SPACING: f64
```

<a id="cc_core-truss-panel_length"></a>

### const `PANEL_LENGTH`

```rust
pub const PANEL_LENGTH: f64
```

The panel length the default panel count aims at, meters.

<a id="cc_core-truss-trussshape"></a>

### enum `TrussShape`

```rust
pub enum TrussShape
```

The outline of the truss: chords parallel, the top chord rising toward
the end (mono), or toward mid-span from both supports (gable).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Flat`
- `Mono`
- `Gable`

**Methods**

- `pub const ALL: [TrussShape; 3]`
- `pub fn code(self) -> &'static str`
- `pub fn from_code(code: &str) -> Option<Self>`
- `pub fn key(self) -> &'static str`

  The i18n key of its name.


<a id="cc_core-truss-trusspattern"></a>

### enum `TrussPattern`

```rust
pub enum TrussPattern
```

The web between the chords.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Pratt`
- `Howe`
- `Warren`
- `KingPost`
- `QueenPost`
- `Fink`

**Methods**

- `pub const ALL: [TrussPattern; 6]`
- `pub fn code(self) -> &'static str`
- `pub fn from_code(code: &str) -> Option<Self>`
- `pub fn key(self) -> &'static str`
- `pub fn needs_gable(self) -> bool`

  Whether it is made for a gable only.

- `pub fn min_panels(self) -> u32`

  The fewest panels it can be laid out on.


<a id="cc_core-truss-trussrolekind"></a>

### enum `TrussRoleKind`

```rust
pub enum TrussRoleKind
```

What a member is in its truss.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Serialize`, `Deserialize`

**Variants**

- `TopChord`
- `BottomChord`
- `Diagonal`
- `Vertical`

**Methods**

- `pub const ALL: [TrussRoleKind; 4]`

<a id="cc_core-truss-trussrole"></a>

### struct `TrussRole`

```rust
pub struct TrussRole
```

A member's place: its kind, its copy and its index there (chords: the
straight run, 0 or, past a gable's apex, 1; verticals: the panel point;
diagonals: their place in the pattern).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Serialize`, `Deserialize`

**Fields**

- `kind: TrussRoleKind`
- `copy: u16`
- `index: u16`

**Methods**

- `pub fn fits(self, kind: &ElementKind) -> bool`

  A column for verticals, a beam for the rest.

- `pub fn key(self) -> RoleKey`
- `pub fn from_key(key: RoleKey) -> Option<Self>`

<a id="cc_core-truss-trussmember"></a>

### struct `TrussMember`

```rust
pub struct TrussMember
```

A member the truss manages.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Serialize`, `Deserialize`

**Fields**

- `role: TrussRole`
- `id: ElementId`

<a id="cc_core-truss-truss"></a>

### struct `Truss`

```rust
pub struct Truss
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`, `MemberHost`

**Fields**

- `start: DVec2`

  The span line in plan, between the supports.

- `end: DVec2`
- `base_offset: f64`

  The bottom chord's axis above the level at the supports, meters.

- `shape: TrussShape`
- `height: f64`

  Chord axis to chord axis (flat), or the rise at the apex or high end, meters.

- `pattern: TrussPattern`
- `panels: u32`

  Bottom-chord panels (even on a gable).

- `top_chord_type: ElementTypeId`
- `bottom_chord_type: ElementTypeId`
- `web_type: ElementTypeId`
- `post_type: ElementTypeId`

  The column type of the verticals.

- `copies: u32`
- `spacing: f64`

  Copy to copy, to the left of start → end, meters.

- `members: Vec<TrussMember>`
- `detached: Vec<TrussRole>`

**Methods**

- `pub fn new(chord_type: ElementTypeId, web_type: ElementTypeId, post_type: ElementTypeId) -> Self`

  A flat Pratt truss 6 m long on the x axis, 1 m deep, at the level.

- `pub fn length(&self) -> f64`
- `pub fn direction(&self) -> DVec2`

  The unit vector along start → end (zero when they meet).

- `pub fn left(&self) -> DVec2`

  The unit vector to the left of start → end, where copies go.

- `pub fn rise(&self, x: f64) -> f64`

  The top chord's axis above the bottom chord's at distance `x` from the start.

- `pub fn pattern_used(&self) -> TrussPattern`

  The pattern it is laid out with: a gable-only pattern falls back to
  Pratt on another shape, or on fewer panels than it needs.

- `pub fn span_lines(&self) -> Vec<[DVec2; 2]>`

  Each copy's span line.

- `pub fn check(&self) -> Result<(), String>`

  Whether every number is in its range (the first one that is not, named).

- `pub fn members_per_copy(&self) -> usize`

  The most members one copy makes (a vertical that is too short once
  it is cut to the chords is left out of the layout).

- `pub fn check_types(&self, project: &Project) -> Result<(), String>`

  Whether the chord and web types are beam types and the post type a column type.

- `pub fn reset_out_of_range(&mut self) -> usize`

  Put every out-of-range number back to its default (an odd gable
  count up to the next even one); returns how many.

- `pub fn type_ids(&self) -> impl Iterator<Item = ElementTypeId> + '_`

  The member types it uses.

- `pub fn transform(&mut self, t: &PlanTransform)`

  Move, turn or mirror it. A mirror maps the whole row of copies: the
  new span line is the mirrored last copy's, so the copies stay where
  the mirror puts them and a mono truss keeps its high end.

- `pub fn layout(&self, s: &Sections) -> Vec<(TrussRole, ElementTypeId, ElementKind)>`

  Its members: role, type and element, copy by copy (chords, verticals,
  diagonals), for member sections `s`. Empty when a number is out of
  range or the members would be more than `MAX_MEMBERS` (see
  `Truss::check`).


<a id="cc_core-truss-default_panels"></a>

### fn `default_panels`

```rust
pub fn default_panels(length: f64, shape: TrussShape) -> u32
```

About one panel per `PANEL_LENGTH`, 2 to 64, even on a gable.

<a id="cc_core-truss-host_type"></a>

### fn `host_type`

```rust
pub fn host_type(id: ElementTypeId, name: String) -> ElementType
```

The type a project gets with its first truss.

<a id="cc_core-truss-sections"></a>

### struct `Sections`

```rust
pub struct Sections
```

What a layout needs to know of the member types' sections, in meters.
The default (all zero) lays members out axis to axis.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`

**Fields**

- `bottom_seat: f64`

  From the bottom chord's axis up to where a vertical seats on it:
  its top face, or the inside of its web when it is a channel open
  upward that the vertical fits in (then negative).

- `top_seat: f64`

  From the top chord's axis down to where a vertical seats, square to the chord.

- `post_along: f64`

  The vertical's size along the truss.

- `post_turned: bool`

  Whether the verticals are turned a quarter, their depth along the truss.


**Methods**

- `pub fn of(project: &Project, truss: &Truss) -> Self`

  The sections of `truss`'s member types in `project`. The verticals
  are turned so the side nearest the chords' width lies across the truss.


<a id="mod-cc_core-types"></a>

## `cc_core::types`

<sub>`crates/cc_core/src/types.rs`</sub>

**Items:** [`Category`](#cc_core-types-category) · [`LayerFunction`](#cc_core-types-layerfunction) · [`Layer`](#cc_core-types-layer) · [`WallFunction`](#cc_core-types-wallfunction) · [`TypeInfo`](#cc_core-types-typeinfo) · [`TypeData`](#cc_core-types-typedata) · [`default_cut_slope`](#cc_core-types-default_cut_slope) · [`default_fill_slope`](#cc_core-types-default_fill_slope) · [`default_contour_interval`](#cc_core-types-default_contour_interval) · [`default_major_every`](#cc_core-types-default_major_every) · [`ColumnSection`](#cc_core-types-columnsection) · [`SectionShape`](#cc_core-types-sectionshape) · [`FoundationKind`](#cc_core-types-foundationkind) · [`StripHaunch`](#cc_core-types-striphaunch) · [`ElementType`](#cc_core-types-elementtype) · [`stair_layout`](#cc_core-types-stair_layout)

<a id="cc_core-types-category"></a>

### enum `Category`

```rust
pub enum Category
```

BIM category of an element or element type.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Serialize`, `Deserialize`

**Variants**

- `Wall`
- `Door`
- `Window`
- `Floor`
- `Ceiling`
- `Roof`
- `Room`
- `Grid`
- `Dimension`
- `Text`
- `Column`
- `Beam`
- `Stair`
- `Railing`
- `Component`
- `Foundation`
- `Rebar`
- `Pipe`

  Plumbing pipes (water supply, drainage, vent, rainwater).

- `Conduit`

  Electrical conduits.

- `Ramp`
- `CadImport`

  Imported DWG/DXF drawings (underlays).

- `Mass`

  Conceptual masses (massing studies).

- `Sweep`

  Profiles swept along paths (baseboards, moldings, fascias, gutters).

- `Shaft`

  Shaft openings through floors, ceilings and roofs.

- `SteelBuilding`

  Parametric steel buildings: hosts of portal frames, purlins and girts.

- `Truss`

  Parametric trusses: hosts of chords, diagonals and posts.

- `Bracing`

  Parametric bracings: hosts of the braces of a bay of a frame.

- `WallFraming`

  Light steel and wood framing of walls: hosts of studs, tracks, headers and sills.

- `Topography`

  Terrain surfaces.

- `Pad`

  Building pads graded into terrain.

- `TitleBlock`

  Title blocks: sheet layouts (components drawn on sheets).

- `DetailLine`

  Detail lines: 2D lines of one view.

- `FilledRegion`

  Filled regions: 2D areas of one view filled with a pattern.

- `MaskingRegion`

  Masking regions: 2D areas of one view hiding the model under them.

- `LevelMark`

  Level marks: the elevation of a point in a plan, elevation or section.

- `BarTag`

  Bar tags: the bars of a reinforcement set written in a view.

- `Pile`

  Piles (deep foundations), free or under pile caps.

- `DetailComponent`

  Detail components: reusable 2D components placed in one view.

- `CableTray`

  Cable trays: open runs carrying electrical circuits.

- `Valve`

  Valves on pipes.

- `LightningConductor`

  Conductors of lightning protection systems.

- `Connection`

  Steel and timber connections: plates and bolts at member ends.

- `Niche`

  Niches: recesses in one face of a wall, hosted like doors and windows.


**Methods**

- `pub fn is_phased(self) -> bool` *(in `cc_core::phase`)*

  Whether elements of the category belong to phases (annotations do not).

- `pub const ALL: [Category; 42]`
- `pub const NAMED: [Category; Category::ALL.len() + 1]`

  Every category by name: `Category::ALL` (the ones views draw and
  filter) and title blocks, whose types sheets use.

- `pub const MODEL: [Category; 24]`

  Categories with physical elements (shown in legends and 3D).

- `pub const SCHEDULABLE: [Category; 27]`

  Categories that can be listed in a schedule.

- `pub fn is_model(self) -> bool`
- `pub fn key(self) -> &'static str`

  Translation key for the category name.

- `pub fn is_mep(self) -> bool`

  Whether the category belongs to the building services (MEP).


<a id="cc_core-types-layerfunction"></a>

### enum `LayerFunction`

```rust
pub enum LayerFunction
```

Role of a layer in a compound type. The core (structure layers) is the
reference for core-based wall location lines.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Structure`
- `Substrate`
- `Insulation`
- `Finish`
- `Membrane`

**Methods**

- `pub const ALL: [LayerFunction; 5]`
- `pub fn key(self) -> &'static str`

  Translation key of the function name.


<a id="cc_core-types-layer"></a>

### struct `Layer`

```rust
pub struct Layer
```

One material layer of a compound wall/floor/roof type, listed exterior first.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `material: String`
- `thickness: f64`

  Thickness in meters.

- `function: LayerFunction`

<a id="cc_core-types-wallfunction"></a>

### enum `WallFunction`

```rust
pub enum WallFunction
```

Purpose of a wall type.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Interior`
- `Exterior`
- `Foundation`
- `Retaining`

**Methods**

- `pub const ALL: [WallFunction; 4]`
- `pub fn key(self) -> &'static str`
- `pub fn is_external(self) -> bool`

  Whether walls of this function face the outside of the building.


<a id="cc_core-types-typeinfo"></a>

### struct `TypeInfo`

```rust
pub struct TypeInfo
```

Descriptive data shared by all instances of a type.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `type_mark: String`

  Short code shown in tags and schedules.

- `description: String`
- `manufacturer: String`
- `model: String`
- `url: String`
- `keynote: String`
- `classification: String`

  Classification or assembly code (e.g. a national construction classification).

- `fire_rating: String`
- `comments: String`
- `cost: Option<f64>`

  Unit cost in the project currency.


<a id="cc_core-types-typedata"></a>

### enum `TypeData`

```rust
pub enum TypeData
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Variants**

- `Wall { layers: Vec<Layer>, #[serde(default)] function: WallFunction, #[serde(default, skip_serializing_if = "Option::is_none")] curtain: Option<crate::curtain::CurtainSystem>, }`
- `Door { width: f64, height: f64, material: String, }`
- `Window { width: f64, height: f64, default_sill: f64, material: String, }`
- `Floor { layers: Vec<Layer>, }`
- `Ceiling { thickness: f64, material: String, }`
- `Roof { layers: Vec<Layer>, #[serde(default, skip_serializing_if = "Option::is_none")] covering: Option<crate::roofing::TileKind>, #[serde(default)] framing: crate::roofing::RoofFraming, }`
- `Room {}`

  Rooms have no type parameters yet; the type groups them in schedules.

- `Grid { bubble: f64, }`

  Grid line with a bubble of `bubble` paper millimeters.

- `Dimension { text_size: f64, #[serde(default)] style: crate::dim_style::DimensionStyle, }`

  Dimensions with text of `text_size` paper millimeters, drawn as `style` says.

- `Text { size: f64, #[serde(default)] symbol: crate::LeaderSymbol, }`

  Text notes of `size` paper millimeters; new leaders get `symbol`.

- `BarTag { text_size: f64, }`

  Level marks with text of `text_size` paper millimeters, writing
  `datum` plus each level, followed by `finished_suffix` (the finished
  level) or `raw_suffix` (the raw, structural one).
  Bar tags with text of `text_size` paper millimeters.

- `LevelMark { text_size: f64, #[serde(default)] datum: f64, #[serde(default)] finished_suffix: String, #[serde(default)] raw_suffix: String, }`
- `Column { section: ColumnSection, material: String, #[serde(default, skip_serializing_if = "Option::is_none")] profile: Option<crate::ProfileId>, #[serde(default, skip_serializing_if = "is_zero")] profile_rotation_deg: f64, #[serde(default, skip_serializing_if = "std::ops::Not::not")] profile_mirror: bool, }`
- `Beam { width: f64, height: f64, material: String, #[serde(default)] shape: SectionShape, #[serde(default, skip_serializing_if = "Option::is_none")] profile: Option<crate::ProfileId>, #[serde(default, skip_serializing_if = "is_zero")] profile_rotation_deg: f64, #[serde(default, skip_serializing_if = "std::ops::Not::not")] profile_mirror: bool, }`

  Beam section `width` x `height`, solid or a profile within that box.

- `Rebar { diameter: f64, material: String, #[serde(default, skip_serializing_if = "String::is_empty")] grade: String, #[serde(default, skip_serializing_if = "Option::is_none")] mass_per_meter: Option<f64>, }`

  Reinforcing bars of `diameter`.

- `Pipe { diameter: f64, material: String, }`

  Pipes of outside `diameter`.

- `Conduit { diameter: f64, material: String, }`

  Conduits of outside `diameter`.

- `CableTray { width: f64, height: f64, #[serde(default)] style: crate::TrayStyle, material: String, }`

  Cable trays of outside `width` x `height`, built as `style` says.

- `Valve { #[serde(default)] kind: crate::ValveKind, material: String, }`

  Valves of `kind` (their diameter is their pipe's).

- `LightningConductor { diameter: f64, material: String, }`

  Lightning protection conductors: round conductors of `diameter`.

- `Foundation { kind: FoundationKind, width: f64, length: f64, thickness: f64, material: String, #[serde(default, skip_serializing_if = "Option::is_none")] haunch: Option<StripHaunch>, #[serde(default, skip_serializing_if = "Option::is_none")] pile_cap: Option<crate::pile::PileCap>, }`

  Foundation: a pad `width` x `length`, a strip `width` wide or a slab,
  each `thickness` deep. A strip with a `haunch` is an inverted T: a
  footing slab `thickness` deep at the faces of a beam on top of it.

- `Pile { section: crate::pile::PileSection, #[serde(default)] kind: crate::pile::PileKind, material: String, }`

  Piles of `section` made as `kind` says, in `material`.

- `Stair { width: f64, max_riser: f64, tread: f64, material: String, #[serde(default)] build: crate::StairBuild, #[serde(default)] rules: crate::StairRules, }`

  Stair: risers at most `max_riser` high, treads `tread` deep (or from
  Blondel's formula, see `rules`), built as `build` says.

- `Ramp { width: f64, thickness: f64, material: String, #[serde(default)] rules: crate::RampRules, }`

  Ramp: a slab `thickness` thick, `width` wide, within `rules`.

- `Railing { height: f64, post_spacing: f64, material: String, }`

  Railing with a top rail at `height` and posts every `post_spacing`.

- `Component { component: String, def: crate::component::ComponentDef, }`

  Parametric component: `component` names the definition shared by its
  types; the parameter values in `def` are this type's.

- `CadImport {}`

  Imported CAD drawings have no type parameters; the type names them.

- `DetailLine {}`

  Detail lines have no type parameters (each line has its style).

- `FilledRegion { #[serde(default)] fill: crate::PatternFill, }`

  Filled regions: how they are filled (see `crate::region`).

- `MaskingRegion {}`

  Masking regions have no type parameters (they are opaque white).

- `Mass { material: String, }`

  Conceptual masses, shown in `material`.

- `Sweep { profile: Vec<glam::DVec2>, material: String, #[serde(default, skip_serializing_if = "Option::is_none")] sketch: Option<crate::sketch::Sketch>, }`

  Swept profiles: the closed `profile` (x across, to the left of the
  path; y up), shown in `material`.

- `Shaft {}`

  Shaft openings (no properties of their own).

- `SteelBuilding {}`

  Steel buildings (no properties of their own; types name them).

- `Truss {}`

  Trusses (no properties of their own; types name them).

- `Bracing {}`

  Bracings (no properties of their own; types name them).

- `Connection {}`

  Connections (no properties of their own; types name them).

- `WallFraming {}`

  Wall framings (no properties of their own; types name them).

- `Topography { material: String, #[serde(default = "crate::types::default_contour_interval")] contour_interval: f64, #[serde(default = "crate::types::default_major_every")] major_every: u32, }`

  Terrain surfaces: the earth `material` they are cut in, and the
  spacing of their contour lines, every `major_every`-th one a major.

- `Pad { #[serde(default = "crate::types::default_cut_slope")] cut_slope: f64, #[serde(default = "crate::types::default_fill_slope")] fill_slope: f64, }`

  Building pads: the horizontal run per unit of rise of their cut and
  fill slopes.


**Methods**

- `pub fn rebar(diameter: f64, material: String) -> TypeData`

  A type of reinforcing bars of `diameter` in `material`, with no grade
  and the mass per meter of a round bar.

- `pub fn materials(&self) -> Vec<&str>`

  Material names referenced by this type.

- `pub fn cut_material(&self) -> &str`

  Material shown where the type is cut as a whole (sections): the first
  structure layer of layered types (else the thickest), or the main material.

- `pub fn uses_material(&self, name: &str) -> bool`
- `pub fn rename_material(&mut self, old: &str, new: &str)`

  Point references to material `old` at `new`.

- `pub fn foundation_depth(&self) -> f64`

  Depth of a foundation from its top to its bottom: its thickness,
  plus the beam on top of a haunched strip. 0 for other types.

- `pub fn strip_haunch(&self) -> Option<StripHaunch>`

  The haunch of a strip footing type (`None` for a plain strip and other types).


<a id="cc_core-types-default_cut_slope"></a>

### fn `default_cut_slope`

```rust
pub(crate) fn default_cut_slope() -> f64
```

<a id="cc_core-types-default_fill_slope"></a>

### fn `default_fill_slope`

```rust
pub(crate) fn default_fill_slope() -> f64
```

<a id="cc_core-types-default_contour_interval"></a>

### fn `default_contour_interval`

```rust
pub(crate) fn default_contour_interval() -> f64
```

<a id="cc_core-types-default_major_every"></a>

### fn `default_major_every`

```rust
pub(crate) fn default_major_every() -> u32
```

<a id="cc_core-types-columnsection"></a>

### enum `ColumnSection`

```rust
pub enum ColumnSection
```

Cross-section of a column type.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`

**Variants**

- `Rectangular { width: f64, depth: f64, }`
- `Round { diameter: f64, }`
- `Profile { width: f64, depth: f64, shape: SectionShape, }`

  A profile (I, channel, angle, tube) within a `width` x `depth` box.

- `Pipe { diameter: f64, wall: f64, }`

  A circular hollow section.


**Methods**

- `pub fn size(&self) -> (f64, f64)`

  The bounding box, `(width, depth)`.


<a id="cc_core-types-sectionshape"></a>

### enum `SectionShape`

```rust
pub enum SectionShape
```

What fills a section's `width` x `depth` box. Profiles are laid out as in
steel tables: the web of an I or channel runs along the depth, the legs of
an angle lie on the left and bottom sides of the box, and the flanges of a
channel point to the right.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Solid`
- `I { web: f64, flange: f64 }`

  I or H: two flanges `flange` thick joined by a web `web` thick.

- `Channel { web: f64, flange: f64 }`

  Channel (U): a web on the left and two flanges.

- `Angle { thickness: f64 }`

  Angle (L): two legs `thickness` thick.

- `Tube { wall: f64 }`

  Rectangular hollow section with walls `wall` thick.


**Methods**

- `pub fn key(&self) -> &'static str`
- `pub fn fitted(self, width: f64, depth: f64) -> Self`

  The same shape with each thickness clamped to fit a `width` x `depth` box.


<a id="cc_core-types-foundationkind"></a>

### enum `FoundationKind`

```rust
pub enum FoundationKind
```

What a foundation type is: a pad under a column, a strip under a wall, a
slab, or a cap on piles (see `crate::pile`).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Pad`
- `Strip`
- `Slab`
- `PileCap`

**Methods**

- `pub const ALL: [FoundationKind; 4]`
- `pub fn key(self) -> &'static str`

<a id="cc_core-types-striphaunch"></a>

### struct `StripHaunch`

```rust
pub struct StripHaunch
```

The section of a strip footing with a beam on top (an inverted T): the
footing slab is the type's `thickness` deep at the faces of the beam and
`edge_thickness` at its edges, its top sloping between them (a haunch;
flat when the two are equal), and the beam `beam_width` wide rises
`beam_height` above the slab (0: a sloped footing with a flat middle).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `edge_thickness: f64`
- `beam_width: f64`
- `beam_height: f64`

**Methods**

- `pub const MAX_HEIGHT: f64`

  Highest beam, meters.

- `pub fn fits(&self, width: f64, thickness: f64) -> bool`

  Whether the values make a section in a strip `width` wide and
  `thickness` deep at the beam: an edge no thicker than that, a beam
  narrower than the strip.

- `pub fn outline(&self, width: f64, thickness: f64) -> Vec<glam::DVec2>`

  The section of a strip `width` wide and `thickness` deep at the beam:
  `x` across (centered), `y` up from the bottom, counter-clockwise.

- `pub fn area(&self, width: f64, thickness: f64) -> f64`

  Area of that section.


<a id="cc_core-types-elementtype"></a>

### struct `ElementType`

```rust
pub struct ElementType
```

An element type: the shared parameters of a group of instances.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `id: ElementTypeId`
- `name: String`
- `data: TypeData`
- `info: TypeInfo`
- `uid: Uid`

**Methods**

- `pub fn category(&self) -> Category`
- `pub fn layers(&self) -> &[Layer]`

  The layers of a layered type (none for curtain walls).

- `pub fn thickness(&self) -> f64`

  Total thickness for layered types, 0 for openings.

- `pub fn diameter(&self) -> Option<f64>`

  Outside diameter of pipe, conduit and rebar types.

- `pub fn tray_section(&self) -> Option<(f64, f64, crate::TrayStyle)>`

  Outside width, height and style of cable tray types.

- `pub fn valve_kind(&self) -> Option<crate::ValveKind>`

  What valves of a valve type do.

- `pub fn curtain(&self) -> Option<&crate::curtain::CurtainSystem>`

  The curtain grid of a curtain wall type.

- `pub fn wall_function(&self) -> Option<WallFunction>`

  Wall function, for wall types.

- `pub fn core_range(&self) -> (f64, f64)`

  Distances from the exterior (first) face to the exterior and interior
  faces of the core: the span from the first to the last structure layer,
  or the whole thickness when there is none.

- `pub fn location_offset(&self, line: crate::LocationLine) -> f64`

  Position of a wall's location line `line` across a wall of this type,
  measured from the centerline towards the exterior (left) face.

- `pub fn opening_size(&self) -> Option<(f64, f64)>`

  Width/height of an opening type (doors, windows).

- `pub fn opening_depth(&self) -> Option<f64>`

  The depth of a niche type's recess; `None` for types that open their
  wall through (doors, windows) and for anything else.

- `pub fn window_operation(&self) -> crate::WindowOperation`

  How the panels of a window type open (fixed for anything else).

- `pub fn door_operation(&self) -> crate::DoorOperation`

  How the leaves of a door type swing (one leaf hinged on the left for anything else).

- `pub fn default_sill(&self) -> f64`

  Sill height of new elements of a door or window type.

- `pub fn main_material(&self) -> &str`

  Material used to color the element in shaded views: the exterior finish
  layer for walls and roofs, the thickest layer for floors.


<a id="cc_core-types-stair_layout"></a>

### fn `stair_layout`

```rust
pub fn stair_layout(height: f64, max_riser: f64, tread: f64) -> (usize, f64, f64)
```

Riser count, riser height and run length of a straight stair of `height`
with at most `max_riser` per riser and `tread`-deep treads (one tread fewer
than risers: the last riser lands on the upper floor).

<a id="mod-cc_core-units"></a>

## `cc_core::units`

<sub>`crates/cc_core/src/units.rs`</sub>

Project units: how lengths, areas, volumes, angles, slopes and money are
displayed and typed. Values in the model stay in meters and degrees;
these settings only affect presentation.

**Items:** [`INCH`](#cc_core-units-inch) · [`FOOT`](#cc_core-units-foot) · [`MeasurementSystem`](#cc_core-units-measurementsystem) · [`inch_text`](#cc_core-units-inch_text) · [`round_imperial`](#cc_core-units-round_imperial) · [`LengthUnit`](#cc_core-units-lengthunit) · [`AreaUnit`](#cc_core-units-areaunit) · [`VolumeUnit`](#cc_core-units-volumeunit) · [`SlopeUnit`](#cc_core-units-slopeunit) · [`NumberStyle`](#cc_core-units-numberstyle) · [`MAX_DECIMALS`](#cc_core-units-max_decimals) · [`Format`](#cc_core-units-format) · [`Units`](#cc_core-units-units)

<a id="cc_core-units-inch"></a>

### const `INCH`

```rust
pub const INCH: f64
```

Meters in an inch (exact, by definition).

<a id="cc_core-units-foot"></a>

### const `FOOT`

```rust
pub const FOOT: f64
```

Meters in a foot (exact, by definition).

<a id="cc_core-units-measurementsystem"></a>

### enum `MeasurementSystem`

```rust
pub enum MeasurementSystem
```

The system a project measures in: what its template sizes its types in
and what its units show. Not stored: a project is imperial when its
length unit is (see `Units::system`).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`

**Variants**

- `Metric`

  Meters and millimeters.

- `Imperial`

  US customary feet and inches.


**Methods**

- `pub const ALL: [MeasurementSystem; 2]`
- `pub fn code(self) -> &'static str`

  The code commands use: `metric` or `imperial`.

- `pub fn from_code(code: &str) -> Option<Self>`

  The system of a code (case and surrounding spaces ignored).

- `pub fn units(self) -> Units`

  The units a new project of this system shows.


<a id="cc_core-units-inch_text"></a>

### fn `inch_text`

```rust
pub fn inch_text(meters: f64) -> String
```

A length in inches with sixteenths as a fraction, with the inch mark:
`8"`, `6 1/2"`, `5/16"` (rounded to the nearest sixteenth).

<a id="cc_core-units-round_imperial"></a>

### fn `round_imperial`

```rust
pub fn round_imperial(meters: f64) -> f64
```

A metric size turned into a round imperial one: the nearest inch up to
a foot, else the nearest 6" (0.1 m is 4", 0.6 m 2'-0", 2.4 m 8'-0").

<a id="cc_core-units-lengthunit"></a>

### enum `LengthUnit`

```rust
pub enum LengthUnit
```

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Millimeter`
- `Centimeter`
- `Meter`
- `Foot`

  Decimal feet.

- `Inch`

  Decimal inches.

- `FeetInches`

  Feet and fractional inches (`12'-6 1/2"`).

- `FractionalInch`

  Fractional inches (`150 1/2"`).


**Methods**

- `pub const ALL: [LengthUnit; 7]`
- `pub fn per_meter(self) -> f64`

  Display units per meter (feet for feet and inches, inches for
  fractional inches: the unit a bare typed number is read in).

- `pub fn symbol(self) -> &'static str`
- `pub fn is_fractional(self) -> bool`

  Shown with fractions of an inch: the decimals setting `d` is then
  the precision, 1/2^d of an inch.

- `pub fn is_imperial(self) -> bool`

  A US customary (imperial) unit.


<a id="cc_core-units-areaunit"></a>

### enum `AreaUnit`

```rust
pub enum AreaUnit
```

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `SquareMeter`
- `SquareCentimeter`
- `SquareFoot`
- `SquareInch`

**Methods**

- `pub const ALL: [AreaUnit; 4]`
- `pub fn per_square_meter(self) -> f64`
- `pub fn symbol(self) -> &'static str`

<a id="cc_core-units-volumeunit"></a>

### enum `VolumeUnit`

```rust
pub enum VolumeUnit
```

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `CubicMeter`
- `Liter`
- `CubicFoot`
- `CubicYard`
- `Gallon`

  US liquid gallon.


**Methods**

- `pub const ALL: [VolumeUnit; 5]`
- `pub fn per_cubic_meter(self) -> f64`
- `pub fn symbol(self) -> &'static str`

<a id="cc_core-units-slopeunit"></a>

### enum `SlopeUnit`

```rust
pub enum SlopeUnit
```

How slopes (rise over run) are shown.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Percent`
- `Degrees`
- `RisePer12`

  The rise per 12 of run, the customary way of US roofs (`6:12`).


**Methods**

- `pub const ALL: [SlopeUnit; 3]`
- `pub fn symbol(self) -> &'static str`

<a id="cc_core-units-numberstyle"></a>

### enum `NumberStyle`

```rust
pub enum NumberStyle
```

Decimal symbol and digit grouping.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Language`

  Decimal symbol of the interface language, no grouping.

- `CommaDecimalPointGroups`

  1.234.567,89

- `CommaDecimalSpaceGroups`

  1 234 567,89

- `CommaDecimal`

  1234567,89

- `PointDecimalCommaGroups`

  1,234,567.89

- `PointDecimal`

  1234567.89


**Methods**

- `pub const ALL: [NumberStyle; 6]`
- `pub fn separators(self, language_comma: bool) -> (char, Option<char>)`

  Decimal symbol and group separator, given the language's decimal comma.


<a id="cc_core-units-max_decimals"></a>

### const `MAX_DECIMALS`

```rust
pub const MAX_DECIMALS: u8
```

Most decimals any unit may show.

<a id="cc_core-units-format"></a>

### struct `Format`

```rust
pub struct Format<U>
```

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `unit: U`
- `decimals: u8`
- `symbol: bool`

  Append the unit symbol (dimensions usually omit it).


<a id="cc_core-units-units"></a>

### struct `Units`

```rust
pub struct Units
```

Display settings for every kind of quantity.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `length: Format<LengthUnit>`
- `area: Format<AreaUnit>`
- `volume: Format<VolumeUnit>`
- `angle: Format<()>`

  Angles are always degrees.

- `slope: Format<SlopeUnit>`
- `currency: String`

  Currency symbol written before amounts (empty for none).

- `currency_decimals: u8`
- `style: NumberStyle`

**Methods**

- `pub fn imperial() -> Self`

  US customary units: feet and inches to 1/16", square feet, cubic
  feet, slopes as the rise per 12 (`6:12`), point decimals with comma
  groups.

- `pub fn system(&self) -> MeasurementSystem`

  The measurement system these units belong to: imperial when lengths
  are in an imperial unit, else metric.

- `pub fn repair(&mut self) -> bool`

  Clamp decimals to `MAX_DECIMALS`; returns whether anything changed.


<a id="mod-cc_core-view"></a>

## `cc_core::view`

<sub>`crates/cc_core/src/view.rs`</sub>

**Items:** [`ViewCategory`](#cc_core-view-viewcategory) · [`Direction`](#cc_core-view-direction) · [`CameraState`](#cc_core-view-camerastate) · [`MIN_PAPER_MM`](#cc_core-view-min_paper_mm) · [`MAX_PAPER_MM`](#cc_core-view-max_paper_mm) · [`SheetSize`](#cc_core-view-sheetsize) · [`Viewport`](#cc_core-view-viewport) · [`ViewKind`](#cc_core-view-viewkind) · [`View`](#cc_core-view-view) · [`ShadingSettings`](#cc_core-view-shadingsettings) · [`VisualStyle`](#cc_core-view-visualstyle) · [`DetailLevel`](#cc_core-view-detaillevel) · [`MAX_SECTION_SEGMENT`](#cc_core-view-max_section_segment) · [`MepDiscipline`](#cc_core-view-mepdiscipline) · [`Underlay`](#cc_core-view-underlay) · [`UnderlayPlan`](#cc_core-view-underlayplan) · [`UnderlayLevel`](#cc_core-view-underlaylevel) · [`Looking`](#cc_core-view-looking)

<a id="cc_core-view-viewcategory"></a>

### enum `ViewCategory`

```rust
pub enum ViewCategory
```

Grouping used by the Project Browser.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Serialize`, `Deserialize`

**Variants**

- `FloorPlan`
- `CeilingPlan`
- `ThreeD`
- `Elevation`
- `Section`
- `Legend`
- `Drafting`

  2D views drawn by hand, independent of the model.

- `Schedule`
- `Diagram`

  Single-line diagrams of distribution boards.

- `Sheet`

**Methods**

- `pub const ALL: [ViewCategory; 10]`
- `pub fn key(self) -> &'static str`

  Translation key for the browser group heading.


<a id="cc_core-view-direction"></a>

### enum `Direction`

```rust
pub enum Direction
```

Elevation direction, named by the side the viewer stands on.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Serialize`, `Deserialize`

**Variants**

- `North`
- `South`
- `East`
- `West`

**Methods**

- `pub fn look(self) -> DVec2`

  Unit vector the viewer looks along (from their side towards the model).


<a id="cc_core-view-camerastate"></a>

### struct `CameraState`

```rust
pub struct CameraState
```

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `target: DVec3`
- `yaw_deg: f64`

  Rotation around Z, degrees; 0 looks along +Y.

- `pitch_deg: f64`

  Elevation angle above the horizon, degrees.

- `distance: f64`

**Methods**

- `pub fn forward(&self) -> DVec3`

  Unit vector from the eye towards the target.

- `pub fn eye(&self) -> DVec3`

<a id="cc_core-view-min_paper_mm"></a>

### const `MIN_PAPER_MM`

```rust
pub const MIN_PAPER_MM: f64
```

Smallest and largest side of a custom sheet, millimeters.

<a id="cc_core-view-max_paper_mm"></a>

### const `MAX_PAPER_MM`

```rust
pub const MAX_PAPER_MM: f64
```

<a id="cc_core-view-sheetsize"></a>

### enum `SheetSize`

```rust
pub enum SheetSize
```

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`

**Variants**

- `A0`
- `A1`
- `A2`
- `A3`
- `A4`
- `Custom { width: f64, height: f64, }`

  Any size, millimeters (width and height as typed; `portrait` puts the long side up).

- `AnsiA`

  ANSI A, 11" × 8 1/2" (US letter).

- `AnsiB`

  ANSI B, 17" × 11" (tabloid).

- `AnsiC`

  ANSI C, 22" × 17".

- `AnsiD`

  ANSI D, 34" × 22".

- `AnsiE`

  ANSI E, 44" × 34".

- `ArchA`

  Architectural A, 12" × 9".

- `ArchB`

  Architectural B, 18" × 12".

- `ArchC`

  Architectural C, 24" × 18".

- `ArchD`

  Architectural D, 36" × 24".

- `ArchE1`

  Architectural E1, 42" × 30".

- `ArchE`

  Architectural E, 48" × 36".


**Methods**

- `pub const ALL: [SheetSize; 16]`

  Every named size: the ISO A sizes, then ANSI, then architectural.

- `pub fn choices(system: MeasurementSystem) -> Vec<SheetSize>`

  Every named size, the system's own first (metric: the A sizes;
  imperial: ANSI and architectural), for pickers.

- `pub fn default_for(system: MeasurementSystem) -> SheetSize`

  The paper of a new sheet: A3, or ANSI B in imperial projects.

- `pub fn size_mm(self) -> DVec2`

  Landscape paper size in millimeters (a custom size as typed, kept
  within `MIN_PAPER_MM` and `MAX_PAPER_MM`).

- `pub fn label(self) -> String`

  The size's name (`A3`, `ANSI B`, `ARCH D`); a custom size in
  millimeters (see `SheetSize::label_in`).

- `pub fn label_in(self, system: MeasurementSystem) -> String`

  The size's name; a custom size in the system's paper unit
  (`1000 × 594 mm`, `17" × 11"`).

- `pub fn from_name(name: &str) -> Option<SheetSize>`

  The named size called `name`, any case, with spaces, `_` or `-`
  between its parts or none (`A3`, `ANSI B`, `ansi_b`, `ArchD`).

- `pub fn is_valid(self) -> bool`

  A custom size within the limits, finite.


<a id="cc_core-view-viewport"></a>

### struct `Viewport`

```rust
pub struct Viewport
```

A view placed on a sheet.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `view: ViewId`
- `center: DVec2`

  Center of the viewport on the sheet, millimeters from the bottom-left corner.


<a id="cc_core-view-viewkind"></a>

### enum `ViewKind`

```rust
pub enum ViewKind
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Variants**

- `FloorPlan { level: LevelId, cut_height: f64, #[serde(default)] structural: bool, #[serde(default)] structure_tags: bool, #[serde(default, skip_serializing_if = "Option::is_none")] mep: Option<MepDiscipline>, #[serde(default, skip_serializing_if = "Underlay::is_empty")] underlay: Underlay, }`
- `CeilingPlan { level: LevelId, cut_height: f64, #[serde(default, skip_serializing_if = "Underlay::is_empty")] underlay: Underlay, }`
- `ThreeD { camera: CameraState, #[serde(default)] analytical: bool, #[serde(default)] rebar: bool, }`
- `Elevation { direction: Direction, }`
- `Section { start: DVec2, end: DVec2, depth: f64, #[serde(default, skip_serializing_if = "Option::is_none")] segments: Option<[f64; 2]>, }`

  Section line from `start` to `end`; looks to the right of the line.

- `Legend { types: Vec<ElementTypeId>, }`
- `Drafting {}`

  A 2D view independent of the model, with its own scale: it shows no
  model geometry, only what is drawn in it (the elements it owns, such
  as text notes, dimensions and imported CAD drawings).

- `Schedule { category: Category, #[serde(default)] def: crate::ScheduleDef, }`
- `Sheet { number: String, size: SheetSize, #[serde(default, skip_serializing_if = "std::ops::Not::not")] portrait: bool, viewports: Vec<Viewport>, #[serde(default, skip_serializing_if = "Option::is_none")] title_block: Option<crate::ElementTypeId>, #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")] fields: std::collections::BTreeMap<String, String>, #[serde(default, skip_serializing_if = "String::is_empty")] issue_date: String, }`
- `Diagram { panel: crate::ElementId, }`

  The single-line diagram and load schedule of a distribution board.


**Methods**

- `pub fn croppable(&self) -> bool` *(in `cc_core::callout`)*

  Whether views of this kind can be cropped and hold callouts: floor
  and ceiling plans, sections and elevations.

- `pub fn flip_section(&mut self) -> bool`

  Turn a section to look the other way: its ends swap, and the pieces
  of a split line go with them. `false` for other views.

- `pub fn has_graphics(&self) -> bool`

  Whether views of this kind have visibility/graphics settings: the
  views of the model (those that take templates) and drafting views.

- `pub fn is_phased(&self) -> bool`

  Whether views of this kind show the model in a phase (plans,
  elevations, sections, 3D views and schedules).

- `pub fn takes_sun(&self) -> bool`

  Whether views of this kind can have a sun (`View::sun`): plans,
  ceiling plans, 3D views, elevations and sections.

- `pub fn takes_render(&self) -> bool`

  Whether views of this kind can have render settings (`View::render`):
  3D views only.

- `pub fn ceiling_plan(level: LevelId) -> Self`

  An ordinary ceiling plan of `level` cut at 1.20 m.

- `pub fn underlay(&self) -> Option<&Underlay>`

  The underlay of a floor or ceiling plan.

- `pub fn underlay_mut(&mut self) -> Option<&mut Underlay>`
- `pub fn floor_plan(level: LevelId) -> Self`

  An ordinary floor plan of `level` cut at 1.20 m.


<a id="cc_core-view-view"></a>

### struct `View`

```rust
pub struct View
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`

**Fields**

- `id: ViewId`
- `name: String`
- `scale: u32`

  Scale denominator (50 means 1:50).

- `kind: ViewKind`
- `graphics: crate::ViewGraphics`

  Category visibility and overrides, filters, element overrides.

- `template: Option<crate::TemplateId>`

  The template the view follows: the properties it controls are
  copied from it on every edit.

- `phase: crate::PhaseId`

  The phase the view shows the model in (see `crate::phase`).

- `phase_filter: Option<crate::PhaseFilterId>`

  Which statuses the view shows, and how (`None`: all, by category).

- `design_options: std::collections::BTreeMap<crate::OptionSetId, crate::OptionId>`

  The option the view shows of each design option set; sets left out
  show automatically (see `crate::design_option`).

- `tag_offsets: std::collections::BTreeMap<crate::ElementId, glam::DVec2>`

  Tags moved by hand in this view: their offset in plan from where they
  sit by default, by the element they tag.

- `detail: DetailLevel`

  How much of the elements the view draws (see `DetailLevel`).

- `visual_style: Option<VisualStyle>`

  How the view shows the model's surfaces (`None`: the usual style of
  its kind, see `View::style`).

- `crop: Option<crate::callout::CropBox>`

  The rectangle of the view's drawing outside which the model is not
  drawn (plans, sections and elevations; see `crate::callout`).

- `callout: Option<crate::callout::Callout>`

  The callout standing for this view in another one: a detail view
  of the model (cropped to it) or a drafting view it points to.

- `sun: Option<crate::sun::SunSettings>`

  The view's sun: a date and time, a study or a light direction
  (see `crate::sun`); only views that [take one](ViewKind::takes_sun).

- `render: Option<crate::render_settings::RenderSettings>`

  How the view is rendered photorealistically (see
  `crate::render_settings`); only views that [take them](ViewKind::takes_render).

- `shading: ShadingSettings`

  How light and dark the shaded, consistent colors and textures
  styles shade faces and cast shadows (see `ShadingSettings`); used
  by the views that [take a sun](ViewKind::takes_sun), kept at the
  defaults on the others.


**Methods**

- `pub fn callout_rect(&self) -> Option<CropBox>` *(in `cc_core::callout`)*

  The rectangle of the view's callout in its parent: the crop of a
  detail view, the callout's own rectangle for a drafting view.

- `pub fn set_callout_rect(&mut self, rect: CropBox) -> bool` *(in `cc_core::callout`)*

  Move or resize the view's callout (the crop of a detail view);
  `false` when it has none.

- `pub fn is_detail(&self) -> bool` *(in `cc_core::callout`)*

  A detail view of the model made from a callout (not a drafting view).

- `pub fn projection(&self) -> Option<Projection>` *(in `cc_core::dim_refs`)*

  The projection of an elevation or section view.

- `pub fn is_plan(&self) -> bool` *(in `cc_core::dim_refs`)*
- `pub fn paper_mm(&self) -> Option<DVec2>`

  A sheet's paper size in millimeters, after its orientation.

- `pub fn style(&self) -> VisualStyle`

  The style the view is drawn in: its own, else plans in hidden line
  (as drawn before styles existed) and other views shaded.

- `pub fn new(id: ViewId, name: impl Into<String>, scale: u32, kind: ViewKind) -> Self`

  A view with default graphics, no template, in the (default) last phase.

- `pub fn category(&self) -> ViewCategory`
- `pub fn level(&self) -> Option<LevelId>`

  Level for plan views, where new elements are placed.

- `pub fn is_drafting(&self) -> bool`

  Whether the view is a drafting view: 2D, drawn by hand, showing no model geometry.

- `pub fn takes_dimensions(&self) -> bool`

  Whether dimensions can be drawn in the view: plans, elevations,
  sections, legends and drafting views.

- `pub fn placeable(&self) -> bool`

  Whether the view can be placed on a sheet.


<a id="cc_core-view-shadingsettings"></a>

### struct `ShadingSettings`

```rust
pub struct ShadingSettings
```

How a view shades faces and casts shadows in the styles that show more
than white faces (shaded, consistent colors, textures), in 2D and in
the 3D view (not in photorealistic renders). The defaults are the
look of `VisualStyle::shaded_grey` and `VisualStyle::SHADED_SHADOW`.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`, `Default`

**Fields**

- `ambient: f64`

  The light of a face turned away from the light, as displayed:
  `ShadingSettings::MIN_AMBIENT` (dark grey) to 1 (as bright as a
  face square to the light: no shading).

- `shadow: f64`

  How dark cast shadows are: 0 (no visible shadow) to 1 (black).
  In the shaded style a shadow's grey is `1 - shadow` (never lighter
  than the face it falls on); the ground's and the other styles'
  shadows grow from none to black with it, their defaults at the default.


**Methods**

- `pub const MIN_AMBIENT: f64`
- `pub const MAX_AMBIENT: f64`
- `pub const DEFAULT_AMBIENT: f64`
- `pub const MIN_SHADOW: f64`
- `pub const MAX_SHADOW: f64`
- `pub const DEFAULT_SHADOW: f64`

  The intensity whose shaded shadow is `VisualStyle::SHADED_SHADOW`.

- `pub fn is_default(&self) -> bool`
- `pub fn is_valid(&self) -> bool`

  Both values are numbers within their ranges.

- `pub fn repair(&mut self) -> usize`

  Bring the values into range (the defaults when not numbers);
  returns how many it fixed.

- `pub fn clamped(mut self) -> Self`

  The values brought into range (see `Self::repair`).

- `pub fn grey(&self, cos: f64) -> f64`

  The shaded style's grey, as displayed (0 black, 1 white), of a face
  whose unit normal has the cosine `cos` with the direction towards the
  light: white square to it, `Self::ambient` edge-on or turned away;
  not a number reads as turned away.

- `pub fn shadow_grey(&self, lit: f64) -> f64`

  The grey of a cast shadow in the shaded style on a face whose grey
  is `lit`: `1 - shadow`, never lighter than the face.

- `pub fn darkening(&self, at_default: f64) -> f64`

  A shadow's darkening (0 none, 1 black) that grows with
  `Self::shadow` from none to black and is `at_default` at the
  default intensity (`at_default` between 0 and 1, exclusive): for
  the shadows whose darkness is not the shaded style's grey (the
  ground's, the unlit styles').


<a id="cc_core-view-visualstyle"></a>

### enum `VisualStyle`

```rust
pub enum VisualStyle
```

How a view shows the model's surfaces.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Wireframe`

  Every edge, seen through the model; no faces.

- `HiddenLine`

  Faces in the paper's color hide what is behind them; only edges show.

- `Shaded`

  White faces with the hidden line's edges, shaded grey by the light
  (the view's sun, else a light from the viewer's side; see
  `VisualStyle::shaded_grey`) and, where the view's sun casts
  shadows, a darker grey in them; no material colors.

- `ConsistentColors`

  Faces in their material's color, the same on every side (no light).

- `Textures`

  Faces in their material's appearance color with its surface pattern, lit.


**Methods**

- `pub const ALL: [VisualStyle; 5]`
- `pub fn key(self) -> &'static str`
- `pub fn name(self) -> &'static str`

  The name commands use: `wireframe`, `hidden_line`, `shaded`, `consistent_colors` or `textures`.

- `pub fn from_name(name: &str) -> Option<VisualStyle>`
- `pub fn fills(self) -> bool`

  Faces are filled (every style but the wireframe).

- `pub fn colored(self) -> bool`

  Faces show more than the paper's white: grey shading (shaded), their
  material's color (consistent colors) or appearance (textures).

- `pub const SHADED_AMBIENT: f64`

  The grey of a face lit in the shaded style where nothing casts a
  shadow on it, as displayed (0 black, 1 white).

- `pub const SHADED_DIFFUSE: f64`

  How much a face square to the light adds to `Self::SHADED_AMBIENT`: it is white.

- `pub const SHADED_SHADOW: f64`

  The grey of a cast shadow in the shaded style: darker than a face
  turned away from the light, so shadows read apart from shade.

- `pub fn shaded_grey(cos: f64) -> f64`

  The shaded style's grey, as displayed (0 black, 1 white), of a face
  whose unit normal has the cosine `cos` with the direction towards the
  light: white square to it, `Self::SHADED_AMBIENT` (a mid grey, never
  black) edge-on or turned away; not a number reads as turned away.
  The [default shading](ShadingSettings::grey)'s; a view's own is `View::shading`.


<a id="cc_core-view-detaillevel"></a>

### enum `DetailLevel`

```rust
pub enum DetailLevel
```

How much a view draws of elements: at `Coarse`, layered walls, floors
and roofs are cut as one region with a plain fill (no layers, no
material patterns); at `Medium` and `Fine` they show their layers and
patterns. Component pieces show at the levels they are set for
(`crate::component::DetailLevels`), so a component can draw a simple
shape at `Coarse` and its details at `Fine`.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Coarse`
- `Medium`
- `Fine`

**Methods**

- `pub const ALL: [DetailLevel; 3]`
- `pub fn key(self) -> &'static str`
- `pub fn name(self) -> &'static str`

  The name commands use: `coarse`, `medium` or `fine`.

- `pub fn is_medium(&self) -> bool`
- `pub fn shows_layers(self) -> bool`

  Layered elements show their layers and material patterns.


<a id="cc_core-view-max_section_segment"></a>

### const `MAX_SECTION_SEGMENT`

```rust
pub const MAX_SECTION_SEGMENT: f64
```

Longest piece of a split section line kept next to a head, meters.

<a id="cc_core-view-mepdiscipline"></a>

### enum `MepDiscipline`

```rust
pub enum MepDiscipline
```

The building services shown by a services plan.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Serialize`, `Deserialize`

**Variants**

- `Plumbing`

  Water supply, drainage, their valves and fixtures.

- `Electrical`

  Conduits, cable trays, outlets, switches, lights and boards.

- `Communications`

  Structured cabling: cable trays, conduits carrying it, data outlets and racks.

- `Lightning`

  Lightning protection: air terminals, conductors and earth electrodes.


**Methods**

- `pub const ALL: [MepDiscipline; 4]`
- `pub fn name(self) -> &'static str`

  Name in commands (`plumbing`, `electrical`, `communications`, `lightning`).

- `pub fn key(self) -> &'static str`

<a id="cc_core-view-underlay"></a>

### struct `Underlay`

```rust
pub struct Underlay
```

Other levels' plans drawn halftone under a floor or ceiling plan, for
reference while drawing: a plan below and one above, each seen looking
down (as a floor plan) or up (as a ceiling plan). They are not picked,
selected or snapped to, and are printed only when `printed`.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `below: Option<UnderlayPlan>`

  The plan of a level below.

- `above: Option<UnderlayPlan>`

  The plan of a level above.

- `printed: bool`

  Show it on sheets and in exports too (by default only on screen).


**Methods**

- `pub fn is_empty(&self) -> bool`
- `pub fn plans(&self, levels: &[crate::Level], level: LevelId) -> Vec<(LevelId, Looking)>`

  The levels shown under a plan of `level` and how they are seen,
  lowest first: the next level is the nearest below (or above) by
  elevation, a given level is shown when it still exists.

- `pub fn repair(&mut self, exists: impl Fn(LevelId) -> bool) -> usize`

  Drop references to levels that no longer exist; returns how many.


<a id="cc_core-view-underlayplan"></a>

### struct `UnderlayPlan`

```rust
pub struct UnderlayPlan
```

One plan shown under a view: which level, and from which side.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`, `Serialize`, `Deserialize`

**Fields**

- `level: UnderlayLevel`
- `looking: Looking`

<a id="cc_core-view-underlaylevel"></a>

### enum `UnderlayLevel`

```rust
pub enum UnderlayLevel
```

The level of an underlay: the next one (below or above the view's
level, so a template works for every level), or a given level.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Next`
- `Level(LevelId)`

<a id="cc_core-view-looking"></a>

### enum `Looking`

```rust
pub enum Looking
```

Which way a plan looks: down (a floor plan) or up (a ceiling plan).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `Down`
- `Up`

**Methods**

- `pub const ALL: [Looking; 2]`
- `pub fn key(self) -> &'static str`

  The translation key of its name.


<a id="mod-cc_core-wall_framing"></a>

## `cc_core::wall_framing`

<sub>`crates/cc_core/src/wall_framing.rs`</sub>

Light steel and wood framing of walls: a host element attached to one
straight wall that owns ordinary columns (studs, king and jack studs,
cripples) and beams (tracks or plates, headers, sills, blocking) laid
out from its parameters, its wall and the doors and windows the wall
hosts. They are kept in step like a steel building's members (see
`crate::member_host`), and any change of the wall or its openings
regenerates them.
The wall frame: `u` along the wall's body centerline
(`Project::wall_axis`) from its start to its length `L`, heights above
the wall's level.

**Items:** [`MAX_MEMBERS`](#cc_core-wall_framing-max_members) · [`MIN_PIECE`](#cc_core-wall_framing-min_piece) · [`END_STUD`](#cc_core-wall_framing-end_stud) · [`SPACING`](#cc_core-wall_framing-spacing) · [`LIGHT_STEEL_SPACING`](#cc_core-wall_framing-light_steel_spacing) · [`WOOD_SPACING`](#cc_core-wall_framing-wood_spacing) · [`FramingSystem`](#cc_core-wall_framing-framingsystem) · [`FramingRoleKind`](#cc_core-wall_framing-framingrolekind) · [`FramingRole`](#cc_core-wall_framing-framingrole) · [`FramingMember`](#cc_core-wall_framing-framingmember) · [`WallFraming`](#cc_core-wall_framing-wallframing) · [`host_type`](#cc_core-wall_framing-host_type)

<a id="cc_core-wall_framing-max_members"></a>

### const `MAX_MEMBERS`

```rust
pub const MAX_MEMBERS: usize
```

Most members one framing makes; the layout stops there.

<a id="cc_core-wall_framing-min_piece"></a>

### const `MIN_PIECE`

```rust
pub const MIN_PIECE: f64
```

A jack, cripple or blocking piece shorter than this is left out, meters.

<a id="cc_core-wall_framing-end_stud"></a>

### const `END_STUD`

```rust
pub const END_STUD: u16
```

The index of the stud at the wall's end (its key does not depend on the length).

<a id="cc_core-wall_framing-spacing"></a>

### const `SPACING`

```rust
pub const SPACING: RangeInclusive<f64>
```

<a id="cc_core-wall_framing-light_steel_spacing"></a>

### const `LIGHT_STEEL_SPACING`

```rust
pub const LIGHT_STEEL_SPACING: f64
```

<a id="cc_core-wall_framing-wood_spacing"></a>

### const `WOOD_SPACING`

```rust
pub const WOOD_SPACING: f64
```

<a id="cc_core-wall_framing-framingsystem"></a>

### enum `FramingSystem`

```rust
pub enum FramingSystem
```

How the wall is framed.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Serialize`, `Deserialize`

**Variants**

- `LightSteel`

  Cold-formed steel studs between tracks.

- `Wood`

  Sawn timber studs between plates.


**Methods**

- `pub const ALL: [FramingSystem; 2]`
- `pub fn code(self) -> &'static str`
- `pub fn from_code(code: &str) -> Option<Self>`
- `pub fn key(self) -> &'static str`

  The i18n key of its name.

- `pub fn default_spacing(self) -> f64`
- `pub fn default_double_top(self) -> bool`

  Wood frames get a double top plate by default.


<a id="cc_core-wall_framing-framingrolekind"></a>

### enum `FramingRoleKind`

```rust
pub enum FramingRoleKind
```

What a member is in its framing.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Serialize`, `Deserialize`

**Variants**

- `BottomTrack`
- `TopTrack`
- `Stud`
- `KingStud`
- `JackStud`
- `Header`
- `Sill`
- `Cripple`
- `Blocking`

**Methods**

- `pub const ALL: [FramingRoleKind; 9]`
- `pub fn is_stud(self) -> bool`

  Studs are columns; tracks, headers, sills and blocking are beams.


<a id="cc_core-wall_framing-framingrole"></a>

### struct `FramingRole`

```rust
pub struct FramingRole
```

A member's place in its framing:

- tracks: index 0 (the top plates from the topmost down);
- studs: index `k` for the stud at `k · spacing`, 0 at the wall start, `END_STUD` at its end;
- king and jack studs, headers, sills: `opening` is the opening's order along the wall, index 0 on the start side, 1 on the end side;
- cripples: index `2k` above the header and `2k + 1` below the sill;
- blocking: keyed by its left stud, `(0, k)` after stud `k` and `(j + 1, 0)`/`(j + 1, 1)` after opening `j`'s end-side/start-side king.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Serialize`, `Deserialize`

**Fields**

- `kind: FramingRoleKind`
- `opening: u16`
- `index: u16`

**Methods**

- `pub fn fits(self, kind: &ElementKind) -> bool`
- `pub fn key(self) -> RoleKey`

  Its key for the shared member-host code (same order as the role).

- `pub fn from_key(key: RoleKey) -> Option<Self>`

<a id="cc_core-wall_framing-framingmember"></a>

### struct `FramingMember`

```rust
pub struct FramingMember
```

A member the framing manages.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Serialize`, `Deserialize`

**Fields**

- `role: FramingRole`
- `id: ElementId`

<a id="cc_core-wall_framing-wallframing"></a>

### struct `WallFraming`

```rust
pub struct WallFraming
```

**Traits:** `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`, `MemberHost`

**Fields**

- `wall: ElementId`

  The framed wall (straight; one framing per wall).

- `system: FramingSystem`
- `stud_type: ElementTypeId`

  Column type of the studs, king and jack studs and cripples.

- `track_type: ElementTypeId`

  Beam type of the bottom and top tracks or plates, and of the blocking.

- `header_type: ElementTypeId`

  Beam type of the headers and sills.

- `spacing: f64`

  Stud spacing on center from the wall start, meters.

- `double_top: bool`

  An extra top plate (wood only; ignored for light steel).

- `blocking: bool`

  A row of blocking at mid-height between full-height studs.

- `members: Vec<FramingMember>`
- `detached: Vec<FramingRole>`

**Methods**

- `pub fn new(wall: ElementId, system: FramingSystem, stud_type: ElementTypeId, track_type: ElementTypeId, header_type: ElementTypeId) -> Self`

  A framing of `wall` with the system's default spacing and top plates, no blocking.

- `pub fn top_plates(&self) -> u16`

  Top tracks or plates: two for a wood frame with `double_top`, else one.

- `pub fn check(&self) -> Result<(), String>`

  Whether the spacing is in range.

- `pub fn check_types(&self, project: &Project) -> Result<(), String>`

  Whether the stud type is a column type and the track and header types beam types.

- `pub fn check_wall(&self, project: &Project) -> Result<(), String>`

  Whether its wall is a wall of the project.

- `pub fn reset_out_of_range(&mut self) -> usize`

  Put an out-of-range spacing back to the system's default; returns how many (0 or 1).

- `pub fn type_ids(&self) -> impl Iterator<Item = ElementTypeId> + '_`

  The member types it uses.

- `pub fn wall_length(&self, project: &Project) -> f64`

  Its wall's centerline length (0 when the wall is gone).

- `pub fn layout(&self, project: &Project) -> Vec<(FramingRole, ElementTypeId, ElementKind)>`

  Its members: role, type and element. Empty when the spacing is out of
  range or the wall is gone, curved, shorter than two studs or too low
  for its tracks, and when the members would be more than
  `MAX_MEMBERS` (see `WallFraming::check_size`): never cut short.

- `pub fn check_size(&self, project: &Project) -> Result<(), String>`

  Whether the members fit in `MAX_MEMBERS`.


<a id="cc_core-wall_framing-host_type"></a>

### fn `host_type`

```rust
pub fn host_type(id: ElementTypeId, name: String) -> ElementType
```

The type a project gets with its first wall framing.

<a id="mod-cc_core-wall_ops"></a>

## `cc_core::wall_ops`

<sub>`crates/cc_core/src/wall_ops.rs`</sub>

Geometry edits of linear elements (walls, beams, grids): trim or extend
to a corner or to a reference line, split, offset and align. Arc walls
take part on their circle: they trim and extend along it, and meet lines
and other arcs where the circles cross.
Picks are plan points on the element: the picked part is the part kept.
Detail lines take part too, through their curve and
`crate::sketch_edit`: they trim and extend to other detail lines and
to the lines of walls, beams and grids, split and offset.

**Items:** [`Path`](#cc_core-wall_ops-path) · [`path_of`](#cc_core-wall_ops-path_of) · [`curve_of`](#cc_core-wall_ops-curve_of) · [`line_of`](#cc_core-wall_ops-line_of) · [`connected_ends`](#cc_core-wall_ops-connected_ends) · [`EditError`](#cc_core-wall_ops-editerror) · [`path`](#cc_core-wall_ops-path)

<a id="cc_core-wall_ops-path"></a>

### enum `Path`

```rust
pub enum Path
```

Where a linear element runs: a straight line, or an arc (arc walls and arc beams).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Variants**

- `Line(DVec2, DVec2)`
- `Arc(Arc)`

<a id="cc_core-wall_ops-path_of"></a>

### fn `path_of`

```rust
pub fn path_of(e: &Element) -> Option<Path>
```

The path of a linear element, if it has one.

<a id="cc_core-wall_ops-curve_of"></a>

### fn `curve_of`

```rust
pub fn curve_of(e: &Element) -> Option<SketchCurve>
```

The curve of an element as a sketch curve: a detail line's own, or the
location line of a wall or beam (an arc for arc walls and arc beams) or grid.

<a id="cc_core-wall_ops-line_of"></a>

### fn `line_of`

```rust
pub fn line_of(e: &Element) -> Option<(DVec2, DVec2)>
```

Location line of a linear element, if it has one.

<a id="cc_core-wall_ops-connected_ends"></a>

### fn `connected_ends`

```rust
pub fn connected_ends(project: &Project, id: ElementId, point: DVec2) -> Vec<(ElementId, bool)>
```

Ends of other walls on the same level that meet `point` (within 1 cm),
as `(wall, is_start)`.

<a id="cc_core-wall_ops-editerror"></a>

### enum `EditError`

```rust
pub enum EditError
```

Why an edit could not be made.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Variants**

- `NotLinear`

  The element has no location line (or does not exist).

- `Parallel`

  The lines are parallel.

- `Degenerate`

  The result would have no length.

- `Curved`

  The edit needs a straight line and the wall is an arc.


<a id="cc_core-wall_ops-path"></a>

### fn `path`

```rust
pub fn path(project: &Project, id: ElementId) -> Result<Path, EditError>
```

The path of an element (line or arc).

{% endraw %}
