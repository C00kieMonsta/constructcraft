# Wall elevation profiles: notes for the next tasks (#306)

Part 1 (model, geometry, views, commands, IFC) is on `feat/wall-profiles`.
These notes are for part 2 (the app's profile sketch) and part 3 (per-layer
vertical extents).

## The model

- `cc_core::wall_profile::WallProfile { outline: Vec<DVec2>, sketch: Option<Sketch> }`
  on `Wall::profile` (format 31; 30 is taken by another open change).
- Frame: `u` = meters along the wall's **drawn location line** from `Wall::start`
  (for a straight wall this equals `u` along the body centerline, which is the
  frame of doors, windows, niches and `WallGeom`), `z` = meters above
  `Project::wall_base_z(e, w)` (level elevation + `base_offset`).
- The outline is counter-clockwise after `WallProfile::fitted`; `check(length)`
  validates and returns `Chains { bottom, top }` (each by increasing `u`;
  consecutive points at one `u` are a vertical step). Rules: one simple loop,
  every vertical line crosses it at most twice, no holes, `u` within
  `[0, length]` (±`TOL`), `z` within `-MAX_BELOW..=MAX_ABOVE`, area ≥ `MIN_AREA`,
  span ≥ `MIN_LENGTH`, at most `MAX_POINTS`.
