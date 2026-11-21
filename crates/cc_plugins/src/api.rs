//! The `ccad` table scripts and plugins see.
//!
//! - `ccad.run(name, args)`: run an engine command; returns its result, or
//!   raises an error.
//! - `ccad.try(name, args)`: the same, returning `result` or `nil, err`
//!   (`err.code`, `err.message`) instead of raising.
//! - `ccad.commands()`: the commands (name, kind, group, description);
//!   `ccad.describe(name)` one of them with its parameters' `schema`.
//! - `ccad.null` (JSON null), `ccad.array(t)` (marks `t`, even empty, as a
//!   list), `ccad.json.encode(v)` / `ccad.json.decode(s)`, `ccad.version`,
//!   `ccad.api` (the plugin API version).
//! - In plugins only (see [`crate::plugin`]): `ccad.plugin`, `ccad.command`,
//!   `ccad.settings`, `ccad.data` and `ccad.file`.
//!
//! What a plugin command may do is not decided here: it runs inside a
//! [`cc_engine::PluginScope`], which [`Session::execute`] checks for every
//! command run during it (also by graphs, scripts or macros it starts). The
//! user's scripts run unrestricted.
//!
//! `ccad.file.read(path)` and `ccad.file.write(path, data)` (plugins only,
//! while a command runs) read and write the files the command's arguments
//! grant ([`FileRun`]): an `open_path` parameter's file to read, a
//! `save_path` parameter's file to write.
//!
//! The functions that reach the session exist only while a script or a
//! plugin command runs: the Lua wrappers stay (a plugin may keep them in
//! locals when it loads) and call the session's functions through a private
//! table filled for the run.

use crate::convert::{from_lua, to_lua};
use cc_engine::{EngineError, ErrorCode, Grants, Session};
use mlua::{Lua, LuaSerdeExt, Table, Value as LuaValue};
use serde_json::Value;
use std::cell::RefCell;
use std::io::Read;
use std::path::{Path, PathBuf};

/// The plugin API version this build provides (`api` in `plugin.toml`).
pub const API_VERSION: u32 = 2;

/// Makes the wrappers over the private table of the run's functions;
/// returns its `get`.
const PRELUDE: &str = r#"
local ccad, impl = ...
local error = error
local function get(name, shown)
  local f = impl[name]
  if not f then error("ccad." .. (shown or name) .. " is only available while a script or plugin command runs", 3) end
  return f
end
function ccad.run(name, args) return get("run")(name, args) end
function ccad.try(name, args) return get("try")(name, args) end
function ccad.commands() return get("commands")() end
function ccad.describe(name) return get("describe")(name) end
return get
"#;

/// `ccad.file`, for plugins.
const FILE_PRELUDE: &str = r#"
local ccad, get = ...
local file = {}
function file.read(path) return get("file_read", "file.read")(path) end
function file.write(path, data) return get("file_write", "file.write")(path, data) end
ccad.file = file
"#;

/// Largest file `ccad.file.read` reads, in bytes.
pub const MAX_FILE_READ: u64 = 64 * 1024 * 1024;
/// Most bytes `ccad.file.write` takes in one command run, all files together.
pub const MAX_FILE_WRITTEN: u64 = 256 * 1024 * 1024;

/// The files a plugin command's arguments grant, and what `ccad.file.write`
/// gave for them: kept in memory and written ([`FileRun::commit`]) only
/// when the command succeeds.
#[derive(Debug, Default)]
pub struct FileRun {
    grants: Grants,
    /// Each written file's key ([`cc_engine::files::write_key`]) and its
    /// content (a later write to the same file replaces it).
    pending: Vec<(PathBuf, Vec<u8>)>,
    /// Bytes given to `ccad.file.write` so far.
    written: u64,
}

impl FileRun {
    pub fn new(grants: Grants) -> FileRun {
        FileRun { grants, pending: Vec::new(), written: 0 }
    }

