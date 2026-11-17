# Spline walls: notes for the next tasks (#307)

Stage 1 (model, geometry, plans, commands, refusals, IFC export) is on
`feat/spline-walls`, stacked on `feat/wall-profiles`, and so is the app task
(the wall tool's Spline mode, see *The app*). These notes are for the later
stages (openings, the other hosts, IFC).

## The representation

- **Model** — `cc_core::wall_spline::WallSpline { control: Vec<DVec2> }` on
  `Wall::spline` (format 32): the control points `p0 c1 c2 p1 c1 c2 p2 …`
  (`3n + 1` for `n` cubic Bézier segments, at most 64) of the wall's **drawn
  location line**, the form `cc_core::spline` and `SketchCurve::Spline`
  already use. `WallSpline::through(fit)` interpolates fit points
  (`spline::interpolate`: the knots are the fit points). A spline wall has
  `bulge == 0`, and `start`/`end` are the curve's ends: `EditCtx::finish` →
  `sync_wall_splines` keeps them so (the curve wins). An edit that moves
  `start`/`end` and leaves the curve alone takes the curve along: the whole
  of it when both ends moved alike, else each end's knot with its handle
  (`WallSpline::move_end`) — so `move_endpoint`, grips and corner moves work
  without knowing about splines.
- **Why control points** — exact under move, rotate and mirror (a mirror
  also reverses them, as it swaps the ends of any wall), exact to split
  (`spline::split_at`), and what the sketch tool, `CurveArg::Spline` and
  the grips of sketches already speak.
- **The flattened path** — everything that needs the shape works on
  `WallPath` (`WallSpline::path()`): stations along the curve with, at each,
  the point, the **true unit normal** of the curve and its signed curvature.
  Wall-local `u` is the distance along the chords from the start, `v` the
  distance to the left. `at(u, v)` interpolates, between the two stations
  around `u`, the points `v` along their normals; so a line at a constant
  `v` (a face, a layer boundary, the body centerline of an off-center
  location line) is the polyline through the **exact offsets** of the
  stations, and every offset shares the same stations. Past the ends the
  path carries on straight along its end direction (mitered ends reach a
  little beyond). `locate(p)` is the exact inverse (a quadratic per chord,
  the nearest stretch wins). `offset(d)` gives the path `d` to the left
  (`Project::wall_path` = the body centerline; an offset of a Bézier is not
  a Bézier, so nothing downstream needs one).
- **Why a polyline with true normals rather than an analytic curve** — the
  rest of the program already draws and meshes arc walls as chords; with
  the patch form every consumer of `WallGeom::world/local/path/tangent`
  works unchanged, areas and volumes are exact for the body actually drawn
  and meshed (plan outline = solid footprint = quantity), joins can cross
  faces exactly, and there is no numeric arc-length inversion to bound.

## Tolerances and bounds (constants of `cc_core::wall_spline`)

| Constant | Value | Meaning |
| --- | --- | --- |
| `SAGITTA` | 0.5 mm | a chord strays at most this from the location line |
| `MAX_TURN` | 6° | the curve turns at most this over one chord |
| `MAX_CHORDS` | 2048 | past it the two tolerances loosen (×2, ×1.5 per round, 16 rounds, then 16 chords per segment) |
| `MAX_SEGMENTS` | 64 | Bézier segments (`spline::MAX_SEGMENTS`) |
| `MAX_COORD` | 1e6 m | control points lie within it |
| `MIN_LENGTH` | 5 cm | least length, and least distance between the ends (no closed splines) |
| `MIN_FACE_RADIUS` | 5 cm | least radius left to the face inside a bend |
| `MAX_KINK_DEG` | 1° | direction change allowed where two segments join |

- Faces are offsets of the stations, so a face on the outside of a bend of
  radius `R` strays `(R + w) / R` times the centerline's sagitta: under 1 mm
  for any bend the radius rule lets through with ordinary thicknesses,
  at most about 2× the centerline's in the worst allowed case.
- Lengths are the chords': short of the true curve by `θ² / 24` per chord,
  under 1 part in 10 000 (tests: a Bézier half circle is within 0.1 % of the
  arc wall in length and volume; a spline through 9 points of the circle
  within 0.5 %, which is the interpolation, not the flattening).
- Areas and volumes are those of the chord polygons, exactly: `outline_area`,
  `strip_area`, `volume`, `layer_volumes` and the closed mesh agree to 1e-9
  (side facets carry their own normals, `WallGeom::facet_normal`, because
  `Solid::volume` trusts face normals).
- `WallSpline::check(left, right)` (`left`/`right`: the faces as signed
  offsets from the location line, `Project::wall_faces(type, line)`;
  `Project::check_wall_spline(type, line, &spline)`) returns the path or a
  `SplineError`: `Points` (count, numbers, size), `Short` (too short or
  closed), `Corner(p)`, `Tight(p)` (curvature at any station leaves a face
  less than `MIN_FACE_RADIUS`, or a cusp the flattening could not resolve),
  `Crossing(p)` (a face polyline crosses itself or the other face: loops,
  hairpins and spirals too close for the thickness). The radius rule is the
  brief's "radius ≥ half the thickness plus a margin", measured from the
  location line to the inner face so off-center location lines are handled.
- Repair (`repair_wall_splines`, run on every command through
  `Project::repairs_needed`) uses the same check: a failing curve is dropped
  and the wall is the straight wall between its stored ends.

## What happens where: every `arc` / `bulge` consumer

"Supported" = works on the curve. "Skipped" = silently left out. "Refused" =
the command fails with a message; repair neutralizes it in a file.

| Consumer | Spline wall | How |
| --- | --- | --- |
| `Wall::arc()` | — | `None`; `Wall::is_curved()` is true |
| `Wall::length/at/point_at/project/tangent_at/normal_at/offset_line` | supported | `spline_path()` |
| `Project::wall_axis`, `wall_length`, `wall_path`, `wall_faces` | supported | offset path |
| `Element::translate/transform` (mirror) | supported | control points mapped; a mirror reverses them |
| copy, delete | supported | nothing hosted to carry |
| `dim_refs` (`dim_ref_at`, `resolve_dim_ref`, `base_point`) | supported | the arc branch reads `is_curved()`: along/across on the path |
| `dim_refs::opening_frame` | n/a | no hosted elements |
| `wall_ops::split` | supported | `WallSpline::split` (exact) |
| `wall_ops::offset_copy` | supported | `spline::offset` (approximate: exact on straight parts, a few mm on bends), result checked |
| `wall_ops::move_endpoint` (and walls sharing the moved end) | supported | the sync stretches the end knot and handle |
| `wall_ops::trim_corner`, `trim_to`, `align_to`, `path`, `path_of` | refused | `EditError::Spline` → `msg-spline-wall`; `path_of` gives `None` |
| `wall_ops::curve_of` | supported | `SketchCurve::Spline` |
| `sync_wall_profiles`, repair of profiles | skipped | a spline wall keeps no profile |
| repair of `bulge` | — | a spline wall's bulge is set to 0 |
| `wall_framing::frame` (layout) | skipped | no members on a curved wall |
| `cc_geom` `WallLine` (`length`, `end_ray`, `locate`, `tangent_at`, `face`, `locate_point`) | supported | `WallLine::path`; faces are `Curve::Poly` |
| `resolve_join`, `t_join`, `core_through` | supported | see *Joins* |
| `embedded_in` (curtain wall inside a wall) | skipped | a spline wall neither embeds nor is embedded |
| `WallGeom::world/local/tangent/normal_at/frame_at/path/outline/footprint` | supported | `WallGeom::spline` |
| `outline_area`, `strip_area`, `volume`, `layer_volumes`, `centroid_u` | supported | chord polygons; heights are level along a spline wall |
| `to_solid`, `banded_solid` (layer extents) | supported | `steps()`: a break at every station of either face; smooth facets |
| `recess_area`, `recess_faces`, `set_openings` with openings | n/a | `build_walls` gives a spline wall no openings or recesses |
| `build_walls`: profiles, hosted, curtain | skipped | no profile, no hosted elements; a curtain type gets no path (straight) |
| `attach_wall_tops` | skipped | spline walls are filtered out |
| `bounding_footprints` (rooms), `closed_meshes`, `shared_volume`, joins of elements | supported | footprints and meshes |
| `rebar` (wall sets), `wall_rebar::scale` | skipped | a set hosted by a spline wall gets no bars |
| `cc_views::plan` (`draw_wall`, `draw_banded_wall`, layer cleanup, hatches, below/above cut) | supported | `path`/`world` only |
| `projected::render`, 3D, shadows, path tracer | supported | from the solid |
| `schedule` (`Length`, `Area`, `Volume`, materials) | supported | `wall_length`, length × height, geometry |
| `annotate` (door/window tags) | n/a | none hosted |
| `wall_create` | supported | `through` / `control` |
| `element_get` / `element_set` | supported | `params.spline`, `derived.spline`; checked by `wall_spline::check_wall` |
| `check_arc`, arc `derived` (`center`, `radius`, `sweep`) | n/a | arcs only |
| `door_create`, `window_create`, `niche_create`, a hosted element's `host` | refused | `hosted_at`, `element_set` |
| `wall_attach`, `attach_top` in `element_set` | refused | |
| `wall_profile_set/slope`, `sketch_set` on a wall, `wall_create` `profile` | refused | `check_takes_profile` |
| curtain wall type (`wall_create` `type`, `element_set` `type`) | refused | `check_wall`; `type_set` turning a type used by spline walls into a curtain type, or thickening it past a curve, is stopped by the repair check (generic message) |
| `wall_rebar_create` | refused / skipped | named walls refused; by level or project left out |
| `rebar_copy` to a spline wall | refused | |
| `wall_framing_create` | refused | "curved" |
| `strip_footings_under_walls` | skipped | counted in `skipped` |
| `trim_extend`, `trim_corner`, `element_align` | refused | subject or reference |
| `element_split`, `element_offset`, move, rotate, mirror, copy, delete | supported | |
| `elements_mirror` `axis_element` = a spline wall | approximate | the chord between the centerline's ends |
| `defaults::walls_loop` (roof and floor outlines from walls) | supported | the path's points |
| `nearest_wall` (hosting by point) | supported | distance to the path (then refused) |
| `mass_walls_create` | unaffected | makes straight walls |
| IFC `write_wall` | supported | `Brep` body, see *IFC* |
| IFC `wall_body`, `wall_axis`, `layer_bodies`, openings | n/a | not reached for a spline wall |
| IFC import | fallback | a component with mesh pieces, class `IfcWall` |
| app `model_snaps` | supported | the location line's chords snap as lines; curved outlines are not snapped |
| app properties (sweep row), arc grips, `axis_of`, wall tool | done (app) | see *The app* below |

## Joins (what was achieved)

- A spline wall's **ends** join exactly: in `resolve_join` its face is
  `Curve::Poly`, the chords of the face within `FACE_REACH` (5 m, or 40 half
  thicknesses) of the joint, carried on straight past the wall's end, and
  `meet` crosses it with lines, circles and other polylines. The end's
  direction (`end_ray`) is the curve's tangent there, so L joins miter,
  T joins butt, three or more ends pick the through pair by direction, and
  collinear tangent ends butt square — as a straight wall with that
  direction would, but with the corner on the true faces (tests: shared
  corners within 1e-6 between spline/straight, spline/spline, spline/arc).
- Walls ending **on the curved side** of a spline wall are supported too:
  the T join cuts their end along the host's face polyline, the host's face
  opens there (`face_gaps`) and the layer cleanup (`core_through`,
  `layer_gaps`) runs as for arcs, since it only uses `locate`, `locate_point`
  and `face`. (The brief allowed a butt without cleanup; the polyline face
  made the full join cheap.)
- Joins look no farther than `FACE_REACH` along a spline wall from the
  joint, and ends are clamped to 10 half thicknesses as before.

## Layer extents (base branch)

The band code only needed `world`, `span_at`, `steps` and the strip area:
banded spline walls build (stepped solid, per-layer volumes; test
`layers_with_their_own_extents_step_a_spline_wall`). In a mitered end column
an interior layer boundary of the **solid** is broken at interpolated
stations, so it can stray up to a couple of millimeters from the plan's line
there (the plan itself uses the exact stations). `layer_outlines` and IFC
`layer_bodies` are not reached: a spline wall is exported as one shell.

## Commands

- `wall_create { start, end, through | control, … }`: `start` and `end` stay
  required, as for lines and arcs (the graph node's ports and every
  existing caller are unchanged). `through` holds the points **between**
  them (the ends may be repeated: points repeated in a row count once);
  `control` is the full list `p0 … pn`, whose first and last points must be
  `start` and `end`. With `mid`, `bulge` or `radius` it is refused. Errors
  are `InvalidArguments` with `SplineError`'s text.
- `element_get`: `params.spline.control`, `derived.length` (centerline),
  `derived.spline { segments, min_radius, face_radius, knots }` (radii `null`
  on a straight curve).
- `element_set`: `params.spline` (`{"control": […]}` or `null`), or
  `params.start` / `params.end` to move an end with its handle.
- `cc_engine::commands::wall_spline` (crate-private helpers): `spline_arg`,
  `check_wall`, `refuse`, `derived`.

## The app (wall tool Spline mode) — done

- **Drawing**: `tools::WallMode` (`AppState::wall_mode`: *Line*, *Arc*,
  *Spline* in the options bar) replaces the old `wall_arc` flag.
  `spline_wall_tool` keeps the points clicked in `Interaction::points`
  (snapped as any wall point, `snap_at` with the last point as anchor; a
  typed length + Enter places the next one, `typing_segment` covers it).
  A click on the last point again (a double-click does that) or Enter runs
  `finish_spline_wall`: the curve is checked first with
  `Project::check_wall_spline`, a failure goes to the status bar as its
  `key()` and the points stay; otherwise `WallCreate { through, ..
  wall_command(..) }` (the same level, type, height, top and location line
  as line walls) in one undo step, recorded with `chain_placed` so
  Backspace after it undoes the wall. Backspace pops a point
  (`chain_back`), Esc drops the wall in progress (the interaction reset).
  Switching into Spline mid-chain starts the spline at the chain's anchor.
- **Preview**: `paint_wall_spline` draws the location line and the faces
  (`wall_faces`, `path.face`), or the curve in `PALETTE.danger` with a
  circle at `SplineError::point()`; the cursor label shows the length or the
  error's text.
- **Grips**: a selected spline wall gets a square grip on each inner knot
  (the ends keep the end grips, which run `element_move_end`; their drag now
  previews the curve moved with `WallSpline::move_end` and refuses a curve
  that fails the check). `Drag::Knot` moves `control[3k-1..=3k+1]` through
  `sketch_mode::spline_moved` (made `pub(crate)`), previews and checks it,
  and writes it on release with `engine_edit::element` (one command, so one
  undo step without a merge key; a failing curve is not sent, its reason
  goes to the status bar). `bendable` already gives none (`arc()` is None).
- **Palette**: the arc angle row and the *Profile* section are left out
  (`wall_profile_ui` reads `is_curved`); the length row was already right.
- **Refusals**: the door/window/niche tool over a spline wall shows
  `msg-spline-wall-hosts` and places nothing; trim/extend/align reach the
  status bar with `msg-spline-wall` (tested).
- **Straight assumptions fixed**: `modify::axis_of` gives a spline wall as a
  mirror axis only when it is straight; Split marks and cuts at the point of
  the curve nearest the pointer (`WallPath::locate`); Offset previews the
  offset curve on the pointer's side (`WallPath::offset`).
- **Tests** (`ui_tests.rs`): `the_wall_tool_draws_spline_walls`,
  `a_spline_wall_too_tight_is_refused_with_the_reason`,
  `a_spline_walls_knot_grip_reshapes_it`,
  `the_door_tool_places_no_door_in_a_spline_wall`,
  `a_spline_walls_palette_shows_its_length_only`,
  `trimming_a_spline_wall_says_why_not`.

### Left for later (app)

- No grips on the handles (tangents) and no way to add or remove a point of
  an existing spline wall in the canvas (`element_set` `params.spline` can).
- Snapping to a spline wall's faces (only its location line's chords and
  ends snap).
