//! Plugin development: the type definitions `docs/lua/ccad.d.lua` cover
//! the runtime `ccad` table, and plugin tests (`cc_plugins::testing`) run
//! in fresh sessions.

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_engine::Session;
use cc_plugins::api::Api;
use cc_plugins::sandbox::Sandbox;
use cc_plugins::testing::{TestOptions, install_helpers, run_tests};
use cc_plugins::{Limits, Plugin, PluginHost};
use serde_json::json;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const TYPES: &str = include_str!("../../../docs/lua/ccad.d.lua");

/// The names `ccad.d.lua` declares, as paths: `ccad.run`, `ccad.data.get`,
/// `ccad.plugin.id`… (from `function ccad.x(`, `ccad.x =` and the
/// `---@field` lines of each `---@class`).
fn declared() -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut class = String::new();
    for line in TYPES.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("---@class ") {
            class = rest.split([' ', ':']).next().unwrap_or_default().to_string();
        } else if let Some(rest) = line.strip_prefix("---@field ") {
            let field = rest.split_whitespace().next().unwrap_or_default().trim_end_matches('?');
            names.insert(format!("{class}.{field}"));
        } else if let Some(rest) = line.strip_prefix("function ") {
            names.insert(rest.split('(').next().unwrap_or_default().to_string());
        } else if line.starts_with("ccad") && line.contains(" = ") {
            names.insert(line.split(" = ").next().unwrap_or_default().to_string());
        } else if !line.starts_with("---") {
            class.clear();
        }
    }
    names
}

/// Lua that lists the paths of the `ccad` table and of its sub-tables (not
/// into `settings` or `args`, which hold values, not API).
const WALK: &str = r#"
local out = {}
local function walk(t, prefix, depth)
  for k, v in next, t do
    local path = prefix .. "." .. tostring(k)
    out[#out + 1] = path
    if type(v) == "table" and depth < 2 and k ~= "settings" and k ~= "args" then walk(v, path, depth + 1) end
  end
end
walk(ccad, "ccad", 1)
table.sort(out)
return out
"#;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ccad-testing-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn files_in(dir: &Path, files: &[(&str, &str)]) {
    for (path, text) in files {
        let p = dir.join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }
}

fn missing(found: &[String]) -> Vec<String> {
    let declared = declared();
    found.iter().filter(|p| !declared.contains(*p)).cloned().collect()
}

#[test]
fn the_type_definitions_declare_the_whole_runtime_table() {
    // A plugin's table: what a command sees, with settings, data, file and process.
    let dir = scratch("types");
    let main = format!(
        "ccad.command {{ name = 'walk', kind = 'query', run = function() {WALK} end }}\n\
         ccad.on('model_changed', function() end)"
    );
    files_in(
        &dir,
        &[
            (
                "plugin.toml",
                "id = \"walker\"\nname = \"W\"\nversion = \"1\"\napi = 2\n[settings.size]\ntype = \"number\"\ndefault = 1\n",
            ),
            ("main.lua", &main),
        ],
    );
    let mut host = PluginHost::new();
    host.add(Plugin::load(&dir, Limits::default()).unwrap()).unwrap();
    let mut s = Session::default();
    s.set_extension(Some(Box::new(host)));
    let paths: Vec<String> = serde_json::from_value(s.execute("walker__walk", json!({})).unwrap()).unwrap();
    for expected in ["ccad.run", "ccad.data.set", "ccad.file.write", "ccad.process.run", "ccad.plugin.id", "ccad.on"] {
        assert!(paths.iter().any(|p| p == expected), "{expected} in {paths:?}");
    }
    assert_eq!(missing(&paths), Vec::<String>::new(), "plugin names missing from docs/lua/ccad.d.lua");

    // A script's (and a test's) table: args and ccad.test.
    let sandbox = Sandbox::new(Limits::default()).unwrap();
    let api = Api::install(&sandbox.lua).unwrap();
    api.ccad.raw_set("args", sandbox.lua.create_table().unwrap()).unwrap();
    install_helpers(&sandbox.lua, &api.ccad).unwrap();
    let list: mlua::Table = sandbox.lua.load(WALK).eval().unwrap();
    let paths: Vec<String> = list.sequence_values::<String>().map(Result::unwrap).collect();
    assert!(paths.iter().any(|p| p == "ccad.test.near"), "{paths:?}");
    assert_eq!(missing(&paths), Vec::<String>::new(), "script names missing from docs/lua/ccad.d.lua");

    // The check itself catches a name left out.
    assert_eq!(missing(&["ccad.frobnicate".into()]), vec!["ccad.frobnicate".to_string()]);
    let _ = std::fs::remove_dir_all(&dir);
}

/// A plugin whose command makes a wall and counts the walls.
fn walls_plugin(name: &str, tests: &[(&str, &str)]) -> PathBuf {
    let dir = scratch(name);
    files_in(
        &dir,
        &[
            ("plugin.toml", "id = \"walls\"\nname = \"Walls\"\nversion = \"1\"\napi = 2\n"),
            (
                "main.lua",
                "ccad.command { name = 'one', run = function() print('one wall') \
                 return ccad.run('wall_create', { start = { 0, 0 }, ['end'] = { 3, 0 } }).id end }",
            ),
        ],
    );
    files_in(&dir, tests);
    dir
}

