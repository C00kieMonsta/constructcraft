//! Scoped file access: the files a plugin command's `save_path` and
//! `open_path` arguments grant, `ccad.file`, and the manifest's exporters
//! and importers.

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_engine::{ErrorCode, Session};
use cc_plugins::{Limits, Permissions, Plugin, PluginHost, ScriptOptions, run_script};
use serde_json::json;
use std::path::{Path, PathBuf};

const MAIN: &str = r#"
ccad.command { name = "export_txt", kind = "query",
  params = { required = { "path" }, properties = {
    path = { type = "string", ui = "save_path", extensions = { "txt", "idf" } },
    text = { type = "string" },
    other = { type = "string" },
    fail = { type = "boolean" },
  } },
  run = function(a)
    ccad.file.write(a.path, a.text or "hello")
    if a.other then ccad.file.write(a.other, "x") end
    if a.fail then error("failed after writing") end
    return a.path
  end }
ccad.command { name = "chunks", kind = "query",
  params = { properties = { path = { type = "string", ui = "save_path" } } },
  run = function(a) ccad.file.write(a.path, { "a", "b", "c" }); return true end }
ccad.command { name = "read_save", kind = "query",
  params = { properties = { path = { type = "string", ui = "save_path" } } },
  run = function(a) return ccad.file.read(a.path) end }
ccad.command { name = "import_txt",
  params = { required = { "path" }, properties = {
    path = { type = "string", ui = "open_path", extensions = { "txt" } },
  } },
  run = function(a)
    local text = ccad.file.read(a.path)
    ccad.data.set(nil, text)
    return #text
  end }
ccad.command { name = "ifc", kind = "query",
  params = { properties = {
    path = { type = "string", ui = "save_path", extensions = { "ifc" } },
    target = { type = "string" },
  } },
  run = function(a) return ccad.run("ifc_export", { path = a.target or a.path }) end }
ccad.command { name = "flood", kind = "query",
  params = { properties = { path = { type = "string", ui = "save_path" } } },
  run = function(a)
    local mib = string.rep("x", 1024 * 1024)
    for _ = 1, 300 do ccad.file.write(a.path, mib) end
    return true
  end }
ccad.command { name = "outside", kind = "query", run = function() return ccad.file.read("plugin.toml") end }
ccad.command { name = "call",
  params = { properties = { path = { type = "string", ui = "save_path" }, name = { type = "string" }, args = { type = "object" } } },
  run = function(a) return ccad.run(a.name, a.args) end }
"#;

const EXPORTERS: &str = r#"
[[exporters]]
command = "export_txt"
label = "%export.txt%"

[[importers]]
command = "import_txt"
label = "Text (.txt)"
"#;

/// A fresh temporary folder.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ccad-files-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The test plugin `p` (api 2) in `dir/p`, with `extra` in its manifest.
fn plugin_dir(dir: &Path, extra: &str, main: &str) -> PathBuf {
    let p = dir.join("p");
    std::fs::create_dir_all(p.join("locales")).unwrap();
    let manifest = format!("id = \"p\"\nname = \"Plugin P\"\nversion = \"1.0\"\napi = 2\n{extra}");
    std::fs::write(p.join("plugin.toml"), manifest).unwrap();
    std::fs::write(p.join("main.lua"), main).unwrap();
    std::fs::write(p.join("locales").join("en.toml"), "[export]\ntxt = \"Text file (.txt)\"\n").unwrap();
    p
}

fn session(dir: &Path, files: bool) -> Session {
    let extra = if files { "permissions = [\"files\"]\n" } else { "" };
    let mut plugin = Plugin::load(&plugin_dir(dir, extra, MAIN), Limits::default()).unwrap();
    plugin.grant(Permissions { files, ..Permissions::NONE });
    let mut host = PluginHost::new();
    host.add(plugin).unwrap();
    let mut s = Session::default();
    s.set_extension(Some(Box::new(host)));
    s
}