- The knot drag previews only; the model changes on release.
- The warning color is the theme's `danger`, lighter in the dark theme on
  the white paper (as the sketch's errors).
- The engine's `elements_mirror` `axis_element` still takes a curved spline
  wall's chord; the app no longer offers it.
- Multi-selection summaries and the 3D view's modify tools were not
  reviewed for spline walls.

## For the later stages

- **Openings (doors, windows, niches)**: geometry is ready for them in
  principle — `set_openings` splits columns in `u`, `steps` meshes each
  column, `frame_at(u)` gives the tangent frame as for arcs — but
  `opening_parts` places straight pieces in that frame, so wide openings on
  tight bends need either a width/radius limit or bent pieces; `Recess`
  areas (`recess_area`, `recess_layer_volume`) have line and arc branches
  only. Then lift: `build_walls`' hosted filter, `hosted_at`'s `refuse`,
  the `host_ok` arms of `repair`, `check_wall`, and `Hosted::offset` is `u`
  along the body centerline (`Project::wall_path`).
- **Attached tops / profiles**: `attach_wall_tops` walks a straight
  centerline (`g.origin + g.dir * u`); it needs `g.world(u, 0)` and breaks
  at the stations. Profiles use `u` along the location line and `Lin`
  heights per column: columns must split at stations where a height varies
  (see the wall-profiles notes, "For a spline wall task").
- **Wall bars / framing**: `wall_rebar` already follows arcs by `scale` and
  true lengths; a spline needs per-bar offsets (`path.offset(v)`) and their
  lengths. Framing is straight-only by design.
- **Trim / extend**: `spline::nearest`, `split_at` and `between` are enough
  for trimming to a crossing; extending needs a rule (tangent extension is
  what `WallPath::at` already does past the ends).
- **Curtain walls**: flat panels on the chords, as arcs do
  (`cc_geom::curtain` reads `g.world` already; `curtain::layout` works in
  `u`); lift the `curtain` clause in `build_walls` and `check_wall`.
- **Strip footings**: `FoundationShape::Strip` has a bulge only.
- **IFC**: export could add an axis (`IfcBSplineCurveWithKnots` or an
  `IfcPolyline` of the stations) and a `ConstructCAD_WallSpline` property
  set with the control points (as `ConstructCAD_WallProfile` does) so
  import rebuilds the wall; a polygonal extrusion (`IfcArbitraryClosedProfileDef`
  of the footprint, extruded) is a lighter body than the shell, and
  `POLYGONAL` the matching predefined type. The fixture
  `crates/cc_ifc/tests/fixtures/spline_walls.ccad` is in the CI's IfcOpenShell
  validation list (added with the app task; not run locally).
- **Closed splines** (a ring wall as one element) are refused (`Short`):
  `start == end` breaks the wall's line everywhere.

## Pitfalls

- `Wall::length()`, `at()`, `project()` flatten the curve on each call
  (tens of microseconds); hold a `spline_path()` when calling in a loop.
- `Project::repairs_needed` checks every spline wall on every command
  (flatten + crossing sweep): linear in practice, bounded by `MAX_CHORDS`.
- A type change (thicker layers) can make a valid curve too tight: commands
  that change a wall's own type check it; `type_set` relies on the repair
  check and answers with the generic "would leave the model invalid".
- `WallGeom::length` of an off-center location line is the body
  centerline's, not the location line's (`Wall::length()`), as for arcs.
- The start cap of every wall solid is wound clockwise seen from outside
  (its normal is right); the IFC shell re-winds faces by their normals.
