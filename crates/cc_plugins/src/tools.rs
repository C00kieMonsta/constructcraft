//! External programs a plugin may run (`[[tools]]` in its manifest, the
//! `tools` permission, `ccad.process.run`).
//!
//! ```toml
//! permissions = ["tools"]
//!
//! [[tools]]                                        # at most 8
//! id = "energyplus"                                # like a plugin id
//! name = "EnergyPlus"                              # may be "%key%"
//! executables = ["energyplus.exe", "energyplus"]   # the program's accepted file names
//! homepage = "https://energyplus.net"              # optional
//! version_args = ["--version"]                     # optional: what "Test" runs
//! ```
//!
//! Which program runs is the user's choice, never the plugin's: the path of
//! each tool is kept per plugin in `<plugins folder>/.tools/<id>.json`
//! (`{ "<tool id>": "<absolute path>" }`, [`read_tool_paths`],
//! [`write_tool_paths`]), or given for one run (`ccad --tool`). A path is
//! valid only when it is an existing file whose name is one of the tool's
//! `executables` ([`check_path`]; without regard to case on Windows).

use crate::process;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Most `[[tools]]` of a manifest.
pub const MAX_TOOLS: usize = 8;
/// Most file names of a tool, and longest one.
const MAX_EXECUTABLES: usize = 8;
const MAX_EXECUTABLE: usize = 64;
/// Most `version_args`, and longest one.
const MAX_VERSION_ARGS: usize = 8;
const MAX_VERSION_ARG: usize = 64;
/// Longest tool name.
const MAX_TOOL_NAME: usize = 64;
/// Longest stored path.
const MAX_PATH: usize = 4096;
/// The folder of stored tool paths, inside a plugins folder.
pub const TOOLS_DIR: &str = ".tools";
/// Largest file of stored tool paths, in bytes.
pub const MAX_TOOLS_FILE: u64 = 64 * 1024;
/// How long "Test" waits for the program.
pub const TEST_TIMEOUT: Duration = Duration::from_secs(10);
/// Lines of the program's answer "Test" shows.
const TEST_LINES: usize = 6;

/// A `[[tools]]` entry of the manifest, as written.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ToolSpec {
    /// Its id (the rule of plugin ids), which `ccad.process.run` names.
    pub id: String,
    /// Its name for people; may be `%key%`.
    pub name: String,
    /// The program's accepted file names.
    pub executables: Vec<String>,
    /// Where to get it (an `https://` link).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub homepage: String,
    /// The arguments that make it tell its version.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub version_args: Vec<String>,
}

impl ToolSpec {
    /// Check the entry.
    pub(crate) fn check(&self) -> Result<(), String> {
        if !crate::plugin::valid_id(&self.id) {
            return Err(format!(
                "id \"{}\": lowercase letters, digits and single underscores, like a plugin id",
                self.id
            ));
        }
        if self.name.trim().is_empty() || self.name.chars().count() > MAX_TOOL_NAME {
            return Err(format!("{}: name: 1 to {MAX_TOOL_NAME} characters", self.id));
        }
        if self.executables.is_empty() || self.executables.len() > MAX_EXECUTABLES {
            return Err(format!("{}: executables: 1 to {MAX_EXECUTABLES} file names", self.id));
        }
        for name in &self.executables {
            check_executable(name).map_err(|e| format!("{}: executables: \"{name}\": {e}", self.id))?;
        }
        if !self.homepage.is_empty() && !self.homepage.starts_with("https://") {
            return Err(format!("{}: homepage: an https:// link", self.id));
        }
        if self.version_args.len() > MAX_VERSION_ARGS
            || self.version_args.iter().any(|a| a.chars().count() > MAX_VERSION_ARG)
        {
            return Err(format!(
                "{}: version_args: at most {MAX_VERSION_ARGS} arguments of {MAX_VERSION_ARG} characters",
                self.id
            ));
        }
        Ok(())
    }
}

