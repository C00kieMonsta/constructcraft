# `cc_cli` API

{% raw %}
[← API reference](index.md) · Generated from the source by `cargo run -p api_doc`; edit the doc comments, not this page.

`ccad`: ConstructCraft without a GUI. Runs engine commands from the
command line, from scripts, or for AI agents as an MCP server; all three
use the engine's one command catalog, plus the commands of the plugins
loaded with `--plugins`.

## Modules

| Module | Items | Summary |
|---|---|---|
| [`cc_cli`](#mod-cc_cli) | 0 | `ccad`: ConstructCraft without a GUI. |
| [`cc_cli::mcp`](#mod-cc_cli-mcp) | 3 | A Model Context Protocol server over standard input/output: JSON-RPC 2.0 messages, one per line. |

<a id="mod-cc_cli"></a>

## `cc_cli`

<sub>`crates/cc_cli/src/main.rs`</sub>

<a id="mod-cc_cli-mcp"></a>

## `cc_cli::mcp`

<sub>`crates/cc_cli/src/mcp.rs`</sub>

A Model Context Protocol server over standard input/output: JSON-RPC 2.0
messages, one per line. Every engine command is an MCP tool with the
same name, description and input schema; one session (one open project)
lives as long as the server.

**Items:** [`MAX_LINE`](#cc_cli-mcp-max_line) · [`serve`](#cc_cli-mcp-serve) · [`handle`](#cc_cli-mcp-handle)

<a id="cc_cli-mcp-max_line"></a>

### const `MAX_LINE`

```rust
pub const MAX_LINE: usize
```

Longest message line read, in bytes; a longer one is skipped and answered
with a parse error.

<a id="cc_cli-mcp-serve"></a>

### fn `serve`

```rust
pub fn serve(session: Session, input: impl BufRead, output: impl Write) -> std::io::Result<()>
```

Serve until standard input closes.

<a id="cc_cli-mcp-handle"></a>

### fn `handle`

```rust
pub fn handle(session: &mut Session, message: Value) -> Option<Value>
```

The reply to one message (none for notifications).

{% endraw %}