    /// Read a granted file to read.
    fn read(&self, path: &Path) -> Result<Vec<u8>, EngineError> {
        if !self.grants.may_read(path) {
            return Err(EngineError::rejected(format!(
                "ccad.file.read: {} is not a file to read given to this command (an open_path parameter)",
                path.display()
            )));
        }
        let io = |e: std::io::Error| EngineError::io(format!("{}: {e}", path.display()));
        let too_large = || {
            EngineError::invalid(format!(
                "ccad.file.read: {} is larger than {} MiB",
                path.display(),
                MAX_FILE_READ >> 20
            ))
        };
        let file = std::fs::File::open(path).map_err(io)?;
        if file.metadata().map_err(io)?.len() > MAX_FILE_READ {
            return Err(too_large());
        }
        let mut bytes = Vec::new();
        file.take(MAX_FILE_READ + 1).read_to_end(&mut bytes).map_err(io)?;
        if bytes.len() as u64 > MAX_FILE_READ {
            return Err(too_large());
        }
        Ok(bytes)
    }

    /// Keep `bytes` to write to a granted file to write.
    fn write(&mut self, path: &Path, bytes: Vec<u8>) -> Result<(), EngineError> {
        if !self.grants.may_write(path) {
            return Err(EngineError::rejected(format!(
                "ccad.file.write: {} is not a file to write given to this command (a save_path parameter)",
                path.display()
            )));
        }
        let key = cc_engine::files::write_key(path).map_err(EngineError::invalid)?;
        let total = self.written.saturating_add(bytes.len() as u64);
        if total > MAX_FILE_WRITTEN {
            return Err(too_much_written());
        }
        self.written = total;
        match self.pending.iter_mut().find(|(k, _)| *k == key) {
            Some((_, old)) => *old = bytes,
            None => self.pending.push((key, bytes)),
        }
        Ok(())
    }

    /// Write the files kept: each to a temporary file in its folder first,
    /// then all renamed into place, so a failure while writing changes none
    /// of them.
    pub fn commit(self) -> Result<(), EngineError> {
        let mut temps: Vec<(PathBuf, &PathBuf)> = Vec::new();
        let clean = |temps: &[(PathBuf, &PathBuf)]| {
            for (temp, _) in temps {
                let _ = std::fs::remove_file(temp);
            }
        };
        for (i, (key, bytes)) in self.pending.iter().enumerate() {
            let name = key.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            let temp = key.with_file_name(format!(".{name}.{}-{i}.tmp", std::process::id()));
            if let Err(e) = std::fs::write(&temp, bytes) {
                let _ = std::fs::remove_file(&temp);
                clean(&temps);
                return Err(EngineError::io(format!("{}: {e}", key.display())));
            }
            temps.push((temp, key));
        }
        for (n, (temp, key)) in temps.iter().enumerate() {
            if let Err(e) = std::fs::rename(temp, key) {
                clean(temps.get(n..).unwrap_or_default());
                return Err(EngineError::io(format!("{}: {e}", key.display())));
            }
        }
        Ok(())
    }
}

fn too_much_written() -> EngineError {
    EngineError::invalid(format!("ccad.file.write: a command writes at most {} MiB", MAX_FILE_WRITTEN >> 20))
}

/// The bytes of `ccad.file.write`'s data: a string, or a list of strings
/// joined.
fn data_bytes(data: &LuaValue) -> mlua::Result<Vec<u8>> {
    match data {
        LuaValue::String(s) => Ok(s.as_bytes().to_vec()),
        LuaValue::Table(t) => {
            let mut bytes = Vec::new();
            for part in t.sequence_values::<LuaValue>() {
                match part? {
                    LuaValue::String(s) => {
                        let part = s.as_bytes();
                        if (bytes.len() as u64).saturating_add(part.len() as u64) > MAX_FILE_WRITTEN {
                            return Err(mlua::Error::external(too_much_written()));
                        }
                        bytes.extend_from_slice(&part);
                    }
                    other => {
                        return Err(mlua::Error::runtime(format!(
                            "ccad.file.write: the list holds a {}, not a string",
                            other.type_name()
                        )));
                    }
                }
            }
            Ok(bytes)
        }
        other => Err(mlua::Error::runtime(format!(
            "ccad.file.write: the data is a {}, not a string or a list of strings",
            other.type_name()
        ))),
    }
}