fn path(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

/// The names of the files in `dir` (sorted).
fn files_in(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn a_command_writes_its_granted_save_path() {
    let dir = scratch("write");
    let mut s = session(&dir, false);
    let out = dir.join("out.idf");
    s.execute("p__export_txt", json!({"path": path(&out), "text": "model"})).unwrap();
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "model");
    // The extension is checked without regard to case; a list of strings is joined.
    let upper = dir.join("OUT2.TXT");
    s.execute("p__export_txt", json!({"path": path(&upper)})).unwrap();
    assert_eq!(std::fs::read_to_string(&upper).unwrap(), "hello");
    let joined = dir.join("joined.csv");
    s.execute("p__chunks", json!({"path": path(&joined)})).unwrap();
    assert_eq!(std::fs::read_to_string(&joined).unwrap(), "abc");
    // A spelling with `..` is the same file.
    let spelled = dir.join("sub").join("..").join("out.idf");
    std::fs::create_dir_all(dir.join("sub")).unwrap();
    s.execute("p__export_txt", json!({"path": path(&spelled), "text": "again"})).unwrap();
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "again");
    // No temporary file is left behind.
    assert_eq!(files_in(&dir), vec!["OUT2.TXT", "joined.csv", "out.idf"]);
}

#[test]
fn other_paths_and_directions_are_refused() {
    let dir = scratch("refused");
    let mut s = session(&dir, false);
    let out = dir.join("out.txt");
    let other = dir.join("other.txt");
    let e = s.execute("p__export_txt", json!({"path": path(&out), "other": path(&other)})).unwrap_err();
    assert_eq!(e.code, ErrorCode::Rejected, "{e:?}");
    assert!(e.message.contains("save_path"), "{}", e.message);
    // Nothing is written, not even the granted file.
    assert!(!out.exists() && !other.exists());
    // A save path cannot be read.
    std::fs::write(&out, "secret").unwrap();
    let e = s.execute("p__read_save", json!({"path": path(&out)})).unwrap_err();
    assert_eq!(e.code, ErrorCode::Rejected, "{e:?}");
    // Nothing is readable without an open_path argument.
    let e = s.execute("p__outside", json!({})).unwrap_err();
    assert_eq!(e.code, ErrorCode::Rejected, "{e:?}");
    // The user's scripts have no ccad.file.
    let out = run_script(&mut s, "return ccad.file == nil", "t.lua", json!({}), ScriptOptions::default());
    assert_eq!(out.result.unwrap(), json!(true));
}

#[test]
fn a_wrong_or_empty_path_is_refused_before_running() {
    let dir = scratch("ext");
    let mut s = session(&dir, false);
    let png = dir.join("out.png");
    let e = s.execute("p__export_txt", json!({"path": path(&png)})).unwrap_err();
    assert_eq!(e.code, ErrorCode::InvalidArguments, "{e:?}");
    assert!(e.message.contains(".txt, .idf"), "{}", e.message);
    let e = s.execute("p__export_txt", json!({"path": ""})).unwrap_err();
    assert_eq!(e.code, ErrorCode::InvalidArguments, "{e:?}");
    let e = s.execute("p__export_txt", json!({"path": 3})).unwrap_err();
    assert_eq!(e.code, ErrorCode::InvalidArguments, "{e:?}");
    // In a folder that does not exist.
    let e = s.execute("p__export_txt", json!({"path": path(&dir.join("no").join("x.txt"))})).unwrap_err();
    assert_eq!(e.code, ErrorCode::InvalidArguments, "{e:?}");
    // A file to read must exist.
    let e = s.execute("p__import_txt", json!({"path": path(&dir.join("missing.txt"))})).unwrap_err();
    assert_eq!(e.code, ErrorCode::InvalidArguments, "{e:?}");
    assert!(files_in(&dir).is_empty());
}

#[test]
fn a_failed_command_leaves_no_file() {
    let dir = scratch("failed");
    let mut s = session(&dir, false);
    let out = dir.join("out.txt");
    let e = s.execute("p__export_txt", json!({"path": path(&out), "fail": true})).unwrap_err();
    assert!(e.message.contains("failed after writing"), "{}", e.message);
    assert!(files_in(&dir).is_empty(), "{:?}", files_in(&dir));
    // An existing file is kept as it was.
    std::fs::write(&out, "before").unwrap();
    assert!(s.execute("p__export_txt", json!({"path": path(&out), "fail": true})).is_err());
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "before");
    assert_eq!(files_in(&dir), vec!["out.txt"]);
}

#[test]
fn a_command_reads_its_granted_open_path() {
    let dir = scratch("read");
    let mut s = session(&dir, false);
    let input = dir.join("in.txt");
    std::fs::write(&input, "weather").unwrap();
    let before = s.doc.undo_depth();
    assert_eq!(s.execute("p__import_txt", json!({"path": path(&input)})).unwrap(), json!(7));
    assert_eq!(s.execute("plugin_data_get", json!({"plugin": "p"})).unwrap(), json!("weather"));
    assert_eq!(s.doc.undo_depth(), before + 1);
}

