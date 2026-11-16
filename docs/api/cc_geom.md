# `cc_geom` API

{% raw %}
[← API reference](index.md) · Generated from the source by `cargo run -p api_doc`; edit the doc comments, not this page.

Derived geometry for ConstructCraft.
`ModelGeometry::build` turns a `cc_core::Project` into boundary
representation `Solid`s (planar polygon faces) that feed the 3D renderer,
elevations and sections, plus `WallGeom` and `BeamGeom` data used by plan views.

## Modules

| Module | Items | Summary |
|---|---|---|
| [`cc_geom`](#mod-cc_geom) | 0 | Derived geometry for ConstructCraft. |
| [`cc_geom::analytical`](#mod-cc_geom-analytical) | 5 | The analytical model: columns and beams as center lines (members) between shared points (nodes), for structural analysis programs. |
| [`cc_geom::bar_marks`](#mod-cc_geom-bar_marks) | 9 | Bar marks: the mark of every bar of every reinforcement set (see `cc_core::bar_marks`), resolved by a pure, deterministic function of the project and its bars. |
| [`cc_geom::bar_shape`](#mod-cc_geom-bar_shape) | 3 | The shape of a bar as a bending schedule describes it: open or closed, its legs in order with their lengths, the radii and turns of arc legs and the bends between legs, read from the bar's center line. |
| [`cc_geom::beam`](#mod-cc_geom-beam) | 7 | Structural framing joins: a beam end inside a structural column stops at the column's face, and beam ends meet other beams the way walls do (a mitered L corner, a butt T against the side of a beam, a collinear pair running through a junction of three or more). |
| [`cc_geom::build`](#mod-cc_geom-build) | 14 |  |
| [`cc_geom::component`](#mod-cc_geom-component) | 6 | Component pieces as solids: the definition's pieces resolved with their parameter values, and the void pieces cut out of the solid pieces they overlap. |
| [`cc_geom::connection`](#mod-cc_geom-connection) | 17 | Steel and timber connections (see `cc_core::connection`): the plates and bolts of each, derived from its member's geometry on every build, so they follow the member. |
| [`cc_geom::coplanar`](#mod-cc_geom-coplanar) | 1 | Coplanar faces of an imported mesh merged into polygons: the triangles a file splits a flat face into come back as one face, so views draw no diagonals across it and the piece holds fewer faces. |
| [`cc_geom::csg`](#mod-cc_geom-csg) | 3 | Boolean operations on closed polygon meshes with binary space partitioning trees (the classic BSP-tree CSG algorithm: each solid's polygons are clipped against the other's tree). |
| [`cc_geom::curtain`](#mod-cc_geom-curtain) | 6 | Curtain walls in the world: the panels and mullions of a wall's grid (see `cc_core::curtain`). |
| [`cc_geom::cut`](#mod-cc_geom-cut) | 2 | Plane cuts used by sections and plan views. |
| [`cc_geom::faces`](#mod-cc_geom-faces) | 8 | The faces of a column's section, a given distance inside it: where its longitudinal bars stand and its stirrups run. |
| [`cc_geom::level_mark`](#mod-cc_geom-level_mark) | 2 | The levels a level mark writes: measured from the floors under its point unless the mark sets them. |
| [`cc_geom::lightning`](#mod-cc_geom-lightning) | 4 | Lightning protection in the model: where an air terminal stands on a roof, and a simplified coverage check of the roofs by the air termination (air terminals and roof conductors). |
| [`cc_geom::mass`](#mod-cc_geom-mass) | 4 | Conceptual masses: the body of a mass (an extrusion, or a loft between two outlines) and its mass floors, the sections at the levels it lists. |
| [`cc_geom::mep`](#mod-cc_geom-mep) | 12 | Pipes, conduits and cable trays: each pipe or conduit segment is a cylinder of its type's diameter along its center line, each cable tray an open channel (or a ladder) of its type's section around it, colored by its system. |
| [`cc_geom::mep_net`](#mod-cc_geom-mep_net) | 7 | The networks formed by pipes, conduits and cable trays: where runs meet (their ends within a centimeter, or an end on another run), the fittings there, the valves on pipes (each a node on its pipe's run) and the fixtures and devices attached to the networks' nodes. |
| [`cc_geom::mesh`](#mod-cc_geom-mesh) | 5 | Plan drawings of polyhedral meshes (component pieces imported from IFC): the edges seen from above and the section at the cut plane. |
| [`cc_geom::pad`](#mod-cc_geom-pad) | 10 | Building pads (`cc_core::pad`) graded into their terrain's surface. |
| [`cc_geom::paint`](#mod-cc_geom-paint) | 9 | Painted faces on the built solids: finding the face a paint (`cc_core::paint::FacePaint`) refers to after a rebuild, and the surface it covers. |
| [`cc_geom::pile`](#mod-cc_geom-pile) | 7 | Piles and pile caps (see `cc_core::pile`): their bodies, plans and exact quantities. |
| [`cc_geom::plan_cover`](#mod-cc_geom-plan_cover) | 1 | Hidden lines of plans. |
| [`cc_geom::polygon`](#mod-cc_geom-polygon) | 15 | 2D polygon helpers. |
| [`cc_geom::profile_sweep`](#mod-cc_geom-profile_sweep) | 2 | The solid of a swept profile: its type's profile carried along its path in plan, at the level's elevation plus its base offset, mitered at the corners (and around a closed path). |
| [`cc_geom::ramp`](#mod-cc_geom-ramp) | 4 | Ramp geometry: the sloped flights and flat landings of a ramp element laid out from its shape, its slab solids and the paths its railings follow. |
| [`cc_geom::rebar`](#mod-cc_geom-rebar) | 9 | Reinforcing bars: the center lines of each set of bars in its beam or column, from the host's body, the cover and the bar diameters. |
| [`cc_geom::region`](#mod-cc_geom-region) | 2 | Regions covered by polygons: the union of simple polygons in a plane, optionally less what other polygons cover and kept within another polygon, as rings a painter can fill. |
| [`cc_geom::roof_framing`](#mod-cc_geom-roof_framing) | 5 | Roof framing: the purlins, rafters and battens under a roof's sloped faces, from its type's framing and covering (see `cc_core::roofing`). |
| [`cc_geom::roofs`](#mod-cc_geom-roofs) | 10 | Pitched roofs (gable and hip) over any footprint, courtyards included: every sloped eave raises a plane and every gable edge a vertical plane; the weighted straight skeleton (`crate::skeleton`) gives the part of each plane over the footprint, with its hips, valleys and ridges. |
| [`cc_geom::rooms`](#mod-cc_geom-rooms) | 3 | Room boundaries: the region enclosed by room-bounding walls around a point. |
| [`cc_geom::section`](#mod-cc_geom-section) | 7 | Cross-sections of columns and beams: type sections at each detail level, placed column bodies with smooth arc facets, and the legacy section shapes as rectangles and outlines (legends and IFC export). |
| [`cc_geom::sizing`](#mod-cc_geom-sizing) | 9 | Sizing of building services from their networks, with simplified tables of common Brazilian practice (from public sources): |
| [`cc_geom::skeleton`](#mod-cc_geom-skeleton) | 4 | Weighted straight skeleton of a polygon with holes, for roofs. |
| [`cc_geom::slab_rebar`](#mod-cc_geom-slab_rebar) | 4 | Slab reinforcement: the bars of floors and foundation slabs. |
| [`cc_geom::solid`](#mod-cc_geom-solid) | 8 |  |
| [`cc_geom::stair`](#mod-cc_geom-stair) | 15 | Stair geometry: the treads, landings and winders of a stair element laid out from its shape, the solids of its construction (solid, cast in place, assembled) and the paths its railings follow. |
| [`cc_geom::strip`](#mod-cc_geom-strip) | 1 | Arc strip footings: the body of a strip (a ground beam) that follows an arc, a band between concentric arcs either side of its centerline, its ends cut where it meets other strips. |
| [`cc_geom::strip_rebar`](#mod-cc_geom-strip_rebar) | 2 | Strip footing reinforcement: bars across the footing slab (radial in an arc strip), bars along it (ring bars), and the bars of a beam for the beam on top of a haunched strip (or the whole section of a plain one). |
| [`cc_geom::topography`](#mod-cc_geom-topography) | 11 | Terrain surfaces (`cc_core::topography`): a constrained Delaunay triangulation (`spade`) of the measured points inside the boundary, with the boundary's edges as constraints, so non-convex boundaries keep no triangle outside. |
| [`cc_geom::wall`](#mod-cc_geom-wall) | 12 |  |
| [`cc_geom::wall_rebar`](#mod-cc_geom-wall_rebar) | 4 | Wall reinforcement: horizontal bars (ring bars in an arc wall) and vertical bars near each face of a wall's structure, and ties between the faces. |

<a id="mod-cc_geom"></a>

## `cc_geom`

<sub>`crates/cc_geom/src/lib.rs`</sub>

**Re-exports**

- `pub use beam::{BeamEnd, BeamGeom}`
- `pub use build::{ BoxPart, FRAME_COLOR, FoundationGeom, HostedPart, ModelGeometry, OpeningSymbol, WorldConnector, column_outline, component_connectors, component_parts, hosted_parts, opening_parts, rect_outline, }`
- `pub use component::{instance_errors, resolve_component, resolve_component_in, resolve_instance, resolve_pieces}`
- `pub use cut::{clip_polygon, face_plane_segments}`
- `pub use mep::{SegmentGeom, TraySection, ValveGeom, segment_geom, valve_geom}`
- `pub use mesh::{MeshPlan, mesh_plan, plan_edges, top_edges}`
- `pub use plan_cover::PlanCover`
- `pub use ramp::{RampGeom, RampPiece, ramp_geom}`
- `pub use rooms::{RoomArrangement, outer_boundary, room_boundary}`
- `pub use solid::{Face, Look, Ray, Solid, plane_axes}`
- `pub use stair::{ Flight, LandingRect, Pieces, StairGeom, StairLanding, StairStep, StepKind, add_railing, railing_length, railing_path, railing_pieces, stair_geom, }`
- `pub use strip::StripArc`
- `pub use wall::{Column, LayerGap, Opening, RECESS_MIN_BACK, Recess, WallGeom}`
- `pub use glam::{DVec2, DVec3}`

<a id="mod-cc_geom-analytical"></a>

## `cc_geom::analytical`

<sub>`crates/cc_geom/src/analytical.rs`</sub>

The analytical model: columns and beams as center lines (members) between
shared points (nodes), for structural analysis programs. A beam framed
into a column, or running on over one it passes, ends on the column's
axis, a beam ending on another beam ends on that beam's line, a column
standing on or under a beam (a truss's vertical between its chords) ends
on the beam's line, and a member is split wherever another member ends
on it, so connected members share their nodes.

**Items:** [`MemberKind`](#cc_geom-analytical-memberkind) · [`Node`](#cc_geom-analytical-node) · [`Member`](#cc_geom-analytical-member) · [`AnalyticalModel`](#cc_geom-analytical-analyticalmodel) · [`analytical_model`](#cc_geom-analytical-analytical_model)

<a id="cc_geom-analytical-memberkind"></a>

### enum `MemberKind`

```rust
pub enum MemberKind
```

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Variants**

- `Column`
- `Beam`

<a id="cc_geom-analytical-node"></a>

### struct `Node`

```rust
pub struct Node
```

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `id: usize`
- `position: DVec3`
- `support: bool`

  The base of a column: where the structure rests on its supports.


<a id="cc_geom-analytical-member"></a>

### struct `Member`

```rust
pub struct Member
```

A straight member between two nodes; an element split by connections
gives several members.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `id: usize`
- `element: ElementId`
- `kind: MemberKind`
- `start: usize`
- `end: usize`

<a id="cc_geom-analytical-analyticalmodel"></a>

### struct `AnalyticalModel`

```rust
pub struct AnalyticalModel
```

**Traits:** `Debug`, `Clone`, `Default`, `PartialEq`

**Fields**

- `nodes: Vec<Node>`
- `members: Vec<Member>`

**Methods**

- `pub fn line(&self, member: &Member) -> Option<(DVec3, DVec3)>`

  The two end points of a member.


<a id="cc_geom-analytical-analytical_model"></a>

### fn `analytical_model`

```rust
pub fn analytical_model(project: &Project, geom: &ModelGeometry) -> AnalyticalModel
```

Build the analytical model of the structural columns and the beams.

<a id="mod-cc_geom-bar_marks"></a>

## `cc_geom::bar_marks`

<sub>`crates/cc_geom/src/bar_marks.rs`</sub>

Bar marks: the mark of every bar of every reinforcement set (see
`cc_core::bar_marks`), resolved by a pure, deterministic function of
the project and its bars.
A mark names one shape: the same diameter, steel grade and
`ShapeSummary` within the project's length tolerance and
`ANGLE_TOLERANCE_DEG` (read from either end, mirrored, a closed bar
from any corner). In order:

1. a set with a `bar_mark` override gives all its bars that mark (bars of
   different shapes make one `Mark` each, of the same text);
2. a bar whose shape matches an entry of the frozen marks table takes the
   first such entry's mark;
3. the shapes left are numbered with the prefix after the highest frozen
   number (from `start`, skipping marks already used), in the stable order
   partition, diameter descending, length descending, shape; a bar joins
   the first numbered shape it matches.
The marks come sorted naturally (N2 before N10), each with its count,
total length (the exact sum of its bars) and weight.

**Items:** [`MarkSource`](#cc_geom-bar_marks-marksource) · [`Mark`](#cc_geom-bar_marks-mark) · [`BarMarks`](#cc_geom-bar_marks-barmarks) · [`resolve`](#cc_geom-bar_marks-resolve) · [`natural`](#cc_geom-bar_marks-natural) · [`canonical`](#cc_geom-bar_marks-canonical) · [`to_mark_shape`](#cc_geom-bar_marks-to_mark_shape) · [`from_mark_shape`](#cc_geom-bar_marks-from_mark_shape) · [`resolve_with`](#cc_geom-bar_marks-resolve_with)

<a id="cc_geom-bar_marks-marksource"></a>

### enum `MarkSource`

```rust
pub enum MarkSource
```

Where a mark comes from.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`

**Variants**

- `Frozen(usize)`

  An entry of the frozen marks table (its index).

- `Override`

  A set's `bar_mark` override.

- `Numbered`

  Numbered for a shape no frozen entry holds.


<a id="cc_geom-bar_marks-mark"></a>

### struct `Mark`

```rust
pub struct Mark
```

One mark: a text naming bars of one diameter, grade and shape.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `mark: String`
- `diameter: f64`
- `grade: String`
- `shape: ShapeSummary`

  The shape, in its canonical form (see `canonical`).

- `source: MarkSource`
- `first: Option<(ElementId, usize)>`

  The first bar of the mark (set, index among its bars); none for a
  frozen entry no bar uses.

- `count: usize`

  How many bars have the mark.

- `total_length: f64`

  Their lengths added up, meters.

- `weight: f64`

  Their weight, kilograms (each set's mass per meter).

- `longest: f64`

  The longest of them, meters.

- `sets: Vec<ElementId>`

  The sets with bars of the mark, in id order.


**Methods**

- `pub fn unit_length(&self) -> f64`

  Mean length of one bar (the shape's when no bar uses the mark).

- `pub fn frozen(&self) -> bool`

<a id="cc_geom-bar_marks-barmarks"></a>

### struct `BarMarks`

```rust
pub struct BarMarks
```

The marks of a project's bars.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `marks: Vec<Mark>`

  Sorted naturally by mark, then diameter (largest first).

- `bars: BTreeMap<ElementId, Vec<usize>>`

  Per set: the index in `marks` of each of its bars, in the bars' order.


**Methods**

- `pub fn of_bar(&self, set: ElementId, index: usize) -> Option<&Mark>`

  The mark of bar `index` of set `set`.

- `pub fn of_set(&self, set: ElementId) -> Vec<&Mark>`

  The marks of set `set`'s bars, each once, in the order its bars first have them.


<a id="cc_geom-bar_marks-resolve"></a>

### fn `resolve`

```rust
pub fn resolve(project: &Project, rebar: &BTreeMap<ElementId, RebarGeom>) -> BarMarks
```

The marks of `rebar` (the model's bars, `ModelGeometry::rebar`) with the
project's marking and the sets' overrides.

<a id="cc_geom-bar_marks-natural"></a>

### fn `natural`

```rust
pub fn natural(a: &str, b: &str) -> Ordering
```

Natural order of marks: runs of digits compare as numbers (N2 before N10).

<a id="cc_geom-bar_marks-canonical"></a>

### fn `canonical`

```rust
pub fn canonical(s: &ShapeSummary) -> ShapeSummary
```

The canonical reading of a shape: of the readings from either end,
mirrored and (closed bars of up to 64 legs) from any corner, the one with
the least `key`. Shapes that match read the same, up to the tolerance.

<a id="cc_geom-bar_marks-to_mark_shape"></a>

### fn `to_mark_shape`

```rust
pub fn to_mark_shape(s: &ShapeSummary) -> Option<MarkShape>
```

A shape as the frozen marks table keeps it (turns in degrees); none for
shapes of more than `MAX_MARK_LEGS` legs or that are not usable.

<a id="cc_geom-bar_marks-from_mark_shape"></a>

### fn `from_mark_shape`

```rust
pub fn from_mark_shape(m: &MarkShape) -> ShapeSummary
```

The summary of a frozen shape.

<a id="cc_geom-bar_marks-resolve_with"></a>

### fn `resolve_with`

```rust
pub fn resolve_with(project: &Project, rebar: &BTreeMap<ElementId, RebarGeom>, marking: &BarMarking, overrides: bool) -> BarMarks
```

`resolve` with `marking` instead of the project's, and the sets'
overrides left out unless `overrides`.

<a id="mod-cc_geom-bar_shape"></a>

## `cc_geom::bar_shape`

<sub>`crates/cc_geom/src/bar_shape.rs`</sub>

The shape of a bar as a bending schedule describes it: open or closed,
its legs in order with their lengths, the radii and turns of arc legs
and the bends between legs, read from the bar's center line. Independent
of where the bar lies, so bars of one shape compare equal wherever they
are (`ShapeSummary::matches`).

**Items:** [`LegSummary`](#cc_geom-bar_shape-legsummary) · [`ShapeSummary`](#cc_geom-bar_shape-shapesummary) · [`summary`](#cc_geom-bar_shape-summary)

<a id="cc_geom-bar_shape-legsummary"></a>

### struct `LegSummary`

```rust
pub struct LegSummary
```

One leg of a `ShapeSummary`.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `length: f64`

  Length along the leg, meters.

- `radius: Option<f64>`

  Center-line radius of an arc leg; none for a straight one.

- `sweep: f64`

  Angle an arc leg turns through, radians (0 for a straight leg),
  signed like the bends.

- `bend: f64`

  Turn at the corner after this leg, radians: positive the way the bar
  first turns (along an arc or at a corner), negative the other way; 0
  after the last leg of an open bar (a closed bar's last leg ends at
  its closing corner). Bars that do not lie in a plane take the sign
  of the turn's axis against the first.

- `hook: bool`

  The leg is part of a hook.


<a id="cc_geom-bar_shape-shapesummary"></a>

### struct `ShapeSummary`

```rust
pub struct ShapeSummary
```

The shape of a bar: see the module.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `closed: bool`
- `legs: Vec<LegSummary>`

**Methods**

- `pub fn length(&self) -> f64`

  Length of the bar.

- `pub fn is_straight(&self) -> bool`

  Whether the bar is one straight leg.

- `pub fn reversed(&self) -> ShapeSummary`

  The same shape read from its other end.

- `pub fn matches(&self, other: &ShapeSummary, length_tol: f64, angle_tol: f64) -> bool`

  Whether `other` is the same shape: as many legs, of lengths within
  `length_tol` and radii alike, turns within `angle_tol`, read from
  either end, either way round (mirrored), and a closed bar from any
  corner.


<a id="cc_geom-bar_shape-summary"></a>

### fn `summary`

```rust
pub fn summary(bar: &Bar) -> ShapeSummary
```

The shape of `bar`, read from its center line. Points in a line make one
straight leg; equal chords turning evenly by a small angle make one arc
leg, with the radius and the length of the arc through their points. A
bar whose true length differs from its center line's (the hooks of a
tie, counted but not drawn) has the difference spread over its legs.

<a id="mod-cc_geom-beam"></a>

## `cc_geom::beam`

<sub>`crates/cc_geom/src/beam.rs`</sub>

Structural framing joins: a beam end inside a structural column stops at
the column's face, and beam ends meet other beams the way walls do (a
mitered L corner, a butt T against the side of a beam, a collinear pair
running through a junction of three or more).

**Items:** [`BeamEnd`](#cc_geom-beam-beamend) · [`BeamGeom`](#cc_geom-beam-beamgeom) · [`box_section`](#cc_geom-beam-box_section) · [`beam_section`](#cc_geom-beam-beam_section) · [`BeamInput`](#cc_geom-beam-beaminput) · [`ColumnInput`](#cc_geom-beam-columninput) · [`resolve_beams`](#cc_geom-beam-resolve_beams)

<a id="cc_geom-beam-beamend"></a>

### enum `BeamEnd`

```rust
pub enum BeamEnd
```

What a beam end is framed into.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Variants**

- `Free`

  Free: the end is square at its location point.

- `Column(ElementId)`

  Cut back to the face of a structural column.

- `Beam`

  Mitered or butted against other beams.


<a id="cc_geom-beam-beamgeom"></a>

### struct `BeamGeom`

```rust
pub struct BeamGeom
```

Resolved beam geometry: frame, section and the ends after joins.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `element: ElementId`
- `origin: DVec2`
- `dir: DVec2`
- `normal: DVec2`

  Left normal (+v).

- `length: f64`

  Length between the location points.

- `half: f64`
- `z_range: (f64, f64)`

  Bottom and top elevations at the start location point (`u = 0`).

- `slope: f64`

  Rise per meter along the axis (sloped beams).

- `section: SectionOutline`

  The section, square to the plan axis: `x` across the beam (`v`,
  centered on its box), `y` up from the bottom of the section.

- `curved: bool`

  Whether the section has arcs (their facets are drawn smooth).

- `start_ext: (f64, f64)`

  Axial extents `(left, right)` of the start/end faces after joins.

- `end_ext: (f64, f64)`
- `start: BeamEnd`
- `end: BeamEnd`
- `arc: Option<Arc>`

  The centerline arc of an arc beam or arc strip footing (`dir` and
  `normal` are then its frame at the start, `u` runs along the arc and
  `v` to its left). `Self::footprint` gives only the four corners:
  `Self::outline` follows the arc.


**Methods**

- `pub fn world(&self, u: f64, v: f64) -> DVec2`

  World plan position of beam-local `(u, v)`.

- `pub fn dir_at(&self, u: f64) -> DVec2`

  Unit direction of the axis at `u` (the tangent of an arc beam).

- `pub fn locate(&self, p: DVec2) -> f64`

  Position of the plan point `p` along the axis.

- `pub fn axis(&self) -> Vec<DVec2>`

  The axis between the location points: its two ends, or the chords of an arc.

- `pub fn local_footprint(&self) -> [DVec2; 4]`

  The footprint corners in beam-local `(u, v)`, counter-clockwise.

- `pub fn footprint(&self) -> [DVec2; 4]`

  Plan footprint of the body, counter-clockwise.

- `pub fn outline(&self) -> Vec<DVec2>`

  Plan outline of the body, counter-clockwise: the footprint, with the
  faces of an arc beam as short chords.

- `pub fn line(&self, v: f64) -> Vec<DVec2>`

  The line `v` across the body from its start face to its end face
  (chords along an arc): an edge of its section in plan.

- `pub fn cut_length(&self) -> f64`

  Length of the body along its (sloped) axis, the mean of its two sides.

- `pub fn stretch(&self) -> f64`

  Length along the slope per meter in plan.

- `pub fn z_at(&self, u: f64) -> (f64, f64)`

  Bottom and top elevations at `u` along the axis.

- `pub fn z_bounds(&self) -> (f64, f64)`

  Lowest and highest elevations of the body.

- `pub fn add_to(&self, s: &mut crate::Solid)`

  Add the body to `s`: the section swept between the start and end
  faces, following the slope. Faces are the cap triangles (start, then
  end, for each triangle), then one side per edge of each ring (per
  chord along an arc beam); sides on the arcs of a curved section, and
  the curved sides of an arc beam, are smooth. Nothing when the
  section makes no cap triangles.

- `pub fn section_at_start(&self) -> Vec<Vec<DVec3>>`

  The start cap in world space: the outline, then the holes (as
  `Self::add_to` winds them).

- `pub fn is_square(&self) -> bool`

  Whether both end faces are square to the axis.

- `pub fn volume(&self) -> f64`

  Volume of the body: each cap triangle swept between the end faces,
  exact since the faces' offsets are linear across the beam (and, along
  an arc, the path of each point grows linearly with its radius: the
  arcs as arcs, not as the chords of the solid).


<a id="cc_geom-beam-box_section"></a>

### fn `box_section`

```rust
pub(crate) fn box_section(width: f64, height: f64) -> SectionOutline
```

A `width` x `height` box section: centered across, up from 0.

<a id="cc_geom-beam-beam_section"></a>

### fn `beam_section`

```rust
pub(crate) fn beam_section(section: &SectionOutline) -> (SectionOutline, f64)
```

A type's section as a beam sweeps it: centered across on its box, its
bottom at 0 (`x` across, `y` up). Returns the section and its half width.

<a id="cc_geom-beam-beaminput"></a>

### struct `BeamInput`

```rust
pub(crate) struct BeamInput
```

A beam as seen by the join resolution.

**Traits:** `Debug`, `Clone`

**Fields**

- `element: ElementId`
- `start: DVec2`
- `end: DVec2`
- `half: f64`
- `z_range: (f64, f64)`

  Bottom and top elevations at the start.

- `slope: f64`
- `section: SectionOutline`
- `curved: bool`
- `join_start: bool`
- `join_end: bool`
- `bulge: f64`

  Bulge of the centerline: 0 when straight; arc beams and arc strip
  footings bend (see `cc_core::arc`).


<a id="cc_geom-beam-columninput"></a>

### struct `ColumnInput`

```rust
pub(crate) struct ColumnInput
```

A structural column as seen by the join resolution.

**Traits:** `Debug`, `Clone`

**Fields**

- `element: ElementId`
- `outline: Vec<DVec2>`

  Plan outline: the convex hull of its section, counter-clockwise.

- `z_range: (f64, f64)`

<a id="cc_geom-beam-resolve_beams"></a>

### fn `resolve_beams`

```rust
pub(crate) fn resolve_beams(beams: &[BeamInput], columns: &[ColumnInput]) -> Vec<BeamGeom>
```

Resolve the joins of every beam: column faces first, then beam to beam
among beams whose depths overlap.

<a id="mod-cc_geom-build"></a>

## `cc_geom::build`

<sub>`crates/cc_geom/src/build.rs`</sub>

**Items:** [`FRAME_COLOR`](#cc_geom-build-frame_color) · [`HostedPart`](#cc_geom-build-hostedpart) · [`opening_parts`](#cc_geom-build-opening_parts) · [`hosted_parts`](#cc_geom-build-hosted_parts) · [`OpeningSymbol`](#cc_geom-build-openingsymbol) · [`ModelGeometry`](#cc_geom-build-modelgeometry) · [`SolidIndex`](#cc_geom-build-solidindex) · [`FoundationGeom`](#cc_geom-build-foundationgeom) · [`BoxPart`](#cc_geom-build-boxpart) · [`column_outline`](#cc_geom-build-column_outline) · [`rect_outline`](#cc_geom-build-rect_outline) · [`component_parts`](#cc_geom-build-component_parts) · [`WorldConnector`](#cc_geom-build-worldconnector) · [`component_connectors`](#cc_geom-build-component_connectors)

<a id="cc_geom-build-frame_color"></a>

### const `FRAME_COLOR`

```rust
pub const FRAME_COLOR: [u8; 3]
```

Shading color of door and window frames.

<a id="cc_geom-build-hostedpart"></a>

### struct `HostedPart`

```rust
pub struct HostedPart
```

One box of a door or window, in opening-local coordinates: `x` along the
wall from the start jamb, `y` across the wall (positive on its left), `z`
up from the sill.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `min: DVec3`
- `max: DVec3`
- `panel: bool`

  Leaf or glazing (colored with the type material) rather than frame.


<a id="cc_geom-build-opening_parts"></a>

### fn `opening_parts`

```rust
pub fn opening_parts(project: &Project, element: &cc_core::Element, width: f64, height: f64, half: f64) -> Vec<cc_core::component::ComponentPart>
```

The pieces of the door or window `element` (of `width` x `height`, in a
wall of half thickness `half`) in opening-local coordinates like
`HostedPart`s: `x` along the wall from the start jamb, `y` across it
(positive on its left), `z` up from the sill. A door or window component
is resolved with the wall's thickness, centered in the opening and
mirrored by the element's flips; other types give their boxes, frames
with no material and panels of the type's material.

<a id="cc_geom-build-hosted_parts"></a>

### fn `hosted_parts`

```rust
pub fn hosted_parts(data: &TypeData, width: f64, height: f64, half: f64) -> Vec<HostedPart>
```

The boxes making up a door or window of `width` x `height` in a wall of
half thickness `half`. Empty for non-hosted types.

<a id="cc_geom-build-openingsymbol"></a>

### struct `OpeningSymbol`

```rust
pub struct OpeningSymbol
```

A window's elevation symbol on one face of its wall (see
`cc_core::WindowOperation::elevation_lines`): the opening's outline
just off the face, facing out along `normal`, and the symbol's lines on
it, drawn by elevations and sections where nothing stands in front.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `element: ElementId`
- `outline: [DVec3; 4]`
- `normal: DVec3`
- `lines: Vec<(DVec3, DVec3, cc_core::SymbolLine)>`

<a id="cc_geom-build-modelgeometry"></a>

### struct `ModelGeometry`

```rust
pub struct ModelGeometry
```

All derived geometry of a project.

**Traits:** `Debug`, `Clone`, `Default`

**Fields**

- `solids: Vec<Solid>`
- `walls: BTreeMap<ElementId, WallGeom>`
- `beams: BTreeMap<ElementId, BeamGeom>`

  Beams with their ends framed into columns and other beams.

- `rebar: BTreeMap<ElementId, crate::rebar::RebarGeom>`

  Reinforcement sets: their bars, placed in their hosts.

- `connections: BTreeMap<ElementId, crate::connection::ConnectionGeom>`

  Connections: their plates and bolts, from their members' geometry.

- `segments: BTreeMap<ElementId, crate::mep::SegmentGeom>`

  Pipes, conduits and cable trays: their center lines and sections.

- `valves: BTreeMap<ElementId, crate::mep::ValveGeom>`

  Valves placed on their pipes.

- `mep: crate::mep_net::MepNetwork`

  Where pipes, conduits and cable trays meet: nodes, fittings, valves and attached fixtures and devices.

- `foundations: BTreeMap<ElementId, FoundationGeom>`

  Foundations: plan outline and elevations (strips meet at corners like beams).

- `rooms: BTreeMap<ElementId, Vec<DVec2>>`

  Boundary of each enclosed room (missing when the room is not enclosed).

- `roof_ridges: BTreeMap<ElementId, Vec<(DVec2, DVec2)>>`

  Ridge and hip lines of pitched roofs, in plan.

- `join_deductions: BTreeMap<ElementId, f64>`

  Volume a joined wall shares with the elements that cut it, taken
  off its quantity (a wall's body stays whole; see `Self::volume`).

- `components: BTreeMap<ElementId, Vec<cc_core::component::ComponentPart>>`

  Solid pieces of each placed component in the world, voids cut out.

- `opening_fills: BTreeMap<ElementId, Vec<cc_core::component::ComponentPart>>`

  Pieces of each door and window in the world (see `opening_parts`).

- `opening_symbols: Vec<OpeningSymbol>`

  The elevation symbols of windows that open, on both faces of their walls.

- `masses: BTreeMap<ElementId, crate::mass::MassGeom>`

  Conceptual masses: body, volume and mass floors.

- `roof_framing: BTreeMap<ElementId, crate::roof_framing::RoofFramingGeom>`

  Roofs: their framing members and covering figures.

- `topographies: BTreeMap<ElementId, crate::topography::TopoGeom>`

  Terrain surfaces: triangles, elevations, area and volume.

- `pads: BTreeMap<ElementId, crate::pad::PadGeom>`

  Building pads graded into their terrains (see `crate::pad`).

- `curved_volumes: BTreeMap<ElementId, f64>`

  Exact volumes of floors sketched with arcs, circles or ellipses, whose
  solids mesh the curves with chords (see `Self::volume`).

- `piles: BTreeMap<ElementId, crate::pile::PileGeom>`

  Piles: axis, section and elevations (see `crate::pile`).

- `solid_index: SolidIndex`

  Where each element's solids are in `solids`, made at the end of a
  build (see `SolidIndex`).


**Methods**

- `pub fn build(project: &Project) -> Self`

  Geometry of the model as views without a design option choice show
  it: the main model plus the primary (or active) option of each set,
  so elements of other options neither join nor bound rooms (see
  `cc_core::design_option`).

- `pub fn shared_volume(&self, a: ElementId, b: ElementId) -> f64`

  Volume the bodies of `a` and `b` share (0 when they only touch). Bodies
  already cut by a join have lost what they shared.

- `pub fn bounding_footprints(&self, project: &Project, level: cc_core::LevelId, phase: Option<usize>) -> Vec<Vec<DVec2>>`

  Footprints of the room-bounding walls of `level`; with a phase (an
  index into the project's phases), only of the walls standing at its
  end (built by then and not demolished in it or before).

- `pub fn room_at(&self, project: &Project, level: cc_core::LevelId, point: DVec2, phase: Option<cc_core::PhaseId>) -> Option<Vec<DVec2>>`

  Boundary of the room that would enclose `point` on `level`, in
  `phase` (unknown: the last phase).

- `pub fn build_phase(project: &Project, phase: usize, full: &ModelGeometry) -> Self`

  Geometry of the model as a view of phase number `phase` shows it:
  only the elements standing in it or built in it, so doors of later
  phases do not open existing walls and walls join only with walls of
  the phase. Rooms of earlier phases keep their boundaries from `full`
  (the geometry of the whole model).

- `pub fn build_view(project: &Project, phase: Option<usize>, options: &std::collections::BTreeSet<cc_core::OptionId>, full: &ModelGeometry) -> Self`

  Geometry of the model as a view shows it: in phase number `phase`
  (see `ModelGeometry::build_phase`; `None`: every phase) and with
  the main model plus the design options `options` only, so elements
  of other options neither join, open walls nor bound rooms. `full` is
  the geometry of `ModelGeometry::build`.

- `pub fn solids_of(&self, id: ElementId) -> impl Iterator<Item = &Solid>`
- `pub fn body_of(&self, id: ElementId) -> impl Iterator<Item = &Solid>`

  The solids of an element's body: its solids without secondary parts
  such as a roof's framing.

- `pub fn bounds(&self) -> Option<(DVec3, DVec3)>`
- `pub fn plan_edges(&self, id: ElementId) -> Vec<(DVec2, DVec2)>`

  The vertical faces of an element seen from above: each one is a plan
  segment (the sides of a column, the faces of a component).

- `pub fn nearest_plan_edge(&self, id: ElementId, p: DVec2) -> Option<(DVec2, DVec2)>`

  The plan segment of an element's vertical faces nearest to `p` (see `Self::plan_edges`).

- `pub fn volume(&self, id: ElementId) -> f64`

  Volume of an element's main body (frames/glass excluded for openings).

- `pub fn pick_where(&self, ray: Ray, shown: &mut dyn FnMut(ElementId) -> bool) -> Option<(ElementId, f64)>`

  The nearest solid along `ray` among the elements `shown` accepts.

- `pub fn pick(&self, ray: Ray, hidden: &[Category]) -> Option<(ElementId, f64)>`

  Closest element hit by `ray`, skipping categories in `hidden`.

- `pub(crate) fn build_mep(&mut self, project: &Project)` *(in `cc_geom::mep`)*

<a id="cc_geom-build-solidindex"></a>

### struct `SolidIndex`

```rust
pub struct SolidIndex
```

The positions of each element's solids in `ModelGeometry::solids`, in
order, so that finding an element's solids does not scan them all. It is
used only while `solids` has the length it was made for; geometry put
together by hand (or changed after the build) is scanned instead.

**Traits:** `Debug`, `Clone`, `Default`

*Some fields are private.*

<a id="cc_geom-build-foundationgeom"></a>

### struct `FoundationGeom`

```rust
pub struct FoundationGeom
```

A foundation's body: a vertical prism.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `element: ElementId`
- `footprint: Vec<DVec2>`

  Plan outline, counter-clockwise (an arc strip's arcs as short chords).

- `z_range: (f64, f64)`

  Bottom and top elevations.

- `arc: Option<crate::StripArc>`

  An arc strip footing's exact plan (`None` for straight strips, pads and slabs).

- `drawn_area: Option<f64>`

  The plan area of a slab drawn with arcs or circles, which its
  footprint follows with chords.

- `strip: Option<BeamGeom>`

  A strip's frame, section and ends after joins (`None` for pads and slabs).

- `holes: Vec<Vec<DVec2>>`

  Holes through the footprint (a ring pile cap's), counter-clockwise.

- `cap: Option<crate::pile::CapGeom>`

  A pile cap's frame, plan and piles (`None` for other foundations).


**Methods**

- `pub fn area(&self) -> f64`

  Plan area, exact for arc strips and slabs drawn with curves.

- `pub fn body(&self) -> Option<&BeamGeom>`

  The body of a haunched strip (an inverted T): its section swept
  along the strip, straight or arc. `None` for plain bodies, which are
  their footprint extruded.

- `pub fn volume(&self) -> f64`

  Volume, exact for arc strips, haunched strips and slabs drawn with curves.


<a id="cc_geom-build-boxpart"></a>

### struct `BoxPart`

```rust
pub struct BoxPart
```

An axis-aligned box in a horizontal frame: `origin`, plan axes `u`/`v`
(unit, perpendicular) and extents along them and in absolute Z.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `origin: DVec2`
- `u: DVec2`
- `v: DVec2`
- `u_range: (f64, f64)`
- `v_range: (f64, f64)`
- `z_range: (f64, f64)`

**Methods**

- `pub fn add_to(&self, s: &mut Solid)`

<a id="cc_geom-build-column_outline"></a>

### fn `column_outline`

```rust
pub fn column_outline(project: &Project, data: &TypeData, position: DVec2, rotation_deg: f64) -> Vec<DVec2>
```

Plan outline of a column type's section (counter-clockwise), placed at
`position` and turned by `rotation_deg`: the outer boundary of its
profile. Empty for other types or a section that fails.

<a id="cc_geom-build-rect_outline"></a>

### fn `rect_outline`

```rust
pub fn rect_outline(width: f64, depth: f64, position: DVec2, rotation_deg: f64) -> Vec<DVec2>
```

A `width` x `depth` rectangle (counter-clockwise) centered at `position`,
turned by `rotation_deg`: the footprint of a pad footing.

<a id="cc_geom-build-component_parts"></a>

### fn `component_parts`

```rust
pub fn component_parts(project: &Project, element: &cc_core::Element, component: &cc_core::Component) -> Vec<cc_core::component::ComponentPart>
```

Solid pieces of a placed component in world coordinates (voids cut
out): footprints rotated and moved to the insertion point, elevations
absolute. Pieces whose expressions fail are left out (the component
editor reports them). `ModelGeometry::components` holds them for every
component of the model.

<a id="cc_geom-build-worldconnector"></a>

### struct `WorldConnector`

```rust
pub struct WorldConnector
```

A component's connector in the world: where a run joins it (`point`, its
z the absolute elevation) and the way the run leaves (`direction`).

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `index: usize`
- `name: String`
- `system: cc_core::SystemKind`
- `point: DVec3`
- `direction: DVec3`
- `vertical: bool`

  Whether the run leaves vertically (down or up).

- `diameter: Option<f64>`

<a id="cc_geom-build-component_connectors"></a>

### fn `component_connectors`

```rust
pub fn component_connectors(project: &Project, element: &cc_core::Element) -> Vec<WorldConnector>
```

The connectors of a placed component, in the world.

<a id="mod-cc_geom-component"></a>

## `cc_geom::component`

<sub>`crates/cc_geom/src/component.rs`</sub>

Component pieces as solids: the definition's pieces resolved with their
parameter values, and the void pieces cut out of the solid pieces they
overlap. Script pieces run through the process's shared script cache
(`cc_script::shared`).

**Items:** [`resolve_component`](#cc_geom-component-resolve_component) · [`resolve_component_in`](#cc_geom-component-resolve_component_in) · [`resolve_pieces`](#cc_geom-component-resolve_pieces) · [`resolve_instance`](#cc_geom-component-resolve_instance) · [`instance_errors`](#cc_geom-component-instance_errors) · [`faces_of`](#cc_geom-component-faces_of)

<a id="cc_geom-component-resolve_component"></a>

### fn `resolve_component`

```rust
pub fn resolve_component(def: &ComponentDef, overrides: &BTreeMap<String, f64>) -> Vec<ComponentPart>
```

The solid pieces of `def` with `overrides`, in the component's frame,
voids cut out. A piece a void reaches becomes faces; past the boolean
budget it stays whole. Pieces whose expressions fail are left out.

<a id="cc_geom-component-resolve_component_in"></a>

### fn `resolve_component_in`

```rust
pub fn resolve_component_in(def: &ComponentDef, overrides: &BTreeMap<String, f64>, extra: &[(&str, f64)]) -> Vec<ComponentPart>
```

`resolve_component` with `extra` names defined (a door's wall thickness).

<a id="cc_geom-component-resolve_pieces"></a>

### fn `resolve_pieces`

```rust
pub fn resolve_pieces(def: &ComponentDef, overrides: &BTreeMap<String, f64>, extra: &[(&str, f64)]) -> (Vec<ComponentPart>, Vec<(usize, PartError)>)
```

Every piece of `def` and the errors of those that fail (script pieces
run), without cutting voids: for editors and checks.

<a id="cc_geom-component-resolve_instance"></a>

### fn `resolve_instance`

```rust
pub fn resolve_instance(ty: &ElementType, def: &ComponentDef, element: &cc_core::Element, component: Option<&cc_core::Component>, extra: &[(&str, f64)]) -> Vec<ComponentPart>
```

The solid pieces of a placed component of type `ty` (its instance
values, text values and moved hotspots), voids cut out. A failing script
shows the last pieces that worked for this element.

<a id="cc_geom-component-instance_errors"></a>

### fn `instance_errors`

```rust
pub fn instance_errors(ty: &ElementType, def: &ComponentDef, element: &cc_core::Element, component: Option<&cc_core::Component>) -> Vec<(usize, PartError)>
```

The errors of a placed component's pieces (a failing script's among them).

<a id="cc_geom-component-faces_of"></a>

### fn `faces_of`

```rust
pub fn faces_of(part: &ComponentPart) -> Vec<Vec<DVec3>>
```

The closed faces of a piece (a prism's are built from its footprint).

<a id="mod-cc_geom-connection"></a>

## `cc_geom::connection`

<sub>`crates/cc_geom/src/connection.rs`</sub>

Steel and timber connections (see `cc_core::connection`): the plates
and bolts of each, derived from its member's geometry on every build, so
they follow the member. Plates are oriented boxes (`Block`), bolts and
washers cylinders (`Rod`). A connection whose member is missing, or
whose member end is not framed as its kind needs, builds nothing and
carries a `ConnectionWarning`.

**Items:** [`BOLT_EXTRA`](#cc_geom-connection-bolt_extra) · [`WASHER_RATIO`](#cc_geom-connection-washer_ratio) · [`FIN_HEIGHT`](#cc_geom-connection-fin_height) · [`HANGER_SIDE`](#cc_geom-connection-hanger_side) · [`STEEL_COLOR`](#cc_geom-connection-steel_color) · [`Block`](#cc_geom-connection-block) · [`Rod`](#cc_geom-connection-rod) · [`Bolt`](#cc_geom-connection-bolt) · [`Plate`](#cc_geom-connection-plate) · [`ConnectionWarning`](#cc_geom-connection-connectionwarning) · [`ConnectionGeom`](#cc_geom-connection-connectiongeom) · [`beam_end_join`](#cc_geom-connection-beam_end_join) · [`suits`](#cc_geom-connection-suits) · [`steel_material`](#cc_geom-connection-steel_material) · [`solid`](#cc_geom-connection-solid) · [`connection_geom`](#cc_geom-connection-connection_geom) · [`build_connections`](#cc_geom-connection-build_connections)

<a id="cc_geom-connection-bolt_extra"></a>

### const `BOLT_EXTRA`

```rust
pub const BOLT_EXTRA: f64
```

End and fin plate bolts are the plate's thickness plus this (nut and head), meters.

<a id="cc_geom-connection-washer_ratio"></a>

### const `WASHER_RATIO`

```rust
pub const WASHER_RATIO: f64
```

Washer diameter per bolt diameter.

<a id="cc_geom-connection-fin_height"></a>

### const `FIN_HEIGHT`

```rust
pub const FIN_HEIGHT: f64
```

A fin plate's height per section height.

<a id="cc_geom-connection-hanger_side"></a>

### const `HANGER_SIDE`

```rust
pub const HANGER_SIDE: f64
```

A hanger's sides per beam height.

<a id="cc_geom-connection-steel_color"></a>

### const `STEEL_COLOR`

```rust
pub const STEEL_COLOR: [u8; 3]
```

Connections drawn without a metal material in the project.

<a id="cc_geom-connection-block"></a>

### struct `Block`

```rust
pub struct Block
```

A box: its center, two square unit axes `x` and `y` (the third is
`x × y`) and its size along the three.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `center: DVec3`
- `x: DVec3`
- `y: DVec3`
- `size: DVec3`

**Methods**

- `pub fn z(&self) -> DVec3`
- `pub fn volume(&self) -> f64`
- `pub fn faces(&self) -> Vec<Vec<DVec3>>`

  The six faces, wound outward.


<a id="cc_geom-connection-rod"></a>

### struct `Rod`

```rust
pub struct Rod
```

A cylinder from `a` to `b`.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `a: DVec3`
- `b: DVec3`
- `diameter: f64`

**Methods**

- `pub fn length(&self) -> f64`

<a id="cc_geom-connection-bolt"></a>

### struct `Bolt`

```rust
pub struct Bolt
```

A bolt (or anchor bolt) and its washers.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `shank: Rod`
- `washers: Vec<Rod>`

<a id="cc_geom-connection-plate"></a>

### struct `Plate`

```rust
pub struct Plate
```

A plate: one box, or several for a bent plate (a hanger's base and sides).

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `blocks: Vec<Block>`

**Methods**

- `pub fn volume(&self) -> f64`

<a id="cc_geom-connection-connectionwarning"></a>

### enum `ConnectionWarning`

```rust
pub enum ConnectionWarning
```

Why a connection built nothing.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Variants**

- `MissingMember`

  Its member is gone.

- `Unsuitable`

  Its member end does not take this kind (or the member's section fails).


**Methods**

- `pub fn code(self) -> &'static str`

<a id="cc_geom-connection-connectiongeom"></a>

### struct `ConnectionGeom`

```rust
pub struct ConnectionGeom
```

The parts of one connection.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `element: ElementId`
- `member: ElementId`
- `kind: ConnectionKind`
- `plates: Vec<Plate>`
- `bolts: Vec<Bolt>`
- `bolt_diameter: f64`
- `warning: Option<ConnectionWarning>`

**Methods**

- `pub fn plate_volume(&self) -> f64`
- `pub fn plate_mass(&self) -> f64`

  Kilograms of steel in the plates.

- `pub fn bolt_count(&self) -> usize`
- `pub fn bolt_length(&self) -> f64`

  The length of one bolt (they are alike); 0 with none.

- `pub fn faces(&self) -> Vec<Vec<DVec3>>`

  The faces of every part.


<a id="cc_geom-connection-beam_end_join"></a>

### fn `beam_end_join`

```rust
pub fn beam_end_join(g: &BeamGeom, end: MemberEnd) -> Option<BeamEnd>
```

How the end `end` of a beam is framed (`None` for a column's base).

<a id="cc_geom-connection-suits"></a>

### fn `suits`

```rust
pub fn suits(kind: ConnectionKind, joined: BeamEnd) -> bool
```

Whether a beam end framed as `joined` takes a connection of `kind`.

<a id="cc_geom-connection-steel_material"></a>

### fn `steel_material`

```rust
pub fn steel_material(project: &Project) -> Option<&str>
```

The material connections are drawn in: the project's first metal.

<a id="cc_geom-connection-solid"></a>

### fn `solid`

```rust
pub fn solid(project: &Project, element: ElementId) -> Solid
```

An empty solid of connection `element`, in steel.

<a id="cc_geom-connection-connection_geom"></a>

### fn `connection_geom`

```rust
pub fn connection_geom(project: &Project, beams: &BTreeMap<ElementId, BeamGeom>, id: ElementId, c: &Connection) -> ConnectionGeom
```

Connection `id` as `c` and its member make it.

<a id="cc_geom-connection-build_connections"></a>

### fn `build_connections`

```rust
pub fn build_connections(project: &Project, beams: &BTreeMap<ElementId, BeamGeom>) -> BTreeMap<ElementId, ConnectionGeom>
```

Every connection of the project, from the resolved beams.

<a id="mod-cc_geom-coplanar"></a>

## `cc_geom::coplanar`

<sub>`crates/cc_geom/src/coplanar.rs`</sub>

Coplanar faces of an imported mesh merged into polygons: the triangles
a file splits a flat face into come back as one face, so views draw no
diagonals across it and the piece holds fewer faces.
Faces grow one neighbor at a time across shared edges (opposite
directions, so the winding agrees), keeping each merged face a simple
polygon: a neighbor joins only when it shares one run of edges with the
face so far and touches it nowhere else, so no hole or pinch can form.
A ring of triangles thus stays two faces or more. Vertices are never
dropped (a point on a straight run may be a corner of a neighbor).

**Items:** [`merge`](#cc_geom-coplanar-merge)

<a id="cc_geom-coplanar-merge"></a>

### fn `merge`

```rust
pub fn merge(vertices: &[[f64; 3]], faces: Vec<Vec<u32>>) -> Vec<Vec<u32>>
```

`faces` (indices into `vertices`) with neighbors in one plane merged.

<a id="mod-cc_geom-csg"></a>

## `cc_geom::csg`

<sub>`crates/cc_geom/src/csg.rs`</sub>

Boolean operations on closed polygon meshes with binary space
partitioning trees (the classic BSP-tree CSG algorithm: each solid's
polygons are clipped against the other's tree). Trees live in arenas and
are walked with explicit stacks, so deep trees cannot overflow the call
stack; operations whose fragments would exceed a budget give up and
return `None`, leaving the caller with the uncut solid.

**Items:** [`subtract`](#cc_geom-csg-subtract) · [`intersect`](#cc_geom-csg-intersect) · [`bounds_overlap`](#cc_geom-csg-bounds_overlap)

<a id="cc_geom-csg-subtract"></a>

### fn `subtract`

```rust
pub fn subtract(a: &[Vec<DVec3>], b: &[Vec<DVec3>]) -> Option<Vec<Vec<DVec3>>>
```

`a` minus `b` (both closed and outward), or `None` past the budget.

<a id="cc_geom-csg-intersect"></a>

### fn `intersect`

```rust
pub fn intersect(a: &[Vec<DVec3>], b: &[Vec<DVec3>]) -> Option<Vec<Vec<DVec3>>>
```

The common part of `a` and `b`, or `None` past the budget.

<a id="cc_geom-csg-bounds_overlap"></a>

### fn `bounds_overlap`

```rust
pub fn bounds_overlap(a: &[Vec<DVec3>], b: &[Vec<DVec3>]) -> bool
```

Whether the bounding boxes of two meshes overlap.

<a id="mod-cc_geom-curtain"></a>

## `cc_geom::curtain`

<sub>`crates/cc_geom/src/curtain.rs`</sub>

Curtain walls in the world: the panels and mullions of a wall's grid
(see `cc_core::curtain`). Panels are flat: in an arc wall each one
spans the chord between its grid lines, and mullions stand square to the
arc on its lines.

**Items:** [`PanelGeom`](#cc_geom-curtain-panelgeom) · [`MullionGeom`](#cc_geom-curtain-mulliongeom) · [`CurtainGeom`](#cc_geom-curtain-curtaingeom) · [`build`](#cc_geom-curtain-build) · [`solids`](#cc_geom-curtain-solids) · [`volume`](#cc_geom-curtain-volume)

<a id="cc_geom-curtain-panelgeom"></a>

### struct `PanelGeom`

```rust
pub struct PanelGeom
```

A panel placed in the world.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `cell: usize`

  Index of its cell in `Layout::panels`.

- `start: DVec2`

  Its ends on the wall's centerline (the chord it spans), at the bottom.

- `end: DVec2`
- `z: (f64, f64)`
- `parts: Vec<ComponentPart>`

  The pieces of a component panel, in the world.


**Methods**

- `pub fn frame(&self) -> (DVec2, DVec2)`

  Unit direction along the panel and its left normal.

- `pub fn width(&self) -> f64`

<a id="cc_geom-curtain-mulliongeom"></a>

### struct `MullionGeom`

```rust
pub struct MullionGeom
```

A mullion piece placed in the world: a vertical prism over `footprint`.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `piece: usize`

  Index of its piece in `Layout::mullions`.

- `footprint: [DVec2; 4]`

  Plan outline, counter-clockwise.

- `z: (f64, f64)`

<a id="cc_geom-curtain-curtaingeom"></a>

### struct `CurtainGeom`

```rust
pub struct CurtainGeom
```

The panels and mullions of a curtain wall.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `layout: Layout`
- `panels: Vec<PanelGeom>`
- `mullions: Vec<MullionGeom>`
- `thickness: f64`

  The thickness of the type (component panels are resolved with it).


**Methods**

- `pub fn cell_at(&self, g: &WallGeom, p: DVec2, z: f64) -> Option<(usize, usize)>`

  The cell `(column, row)` holding plan point `p` at height `z`.


<a id="cc_geom-curtain-build"></a>

### fn `build`

```rust
pub fn build(project: &Project, element: &Element, g: &WallGeom, system: &curtain::CurtainSystem) -> CurtainGeom
```

Lay out the curtain grid of wall `element` (type `system`) over its
geometry `g`: between its ends after joins, from its base to its top.

<a id="cc_geom-curtain-solids"></a>

### fn `solids`

```rust
pub fn solids(project: &Project, element: &Element, c: &CurtainGeom) -> Vec<Solid>
```

The solids of a curtain wall: one per material of its panels and mullions.

<a id="cc_geom-curtain-volume"></a>

### fn `volume`

```rust
pub fn volume(c: &CurtainGeom) -> f64
```

Volume of the glass and solid panels and of the mullions.

<a id="mod-cc_geom-cut"></a>

## `cc_geom::cut`

<sub>`crates/cc_geom/src/cut.rs`</sub>

Plane cuts used by sections and plan views.

**Items:** [`face_plane_segments`](#cc_geom-cut-face_plane_segments) · [`clip_polygon`](#cc_geom-cut-clip_polygon)

<a id="cc_geom-cut-face_plane_segments"></a>

### fn `face_plane_segments`

```rust
pub fn face_plane_segments(face: &Face, point: DVec3, normal: DVec3) -> Vec<(DVec3, DVec3)>
```

Segments where `face` crosses the plane through `point` with `normal`.

<a id="cc_geom-cut-clip_polygon"></a>

### fn `clip_polygon`

```rust
pub fn clip_polygon(verts: &[DVec3], point: DVec3, normal: DVec3) -> Vec<DVec3>
```

Sutherland–Hodgman clip keeping the part with `(p - point)·normal >= 0`.

<a id="mod-cc_geom-faces"></a>

## `cc_geom::faces`

<sub>`crates/cc_geom/src/faces.rs`</sub>

The faces of a column's section, a given distance inside it: where its
longitudinal bars stand and its stirrups run. A face is a straight side,
a run of the chords of a curve, or (for an annular sector) a true arc;
faces meet at corners, in counter-clockwise order.

**Items:** [`SectionFace`](#cc_geom-faces-sectionface) · [`sector_faces`](#cc_geom-faces-sector_faces) · [`outline_faces`](#cc_geom-faces-outline_faces) · [`FaceBar`](#cc_geom-faces-facebar) · [`face_bars`](#cc_geom-faces-face_bars) · [`rectangles`](#cc_geom-faces-rectangles) · [`face_loop`](#cc_geom-faces-face_loop) · [`across`](#cc_geom-faces-across)

<a id="cc_geom-faces-sectionface"></a>

### struct `SectionFace`

```rust
pub struct SectionFace
```

One face of a section.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `points: Vec<DVec2>`

  Points along the face from its first corner to its last (at least 2).

- `arc: Option<Arc>`

  The circle the face lies on when it is a true arc (its points are on it).


**Methods**

- `pub fn is_curved(&self) -> bool`

  Whether the face is curved (an arc, or the chords of a curve).

- `pub fn is_closed(&self) -> bool`

  Whether the face runs all around a section without corners (a circle).

- `pub fn length(&self) -> f64`

  Length along the face: the arc's, else its chords'.

- `pub fn at(&self, t: f64) -> (DVec2, DVec2)`

  The point the fraction `t` of the way along the face, and the unit
  normal into the section there (to the left of the way round).


<a id="cc_geom-faces-sector_faces"></a>

### fn `sector_faces`

```rust
pub fn sector_faces(s: &SectorPlacement, inset: f64) -> Vec<SectionFace>
```

The faces of an annular sector `inset` inside it, exact: the outer arc,
the left radial side, the inner arc and the right radial side (a
circular sector, or one whose radial sides meet before the inner arc,
has no inner arc: its radial sides meet at a point). Empty when nothing
is left inside the inset.

<a id="cc_geom-faces-outline_faces"></a>

### fn `outline_faces`

```rust
pub fn outline_faces(outline: &[DVec2], inset: f64) -> Vec<SectionFace>
```

The faces of a section's `outline` moved `inset` inside it: its
corners are where it turns sharply, or where the lengths of its sides
jump (a straight side beside the chords of a fillet); the sides between
two corners are one face. A circle is one closed face. Empty when the
outline does not survive the inset.

<a id="cc_geom-faces-facebar"></a>

### struct `FaceBar`

```rust
pub struct FaceBar
```

A longitudinal bar in the section: where it stands, the normal into the
section there, and whether it stands at a corner.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `at: DVec2`
- `inward: DVec2`
- `corner: bool`

<a id="cc_geom-faces-face_bars"></a>

### fn `face_bars`

```rust
pub fn face_bars(faces: &[SectionFace], spec: &[FaceBars], count: usize) -> Vec<FaceBar>
```

The bars on `faces`: one at every corner, and between the corners of
each face what its entry of `spec` says; without a `spec`, `count` bars,
the corners first and each of the others on the face with the widest
gap. A closed face (a circle) takes its bars evenly around.

<a id="cc_geom-faces-rectangles"></a>

### fn `rectangles`

```rust
pub fn rectangles(outline: &[DVec2]) -> Vec<(DVec2, DVec2)>
```

The closed loop around `faces` (a stirrup's center line) and its true
length, arcs as arcs.
The largest rectangles `(min, max)` inside an outline whose sides all
run along its own X or Y axis: two in an L or a T, three in a U, one in a
rectangle. Together they cover it, overlapping where its parts meet.
Empty for any other outline, or one with too many corners.

<a id="cc_geom-faces-face_loop"></a>

### fn `face_loop`

```rust
pub fn face_loop(faces: &[SectionFace]) -> (Vec<DVec2>, f64)
```

<a id="cc_geom-faces-across"></a>

### fn `across`

```rust
pub fn across(faces: &[SectionFace], from: DVec2, inward: DVec2, clear: f64) -> Option<DVec2>
```

Where a cross-tie from `from` (on a face whose normal into the section
is `inward`) meets the face opposite, at least `clear` away: straight
across, or square to the bisector of that face and a straight face that
leans towards it (the radial sides of a sector); whichever leaves and
arrives nearest to square, and within 45° of both faces' normals.

<a id="mod-cc_geom-level_mark"></a>

## `cc_geom::level_mark`

<sub>`crates/cc_geom/src/level_mark.rs`</sub>

The levels a level mark writes: measured from the floors under its point
unless the mark sets them.
In a plan the finished level is the top of the highest floor of the
plan's level under the point (the level itself where there is none); in
elevations and sections it is the height of the point. The raw level is
the top of that floor's structure (below its finish layers): in sections
from the floor whose top the point lies on, on the cut line.

**Items:** [`MarkLevels`](#cc_geom-level_mark-marklevels) · [`levels`](#cc_geom-level_mark-levels)

<a id="cc_geom-level_mark-marklevels"></a>

### struct `MarkLevels`

```rust
pub struct MarkLevels
```

Levels to write, as elevations of the project (the type's datum not added).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `finished: f64`
- `raw: Option<f64>`

  The raw level, when the mark writes one.


<a id="cc_geom-level_mark-levels"></a>

### fn `levels`

```rust
pub fn levels(project: &Project, view: &View, mark: &LevelMark) -> Option<MarkLevels>
```

The levels `mark` writes in `view` (none in views without levels: 3D,
legends, diagrams…).

<a id="mod-cc_geom-lightning"></a>

## `cc_geom::lightning`

<sub>`crates/cc_geom/src/lightning.rs`</sub>

Lightning protection in the model: where an air terminal stands on a
roof, and a simplified coverage check of the roofs by the air
termination (air terminals and roof conductors).
The check samples the exposed upward faces of roofs (and of floors above
the lowest level that nothing covers) and asks, for each point, whether
one air termination point protects it by itself: the point lies inside
the protected volume of a single rod under the rolling sphere method,
over a horizontal reference plane at the lowest point of the element's
exposed faces (its eaves). On flat parts (slope up to 1:10) a point
within half the class's mesh width of a roof conductor is protected as
by the mesh method. Mutual protection between terminals (the sphere
sagging between two rods) and the shielding by the building are left
out, which errs on the safe side.

**Items:** [`surface_top`](#cc_geom-lightning-surface_top) · [`air_points`](#cc_geom-lightning-air_points) · [`Coverage`](#cc_geom-lightning-coverage) · [`check`](#cc_geom-lightning-check)

<a id="cc_geom-lightning-surface_top"></a>

### fn `surface_top`

```rust
pub fn surface_top(geom: &ModelGeometry, p: DVec2) -> Option<f64>
```

The world height of the highest roof or floor surface over plan point
`p` (`None` where there is none).

<a id="cc_geom-lightning-air_points"></a>

### fn `air_points`

```rust
pub fn air_points(project: &Project, geom: &ModelGeometry) -> Vec<(ElementId, DVec3)>
```

The points of the air termination: the tip of each air terminal (the
top of its solids) and points along each roof (air) conductor, at most
`step` meters apart.

<a id="cc_geom-lightning-coverage"></a>

### struct `Coverage`

```rust
pub struct Coverage
```

The result of a coverage check.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `class: LpsClass`
- `radius: f64`

  Rolling sphere radius and mesh width used, meters.

- `mesh: f64`
- `spacing: f64`

  Distance between the sampled points, meters.

- `air_points: usize`

  Air termination points (terminal tips and conductor samples).

- `checked: usize`

  Roof points checked and how many of them are protected.

- `protected: usize`
- `roofs: Vec<ElementId>`

  The roof elements checked.

- `unprotected: Vec<(ElementId, DVec3)>`

  The points left unprotected, with their element.


**Methods**

- `pub fn ratio(&self) -> f64`

  Share of the checked points that are protected (1 with nothing to check).


<a id="cc_geom-lightning-check"></a>

### fn `check`

```rust
pub fn check(project: &Project, geom: &ModelGeometry, class: LpsClass, spacing: f64) -> Coverage
```

Check how the air termination covers the roofs for `class`, sampling
the roofs about `spacing` meters apart (more when they are large).

<a id="mod-cc_geom-mass"></a>

## `cc_geom::mass`

<sub>`crates/cc_geom/src/mass.rs`</sub>

Conceptual masses: the body of a mass (an extrusion, or a loft between
two outlines) and its mass floors, the sections at the levels it lists.

**Items:** [`MassGeom`](#cc_geom-mass-massgeom) · [`mass_geom`](#cc_geom-mass-mass_geom) · [`face_at`](#cc_geom-mass-face_at) · [`face_outline`](#cc_geom-mass-face_outline)

<a id="cc_geom-mass-massgeom"></a>

### struct `MassGeom`

```rust
pub struct MassGeom
```

A mass's body and floors.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `faces: Vec<Vec<DVec3>>`

  Faces of the closed body.

- `z_range: (f64, f64)`

  Elevations of the base and the top.

- `volume: f64`
- `floors: Vec<MassFloor>`

  Mass floors from the lowest up: one per listed level the mass spans
  (at or above its base, below its top).


**Methods**

- `pub fn floor_area(&self) -> f64`

  Total area of the mass floors (the gross floor area).


<a id="cc_geom-mass-mass_geom"></a>

### fn `mass_geom`

```rust
pub fn mass_geom(project: &Project, e: &Element, m: &Mass) -> Option<MassGeom>
```

The body and floors of mass `m` (element `e`), or `None` when it is degenerate.

<a id="cc_geom-mass-face_at"></a>

### fn `face_at`

```rust
pub fn face_at(m: &Mass, g: &MassGeom, p: DVec3) -> Option<cc_core::mass::MassFace>
```

The face of mass `m` (built as `g`) at `p`, a point on its body: the top
or bottom, or the side over the nearest edge of an extrusion's outline
(a loft's sides are not pushed).

<a id="cc_geom-mass-face_outline"></a>

### fn `face_outline`

```rust
pub fn face_outline(m: &Mass, g: &MassGeom, face: cc_core::mass::MassFace) -> Option<(Vec<DVec3>, DVec3)>
```

The outline of a face of `m` in the world, and its outward normal.

<a id="mod-cc_geom-mep"></a>

## `cc_geom::mep`

<sub>`crates/cc_geom/src/mep.rs`</sub>

Pipes, conduits and cable trays: each pipe or conduit segment is a
cylinder of its type's diameter along its center line, each cable tray an
open channel (or a ladder) of its type's section around it, colored by
its system. Valves sit on their pipes as a body, a stem and a handwheel
or lever.

**Items:** [`TraySection`](#cc_geom-mep-traysection) · [`SegmentGeom`](#cc_geom-mep-segmentgeom) · [`ValveGeom`](#cc_geom-mep-valvegeom) · [`segment_geom`](#cc_geom-mep-segment_geom) · [`valve_geom`](#cc_geom-mep-valve_geom) · [`frustum`](#cc_geom-mep-frustum) · [`cylinder`](#cc_geom-mep-cylinder) · [`section_frame`](#cc_geom-mep-section_frame) · [`tray_sheet`](#cc_geom-mep-tray_sheet) · [`tray_profile`](#cc_geom-mep-tray_profile) · [`tray`](#cc_geom-mep-tray) · [`valve`](#cc_geom-mep-valve)

<a id="cc_geom-mep-traysection"></a>

### struct `TraySection`

```rust
pub struct TraySection
```

The section of a cable tray run.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `width: f64`
- `height: f64`
- `style: TrayStyle`

<a id="cc_geom-mep-segmentgeom"></a>

### struct `SegmentGeom`

```rust
pub struct SegmentGeom
```

A pipe, conduit or cable tray run in the world.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `element: ElementId`
- `a: DVec3`

  Center line ends.

- `b: DVec3`
- `diameter: f64`

  Outside diameter (cable trays: their width).

- `tray: Option<TraySection>`

  Cable trays: their section.


**Methods**

- `pub fn length(&self) -> f64`

<a id="cc_geom-mep-valvegeom"></a>

### struct `ValveGeom`

```rust
pub struct ValveGeom
```

A valve in the world.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `element: ElementId`
- `host: ElementId`

  The pipe it sits on.

- `kind: ValveKind`
- `point: DVec3`

  Its center, on the pipe's center line.

- `dir: DVec3`

  Unit direction of the pipe (start to end).

- `diameter: f64`

  The pipe's diameter.


<a id="cc_geom-mep-segment_geom"></a>

### fn `segment_geom`

```rust
pub fn segment_geom(project: &Project, e: &Element) -> Option<SegmentGeom>
```

The run of a pipe, conduit or cable tray element (`None` for other
elements or a degenerate run).

<a id="cc_geom-mep-valve_geom"></a>

### fn `valve_geom`

```rust
pub fn valve_geom(project: &Project, e: &Element) -> Option<ValveGeom>
```

The valve element `e` placed on its pipe (`None` for other elements, or
when its pipe has no run).

<a id="cc_geom-mep-frustum"></a>

### fn `frustum`

```rust
pub fn frustum(a: DVec3, b: DVec3, ra: f64, rb: f64) -> Vec<Vec<DVec3>>
```

The faces of a closed frustum from `a` (radius `ra`) to `b` (radius `rb`).

<a id="cc_geom-mep-cylinder"></a>

### fn `cylinder`

```rust
pub fn cylinder(a: DVec3, b: DVec3, radius: f64) -> Vec<Vec<DVec3>>
```

The faces of a closed cylinder of `radius` from `a` to `b`.

<a id="cc_geom-mep-section_frame"></a>

### fn `section_frame`

```rust
pub fn section_frame(axis: DVec3) -> (DVec3, DVec3)
```

A frame across `axis`: `side` to the left of a horizontal run and `up`
above it (`side × up = axis`); vertical runs take the X axis as side.

<a id="cc_geom-mep-tray_sheet"></a>

### fn `tray_sheet`

```rust
pub fn tray_sheet(height: f64) -> f64
```

Thickness of a tray's sheet for a section `height` tall.

<a id="cc_geom-mep-tray_profile"></a>

### fn `tray_profile`

```rust
pub fn tray_profile(t: TraySection) -> Vec<glam::DVec2>
```

The section of a tray as an open channel, counter-clockwise in
(`side`, `up`) coordinates of `section_frame` around its center line.

<a id="cc_geom-mep-tray"></a>

### fn `tray`

```rust
pub fn tray(a: DVec3, b: DVec3, t: TraySection) -> Vec<Vec<DVec3>>
```

The faces of a cable tray of section `t` from `a` to `b` (center line in
the middle of the section): two side rails and a bottom, or rungs for a
ladder.

<a id="cc_geom-mep-valve"></a>

### fn `valve`

```rust
pub fn valve(v: &ValveGeom) -> Vec<Vec<DVec3>>
```

The faces of a valve: a body narrowing to its middle along the pipe and,
except for check valves, a stem with a handwheel (or a lever on ball valves).

<a id="mod-cc_geom-mep_net"></a>

## `cc_geom::mep_net`

<sub>`crates/cc_geom/src/mep_net.rs`</sub>

The networks formed by pipes, conduits and cable trays: where runs meet
(their ends within a centimeter, or an end on another run), the fittings
there, the valves on pipes (each a node on its pipe's run) and the
fixtures and devices attached to the networks' nodes.
Pipes connect only to pipes of the same system; conduits connect to any
conduit (they meet in boxes), cable trays to any cable tray. Fittings
are derived like wall joins: an elbow where two runs turn, a coupling
(or a reducer, between diameters or tray widths) where they continue
straight, a tee where three branches meet and a cross from four.

**Items:** [`TOL`](#cc_geom-mep_net-tol) · [`ATTACH`](#cc_geom-mep_net-attach) · [`FittingKind`](#cc_geom-mep_net-fittingkind) · [`Fitting`](#cc_geom-mep_net-fitting) · [`NetNode`](#cc_geom-mep_net-netnode) · [`NetEdge`](#cc_geom-mep_net-netedge) · [`MepNetwork`](#cc_geom-mep_net-mepnetwork)

<a id="cc_geom-mep_net-tol"></a>

### const `TOL`

```rust
pub const TOL: f64
```

Ends closer than this meet (1 cm).

<a id="cc_geom-mep_net-attach"></a>

### const `ATTACH`

```rust
pub const ATTACH: f64
```

A fixture or device is attached to a node within this distance in plan.

<a id="cc_geom-mep_net-fittingkind"></a>

### enum `FittingKind`

```rust
pub enum FittingKind
```

What a fitting is.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Variants**

- `Elbow { angle: f64, }`

  Two runs turning by `angle` degrees.

- `Coupling`

  Two runs of one diameter continuing straight.

- `Reducer`

  Two runs of different diameters continuing straight.

- `Tee`
- `Cross`

**Methods**

- `pub fn key(&self) -> &'static str`
- `pub fn nominal_angle(&self) -> Option<f64>`

  The angle of an elbow rounded to the usual 45° or 90° (other angles to the degree).


<a id="cc_geom-mep_net-fitting"></a>

### struct `Fitting`

```rust
pub struct Fitting
```

A fitting at a node.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `kind: FittingKind`
- `category: Category`
- `node: usize`
- `point: DVec3`
- `level: LevelId`
- `system: Option<SystemId>`
- `runs: Vec<ElementId>`

  Runs meeting there (one per end; a run passing through counts once).

- `branches: Vec<(DVec3, f64)>`

  Unit direction and diameter of each branch leaving the node.


**Methods**

- `pub fn diameters(&self) -> Vec<f64>`

  Diameters of the branches, largest first, without repeats.

- `pub fn diameter(&self) -> f64`

<a id="cc_geom-mep_net-netnode"></a>

### struct `NetNode`

```rust
pub struct NetNode
```

A point where runs end or meet.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `point: DVec3`
- `category: Category`
- `system: Option<SystemId>`
- `level: LevelId`

<a id="cc_geom-mep_net-netedge"></a>

### struct `NetEdge`

```rust
pub struct NetEdge
```

A piece of a run between two consecutive nodes along it.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `run: ElementId`
- `a: usize`
- `b: usize`
- `length: f64`

<a id="cc_geom-mep_net-mepnetwork"></a>

### struct `MepNetwork`

```rust
pub struct MepNetwork
```

The pipe, conduit and cable tray networks of a project.

**Traits:** `Debug`, `Clone`, `Default`, `PartialEq`

**Fields**

- `nodes: Vec<NetNode>`
- `edges: Vec<NetEdge>`
- `fittings: Vec<Fitting>`
- `valves: Vec<(ElementId, usize)>`

  Valves, each on the node of its pipe where it sits.

- `attached: Vec<(ElementId, usize)>`

  Fixtures (on pipe nodes), devices (on conduit nodes) and air terminals
  and earth electrodes (on lightning conductor nodes) attached to the networks.


**Methods**

- `pub fn build(project: &Project, segments: &BTreeMap<ElementId, SegmentGeom>, valves: &BTreeMap<ElementId, ValveGeom>) -> Self`
- `pub fn adjacency(&self) -> Vec<Vec<(usize, usize)>>`

  Edges at each node: (edge index, the node at its other end).


<a id="mod-cc_geom-mesh"></a>

## `cc_geom::mesh`

<sub>`crates/cc_geom/src/mesh.rs`</sub>

Plan drawings of polyhedral meshes (component pieces imported from IFC):
the edges seen from above and the section at the cut plane.

**Items:** [`UP`](#cc_geom-mesh-up) · [`MeshPlan`](#cc_geom-mesh-meshplan) · [`plan_edges`](#cc_geom-mesh-plan_edges) · [`top_edges`](#cc_geom-mesh-top_edges) · [`mesh_plan`](#cc_geom-mesh-mesh_plan)

<a id="cc_geom-mesh-up"></a>

### const `UP`

```rust
pub(crate) const UP: f64
```

Faces with a normal Z component above this face upwards.

<a id="cc_geom-mesh-meshplan"></a>

### struct `MeshPlan`

```rust
pub struct MeshPlan
```

What a mesh shows in a plan cut at `cut_z`.

**Traits:** `Debug`, `Clone`, `Default`, `PartialEq`

**Fields**

- `cut: Vec<(DVec2, DVec2)>`

  Section of the mesh by the cut plane.

- `below: Vec<(DVec3, DVec3)>`

  Edges seen from above at or below the cut plane, in the model (their
  heights tell what hides them, see `crate::PlanCover`).

- `above: Vec<(DVec2, DVec2)>`

  Edges above the cut plane.


<a id="cc_geom-mesh-plan_edges"></a>

### fn `plan_edges`

```rust
pub fn plan_edges(faces: &[Vec<DVec3>]) -> Vec<(DVec3, DVec3)>
```

Edges of `faces` that read in a view from above: outlines of upward
faces where they meet faces turned away, open boundaries and sharp
creases (shared edges of smooth or flat regions are left out).

<a id="cc_geom-mesh-top_edges"></a>

### fn `top_edges`

```rust
pub fn top_edges(faces: &[Vec<DVec3>], smooth: &[bool]) -> Vec<(DVec3, DVec3)>
```

Edges of the closed piece `faces` seen from above, as views draw a
solid: the outlines of its faces turned up where they meet a face in
another plane, and between the facets of a curved surface only the
silhouette, where a facet turned up meets one turned away (the edges
between facets that both face up are never drawn, nor the pieces of
facets a boolean split). `smooth` flags the facets of curved surfaces,
one per face (see `crate::Solid::smooth_curved`); with any other
length they are found here. Each edge comes once, in a stable order.

<a id="cc_geom-mesh-mesh_plan"></a>

### fn `mesh_plan`

```rust
pub fn mesh_plan(faces: &[Vec<DVec3>], smooth: &[bool], cut_z: f64) -> MeshPlan
```

Plan of a mesh cut at `cut_z`, its edges as `top_edges` finds them
(`smooth`: its curved surfaces' facets, or empty).

<a id="mod-cc_geom-pad"></a>

## `cc_geom::pad`

<sub>`crates/cc_geom/src/pad.rs`</sub>

Building pads (`cc_core::pad`) graded into their terrain's surface.
For a pad at absolute elevation `z_p`, with `d` the plan distance from its
outline (0 inside), the graded surface is `F = max(L, min(T, C))`, where
`T` is the surface before the pad, `C = z_p + d / cut` the cut slope and
`L = z_p - d / fill` the fill slope. Pads apply one after another.
Only the surface's triangles near the pad (within the farthest distance a
slope can reach) are rebuilt: a constrained Delaunay triangulation of
their vertices, with their edges as constraints (split where the new ones
cross them), plus samples along rays out of the outline, the daylight
points where the slopes meet the ground (found by bisection) and the
outline and daylight lines as constraints. Every new triangle lies inside
one old triangle, so the old surface is linear on it, and its vertices
take `F`: cut and fill volumes are integrated exactly between the two.

**Items:** [`MAX_RINGS`](#cc_geom-pad-max_rings) · [`MAX_PAD_VERTICES`](#cc_geom-pad-max_pad_vertices) · [`PadWarning`](#cc_geom-pad-padwarning) · [`SlopeRay`](#cc_geom-pad-sloperay) · [`PadGeom`](#cc_geom-pad-padgeom) · [`Surface`](#cc_geom-pad-surface) · [`PadInput`](#cc_geom-pad-padinput) · [`grade`](#cc_geom-pad-grade) · [`pad_at`](#cc_geom-pad-pad_at) · [`mesh_boundary`](#cc_geom-pad-mesh_boundary)

<a id="cc_geom-pad-max_rings"></a>

### const `MAX_RINGS`

```rust
pub const MAX_RINGS: usize
```

Most offset rings (sample distances along each ray, doubling from `FIRST_RING`).

<a id="cc_geom-pad-max_pad_vertices"></a>

### const `MAX_PAD_VERTICES`

```rust
pub const MAX_PAD_VERTICES: usize
```

Most vertices a pad adds to the surface: past it, the slopes are sampled more coarsely.

<a id="cc_geom-pad-padwarning"></a>

### enum `PadWarning`

```rust
pub enum PadWarning
```

Something about a pad the user should know (shown with the element).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Variants**

- `Coarse`

  The slopes were sampled more coarsely than usual (a large pad or long slopes).

- `Failed`

  The grading failed; the terrain is left as it was before this pad.

- `OffTerrain`

  The pad or its slopes reach past the terrain's boundary, where they stop.


**Methods**

- `pub fn code(self) -> &'static str`

  A stable code (engine output and translation keys `pad-warning-<code>`).


<a id="cc_geom-pad-sloperay"></a>

### struct `SlopeRay`

```rust
pub struct SlopeRay
```

A slope sample: from a point of the outline out to where its slope meets
the ground (or the terrain's edge).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `origin: DVec2`
- `end: DVec2`
- `cut: bool`

  The ground stands above the pad here (else it lies below or level).

- `edge: Option<usize>`

  The outline edge it leaves from (`None` at a corner).


<a id="cc_geom-pad-padgeom"></a>

### struct `PadGeom`

```rust
pub struct PadGeom
```

A graded pad.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `outline: Vec<DVec2>`

  The outline, counter-clockwise.

- `z: f64`

  Absolute elevation.

- `cut: f64`

  Volume of earth removed and added.

- `fill: f64`
- `area: f64`

  Area in plan.

- `daylight: Vec<Vec<DVec2>>`

  Where the slopes meet the ground.

- `rays: Vec<SlopeRay>`

  The slope samples, in order around the outline.

- `warnings: Vec<PadWarning>`

<a id="cc_geom-pad-surface"></a>

### struct `Surface`

```rust
pub struct Surface
```

A surface being graded: triangles (absolute z, counter-clockwise from
above) and the elevation the original ground had at each of their vertices.

**Traits:** `Debug`, `Clone`, `Default`

**Fields**

- `triangles: Vec<[DVec3; 3]>`
- `original: Vec<[f64; 3]>`

**Methods**

- `pub fn new(triangles: Vec<[DVec3; 3]>) -> Self`
- `pub fn changed(&self) -> Vec<[DVec3; 3]>`

  The original ground where grading changed it (triangles at its elevations).


<a id="cc_geom-pad-padinput"></a>

### struct `PadInput`

```rust
pub struct PadInput<'a>
```

What a pad grades: its outline, absolute elevation and slope ratios.

**Traits:** `Debug`, `Clone`

**Fields**

- `outline: &'a [DVec2]`
- `z: f64`
- `cut_slope: f64`
- `fill_slope: f64`

<a id="cc_geom-pad-grade"></a>

### fn `grade`

```rust
pub fn grade(surface: &mut Surface, pad: &PadInput) -> PadGeom
```

Grade `pad` into `surface`. A failure leaves the surface as it was, with a warning.

<a id="cc_geom-pad-pad_at"></a>

### fn `pad_at`

```rust
pub fn pad_at(project: &cc_core::Project, terrain: cc_core::ElementId, p: DVec2) -> Option<cc_core::ElementId>
```

The pad of terrain `terrain` whose outline holds plan point `p` (the last
applied wins), so a click on the graded earth picks the pad there.

<a id="cc_geom-pad-mesh_boundary"></a>

### fn `mesh_boundary`

```rust
pub fn mesh_boundary(triangles: &[[DVec3; 3]]) -> Option<Vec<DVec3>>
```

The boundary of `triangles` (their unshared edges) as one loop,
counter-clockwise, or `None` when they do not make exactly one.

<a id="mod-cc_geom-paint"></a>

## `cc_geom::paint`

<sub>`crates/cc_geom/src/paint.rs`</sub>

Painted faces on the built solids: finding the face a paint
(`cc_core::paint::FacePaint`) refers to after a rebuild, and the
surface it covers.
The rule: among the faces of the element's body (framing and other
secondary parts left out) whose normal is within
`cc_core::paint::NORMAL_TOLERANCE_DEG` of the paint's, the one that
passes nearest the paint's point; then, from it, the faces that touch it
in the same plane (a wall face split around its openings) and, on a curved
surface, the smooth facets that join it, face after face.

**Items:** [`FaceRef`](#cc_geom-paint-faceref) · [`distance_to_face`](#cc_geom-paint-distance_to_face) · [`nearest_face`](#cc_geom-paint-nearest_face) · [`surface`](#cc_geom-paint-surface) · [`resolve`](#cc_geom-paint-resolve) · [`body_solids`](#cc_geom-paint-body_solids) · [`face_hit`](#cc_geom-paint-face_hit) · [`apply`](#cc_geom-paint-apply) · [`painted_area`](#cc_geom-paint-painted_area)

<a id="cc_geom-paint-faceref"></a>

### type alias `FaceRef`

```rust
pub type FaceRef = (usize, usize)
```

A face of one of an element's solids: the solid's index in the list
searched, and the face's index in it.

<a id="cc_geom-paint-distance_to_face"></a>

### fn `distance_to_face`

```rust
pub fn distance_to_face(face: &Face, p: DVec3) -> f64
```

Distance from `p` to the polygon of `face`.

<a id="cc_geom-paint-nearest_face"></a>

### fn `nearest_face`

```rust
pub fn nearest_face(solids: &[&Solid], point: DVec3, normal: Option<DVec3>) -> Option<(FaceRef, f64)>
```

The face of `solids` nearest `point` (with a normal within the tolerance
of `normal`, when given), and its distance.

<a id="cc_geom-paint-surface"></a>

### fn `surface`

```rust
pub fn surface(solids: &[&Solid], start: FaceRef) -> Vec<FaceRef>
```

The surface of face `start`: it and, face after face, those touching it
in its plane or (from a smooth facet) the smooth facets joining it.

<a id="cc_geom-paint-resolve"></a>

### fn `resolve`

```rust
pub fn resolve(solids: &[&Solid], paint: &FacePaint) -> Vec<FaceRef>
```

The faces `paint` covers among `solids` (none when no face is turned like it).

<a id="cc_geom-paint-body_solids"></a>

### fn `body_solids`

```rust
pub fn body_solids(geom: &ModelGeometry, id: ElementId) -> Vec<&Solid>
```

The faces of an element's body: its solids that are not secondary parts.

<a id="cc_geom-paint-face_hit"></a>

### fn `face_hit`

```rust
pub fn face_hit(geom: &ModelGeometry, id: ElementId, ray: crate::Ray) -> Option<(DVec3, DVec3)>
```

Where `ray` meets the body of element `id`: the point and the normal of
the face it meets first.

<a id="cc_geom-paint-apply"></a>

### fn `apply`

```rust
pub(crate) fn apply(project: &Project, solids: &mut [Solid])
```

Paint the solids of every element with paints (a later paint wins where
two cover the same face).

<a id="cc_geom-paint-painted_area"></a>

### fn `painted_area`

```rust
pub fn painted_area(solids: &[&Solid]) -> BTreeMap<String, f64>
```

Area painted with each material, square meters (the faces each paint covers).

<a id="mod-cc_geom-pile"></a>

## `cc_geom::pile`

<sub>`crates/cc_geom/src/pile.rs`</sub>

Piles and pile caps (see `cc_core::pile`): their bodies, plans and
exact quantities. A round pile is a cylinder of smooth facets, a square
one a prism; a cap is a prism of its outline, a round cap a cylinder and
a ring cap an annulus, their curved faces smooth. Volumes and areas are
exact (circles as circles, not chords).

**Items:** [`circle_segments`](#cc_geom-pile-circle_segments) · [`PileGeom`](#cc_geom-pile-pilegeom) · [`pile_geom`](#cc_geom-pile-pile_geom) · [`CapGeom`](#cc_geom-pile-capgeom) · [`cap_geom`](#cc_geom-pile-cap_geom) · [`add_cylinder`](#cc_geom-pile-add_cylinder) · [`add_ring`](#cc_geom-pile-add_ring)

<a id="cc_geom-pile-circle_segments"></a>

### fn `circle_segments`

```rust
pub fn circle_segments() -> usize
```

Number of chords of a full circle.

<a id="cc_geom-pile-pilegeom"></a>

### struct `PileGeom`

```rust
pub struct PileGeom
```

A pile in the world.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `element: ElementId`
- `position: DVec2`

  Axis in plan.

- `rotation_deg: f64`

  Rotation of a square section, degrees.

- `section: PileSection`
- `z_range: (f64, f64)`

  Elevations of the toe and the head (cut-off).

- `cap: Option<ElementId>`

  The pile cap it belongs to.


**Methods**

- `pub fn length(&self) -> f64`

  Length from the toe to the head.

- `pub fn volume(&self) -> f64`

  Exact volume.

- `pub fn outline(&self) -> Vec<DVec2>`

  Plan outline, counter-clockwise (a circle as chords).

- `pub fn add_to(&self, s: &mut Solid)`

  Add the body to `s`.


<a id="cc_geom-pile-pile_geom"></a>

### fn `pile_geom`

```rust
pub fn pile_geom(project: &Project, e: &cc_core::Element, p: &cc_core::pile::Pile) -> Option<PileGeom>
```

The pile `e` in the world (`None` when its type is no pile type or it has no length).

<a id="cc_geom-pile-capgeom"></a>

### struct `CapGeom`

```rust
pub struct CapGeom
```

A pile cap's frame and plan, for its geometry and reinforcement.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `position: DVec2`

  Origin of the cap's frame in plan.

- `rotation_deg: f64`

  Direction of the frame's `u` axis, degrees counter-clockwise (`v` to its left).

- `outline: CapOutline`

  The plan in the cap's frame.

- `piles: Vec<DVec2>`

  The piles' axes in the cap's frame, in layout order.


**Methods**

- `pub fn world(&self, local: DVec2) -> DVec2`

  A point of the cap's frame in plan.

- `pub fn loops(&self) -> Vec<Vec<DVec2>>`

  The outer boundary and the holes in plan, counter-clockwise (circles as chords).

- `pub fn add_to(&self, s: &mut Solid, z0: f64, z1: f64)`

  Add the body between `z0` and `z1` to `s`: curved faces as smooth facets.


<a id="cc_geom-pile-cap_geom"></a>

### fn `cap_geom`

```rust
pub fn cap_geom(project: &Project, e: &cc_core::Element) -> Option<CapGeom>
```

The cap `e` in the world (`None` when it is no pile cap).

<a id="cc_geom-pile-add_cylinder"></a>

### fn `add_cylinder`

```rust
pub(crate) fn add_cylinder(s: &mut Solid, center: DVec2, radius: f64, z0: f64, z1: f64)
```

A vertical cylinder: flat top and bottom, its side as smooth facets.

<a id="cc_geom-pile-add_ring"></a>

### fn `add_ring`

```rust
pub(crate) fn add_ring(s: &mut Solid, center: DVec2, inner: f64, outer: f64, z0: f64, z1: f64)
```

A vertical annulus: flat top and bottom (one quad per chord), its inner
and outer sides as smooth facets.

<a id="mod-cc_geom-plan_cover"></a>

## `cc_geom::plan_cover`

<sub>`crates/cc_geom/src/plan_cover.rs`</sub>

Hidden lines of plans. A plan sees what lies below its cut plane from
above: an edge shows only where no face lies over it, so the faces of
the pieces turned up below the cut plane, and their sections at it,
hide the parts of the edges they cover from above.
The test is exact for planar faces: an edge is cut where it crosses the
outline of a face seen from above, and along each stretch inside the
outline the height of the face over the edge varies linearly, so the
stretch is hidden from where the face rises above the edge. A margin
(a line's width on paper) also hides what runs that close along the
outline of a face over it, which would only thicken that outline.

**Items:** [`PlanCover`](#cc_geom-plan_cover-plancover)

<a id="cc_geom-plan_cover-plancover"></a>

### struct `PlanCover`

```rust
pub struct PlanCover
```

What hides edges in a plan cut at `cut_z` (see the module): faces and
sections added, then edges asked about with `PlanCover::visible`.

**Traits:** `Debug`, `Clone`

*Some fields are private.*

**Methods**

- `pub fn new(cut_z: f64) -> Self`
- `pub fn with_margin(mut self, margin: f64) -> Self`

  Hide too the stretches of edges that run within `margin` (in plan)
  of a face of another piece lying over them all along: a line that
  close to the outline of what covers it merges with it on paper, and
  two outlines of one curve drawn with different chords would
  otherwise leave the slivers of the lower one between the chords of
  the upper one. A piece's own faces never hide its edges so (its
  surface rising from its outline is no other outline).

- `pub fn begin_piece(&mut self) -> usize`

  Start a new piece: the faces added from now on belong to it. Returns
  its number, to give `Self::visible` with the piece's own edges.

- `pub fn is_empty(&self) -> bool`
- `pub fn add_faces(&mut self, faces: &[Vec<DVec3>])`

  The faces of a closed piece: those turned up hide what is under
  them, their parts below the cut plane. A face that is not flat (a
  blend's twisted side) hides as its triangles, each in its own plane,
  so it never hides its own edges.

- `pub fn add_section(&mut self, segments: &[(DVec2, DVec2)])`

  A piece's section at the cut plane (its segments, in any order):
  it hides everything under it. Left out unless the segments close.

- `pub fn add_outline(&mut self, outline: &[DVec2], z: f64)`

  A level region at height `z` (a prism's top; at the cut plane when
  it rises through it).

- `pub fn visible(&mut self, a: DVec3, b: DVec3, own: Option<usize>) -> Vec<(DVec2, DVec2)>`

  The parts of the edge `a..b` of piece `own` (see
  `Self::begin_piece`) seen from above (in plan), in order from `a`:
  none where a face lies over it (or, of another piece, within the
  margin of it all along, see `Self::with_margin`).


<a id="mod-cc_geom-polygon"></a>

## `cc_geom::polygon`

<sub>`crates/cc_geom/src/polygon.rs`</sub>

2D polygon helpers.

**Items:** [`signed_area`](#cc_geom-polygon-signed_area) · [`area`](#cc_geom-polygon-area) · [`perimeter`](#cc_geom-polygon-perimeter) · [`MAX_HULL_POINTS`](#cc_geom-polygon-max_hull_points) · [`convex_hull`](#cc_geom-polygon-convex_hull) · [`ccw`](#cc_geom-polygon-ccw) · [`offset`](#cc_geom-polygon-offset) · [`contains`](#cc_geom-polygon-contains) · [`triangulate`](#cc_geom-polygon-triangulate) · [`triangulate_with_holes`](#cc_geom-polygon-triangulate_with_holes) · [`keyhole`](#cc_geom-polygon-keyhole) · [`clip_line`](#cc_geom-polygon-clip_line) · [`closest_on_segment`](#cc_geom-polygon-closest_on_segment) · [`line_intersect`](#cc_geom-polygon-line_intersect) · [`segment_meets_rect`](#cc_geom-polygon-segment_meets_rect)

<a id="cc_geom-polygon-signed_area"></a>

### fn `signed_area`

```rust
pub fn signed_area(poly: &[DVec2]) -> f64
```

Signed area (positive for counter-clockwise).

<a id="cc_geom-polygon-area"></a>

### fn `area`

```rust
pub fn area(poly: &[DVec2]) -> f64
```

<a id="cc_geom-polygon-perimeter"></a>

### fn `perimeter`

```rust
pub fn perimeter(poly: &[DVec2]) -> f64
```

<a id="cc_geom-polygon-max_hull_points"></a>

### const `MAX_HULL_POINTS`

```rust
pub const MAX_HULL_POINTS: usize
```

Most points `convex_hull` takes; the rest are left out.

<a id="cc_geom-polygon-convex_hull"></a>

### fn `convex_hull`

```rust
pub fn convex_hull(points: &[DVec2]) -> Vec<DVec2>
```

The convex hull of `points`, counter-clockwise, without collinear points
(Andrew's monotone chain). Non-finite points are skipped; fewer than three
distinct points give them back as they are.

<a id="cc_geom-polygon-ccw"></a>

### fn `ccw`

```rust
pub fn ccw(poly: &[DVec2]) -> Vec<DVec2>
```

Copy of `poly` wound counter-clockwise.

<a id="cc_geom-polygon-offset"></a>

### fn `offset`

```rust
pub fn offset(poly: &[DVec2], d: f64) -> Vec<DVec2>
```

Offset a simple polygon outward by `d` using mitered corners.

<a id="cc_geom-polygon-contains"></a>

### fn `contains`

```rust
pub fn contains(poly: &[DVec2], pt: DVec2) -> bool
```

<a id="cc_geom-polygon-triangulate"></a>

### fn `triangulate`

```rust
pub fn triangulate(poly: &[DVec2]) -> Vec<[usize; 3]>
```

Triangulate a simple polygon; returns index triples (none when a point
is not finite: earcutr panics on such input).

<a id="cc_geom-polygon-triangulate_with_holes"></a>

### fn `triangulate_with_holes`

```rust
pub fn triangulate_with_holes(outer: &[DVec2], holes: &[Vec<DVec2>]) -> Vec<[DVec2; 3]>
```

Triangulate a polygon with holes; triangles keep the winding of `outer`.

<a id="cc_geom-polygon-keyhole"></a>

### fn `keyhole`

```rust
pub fn keyhole(outer: &[DVec2], holes: &[Vec<DVec2>]) -> Option<Vec<DVec2>>
```

One ring covering `outer` less `holes` (simple, inside it, apart): the
outline counter-clockwise with each hole, clockwise, joined to it by a
bridge walked there and back. A single polygon with holes, for painters
and formats that fill one ring (nonzero and even-odd fills both leave
the holes out). `None` when a bridge could not be found within the work
limit or the input is degenerate; callers then fall back to
`triangulate_with_holes`.

<a id="cc_geom-polygon-clip_line"></a>

### fn `clip_line`

```rust
pub fn clip_line(poly: &[DVec2], origin: DVec2, dir: DVec2) -> Vec<(f64, f64)>
```

Parts of the infinite line `origin + t*dir` inside `poly`, as `(t0, t1)` pairs.

<a id="cc_geom-polygon-closest_on_segment"></a>

### fn `closest_on_segment`

```rust
pub fn closest_on_segment(a: DVec2, b: DVec2, p: DVec2) -> (DVec2, f64)
```

Closest point on segment `a`-`b` to `p` and the segment parameter.

<a id="cc_geom-polygon-line_intersect"></a>

### fn `line_intersect`

```rust
pub fn line_intersect(p1: DVec2, d1: DVec2, p2: DVec2, d2: DVec2) -> Option<f64>
```

Intersection of lines `p1 + t*d1` and `p2 + s*d2`, returning `t`.

<a id="cc_geom-polygon-segment_meets_rect"></a>

### fn `segment_meets_rect`

```rust
pub fn segment_meets_rect(a: DVec2, b: DVec2, lo: DVec2, hi: DVec2) -> bool
```

Whether segment `a`–`b` passes through the rectangle `lo`..`hi`
(Liang–Barsky clipping; touching counts).

<a id="mod-cc_geom-profile_sweep"></a>

## `cc_geom::profile_sweep`

<sub>`crates/cc_geom/src/profile_sweep.rs`</sub>

The solid of a swept profile: its type's profile carried along its path
in plan, at the level's elevation plus its base offset, mitered at the
corners (and around a closed path).

**Items:** [`sweep_profile`](#cc_geom-profile_sweep-sweep_profile) · [`sweep_faces`](#cc_geom-profile_sweep-sweep_faces)

<a id="cc_geom-profile_sweep-sweep_profile"></a>

### fn `sweep_profile`

```rust
pub fn sweep_profile(project: &Project, element: &Element, sweep: &Sweep) -> Option<Vec<DVec2>>
```

The profile of a sweep's type, on the side the sweep puts it.

<a id="cc_geom-profile_sweep-sweep_faces"></a>

### fn `sweep_faces`

```rust
pub fn sweep_faces(project: &Project, element: &Element, sweep: &Sweep) -> Option<Vec<Vec<DVec3>>>
```

The faces of a sweep (outward), or `None` when its path or profile cannot make one.

<a id="mod-cc_geom-ramp"></a>

## `cc_geom::ramp`

<sub>`crates/cc_geom/src/ramp.rs`</sub>

Ramp geometry: the sloped flights and flat landings of a ramp element
laid out from its shape, its slab solids and the paths its railings
follow.
Like stairs, shapes are built in a local frame `(a, b)`: `a` along the
first flight from the middle of its foot, `b` across it towards the side
the ramp turns to; a ramp turning right is the mirror image.

**Items:** [`RampPiece`](#cc_geom-ramp-ramppiece) · [`RampGeom`](#cc_geom-ramp-rampgeom) · [`ramp_geom`](#cc_geom-ramp-ramp_geom) · [`layout_ramp`](#cc_geom-ramp-layout_ramp)

<a id="cc_geom-ramp-ramppiece"></a>

### struct `RampPiece`

```rust
pub struct RampPiece
```

One surface of a ramp: a sloped flight or a flat landing.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `outline: Vec<DVec2>`

  Plan outline (world).

- `z: Vec<f64>`

  Elevation of the walking surface at each outline vertex.

- `landing: bool`

<a id="cc_geom-ramp-rampgeom"></a>

### struct `RampGeom`

```rust
pub struct RampGeom
```

Layout of a ramp element.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `start: DVec2`
- `u: DVec2`
- `v: DVec2`
- `width: f64`
- `base: f64`
- `top: f64`
- `layout: RampLayout`
- `check: RampCheck`
- `landing: f64`

  Landing length along the way.

- `pieces: Vec<RampPiece>`
- `walkline: Vec<DVec3>`

  Walking line from the foot to the top, with elevations.

- `center: Option<DVec2>`

  Center of a spiral ramp.


**Methods**

- `pub fn run(&self) -> f64`

  Plan length of the walking line.

- `pub fn pieces(&self, thickness: f64) -> Pieces`

  The slab of each piece, `thickness` thick square to its slope.
  The slab of each piece, `thickness` thick square to its slope; a
  flight whose underside meets the floor is split there, so the slab
  keeps its thickness above it.

- `pub fn add_to(&self, thickness: f64, solid: &mut crate::Solid)`
- `pub fn railing_path(&self, side: StairSide, inset: f64) -> Vec<DVec3>`

  Path of a railing along `side`, `inset` in from the edge, at the
  height of the walking surface.


<a id="cc_geom-ramp-ramp_geom"></a>

### fn `ramp_geom`

```rust
pub fn ramp_geom(project: &Project, element: &cc_core::Element, ramp: &cc_core::Ramp) -> Option<RampGeom>
```

Layout of a ramp element from its type, constraints and shape.

<a id="cc_geom-ramp-layout_ramp"></a>

### fn `layout_ramp`

```rust
pub fn layout_ramp(start: DVec2, u: DVec2, width: f64, base: f64, top: f64, rules: &RampRules, layout: RampLayout) -> RampGeom
```

Lay out a ramp from its foot `start`, first-flight direction `u`, width
and elevations.

<a id="mod-cc_geom-rebar"></a>

## `cc_geom::rebar`

<sub>`crates/cc_geom/src/rebar.rs`</sub>

Reinforcing bars: the center lines of each set of bars in its beam or
column, from the host's body, the cover and the bar diameters.
Longitudinal bars sit inside the stirrups (cover plus the host's largest
stirrup diameter); stirrups are closed loops `cover` inside the faces,
spaced evenly and centered along the host. Along an arc beam the bars
follow the arc, each at its own radius, and the stirrups are radial. Top
and bottom bars that do not fit side by side go in further layers, side
bars run between them, and bars longer than the stock length are spliced.
A column's bars stand around its section (`crate::faces`): a count
spread around it, or so many on each face with the corner bars shared;
its ties follow the section's outline, cross-ties link the bars between
the corners to the bars opposite, and the vertical bars may be cut in
lifts that lap.

**Items:** [`STEEL_DENSITY`](#cc_geom-rebar-steel_density) · [`MAX_BARS`](#cc_geom-rebar-max_bars) · [`Bar`](#cc_geom-rebar-bar) · [`RebarGeom`](#cc_geom-rebar-rebargeom) · [`stations`](#cc_geom-rebar-stations) · [`build_rebar`](#cc_geom-rebar-build_rebar) · [`run_past`](#cc_geom-rebar-run_past) · [`column_faces`](#cc_geom-rebar-column_faces) · [`hook`](#cc_geom-rebar-hook)

<a id="cc_geom-rebar-steel_density"></a>

### const `STEEL_DENSITY`

```rust
pub const STEEL_DENSITY: f64
```

Density of steel, kg/m³.

<a id="cc_geom-rebar-max_bars"></a>

### const `MAX_BARS`

```rust
pub(crate) const MAX_BARS: usize
```

At most this many bars in one set, splice pieces included.

<a id="cc_geom-rebar-bar"></a>

### struct `Bar`

```rust
pub struct Bar
```

One bar: its center line, closed for stirrups.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `points: Vec<DVec3>`
- `closed: bool`
- `true_length: Option<f64>`

  The bar's true length when its center line is drawn with chords (a
  bar following an arc): the arc's length at the bar's radius.


**Methods**

- `pub fn new(points: Vec<DVec3>, closed: bool) -> Self`

  A bar along straight segments.

- `pub fn length(&self) -> f64`

<a id="cc_geom-rebar-rebargeom"></a>

### struct `RebarGeom`

```rust
pub struct RebarGeom
```

The bars of one reinforcement set.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `element: ElementId`
- `host: Option<ElementId>`

  The beam or column the set is laid out in (none for shaped bars).

- `diameter: f64`
- `grade: String`

  The steel grade of the set's type (empty: none).

- `mass_per_meter: Option<f64>`

  The type's mass per meter, kg/m, when it gives one.

- `bars: Vec<Bar>`

**Methods**

- `pub fn mass_per_meter(&self) -> f64`

  Mass of a meter of bar, kilograms: the type's, else a round bar of
  the diameter in steel of `STEEL_DENSITY`.

- `pub fn count(&self) -> usize`
- `pub fn total_length(&self) -> f64`
- `pub fn bar_length(&self) -> f64`

  Mean length of one bar.

- `pub fn weight(&self) -> f64`

  Steel weight in kilograms.


<a id="cc_geom-rebar-stations"></a>

### fn `stations`

```rust
pub(crate) fn stations(a: f64, b: f64, spacing: f64) -> Vec<f64>
```

Positions `spacing` apart filling `[a, b]`, centered.

<a id="cc_geom-rebar-build_rebar"></a>

### fn `build_rebar`

```rust
pub fn build_rebar(project: &Project, beams: &BTreeMap<ElementId, BeamGeom>, walls: &BTreeMap<ElementId, crate::WallGeom>, foundations: &BTreeMap<ElementId, crate::FoundationGeom>) -> BTreeMap<ElementId, RebarGeom>
```

Bars of every reinforcement set whose host has a body.

<a id="cc_geom-rebar-run_past"></a>

### fn `run_past`

```rust
pub(crate) fn run_past(inside: (f64, f64), cover: f64, past: (f64, f64), scale: f64) -> (f64, f64)
```

The stretch of the axis a bar along its host covers: `inside` is the
stretch the `cover` clear of both end faces, and where the bar runs
`past` an end (a length on its own line, `scale` times the axis's) it
starts that far beyond the face instead.

<a id="cc_geom-rebar-column_faces"></a>

### fn `column_faces`

```rust
pub fn column_faces(project: &Project, data: &TypeData, inset: f64) -> Vec<SectionFace>
```

The faces of a column type's section `inset` inside it, in the order
bars are given face by face: counter-clockwise, an annular sector's
from its outer arc. Empty for other types and failed sections.

<a id="cc_geom-rebar-hook"></a>

### fn `hook`

```rust
pub(crate) fn hook(turn: f64, d: f64) -> f64
```

Length of a tie's hook bent by `turn` radians: the bend around a pin
three bar diameters across, then a straight end of five diameters (at
least 5 cm).

<a id="mod-cc_geom-region"></a>

## `cc_geom::region`

<sub>`crates/cc_geom/src/region.rs`</sub>

Regions covered by polygons: the union of simple polygons in a plane,
optionally less what other polygons cover and kept within another
polygon, as rings a painter can fill.
The polygons' edges are first welded (`weld`): points closer than a
tolerance become one, and every edge is split where another crosses it
or a point lies on it, so the edges meet only at their ends. They go as
constraints into a constrained Delaunay triangulation, every triangle is
in or out of the region by its centroid, and the region's outline is the
edges between a triangle in it and one out of it: always closed loops,
whatever the input. Bounded by the number of points and edges, errors
as values.

**Items:** [`covered`](#cc_geom-region-covered) · [`covered_except`](#cc_geom-region-covered_except)

<a id="cc_geom-region-covered"></a>

### fn `covered`

```rust
pub fn covered(polys: &[Vec<DVec2>], clip: Option<&[DVec2]>, max_points: usize) -> Option<Vec<Vec<DVec2>>>
```

The region covered by any of `polys` (simple polygons of either winding,
which may overlap, touch or cross), within `clip` (a simple polygon) when
given: rings to fill, each a simple polygon running counter-clockwise
around a piece of the region with its holes joined to the outline by
bridges walked there and back (`polygon::keyhole`; a piece whose holes
could not be joined comes as triangles). The rings do not overlap.
`None` when the polygons have more than `max_points` points in all, a
coordinate is not a finite number, the polygons cross one another too
many times (`MAX_EDGES`, `MAX_WELD_WORK`), the triangulation fails or
the outlines found do not make a region (one left open, a hole inside
none of them): never a region that covers more than the polygons do.
Nothing covered is `Some` of no rings.

<a id="cc_geom-region-covered_except"></a>

### fn `covered_except`

```rust
pub fn covered_except(polys: &[Vec<DVec2>], except: &[Vec<DVec2>], clip: Option<&[DVec2]>, max_points: usize) -> Option<Vec<Vec<DVec2>>>
```

The region covered by any of `polys` and by none of `except` (simple
polygons too, of either winding, which may overlap one another and the
others), within `clip` when given: rings as `covered` gives them, and
`None` in the same cases, the points of `except` counting among the
`max_points`.

<a id="mod-cc_geom-roof_framing"></a>

## `cc_geom::roof_framing`

<sub>`crates/cc_geom/src/roof_framing.rs`</sub>

Roof framing: the purlins, rafters and battens under a roof's sloped
faces, from its type's framing and covering (see `cc_core::roofing`).
Each sloped underside of the roof's body is framed on its own: battens
along the eaves at the tile's gauge, right under the roof; rafters down
the slope under the battens; purlins along the eaves under the rafters,
spread evenly from eave to ridge no farther apart than the purlin step.
Steel framing has purlins only, right under the sheets. Members are
boxes with their height across the roof plane. Coplanar faces are framed
together (a face with openings comes as several polygons).

**Items:** [`MemberKind`](#cc_geom-roof_framing-memberkind) · [`TrussPart`](#cc_geom-roof_framing-trusspart) · [`Member`](#cc_geom-roof_framing-member) · [`RoofFramingGeom`](#cc_geom-roof_framing-roofframinggeom) · [`roof_framing`](#cc_geom-roof_framing-roof_framing)

<a id="cc_geom-roof_framing-memberkind"></a>

### enum `MemberKind`

```rust
pub enum MemberKind
```

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`

**Variants**

- `Purlin`
- `Rafter`
- `Batten`
- `Truss`

  A member of a truss: top chord, tie, king post or strut.


<a id="cc_geom-roof_framing-trusspart"></a>

### enum `TrussPart`

```rust
pub enum TrussPart
```

The part of a truss a member is.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`

**Variants**

- `Chord`

  A top chord, along a roof face.

- `Tie`

  The level tie (bottom chord).

- `Post`

  The king post, under the apex.

- `Strut`

  A strut, from the post's foot to a chord.


<a id="cc_geom-roof_framing-member"></a>

### struct `Member`

```rust
pub struct Member
```

A framing member: its axis (along the roof plane), section, the
in-plane direction its width runs along and the direction its height
runs down (away from the roof), with the depth below the roof's
underside where it starts.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `kind: MemberKind`
- `a: DVec3`
- `b: DVec3`
- `size: MemberSize`
- `truss: Option<TrussPart>`

  Which part of its truss, for truss members.


*Some fields are private.*

**Methods**

- `pub fn length(&self) -> f64`
- `pub fn faces(&self) -> Vec<crate::Face>`

  The member's box as polygons, faces turned outward.

- `pub fn add_to(&self, solid: &mut Solid)`

  Add the member's box to `solid`, faces turned outward.


<a id="cc_geom-roof_framing-roofframinggeom"></a>

### struct `RoofFramingGeom`

```rust
pub struct RoofFramingGeom
```

The framing of one roof and the figures of its covering.

**Traits:** `Debug`, `Clone`, `Default`, `PartialEq`

**Fields**

- `members: Vec<Member>`
- `sloped_area: f64`

  True area of the sloped faces (what the covering covers).

- `slopes: Option<(f64, f64)>`

  The gentlest and steepest slopes of the faces, rise over run.

- `trusses: usize`

  How many trusses carry the purlins.


**Methods**

- `pub fn length(&self, kind: MemberKind) -> f64`

  Total length of the members of `kind`.

- `pub fn total_length(&self) -> f64`

  Total length of every member.

- `pub fn volume(&self) -> f64`

  Volume of every member.


<a id="cc_geom-roof_framing-roof_framing"></a>

### fn `roof_framing`

```rust
pub fn roof_framing(roof: &Solid, framing: &RoofFraming, covering: Option<TileKind>) -> RoofFramingGeom
```

The framing under a roof's body `roof` (its solid) for `framing` and
`covering`.

<a id="mod-cc_geom-roofs"></a>

## `cc_geom::roofs`

<sub>`crates/cc_geom/src/roofs.rs`</sub>

Pitched roofs (gable and hip) over any footprint, courtyards included:
every sloped eave raises a plane and every gable edge a vertical plane;
the weighted straight skeleton (`crate::skeleton`) gives the part of
each plane over the footprint, with its hips, valleys and ridges.
The older construction, kept as a fallback when the skeleton fails,
takes the lowest of the planes rising from each sloped eave: right for
convex footprints only.

**Items:** [`Eave`](#cc_geom-roofs-eave) · [`edge_slopes`](#cc_geom-roofs-edge_slopes) · [`ccw_edge_slopes`](#cc_geom-roofs-ccw_edge_slopes) · [`sloped_eaves`](#cc_geom-roofs-sloped_eaves) · [`roof_regions`](#cc_geom-roofs-roof_regions) · [`ridge_lines`](#cc_geom-roofs-ridge_lines) · [`add_pitched_roof`](#cc_geom-roofs-add_pitched_roof) · [`plan_area`](#cc_geom-roofs-plan_area) · [`eave_weights`](#cc_geom-roofs-eave_weights) · [`add_skeleton_roof`](#cc_geom-roofs-add_skeleton_roof)

<a id="cc_geom-roofs-eave"></a>

### struct `Eave`

```rust
pub struct Eave
```

One plane of a pitched roof: rises with `slope` from the eave through
`a` (counter-clockwise footprint, so the inside is on the left).

**Traits:** `Debug`, `Clone`, `Copy`

**Fields**

- `a: DVec2`
- `normal: DVec2`
- `slope: f64`

**Methods**

- `pub fn height(&self, p: DVec2) -> f64`

  Height above the eave of the plane at `p`.


<a id="cc_geom-roofs-edge_slopes"></a>

### fn `edge_slopes`

```rust
pub fn edge_slopes(pts: &[DVec2], slope: f64, ridge: Option<DVec2>, own: &[Option<f64>]) -> Vec<f64>
```

The slope of each edge of a counter-clockwise loop: its own slope when
`own` sets one (0 = a gable end), else `slope` on a hip roof (no
`ridge`) or on edges within 45° of the ridge direction, 0 on the others.

<a id="cc_geom-roofs-ccw_edge_slopes"></a>

### fn `ccw_edge_slopes`

```rust
pub fn ccw_edge_slopes(r: &cc_core::Roof) -> Vec<Option<f64>>
```

A roof's own edge slopes in the order of its counter-clockwise outline
(`polygon::ccw` reverses a clockwise footprint, and with it the edges).

<a id="cc_geom-roofs-sloped_eaves"></a>

### fn `sloped_eaves`

```rust
pub fn sloped_eaves(footprint: &[DVec2], slopes: &[f64]) -> Vec<Eave>
```

The sloped eaves of a counter-clockwise footprint, given the slope of
each edge (see `edge_slopes`); edges of slope 0 are gables.

<a id="cc_geom-roofs-roof_regions"></a>

### fn `roof_regions`

```rust
pub fn roof_regions(footprint: &[DVec2], eaves: &[Eave]) -> Vec<(usize, Vec<DVec2>)>
```

Part of the footprint owned by each eave's plane (empty parts are skipped).

<a id="cc_geom-roofs-ridge_lines"></a>

### fn `ridge_lines`

```rust
pub fn ridge_lines(footprint: &[DVec2], regions: &[(usize, Vec<DVec2>)]) -> Vec<(DVec2, DVec2)>
```

Ridge and hip lines in plan: region edges that are not on the footprint boundary.

<a id="cc_geom-roofs-add_pitched_roof"></a>

### fn `add_pitched_roof`

```rust
pub fn add_pitched_roof(solid: &mut Solid, footprint: &[DVec2], eaves: &[Eave], base: f64, thickness: f64)
```

Add the closed solid of a pitched roof of vertical thickness `thickness`
whose underside is at `base` along the eaves.

<a id="cc_geom-roofs-plan_area"></a>

### fn `plan_area`

```rust
pub fn plan_area(r: &cc_core::Roof) -> f64
```

Plan area covered by a roof: its outline grown by the overhang, less
its openings shrunk by it.

<a id="cc_geom-roofs-eave_weights"></a>

### fn `eave_weights`

```rust
pub fn eave_weights(loops: &[Vec<DVec2>], slope: f64, ridge: Option<DVec2>, own: &[Option<f64>]) -> Vec<Vec<f64>>
```

Edge weights of a footprint's loops for the skeleton: sloped eaves move
in by `1 / slope` per unit of height, gables (see `edge_slopes`) stay.
`own` holds the outline's own edge slopes; openings follow the shape.

<a id="cc_geom-roofs-add_skeleton_roof"></a>

### fn `add_skeleton_roof`

```rust
pub fn add_skeleton_roof(solid: &mut Solid, sk: &crate::skeleton::Skeleton, base: f64, thickness: f64)
```

Add the closed solid of a roof from its skeleton: each plane's part
lifted from `base` (its underside) with `thickness` measured vertically,
flat tops where the planes never close, and vertical faces along the
eaves and the gables.

<a id="mod-cc_geom-rooms"></a>

## `cc_geom::rooms`

<sub>`crates/cc_geom/src/rooms.rs`</sub>

Room boundaries: the region enclosed by room-bounding walls around a point.
The wall footprints form a planar arrangement once their edges are split
where they meet. A ray from the room point finds the nearest edge, and the
face containing the point is traced by turning as far left as possible at
every vertex. Openings do not break the boundary (footprints are full walls).

**Items:** [`room_boundary`](#cc_geom-rooms-room_boundary) · [`RoomArrangement`](#cc_geom-rooms-roomarrangement) · [`outer_boundary`](#cc_geom-rooms-outer_boundary)

<a id="cc_geom-rooms-room_boundary"></a>

### fn `room_boundary`

```rust
pub fn room_boundary(walls: &[Vec<DVec2>], point: DVec2) -> Option<Vec<DVec2>>
```

Boundary (counter-clockwise) of the region enclosed by `walls` (wall
footprints) that contains `point`, or `None` if the point is not enclosed.

<a id="cc_geom-rooms-roomarrangement"></a>

### struct `RoomArrangement`

```rust
pub struct RoomArrangement
```

The arrangement of a set of wall footprints, made once to find the
boundaries of many rooms among the same walls (see `room_boundary`).

*Some fields are private.*

**Methods**

- `pub fn new(walls: Vec<Vec<DVec2>>) -> Self`
- `pub fn room_boundary(&self, point: DVec2) -> Option<Vec<DVec2>>`

  `room_boundary` among these walls.

- `pub fn spaces(&self) -> Vec<Vec<DVec2>>`

  Every space the walls' outlines close, their own bodies among them:
  the bounded faces of the arrangement, each counter-clockwise (a face
  around others, such as a room around a free-standing wall, as its
  outer boundary alone). Whatever lies in none of them is open to the
  outside. Each face is traced once.


<a id="cc_geom-rooms-outer_boundary"></a>

### fn `outer_boundary`

```rust
pub fn outer_boundary(lines: &[Vec<DVec2>]) -> Option<Vec<DVec2>>
```

The outer boundary (counter-clockwise) of the arrangement of `lines`
(open polylines, such as wall location lines): the loop around the
largest enclosed area, with lines inside it and lines hanging off it left
out. `None` when the lines enclose nothing.

<a id="mod-cc_geom-section"></a>

## `cc_geom::section`

<sub>`crates/cc_geom/src/section.rs`</sub>

Cross-sections of columns and beams: type sections at each detail level,
placed column bodies with smooth arc facets, and the legacy section
shapes as rectangles and outlines (legends and IFC export).

**Items:** [`shape_rects`](#cc_geom-section-shape_rects) · [`shape_loops`](#cc_geom-section-shape_loops) · [`detailed`](#cc_geom-section-detailed) · [`detailed_section`](#cc_geom-section-detailed_section) · [`column_body`](#cc_geom-section-column_body) · [`smooth_sides`](#cc_geom-section-smooth_sides) · [`add_column_body`](#cc_geom-section-add_column_body)

<a id="cc_geom-section-shape_rects"></a>

### fn `shape_rects`

```rust
pub fn shape_rects(shape: SectionShape, width: f64, depth: f64) -> Vec<(DVec2, DVec2)>
```

The solid parts of a profile in its `width` x `depth` box centered at the
origin, as `(min, max)` rectangles that do not overlap.

<a id="cc_geom-section-shape_loops"></a>

### fn `shape_loops`

```rust
pub fn shape_loops(shape: SectionShape, width: f64, depth: f64) -> Vec<Vec<DVec2>>
```

The outlines of a profile (outer boundary, then holes), counter-clockwise.

<a id="cc_geom-section-detailed"></a>

### fn `detailed`

```rust
pub fn detailed(project: &Project, data: &TypeData, detail: DetailLevel) -> Option<SectionOutline>
```

The section of a column or beam type at a detail level: Fine the outline
with radii, Medium without (arcs replaced by their chords' corners:
rebuild the profile with r/ri/ro = 0), Coarse its bounding rectangle.

<a id="cc_geom-section-detailed_section"></a>

### fn `detailed_section`

```rust
pub fn detailed_section(project: &Project, data: &TypeData, detail: DetailLevel) -> Option<TypeSection>
```

`detailed` with whether the outline has arcs.

<a id="cc_geom-section-column_body"></a>

### fn `column_body`

```rust
pub fn column_body(section: &SectionOutline, position: DVec2, rotation_deg: f64) -> (Vec<DVec2>, Vec<Vec<DVec2>>)
```

Solid parts of a column (outline + holes) placed in plan.

<a id="cc_geom-section-smooth_sides"></a>

### fn `smooth_sides`

```rust
pub fn smooth_sides(ring: &[DVec2]) -> Vec<bool>
```

Whether each side of a closed ring (from point `i` to `i + 1`) is a facet
of an arc: the ring turns by less than `SMOOTH_TURN_DEG` at both ends,
and the side is at most `SMOOTH_LENGTH_RATIO` times as long as the
shorter of its neighbors (so a long flat side between two fillets stays
flat).

<a id="cc_geom-section-add_column_body"></a>

### fn `add_column_body`

```rust
pub fn add_column_body(s: &mut Solid, outer: &[DVec2], holes: &[Vec<DVec2>], base: f64, top: f64, curved: bool)
```

Add a column body from `base` to `top` to `s`; the sides of a `curved`
section that are facets of its arcs are smooth.

<a id="mod-cc_geom-sizing"></a>

## `cc_geom::sizing`

<sub>`crates/cc_geom/src/sizing.rs`</sub>

Sizing of building services from their networks, with simplified tables
of common Brazilian practice (from public sources):

- **Sanitary drainage**: each run carries the fixture units of the
  fixtures upstream (towards the network's highest ends; the outlet is
  its lowest node); the diameter follows the usual table of branches
  (40 mm up to 3 units, 50 up to 6, 75 up to 20, 100 up to 160, 150 up
  to 620), at least 100 mm downstream of a toilet.
- **Cold and hot water**: the flow of a run is 0.3 √ΣP l/s from the
  relative weights of the fixtures it feeds (the source is the network's
  highest node); the diameter is the smallest PVC pipe keeping the speed
  at most 3 m/s.
- **Circuits**: the design current is the load over the voltage; the
  breaker is the next standard rating; the copper conductor is the
  smallest whose capacity (two loaded conductors in conduit, PVC 70 °C)
  covers the breaker, at least 1.5 mm² for lighting and 2.5 mm² for
  power, and larger until the voltage drop to the farthest device
  (along the conduits, else straight) is at most 4 %.

**Items:** [`MAX_SPEED`](#cc_geom-sizing-max_speed) · [`MAX_DROP`](#cc_geom-sizing-max_drop) · [`PipeSizing`](#cc_geom-sizing-pipesizing) · [`CircuitSizing`](#cc_geom-sizing-circuitsizing) · [`drain_diameter`](#cc_geom-sizing-drain_diameter) · [`water_flow`](#cc_geom-sizing-water_flow) · [`water_diameter`](#cc_geom-sizing-water_diameter) · [`size_pipes`](#cc_geom-sizing-size_pipes) · [`size_circuits`](#cc_geom-sizing-size_circuits)

<a id="cc_geom-sizing-max_speed"></a>

### const `MAX_SPEED`

```rust
pub const MAX_SPEED: f64
```

Maximum speed in water pipes, m/s.

<a id="cc_geom-sizing-max_drop"></a>

### const `MAX_DROP`

```rust
pub const MAX_DROP: f64
```

Maximum voltage drop of a circuit, percent.

<a id="cc_geom-sizing-pipesizing"></a>

### struct `PipeSizing`

```rust
pub struct PipeSizing
```

The sizing of one pipe run.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `run: ElementId`
- `system: SystemId`
- `kind: SystemKind`
- `load: f64`

  Fixture units (drains) or the sum of weights (water) it carries.

- `flow: Option<f64>`

  Water flow, l/s.

- `speed: Option<f64>`

  Water speed in the current pipe, m/s.

- `diameter: f64`

  Current and required outside diameters, m.

- `required: f64`

**Methods**

- `pub fn ok(&self) -> bool`

<a id="cc_geom-sizing-circuitsizing"></a>

### struct `CircuitSizing`

```rust
pub struct CircuitSizing
```

The sizing of one circuit.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `system: SystemId`
- `power: f64`

  Installed load, VA.

- `voltage: f64`
- `current: f64`

  Design current, A.

- `breaker: f64`

  Breaker rating, A.

- `conductor: f64`

  Conductor section, mm².

- `length: Option<f64>`

  Length to the farthest device, m (none without a board).

- `drop: Option<f64>`

  Voltage drop there, percent.


**Methods**

- `pub fn ok(&self) -> bool`

<a id="cc_geom-sizing-drain_diameter"></a>

### fn `drain_diameter`

```rust
pub fn drain_diameter(units: f64, toilet: bool) -> f64
```

The required drain diameter (m) for `units` fixture units.

<a id="cc_geom-sizing-water_flow"></a>

### fn `water_flow`

```rust
pub fn water_flow(weights: f64) -> f64
```

Flow (l/s) from the sum of relative weights.

<a id="cc_geom-sizing-water_diameter"></a>

### fn `water_diameter`

```rust
pub fn water_diameter(flow: f64) -> f64
```

The smallest PVC water pipe (outside diameter, m) for `flow` l/s.

<a id="cc_geom-sizing-size_pipes"></a>

### fn `size_pipes`

```rust
pub fn size_pipes(project: &Project, geom: &ModelGeometry) -> Vec<PipeSizing>
```

Sizes the drains and water pipes of every plumbing system.

<a id="cc_geom-sizing-size_circuits"></a>

### fn `size_circuits`

```rust
pub fn size_circuits(project: &Project, geom: &ModelGeometry) -> Vec<CircuitSizing>
```

Sizes every electrical circuit.

<a id="mod-cc_geom-skeleton"></a>

## `cc_geom::skeleton`

<sub>`crates/cc_geom/src/skeleton.rs`</sub>

Weighted straight skeleton of a polygon with holes, for roofs.
Every edge of the footprint is the eave of a roof plane rising inwards;
its weight is how far the plane's line moves inwards per unit of height
(`1 / slope`), and 0 for a gable (a vertical plane that stays put). The
skeleton follows the textbook wavefront simulation: the edges move in
at their speeds, each vertex along the line where its two planes meet,
and the topology changes at two kinds of events:

* an *edge event*, when an edge shrinks to nothing and its neighbors meet;
* a *split event*, when a reflex vertex runs into another edge, which
  splits a wavefront loop in two (or joins a hole to the loop around it).
Time is height above the eaves. Between events the wavefront moves
linearly, so each edge sweeps quads of its own roof plane; these are
merged into one polygon per plane. Vertex paths are the hips and
valleys, and loops that collapse leave the ridges. The simulation looks
for the next event among all vertices and edges at each step
(quadratic per event, cubic overall: footprints have tens of edges),
merges points closer than a tolerance relative to the footprint size,
and gives up with an error after a bounded number of events.

**Items:** [`SkeletonError`](#cc_geom-skeleton-skeletonerror) · [`Plane`](#cc_geom-skeleton-plane) · [`Skeleton`](#cc_geom-skeleton-skeleton) · [`skeleton`](#cc_geom-skeleton-skeleton)

<a id="cc_geom-skeleton-skeletonerror"></a>

### enum `SkeletonError`

```rust
pub enum SkeletonError
```

Why a skeleton could not be computed.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Variants**

- `Invalid`

  Fewer than three distinct points, or numbers that are not finite.

- `Budget`

  The simulation did not finish within its event budget.


<a id="cc_geom-skeleton-plane"></a>

### struct `Plane`

```rust
pub struct Plane
```

One roof plane: its eave line, inward normal and speed.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `origin: DVec2`

  A point of the eave and its direction (unit), interior on the left.

- `dir: DVec2`
- `length: f64`

  Length of the eave.

- `weight: f64`

  Inward horizontal distance per unit of height (0: a vertical gable).


**Methods**

- `pub fn height(&self, p: DVec2) -> f64`

  Height above the eaves of the plane at `p` (meaningless for gables).


<a id="cc_geom-skeleton-skeleton"></a>

### struct `Skeleton`

```rust
pub struct Skeleton
```

The skeleton of a footprint.

**Traits:** `Debug`, `Clone`, `Default`, `PartialEq`

**Fields**

- `planes: Vec<Plane>`

  The planes, one per input edge (loops in order, edge `i` from point `i` to `i + 1`).

- `faces: Vec<(usize, Vec<DVec2>)>`

  The part of each sloped plane over the footprint: `(plane, polygon)`,
  counter-clockwise in plan.

- `lines: Vec<(DVec2, DVec2)>`

  Ridges, hips and valleys in plan.

- `gables: Vec<(DVec2, f64, DVec2, f64, DVec2)>`

  Where sloped planes end at a gable: segments along gable eaves with
  their heights, and the horizontal direction away from the sloped
  plane (the outside of the roof there).

- `flats: Vec<(f64, Vec<DVec2>)>`

  Horizontal tops where the planes never close (all edges around them gables): `(height, polygon)`.


**Methods**

- `pub fn face_height(&self, plane: usize, p: DVec2) -> f64`

  Height above the eaves at a point of face `i`.


<a id="cc_geom-skeleton-skeleton"></a>

### fn `skeleton`

```rust
pub fn skeleton(loops: &[Vec<DVec2>], weights: &[Vec<f64>]) -> Result<Skeleton, SkeletonError>
```

The weighted straight skeleton of `loops` (the outer boundary
counter-clockwise, holes clockwise: the interior on the left of every
edge), with `weights[l][i]` the inward speed of edge `i` of loop `l`
per unit of height (0 for a gable).

<a id="mod-cc_geom-slab_rebar"></a>

## `cc_geom::slab_rebar`

<sub>`crates/cc_geom/src/slab_rebar.rs`</sub>

Slab reinforcement: the bars of floors and foundation slabs. Straight
bars of a mesh, ring and radial bars, and trimming bars around openings
lie in the slab's top or bottom layer, `cover` inside its faces, edge and
openings. Each bar is clipped to the region the cover leaves, so every
bar has its own length; where the boundary was drawn with arcs or
circles the bars end on the true curve, not on its chords.

**Items:** [`Slab`](#cc_geom-slab_rebar-slab) · [`Region`](#cc_geom-slab_rebar-region) · [`Stretch`](#cc_geom-slab_rebar-stretch) · [`slab_bars`](#cc_geom-slab_rebar-slab_bars)

<a id="cc_geom-slab_rebar-slab"></a>

### struct `Slab`

```rust
pub(crate) struct Slab
```

A slab as its bars see it.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `outline: Vec<DVec2>`

  Boundary, counter-clockwise.

- `holes: Vec<Vec<DVec2>>`
- `circles: Vec<(DVec2, f64)>`

  The circles its boundary and openings were drawn on (center, radius).

- `z: (f64, f64)`

  Bottom and top of its structure.


**Methods**

- `pub fn of(project: &Project, e: &Element) -> Option<Slab>`

  The slab of a floor or a foundation slab; `None` for other elements.


<a id="cc_geom-slab_rebar-region"></a>

### struct `Region`

```rust
pub(crate) struct Region
```

Where bars may lie: inside the boundary and outside the openings, the
cover clear of both.

*Some fields are private.*

**Methods**

- `pub fn new(slab: &Slab, cover: f64) -> Region`

  The region `cover` inside `slab`. Empty when the cover leaves nothing.

- `pub fn is_empty(&self) -> bool`
- `pub fn stretches(&self, origin: DVec2, dir: DVec2) -> Vec<Stretch>`

  The stretches of the line through `origin` along the unit `dir` that lie inside the region.

- `pub fn contains(&self, p: DVec2) -> bool`

  Whether `p` lies inside the region.

- `pub fn arcs(&self, center: DVec2, radius: f64) -> Vec<(f64, f64)>`

  The parts of the circle around `center` inside the region, as
  `(start angle, sweep)`; a whole turn is one part with a sweep of τ.


<a id="cc_geom-slab_rebar-stretch"></a>

### struct `Stretch`

```rust
pub(crate) struct Stretch
```

A stretch of a line inside the region, and whether each end lies on the slab's own edge.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `from: f64`
- `to: f64`
- `edge: (bool, bool)`

<a id="cc_geom-slab_rebar-slab_bars"></a>

### fn `slab_bars`

```rust
pub(crate) fn slab_bars(slab: &Slab, r: &Rebar, d: f64, other: f64) -> Vec<Bar>
```

The bars of the set `r` (bars `d` thick) in `slab`; `other` is the
largest bar of the other sets of its layer, which an `inner` set lies inside.

<a id="mod-cc_geom-solid"></a>

## `cc_geom::solid`

<sub>`crates/cc_geom/src/solid.rs`</sub>

**Items:** [`Face`](#cc_geom-solid-face) · [`SMOOTH_COS`](#cc_geom-solid-smooth_cos) · [`Solid`](#cc_geom-solid-solid) · [`plane_axes`](#cc_geom-solid-plane_axes) · [`Look`](#cc_geom-solid-look) · [`Ray`](#cc_geom-solid-ray) · [`curved_facets`](#cc_geom-solid-curved_facets) · [`ray_box`](#cc_geom-solid-ray_box)

<a id="cc_geom-solid-face"></a>

### struct `Face`

```rust
pub struct Face
```

A planar polygon face with an outward normal.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `verts: Vec<DVec3>`
- `normal: DVec3`
- `smooth: bool`

  A facet of a curved surface: edges shared with neighboring smooth
  facets are not outlines (only silhouettes, see `Solid::smooth_edges`).


**Methods**

- `pub fn new(verts: Vec<DVec3>, normal: DVec3) -> Self`
- `pub fn of_polygon(verts: Vec<DVec3>) -> Option<Self>`

  A polygon with the normal of its winding (Newell's method); `None`
  when it has fewer than three points or no area.

- `pub(crate) fn basis(&self) -> (DVec3, DVec3)`

  Orthonormal in-plane basis `(u, v)`.

- `pub fn triangulate(&self) -> Vec<[usize; 3]>`

  Triangles as index triples into `verts`.

- `pub fn centroid(&self) -> DVec3`
- `pub fn area(&self) -> f64`

  Area of the planar polygon.


<a id="cc_geom-solid-smooth_cos"></a>

### const `SMOOTH_COS`

```rust
pub(crate) const SMOOTH_COS: f64
```

Smooth facets meeting at less than this angle (cosine) read as one surface.

<a id="cc_geom-solid-solid"></a>

### struct `Solid`

```rust
pub struct Solid
```

Boundary representation of one element (or a part of one).

**Traits:** `Debug`, `Clone`

**Fields**

- `element: ElementId`
- `category: Category`
- `color: [u8; 3]`
- `transparency: f64`

  0 (opaque) to 1 (invisible), from the material.

- `faces: Vec<Face>`
- `secondary: bool`

  A part the element carries that is not its body (a roof's framing):
  drawn, but left out of its volume, joins and exported body.

- `detail: cc_core::component::DetailLevels`

  The detail levels of the views that draw it (a component's piece).

- `appearance: [u8; 3]`

  The material's appearance color and surface pattern (the textures style).

- `surface: cc_core::PatternFill`
- `material: Option<String>`

  The material the solid is made of (its texture in the textures style).

- `painted: std::collections::BTreeMap<usize, Look>`

  Faces painted with another material, by face index (see `crate::paint`).


**Methods**

- `pub fn add_part(&mut self, part: &ComponentPart)` *(in `cc_geom::component`)*

  Append the faces of a component piece (a prism's built from its
  footprint), the facets of its curved surfaces smooth: as the piece
  says (see `resolve_instance`), else found here.

- `pub fn new(element: ElementId, category: Category, color: [u8; 3]) -> Self`
- `pub fn of_material(element: ElementId, category: Category, project: &cc_core::Project, name: &str) -> Self`

  An empty solid colored and made transparent like material `name`.

- `pub fn set_appearance(&mut self, project: &cc_core::Project, name: &str)`

  Take the appearance color and surface pattern of material `name`
  (what the textures style shows).

- `pub fn look(&self) -> Look`

  How the solid's own material shows (its faces that are not painted).

- `pub fn paint_of(&self, face: usize) -> Option<&Look>`

  The paint of face `face`, if it is painted.

- `pub fn raycast_face(&self, ray: Ray) -> Option<(f64, usize)>`

  The nearest face along `ray`: its distance and index.

- `pub fn bounds(&self) -> Option<(DVec3, DVec3)>`
- `pub fn volume(&self) -> f64`

  Volume via the divergence theorem (faces must be closed and outward).

- `pub fn feature_edges(&self) -> Vec<Vec<(DVec3, DVec3)>>`

  Visible outline edges grouped by face: the parts of each face boundary
  not shared with a coplanar neighbor face (so split faces read as one).

- `pub fn vertex_normals(&self) -> Vec<Vec<DVec3>>`

  The shading normal at each vertex of each face: a flat face's own
  normal, and on a smooth facet the average of the normals of the
  smooth facets meeting at that point (those that read as one surface
  with it), each weighted by its angle there, so a curved surface
  shades without steps however its facets were split.

- `pub fn facet_shifts(&self) -> Vec<[f64; 2]>`

  What to add to the plane coordinates of each face (see `plane_axes`)
  so they run on from one facet of a curved surface to the next: every
  facet has its own axes, so without it patterns and textures would
  start again at each one. Facets are reached from the first of their
  surface across shared edges and agree with the facet they were reached
  from at the middle of that edge (exactly along it when it runs up the
  surface, as on an arc wall); a surface that closes on itself keeps one
  seam.

- `pub fn smooth_neighbors(&self) -> Vec<Vec<(usize, DVec3)>>`

  Per face, the smooth facets that read as one surface with it across a
  shared edge, each with that edge's midpoint (none for a flat face).

- `pub fn smooth_edges(&self) -> Vec<Vec<(DVec3, DVec3, DVec3)>>`

  Edges between smooth facets (hidden by `Self::feature_edges`), per
  face, with the neighbor's normal: a view draws one as a silhouette
  where the face is seen and its neighbor is not.

- `pub fn raycast(&self, ray: Ray) -> Option<f64>`

  Nearest hit distance along `ray`, if any.

- `pub fn raycast_within(&self, ray: Ray, limit: f64) -> Option<f64>`

  Like `Self::raycast`, for a caller that already has a hit at
  `limit`: faces whose boxes the ray meets beyond it are skipped (a
  result beyond `limit` is not then the nearest).

- `pub fn add_polygons(&mut self, polygons: impl IntoIterator<Item = Vec<DVec3>>)`

  Append planar polygons, each with the normal of its winding (Newell's
  method); degenerate polygons are dropped.

- `pub fn smooth_curved(&mut self, from: usize)`

  Mark smooth the facets of curved surfaces among the faces from index
  `from` on (see `curved_facets`), for solids whose builder does not
  know which they are (component pieces, imported meshes).

- `pub fn add_box(&mut self, origin: DVec2, u: DVec2, v: DVec2, u0: f64, u1: f64, v0: f64, v1: f64, z0: f64, z1: f64)`

  Append an axis-aligned (in the local frame) box.
  `origin` is the frame origin, `u`/`v` horizontal unit axes, Z is up.

- `pub fn add_prism_with_holes(&mut self, footprint: &[DVec2], holes: &[Vec<DVec2>], bottom: impl Fn(DVec2) -> f64, top: impl Fn(DVec2) -> f64)`

  Append a prism like `Self::add_prism` with openings: `holes` are
  outlines inside `footprint` (either winding).
  Faces are pushed caps first (top and bottom triangles), then one side
  per edge, last: the edges of `polygon::ccw(footprint)`, then those of
  each hole of three points or more, in the given order, as
  `polygon::ccw(hole)` reversed (clockwise). Without holes it is
  `Self::add_prism`. `section::add_column_body` relies on this order.

- `pub fn add_prism(&mut self, footprint: &[DVec2], bottom: impl Fn(DVec2) -> f64, top: impl Fn(DVec2) -> f64)`

  Append a prism between two copies of a polygon: `bottom(p)`/`top(p)` give
  the z of each footprint vertex (planar surfaces only).
  Faces are pushed top, bottom, then one side per edge, last, in the
  order of the footprint wound counter-clockwise (`polygon::ccw`).


<a id="cc_geom-solid-plane_axes"></a>

### fn `plane_axes`

```rust
pub fn plane_axes(n: DVec3) -> (DVec3, DVec3)
```

The axes a texture is laid along on a face with normal `n`: plan X and Y
on faces turned up or down, else level across the face (to the right,
seen from outside) and up its slope. A point's plane coordinates are its
dot products with them, in meters (see `Solid::facet_shifts`).

<a id="cc_geom-solid-look"></a>

### struct `Look`

```rust
pub struct Look
```

How a material shows on a face: shading color, appearance color, surface
pattern and transparency, and the material's name (for its texture).

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `material: Option<String>`
- `color: [u8; 3]`
- `appearance: [u8; 3]`
- `surface: cc_core::PatternFill`
- `transparency: f64`

**Methods**

- `pub fn of(project: &cc_core::Project, name: &str) -> Option<Self>`

  The look of material `name`; `None` when the project has no such material.


<a id="cc_geom-solid-ray"></a>

### struct `Ray`

```rust
pub struct Ray
```

**Traits:** `Debug`, `Clone`, `Copy`

**Fields**

- `origin: DVec3`
- `dir: DVec3`

<a id="cc_geom-solid-curved_facets"></a>

### fn `curved_facets`

```rust
pub fn curved_facets(polygons: &[Vec<DVec3>]) -> Vec<bool>
```

Which of `polygons` (the faces of one closed piece, such as a component
piece) are facets of curved surfaces (see `Solid::smooth_curved`), one
flag per polygon; degenerate polygons are not.

<a id="cc_geom-solid-ray_box"></a>

### fn `ray_box`

```rust
pub(crate) fn ray_box(ray: Ray, lo: DVec3, hi: DVec3) -> Option<f64>
```

Distance along `ray` at which it enters the box `lo`-`hi` (grown by a
small margin; 0 from inside), if it meets the box ahead.

<a id="mod-cc_geom-stair"></a>

## `cc_geom::stair`

<sub>`crates/cc_geom/src/stair.rs`</sub>

Stair geometry: the treads, landings and winders of a stair element laid
out from its shape, the solids of its construction (solid, cast in place,
assembled) and the paths its railings follow.
Shapes are built in a local frame `(a, b)`: `a` along the first flight
from the foot of the first riser, `b` across it towards the side the stair
turns to; a stair turning right is the mirror image.

**Items:** [`StepKind`](#cc_geom-stair-stepkind) · [`StairStep`](#cc_geom-stair-stairstep) · [`Flight`](#cc_geom-stair-flight) · [`StairLanding`](#cc_geom-stair-stairlanding) · [`LandingRect`](#cc_geom-stair-landingrect) · [`Spiral`](#cc_geom-stair-spiral) · [`StairGeom`](#cc_geom-stair-stairgeom) · [`Pieces`](#cc_geom-stair-pieces) · [`add_wedge`](#cc_geom-stair-add_wedge) · [`stair_geom`](#cc_geom-stair-stair_geom) · [`layout_stair`](#cc_geom-stair-layout_stair) · [`railing_path`](#cc_geom-stair-railing_path) · [`railing_length`](#cc_geom-stair-railing_length) · [`add_railing`](#cc_geom-stair-add_railing) · [`railing_pieces`](#cc_geom-stair-railing_pieces)

<a id="cc_geom-stair-stepkind"></a>

### enum `StepKind`

```rust
pub enum StepKind
```

What a stair step is.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Variants**

- `Tread`
- `Winder`

  A tapered tread turning a corner (winders, spiral treads).

- `Landing`

<a id="cc_geom-stair-stairstep"></a>

### struct `StairStep`

```rust
pub struct StairStep
```

One walking surface of a stair: a tread, winder or landing.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `outline: Vec<DVec2>`

  Plan outline (world).

- `along: Vec<f64>`

  Position of each outline vertex from the front of the step (0) to its
  back (1), along the way up: the pitch of soffits and stringers.

- `front: [DVec2; 2]`

  Front edge (the riser line), in plan.

- `z: f64`

  Elevation of the walking surface.

- `kind: StepKind`

<a id="cc_geom-stair-flight"></a>

### struct `Flight`

```rust
pub struct Flight
```

A straight flight: its treads go up along `dir` from `origin` (the
middle of its first riser).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `origin: DVec2`
- `dir: DVec2`
- `left: DVec2`

  Unit vector to the left of `dir`.

- `treads: usize`
- `z0: f64`

  Elevation of the first tread and of the floor under its riser.

- `floor: f64`
- `landing_ahead: f64`

  Depth of the landing its last riser rises to (0: it ends on the floor above).


<a id="cc_geom-stair-stairlanding"></a>

### struct `StairLanding`

```rust
pub struct StairLanding
```

The frame of a landing: its center, the unit vectors along the flight
arriving at it and across it (to the side the stair turns to), its half
sizes along them and the center lines of the flights it joins. Landing
coordinates `(x, y)` are along and across from the center.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `center: DVec2`
- `along: DVec2`
- `across: DVec2`
- `half: DVec2`

  Half its depth along and of its width across.

- `z: f64`

  Elevation of its walking surface.

- `arrive: f64`

  Offset across of the arriving flight's center line.

- `leave: DVec2`

  Direction of the leaving flight in landing coordinates, and the
  offset of its center line (across when it goes along, else along).

- `leave_at: f64`

**Methods**

- `pub fn point(&self, p: DVec2) -> DVec2`

  World point of landing coordinates `p`.

- `pub fn outline(&self, [x0, x1, y0, y1]: LandingRect) -> Vec<DVec2>`

  World plan outline of a rectangle in landing coordinates.

- `pub fn beams(&self, stringers: Stringers, beams: LandingBeams, width: f64) -> Vec<LandingRect>`

  Plan rectangles (landing coordinates) of the beams under it, `width`
  wide: a frame round the edge for side stringers, the stringers'
  lines carried across it (as `beams` says) for a central one. Beams
  that cross are split at the one carried through, so none overlap.

- `pub fn post_spots(&self, stringers: Stringers, beams: LandingBeams) -> Vec<(PostAt, DVec2)>`

  Where posts can stand under it, on its edge (landing coordinates):
  the corners of a frame, the ends of a central stringer's beams.

- `pub fn post_outline(&self, spot: DVec2, width: f64) -> Vec<DVec2>`

  Plan outline of a square post `width` wide at `spot` on its edge,
  pushed in so it stands under the landing.


<a id="cc_geom-stair-landingrect"></a>

### type alias `LandingRect`

```rust
pub type LandingRect = [f64; 4]
```

A rectangle `[x0, x1, y0, y1]` in landing coordinates.

<a id="cc_geom-stair-spiral"></a>

### struct `Spiral`

```rust
pub struct Spiral
```

A spiral stair's circle.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `center: DVec2`
- `inner: f64`
- `outer: f64`

<a id="cc_geom-stair-stairgeom"></a>

### struct `StairGeom`

```rust
pub struct StairGeom
```

Layout of a stair element.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `start: DVec2`
- `u: DVec2`

  Direction of the first flight and the perpendicular to its left.

- `v: DVec2`
- `width: f64`
- `risers: usize`
- `riser: f64`
- `tread: f64`

  Tread depth (on the walking line).

- `base: f64`
- `top: f64`
- `layout: StairLayout`
- `steps: Vec<StairStep>`
- `flights: Vec<Flight>`
- `landings: Vec<StairLanding>`

  The landings, up the stair.

- `spiral: Option<Spiral>`
- `walkline: Vec<DVec3>`

  Walking line from the foot of the stair to its top, with elevations.

- `parts: Vec<Vec<DVec2>>`

  Plan outlines of the flights and landings (for snapping).

- `check: StairCheck`

*Some fields are private.*

**Methods**

- `pub fn run(&self) -> f64`

  Plan length of the walking line.

- `pub fn number_spots(&self) -> Vec<(DVec2, i32)>`

  Where the step numbers go in plan, with each number: one in every
  tread and winder up the stair (landings are not numbered) from
  `layout.number_start`, then one on the floor the stair arrives at, a
  tread beyond its last riser. Each sits a quarter of the step's width
  to the left of its middle, walking up (to the right, or in the middle,
  when that falls outside the step), clear of the walking line's arrow.

- `pub fn railing_path(&self, side: StairSide, inset: f64) -> Vec<DVec3>`

  Path of a railing along `side`, `inset` in from the edge, at the
  height of the nosings (from the first nosing to the top).

- `pub fn add_to(&self, build: &StairBuild, solid: &mut Solid)`

  Add the stair's solids, built as `build` says.

- `pub fn pieces(&self, build: &StairBuild) -> Pieces`

  The stair's solids as closed pieces (treads, landings, stringers...), built as `build` says.

- `pub(crate) fn stringer_profiles(&self, f: &Flight, lift: f64, depth: f64, landing_depth: f64) -> Vec<Vec<(f64, f64)>>`

  Side profiles, in `(s, z)` along flight `f`, of a stringer whose top
  runs `lift` above the nosings, `depth` deep measured plumb: a band
  parallel to the pitch, cut level where it stands (the floor, or the
  underside of the landing the flight leaves, `landing_depth` below
  its top) and, where the flight rises to a landing, carried on under
  it up to the landing's underside, so it bears on the landing instead
  of touching its edge.


<a id="cc_geom-stair-pieces"></a>

### type alias `Pieces`

```rust
pub type Pieces = Vec<Vec<Vec<DVec3>>>
```

Closed solids, each a list of planar polygons wound outward.

<a id="cc_geom-stair-add_wedge"></a>

### fn `add_wedge`

```rust
pub(crate) fn add_wedge(s: &mut Pieces, outline: &[DVec2], bottom: &[f64], top: &[f64])
```

A prism over `outline` with an elevation per vertex at the bottom and the
top; uneven faces are split into triangles.

<a id="cc_geom-stair-stair_geom"></a>

### fn `stair_geom`

```rust
pub fn stair_geom(project: &Project, element: &cc_core::Element, stair: &cc_core::Stair) -> Option<StairGeom>
```

Layout of a stair element from its type, constraints and shape.

<a id="cc_geom-stair-layout_stair"></a>

### fn `layout_stair`

```rust
pub fn layout_stair(start: DVec2, u: DVec2, width: f64, base: f64, top: f64, max_riser: f64, tread: f64, rules: &StairRules, mut layout: StairLayout) -> StairGeom
```

Lay out a stair from its foot `start`, first-flight direction `u`, width
and elevations.

<a id="cc_geom-stair-railing_path"></a>

### fn `railing_path`

```rust
pub fn railing_path(project: &Project, element: &cc_core::Element, railing: &cc_core::Railing) -> Vec<DVec3>
```

Path of a railing with elevations: along its stair's or ramp's side when it has
one, else its own path at its base.

<a id="cc_geom-stair-railing_length"></a>

### fn `railing_length`

```rust
pub fn railing_length(project: &Project, element: &cc_core::Element, railing: &cc_core::Railing) -> f64
```

Length of a railing along its (sloped) path.

<a id="cc_geom-stair-add_railing"></a>

### fn `add_railing`

```rust
pub fn add_railing(project: &Project, element: &cc_core::Element, railing: &cc_core::Railing, solid: &mut Solid)
```

Add a railing's solids: a top rail its type's height above the path and
posts at the path's corners and every `post_spacing` along it.

<a id="cc_geom-stair-railing_pieces"></a>

### fn `railing_pieces`

```rust
pub fn railing_pieces(project: &Project, element: &cc_core::Element, railing: &cc_core::Railing) -> Pieces
```

The rail and posts of a railing as closed pieces.

<a id="mod-cc_geom-strip"></a>

## `cc_geom::strip`

<sub>`crates/cc_geom/src/strip.rs`</sub>

Arc strip footings: the body of a strip (a ground beam) that follows an
arc, a band between concentric arcs either side of its centerline, its
ends cut where it meets other strips. Plans draw it as short chords of
the arcs; its area and volume are exact.

**Items:** [`StripArc`](#cc_geom-strip-striparc)

<a id="cc_geom-strip-striparc"></a>

### struct `StripArc`

```rust
pub struct StripArc
```

The plan of an arc strip: `half` either side of the centerline `arc`,
in strip-local `(u, v)` (along the arc from its start, and to its left);
the left face spans `left` and the right face `right` along the arc.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `arc: Arc`
- `half: f64`
- `left: (f64, f64)`
- `right: (f64, f64)`

**Methods**

- `pub fn world(&self, u: f64, v: f64) -> DVec2`

  World plan position of strip-local `(u, v)`.

- `pub fn outline(&self) -> Vec<DVec2>`

  Plan outline, counter-clockwise: the arcs as short chords.

- `pub fn area(&self) -> f64`

  Exact plan area (the arcs as arcs).

- `pub(crate) fn add_to(&self, s: &mut Solid, z0: f64, z1: f64)`

  Add the body between `z0` and `z1` to `s`: the curved faces as smooth facets.


<a id="mod-cc_geom-strip_rebar"></a>

## `cc_geom::strip_rebar`

<sub>`crates/cc_geom/src/strip_rebar.rs`</sub>

Strip footing reinforcement: bars across the footing slab (radial in an
arc strip), bars along it (ring bars), and the bars of a beam for the
beam on top of a haunched strip (or the whole section of a plain one).
Top bars follow the top of the footing: up its slopes and flat between
the faces of its beam.

**Items:** [`StripSection`](#cc_geom-strip_rebar-stripsection) · [`strip_bars`](#cc_geom-strip_rebar-strip_bars)

<a id="cc_geom-strip_rebar-stripsection"></a>

### struct `StripSection`

```rust
pub(crate) struct StripSection
```

A strip's section as its bars see it: its width, the footing's thickness
at the beam, and the beam on top, when it has one.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `width: f64`
- `thickness: f64`
- `haunch: Option<StripHaunch>`

**Methods**

- `pub fn top(&self, v: f64) -> f64`

  Height of the top of the footing slab above its bottom, `v` across it.

- `pub fn beam_half(&self) -> f64`

  Half the width of the beam part: the beam on top, or the whole strip.


<a id="cc_geom-strip_rebar-strip_bars"></a>

### fn `strip_bars`

```rust
pub(crate) fn strip_bars(g: &BeamGeom, section: &StripSection, r: &Rebar, d: f64, other: f64, past: (f64, f64)) -> Vec<Bar>
```

The bars of the strip's own set `r` (bars `d` thick): `other` is the
largest bar of the sets of the same layer that run the other way, which
the bars along the strip lie inside. The bars along the strip run `past`
its start and end by those lengths instead of stopping the cover short.

<a id="mod-cc_geom-topography"></a>

## `cc_geom::topography`

<sub>`crates/cc_geom/src/topography.rs`</sub>

Terrain surfaces (`cc_core::topography`): a constrained Delaunay
triangulation (`spade`) of the measured points inside the boundary, with
the boundary's edges as constraints, so non-convex boundaries keep no
triangle outside. Boundary vertices take the elevation all the points
give them, those beyond the boundary too (linear over the points'
triangulation, the nearest point's beyond it). The surface closes down to a flat base into a solid; contours cut it
at multiples of an interval. The terrain's building pads then grade the
surface (`crate::pad`), in id order.

**Items:** [`MAX_CONTOURS`](#cc_geom-topography-max_contours) · [`TopoWarning`](#cc_geom-topography-topowarning) · [`TopoGeom`](#cc_geom-topography-topogeom) · [`Contour`](#cc_geom-topography-contour) · [`topo_geom`](#cc_geom-topography-topo_geom) · [`topo_geom_with_pads`](#cc_geom-topography-topo_geom_with_pads) · [`add_solid`](#cc_geom-topography-add_solid) · [`contours`](#cc_geom-topography-contours) · [`contours_of`](#cc_geom-topography-contours_of) · [`slice`](#cc_geom-topography-slice) · [`chain`](#cc_geom-topography-chain)

<a id="cc_geom-topography-max_contours"></a>

### const `MAX_CONTOURS`

```rust
pub const MAX_CONTOURS: usize
```

Most contour lines drawn for one surface: the interval doubles until fewer are left.

<a id="cc_geom-topography-topowarning"></a>

### enum `TopoWarning`

```rust
pub enum TopoWarning
```

Something about a surface the user should know (shown with the element).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Variants**

- `FewPoints`

  Fewer than three points inside the boundary: the surface is flat or a plane through them.

- `Collinear`

  The points inside the boundary lie on one line.

- `BoundaryCrossing`

  Some boundary edges cross others and were left out of the surface.

- `Failed`

  The triangulation failed; the surface is flat at the level.


**Methods**

- `pub fn code(self) -> &'static str`

  A stable code (engine output and translation keys `topo-warning-<code>`).


<a id="cc_geom-topography-topogeom"></a>

### struct `TopoGeom`

```rust
pub struct TopoGeom
```

A built terrain surface, in absolute elevations.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `triangles: Vec<[DVec3; 3]>`

  Surface triangles (graded by the terrain's pads), counter-clockwise seen from above.

- `existing: Vec<[DVec3; 3]>`

  The existing ground where pads changed it.

- `boundary: Vec<DVec3>`

  The boundary, counter-clockwise, with each vertex's elevation.

- `min_z: f64`

  Lowest and highest points of the graded and existing surfaces.

- `max_z: f64`
- `base_z: f64`

  Elevation of the flat base.

- `area: f64`

  Area in plan.

- `volume: f64`

  Volume between the surface and the base.

- `warnings: Vec<TopoWarning>`

<a id="cc_geom-topography-contour"></a>

### struct `Contour`

```rust
pub struct Contour
```

A contour line: the surface cut at elevation `z`.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `z: f64`
- `major: bool`

  Every `major_every`-th contour (elevations that are multiples of the major interval).

- `points: Vec<DVec2>`

  Plan points; closed when the first and last coincide.


<a id="cc_geom-topography-topo_geom"></a>

### fn `topo_geom`

```rust
pub fn topo_geom(project: &Project, e: &Element, t: &Topography) -> Option<TopoGeom>
```

The surface of terrain `t` of element `e`, or `None` when its boundary is invalid.

<a id="cc_geom-topography-topo_geom_with_pads"></a>

### fn `topo_geom_with_pads`

```rust
pub fn topo_geom_with_pads(project: &Project, e: &Element, t: &Topography) -> Option<(TopoGeom, Vec<(ElementId, crate::pad::PadGeom)>)>
```

The surface of terrain `t` of element `e` graded by its pads, and each pad's figures.

<a id="cc_geom-topography-add_solid"></a>

### fn `add_solid`

```rust
pub fn add_solid(s: &mut Solid, g: &TopoGeom)
```

The closed solid: the surface on top, a flat base, sides along the boundary.

<a id="cc_geom-topography-contours"></a>

### fn `contours`

```rust
pub fn contours(g: &TopoGeom, interval: f64, major_every: u32, max: usize) -> Vec<Contour>
```

Contour lines of `g` every `interval` (doubled until at most `max`
remain), every `major_every`-th one a major.

<a id="cc_geom-topography-contours_of"></a>

### fn `contours_of`

```rust
pub fn contours_of(triangles: &[[DVec3; 3]], (min_z, max_z): (f64, f64), interval: f64, major_every: u32, max: usize) -> Vec<Contour>
```

Contour lines of `triangles` (elevations within `range`), as `contours`.

<a id="cc_geom-topography-slice"></a>

### fn `slice`

```rust
pub fn slice(triangles: &[[DVec3; 3]], z: f64) -> Vec<(DVec2, DVec2)>
```

Segments where the plane at `z` cuts `triangles` (vertices exactly at
`z` count as just above, so shared edges give no duplicates).

<a id="cc_geom-topography-chain"></a>

### fn `chain`

```rust
pub fn chain(segments: Vec<(DVec2, DVec2)>) -> Vec<Vec<DVec2>>
```

Join segments that share ends into polylines (bounded by the segment count).

<a id="mod-cc_geom-wall"></a>

## `cc_geom::wall`

<sub>`crates/cc_geom/src/wall.rs`</sub>

**Items:** [`Opening`](#cc_geom-wall-opening) · [`Recess`](#cc_geom-wall-recess) · [`RECESS_MIN_BACK`](#cc_geom-wall-recess_min_back) · [`Column`](#cc_geom-wall-column) · [`WallGeom`](#cc_geom-wall-wallgeom) · [`LayerGap`](#cc_geom-wall-layergap) · [`EndJoin`](#cc_geom-wall-endjoin) · [`WallLine`](#cc_geom-wall-wallline) · [`core_through`](#cc_geom-wall-core_through) · [`embedded_in`](#cc_geom-wall-embedded_in) · [`resolve_join`](#cc_geom-wall-resolve_join) · [`arc_band_area`](#cc_geom-wall-arc_band_area)

<a id="cc_geom-wall-opening"></a>

### struct `Opening`

```rust
pub struct Opening
```

A rectangular hole through a wall, in wall-local coordinates.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `element: ElementId`
- `u0: f64`
- `u1: f64`
- `z0: f64`
- `z1: f64`

<a id="cc_geom-wall-recess"></a>

### struct `Recess`

```rust
pub struct Recess
```

A recess in one face of a wall (a niche): the wall is open from that
face `depth` in, between `u0` and `u1` along it and `z0` and `z1` in
height, and whole behind it. Unlike an `Opening` it does not split the
wall into columns: the wall stands there for joins, rooms and bars.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `element: ElementId`
- `u0: f64`
- `u1: f64`
- `z0: f64`
- `z1: f64`
- `side: f64`

  The face it is in: +1 the left one, -1 the right one.

- `depth: f64`

**Methods**

- `pub fn across(&self, half: f64) -> (f64, f64)`

  Offsets across a wall `half` thick of the face it opens and of its back.


<a id="cc_geom-wall-recess_min_back"></a>

### const `RECESS_MIN_BACK`

```rust
pub const RECESS_MIN_BACK: f64
```

The least a recess leaves of its wall behind it, meters.

<a id="cc_geom-wall-column"></a>

### struct `Column`

```rust
pub struct Column
```

A vertical slice of a wall between opening boundaries.
Its footprint is a quad whose left (+v) edge spans `left` and right (-v)
edge spans `right` along the wall axis; end columns may be mitered.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `left: (f64, f64)`
- `right: (f64, f64)`
- `solid: Vec<(f64, f64)>`

  Solid z-intervals (absolute elevations), bottom to top.


**Methods**

- `pub fn is_solid_at(&self, z: f64) -> bool`
- `pub fn top(&self) -> Option<f64>`

<a id="cc_geom-wall-wallgeom"></a>

### struct `WallGeom`

```rust
pub struct WallGeom
```

Resolved wall geometry: frame, joins, openings and columns.
Wall-local coordinates are `u`, the distance along the body centerline
from its start, and `v`, the distance to its left. A straight wall's
frame is `origin`, `dir` and `normal`; an arc wall's centerline is `arc`
(`dir` and `normal` are then its frame at the start), and its faces and
layer boundaries are concentric arcs.

**Traits:** `Debug`, `Clone`

**Fields**

- `element: ElementId`
- `origin: DVec2`
- `dir: DVec2`
- `normal: DVec2`

  Left normal (+v).

- `length: f64`
- `half: f64`
- `base_z: f64`
- `top_z: f64`
- `start_ext: (f64, f64)`

  Axial extents `(left, right)` of the start/end faces after joins.

- `end_ext: (f64, f64)`
- `start_joined: bool`
- `end_joined: bool`
- `openings: Vec<Opening>`
- `recesses: Vec<Recess>`

  Recesses in its faces (niches), apart from each other and from the openings.

- `columns: Vec<Column>`
- `layers: Vec<(f64, f64, String)>`

  Layer boundaries as `(v_from, v_to, material)`, from the left face inward.

- `face_gaps: Vec<(f64, f64, f64)>`

  Stretches of a face where another wall meets it in a T, as
  `(side, u0, u1)` with side +1 for the left face and -1 for the right.

- `core: Option<(f64, f64)>`

  Offsets `(left, right)` of the core's faces, when the type has a
  structure core with other layers beside it.

- `core_ends: [Option<(f64, f64)>; 2]`

  At an end met in a T (`[start, end]`), where the core layers end
  instead: the `u` of the core's left and right faces, through the
  other wall's outer layers to its core.

- `layer_gaps: Vec<LayerGap>`

  Where the cores of walls met in a T pass through this wall's outer layers.

- `arc: Option<Arc>`

  The body centerline of an arc wall.

- `top_profile: Vec<(f64, f64, f64, f64)>`

  A top that follows what the wall is attached to: segments
  `(u0, u1, z0, z1)` along the centerline, the top running straight
  from `z0` to `z1` (empty: the flat `top_z`).

- `curtain: Option<crate::curtain::CurtainGeom>`

  The panels and mullions of a curtain wall (its body instead of layers).


**Methods**

- `pub(crate) fn new(element: ElementId, line: WallLine, base_z: f64, top_z: f64, start: Option<EndJoin>, end: Option<EndJoin>, layers: Vec<(f64, f64, String)>) -> Self`
- `pub fn profile_top(&self, mid: f64, u: f64) -> Option<f64>`

  The top of the profile segment that holds `mid`, carried on to `u`
  (the corners of a mitered end lie a little beyond); `None` with a
  flat top.

- `pub(crate) fn set_top_profile(&mut self, profile: Vec<(f64, f64, f64, f64)>)`

  Follow a top profile: the columns split where its segments meet, so
  each one's top is a single plane.

- `pub fn world(&self, u: f64, v: f64) -> DVec2`

  World plan position of wall-local `(u, v)`.

- `pub fn local(&self, p: DVec2) -> (f64, f64)`

  Wall-local `(u, v)` of a plan point (arc walls: along the arc, the
  distance from its start measured to the nearer side of its gap).

- `pub fn tangent(&self, u: f64) -> DVec2`

  Direction of the centerline at `u`.

- `pub fn normal_at(&self, u: f64) -> DVec2`

  Left normal of the centerline at `u`.

- `pub fn frame_at(&self, u: f64) -> (DVec2, DVec2, DVec2)`

  A straight frame `(origin, dir, normal)` matching the wall at `u`:
  `origin + dir * x + normal * y` is the wall point `(x, y)` for `x`
  near `u` (exact for straight walls; the tangent frame for arcs).

- `pub fn path(&self, u0: f64, u1: f64, v: f64) -> Vec<DVec2>`

  The line `v` to the left of the centerline between `u0` and `u1`,
  as a polyline (two points for straight walls, short chords of the arc otherwise).

- `pub fn inner_range(&self) -> (f64, f64)`

  Range of u where both faces exist (where openings may be placed).

- `pub fn face_segments(&self, side: f64, u0: f64, u1: f64) -> Vec<(f64, f64)>`

  The parts of the face on `side` (+1 left, -1 right) between `u0` and
  `u1` that no T-joined wall covers.

- `pub fn face_segments_at(&self, side: f64, u0: f64, u1: f64, z: f64) -> Vec<(f64, f64)>`

  `Self::face_segments` of the wall cut at height `z`: also open where
  a recess in that face is cut.

- `pub fn boundary_segments(&self, v: f64, u0: f64, u1: f64) -> Vec<(f64, f64)>`

  The parts from `u0` to `u1` of the layer boundary at offset `v` that
  no joining core passes through.

- `pub fn layer_span(&self, c: &Column, i: usize, count: usize, v: f64) -> (f64, f64)`

  The ends along the wall of the layer offset `v` in column `i` (of
  `count`): the column's edges interpolated across the wall, with the
  core reaching on through a T-joined wall's outer layers at the ends.

- `pub fn layer_pieces(&self, v0: f64, v1: f64, a: (f64, f64), b: (f64, f64)) -> Vec<((f64, f64), (f64, f64))>`

  A layer strip from `v0` to `v1`, spanning `a` along the wall at `v0`
  and `b` at `v1`, split where joining cores pass through it: the
  pieces as `(a, b)` pairs.

- `pub fn outline(&self, left: (f64, f64), right: (f64, f64)) -> Vec<DVec2>`

  Plan outline, counter-clockwise, of the part of the wall whose left
  face spans `left` and right face spans `right`.

- `pub fn outline_area(&self, left: (f64, f64), right: (f64, f64)) -> f64`

  Exact plan area of `Self::outline` (arcs included).

- `pub fn column_polygon(&self, c: &Column) -> Vec<DVec2>`

  Plan footprint of a column, counter-clockwise.

- `pub fn footprint(&self) -> Vec<DVec2>`

  Full plan footprint of the wall.

- `pub(crate) fn set_openings(&mut self, mut openings: Vec<Opening>)`

  Cut openings and split the wall into columns.

- `pub(crate) fn set_recesses(&mut self, recesses: Vec<Recess>)`

  Recess the faces (niches): each is kept inside the wall's faces and
  height and no deeper than leaves `RECESS_MIN_BACK` behind it; one
  that overlaps an opening or an earlier recess is left out. Call after
  `Self::set_openings`.

- `pub fn recess_area(&self, r: &Recess) -> f64`

  Exact plan area of a recess (between its face and its back).

- `pub fn recess_volume(&self, r: &Recess) -> f64`

  The volume a recess takes out of the wall.

- `pub fn recess_of(&self, element: ElementId) -> Option<&Recess>`

  The recess of `element`, if it made one.

- `pub fn recesses_at(&self, z: f64) -> impl Iterator<Item = &Recess>`

  The recesses cut by a horizontal plane at `z`.

- `pub fn volume(&self) -> f64`

  Net volume: each column's plan area times its solid height, less the
  recesses. Computed analytically because the body solid omits end caps
  at joins (and meshes arcs with chords).

- `pub fn column_at(&self, u: f64) -> Option<&Column>`
- `pub fn to_solid(&self, color: [u8; 3]) -> Solid`

  Exterior faces of the wall body (arc faces as smooth facets).


<a id="cc_geom-wall-layergap"></a>

### struct `LayerGap`

```rust
pub struct LayerGap
```

Where the core of a wall met in a T runs through the outer layers of the
wall it meets (layer cleanup): those layers open between the core's faces.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `v_face: f64`

  Offsets of this wall's face and of its core's face on that side.

- `v_core: f64`
- `at_face: (f64, f64)`

  Along this wall, where the joining core's faces cross the face and
  the core's face (each sorted).

- `at_core: (f64, f64)`
- `merge: bool`

  The cores are of one material: no line between them along the core's face.


**Methods**

- `pub fn at(&self, v: f64) -> Option<(f64, f64)>`

  The opening along the wall at offset `v`; `None` outside the outer layers.


<a id="cc_geom-wall-endjoin"></a>

### struct `EndJoin`

```rust
pub(crate) struct EndJoin
```

How one end of a wall is cut by a neighbor.

**Traits:** `Debug`, `Clone`, `Copy`

**Fields**

- `ray_left: f64`

  Distance along the ray (pointing into this wall) of the ray-left and ray-right faces.

- `ray_right: f64`
- `host: Option<usize>`

  For a T-join, the wall whose face this end meets.


<a id="cc_geom-wall-wallline"></a>

### struct `WallLine`

```rust
pub(crate) struct WallLine
```

Minimal wall description used while resolving joins.

**Traits:** `Debug`, `Clone`, `Copy`

**Fields**

- `start: DVec2`
- `end: DVec2`
- `half: f64`
- `join_start: bool`

  Whether each end may join (a disallowed end stays square and is ignored by neighbors).

- `join_end: bool`
- `bulge: f64`

  Bulge of the centerline (0 for a straight wall; see `cc_core::arc`).

- `drawn: [DVec2; 2]`

  The ends of the line the wall was drawn on (its location line): ends
  meet where these coincide, even when the body centerlines, shifted
  across by the location line, do not.


**Methods**

- `pub(crate) fn arc(&self) -> Option<Arc>`

  The centerline arc, when the wall bulges and both faces keep a positive radius.


<a id="cc_geom-wall-core_through"></a>

### fn `core_through`

```rust
pub(crate) fn core_through(walls: &[WallLine], me: usize, at_start: bool, host: usize, core: (f64, f64), host_core: (f64, f64)) -> Option<((f64, f64), LayerGap)>
```

Layer cleanup at a T: the end of `walls[me]` (core faces at offsets
`core`) runs its core on through the outer layers of `walls[host]` on its
side, to the host's core (faces at `host_core`). Returns the `u` of the
core's left and right faces' ends and the host's gap (`merge` unset).

<a id="cc_geom-wall-embedded_in"></a>

### fn `embedded_in`

```rust
pub(crate) fn embedded_in(wall: &WallLine, host: &WallLine) -> Option<(f64, f64)>
```

Where a straight wall (a curtain wall) lies inside another straight
wall `host`, parallel to it and within its faces: the span `(u0, u1)`
along the host it takes up.

<a id="cc_geom-wall-resolve_join"></a>

### fn `resolve_join`

```rust
pub(crate) fn resolve_join(walls: &[WallLine], me: usize, at_start: bool) -> Option<EndJoin>
```

Resolve the join at one end of `walls[me]` (`at_start` selects the end).
Faces of arc walls are their true circles; where two faces do not cross
the end is cut along the tangent of the arc at the end instead.

<a id="cc_geom-wall-arc_band_area"></a>

### fn `arc_band_area`

```rust
pub(crate) fn arc_band_area(a: &Arc, half: f64, left: (f64, f64), right: (f64, f64)) -> f64
```

Exact plan area of the band `half` either side of the arc `a` whose left
side spans `left` and right side `right` along the arc, its ends cut
straight across.

<a id="mod-cc_geom-wall_rebar"></a>

## `cc_geom::wall_rebar`

<sub>`crates/cc_geom/src/wall_rebar.rs`</sub>

Wall reinforcement: horizontal bars (ring bars in an arc wall) and
vertical bars near each face of a wall's structure, and ties between
the faces. Bars lie `cover` inside the face of the wall's core (its
whole body when it has no layers), stop at its free ends and around its
openings (horizontal bars run to an end joined to another wall), follow
the arc of an arc wall at their own radius, and are lap spliced past the
stock length.

**Items:** [`WallSets`](#cc_geom-wall_rebar-wallsets) · [`levels`](#cc_geom-wall_rebar-levels) · [`stations`](#cc_geom-wall_rebar-stations) · [`wall_bars`](#cc_geom-wall_rebar-wall_bars)

<a id="cc_geom-wall_rebar-wallsets"></a>

### struct `WallSets`

```rust
pub(crate) struct WallSets<'a>
```

What a set needs to know of the other sets of its wall.

**Traits:** `Debug`, `Clone`, `Copy`, `Default`

**Fields**

- `other: f64`

  Largest bar of the other family (horizontal or vertical) near the
  same face: an `inner` set lies inside it.

- `horizontal: Option<(&'a Rebar, f64)>`

  The exterior face's horizontal and vertical sets with their bar
  diameters, and the interior face's cover and outer bar diameter:
  what ties are placed by.

- `vertical: Option<(&'a Rebar, f64)>`
- `interior: Option<(f64, f64)>`
- `past: (f64, f64)`

  How far horizontal bars run past the wall's start and end, where
  another wall carries on in the same line (0: they stop at the joint).


<a id="cc_geom-wall_rebar-levels"></a>

### fn `levels`

```rust
pub(crate) fn levels(zones: &[SpacingZone], height: f64, cover: f64) -> Vec<f64>
```

Heights above the base of the horizontal bars of `zones` in a wall
`height` high: the first `cover` up, each next one the spacing of the
zone it starts in further, the last at most `cover` below the top.

<a id="cc_geom-wall_rebar-stations"></a>

### fn `stations`

```rust
pub(crate) fn stations(range: (f64, f64), spacing: f64, offset: f64, cover: f64) -> Vec<f64>
```

Positions along the wall of its vertical bars: every `spacing` from
`offset` after `cover` inside the start, to `cover` inside the end.

<a id="cc_geom-wall_rebar-wall_bars"></a>

### fn `wall_bars`

```rust
pub(crate) fn wall_bars(g: &WallGeom, r: &Rebar, d: f64, sets: &WallSets) -> Vec<Bar>
```

The bars of the set `r` (bars `d` thick) in the wall `g`.

{% endraw %}