/// The `ccad` table and the private table of the run's functions.
pub struct Api {
    pub ccad: Table,
    imp: Table,
    /// The prelude's `get` (reads the private table), for `ccad.file`.
    get: mlua::Function,
}

impl Api {
    /// Make the `ccad` global in `lua` for a plugin: also `ccad.file`.
    pub fn install_for_plugin(lua: &Lua) -> mlua::Result<Api> {
        let api = Api::install(lua)?;
        lua.load(FILE_PRELUDE).set_name("=ccad.file").call::<()>((api.ccad.clone(), api.get.clone()))?;
        Ok(api)
    }

    /// Make the `ccad` global in `lua`.
    pub fn install(lua: &Lua) -> mlua::Result<Api> {
        let ccad = lua.create_table()?;
        let imp = lua.create_table()?;
        ccad.raw_set("null", LuaValue::NULL)?;
        ccad.raw_set("version", env!("CARGO_PKG_VERSION"))?;
        ccad.raw_set("api", API_VERSION)?;
        ccad.raw_set(
            "array",
            lua.create_function(|lua, t: Option<Table>| {
                let t = match t {
                    Some(t) => t,
                    None => lua.create_table()?,
                };
                t.set_metatable(Some(lua.array_metatable()))?;
                Ok(t)
            })?,
        )?;
        let json = lua.create_table()?;
        json.raw_set(
            "encode",
            lua.create_function(|lua, v: LuaValue| {
                let value = from_lua(lua, &v).map_err(mlua::Error::runtime)?;
                Ok(serde_json::to_string(&value).unwrap_or_default())
            })?,
        )?;
        json.raw_set(
            "decode",
            lua.create_function(|lua, s: mlua::String| {
                let value: Value = serde_json::from_slice(&s.as_bytes()).map_err(mlua::Error::runtime)?;
                to_lua(lua, &value)
            })?,
        )?;
        ccad.raw_set("json", json)?;
        let get: mlua::Function = lua.load(PRELUDE).set_name("=ccad").call((ccad.clone(), imp.clone()))?;
        lua.globals().raw_set("ccad", ccad.clone())?;
        Ok(Api { ccad, imp, get })
    }