#[test]
fn size_limits_hold() {
    let dir = scratch("limits");
    let mut s = session(&dir, false);
    // Past 256 MiB written in one run.
    let out = dir.join("flood.bin");
    let e = s.execute("p__flood", json!({"path": path(&out)})).unwrap_err();
    assert_eq!(e.code, ErrorCode::InvalidArguments, "{e:?}");
    assert!(e.message.contains("256 MiB"), "{}", e.message);
    assert!(!out.exists());
    // Past 64 MiB read.
    let big = dir.join("big.txt");
    std::fs::File::create(&big).unwrap().set_len(cc_plugins::api::MAX_FILE_READ + 1).unwrap();
    let e = s.execute("p__import_txt", json!({"path": path(&big)})).unwrap_err();
    assert_eq!(e.code, ErrorCode::InvalidArguments, "{e:?}");
    assert!(e.message.contains("64 MiB"), "{}", e.message);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn engine_exports_reach_only_the_granted_path() {
    let dir = scratch("ifc");
    let mut s = session(&dir, false);
    let out = dir.join("model.ifc");
    s.execute("p__ifc", json!({"path": path(&out)})).unwrap();
    assert!(out.exists());
    let elsewhere = dir.join("elsewhere.ifc");
    let e = s.execute("p__ifc", json!({"path": path(&out), "target": path(&elsewhere)})).unwrap_err();
    assert_eq!(e.code, ErrorCode::Rejected, "{e:?}");
    assert!(!elsewhere.exists());
    // Without a granted path at all.
    let e = s.execute("p__ifc", json!({"target": path(&elsewhere)})).unwrap_err();
    assert_eq!(e.code, ErrorCode::Rejected, "{e:?}");
    // File commands that do not check their paths stay out of reach, even
    // given the granted path; and a granted path to write is not one to read.
    let ccad = dir.join("x.ccad");
    let args = json!({"path": path(&ccad), "name": "project_save", "args": {"path": path(&ccad)}});
    let e = s.execute("p__call", args).unwrap_err();
    assert_eq!(e.code, ErrorCode::Rejected, "{e:?}");
    assert!(!ccad.exists());
    let args = json!({"path": path(&out), "name": "component_load", "args": {"path": path(&out)}});
    let e = s.execute("p__call", args).unwrap_err();
    assert_eq!(e.code, ErrorCode::Rejected, "{e:?}");
    // A plugin with the files permission writes anywhere.
    let mut s = session(&scratch("ifc-files"), true);
    s.execute("p__ifc", json!({"path": path(&out), "target": path(&elsewhere)})).unwrap();
    assert!(elsewhere.exists());
}

#[test]
fn exporters_and_importers_are_read() {
    let dir = scratch("actions");
    let plugin = Plugin::load(&plugin_dir(&dir, EXPORTERS, MAIN), Limits::default()).unwrap();
    cc_engine::i18n::set_language(cc_engine::i18n::Language::En);
    let exporters = plugin.exporters();
    assert_eq!(exporters.len(), 1);
    assert_eq!(exporters[0].command, "p__export_txt");
    assert_eq!(exporters[0].label, "Text file (.txt)");
    assert_eq!(exporters[0].param, "path");
    assert_eq!(exporters[0].extensions, vec!["txt", "idf"]);
    let importers = plugin.importers();
    assert_eq!(importers[0].command, "p__import_txt");
    assert_eq!(importers[0].label, "Text (.txt)");
    let mut host = PluginHost::new();
    host.add(plugin).unwrap();
    let summary = host.summary();
    assert_eq!(summary[0]["exporters"][0]["command"], "p__export_txt");
    assert_eq!(summary[0]["importers"][0]["extensions"], json!(["txt"]));
}

#[test]
fn exporters_and_importers_are_checked() {
    let dir = scratch("actions-bad");
    let fails = |name: &str, extra: &str, main: &str| -> String {
        let d = dir.join(name);
        std::fs::create_dir_all(&d).unwrap();
        Plugin::load(&plugin_dir(&d, extra, main), Limits::default()).unwrap_err().message
    };
    let e = fails("unknown", "[[exporters]]\ncommand = \"nope\"\nlabel = \"X\"\n", MAIN);
    assert!(e.contains("exporters[1]") && e.contains("does not register"), "{e}");
    // An exporter needs exactly one save_path parameter; an importer one open_path.
    let e = fails("none", "[[exporters]]\ncommand = \"import_txt\"\nlabel = \"X\"\n", MAIN);
    assert!(e.contains("exactly one") && e.contains("save_path") && e.contains("has 0"), "{e}");
    let two = r#"ccad.command { name = "two", params = { properties = {
      a = { type = "string", ui = "save_path" }, b = { type = "string", ui = "save_path" } } },
      run = function() end }"#;
    let e = fails("two", "[[exporters]]\ncommand = \"two\"\nlabel = \"X\"\n", two);
    assert!(e.contains("has 2"), "{e}");
    let e = fails("imp", "[[importers]]\ncommand = \"export_txt\"\nlabel = \"X\"\n", MAIN);
    assert!(e.contains("importers[1]") && e.contains("open_path"), "{e}");
    let e = fails("label", "[[exporters]]\ncommand = \"export_txt\"\nlabel = \"\"\n", MAIN);
    assert!(e.contains("label"), "{e}");
    let many: String = (0..17).map(|_| "[[importers]]\ncommand = \"import_txt\"\nlabel = \"X\"\n").collect();
    let e = fails("many", &many, MAIN);
    assert!(e.contains("at most 16 importers"), "{e}");
    let e = fails("unknown-field", "[[exporters]]\ncommand = \"export_txt\"\nlabel = \"X\"\nicon = \"a\"\n", MAIN);
    assert!(e.contains("unknown field"), "{e}");
    // Under api 1 they are refused.
    let d = dir.join("api1");
    let p = plugin_dir(&d, "[[exporters]]\ncommand = \"export_txt\"\nlabel = \"X\"\n", MAIN);
    let manifest = std::fs::read_to_string(p.join("plugin.toml")).unwrap().replace("api = 2", "api = 1");
    std::fs::write(p.join("plugin.toml"), manifest).unwrap();
    let e = Plugin::load(&p, Limits::default()).unwrap_err().message;
    assert!(e.contains("exporters need api = 2"), "{e}");
}

