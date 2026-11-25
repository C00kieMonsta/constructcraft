//! Revit files: `rvt_import` converts a .rvt to IFC4 with the `rvt` crate and imports it.

use cc_engine::{ErrorCode, Session};
use serde_json::json;
use std::path::PathBuf;

/// A synthetic Revit 2024 file written by rvt-rs's `gen-fixture` (no Autodesk content).
fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../cc_rvt/tests/fixtures/synthetic.rvt")
}

#[test]
fn a_revit_file_replaces_the_project() {
    let mut s = Session::default();
    s.execute("wall_create", json!({"start": [0.0, 0.0], "end": [5.0, 0.0]})).unwrap();
    let out = s.execute("rvt_import", json!({"path": fixture()})).unwrap();
    assert_eq!(out["schema"], "IFC4");
    assert_eq!(out["revit_version"], 2024);
    for key in ["elements", "with_geometry", "boxes"] {
        assert!(out["converted"][key].is_u64(), "{key}: {out}");
    }
    assert!(out["converted"]["notes"].is_array());
    assert_eq!(s.project().walls().count(), 0, "the import replaces the open project");
    assert!(s.doc.path.is_none());
}

#[test]
fn a_file_that_is_not_revit_is_an_error_and_keeps_the_project() {
    let path = std::env::temp_dir().join(format!("ccad-not-revit-{}.rvt", std::process::id()));
    std::fs::write(&path, "ISO-10303-21;\nHEADER;\nENDSEC;\n").unwrap();
    let mut s = Session::default();
    s.execute("wall_create", json!({"start": [0.0, 0.0], "end": [5.0, 0.0]})).unwrap();
    let err = s.execute("rvt_import", json!({"path": path})).unwrap_err();
    std::fs::remove_file(&path).ok();
    assert_eq!(err.code, ErrorCode::Io, "{err:?}");
    assert_eq!(s.project().walls().count(), 1);
    let missing = s.execute("rvt_import", json!({"path": "/no/such/file.rvt"})).unwrap_err();
    assert_eq!(missing.code, ErrorCode::Io);
}
