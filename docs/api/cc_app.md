# `cc_app` API

{% raw %}
[← API reference](index.md) · Generated from the source by `cargo run -p api_doc`; edit the doc comments, not this page.

ConstructCraft — open source BIM authoring.

## Modules

| Module | Items | Summary |
|---|---|---|
| [`cc_app`](#mod-cc_app) | 0 | ConstructCraft — open source BIM authoring. |
| [`cc_app::actions`](#mod-cc_app-actions) | 73 | Document-level commands shared by the ribbon, browser and shortcuts. |
| [`cc_app::app`](#mod-cc_app-app) | 4 | Top-level eframe application: layout, dock and keyboard shortcuts. |
| [`cc_app::bracing_ui`](#mod-cc_app-bracing_ui) | 5 | Bracing in the app: the Bracing command (the bay between two selected columns, else the two-click tool), the tool's options bar and the properties of a selected bracing (every parameter through `element_set`, which regenerates its braces in the same undo step). |
| [`cc_app::browser`](#mod-cc_app-browser) | 3 | Project Browser: the tree of views, schedules and sheets, grouped as the project's browser organization says (`cc_core::browser`): by view type alone, or by the criteria of the scheme in use. |
| [`cc_app::browser_org_dialog`](#mod-cc_app-browser_org_dialog) | 3 | The Browser Organization dialog (*View → Browser Organization*, also in the menu of the project browser's groups): the schemes that group the browser's views. |
| [`cc_app::cad_ui`](#mod-cc_app-cad_ui) | 10 | Imported CAD drawings in the app: the Import CAD and Import Image commands (file dialog or files dropped on the window), and their properties and layers, and Explode. |
| [`cc_app::callout_ui`](#mod-cc_app-callout_ui) | 6 | Callouts in the app: the Callout tool draws a rectangle in a plan, ceiling plan, section or elevation and makes a detail view of that region (`callout_create`); the Select tool picks a callout's mark, moves it and resizes it by its corners (the detail view's crop, `view_set`) and opens its view on a double-click; the view's properties show the callout's parent and open the view. |
| [`cc_app::canvas`](#mod-cc_app-canvas) | 7 | 2D view canvas: plans, ceilings, elevations, sections, legends and drafting views. |
| [`cc_app::clipboard`](#mod-cc_app-clipboard) | 8 | Clipboard: cut, copy and paste of elements within the open project, and match type (pick an element, then click others to give them its type). |
| [`cc_app::commands`](#mod-cc_app-commands) | 8 | Interface commands: every ribbon, quick access and palette entry is a `UiCommand`, so its label, shortcut, description, the engine command it runs and whether it is available now (and why not) are defined once and read by every place that shows it. |
| [`cc_app::component_env`](#mod-cc_app-component_env) | 31 | The component modeling environment: a workspace of its own where one component is shaped. |
| [`cc_app::component_sketch`](#mod-cc_app-component_sketch) | 14 | Sketching pieces in the component modeling environment. |
| [`cc_app::connection_ui`](#mod-cc_app-connection_ui) | 3 | Connections in the app: the Structure tab's *Connections* buttons, which run `connections_create` on the selected members (else the active level's), and the rows of a selected connection, whose sizes change through `element_set`. |
| [`cc_app::curtain_ui`](#mod-cc_app-curtain_ui) | 10 | Curtain walls in the app: the Curtain Grid tool (add and remove grid lines, merge cells across a segment, swap panels by clicking a curtain wall in a plan, elevation or section), its options bar, the grid rows of the properties palette and the curtain sections of the Type Properties dialog. |
| [`cc_app::design_options_ui`](#mod-cc_app-design_options_ui) | 8 | Design options in the application: the Design Options dialog (sets, their options and primary option, accepting the primary), the menu of the option being edited, and the option rows of view, template and element properties. |
| [`cc_app::detail_component_ui`](#mod-cc_app-detail_component_ui) | 6 | Detail components in the app: the Detail Component tool (Annotate tab), which places a type in the active 2D view with the rotation, scale and mirror of its options (`detail_component_place`); *Create Detail Component*, which makes a type from the selected detail lines, regions, texts and detail components of the view and puts it in their place (`detail_component_create`); and the rows of a placed one in the properties palette. |
| [`cc_app::detail_ui`](#mod-cc_app-detail_ui) | 17 | Detail lines in the app: the *Detail Line* tool (Annotate tab) with its line, polyline, arc, circle, ellipse and rectangle modes and the line style in the options bar, the grips of selected detail lines (ends, centers, an arc's middle, axis points), their properties, and their previews in the Split and Offset tools. |
| [`cc_app::dialog`](#mod-cc_app-dialog) | 8 | Standard dialog frame shared by every modal window: a header with the title and a close button, a padded body (with an icon for alerts) and a footer with the buttons. |
| [`cc_app::dialog::session`](#mod-cc_app-dialog-session) | 3 | Edits made while a data dialog is open can be cancelled as a whole: the dialog remembers the undo depth when it opens and undoes back to it. |
| [`cc_app::dialogs`](#mod-cc_app-dialogs) | 6 | Application dialogs, all built on `crate::dialog::Dialog`. |
| [`cc_app::dim_edit`](#mod-cc_app-dim_edit) | 9 | Editing measured values in place. |
| [`cc_app::dim_type_ui`](#mod-cc_app-dim_type_ui) | 1 | The Type Properties rows of a dimension type: the marks, witness and dimension lines and pens (Graphics) and the text (Text). |
| [`cc_app::engine_edit`](#mod-cc_app-engine_edit) | 12 | Property edits through the engine. |
| [`cc_app::fill_patterns_dialog`](#mod-cc_app-fill_patterns_dialog) | 9 | The Fill Patterns dialog (*Manage → Fill Patterns*, also opened from the material browser and a filled region type's fill): the built-in patterns, read only, and the project's own with a drawn swatch each; the selected one's name, description, kind (drafting or model), scale and line families, edited in place over a live preview. |
| [`cc_app::gestures`](#mod-cc_app-gestures) | 3 | Scroll and pinch gestures over the views (issue #174). |
| [`cc_app::gpu`](#mod-cc_app-gpu) | 11 | Graphics start-up: which wgpu backends the window tries, in what order, and what its device asks of the adapter. |
| [`cc_app::graph_ui`](#mod-cc_app-graph_ui) | 6 | Visual programming: the graph editor, a panel under the views. |
| [`cc_app::graphics_dialog`](#mod-cc_app-graphics_dialog) | 19 | View graphics dialogs: Visibility/Graphics of a view or view template (categories and filters), graphic overrides of selected elements, the view filter editor and the view template manager. |
| [`cc_app::grid_frame_ui`](#mod-cc_app-grid_frame_ui) | 4 | *Structure → Grid Frame*: the levels, the types (of the tab's material) and optional secondary beams for `grid_frame_create`. |
| [`cc_app::home`](#mod-cc_app-home) | 3 | Home screen: open or create a project, recent projects and samples. |
| [`cc_app::i18n`](#mod-cc_app-i18n) | 1 | Localization lives in the engine (`cc_engine::i18n`), so the GUI, the command line and the MCP server name things alike; this module re-exports it and defines the app's `t!` macro. |
| [`cc_app::icons`](#mod-cc_app-icons) | 5 | Interface icons: SVG drawings in `assets/icons/ui` (24 x 24 grid, white strokes and fills), rasterized with resvg at the exact pixel size they are shown at and tinted with theme colors: strokes in one tone, soft fills in a second (see `Tones`), and an optional `Badge` marks what kind of action a command is. |
| [`cc_app::jobs`](#mod-cc_app-jobs) | 9 | Long commands (opening a project, importing IFC, DWG/DXF, SketchUp, pictures, component files) run on a worker thread over a copy of the document, while a modal dialog shows what is being read and for how long. |
| [`cc_app::level_mark_ui`](#mod-cc_app-level_mark_ui) | 4 | Level marks in the app: the tool that places them in plans, elevations and sections (through `level_mark_create`), their rows in the properties palette and the rows of their type. |
| [`cc_app::library`](#mod-cc_app-library) | 4 | Component library: browse the component files (`.ccomp`) of the folders the user adds (such as a copy of the ConstructCraft Components library), search them, see a component's types, parameters, connectors and plan, load it into the project, and save the project's components into a library folder. |
| [`cc_app::line_styles_dialog`](#mod-cc_app-line_styles_dialog) | 3 | The Line Styles dialog (*Annotate → Line Styles*): the built-in line styles, read only, and the project's own, whose name, weight, color and dash pattern are edited in place; *New* adds one and *Delete* removes the selected one (what uses it moves to the thin style). |
| [`cc_app::lps_cabling_ui`](#mod-cc_app-lps_cabling_ui) | 4 | Lightning protection and structured cabling in the app: the tools that place air terminals, earth electrodes, racks and data outlets (their built-in types made on first use by `device_type_create`), the earth ring around the walls, the coverage check (`lps_check`) and the cabling per rack (`cabling_list`). |
| [`cc_app::material_browser`](#mod-cc_app-material_browser) | 4 | Material browser: the project's materials in a searchable list and the selected one's identity, graphics, appearance and physical properties in tabs. |
| [`cc_app::mcp_bridge`](#mod-cc_app-mcp_bridge) | 2 | `constructcraft mcp` (or `--mcp`): MCP clients pointed at the app instead of `ccad` get the MCP server of the `ccad` next to it, on the same standard input and output, instead of a window that never answers (issue #265). |
| [`cc_app::member_host_ui`](#mod-cc_app-member_host_ui) | 3 | Rows shared by the properties of member hosts (steel buildings, trusses): the member counts and *Reset Members*, and length rows that report when an edit ends. |
| [`cc_app::modify`](#mod-cc_app-modify) | 21 | Transform tools on the selection: move, copy, rotate, mirror (picking a line or drawing the axis) and linear array. |
| [`cc_app::modify3d`](#mod-cc_app-modify3d) | 9 | The modify tools in the 3D view: move, copy, array, rotate and mirror the selection, align and join geometry. |
| [`cc_app::new_component_dialog`](#mod-cc_app-new_component_dialog) | 6 | The New Component dialog: a name and the kind the component is, picked from a list that shows the parameters each kind starts with (a generic component starts with none: they are the user's to add). |
| [`cc_app::new_project_dialog`](#mod-cc_app-new_project_dialog) | 2 | The New Project dialog: the template a new project starts from, metric or imperial, picked on one of two cards. |
| [`cc_app::obj_ui`](#mod-cc_app-obj_ui) | 4 | Import OBJ and glTF (*Insert → Import OBJ…*, *Import glTF…*): after the file is picked, a dialog asks what the format does not say (for OBJ the units and which axis is up; for both one element or one per object), then `obj_import` or `gltf_import` runs as a background job. |
| [`cc_app::opening_place`](#mod-cc_app-opening_place) | 6 | Placing a door or window on a wall in a plan: where its center goes along the wall, from the cursor, a snap of its center or one of its edges to an imported drawing or the underlay (issue #251), or a distance typed from either end of the wall (issue #250), and the temporary dimensions from its center to both ends. |
| [`cc_app::paint`](#mod-cc_app-paint) | 16 | Painting `Drawing2D`s with egui. |
| [`cc_app::paint_cache`](#mod-cc_app-paint_cache) | 1 | Painted drawings kept between frames. |
| [`cc_app::paint_ui`](#mod-cc_app-paint_ui) | 3 | The Paint and Remove Paint tools: click a face of an element in a 3D view to paint it with the material chosen in the options bar (`paint`), or to take its paint off (`paint_remove`). |
| [`cc_app::palette`](#mod-cc_app-palette) | 4 | Command palette (Ctrl+Shift+P): search every interface command, the project's views and the engine's command catalog by name, description or shortcut, and run the chosen one. |
| [`cc_app::phases_dialog`](#mod-cc_app-phases_dialog) | 9 | Construction phases in the application: the Phases dialog (phases, phase filters and the graphics of each status), the phase rows of view and element properties, and the Demolish tool. |
| [`cc_app::plugins_ui`](#mod-cc_app-plugins_ui) | 7 | Plugins in the app: loading the plugins of the plugins folder into the session (their commands join the engine's catalog, the palette and the ribbon's *Plugin Commands* menu) and the Plugins window, which turns them on and off, grants the `files` permission, removes them, and installs and updates them from the community index. |
| [`cc_app::preferences`](#mod-cc_app-preferences) | 16 | App preferences (not model data): what they hold, how they are kept in eframe's storage (every key lives here), how the interface follows them, and the Preferences window (File tab, Ctrl+,, home screen). |
| [`cc_app::profile_sketch`](#mod-cc_app-profile_sketch) | 10 | *Edit Profile*: the profile of a sweep type drawn in a sketch, in the profile's own plane (x across the path, to its left; y up from the path's height), which takes the place of the views while it is open. |
| [`cc_app::profiles_dialog`](#mod-cc_app-profiles_dialog) | 9 | The Profile Manager (*Structure → Profiles*, also *Manage → Profiles*): the project's structural profiles grouped by material, with a search; the selected one's outline drawn to fit, its name, material and dimensions (millimeters on screen, meters to the engine) and its section properties. |
| [`cc_app::properties`](#mod-cc_app-properties) | 16 | Properties palette: instance parameters of the selection, or of the active view. |
| [`cc_app::recovery`](#mod-cc_app-recovery) | 7 | Autosave and crash recovery of the open project (the files are written by `cc_core::recovery`): a copy of a project with unsaved changes is written in the background an interval (a preference: a minute by default, or off) after its last copy; a clean save, close or exit removes it; a panic writes a final copy and appends to a crash log; at start-up, copies left by earlier sessions are offered for restoring. |
| [`cc_app::region_ui`](#mod-cc_app-region_ui) | 5 | Filled and masking regions in the app: *Annotate → Filled Region* and *Masking Region* open the sketch mode in the active view (a plan, ceiling plan, elevation, section, legend or drafting view; curves in the view's drawing coordinates), and *Finish* runs `filled_region_create` or `masking_region_create` with the curves. |
| [`cc_app::render_ui`](#mod-cc_app-render_ui) | 5 | The render window (*View → Render*): the active 3D view path traced progressively (`cc_raytracing`), the picture on the left (fitted; the wheel zooms about the pointer, a drag pans, a double click fits again) and the view's render settings on the right (quality, output, environment), with *Save Image…*, *Region*, *Render*, *Stop* and *Close*. |
| [`cc_app::ribbon`](#mod-cc_app-ribbon) | 10 | Ribbon, options bar and status bar. |
| [`cc_app::schedule_ui`](#mod-cc_app-schedule_ui) | 5 | Schedule (table) view, its definition dialog (fields, filter, sorting and grouping, formatting, appearance) and CSV export. |
| [`cc_app::section_mark`](#mod-cc_app-section_mark) | 4 | Section marks in plans: picked with the Select tool, moved by dragging, their ends and the pieces of a split line dragged by their handles, flipped with Space. |
| [`cc_app::sheet`](#mod-cc_app-sheet) | 1 | Sheet composer: paper, title block and placed viewports. |
| [`cc_app::sheet_props`](#mod-cc_app-sheet_props) | 4 | A sheet's rows in the properties palette: its number, title block, paper and issue date, and the title block's per-sheet fields. |
| [`cc_app::side_panels`](#mod-cc_app-side_panels) | 7 | The side panels, Project Browser and Properties: docked on either side of the window, stacked in one column or each in a column of its own. |
| [`cc_app::site_ui`](#mod-cc_app-site_ui) | 3 | Site Location (Manage → Project): a window with an OpenStreetMap map, an address search, a marker fixed at the center and the square of map data to bring in. |
| [`cc_app::sketch_mode`](#mod-cc_app-sketch_mode) | 21 | Sketch mode: the outline of a floor, ceiling, roof, foundation slab or mass drawn with lines, rectangles, polygons, circles, arcs and ellipses (see `cc_core::sketch`), in the active plan. |
| [`cc_app::snap_geom`](#mod-cc_app-snap_geom) | 3 | Snap points that come from two pieces of geometry, or from the point a segment is drawn from (issue #159): where lines and arcs cross, the foot of the perpendicular from the anchor, and the points where a line from the anchor touches an arc or circle. |
| [`cc_app::snaps`](#mod-cc_app-snaps) | 3 | Object snaps (issue #317): which points the tools snap to (a preference), and the snap points of the elements a view draws, so that every element shown has them, not only those the tools know by their geometry. |
| [`cc_app::stair_ui`](#mod-cc_app-stair_ui) | 13 | Stair and ramp controls: the plan layout (options bar and properties), the construction and step rules of stair types, the rules of ramp types, and their checks. |
| [`cc_app::state`](#mod-cc_app-state) | 4 | Application state shared by all panels and views. |
| [`cc_app::steel_building_ui`](#mod-cc_app-steel_building_ui) | 5 | *Structure → Steel Building*: the dialog that runs `steel_building_create` on the active level, and the properties of a selected building (every parameter, through `element_set`, which regenerates its members in the same undo step). |
| [`cc_app::sun_overlay`](#mod-cc_app-sun_overlay) | 1 | A 3D view's sun: the light the renderer draws it with (`cc_render::Light`) and its sun path diagram painted over the image (`cc_views::sun_path::diagram`, projected through the live camera). |
| [`cc_app::sun_ui`](#mod-cc_app-sun_ui) | 5 | The Sun Settings dialog (*View → Sun Settings*, and the *Sun* row of a view's properties): how a view places its sun (one moment, a study over one or several days, or a light from a direction given by hand), the project's sun presets, the site's time zone and what is drawn (the sun path, shadows, a ground plane). |
| [`cc_app::systems_dialog`](#mod-cc_app-systems_dialog) | 5 | Plumbing systems, electrical circuits, the structured cabling of racks and lightning protection systems: names, codes, colors, the board (or rack) feeding each circuit (or cabling) and its voltage, with the circuits' loads. |
| [`cc_app::table`](#mod-cc_app-table) | 16 | Property table primitives shared by the properties palette and dialogs: collapsible sections of two-column rows, header blocks and padded content. |
| [`cc_app::text_tool`](#mod-cc_app-text_tool) | 10 | Text notes in 2D views: the Text tool (single-line or multiline text, optionally with a leader of one or two segments), the Add Leader tool and the text editor shown over the view. |
| [`cc_app::theme`](#mod-cc_app-theme) | 22 | Visual theme: color tokens, typography and spacing applied to egui. |
| [`cc_app::thumbnails`](#mod-cc_app-thumbnails) | 5 | Project thumbnails: a small render of the model, stored in the application data folder and shown on the home screen. |
| [`cc_app::title_block_env`](#mod-cc_app-title_block_env) | 38 | The title block editor: the component environment for a title block (`ComponentKind::TitleBlock`). |
| [`cc_app::tools`](#mod-cc_app-tools) | 61 | Modeling tools: selection, walls, hosted openings, sketches, sections, rooms, grids and dimensions. |
| [`cc_app::truss_ui`](#mod-cc_app-truss_ui) | 4 | Trusses in the app: the Truss tool's options bar and the properties of a selected truss (every parameter through `element_set`, which regenerates its members in the same undo step). |
| [`cc_app::type_dialog`](#mod-cc_app-type_dialog) | 1 | Type Properties dialog: switch, duplicate and rename types, edit the compound structure (layers) and the type parameters in sections. |
| [`cc_app::units_dialog`](#mod-cc_app-units_dialog) | 1 | Project Units dialog: unit, decimals and symbol of each quantity, the currency and the number style, with a live sample of each format. |
| [`cc_app::update`](#mod-cc_app-update) | 16 | Update notice: at startup (at most once a day) and on request, the app asks GitHub for the latest release of ConstructCraft in a background thread and, when it is newer than this build, shows a notice with a link to the release page. |
| [`cc_app::update_install`](#mod-cc_app-update_install) | 19 | One-click updates: download the package of a newer release that fits how this copy was installed, verify it, and put it in place. |
| [`cc_app::view3d`](#mod-cc_app-view3d) | 3 | Interactive 3D view backed by `cc_render::Renderer3D`. |
| [`cc_app::view_cube`](#mod-cc_app-view_cube) | 4 | Navigation cube for 3D views: a cube aligned with the world axes and a compass ring, drawn in the view's corner. |
| [`cc_app::wall_framing_ui`](#mod-cc_app-wall_framing_ui) | 8 | *Structure → Light Steel Frame / Wood Frame*: the dialog that runs `wall_framing_create` on the selected walls (or the active level's), the properties of a selected framing (through `element_set`, which regenerates its members in the same undo step), and the *Framing* row of its members. |

<a id="mod-cc_app"></a>

## `cc_app`

<sub>`crates/cc_app/src/main.rs`</sub>

<a id="mod-cc_app-actions"></a>

## `cc_app::actions`

<sub>`crates/cc_app/src/actions.rs`</sub>

Document-level commands shared by the ribbon, browser and shortcuts.

**Items:** [`material_type_ref`](#cc_app-actions-material_type_ref) · [`structure_place`](#cc_app-actions-structure_place) · [`run_command`](#cc_app-actions-run_command) · [`new_plan`](#cc_app-actions-new_plan) · [`new_3d`](#cc_app-actions-new_3d) · [`new_legend`](#cc_app-actions-new_legend) · [`new_drafting_view`](#cc_app-actions-new_drafting_view) · [`new_schedule`](#cc_app-actions-new_schedule) · [`new_sheet`](#cc_app-actions-new_sheet) · [`t_category`](#cc_app-actions-t_category) · [`duplicate_view`](#cc_app-actions-duplicate_view) · [`delete_view`](#cc_app-actions-delete_view) · [`rename_view`](#cc_app-actions-rename_view) · [`sheet_of`](#cc_app-actions-sheet_of) · [`place_on_sheet`](#cc_app-actions-place_on_sheet) · [`place_on_sheet_at`](#cc_app-actions-place_on_sheet_at) · [`remove_viewport`](#cc_app-actions-remove_viewport) · [`add_level_beside`](#cc_app-actions-add_level_beside) · [`delete_level`](#cc_app-actions-delete_level) · [`open_default_3d`](#cc_app-actions-open_default_3d) · [`activate_category_tool`](#cc_app-actions-activate_category_tool) · [`new_project`](#cc_app-actions-new_project) · [`set_language`](#cc_app-actions-set_language) · [`guarded`](#cc_app-actions-guarded) · [`run_pending`](#cc_app-actions-run_pending) · [`open`](#cc_app-actions-open) · [`open_path`](#cc_app-actions-open_path) · [`save_component`](#cc_app-actions-save_component) · [`load_component`](#cc_app-actions-load_component) · [`load_component_path`](#cc_app-actions-load_component_path) · [`import_skp`](#cc_app-actions-import_skp) · [`export_ifc`](#cc_app-actions-export_ifc) · [`SheetFormat`](#cc_app-actions-sheetformat) · [`active_sheet`](#cc_app-actions-active_sheet) · [`export_sheet`](#cc_app-actions-export_sheet) · [`IMAGE_DPI`](#cc_app-actions-image_dpi) · [`image_view`](#cc_app-actions-image_view) · [`export_image`](#cc_app-actions-export_image) · [`export_dxf`](#cc_app-actions-export_dxf) · [`toggle_macro_recording`](#cc_app-actions-toggle_macro_recording) · [`run_macro`](#cc_app-actions-run_macro) · [`run_macro_file`](#cc_app-actions-run_macro_file) · [`save`](#cc_app-actions-save) · [`select_instances`](#cc_app-actions-select_instances) · [`group_selection`](#cc_app-actions-group_selection) · [`ungroup_selection`](#cc_app-actions-ungroup_selection) · [`grids_for`](#cc_app-actions-grids_for) · [`reference_grid`](#cc_app-actions-reference_grid) · [`align_grid_ends`](#cc_app-actions-align_grid_ends) · [`selected_columns`](#cc_app-actions-selected_columns) · [`columns_at_grids`](#cc_app-actions-columns_at_grids) · [`beams_on_grids`](#cc_app-actions-beams_on_grids) · [`footings_under_columns`](#cc_app-actions-footings_under_columns) · [`strip_footings_under_walls`](#cc_app-actions-strip_footings_under_walls) · [`slab_join_supports`](#cc_app-actions-slab_join_supports) · [`open_analytical_view`](#cc_app-actions-open_analytical_view) · [`export_analytical`](#cc_app-actions-export_analytical) · [`typical_rebar`](#cc_app-actions-typical_rebar) · [`open_rebar_view`](#cc_app-actions-open_rebar_view) · [`open_structural_plan`](#cc_app-actions-open_structural_plan) · [`mep_connect`](#cc_app-actions-mep_connect) · [`open_panel_diagram`](#cc_app-actions-open_panel_diagram) · [`open_mep_plan`](#cc_app-actions-open_mep_plan) · [`number_structure`](#cc_app-actions-number_structure) · [`active_floor_plan`](#cc_app-actions-active_floor_plan) · [`toggle_structure_tags`](#cc_app-actions-toggle_structure_tags) · [`beams_between_columns`](#cc_app-actions-beams_between_columns) · [`hide_in_view`](#cc_app-actions-hide_in_view) · [`temporary`](#cc_app-actions-temporary) · [`temporary_permanent`](#cc_app-actions-temporary_permanent) · [`selected_masses`](#cc_app-actions-selected_masses) · [`from_masses`](#cc_app-actions-from_masses) · [`terrain_on_level`](#cc_app-actions-terrain_on_level)

<a id="cc_app-actions-material_type_ref"></a>

### fn `material_type_ref`

```rust
pub fn material_type_ref(state: &mut AppState, category: Category) -> Option<cc_engine::Ref>
```

The type of `category` in the Structure tab's material (made when the
project has none, as an undo step).

<a id="cc_app-actions-structure_place"></a>

### fn `structure_place`

```rust
pub fn structure_place(state: &mut AppState, category: Category, material: cc_core::StructuralMaterial)
```

Start the column or beam tool of the Structure tab on a type of `material`.

<a id="cc_app-actions-run_command"></a>

### fn `run_command`

```rust
pub fn run_command<C: cc_engine::Command + serde::Serialize>(state: &mut AppState, command: C) -> Option<serde_json::Value>
```

Run an engine command; failures go to the status bar.

<a id="cc_app-actions-new_plan"></a>

### fn `new_plan`

```rust
pub fn new_plan(state: &mut AppState, level: LevelId, ceiling: bool) -> Option<ViewId>
```

<a id="cc_app-actions-new_3d"></a>

### fn `new_3d`

```rust
pub fn new_3d(state: &mut AppState) -> Option<ViewId>
```

<a id="cc_app-actions-new_legend"></a>

### fn `new_legend`

```rust
pub fn new_legend(state: &mut AppState) -> Option<ViewId>
```

<a id="cc_app-actions-new_drafting_view"></a>

### fn `new_drafting_view`

```rust
pub fn new_drafting_view(state: &mut AppState) -> Option<ViewId>
```

<a id="cc_app-actions-new_schedule"></a>

### fn `new_schedule`

```rust
pub fn new_schedule(state: &mut AppState, category: Category) -> Option<ViewId>
```

<a id="cc_app-actions-new_sheet"></a>

### fn `new_sheet`

```rust
pub fn new_sheet(state: &mut AppState) -> Option<ViewId>
```

<a id="cc_app-actions-t_category"></a>

### fn `t_category`

```rust
pub fn t_category(category: Category) -> String
```

<a id="cc_app-actions-duplicate_view"></a>

### fn `duplicate_view`

```rust
pub fn duplicate_view(state: &mut AppState, id: ViewId, detailing: bool)
```

Copy a view and open the copy; with `detailing`, what is drawn in it too.

<a id="cc_app-actions-delete_view"></a>

### fn `delete_view`

```rust
pub fn delete_view(state: &mut AppState, id: ViewId)
```

<a id="cc_app-actions-rename_view"></a>

### fn `rename_view`

```rust
pub fn rename_view(state: &mut AppState, id: ViewId, name: String)
```

<a id="cc_app-actions-sheet_of"></a>

### fn `sheet_of`

```rust
pub fn sheet_of(project: &Project, view: ViewId) -> Option<ViewId>
```

Sheet that already shows `view`, if any.

<a id="cc_app-actions-place_on_sheet"></a>

### fn `place_on_sheet`

```rust
pub fn place_on_sheet(state: &mut AppState, sheet: ViewId, view: ViewId)
```

<a id="cc_app-actions-place_on_sheet_at"></a>

### fn `place_on_sheet_at`

```rust
pub fn place_on_sheet_at(state: &mut AppState, sheet: ViewId, view: ViewId, center: glam::DVec2)
```

Place `view` on `sheet` centered at `center` (paper millimeters) and
select the new viewport; errors (already on a sheet…) go to the status bar.

<a id="cc_app-actions-remove_viewport"></a>

### fn `remove_viewport`

```rust
pub fn remove_viewport(state: &mut AppState, sheet: ViewId, index: usize)
```

<a id="cc_app-actions-add_level_beside"></a>

### fn `add_level_beside`

```rust
pub fn add_level_beside(state: &mut AppState, level: LevelId, above: bool) -> Option<LevelId>
```

Add a level as a new storey above (or below) `level`, with its plans; the
levels beyond it move away, so the order of the levels stays.

<a id="cc_app-actions-delete_level"></a>

### fn `delete_level`

```rust
pub fn delete_level(state: &mut AppState, level: LevelId)
```

Delete a level and its plan views; refused while elements use it.

<a id="cc_app-actions-open_default_3d"></a>

### fn `open_default_3d`

```rust
pub fn open_default_3d(state: &mut AppState)
```

Open the project's first 3D view, creating one when there is none.

<a id="cc_app-actions-activate_category_tool"></a>

### fn `activate_category_tool`

```rust
pub fn activate_category_tool(state: &mut AppState, category: Category)
```

<a id="cc_app-actions-new_project"></a>

### fn `new_project`

```rust
pub fn new_project() -> Project
```

A new project from the template, named in the active UI language.

<a id="cc_app-actions-set_language"></a>

### fn `set_language`

```rust
pub fn set_language(state: &mut AppState, lang: crate::i18n::Language)
```

Switch the UI language and refresh text baked into cached drawings.

<a id="cc_app-actions-guarded"></a>

### fn `guarded`

```rust
pub fn guarded(state: &mut AppState, action: PendingAction)
```

Run `action` now, or ask for confirmation first if there are unsaved changes.

<a id="cc_app-actions-run_pending"></a>

### fn `run_pending`

```rust
pub fn run_pending(state: &mut AppState, action: PendingAction)
```

<a id="cc_app-actions-open"></a>

### fn `open`

```rust
pub fn open(state: &mut AppState)
```

<a id="cc_app-actions-open_path"></a>

### fn `open_path`

```rust
pub fn open_path(state: &mut AppState, path: std::path::PathBuf)
```

Open the project at `path`, read on a worker thread behind the progress dialog.

<a id="cc_app-actions-save_component"></a>

### fn `save_component`

```rust
pub fn save_component(state: &mut AppState, name: &str)
```

Save the component `name` (all its types) to a `.ccomp` file.

<a id="cc_app-actions-load_component"></a>

### fn `load_component`

```rust
pub fn load_component(state: &mut AppState)
```

Load a `.ccomp` file: its types (and missing materials) join the project
in one undo step, and the component tool picks the first type.

<a id="cc_app-actions-load_component_path"></a>

### fn `load_component_path`

```rust
pub fn load_component_path(state: &mut AppState, path: std::path::PathBuf)
```

Load the component file at `path` (see `load_component`).

<a id="cc_app-actions-import_skp"></a>

### fn `import_skp`

```rust
pub fn import_skp(state: &mut AppState)
```

Insert a SketchUp model into the project, on the active view's level.

<a id="cc_app-actions-export_ifc"></a>

### fn `export_ifc`

```rust
pub fn export_ifc(state: &mut AppState)
```

<a id="cc_app-actions-sheetformat"></a>

### enum `SheetFormat`

```rust
pub enum SheetFormat
```

Vector formats a sheet can be exported to.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Variants**

- `Pdf`
- `Svg`

<a id="cc_app-actions-active_sheet"></a>

### fn `active_sheet`

```rust
pub fn active_sheet(state: &AppState) -> Option<ViewId>
```

The active view when it is a sheet.

<a id="cc_app-actions-export_sheet"></a>

### fn `export_sheet`

```rust
pub fn export_sheet(state: &mut AppState, format: SheetFormat)
```

Export the active sheet as PDF or SVG.

<a id="cc_app-actions-image_dpi"></a>

### const `IMAGE_DPI`

```rust
pub const IMAGE_DPI: f64
```

Resolution of exported images, dots per inch.

<a id="cc_app-actions-image_view"></a>

### fn `image_view`

```rust
pub fn image_view(state: &AppState) -> Option<ViewId>
```

The active view, to export as an image or DXF: every view has a drawing
(a schedule draws as its table).

<a id="cc_app-actions-export_image"></a>

### fn `export_image`

```rust
pub fn export_image(state: &mut AppState)
```

Export the active view as a PNG image, as printed (no temporary hiding).

<a id="cc_app-actions-export_dxf"></a>

### fn `export_dxf`

```rust
pub fn export_dxf(state: &mut AppState)
```

Export the active view as DXF, for other CAD programs: at full size in
millimeters (a sheet in paper millimeters), as printed.

<a id="cc_app-actions-toggle_macro_recording"></a>

### fn `toggle_macro_recording`

```rust
pub fn toggle_macro_recording(state: &mut AppState)
```

Start recording a macro, or stop and save it as a script of commands
(the format `ccad script` runs).

<a id="cc_app-actions-run_macro"></a>

### fn `run_macro`

```rust
pub fn run_macro(state: &mut AppState)
```

Run a macro (a script of commands) on the open project; each step is one undo step.

<a id="cc_app-actions-run_macro_file"></a>

### fn `run_macro_file`

```rust
pub fn run_macro_file(state: &mut AppState, path: &std::path::Path)
```

Run the macro at `path` (see `run_macro`).

<a id="cc_app-actions-save"></a>

### fn `save`

```rust
pub fn save(state: &mut AppState, save_as: bool) -> bool
```

<a id="cc_app-actions-select_instances"></a>

### fn `select_instances`

```rust
pub fn select_instances(state: &mut AppState)
```

Select every instance of the selected elements' types shown in the
active view (the engine's `elements_list` by type and view).

<a id="cc_app-actions-group_selection"></a>

### fn `group_selection`

```rust
pub fn group_selection(state: &mut AppState)
```

Group the selection (the engine's `group_create`).

<a id="cc_app-actions-ungroup_selection"></a>

### fn `ungroup_selection`

```rust
pub fn ungroup_selection(state: &mut AppState)
```

Ungroup every group the selection touches (the engine's `group_ungroup`).

<a id="cc_app-actions-grids_for"></a>

### fn `grids_for`

```rust
pub fn grids_for(state: &AppState) -> Vec<u64>
```

Ids of the selected grids, or of every grid when none is selected.

<a id="cc_app-actions-reference_grid"></a>

### fn `reference_grid`

```rust
pub fn reference_grid(state: &AppState) -> Option<u64>
```

The single selected grid, the reference of `align_grid_ends`.

<a id="cc_app-actions-align_grid_ends"></a>

### fn `align_grid_ends`

```rust
pub fn align_grid_ends(state: &mut AppState)
```

Give every grid parallel to the selected one its extents, so their
bubbles line up (the engine's `grids_align_ends`).

<a id="cc_app-actions-selected_columns"></a>

### fn `selected_columns`

```rust
pub fn selected_columns(state: &AppState) -> Vec<u64>
```

The selected columns as a chain: from the lowest-left one, each followed by the nearest left.

<a id="cc_app-actions-columns_at_grids"></a>

### fn `columns_at_grids`

```rust
pub fn columns_at_grids(state: &mut AppState)
```

Columns at the intersections of the selected grids (the engine's `columns_at_grids`).

<a id="cc_app-actions-beams_on_grids"></a>

### fn `beams_on_grids`

```rust
pub fn beams_on_grids(state: &mut AppState)
```

Beams between the columns along the selected grids (the engine's `beams_on_grids`).

<a id="cc_app-actions-footings_under_columns"></a>

### fn `footings_under_columns`

```rust
pub fn footings_under_columns(state: &mut AppState)
```

Pad footings under the selected columns, or every column of the active level.

<a id="cc_app-actions-strip_footings_under_walls"></a>

### fn `strip_footings_under_walls`

```rust
pub fn strip_footings_under_walls(state: &mut AppState)
```

Strip footings under the selected walls, or every wall of the active level.

<a id="cc_app-actions-slab_join_supports"></a>

### fn `slab_join_supports`

```rust
pub fn slab_join_supports(state: &mut AppState)
```

Join the selected floors (or those of the active level) with the beams, columns and walls carrying them.

<a id="cc_app-actions-open_analytical_view"></a>

### fn `open_analytical_view`

```rust
pub fn open_analytical_view(state: &mut AppState)
```

Open the analytical model view, creating it the first time.

<a id="cc_app-actions-export_analytical"></a>

### fn `export_analytical`

```rust
pub fn export_analytical(state: &mut AppState)
```

Export the analytical model as JSON.

<a id="cc_app-actions-typical_rebar"></a>

### fn `typical_rebar`

```rust
pub fn typical_rebar(state: &mut AppState)
```

X bracing between the two selected columns.
Typical reinforcement for the selected beams and columns, or all of the active level.

<a id="cc_app-actions-open_rebar_view"></a>

### fn `open_rebar_view`

```rust
pub fn open_rebar_view(state: &mut AppState)
```

Open the first 3D view showing reinforcement, creating one the first time.

<a id="cc_app-actions-open_structural_plan"></a>

### fn `open_structural_plan`

```rust
pub fn open_structural_plan(state: &mut AppState)
```

Open the structural plan of the active level (or the lowest), creating it the first time.

<a id="cc_app-actions-mep_connect"></a>

### fn `mep_connect`

```rust
pub fn mep_connect(state: &mut AppState)
```

Connect the selected fixtures and devices to the nearest pipes and conduits (`mep_connect`).

<a id="cc_app-actions-open_panel_diagram"></a>

### fn `open_panel_diagram`

```rust
pub fn open_panel_diagram(state: &mut AppState)
```

Open the diagram of the selected distribution board (or the first), creating it the first time.

<a id="cc_app-actions-open_mep_plan"></a>

### fn `open_mep_plan`

```rust
pub fn open_mep_plan(state: &mut AppState, discipline: cc_core::MepDiscipline)
```

Open the plumbing, electrical, communications or lightning protection plan of the active level (or the lowest), creating it the first time.

<a id="cc_app-actions-number_structure"></a>

### fn `number_structure`

```rust
pub fn number_structure(state: &mut AppState)
```

Number the beams, columns and footings of the active level (or the whole project).

<a id="cc_app-actions-active_floor_plan"></a>

### fn `active_floor_plan`

```rust
pub fn active_floor_plan(state: &AppState) -> Option<(ViewId, bool)>
```

The active view when it is a floor plan, with whether it shows structure tags.

<a id="cc_app-actions-toggle_structure_tags"></a>

### fn `toggle_structure_tags`

```rust
pub fn toggle_structure_tags(state: &mut AppState)
```

Show or hide the beam and column tags of the active floor plan.

<a id="cc_app-actions-beams_between_columns"></a>

### fn `beams_between_columns`

```rust
pub fn beams_between_columns(state: &mut AppState)
```

Beams through the selected columns (the engine's `beams_between_columns`).

<a id="cc_app-actions-hide_in_view"></a>

### fn `hide_in_view`

```rust
pub fn hide_in_view(state: &mut AppState)
```

Hide the selection in the active view (permanently; Visibility/Graphics shows it again).

<a id="cc_app-actions-temporary"></a>

### fn `temporary`

```rust
pub fn temporary(state: &mut AppState, action: &str)
```

Temporarily hide or isolate the selection (or its categories) in the active view, or reset.

<a id="cc_app-actions-temporary_permanent"></a>

### fn `temporary_permanent`

```rust
pub fn temporary_permanent(state: &mut AppState)
```

Make the active view's temporary hide/isolate permanent.

<a id="cc_app-actions-selected_masses"></a>

### fn `selected_masses`

```rust
pub fn selected_masses(state: &AppState) -> Vec<ElementId>
```

The masses in the selection.

<a id="cc_app-actions-from_masses"></a>

### fn `from_masses`

```rust
pub fn from_masses(state: &mut AppState, what: &str)
```

Walls (`walls`), floors (`slabs`) or roofs (`roof`) from each selected
mass; what they make is selected.

<a id="cc_app-actions-terrain_on_level"></a>

### fn `terrain_on_level`

```rust
pub fn terrain_on_level(state: &AppState) -> bool
```

Whether the active plan's level has a terrain (for building pads to grade).

<a id="mod-cc_app-app"></a>

## `cc_app::app`

<sub>`crates/cc_app/src/app.rs`</sub>

Top-level eframe application: layout, dock and keyboard shortcuts.

**Items:** [`Args`](#cc_app-app-args) · [`ConstructApp`](#cc_app-app-constructapp) · [`zoom_fit`](#cc_app-app-zoom_fit) · [`shortcuts`](#cc_app-app-shortcuts)

<a id="cc_app-app-args"></a>

### struct `Args`

```rust
pub struct Args
```

Command-line options.

**Traits:** `Debug`, `Default`, `Clone`

**Fields**

- `path: Option<std::path::PathBuf>`
- `views: Vec<String>`

  Views (by name or tab title) to open at startup.

- `language: Option<Language>`
- `graph: Option<std::path::PathBuf>`

  A visual programming graph to open in the graph editor.


<a id="cc_app-app-constructapp"></a>

### struct `ConstructApp`

```rust
pub struct ConstructApp
```

**Traits:** `eframe::App`

*Some fields are private.*

**Methods**

- `pub fn new(cc: &eframe::CreationContext<'_>, args: Args) -> Self`

  Create the app; see `Args` for the startup options.


<a id="cc_app-app-zoom_fit"></a>

### fn `zoom_fit`

```rust
pub fn zoom_fit(state: &mut AppState)
```

Zoom the active view to fit its contents.

<a id="cc_app-app-shortcuts"></a>

### fn `shortcuts`

```rust
pub(crate) fn shortcuts(ctx: &egui::Context, state: &mut AppState)
```

The keyboard shortcuts of the app: Ctrl ones, plain keys and two-letter tool keys.

<a id="mod-cc_app-bracing_ui"></a>

## `cc_app::bracing_ui`

<sub>`crates/cc_app/src/bracing_ui.rs`</sub>

Bracing in the app: the Bracing command (the bay between two selected
columns, else the two-click tool), the tool's options bar and the
properties of a selected bracing (every parameter through `element_set`,
which regenerates its braces in the same undo step).

**Items:** [`between_columns`](#cc_app-bracing_ui-between_columns) · [`start`](#cc_app-bracing_ui-start) · [`tool_options`](#cc_app-bracing_ui-tool_options) · [`edit`](#cc_app-bracing_ui-edit) · [`props`](#cc_app-bracing_ui-props)

<a id="cc_app-bracing_ui-between_columns"></a>

### fn `between_columns`

```rust
pub fn between_columns(state: &AppState, columns: Vec<u64>, material: StructuralMaterial) -> BracingCreate
```

The command for the bay between two columns, with the tool's options
(the storey height comes from the columns).

<a id="cc_app-bracing_ui-start"></a>

### fn `start`

```rust
pub fn start(state: &mut AppState)
```

The Bracing command: with two columns selected it braces their bay,
otherwise it starts the tool (two clicks: the bay's column axes).

<a id="cc_app-bracing_ui-tool_options"></a>

### fn `tool_options`

```rust
pub fn tool_options(ui: &mut Ui, state: &mut AppState)
```

The options bar of the Bracing tool.

<a id="cc_app-bracing_ui-edit"></a>

### fn `edit`

```rust
pub fn edit(state: &mut AppState, id: ElementId, field: &str, f: impl FnOnce(&mut Bracing))
```

Change bracing `id` as `f` changes a copy of it (`element_set`, which
regenerates its braces); edits under one `field` merge while dragging.

<a id="cc_app-bracing_ui-props"></a>

### fn `props`

```rust
pub fn props(ui: &mut Ui, state: &mut AppState, id: ElementId, bracing: &Bracing)
```

The rows of a selected bracing.

<a id="mod-cc_app-browser"></a>

## `cc_app::browser`

<sub>`crates/cc_app/src/browser.rs`</sub>

Project Browser: the tree of views, schedules and sheets, grouped as the
project's browser organization says (`cc_core::browser`): by view type
alone, or by the criteria of the scheme in use.

**Items:** [`ViewDrag`](#cc_app-browser-viewdrag) · [`category_icon`](#cc_app-browser-category_icon) · [`body`](#cc_app-browser-body)

<a id="cc_app-browser-viewdrag"></a>

### struct `ViewDrag`

```rust
pub struct ViewDrag(pub ViewId)
```

A view dragged from the browser (dropped on a sheet, it is placed there).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

<a id="cc_app-browser-category_icon"></a>

### fn `category_icon`

```rust
pub fn category_icon(category: ViewCategory) -> Icon
```

<a id="cc_app-browser-body"></a>

### fn `body`

```rust
pub fn body(ui: &mut Ui, state: &mut AppState)
```

The browser's tree, under a header drawn by the caller.

<a id="mod-cc_app-browser_org_dialog"></a>

## `cc_app::browser_org_dialog`

<sub>`crates/cc_app/src/browser_org_dialog.rs`</sub>

The Browser Organization dialog (*View → Browser Organization*, also in
the menu of the project browser's groups): the schemes that group the
browser's views. The marked scheme is the one in use, and the browser
follows it at once; *Default* lists the views by type alone. A project
scheme has a name and the criteria it nests, outermost first. Every
change runs a `browser_scheme_*` command; OK keeps them, Cancel undoes
them all (`dialog::session`, through `crate::graphics_dialog::Edits`).

**Items:** [`BrowserOrgDialog`](#cc_app-browser_org_dialog-browserorgdialog) · [`open`](#cc_app-browser_org_dialog-open) · [`show`](#cc_app-browser_org_dialog-show)

<a id="cc_app-browser_org_dialog-browserorgdialog"></a>

### struct `BrowserOrgDialog`

```rust
pub struct BrowserOrgDialog
```

The open Browser Organization dialog.

*Some fields are private.*

**Methods**

- `pub fn edits_finish(self, state: &mut AppState, keep: bool)`

  Close the dialog's edits: keep them (OK) or undo them (Cancel).


<a id="cc_app-browser_org_dialog-open"></a>

### fn `open`

```rust
pub fn open(state: &mut AppState)
```

<a id="cc_app-browser_org_dialog-show"></a>

### fn `show`

```rust
pub fn show(ctx: &Context, state: &mut AppState)
```

<a id="mod-cc_app-cad_ui"></a>

## `cc_app::cad_ui`

<sub>`crates/cc_app/src/cad_ui.rs`</sub>

Imported CAD drawings in the app: the Import CAD and Import Image
commands (file dialog or files dropped on the window), and their
properties and layers, and Explode. Every change runs an engine command
(`cad_import`, `image_import`, `element_set`, `cad_layers_set`,
`cad_explode`).

**Items:** [`EXTENSIONS`](#cc_app-cad_ui-extensions) · [`cad_view`](#cc_app-cad_ui-cad_view) · [`selected_import`](#cc_app-cad_ui-selected_import) · [`explode_selected`](#cc_app-cad_ui-explode_selected) · [`import_cad`](#cc_app-cad_ui-import_cad) · [`import_cad_path`](#cc_app-cad_ui-import_cad_path) · [`import_image`](#cc_app-cad_ui-import_image) · [`import_image_path`](#cc_app-cad_ui-import_image_path) · [`dropped_files`](#cc_app-cad_ui-dropped_files) · [`props`](#cc_app-cad_ui-props)

<a id="cc_app-cad_ui-extensions"></a>

### const `EXTENSIONS`

```rust
pub const EXTENSIONS: [&str; 2]
```

File extensions read as CAD drawings.

<a id="cc_app-cad_ui-cad_view"></a>

### fn `cad_view`

```rust
pub fn cad_view(state: &AppState) -> Option<ViewId>
```

The active view when it can hold an imported drawing.

<a id="cc_app-cad_ui-selected_import"></a>

### fn `selected_import`

```rust
pub fn selected_import(state: &AppState) -> Option<ElementId>
```

The imported drawing selected alone, if any.

<a id="cc_app-cad_ui-explode_selected"></a>

### fn `explode_selected`

```rust
pub fn explode_selected(state: &mut AppState)
```

Explode the selected imported drawing into detail lines, filled regions
and text notes of its view (`cad_explode`), selecting what it made.

<a id="cc_app-cad_ui-import_cad"></a>

### fn `import_cad`

```rust
pub fn import_cad(state: &mut AppState)
```

Pick a DWG or DXF file and import it into the active view.

<a id="cc_app-cad_ui-import_cad_path"></a>

### fn `import_cad_path`

```rust
pub fn import_cad_path(state: &mut AppState, path: PathBuf)
```

Import the drawing at `path` into the active view, select it and report
what was read.

<a id="cc_app-cad_ui-import_image"></a>

### fn `import_image`

```rust
pub fn import_image(state: &mut AppState)
```

Pick a picture or an SVG file and import it into the active view.

<a id="cc_app-cad_ui-import_image_path"></a>

### fn `import_image_path`

```rust
pub fn import_image_path(state: &mut AppState, path: PathBuf)
```

Import the picture or SVG at `path` into the active view, centered on
what it shows, and select it.

<a id="cc_app-cad_ui-dropped_files"></a>

### fn `dropped_files`

```rust
pub fn dropped_files(ctx: &egui::Context, state: &mut AppState)
```

DWG and DXF drawings, pictures and SVG files dropped on the window go into the active view.

<a id="cc_app-cad_ui-props"></a>

### fn `props`

```rust
pub fn props(ui: &mut Ui, state: &mut AppState, id: ElementId, c: &CadImport)
```

Properties of an imported drawing: file, placement, colors and layers.

<a id="mod-cc_app-callout_ui"></a>

## `cc_app::callout_ui`

<sub>`crates/cc_app/src/callout_ui.rs`</sub>

Callouts in the app: the Callout tool draws a rectangle in a plan,
ceiling plan, section or elevation and makes a detail view of that
region (`callout_create`); the Select tool picks a callout's mark, moves
it and resizes it by its corners (the detail view's crop, `view_set`)
and opens its view on a double-click; the view's properties show the
callout's parent and open the view.

**Items:** [`CalloutOptions`](#cc_app-callout_ui-calloutoptions) · [`Part`](#cc_app-callout_ui-part) · [`tool`](#cc_app-callout_ui-tool) · [`options`](#cc_app-callout_ui-options) · [`select`](#cc_app-callout_ui-select) · [`view_rows`](#cc_app-callout_ui-view_rows)

<a id="cc_app-callout_ui-calloutoptions"></a>

### struct `CalloutOptions`

```rust
pub struct CalloutOptions
```

Options of the Callout tool.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`

**Fields**

- `scale: u32`

  Scale denominator of the new detail view (`0`: about five times the parent's).

- `drafting: Option<ViewId>`

  A drafting view the callout points to instead of a new detail view.


<a id="cc_app-callout_ui-part"></a>

### enum `Part`

```rust
pub enum Part
```

What a drag of a callout mark holds.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Variants**

- `Body`

  The whole mark, moved.

- `Corner(usize)`

  A corner of the rectangle (as `CropBox::corners` counts them).


<a id="cc_app-callout_ui-tool"></a>

### fn `tool`

```rust
pub fn tool(_ui: &Ui, resp: &Response, painter: &Painter, state: &mut AppState, view: &View, xf: &Xf)
```

The Callout tool: two clicks give opposite corners of the rectangle.

<a id="cc_app-callout_ui-options"></a>

### fn `options`

```rust
pub fn options(ui: &mut Ui, state: &mut AppState)
```

The Callout tool's options: the detail view's scale, or a drafting view to point to.

<a id="cc_app-callout_ui-select"></a>

### fn `select`

```rust
pub fn select(ui: &Ui, resp: &Response, painter: &Painter, state: &mut AppState, xf: &Xf, drawing: &Drawing2D) -> bool
```

Callout marks under the Select tool. Returns `true` when they took the
pointer this frame (the element selection is left alone).

<a id="cc_app-callout_ui-view_rows"></a>

### fn `view_rows`

```rust
pub fn view_rows(ui: &mut Ui, state: &mut AppState, id: ViewId)
```

The callout and crop rows of a view's properties: the view its callout
is drawn in, opening the view, removing the callout or the crop.

<a id="mod-cc_app-canvas"></a>

## `cc_app::canvas`

<sub>`crates/cc_app/src/canvas.rs`</sub>

2D view canvas: plans, ceilings, elevations, sections, legends and drafting views.

**Items:** [`ZoomCommand`](#cc_app-canvas-zoomcommand) · [`ZoomNav`](#cc_app-canvas-zoomnav) · [`View2DState`](#cc_app-canvas-view2dstate) · [`zoom`](#cc_app-canvas-zoom) · [`show`](#cc_app-canvas-show) · [`temporary_frame`](#cc_app-canvas-temporary_frame) · [`view_toolbar`](#cc_app-canvas-view_toolbar)

<a id="cc_app-canvas-zoomcommand"></a>

### enum `ZoomCommand`

```rust
pub enum ZoomCommand
```

Zoom commands of the view toolbar menu.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Default`

**Variants**

- `Region`

  Drag a window to zoom into.

- `OutTwo`

  Zoom out by half around the view center.

- `Fit`
- `FitAll`

  Zoom every open view to fit.

- `SheetSize`

  One paper millimeter shows as one screen millimeter.

- `Previous`
- `Next`

**Methods**

- `pub const ALL: [ZoomCommand; 7]`
- `pub fn key(self) -> &'static str`

<a id="cc_app-canvas-zoomnav"></a>

### struct `ZoomNav`

```rust
pub struct ZoomNav
```

What the zoom menu can offer in a view.

**Traits:** `Debug`, `Clone`, `Copy`, `Default`

**Fields**

- `back: bool`
- `forward: bool`
- `sheet_size: bool`

  The view has a paper scale ("zoom sheet size").


<a id="cc_app-canvas-view2dstate"></a>

### struct `View2DState`

```rust
pub struct View2DState
```

Pan/zoom state of a 2D view.

**Traits:** `Debug`, `Clone`, `Default`

**Fields**

- `center: DVec2`

  Drawing coordinates shown at the canvas center.

- `ppm: f64`

  Screen points per drawing unit.

- `fitted: bool`

  Whether the view has been zoomed to fit at least once (or after "zoom extents").

- `zoom_region: bool`

  "Zoom in region" waits for a window; the drag's start once it began.


*Some fields are private.*

**Methods**

- `pub fn xf(&self, rect: Rect) -> Xf`
- `pub fn nav(&self) -> ZoomNav`
- `pub fn apply(&mut self, cmd: ZoomCommand, mm_scale: f64)`

  Run a zoom command; `mm_scale` is the drawing's paper millimeters
  per drawing unit (sheets: 1; plans at 1:100: 10).

- `pub fn request_fit(&mut self)`

  Zoom to fit on the next frame (fitting needs the drawing's bounds),
  remembering the current pan/zoom.

- `pub fn start_empty(&mut self, scale: u32, drafting: bool)`

  Where a view with nothing to fit starts. A view of the model shows
  the first meters of a building; a drafting view, drawn at its own
  scale, shows as much paper as a 1:100 plan does at that zoom, so what
  is annotated in it can be read (issue #318: at the zoom of a building
  the text of a 1:10 detail is a pixel tall).

- `pub fn fit(&mut self, (lo, hi): (DVec2, DVec2), rect: Rect)`
- `pub fn needs_fit(&self, rect: Rect) -> bool`

  Whether to zoom to fit now: the view was never fitted, or it was left
  as fitted and its canvas changed size.

- `pub fn navigate(&mut self, ui: &Ui, resp: &Response, rect: Rect)`

  Wheel or pinch zoom around the cursor, two-finger (trackpad) and
  middle-button pan (see `crate::gestures`).

- `pub fn region(&mut self, ui: &Ui, resp: &Response, painter: &egui::Painter, rect: Rect) -> bool`

  "Zoom in region": drag a window and zoom to it; a click or Esc cancels.
  Returns true while it takes the pointer (tools wait meanwhile).


<a id="cc_app-canvas-zoom"></a>

### fn `zoom`

```rust
pub fn zoom(state: &mut AppState, view: ViewId, cmd: ZoomCommand, mm_scale: f64)
```

Run a toolbar zoom command on `view` (or, for "fit all", on every view).

<a id="cc_app-canvas-show"></a>

### fn `show`

```rust
pub fn show(ui: &mut Ui, state: &mut AppState, view_id: ViewId)
```

Show a graphical 2D view and run the active tool on it.

<a id="cc_app-canvas-temporary_frame"></a>

### fn `temporary_frame`

```rust
pub fn temporary_frame(ui: &mut Ui, state: &mut AppState, rect: Rect, view_id: ViewId)
```

While something is temporarily hidden or isolated in the view: a colored
border and a chip naming the state, with Reset and Make Permanent.

<a id="cc_app-canvas-view_toolbar"></a>

### fn `view_toolbar`

```rust
pub fn view_toolbar(ui: &mut Ui, default: ZoomCommand, rect: Rect, view_id: ViewId, scale: Option<u32>, nav: ZoomNav) -> Option<ZoomCommand>
```

Floating chip in the view corner with the scale (when given) and the
zoom button: a click runs the default zoom command, the arrow opens the
menu of all of them. Returns the command to run.

<a id="mod-cc_app-clipboard"></a>

## `cc_app::clipboard`

<sub>`crates/cc_app/src/clipboard.rs`</sub>

Clipboard: cut, copy and paste of elements within the open project, and
match type (pick an element, then click others to give them its type).
The clipboard holds a snapshot of the copied elements (with the doors and
windows of copied walls), so a cut can be pasted after its originals are
gone. It lives in the app state and is dropped with the document, since
its type and level ids only mean something in that project.

**Items:** [`Clipboard`](#cc_app-clipboard-clipboard) · [`copy`](#cc_app-clipboard-copy) · [`cut`](#cc_app-clipboard-cut) · [`paste`](#cc_app-clipboard-paste) · [`paste_aligned`](#cc_app-clipboard-paste_aligned) · [`paste_tool`](#cc_app-clipboard-paste_tool) · [`start_match_type`](#cc_app-clipboard-start_match_type) · [`match_type_tool`](#cc_app-clipboard-match_type_tool)

<a id="cc_app-clipboard-clipboard"></a>

### struct `Clipboard`

```rust
pub struct Clipboard
```

Copied elements and, while pasting, their preview in the active view.

**Traits:** `Default`

*Some fields are private.*

**Methods**

- `pub fn is_empty(&self) -> bool`

<a id="cc_app-clipboard-copy"></a>

### fn `copy`

```rust
pub fn copy(ctx: &egui::Context, state: &mut AppState) -> bool
```

Copy the selection to the clipboard. The system clipboard gets a short
note, which also makes Ctrl+V reach the app (egui only reports a paste
when the system clipboard holds text).

<a id="cc_app-clipboard-cut"></a>

### fn `cut`

```rust
pub fn cut(ctx: &egui::Context, state: &mut AppState)
```

Copy the selection to the clipboard and delete it (pinned elements stay).

<a id="cc_app-clipboard-paste"></a>

### fn `paste`

```rust
pub fn paste(state: &mut AppState)
```

Ctrl+V: place the clipboard with a click in a plan view; elsewhere paste
it where it was copied. What was drawn in a view (detail lines, regions,
notes, dimensions…) goes into the active view when it is a 2D view that
holds such elements, else back into its own view.

<a id="cc_app-clipboard-paste_aligned"></a>

### fn `paste_aligned`

```rust
pub fn paste_aligned(state: &mut AppState, current_level: bool)
```

Paste where the elements were copied, on their own levels or (with
`current_level`) on the active plan's level.

<a id="cc_app-clipboard-paste_tool"></a>

### fn `paste_tool`

```rust
pub fn paste_tool(ui: &Ui, resp: &Response, painter: &Painter, state: &mut AppState, view: ViewId, level: LevelId, xf: &Xf)
```

Paste tool: the clipboard follows the cursor (its center on the snapped
point) and a click places it on the view's level.

<a id="cc_app-clipboard-start_match_type"></a>

### fn `start_match_type`

```rust
pub fn start_match_type(state: &mut AppState)
```

Start match type: with one element selected it is the source, otherwise
the first click picks it.

<a id="cc_app-clipboard-match_type_tool"></a>

### fn `match_type_tool`

```rust
pub fn match_type_tool(resp: &Response, painter: &Painter, state: &mut AppState, xf: &Xf, drawing: &Drawing2D)
```

Match type tool: the first click picks the source, later clicks give its
type to elements of the same category (a click on empty space picks again).

<a id="mod-cc_app-commands"></a>

## `cc_app::commands`

<sub>`crates/cc_app/src/commands.rs`</sub>

Interface commands: every ribbon, quick access and palette entry is a
`UiCommand`, so its label, shortcut, description, the engine command it
runs and whether it is available now (and why not) are defined once and
read by every place that shows it.

**Items:** [`set_key_overrides`](#cc_app-commands-set_key_overrides) · [`is_letter_keys`](#cc_app-commands-is_letter_keys) · [`UiCommand`](#cc_app-commands-uicommand) · [`DEVICE_TOOLS`](#cc_app-commands-device_tools) · [`phase_view`](#cc_app-commands-phase_view) · [`graphics_view`](#cc_app-commands-graphics_view) · [`invoke`](#cc_app-commands-invoke) · [`run`](#cc_app-commands-run)

<a id="cc_app-commands-set_key_overrides"></a>

### fn `set_key_overrides`

```rust
pub fn set_key_overrides(keys: &std::collections::BTreeMap<String, String>)
```

Use `keys` (command id → two letters, or empty) as the custom shortcuts.

<a id="cc_app-commands-is_letter_keys"></a>

### fn `is_letter_keys`

```rust
pub fn is_letter_keys(keys: &str) -> bool
```

Two ASCII capital letters, the form of the keys typed in a view.

<a id="cc_app-commands-uicommand"></a>

### enum `UiCommand`

```rust
pub enum UiCommand
```

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`

**Variants**

- `Home`
- `New`
- `Open`
- `Save`
- `SaveAs`
- `CloseProject`
- `ImportIfc`
- `ImportCad`

  Import a DWG or DXF drawing into the active view.

- `ImportImage`

  Import a picture or an SVG drawing into the active view.

- `ImportSkp`

  Insert a SketchUp model into the open project.

- `ImportObj`

  Insert a Wavefront OBJ model into the open project.

- `ImportGltf`

  Insert a glTF model (.glb, .gltf) into the open project.

- `ExportIfc`
- `ExportPdf`
- `ExportSvg`
- `ExportImage`
- `ExportDxf`
- `About`
- `CheckUpdates`
- `Preferences`
- `Undo`
- `Redo`
- `Default3D`
- `Place(Category)`

  The placement tool of a model category.

- `StructurePlace(Category, cc_core::StructuralMaterial)`

  Start the column or beam tool on a type of the given structural material.

- `NewComponent`
- `LoadComponent`
- `ComponentLibrary`
- `EditComponent`

  Open the component modeling environment on the selected component.

- `CompExtrusion`

  Environment tools and actions.

- `CompRevolution`
- `CompSweep`
- `CompBlend`
- `CompRefPlane`
- `CompDimension`
- `CompEqual`
- `CompConnector`
- `CompSaveFile`
- `CompFinish`
- `CompCancel`
- `SketchLine`

  Sketch mode: drawing tools, picking the walls, finish and cancel.

- `SketchRect`
- `SketchPolygon`
- `SketchPolygonOut`
- `SketchCircle`
- `SketchEllipse`
- `SketchSpline`
- `SketchArc3`
- `SketchArcCenter`
- `SketchArcTangent`
- `SketchPoint`

  A measured point of a terrain (its elevation from the options bar).

- `SketchPickWalls`
- `SketchFinish`
- `SketchCancel`
- `EditBoundary`

  Open the sketch of the selected floor, ceiling, roof or mass.

- `EditProfile`

  Open the profile of the selected sweep's type in a sketch.

- `EditTitleBlock`

  Open the active sheet's title block in the editor.

- `NewTitleBlock`

  Create a title block from the standard layout and open it.

- `TbLines`

  Sketch lines and arcs of a title block.

- `TbText`

  Place a text on a title block.

- `TbImage`

  Place a picture on a title block.

- `Select`
- `SelectInstances`
- `ColumnsAtGrids`
- `GridFrame`

  Opens the Grid Frame dialog.

- `SteelBuilding`
- `LightSteelFrame`

  Opens the Wall Framing dialog for light steel.

- `WoodFrame`

  Opens the Wall Framing dialog for wood.

- `AlignGridEnds`
- `BeamsOnGrids`
- `BeamsBetweenColumns`
- `StructuralSchedules`
- `FootingsUnderColumns`
- `StripFootingsUnderWalls`
- `FoundationSlab`
- `SlabJoinSupports`
- `AnalyticalView`
- `AnalyticalExport`
- `Truss`
- `Bracing`

  Brace the bay between two selected columns, else start the Bracing tool.

- `TypicalRebar`
- `Connection(cc_core::connection::ConnectionKind)`

  Place connections of a kind on the selected members, else the active level's.

- `RebarView`
- `SteelSchedule`
- `StructuralPlan`
- `NumberStructure`
- `StructureTags`
- `PlaceFixture`
- `PlaceDevice`
- `Systems`
- `PlumbingPlan`
- `ElectricalPlan`
- `MepSchedules`
- `MepConnect`
- `MepSizing`
- `PanelDiagram`
- `PlaceDeviceOf(cc_core::DeviceKind)`

  Place a communications or lightning protection device (its built-in type made on first use).

- `EarthRing`
- `LpsCheck`
- `Cabling`
- `CommunicationsPlan`
- `LightningPlan`
- `Dimension(DimensionMode)`
- `Text`
- `LevelMarkPlan`
- `LevelMarkSection`
- `AddLeader`
- `DetailLine`
- `FilledRegion`
- `MaskingRegion`
- `LineStyles`
- `BrowserOrganization`
- `DetailComponent`
- `CreateDetailComponent`
- `Move`
- `CopyElements`
- `Rotate`
- `MirrorPick`
- `MirrorDraw`
- `Array`
- `Align`
- `PushPull`
- `MassWalls`

  Walls, floors and a roof from the selected masses.

- `MassSlabs`
- `MassRoof`
- `Offset`
- `TrimCorner`
- `TrimExtend`
- `Split`
- `ToggleJoin`
- `Measure`
- `JoinGeometry`
- `UnjoinGeometry`
- `AttachTop`
- `DetachTop`
- `Paint`

  Paint faces with a material in a 3D view.

- `RemovePaint`
- `Group`
- `Ungroup`
- `Explode`

  Explode the selected imported drawing into native 2D elements.

- `Pin`
- `Unpin`
- `Delete`
- `Paste`
- `Cut`
- `ClipboardCopy`
- `MatchType`
- `FloorPlan`
- `CeilingPlan`
- `New3D`
- `Section`
- `Callout`

  Draw a callout: a detail view of a region of the active view.

- `Elevations`
- `Legend`
- `DraftingView`
- `Schedule`
- `Sheet`
- `ZoomFit`
- `CloseInactiveViews`

  Close every view tab but the active one.

- `VisibilityGraphics`
- `ViewFilters`
- `ViewTemplates`
- `SunSettings`
- `Render`

  Path trace the active 3D view in the render window.

- `OverrideGraphics`
- `HideInView`
- `TempHide`
- `TempIsolate`
- `TempHideCategory`
- `TempIsolateCategory`
- `TempReset`
- `TempPermanent`
- `Phases`
- `DesignOptions`
- `ActiveOption`
- `Demolish`
- `CurtainGrid`
- `ProjectInfo`
- `SiteLocation`
- `Units`
- `Levels`
- `Materials`
- `FillPatterns`
- `Profiles`
- `MacroRecord`
- `MacroRun`
- `VisualProgramming`

  Open or close the visual programming editor.

- `Plugins`
- `PluginCommands`
- `ToggleRibbon`
- `CommandPalette`

**Methods**

- `pub fn all() -> Vec<UiCommand>`

  Every command, in ribbon order.

- `pub fn from_keys(keys: &str) -> Option<UiCommand>`

  The command a two-letter key sequence runs.

- `pub fn from_id(id: &str) -> Option<UiCommand>`

  The command named `id` (see `UiCommand::id`).

- `pub fn icon(self) -> Icon`
- `pub fn badge(self) -> Option<Badge>`

  The mark of the kind of action: creating, importing, exporting.

- `pub fn label(self, state: &AppState) -> String`

  Button label.

- `pub fn title(self, state: &AppState) -> String`

  Tooltip title: the label, plus what undo and redo would change.

- `pub fn description(self) -> String`

  What the command does, one or two sentences.

- `pub fn shortcut(self) -> Option<String>`

  Keyboard shortcut: a Ctrl chord or a two-letter sequence typed in a
  view; two-letter keys follow the user's choices (Preferences →
  Shortcuts).

- `pub fn id(self) -> String`

  A stable name of the command, under which its custom keys are kept.

- `pub fn customizable(self) -> bool`

  Whether the user may change the keys: two-letter ones and commands
  without keys (Ctrl chords, Delete and Esc are fixed).

- `pub fn default_shortcut(self) -> Option<&'static str>`

  The keys the command has before any change by the user.

- `pub fn engine(self) -> Option<&'static str>`

  The engine command that makes the change, as scripts and the MCP server call it.

- `pub fn unavailable(self, state: &AppState) -> Option<String>`

  Why the command cannot run now, or `None` when it can.

- `pub fn selected(self, state: &AppState) -> bool`

  Whether the command's tool or mode is the active one.


<a id="cc_app-commands-device_tools"></a>

### const `DEVICE_TOOLS`

```rust
pub const DEVICE_TOOLS: [cc_core::DeviceKind; 4]
```

Communications and lightning protection devices placed by a tool of their own.

<a id="cc_app-commands-phase_view"></a>

### fn `phase_view`

```rust
pub fn phase_view(state: &AppState) -> Option<cc_core::ViewId>
```

The active view when it is a 2D view with a phase (plans, elevations, sections).

<a id="cc_app-commands-graphics_view"></a>

### fn `graphics_view`

```rust
pub fn graphics_view(state: &AppState) -> Option<cc_core::ViewId>
```

The active view when it has visibility/graphics (plans, elevations, sections, 3D, drafting views).

<a id="cc_app-commands-invoke"></a>

### fn `invoke`

```rust
pub fn invoke(state: &mut AppState, ctx: &egui::Context, command: UiCommand)
```

Run `command` if it is available; otherwise say why in the status bar.

<a id="cc_app-commands-run"></a>

### fn `run`

```rust
pub fn run(state: &mut AppState, ctx: &egui::Context, command: UiCommand)
```

<a id="mod-cc_app-component_env"></a>

## `cc_app::component_env`

<sub>`crates/cc_app/src/component_env.rs`</sub>

The component modeling environment: a workspace of its own where one
component is shaped. It replaces the project's views with the
component's plan, front, side and 3D views, the ribbon with the *Create*
tab and the left panel with the component's parameters, types and the
selected reference plane or dimension.
Every change is an engine command on the component (`component_*`), so
it applies to every type of the component and undoes like any edit.
*Finish* keeps the changes; *Cancel* undoes everything done since the
environment opened.

**Items:** [`VIEW_SCALE`](#cc_app-component_env-view_scale) · [`Pick`](#cc_app-component_env-pick) · [`EnvTool`](#cc_app-component_env-envtool) · [`Env`](#cc_app-component_env-env) · [`open`](#cc_app-component_env-open) · [`open_on`](#cc_app-component_env-open_on) · [`open_new`](#cc_app-component_env-open_new) · [`target_type`](#cc_app-component_env-target_type) · [`save_file`](#cc_app-component_env-save_file) · [`finish`](#cc_app-component_env-finish) · [`cancel`](#cc_app-component_env-cancel) · [`component`](#cc_app-component_env-component) · [`type_ref`](#cc_app-component_env-type_ref) · [`plane_from_line`](#cc_app-component_env-plane_from_line) · [`plane_at`](#cc_app-component_env-plane_at) · [`piece_at`](#cc_app-component_env-piece_at) · [`connector_at`](#cc_app-component_env-connector_at) · [`connector_from_click`](#cc_app-component_env-connector_from_click) · [`dimension_at`](#cc_app-component_env-dimension_at) · [`ribbon`](#cc_app-component_env-ribbon) · [`options`](#cc_app-component_env-options) · [`central`](#cc_app-component_env-central) · [`set_tool`](#cc_app-component_env-set_tool) · [`escape`](#cc_app-component_env-escape) · [`delete_pick`](#cc_app-component_env-delete_pick) · [`draft_field`](#cc_app-component_env-draft_field) · [`panel`](#cc_app-component_env-panel) · [`ref_problems`](#cc_app-component_env-ref_problems) · [`types_row`](#cc_app-component_env-types_row) · [`params`](#cc_app-component_env-params) · [`piece_props`](#cc_app-component_env-piece_props)

<a id="cc_app-component_env-view_scale"></a>

### const `VIEW_SCALE`

```rust
pub(crate) const VIEW_SCALE: u32
```

Scale the views are drawn at (text and dimension sizes).

<a id="cc_app-component_env-pick"></a>

### enum `Pick`

```rust
pub enum Pick
```

What is selected in the environment.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Variants**

- `Plane(String)`
- `Dimension(usize)`
- `Piece(usize)`

  A piece, by its index in the definition.

- `Equal(usize)`

  An equality (EQ), by its index.

- `Connector(usize)`

  A connector, by its index.

- `Sheet(Vec<usize>)`

  2D pieces of a title block, by their indices in `def.sheet`.


<a id="cc_app-component_env-envtool"></a>

### enum `EnvTool`

```rust
pub enum EnvTool
```

The tool of the environment.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Variants**

- `Select`
- `RefPlane { first: Option<(ComponentViewKind, DVec2)> }`

  Two clicks along the plane's line in a view.

- `Dimension { first: Option<(ComponentViewKind, String)> }`

  Two parallel planes, then the line's place.

- `Equal { picked: Vec<String>, view: Option<ComponentViewKind> }`

  Three parallel planes: an outer one, the one kept halfway, the other.

- `Connector`

  A click in the plan, front or side view places a connector.

- `SheetText`

  A click on a title block's Sheet view places a text.

- `SheetImage { picture: std::sync::Arc<crate::title_block_env::Picture>, start: Option<DVec2> }`

  A drag on a title block's Sheet view places a picture.


<a id="cc_app-component_env-env"></a>

### struct `Env`

```rust
pub struct Env
```

**Fields**

- `type_id: ElementTypeId`

  The type the environment shows (its parameter values).

- `dock: DockState<ComponentViewKind>`
- `pick: Option<Pick>`
- `tool: EnvTool`
- `start_depth: usize`

  Undo depth when the environment opened (Cancel goes back to it).

- `created: bool`

  The component was made as the environment opened (*New Component*):
  *Finish* picks the tool that places it.

- `sheet: Option<crate::title_block_env::SheetEditor>`

  A title block shows on one Sheet view instead (`crate::title_block_env`).


*Some fields are private.*

<a id="cc_app-component_env-open"></a>

### fn `open`

```rust
pub fn open(state: &mut AppState, type_id: ElementTypeId)
```

Open the environment on the component of type `type_id` (a title block
previews the active sheet).

<a id="cc_app-component_env-open_on"></a>

### fn `open_on`

```rust
pub fn open_on(state: &mut AppState, type_id: ElementTypeId, sheet: Option<cc_core::ViewId>)
```

Open the environment on component type `type_id`; a title block
previews `sheet` (its paper and fields), or sample values on A3 paper
when there is none.

<a id="cc_app-component_env-open_new"></a>

### fn `open_new`

```rust
pub fn open_new(state: &mut AppState, type_id: ElementTypeId, start_depth: usize)
```

Open the environment on the component type `type_id` just made, at
undo depth `start_depth` before it was: *Cancel Editing* removes it.

<a id="cc_app-component_env-target_type"></a>

### fn `target_type`

```rust
pub fn target_type(state: &AppState) -> Option<ElementTypeId>
```

The type of the selected component element, or the one the component tool places.

<a id="cc_app-component_env-save_file"></a>

### fn `save_file`

```rust
pub fn save_file(state: &mut AppState)
```

Save the component as a `.ccomp` file.

<a id="cc_app-component_env-finish"></a>

### fn `finish`

```rust
pub fn finish(state: &mut AppState)
```

Leave the environment keeping the changes.

<a id="cc_app-component_env-cancel"></a>

### fn `cancel`

```rust
pub fn cancel(state: &mut AppState)
```

Leave the environment undoing every change made in it.

<a id="cc_app-component_env-component"></a>

### fn `component`

```rust
pub(crate) fn component(state: &AppState) -> Option<(String, cc_core::ComponentDef)>
```

The component's definition, name and parameter values of the shown type.

<a id="cc_app-component_env-type_ref"></a>

### fn `type_ref`

```rust
pub(crate) fn type_ref(state: &AppState) -> Ref
```

<a id="cc_app-component_env-plane_from_line"></a>

### fn `plane_from_line`

```rust
pub fn plane_from_line(kind: ComponentViewKind, a: DVec2, b: DVec2) -> Option<(PlaneAxis, f64)>
```

The plane a two-click line in view `kind` makes: the axis it is square
to and its position. Mostly vertical lines make planes of constant u.

<a id="cc_app-component_env-plane_at"></a>

### fn `plane_at`

```rust
pub fn plane_at(view: &ComponentView, p: DVec2, tol: f64) -> Option<String>
```

The plane of `view` whose line passes within `tol` (model units) of `p`.

<a id="cc_app-component_env-piece_at"></a>

### fn `piece_at`

```rust
pub fn piece_at(view: &ComponentView, p: DVec2, tol: f64) -> Option<usize>
```

The piece with an edge within `tol` of `p`.

<a id="cc_app-component_env-connector_at"></a>

### fn `connector_at`

```rust
pub fn connector_at(view: &ComponentView, p: DVec2, tol: f64) -> Option<usize>
```

The connector whose marker is within `tol` of `p`.

<a id="cc_app-component_env-connector_from_click"></a>

### fn `connector_from_click`

```rust
pub fn connector_from_click(view: &ComponentView, kind: ComponentViewKind, p: DVec2, tol: f64) -> Option<([String; 3], cc_core::component::ConnectorDirection)>
```

Where a click at `p` in view `kind` places a connector: `x`, `y` and `z`
as expressions, each coordinate of the view locked to a plane seen
edge-on within `tol` (else to the millimeter), the depth at 0; and the
way runs leave it (down in plan, back elsewhere).

<a id="cc_app-component_env-dimension_at"></a>

### fn `dimension_at`

```rust
pub fn dimension_at(view: &ComponentView, p: DVec2, tol: f64) -> Option<usize>
```

The dimension whose text is within `tol` of `p`.

<a id="cc_app-component_env-ribbon"></a>

### fn `ribbon`

```rust
pub fn ribbon(ui: &mut Ui, state: &mut AppState)
```

The *Create* tab of the ribbon.

<a id="cc_app-component_env-options"></a>

### fn `options`

```rust
pub fn options(ui: &mut Ui, state: &mut AppState)
```

The hint of the active tool, for the options bar.

<a id="cc_app-component_env-central"></a>

### fn `central`

```rust
pub fn central(ui: &mut Ui, state: &mut AppState)
```

The central area: the component's views in tabs.

<a id="cc_app-component_env-set_tool"></a>

### fn `set_tool`

```rust
pub fn set_tool(state: &mut AppState, tool: EnvTool)
```

<a id="cc_app-component_env-escape"></a>

### fn `escape`

```rust
pub fn escape(state: &mut AppState)
```

Esc steps back: the tool's first pick, the tool, the selection.

<a id="cc_app-component_env-delete_pick"></a>

### fn `delete_pick`

```rust
pub fn delete_pick(state: &mut AppState)
```

Delete removes the selected plane or dimension.

<a id="cc_app-component_env-draft_field"></a>

### fn `draft_field`

```rust
pub(crate) fn draft_field(ui: &mut Ui, state: &mut AppState, key: &str, current: &str, width: f32) -> Option<String>
```

Text field that commits on Enter or when it loses focus; `None` until then.

<a id="cc_app-component_env-panel"></a>

### fn `panel`

```rust
pub fn panel(ui: &mut Ui, state: &mut AppState)
```

The component's types, parameters and the selection's properties.

<a id="cc_app-component_env-ref_problems"></a>

### fn `ref_problems`

```rust
pub(crate) fn ref_problems(ui: &mut Ui, def: &cc_core::ComponentDef)
```

The planes and dimensions that do not resolve, in the danger color.

<a id="cc_app-component_env-types_row"></a>

### fn `types_row`

```rust
pub(crate) fn types_row(ui: &mut Ui, state: &mut AppState)
```

The type whose values the views show.

<a id="cc_app-component_env-params"></a>

### fn `params`

```rust
pub(crate) fn params(ui: &mut Ui, state: &mut AppState, def: &cc_core::ComponentDef, title_block: bool)
```

The parameters: name, value, instance and delete per row, then *Add*.
A title block's (`title_block`) numbers are plain millimeters, its texts
are typed in, and a new parameter is a number starting at 0.

<a id="cc_app-component_env-piece_props"></a>

### fn `piece_props`

```rust
pub fn piece_props(ui: &mut Ui, state: &mut AppState, def: &cc_core::ComponentDef, index: usize)
```

A piece: name, solid or void, material, shown in plans, and where a
sketched one starts and ends (or its axis and angle).

<a id="mod-cc_app-component_sketch"></a>

## `cc_app::component_sketch`

<sub>`crates/cc_app/src/component_sketch.rs`</sub>

Sketching pieces in the component modeling environment. *Extrusion*,
*Revolution*, *Sweep* and *Blend* open the same sketch as outlines in
plans (`crate::sketch_mode`), in the plan, front or side view: its
drawing and modify tools, picked curves and their grips, its own undo.
*Finish* turns the curves into the piece's profile points: an extrusion
is made at once, a revolution then asks for its axis, a sweep's curves
are its open path (the section comes from the options bar), and a blend
takes a bottom outline, then a top one. *Edit Sketch* opens a piece's
profile again.
A coordinate that lies on a reference plane seen edge-on is written as
the plane's name, so the piece follows the plane when a parameter moves
it; a point on a circle or an ellipse is written from its center. Other
numbers are rounded to the micrometer. Editing keeps the expressions of
the points that did not move.

**Items:** [`PieceKind`](#cc_app-component_sketch-piecekind) · [`PieceSketch`](#cc_app-component_sketch-piecesketch) · [`section_points`](#cc_app-component_sketch-section_points) · [`um`](#cc_app-component_sketch-um) · [`offset`](#cc_app-component_sketch-offset) · [`snap`](#cc_app-component_sketch-snap) · [`same`](#cc_app-component_sketch-same) · [`written`](#cc_app-component_sketch-written) · [`piece`](#cc_app-component_sketch-piece) · [`start`](#cc_app-component_sketch-start) · [`edit`](#cc_app-component_sketch-edit) · [`finish`](#cc_app-component_sketch-finish) · [`canvas`](#cc_app-component_sketch-canvas) · [`options`](#cc_app-component_sketch-options)

<a id="cc_app-component_sketch-piecekind"></a>

### enum `PieceKind`

```rust
pub enum PieceKind
```

What the sketch becomes.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Default`

**Variants**

- `Extrusion`
- `Revolution`

  Turned about an axis picked after the outline is finished.

- `Sweep`

  A section carried along an open path (the path is what is drawn).

- `Blend`

  A bottom outline, then a top one.


<a id="cc_app-component_sketch-piecesketch"></a>

### struct `PieceSketch`

```rust
pub struct PieceSketch
```

A piece being sketched: what it becomes and where it stands.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `kind: PieceKind`
- `void: bool`
- `view: Option<ComponentViewKind>`

  The view the sketch is drawn in (set by its first curve, or the
  edited piece's plane).

- `edit: Option<usize>`

  The piece whose sketch is edited.

- `old: Vec<ProfilePoint>`

  The edited outline's (or path's) points as they were, and a blend's top.

- `old_top: Vec<ProfilePoint>`
- `bottom: Option<Vec<SketchCurve>>`

  A blend's bottom outline, once finished.

- `axis: bool`

  The outline of a revolution is finished and waits for its axis.

- `section: (f64, f64)`

  A sweep's section: width (or diameter) and height, round or not.

- `round: bool`

**Methods**

- `pub fn new(kind: PieceKind) -> Self`

<a id="cc_app-component_sketch-section_points"></a>

### fn `section_points`

```rust
pub fn section_points(w: f64, h: f64, round: bool) -> Vec<SketchPointArg>
```

A sweep's section about the path: a `w` by `h` rectangle, or a circle of
diameter `w`, centered on it (u to the left of the path, v along the normal).

<a id="cc_app-component_sketch-um"></a>

### fn `um`

```rust
pub(crate) fn um(v: f64) -> String
```

A number rounded to the micrometer, as an expression.

<a id="cc_app-component_sketch-offset"></a>

### fn `offset`

```rust
pub fn offset(e: &str, d: f64) -> String
```

`e + d`, folded into a number when `e` is one.

<a id="cc_app-component_sketch-snap"></a>

### fn `snap`

```rust
pub fn snap(view: &ComponentView, p: DVec2, tol: f64) -> (DVec2, bool)
```

The point at `p` in `view`: on the planes whose lines pass within `tol`
(a vertical line fixes u, a horizontal one v), elsewhere rounded to the
millimeter; and whether it lies on a plane.

<a id="cc_app-component_sketch-same"></a>

### fn `same`

```rust
pub(crate) fn same(a: Segment, b: Segment) -> bool
```

Whether two segments are the same (an arc of the same radius, the same ellipse).

<a id="cc_app-component_sketch-written"></a>

### fn `written`

```rust
pub(crate) fn written(def: &cc_core::ComponentDef, view: &ComponentView, curves: &[SketchCurve], closed: bool, old: &[ProfilePoint], extra: &[(&str, f64)]) -> Result<Vec<SketchPointArg>, String>
```

The profile points that draw `curves` (closed, or an open path), or why
not; `extra` are names the old points' expressions may read (a title
block's paper).

<a id="cc_app-component_sketch-piece"></a>

### fn `piece`

```rust
pub fn piece(state: &AppState) -> Option<&PieceSketch>
```

The open sketch's piece, if it is one.

<a id="cc_app-component_sketch-start"></a>

### fn `start`

```rust
pub fn start(state: &mut AppState, kind: PieceKind)
```

Start sketching a new piece of `kind`.

<a id="cc_app-component_sketch-edit"></a>

### fn `edit`

```rust
pub fn edit(state: &mut AppState, index: usize)
```

Open the sketch of piece `index` (*Edit Sketch*).

<a id="cc_app-component_sketch-finish"></a>

### fn `finish`

```rust
pub fn finish(state: &mut AppState)
```

*Finish*: the piece from the curves, or the next step (a revolution's
axis, a blend's top); a sketch that does not make a profile stays open.

<a id="cc_app-component_sketch-canvas"></a>

### fn `canvas`

```rust
pub fn canvas(ui: &Ui, resp: &Response, painter: &Painter, state: &mut AppState, view: &ComponentView, kind: ComponentViewKind, xf: &Xf, tol: f64)
```

The piece sketch in one of the environment's views: drawn in the view
it started in, snapping to the sketch's points and the reference planes.

<a id="cc_app-component_sketch-options"></a>

### fn `options`

```rust
pub fn options(ui: &mut Ui, state: &mut AppState)
```

The piece's options in the options bar: solid or void, a sweep's
section, and what the sketch is waiting for.

<a id="mod-cc_app-connection_ui"></a>

## `cc_app::connection_ui`

<sub>`crates/cc_app/src/connection_ui.rs`</sub>

Connections in the app: the Structure tab's *Connections* buttons, which
run `connections_create` on the selected members (else the active
level's), and the rows of a selected connection, whose sizes change
through `element_set`.

**Items:** [`place`](#cc_app-connection_ui-place) · [`edit`](#cc_app-connection_ui-edit) · [`props`](#cc_app-connection_ui-props)

<a id="cc_app-connection_ui-place"></a>

### fn `place`

```rust
pub fn place(state: &mut AppState, kind: ConnectionKind)
```

Place connections of `kind` with their default sizes: on the selected
columns (base plates) or beams, else on the active level's; the result
goes to the status bar.

<a id="cc_app-connection_ui-edit"></a>

### fn `edit`

```rust
pub fn edit(state: &mut AppState, id: ElementId, field: &str, f: impl FnOnce(&mut Connection))
```

Change connection `id` as `f` changes a copy of it (`element_set`);
edits under one `field` merge while dragging.

<a id="cc_app-connection_ui-props"></a>

### fn `props`

```rust
pub fn props(ui: &mut Ui, state: &mut AppState, id: ElementId, connection: &Connection)
```

The rows of a selected connection.

<a id="mod-cc_app-curtain_ui"></a>

## `cc_app::curtain_ui`

<sub>`crates/cc_app/src/curtain_ui.rs`</sub>

Curtain walls in the app: the Curtain Grid tool (add and remove grid
lines, merge cells across a segment, swap panels by clicking a curtain
wall in a plan, elevation or section), its options bar, the grid rows of
the properties palette and the curtain sections of the Type Properties
dialog. Every change runs an engine command (`curtain_grid_set`,
`curtain_panel_set`, `type_set`).

**Items:** [`GridMode`](#cc_app-curtain_ui-gridmode) · [`PanelChoice`](#cc_app-curtain_ui-panelchoice) · [`CurtainOptions`](#cc_app-curtain_ui-curtainoptions) · [`grid_tool`](#cc_app-curtain_ui-grid_tool) · [`has_curtain_type`](#cc_app-curtain_ui-has_curtain_type) · [`new_type_label`](#cc_app-curtain_ui-new_type_label) · [`new_curtain_type`](#cc_app-curtain_ui-new_curtain_type) · [`options_bar`](#cc_app-curtain_ui-options_bar) · [`palette_rows`](#cc_app-curtain_ui-palette_rows) · [`type_sections`](#cc_app-curtain_ui-type_sections)

<a id="cc_app-curtain_ui-gridmode"></a>

### enum `GridMode`

```rust
pub enum GridMode
```

What a click of the Curtain Grid tool does.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Default`

**Variants**

- `AddLine`

  Put in a grid line (vertical or horizontal, see `CurtainOptions::axis`).

- `RemoveLine`

  Take out the line nearest the click.

- `Segment`

  Merge the cells on both sides of the clicked segment, or split them again.

- `Panel`

  Give the clicked panel `CurtainOptions::panel`.


**Methods**

- `pub const ALL: [GridMode; 4]`
- `pub fn key(self) -> &'static str`

<a id="cc_app-curtain_ui-panelchoice"></a>

### enum `PanelChoice`

```rust
pub enum PanelChoice
```

The panel the tool puts in.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Default`

**Variants**

- `Type`

  The type's panel again.

- `Glass`
- `Solid`
- `Empty`
- `Component(ElementTypeId)`

<a id="cc_app-curtain_ui-curtainoptions"></a>

### struct `CurtainOptions`

```rust
pub struct CurtainOptions
```

Options of the Curtain Grid tool.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`

**Fields**

- `mode: GridMode`
- `axis: GridAxis`

  Direction of new lines (plans only add vertical ones).

- `panel: PanelChoice`

<a id="cc_app-curtain_ui-grid_tool"></a>

### fn `grid_tool`

```rust
pub fn grid_tool(resp: &egui::Response, painter: &egui::Painter, state: &mut AppState, view: &View, xf: &Xf)
```

The Curtain Grid tool: hovering a curtain wall shows what a click does
(the line to add or remove, the segment, the panel); clicking does it.

<a id="cc_app-curtain_ui-has_curtain_type"></a>

### fn `has_curtain_type`

```rust
pub fn has_curtain_type(state: &AppState) -> bool
```

Whether the project has a curtain wall type.

<a id="cc_app-curtain_ui-new_type_label"></a>

### fn `new_type_label`

```rust
pub fn new_type_label() -> String
```

The wall tool's list entry that makes a curtain type.

<a id="cc_app-curtain_ui-new_curtain_type"></a>

### fn `new_curtain_type`

```rust
pub fn new_curtain_type(state: &mut AppState) -> Option<ElementTypeId>
```

The project's first curtain wall type, made (glass between steel
mullions) when it has none.

<a id="cc_app-curtain_ui-options_bar"></a>

### fn `options_bar`

```rust
pub fn options_bar(ui: &mut Ui, state: &mut AppState)
```

The options bar of the Curtain Grid tool.

<a id="cc_app-curtain_ui-palette_rows"></a>

### fn `palette_rows`

```rust
pub fn palette_rows(ui: &mut Ui, state: &mut AppState, el: &Element, w: &cc_core::Wall)
```

The grid of a selected curtain wall: its columns and rows, panel count,
glazed area and mullion length, and resetting a direction to the type's lines.

<a id="cc_app-curtain_ui-type_sections"></a>

### fn `type_sections`

```rust
pub fn type_sections(ui: &mut Ui, state: &mut AppState, ty: &ElementType)
```

The curtain sections of the Type Properties dialog: both grids, the
panel and the three kinds of mullions.

<a id="mod-cc_app-design_options_ui"></a>

## `cc_app::design_options_ui`

<sub>`crates/cc_app/src/design_options_ui.rs`</sub>

Design options in the application: the Design Options dialog (sets,
their options and primary option, accepting the primary), the menu of
the option being edited, and the option rows of view, template and
element properties. Every change goes through the engine's commands; OK
keeps the dialog's changes, Cancel undoes them.

**Items:** [`OptionsDialog`](#cc_app-design_options_ui-optionsdialog) · [`open`](#cc_app-design_options_ui-open) · [`show`](#cc_app-design_options_ui-show) · [`active_label`](#cc_app-design_options_ui-active_label) · [`active_menu`](#cc_app-design_options_ui-active_menu) · [`view_section`](#cc_app-design_options_ui-view_section) · [`template_rows`](#cc_app-design_options_ui-template_rows) · [`element_rows`](#cc_app-design_options_ui-element_rows)

<a id="cc_app-design_options_ui-optionsdialog"></a>

### struct `OptionsDialog`

```rust
pub struct OptionsDialog
```

The open Design Options dialog.

*Some fields are private.*

<a id="cc_app-design_options_ui-open"></a>

### fn `open`

```rust
pub fn open(state: &mut AppState)
```

<a id="cc_app-design_options_ui-show"></a>

### fn `show`

```rust
pub fn show(ctx: &Context, state: &mut AppState)
```

<a id="cc_app-design_options_ui-active_label"></a>

### fn `active_label`

```rust
pub fn active_label(state: &AppState) -> String
```

The name of the option being edited, or of the main model.

<a id="cc_app-design_options_ui-active_menu"></a>

### fn `active_menu`

```rust
pub fn active_menu(ui: &mut Ui, state: &mut AppState)
```

The menu of the option being edited: the main model, then every option.

<a id="cc_app-design_options_ui-view_section"></a>

### fn `view_section`

```rust
pub fn view_section(ui: &mut Ui, state: &mut AppState, view: ViewId)
```

The Design Options section of a view's properties (plans, elevations,
sections, 3D views and schedules): the option shown of each set,
read-only when the view's template controls it.

<a id="cc_app-design_options_ui-template_rows"></a>

### fn `template_rows`

```rust
pub fn template_rows(ui: &mut Ui, state: &mut AppState, template: cc_core::TemplateId)
```

A view template's choices (in the template manager's grid).

<a id="cc_app-design_options_ui-element_rows"></a>

### fn `element_rows`

```rust
pub fn element_rows(ui: &mut Ui, state: &mut AppState, el: &Element)
```

The design option an element belongs to (hosted elements show their
host's, read-only).

<a id="mod-cc_app-detail_component_ui"></a>

## `cc_app::detail_component_ui`

<sub>`crates/cc_app/src/detail_component_ui.rs`</sub>

Detail components in the app: the Detail Component tool (Annotate tab),
which places a type in the active 2D view with the rotation, scale and
mirror of its options (`detail_component_place`); *Create Detail
Component*, which makes a type from the selected detail lines, regions,
texts and detail components of the view and puts it in their place
(`detail_component_create`); and the rows of a placed one in the
properties palette. Pieces are edited with the `component_sheet_piece_*`
commands (the command palette).

**Items:** [`PlaceOptions`](#cc_app-detail_component_ui-placeoptions) · [`options`](#cc_app-detail_component_ui-options) · [`tool`](#cc_app-detail_component_ui-tool) · [`selection`](#cc_app-detail_component_ui-selection) · [`create_from_selection`](#cc_app-detail_component_ui-create_from_selection) · [`props`](#cc_app-detail_component_ui-props)

<a id="cc_app-detail_component_ui-placeoptions"></a>

### struct `PlaceOptions`

```rust
pub struct PlaceOptions
```

Options of the Detail Component tool.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`

**Fields**

- `rotation: f64`

  Degrees counter-clockwise.

- `scale: f64`
- `mirrored: bool`

<a id="cc_app-detail_component_ui-options"></a>

### fn `options`

```rust
pub fn options(ui: &mut Ui, state: &mut AppState)
```

The options bar of the tool: rotation, scale and mirror.

<a id="cc_app-detail_component_ui-tool"></a>

### fn `tool`

```rust
pub fn tool(ui: &Ui, resp: &Response, painter: &Painter, state: &mut AppState, view: &View, type_id: ElementTypeId, xf: &Xf)
```

The Detail Component tool: the type follows the cursor; a click places it.

<a id="cc_app-detail_component_ui-selection"></a>

### fn `selection`

```rust
pub fn selection(state: &AppState) -> Option<(cc_core::ViewId, Vec<ElementId>)>
```

The view and the selected elements a detail component can be made of:
detail lines, regions, text notes and detail components, all of the
active view (one that holds drawings).

<a id="cc_app-detail_component_ui-create_from_selection"></a>

### fn `create_from_selection`

```rust
pub fn create_from_selection(state: &mut AppState)
```

*Create Detail Component*: the selection becomes a new detail component
type (named `Detail 1`, `Detail 2`…, renamed in its type's properties),
placed where the elements were; the new element is selected.

<a id="cc_app-detail_component_ui-props"></a>

### fn `props`

```rust
pub fn props(ui: &mut Ui, state: &mut AppState, id: ElementId, c: &DetailComponent)
```

A placed detail component in the properties palette: its component and
view, and its rotation, scale and mirror.

<a id="mod-cc_app-detail_ui"></a>

## `cc_app::detail_ui`

<sub>`crates/cc_app/src/detail_ui.rs`</sub>

Detail lines in the app: the *Detail Line* tool (Annotate tab) with its
line, polyline, arc, circle, ellipse and rectangle modes and the line
style in the options bar, the grips of selected detail lines (ends,
centers, an arc's middle, axis points), their properties, and their
previews in the Split and Offset tools. The tool draws with the sketch
mode's curve tools (see `crate::sketch_mode::draw`) and makes each
curve an element through `detail_line_create` as soon as it is drawn;
a polyline goes on from the last end until it closes or Esc.

**Items:** [`DetailMode`](#cc_app-detail_ui-detailmode) · [`DetailOptions`](#cc_app-detail_ui-detailoptions) · [`DetailDraw`](#cc_app-detail_ui-detaildraw) · [`tool`](#cc_app-detail_ui-tool) · [`create`](#cc_app-detail_ui-create) · [`typing`](#cc_app-detail_ui-typing) · [`typed_target`](#cc_app-detail_ui-typed_target) · [`escape`](#cc_app-detail_ui-escape) · [`view_owned`](#cc_app-detail_ui-view_owned) · [`near`](#cc_app-detail_ui-near) · [`grips`](#cc_app-detail_ui-grips) · [`split_pick`](#cc_app-detail_ui-split_pick) · [`offset_pick`](#cc_app-detail_ui-offset_pick) · [`options`](#cc_app-detail_ui-options) · [`style_label`](#cc_app-detail_ui-style_label) · [`style_picker`](#cc_app-detail_ui-style_picker) · [`props`](#cc_app-detail_ui-props)

<a id="cc_app-detail_ui-detailmode"></a>

### enum `DetailMode`

```rust
pub enum DetailMode
```

How the Detail Line tool draws.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Default`

**Variants**

- `Line`

  One line from two points.

- `Polyline`

  Lines chained end to end.

- `Arc3`

  An arc through its two ends and a point on it.

- `ArcCenter`

  An arc by its center, start and end.

- `Circle`
- `Ellipse`
- `Rectangle`

**Methods**

- `pub const ALL: [DetailMode; 7]`
- `pub fn key(self) -> &'static str`

<a id="cc_app-detail_ui-detailoptions"></a>

### struct `DetailOptions`

```rust
pub struct DetailOptions
```

Options of the Detail Line tool (kept between uses, in the options bar).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Default`

**Fields**

- `mode: DetailMode`
- `style: LineStyleRef`

  A built-in style or one of the project's.


<a id="cc_app-detail_ui-detaildraw"></a>

### struct `DetailDraw`

```rust
pub struct DetailDraw
```

The curve being drawn in a view.

**Traits:** `Debug`, `Clone`

*Some fields are private.*

<a id="cc_app-detail_ui-tool"></a>

### fn `tool`

```rust
pub fn tool(ui: &Ui, resp: &Response, painter: &Painter, state: &mut AppState, view: &cc_core::View, xf: &Xf)
```

The Detail Line tool in a 2D view: snaps, previews the curve and makes
each finished curve a detail line of the view.

<a id="cc_app-detail_ui-create"></a>

### fn `create`

```rust
pub(crate) fn create(state: &mut AppState, view: ViewId, style: LineStyleRef, curves: &[SketchCurve])
```

Make `curves` detail lines of `view` (one undo step); they are selected.

<a id="cc_app-detail_ui-typing"></a>

### fn `typing`

```rust
pub fn typing(state: &AppState) -> bool
```

The Detail Line tool takes typed values now: it has the first point of
a line, arc, circle, ellipse or rectangle.

<a id="cc_app-detail_ui-typed_target"></a>

### fn `typed_target`

```rust
pub fn typed_target(state: &mut AppState) -> &mut TypedInput
```

Where typed values go: the detail line being drawn (see `typing`),
else the other tools' lengths.

<a id="cc_app-detail_ui-escape"></a>

### fn `escape`

```rust
pub fn escape(state: &mut AppState) -> bool
```

Esc while a curve is being drawn: drop the typed value, else its picks.
Returns whether it did.

<a id="cc_app-detail_ui-view_owned"></a>

### fn `view_owned`

```rust
pub fn view_owned(state: &AppState, tool: &Tool) -> bool
```

Whether a modify tool may run outside a plan: the elements it moves
are all owned by a view (detail lines, texts, dimensions), whose
coordinates are the view's own. Tools that pick their element take only
detail lines there (see `crate::modify`).

<a id="cc_app-detail_ui-near"></a>

### fn `near`

```rust
pub fn near(line: &DetailLine, p: DVec2, r: f64) -> bool
```

Whether a detail line passes within `r` of `p` (a quick bounds check for snapping).

<a id="cc_app-detail_ui-grips"></a>

### fn `grips`

```rust
pub fn grips(ui: &Ui, resp: &Response, painter: &Painter, state: &mut AppState, xf: &Xf) -> bool
```

The grips of the selected detail lines of the active view: dragging one
reshapes the line (`element_set`). Returns whether it took the pointer.

<a id="cc_app-detail_ui-split_pick"></a>

### fn `split_pick`

```rust
pub fn split_pick(painter: &Painter, xf: &Xf, state: &mut AppState, id: ElementId, raw: DVec2, click: bool) -> bool
```

The Split tool over a detail line: a tick where it splits; a click
splits it. Returns whether `id` is a detail line.

<a id="cc_app-detail_ui-offset_pick"></a>

### fn `offset_pick`

```rust
pub fn offset_pick(painter: &Painter, xf: &Xf, state: &mut AppState, id: ElementId, raw: DVec2, distance: f64, click: bool) -> bool
```

The Offset tool over a detail line: the copy on the cursor's side at the
distance of the options bar; a click makes it. Returns whether `id` is a
detail line.

<a id="cc_app-detail_ui-options"></a>

### fn `options`

```rust
pub fn options(ui: &mut Ui, state: &mut AppState)
```

The options bar of the Detail Line tool: the mode and the line style.

<a id="cc_app-detail_ui-style_label"></a>

### fn `style_label`

```rust
pub fn style_label(style: Option<LineStyleRef>, styles: &LineStyles) -> String
```

The name of a line style as shown: a built-in style's translated name,
a project style's own name; `None` (no line) when allowed.

<a id="cc_app-detail_ui-style_picker"></a>

### fn `style_picker`

```rust
pub fn style_picker(ui: &mut Ui, id: impl std::hash::Hash + std::fmt::Debug, styles: &LineStyles, style: &mut Option<LineStyleRef>, allow_none: bool) -> bool
```

A combo box of the built-in line styles and the project's (and "none"
with `allow_none`); returns whether it changed.

<a id="cc_app-detail_ui-props"></a>

### fn `props`

```rust
pub fn props(ui: &mut Ui, state: &mut AppState, id: ElementId, line: &DetailLine)
```

The properties of a selected detail line: its style, view and size.

<a id="mod-cc_app-dialog"></a>

## `cc_app::dialog`

<sub>`crates/cc_app/src/dialog.rs`</sub>

Standard dialog frame shared by every modal window: a header with the
title and a close button, a padded body (with an icon for alerts) and a
footer with the buttons.
Footer conventions: action buttons are right-aligned in the usual desktop
order (main action first, Cancel last), auxiliary buttons sit on the left,
the main action is highlighted and destructive actions are red. Esc and the
close button dismiss the dialog, like Cancel; clicks outside are ignored.

**Items:** [`BUTTON_SIZE`](#cc_app-dialog-button_size) · [`Kind`](#cc_app-dialog-kind) · [`Style`](#cc_app-dialog-style) · [`styled_button`](#cc_app-dialog-styled_button) · [`Footer`](#cc_app-dialog-footer) · [`DialogResponse`](#cc_app-dialog-dialogresponse) · [`Dialog`](#cc_app-dialog-dialog) · [`two_columns`](#cc_app-dialog-two_columns)

<a id="cc_app-dialog-button_size"></a>

### const `BUTTON_SIZE`

```rust
pub const BUTTON_SIZE: egui::Vec2
```

Minimum size of footer buttons.

<a id="cc_app-dialog-kind"></a>

### enum `Kind`

```rust
pub enum Kind
```

Icon shown next to the message of an alert.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Variants**

- `Warning`
- `Error`

<a id="cc_app-dialog-style"></a>

### enum `Style`

```rust
pub enum Style
```

Visual weight of a footer button.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Variants**

- `Primary`
- `Secondary`
- `Danger`

<a id="cc_app-dialog-styled_button"></a>

### fn `styled_button`

```rust
pub fn styled_button(ui: &mut Ui, label: &str, style: Style) -> egui::Response
```

A highlighted button with the standard footer size.

<a id="cc_app-dialog-footer"></a>

### struct `Footer`

```rust
pub struct Footer
```

Footer buttons, declared in display order; `Dialog::show` reports which was clicked.

**Traits:** `Default`

*Some fields are private.*

**Methods**

- `pub fn primary(&mut self, label: impl Into<String>) -> usize`

  The main action (Enter triggers it when no text field has focus).

- `pub fn secondary(&mut self, label: impl Into<String>) -> usize`
- `pub fn danger(&mut self, label: impl Into<String>) -> usize`
- `pub fn disable(&mut self, index: usize)`

  Grey out the button `index` (as returned by `Footer::primary` and
  the others, auxiliary buttons too): it cannot be clicked and Enter
  does not press it.

- `pub fn extra(&mut self, label: impl Into<String>) -> usize`

  An auxiliary button on the left side. Its index follows the actions'.

- `pub fn toggle(&mut self, label: impl Into<String>, on: bool) -> usize`

  An auxiliary button that stays pressed while `on` (a mode of the
  dialog the button turns on and off). Its index follows the actions'.


<a id="cc_app-dialog-dialogresponse"></a>

### struct `DialogResponse`

```rust
pub struct DialogResponse
```

**Fields**

- `clicked: Option<usize>`

  Index returned by the `Footer` method of the clicked button.

- `dismissed: bool`

  Closed with Esc or the close button.


<a id="cc_app-dialog-dialog"></a>

### struct `Dialog`

```rust
pub struct Dialog
```

*Some fields are private.*

**Methods**

- `pub fn new(id: impl std::hash::Hash + std::fmt::Debug, title: impl Into<String>) -> Self`
- `pub fn width(mut self, width: f32) -> Self`

  Width of the body content.

- `pub fn kind(mut self, kind: Kind) -> Self`

  Show an alert icon next to the body.

- `pub fn show(self, ctx: &Context, body: impl FnOnce(&mut Ui), footer: impl FnOnce(&mut Footer)) -> DialogResponse`

<a id="cc_app-dialog-two_columns"></a>

### fn `two_columns`

```rust
pub fn two_columns(ui: &mut Ui, widths: [f32; 2], mut column: impl FnMut(&mut Ui, usize))
```

Two top-down columns side by side (a list and its details), with a divider
as tall as the taller column; `column` draws column 0 (left) and 1 (right).
`ui.separator()` in a row takes the available height instead, and in a
modal that grows with the modal every frame until the dialog runs off the
window.

<a id="mod-cc_app-dialog-session"></a>

## `cc_app::dialog::session`

<sub>`crates/cc_app/src/dialog.rs`</sub>

Edits made while a data dialog is open can be cancelled as a whole: the
dialog remembers the undo depth when it opens and undoes back to it.

**Items:** [`begin`](#cc_app-dialog-session-begin) · [`commit`](#cc_app-dialog-session-commit) · [`cancel`](#cc_app-dialog-session-cancel)

<a id="cc_app-dialog-session-begin"></a>

### fn `begin`

```rust
pub fn begin(state: &mut AppState)
```

Mark the start of the dialog's edits (no-op if already marked).

<a id="cc_app-dialog-session-commit"></a>

### fn `commit`

```rust
pub fn commit(state: &mut AppState)
```

Keep the edits.

<a id="cc_app-dialog-session-cancel"></a>

### fn `cancel`

```rust
pub fn cancel(state: &mut AppState)
```

Undo every edit made since `begin`.

<a id="mod-cc_app-dialogs"></a>

## `cc_app::dialogs`

<sub>`crates/cc_app/src/dialogs.rs`</sub>

Application dialogs, all built on `crate::dialog::Dialog`.

**Items:** [`show`](#cc_app-dialogs-show) · [`ok_cancel`](#cc_app-dialogs-ok_cancel) · [`add_project_field`](#cc_app-dialogs-add_project_field) · [`rename_project_field`](#cc_app-dialogs-rename_project_field) · [`remove_project_field`](#cc_app-dialogs-remove_project_field) · [`material_combo`](#cc_app-dialogs-material_combo)

<a id="cc_app-dialogs-show"></a>

### fn `show`

```rust
pub fn show(ctx: &Context, state: &mut AppState)
```

<a id="cc_app-dialogs-ok_cancel"></a>

### fn `ok_cancel`

```rust
pub(crate) fn ok_cancel(state: &mut AppState, response: &crate::dialog::DialogResponse, ok: usize) -> bool
```

Footer of a data dialog: OK keeps the edits, Cancel (or Esc / close) undoes them.
Returns true when the dialog closes.

<a id="cc_app-dialogs-add_project_field"></a>

### fn `add_project_field`

```rust
pub(crate) fn add_project_field(state: &mut AppState) -> bool
```

Add a project field with an unused name (`Field 1`, `Field 2`…) and no value.

<a id="cc_app-dialogs-rename_project_field"></a>

### fn `rename_project_field`

```rust
pub(crate) fn rename_project_field(state: &mut AppState, old: &str, new: &str) -> bool
```

Rename project field `old` (refused when the new name is empty or taken).

<a id="cc_app-dialogs-remove_project_field"></a>

### fn `remove_project_field`

```rust
pub(crate) fn remove_project_field(state: &mut AppState, name: &str) -> bool
```

Remove project field `name`.

<a id="cc_app-dialogs-material_combo"></a>

### fn `material_combo`

```rust
pub(crate) fn material_combo(ui: &mut Ui, state: &AppState, id: impl Hash + std::fmt::Debug, current: &mut String) -> bool
```

<a id="mod-cc_app-dim_edit"></a>

## `cc_app::dim_edit`

<sub>`crates/cc_app/src/dim_edit.rs`</sub>

Editing measured values in place. Clicking the text of a selected
dimension (or double-clicking a dimension) opens a small field over the
view; Enter applies the value, Esc or a click elsewhere cancels.
Dimensions store points, not references, so a new value moves what the
dimension touches: the elements whose linework passes through its moving
end (`b`, or `a` when only that end touches something). Elements crossing
the move direction translate; walls, beams and grids along it stretch
their nearest end. This works in plans, where drawing coordinates are
model coordinates; elsewhere only free dimensions change.

**Items:** [`Target`](#cc_app-dim_edit-target) · [`ValueEdit`](#cc_app-dim_edit-valueedit) · [`open`](#cc_app-dim_edit-open) · [`canvas`](#cc_app-dim_edit-canvas) · [`show`](#cc_app-dim_edit-show) · [`apply_text`](#cc_app-dim_edit-apply_text) · [`set_dimension_value`](#cc_app-dim_edit-set_dimension_value) · [`Temp`](#cc_app-dim_edit-temp) · [`temps`](#cc_app-dim_edit-temps)

<a id="cc_app-dim_edit-target"></a>

### enum `Target`

```rust
pub enum Target
```

What an edited value drives.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Variants**

- `Dimension(ElementId)`

  The value of a dimension element.

- `Offset { ids: Vec<ElementId>, dir: DVec2, current: f64 }`

  A temporary dimension of the selection: `ids` move along `dir` so
  that the measured `current` distance becomes the typed one.


<a id="cc_app-dim_edit-valueedit"></a>

### struct `ValueEdit`

```rust
pub struct ValueEdit
```

The open value field.

**Traits:** `Debug`, `Clone`

*Some fields are private.*

<a id="cc_app-dim_edit-open"></a>

### fn `open`

```rust
pub fn open(state: &mut AppState, view: ViewId, target: Target, at: Pos2)
```

Open the value field for `target` at the screen point `at`.

<a id="cc_app-dim_edit-canvas"></a>

### fn `canvas`

```rust
pub fn canvas(ui: &Ui, resp: &Response, painter: &Painter, state: &mut AppState, view: ViewId, xf: &Xf, px_per_mm: f64, drawing: &Drawing2D) -> bool
```

Clicks on dimension text with the Select tool open the value field.
Returns true when the pointer was used (the tools skip the frame).

<a id="cc_app-dim_edit-show"></a>

### fn `show`

```rust
pub fn show(ui: &Ui, state: &mut AppState, view: ViewId)
```

The value field over `view`, when open there.

<a id="cc_app-dim_edit-apply_text"></a>

### fn `apply_text`

```rust
pub fn apply_text(state: &mut AppState, target: &Target, text: &str) -> Result<(), String>
```

Parse `text` for `target` and apply it.

<a id="cc_app-dim_edit-set_dimension_value"></a>

### fn `set_dimension_value`

```rust
pub fn set_dimension_value(state: &mut AppState, id: ElementId, value: f64) -> Result<(), String>
```

Give dimension `id` the measured `value` (meters, or radians for angles)
through the engine's `dimension_set_value`.

<a id="cc_app-dim_edit-temp"></a>

### struct `Temp`

```rust
pub struct Temp
```

A temporary dimension of the selected element: from a point of it to
the nearest reference linework, editable to move the element.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `from: DVec2`
- `to: DVec2`
- `away: DVec2`

  Unit direction the element moves for a larger value (away from `to`).


<a id="cc_app-dim_edit-temps"></a>

### fn `temps`

```rust
pub fn temps(state: &AppState, drawing: &Drawing2D) -> Vec<Temp>
```

Temporary dimensions of the single selected element in a plan: walls,
beams and grids to the nearest parallel linework on each side; columns
and components along X and Y; doors and windows along their wall.

<a id="mod-cc_app-dim_type_ui"></a>

## `cc_app::dim_type_ui`

<sub>`crates/cc_app/src/dim_type_ui.rs`</sub>

The Type Properties rows of a dimension type: the marks, witness and
dimension lines and pens (Graphics) and the text (Text).

**Items:** [`type_sections`](#cc_app-dim_type_ui-type_sections)

<a id="cc_app-dim_type_ui-type_sections"></a>

### fn `type_sections`

```rust
pub fn type_sections(ui: &mut Ui, state: &mut AppState, ty: &ElementType)
```

The dimension type's sections; nothing for other types.

<a id="mod-cc_app-engine_edit"></a>

## `cc_app::engine_edit`

<sub>`crates/cc_app/src/engine_edit.rs`</sub>

Property edits through the engine. Panels and dialogs describe a change
as a function on a copy of the object; these helpers send the result to
the engine's `*_set` command, merged into one undo step per `key` while a
value is dragged. Refusals (invalid values) go to the status bar.

**Items:** [`elements`](#cc_app-engine_edit-elements) · [`elements_identity`](#cc_app-engine_edit-elements_identity) · [`element`](#cc_app-engine_edit-element) · [`element_type`](#cc_app-engine_edit-element_type) · [`view`](#cc_app-engine_edit-view) · [`sheet`](#cc_app-engine_edit-sheet) · [`material`](#cc_app-engine_edit-material) · [`units`](#cc_app-engine_edit-units) · [`info`](#cc_app-engine_edit-info) · [`system`](#cc_app-engine_edit-system) · [`level`](#cc_app-engine_edit-level) · [`level_height`](#cc_app-engine_edit-level_height)

<a id="cc_app-engine_edit-elements"></a>

### fn `elements`

```rust
pub fn elements(state: &mut AppState, ids: &[ElementId], key: Option<u64>, level: Option<LevelId>, params: Option<serde_json::Value>) -> bool
```

Change elements `ids` at once (`elements_set`): a new `level` and the
same `params` merged into each, in one undo step per `key`.

<a id="cc_app-engine_edit-elements_identity"></a>

### fn `elements_identity`

```rust
pub fn elements_identity(state: &mut AppState, ids: &[ElementId], key: Option<u64>, mark: Option<String>, comments: Option<String>) -> bool
```

Give elements `ids` the same mark or comments (`elements_set`), in one
undo step per `key`.

<a id="cc_app-engine_edit-element"></a>

### fn `element`

```rust
pub fn element(state: &mut AppState, id: ElementId, key: Option<u64>, f: impl FnOnce(&mut Element)) -> bool
```

Change element `id` as `f` changes a copy of it (`element_set`).

<a id="cc_app-engine_edit-element_type"></a>

### fn `element_type`

```rust
pub fn element_type(state: &mut AppState, id: ElementTypeId, key: Option<u64>, f: impl FnOnce(&mut ElementType)) -> bool
```

Change type `id` as `f` changes a copy of it (`type_set`).

<a id="cc_app-engine_edit-view"></a>

### fn `view`

```rust
pub fn view(state: &mut AppState, id: ViewId, key: Option<u64>, f: impl FnOnce(&mut View)) -> bool
```

Change view `id` as `f` changes a copy of it (`view_set`).

<a id="cc_app-engine_edit-sheet"></a>

### fn `sheet`

```rust
pub fn sheet(state: &mut AppState, id: ViewId, key: Option<u64>, f: impl FnOnce(&mut View)) -> bool
```

Change sheet `id` as `f` changes a copy of it (`sheet_set`): its
number, paper, title block, per-sheet field values and issue date, in
one undo step per `key`.

<a id="cc_app-engine_edit-material"></a>

### fn `material`

```rust
pub fn material(state: &mut AppState, name: &str, key: Option<u64>, f: impl FnOnce(&mut Material)) -> bool
```

Change material `name` as `f` changes a copy of it (`material_set`; renaming has its own command).

<a id="cc_app-engine_edit-units"></a>

### fn `units`

```rust
pub fn units(state: &mut AppState, key: Option<u64>, f: impl FnOnce(&mut cc_core::Units)) -> bool
```

Change the project units as `f` changes a copy of them (`units_set`).

<a id="cc_app-engine_edit-info"></a>

### fn `info`

```rust
pub fn info(state: &mut AppState, key: Option<u64>, f: impl FnOnce(&mut cc_core::ProjectInfo)) -> bool
```

Change the project information as `f` changes a copy of it (`project_info_set`).

<a id="cc_app-engine_edit-system"></a>

### fn `system`

```rust
pub fn system(state: &mut AppState, id: cc_core::SystemId, key: Option<u64>, f: impl FnOnce(&mut cc_core::MepSystem)) -> bool
```

Change plumbing system or circuit `id` as `f` changes a copy (`system_set`).

<a id="cc_app-engine_edit-level"></a>

### fn `level`

```rust
pub fn level(state: &mut AppState, id: LevelId, key: Option<u64>, f: impl FnOnce(&mut Level)) -> bool
```

Rename a level or change its elevation as `f` changes a copy (`level_set`).

<a id="cc_app-engine_edit-level_height"></a>

### fn `level_height`

```rust
pub fn level_height(state: &mut AppState, id: LevelId, key: Option<u64>, height: f64) -> bool
```

Set the height from a level up to the next one; the levels above move along (`level_set`).

<a id="mod-cc_app-fill_patterns_dialog"></a>

## `cc_app::fill_patterns_dialog`

<sub>`crates/cc_app/src/fill_patterns_dialog.rs`</sub>

The Fill Patterns dialog (*Manage → Fill Patterns*, also opened from the
material browser and a filled region type's fill): the built-in
patterns, read only, and the project's own with a drawn swatch each;
the selected one's name, description, kind (drafting or model), scale
and line families, edited in place over a live preview. *Import .pat…*
reads a file and lets the user pick which of its patterns come in;
*New*, *Duplicate* and *Delete* (with the pattern that takes its place
where it is used) complete it. Every change runs a `fill_pattern_*`
command; OK keeps them, Cancel undoes them all (`dialog::session`,
through `crate::graphics_dialog::Edits`).
The module also holds what the pattern pickers share: labels, swatches
and the picker itself, listing custom patterns after the built-ins.

**Items:** [`label`](#cc_app-fill_patterns_dialog-label) · [`choices`](#cc_app-fill_patterns_dialog-choices) · [`paint`](#cc_app-fill_patterns_dialog-paint) · [`swatch`](#cc_app-fill_patterns_dialog-swatch) · [`picker`](#cc_app-fill_patterns_dialog-picker) · [`FillPatternsDialog`](#cc_app-fill_patterns_dialog-fillpatternsdialog) · [`open`](#cc_app-fill_patterns_dialog-open) · [`show`](#cc_app-fill_patterns_dialog-show) · [`open_import`](#cc_app-fill_patterns_dialog-open_import)

<a id="cc_app-fill_patterns_dialog-label"></a>

### fn `label`

```rust
pub fn label(project: &Project, p: Pattern) -> String
```

A pattern's name: the built-in's, translated, or the project pattern's.

<a id="cc_app-fill_patterns_dialog-choices"></a>

### fn `choices`

```rust
pub fn choices(project: &Project) -> Vec<(Pattern, String)>
```

The patterns a picker offers: the built-in ones, then the project's.

<a id="cc_app-fill_patterns_dialog-paint"></a>

### fn `paint`

```rust
pub fn paint(painter: &egui::Painter, rect: Rect, project: &Project, pattern: Pattern, color: [u8; 3], spacing: f64)
```

Paint `pattern` in `color` over `rect` (built-ins with lines `spacing`
points apart); `Solid` fills it.

<a id="cc_app-fill_patterns_dialog-swatch"></a>

### fn `swatch`

```rust
pub fn swatch(ui: &mut Ui, project: &Project, pattern: Pattern, size: egui::Vec2) -> egui::Response
```

A small framed swatch of `pattern` in black on white.

<a id="cc_app-fill_patterns_dialog-picker"></a>

### fn `picker`

```rust
pub fn picker(ui: &mut Ui, id: impl std::hash::Hash + std::fmt::Debug, project: &Project, value: &mut Pattern, width: f32) -> bool
```

A drop-down of every pattern (custom ones after the built-ins, each
with its swatch); returns whether the choice changed.

<a id="cc_app-fill_patterns_dialog-fillpatternsdialog"></a>

### struct `FillPatternsDialog`

```rust
pub struct FillPatternsDialog
```

The open Fill Patterns dialog.

*Some fields are private.*

<a id="cc_app-fill_patterns_dialog-open"></a>

### fn `open`

```rust
pub fn open(state: &mut AppState)
```

<a id="cc_app-fill_patterns_dialog-show"></a>

### fn `show`

```rust
pub fn show(ctx: &Context, state: &mut AppState)
```

<a id="cc_app-fill_patterns_dialog-open_import"></a>

### fn `open_import`

```rust
pub fn open_import(state: &mut AppState, path: PathBuf)
```

Read the patterns of the `.pat` file at `path` and offer them.

<a id="mod-cc_app-gestures"></a>

## `cc_app::gestures`

<sub>`crates/cc_app/src/gestures.rs`</sub>

Scroll and pinch gestures over the views (issue #174).
egui adds the mouse wheel and a trackpad's two-finger scroll into one
scroll delta. The views tell them apart by the wheel events of the frame:
a wheel reports lines (or pages), a trackpad reports points. A wheel
zooms, as in CAD programs; with the *Trackpad scrolling pans* preference
(on by default) two fingers move the view and a pinch zooms. The kind of
the last wheel event is remembered, since egui spreads a wheel notch over
a few frames that have no events of their own.

**Items:** [`set_trackpad_pans`](#cc_app-gestures-set_trackpad_pans) · [`Gesture`](#cc_app-gestures-gesture) · [`read`](#cc_app-gestures-read)

<a id="cc_app-gestures-set_trackpad_pans"></a>

### fn `set_trackpad_pans`

```rust
pub fn set_trackpad_pans(on: bool)
```

Follow the *Trackpad scrolling pans* preference.

<a id="cc_app-gestures-gesture"></a>

### struct `Gesture`

```rust
pub struct Gesture
```

What the scroll and pinch of this frame ask a view to do.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `zoom_scroll: f32`

  Scroll that zooms, in points (a wheel; positive: in).

- `pinch: f32`

  Pinch (or Ctrl+scroll) zoom factor (1: none).

- `pan: Vec2`

  How far the content moves on screen, in points (two fingers on a trackpad).


**Methods**

- `pub fn zoom(&self, rate: f64) -> f64`

  The zoom factor, with `rate` per point of wheel scroll.


<a id="cc_app-gestures-read"></a>

### fn `read`

```rust
pub fn read(ui: &Ui) -> Gesture
```

The gesture of this frame (call it only while the view is hovered).

<a id="mod-cc_app-gpu"></a>

## `cc_app::gpu`

<sub>`crates/cc_app/src/gpu.rs`</sub>

Graphics start-up: which wgpu backends the window tries, in what order,
and what its device asks of the adapter.
The native APIs (Vulkan, DirectX 12, Metal) are tried first and OpenGL
after them, unless `WGPU_BACKEND` names the backends to use. A start-up
that fails with an error moves on to the next API in the same process; one
that panics before the first frame restarts the app once; one that kills
the process (a driver crash) leaves a marker file behind, so the next start
tries the other API first. Once OpenGL has been needed it stays first
(`gpu-backend` in the data folder; delete it to try the native APIs again).
Every failure is written to the crash log.

**Items:** [`Api`](#cc_app-gpu-api) · [`plan`](#cc_app-gpu-plan) · [`required_limits`](#cc_app-gpu-required_limits) · [`wgpu_options`](#cc_app-gpu-wgpu_options) · [`startup_plan`](#cc_app-gpu-startup_plan) · [`begin`](#cc_app-gpu-begin) · [`started`](#cc_app-gpu-started) · [`note_frame`](#cc_app-gpu-note_frame) · [`failed`](#cc_app-gpu-failed) · [`restart`](#cc_app-gpu-restart) · [`report_failure`](#cc_app-gpu-report_failure)

<a id="cc_app-gpu-api"></a>

### enum `Api`

```rust
pub enum Api
```

A family of graphics backends to start the window with.

**Traits:** `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`

**Variants**

- `Native`

  The platform's native APIs (Vulkan, DirectX 12, Metal).

- `Gl`

  OpenGL.

- `Env(wgpu::Backends)`

  The backends named by the `WGPU_BACKEND` environment variable.


**Methods**

- `pub fn backends(self) -> wgpu::Backends`

<a id="cc_app-gpu-plan"></a>

### fn `plan`

```rust
pub fn plan(env: Option<wgpu::Backends>, saved: Option<Api>, interrupted: Option<Api>) -> Vec<Api>
```

The APIs to try, in order: only those of `WGPU_BACKEND` when it names any
(`env`); otherwise the native APIs and OpenGL, the other API first when the
last start-up died with one (`interrupted`), else OpenGL first once it was
needed (`saved`).

<a id="cc_app-gpu-required_limits"></a>

### fn `required_limits`

```rust
pub fn required_limits(backend: wgpu::Backend, adapter: &wgpu::Limits) -> wgpu::Limits
```

The limits the device asks for: wgpu's defaults (WebGL 2's on OpenGL) with
textures up to 8192 px (a depth buffer as large as a 4K+ window), lowered to
what the adapter supports, since asking for more than it has fails the start.

<a id="cc_app-gpu-wgpu_options"></a>

### fn `wgpu_options`

```rust
pub fn wgpu_options(api: Api) -> egui_wgpu::WgpuConfiguration
```

eframe's wgpu options for a start-up with `api`.

<a id="cc_app-gpu-startup_plan"></a>

### fn `startup_plan`

```rust
pub fn startup_plan() -> Vec<Api>
```

The APIs to try at this start, from `WGPU_BACKEND` and the files of earlier
start-ups (a marker left behind by one that died is logged).

<a id="cc_app-gpu-begin"></a>

### fn `begin`

```rust
pub fn begin(api: Api)
```

A start-up with `api` begins: write its marker.

<a id="cc_app-gpu-started"></a>

### fn `started`

```rust
pub fn started() -> bool
```

Whether the start-up under way has shown its first frame.

<a id="cc_app-gpu-note_frame"></a>

### fn `note_frame`

```rust
pub fn note_frame()
```

Called at the start of every frame: at the second, the first has been
shown, so the start-up worked; remove its marker and keep OpenGL first if
it took OpenGL.

<a id="cc_app-gpu-failed"></a>

### fn `failed`

```rust
pub fn failed(api: Api, error: &str)
```

A start-up with `api` failed with an error: log it and remove its marker.

<a id="cc_app-gpu-restart"></a>

### fn `restart`

```rust
pub fn restart(api: Api) -> bool
```

After a panic before the first frame: start the app again (once), which
then tries the other API first (the marker is still there). Returns whether
it was started.

<a id="cc_app-gpu-report_failure"></a>

### fn `report_failure`

```rust
pub fn report_failure(error: &str)
```

Every API failed: log it and tell the user (the app has no window).

<a id="mod-cc_app-graph_ui"></a>

## `cc_app::graph_ui`

<sub>`crates/cc_app/src/graph_ui.rs`</sub>

Visual programming: the graph editor, a panel under the views.
The graph (`cc_engine::graph::Graph`) is edited here and run through the
engine's `graph_run` command, like every model change of the app; the
nodes come from `graph_nodes` (built-in nodes, the Lua node and a node per
engine command). The canvas is drawn with egui's painter: nodes, ports
colored by type, wires as curves, the value or error of each node under
it; an inspector on the right edits the selected node's inputs, Lua code
and lacing.

**Items:** [`GraphEditor`](#cc_app-graph_ui-grapheditor) · [`toggle`](#cc_app-graph_ui-toggle) · [`run`](#cc_app-graph_ui-run) · [`open_path`](#cc_app-graph_ui-open_path) · [`keys`](#cc_app-graph_ui-keys) · [`show`](#cc_app-graph_ui-show)

<a id="cc_app-graph_ui-grapheditor"></a>

### struct `GraphEditor`

```rust
pub struct GraphEditor
```

The graph editor's state (`AppState::graph`).

**Traits:** `Default`

**Fields**

- `open: bool`
- `graph: Graph`
- `path: Option<PathBuf>`
- `dirty: bool`
- `report: Option<Result<Value, String>>`

  The last run's result (`graph_run`), or why it could not run.

- `selected: BTreeSet<u64>`
- `hovered: bool`

  Whether the pointer is over the panel: its keys go to the graph.

- `allow_files: bool`

  The user let this graph's commands work with files: a grant of this
  session, never saved with the graph and reset when another opens.


*Some fields are private.*

<a id="cc_app-graph_ui-toggle"></a>

### fn `toggle`

```rust
pub fn toggle(state: &mut AppState)
```

Open or close the panel.

<a id="cc_app-graph_ui-run"></a>

### fn `run`

```rust
pub fn run(state: &mut AppState)
```

Run the graph through the engine (one undo step).

<a id="cc_app-graph_ui-open_path"></a>

### fn `open_path`

```rust
pub fn open_path(state: &mut AppState, path: PathBuf)
```

Open the graph file at `path` in the editor.

<a id="cc_app-graph_ui-keys"></a>

### fn `keys`

```rust
pub fn keys(ctx: &egui::Context, state: &mut AppState, undo: bool, redo: bool, save: bool, save_as: bool) -> bool
```

Keys while the pointer is over the panel; returns whether the graph took them.

<a id="cc_app-graph_ui-show"></a>

### fn `show`

```rust
pub fn show(ui: &mut Ui, state: &mut AppState)
```

The panel: toolbar, canvas and inspector.

<a id="mod-cc_app-graphics_dialog"></a>

## `cc_app::graphics_dialog`

<sub>`crates/cc_app/src/graphics_dialog.rs`</sub>

View graphics dialogs: Visibility/Graphics of a view or view template
(categories and filters), graphic overrides of selected elements, the
view filter editor and the view template manager. Every change goes
through the engine's commands; OK keeps the changes, Cancel undoes them.

**Items:** [`Target`](#cc_app-graphics_dialog-target) · [`Edits`](#cc_app-graphics_dialog-edits) · [`GraphicsDialog`](#cc_app-graphics_dialog-graphicsdialog) · [`OverrideDialog`](#cc_app-graphics_dialog-overridedialog) · [`FilterEditor`](#cc_app-graphics_dialog-filtereditor) · [`TemplateManager`](#cc_app-graphics_dialog-templatemanager) · [`key`](#cc_app-graphics_dialog-key) · [`run`](#cc_app-graphics_dialog-run) · [`overrides_json`](#cc_app-graphics_dialog-overrides_json) · [`open`](#cc_app-graphics_dialog-open) · [`open_overrides`](#cc_app-graphics_dialog-open_overrides) · [`open_filters`](#cc_app-graphics_dialog-open_filters) · [`open_templates`](#cc_app-graphics_dialog-open_templates) · [`show`](#cc_app-graphics_dialog-show) · [`override_cells`](#cc_app-graphics_dialog-override_cells) · [`header`](#cc_app-graphics_dialog-header) · [`override_headers`](#cc_app-graphics_dialog-override_headers) · [`view_rows`](#cc_app-graphics_dialog-view_rows) · [`controlled`](#cc_app-graphics_dialog-controlled)

<a id="cc_app-graphics_dialog-target"></a>

### enum `Target`

```rust
pub enum Target
```

Whose graphics are edited.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`

**Variants**

- `View(ViewId)`
- `Template(TemplateId)`

<a id="cc_app-graphics_dialog-edits"></a>

### struct `Edits`

```rust
pub struct Edits
```

Edits made while a dialog is open, cancelled as a whole. A dialog opened
over another one undoes back to its own start (the outer one owns the
shared `session`).

**Traits:** `Debug`, `Clone`, `Copy`

*Some fields are private.*

**Methods**

- `pub(crate) fn begin(state: &mut AppState) -> Self`
- `pub(crate) fn owns(&self) -> bool`

  Whether these edits own the shared session (no other data dialog
  was open when they began).

- `pub(crate) fn finish(self, state: &mut AppState, ok: bool)`

<a id="cc_app-graphics_dialog-graphicsdialog"></a>

### struct `GraphicsDialog`

```rust
pub struct GraphicsDialog
```

The open Visibility/Graphics dialog.

**Fields**

- `target: Target`

*Some fields are private.*

<a id="cc_app-graphics_dialog-overridedialog"></a>

### struct `OverrideDialog`

```rust
pub struct OverrideDialog
```

The open element override dialog.

*Some fields are private.*

<a id="cc_app-graphics_dialog-filtereditor"></a>

### struct `FilterEditor`

```rust
pub struct FilterEditor
```

The open filter editor.

*Some fields are private.*

<a id="cc_app-graphics_dialog-templatemanager"></a>

### struct `TemplateManager`

```rust
pub struct TemplateManager
```

The open view template manager.

*Some fields are private.*

<a id="cc_app-graphics_dialog-key"></a>

### fn `key`

```rust
pub(crate) fn key(v: impl Hash) -> u64
```

<a id="cc_app-graphics_dialog-run"></a>

### fn `run`

```rust
pub(crate) fn run<C: cc_engine::Command + serde::Serialize>(state: &mut AppState, merge: Option<u64>, command: C) -> bool
```

<a id="cc_app-graphics_dialog-overrides_json"></a>

### fn `overrides_json`

```rust
pub(crate) fn overrides_json(o: &GraphicOverride) -> serde_json::Value
```

<a id="cc_app-graphics_dialog-open"></a>

### fn `open`

```rust
pub fn open(state: &mut AppState, target: Target)
```

Open Visibility/Graphics for a view or template.

<a id="cc_app-graphics_dialog-open_overrides"></a>

### fn `open_overrides`

```rust
pub fn open_overrides(state: &mut AppState)
```

Open the override dialog for the selected elements in the active view.

<a id="cc_app-graphics_dialog-open_filters"></a>

### fn `open_filters`

```rust
pub fn open_filters(state: &mut AppState)
```

<a id="cc_app-graphics_dialog-open_templates"></a>

### fn `open_templates`

```rust
pub fn open_templates(state: &mut AppState)
```

<a id="cc_app-graphics_dialog-show"></a>

### fn `show`

```rust
pub fn show(ctx: &Context, state: &mut AppState)
```

<a id="cc_app-graphics_dialog-override_cells"></a>

### fn `override_cells`

```rust
pub(crate) fn override_cells(ui: &mut Ui, ov: &mut GraphicOverride, enabled: bool, choices: &[(Pattern, String)]) -> bool
```

The override cells of one row, in the column order of `override_headers`.

<a id="cc_app-graphics_dialog-header"></a>

### fn `header`

```rust
pub(crate) fn header(ui: &mut Ui, text: String)
```

<a id="cc_app-graphics_dialog-override_headers"></a>

### fn `override_headers`

```rust
pub(crate) fn override_headers(ui: &mut Ui)
```

<a id="cc_app-graphics_dialog-view_rows"></a>

### fn `view_rows`

```rust
pub fn view_rows(ui: &mut Ui, state: &mut AppState, view: ViewId)
```

The view's template and Visibility/Graphics in its properties.

<a id="cc_app-graphics_dialog-controlled"></a>

### fn `controlled`

```rust
pub fn controlled(state: &AppState, view: ViewId, what: &str) -> bool
```

Whether the view's template controls `what` (`scale`, `detail`,
`visual_style`, `cut_height`, `discipline`, `structural`), for read-only rows.

<a id="mod-cc_app-grid_frame_ui"></a>

## `cc_app::grid_frame_ui`

<sub>`crates/cc_app/src/grid_frame_ui.rs`</sub>

*Structure → Grid Frame*: the levels, the types (of the tab's material)
and optional secondary beams for `grid_frame_create`.

**Items:** [`GridFrameDialog`](#cc_app-grid_frame_ui-gridframedialog) · [`open`](#cc_app-grid_frame_ui-open) · [`combo`](#cc_app-grid_frame_ui-combo) · [`show`](#cc_app-grid_frame_ui-show)

<a id="cc_app-grid_frame_ui-gridframedialog"></a>

### struct `GridFrameDialog`

```rust
pub struct GridFrameDialog
```

**Traits:** `Debug`, `Clone`

**Fields**

- `base: LevelId`
- `top: LevelId`
- `column_type: Option<ElementTypeId>`
- `beam_type: Option<ElementTypeId>`
- `secondary: bool`
- `parallel_to: Option<ElementId>`
- `spacing: f64`
- `error: Option<String>`

<a id="cc_app-grid_frame_ui-open"></a>

### fn `open`

```rust
pub fn open(state: &mut AppState)
```

Open the dialog: from the active level (else the lowest) to the next one.

<a id="cc_app-grid_frame_ui-combo"></a>

### fn `combo`

```rust
pub(crate) fn combo<T: PartialEq + Copy>(ui: &mut egui::Ui, id: &str, value: &mut T, items: &[(T, String)])
```

<a id="cc_app-grid_frame_ui-show"></a>

### fn `show`

```rust
pub fn show(ctx: &Context, state: &mut AppState)
```

<a id="mod-cc_app-home"></a>

## `cc_app::home`

<sub>`crates/cc_app/src/home.rs`</sub>

Home screen: open or create a project, recent projects and samples.

**Items:** [`sample_project`](#cc_app-home-sample_project) · [`show`](#cc_app-home-show) · [`push_recent`](#cc_app-home-push_recent)

<a id="cc_app-home-sample_project"></a>

### fn `sample_project`

```rust
pub fn sample_project(name: &str) -> Option<Project>
```

<a id="cc_app-home-show"></a>

### fn `show`

```rust
pub fn show(ui: &mut Ui, state: &mut AppState)
```

<a id="cc_app-home-push_recent"></a>

### fn `push_recent`

```rust
pub fn push_recent(recent: &mut Vec<PathBuf>, path: &Path, max: usize)
```

Remember `path` at the top of the recent projects list, which keeps at most `max` (at least one).

<a id="mod-cc_app-i18n"></a>

## `cc_app::i18n`

<sub>`crates/cc_app/src/i18n.rs`</sub>

Localization lives in the engine (`cc_engine::i18n`), so the GUI, the
command line and the MCP server name things alike; this module re-exports
it and defines the app's `t!` macro.

**Re-exports**

- `pub use cc_engine::i18n::*`

**Items:** [`t`](#cc_app-i18n-t)

<a id="cc_app-i18n-t"></a>

### macro `t`

```rust
macro_rules! t
```

`t!("key")` or `t!("key", "name" => value, ...)`.

<a id="mod-cc_app-icons"></a>

## `cc_app::icons`

<sub>`crates/cc_app/src/icons.rs`</sub>

Interface icons: SVG drawings in `assets/icons/ui` (24 x 24 grid, white
strokes and fills), rasterized with resvg at the exact pixel size they
are shown at and tinted with theme colors: strokes in one tone, soft
fills in a second (see `Tones`), and an optional `Badge` marks what
kind of action a command is. Textures are cached per icon, layer and
size, so each one is rasterized once.

**Items:** [`Icon`](#cc_app-icons-icon) · [`Badge`](#cc_app-icons-badge) · [`Tones`](#cc_app-icons-tones) · [`paint`](#cc_app-icons-paint) · [`paint_toned`](#cc_app-icons-paint_toned)

<a id="cc_app-icons-icon"></a>

### enum `Icon`

```rust
pub enum Icon
```

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`

**Variants**

- `New`
- `Open`
- `Save`
- `SaveAs`
- `Export`
- `About`
- `Update`
- `Wall`
- `Door`
- `Window`
- `Niche`
- `Floor`
- `Ceiling`
- `Roof`
- `Select`
- `Measure`
- `Group`
- `Ungroup`
- `Explode`
- `SelectInstances`
- `Copy`
- `Delete`
- `Undo`
- `Redo`
- `FloorPlan`
- `CeilingPlan`
- `View3D`
- `Section`
- `Callout`
- `Elevation`
- `Legend`
- `DraftingView`
- `Schedule`
- `Sheet`
- `TitleBlock`
- `ZoomFit`
- `ProjectInfo`
- `SiteLocation`
- `Levels`
- `Materials`
- `FillPatterns`
- `Profiles`
- `MacroRecord`
- `MacroRun`
- `Text`
- `DetailLine`
- `DetailComponent`
- `FilledRegion`
- `MaskingRegion`
- `LineStyles`
- `BrowserOrganization`
- `LevelMarkPlan`
- `LevelMarkSection`
- `Leader`
- `ChevronDown`
- `ChevronUp`
- `Search`
- `ChevronRight`
- `Plus`
- `Home`
- `ArrowLeft`
- `CloseProject`
- `Close`
- `Room`
- `Grid`
- `Dimension`
- `Column`
- `Beam`
- `ColumnsAtGrids`
- `GridFrame`
- `SteelBuilding`
- `WallFraming`
- `AlignGridEnds`
- `BeamsOnGrids`
- `BeamsBetweenColumns`
- `Footing`
- `FootingsUnderColumns`
- `StripFooting`
- `FoundationSlab`
- `SlabSupports`
- `Analytical`
- `Truss`
- `Brace`
- `Connection`
- `Rebar`
- `RebarView`
- `StructuralPlan`
- `NumberMarks`
- `StructureTags`
- `Stair`
- `Ramp`
- `Railing`
- `Component`
- `Move`
- `Rotate`
- `MirrorPick`
- `MirrorDraw`
- `Array`
- `TrimCorner`
- `TrimExtend`
- `Split`
- `Offset`
- `Align`
- `Pin`
- `Unpin`
- `Join`
- `JoinGeometry`
- `UnjoinGeometry`
- `Cut`
- `Paste`
- `MatchType`
- `DimLinear`
- `RefPlane`
- `Connector`
- `Shaft`
- `Topography`
- `Pad`
- `AttachTop`
- `Finish`
- `Extrusion`
- `DrawLine`
- `DrawRect`
- `DrawPolygon`
- `DrawPolygonOut`
- `DrawCircle`
- `DrawEllipse`
- `DrawSpline`
- `DrawArc3`
- `DrawArcCenter`
- `DrawArcTangent`
- `DrawPoint`
- `EditBoundary`
- `EditProfile`
- `Revolution`
- `Sweep`
- `Blend`
- `Equal`
- `DimOrdinate`
- `DimAngular`
- `DimRadial`
- `DimDiameter`
- `Pipe`
- `Conduit`
- `CableTray`
- `Valve`
- `LightningConductor`
- `AirTerminal`
- `EarthElectrode`
- `EarthRing`
- `LpsCheck`
- `DataOutlet`
- `Rack`
- `Cabling`
- `LightningPlan`
- `CommunicationsPlan`
- `PlumbingFixture`
- `Outlet`
- `PlumbingPlan`
- `ElectricalPlan`
- `Systems`
- `Connect`
- `Sizing`
- `Visibility`
- `Filter`
- `ViewTemplate`
- `Sun`
- `Render`
- `Hide`
- `Isolate`
- `OverrideGraphics`
- `ResetTemporary`
- `Phases`
- `DesignOptions`
- `ActiveOption`
- `Demolish`
- `Paint`
- `CurtainGrid`
- `Image`
- `Library`
- `Settings`
- `Keyboard`
- `Interface`
- `CloseViews`
- `Mass`
- `PushPull`
- `CadImport`
- `Plugin`
- `Graph`

**Methods**

- `pub const ALL: [Icon; 188]`

<a id="cc_app-icons-badge"></a>

### enum `Badge`

```rust
pub enum Badge
```

A small mark in the bottom-right corner of an icon telling what kind
of action it is.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`

**Variants**

- `New`

  Creates something new.

- `Import`

  Brings data in from a file.

- `Export`

  Writes data out to a file.

- `Remove`

  Removes something.


<a id="cc_app-icons-tones"></a>

### struct `Tones`

```rust
pub struct Tones
```

Colors of a two-tone icon and its badge.

**Traits:** `Debug`, `Clone`, `Copy`

**Fields**

- `ink: Color32`
- `tint: Color32`
- `badge: Color32`

  Badge background; the badge glyph uses `badge_glyph`.

- `badge_glyph: Color32`

**Methods**

- `pub fn of(palette: &crate::theme::Palette, badge: Option<Badge>, enabled: bool, selected: bool) -> Tones`

  Theme tones of an icon in a given state; badges take their semantic color.


<a id="cc_app-icons-paint"></a>

### fn `paint`

```rust
pub fn paint(painter: &Painter, rect: Rect, icon: Icon, color: Color32)
```

Paint `icon` centered in `rect` (the largest square that fits), in one color.

<a id="cc_app-icons-paint_toned"></a>

### fn `paint_toned`

```rust
pub fn paint_toned(painter: &Painter, rect: Rect, icon: Icon, tones: Tones, badge: Option<Badge>)
```

Paint `icon` in two tones, with an optional badge.

<a id="mod-cc_app-jobs"></a>

## `cc_app::jobs`

<sub>`crates/cc_app/src/jobs.rs`</sub>

Long commands (opening a project, importing IFC, DWG/DXF, SketchUp,
pictures, component files) run on a worker thread over a copy of the
document, while a modal dialog shows what is being read and for how long.
The app keeps drawing the document as it was; when the command ends, its
document comes back to the UI thread, which installs it and reports.
Only one job runs at a time, and while it runs the dialog blocks the
interface and `busy` keeps keyboard shortcuts and dropped files out, so
nothing edits the document the job's copy will replace.

**Items:** [`Base`](#cc_app-jobs-base) · [`Finish`](#cc_app-jobs-finish) · [`Job`](#cc_app-jobs-job) · [`busy`](#cc_app-jobs-busy) · [`start`](#cc_app-jobs-start) · [`poll`](#cc_app-jobs-poll) · [`wait`](#cc_app-jobs-wait) · [`cancel`](#cc_app-jobs-cancel) · [`show`](#cc_app-jobs-show)

<a id="cc_app-jobs-base"></a>

### enum `Base`

```rust
pub enum Base
```

What the command runs on.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Variants**

- `Fresh`

  A new document (opening a project, importing an IFC model): the
  result replaces the open one, as the finish step decides.

- `Current`

  A copy of the open document, which the result replaces (with its
  undo history and the command's undo step) when the command succeeds.


<a id="cc_app-jobs-finish"></a>

### type alias `Finish`

```rust
pub type Finish = Box<dyn FnOnce(&mut AppState, Result<Value, EngineError>, Option<Document>)>
```

The step that runs on the UI thread when the command ends: the answer,
and for a `Base::Fresh` job that succeeded, the new document.

<a id="cc_app-jobs-job"></a>

### struct `Job`

```rust
pub struct Job
```

A running job.

*Some fields are private.*

<a id="cc_app-jobs-busy"></a>

### fn `busy`

```rust
pub fn busy(state: &AppState) -> bool
```

A job is running: shortcuts, dropped files and new jobs wait.

<a id="cc_app-jobs-start"></a>

### fn `start`

```rust
pub fn start(state: &mut AppState, title: String, path: PathBuf, base: Base, run: impl FnOnce(&mut Session) -> Result<Value, EngineError> + Send + 'static, finish: Finish)
```

Run `run` on a worker thread over the document `base` names, showing the
progress dialog titled `title` for `path`; `finish` reports the result.

<a id="cc_app-jobs-poll"></a>

### fn `poll`

```rust
pub fn poll(state: &mut AppState)
```

Install the result of a job that ended; call it once per frame.

<a id="cc_app-jobs-wait"></a>

### fn `wait`

```rust
pub fn wait(state: &mut AppState)
```

Wait for the running job and install its result (at startup, before the
window shows, and in tests).

<a id="cc_app-jobs-cancel"></a>

### fn `cancel`

```rust
pub fn cancel(state: &mut AppState)
```

Stop waiting for the running job: its result is dropped and the document stays as it was.

<a id="cc_app-jobs-show"></a>

### fn `show`

```rust
pub fn show(ctx: &Context, state: &mut AppState)
```

The progress dialog of the running job.

<a id="mod-cc_app-level_mark_ui"></a>

## `cc_app::level_mark_ui`

<sub>`crates/cc_app/src/level_mark_ui.rs`</sub>

Level marks in the app: the tool that places them in plans, elevations
and sections (through `level_mark_create`), their rows in the properties
palette and the rows of their type.

**Items:** [`hosts`](#cc_app-level_mark_ui-hosts) · [`tool`](#cc_app-level_mark_ui-tool) · [`props`](#cc_app-level_mark_ui-props) · [`type_rows`](#cc_app-level_mark_ui-type_rows)

<a id="cc_app-level_mark_ui-hosts"></a>

### fn `hosts`

```rust
pub fn hosts(view: &View) -> bool
```

Whether level marks go in `view`: plans, elevations and sections.

<a id="cc_app-level_mark_ui-tool"></a>

### fn `tool`

```rust
pub fn tool(ui: &Ui, resp: &Response, painter: &Painter, state: &mut AppState, view: &View, type_id: ElementTypeId, xf: &Xf, drawing: &Drawing2D)
```

The Level Mark tool: the levels a mark would write follow the cursor; a
click places one.

<a id="cc_app-level_mark_ui-props"></a>

### fn `props`

```rust
pub fn props(ui: &mut Ui, state: &mut AppState, id: ElementId, mark: &LevelMark)
```

A level mark in the properties palette: its view, the side of its
values, the levels it writes (measured, or set) and whether the raw one shows.

<a id="cc_app-level_mark_ui-type_rows"></a>

### fn `type_rows`

```rust
pub fn type_rows(ui: &mut Ui, state: &mut AppState, tid: ElementTypeId, data: &TypeData)
```

The rows of a level mark type: text size, datum and the suffixes.

<a id="mod-cc_app-library"></a>

## `cc_app::library`

<sub>`crates/cc_app/src/library.rs`</sub>

Component library: browse the component files (`.ccomp`) of the folders
the user adds (such as a copy of the ConstructCraft Components library),
search them, see a component's types, parameters, connectors and plan,
load it into the project, and save the project's components into a
library folder. Listing, loading and saving are engine commands.

**Items:** [`Entry`](#cc_app-library-entry) · [`Library`](#cc_app-library-library) · [`open`](#cc_app-library-open) · [`show`](#cc_app-library-show)

<a id="cc_app-library-entry"></a>

### struct `Entry`

```rust
pub struct Entry
```

A component file found in a library folder (from `component_library_list`).

**Traits:** `Debug`, `Clone`, `Default`

**Fields**

- `path: PathBuf`
- `component: String`
- `kind: String`
- `types: Vec<(String, String)>`
- `parameters: Vec<String>`
- `connectors: Vec<String>`

<a id="cc_app-library-library"></a>

### struct `Library`

```rust
pub struct Library
```

State of the open library window.

**Traits:** `Debug`, `Clone`, `Default`

**Fields**

- `entries: Vec<Entry>`
- `errors: usize`

  Files that could not be read.

- `search: String`
- `kind: Option<String>`

  Only this kind (by its `Debug` name), or all.

- `selected: Option<PathBuf>`
- `outlines: Vec<Vec<DVec2>>`

  Plan outlines of the selected component (for its thumbnail).

- `save_component: Option<String>`

  The project component to save into the library, and the folder.

- `save_folder: Option<PathBuf>`

*Some fields are private.*

<a id="cc_app-library-open"></a>

### fn `open`

```rust
pub fn open(state: &mut AppState)
```

Open the library window, reading the folders.

<a id="cc_app-library-show"></a>

### fn `show`

```rust
pub fn show(ctx: &Context, state: &mut AppState)
```

<a id="mod-cc_app-line_styles_dialog"></a>

## `cc_app::line_styles_dialog`

<sub>`crates/cc_app/src/line_styles_dialog.rs`</sub>

The Line Styles dialog (*Annotate → Line Styles*): the built-in line
styles, read only, and the project's own, whose name, weight, color and
dash pattern are edited in place; *New* adds one and *Delete* removes
the selected one (what uses it moves to the thin style). Every change
runs a `line_style_*` command; OK keeps them, Cancel undoes them all
(`dialog::session`, through `crate::graphics_dialog::Edits`).

**Items:** [`LineStylesDialog`](#cc_app-line_styles_dialog-linestylesdialog) · [`open`](#cc_app-line_styles_dialog-open) · [`show`](#cc_app-line_styles_dialog-show)

<a id="cc_app-line_styles_dialog-linestylesdialog"></a>

### struct `LineStylesDialog`

```rust
pub struct LineStylesDialog
```

The open Line Styles dialog.

*Some fields are private.*

**Methods**

- `pub fn edits_finish(self, state: &mut AppState, keep: bool)`

  Close the dialog's edits: keep them (OK) or undo them (Cancel).


<a id="cc_app-line_styles_dialog-open"></a>

### fn `open`

```rust
pub fn open(state: &mut AppState)
```

<a id="cc_app-line_styles_dialog-show"></a>

### fn `show`

```rust
pub fn show(ctx: &Context, state: &mut AppState)
```

<a id="mod-cc_app-lps_cabling_ui"></a>

## `cc_app::lps_cabling_ui`

<sub>`crates/cc_app/src/lps_cabling_ui.rs`</sub>

Lightning protection and structured cabling in the app: the tools that
place air terminals, earth electrodes, racks and data outlets (their
built-in types made on first use by `device_type_create`), the earth
ring around the walls, the coverage check (`lps_check`) and the cabling
per rack (`cabling_list`). Every change is an engine command.

**Items:** [`device_tool`](#cc_app-lps_cabling_ui-device_tool) · [`earth_ring`](#cc_app-lps_cabling_ui-earth_ring) · [`show`](#cc_app-lps_cabling_ui-show) · [`meters_text`](#cc_app-lps_cabling_ui-meters_text)

<a id="cc_app-lps_cabling_ui-device_tool"></a>

### fn `device_tool`

```rust
pub fn device_tool(state: &mut AppState, device: DeviceKind)
```

Start placing devices of `device`: its type (made when the project has
none) in the component tool, in the services plan that shows it.

<a id="cc_app-lps_cabling_ui-earth_ring"></a>

### fn `earth_ring`

```rust
pub fn earth_ring(state: &mut AppState)
```

An earth ring around the walls of the active level (or the lowest).

<a id="cc_app-lps_cabling_ui-show"></a>

### fn `show`

```rust
pub fn show(ctx: &Context, state: &mut AppState)
```

<a id="cc_app-lps_cabling_ui-meters_text"></a>

### fn `meters_text`

```rust
pub fn meters_text(v: &Value, decimals: usize) -> String
```

A length in meters from a command's output: `12,5 m` in metric projects
(as always), feet and inches in imperial ones.

<a id="mod-cc_app-material_browser"></a>

## `cc_app::material_browser`

<sub>`crates/cc_app/src/material_browser.rs`</sub>

Material browser: the project's materials in a searchable list and the
selected one's identity, graphics, appearance and physical properties
in tabs. Edits are undoable and the dialog's Cancel reverts them all.

**Items:** [`Tab`](#cc_app-material_browser-tab) · [`Browser`](#cc_app-material_browser-browser) · [`show`](#cc_app-material_browser-show) · [`KEYS`](#cc_app-material_browser-keys)

<a id="cc_app-material_browser-tab"></a>

### enum `Tab`

```rust
pub enum Tab
```

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Default`

**Variants**

- `Identity`
- `Graphics`
- `Appearance`
- `Physical`

<a id="cc_app-material_browser-browser"></a>

### struct `Browser`

```rust
pub struct Browser
```

Browser state kept between frames.

**Traits:** `Debug`, `Clone`, `Default`

**Fields**

- `selected: Option<String>`
- `tab: Tab`
- `search: String`
- `name_edit: Option<String>`

  Name being typed, committed on Enter or when the field loses focus.

- `error: Option<String>`

<a id="cc_app-material_browser-show"></a>

### fn `show`

```rust
pub fn show(ctx: &Context, state: &mut AppState)
```

<a id="cc_app-material_browser-keys"></a>

### const `KEYS`

```rust
pub const KEYS: [&str; 12]
```

Keys used at runtime (checked by the i18n tests).

<a id="mod-cc_app-mcp_bridge"></a>

## `cc_app::mcp_bridge`

<sub>`crates/cc_app/src/mcp_bridge.rs`</sub>

`constructcraft mcp` (or `--mcp`): MCP clients pointed at the app instead of `ccad` get the
MCP server of the `ccad` next to it, on the same standard input and output, instead of a
window that never answers (issue #265).

**Items:** [`mcp_args`](#cc_app-mcp_bridge-mcp_args) · [`run`](#cc_app-mcp_bridge-run)

<a id="cc_app-mcp_bridge-mcp_args"></a>

### fn `mcp_args`

```rust
pub fn mcp_args(args: &[String]) -> Option<Vec<String>>
```

The `ccad` arguments for the app's arguments, when they ask for the MCP server.

<a id="cc_app-mcp_bridge-run"></a>

### fn `run`

```rust
pub fn run(args: Vec<String>) -> i32
```

Runs `ccad mcp` with the app's standard streams and returns its exit code.

<a id="mod-cc_app-member_host_ui"></a>

## `cc_app::member_host_ui`

<sub>`crates/cc_app/src/member_host_ui.rs`</sub>

Rows shared by the properties of member hosts (steel buildings,
trusses): the member counts and *Reset Members*, and length rows that
report when an edit ends.

**Items:** [`finished`](#cc_app-member_host_ui-finished) · [`length`](#cc_app-member_host_ui-length) · [`member_rows`](#cc_app-member_host_ui-member_rows)

<a id="cc_app-member_host_ui-finished"></a>

### fn `finished`

```rust
pub(crate) fn finished(r: &egui::Response) -> bool
```

Whether a drag or a typed value ended.

<a id="cc_app-member_host_ui-length"></a>

### fn `length`

```rust
pub(crate) fn length(ui: &mut Ui, label: &str, v: &mut f64, range: RangeInclusive<f64>) -> bool
```

A length row; true when the edit ended.

<a id="cc_app-member_host_ui-member_rows"></a>

### fn `member_rows`

```rust
pub(crate) fn member_rows(ui: &mut Ui, host: &dyn MemberHost) -> bool
```

The members and detached counts and *Reset Members* (enabled when a
role is detached); true when it was clicked: the caller clears the
host's detached roles in its own edit.

<a id="mod-cc_app-modify"></a>

## `cc_app::modify`

<sub>`crates/cc_app/src/modify.rs`</sub>

Transform tools on the selection: move, copy, rotate, mirror (picking a
line or drawing the axis) and linear array. Each works in plan views,
previews the result as a ghost of the selection's linework and applies
in one undo step.

**Items:** [`ModifyOptions`](#cc_app-modify-modifyoptions) · [`start`](#cc_app-modify-start) · [`start_pick`](#cc_app-modify-start_pick) · [`paint_ghost`](#cc_app-modify-paint_ghost) · [`handle`](#cc_app-modify-handle) · [`apply_base_target`](#cc_app-modify-apply_base_target) · [`apply_rotation`](#cc_app-modify-apply_rotation) · [`axis_of`](#cc_app-modify-axis_of) · [`apply_mirror`](#cc_app-modify-apply_mirror) · [`alignable`](#cc_app-modify-alignable) · [`align_edge`](#cc_app-modify-align_edge) · [`align_command`](#cc_app-modify-align_command) · [`join_tool`](#cc_app-modify-join_tool) · [`start_attach`](#cc_app-modify-start_attach) · [`attach_target`](#cc_app-modify-attach_target) · [`attach_tool`](#cc_app-modify-attach_tool) · [`attach_pick`](#cc_app-modify-attach_pick) · [`attach_to`](#cc_app-modify-attach_to) · [`detach_tops`](#cc_app-modify-detach_tops) · [`join_pick`](#cc_app-modify-join_pick) · [`set_pinned`](#cc_app-modify-set_pinned)

<a id="cc_app-modify-modifyoptions"></a>

### struct `ModifyOptions`

```rust
pub struct ModifyOptions
```

Options of the transform tools, kept between uses.

**Traits:** `Debug`, `Clone`, `Default`

**Fields**

- `multiple: bool`

  Copy keeps placing copies from the same base point.

- `rotate_copy: bool`

  Rotate and mirror make a copy instead of moving.

- `mirror_copy: bool`
- `count: u32`

  Number of items in an array, including the original.

- `angle: f64`

  Angle typed in the options bar, degrees.

- `offset: f64`

  Offset distance, meters.

- `point_elevation: f64`

  Elevation of the next terrain point a sketch places, meters above
  the level (kept from one sketch to the next).

- `pad_elevation: f64`

  Elevation of the next building pad sketched, meters above the level.

- `wall_face: Option<cc_core::LocationLine>`

  The face of the walls a sketch's *From Walls* follows (`None`: the
  outside finish faces, the inside ones for ceilings).


<a id="cc_app-modify-start"></a>

### fn `start`

```rust
pub fn start(state: &mut AppState, tool: Tool)
```

Start a transform tool on the current selection; without one, explain.

<a id="cc_app-modify-start_pick"></a>

### fn `start_pick`

```rust
pub fn start_pick(state: &mut AppState, tool: Tool)
```

Start a pick tool (trim, split, offset, align): it acts on clicked elements.

<a id="cc_app-modify-paint_ghost"></a>

### fn `paint_ghost`

```rust
pub(crate) fn paint_ghost(painter: &Painter, xf: &Xf, drawing: &Drawing2D, ids: &BTreeSet<ElementId>, t: &PlanTransform)
```

<a id="cc_app-modify-handle"></a>

### fn `handle`

```rust
pub fn handle(ui: &Ui, resp: &Response, painter: &Painter, state: &mut AppState, level: LevelId, xf: &Xf, drawing: &Drawing2D, tool: &Tool)
```

<a id="cc_app-modify-apply_base_target"></a>

### fn `apply_base_target`

```rust
pub(crate) fn apply_base_target(state: &mut AppState, tool: &Tool, targets: &BTreeSet<ElementId>, delta: DVec2)
```

Move, copy or array the targets by `delta` (the target point less the
base point), as plans and the 3D view pick them.

<a id="cc_app-modify-apply_rotation"></a>

### fn `apply_rotation`

```rust
pub fn apply_rotation(state: &mut AppState, targets: &BTreeSet<ElementId>, center: DVec2, angle: f64)
```

Rotate (or copy rotated) by `angle` radians about `center`, and finish.

<a id="cc_app-modify-axis_of"></a>

### fn `axis_of`

```rust
pub(crate) fn axis_of(state: &AppState, id: ElementId) -> Option<(DVec2, DVec2)>
```

The line of a model element usable as a mirror axis (walls, grids, beams).

<a id="cc_app-modify-apply_mirror"></a>

### fn `apply_mirror`

```rust
pub(crate) fn apply_mirror(state: &mut AppState, targets: &BTreeSet<ElementId>, a: DVec2, b: DVec2)
```

Mirror (or copy mirrored) across the line through `a` and `b`, and finish.

<a id="cc_app-modify-alignable"></a>

### fn `alignable`

```rust
pub(crate) fn alignable(state: &AppState, id: ElementId) -> bool
```

Whether an element can be aligned or aligned to: it has a location line
(wall, beam, grid) or vertical faces to align by (column, component).

<a id="cc_app-modify-align_edge"></a>

### fn `align_edge`

```rust
pub(crate) fn align_edge(state: &AppState, id: ElementId, at: DVec2) -> Option<(DVec2, DVec2)>
```

The plan line Align uses when element `id` is picked at `at`, as the
engine does: the face or side nearest to the pick, else its location line.

<a id="cc_app-modify-align_command"></a>

### fn `align_command`

```rust
pub(crate) fn align_command(id: ElementId, pick: DVec2, reference: ElementId, reference_pick: DVec2) -> cmd::ElementAlign
```

Align `id` (picked at `pick`) onto `reference` (picked at `reference_pick`), in plan.

<a id="cc_app-modify-join_tool"></a>

### fn `join_tool`

```rust
pub fn join_tool(resp: &Response, painter: &Painter, state: &mut AppState, xf: &Xf, drawing: &Drawing2D, unjoin: bool)
```

Join or unjoin geometry: the first pick stays, and each further pick is
joined with it (or unjoined from it) until Esc.

<a id="cc_app-modify-start_attach"></a>

### fn `start_attach`

```rust
pub fn start_attach(state: &mut AppState)
```

Attach Top: the selected walls wait for a click on the roof, ceiling or
floor their tops attach to.

<a id="cc_app-modify-attach_target"></a>

### fn `attach_target`

```rust
pub fn attach_target(state: &AppState, id: ElementId) -> bool
```

Whether `id` is something a wall's top attaches to.

<a id="cc_app-modify-attach_tool"></a>

### fn `attach_tool`

```rust
pub fn attach_tool(resp: &Response, painter: &Painter, state: &mut AppState, xf: &Xf, drawing: &Drawing2D)
```

The Attach Top tool: the roof, ceiling or floor under the pointer is
highlighted, and a click attaches the walls' tops to it.

<a id="cc_app-modify-attach_pick"></a>

### fn `attach_pick`

```rust
pub(crate) fn attach_pick(state: &mut AppState, picked: Option<ElementId>)
```

A click of the Attach Top tool in a 3D view on `picked`: a roof, ceiling
or floor takes the walls' tops; anything else repeats the hint.

<a id="cc_app-modify-attach_to"></a>

### fn `attach_to`

```rust
pub(crate) fn attach_to(state: &mut AppState, target: ElementId)
```

Attach the walls the tool holds to `target`, then go back to selecting them.

<a id="cc_app-modify-detach_tops"></a>

### fn `detach_tops`

```rust
pub fn detach_tops(state: &mut AppState)
```

Detach Top: the selected walls keep their own tops again.

<a id="cc_app-modify-join_pick"></a>

### fn `join_pick`

```rust
pub(crate) fn join_pick(state: &mut AppState, hovered: Option<ElementId>, unjoin: bool)
```

A click of the join tool on `hovered` (a joinable element or nothing):
the first pick stays, each further one is joined with it (or unjoined).

<a id="cc_app-modify-set_pinned"></a>

### fn `set_pinned`

```rust
pub fn set_pinned(state: &mut AppState, pinned: bool)
```

Pin or unpin the selection (one undo step).

<a id="mod-cc_app-modify3d"></a>

## `cc_app::modify3d`

<sub>`crates/cc_app/src/modify3d.rs`</sub>

The modify tools in the 3D view: move, copy, array, rotate and mirror
the selection, align and join geometry. Points are picked on a
horizontal working plane (the level of the elements changed, then the
height of the first point picked) or snapped to model points near the
pointer on screen: corners of solids, column and component insertion
points, wall ends and grid crossings. Every result is a plan transform
run through the same engine commands as in plans, so elements keep their
levels: a level 2 column can be moved or aligned onto a ground-floor one.

**Items:** [`PickCache`](#cc_app-modify3d-pickcache) · [`works_in_3d`](#cc_app-modify3d-works_in_3d) · [`active_3d`](#cc_app-modify3d-active_3d) · [`Frame`](#cc_app-modify3d-frame) · [`ray_plane_z`](#cc_app-modify3d-ray_plane_z) · [`nearest_on_screen`](#cc_app-modify3d-nearest_on_screen) · [`snap_points`](#cc_app-modify3d-snap_points) · [`ortho`](#cc_app-modify3d-ortho) · [`handle`](#cc_app-modify3d-handle)

<a id="cc_app-modify3d-pickcache"></a>

### struct `PickCache`

```rust
pub struct PickCache
```

Cached pick data of one 3D view.

**Traits:** `Default`

*Some fields are private.*

<a id="cc_app-modify3d-works_in_3d"></a>

### fn `works_in_3d`

```rust
pub fn works_in_3d(tool: &Tool) -> bool
```

Whether `tool` works in the 3D view (it then takes the clicks there).

<a id="cc_app-modify3d-active_3d"></a>

### fn `active_3d`

```rust
pub fn active_3d(state: &AppState) -> bool
```

Whether the active view is a 3D view.

<a id="cc_app-modify3d-frame"></a>

### struct `Frame`

```rust
pub struct Frame
```

The camera of a 3D view as drawn in `rect`.

**Traits:** `Debug`, `Clone`, `Copy`

**Fields**

- `rect: Rect`
- `camera: CameraState`

*Some fields are private.*

**Methods**

- `pub fn new(rect: Rect, camera: CameraState) -> Self`
- `pub fn ray(&self, pos: Pos2) -> Ray`

  The ray through a screen position.

- `pub fn screen(&self, p: DVec3) -> Option<Pos2>`

  Where a world point shows on screen; `None` behind the eye.

- `pub fn segment(&self, a: DVec3, b: DVec3) -> Option<(Pos2, Pos2)>`

  The part of the segment `a`–`b` in front of the camera's near plane,
  on screen: an end behind it moves to where the segment crosses it
  (`None` when the whole segment is behind), so a line passing by the
  eye is not flung across the screen.


<a id="cc_app-modify3d-ray_plane_z"></a>

### fn `ray_plane_z`

```rust
pub fn ray_plane_z(ray: Ray, z: f64) -> Option<DVec3>
```

Where `ray` meets the horizontal plane at height `z` (in front of its origin).

<a id="cc_app-modify3d-nearest_on_screen"></a>

### fn `nearest_on_screen`

```rust
pub fn nearest_on_screen(frame: &Frame, points: impl IntoIterator<Item = DVec3>, cursor: Pos2, radius: f32) -> Option<DVec3>
```

The point that shows nearest to `cursor`, within `radius` points; of
points showing (about) as near, the one closest to the eye.

<a id="cc_app-modify3d-snap_points"></a>

### fn `snap_points`

```rust
pub fn snap_points(project: &Project, geom: &ModelGeometry, shown: &mut dyn FnMut(ElementId) -> bool) -> Vec<DVec3>
```

Model points to snap to: corners of the solids shown, the insertion
points of columns and components and the ends of walls, at the bottom
and top of each.

<a id="cc_app-modify3d-ortho"></a>

### fn `ortho`

```rust
pub fn ortho(a: DVec2, p: DVec2) -> DVec2
```

Keep the plan offset from `a` to `p` along its main axis (Shift).

<a id="cc_app-modify3d-handle"></a>

### fn `handle`

```rust
pub fn handle(ui: &Ui, resp: &Response, painter: &Painter, state: &mut AppState, view_id: ViewId, frame: Frame)
```

Run the active tool in a 3D view (after the scene is painted).

<a id="mod-cc_app-new_component_dialog"></a>

## `cc_app::new_component_dialog`

<sub>`crates/cc_app/src/new_component_dialog.rs`</sub>

The New Component dialog: a name and the kind the component is, picked
from a list that shows the parameters each kind starts with (a generic
component starts with none: they are the user's to add). Create makes
the component through `component_create` and opens it in the component
environment, where it is shaped; cancelling the environment removes it.

**Items:** [`NewComponent`](#cc_app-new_component_dialog-newcomponent) · [`kinds`](#cc_app-new_component_dialog-kinds) · [`open`](#cc_app-new_component_dialog-open) · [`show`](#cc_app-new_component_dialog-show) · [`create`](#cc_app-new_component_dialog-create) · [`place`](#cc_app-new_component_dialog-place)

<a id="cc_app-new_component_dialog-newcomponent"></a>

### struct `NewComponent`

```rust
pub struct NewComponent
```

The dialog's state: the name typed and the kind picked.

**Traits:** `Debug`, `Clone`

**Fields**

- `name: String`
- `kind: ComponentKind`

<a id="cc_app-new_component_dialog-kinds"></a>

### fn `kinds`

```rust
pub(crate) fn kinds() -> Vec<ComponentKind>
```

The kinds a new component may be: title blocks and detail components
are 2D and made in their own places.

<a id="cc_app-new_component_dialog-open"></a>

### fn `open`

```rust
pub fn open(state: &mut AppState)
```

Open the dialog (kept as it is when already open).

<a id="cc_app-new_component_dialog-show"></a>

### fn `show`

```rust
pub fn show(ctx: &Context, state: &mut AppState)
```

<a id="cc_app-new_component_dialog-create"></a>

### fn `create`

```rust
pub fn create(state: &mut AppState, name: &str, kind: ComponentKind)
```

Create the component of `kind` and open it in the component environment.
*Cancel Editing* there removes it, and *Finish* picks the tool that
places it.

<a id="cc_app-new_component_dialog-place"></a>

### fn `place`

```rust
pub fn place(state: &mut AppState, type_id: ElementTypeId)
```

Pick the tool that places the component type `type_id`: doors and
windows go in walls.

<a id="mod-cc_app-new_project_dialog"></a>

## `cc_app::new_project_dialog`

<sub>`crates/cc_app/src/new_project_dialog.rs`</sub>

The New Project dialog: the template a new project starts from, metric
or imperial, picked on one of two cards. Create goes on to
`actions::guarded`, so the question about unsaved changes comes after
the choice, and cancelling either leaves the open project as it is.

**Items:** [`open`](#cc_app-new_project_dialog-open) · [`show`](#cc_app-new_project_dialog-show)

<a id="cc_app-new_project_dialog-open"></a>

### fn `open`

```rust
pub fn open(state: &mut AppState)
```

Open the dialog with the template picked last (kept when it is already open).

<a id="cc_app-new_project_dialog-show"></a>

### fn `show`

```rust
pub fn show(ctx: &Context, state: &mut AppState)
```

<a id="mod-cc_app-obj_ui"></a>

## `cc_app::obj_ui`

<sub>`crates/cc_app/src/obj_ui.rs`</sub>

Import OBJ and glTF (*Insert → Import OBJ…*, *Import glTF…*): after the
file is picked, a dialog asks what the format does not say (for OBJ the
units and which axis is up; for both one element or one per object),
then `obj_import` or `gltf_import` runs as a background job.

**Items:** [`ObjOptions`](#cc_app-obj_ui-objoptions) · [`import_obj`](#cc_app-obj_ui-import_obj) · [`import_gltf`](#cc_app-obj_ui-import_gltf) · [`show`](#cc_app-obj_ui-show)

<a id="cc_app-obj_ui-objoptions"></a>

### struct `ObjOptions`

```rust
pub struct ObjOptions
```

The choices of the import dialog, kept for the next import.

**Traits:** `Debug`, `Clone`, `Copy`, `Default`

**Fields**

- `units: ObjUnits`
- `z_up: bool`
- `merge: bool`

<a id="cc_app-obj_ui-import_obj"></a>

### fn `import_obj`

```rust
pub fn import_obj(state: &mut AppState)
```

Pick an .obj file and open the options dialog for it.

<a id="cc_app-obj_ui-import_gltf"></a>

### fn `import_gltf`

```rust
pub fn import_gltf(state: &mut AppState)
```

Pick a .glb or .gltf file and open the options dialog for it.

<a id="cc_app-obj_ui-show"></a>

### fn `show`

```rust
pub fn show(ctx: &Context, state: &mut AppState)
```

The options dialog of a picked file; OK starts the import.

<a id="mod-cc_app-opening_place"></a>

## `cc_app::opening_place`

<sub>`crates/cc_app/src/opening_place.rs`</sub>

Placing a door or window on a wall in a plan: where its center goes
along the wall, from the cursor, a snap of its center or one of its
edges to an imported drawing or the underlay (issue #251), or a
distance typed from either end of the wall (issue #250), and the
temporary dimensions from its center to both ends.

**Items:** [`Part`](#cc_app-opening_place-part) · [`Snapped`](#cc_app-opening_place-snapped) · [`Placement`](#cc_app-opening_place-placement) · [`center_range`](#cc_app-opening_place-center_range) · [`place`](#cc_app-opening_place-place) · [`paint_dimensions`](#cc_app-opening_place-paint_dimensions)

<a id="cc_app-opening_place-part"></a>

### enum `Part`

```rust
pub enum Part
```

Which part of the opening a snap lines up.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Variants**

- `Center`
- `StartEdge`

  The edge toward the wall's start.

- `EndEdge`

  The edge toward the wall's end.


<a id="cc_app-opening_place-snapped"></a>

### struct `Snapped`

```rust
pub struct Snapped
```

A feature of a drawing the opening was lined up with: the point snapped
to and the part of the opening on it.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `point: DVec2`
- `part: Part`

<a id="cc_app-opening_place-placement"></a>

### struct `Placement`

```rust
pub struct Placement
```

Where the opening goes: its center along the wall, or why it cannot.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `offset: f64`

  Center along the wall (wall-local `u`).

- `snapped: Option<Snapped>`
- `outside: bool`

  The typed distance places it past an end of the wall.


<a id="cc_app-opening_place-center_range"></a>

### fn `center_range`

```rust
pub fn center_range(g: &WallGeom, width: f64) -> Option<(f64, f64)>
```

The range of centers an opening `width` wide may take on `g`, or `None`
when the wall is too short for it.

<a id="cc_app-opening_place-place"></a>

### fn `place`

```rust
pub fn place(g: &WallGeom, width: f64, (u, v): (f64, f64), features: &CadSnap, r: f64, typed: &TypedInput) -> Option<Placement>
```

The center of an opening `width` wide on `g` for the cursor at wall-local
`(u, v)`: a typed distance first (the first field from the wall's start,
the second, after Tab, from its end; the active one wins when both are
typed), else a snap of its center or an edge to `features` within `r`,
else the cursor; always kept inside the wall. `None` when the wall is too short.

<a id="cc_app-opening_place-paint_dimensions"></a>

### fn `paint_dimensions`

```rust
pub fn paint_dimensions(painter: &Painter, xf: &crate::paint::Xf, g: &WallGeom, offset: f64, side: f64, typed: &TypedInput)
```

Draw the temporary dimensions from the opening's center at `offset` to
both ends of the wall, beside its face on side `side` (1 left, -1
right), with the typed values in place of the measures being typed.

<a id="mod-cc_app-paint"></a>

## `cc_app::paint`

<sub>`crates/cc_app/src/paint.rs`</sub>

Painting `Drawing2D`s with egui.

**Items:** [`HIGHLIGHT`](#cc_app-paint-highlight) · [`PREVIEW`](#cc_app-paint-preview) · [`Xf`](#cc_app-paint-xf) · [`stroke_for`](#cc_app-paint-stroke_for) · [`fill_polygon`](#cc_app-paint-fill_polygon) · [`PaintOptions`](#cc_app-paint-paintoptions) · [`Budget`](#cc_app-paint-budget) · [`frame_left`](#cc_app-paint-frame_left) · [`frame_use`](#cc_app-paint-frame_use) · [`Gate`](#cc_app-paint-gate) · [`Drawn`](#cc_app-paint-drawn) · [`paint_drawing`](#cc_app-paint-paint_drawing) · [`prim_shapes`](#cc_app-paint-prim_shapes) · [`text_culled`](#cc_app-paint-text_culled) · [`image_texture`](#cc_app-paint-image_texture) · [`paint_text`](#cc_app-paint-paint_text)

<a id="cc_app-paint-highlight"></a>

### const `HIGHLIGHT`

```rust
pub const HIGHLIGHT: Color32
```

Highlight on the drawing paper, white in both themes.

<a id="cc_app-paint-preview"></a>

### const `PREVIEW`

```rust
pub const PREVIEW: Color32
```

<a id="cc_app-paint-xf"></a>

### struct `Xf`

```rust
pub struct Xf
```

Maps drawing coordinates to the screen.

**Traits:** `Clone`, `Copy`

**Fields**

- `screen: Pos2`

  Screen position of `origin`.

- `origin: DVec2`
- `scale: f64`

  Screen points per drawing unit.


**Methods**

- `pub fn to_screen(self, p: DVec2) -> Pos2`
- `pub fn to_world(self, s: Pos2) -> DVec2`

<a id="cc_app-paint-stroke_for"></a>

### fn `stroke_for`

```rust
pub fn stroke_for(style: LineStyle, highlighted: bool) -> Stroke
```

<a id="cc_app-paint-fill_polygon"></a>

### fn `fill_polygon`

```rust
pub fn fill_polygon(painter: &Painter, pts: Vec<Pos2>, color: Color32)
```

<a id="cc_app-paint-paintoptions"></a>

### struct `PaintOptions`

```rust
pub struct PaintOptions<'a>
```

**Fields**

- `highlight: &'a BTreeSet<ElementId>`
- `px_per_mm: f64`

  Screen pixels per paper millimeter, for text.

- `mono: Option<Color32>`

  Draw everything in a single color (ghost previews).

- `only: Option<&'a BTreeSet<ElementId>>`

  Only elements in this set (ghost previews).


<a id="cc_app-paint-budget"></a>

### struct `Budget`

```rust
pub(crate) struct Budget
```

The vertices a drawing still may add, and the pixels its tiny lines took.

**Fields**

- `used: usize`
- `refused: bool`

  Whether anything was refused for want of room.


*Some fields are private.*

**Methods**

- `pub(crate) fn with_left(left: usize) -> Self`

  A budget of `left` vertices.


<a id="cc_app-paint-frame_left"></a>

### fn `frame_left`

```rust
pub(crate) fn frame_left(ctx: &egui::Context) -> usize
```

The vertices the drawings painted so far in this frame left.

<a id="cc_app-paint-frame_use"></a>

### fn `frame_use`

```rust
pub(crate) fn frame_use(ctx: &egui::Context, n: usize)
```

Count `n` vertices as used in this frame.

<a id="cc_app-paint-gate"></a>

### enum `Gate`

```rust
pub(crate) enum Gate<'a>
```

How a primitive is let in: by the budget (painting, or building a
cached drawing), or as an earlier build decided (redrawing part of one).

**Variants**

- `Budget(&'a mut Budget)`
- `Decided(bool)`

<a id="cc_app-paint-drawn"></a>

### enum `Drawn`

```rust
pub(crate) enum Drawn
```

What a primitive draws.

**Variants**

- `Nothing`

  Nothing: out of view, too small, left out or out of budget.

- `Shapes`

  The shapes added to the output.

- `Text { at: Pos2, px: f32 }`

  Its text, at this screen point and size (laid out by the caller).


<a id="cc_app-paint-paint_drawing"></a>

### fn `paint_drawing`

```rust
pub fn paint_drawing(painter: &Painter, d: &Drawing2D, xf: &Xf, opts: &PaintOptions)
```

<a id="cc_app-paint-prim_shapes"></a>

### fn `prim_shapes`

```rust
pub(crate) fn prim_shapes(ctx: &egui::Context, d: &Drawing2D, prim: &Prim, xf: &Xf, opts: &PaintOptions, clip: egui::Rect, gate: &mut Gate, out: &mut Vec<Shape>) -> Drawn
```

The shapes of one primitive of `d` on screen (into `out`), unless it lies
outside `clip`, is left out by `opts` or `gate` refuses it. Texts are left
to the caller.

<a id="cc_app-paint-text_culled"></a>

### fn `text_culled`

```rust
pub(crate) fn text_culled(text: &str, px: f32, at: Pos2, clip: egui::Rect) -> bool
```

Whether a text of `px` points at `at` is too small to read or too far
outside `clip` to reach into it.

<a id="cc_app-paint-image_texture"></a>

### fn `image_texture`

```rust
pub(crate) fn image_texture(ctx: &egui::Context, image: &std::sync::Arc<RasterImage>, faded: bool) -> Option<egui::TextureHandle>
```

The texture of `image`, decoded the first time it is shown; `None` when
it cannot be decoded.

<a id="cc_app-paint-paint_text"></a>

### fn `paint_text`

```rust
pub(crate) fn paint_text(painter: &Painter, pos: Pos2, text: &str, px: f32, anchor: cc_views::Anchor, angle: f32)
```

Text anchored at `pos` and rotated by `angle` (radians, counter-clockwise on the drawing).

<a id="mod-cc_app-paint_cache"></a>

## `cc_app::paint_cache`

<sub>`crates/cc_app/src/paint_cache.rs`</sub>

Painted drawings kept between frames.
A large drawing (an imported DXF holds 100 000 primitives) is slow to
turn into shapes and tessellate, and a 2D view repaints on every pointer
move. `PaintCache` keeps each view's drawing tessellated at the view's
zoom, in screen points relative to where it was built: a frame only
moves the meshes by the pan since. It is built again when the drawing
(its shared allocation), the zoom, the text scale or the pixel density
changes, when the view pans past the area it was built for (the drawing
is built for the view and a view's size around it, or all of it when it
fits there), or when the frame's vertex budget changes what fits. A
change of selection redraws only the chunks holding the elements whose
highlight changed. Texts are laid out every frame (egui keeps the
layouts), since their glyphs live in a font atlas that may be rebuilt.

**Items:** [`PaintCache`](#cc_app-paint_cache-paintcache)

<a id="cc_app-paint_cache-paintcache"></a>

### struct `PaintCache`

```rust
pub struct PaintCache
```

Painted drawings by view.

**Traits:** `Default`

*Some fields are private.*

**Methods**

- `pub fn paint<T: 'static>(&mut self, painter: &Painter, view: ViewId, owner: &Rc<T>, d: &Drawing2D, xf: Xf, highlight: &BTreeSet<ElementId>, px_per_mm: f64) -> Xf`

  Paint `d`, the drawing of `view` belonging to `owner`, with `xf` and
  the selection `highlight` (texts at `px_per_mm`). Returns the
  transform it was painted with: `xf` moved by less than a pixel, so
  the cached meshes stay on whole pixels; what is drawn over the
  drawing should use it.


<a id="mod-cc_app-paint_ui"></a>

## `cc_app::paint_ui`

<sub>`crates/cc_app/src/paint_ui.rs`</sub>

The Paint and Remove Paint tools: click a face of an element in a 3D view
to paint it with the material chosen in the options bar (`paint`), or to
take its paint off (`paint_remove`).

**Items:** [`start`](#cc_app-paint_ui-start) · [`options_bar`](#cc_app-paint_ui-options_bar) · [`pick`](#cc_app-paint_ui-pick)

<a id="cc_app-paint_ui-start"></a>

### fn `start`

```rust
pub fn start(state: &mut AppState, remove: bool)
```

Start painting (or removing paint): the material defaults to the last
one used, else the project's first.

<a id="cc_app-paint_ui-options_bar"></a>

### fn `options_bar`

```rust
pub fn options_bar(ui: &mut Ui, state: &mut AppState, remove: bool)
```

The options bar: the material to paint with, and what to do.

<a id="cc_app-paint_ui-pick"></a>

### fn `pick`

```rust
pub fn pick(state: &mut AppState, view: ViewId, remove: bool, picked: Option<ElementId>, ray: cc_geom::Ray)
```

A click in 3D view `view` with the tool: `picked` is the element under
the pointer, `ray` the pointer's ray.

<a id="mod-cc_app-palette"></a>

## `cc_app::palette`

<sub>`crates/cc_app/src/palette.rs`</sub>

Command palette (Ctrl+Shift+P): search every interface command, the
project's views and the engine's command catalog by name, description
or shortcut, and run the chosen one. Engine commands open a small form
with their JSON arguments, the same ones scripts and the MCP server use.

**Items:** [`Palette`](#cc_app-palette-palette) · [`open_command`](#cc_app-palette-open_command) · [`toggle`](#cc_app-palette-toggle) · [`show`](#cc_app-palette-show)

<a id="cc_app-palette-palette"></a>

### struct `Palette`

```rust
pub struct Palette
```

The open palette.

**Traits:** `Default`

*Some fields are private.*

**Methods**

- `pub fn new(session: &cc_engine::Session) -> Palette`

  The palette over the session's commands (built-in and plugins').


<a id="cc_app-palette-open_command"></a>

### fn `open_command`

```rust
pub fn open_command(state: &mut AppState, name: &str)
```

Open the palette on the form of the engine command `name` (a plugin's
command from the ribbon); nothing happens for an unknown name.

<a id="cc_app-palette-toggle"></a>

### fn `toggle`

```rust
pub fn toggle(state: &mut AppState)
```

Open or close the palette.

<a id="cc_app-palette-show"></a>

### fn `show`

```rust
pub fn show(ctx: &egui::Context, state: &mut AppState)
```

<a id="mod-cc_app-phases_dialog"></a>

## `cc_app::phases_dialog`

<sub>`crates/cc_app/src/phases_dialog.rs`</sub>

Construction phases in the application: the Phases dialog (phases,
phase filters and the graphics of each status), the phase rows of view
and element properties, and the Demolish tool. Every change goes through
the engine's commands; OK keeps the dialog's changes, Cancel undoes them.

**Items:** [`PhasesDialog`](#cc_app-phases_dialog-phasesdialog) · [`open`](#cc_app-phases_dialog-open) · [`show`](#cc_app-phases_dialog-show) · [`view_section`](#cc_app-phases_dialog-view_section) · [`element_rows`](#cc_app-phases_dialog-element_rows) · [`demolishable`](#cc_app-phases_dialog-demolishable) · [`demolish`](#cc_app-phases_dialog-demolish) · [`start_demolish`](#cc_app-phases_dialog-start_demolish) · [`demolish_tool`](#cc_app-phases_dialog-demolish_tool)

<a id="cc_app-phases_dialog-phasesdialog"></a>

### struct `PhasesDialog`

```rust
pub struct PhasesDialog
```

The open Phases dialog.

*Some fields are private.*

<a id="cc_app-phases_dialog-open"></a>

### fn `open`

```rust
pub fn open(state: &mut AppState)
```

<a id="cc_app-phases_dialog-show"></a>

### fn `show`

```rust
pub fn show(ctx: &Context, state: &mut AppState)
```

<a id="cc_app-phases_dialog-view_section"></a>

### fn `view_section`

```rust
pub fn view_section(ui: &mut Ui, state: &mut AppState, view: ViewId)
```

The Phasing section of a view's properties (plans, elevations, sections,
3D views and schedules).

<a id="cc_app-phases_dialog-element_rows"></a>

### fn `element_rows`

```rust
pub fn element_rows(ui: &mut Ui, state: &mut AppState, el: &Element)
```

The phases an element is built and demolished in.

<a id="cc_app-phases_dialog-demolishable"></a>

### fn `demolishable`

```rust
pub fn demolishable(state: &AppState, view: ViewId, ids: impl IntoIterator<Item = ElementId>) -> Vec<ElementId>
```

The elements of `ids` standing in the view's phase (existing or new
there), which the view can demolish.

<a id="cc_app-phases_dialog-demolish"></a>

### fn `demolish`

```rust
pub fn demolish(state: &mut AppState, view: ViewId, ids: &[ElementId]) -> bool
```

Demolish `ids` in the view's phase; returns whether it was done.

<a id="cc_app-phases_dialog-start_demolish"></a>

### fn `start_demolish`

```rust
pub fn start_demolish(state: &mut AppState)
```

Starting the Demolish command: a selection standing in the active view's
phase is demolished at once; otherwise the tool waits for clicks or a window.

<a id="cc_app-phases_dialog-demolish_tool"></a>

### fn `demolish_tool`

```rust
pub fn demolish_tool(resp: &egui::Response, painter: &egui::Painter, state: &mut AppState, view: &cc_core::View, xf: &crate::paint::Xf, drawing: &cc_views::Drawing2D)
```

Click elements, or drag a window around them (left to right: inside it;
right to left: touching it), to demolish them in the active view's phase.

<a id="mod-cc_app-plugins_ui"></a>

## `cc_app::plugins_ui`

<sub>`crates/cc_app/src/plugins_ui.rs`</sub>

Plugins in the app: loading the plugins of the plugins folder into the
session (their commands join the engine's catalog, the palette and the
ribbon's *Plugin Commands* menu) and the Plugins window, which turns them
on and off, grants the `files` permission, removes them, and installs and
updates them from the community index.
Plugins run sandboxed in `cc_plugins`; installing is `cc_plugins::store` (checked
archives), and the only network use is here: the index and the archives
it lists, downloaded over HTTPS in background threads with bounded sizes,
only when the user opens the *Available* tab or installs.

**Items:** [`Tab`](#cc_app-plugins_ui-tab) · [`Listed`](#cc_app-plugins_ui-listed) · [`Plugins`](#cc_app-plugins_ui-plugins) · [`load`](#cc_app-plugins_ui-load) · [`drain_messages`](#cc_app-plugins_ui-drain_messages) · [`open`](#cc_app-plugins_ui-open) · [`show`](#cc_app-plugins_ui-show)

<a id="cc_app-plugins_ui-tab"></a>

### enum `Tab`

```rust
pub enum Tab
```

Which list the window shows.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Default`

**Variants**

- `Installed`
- `Available`

<a id="cc_app-plugins_ui-listed"></a>

### struct `Listed`

```rust
pub struct Listed
```

A plugin of the plugins folder, as listed.

**Traits:** `Debug`, `Clone`

**Fields**

- `dir: PathBuf`
- `id: String`
- `name: String`
- `version: String`
- `description: String`
- `authors: Vec<String>`
- `license: String`
- `homepage: String`
- `asks_files: bool`
- `commands: Vec<(String, String)>`

  Its commands, when loaded: `(full name, description)`.

- `error: Option<String>`

  Why it could not be read or loaded.


<a id="cc_app-plugins_ui-plugins"></a>

### struct `Plugins`

```rust
pub struct Plugins
```

Plugins state of the app (kept when the document changes).

**Traits:** `Debug`, `Default`

**Fields**

- `open: bool`

  The Plugins window is open.

- `tab: Tab`
- `listed: Vec<Listed>`

  The plugins of the folder, read at the last load.

- `selected: Option<String>`

  The plugin picked in the window (by id).

- `search: String`
- `notice: Option<Result<String, String>>`

  The last install or removal, for the window.


*Some fields are private.*

**Methods**

- `pub fn commands(&self) -> Vec<(&str, &[(String, String)])>`

  The plugin commands, by plugin: `(plugin name, [(command, description)])`.


<a id="cc_app-plugins_ui-load"></a>

### fn `load`

```rust
pub fn load(state: &mut AppState)
```

Load the plugins of the plugins folder into the session, as the
preferences say (turned off ones are left out and run no code; `files`
only where the user allowed it), and list them for the window.

<a id="cc_app-plugins_ui-drain_messages"></a>

### fn `drain_messages`

```rust
pub fn drain_messages(state: &mut AppState)
```

Every frame: load the plugins again when the plugins folder changed, and
show what plugins printed while their commands ran (logged, the last one
in the status bar).

<a id="cc_app-plugins_ui-open"></a>

### fn `open`

```rust
pub fn open(state: &mut AppState)
```

Open the Plugins window.

<a id="cc_app-plugins_ui-show"></a>

### fn `show`

```rust
pub fn show(ctx: &Context, state: &mut AppState)
```

<a id="mod-cc_app-preferences"></a>

## `cc_app::preferences`

<sub>`crates/cc_app/src/preferences.rs`</sub>

App preferences (not model data): what they hold, how they are kept in
eframe's storage (every key lives here), how the interface follows them,
and the Preferences window (File tab, Ctrl+,, home screen).
Stored values are hostile input: numbers are parsed and clamped, lists
are bounded, and folders that no longer exist are kept but skipped where
they are used. Missing values take defaults that match the behavior
before the window existed.

**Items:** [`UI_SCALES`](#cc_app-preferences-ui_scales) · [`AUTOSAVE_MINUTES`](#cc_app-preferences-autosave_minutes) · [`RECENT_COUNTS`](#cc_app-preferences-recent_counts) · [`MAX_MACROS`](#cc_app-preferences-max_macros) · [`Preferences`](#cc_app-preferences-preferences) · [`load_language`](#cc_app-preferences-load_language) · [`load`](#cc_app-preferences-load) · [`save`](#cc_app-preferences-save) · [`Look`](#cc_app-preferences-look) · [`sync_look`](#cc_app-preferences-sync_look) · [`trim_recent`](#cc_app-preferences-trim_recent) · [`macro_files`](#cc_app-preferences-macro_files) · [`Section`](#cc_app-preferences-section) · [`PreferencesDialog`](#cc_app-preferences-preferencesdialog) · [`open`](#cc_app-preferences-open) · [`show`](#cc_app-preferences-show)

<a id="cc_app-preferences-ui_scales"></a>

### const `UI_SCALES`

```rust
pub const UI_SCALES: [u32; 8]
```

Interface scales offered (percent).

<a id="cc_app-preferences-autosave_minutes"></a>

### const `AUTOSAVE_MINUTES`

```rust
pub const AUTOSAVE_MINUTES: [u32; 6]
```

Autosave intervals offered (minutes; 0 turns autosave off).

<a id="cc_app-preferences-recent_counts"></a>

### const `RECENT_COUNTS`

```rust
pub const RECENT_COUNTS: [usize; 6]
```

Numbers of recent projects offered.

<a id="cc_app-preferences-max_macros"></a>

### const `MAX_MACROS`

```rust
pub const MAX_MACROS: usize
```

The macro menu lists at most this many files.

<a id="cc_app-preferences-preferences"></a>

### struct `Preferences`

```rust
pub struct Preferences
```

The app preferences. Language and the update check's bookkeeping are kept
elsewhere (`i18n`, `Updates`), but stored and restored here too.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `theme: ThemeChoice`
- `check_updates: bool`

  Check for a newer release at startup (at most once a day).

- `ui_scale: u32`

  Interface scale in percent (the egui zoom factor), within `UI_SCALES`'s range.

- `font_size: u32`

  Text size in percent of the normal size, independent of the scale.

- `ribbon_collapsed: bool`

  The ribbon starts collapsed (only its tabs shown).

- `detailed_tooltips: bool`

  Tooltips of commands show what they do and the engine command, not only the name.

- `select_drag_3d: bool`

  A left drag in 3D views draws a selection box (Shift+drag orbits);
  otherwise it orbits and Shift+drag selects.

- `trackpad_pans: bool`

  Two fingers on a trackpad pan the views and a pinch zooms; otherwise
  any scroll zooms, as a mouse wheel does (see `gestures.rs`).

- `cad_blocks: bool`

  Import CAD brings a drawing's blocks in as detail components instead
  of exploding them into the imported drawing.

- `snaps: crate::snaps::SnapSettings`

  Which points the tools snap to.

- `shortcut_keys: BTreeMap<String, String>`

  Custom two-letter keys by command id (`UiCommand::id`); empty keys
  take a command's keys away.

- `library_folders: Vec<PathBuf>`

  Folders of the component library.

- `macro_folder: Option<PathBuf>`

  Where macros are recorded to and listed from.

- `projects_folder: Option<PathBuf>`

  Where the project file dialogs start.

- `autosave_minutes: u32`

  Minutes between autosave copies of unsaved changes (0: off).

- `recent_count: usize`

  How many recent projects the home screen lists.

- `plugin_folder: Option<PathBuf>`

  Where plugins are installed and loaded from (`None`: the default
  folder, see `Preferences::plugins_dir`).

- `plugins_off: BTreeSet<String>`

  Plugins turned off (by id): not loaded.

- `plugins_files: BTreeSet<String>`

  Plugins the user allowed to work with files (the `files` permission).

- `panels: crate::side_panels::PanelLayout`

  Where the Project Browser and Properties are docked.

- `structure_material: cc_core::StructuralMaterial`

  The material the Structure tab's column and beam tools place.

- `new_project_system: cc_core::MeasurementSystem`

  The template last picked in the New Project dialog, picked first next time.


**Methods**

- `pub fn read(get: impl Fn(&str) -> Option<String>, zoom: f32) -> Self`

  Read the preferences with `get` (a storage lookup). `zoom` is the
  interface zoom eframe restored, used when no scale was stored yet
  (it kept the Ctrl+Plus/Minus zoom before this preference existed).

- `pub fn write(&self, mut set: impl FnMut(&str, String))`

  Write the preferences with `set` (a storage write).

- `pub fn autosave_interval(&self) -> Option<Duration>`

  Time between autosave copies, or `None` when autosave is off.

- `pub fn macro_dir(&self) -> Option<&Path>`

  The macros folder, if set and present.

- `pub fn projects_dir(&self) -> Option<&Path>`

  The default projects folder, if set and present.

- `pub fn plugins_dir(&self) -> Option<PathBuf>`

  The plugins folder: the one set, else `plugins` in the app's data
  folder (`None` when there is none).

- `pub fn zoom(&self) -> f32`

  The interface zoom factor.

- `pub fn font_scale(&self) -> f32`

  The font size multiplier.


<a id="cc_app-preferences-load_language"></a>

### fn `load_language`

```rust
pub fn load_language(get: impl Fn(&str) -> Option<String>, language: Option<Language>)
```

Set the UI language at startup: `language` (from the command line), else
the stored one, else the operating system's.

<a id="cc_app-preferences-load"></a>

### fn `load`

```rust
pub fn load(state: &mut AppState, get: impl Fn(&str) -> Option<String>, zoom: f32)
```

Restore every other stored preference into `state` at startup (`zoom`: see `Preferences::read`).

<a id="cc_app-preferences-save"></a>

### fn `save`

```rust
pub fn save(state: &AppState, mut set: impl FnMut(&str, String))
```

Store every preference of `state` (and the language).

<a id="cc_app-preferences-look"></a>

### struct `Look`

```rust
pub struct Look
```

What the interface last applied of the preferences (theme, zoom, font size).

**Traits:** `Debug`, `Clone`, `Copy`

*Some fields are private.*

**Methods**

- `pub fn current(ctx: &Context) -> Self`

  The look `ctx` shows now.


<a id="cc_app-preferences-sync_look"></a>

### fn `sync_look`

```rust
pub fn sync_look(ctx: &Context, prefs: &mut Preferences, look: &mut Look)
```

Make the interface follow the preferences; call it once at the start of
every frame. A zoom changed with Ctrl+Plus/Minus since the last frame
becomes the scale preference.

<a id="cc_app-preferences-trim_recent"></a>

### fn `trim_recent`

```rust
pub fn trim_recent(state: &mut AppState)
```

Keep at most `prefs.recent_count` recent projects.

<a id="cc_app-preferences-macro_files"></a>

### fn `macro_files`

```rust
pub fn macro_files(folder: &Path) -> Vec<PathBuf>
```

The macro files (`.ccmacro`) of `folder`, sorted by name: not recursive,
at most `MAX_MACROS`; a missing or unreadable folder has none.

<a id="cc_app-preferences-section"></a>

### enum `Section`

```rust
pub enum Section
```

A page of the Preferences window.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Default`

**Variants**

- `General`
- `Interface`
- `FileLocations`
- `Saving`
- `Shortcuts`

**Methods**

- `pub const ALL: [Section; 5]`
- `pub fn label(self) -> String`

<a id="cc_app-preferences-preferencesdialog"></a>

### struct `PreferencesDialog`

```rust
pub struct PreferencesDialog
```

State of the open Preferences window. Changes show at once; Cancel
restores the values the window opened with.

**Traits:** `Debug`, `Clone`

**Fields**

- `section: Section`

*Some fields are private.*

<a id="cc_app-preferences-open"></a>

### fn `open`

```rust
pub fn open(state: &mut AppState, section: Section)
```

Open the Preferences window on `section` (or show that section if it is open).

<a id="cc_app-preferences-show"></a>

### fn `show`

```rust
pub fn show(ctx: &Context, state: &mut AppState)
```

<a id="mod-cc_app-profile_sketch"></a>

## `cc_app::profile_sketch`

<sub>`crates/cc_app/src/profile_sketch.rs`</sub>

*Edit Profile*: the profile of a sweep type drawn in a sketch, in the
profile's own plane (x across the path, to its left; y up from the
path's height), which takes the place of the views while it is open.
The sketch is the same as for outlines in plans (its ribbon, tools,
grips, properties and undo); *Finish* runs `sweep_profile_set` with its
curves, so every sweep of the type follows in one undo step.
*Edit sketch* in the Profile Manager opens a sketched structural profile
the same way, around the axes x = 0 and y = 0 of its sketch, with no
level: *Finish* writes its outline and openings with `profile_set`,
keeping the expressions of the points that did not move, and both
*Finish* and *Cancel* go back to the manager.

**Items:** [`target`](#cc_app-profile_sketch-target) · [`curves_of`](#cc_app-profile_sketch-curves_of) · [`open`](#cc_app-profile_sketch-open) · [`edit_selected`](#cc_app-profile_sketch-edit_selected) · [`is_open`](#cc_app-profile_sketch-is_open) · [`section_curves`](#cc_app-profile_sketch-section_curves) · [`open_section`](#cc_app-profile_sketch-open_section) · [`finish_section`](#cc_app-profile_sketch-finish_section) · [`leave_section`](#cc_app-profile_sketch-leave_section) · [`central`](#cc_app-profile_sketch-central)

<a id="cc_app-profile_sketch-target"></a>

### fn `target`

```rust
pub fn target(state: &AppState) -> Option<ElementTypeId>
```

The sweep type *Edit Profile* would open: that of the one selected sweep.

<a id="cc_app-profile_sketch-curves_of"></a>

### fn `curves_of`

```rust
pub fn curves_of(state: &AppState, type_id: ElementTypeId) -> Option<Vec<SketchCurve>>
```

The curves of a sweep type's profile: those it was drawn with, or lines
along its points.

<a id="cc_app-profile_sketch-open"></a>

### fn `open`

```rust
pub fn open(state: &mut AppState, type_id: ElementTypeId)
```

Open the profile of sweep type `type_id` in a sketch.

<a id="cc_app-profile_sketch-edit_selected"></a>

### fn `edit_selected`

```rust
pub fn edit_selected(state: &mut AppState)
```

*Edit Profile* from the ribbon: the selected sweep's type.

<a id="cc_app-profile_sketch-is_open"></a>

### fn `is_open`

```rust
pub fn is_open(state: &AppState) -> bool
```

Whether the open sketch is a sweep's or a structural profile (which then
takes the central area).

<a id="cc_app-profile_sketch-section_curves"></a>

### fn `section_curves`

```rust
pub fn section_curves(project: &Project, id: ProfileId) -> Option<Vec<SketchCurve>>
```

The curves that draw a sketched structural profile: its outline's, then
each opening's.

<a id="cc_app-profile_sketch-open_section"></a>

### fn `open_section`

```rust
pub fn open_section(state: &mut AppState, id: ProfileId) -> bool
```

Open a sketched structural profile in a sketch (*Edit sketch*);
Whether it opened (a profile whose points do not evaluate does not).

<a id="cc_app-profile_sketch-finish_section"></a>

### fn `finish_section`

```rust
pub fn finish_section(state: &mut AppState)
```

*Finish* on a structural profile's sketch: its outline and openings
written with `profile_set` (nothing when they draw the same loops, from
any start and either way round), then
back to the Profile Manager; a sketch that makes no profile stays open
with the reason.

<a id="cc_app-profile_sketch-leave_section"></a>

### fn `leave_section`

```rust
pub fn leave_section(state: &mut AppState, id: ProfileId)
```

Close a structural profile's sketch and go back to the Profile Manager on
it, with the manager's edits from before the sketch.

<a id="cc_app-profile_sketch-central"></a>

### fn `central`

```rust
pub fn central(ui: &mut Ui, state: &mut AppState)
```

The profile's plane in the central area: the path's point and the axes,
the sketch and its tools; middle or right drag pans, the wheel zooms, a
middle double click fits.

<a id="mod-cc_app-profiles_dialog"></a>

## `cc_app::profiles_dialog`

<sub>`crates/cc_app/src/profiles_dialog.rs`</sub>

The Profile Manager (*Structure → Profiles*, also *Manage → Profiles*):
the project's structural profiles grouped by material, with a search;
the selected one's outline drawn to fit, its name, material and
dimensions (millimeters on screen, meters to the engine) and its section
properties. *New* makes a profile of a standard shape, *Duplicate* copies
one, *Delete* removes one (with the profile that takes its place in the
types using it) and *Import from catalog* copies rows of the built-in
series in. Every change is a `profile_*` command; OK keeps them, Cancel
undoes them all (`dialog::session`, through `crate::graphics_dialog::Edits`).
What is shown is read back with `profiles_list` and `profile_get`.

**Items:** [`shape_key`](#cc_app-profiles_dialog-shape_key) · [`param_key`](#cc_app-profiles_dialog-param_key) · [`ProfilesDialog`](#cc_app-profiles_dialog-profilesdialog) · [`Catalog`](#cc_app-profiles_dialog-catalog) · [`open`](#cc_app-profiles_dialog-open) · [`section_sketch_open`](#cc_app-profiles_dialog-section_sketch_open) · [`settle_abandoned`](#cc_app-profiles_dialog-settle_abandoned) · [`resume`](#cc_app-profiles_dialog-resume) · [`show`](#cc_app-profiles_dialog-show)

<a id="cc_app-profiles_dialog-shape_key"></a>

### fn `shape_key`

```rust
pub fn shape_key(code: &str) -> String
```

The i18n key of a shape code's name (`profile-shape-cold-lipped-u`).

<a id="cc_app-profiles_dialog-param_key"></a>

### fn `param_key`

```rust
pub fn param_key(name: &str) -> String
```

The i18n key of what a shape parameter measures (`profile-param-bf-top`).

<a id="cc_app-profiles_dialog-profilesdialog"></a>

### struct `ProfilesDialog`

```rust
pub struct ProfilesDialog
```

The open Profile Manager.

**Fields**

- `selected: Option<ProfileId>`
- `catalog: Option<Catalog>`

*Some fields are private.*

<a id="cc_app-profiles_dialog-catalog"></a>

### struct `Catalog`

```rust
pub struct Catalog
```

*Import from catalog*: a series, a filter and the designations picked.

**Fields**

- `series: String`

  The series' code.


*Some fields are private.*

<a id="cc_app-profiles_dialog-open"></a>

### fn `open`

```rust
pub fn open(state: &mut AppState)
```

Open the Profile Manager; not while one of its profile sketches is open
(the manager comes back when that sketch closes).

<a id="cc_app-profiles_dialog-section_sketch_open"></a>

### fn `section_sketch_open`

```rust
pub fn section_sketch_open(state: &AppState) -> bool
```

Whether a structural profile's sketch is open (the manager waits for it).

<a id="cc_app-profiles_dialog-settle_abandoned"></a>

### fn `settle_abandoned`

```rust
pub fn settle_abandoned(state: &mut AppState)
```

Keep the manager's edits when its profile sketch closed without going
back to it (another tool, a palette command…), so the shared dialog
session ends and a later dialog's Cancel undoes only its own edits. The
one place that settles them, run when the tool changes and every frame.

<a id="cc_app-profiles_dialog-resume"></a>

### fn `resume`

```rust
pub fn resume(state: &mut AppState, id: ProfileId, edits: Option<Edits>)
```

Open the manager again on profile `id` after its sketch closes, with
`edits`, the ones it had when the sketch opened (a fresh start if none).

<a id="cc_app-profiles_dialog-show"></a>

### fn `show`

```rust
pub fn show(ctx: &Context, state: &mut AppState)
```

<a id="mod-cc_app-properties"></a>

## `cc_app::properties`

<sub>`crates/cc_app/src/properties.rs`</sub>

Properties palette: instance parameters of the selection, or of the active view.
A header block names what is shown (and its type, with "Edit Type"); below
it, parameters are listed in collapsible sections of a two-column table.

**Items:** [`key`](#cc_app-properties-key) · [`body`](#cc_app-properties-body) · [`MultiSummary`](#cc_app-properties-multisummary) · [`underlay_rows`](#cc_app-properties-underlay_rows) · [`underlay_patch`](#cc_app-properties-underlay_patch) · [`length_drag`](#cc_app-properties-length_drag) · [`size_drag`](#cc_app-properties-size_drag) · [`field_label`](#cc_app-properties-field_label) · [`slope_drag`](#cc_app-properties-slope_drag) · [`paper_drag`](#cc_app-properties-paper_drag) · [`weight_drag`](#cc_app-properties-weight_drag) · [`decimal_paper_drag`](#cc_app-properties-decimal_paper_drag) · [`ParamEdit`](#cc_app-properties-paramedit) · [`param_editor`](#cc_app-properties-param_editor) · [`number_drag`](#cc_app-properties-number_drag) · [`param_value`](#cc_app-properties-param_value)

<a id="cc_app-properties-key"></a>

### fn `key`

```rust
pub(crate) fn key(parts: impl Hash) -> u64
```

<a id="cc_app-properties-body"></a>

### fn `body`

```rust
pub fn body(ui: &mut Ui, state: &mut AppState)
```

The properties of the selection (or the active view), under a header drawn by the caller.

<a id="cc_app-properties-multisummary"></a>

### struct `MultiSummary`

```rust
pub struct MultiSummary
```

The selected elements and their parameters as the multi-selection rows read them, kept while
the document and the selection stay the same (copying and serializing every selected element
each frame is costly for large selections).

*Some fields are private.*

<a id="cc_app-properties-underlay_rows"></a>

### fn `underlay_rows`

```rust
pub fn underlay_rows(ui: &mut Ui, state: &AppState, salt: u64, own: Option<cc_core::LevelId>, enabled: bool, u: &mut cc_core::Underlay) -> bool
```

The rows of an underlay (`cc_core::Underlay`): the plan below and the
plan above (none, the next level, or a level), each looking down or up,
and whether it is printed. `own` is the view's level (left out of the
choices), `None` for a template. Returns whether the user changed it.

<a id="cc_app-properties-underlay_patch"></a>

### fn `underlay_patch`

```rust
pub fn underlay_patch(old: &cc_core::Underlay, new: &cc_core::Underlay) -> serde_json::Value
```

The patch of `view_set` or `template_set` that turns a view's or
template's underlay `old` into `new`. Plans set back to none are
cleared (issue #235): the plain JSON of `new` leaves them out, and a
merge would keep the old ones.

<a id="cc_app-properties-length_drag"></a>

### fn `length_drag`

```rust
pub fn length_drag(value: &mut f64) -> DragValue<'_>
```

Number, name, height and computed area of a room.
Drag value for a length stored in meters, shown and typed in the
project's length unit (a typed unit symbol overrides it).

<a id="cc_app-properties-size_drag"></a>

### fn `size_drag`

```rust
pub fn size_drag(value: &mut f64) -> DragValue<'_>
```

Drag value for a small size stored in meters (a layer, a section's side,
a diameter, a riser): a length in metric projects (see `length_drag`),
inches with a fraction in imperial ones (`3/4"`; a bare number typed is
inches, see `format::parse_size`).

<a id="cc_app-properties-field_label"></a>

### fn `field_label`

```rust
pub fn field_label(field: cc_core::ScheduleField) -> String
```

The header of a schedule field as a label in the current units: `Area
(m²)` in metric projects, `Area` in imperial ones, whose values carry
their own symbols (see `cc_core::ScheduleField::key_in`).

<a id="cc_app-properties-slope_drag"></a>

### fn `slope_drag`

```rust
pub fn slope_drag(value: &mut f64) -> DragValue<'_>
```

Drag value for a slope stored as rise over run, shown and typed in the
project's slope unit (`8,3 %`, `4,76°`, `6:12`; a typed unit or ratio
overrides it, see `format::parse_slope`).

<a id="cc_app-properties-paper_drag"></a>

### fn `paper_drag`

```rust
pub fn paper_drag(value: &mut f64) -> DragValue<'_>
```

Drag value for a size on paper stored in millimeters (text heights,
bubbles, gaps, sheet sides): ` mm` after the number in metric
projects; in imperial ones inches with a fraction (`3/32"`), typed in
inches unless a unit says otherwise (see `format::parse_paper`).

<a id="cc_app-properties-weight_drag"></a>

### fn `weight_drag`

```rust
pub fn weight_drag(value: &mut f64) -> DragValue<'_>
```

Drag value for a pen weight stored in millimeters: ` mm` after the
number in metric projects, decimal inches in imperial ones (`0.010"`).

<a id="cc_app-properties-decimal_paper_drag"></a>

### fn `decimal_paper_drag`

```rust
pub fn decimal_paper_drag(value: &mut f64) -> DragValue<'_>
```

Drag value for a paper length shown as decimal inches in imperial
projects (`0.125"`, drafting fill patterns), ` mm` after the number in
metric ones.

<a id="cc_app-properties-paramedit"></a>

### enum `ParamEdit`

```rust
pub enum ParamEdit
```

A new value typed in a `param_editor`.

**Variants**

- `Number(f64)`
- `Text(String)`

<a id="cc_app-properties-param_editor"></a>

### fn `param_editor`

```rust
pub fn param_editor(ui: &mut Ui, p: &cc_core::ComponentParam, number: f64, text: &str, materials: &[String]) -> (Response, Option<ParamEdit>)
```

The widget of a component parameter of any kind: a drag value, a
checkbox, a text box, or a list of its choices or of the project's
materials. Returns its response and the new value when it changed.

<a id="cc_app-properties-number_drag"></a>

### fn `number_drag`

```rust
pub fn number_drag(value: &mut f64) -> DragValue<'_>
```

A number field whose typed text is taken when the field is left (Enter,
Tab or a click elsewhere), not at every key. A value written at every key
goes through the model, which clamps or refuses half-typed numbers, and
the field then shows the model's value in place of the text being typed
(issue 319: `1` read `5`, `0.9` over `0.80` gave `0.9.80`). Dragging
still changes the value as it moves.

<a id="cc_app-properties-param_value"></a>

### fn `param_value`

```rust
pub fn param_value(value: &mut f64, kind: cc_core::ParamKind) -> DragValue<'_>
```

Drag value for a component parameter, with the unit of its kind.

<a id="mod-cc_app-recovery"></a>

## `cc_app::recovery`

<sub>`crates/cc_app/src/recovery.rs`</sub>

Autosave and crash recovery of the open project (the files are written by
`cc_core::recovery`): a copy of a project with unsaved changes is written
in the background an interval (a preference: a minute by default, or off)
after its last copy; a clean save, close or
exit removes it; a panic writes a final copy and appends to a crash log;
at start-up, copies left by earlier sessions are offered for restoring.

**Items:** [`data_dir`](#cc_app-recovery-data_dir) · [`folder`](#cc_app-recovery-folder) · [`crash_log`](#cc_app-recovery-crash_log) · [`Recovery`](#cc_app-recovery-recovery) · [`log_panic`](#cc_app-recovery-log_panic) · [`log_event`](#cc_app-recovery-log_event) · [`install_panic_hook`](#cc_app-recovery-install_panic_hook)

<a id="cc_app-recovery-data_dir"></a>

### fn `data_dir`

```rust
pub fn data_dir() -> Option<PathBuf>
```

The app's data folder (where eframe keeps its preferences).

<a id="cc_app-recovery-folder"></a>

### fn `folder`

```rust
pub fn folder() -> Option<PathBuf>
```

The folder of the recovery copies.

<a id="cc_app-recovery-crash_log"></a>

### fn `crash_log`

```rust
pub fn crash_log() -> Option<PathBuf>
```

The crash log: one entry per panic, with its message, place and backtrace,
and per failed graphics start-up or lost GPU device.

<a id="cc_app-recovery-recovery"></a>

### struct `Recovery`

```rust
pub struct Recovery
```

Recovery state of the running session.

*Some fields are private.*

**Methods**

- `pub fn new(dir: Option<PathBuf>) -> Self`

  Recovery in `dir`; the entries already there are offered.

- `pub fn set_interval(&mut self, interval: Option<Duration>)`

  Follow the autosave preference: copies every `interval`, or none (`None`).

- `pub fn tick(&mut self, state: &AppState)`

  Called once a frame: write a copy in the background when it is due,
  remove it once the document has no unsaved changes.

- `pub fn write_now(&mut self, state: &AppState)`

  Write a copy now, on this thread (after a panic, before the app goes).

- `pub fn exit(&mut self)`

  A clean exit: the user saved or chose to discard the changes.

- `pub fn dialog(&mut self, ctx: &egui::Context, state: &mut AppState)`

  The start-up dialog offering the copies of earlier sessions.


<a id="cc_app-recovery-log_panic"></a>

### fn `log_panic`

```rust
pub fn log_panic(info: &std::panic::PanicHookInfo<'_>)
```

Append a panic to the crash log (no recursion, no panics: errors are ignored).

<a id="cc_app-recovery-log_event"></a>

### fn `log_event`

```rust
pub fn log_event(what: &str)
```

Append an event that is not a panic (a failed graphics start-up, a lost
GPU device) to the crash log, without a backtrace; errors are ignored.

<a id="cc_app-recovery-install_panic_hook"></a>

### fn `install_panic_hook`

```rust
pub fn install_panic_hook()
```

Install the panic hook: every panic is logged to the crash log, then
reported as before (on the console, in debug builds).

<a id="mod-cc_app-region_ui"></a>

## `cc_app::region_ui`

<sub>`crates/cc_app/src/region_ui.rs`</sub>

Filled and masking regions in the app: *Annotate → Filled Region* and
*Masking Region* open the sketch mode in the active view (a plan,
ceiling plan, elevation, section, legend or drafting view; curves in the
view's drawing coordinates), and *Finish* runs `filled_region_create` or
`masking_region_create` with the curves. *Edit Boundary* on a selected
region redraws it (`sketch_set`). The properties palette shows the
boundary's line style, the view and the area; Type Properties the fill
of a filled region type.

**Items:** [`takes_openings`](#cc_app-region_ui-takes_openings) · [`start`](#cc_app-region_ui-start) · [`create`](#cc_app-region_ui-create) · [`props`](#cc_app-region_ui-props) · [`type_section`](#cc_app-region_ui-type_section)

<a id="cc_app-region_ui-takes_openings"></a>

### fn `takes_openings`

```rust
pub fn takes_openings(category: Category) -> bool
```

Whether a sketch of `category` takes loops inside its outline as openings.

<a id="cc_app-region_ui-start"></a>

### fn `start`

```rust
pub fn start(state: &mut AppState, category: Category)
```

Start sketching a region of `category` in the active view. A project
without a type of it gets one when the first region is made.

<a id="cc_app-region_ui-create"></a>

### fn `create`

```rust
pub fn create(state: &mut AppState, category: Category, type_id: ElementTypeId, curves: Vec<CurveArg>) -> Option<ElementId>
```

Make the region the curves draw in the active view; returns its id.

<a id="cc_app-region_ui-props"></a>

### fn `props`

```rust
pub fn props(ui: &mut Ui, state: &mut AppState, id: ElementId, region: &Region)
```

The properties of a selected region: its boundary style, view and size.

<a id="cc_app-region_ui-type_section"></a>

### fn `type_section`

```rust
pub fn type_section(ui: &mut Ui, state: &mut AppState, ty: &ElementType)
```

The fill of a filled region type in Type Properties: the foreground
and background patterns and their colors.

<a id="mod-cc_app-render_ui"></a>

## `cc_app::render_ui`

<sub>`crates/cc_app/src/render_ui.rs`</sub>

The render window (*View → Render*): the active 3D view path traced
progressively (`cc_raytracing`), the picture on the left (fitted; the
wheel zooms about the pointer, a drag pans, a double click fits again)
and the view's render settings on the right (quality, output,
environment), with *Save Image…*, *Region*, *Render*, *Stop* and *Close*.
The window is a tool, like a viewer, not a data dialog: it has no Cancel.
A setting is written to the view (`view_set`, through
`crate::engine_edit::view`) only when the user changes one, all of them
under one merge key, so whatever is changed in a session of the window is
one undo step; Close keeps it. It is modal, so the model cannot change
under a running render.
The camera is the one the viewport shows when the window opens (its live
camera, not the one last written to the model). A camera still fitted to
the model, or the default camera of a view never shown, is fitted again
for the render's field of view and aspect, as the viewport fits it to its
own.
Rendering runs on two threads of its own, handed copies of the project,
the view's geometry, the view and its temporary visibility: the render
thread builds the scene and traces it, sending a copy of the linear image
at most four times a second (and the last one); the display thread
exposes and tone maps the newest of those (composited over the previous
image for a region) and hands the picture to the window, so neither the
tracing nor the window waits for that. A stop flag ends the render
between steps: *Stop*, closing the window, a new render and dropping the
window (the app closing, the document replaced) all set it, and nothing
waits for the threads. Each render has its own channels and number, so a
late picture of an earlier render never shows in a later one. Exposure and
tone mapping only change how the linear image is shown: changing them
maps the image again (on a thread) without tracing it again.
*Sun Settings…* hides the window and opens the Sun Settings dialog on the
view; when that closes (OK or Cancel) the window opens again on the same
view and camera, without the picture (the light changed).

**Items:** [`Resume`](#cc_app-render_ui-resume) · [`RenderWindow`](#cc_app-render_ui-renderwindow) · [`open`](#cc_app-render_ui-open) · [`open_for`](#cc_app-render_ui-open_for) · [`show`](#cc_app-render_ui-show)

<a id="cc_app-render_ui-resume"></a>

### struct `Resume`

```rust
pub struct Resume
```

What the window keeps of how it was opened (also while Sun Settings,
opened from it, are open).

**Traits:** `Debug`, `Clone`, `Copy`

*Some fields are private.*

<a id="cc_app-render_ui-renderwindow"></a>

### struct `RenderWindow`

```rust
pub struct RenderWindow
```

The open render window.

*Some fields are private.*

<a id="cc_app-render_ui-open"></a>

### fn `open`

```rust
pub fn open(state: &mut AppState)
```

Open the window on the active view (when it is a 3D view).

<a id="cc_app-render_ui-open_for"></a>

### fn `open_for`

```rust
pub fn open_for(state: &mut AppState, view: ViewId)
```

Open the window on `view`, with the camera its viewport shows now.

<a id="cc_app-render_ui-show"></a>

### fn `show`

```rust
pub fn show(ctx: &Context, state: &mut AppState)
```

<a id="mod-cc_app-ribbon"></a>

## `cc_app::ribbon`

<sub>`crates/cc_app/src/ribbon.rs`</sub>

Ribbon, options bar and status bar.

**Items:** [`category_icon`](#cc_app-ribbon-category_icon) · [`env_group`](#cc_app-ribbon-env_group) · [`command_tooltip`](#cc_app-ribbon-command_tooltip) · [`key_cap`](#cc_app-ribbon-key_cap) · [`ribbon`](#cc_app-ribbon-ribbon) · [`dimension_icon`](#cc_app-ribbon-dimension_icon) · [`selected_text`](#cc_app-ribbon-selected_text) · [`env_stacks`](#cc_app-ribbon-env_stacks) · [`options_bar`](#cc_app-ribbon-options_bar) · [`status_bar`](#cc_app-ribbon-status_bar)

<a id="cc_app-ribbon-category_icon"></a>

### fn `category_icon`

```rust
pub fn category_icon(category: Category) -> Icon
```

Icon of a model category, shared by the ribbon, options bar and browser.

<a id="cc_app-ribbon-env_group"></a>

### fn `env_group`

```rust
pub fn env_group(ui: &mut Ui, state: &mut AppState, caption: String, commands: &[UiCommand])
```

A group of the component environment's ribbon.

<a id="cc_app-ribbon-command_tooltip"></a>

### fn `command_tooltip`

```rust
pub fn command_tooltip(response: Response, state: &AppState, command: UiCommand, why: Option<String>) -> Response
```

The rich tooltip of a command: title and shortcut, what it does, why it
is unavailable, and the engine command that scripts call for the same change.
With detailed tooltips off (a preference), only the title, shortcut and
why it is unavailable.

<a id="cc_app-ribbon-key_cap"></a>

### fn `key_cap`

```rust
pub(crate) fn key_cap(ui: &mut Ui, keys: &str) -> egui::Response
```

A keyboard shortcut drawn as a key cap.

<a id="cc_app-ribbon-ribbon"></a>

### fn `ribbon`

```rust
pub fn ribbon(ui: &mut Ui, state: &mut AppState)
```

<a id="cc_app-ribbon-dimension_icon"></a>

### fn `dimension_icon`

```rust
pub fn dimension_icon(mode: DimensionMode) -> Icon
```

<a id="cc_app-ribbon-selected_text"></a>

### fn `selected_text`

```rust
pub fn selected_text(state: &AppState) -> Option<cc_core::ElementId>
```

The text note selected alone, if any.

<a id="cc_app-ribbon-env_stacks"></a>

### fn `env_stacks`

```rust
pub fn env_stacks(ui: &mut Ui, state: &mut AppState, caption: String, families: &[&[UiCommand]])
```

Families of small buttons, each starting its own column.
A captioned group of small buttons in columns (a contextual ribbon's).

<a id="cc_app-ribbon-options_bar"></a>

### fn `options_bar`

```rust
pub fn options_bar(ui: &mut Ui, state: &mut AppState)
```

<a id="cc_app-ribbon-status_bar"></a>

### fn `status_bar`

```rust
pub fn status_bar(ui: &mut Ui, state: &mut AppState)
```

<a id="mod-cc_app-schedule_ui"></a>

## `cc_app::schedule_ui`

<sub>`crates/cc_app/src/schedule_ui.rs`</sub>

Schedule (table) view, its definition dialog (fields, filter, sorting and
grouping, formatting, appearance) and CSV export.

**Items:** [`Tab`](#cc_app-schedule_ui-tab) · [`open`](#cc_app-schedule_ui-open) · [`props`](#cc_app-schedule_ui-props) · [`show`](#cc_app-schedule_ui-show) · [`dialog`](#cc_app-schedule_ui-dialog)

<a id="cc_app-schedule_ui-tab"></a>

### enum `Tab`

```rust
pub enum Tab
```

The pages of the schedule definition dialog.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Default`

**Variants**

- `Fields`
- `Filter`
- `Sorting`
- `Formatting`
- `Appearance`

**Methods**

- `pub const ALL: [Tab; 5]`
- `pub fn label(self) -> String`

<a id="cc_app-schedule_ui-open"></a>

### fn `open`

```rust
pub fn open(state: &mut AppState, view: ViewId, tab: Tab)
```

Open the definition dialog of schedule `view` on `tab`.

<a id="cc_app-schedule_ui-props"></a>

### fn `props`

```rust
pub fn props(ui: &mut Ui, state: &mut AppState, view: ViewId)
```

The rows of a schedule's properties: one per page of its definition, each with its *Edit* button.

<a id="cc_app-schedule_ui-show"></a>

### fn `show`

```rust
pub fn show(ui: &mut Ui, state: &mut AppState, view_id: ViewId)
```

<a id="cc_app-schedule_ui-dialog"></a>

### fn `dialog`

```rust
pub fn dialog(ctx: &Context, state: &mut AppState)
```

The definition of the schedule being edited: fields, filter, sorting and
grouping, formatting and appearance, a page each.

<a id="mod-cc_app-section_mark"></a>

## `cc_app::section_mark`

<sub>`crates/cc_app/src/section_mark.rs`</sub>

Section marks in plans: picked with the Select tool, moved by dragging,
their ends and the pieces of a split line dragged by their handles,
flipped with Space. Every change goes through `view_set`.

**Items:** [`select`](#cc_app-section_mark-select) · [`Part`](#cc_app-section_mark-part) · [`flip`](#cc_app-section_mark-flip) · [`set_split`](#cc_app-section_mark-set_split)

<a id="cc_app-section_mark-select"></a>

### fn `select`

```rust
pub fn select(ui: &Ui, resp: &Response, painter: &Painter, state: &mut AppState, xf: &Xf, drawing: &Drawing2D) -> bool
```

Section marks under the Select tool in a plan. Returns `true` when they
took the pointer this frame (the element selection is left alone).

<a id="cc_app-section_mark-part"></a>

### enum `Part`

```rust
pub enum Part
```

What a drag of a section mark holds.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Variants**

- `Body`

  The whole mark, moved.

- `End { at_start: bool, fixed: DVec2 }`

  One end; `fixed` is the other.

- `Segment { at_start: bool }`

  The inner end of a split line's piece next to the start or the end.


<a id="cc_app-section_mark-flip"></a>

### fn `flip`

```rust
pub fn flip(state: &mut AppState) -> bool
```

Turn the selected section to look the other way (Space).

<a id="cc_app-section_mark-set_split"></a>

### fn `set_split`

```rust
pub fn set_split(state: &mut AppState, view: ViewId, split: bool)
```

Split the section's line (a piece next to each head, a quarter of it
long), or draw it whole again.

<a id="mod-cc_app-sheet"></a>

## `cc_app::sheet`

<sub>`crates/cc_app/src/sheet.rs`</sub>

Sheet composer: paper, title block and placed viewports.

**Items:** [`show`](#cc_app-sheet-show)

<a id="cc_app-sheet-show"></a>

### fn `show`

```rust
pub fn show(ui: &mut Ui, state: &mut AppState, sheet_id: ViewId)
```

<a id="mod-cc_app-sheet_props"></a>

## `cc_app::sheet_props`

<sub>`crates/cc_app/src/sheet_props.rs`</sub>

A sheet's rows in the properties palette: its number, title block,
paper and issue date, and the title block's per-sheet fields. Each
change runs `sheet_set` (through `crate::engine_edit::sheet`).

**Items:** [`today_iso`](#cc_app-sheet_props-today_iso) · [`rows`](#cc_app-sheet_props-rows) · [`fields_section`](#cc_app-sheet_props-fields_section) · [`title_block_error`](#cc_app-sheet_props-title_block_error)

<a id="cc_app-sheet_props-today_iso"></a>

### fn `today_iso`

```rust
pub fn today_iso() -> String
```

Today as `YYYY-MM-DD`.

<a id="cc_app-sheet_props-rows"></a>

### fn `rows`

```rust
pub fn rows(ui: &mut Ui, state: &mut AppState, id: ViewId)
```

The sheet's own rows (inside its properties section).

<a id="cc_app-sheet_props-fields_section"></a>

### fn `fields_section`

```rust
pub fn fields_section(ui: &mut Ui, state: &mut AppState, id: ViewId)
```

One row per per-sheet field: empty shows (and uses) the type's text.

<a id="cc_app-sheet_props-title_block_error"></a>

### fn `title_block_error`

```rust
pub fn title_block_error(state: &AppState) -> Option<String>
```

The first piece of the active sheet's title block that does not draw,
as a status bar message.

<a id="mod-cc_app-side_panels"></a>

## `cc_app::side_panels`

<sub>`crates/cc_app/src/side_panels.rs`</sub>

The side panels, Project Browser and Properties: docked on either side of
the window, stacked in one column or each in a column of its own. A panel
moves by dragging its header onto a side or another panel, or from the
header's context menu; the layout is a preference (`PanelLayout`).

**Items:** [`Pane`](#cc_app-side_panels-pane) · [`Side`](#cc_app-side_panels-side) · [`PanelLayout`](#cc_app-side_panels-panellayout) · [`Target`](#cc_app-side_panels-target) · [`show`](#cc_app-side_panels-show) · [`menu`](#cc_app-side_panels-menu) · [`target_at`](#cc_app-side_panels-target_at)

<a id="cc_app-side_panels-pane"></a>

### enum `Pane`

```rust
pub enum Pane
```

A side panel.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`

**Variants**

- `Browser`
- `Properties`

**Methods**

- `pub const ALL: [Pane; 2]`
- `pub fn title(self) -> String`

<a id="cc_app-side_panels-side"></a>

### enum `Side`

```rust
pub enum Side
```

A side of the window.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`

**Variants**

- `Left`
- `Right`

<a id="cc_app-side_panels-panellayout"></a>

### struct `PanelLayout`

```rust
pub struct PanelLayout
```

Where the panels are: the columns on each side, from the window's edge
inward, each stacking its panels top to bottom. Every panel is in exactly
one column and no column is empty.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Eq`, `Default`

**Fields**

- `left: Vec<Vec<Pane>>`
- `right: Vec<Vec<Pane>>`

**Methods**

- `pub fn side(&self, side: Side) -> &Vec<Vec<Pane>>`
- `pub fn find(&self, pane: Pane) -> Option<(Side, usize, usize)>`

  The side, column (from the edge) and row of `pane`.

- `pub fn apply(&mut self, pane: Pane, target: Target) -> bool`

  Move `pane` to `target`; false when nothing changed (or the target
  names the pane itself).

- `pub fn to_text(&self) -> String`

  The stored form: `left=browser+properties;right=`, columns separated by
  commas and the panels of a column by `+`.

- `pub fn from_text(text: &str) -> Option<Self>`

  Read `Self::to_text`'s form; `None` unless it names every panel once.


<a id="cc_app-side_panels-target"></a>

### enum `Target`

```rust
pub enum Target
```

Where a moved panel goes.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Variants**

- `Edge(Side)`

  A column of its own at the window's edge on that side.

- `Stack { other: Pane, below: bool }`

  In `other`'s column, above or below it.

- `Beside { other: Pane, right: bool }`

  A column of its own next to `other`'s, to its right or left on screen.


<a id="cc_app-side_panels-show"></a>

### fn `show`

```rust
pub fn show(ui: &mut Ui, state: &mut AppState)
```

Show the side panels; call it after the top and bottom bars and before
the central area.

<a id="cc_app-side_panels-menu"></a>

### fn `menu`

```rust
pub fn menu(ui: &mut Ui, state: &mut AppState, pane: Pane) -> bool
```

The docking choices for `pane`; true when one was taken.

<a id="cc_app-side_panels-target_at"></a>

### fn `target_at`

```rust
pub fn target_at(pos: Pos2, pane: Pane, area: Rect, rects: &[(Pane, Rect)]) -> Option<(Target, Rect)>
```

Where a panel dragged to `pos` would go, and the area that shows it:
the strips along the window's sides dock it there, and over another
panel the nearest edge picks above, below, left or right of it.

<a id="mod-cc_app-site_ui"></a>

## `cc_app::site_ui`

<sub>`crates/cc_app/src/site_ui.rs`</sub>

Site Location (Manage → Project): a window with an OpenStreetMap map, an
address search, a marker fixed at the center and the square of map data
to bring in. The map moves under the marker; OK stores the location,
elevation, true north and address (`site_set`); *Import Context* also
downloads the map data of the square (Overpass) and runs
`site_context_import`. Map tiles, the search and the elevation come
from public services in background threads (never in tests); tiles are
kept in memory while the window is open.
Usage policies followed: an identifying User-Agent, at most two tile
connections, one search per second and only on request (no
autocomplete), and the OpenStreetMap attribution on the map.

**Items:** [`SiteDialog`](#cc_app-site_ui-sitedialog) · [`open`](#cc_app-site_ui-open) · [`show`](#cc_app-site_ui-show)

<a id="cc_app-site_ui-sitedialog"></a>

### struct `SiteDialog`

```rust
pub struct SiteDialog
```

The open Site Location window.

*Some fields are private.*

<a id="cc_app-site_ui-open"></a>

### fn `open`

```rust
pub fn open(state: &mut AppState)
```

Open the window on the project's location (else a view of the whole country map).

<a id="cc_app-site_ui-show"></a>

### fn `show`

```rust
pub fn show(ctx: &Context, state: &mut AppState)
```

<a id="mod-cc_app-sketch_mode"></a>

## `cc_app::sketch_mode`

<sub>`crates/cc_app/src/sketch_mode.rs`</sub>

Sketch mode: the outline of a floor, ceiling, roof, foundation slab or
mass drawn with lines, rectangles, polygons, circles, arcs and ellipses
(see `cc_core::sketch`), in the active plan. The ribbon shows the
drawing tools and *Finish*/*Cancel* while the sketch is open; in its
*Select* mode a click picks a curve, whose grips stretch it and whose
values the properties palette edits; a window picks the curves inside it
(or, dragged to the left, those it touches) and a double click the chain
of joined curves. The modify tools move, copy, rotate and mirror the
picked curves, and offset, trim, extend and split curves (see
`cc_core::sketch_edit`). *Finish* creates the element (or draws the
edited element's outline again, *Edit Boundary*) through the engine's
`curves`; the sketch's own changes undo inside it. A terrain's sketch
also takes its measured points (*Point*, at the elevation in the options
bar), which undo with the curves.

**Items:** [`Target`](#cc_app-sketch_mode-target) · [`Draw`](#cc_app-sketch_mode-draw) · [`Session`](#cc_app-sketch_mode-session) · [`curve_grips`](#cc_app-sketch_mode-curve_grips) · [`moved`](#cc_app-sketch_mode-moved) · [`boundary_target`](#cc_app-sketch_mode-boundary_target) · [`edit_boundary`](#cc_app-sketch_mode-edit_boundary) · [`takes_points`](#cc_app-sketch_mode-takes_points) · [`cancel`](#cc_app-sketch_mode-cancel) · [`finish`](#cc_app-sketch_mode-finish) · [`error_text`](#cc_app-sketch_mode-error_text) · [`escape`](#cc_app-sketch_mode-escape) · [`set_draw`](#cc_app-sketch_mode-set_draw) · [`pick_walls`](#cc_app-sketch_mode-pick_walls) · [`canvas`](#cc_app-sketch_mode-canvas) · [`interact`](#cc_app-sketch_mode-interact) · [`draw`](#cc_app-sketch_mode-draw) · [`finish_spline`](#cc_app-sketch_mode-finish_spline) · [`ribbon`](#cc_app-sketch_mode-ribbon) · [`options`](#cc_app-sketch_mode-options) · [`properties`](#cc_app-sketch_mode-properties)

<a id="cc_app-sketch_mode-target"></a>

### enum `Target`

```rust
pub enum Target
```

What finishing the sketch makes.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Variants**

- `New { category: Category, type_id: ElementTypeId }`

  A new element of `category` and type.

- `Edit { id: ElementId }`

  A new outline for an element (*Edit Boundary*).

- `Piece`

  A piece of the component environment (see `Session::piece`).

- `Profile { type_id: ElementTypeId }`

  The profile of a sweep type (*Edit Profile*, see `crate::profile_sketch`).

- `Section { profile: ProfileId }`

  Section = a structural profile's sketch (*Edit sketch* in the Profile Manager,
  see `crate::profile_sketch`).

- `Sheet`

  Lines of a title block on its Sheet view (see `crate::title_block_env`).

- `Detail`

  Detail lines being drawn (see `crate::detail_ui`): each curve
  becomes an element as soon as it is drawn, so there is nothing to finish.


<a id="cc_app-sketch_mode-draw"></a>

### enum `Draw`

```rust
pub enum Draw
```

How clicks draw.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Default`

**Variants**

- `Select`

  Pick curves to change them.

- `Line`

  Two points, chained by default.

- `Rect`

  Two opposite corners.

- `Polygon`

  The center, then a corner.

- `PolygonOut`

  The center, then the middle of a side.

- `Circle`

  The center, then a point on it.

- `Arc3`

  The two ends, then a point on the arc.

- `ArcCenter`

  The center, the start, then the end.

- `ArcTangent`

  From the free end of a curve, along it, to the end.

- `Ellipse`

  The center, the end of an axis, then the other axis's half length.

- `Spline`

  Points a smooth curve passes through: a click on the last point
  (a double click) or Enter ends it, a click on the first closes it.

- `Move`

  Move the picked curves: base point, then target.

- `Copy`

  Copies of the picked curves: base point, then targets.

- `Rotate`

  Rotate the picked curves: center, reference direction, final direction.

- `MirrorPick`

  Mirror the picked curves across a line of the sketch.

- `MirrorAxis`

  Mirror the picked curves across an axis drawn with two points.

- `Offset`

  A copy of a curve, or of its chain, at a distance.

- `TrimCorner`

  Two curves trimmed or extended to meet.

- `TrimExtend`

  Curves trimmed or extended to a reference curve.

- `Split`

  A curve split in two.

- `Point`

  A terrain's measured point, at the elevation of the options bar.


**Methods**

- `pub fn on_selection(self) -> bool`

  The tools that act on the picked curves.


<a id="cc_app-sketch_mode-session"></a>

### struct `Session`

```rust
pub struct Session
```

An open sketch.

**Traits:** `Debug`, `Clone`

**Fields**

- `target: Target`
- `curves: Vec<SketchCurve>`
- `draw: Draw`
- `picks: Vec<DVec2>`

  Points picked for the curve being drawn.

- `chain: bool`

  Lines continue from the last end.

- `sides: usize`

  Sides of the next polygon.

- `selected: BTreeSet<usize>`
- `piece: Option<crate::component_sketch::PieceSketch>`

  What a sketch of the component environment makes.

- `points: Vec<DVec3>`

  A terrain's measured points (plan position and elevation above the
  level); the elevation of the next one is in the options bar
  (`crate::modify::ModifyOptions::point_elevation`).

- `selected_points: BTreeSet<usize>`

  The picked points (indices into `points`), deleted with the curves.

- `typed: TypedInput`

  Values typed after the first point (a length and an angle, or a
  rectangle's width and depth; Tab switches the field).


*Some fields are private.*

**Methods**

- `pub fn new(target: Target, curves: Vec<SketchCurve>) -> Self`
- `pub fn typing(&self) -> bool`

  The tool takes typed values: it has its first point and the next
  one ends a segment from it (a line, a radius, an axis, a move) or is
  a rectangle's opposite corner.

- `pub fn typed_point(&self, p: DVec2) -> DVec2`

  The point the cursor `p` stands for with the typed values: along
  the cursor's direction from the last pick at the typed length (and
  angle from the X axis), or a rectangle's corner with the typed width
  and depth on the cursor's side.

- `pub fn nothing_selected(&self) -> bool`

  Nothing is picked: no curve and no point.

- `pub fn clear_selection(&mut self)`

  Unpick every curve and point.

- `pub fn add_point(&mut self, p: DVec3)`

  Add a terrain point (one undo step).

- `pub fn open_path(&self) -> bool`

  The sketch is a sweep's path, open by design.

- `pub fn add(&mut self, curves: Vec<SketchCurve>)`
- `pub fn can_undo(&self) -> bool`
- `pub fn can_redo(&self) -> bool`
- `pub fn undo(&mut self)`
- `pub fn redo(&mut self)`
- `pub fn delete_selected(&mut self)`

  Remove the selected curves and points (one undo step).

- `pub fn set_curve(&mut self, i: usize, curve: SketchCurve)`

  Change curve `i` (one undo step).

- `pub fn transform_selected(&mut self, t: &PlanTransform, copy: bool) -> BTreeSet<usize>`

  The picked curves moved by `t` (one undo step), or copies of them;
  returns the copies' indices.

- `pub fn snap_points(&self) -> Vec<DVec2>`

  Ends, centers and middles to snap to.


<a id="cc_app-sketch_mode-curve_grips"></a>

### fn `curve_grips`

```rust
pub(crate) fn curve_grips(c: &SketchCurve) -> Vec<(u8, DVec2)>
```

The grips of a curve, each with its handle number (see `moved`).

<a id="cc_app-sketch_mode-moved"></a>

### fn `moved`

```rust
pub(crate) fn moved(c: &SketchCurve, which: u8, p: DVec2) -> SketchCurve
```

Curve `c` with its grip `which` moved to `p`.

<a id="cc_app-sketch_mode-boundary_target"></a>

### fn `boundary_target`

```rust
pub fn boundary_target(state: &AppState) -> Option<ElementId>
```

The element whose boundary *Edit Boundary* would open: one selected
floor, ceiling, roof or mass.

<a id="cc_app-sketch_mode-edit_boundary"></a>

### fn `edit_boundary`

```rust
pub fn edit_boundary(state: &mut AppState)
```

Open the sketch of the selected element's outline (*Edit Boundary*).

<a id="cc_app-sketch_mode-takes_points"></a>

### fn `takes_points`

```rust
pub fn takes_points(state: &AppState) -> bool
```

The open sketch makes (or edits) a terrain, which takes measured points.

<a id="cc_app-sketch_mode-cancel"></a>

### fn `cancel`

```rust
pub fn cancel(state: &mut AppState)
```

Leave the sketch without changing the model.

<a id="cc_app-sketch_mode-finish"></a>

### fn `finish`

```rust
pub fn finish(state: &mut AppState)
```

Make the element (or its new outline) from the sketch; a sketch that
does not close shows where and stays open.

<a id="cc_app-sketch_mode-error_text"></a>

### fn `error_text`

```rust
pub fn error_text(e: &SketchError) -> String
```

A sketch error as a message.

<a id="cc_app-sketch_mode-escape"></a>

### fn `escape`

```rust
pub fn escape(state: &mut AppState)
```

Escape: drop the curve being drawn, then leave the drawing tool, then the selection.

<a id="cc_app-sketch_mode-set_draw"></a>

### fn `set_draw`

```rust
pub fn set_draw(state: &mut AppState, draw: Draw)
```

Pick a drawing or modify tool; those acting on the picked curves need some.

<a id="cc_app-sketch_mode-pick_walls"></a>

### fn `pick_walls`

```rust
pub fn pick_walls(state: &mut AppState)
```

Add lines along the loop of the level's walls, on the face of each wall
chosen in the options bar (issue #311).

<a id="cc_app-sketch_mode-canvas"></a>

### fn `canvas`

```rust
pub fn canvas(ui: &Ui, resp: &Response, painter: &Painter, state: &mut AppState, level: Option<LevelId>, xf: &Xf)
```

The sketch in a plan (or, for regions, any view that holds drawings,
with no level): its curves, the tool's preview, snapping, picks and grips.

<a id="cc_app-sketch_mode-interact"></a>

### fn `interact`

```rust
pub fn interact(ui: &Ui, resp: &Response, painter: &Painter, s: &mut Session, opts: &crate::modify::ModifyOptions, raw: Option<DVec2>, snapped: Option<Snap>, xf: &Xf) -> Option<String>
```

The sketch in a canvas (a plan, or a view of the component environment):
its curves, where it does not close, and the active tool at the cursor
(`raw`, and `snapped` where it snaps). Returns a message for the status bar.

<a id="cc_app-sketch_mode-draw"></a>

### fn `draw`

```rust
pub(crate) fn draw(click: bool, painter: &Painter, s: &mut Session, p: DVec2, xf: &Xf, tol: f64)
```

The drawing tools: preview at `p`, and a click adds a point or the curve.

<a id="cc_app-sketch_mode-finish_spline"></a>

### fn `finish_spline`

```rust
pub(crate) fn finish_spline(s: &mut Session, closed: bool)
```

Add the spline through the picks (closed back to the first when
`closed`) and start a new one.

<a id="cc_app-sketch_mode-ribbon"></a>

### fn `ribbon`

```rust
pub fn ribbon(ui: &mut Ui, state: &mut AppState)
```

The sketch's ribbon: mode, drawing tools and tools.

<a id="cc_app-sketch_mode-options"></a>

### fn `options`

```rust
pub fn options(ui: &mut Ui, state: &mut AppState)
```

The options bar: chaining, polygon sides, the modify tools' options and the hint.

<a id="cc_app-sketch_mode-properties"></a>

### fn `properties`

```rust
pub fn properties(ui: &mut Ui, state: &mut AppState)
```

The selected curve's values in the properties palette.

<a id="mod-cc_app-snap_geom"></a>

## `cc_app::snap_geom`

<sub>`crates/cc_app/src/snap_geom.rs`</sub>

Snap points that come from two pieces of geometry, or from the point a
segment is drawn from (issue #159): where lines and arcs cross, the foot
of the perpendicular from the anchor, and the points where a line from
the anchor touches an arc or circle. Lines are segments, arcs are
`cc_core::arc::Arc`s (a whole circle sweeps a full turn).

**Items:** [`crossing`](#cc_app-snap_geom-crossing) · [`perpendicular`](#cc_app-snap_geom-perpendicular) · [`tangent`](#cc_app-snap_geom-tangent)

<a id="cc_app-snap_geom-crossing"></a>

### fn `crossing`

```rust
pub fn crossing(lines: &[(DVec2, DVec2)], arcs: &[Arc], raw: DVec2, r: f64) -> Option<DVec2>
```

The crossing of two of the `lines` and `arcs` nearest `raw`, within `r`.

<a id="cc_app-snap_geom-perpendicular"></a>

### fn `perpendicular`

```rust
pub fn perpendicular(anchor: DVec2, lines: &[(DVec2, DVec2)], arcs: &[Arc], raw: DVec2, r: f64) -> Option<DVec2>
```

The foot of the perpendicular from `anchor` to one of the `lines` (on
the segment) or `arcs` (towards or away from the center, on the arc)
nearest `raw`, within `r`.

<a id="cc_app-snap_geom-tangent"></a>

### fn `tangent`

```rust
pub fn tangent(anchor: DVec2, arcs: &[Arc], raw: DVec2, r: f64) -> Option<DVec2>
```

A point where a line from `anchor` touches one of the `arcs`, nearest
`raw`, within `r` (none from inside a circle).

<a id="mod-cc_app-snaps"></a>

## `cc_app::snaps`

<sub>`crates/cc_app/src/snaps.rs`</sub>

Object snaps (issue #317): which points the tools snap to (a preference),
and the snap points of the elements a view draws, so that every element
shown has them, not only those the tools know by their geometry.

**Items:** [`SnapSettings`](#cc_app-snaps-snapsettings) · [`SnapOption`](#cc_app-snaps-snapoption) · [`drawing_snap`](#cc_app-snaps-drawing_snap)

<a id="cc_app-snaps-snapsettings"></a>

### struct `SnapSettings`

```rust
pub struct SnapSettings
```

Which object snaps are on. Rounding free points to the zoom's increment
and the angle increments from the point a segment starts at stay.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Default`

**Fields**

- `enabled: bool`

  Off: no object snap at all, whatever the others say.

- `elements: bool`

  Every element the view draws, by its lines; otherwise only the points
  and lines the tools take from the model (walls, grids, outlines).

- `endpoint: bool`
- `midpoint: bool`
- `center: bool`
- `intersection: bool`
- `perpendicular: bool`
- `tangent: bool`
- `nearest: bool`

  The nearest point of a line or an arc.

- `alignment: bool`

  Guides from points acquired by hovering them.


**Methods**

- `pub fn get(&self, option: SnapOption) -> bool`
- `pub fn get_mut(&mut self, option: SnapOption) -> &mut bool`
- `pub fn effective(self) -> Self`

  The settings the tools go by: with the master switch off, no kind is on.

- `pub fn to_text(self) -> String`

  As stored: the codes of the options that are off, separated by commas
  (so an option added later is on for settings stored before it).

- `pub fn from_text(text: &str) -> Self`

  Read what `Self::to_text` wrote; unknown codes are ignored.


<a id="cc_app-snaps-snapoption"></a>

### enum `SnapOption`

```rust
pub enum SnapOption
```

A snap setting: its storage code and the key of its name.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Variants**

- `Enabled`
- `Elements`
- `Endpoint`
- `Midpoint`
- `Center`
- `Intersection`
- `Perpendicular`
- `Tangent`
- `Nearest`
- `Alignment`

**Methods**

- `pub const ALL: [SnapOption; 10]`
- `pub const KINDS: [SnapOption; 8]`

  The kinds of snap point, which the master switch turns off together.

- `pub fn code(self) -> &'static str`
- `pub fn key(self) -> &'static str`

  The i18n key of its name.


<a id="cc_app-snaps-drawing_snap"></a>

### fn `drawing_snap`

```rust
pub fn drawing_snap(drawing: &Drawing2D, raw: DVec2, r: f64, out: &mut CadSnap)
```

Snap features of the elements `drawing` shows near `raw`: the lines
passing within `r`, their ends where they are corners and their
midpoints. See `Drawing2D::lines_near` for what is left out.

<a id="mod-cc_app-stair_ui"></a>

## `cc_app::stair_ui`

<sub>`crates/cc_app/src/stair_ui.rs`</sub>

Stair and ramp controls: the plan layout (options bar and properties), the
construction and step rules of stair types, the rules of ramp types, and their checks.

**Items:** [`Edited`](#cc_app-stair_ui-edited) · [`layout_controls`](#cc_app-stair_ui-layout_controls) · [`build_rows`](#cc_app-stair_ui-build_rows) · [`posts_rows`](#cc_app-stair_ui-posts_rows) · [`rules_rows`](#cc_app-stair_ui-rules_rows) · [`check_rows`](#cc_app-stair_ui-check_rows) · [`create_command`](#cc_app-stair_ui-create_command) · [`tool_options`](#cc_app-stair_ui-tool_options) · [`ramp_layout_controls`](#cc_app-stair_ui-ramp_layout_controls) · [`ramp_rules_rows`](#cc_app-stair_ui-ramp_rules_rows) · [`ramp_check_rows`](#cc_app-stair_ui-ramp_check_rows) · [`ramp_command`](#cc_app-stair_ui-ramp_command) · [`ramp_tool_options`](#cc_app-stair_ui-ramp_tool_options)

<a id="cc_app-stair_ui-edited"></a>

### struct `Edited`

```rust
pub struct Edited
```

What a set of controls did this frame.

**Traits:** `Default`

**Fields**

- `changed: bool`
- `done: bool`

  A drag or text edit ended, or a discrete choice was made.


<a id="cc_app-stair_ui-layout_controls"></a>

### fn `layout_controls`

```rust
pub fn layout_controls(ui: &mut Ui, salt: u64, layout: &mut StairLayout, table: bool) -> Edited
```

Shape, turn, riser count, tread and the sizes the shape uses.

<a id="cc_app-stair_ui-build_rows"></a>

### fn `build_rows`

```rust
pub fn build_rows(ui: &mut Ui, salt: u64, build: &mut StairBuild) -> Edited
```

Construction of a stair type: solid, cast in place (waist) or assembled
(stringers and treads).

<a id="cc_app-stair_ui-posts_rows"></a>

### fn `posts_rows`

```rust
pub fn posts_rows(ui: &mut Ui, layout: &mut StairLayout, g: &cc_geom::StairGeom, build: &StairBuild) -> Edited
```

The posts under each landing of an assembled stair: a checkbox for each
place round its edge its beams reach.

<a id="cc_app-stair_ui-rules_rows"></a>

### fn `rules_rows`

```rust
pub fn rules_rows(ui: &mut Ui, salt: u64, rules: &mut StairRules, max_riser: f64, tread: f64) -> Edited
```

Step rules of a stair type: how the tread is set and the comfort limits.

<a id="cc_app-stair_ui-check_rows"></a>

### fn `check_rows`

```rust
pub fn check_rows(ui: &mut Ui, g: &cc_geom::StairGeom)
```

Riser, tread, Blondel's step and the broken rules of a laid-out stair.

<a id="cc_app-stair_ui-create_command"></a>

### fn `create_command`

```rust
pub fn create_command(layout: StairLayout, start: [f64; 2], direction: f64, level: cc_core::LevelId, type_id: cc_core::ElementTypeId) -> cc_engine::commands::stair::StairCreate
```

The engine command creating a stair with `layout`.

<a id="cc_app-stair_ui-tool_options"></a>

### fn `tool_options`

```rust
pub fn tool_options(ui: &mut Ui, state: &mut AppState)
```

Options bar of the stair tool.

<a id="cc_app-stair_ui-ramp_layout_controls"></a>

### fn `ramp_layout_controls`

```rust
pub fn ramp_layout_controls(ui: &mut Ui, salt: u64, layout: &mut cc_core::RampLayout, table: bool) -> Edited
```

Shape, turn, slope and the sizes a ramp's shape uses.

<a id="cc_app-stair_ui-ramp_rules_rows"></a>

### fn `ramp_rules_rows`

```rust
pub fn ramp_rules_rows(ui: &mut Ui, rules: &mut cc_core::RampRules) -> Edited
```

Accessibility rules of a ramp type.

<a id="cc_app-stair_ui-ramp_check_rows"></a>

### fn `ramp_check_rows`

```rust
pub fn ramp_check_rows(ui: &mut Ui, g: &cc_geom::RampGeom)
```

Slope, flights and the broken rules of a laid-out ramp.

<a id="cc_app-stair_ui-ramp_command"></a>

### fn `ramp_command`

```rust
pub fn ramp_command(layout: cc_core::RampLayout, start: [f64; 2], direction: f64, level: cc_core::LevelId, type_id: cc_core::ElementTypeId) -> cc_engine::commands::ramp::RampCreate
```

The engine command creating a ramp with `layout`.

<a id="cc_app-stair_ui-ramp_tool_options"></a>

### fn `ramp_tool_options`

```rust
pub fn ramp_tool_options(ui: &mut Ui, state: &mut AppState)
```

Options bar of the ramp tool.

<a id="mod-cc_app-state"></a>

## `cc_app::state`

<sub>`crates/cc_app/src/state.rs`</sub>

Application state shared by all panels and views.

**Items:** [`RibbonTab`](#cc_app-state-ribbontab) · [`PendingAction`](#cc_app-state-pendingaction) · [`Dialogs`](#cc_app-state-dialogs) · [`AppState`](#cc_app-state-appstate)

<a id="cc_app-state-ribbontab"></a>

### enum `RibbonTab`

```rust
pub enum RibbonTab
```

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Default`

**Variants**

- `File`
- `Architecture`
- `Structure`
- `Mep`

  Building services: plumbing and electrical.

- `MassingSite`

  Conceptual masses and the terrain.

- `Insert`

  Importing and exporting files, loading components.

- `Modify`
- `Annotate`
- `View`
- `Manage`

<a id="cc_app-state-pendingaction"></a>

### enum `PendingAction`

```rust
pub enum PendingAction
```

An action that discards unsaved changes, waiting for confirmation.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Variants**

- `New(cc_core::MeasurementSystem)`

  A new project from the template of this measurement system.

- `Open`
- `OpenPath(std::path::PathBuf)`
- `OpenSample(&'static str)`
- `ImportIfc`
- `Close`
- `Quit`

<a id="cc_app-state-dialogs"></a>

### struct `Dialogs`

```rust
pub struct Dialogs
```

**Traits:** `Default`

**Fields**

- `project_info: bool`
- `units: bool`
- `material_browser: crate::material_browser::Browser`
- `levels: bool`
- `materials: bool`
- `type_editor: Option<ElementTypeId>`
- `new_component: Option<crate::new_component_dialog::NewComponent>`

  The New Component dialog: the name typed and the kind picked.

- `type_rename: Option<String>`

  Type name being edited by "Rename" in the type dialog.

- `type_rename_focus: bool`

  The rename field takes the focus on its next frame.

- `type_layers_open: bool`

  The layer editor is expanded in the type dialog.

- `edit_start: Option<usize>`

  Undo depth when the open data dialog started editing (see `dialog::session`).

- `level_selected: Option<LevelId>`

  Level picked in the levels dialog: the one a level is added above or below.

- `delete_level: Option<LevelId>`

  Level waiting for delete confirmation.

- `schedule_def: Option<ViewId>`

  Schedule whose definition dialog is open, and the page it shows.

- `schedule_tab: crate::schedule_ui::Tab`
- `confirm: Option<PendingAction>`
- `new_project: Option<cc_core::MeasurementSystem>`

  The New Project dialog, with the template picked in it.

- `error: Option<String>`
- `about: bool`
- `systems: bool`

  Plumbing systems and electrical circuits.

- `sizing: bool`

  Sizing of the pipes and circuits.

- `lps_check: bool`

  The lightning protection coverage check, with the class it checks
  and its last result (class, document revision, `lps_check` output).

- `lps_class: cc_core::LpsClass`
- `lps_result: Option<(cc_core::LpsClass, u64, serde_json::Value)>`
- `cabling: bool`

  The structured cabling per rack, with its last result (document revision, `cabling_list` output).

- `cabling_result: Option<(u64, serde_json::Value)>`
- `graphics: Option<crate::graphics_dialog::GraphicsDialog>`

  Visibility/Graphics of a view or view template.

- `element_override: Option<crate::graphics_dialog::OverrideDialog>`

  Graphic overrides of the selected elements in a view.

- `filters: Option<crate::graphics_dialog::FilterEditor>`

  The view filter editor.

- `templates: Option<crate::graphics_dialog::TemplateManager>`

  The view template manager.

- `phases: Option<crate::phases_dialog::PhasesDialog>`

  Phases, phase filters and phase graphics.

- `design_options: Option<crate::design_options_ui::OptionsDialog>`

  Design option sets and their options.

- `browser_org: Option<crate::browser_org_dialog::BrowserOrgDialog>`

  The project browser's organization schemes.

- `line_styles: Option<crate::line_styles_dialog::LineStylesDialog>`

  The project's line styles.

- `sun: Option<crate::sun_ui::SunDialog>`

  The Sun Settings of a view.

- `render: Option<crate::render_ui::RenderWindow>`

  The render window of a 3D view (*View → Render*).

- `render_resume: Option<crate::render_ui::Resume>`

  The render window waiting for Sun Settings, opened from it, to close.

- `fill_patterns: Option<crate::fill_patterns_dialog::FillPatternsDialog>`

  The fill pattern manager.

- `profiles: Option<crate::profiles_dialog::ProfilesDialog>`

  The Profile Manager.

- `grid_frame: Option<crate::grid_frame_ui::GridFrameDialog>`

  *Structure → Grid Frame*.

- `steel_building: Option<crate::steel_building_ui::SteelBuildingDialog>`

  *Structure → Steel Building*.

- `wall_framing: Option<crate::wall_framing_ui::WallFramingDialog>`

  *Structure → Light Steel Frame / Wood Frame*.

- `profiles_resume: Option<crate::graphics_dialog::Edits>`

  The Profile Manager's edits while one of its profiles is in a
  sketch (*Edit sketch*): Finish and Cancel open it again with them, so
  its Cancel still undoes everything.

- `library: Option<crate::library::Library>`

  The component library window.

- `obj_import: Option<std::path::PathBuf>`

  The .obj file whose import options are being asked.

- `obj_options: crate::obj_ui::ObjOptions`

  The last choices of the OBJ import dialog.

- `preferences: Option<crate::preferences::PreferencesDialog>`

  The Preferences window.


<a id="cc_app-state-appstate"></a>

### struct `AppState`

```rust
pub struct AppState
```

**Fields**

- `session: Session`

  The engine session: document, derived geometry, drawings and sheets.

- `multi_summary: Option<std::sync::Arc<crate::properties::MultiSummary>>`

  The multi-selection rows' copy of the selected elements, per revision and selection.

- `paint_cache: crate::paint_cache::PaintCache`

  The 2D views' drawings as painted, kept between frames.

- `snap_cache: Option<(u64, Option<LevelId>, std::sync::Arc<crate::tools::ModelSnaps>)>`

  The model's snap candidates (document revision, level, candidates).

- `cad_snap_index: std::collections::HashMap<ElementId, crate::tools::CadIndexEntry>`

  Each imported drawing's snap index, kept while its drawing stays the same.

- `selection: BTreeSet<ElementId>`
- `tool: Tool`
- `interaction: Interaction`
- `modify: crate::modify::ModifyOptions`

  Options of the transform tools.

- `dim_continuous: bool`

  Continuous (chained) placement of aligned and linear dimensions.

- `truss: crate::tools::TrussOptions`

  Options of the truss tool.

- `bracing: crate::tools::BracingOptions`

  Options of the bracing tool.

- `mep: crate::tools::MepOptions`

  Options of the pipe, conduit and device tools.

- `curtain: crate::curtain_ui::CurtainOptions`

  Options of the Curtain Grid tool.

- `paint_material: String`

  The material of the Paint tool.

- `active_view: Option<ViewId>`
- `view2d: HashMap<ViewId, View2DState>`
- `view3d: HashMap<ViewId, View3DState>`
- `open_requests: Vec<ViewId>`

  Views the dock should open (or focus) this frame.

- `open_tabs: usize`

  View tabs open (kept by the dock each frame).

- `close_others_than: Option<ViewId>`

  Close every view tab but this one on the next frame.

- `dialogs: Dialogs`
- `ribbon_tab: RibbonTab`
- `ribbon_collapsed: bool`

  The ribbon shows only its tabs; clicking one shows its panel over the views (`ribbon_peek`).

- `ribbon_peek: bool`
- `palette: Option<crate::palette::Palette>`

  The open command palette.

- `rename: Option<(ViewId, String)>`

  Browser inline rename in progress.

- `sheet_selection: Option<(ViewId, usize)>`

  Selected viewport on a sheet: (sheet, viewport index).

- `section_selection: Option<ViewId>`

  View whose mark is selected: a section's in a plan (see
  `crate::section_mark`) or a callout's (see `crate::callout_ui`).

- `message: String`

  Status bar message.

- `cursor: Option<glam::DVec2>`

  Cursor position in the active 2D view's coordinates.

- `key_buffer: String`

  Typed shortcut letters (e.g. "WA").

- `typed: crate::tools::TypedInput`

  Typed length for the wall tool.

- `wall_height: f64`

  Default height for new walls.

- `tool_system: cc_core::MeasurementSystem`

  The measurement system the tools' sizes were last set for (see
  `AppState::follow_system`).

- `wall_location_line: cc_core::LocationLine`

  Location line for new walls.

- `stair_layout: cc_core::StairLayout`

  Plan layout for new stairs (shape, turn, landing, winders, radius).

- `sweep: crate::tools::SweepOptions`

  Options for new swept profiles (height, closed path, side).

- `railing_host: crate::tools::RailingHostOptions`

  Railings on a picked stair or ramp instead of a drawn path.

- `ramp_layout: cc_core::RampLayout`

  Plan layout for new ramps (shape, turn, slope, landing, radius).

- `wall_arc: bool`

  The wall tool draws arcs (start, end, then a point on the arc).

- `strip_arc: bool`

  The footing tool, with a strip type, draws arc strips as the wall tool draws arcs.

- `beam_arc: bool`

  The beam tool draws arc beams the same way.

- `wall_top_level: Option<LevelId>`

  Level new walls rise to (`None`: unconnected, `wall_height` tall).

- `quit_confirmed: bool`

  Allow the window to close after the user confirmed.

- `textures_to_free: Vec<egui::TextureId>`

  egui textures of closed 3D views, freed by the app when the GPU is available.

- `generation: u64`

  Bumped whenever the document is replaced (new/open).

- `show_home: bool`

  The home screen is shown instead of the project.

- `project_open: bool`

  A project is open (false after "Close Project" and at startup without
  a file; the document is then an unused placeholder).

- `recent: Vec<std::path::PathBuf>`

  Recently opened or saved projects, most recent first.

- `prefs: crate::preferences::Preferences`

  The app preferences (saved with eframe; see `preferences.rs`).

- `updates: crate::update::Updates`

  Update checks and the notice of a newer release.

- `plugins: crate::plugins_ui::Plugins`

  The plugins of the plugins folder and the Plugins window.

- `site: Option<crate::site_ui::SiteDialog>`

  The open Site Location window.

- `thumbnail_jobs: Vec<crate::thumbnails::Job>`
- `job: Option<crate::jobs::Job>`

  The file being opened or imported on a worker thread (see `jobs.rs`).

- `requested_samples: std::collections::BTreeSet<&'static str>`

  Samples whose thumbnail was already requested this session.

- `clipboard: crate::clipboard::Clipboard`

  Cut or copied elements of this document.

- `zoom_default: crate::canvas::ZoomCommand`

  Command of the views' zoom button (the last one chosen from its menu).

- `value_edit: Option<crate::dim_edit::ValueEdit>`

  Value field open over a view (dimension values).

- `component_env: Option<crate::component_env::Env>`

  The component modeling environment, when open (it replaces the views).

- `sketch: Option<crate::sketch_mode::Session>`

  The open sketch of an outline (see `crate::sketch_mode`).

- `text_edit: Option<crate::text_tool::TextEditor>`

  Text editor open over a view (text notes).

- `text_options: crate::text_tool::TextOptions`

  Options of the text tools.

- `detail: crate::detail_ui::DetailOptions`

  Options of the Detail Line tool (mode and line style).

- `detail_draw: Option<crate::detail_ui::DetailDraw>`

  Detail lines being drawn (the picks of the curve in progress).

- `detail_component: crate::detail_component_ui::PlaceOptions`

  Options of the Detail Component tool (rotation, scale, mirror).

- `callout: crate::callout_ui::CalloutOptions`

  Options of the Callout tool (scale, drafting view pointed to).

- `graph: crate::graph_ui::GraphEditor`

  The visual programming editor (see `crate::graph_ui`).


*Some fields are private.*

**Methods**

- `pub fn new(doc: Document) -> Self`
- `pub fn replace_document(&mut self, doc: Document)`

  Replace the document (new/open), resetting view state.

- `pub fn follow_system(&mut self)`

  Tool sizes still at one measurement system's default take the other
  system's when the project's changes (a new imperial project, or its
  units switched): new walls 3 m or 10'-0" tall, offsets and truss
  depths 1 m or 3'-0", the gaps and spiral radii of new stairs and
  ramps (see `cc_core::Defaults::flights`). Sizes the user typed stay.

- `pub fn sync(&mut self)`

  Rebuild derived geometry after model changes and drop stale selection.

- `pub fn project(&self) -> &Project`
- `pub fn drawing(&mut self, view: ViewId) -> Option<Rc<Drawing2D>>`

  Cached drawing for a graphical view.

- `pub fn invalidate_drawings(&mut self)`

  Drop cached drawings (e.g. after a language change alters their text).

- `pub fn sheet(&mut self, view: ViewId) -> Option<Rc<cc_views::Sheet>>`

  Cached composition of a sheet view (paper millimeters).

- `pub fn open_view(&mut self, view: ViewId)`
- `pub fn follow_selection(&mut self)`

  Picking objects with the Select tool opens the Modify tab (drawing
  tools, which select what they create, keep their tab).

- `pub fn typing_segment(&self) -> bool`

  A segment is being drawn with typed input (wall, grid, move, copy,
  array), or a door or window placed at a typed distance (issue #250).

- `pub fn typing_two_lengths(&self) -> bool`

  The tool takes two typed lengths (Tab switches between them): a door
  or window's distances from the start and from the end of its wall.

- `pub fn set_tool(&mut self, tool: Tool)`
- `pub fn active_level(&self) -> Option<LevelId>`

  Level of the active plan view, if the active view is a plan.

- `pub fn delete_selection(&mut self)`
- `pub fn create_element(&mut self, name: String, level: LevelId, type_id: ElementTypeId, kind: cc_core::ElementKind) -> ElementId`

  Create an element in one transaction and select it (test setup; the
  tools create elements through engine commands).

- `pub fn view_title(&self, view: ViewId) -> String`

  Title shown in tabs and window title for a view.


<a id="mod-cc_app-steel_building_ui"></a>

## `cc_app::steel_building_ui`

<sub>`crates/cc_app/src/steel_building_ui.rs`</sub>

*Structure → Steel Building*: the dialog that runs
`steel_building_create` on the active level, and the properties of a
selected building (every parameter, through `element_set`, which
regenerates its members in the same undo step).

**Items:** [`SteelBuildingDialog`](#cc_app-steel_building_ui-steelbuildingdialog) · [`open`](#cc_app-steel_building_ui-open) · [`show`](#cc_app-steel_building_ui-show) · [`edit`](#cc_app-steel_building_ui-edit) · [`props`](#cc_app-steel_building_ui-props)

<a id="cc_app-steel_building_ui-steelbuildingdialog"></a>

### struct `SteelBuildingDialog`

```rust
pub struct SteelBuildingDialog
```

**Traits:** `Debug`, `Clone`, `Default`

**Fields**

- `span: f64`
- `eave_height: f64`
- `slope_percent: f64`
- `roof: FrameRoof`
- `bays: u32`
- `bay_spacing: f64`
- `origin: [f64; 2]`
- `rotation: f64`
- `column_type: Option<ElementTypeId>`

  `None`: the catalog default.

- `rafter_type: Option<ElementTypeId>`
- `purlins: bool`
- `purlin_type: Option<ElementTypeId>`
- `purlin_spacing: f64`
- `girts: bool`
- `girt_type: Option<ElementTypeId>`
- `girt_spacing: f64`
- `error: Option<String>`

<a id="cc_app-steel_building_ui-open"></a>

### fn `open`

```rust
pub fn open(state: &mut AppState)
```

<a id="cc_app-steel_building_ui-show"></a>

### fn `show`

```rust
pub fn show(ctx: &Context, state: &mut AppState)
```

<a id="cc_app-steel_building_ui-edit"></a>

### fn `edit`

```rust
pub fn edit(state: &mut AppState, id: ElementId, field: &str, f: impl FnOnce(&mut SteelBuilding))
```

Change building `id` as `f` changes a copy of it (`element_set`, which
regenerates its members); edits under one `field` merge while dragging.

<a id="cc_app-steel_building_ui-props"></a>

### fn `props`

```rust
pub fn props(ui: &mut Ui, state: &mut AppState, id: ElementId, building: &SteelBuilding)
```

The rows of a selected steel building.

<a id="mod-cc_app-sun_overlay"></a>

## `cc_app::sun_overlay`

<sub>`crates/cc_app/src/sun_overlay.rs`</sub>

A 3D view's sun: the light the renderer draws it with
(`cc_render::Light`) and its sun path diagram painted over the image
(`cc_views::sun_path::diagram`, projected through the live camera).
Both are resolved once per document revision and kept: a sun placed
relative to the view (lighting `relative_to_view`) is resolved again
when the live camera turns, so it follows the viewer while orbiting;
every other sun stays put. The diagram's ring and paths do not depend on
the camera; only the sun's disc of a view-relative sun moves with it.
The diagram is drawn over the model (never hidden by it) and is not
picked.

**Items:** [`SunView`](#cc_app-sun_overlay-sunview)

<a id="cc_app-sun_overlay-sunview"></a>

### struct `SunView`

```rust
pub struct SunView
```

A 3D view's resolved sun.

**Traits:** `Default`

*Some fields are private.*

**Methods**

- `pub fn update(&mut self, project: &Project, geom: &ModelGeometry, view: &View, revision: u64, camera: &CameraState) -> Option<cc_render::Light>`

  Bring the light and the diagram up to date for `view` at document
  `revision`, the viewer looking through `camera`; returns the light
  (`None`: the view has no sun, and the camera's light is used).

- `pub fn paint(&self, painter: &Painter, frame: &Frame)`

  Paint the sun path diagram (when the view shows it) through `frame`.


<a id="mod-cc_app-sun_ui"></a>

## `cc_app::sun_ui`

<sub>`crates/cc_app/src/sun_ui.rs`</sub>

The Sun Settings dialog (*View → Sun Settings*, and the *Sun* row of a
view's properties): how a view places its sun (one moment, a study over
one or several days, or a light from a direction given by hand), the
project's sun presets, the site's time zone and what is drawn (the sun
path, shadows, a ground plane). Every change runs a command as it is
made (`view_set`, `sun_preset_*`, `site_set`), so the view behind shows
it at once; OK keeps them all as one undo step, Cancel undoes them
(`dialog::session`, through `crate::graphics_dialog::Edits`).
The Site Location window is not opened from here: it keeps its own
dialog session, which would end this one's. A hint names where it is.

**Items:** [`SunDialog`](#cc_app-sun_ui-sundialog) · [`open`](#cc_app-sun_ui-open) · [`open_for`](#cc_app-sun_ui-open_for) · [`show`](#cc_app-sun_ui-show) · [`view_rows`](#cc_app-sun_ui-view_rows)

<a id="cc_app-sun_ui-sundialog"></a>

### struct `SunDialog`

```rust
pub struct SunDialog
```

The open Sun Settings dialog.

*Some fields are private.*

<a id="cc_app-sun_ui-open"></a>

### fn `open`

```rust
pub fn open(state: &mut AppState)
```

Open the dialog on the active view (when it takes a sun).

<a id="cc_app-sun_ui-open_for"></a>

### fn `open_for`

```rust
pub fn open_for(state: &mut AppState, view: ViewId)
```

Open the dialog on `view`. A view without a sun gets the default one,
inside the dialog's edits (Cancel takes it away again).

<a id="cc_app-sun_ui-show"></a>

### fn `show`

```rust
pub fn show(ctx: &Context, state: &mut AppState)
```

<a id="cc_app-sun_ui-view_rows"></a>

### fn `view_rows`

```rust
pub fn view_rows(ui: &mut Ui, state: &mut AppState, id: ViewId)
```

The sun rows of a view's graphics section: the *Sun* button (it opens
the dialog), when the view has a sun the sun path and shadows, and the
view's shading (ambient light and shadow intensity).

<a id="mod-cc_app-systems_dialog"></a>

## `cc_app::systems_dialog`

<sub>`crates/cc_app/src/systems_dialog.rs`</sub>

Plumbing systems, electrical circuits, the structured cabling of racks
and lightning protection systems: names, codes, colors, the board (or
rack) feeding each circuit (or cabling) and its voltage, with the
circuits' loads.
Every change is an engine command (`system_*`); Cancel undoes them all.

**Items:** [`panels`](#cc_app-systems_dialog-panels) · [`boards`](#cc_app-systems_dialog-boards) · [`system_combo`](#cc_app-systems_dialog-system_combo) · [`show`](#cc_app-systems_dialog-show) · [`sizing`](#cc_app-systems_dialog-sizing)

<a id="cc_app-systems_dialog-panels"></a>

### fn `panels`

```rust
pub fn panels(state: &AppState) -> Vec<(ElementId, String)>
```

The distribution boards of the project: (element, label).

<a id="cc_app-systems_dialog-boards"></a>

### fn `boards`

```rust
pub fn boards(state: &AppState, board: DeviceKind) -> Vec<(ElementId, String)>
```

The boards (or racks) of the project: (element, label).

<a id="cc_app-systems_dialog-system_combo"></a>

### fn `system_combo`

```rust
pub fn system_combo(ui: &mut Ui, state: &AppState, id: impl std::hash::Hash + std::fmt::Debug, classes: &[SystemClass], current: &mut Option<SystemId>) -> bool
```

A choice of the systems of the families `classes` (any, when empty), with "none".

<a id="cc_app-systems_dialog-show"></a>

### fn `show`

```rust
pub fn show(ctx: &Context, state: &mut AppState)
```

<a id="cc_app-systems_dialog-sizing"></a>

### fn `sizing`

```rust
pub fn sizing(ctx: &Context, state: &mut AppState)
```

The sizing of the pipes and circuits (`mep_sizing`), with a button that
gives undersized pipes the required diameters (`mep_size_apply`).

<a id="mod-cc_app-table"></a>

## `cc_app::table`

<sub>`crates/cc_app/src/table.rs`</sub>

Property table primitives shared by the properties palette and dialogs:
collapsible sections of two-column rows, header blocks and padded content.

**Items:** [`ROW_H`](#cc_app-table-row_h) · [`LABEL_SHARE`](#cc_app-table-label_share) · [`FONT_SIZE`](#cc_app-table-font_size) · [`font_size`](#cc_app-table-font_size) · [`section`](#cc_app-table-section) · [`row`](#cc_app-table-row) · [`row_tip`](#cc_app-table-row_tip) · [`boxed`](#cc_app-table-boxed) · [`checkbox`](#cc_app-table-checkbox) · [`labeled_checkbox`](#cc_app-table-labeled_checkbox) · [`button_row`](#cc_app-table-button_row) · [`readonly_row`](#cc_app-table-readonly_row) · [`header_block`](#cc_app-table-header_block) · [`padded`](#cc_app-table-padded) · [`live_text`](#cc_app-table-live_text) · [`commit_text`](#cc_app-table-commit_text)

<a id="cc_app-table-row_h"></a>

### const `ROW_H`

```rust
pub const ROW_H: f32
```

Height of a table row and of a section header.

<a id="cc_app-table-label_share"></a>

### const `LABEL_SHARE`

```rust
pub const LABEL_SHARE: f32
```

Share of the table width taken by the label column.

<a id="cc_app-table-font_size"></a>

### const `FONT_SIZE`

```rust
pub const FONT_SIZE: f32
```

<a id="cc_app-table-font_size"></a>

### fn `font_size`

```rust
pub fn font_size() -> f32
```

Text size of the tables, following the font size preference.

<a id="cc_app-table-section"></a>

### fn `section`

```rust
pub fn section(ui: &mut Ui, id: &str, title: String, add: impl FnOnce(&mut Ui))
```

Collapsible section header; `add` draws the rows while it is open.

<a id="cc_app-table-row"></a>

### fn `row`

```rust
pub fn row<R>(ui: &mut Ui, label: &str, add: impl FnOnce(&mut Ui) -> R) -> R
```

One table row: a label cell and a value cell filled by `add`.

<a id="cc_app-table-row_tip"></a>

### fn `row_tip`

```rust
pub fn row_tip<R>(ui: &mut Ui, label: &str, tip: Option<&str>, add: impl FnOnce(&mut Ui) -> R) -> R
```

`row` whose label shows `tip` when hovered (else the label itself).

<a id="cc_app-table-boxed"></a>

### fn `boxed`

```rust
pub fn boxed<'a>(checkbox: egui::Checkbox<'a>) -> impl egui::Widget + 'a
```

A checkbox that shows its box wherever it stands. Table cells are flat
(no border or fill until hovered), which would leave an unticked box the
color of the background.

<a id="cc_app-table-checkbox"></a>

### fn `checkbox`

```rust
pub fn checkbox(ui: &mut Ui, checked: &mut bool) -> egui::Response
```

A checkbox without text, as `boxed`.

<a id="cc_app-table-labeled_checkbox"></a>

### fn `labeled_checkbox`

```rust
pub fn labeled_checkbox(ui: &mut Ui, checked: &mut bool, text: impl Into<egui::WidgetText>) -> egui::Response
```

A checkbox with a text beside it, as `boxed`.

<a id="cc_app-table-button_row"></a>

### fn `button_row`

```rust
pub fn button_row<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R
```

A row of buttons across the whole table, wrapping when they do not fit;
`add` draws them.

<a id="cc_app-table-readonly_row"></a>

### fn `readonly_row`

```rust
pub fn readonly_row(ui: &mut Ui, label: &str, value: String)
```

A row whose value cannot be edited (shown dimmed).

<a id="cc_app-table-header_block"></a>

### fn `header_block`

```rust
pub fn header_block(ui: &mut Ui, icon: Icon, title: &str, subtitle: &str)
```

Header with a large icon tile, a title and a subtitle.

<a id="cc_app-table-padded"></a>

### fn `padded`

```rust
pub fn padded<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R
```

Content with the panel's side padding (headers, buttons, lists).

<a id="cc_app-table-live_text"></a>

### fn `live_text`

```rust
pub fn live_text(ui: &mut Ui, salt: impl std::hash::Hash + std::fmt::Debug, text: &mut String, add: impl FnOnce(&mut Ui, &mut String) -> egui::Response) -> egui::Response
```

A single-line text field over a value read from the model every frame.
While it has the focus it shows what was typed, not the value read back,
so a command that trims or refuses the text mid-typing (a trailing
space, an emptied name) or a value only kept when the field is left
never undoes the typing. `text` holds what was typed this frame; `salt`
names the field (unique among those shown), `add` draws it.

<a id="cc_app-table-commit_text"></a>

### fn `commit_text`

```rust
pub fn commit_text(ui: &mut Ui, salt: impl std::hash::Hash + std::fmt::Debug, current: &str, width: f32, hint: &str) -> Option<String>
```

A one-line field that hands back its text, trimmed, when Enter is
pressed or it loses the focus, and only if it changed. While focused it
keeps what is typed; otherwise it shows `current`. `hint` shows when empty.

<a id="mod-cc_app-text_tool"></a>

## `cc_app::text_tool`

<sub>`crates/cc_app/src/text_tool.rs`</sub>

Text notes in 2D views: the Text tool (single-line or multiline text,
optionally with a leader of one or two segments), the Add Leader tool and
the text editor shown over the view. Notes are created and changed
through the engine's commands.

**Items:** [`LeaderShape`](#cc_app-text_tool-leadershape) · [`TextOptions`](#cc_app-text_tool-textoptions) · [`TextTarget`](#cc_app-text_tool-texttarget) · [`TextEditor`](#cc_app-text_tool-texteditor) · [`text_tool`](#cc_app-text_tool-text_tool) · [`add_leader_tool`](#cc_app-text_tool-add_leader_tool) · [`canvas`](#cc_app-text_tool-canvas) · [`show`](#cc_app-text_tool-show) · [`options`](#cc_app-text_tool-options) · [`symbol_combo`](#cc_app-text_tool-symbol_combo)

<a id="cc_app-text_tool-leadershape"></a>

### enum `LeaderShape`

```rust
pub enum LeaderShape
```

The leader a new text gets.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Default`

**Variants**

- `None`
- `Straight`

  One segment from the text to the point.

- `Elbow`

  Two segments: from the text to an elbow, then to the point.


**Methods**

- `pub const ALL: [LeaderShape; 3]`
- `pub fn key(self) -> &'static str`

<a id="cc_app-text_tool-textoptions"></a>

### struct `TextOptions`

```rust
pub struct TextOptions
```

Options of the text tools (kept between uses, shown in the options bar).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`

**Fields**

- `multiline: bool`
- `width: f64`

  Wrapping width of multiline text, paper millimeters.

- `leader: LeaderShape`
- `symbol: LeaderSymbol`

<a id="cc_app-text_tool-texttarget"></a>

### enum `TextTarget`

```rust
pub enum TextTarget
```

What the editor's text goes to.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Variants**

- `New { type_id: ElementTypeId, position: DVec2, leaders: Vec<Leader>, width: Option<f64> }`

  A new note at `position` with `leaders`.

- `Existing(ElementId)`

  An existing note.


<a id="cc_app-text_tool-texteditor"></a>

### struct `TextEditor`

```rust
pub struct TextEditor
```

The text editor open over a view.

**Traits:** `Debug`, `Clone`

*Some fields are private.*

<a id="cc_app-text_tool-text_tool"></a>

### fn `text_tool`

```rust
pub fn text_tool(ui: &Ui, resp: &Response, painter: &Painter, state: &mut AppState, view: &cc_core::View, type_id: ElementTypeId, xf: &Xf, px_per_mm: f64)
```

The Text tool: click the leader's end and elbow (when the options ask for
a leader), then where the text goes, and type it.

<a id="cc_app-text_tool-add_leader_tool"></a>

### fn `add_leader_tool`

```rust
pub fn add_leader_tool(ui: &Ui, resp: &Response, painter: &Painter, state: &mut AppState, view: &cc_core::View, id: ElementId, xf: &Xf, px_per_mm: f64)
```

The Add Leader tool: click the end (and elbow) of a new leader of note `id`.

<a id="cc_app-text_tool-canvas"></a>

### fn `canvas`

```rust
pub fn canvas(resp: &Response, state: &mut AppState, view: ViewId, xf: &Xf, drawing: &Drawing2D) -> bool
```

A double click on a note with the Select tool opens it in the editor.
Returns true when the pointer was used.

<a id="cc_app-text_tool-show"></a>

### fn `show`

```rust
pub fn show(ui: &Ui, state: &mut AppState, view: ViewId)
```

The text editor over `view`, when open there: Enter (single-line) or
Ctrl+Enter (multiline) or OK applies, Esc or Cancel closes it.

<a id="cc_app-text_tool-options"></a>

### fn `options`

```rust
pub fn options(ui: &mut Ui, state: &mut AppState, adding_leader: bool)
```

Options bar of the text tools.

<a id="cc_app-text_tool-symbol_combo"></a>

### fn `symbol_combo`

```rust
pub fn symbol_combo(ui: &mut Ui, id: impl std::hash::Hash + std::fmt::Debug, symbol: &mut LeaderSymbol) -> bool
```

A combo box of leader symbols; returns whether it changed.

<a id="mod-cc_app-theme"></a>

## `cc_app::theme`

<sub>`crates/cc_app/src/theme.rs`</sub>

Visual theme: color tokens, typography and spacing applied to egui.
Colors are defined once in `Palette` and every panel reads them from
`PALETTE` instead of hard-coding values, so the look stays consistent.
`LIGHT` and `DARK` hold the tokens of the two themes; `PALETTE` reads
the one the interface shows (`sync` follows the egui theme each frame),
and `ThemeChoice` is the user's preference: light, dark or the system's.

**Items:** [`Palette`](#cc_app-theme-palette) · [`LIGHT`](#cc_app-theme-light) · [`DARK`](#cc_app-theme-dark) · [`palette`](#cc_app-theme-palette) · [`FONT_SCALE_RANGE`](#cc_app-theme-font_scale_range) · [`font_scale`](#cc_app-theme-font_scale) · [`set_font_scale`](#cc_app-theme-set_font_scale) · [`ShownPalette`](#cc_app-theme-shownpalette) · [`PALETTE`](#cc_app-theme-palette) · [`sync`](#cc_app-theme-sync) · [`ThemeChoice`](#cc_app-theme-themechoice) · [`set_theme`](#cc_app-theme-set_theme) · [`APP_ICON_PNG`](#cc_app-theme-app_icon_png) · [`logo`](#cc_app-theme-logo) · [`RADIUS`](#cc_app-theme-radius) · [`semibold`](#cc_app-theme-semibold) · [`title`](#cc_app-theme-title) · [`panel_header`](#cc_app-theme-panel_header) · [`caption`](#cc_app-theme-caption) · [`muted`](#cc_app-theme-muted) · [`apply`](#cc_app-theme-apply) · [`dock_style`](#cc_app-theme-dock_style)

<a id="cc_app-theme-palette"></a>

### struct `Palette`

```rust
pub struct Palette
```

Color tokens of a theme.

**Traits:** `Debug`, `Clone`, `Copy`

**Fields**

- `app_bg: Color32`

  Application background behind panels (ribbon strip, dock gaps).

- `panel: Color32`

  Side panels, ribbon and popups.

- `surface: Color32`

  Section headers and subtle fills inside panels.

- `canvas: Color32`

  Background of view tabs and the home screen.

- `paper: Color32`

  Paper under 2D drawings: white in both themes, since drawings are
  dark lines and text.

- `sheet_bg: Color32`

  Area around a sheet.

- `border: Color32`
- `border_strong: Color32`
- `text: Color32`
- `text_muted: Color32`
- `text_faint: Color32`
- `accent: Color32`
- `accent_soft: Color32`
- `accent_text: Color32`
- `hover: Color32`
- `pressed: Color32`
- `danger: Color32`
- `warning: Color32`
- `success: Color32`

  Positive actions: badges of commands that create something.

- `temporary: Color32`

  Border of a view with temporary hide/isolate.

- `icon_ink: Color32`

  Plain interface icons (strokes).

- `icon_tint: Color32`

  Second tone of two-tone icons (their soft fills).

- `matte: Color32`

  Neutral dark matte around a rendered picture (dark in both themes,
  so the picture's tones read the same).

- `matte_text: Color32`

  Text and marks drawn on `Palette::matte`.

- `dark: bool`

  Whether this is a dark theme (egui base visuals).


<a id="cc_app-theme-light"></a>

### const `LIGHT`

```rust
pub const LIGHT: Palette
```

Color tokens of the light theme.

<a id="cc_app-theme-dark"></a>

### const `DARK`

```rust
pub const DARK: Palette
```

Color tokens of the dark theme.

<a id="cc_app-theme-palette"></a>

### fn `palette`

```rust
pub fn palette(ctx: &egui::Context) -> &'static Palette
```

The palette of the theme `ctx` shows.

<a id="cc_app-theme-font_scale_range"></a>

### const `FONT_SCALE_RANGE`

```rust
pub const FONT_SCALE_RANGE: std::ops::RangeInclusive<f32>
```

Smallest and largest font size multipliers.

<a id="cc_app-theme-font_scale"></a>

### fn `font_scale`

```rust
pub fn font_scale() -> f32
```

The multiplier of the interface text sizes (see `set_font_scale`).

<a id="cc_app-theme-set_font_scale"></a>

### fn `set_font_scale`

```rust
pub fn set_font_scale(ctx: &egui::Context, scale: f32)
```

Make the interface text `scale` times its normal size (independent of
the interface zoom): the text styles of both themes and the property
tables follow it. Out-of-range values are clamped.

<a id="cc_app-theme-shownpalette"></a>

### struct `ShownPalette`

```rust
pub struct ShownPalette
```

The palette of the theme shown: read like a constant (`PALETTE.text`).

**Traits:** `Debug`, `Clone`, `Copy`, `std::ops::Deref`

<a id="cc_app-theme-palette"></a>

### static `PALETTE`

```rust
pub static PALETTE: ShownPalette
```

The colors of the theme the interface shows (see `sync`).

<a id="cc_app-theme-sync"></a>

### fn `sync`

```rust
pub fn sync(ctx: &egui::Context)
```

Make `PALETTE` follow the theme `ctx` shows; call it at the start of
every frame (the system theme may change at any time).

<a id="cc_app-theme-themechoice"></a>

### enum `ThemeChoice`

```rust
pub enum ThemeChoice
```

The user's theme preference.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Default`

**Variants**

- `System`

  Follow the operating system.

- `Light`
- `Dark`

**Methods**

- `pub const ALL: [ThemeChoice; 3]`
- `pub fn code(self) -> &'static str`

  Stored form of the preference.

- `pub fn from_code(code: &str) -> Option<Self>`
- `pub fn label(self) -> String`

<a id="cc_app-theme-set_theme"></a>

### fn `set_theme`

```rust
pub fn set_theme(ctx: &egui::Context, choice: ThemeChoice)
```

Show the theme `choice` asks for.

<a id="cc_app-theme-app_icon_png"></a>

### const `APP_ICON_PNG`

```rust
pub const APP_ICON_PNG: &[u8]
```

The application icon (256 px PNG).

<a id="cc_app-theme-logo"></a>

### fn `logo`

```rust
pub fn logo(ctx: &egui::Context) -> Option<egui::TextureHandle>
```

Texture of the application icon, loaded by `apply` for the ribbon.

<a id="cc_app-theme-radius"></a>

### const `RADIUS`

```rust
pub const RADIUS: u8
```

Corner radius of buttons, fields and popups.

<a id="cc_app-theme-semibold"></a>

### fn `semibold`

```rust
pub fn semibold() -> FontFamily
```

Font family of the semibold weight, for titles and emphasis.

<a id="cc_app-theme-title"></a>

### fn `title`

```rust
pub fn title(text: impl Into<String>) -> RichText
```

Section or panel title text.

<a id="cc_app-theme-panel_header"></a>

### fn `panel_header`

```rust
pub fn panel_header(ui: &mut egui::Ui, title_text: &str, subtitle: Option<&str>) -> egui::Response
```

Full-width header strip at the top of a side panel; its response covers the strip.

<a id="cc_app-theme-caption"></a>

### fn `caption`

```rust
pub fn caption(text: impl Into<String>) -> RichText
```

Small uppercase-free caption, used for group labels.

<a id="cc_app-theme-muted"></a>

### fn `muted`

```rust
pub fn muted(text: impl Into<String>) -> RichText
```

Secondary text.

<a id="cc_app-theme-apply"></a>

### fn `apply`

```rust
pub fn apply(ctx: &egui::Context)
```

Install fonts, text styles, spacing and colors.

<a id="cc_app-theme-dock_style"></a>

### fn `dock_style`

```rust
pub fn dock_style(style: &egui::Style) -> egui_dock::Style
```

Dock (view tabs) style matching the theme.

<a id="mod-cc_app-thumbnails"></a>

## `cc_app::thumbnails`

<sub>`crates/cc_app/src/thumbnails.rs`</sub>

Project thumbnails: a small render of the model, stored in the application
data folder and shown on the home screen.

**Items:** [`SIZE`](#cc_app-thumbnails-size) · [`Job`](#cc_app-thumbnails-job) · [`project_file`](#cc_app-thumbnails-project_file) · [`sample_file`](#cc_app-thumbnails-sample_file) · [`process`](#cc_app-thumbnails-process)

<a id="cc_app-thumbnails-size"></a>

### const `SIZE`

```rust
pub const SIZE: [u32; 2]
```

Thumbnail size in pixels (3:2).

<a id="cc_app-thumbnails-job"></a>

### enum `Job`

```rust
pub enum Job
```

A thumbnail waiting to be rendered (rendering needs the GPU, which only the
app loop has).

**Traits:** `Debug`, `Clone`, `PartialEq`

**Variants**

- `Project(PathBuf)`

  The open document, saved at this path.

- `Sample(&'static str)`

  A bundled sample project.


<a id="cc_app-thumbnails-project_file"></a>

### fn `project_file`

```rust
pub fn project_file(path: &Path) -> Option<PathBuf>
```

<a id="cc_app-thumbnails-sample_file"></a>

### fn `sample_file`

```rust
pub fn sample_file(name: &str) -> Option<PathBuf>
```

<a id="cc_app-thumbnails-process"></a>

### fn `process`

```rust
pub fn process(state: &mut AppState, rs: &egui_wgpu::RenderState)
```

Render pending thumbnails.

<a id="mod-cc_app-title_block_env"></a>

## `cc_app::title_block_env`

<sub>`crates/cc_app/src/title_block_env.rs`</sub>

The title block editor: the component environment for a title block
(`ComponentKind::TitleBlock`). One *Sheet* view takes the place of the
plan, front, side and 3D views: the paper at a preview size (the sheet
the editor opened from, or A0–A4 portrait or landscape), its four edges
as locked reference planes, the component's own reference planes, and
the title block's pieces with their fields filled in (from that sheet,
else sample values) or shown as typed.
The canvas works in paper meters (millimeters / `MM`), so the sketch's
snapping to the millimeter, its pick distances and typed lengths
measure the printed paper; pieces are written in millimeters, the unit
of title block expressions. A coordinate on an edge is written `0`,
`SheetWidth` or `SheetHeight`, one on a reference plane as its name, so
the piece follows the paper. Every change is a `component_*` command.

**Items:** [`MM`](#cc_app-title_block_env-mm) · [`Preview`](#cc_app-title_block_env-preview) · [`SheetEditor`](#cc_app-title_block_env-sheeteditor) · [`is_open`](#cc_app-title_block_env-is_open) · [`sheet_title_block_of`](#cc_app-title_block_env-sheet_title_block_of) · [`sheet_title_block`](#cc_app-title_block_env-sheet_title_block) · [`edit_sheet`](#cc_app-title_block_env-edit_sheet) · [`new_title_block`](#cc_app-title_block_env-new_title_block) · [`iso_paper`](#cc_app-title_block_env-iso_paper) · [`paper`](#cc_app-title_block_env-paper) · [`scope_extra`](#cc_app-title_block_env-scope_extra) · [`SheetPlane`](#cc_app-title_block_env-sheetplane) · [`planes`](#cc_app-title_block_env-planes) · [`plane_view`](#cc_app-title_block_env-plane_view) · [`field_value`](#cc_app-title_block_env-field_value) · [`central`](#cc_app-title_block_env-central) · [`text_box`](#cc_app-title_block_env-text_box) · [`piece_at`](#cc_app-title_block_env-piece_at) · [`pieces_in`](#cc_app-title_block_env-pieces_in) · [`Picture`](#cc_app-title_block_env-picture) · [`picture`](#cc_app-title_block_env-picture) · [`picture_file`](#cc_app-title_block_env-picture_file) · [`image_rect`](#cc_app-title_block_env-image_rect) · [`pick_image`](#cc_app-title_block_env-pick_image) · [`coordinate`](#cc_app-title_block_env-coordinate) · [`place_text`](#cc_app-title_block_env-place_text) · [`field_menu`](#cc_app-title_block_env-field_menu) · [`insert_field`](#cc_app-title_block_env-insert_field) · [`sheet_plane_from_line`](#cc_app-title_block_env-sheet_plane_from_line) · [`is_sheet_sketch`](#cc_app-title_block_env-is_sheet_sketch) · [`start_lines`](#cc_app-title_block_env-start_lines) · [`chains`](#cc_app-title_block_env-chains) · [`finish_stroke`](#cc_app-title_block_env-finish_stroke) · [`edit_stroke`](#cc_app-title_block_env-edit_stroke) · [`delete_pieces`](#cc_app-title_block_env-delete_pieces) · [`ribbon`](#cc_app-title_block_env-ribbon) · [`options`](#cc_app-title_block_env-options) · [`panel_content`](#cc_app-title_block_env-panel_content)

<a id="cc_app-title_block_env-mm"></a>

### const `MM`

```rust
pub const MM: f64
```

Millimeters per canvas unit (the canvas is in paper meters).

<a id="cc_app-title_block_env-preview"></a>

### enum `Preview`

```rust
pub enum Preview
```

The paper the Sheet view shows.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Variants**

- `Sheet(ViewId)`

  A sheet's own paper.

- `Paper { size: SheetSize, portrait: bool }`

  An ISO size, portrait or landscape.


<a id="cc_app-title_block_env-sheeteditor"></a>

### struct `SheetEditor`

```rust
pub struct SheetEditor
```

The Sheet view's state.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `sheet: Option<ViewId>`

  The sheet the editor opened from: fields read it (else sample values).

- `preview: Preview`
- `raw_keys: bool`

  Show `{keys}` as typed instead of their values.

- `editing: Option<(usize, Vec<ProfilePoint>)>`

  The stroke whose sketch is open, with its points as they were.


*Some fields are private.*

**Methods**

- `pub fn new(sheet: Option<ViewId>, system: MeasurementSystem) -> Self`

  The editor of `sheet`'s title block, else previewed on the paper of
  a new sheet of the project's `system`.


<a id="cc_app-title_block_env-is_open"></a>

### fn `is_open`

```rust
pub fn is_open(state: &AppState) -> bool
```

Whether the component environment shows a title block.

<a id="cc_app-title_block_env-sheet_title_block_of"></a>

### fn `sheet_title_block_of`

```rust
pub fn sheet_title_block_of(state: &AppState, id: ViewId) -> Option<ElementTypeId>
```

The title block type of sheet `id`.

<a id="cc_app-title_block_env-sheet_title_block"></a>

### fn `sheet_title_block`

```rust
pub fn sheet_title_block(state: &AppState) -> Option<ElementTypeId>
```

The active sheet's title block type.

<a id="cc_app-title_block_env-edit_sheet"></a>

### fn `edit_sheet`

```rust
pub fn edit_sheet(state: &mut AppState, id: ViewId)
```

*Edit Title Block* on sheet `id`: make it the active view, then open its
title block on it. `open_view` only queues the tab, so the active view is
set here for the editor to read the sheet.

<a id="cc_app-title_block_env-new_title_block"></a>

### fn `new_title_block`

```rust
pub fn new_title_block(state: &mut AppState)
```

*New Title Block*: a copy of the standard layout under a name no type and
no component uses (`title_block_create` refuses both), opened.

<a id="cc_app-title_block_env-iso_paper"></a>

### fn `iso_paper`

```rust
pub fn iso_paper(size: SheetSize, portrait: bool) -> DVec2
```

An ISO paper in millimeters, portrait (long side up) or landscape.

<a id="cc_app-title_block_env-paper"></a>

### fn `paper`

```rust
pub fn paper(state: &AppState, preview: Preview) -> DVec2
```

The paper shown, millimeters.

<a id="cc_app-title_block_env-scope_extra"></a>

### fn `scope_extra`

```rust
pub fn scope_extra(paper: DVec2) -> [(&'static str, f64); 2]
```

The names expressions read for the paper, millimeters.

<a id="cc_app-title_block_env-sheetplane"></a>

### struct `SheetPlane`

```rust
pub struct SheetPlane
```

A line pieces snap to and are written against.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `name: String`

  What a coordinate on it is written as (`0`, `SheetWidth`, a plane's name).

- `label: String`

  What the canvas labels it.

- `vertical: bool`

  A vertical line (it fixes x); else horizontal (it fixes y).

- `at: f64`

  Where it stands, millimeters.

- `edge: bool`

  A paper edge (locked).


<a id="cc_app-title_block_env-planes"></a>

### fn `planes`

```rust
pub fn planes(def: &ComponentDef, paper: DVec2) -> Vec<SheetPlane>
```

The four paper edges, then the component's planes square to x or y
that resolve on this paper.

<a id="cc_app-title_block_env-plane_view"></a>

### fn `plane_view`

```rust
pub fn plane_view(planes: &[SheetPlane], k: f64) -> ComponentView
```

The planes as the lines of a `ComponentView`, `k` canvas units per
millimeter: the sketch snaps to them (`crate::component_sketch::snap`)
and writes coordinates on them as their names.

<a id="cc_app-title_block_env-field_value"></a>

### fn `field_value`

```rust
pub fn field_value(state: &AppState, ed: &SheetEditor, def: &ComponentDef, key: &str) -> Option<String>
```

The value of field `key` in the preview: from the sheet the editor
opened from, else a sample; `paper` is always the preview's.

<a id="cc_app-title_block_env-central"></a>

### fn `central`

```rust
pub fn central(ui: &mut Ui, state: &mut AppState)
```

The Sheet view: the paper, the planes, the title block, and the tools.

<a id="cc_app-title_block_env-text_box"></a>

### fn `text_box`

```rust
pub fn text_box(t: &SheetText) -> (DVec2, DVec2)
```

A text's extents (millimeters), estimated: 0.6 × its height per
character and 1.5 × its height per line, from its anchor, turned.

<a id="cc_app-title_block_env-piece_at"></a>

### fn `piece_at`

```rust
pub fn piece_at(pieces: &SheetPieces, p: DVec2, tol: f64) -> Option<usize>
```

The piece at `p` (millimeters): a text whose box holds it, else the
nearest stroke within `tol`, else a picture under it.

<a id="cc_app-title_block_env-pieces_in"></a>

### fn `pieces_in`

```rust
pub fn pieces_in(pieces: &SheetPieces, a: DVec2, b: DVec2) -> Vec<usize>
```

The pieces wholly inside the window from `a` to `b` (millimeters), in order.

<a id="cc_app-title_block_env-picture"></a>

### struct `Picture`

```rust
pub struct Picture
```

A picture for the Image tool: PNG, JPEG, GIF, BMP or WebP bytes (an SVG
drawn into a PNG), and its width / height.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `bytes: Vec<u8>`
- `aspect: f64`

<a id="cc_app-title_block_env-picture"></a>

### fn `picture`

```rust
pub fn picture(bytes: Vec<u8>) -> Option<Picture>
```

The picture in a file's bytes; an SVG is drawn into a PNG. `None` for
anything else, a file over the picture size limit, or a reader that
fails (a panic in a decoder is caught: files are hostile input).
usvg parses and renders nested elements recursively, and a deeply nested
file could overflow the caller's stack, which aborts the process; so the
file is read on a thread of its own with a large stack.

<a id="cc_app-title_block_env-picture_file"></a>

### fn `picture_file`

```rust
pub fn picture_file(path: &std::path::Path) -> Result<Picture, String>
```

The picture in file `path`; a file over the picture size limit is
refused before it is read.

<a id="cc_app-title_block_env-image_rect"></a>

### fn `image_rect`

```rust
pub fn image_rect(a: DVec2, b: DVec2, aspect: f64, free: bool) -> (DVec2, DVec2)
```

The rectangle a picture dragged from `a` to `b` fills (millimeters):
its bottom-left corner and size; the size keeps `aspect` (width /
height) unless `free`, following the longer dragged side.

<a id="cc_app-title_block_env-pick_image"></a>

### fn `pick_image`

```rust
pub fn pick_image(state: &mut AppState)
```

*Image*: pick a picture file, then drag its rectangle.

<a id="cc_app-title_block_env-coordinate"></a>

### fn `coordinate`

```rust
pub fn coordinate(planes: &[SheetPlane], value: f64, vertical: bool) -> String
```

The expression of a coordinate `value` (millimeters): the name of an
edge or plane standing there, else the number to a hundredth.

<a id="cc_app-title_block_env-place_text"></a>

### fn `place_text`

```rust
pub fn place_text(state: &mut AppState, planes: &[SheetPlane], at: DVec2)
```

Place a text at `at` (millimeters) and pick it.

<a id="cc_app-title_block_env-field_menu"></a>

### fn `field_menu`

```rust
pub fn field_menu(p: &cc_core::Project, def: &ComponentDef) -> Vec<(String, Vec<(String, String)>)>
```

The *Insert field* menu: groups of (name shown, key). The fixed fields,
with `issue_date` right after `date`, then a `project:` key per project
field and a `param:` key per text parameter of the title block.

<a id="cc_app-title_block_env-insert_field"></a>

### fn `insert_field`

```rust
pub fn insert_field(text: &str, key: &str, at: Option<usize>) -> (String, usize)
```

`text` with `{key}` inserted at character `at` (the end when `None` or
past it), and the character the cursor goes to after it.

<a id="cc_app-title_block_env-sheet_plane_from_line"></a>

### fn `sheet_plane_from_line`

```rust
pub fn sheet_plane_from_line(a: DVec2, b: DVec2) -> Option<(&'static str, f64)>
```

The plane two clicks (millimeters) make: `x` (a vertical line) or `y`,
and its position to a tenth of a millimeter.

<a id="cc_app-title_block_env-is_sheet_sketch"></a>

### fn `is_sheet_sketch`

```rust
pub fn is_sheet_sketch(state: &AppState) -> bool
```

Whether the open sketch draws a title block's lines.

<a id="cc_app-title_block_env-start_lines"></a>

### fn `start_lines`

```rust
pub fn start_lines(state: &mut AppState)
```

*Lines*: a new sketch on the Sheet view.

<a id="cc_app-title_block_env-chains"></a>

### fn `chains`

```rust
pub fn chains(curves: &[SketchCurve]) -> Vec<(Vec<SketchCurve>, bool)>
```

The curves in chains joined end to end (a chain stops where more than
two curves meet), each with whether it closes.

<a id="cc_app-title_block_env-finish_stroke"></a>

### fn `finish_stroke`

```rust
pub fn finish_stroke(state: &mut AppState)
```

*Finish* of a lines sketch: each chain becomes a stroke, closed when its
ends meet; an edited stroke takes the first chain. A chain that does not
make a stroke keeps the sketch open with the reason.

<a id="cc_app-title_block_env-edit_stroke"></a>

### fn `edit_stroke`

```rust
pub fn edit_stroke(state: &mut AppState, index: usize)
```

*Edit Sketch* of stroke `index`: its lines and arcs on the Sheet view.

<a id="cc_app-title_block_env-delete_pieces"></a>

### fn `delete_pieces`

```rust
pub fn delete_pieces(state: &mut AppState, list: &[usize])
```

Delete pieces `list`, in one step.

<a id="cc_app-title_block_env-ribbon"></a>

### fn `ribbon`

```rust
pub fn ribbon(ui: &mut Ui, state: &mut AppState)
```

The *Create* tab of the title block editor.

<a id="cc_app-title_block_env-options"></a>

### fn `options`

```rust
pub fn options(ui: &mut Ui, state: &mut AppState)
```

The options bar: the preview paper, raw keys, and the tool's hint.

<a id="cc_app-title_block_env-panel_content"></a>

### fn `panel_content`

```rust
pub fn panel_content(ui: &mut Ui, state: &mut AppState, name: &str, def: &ComponentDef)
```

The left panel: name, type, parameters (and the paper's size), and the
picked piece or plane.

<a id="mod-cc_app-tools"></a>

## `cc_app::tools`

<sub>`crates/cc_app/src/tools.rs`</sub>

Modeling tools: selection, walls, hosted openings, sketches, sections,
rooms, grids and dimensions.

**Items:** [`Tool`](#cc_app-tools-tool) · [`DimensionMode`](#cc_app-tools-dimensionmode) · [`Drag`](#cc_app-tools-drag) · [`Interaction`](#cc_app-tools-interaction) · [`ChainAt`](#cc_app-tools-chainat) · [`ChainStep`](#cc_app-tools-chainstep) · [`chain_back`](#cc_app-tools-chain_back) · [`chain_forward`](#cc_app-tools-chain_forward) · [`DimChain`](#cc_app-tools-dimchain) · [`Snap`](#cc_app-tools-snap) · [`snap_at`](#cc_app-tools-snap_at) · [`cad_snap`](#cc_app-tools-cad_snap) · [`CadIndexEntry`](#cc_app-tools-cadindexentry) · [`underlay_snap`](#cc_app-tools-underlay_snap) · [`round_step`](#cc_app-tools-round_step) · [`round_fine`](#cc_app-tools-round_fine) · [`select_box`](#cc_app-tools-select_box) · [`ModelSnaps`](#cc_app-tools-modelsnaps) · [`draw_snap`](#cc_app-tools-draw_snap) · [`label`](#cc_app-tools-label) · [`TEMP_DIM`](#cc_app-tools-temp_dim) · [`dim_text`](#cc_app-tools-dim_text) · [`temp_measure`](#cc_app-tools-temp_measure) · [`temp_dimension`](#cc_app-tools-temp_dimension) · [`dimension_line`](#cc_app-tools-dimension_line) · [`TypedInput`](#cc_app-tools-typedinput) · [`handle`](#cc_app-tools-handle) · [`run_create`](#cc_app-tools-run_create) · [`MepOptions`](#cc_app-tools-mepoptions) · [`device_type`](#cc_app-tools-device_type) · [`ComponentGroup`](#cc_app-tools-componentgroup) · [`component_group`](#cc_app-tools-component_group) · [`tool_types`](#cc_app-tools-tool_types) · [`mep_component_type`](#cc_app-tools-mep_component_type) · [`Run`](#cc_app-tools-run) · [`TrussOptions`](#cc_app-tools-trussoptions) · [`truss_command`](#cc_app-tools-truss_command) · [`BracingOptions`](#cc_app-tools-bracingoptions) · [`bracing_command`](#cc_app-tools-bracing_command) · [`foundation_type`](#cc_app-tools-foundation_type) · [`is_strip`](#cc_app-tools-is_strip) · [`grid_crossings`](#cc_app-tools-grid_crossings) · [`beam_command`](#cc_app-tools-beam_command) · [`RailingHostOptions`](#cc_app-tools-railinghostoptions) · [`railing_host`](#cc_app-tools-railing_host) · [`SweepOptions`](#cc_app-tools-sweepoptions) · [`finish_sweep`](#cc_app-tools-finish_sweep) · [`finish_railing`](#cc_app-tools-finish_railing) · [`linear_direction`](#cc_app-tools-linear_direction) · [`line_at`](#cc_app-tools-line_at) · [`point_on_segment`](#cc_app-tools-point_on_segment) · [`arc_at`](#cc_app-tools-arc_at) · [`arc_preview`](#cc_app-tools-arc_preview) · [`place_opening_3d`](#cc_app-tools-place_opening_3d) · [`add_opening`](#cc_app-tools-add_opening) · [`move_elements`](#cc_app-tools-move_elements) · [`copy_elements`](#cc_app-tools-copy_elements) · [`duplicate`](#cc_app-tools-duplicate) · [`rotate_elements`](#cc_app-tools-rotate_elements) · [`mirror_elements`](#cc_app-tools-mirror_elements) · [`rotate_selection`](#cc_app-tools-rotate_selection)

<a id="cc_app-tools-tool"></a>

### enum `Tool`

```rust
pub enum Tool
```

**Traits:** `Debug`, `Clone`, `PartialEq`

**Variants**

- `Select`
- `Wall { type_id: ElementTypeId, }`
- `Hosted { category: Category, type_id: ElementTypeId, }`
- `Sketch { category: Category, type_id: ElementTypeId, }`
- `Section`
- `Callout`

  Draw a callout's rectangle in a plan, section or elevation.

- `DetailLine`

  Draw detail lines in the active 2D view (mode and style in
  `AppState::detail`, see `crate::detail_ui`).

- `Room { type_id: ElementTypeId, }`
- `Grid { type_id: ElementTypeId, }`
- `Dimension { type_id: ElementTypeId, mode: DimensionMode, }`
- `Column { type_id: ElementTypeId, material: Option<cc_core::StructuralMaterial>, }`
- `Beam { type_id: ElementTypeId, material: Option<cc_core::StructuralMaterial>, }`
- `Footing { type_id: ElementTypeId, }`

  Click to place a pad footing; Space turns it by 90°. With a strip
  type, draw strip footings as walls are drawn (lines or arcs).

- `Truss { material: cc_core::StructuralMaterial, }`

  Click the two ends of a truss (options in `AppState::truss`) of the Structure tab's material.

- `Bracing { material: cc_core::StructuralMaterial, }`

  Click the two column axes of a braced bay (options in `AppState::bracing`).

- `Stair { type_id: ElementTypeId, }`
- `Ramp { type_id: ElementTypeId, }`
- `Railing { type_id: ElementTypeId, }`
- `Sweep { type_id: ElementTypeId, }`

  Click the points of a swept profile's path (options in `AppState::sweep`).

- `Component { type_id: ElementTypeId, }`
- `Pipe { type_id: ElementTypeId, }`

  Click the ends of pipe runs, one after the other (options in `AppState::mep`).

- `Conduit { type_id: ElementTypeId, }`

  Click the ends of conduit runs, one after the other.

- `CableTray { type_id: ElementTypeId, }`

  Click the ends of cable tray runs, one after the other.

- `LightningConductor { type_id: ElementTypeId, }`

  Click the ends of lightning protection conductors, one after the other.

- `Valve { type_id: ElementTypeId, }`

  Click a pipe to place a valve on it.

- `Move`
- `Copy`
- `Rotate`
- `Mirror { draw: bool, }`

  Mirror across a picked line (`draw: false`) or a drawn axis.

- `Array`
- `TrimCorner`
- `TrimExtend`
- `Split`
- `Offset`
- `Align`
- `PushPull`

  Push or pull a face of a mass in a 3D view.

- `ToggleJoin`

  Click near a wall end to allow or disallow its join.

- `Measure`

  Measure between two points; the result stays until the next pick.

- `JoinGeometry { unjoin: bool, }`

  Pick an element, then others to join (or unjoin) their geometry with it.

- `AttachTop`

  Click the roof, ceiling or floor the selected walls' tops attach to
  (the walls are kept in `interaction.targets`).

- `Paste`

  Place the clipboard with a click.

- `MatchType`

  Pick an element, then click others to give them its type.

- `Text { type_id: ElementTypeId, }`

  Place text notes (options in `AppState::text_options`).

- `LevelMark { type_id: ElementTypeId, }`

  Place level marks in plans, elevations and sections.

- `DetailComponent { type_id: ElementTypeId, }`

  Place detail components in 2D views (options in `AppState::detail_component`).

- `AddLeader { id: ElementId, }`

  Add a leader to the text note `id`.

- `Opening { host: ElementId, floor: bool, }`

  Sketch an opening in `host`: a courtyard in a roof, or a shaft or
  stairwell in a floor (`floor`).

- `Demolish`

  Click elements to demolish them in the view's phase.

- `CurtainGrid`

  Click curtain walls to edit their grids and panels (options in `AppState::curtain`).

- `Paint { remove: bool, }`

  Click faces in a 3D view to paint them with `AppState::paint_material`
  (or to take their paint off).


**Methods**

- `pub fn name_key(&self, project: &Project) -> &'static str`

  Translation key of the tool's name as the options bar shows it: the
  footing tool drawing a strip type is named for strips, the component
  tool for the plumbing fixtures or devices it places.

- `pub fn key(&self) -> &'static str`

  Translation key of the tool name.

- `pub fn type_id(&self) -> Option<ElementTypeId>`
- `pub fn material(&self) -> Option<cc_core::StructuralMaterial>`

  The structural material a column or beam tool's types keep to.

- `pub fn category(&self) -> Option<Category>`
- `pub fn with_type(&self, id: ElementTypeId) -> Tool`
- `pub fn for_category(project: &Project, category: Category) -> Option<Tool>`

  Tool for creating elements of `category` with its default type
  (foundations: a pad footing with the first pad type).


<a id="cc_app-tools-dimensionmode"></a>

### enum `DimensionMode`

```rust
pub enum DimensionMode
```

What the dimension tool places.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`

**Variants**

- `Aligned`
- `Linear`

  Horizontal or vertical, chosen by where the line is placed.

- `Ordinate`

  Running dimensions from an origin, horizontal or vertical.

- `Angular`
- `Radial`
- `Diameter`

**Methods**

- `pub const ALL: [DimensionMode; 6]`
- `pub fn key(self) -> &'static str`
- `pub fn hint_key(self) -> &'static str`

<a id="cc_app-tools-drag"></a>

### enum `Drag`

```rust
pub enum Drag
```

**Traits:** `Debug`, `Clone`

**Variants**

- `Box { start: Pos2, }`
- `Grip { id: ElementId, at_start: bool, fixed: DVec2, }`

  An end handle of a wall, beam or grid; `fixed` is the other end.

- `DetailGrip { id: ElementId, which: u8, }`

  A grip of a selected detail line (see `crate::sketch_mode::moved` for `which`).

- `Bend { id: ElementId, a: DVec2, b: DVec2, }`

  The curvature handle of an arc wall from `a` to `b`: the arc passes through the cursor.

- `Move { start: DVec2, ids: Vec<ElementId>, copy: bool, }`
- `Tag { tag: cc_views::TagHandle, start: DVec2, }`

  The tag of an element, as drawn when the drag started.

- `Section { view: ViewId, part: crate::section_mark::Part, start: DVec2, }`

  A section's mark in a plan: the whole mark, an end or a piece of
  its split line, grabbed at `start`.

- `Callout { view: ViewId, part: crate::callout_ui::Part, start: DVec2, }`

  A callout's mark: the whole mark or a corner, grabbed at `start`.

- `Hotspot { id: ElementId, name: String, z: f64, shape: cc_core::HotspotShape, color: Color32, }`

  A hotspot of a scripted component, at elevation `z` (absolute).


<a id="cc_app-tools-interaction"></a>

### struct `Interaction`

```rust
pub struct Interaction
```

Transient state of the active tool.

**Traits:** `Debug`, `Clone`, `Default`

**Fields**

- `targets_lines: Vec<(glam::DVec2, glam::DVec2)>`

  Lines under the measuring tool's first pick.

- `anchor: Option<DVec2>`

  Wall chain anchor or section start.

- `points: Vec<DVec2>`

  Sketch points.

- `flip_hand: bool`
- `drag: Option<Drag>`
- `targets: Vec<ElementId>`

  Elements a transform tool acts on (the selection when it started).

- `center: Option<DVec2>`

  Rotation center.

- `placing_center: bool`

  The next click sets the rotation center.

- `tracked: Vec<DVec2>`

  Points acquired for alignment guides (see `snap`).

- `prev_dir: Option<DVec2>`

  Direction back along the previous segment of a chain (angle reference).

- `rotation: f64`

  Rotation of the next component, degrees.

- `plane_z: Option<f64>`

  Height of the working plane of a tool in the 3D view (the first point picked).

- `dim_chain: Option<DimChain>`

  The dimension chain being continued (continuous dimensioning).

- `push: Option<(ElementId, cc_core::mass::MassFace, glam::DVec3, glam::DVec3)>`

  Push/pull: the mass face picked (the mass, the face, a point on it and its outward normal).

- `chain: Vec<ChainStep>`

  The segments a chain tool placed, latest last (see `chain_back`).

- `chain_redo: Vec<ChainStep>`

  Segments taken back, latest last, for redo.


**Methods**

- `pub fn is_idle(&self) -> bool`

<a id="cc_app-tools-chainat"></a>

### struct `ChainAt`

```rust
pub struct ChainAt
```

Where a chain stands: its anchor, the direction angles are measured
from, and the elevation a pipe or conduit run goes on at.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`

*Some fields are private.*

<a id="cc_app-tools-chainstep"></a>

### struct `ChainStep`

```rust
pub struct ChainStep
```

A segment a chain tool placed: the chain before and after it, its undo
steps (a pipe's rise and run are two), and the document revision it left
(or, taken back, the revision the undo left), so a step is only taken
back or redone when nothing else changed the model meanwhile.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

*Some fields are private.*

<a id="cc_app-tools-chain_back"></a>

### fn `chain_back`

```rust
pub fn chain_back(state: &mut AppState) -> bool
```

Backspace or Ctrl+Z while a chain tool is drawing: take back the last
segment (undoing it) and go on from where it started, without leaving
the tool; with no segment left, drop the start point. A railing's last
point and an arc wall's picked end go first. Returns whether it did
anything.

<a id="cc_app-tools-chain_forward"></a>

### fn `chain_forward`

```rust
pub fn chain_forward(state: &mut AppState) -> bool
```

Ctrl+Y after `chain_back`: place the segment again and go on from its end.

<a id="cc_app-tools-dimchain"></a>

### struct `DimChain`

```rust
pub struct DimChain
```

A chain of dimensions being placed: the next one measures from `last`
(the origin of ordinate dimensions, which stays) along `dir` and sits on
the line through `line`.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`

**Fields**

- `view: ViewId`
- `last: DVec2`
- `dir: DVec2`
- `line: DVec2`
- `linear: bool`

  Linear (horizontal or vertical) rather than aligned dimensions.

- `ordinate: bool`

  Running dimensions from the origin `last`.


<a id="cc_app-tools-snap"></a>

### struct `Snap`

```rust
pub(crate) struct Snap
```

**Traits:** `Debug`, `Clone`, `Copy`

**Fields**

- `point: DVec2`

*Some fields are private.*

**Methods**

- `pub(crate) fn plain(point: DVec2, on_point: bool) -> Self`

  A snap to `point`, shown as an end point when `on_point`, else free.

- `pub(crate) fn nearest(point: DVec2) -> Self`

  A snap to the nearest point of a line.

- `pub(crate) fn is_free(&self) -> bool`

  Whether nothing was snapped to.


<a id="cc_app-tools-snap_at"></a>

### fn `snap_at`

```rust
pub(crate) fn snap_at(state: &mut AppState, level: Option<LevelId>, raw: DVec2, anchor: Option<DVec2>, extra: &[DVec2], scale: f64, ortho: bool) -> Snap
```

Snap through `snap` with the interaction's acquired points, and
acquire the point when it is a model point.

<a id="cc_app-tools-cad_snap"></a>

### fn `cad_snap`

```rust
pub(crate) fn cad_snap(state: &mut AppState, raw: DVec2, r: f64) -> cc_core::cad::CadSnap
```

Snap features of the imported drawings and detail lines shown in the
active view near `raw`, and of the lines of its underlay (the other
levels' plans drawn under it).

<a id="cc_app-tools-cadindexentry"></a>

### type alias `CadIndexEntry`

```rust
pub type CadIndexEntry = (std::sync::Arc<cc_core::CadDrawing>, std::sync::Arc<cc_core::cad::CadSnapIndex>)
```

An imported drawing and the snap index built from it.

<a id="cc_app-tools-underlay_snap"></a>

### fn `underlay_snap`

```rust
pub(crate) fn underlay_snap(drawing: &Drawing2D, raw: DVec2, r: f64, out: &mut cc_core::cad::CadSnap)
```

Lines of `drawing`'s underlay passing within `r` of `raw`, with their
ends and midpoints (the underlay has no arcs: plans draw them as chords).

<a id="cc_app-tools-round_step"></a>

### fn `round_step`

```rust
pub(crate) fn round_step(scale: f64) -> f64
```

Increment for rounding free coordinates at the current zoom: 1 cm to
10 m in metric projects, 1/2" to 20'-0" in imperial ones, so drawn
lengths land on round values of the project's system.

<a id="cc_app-tools-round_fine"></a>

### fn `round_fine`

```rust
pub(crate) fn round_fine(meters: f64) -> f64
```

`meters` rounded to a millimeter, or to 1/16" in imperial projects
(component reference planes and sketches), to 1e-7 m so it writes short.

<a id="cc_app-tools-select_box"></a>

### fn `select_box`

```rust
pub(crate) fn select_box(painter: &egui::Painter, start: egui::Pos2, cur: egui::Pos2)
```

Snap `raw` to model points, wall lines, alignments and angle/length
increments. `tracked` are points acquired by hovering them earlier:
with the anchor and `extra` they cast horizontal, vertical and wall-line
guides; a crossing of two guides, or of a guide with the current
direction from the anchor, is a snap point.
The selection rectangle from `start` to `cur`: a window (solid outline)
dragged to the right, a crossing (dashed, green) dragged to the left.

<a id="cc_app-tools-modelsnaps"></a>

### struct `ModelSnaps`

```rust
pub(crate) struct ModelSnaps
```

The model's snap candidates on a level: element ends, midpoints, lines and arcs, outlines,
grid lines and crossings, risers. They depend only on the model, so they are gathered once
per revision and level (see `model_snaps_cached`), not on every pointer move.

**Traits:** `Debug`, `Default`

*Some fields are private.*

<a id="cc_app-tools-draw_snap"></a>

### fn `draw_snap`

```rust
pub(crate) fn draw_snap(painter: &Painter, xf: &Xf, s: Snap)
```

<a id="cc_app-tools-label"></a>

### fn `label`

```rust
pub(crate) fn label(painter: &Painter, pos: Pos2, text: String)
```

<a id="cc_app-tools-temp_dim"></a>

### const `TEMP_DIM`

```rust
pub(crate) const TEMP_DIM: Color32
```

Color of temporary dimensions.

<a id="cc_app-tools-dim_text"></a>

### fn `dim_text`

```rust
pub(crate) fn dim_text(painter: &Painter, pos: Pos2, text: String, angle: f32, strong: bool)
```

Text on a light background, centered at `pos` and rotated by `angle`
(radians, screen orientation), kept upright.

<a id="cc_app-tools-temp_measure"></a>

### fn `temp_measure`

```rust
pub(crate) fn temp_measure(painter: &Painter, xf: &Xf, a: DVec2, b: DVec2) -> (Pos2, egui::Vec2, f32)
```

Temporary dimension measuring from `a` to `b` (both drawn as small
ticks on the measuring line). Returns the screen center, size and
rotation of its value, for clicking.

<a id="cc_app-tools-temp_dimension"></a>

### fn `temp_dimension`

```rust
pub(crate) fn temp_dimension(painter: &Painter, xf: &Xf, a: DVec2, p: DVec2, reference: Option<DVec2>, typed: Option<&TypedInput>, side: f64)
```

Temporary dimension of a segment being drawn from `a` to `p`: a
dimension line beside it with its length (or the typed value) and an
arc with the angle to `reference` (the previous segment, or the X axis).

<a id="cc_app-tools-dimension_line"></a>

### fn `dimension_line`

```rust
pub(crate) fn dimension_line(painter: &Painter, xf: &Xf, a: DVec2, p: DVec2, typed: &str, active: bool, side: f64)
```

A temporary dimension line beside the segment from `a` to `p` (to its
left, `side` further out) with its length, or the `typed` value;
`active` marks the field typing goes to.

<a id="cc_app-tools-typedinput"></a>

### struct `TypedInput`

```rust
pub struct TypedInput
```

Values typed while drawing a segment: digits go to the active field
(length, or angle after Tab), in the project's length unit and degrees.

**Traits:** `Debug`, `Clone`, `Default`, `PartialEq`

**Fields**

- `length: String`
- `angle: String`
- `angle_active: bool`

  Typing goes to the angle.


**Methods**

- `pub fn is_empty(&self) -> bool`
- `pub fn clear(&mut self)`
- `pub fn push(&mut self, ch: char)`
- `pub fn pop(&mut self)`
- `pub fn length_value(&self) -> Option<f64>`
- `pub fn angle_value(&self) -> Option<f64>`
- `pub fn second_length(&self) -> Option<f64>`

  The second field read as a length, for tools that take two (a
  rectangle's width and depth).

- `pub fn type_char(&mut self, ch: char, second_is_length: bool) -> bool`

  Takes a character typed into the active field, if it belongs in a
  value: digits and decimal marks, and in feet and inches the marks,
  fractions (`12'6 1/2"`) and, inside a length already started, a space
  or a dash. `second_is_length` when the second field is a length too.

- `pub fn apply(&self, a: DVec2, p: DVec2, reference: Option<DVec2>) -> DVec2`

  The end of a segment from `a` toward the cursor `p`, with the typed
  length and the typed angle from `reference` (the X axis by default;
  the angle turns to the cursor's side of it).


<a id="cc_app-tools-handle"></a>

### fn `handle`

```rust
pub fn handle(ui: &Ui, resp: &Response, painter: &Painter, state: &mut AppState, view: &View, xf: &Xf, drawing: &Drawing2D)
```

<a id="cc_app-tools-run_create"></a>

### fn `run_create`

```rust
pub(crate) fn run_create<C: cc_engine::Command + serde::Serialize>(state: &mut AppState, command: C) -> Option<ElementId>
```

Run a creating command: the new element is selected; a refusal goes to the status bar.

<a id="cc_app-tools-mepoptions"></a>

### struct `MepOptions`

```rust
pub struct MepOptions
```

Options of the pipe and conduit tools and of placing devices, kept between uses.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `system: Option<cc_core::SystemId>`

  System of new pipes.

- `circuit: Option<cc_core::SystemId>`

  Circuit of new conduits and devices.

- `data: Option<cc_core::SystemId>`

  Cabling (a rack's data system) of new data outlets.

- `lightning: Option<cc_core::SystemId>`

  Lightning protection system of new conductors and devices (`None`: the engine's default).

- `offset: Option<f64>`

  Elevation of new pipes and conduits above the level (`None`: the engine's default).

- `slope: Option<f64>`

  Fall of new pipes, percent (`None`: the default for the system and diameter).

- `chain: Option<f64>`

  Elevation where the next run of a chain starts (the end of the last one).

- `rise: Option<f64>`

  Elevation a chain goes up or down to at its last point before the next
  run (a vertical run), set by changing the elevation while drawing.


<a id="cc_app-tools-device_type"></a>

### fn `device_type`

```rust
pub fn device_type(project: &Project, type_id: ElementTypeId) -> Option<cc_core::DeviceKind>
```

The device kind of a component type, if it is an electrical device.

<a id="cc_app-tools-componentgroup"></a>

### enum `ComponentGroup`

```rust
pub enum ComponentGroup
```

The ribbon tool that places a component type: the generic component tool,
plumbing fixtures, electrical devices, or a device with a tool of its own
(`commands::DEVICE_TOOLS`). The options bar lists the types of one of these.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Variants**

- `Generic`
- `Fixture`
- `Device`
- `Own(cc_core::DeviceKind)`

<a id="cc_app-tools-component_group"></a>

### fn `component_group`

```rust
pub fn component_group(project: &Project, type_id: ElementTypeId) -> ComponentGroup
```

Which tool places `type_id` (`Generic` for any type that is no fixture or device).

<a id="cc_app-tools-tool_types"></a>

### fn `tool_types`

```rust
pub fn tool_types(project: &Project, category: Category, type_id: ElementTypeId, material: Option<cc_core::StructuralMaterial>) -> Vec<(ElementTypeId, String)>
```

The types the options bar offers to a tool of `category` placing `type_id`:
all of the category, but for components only those placed by the same
tool (fixtures, electrical devices, a device with its own tool, the rest).

<a id="cc_app-tools-mep_component_type"></a>

### fn `mep_component_type`

```rust
pub fn mep_component_type(project: &Project, device: bool) -> Option<ElementTypeId>
```

The first component type that is an electrical device (`device`) or a plumbing fixture.

<a id="cc_app-tools-run"></a>

### enum `Run`

```rust
pub enum Run
```

What a run tool draws.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Drop`

**Variants**

- `Pipe`
- `Conduit`
- `Tray`
- `Conductor`

<a id="cc_app-tools-trussoptions"></a>

### struct `TrussOptions`

```rust
pub struct TrussOptions
```

Truss tool options, kept between uses.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `shape: cc_core::truss::TrussShape`
- `pattern: cc_core::truss::TrussPattern`
- `height: f64`

  Chord to chord (flat), or the rise (mono, gable), meters.

- `panels: u32`

  Number of panels; 0 picks about one per 1.5 m.

- `copies: u32`
- `spacing: f64`

  Copy to copy, meters.


<a id="cc_app-tools-truss_command"></a>

### fn `truss_command`

```rust
pub fn truss_command(o: &TrussOptions, material: cc_core::StructuralMaterial, level: LevelId, a: DVec2, b: DVec2) -> cc_engine::commands::framing::TrussCreate
```

The command the tool runs for a truss from `a` to `b`.

<a id="cc_app-tools-bracingoptions"></a>

### struct `BracingOptions`

```rust
pub struct BracingOptions
```

Bracing tool options, kept between uses.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `pattern: cc_core::bracing::BracingPattern`
- `height: f64`

  One storey, meters.

- `stories: u32`
- `flip: bool`

  A diagonal rises toward the first click instead of the second.

- `alternate: bool`

  Diagonals change direction storey by storey.


<a id="cc_app-tools-bracing_command"></a>

### fn `bracing_command`

```rust
pub fn bracing_command(o: &BracingOptions, material: cc_core::StructuralMaterial, level: LevelId, a: DVec2, b: DVec2) -> cc_engine::commands::bracing::BracingCreate
```

The command the tool runs for a braced bay from `a` to `b`.

<a id="cc_app-tools-foundation_type"></a>

### fn `foundation_type`

```rust
pub fn foundation_type(project: &Project, kind: cc_core::FoundationKind) -> Option<ElementTypeId>
```

The first foundation type of `kind`.

<a id="cc_app-tools-is_strip"></a>

### fn `is_strip`

```rust
pub fn is_strip(project: &Project, type_id: ElementTypeId) -> bool
```

<a id="cc_app-tools-grid_crossings"></a>

### fn `grid_crossings`

```rust
pub(crate) fn grid_crossings(project: &Project) -> Vec<DVec2>
```

Intersections of grid lines, offered as snap points.

<a id="cc_app-tools-beam_command"></a>

### fn `beam_command`

```rust
pub fn beam_command(level: LevelId, type_id: ElementTypeId, a: DVec2, b: DVec2, bulge: f64) -> create::BeamCreate
```

Two clicks: start and end of a beam. Its top is at the level above by default.
The command the beam tool runs for a beam from `a` to `b` (an arc with a `bulge`).

<a id="cc_app-tools-railinghostoptions"></a>

### struct `RailingHostOptions`

```rust
pub struct RailingHostOptions
```

Options of the railing tool: put railings on a stair or ramp picked in
the view instead of drawing a path.

**Traits:** `Debug`, `Clone`, `Copy`, `Default`, `PartialEq`

**Fields**

- `on: bool`
- `both: bool`

  Both sides of the host, not only the side nearest the click.


<a id="cc_app-tools-railing_host"></a>

### fn `railing_host`

```rust
pub fn railing_host(state: &AppState, id: ElementId) -> bool
```

Whether `id` is a stair or a ramp (an element railings can follow).

<a id="cc_app-tools-sweepoptions"></a>

### struct `SweepOptions`

```rust
pub struct SweepOptions
```

Options of the swept profile tool.

**Traits:** `Debug`, `Clone`, `Copy`, `Default`, `PartialEq`

**Fields**

- `base_offset: f64`

  Height of the path above the level.

- `closed: bool`

  The path runs back to its first point.

- `flip: bool`

  The profile on the right of the path.


<a id="cc_app-tools-finish_sweep"></a>

### fn `finish_sweep`

```rust
pub fn finish_sweep(state: &mut AppState, level: LevelId, type_id: ElementTypeId)
```

Create the swept profile from the points clicked so far.

<a id="cc_app-tools-finish_railing"></a>

### fn `finish_railing`

```rust
pub fn finish_railing(state: &mut AppState, level: LevelId, type_id: ElementTypeId)
```

Create the railing from the points clicked so far.

<a id="cc_app-tools-linear_direction"></a>

### fn `linear_direction`

```rust
pub(crate) fn linear_direction(a: DVec2, b: DVec2, cursor: DVec2) -> DVec2
```

Horizontal or vertical measuring direction for a linear dimension from
`a` to `b`, by where the cursor places the line: above or below the
points' box measures horizontally, beside it vertically.

<a id="cc_app-tools-line_at"></a>

### fn `line_at`

```rust
pub(crate) fn line_at(drawing: &Drawing2D, p: DVec2, tol: f64) -> Option<(DVec2, DVec2)>
```

The linework segment (of an element) nearest to `p` within `tol`.

<a id="cc_app-tools-point_on_segment"></a>

### fn `point_on_segment`

```rust
pub(crate) fn point_on_segment(a: DVec2, b: DVec2, p: DVec2, tol: f64) -> bool
```

Whether `p` lies on the segment `a`-`b` within `tol`.

<a id="cc_app-tools-arc_at"></a>

### fn `arc_at`

```rust
pub(crate) fn arc_at(drawing: &Drawing2D, p: DVec2, tol: f64) -> Option<(DVec2, f64)>
```

An arc or circle drawn as a chain of short segments near `p`: its
center and radius. Needs four consecutive points of one element on a
common circle (corners of straight linework do not qualify).

<a id="cc_app-tools-arc_preview"></a>

### fn `arc_preview`

```rust
pub(crate) fn arc_preview(painter: &Painter, xf: &Xf, a: DVec2, b: DVec2, bulge: f64, center: f64, half: f64)
```

Preview of an arc wall drawn from `a` to `b` with `bulge`: its location
line and faces, as in `paint_wall_preview`.

<a id="cc_app-tools-place_opening_3d"></a>

### fn `place_opening_3d`

```rust
pub(crate) fn place_opening_3d(state: &mut AppState, tool: &Tool, host: ElementId, point: DVec2)
```

A door or window tool clicked on `point` of wall `host` in a 3D view: the
opening is centered there (at the type's sill), facing the side clicked.

<a id="cc_app-tools-add_opening"></a>

### fn `add_opening`

```rust
pub fn add_opening(state: &mut AppState, host: ElementId, pts: Vec<DVec2>)
```

Add the sketched outline `pts` as an opening of `roof` (refused, with a
message, when it is not inside the roof or overlaps another opening).
Add the sketched opening `pts` to the roof or floor `host`.

<a id="cc_app-tools-move_elements"></a>

### fn `move_elements`

```rust
pub fn move_elements(state: &mut AppState, ids: &[ElementId], delta: DVec2)
```

Translate elements; hosted elements slide along their host unless it moves too.

<a id="cc_app-tools-copy_elements"></a>

### fn `copy_elements`

```rust
pub fn copy_elements(state: &mut AppState, ids: &[ElementId], delta: DVec2, count: u32) -> Vec<ElementId>
```

Copy elements by `delta`, `count` times (an array); the copies are selected and returned.

<a id="cc_app-tools-duplicate"></a>

### fn `duplicate`

```rust
pub fn duplicate(state: &mut AppState, ids: &[ElementId], delta: DVec2)
```

Copy elements by `delta`; hosted elements follow copied hosts.

<a id="cc_app-tools-rotate_elements"></a>

### fn `rotate_elements`

```rust
pub fn rotate_elements(state: &mut AppState, ids: &[ElementId], center: DVec2, angle: f64, copy: bool) -> Vec<ElementId>
```

Rotate (or copy rotated) by `angle` radians about `center`; returns the
elements rotated or created (copies are selected).

<a id="cc_app-tools-mirror_elements"></a>

### fn `mirror_elements`

```rust
pub fn mirror_elements(state: &mut AppState, ids: &[ElementId], a: DVec2, b: DVec2, copy: bool) -> Vec<ElementId>
```

Mirror (or copy mirrored) across the line through `a` and `b`; returns the
elements mirrored or created (copies are selected).

<a id="cc_app-tools-rotate_selection"></a>

### fn `rotate_selection`

```rust
pub fn rotate_selection(state: &mut AppState) -> bool
```

Space on a selection: components and columns turn 90° about their
insertion point, doors and windows flip to the other side of the wall.
Returns whether anything changed (one undo step).

<a id="mod-cc_app-truss_ui"></a>

## `cc_app::truss_ui`

<sub>`crates/cc_app/src/truss_ui.rs`</sub>

Trusses in the app: the Truss tool's options bar and the properties of a
selected truss (every parameter through `element_set`, which regenerates
its members in the same undo step).

**Items:** [`patterns_for`](#cc_app-truss_ui-patterns_for) · [`tool_options`](#cc_app-truss_ui-tool_options) · [`edit`](#cc_app-truss_ui-edit) · [`props`](#cc_app-truss_ui-props)

<a id="cc_app-truss_ui-patterns_for"></a>

### fn `patterns_for`

```rust
pub fn patterns_for(shape: TrussShape) -> Vec<TrussPattern>
```

The patterns offered for a shape (gable-only ones on gables).

<a id="cc_app-truss_ui-tool_options"></a>

### fn `tool_options`

```rust
pub fn tool_options(ui: &mut Ui, state: &mut AppState)
```

The options bar of the Truss tool.

<a id="cc_app-truss_ui-edit"></a>

### fn `edit`

```rust
pub fn edit(state: &mut AppState, id: ElementId, field: &str, f: impl FnOnce(&mut Truss))
```

Change truss `id` as `f` changes a copy of it (`element_set`, which
regenerates its members); edits under one `field` merge while dragging.

<a id="cc_app-truss_ui-props"></a>

### fn `props`

```rust
pub fn props(ui: &mut Ui, state: &mut AppState, id: ElementId, truss: &Truss)
```

The rows of a selected truss.

<a id="mod-cc_app-type_dialog"></a>

## `cc_app::type_dialog`

<sub>`crates/cc_app/src/type_dialog.rs`</sub>

Type Properties dialog: switch, duplicate and rename types, edit the
compound structure (layers) and the type parameters in sections.

**Items:** [`show`](#cc_app-type_dialog-show)

<a id="cc_app-type_dialog-show"></a>

### fn `show`

```rust
pub fn show(ctx: &Context, state: &mut AppState)
```

<a id="mod-cc_app-units_dialog"></a>

## `cc_app::units_dialog`

<sub>`crates/cc_app/src/units_dialog.rs`</sub>

Project Units dialog: unit, decimals and symbol of each quantity, the
currency and the number style, with a live sample of each format.

**Items:** [`show`](#cc_app-units_dialog-show)

<a id="cc_app-units_dialog-show"></a>

### fn `show`

```rust
pub fn show(ctx: &Context, state: &mut AppState)
```

<a id="mod-cc_app-update"></a>

## `cc_app::update`

<sub>`crates/cc_app/src/update.rs`</sub>

Update notice: at startup (at most once a day) and on request, the app
asks GitHub for the latest release of ConstructCraft in a background thread
and, when it is newer than this build, shows a notice with a link to the
release page. Checks only notify; *Update Now* (see `crate::update_install`)
downloads, verifies and installs a release, and only when the user asks.
The one request is an HTTPS GET of `RELEASES_API` with a short timeout
and a bounded response; every failure is silent (logged) and reported as
"could not check" only when the user asked. Automatic checks can be turned
off in the About dialog, and the environment variable `NO_CHECK_ENV`
turns every check off (CI and tests). Version comparison and the response
parsing are pure functions, tested without a network.

**Items:** [`RELEASES_API`](#cc_app-update-releases_api) · [`NO_CHECK_ENV`](#cc_app-update-no_check_env) · [`CHECK_INTERVAL`](#cc_app-update-check_interval) · [`Version`](#cc_app-update-version) · [`Release`](#cc_app-update-release) · [`Outcome`](#cc_app-update-outcome) · [`current`](#cc_app-update-current) · [`evaluate`](#cc_app-update-evaluate) · [`disabled_by_env`](#cc_app-update-disabled_by_env) · [`auto_check_due`](#cc_app-update-auto_check_due) · [`now`](#cc_app-update-now) · [`Updates`](#cc_app-update-updates) · [`Install`](#cc_app-update-install) · [`Plan`](#cc_app-update-plan) · [`plan`](#cc_app-update-plan) · [`notice`](#cc_app-update-notice)

<a id="cc_app-update-releases_api"></a>

### const `RELEASES_API`

```rust
pub const RELEASES_API: &str
```

The GitHub API endpoint of the latest (non-draft, non-prerelease) release.

<a id="cc_app-update-no_check_env"></a>

### const `NO_CHECK_ENV`

```rust
pub const NO_CHECK_ENV: &str
```

Set (to anything but `0` or nothing) to turn every update check off.

<a id="cc_app-update-check_interval"></a>

### const `CHECK_INTERVAL`

```rust
pub const CHECK_INTERVAL: Duration
```

Time between automatic checks.

<a id="cc_app-update-version"></a>

### struct `Version`

```rust
pub struct Version
```

A release version `major.minor.patch` (build metadata ignored).

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, `std::fmt::Display`

**Fields**

- `major: u64`
- `minor: u64`
- `patch: u64`

**Methods**

- `pub fn parse(text: &str) -> Option<Version>`

  Parse `1.2.3` or `v1.2.3` (build metadata `+...` ignored). Pre-releases
  (`1.2.3-beta`) and anything else give `None`.


<a id="cc_app-update-release"></a>

### struct `Release`

```rust
pub struct Release
```

A published release.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Eq`

**Fields**

- `version: Version`
- `url: String`

  Its page on GitHub (built from the tag, never taken from the response).

- `assets: Vec<crate::update_install::Asset>`

  Its packages and checksum list (only links to this repository's releases).


<a id="cc_app-update-outcome"></a>

### enum `Outcome`

```rust
pub enum Outcome
```

The result of a check.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Eq`

**Variants**

- `UpToDate`
- `Available(Release)`
- `Failed`

  No answer, or one that could not be read.

- `Disabled`

  Checks are turned off by `NO_CHECK_ENV`.


<a id="cc_app-update-current"></a>

### fn `current`

```rust
pub fn current() -> Option<Version>
```

The version of this build.

<a id="cc_app-update-evaluate"></a>

### fn `evaluate`

```rust
pub fn evaluate(current: Version, json: &str) -> Outcome
```

Compare the latest-release JSON of the GitHub API with `current`.
Drafts and pre-releases are ignored (reported as up to date).

<a id="cc_app-update-disabled_by_env"></a>

### fn `disabled_by_env`

```rust
pub fn disabled_by_env() -> bool
```

Whether `NO_CHECK_ENV` turns checks off.

<a id="cc_app-update-auto_check_due"></a>

### fn `auto_check_due`

```rust
pub fn auto_check_due(enabled: bool, last_check: Option<u64>, now: u64) -> bool
```

Whether an automatic check is due: turned on, and none in the last
`CHECK_INTERVAL` (or the clock went back). Times are seconds since the Unix epoch.

<a id="cc_app-update-now"></a>

### fn `now`

```rust
pub fn now() -> u64
```

Seconds since the Unix epoch (0 if the clock is before it).

<a id="cc_app-update-updates"></a>

### struct `Updates`

```rust
pub struct Updates
```

Update checks of the app: when the last ran, the check in progress and what to show.

**Traits:** `Debug`, `Default`

**Fields**

- `last_check: Option<u64>`

  When the last check started (seconds since the Unix epoch).

- `skipped: Option<Version>`

  A version the user chose to skip: its notice is not shown again.

- `available: Option<Release>`

  A newer release whose notice is shown.

- `report: Option<Outcome>`

  The result of a check the user asked for, shown in a dialog.

- `install: Option<Install>`

  Update Now: its progress or result, shown in a dialog.

- `on_exit: Option<crate::update_install::Next>`

  What an installed update starts when the app closes (the new version, or the installer).


*Some fields are private.*

**Methods**

- `pub fn start_if_due(&mut self, ctx: &egui::Context, enabled: bool)`

  Start an automatic check if one is due (`enabled`: the preference is on).

- `pub fn start(&mut self, ctx: &egui::Context, manual: bool)`

  Start a check (`manual`: the user asked, so the result is reported).

- `pub fn checking(&self) -> bool`

  A check the user asked for is running.

- `pub fn cancel_report(&mut self)`

  Stop reporting the check in progress (it still finishes quietly).

- `pub fn poll(&mut self)`

  Take the result of the check in progress, if it arrived.

- `pub fn update_now(&mut self, ctx: &egui::Context, release: &Release)`

  Update Now: install `release` here (or show how), in the update dialog.

- `pub fn poll_install(&mut self)`

  Take the result of Update Now, if it arrived.

- `pub fn dismiss(&mut self)`

  Hide the notice until the next check.

- `pub fn skip(&mut self)`

  Hide the notice and do not notify this version again.


<a id="cc_app-update-install"></a>

### enum `Install`

```rust
pub enum Install
```

Update Now, as the dialog shows it.

**Traits:** `Debug`

**Variants**

- `Running(Version, String, crate::update_install::Running)`

  Downloading and verifying `version` (its release page for a manual download).

- `Ready(Version, crate::update_install::Next)`

  Done: what is left to do.

- `Command(String, String)`

  The package manager updates this copy: the command, and the release page.

- `Failed(String, String)`

  Failed (why, and the release page for a manual download).


<a id="cc_app-update-plan"></a>

### enum `Plan`

```rust
pub enum Plan
```

How Update Now would update to `release` here, if it can.

**Variants**

- `Install(crate::update_install::InstallKind, crate::update_install::Asset, String)`

  Download, verify and install `package` (with the release's checksum list).

- `Command(String)`

  Show the package manager's command.


<a id="cc_app-update-plan"></a>

### fn `plan`

```rust
pub fn plan(release: &Release) -> Option<Plan>
```

What Update Now does for `release` on this installation, or `None` (Download only).

<a id="cc_app-update-notice"></a>

### fn `notice`

```rust
pub fn notice(ui: &mut egui::Ui, updates: &mut Updates)
```

The notice of a newer release, in one row: message, Download, Skip, dismiss.
Shows nothing without one.

<a id="mod-cc_app-update_install"></a>

## `cc_app::update_install`

<sub>`crates/cc_app/src/update_install.rs`</sub>

One-click updates: download the package of a newer release that fits
how this copy was installed, verify it, and put it in place.
Verification is never skipped: the package's SHA-256 must match the
release's `SHA256SUMS.txt`, both fetched over HTTPS from the release's
own download links on GitHub. A package that does not match is deleted
and nothing is replaced. What "put in place" means:

- AppImage: written next to the running AppImage, then renamed over it
  (one file system, so the swap is atomic; the running copy keeps its
  file), and the app offers to restart.
- Windows installer (`.msi`, under Program Files): run `msiexec /passive`
  when the app closes; ConstructCraft starts again when it is done.
- Windows `.zip` (anywhere else on Windows, `ccad.exe` beside the app):
  the archive is unpacked in a staging folder, the running executables
  are renamed to `.old` (Windows allows renaming a running program) and
  the new ones take their names; the `.old` files go at the next start.
- macOS app: the `.dmg` is opened; the user drags the app to Applications.
- `.deb`: never replaced; the window shows the `apt` command.
- Anything else (`.tar.gz`, builds from source): Download only.

**Items:** [`DOWNLOAD_PREFIX`](#cc_app-update_install-download_prefix) · [`SUMS`](#cc_app-update_install-sums) · [`Asset`](#cc_app-update_install-asset) · [`assets_of`](#cc_app-update_install-assets_of) · [`InstallKind`](#cc_app-update_install-installkind) · [`detect`](#cc_app-update_install-detect) · [`this_install`](#cc_app-update_install-this_install) · [`package_for`](#cc_app-update_install-package_for) · [`installs_itself`](#cc_app-update_install-installs_itself) · [`deb_command`](#cc_app-update_install-deb_command) · [`expected_sha256`](#cc_app-update_install-expected_sha256) · [`Progress`](#cc_app-update_install-progress) · [`Next`](#cc_app-update_install-next) · [`install`](#cc_app-update_install-install) · [`swap_executable`](#cc_app-update_install-swap_executable) · [`remove_old_executables`](#cc_app-update_install-remove_old_executables) · [`installer_command`](#cc_app-update_install-installer_command) · [`finish`](#cc_app-update_install-finish) · [`Running`](#cc_app-update_install-running)

<a id="cc_app-update_install-download_prefix"></a>

### const `DOWNLOAD_PREFIX`

```rust
pub const DOWNLOAD_PREFIX: &str
```

Packages and the checksum list must come from this repository's releases.

<a id="cc_app-update_install-sums"></a>

### const `SUMS`

```rust
pub const SUMS: &str
```

The checksum list every release carries.

<a id="cc_app-update_install-asset"></a>

### struct `Asset`

```rust
pub struct Asset
```

A file attached to a release.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Eq`

**Fields**

- `name: String`
- `url: String`

<a id="cc_app-update_install-assets_of"></a>

### fn `assets_of`

```rust
pub fn assets_of(release: &serde_json::Value) -> Vec<Asset>
```

The assets of a release response that can be downloaded safely: plain
file names, links under `DOWNLOAD_PREFIX`.

<a id="cc_app-update_install-installkind"></a>

### enum `InstallKind`

```rust
pub enum InstallKind
```

How this copy of ConstructCraft was installed.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Eq`

**Variants**

- `AppImage(PathBuf)`

  An AppImage (the file).

- `Msi`

  The Windows installer, under Program Files.

- `Portable(PathBuf)`

  The Windows `.zip`: this executable (with `ccad.exe` beside it).

- `MacApp`

  A macOS `.app` (from the `.dmg`).

- `Deb`

  The Debian package, under `/usr`.

- `Other`

  Anything else: Download only.


<a id="cc_app-update_install-detect"></a>

### fn `detect`

```rust
pub fn detect(os: &str, exe: Option<&Path>, env: impl Fn(&str) -> Option<String>) -> InstallKind
```

Tell how a copy was installed from the OS (`std::env::consts::OS`), its
executable and the environment (`env`).

<a id="cc_app-update_install-this_install"></a>

### fn `this_install`

```rust
pub fn this_install() -> InstallKind
```

This copy's installation.

<a id="cc_app-update_install-package_for"></a>

### fn `package_for`

```rust
pub fn package_for<'a>(kind: &InstallKind, arch: &str, assets: &'a [Asset]) -> Option<&'a Asset>
```

The release package for `kind` on `arch` (`std::env::consts::ARCH`), by the
end of its name (release names carry the version).

<a id="cc_app-update_install-installs_itself"></a>

### fn `installs_itself`

```rust
pub fn installs_itself(kind: &InstallKind) -> bool
```

Whether Update Now can install for `kind` (the others get Download, or a command).

<a id="cc_app-update_install-deb_command"></a>

### fn `deb_command`

```rust
pub fn deb_command(asset: &Asset) -> String
```

The command that installs a downloaded `.deb`.

<a id="cc_app-update_install-expected_sha256"></a>

### fn `expected_sha256`

```rust
pub fn expected_sha256(sums: &str, name: &str) -> Option<String>
```

The lowercase hex SHA-256 that `SHA256SUMS.txt` (`<hex>  <name>` or
`<hex> *<name>` lines) gives for `name`.

<a id="cc_app-update_install-progress"></a>

### struct `Progress`

```rust
pub struct Progress
```

Progress of an update, shared with the window that shows it.

**Traits:** `Debug`, `Default`

**Fields**

- `done: AtomicU64`
- `total: AtomicU64`

  The package's size when the server gives it (0: unknown).

- `cancel: AtomicBool`

<a id="cc_app-update_install-next"></a>

### enum `Next`

```rust
pub enum Next
```

What to do once the package is in place.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Eq`

**Variants**

- `Restart(PathBuf)`

  The new version replaced this one: start this program when the app closes.

- `Installer(PathBuf, String)`

  Run this installer (`.msi`) when the app closes, then start the app. The SHA-256 it was
  verified with at download is checked again just before it runs: it waits in the temporary
  folder, where another program could replace it.

- `Opened`

  The disk image was opened: the user finishes in the Finder.


<a id="cc_app-update_install-install"></a>

### fn `install`

```rust
pub fn install(kind: &InstallKind, package: &Asset, sums_url: &str, progress: &Progress) -> Result<Next, String>
```

Download, verify and put in place `package` of the release whose checksum
list is `sums_url`, for this installation. Runs on a background thread.

<a id="cc_app-update_install-swap_executable"></a>

### fn `swap_executable`

```rust
pub fn swap_executable(current: &Path, new: &Path) -> Result<(), String>
```

Replace the (possibly running) executable `current` with `new`: `current`
is renamed to `<name>.old` first, and put back if the new one can't take
its place.

<a id="cc_app-update_install-remove_old_executables"></a>

### fn `remove_old_executables`

```rust
pub fn remove_old_executables(exe: &Path)
```

Remove the `.old` executables a previous update left beside `exe`.

<a id="cc_app-update_install-installer_command"></a>

### fn `installer_command`

```rust
pub fn installer_command(msi: &Path, exe: &Path) -> (String, Vec<String>)
```

The command that runs the installer `msi`, waits for it and then starts
`exe` (the installed program). If the user cancels the installer, the old
version is still there to start.

<a id="cc_app-update_install-finish"></a>

### fn `finish`

```rust
pub fn finish(next: &Next)
```

Start what an update left to do, as the app closes.

<a id="cc_app-update_install-running"></a>

### struct `Running`

```rust
pub struct Running
```

A running update: its progress, and the result when it is done.

**Traits:** `std::fmt::Debug`

**Fields**

- `progress: Arc<Progress>`

*Some fields are private.*

**Methods**

- `pub fn start(ctx: &egui::Context, kind: InstallKind, package: Asset, sums_url: String) -> Result<Running, String>`

  Start installing `package` on a background thread.

- `pub fn poll(&self) -> Option<Result<Next, String>>`

  The result, once the thread is done.


<a id="mod-cc_app-view3d"></a>

## `cc_app::view3d`

<sub>`crates/cc_app/src/view3d.rs`</sub>

Interactive 3D view backed by `cc_render::Renderer3D`.

**Items:** [`View3DState`](#cc_app-view3d-view3dstate) · [`fit_camera`](#cc_app-view3d-fit_camera) · [`show`](#cc_app-view3d-show)

<a id="cc_app-view3d-view3dstate"></a>

### struct `View3DState`

```rust
pub struct View3DState
```

**Fields**

- `camera: CameraState`
- `texture: Option<egui::TextureId>`
- `picks: crate::modify3d::PickCache`

  Snap points and previews of the modify tools.

- `pixel_size: Option<[u32; 2]>`

  Size of the viewport in pixels when last drawn (the render window's *View size*).


*Some fields are private.*

**Methods**

- `pub(crate) fn auto_fitted(&self) -> bool`

  Whether the camera is still the one fitted to the model (never moved
  since: a resized view fits it again).

- `pub fn request_fit(&mut self)`

  Frame the model on the next frame, remembering the camera.

- `pub fn nav(&self) -> ZoomNav`
- `pub fn apply(&mut self, cmd: ZoomCommand)`

  Run a zoom command of the view toolbar ("fit all" is handled by the caller).


<a id="cc_app-view3d-fit_camera"></a>

### fn `fit_camera`

```rust
pub fn fit_camera(camera: &mut CameraState, geom: &cc_geom::ModelGeometry, aspect: f32)
```

Zoom the camera to frame the whole model.

<a id="cc_app-view3d-show"></a>

### fn `show`

```rust
pub fn show(ui: &mut Ui, state: &mut AppState, view_id: ViewId, render_state: Option<&egui_wgpu::RenderState>)
```

<a id="mod-cc_app-view_cube"></a>

## `cc_app::view_cube`

<sub>`crates/cc_app/src/view_cube.rs`</sub>

Navigation cube for 3D views: a cube aligned with the world axes and a
compass ring, drawn in the view's corner.
Each cube face is split into a 3 x 3 grid: the center cell looks at the
face, edge cells look along the edge and corner cells give the
isometric views (26 directions). Dragging the cube orbits, dragging the
ring turns around the vertical axis, and the home button returns to the
default view.

**Items:** [`KEYS`](#cc_app-view_cube-keys) · [`CubeAction`](#cc_app-view_cube-cubeaction) · [`angles_from`](#cc_app-view_cube-angles_from) · [`show`](#cc_app-view_cube-show)

<a id="cc_app-view_cube-keys"></a>

### const `KEYS`

```rust
pub const KEYS: [&str; 10]
```

Translation keys used at runtime (checked by the i18n tests).

<a id="cc_app-view_cube-cubeaction"></a>

### struct `CubeAction`

```rust
pub struct CubeAction
```

What the user asked of the cube this frame.

**Traits:** `Debug`, `Default`

**Fields**

- `look_from: Option<DVec3>`

  View from this direction (unit vector from the target to the eye).

- `orbit: Option<Vec2>`

  Orbit by a drag (as on the canvas).

- `turn: f64`

  Turn around Z by this many degrees.

- `home: bool`

<a id="cc_app-view_cube-angles_from"></a>

### fn `angles_from`

```rust
pub fn angles_from(dir: DVec3, yaw: f64) -> (f64, f64)
```

Yaw and pitch that look at the target from `dir` (keeping `yaw` when looking straight down or up).

<a id="cc_app-view_cube-show"></a>

### fn `show`

```rust
pub fn show(ui: &mut Ui, view: Rect, cam: &CameraState, north: glam::DVec2) -> CubeAction
```

Show the cube in the top-right corner of `view`, its compass turned to
`north` (true north in plan, a unit vector; project +Y without a site).

<a id="mod-cc_app-wall_framing_ui"></a>

## `cc_app::wall_framing_ui`

<sub>`crates/cc_app/src/wall_framing_ui.rs`</sub>

*Structure → Light Steel Frame / Wood Frame*: the dialog that runs
`wall_framing_create` on the selected walls (or the active level's), the
properties of a selected framing (through `element_set`, which
regenerates its members in the same undo step), and the *Framing* row
of its members.

**Items:** [`WallFramingDialog`](#cc_app-wall_framing_ui-wallframingdialog) · [`open`](#cc_app-wall_framing_ui-open) · [`walls_to_frame`](#cc_app-wall_framing_ui-walls_to_frame) · [`command`](#cc_app-wall_framing_ui-command) · [`show`](#cc_app-wall_framing_ui-show) · [`edit`](#cc_app-wall_framing_ui-edit) · [`props`](#cc_app-wall_framing_ui-props) · [`member_row`](#cc_app-wall_framing_ui-member_row)

<a id="cc_app-wall_framing_ui-wallframingdialog"></a>

### struct `WallFramingDialog`

```rust
pub struct WallFramingDialog
```

**Traits:** `Debug`, `Clone`

**Fields**

- `system: FramingSystem`
- `spacing: f64`
- `stud_type: Option<ElementTypeId>`

  `None`: the catalog default.

- `track_type: Option<ElementTypeId>`
- `header_type: Option<ElementTypeId>`
- `double_top: bool`
- `blocking: bool`
- `error: Option<String>`

**Methods**

- `pub fn new(system: FramingSystem) -> Self`

  The system's defaults: spacing, top plates, catalog types, no blocking.


<a id="cc_app-wall_framing_ui-open"></a>

### fn `open`

```rust
pub fn open(state: &mut AppState, system: FramingSystem)
```

<a id="cc_app-wall_framing_ui-walls_to_frame"></a>

### fn `walls_to_frame`

```rust
pub fn walls_to_frame(state: &AppState) -> Vec<ElementId>
```

The walls OK frames: the selected walls, or every wall of the active
level when nothing is selected (a selection without walls frames
nothing); only straight walls not framed yet.

<a id="cc_app-wall_framing_ui-command"></a>

### fn `command`

```rust
pub fn command(d: &WallFramingDialog, walls: &[ElementId]) -> WallFramingCreate
```

The command OK runs for `walls`.

<a id="cc_app-wall_framing_ui-show"></a>

### fn `show`

```rust
pub fn show(ctx: &Context, state: &mut AppState)
```

<a id="cc_app-wall_framing_ui-edit"></a>

### fn `edit`

```rust
pub fn edit(state: &mut AppState, id: ElementId, field: &str, f: impl FnOnce(&mut WallFraming))
```

Change framing `id` as `f` changes a copy of it (`element_set`, which
regenerates its members); edits under one `field` merge while dragging.

<a id="cc_app-wall_framing_ui-props"></a>

### fn `props`

```rust
pub fn props(ui: &mut Ui, state: &mut AppState, id: ElementId, framing: &WallFraming)
```

The rows of a selected framing.

<a id="cc_app-wall_framing_ui-member_row"></a>

### fn `member_row`

```rust
pub fn member_row(ui: &mut Ui, state: &mut AppState, member: ElementId)
```

The *Framing* row of a member of a wall framing: its button selects the framing.

{% endraw %}