/// Check an accepted file name: a plain name (no folders), not a batch
/// file, which would run through the shell.
fn check_executable(name: &str) -> Result<(), String> {
    if name.is_empty() || name.chars().count() > MAX_EXECUTABLE {
        return Err(format!("1 to {MAX_EXECUTABLE} characters"));
    }
    if name.starts_with('.')
        || name.chars().any(|c| c.is_control() || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
    {
        return Err("a file name, without folders".into());
    }
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".bat") || lower.ends_with(".cmd") {
        return Err("batch files run through the shell, which plugins may not use".into());
    }
    Ok(())
}

/// A plugin's external program, ready to show: name in the current
/// language, and the path the user chose for it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Tool {
    pub id: String,
    pub name: String,
    pub executables: Vec<String>,
    pub homepage: String,
    pub version_args: Vec<String>,
    /// The path chosen for it, if any.
    pub path: Option<PathBuf>,
}

impl Tool {
    /// The tool of `spec`, its name resolved with `resolve`, with `path`.
    pub fn of(spec: &ToolSpec, resolve: impl Fn(&str) -> String, path: Option<PathBuf>) -> Tool {
        Tool {
            id: spec.id.clone(),
            name: resolve(&spec.name),
            executables: spec.executables.clone(),
            homepage: spec.homepage.clone(),
            version_args: spec.version_args.clone(),
            path,
        }
    }

    /// Why its path cannot run (none chosen, not a file, a name not in
    /// `executables`); `None` when it can.
    pub fn problem(&self) -> Option<String> {
        match &self.path {
            None => Some("no program chosen".into()),
            Some(p) => check_path(&self.executables, p).err(),
        }
    }
}

/// Whether `name` is one of `executables` (without regard to case on
/// Windows).
fn accepted(executables: &[String], name: &str) -> bool {
    executables.iter().any(|e| if cfg!(windows) { e.eq_ignore_ascii_case(name) } else { e == name })
}

/// Check a program's path: absolute, an existing file, named one of
/// `executables`.
pub fn check_path(executables: &[String], path: &Path) -> Result<(), String> {
    let shown = path.display();
    if !path.is_absolute() {
        return Err(format!("{shown}: not an absolute path"));
    }
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
    if !accepted(executables, name) {
        return Err(format!("{shown}: the program must be named {}", executables.join(" or ")));
    }
    if !path.is_file() {
        return Err(format!("{shown}: no such file"));
    }
    Ok(())
}

