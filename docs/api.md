# API reference: overview

[Português](api.pt-BR.md) · [Full reference by crate](api/index.md)

This page explains how ConstructCraft's code is organized and where its main types live. The [full reference](api/index.md) lists, crate by crate and module by module, every item that is not private: functions, structs, enums, traits, type aliases, constants and re-exports, with their signatures and documentation, the fields of structs, the variants of enums, the methods of each type and the traits each type implements. It is generated from the source code by `cargo run -p api_doc` (`tools/api_doc`), so it always says what the doc comments say; to change it, edit the comments and run the generator again.

If you want to *use* ConstructCraft from outside (scripts, the command line, AI agents), the [command reference](commands.md) and the [automation guide](automation.md) are what you need: commands are the stable interface. This reference is for people working on the code itself.

## Classes and inheritance in Rust

ConstructCraft is written in Rust, which has no classes and no inheritance. The same ideas are expressed with other building blocks, and the reference is organized around them:

| Object-oriented idea | In Rust | In this reference |
|---|---|---|
| Class with fields | `struct` | *struct*, with its **Fields** |
| Class hierarchy with a closed set of subclasses | `enum` whose variants carry data | *enum*, with its **Variants** |
| Methods | functions in `impl Type { … }` blocks, which may live in any module of the crate | **Methods** of the type; *(in `module`)* says where a method is defined when it is not the type's own module |
| Interface / abstract base class | `trait` | *trait*, with its **Required and provided items** and **Implemented by** |
| Inheriting an interface | a supertrait: `trait Command: DeserializeOwned + JsonSchema` | the trait's signature |
| Implementing an interface | `impl Trait for Type` | **Traits** of the type |
| Generated standard behaviour | `#[derive(Clone, Debug, Serialize, …)]` | also listed under **Traits** |
| Static members | associated functions (no `self`) and associated constants | under **Methods** |

Two patterns stand in for inheritance throughout the code:

