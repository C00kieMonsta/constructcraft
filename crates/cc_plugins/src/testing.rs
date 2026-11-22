//! Plugin tests (`ccad plugins test <folder>`): the Lua files
//! `tests/*_test.lua` of a plugin's folder, each run like a user's script
//! (full access, unrestricted) over a fresh session from the default metric
//! template with the plugin loaded, granted what its manifest asks for.
//!
//! The scripts get `ccad.test` ([`install_helpers`]): `eq(a, b, msg)` (deep
//! equality of tables), `near(a, b, tol, msg)`, `fails(fn, pattern)` and
//! `ok(v, msg)`, which raise an error naming what differed. A file that
//! defines global functions named `test_*` is a set of tests: each runs in
//! a fresh session of its own, after the file's top level (its setup) ran
//! there; a file without them is one test, its top level.

use crate::api::{Api, engine_error};
use crate::plugin::{MANIFEST, Manifest, PluginHost};
use crate::sandbox::{Limits, Sandbox};
use cc_engine::Session;
use mlua::{Lua, Table, Value as LuaValue};
use std::path::{Path, PathBuf};

/// The folder of a plugin's tests, and the end of their file names.
pub const TESTS_DIR: &str = "tests";
pub const TEST_SUFFIX: &str = "_test.lua";
/// Most test files read, and most `test_*` functions of one file.
const MAX_FILES: usize = 1000;
const MAX_FUNCTIONS: usize = 1000;
/// Largest test file, in bytes.
const MAX_FILE: u64 = 4 * 1024 * 1024;

/// `ccad.test`: assertions that raise an error at the caller's line.
const HELPERS: &str = r#"
local ccad = ...
local type, pairs, tostring, error, pcall, rawequal = type, pairs, tostring, error, pcall, rawequal
local find, format, abs = string.find, string.format, math.abs
local encode = ccad.json.encode

local function show(v)
  if type(v) == "string" then return format("%q", v) end
  if type(v) == "table" then
    local ok, text = pcall(encode, v)
    if ok then return text end
  end
  return tostring(v)
end

local function same(a, b, seen)
  if rawequal(a, b) or a == b then return true end
  if type(a) ~= "table" or type(b) ~= "table" then return false end
  if seen[a] == b then return true end
  seen[a] = b
  for k, v in pairs(a) do
    if not same(v, b[k], seen) then return false end
  end
  for k in pairs(b) do
    if a[k] == nil then return false end
  end
  return true
end

local function say(msg, text)
  if msg ~= nil then return tostring(msg) .. ": " .. text end
  return text
end

local test = {}

function test.eq(a, b, msg)
  if not same(a, b, {}) then error(say(msg, "expected " .. show(b) .. ", got " .. show(a)), 2) end
end

function test.near(a, b, tol, msg)
  tol = tol or 1e-9
  if type(a) ~= "number" or type(b) ~= "number" or not (abs(a - b) <= tol) then
    error(say(msg, "expected " .. show(b) .. " within " .. show(tol) .. ", got " .. show(a)), 2)
  end
end

function test.fails(fn, pattern)
  local ok, err = pcall(fn)
  if ok then error("expected an error, but it ran without one", 2) end
  local text = tostring(err)
  if pattern ~= nil then
    local plain = find(text, pattern, 1, true)
    local matched = plain or select(2, pcall(find, text, pattern))
    if not matched then error("the error " .. show(text) .. " does not match " .. show(pattern), 2) end
  end
  return text
end

function test.ok(v, msg)
  if not v then error(say(msg, "expected a true value, got " .. show(v)), 2) end
  return v
end

ccad.test = test
"#;

/// Add `ccad.test` to the `ccad` table `ccad` of `lua`.
pub fn install_helpers(lua: &Lua, ccad: &Table) -> mlua::Result<()> {
    lua.load(HELPERS).set_name("=ccad.test").call::<()>(ccad.clone())
}

/// How the tests run.
#[derive(Debug, Clone, Default)]
pub struct TestOptions {
    /// Run only the tests whose name holds this text.
    pub filter: Option<String>,
    /// The programs of the plugin's tools: plugin id, tool id, path.
    pub tools: Vec<(String, String, PathBuf)>,
    pub limits: Limits,
}

/// One test: its name (`<file>` or `<file>::<function>`), how it ended and
/// what it (and the plugin) printed.
#[derive(Debug, Clone, PartialEq)]
pub struct TestResult {
    pub name: String,
    pub result: Result<(), String>,
    pub output: Vec<String>,
}

/// The test files of the plugin in `dir`: `tests/*_test.lua`, by name.
pub fn test_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let tests = dir.join(TESTS_DIR);
    if !tests.is_dir() {
        return Ok(Vec::new());
    }
    let entries = std::fs::read_dir(&tests).map_err(|e| format!("{}: {e}", tests.display()))?;
    let mut files: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file() && p.file_name().is_some_and(|n| n.to_string_lossy().ends_with(TEST_SUFFIX)))
        .take(MAX_FILES)
        .collect();
    files.sort();
    Ok(files)
}