/// The first program on `PATH` named one of `executables` (a suggestion
/// for the user; never used without them choosing it).
pub fn find_on_path(executables: &[String]) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path).take(256) {
        if !dir.is_absolute() {
            continue;
        }
        for name in executables {
            let candidate = dir.join(name);
            if check_path(executables, &candidate).is_ok() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Run `tool`'s `version_args` with the program at `path` (at most
/// [`TEST_TIMEOUT`]); the first lines it wrote, or why it failed.
pub fn test(plugin: &str, tool: &Tool, path: &Path) -> Result<String, String> {
    check_path(&tool.executables, path)?;
    let done = process::run_now(plugin, path, &tool.version_args, TEST_TIMEOUT)?;
    if done.timed_out {
        return Err(format!("no answer within {} seconds", TEST_TIMEOUT.as_secs()));
    }
    let mut text = String::from_utf8_lossy(&done.stdout).into_owned();
    let errors = String::from_utf8_lossy(&done.stderr);
    if text.trim().is_empty() {
        text = errors.into_owned();
    }
    let lines: Vec<&str> = text.lines().map(str::trim_end).filter(|l| !l.trim().is_empty()).take(TEST_LINES).collect();
    let mut out = lines.join("\n");
    match done.code {
        Some(0) => {}
        Some(code) => out.push_str(&format!("\n(exit code {code})")),
        None => out.push_str("\n(no exit code)"),
    }
    Ok(out.trim().to_string())
}

/// Where the tool paths of the plugin `id` of the plugins folder `folder`
/// are kept: `<folder>/.tools/<id>.json`.
pub fn tools_path(folder: &Path, id: &str) -> PathBuf {
    folder.join(TOOLS_DIR).join(format!("{id}.json"))
}

/// The stored tool paths of the plugin `id`: a missing, unreadable, too
/// large or invalid file (or a bad id) gives none; entries whose key is not
/// a tool id or whose value is not an absolute path are left out. The paths
/// are checked against the tools when used.
pub fn read_tool_paths(folder: &Path, id: &str) -> BTreeMap<String, PathBuf> {
    if !crate::plugin::valid_id(id) {
        return BTreeMap::new();
    }
    let Ok(file) = std::fs::File::open(tools_path(folder, id)) else { return BTreeMap::new() };
    let mut bytes = Vec::new();
    if file.take(MAX_TOOLS_FILE + 1).read_to_end(&mut bytes).is_err() || bytes.len() as u64 > MAX_TOOLS_FILE {
        return BTreeMap::new();
    }
    let Ok(serde_json::Value::Object(map)) = serde_json::from_slice(&bytes) else { return BTreeMap::new() };
    map.into_iter()
        .filter(|(k, _)| crate::plugin::valid_id(k))
        .filter_map(|(k, v)| {
            let text = v.as_str()?;
            let path = PathBuf::from(text);
            (text.len() <= MAX_PATH && path.is_absolute()).then_some((k, path))
        })
        .take(MAX_TOOLS)
        .collect()
}

/// Store the tool paths of the plugin `id` (creating `.tools`), replacing the
/// file at once (a temporary file renamed over it); no paths removes it.
pub fn write_tool_paths(folder: &Path, id: &str, paths: &BTreeMap<String, PathBuf>) -> Result<(), String> {
    if !crate::plugin::valid_id(id) {
        return Err(format!("plugin id \"{id}\": not a valid id"));
    }
    let path = tools_path(folder, id);
    if paths.is_empty() {
        return match std::fs::remove_file(&path) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(format!("{}: {e}", path.display())),
            _ => Ok(()),
        };
    }
    let mut map = serde_json::Map::new();
    for (tool, p) in paths {
        let text = p.to_str().ok_or_else(|| format!("{}: not a Unicode path", p.display()))?;
        map.insert(tool.clone(), serde_json::Value::String(text.to_string()));
    }
    let text = serde_json::to_string_pretty(&map).map_err(|e| e.to_string())?;
    let dir = folder.join(TOOLS_DIR);
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let temp = dir.join(format!(".{id}.json.tmp"));
    if let Err(e) = std::fs::write(&temp, text) {
        let _ = std::fs::remove_file(&temp);
        return Err(format!("{}: {e}", temp.display()));
    }
    std::fs::rename(&temp, &path).map_err(|e| {
        let _ = std::fs::remove_file(&temp);
        format!("{}: {e}", path.display())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> ToolSpec {
        ToolSpec {
            id: "sim".into(),
            name: "Sim".into(),
            executables: vec!["sim.exe".into(), "sim".into()],
            homepage: String::new(),
            version_args: vec!["--version".into()],
        }
    }

    #[test]
    fn specs_are_checked() {
        assert!(spec().check().is_ok());
        for bad in [
            ToolSpec { id: "Bad".into(), ..spec() },
            ToolSpec { name: " ".into(), ..spec() },
            ToolSpec { executables: vec![], ..spec() },
            ToolSpec { executables: vec!["bin/sim".into()], ..spec() },
            ToolSpec { executables: vec!["C:\\sim.exe".into()], ..spec() },
            ToolSpec { executables: vec!["run.BAT".into()], ..spec() },
            ToolSpec { executables: vec!["run.cmd".into()], ..spec() },
            ToolSpec { homepage: "http://x".into(), ..spec() },
            ToolSpec { version_args: vec!["x".repeat(65)], ..spec() },
        ] {
            assert!(bad.check().is_err(), "{bad:?}");
        }
    }
}
