# `cc_plugins` API

{% raw %}
[← API reference](index.md) · Generated from the source by `cargo run -p api_doc`; edit the doc comments, not this page.

Lua scripts and plugins for ConstructCraft.
Lua 5.4 runs sandboxed (`sandbox`, from `cc_script`): no files, no loading of code, an
instruction budget and a memory limit. Lua code reaches the project only
through the engine's commands (`ccad.run`, see `api`), the same ones the
GUI, the command line and the MCP server use.

- `run_script` runs a script once (`ccad lua file.lua`).
- `plugin` loads plugins (a folder with `plugin.toml` and `main.lua`)
  whose commands join the session's catalog through `PluginHost`, an
  engine `cc_engine::Extension`: each plugin command is one undo step
  and leaves nothing behind when it fails.

## Modules

| Module | Items | Summary |
|---|---|---|
| [`cc_plugins`](#mod-cc_plugins) | 0 | Lua scripts and plugins for ConstructCraft. |
| [`cc_plugins::api`](#mod-cc_plugins-api) | 5 | The `ccad` table scripts and plugins see. |
| [`cc_plugins::plugin`](#mod-cc_plugins-plugin) | 7 | Plugins: a folder with a `plugin.toml` manifest and Lua code that adds commands to the session. |
| [`cc_plugins::script`](#mod-cc_plugins-script) | 4 | Lua scripts: a file of Lua code run once over a session (`ccad lua`). |
| [`cc_plugins::store`](#mod-cc_plugins-store) | 13 | Installing plugins: the community index (`index.json` of [constructcraft-plugins](https://github.com/francisreboucas/constructcraft-plugins)), the version to offer, and checked installs from a downloaded archive. |

<a id="mod-cc_plugins"></a>

## `cc_plugins`

<sub>`crates/cc_plugins/src/lib.rs`</sub>

**Re-exports**

- `pub use cc_script::{convert, sandbox}`
- `pub use api::{API_VERSION, Access}`
- `pub use plugin::{Manifest, Permissions, Plugin, PluginError, PluginHost}`
- `pub use sandbox::Limits`
- `pub use script::{ScriptOptions, ScriptOutput, run_script}`

<a id="mod-cc_plugins-api"></a>

## `cc_plugins::api`

<sub>`crates/cc_plugins/src/api.rs`</sub>

The `ccad` table scripts and plugins see.

- `ccad.run(name, args)`: run an engine command; returns its result, or
  raises an error.
- `ccad.try(name, args)`: the same, returning `result` or `nil, err`
  (`err.code`, `err.message`) instead of raising.
- `ccad.commands()`: the commands (name, kind, group, description);
  `ccad.describe(name)` one of them with its parameters' `schema`.
- `ccad.null` (JSON null), `ccad.array(t)` (marks `t`, even empty, as a
  list), `ccad.json.encode(v)` / `ccad.json.decode(s)`, `ccad.version`,
  `ccad.api` (the plugin API version).
The functions that reach the session exist only while a script or a
plugin command runs: the Lua wrappers stay (a plugin may keep them in
locals when it loads) and call the session's functions through a private
table filled for the run.

**Items:** [`API_VERSION`](#cc_plugins-api-api_version) · [`Access`](#cc_plugins-api-access) · [`Api`](#cc_plugins-api-api) · [`code_name`](#cc_plugins-api-code_name) · [`engine_error`](#cc_plugins-api-engine_error)

<a id="cc_plugins-api-api_version"></a>

### const `API_VERSION`

```rust
pub const API_VERSION: u32
```

The plugin API version this build provides (`api` in `plugin.toml`).

<a id="cc_plugins-api-access"></a>

### struct `Access`

```rust
pub struct Access
```

What the commands a run calls may do.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

**Fields**

- `edits: bool`

  Commands that change the model.

- `files: bool`

  Commands that read or write files.

- `history: bool`

  Commands that replace the document or move in its history (open,
  new, undo, redo), which a plugin command, one undo step, cannot.

- `language: bool`

  The language (a session-wide setting).


**Methods**

- `pub const ALL: Access`

  Everything: a script the user runs.


<a id="cc_plugins-api-api"></a>

### struct `Api`

```rust
pub struct Api
```

The `ccad` table and the private table of the run's functions.

**Fields**

- `ccad: Table`

*Some fields are private.*

**Methods**

- `pub fn install(lua: &Lua) -> mlua::Result<Api>`

  Make the `ccad` global in `lua`.

- `pub fn with_session<R>(&self, lua: &Lua, session: &mut Session, access: Access, f: impl FnOnce(&Lua) -> mlua::Result<R>) -> mlua::Result<R>`

  Run `f` with the session's functions available to the Lua code.


<a id="cc_plugins-api-code_name"></a>

### fn `code_name`

```rust
pub fn code_name(code: ErrorCode) -> String
```

An error code as its JSON name.

<a id="cc_plugins-api-engine_error"></a>

### fn `engine_error`

```rust
pub fn engine_error(e: &mlua::Error) -> EngineError
```

A Lua error as an engine error: a command's error raised through
`ccad.run` keeps its code; anything else is a script error.

<a id="mod-cc_plugins-plugin"></a>

## `cc_plugins::plugin`

<sub>`crates/cc_plugins/src/plugin.rs`</sub>

Plugins: a folder with a `plugin.toml` manifest and Lua code that adds
commands to the session.
```toml
id = "room_tools"        # lowercase letters, digits and _, no "__"
name = "Room tools"
version = "0.1.0"
api = 1                  # the plugin API it was written for
description = "…"
authors = ["…"]
license = "MIT"
homepage = "https://…"
main = "main.lua"        # the default
permissions = ["files"]  # optional: run commands that read or write files
```rust
`main.lua` registers commands with `ccad.command { name, title,
description, kind, params, run }` while it loads; `require("a.b")` loads
`a/b.lua` from the plugin's folder. A command `walls` of the plugin
`room_tools` is `room_tools__walls` in the catalog. Each plugin has its
own sandboxed Lua state.

**Items:** [`MANIFEST`](#cc_plugins-plugin-manifest) · [`Manifest`](#cc_plugins-plugin-manifest) · [`Permissions`](#cc_plugins-plugin-permissions) · [`PluginError`](#cc_plugins-plugin-pluginerror) · [`valid_id`](#cc_plugins-plugin-valid_id) · [`Plugin`](#cc_plugins-plugin-plugin) · [`PluginHost`](#cc_plugins-plugin-pluginhost)

<a id="cc_plugins-plugin-manifest"></a>

### const `MANIFEST`

```rust
pub const MANIFEST: &str
```

The manifest's file name.

<a id="cc_plugins-plugin-manifest"></a>

### struct `Manifest`

```rust
pub struct Manifest
```

A plugin's `plugin.toml`.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Deserialize`, `Serialize`

**Fields**

- `id: String`
- `name: String`
- `version: String`
- `api: u32`
- `description: String`
- `authors: Vec<String>`
- `license: String`
- `homepage: String`
- `main: String`
- `permissions: Vec<String>`

**Methods**

- `pub fn parse(text: &str) -> Result<Manifest, String>`

  Read and check a manifest.

- `pub fn requested(&self) -> Permissions`

  The permissions it asks for.


<a id="cc_plugins-plugin-permissions"></a>

### struct `Permissions`

```rust
pub struct Permissions
```

What a plugin may do beyond querying and editing the model.

**Traits:** `Debug`, `Clone`, `Copy`, `Default`, `PartialEq`, `Eq`, `Serialize`

**Fields**

- `files: bool`

  Run commands that read or write files (import, export, save).


**Methods**

- `pub const NONE: Permissions`

<a id="cc_plugins-plugin-pluginerror"></a>

### struct `PluginError`

```rust
pub struct PluginError
```

A plugin that could not be loaded, or a plugin command refused.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Eq`, `std::fmt::Display`, `std::error::Error`

**Fields**

- `dir: PathBuf`

  The plugin's folder.

- `message: String`

<a id="cc_plugins-plugin-valid_id"></a>

### fn `valid_id`

```rust
pub fn valid_id(s: &str) -> bool
```

Whether `s` is a valid plugin id (see `Manifest`).

<a id="cc_plugins-plugin-plugin"></a>

### struct `Plugin`

```rust
pub struct Plugin
```

A loaded plugin.

**Traits:** `std::fmt::Debug`

**Fields**

- `manifest: Manifest`
- `dir: PathBuf`

*Some fields are private.*

**Methods**

- `pub fn load(dir: &Path, limits: Limits) -> Result<Plugin, PluginError>`

  Load the plugin in `dir`: read its manifest, run its main file and
  keep the commands it registers. It is granted no permission yet.

- `pub fn grant(&mut self, permissions: Permissions)`

  Grant permissions (only those the manifest asks for are kept).

- `pub fn granted(&self) -> Permissions`
- `pub fn commands(&self) -> impl Iterator<Item = &CommandInfo>`

  Its commands' catalog entries (full names).


<a id="cc_plugins-plugin-pluginhost"></a>

### struct `PluginHost`

```rust
pub struct PluginHost
```

The loaded plugins, as the session's `Extension`.

**Traits:** `Debug`, `Default`, `Extension`

*Some fields are private.*

**Methods**

- `pub fn new() -> PluginHost`
- `pub fn add(&mut self, plugin: Plugin) -> Result<(), PluginError>`

  Add a loaded plugin; refused if one with its id is already there.

- `pub fn load_folder(&mut self, folder: &Path, limits: Limits, grant: impl Fn(&Manifest) -> Option<Permissions>) -> Vec<PluginError>`

  Load the plugins of `folder`: the folder itself when it holds a
  manifest, else each of its subfolders that does (in name order),
  skipping hidden ones (an install's `.<id>.installing` and `.<id>.old`)
  and refusing one whose name is not its plugin's id (installs are named
  so; permissions are granted by id). `grant` decides the permissions of
  each, or `None` to leave it out (turned off). Returns the plugins that
  could not be loaded.

- `pub fn plugins(&self) -> &[Plugin]`
- `pub fn summary(&self) -> Value`

  A summary of the plugins for listings (`ccad plugins`).


<a id="mod-cc_plugins-script"></a>

## `cc_plugins::script`

<sub>`crates/cc_plugins/src/script.rs`</sub>

Lua scripts: a file of Lua code run once over a session (`ccad lua`).
The script gets its arguments as `ccad.args` (and as `...`), and what it
returns is its result.

**Items:** [`MAX_SCRIPT`](#cc_plugins-script-max_script) · [`ScriptOptions`](#cc_plugins-script-scriptoptions) · [`ScriptOutput`](#cc_plugins-script-scriptoutput) · [`run_script`](#cc_plugins-script-run_script)

<a id="cc_plugins-script-max_script"></a>

### const `MAX_SCRIPT`

```rust
pub const MAX_SCRIPT: usize
```

Largest script accepted, in bytes.

<a id="cc_plugins-script-scriptoptions"></a>

### struct `ScriptOptions`

```rust
pub struct ScriptOptions
```

How a script runs.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Default`

**Fields**

- `limits: Limits`
- `access: Access`

<a id="cc_plugins-script-scriptoutput"></a>

### struct `ScriptOutput`

```rust
pub struct ScriptOutput
```

What a script did: its result (or error) and the lines it printed.

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `result: Result<Value, EngineError>`
- `output: Vec<String>`

<a id="cc_plugins-script-run_script"></a>

### fn `run_script`

```rust
pub fn run_script(session: &mut Session, source: &str, name: &str, args: Value, options: ScriptOptions) -> ScriptOutput
```

Run the Lua `source` (named `name` in errors) over `session` with `args`.
Each command it runs is its own undo step.

<a id="mod-cc_plugins-store"></a>

## `cc_plugins::store`

<sub>`crates/cc_plugins/src/store.rs`</sub>

Installing plugins: the community index (`index.json` of
[constructcraft-plugins](https://github.com/francisreboucas/constructcraft-plugins)),
the version to offer, and checked installs from a downloaded archive.
Nothing here reaches the network: callers download the index and the
archives (the app does it in a background thread) and pass the bytes.
Archives are hostile input: the checksum must match the index, the zip is
read in memory with bounded sizes, entries with unsafe paths or links are
refused, and the plugin must load (in the sandbox) before it replaces an
installed one.

**Items:** [`INDEX_URL`](#cc_plugins-store-index_url) · [`INDEX_FORMAT`](#cc_plugins-store-index_format) · [`MAX_INDEX`](#cc_plugins-store-max_index) · [`MAX_ARCHIVE`](#cc_plugins-store-max_archive) · [`Index`](#cc_plugins-store-index) · [`IndexPlugin`](#cc_plugins-store-indexplugin) · [`IndexVersion`](#cc_plugins-store-indexversion) · [`parse_version`](#cc_plugins-store-parse_version) · [`Installed`](#cc_plugins-store-installed) · [`installed`](#cc_plugins-store-installed) · [`install`](#cc_plugins-store-install) · [`install_archive`](#cc_plugins-store-install_archive) · [`uninstall`](#cc_plugins-store-uninstall)

<a id="cc_plugins-store-index_url"></a>

### const `INDEX_URL`

```rust
pub const INDEX_URL: &str
```

Where the app reads the index.

<a id="cc_plugins-store-index_format"></a>

### const `INDEX_FORMAT`

```rust
pub const INDEX_FORMAT: u32
```

The index format this build reads.

<a id="cc_plugins-store-max_index"></a>

### const `MAX_INDEX`

```rust
pub const MAX_INDEX: usize
```

Largest index, archive, unpacked plugin and number of files accepted.

<a id="cc_plugins-store-max_archive"></a>

### const `MAX_ARCHIVE`

```rust
pub const MAX_ARCHIVE: usize
```

<a id="cc_plugins-store-index"></a>

### struct `Index`

```rust
pub struct Index
```

The plugin index.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Deserialize`

**Fields**

- `format: u32`
- `plugins: Vec<IndexPlugin>`

**Methods**

- `pub fn parse(json: &[u8]) -> Result<Index, String>`

  Read an index; plugins with bad ids or no usable version are left out.


<a id="cc_plugins-store-indexplugin"></a>

### struct `IndexPlugin`

```rust
pub struct IndexPlugin
```

A plugin in the index.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Deserialize`

**Fields**

- `id: String`
- `name: String`
- `description: String`
- `authors: Vec<String>`
- `license: String`
- `homepage: Option<String>`
- `repository: Option<String>`
- `tags: Vec<String>`
- `versions: Vec<IndexVersion>`

**Methods**

- `pub fn newest(&self) -> Option<&IndexVersion>`

  The newest version, whether this build can run it or not.

- `pub fn best(&self, app: (u64, u64, u64)) -> Option<&IndexVersion>`

  The newest version this build can run: plugin API at most
  `API_VERSION`, `min_app` at most `app`, an https archive.


<a id="cc_plugins-store-indexversion"></a>

### struct `IndexVersion`

```rust
pub struct IndexVersion
```

One downloadable version of a plugin.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Deserialize`

**Fields**

- `version: String`
- `api: u32`
- `url: String`
- `sha256: String`
- `permissions: Vec<String>`
- `path: Option<String>`

  The folder of `plugin.toml` inside the archive.

- `min_app: Option<String>`

  The oldest ConstructCraft version it works with.

- `notes: Option<String>`

<a id="cc_plugins-store-parse_version"></a>

### fn `parse_version`

```rust
pub fn parse_version(text: &str) -> Option<(u64, u64, u64)>
```

A version `major.minor.patch`, ignoring any pre-release or build suffix.

<a id="cc_plugins-store-installed"></a>

### struct `Installed`

```rust
pub struct Installed
```

A plugin found in a plugins folder (loaded or not).

**Traits:** `Debug`, `Clone`, `PartialEq`

**Fields**

- `dir: PathBuf`
- `manifest: Result<Manifest, String>`

  Its manifest, or why it cannot be read.


<a id="cc_plugins-store-installed"></a>

### fn `installed`

```rust
pub fn installed(folder: &Path) -> Vec<Installed>
```

The plugins of `folder`: each subfolder with a manifest, in name order.

<a id="cc_plugins-store-install"></a>

### fn `install`

```rust
pub fn install(archive: &[u8], plugin: &IndexPlugin, version: &IndexVersion, folder: &Path) -> Result<PathBuf, String>
```

Install `version` of `plugin` from its downloaded `archive` into
`folder` (as `folder/<id>`, replacing an installed one). Returns the
plugin's folder.

<a id="cc_plugins-store-install_archive"></a>

### fn `install_archive`

```rust
pub fn install_archive(archive: &[u8], folder: &Path) -> Result<PathBuf, String>
```

Install a plugin archive that is not in the index (a local `.zip`):
the same checks, except the checksum and the index's own fields.

<a id="cc_plugins-store-uninstall"></a>

### fn `uninstall`

```rust
pub fn uninstall(folder: &Path, id: &str) -> Result<(), String>
```

Remove the installed plugin `id` from `folder` (only a folder whose
manifest has that id).

{% endraw %}