    /// Run `f` with the session's functions available to the Lua code, and
    /// `ccad.file` working on `files` (a plugin command's granted files).
    pub fn with_session<R>(
        &self,
        lua: &Lua,
        session: &mut Session,
        files: Option<&RefCell<FileRun>>,
        f: impl FnOnce(&Lua) -> mlua::Result<R>,
    ) -> mlua::Result<R> {
        let cell = RefCell::new(session);
        let cell = &cell;
        let result = lua.scope(|scope| {
            let run = scope.create_function(move |lua, (name, args): (String, LuaValue)| {
                let mut session = cell.try_borrow_mut().map_err(mlua::Error::runtime)?;
                match execute(lua, &mut session, &name, &args)? {
                    Ok(v) => to_lua(lua, &v),
                    Err(e) => Err(mlua::Error::external(e)),
                }
            })?;
            let try_run = scope.create_function(move |lua, (name, args): (String, LuaValue)| {
                let mut session = cell.try_borrow_mut().map_err(mlua::Error::runtime)?;
                match execute(lua, &mut session, &name, &args)? {
                    Ok(v) => Ok((to_lua(lua, &v)?, LuaValue::Nil)),
                    Err(e) => {
                        let err = lua.create_table()?;
                        err.raw_set("code", code_name(e.code))?;
                        err.raw_set("message", e.message)?;
                        Ok((LuaValue::Nil, LuaValue::Table(err)))
                    }
                }
            })?;
            let commands = scope.create_function(move |lua, ()| {
                let session = cell.try_borrow().map_err(mlua::Error::runtime)?;
                let list: Vec<Value> = session
                    .commands()
                    .into_iter()
                    .map(|c| {
                        serde_json::json!({ "name": c.name, "kind": c.kind, "group": c.group, "description": c.description })
                    })
                    .collect();
                to_lua(lua, &Value::Array(list))
            })?;
            let describe = scope.create_function(move |lua, name: String| {
                let session = cell.try_borrow().map_err(mlua::Error::runtime)?;
                match session.describe(&name) {
                    Some(c) => to_lua(
                        lua,
                        &serde_json::json!({
                            "name": c.name, "kind": c.kind, "group": c.group,
                            "description": c.description, "schema": c.input_schema,
                        }),
                    ),
                    None => Ok(LuaValue::Nil),
                }
            })?;
            self.imp.raw_set("run", run)?;
            self.imp.raw_set("try", try_run)?;
            self.imp.raw_set("commands", commands)?;
            self.imp.raw_set("describe", describe)?;
            if let Some(files) = files {
                let read = scope.create_function(move |lua, path: String| {
                    let run = files.try_borrow().map_err(mlua::Error::runtime)?;
                    let bytes = run.read(Path::new(&path)).map_err(mlua::Error::external)?;
                    lua.create_string(&bytes)
                })?;
                let write = scope.create_function(move |_, (path, data): (String, LuaValue)| {
                    let bytes = data_bytes(&data)?;
                    let mut run = files.try_borrow_mut().map_err(mlua::Error::runtime)?;
                    run.write(Path::new(&path), bytes).map_err(mlua::Error::external)
                })?;
                self.imp.raw_set("file_read", read)?;
                self.imp.raw_set("file_write", write)?;
            }
            let result = f(lua);
            self.imp.clear()?;
            result
        });
        // Leave nothing that points at the finished scope, even after an error.
        let _ = self.imp.clear();
        result
    }
}

/// Run one command for the Lua code: `Err` for errors of the Lua side (bad
/// arguments), `Ok(Err)` for the command's own failure. What a plugin
/// command may run is checked by the session (its scope).
fn execute(lua: &Lua, session: &mut Session, name: &str, args: &LuaValue) -> mlua::Result<Result<Value, EngineError>> {
    let args = match args {
        LuaValue::Nil => Value::Object(Default::default()),
        other => from_lua(lua, other).map_err(|e| mlua::Error::runtime(format!("{name}: arguments: {e}")))?,
    };
    Ok(session.execute(name, args))
}

/// An error code as its JSON name.
pub fn code_name(code: ErrorCode) -> String {
    serde_json::to_value(code).ok().and_then(|v| v.as_str().map(str::to_string)).unwrap_or_default()
}

/// Longest error message kept from a Lua error (tracebacks can be long).
const MAX_MESSAGE: usize = 4000;

/// A Lua error as an engine error: a command's error raised through
/// `ccad.run` keeps its code; anything else is a script error.
pub fn engine_error(e: &mlua::Error) -> EngineError {
    fn find(e: &mlua::Error) -> Option<EngineError> {
        match e {
            mlua::Error::CallbackError { cause, .. } | mlua::Error::WithContext { cause, .. } => find(cause),
            mlua::Error::ExternalError(x) => x.downcast_ref::<EngineError>().cloned(),
            _ => None,
        }
    }
    if let Some(found) = find(e) {
        return found;
    }
    let mut message = match e {
        mlua::Error::RuntimeError(m) | mlua::Error::SyntaxError { message: m, .. } => m.clone(),
        mlua::Error::MemoryError(_) => "the script used too much memory".to_string(),
        other => other.to_string(),
    };
    // The message, without the stack traceback Lua appends.
    if let Some(cut) = message.find("\nstack traceback:") {
        message.truncate(cut);
    }
    if message.len() > MAX_MESSAGE {
        let mut cut = MAX_MESSAGE;
        while !message.is_char_boundary(cut) {
            cut -= 1;
        }
        message.truncate(cut);
        message.push('…');
    }
    EngineError::new(ErrorCode::Script, message)
}