#[test]
fn path_hints_are_checked() {
    let dir = scratch("hints");
    let fails = |name: &str, prop: &str| -> String {
        let d = dir.join(name);
        std::fs::create_dir_all(&d).unwrap();
        let main = format!(
            "ccad.command {{ name = \"c\", params = {{ properties = {{ f = {prop} }} }}, run = function() end }}"
        );
        Plugin::load(&plugin_dir(&d, "", &main), Limits::default()).unwrap_err().message
    };
    let e = fails("type", r#"{ type = "number", ui = "save_path" }"#);
    assert!(e.contains("needs type \"string\""), "{e}");
    for (name, list) in [
        ("upper", r#"{ "IDF" }"#),
        ("dot", r#"{ ".idf" }"#),
        ("empty", r#"{ "" }"#),
        ("twice", r#"{ "a", "a" }"#),
        ("long", r#"{ "abcdefghijklmnopq" }"#),
        ("none", r#"ccad.array()"#),
        ("number", r#"{ 3 }"#),
    ] {
        let e = fails(name, &format!(r#"{{ type = "string", ui = "open_path", extensions = {list} }}"#));
        assert!(e.contains("extensions"), "{name}: {e}");
    }
    let seventeen: Vec<String> = (0..17).map(|i| format!("\"e{i}\"")).collect();
    let e = fails(
        "many",
        &format!(r#"{{ type = "string", ui = "open_path", extensions = {{ {} }} }}"#, seventeen.join(", ")),
    );
    assert!(e.contains("extensions"), "{e}");
    let e = fails("no-ui", r#"{ type = "string", extensions = { "txt" } }"#);
    assert!(e.contains("extensions needs ui"), "{e}");
    // Good ones load.
    let d = dir.join("good");
    std::fs::create_dir_all(&d).unwrap();
    let main = r#"ccad.command { name = "c", params = { properties = {
      f = { type = "string", ui = "open_path", extensions = { "epw", "csv2" } },
      g = { type = "string", ui = "save_path" } } }, run = function() end }"#;
    let plugin = Plugin::load(&plugin_dir(&d, "", main), Limits::default()).unwrap();
    let info = plugin.commands().next().unwrap();
    assert_eq!(info.input_schema["properties"]["f"]["extensions"], json!(["epw", "csv2"]));
}