/// Run the tests of the plugin in `dir`. `Err` when they cannot start (no
/// plugin, no tests, a tool that does not fit); a plugin that does not load
/// fails each test.
pub fn run_tests(dir: &Path, options: &TestOptions) -> Result<Vec<TestResult>, String> {
    if !dir.join(MANIFEST).is_file() {
        return Err(format!("{}: no {MANIFEST} (not a plugin's folder)", dir.display()));
    }
    let manifest = Manifest::read(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for (plugin, tool, path) in &options.tools {
        if *plugin != manifest.id {
            return Err(format!("--tool {plugin}.{tool}: the plugin is {}", manifest.id));
        }
        let spec = manifest
            .tools
            .iter()
            .find(|t| t.id == *tool)
            .ok_or_else(|| format!("--tool {plugin}.{tool}: the plugin has no tool \"{tool}\""))?;
        crate::tools::check_path(&spec.executables, path).map_err(|e| format!("--tool {plugin}.{tool}: {e}"))?;
    }
    let files = test_files(dir)?;
    if files.is_empty() {
        return Err(format!("{}: no {TESTS_DIR}/*{TEST_SUFFIX} files", dir.display()));
    }
    let wanted = |name: &str| options.filter.as_deref().is_none_or(|f| name.contains(f));
    let mut results = Vec::new();
    for file in files {
        let name = file.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let source = match read_source(&file) {
            Ok(s) => s,
            Err(e) => {
                if wanted(&name) {
                    results.push(TestResult { name, result: Err(e), output: Vec::new() });
                }
                continue;
            }
        };
        let label = format!("{TESTS_DIR}/{name}");
        let (found, output) = run_once(dir, options, &source, &label, None);
        match found {
            Err(message) => {
                if wanted(&name) {
                    results.push(TestResult { name, result: Err(message), output });
                }
            }
            Ok(functions) if functions.is_empty() => {
                if wanted(&name) {
                    results.push(TestResult { name, result: Ok(()), output });
                }
            }
            Ok(functions) => {
                for function in functions {
                    let full = format!("{name}::{function}");
                    if !wanted(&full) {
                        continue;
                    }
                    let (result, output) = run_once(dir, options, &source, &label, Some(&function));
                    results.push(TestResult { name: full, result: result.map(drop), output });
                }
            }
        }
    }
    Ok(results)
}

fn read_source(file: &Path) -> Result<String, String> {
    let shown = |e: std::io::Error| format!("{}: {e}", file.display());
    let size = std::fs::metadata(file).map_err(shown)?.len();
    if size > MAX_FILE {
        return Err(format!("{}: larger than {MAX_FILE} bytes", file.display()));
    }
    std::fs::read_to_string(file).map_err(shown)
}

/// A fresh session from the default metric template with the plugin in
/// `dir` loaded, granted what its manifest asks for, with the tools of
/// `options`.
pub fn session_with_plugin(dir: &Path, options: &TestOptions) -> Result<Session, String> {
    let mut host = PluginHost::new();
    let errors = host.load_folder(dir, options.limits, |m| Some(m.requested()));
    if let Some(e) = errors.first() {
        return Err(format!("the plugin did not load: {}", e.message));
    }
    for (plugin, tool, path) in &options.tools {
        host.set_tool_path(plugin, tool, Some(path.clone()))?;
    }
    let mut session = Session::default();
    session.set_extension(Some(Box::new(host)));
    Ok(session)
}

/// Run `source` (named `label`) in a fresh session, then its function
/// `call` if given. Returns the names of its `test_*` functions (sorted;
/// none after `call`) or the error, and what was printed.
fn run_once(
    dir: &Path,
    options: &TestOptions,
    source: &str,
    label: &str,
    call: Option<&str>,
) -> (Result<Vec<String>, String>, Vec<String>) {
    let mut session = match session_with_plugin(dir, options) {
        Ok(s) => s,
        Err(e) => return (Err(e), Vec::new()),
    };
    let mut output = session.take_extension_messages();
    let sandbox = match Sandbox::new(options.limits) {
        Ok(s) => s,
        Err(e) => return (Err(engine_error(&e).message), output),
    };
    let result = run_in(&sandbox, &mut session, source, label, call).map_err(|e| engine_error(&e).message);
    output.extend(sandbox.take_output());
    output.extend(session.take_extension_messages());
    (result, output)
}

fn run_in(
    sandbox: &Sandbox,
    session: &mut Session,
    source: &str,
    label: &str,
    call: Option<&str>,
) -> mlua::Result<Vec<String>> {
    let lua = &sandbox.lua;
    let api = Api::install(lua)?;
    api.ccad.raw_set("args", lua.create_table()?)?;
    install_helpers(lua, &api.ccad)?;
    let chunk = lua.load(source).set_name(format!("@{label}")).set_mode(mlua::ChunkMode::Text).into_function()?;
    sandbox.reset_budget();
    api.with_session(lua, session, None, |lua| {
        chunk.call::<()>(())?;
        if let Some(name) = call {
            let LuaValue::Function(f) = lua.globals().raw_get::<LuaValue>(name)? else {
                return Err(mlua::Error::runtime(format!("{name} is no longer a function")));
            };
            sandbox.reset_budget();
            f.call::<()>(())?;
            return Ok(Vec::new());
        }
        let mut names = Vec::new();
        for pair in lua.globals().pairs::<LuaValue, LuaValue>() {
            let (k, v) = pair?;
            if let (LuaValue::String(k), LuaValue::Function(_)) = (k, v) {
                let k = k.to_string_lossy();
                if k.starts_with("test_") {
                    names.push(k);
                }
            }
        }
        names.sort();
        names.truncate(MAX_FUNCTIONS);
        Ok(names)
    })
}
