//! The scope of a plugin command (`Session::with_plugin_scope`) and file
//! access (`FileAccess`), checked by `Session::execute` for every command.

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_engine::{ErrorCode, FileAccess, Grants, PluginScope, ScopeEdits, Session};
use serde_json::json;

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("ccad-scope-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn scope(edits: bool, files: FileAccess) -> PluginScope {
    PluginScope { plugin: "p".into(), edits: if edits { ScopeEdits::All } else { ScopeEdits::None }, files }
}

#[test]
fn a_scope_holds_for_every_command_and_ends() {
    let mut s = Session::default();
    let graph = json!({"nodes": [{"id": 1, "kind": {"command": "plugin_data_set"},
        "values": {"plugin": "other", "data": 1}}]});
    s.with_plugin_scope(scope(true, FileAccess::All), |s| {
        let e = s.execute("plugin_data_set", json!({"plugin": "other", "data": 1})).unwrap_err();
        assert_eq!(e.code, ErrorCode::Rejected);
        s.execute("plugin_data_set", json!({"plugin": "p", "data": 1})).unwrap();
        // A graph's nodes are held to it too.
        let report = s.execute("graph_run", json!({"graph": graph})).unwrap();
        assert!(report["nodes"]["1"]["error"].as_str().unwrap().contains("only its own data"), "{report}");
        for name in ["undo", "redo", "project_new", "language_set"] {
            assert_eq!(s.execute(name, json!({})).unwrap_err().code, ErrorCode::Rejected, "{name}");
        }
        assert_eq!(s.plugin_scope().map(|p| p.plugin.as_str()), Some("p"));
    });
    // A query scope cannot edit, also in a scope nested in an edit one.
    s.with_plugin_scope(scope(true, FileAccess::All), |s| {
        s.with_plugin_scope(scope(false, FileAccess::All), |s| {
            let e = s.execute("level_create", json!({"name": "X", "elevation": 7})).unwrap_err();
            assert_eq!(e.code, ErrorCode::Rejected);
        });
        assert!(s.execute("level_create", json!({"name": "X", "elevation": 7})).is_ok());
    });
    assert!(s.plugin_scope().is_none());
    s.execute("plugin_data_set", json!({"plugin": "other", "data": 2})).unwrap();
    s.execute("undo", json!({})).unwrap();
}

#[test]
fn granted_paths_reach_only_commands_that_check_them() {
    let dir = scratch("granted");
    let out = dir.join("model.ifc");
    let mut grants = Grants::new();
    grants.grant_write(&out).unwrap();
    let mut s = Session::default();
    s.with_plugin_scope(scope(true, FileAccess::Granted(grants)), |s| {
        assert!(!s.files_allowed());
        s.execute("ifc_export", json!({"path": out})).unwrap();
        let e = s.execute("ifc_export", json!({"path": dir.join("other.ifc")})).unwrap_err();
        assert_eq!(e.code, ErrorCode::Rejected);
        // A file command that does not check its paths, and the wrong direction.
        let e = s.execute("project_save", json!({"path": out})).unwrap_err();
        assert_eq!(e.code, ErrorCode::Rejected);
        let e = s.execute("cad_import", json!({"path": out, "view": "Plan"})).unwrap_err();
        assert_eq!(e.code, ErrorCode::Rejected);
        // A graph without allow_files narrows it to nothing.
        let g = json!({"nodes": [{"id": 1, "kind": {"command": "ifc_export"}, "values": {"path": out}}]});
        let report = s.execute("graph_run", json!({"graph": g})).unwrap();
        assert!(report["nodes"]["1"]["error"].as_str().unwrap().contains("files"), "{report}");
    });
    assert!(out.exists());
    assert!(!dir.join("other.ifc").exists());
    assert!(s.files_allowed(), "access comes back with the scope's end");
    let _ = std::fs::remove_dir_all(&dir);
}