- **Enums as closed hierarchies.** An element of the model is one `Element` struct (id, uid, level, type, phases, design option, properties) whose `kind: ElementKind` holds what kind of element it is: `ElementKind::Wall(Wall)`, `Floor(Floor)`, `Door(Hosted)`, `Pipe(MepSegment)`, `Truss(Truss)` and so on. Code that would call a virtual method matches on the variant. Types (`ElementType` with `TypeData`), views (`View` with `ViewKind`) and many smaller things follow the same pattern.
- **Traits as open interfaces.** Where new implementations are added often, the code uses a trait. The main ones:
  - [`Command`](api/cc_engine.md#cc_engine-registry-command) (`cc_engine`): every engine command is a struct of parameters implementing it (several hundred implementors).
  - [`Extension`](api/cc_engine.md#cc_engine-extension-extension) (`cc_engine`): adds commands to a session; implemented by the plugin host.
  - [`MemberHost`](api/cc_core.md#cc_core-member_host-memberhost) (`cc_core`): an element that generates structural members (steel buildings, trusses, bracing, wall framing).

## Crates

The code is a Cargo workspace of crates, each a library (or a program) with a single responsibility. Each crate uses only those listed before it in this order, so `cc_core` depends on no other crate and nothing depends on `cc_app`:

| Crate | Uses (workspace crates) |
|---|---|
| `cc_core` | — |
| `cc_skp` | — |
| `cc_script` | `cc_core` |
| `cc_cad` | `cc_core` |
| `cc_geom` | `cc_core`, `cc_script` |
| `cc_views`, `cc_ifc`, `cc_render` | `cc_core`, `cc_geom` |
| `cc_raytracing` | `cc_core`, `cc_geom`, `cc_views` |
| `cc_engine` | `cc_core`, `cc_cad`, `cc_geom`, `cc_ifc`, `cc_raytracing`, `cc_script`, `cc_skp`, `cc_views` |
| `cc_plugins` | `cc_engine`, `cc_script`, `cc_skp` |
| `cc_cli` | `cc_core`, `cc_engine`, `cc_plugins` |
| `cc_app` | `cc_core`, `cc_engine`, `cc_geom`, `cc_ifc`, `cc_plugins`, `cc_raytracing`, `cc_render`, `cc_script`, `cc_views` |

What each crate holds:

| Crate | What it holds | Main types |
|---|---|---|
| [`cc_core`](api/cc_core.md) | The BIM model, the single source of truth: project, levels, materials, types, elements, views, undo/redo, files. Knows nothing of geometry or UI. | [`Project`](api/cc_core.md#cc_core-project-project), [`Element`](api/cc_core.md#cc_core-element-element), [`ElementKind`](api/cc_core.md#cc_core-element-elementkind), [`ElementType`](api/cc_core.md#cc_core-types-elementtype), [`TypeData`](api/cc_core.md#cc_core-types-typedata), [`View`](api/cc_core.md#cc_core-view-view), [`ViewKind`](api/cc_core.md#cc_core-view-viewkind), [`Document`](api/cc_core.md#cc_core-document-document), [`EditCtx`](api/cc_core.md#cc_core-document-editctx), [`Transaction`](api/cc_core.md#cc_core-document-transaction), [`ComponentDef`](api/cc_core.md#cc_core-component-componentdef), [`Sketch`](api/cc_core.md#cc_core-sketch-sketch), [`ScheduleDef`](api/cc_core.md#cc_core-schedule-scheduledef) |
| [`cc_geom`](api/cc_geom.md) | Geometry derived from the model, rebuilt when it changes: wall joins, solids, roofs, rooms, structure, reinforcement, MEP networks, terrain. | [`ModelGeometry`](api/cc_geom.md#cc_geom-build-modelgeometry), [`WallGeom`](api/cc_geom.md#cc_geom-wall-wallgeom), [`Solid`](api/cc_geom.md#cc_geom-solid-solid) |
| [`cc_views`](api/cc_views.md) | Views as queries over the model: plans, elevations, sections, schedules, sheets, annotations; export to SVG, PDF, PNG and DXF. | [`generate`](api/cc_views.md#cc_views-generate), [`Drawing2D`](api/cc_views.md#cc_views-drawing-drawing2d), [`Resolver`](api/cc_views.md#cc_views-graphics-resolver) |
| [`cc_render`](api/cc_render.md) | The 3D view's GPU renderer (wgpu). | [`Renderer3D`](api/cc_render.md#cc_render-renderer-renderer3d) |
| [`cc_raytracing`](api/cc_raytracing.md) | Photorealistic rendering on the CPU (path tracing). | [`RenderScene`](api/cc_raytracing.md#cc_raytracing-scene-renderscene) |
| [`cc_ifc`](api/cc_ifc.md) | IFC export and import, with its own STEP reader and writer. | |
| [`cc_cad`](api/cc_cad.md) | Reading DXF, DWG and SVG drawings into the model's neutral drawing. | [`CadDrawing`](api/cc_core.md#cc_core-cad-caddrawing) (in `cc_core`) |
| [`cc_skp`](api/cc_skp.md) | Reading and writing SketchUp files, with no external dependencies. | [`SkpFile`](api/cc_skp.md#cc_skp-parser-skpfile) |
| [`cc_script`](api/cc_script.md) | The sandboxed Lua 5.4 used by scripted components and plugins. | |
| [`cc_engine`](api/cc_engine.md) | The engine: a session and the catalog of commands every client calls; visual programming graphs; localization. | [`Session`](api/cc_engine.md#cc_engine-session-session), [`Command`](api/cc_engine.md#cc_engine-registry-command), [`EngineError`](api/cc_engine.md#cc_engine-error-engineerror), [`Graph`](api/cc_engine.md#cc_engine-graph-graph), [`Evaluator`](api/cc_engine.md#cc_engine-graph-eval-evaluator) |
| [`cc_plugins`](api/cc_plugins.md) | Lua scripts and plugins over the engine. | [`PluginHost`](api/cc_plugins.md#cc_plugins-plugin-pluginhost) |
| [`cc_cli`](api/cc_cli.md) | The `ccad` program: commands, scripts and the MCP server, without a GUI. | |
| [`cc_app`](api/cc_app.md) | The desktop application (egui). | [`AppState`](api/cc_app.md#cc_app-state-appstate), [`UiCommand`](api/cc_app.md#cc_app-commands-uicommand), [`Tool`](api/cc_app.md#cc_app-tools-tool) |

## How an edit flows through the code

1. A client (the app, `ccad`, the MCP server, a Lua script or a graph) asks the `Session` to run a command by name with JSON parameters, or a typed command struct.
2. The command's `run` (the `Command` trait) resolves its references (levels, types and views by id or name) and edits the model through `commands::edit`, which calls `Document::try_edit`.
3. Inside the edit, an `EditCtx` changes the `Project`; when it finishes, `EditCtx::finish` keeps derived data in step (dimensions, attachments, piles, shafts, member hosts, phases, design options, view templates…) and the result passes the repair check. A command that would leave the model invalid is refused without an undo step; otherwise the change is recorded as one reversible `Transaction`.
4. The document's revision changes, so `ModelGeometry::build` (`cc_geom`) rebuilds the geometry and `cc_views::generate` redraws the views that are shown, each as a `Drawing2D`; the 3D view goes to `Renderer3D`.

## Conventions you will meet in the reference

- Lengths are meters, angles degrees, points `[x, y]` in plan, unless a doc comment says otherwise (sizes on paper are millimeters).
- Errors are values: functions return `Result` or `Option` instead of panicking; the engine's errors are `EngineError { code, message }`.
- Items marked `pub(crate)` are internal to their crate; they are listed because the reference is for people working on the code.
- Test code (`#[cfg(test)]`) is left out.
