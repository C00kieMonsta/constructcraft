//! The engine checks what a plugin command runs (its `PluginScope`) for
//! every command executed during it, also those a graph it starts runs.

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_engine::{ErrorCode, Session};
use cc_plugins::{Limits, Plugin, PluginHost};
use serde_json::{Value, json};
use std::path::PathBuf;

const MAIN: &str = r#"
local function node_graph(command, values)
  return { nodes = { { id = 1, kind = { command = command }, values = values } } }
end
ccad.command { name = "graph_data", run = function(a)
  return ccad.run("graph_run", { graph = node_graph("plugin_data_set", { plugin = a.plugin, data = 7 }) })
end }
ccad.command { name = "graph_edit", kind = "query", run = function()
  return ccad.run("graph_run", { graph = node_graph("level_create", { name = "Sneaky", elevation = 9 }) })
end }
ccad.command { name = "call", run = function(a) return ccad.run(a.name, a.args) end }
ccad.command { name = "fail", run = function() error("stop") end }
"#;

fn session() -> Session {
    let dir = std::env::temp_dir().join(format!("ccad-scope-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("plugin.toml"), "id = \"p\"\nname = \"P\"\nversion = \"1.0\"\n").unwrap();
    std::fs::write(dir.join("main.lua"), MAIN).unwrap();
    let plugin = Plugin::load(&PathBuf::from(&dir), Limits::default()).unwrap();
    let mut host = PluginHost::new();
    host.add(plugin).unwrap();
    let mut s = Session::default();
    s.set_extension(Some(Box::new(host)));
    s
}

fn node_error(report: &Value) -> String {
    report["nodes"]["1"]["error"].as_str().unwrap_or_default().to_string()
}

/// Before the scope moved into the engine, a graph's nodes (which call
/// `Session::execute` directly) escaped the plugin's own-data check: this
/// wrote the other plugin's data.
#[test]
fn a_graph_run_by_a_plugin_keeps_to_its_own_data() {
    let mut s = session();
    let report = s.execute("p__graph_data", json!({"plugin": "other"})).unwrap();
    assert!(node_error(&report).contains("only its own data"), "{report}");
    assert_eq!(s.execute("plugin_data_get", json!({"plugin": "other"})).unwrap(), Value::Null);
    // Its own id goes through.
    let report = s.execute("p__graph_data", json!({"plugin": "p"})).unwrap();
    assert_eq!(node_error(&report), "", "{report}");
    assert_eq!(s.execute("plugin_data_get", json!({"plugin": "p"})).unwrap(), json!(7));
}

#[test]
fn a_query_command_cannot_edit_through_a_graph() {
    let mut s = session();
    let levels = s.project().levels.len();
    // graph_run itself is an edit command.
    let e = s.execute("p__graph_edit", json!({})).unwrap_err();
    assert_eq!(e.code, ErrorCode::Rejected, "{e:?}");
    assert_eq!(s.project().levels.len(), levels);
}

#[test]
fn history_and_language_stay_out_of_plugin_commands() {
    let mut s = session();
    for name in ["undo", "redo", "project_new", "language_set"] {
        let e = s.execute("p__call", json!({"name": name, "args": {}})).unwrap_err();
        assert_eq!(e.code, ErrorCode::Rejected, "{name}");
        assert!(e.message.contains("inside a plugin command"), "{name}: {}", e.message);
    }
}

#[test]
fn the_scope_ends_with_the_command() {
    let mut s = session();
    assert!(s.execute("p__fail", json!({})).is_err());
    assert!(s.plugin_scope().is_none());
    assert!(s.files_allowed());
    // The user's own commands reach any plugin's data again.
    s.execute("plugin_data_set", json!({"plugin": "other", "data": 1})).unwrap();
    assert_eq!(s.execute("plugin_data_get", json!({"plugin": "other"})).unwrap(), json!(1));
}