fn summary(dir: &Path, filter: Option<&str>) -> Vec<(String, Result<(), String>)> {
    let options = TestOptions { filter: filter.map(str::to_string), ..TestOptions::default() };
    run_tests(dir, &options).unwrap().into_iter().map(|r| (r.name, r.result)).collect()
}

#[test]
fn plugin_tests_pass_and_fail_each_in_a_fresh_session() {
    let dir = walls_plugin(
        "run",
        &[
            (
                "tests/a_test.lua",
                "local id = ccad.run('walls__one')\nccad.test.ok(id > 0)\n\
                 ccad.test.eq(#ccad.run('elements_list', { category = 'wall' }), 1, 'one wall')",
            ),
            (
                "tests/b_test.lua",
                "count = 0\n\
                 local function walls() return #ccad.run('elements_list', { category = 'wall' }) end\n\
                 function test_first() ccad.run('walls__one') ccad.test.eq(walls(), 1) end\n\
                 function test_second() ccad.run('walls__one') ccad.test.eq(walls(), 1, 'a fresh session') end\n\
                 function test_tables() ccad.test.eq({ a = { 1, 2 }, b = 'x' }, { b = 'x', a = { 1, 2 } }) end\n\
                 function test_near() ccad.test.near(0.1 + 0.2, 0.3) ccad.test.near(1.0, 1.05, 0.1) end\n\
                 function test_fails() ccad.test.fails(function() ccad.run('element_get', { id = 99999 }) end, '99999')\n\
                 ccad.test.fails(function() error('a (plain) message') end, '(plain)') end\n\
                 function test_broken() ccad.test.eq({ 1, 2 }, { 1, 3 }, 'lists') end\n\
                 function test_no_error() ccad.test.fails(function() end) end\n\
                 function test_false() ccad.test.ok(false, 'it holds') end",
            ),
            ("tests/c_test.lua", "error('bad top level')"),
            ("tests/helper.lua", "error('not a test file')"),
        ],
    );
    let results = summary(&dir, None);
    let names: Vec<&str> = results.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(
        names,
        [
            "a_test.lua",
            "b_test.lua::test_broken",
            "b_test.lua::test_fails",
            "b_test.lua::test_false",
            "b_test.lua::test_first",
            "b_test.lua::test_near",
            "b_test.lua::test_no_error",
            "b_test.lua::test_second",
            "b_test.lua::test_tables",
            "c_test.lua",
        ]
    );
    let result = |name: &str| results.iter().find(|(n, _)| n == name).unwrap().1.clone();
    for passing in ["a_test.lua", "b_test.lua::test_first", "b_test.lua::test_second", "b_test.lua::test_tables"] {
        assert_eq!(result(passing), Ok(()), "{passing}");
    }
    assert_eq!(result("b_test.lua::test_near"), Ok(()));
    assert_eq!(result("b_test.lua::test_fails"), Ok(()));
    let broken = result("b_test.lua::test_broken").unwrap_err();
    assert!(broken.contains("b_test.lua:9") && broken.contains("lists: expected [1,3], got [1,2]"), "{broken}");
    assert!(result("b_test.lua::test_no_error").unwrap_err().contains("expected an error"));
    assert!(result("b_test.lua::test_false").unwrap_err().contains("it holds: expected a true value, got false"));
    assert!(result("c_test.lua").unwrap_err().contains("bad top level"));

    // What the plugin printed is kept with the test.
    let options = TestOptions { filter: Some("a_test".into()), ..TestOptions::default() };
    let a = run_tests(&dir, &options).unwrap();
    assert_eq!(a.len(), 1);
    assert!(a[0].output.iter().any(|l| l == "[walls] one wall"), "{:?}", a[0].output);

    // A filter picks tests by name.
    let picked: Vec<String> = summary(&dir, Some("test_f")).into_iter().map(|(n, _)| n).collect();
    assert_eq!(picked, ["b_test.lua::test_fails", "b_test.lua::test_false", "b_test.lua::test_first"]);
    assert!(summary(&dir, Some("nothing matches")).is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn plugin_tests_need_a_plugin_and_tests() {
    let empty = scratch("empty");
    let err = run_tests(&empty, &TestOptions::default()).unwrap_err();
    assert!(err.contains("plugin.toml"), "{err}");
    let dir = walls_plugin("no-tests", &[]);
    assert!(run_tests(&dir, &TestOptions::default()).unwrap_err().contains("tests/"));
    // A tool the plugin does not have.
    files_in(&dir, &[("tests/x_test.lua", "")]);
    let options = TestOptions { tools: vec![("walls".into(), "nope".into(), dir.join("x"))], ..Default::default() };
    assert!(run_tests(&dir, &options).unwrap_err().contains("no tool"));
    // A plugin that does not load fails its tests.
    files_in(&dir, &[("main.lua", "error('broken plugin')")]);
    let results = run_tests(&dir, &TestOptions::default()).unwrap();
    let err = results[0].result.clone().unwrap_err();
    assert!(err.contains("did not load") && err.contains("broken plugin"), "{err}");
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&empty);
}