- Build one with `WallProfile::from_sketch(sketch, wall.length())`,
  `from_outline(points, length)` or `sloped(length, base, top)`. Errors are
  `ProfileError` values with `point()` (where, in `[u, z]`) and `key()` (i18n:
  `wall-profile-*`, or the sketch's `sketch-*` keys; all 12 languages have them).
- `WallProfile::curves()` gives the sketch, or lines along the outline.
- `holds(u0, u1, z0, z1)` tells whether a door/window rectangle fits.
- Interactions (all in one undo step, `EditCtx::finish` → `sync_wall_profiles`):
  - length change with the profile unchanged by the edit → `refit(old, shift, new)`:
    kept in place along the wall (a start moved along the wall's own line gives
    `shift`), cut at the ends, points of an end it reached follow that end when
    the wall grows there; dropped when nothing is left.
  - wall becomes an arc or a curtain wall → profile dropped.
  - mirror → `reversed(length)` in `Element::transform` (walls swap ends).
  - split → `wall_ops::split` gives the second wall `refit(len, t, len - t)`.
  - a profile and `attach_top` never coexist: commands refuse, repair drops
    `attach_top`.
- Repair (`Project::repair`): invalid → dropped; past the wall's end → cut;
  stale sketch → dropped.

## Commands (cc_engine::commands::wall_profile)

- `wall_profile_set { wall, curves | points }`, `wall_profile_slope { wall,
  top_start, top_end, base_start, base_end }`, `wall_profile_remove { wall }`.
- `sketch_get { id: wall }` → `{curves, drawn, frame: "wall_elevation", profile}`;
  without a profile it returns the nominal rectangle `0..length × 0..height`
  with `profile: false` — the natural starting sketch for the app.
- `sketch_set { id: wall, curves }` sets the profile (same checks).
- `wall_create { …, profile: [curves] }`.
- `element_get` → `params.profile` and `derived.profile {area, min_z, max_z, start, end}`.
- Refused: arc walls, curtain walls, walls with `attach_top`, framed walls
  (`Project::wall_framing_of`). `check_takes_profile` is `pub(crate)` there.

## Geometry (cc_geom::wall, build.rs)

- `build_walls` lifts the chains to absolute `z` and:
  - turns off `join_start`/`join_end` on the `WallLine` of an end the profile
    stops short of (`END_REACH` = 1 mm), and sets `start_ext`/`end_ext` to the
    profile's ends there (square caps);
  - uses the profile's `z` range for the "joins only walls that overlap in
    height" filter;
  - keeps only openings/recesses that `Chains::holds`;
  - calls `WallGeom::set_elevation(chains)` after `set_openings` and before
    `set_recesses`.
- `WallGeom`: `base_z` stays the **nominal** base (sills and IFC placements are
  relative to it); `top_z` is the profile's highest point; `profile:
  Option<Chains>` (absolute), `bottom_profile`/`top_profile` segments
  `(u0, u1, z0, z1)`, `is_profiled()`.
- Columns are split at every chain point; a column's raw `solid` intervals are
  the heights **at its middle**. For exact heights use
  `intervals_at(column, u)` (public now), `solid_at(u, z)`, or
  `cut_columns(z)` (columns split where the chains cross a level plane; each
  piece's `solid` is exact for `is_solid_at(z)`).
- `profile_outline()` gives the outline in `(u, z_abs)` carried on to the
  corners of mitered ends (what IFC sweeps).
- `volume()` evaluates heights at the centroid of each column's footprint, so
  it is exact for planar tops/bases, also at mitered ends (this also made
  attached-top walls' volume exact).

## Pitfalls

- A chain point inside a mitered end zone (between `left.0` and `right.0`)
  does not split the column; the end column uses the first segment's line.
  Harmless in practice, but per-layer extents should not rely on it.
- `Sketch::makes` compares within `JOIN_TOL`; `fitted` snaps `u` within
  `TOL / 2` of the wall's ends, so a sketch that ends a hair past the end
  still "makes" the outline.
- `Sketch::from_polygons` (used by `check`'s crossing test) drops sides shorter
  than `JOIN_TOL`: an outline with points closer than 0.1 mm can fail as an
  open sketch.
- Doors and windows that do not fit are silently left out of the wall
  (`WallGeom::openings` lacks them; `opening_fills` too). The app may want to
  warn: compare `Project::hosted_by(wall)` with `g.openings`.
- `set_openings` still clamps openings to `[base_z, top_z]`: an opening below
  the nominal base cannot be cut even where the profile goes lower.
- Plans of profiled walls use `cut_columns`; attached-top walls still use the
  raw columns (unchanged behaviour).
- Wall framing is refused on profiled walls (members would be nominal);
  wall rebar works and stays inside the body (horizontal bars stop where the
  body ends at their height, vertical bars run between the chains with the
  cover at the base).

## Part 2: the app's profile sketch

- Open the sketch in an elevation or section that looks at the wall square
  on (or in a temporary wall elevation): map view drawing coordinates to the
  wall frame with `u = (P - w.start)·dir` (dir = `Wall::direction()`) and
  `z = height - wall_base_z`; a view looking at the wall's back face sees `u`
  growing to the left.
- Start from `sketch_get` (the nominal rectangle when there is no profile),
  finish with `sketch_set` (or `wall_profile_set`), cancel by not running it.
  Show `ProfileError::point()` with `key()` like the plan sketch does for
  `SketchError`.
- `sketch_mode`'s `Target` will need a wall-elevation target whose curves
  live in `(u, z)`; snapping to the wall's ends (`u = 0`, `u = length`), its
  base (`z = 0`) and its nominal top (`z = height`), and to levels, helps.
- "Reset profile" maps to `wall_profile_remove`.

## Part 3: per-layer vertical extents

- Today a wall is one body solid (`to_solid`) and one set of chains; layers
  exist only in plans (`layers`, `layer_span`, `layer_pieces`). Per-layer
  extents mean, per layer band `v0..v1`, its own bottom/top: e.g. offsets of
  the wall's chains, or the wall's base/top when there is no profile.
- `to_solid` would need a band per layer (or steps across `v`): the top/bottom
  faces, side faces and caps already take per-corner heights, and
  `face_pieces` takes a bottom and a top line; `intervals_at` would need a
  `v` (or per-layer chains).
- Plans: `cut_columns` per layer band, so a layer cut off below the plane is
  drawn as seen below; `layer_span`/core cleanup (`core_ends`, `layer_gaps`)
  assume all layers share the column's state.
- Volumes: per layer, so quantities by material can follow.
- IFC: a single swept outline no longer describes the wall; consider one
  item per layer (each its own outline/heights), or `IfcBuildingElementPart`s.
