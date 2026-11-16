# `cc_script` API

{% raw %}
[← API reference](index.md) · Generated from the source by `cargo run -p api_doc`; edit the doc comments, not this page.

ConstructCraft's sandboxed Lua 5.4.
`sandbox` is the restricted state every Lua run uses: no files, no
loading of code, an instruction budget and a memory limit. `convert`
turns JSON values into Lua values and back, bounded in depth and size.
`object` runs the scripts of scripted component pieces.
The crate depends only on `cc_core`, so geometry can run scripts without
the engine; `cc_plugins` builds scripts and plugins over engine commands on
top of it.

## Modules

| Module | Items | Summary |
|---|---|---|
| [`cc_script`](#mod-cc_script) | 0 | ConstructCraft's sandboxed Lua 5.4. |
| [`cc_script::convert`](#mod-cc_script-convert) | 5 | JSON values to Lua and back, for command arguments and results. |
| [`cc_script::object`](#mod-cc_script-object) | 11 | Scripted component pieces: running the Lua of a `cc_core::ComponentShape::Script` piece. |
| [`cc_script::patterns`](#mod-cc_script-patterns) | 1 | Lua's pattern matching (`string.find`, `match`, `gmatch` and `gsub`) with every step counted against the run's budget. |
| [`cc_script::sandbox`](#mod-cc_script-sandbox) | 6 | The sandboxed Lua state scripts and plugins run in. |

<a id="mod-cc_script"></a>

## `cc_script`

<sub>`crates/cc_script/src/lib.rs`</sub>

**Re-exports**

- `pub use object::{Declarations, ScriptCache, ScriptError, declare, evaluate, shared}`
- `pub use sandbox::{Limits, Sandbox}`

<a id="mod-cc_script-convert"></a>

## `cc_script::convert`

<sub>`crates/cc_script/src/convert.rs`</sub>

JSON values to Lua and back, for command arguments and results.
Objects become tables with string keys, arrays become sequences marked
with the array metatable (so an empty array stays an array), `null`
becomes `ccad.null`. Back from Lua, a sequence `1..n` (or a table marked
as an array) is an array, any other table an object with string keys;
whole floats become integers (Lua's `/` always gives a float, and ids and
counts are integers). Nesting and size are bounded: tables can be cyclic
or huge.

**Items:** [`MAX_DEPTH`](#cc_script-convert-max_depth) · [`MAX_VALUES`](#cc_script-convert-max_values) · [`MAX_BYTES`](#cc_script-convert-max_bytes) · [`to_lua`](#cc_script-convert-to_lua) · [`from_lua`](#cc_script-convert-from_lua)

<a id="cc_script-convert-max_depth"></a>

### const `MAX_DEPTH`

```rust
pub const MAX_DEPTH: usize
```

Deepest nesting of tables or JSON values converted.

<a id="cc_script-convert-max_values"></a>

### const `MAX_VALUES`

```rust
pub const MAX_VALUES: usize
```

Most values converted at once.

<a id="cc_script-convert-max_bytes"></a>

### const `MAX_BYTES`

```rust
pub const MAX_BYTES: usize
```

Most bytes of text (strings and keys) converted from Lua at once: a
table can hold one large string many times.

<a id="cc_script-convert-to_lua"></a>

### fn `to_lua`

```rust
pub fn to_lua(lua: &Lua, value: &Value) -> mlua::Result<LuaValue>
```

A JSON value as a Lua value.

<a id="cc_script-convert-from_lua"></a>

### fn `from_lua`

```rust
pub fn from_lua(lua: &Lua, value: &LuaValue) -> Result<Value, String>
```

A Lua value as JSON; `Err` says what cannot be converted.

<a id="mod-cc_script-object"></a>

## `cc_script::object`

<sub>`crates/cc_script/src/object.rs`</sub>

Scripted component pieces: running the Lua of a
`cc_core::ComponentShape::Script` piece.
A script declares its parameters and hotspots at its top level and
defines `geometry(p)`, which returns the pieces:
```lua
param.length  { "depth", label = "Depth", default = 0.6 }
param.choice  { "finish", { {"plain", "Plain"}, {"blue", "Blue"} }, default = "plain" }
param.boolean { "doors", default = true, instance = true }
hotspot       { "size", 0.9, 0.6, 0, label = "Size", shape = "diamond", color = "#ffccddee" }
function geometry(p)
  if not p.doors then return {} end
  return { box { 0, 0, 0, p.size.x, p.depth, 0.9, material = "Wood" } }
end
```rust
Scripts are pure: they see the parameter values and nothing else (no
engine commands, files or model), so the same values always give the
same pieces and results can be cached. [`declare`] reads the
declarations, [`evaluate`] runs `geometry`, and [`ScriptCache`] is the
[`ScriptEval`] geometry uses: results by source and values, and the last
pieces that worked for each owner.

**Items:** [`LIMITS`](#cc_script-object-limits) · [`MAX_PIECES`](#cc_script-object-max_pieces) · [`MAX_POINTS`](#cc_script-object-max_points) · [`MAX_PARAMS`](#cc_script-object-max_params) · [`ScriptError`](#cc_script-object-scripterror) · [`Declarations`](#cc_script-object-declarations) · [`declare`](#cc_script-object-declare) · [`evaluate`](#cc_script-object-evaluate) · [`ScriptCache`](#cc_script-object-scriptcache) · [`shared`](#cc_script-object-shared) · [`sync`](#cc_script-object-sync)

<a id="cc_script-object-limits"></a>

### const `LIMITS`

```rust
pub const LIMITS: Limits
```

What one evaluation may use: less than a plugin command, since scripts
come with project files.

<a id="cc_script-object-max_pieces"></a>

### const `MAX_PIECES`

```rust
pub const MAX_PIECES: usize
```

Most pieces a script returns.

<a id="cc_script-object-max_points"></a>

### const `MAX_POINTS`

```rust
pub const MAX_POINTS: usize
```

Most points (outline points and mesh vertices) of all its pieces.

<a id="cc_script-object-max_params"></a>

### const `MAX_PARAMS`

```rust
pub const MAX_PARAMS: usize
```

Most parameters a script declares.

<a id="cc_script-object-scripterror"></a>

### struct `ScriptError`

```rust
pub struct ScriptError
```

Why a script failed, with the line when Lua gave one.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Eq`, `std::fmt::Display`

**Fields**

- `message: String`
- `line: Option<u32>`

<a id="cc_script-object-declarations"></a>

### struct `Declarations`

```rust
pub struct Declarations
```

What a script declares.

**Traits:** `Debug`, `Clone`, `PartialEq`, `Default`

**Fields**

- `params: Vec<ComponentParam>`

  Marked `from_script`.

- `hotspots: Vec<Hotspot>`

<a id="cc_script-object-declare"></a>

### fn `declare`

```rust
pub fn declare(source: &str) -> Result<Declarations, ScriptError>
```

Read what `source` declares (its top level runs; `geometry` does not).

<a id="cc_script-object-evaluate"></a>

### fn `evaluate`

```rust
pub fn evaluate(source: &str, input: &ScriptInput) -> Result<Vec<ComponentItem>, ScriptError>
```

Run `geometry(p)` with the values of `input` and return its pieces.

<a id="cc_script-object-scriptcache"></a>

### struct `ScriptCache`

```rust
pub struct ScriptCache
```

The `ScriptEval` geometry uses: runs a script only for source and
values it has not seen, and keeps the last pieces that worked for each
owner to show while a script fails.

**Traits:** `Default`, `ScriptEval`

*Some fields are private.*

**Methods**

- `pub fn new() -> ScriptCache`
- `pub fn runs(&self) -> u64`

  How many times a script ran (not counting cached results).


<a id="cc_script-object-shared"></a>

### fn `shared`

```rust
pub fn shared() -> &'static ScriptCache
```

The cache shared by everything in the process that resolves components.
Scripts are pure, so sharing it only saves runs; the last pieces that
worked are kept per owner.

<a id="cc_script-object-sync"></a>

### fn `sync`

```rust
pub fn sync(params: &mut Vec<ComponentParam>, hotspots: &mut Vec<Hotspot>, decl: &Declarations) -> Result<(), String>
```

Merge what a script declares into a definition's parameters and
hotspots: new ones are added, those still declared keep their values
when the kind (and, for choices, the value) still fits, and those the
script no longer declares are removed. Parameters made by hand stay.
Fails when a declaration has the name of a parameter made by hand.

<a id="mod-cc_script-patterns"></a>

## `cc_script::patterns`

<sub>`crates/cc_script/src/patterns.rs`</sub>

Lua's pattern matching (`string.find`, `match`, `gmatch` and `gsub`)
with every step counted against the run's budget.
Lua's own matcher runs inside one VM instruction, where the instruction
hook cannot stop it, and backtracking patterns take exponential time
(`("a"):rep(40):find(("a*"):rep(20) .. "b")`). These functions follow
the reference implementation's matcher step for step (same results,
same errors), but charge their steps to the sandbox's `Meter`, which
stops the run when its instructions or its time are spent.

**Items:** [`install`](#cc_script-patterns-install)

<a id="cc_script-patterns-install"></a>

### fn `install`

```rust
pub fn install(lua: &Lua, string: &mlua::Table, meter: &Meter) -> R<()>
```

Put the metered `find`, `match`, `gmatch` and `gsub` in the `string`
table (which strings' methods come from).

<a id="mod-cc_script-sandbox"></a>

## `cc_script::sandbox`

<sub>`crates/cc_script/src/sandbox.rs`</sub>

The sandboxed Lua state scripts and plugins run in.
Only the `table`, `string`, `math`, `utf8` and `coroutine` libraries are
loaded, and the base library without the functions that reach files or
load code (`dofile`, `loadfile`, `load`, `require`) or the collector:
a script touches the project only through engine commands, and files only
through the commands that read and write them, when it may. Each run has
an instruction budget and a time limit (a hook checks them in every
coroutine) and the state a memory limit; `pcall`, `xpcall` and
`coroutine.resume` pass their errors on, so a script cannot catch them
and keep going. Library functions whose work grows with their strings
run inside one instruction, so they are charged to the budget by their
size, `string.rep` refuses results past `MAX_STRING`, and the pattern
functions count their steps (`crate::patterns`).

**Items:** [`Limits`](#cc_script-sandbox-limits) · [`BUDGET_SPENT`](#cc_script-sandbox-budget_spent) · [`TIME_SPENT`](#cc_script-sandbox-time_spent) · [`MAX_STRING`](#cc_script-sandbox-max_string) · [`Meter`](#cc_script-sandbox-meter) · [`Sandbox`](#cc_script-sandbox-sandbox)

<a id="cc_script-sandbox-limits"></a>

### struct `Limits`

```rust
pub struct Limits
```

How much a script may use.

**Traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Default`

**Fields**

- `instructions: u64`

  Lua instructions per run (a script, or one plugin command).

- `memory: usize`

  Memory of the Lua state, in bytes.

- `time: Duration`

  Wall-clock time per run.


<a id="cc_script-sandbox-budget_spent"></a>

### const `BUDGET_SPENT`

```rust
pub const BUDGET_SPENT: &str
```

The message of the error raised when the instruction budget is spent.

<a id="cc_script-sandbox-time_spent"></a>

### const `TIME_SPENT`

```rust
pub const TIME_SPENT: &str
```

The message of the error raised when the time limit is reached.

<a id="cc_script-sandbox-max_string"></a>

### const `MAX_STRING`

```rust
pub const MAX_STRING: usize
```

Largest string `string.rep` and `string.gsub` make, in bytes.

<a id="cc_script-sandbox-meter"></a>

### struct `Meter`

```rust
pub struct Meter
```

A run's budget: instructions spent against the limit, and the time.

**Traits:** `Debug`, `Clone`

*Some fields are private.*

**Methods**

- `pub fn exceeded(&self) -> Option<&'static str>`

  Why the run must stop, if it must.

- `pub fn charge(&self, n: u64) -> mlua::Result<()>`

  Count `n` instructions' worth of work; an error when the budget or
  the time is spent.

- `pub fn max_string(&self) -> usize`

  Largest string a library function may make.


<a id="cc_script-sandbox-sandbox"></a>

### struct `Sandbox`

```rust
pub struct Sandbox
```

A sandboxed Lua state with its budget and captured output.

**Fields**

- `lua: Lua`

*Some fields are private.*

**Methods**

- `pub fn new(limits: Limits) -> mlua::Result<Sandbox>`
- `pub fn reset_budget(&self)`

  Start a new run: a full instruction budget and time.

- `pub fn limits(&self) -> Limits`
- `pub fn take_output(&self) -> Vec<String>`

  The lines printed since the last call (a last line notes any dropped).


{% endraw %}
