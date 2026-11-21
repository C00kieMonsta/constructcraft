//! Plugins: a folder with a `plugin.toml` manifest and Lua code that adds
//! commands to the session.
//!
//! ```toml
//! id = "room_tools"        # lowercase letters, digits and _, no "__"
//! name = "Room tools"      # may be "%key%" (api 2, see the locales)
//! version = "0.1.0"
//! api = 2                  # the plugin API it was written for (1 or 2)
//! description = "…"
//! authors = ["…"]
//! license = "MIT"
//! homepage = "https://…"
//! main = "main.lua"        # the default
//! permissions = ["files"]  # optional: run commands that read or write files
//!
//! [[buttons]]              # api 2: a ribbon button (at most 64)
//! command = "walls"        # a command main.lua registers
//! label = "Walls"          # at most 64 characters, may be "%key%"
//! icon = "icons/walls.svg" # optional, an SVG inside the folder, at most 64 KiB
//! panel = "Rooms"          # optional (default: the plugin's name), may be "%key%"
//! size = "large"           # or "small"
//! when = "selection:wall"  # optional: selection, selection:one,
//!                          # selection:<category>, view:<kind>; comma-separated, all hold
//!
//! [settings.height]        # api 2: a setting (at most 64), key like a command name
//! type = "number"          # boolean, number, integer or string
//! default = 2.8
//! title = "%settings.height%"
//! ui = "length"            # numbers only
//! minimum = 0.5            # numbers and integers; also maximum
//! # enum = ["a", "b"] and enum_labels = […] for strings
//!
//! [[exporters]]            # api 2: an entry of the app's Export list (at most 16)
//! command = "export_idf"   # a command with exactly one ui = "save_path" parameter
//! label = "%export.idf%"   # at most 64 characters, may be "%key%"
//!
//! [[importers]]            # api 2: an entry of the app's Import list (at most 16)
//! command = "import_epw"   # a command with exactly one ui = "open_path" parameter
//! label = "EnergyPlus weather (.epw)"
//! ```
//!
//! `main.lua` registers commands with `ccad.command { name, title,
//! description, kind, params, run }` while it loads; `require("a.b")` loads
//! `a/b.lua` from the plugin's folder. A command `walls` of the plugin
//! `room_tools` is `room_tools__walls` in the catalog. Each plugin has its
//! own sandboxed Lua state.
//!
//! Every plugin gets `ccad.settings`, the current values of its settings
//! (stored over defaults; read-only, present while `main.lua` runs and
//! refreshed before each command), and `ccad.data.get(element)`,
//! `ccad.data.set(element, value)` and `ccad.data.list()`, its own data in
//! the project (`element` nil: the project's) through
//! `plugin_data_get`/`set`/`list`; a plugin command cannot run those
//! commands for another plugin's id. With api 2 a plugin also has:
//! - buttons and settings in its manifest (refused under api 1, so the index
//!   never offers such a plugin to an older build);
//! - translations in `locales/<code>.toml` ([`crate::locales`]) of its name,
//!   description, buttons, settings and commands (`title`, `description`,
//!   and the parameters' `title`, `description` and `enum_labels`), looked
//!   up in the current language each time they are listed;
//! - UI hints on command parameters, checked when a command registers: `ui`
//!   (`length` on a number, `element`/`level`/`view`/`type` on an integer,
//!   `elements`/`point` on an array, `material`, `save_path` and
//!   `open_path` on a string), `category` (with `element`, `elements` or
//!   `type`), `extensions` (with `save_path` or `open_path`), `enum_labels`
//!   (one per `enum` value) and `title`;
//! - files without the `files` permission: the path a command's
//!   `save_path` argument names may be written, and an `open_path` one read,
//!   during that run only (by `ccad.file` or by the engine's own exports and
//!   imports), after its extension is checked against `extensions`; see
//!   [`cc_engine::files`]. Files `ccad.file.write` writes are kept in memory
//!   and written (each to a temporary file renamed into place) only when the
//!   command succeeds; a file an engine command (`ifc_export`…) wrote during
//!   a command that then fails stays.
//!
//! Every command a plugin command runs, also through `graph_run`, scripts
//! or macros, is checked by the engine against the command's
//! [`cc_engine::PluginScope`]: only its own plugin data, no `project_new`,
//! `project_open`, `undo`, `redo` or `language_set`, no edits from a query
//! command, and only its granted files (all files with `files`).
//!
//! Settings are kept per plugin in `<plugins folder>/.settings/<id>.json`
//! ([`crate::store::read_settings`]), read by [`PluginHost::load_folder`];
//! a plugin loaded from its own folder (not a folder of plugins) starts
//! with the defaults.

use crate::api::FileRun;
use crate::api::{API_VERSION, Api, engine_error};
use crate::contributions::{
    Button, ButtonSize, ButtonSpec, FileAction, FileActionSpec, MAX_BUTTONS, MAX_FILE_ACTIONS, MAX_ICON, MAX_SETTINGS,
    Setting, SettingSpec, When, check_param_hints, current_values, path_params, resolve_schema, settings_json,
};
use crate::convert::{from_lua, to_lua};
use crate::locales::Locales;
use crate::sandbox::{Limits, Sandbox};
use cc_engine::extension::SEPARATOR;
use cc_engine::{CommandInfo, CommandKind, EngineError, Extension, FileAccess, Grants, PluginScope, Session};
use mlua::{Function, Lua, Table, Value as LuaValue};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value, json};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// The manifest's file name.
pub const MANIFEST: &str = "plugin.toml";
/// Largest manifest, and largest Lua file of a plugin, in bytes.
const MAX_MANIFEST: u64 = 64 * 1024;
const MAX_SOURCE: u64 = 4 * 1024 * 1024;
/// Most commands one plugin registers, and most plugins in a folder.
const MAX_COMMANDS: usize = 256;
const MAX_PLUGINS: usize = 256;
/// Longest full command name (a model tool name).
const MAX_NAME: usize = 64;

/// A plugin's `plugin.toml`.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Manifest {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default = "api_one")]
    pub api: u32,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub homepage: String,
    #[serde(default = "main_lua")]
    pub main: String,
    #[serde(default)]
    pub permissions: Vec<String>,
    /// Ribbon buttons (api 2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub buttons: Vec<ButtonSpec>,
    /// Settings (api 2): the `[settings.<key>]` tables, in the manifest's
    /// order.
    #[serde(
        default,
        deserialize_with = "settings_in_order",
        serialize_with = "settings_as_table",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub settings: Vec<(String, SettingSpec)>,
    /// Entries of the app's Export list (api 2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exporters: Vec<FileActionSpec>,
    /// Entries of the app's Import list (api 2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub importers: Vec<FileActionSpec>,
}

/// The `[settings.*]` tables in the order the manifest writes them.
fn settings_in_order<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<(String, SettingSpec)>, D::Error> {
    let map: BTreeMap<String, toml::Spanned<SettingSpec>> = Deserialize::deserialize(d)?;
    let mut list: Vec<(usize, String, SettingSpec)> =
        map.into_iter().map(|(k, v)| (v.span().start, k, v.into_inner())).collect();
    list.sort_by_key(|(start, ..)| *start);
    Ok(list.into_iter().map(|(_, k, v)| (k, v)).collect())
}

fn settings_as_table<S: Serializer>(settings: &[(String, SettingSpec)], s: S) -> Result<S::Ok, S::Error> {
    s.collect_map(settings.iter().map(|(k, v)| (k, v)))
}

fn api_one() -> u32 {
    1
}

fn main_lua() -> String {
    "main.lua".into()
}

/// What a plugin may do beyond querying and editing the model.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Permissions {
    /// Run commands that read or write files (import, export, save).
    pub files: bool,
}

impl Permissions {
    pub const NONE: Permissions = Permissions { files: false };

    fn and(self, other: Permissions) -> Permissions {
        Permissions { files: self.files && other.files }
    }
}

/// A plugin that could not be loaded, or a plugin command refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginError {
    /// The plugin's folder.
    pub dir: PathBuf,
    pub message: String,
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.dir.display(), self.message)
    }
}

impl std::error::Error for PluginError {}

/// Whether `s` is a valid command name: a lowercase letter, then lowercase
/// letters, digits and single underscores, not ending in one (the rule of
/// [`cc_core::plugin_data::valid_plugin_name`]).
pub(crate) fn valid_command_name(s: &str) -> bool {
    cc_core::plugin_data::valid_plugin_name(s) && !s.contains(SEPARATOR)
}

/// Whether `s` is a valid plugin id (see [`Manifest`]): the rule of
/// [`cc_core::plugin_data::valid_plugin_id`], which keys plugin data.
pub fn valid_id(s: &str) -> bool {
    cc_core::plugin_data::valid_plugin_id(s) && !s.contains(SEPARATOR)
}

impl Manifest {
    /// Read and check a manifest.
    pub fn parse(text: &str) -> Result<Manifest, String> {
        let m: Manifest = toml::from_str(text).map_err(|e| format!("{MANIFEST}: {e}"))?;
        m.check()?;
        Ok(m)
    }

    fn check(&self) -> Result<(), String> {
        if !valid_id(&self.id) {
            return Err(format!(
                "id \"{}\": up to 32 lowercase letters, digits and single underscores, starting with a letter",
                self.id
            ));
        }
        if self.name.trim().is_empty() || self.name.len() > 100 {
            return Err("name: 1 to 100 characters".into());
        }
        if self.version.trim().is_empty() || self.version.len() > 32 {
            return Err("version: 1 to 32 characters".into());
        }
        if self.api == 0 || self.api > API_VERSION {
            return Err(format!("api {}: this version of ConstructCraft provides plugin API {API_VERSION}", self.api));
        }
        if module_path(&self.main).is_none() {
            return Err(format!("main \"{}\": a .lua file inside the plugin's folder", self.main));
        }
        for p in &self.permissions {
            if p != "files" {
                return Err(format!("unknown permission \"{p}\" (known: files)"));
            }
        }
        if self.api < 2 && !self.buttons.is_empty() {
            return Err("buttons need api = 2".into());
        }
        if self.api < 2 && !self.settings.is_empty() {
            return Err("settings need api = 2".into());
        }
        if self.buttons.len() > MAX_BUTTONS {
            return Err(format!("at most {MAX_BUTTONS} buttons"));
        }
        for (i, b) in self.buttons.iter().enumerate() {
            b.check().map_err(|e| format!("buttons[{}]: {e}", i + 1))?;
        }
        if self.settings.len() > MAX_SETTINGS {
            return Err(format!("at most {MAX_SETTINGS} settings"));
        }
        self.setting_list()?;
        for (key, list) in [("exporters", &self.exporters), ("importers", &self.importers)] {
            if self.api < 2 && !list.is_empty() {
                return Err(format!("{key} need api = 2"));
            }
            if list.len() > MAX_FILE_ACTIONS {
                return Err(format!("at most {MAX_FILE_ACTIONS} {key}"));
            }
            for (i, a) in list.iter().enumerate() {
                a.check().map_err(|e| format!("{key}[{}]: {e}", i + 1))?;
            }
        }
        Ok(())
    }

    /// Read and check the manifest of the plugin in `dir`.
    pub fn read(dir: &Path) -> Result<Manifest, String> {
        Manifest::parse(&read_inside(dir, Path::new(MANIFEST), MAX_MANIFEST)?)
    }

    /// The permissions it asks for.
    pub fn requested(&self) -> Permissions {
        Permissions { files: self.permissions.iter().any(|p| p == "files") }
    }

    /// Its settings, checked, with their text as written (`%key%` not
    /// looked up).
    pub fn setting_list(&self) -> Result<Vec<Setting>, String> {
        self.settings.iter().map(|(key, spec)| spec.setting(key)).collect()
    }
}

/// The manifest and settings of the plugin in `dir`, the settings' text in
/// the current language, without running its code (to show or change
/// settings of a plugin that is not loaded).
pub fn settings_schema(dir: &Path) -> Result<(Manifest, Vec<Setting>), String> {
    let manifest = Manifest::read(dir)?;
    let settings = manifest.setting_list()?;
    let settings = if manifest.api >= 2 {
        let locales = Locales::load(dir)?;
        settings.iter().map(|s| s.resolved(|t| locales.resolve(t))).collect()
    } else {
        settings
    };
    Ok((manifest, settings))
}

/// A relative path made of plain names (letters, digits, `_`, `-`), ending
/// in `.lua`: no `..`, no absolute paths, no drive letters.
fn module_path(rel: &str) -> Option<PathBuf> {
    plain_path(rel, "lua")
}

/// A relative path made of plain names (letters, digits, `_`, `-`) joined
/// by `/`, ending in `.<ext>`.
pub(crate) fn plain_path(rel: &str, ext: &str) -> Option<PathBuf> {
    let stem = rel.strip_suffix(ext)?.strip_suffix('.')?;
    let mut path = PathBuf::new();
    for part in stem.split('/') {
        if part.is_empty() || !part.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            return None;
        }
        path.push(part);
    }
    path.set_extension(ext);
    Some(path)
}

/// The canonical path of a file of the plugin, refusing ones that are
/// outside its folder (through links), not files, or larger than `max`.
fn file_inside(dir: &Path, rel: &Path, max: u64) -> Result<PathBuf, String> {
    let shown = rel.display();
    let root = dir.canonicalize().map_err(|e| format!("{}: {e}", dir.display()))?;
    let real = dir.join(rel).canonicalize().map_err(|e| format!("{shown}: {e}"))?;
    if !real.starts_with(&root) {
        return Err(format!("{shown} is outside the plugin's folder"));
    }
    let meta = std::fs::metadata(&real).map_err(|e| format!("{shown}: {e}"))?;
    if !meta.is_file() {
        return Err(format!("{shown} is not a file"));
    }
    if meta.len() > max {
        return Err(format!("{shown} is larger than {max} bytes"));
    }
    Ok(real)
}

/// Read a file of the plugin, refusing large ones and ones outside its folder
/// (through links).
pub(crate) fn read_inside(dir: &Path, rel: &Path, max: u64) -> Result<String, String> {
    let real = file_inside(dir, rel, max)?;
    std::fs::read_to_string(&real).map_err(|e| format!("{}: {e}", rel.display()))
}

/// A command a plugin registered, its text as written.
struct PluginCommand {
    short: String,
    title: Option<String>,
    /// The catalog entry; `description` and the schema's text as written.
    info: CommandInfo,
    required: Vec<String>,
    func: Function,
}

/// A button of the manifest, checked.
#[derive(Debug, Clone)]
struct ButtonDef {
    command: String,
    label: String,
    panel: Option<String>,
    icon: Option<PathBuf>,
    size: ButtonSize,
    when: Vec<When>,
}

/// An exporter or importer of the manifest, checked against its command.
#[derive(Debug, Clone)]
struct FileActionDef {
    command: String,
    label: String,
    param: String,
    extensions: Vec<String>,
}

/// A loaded plugin.
pub struct Plugin {
    pub manifest: Manifest,
    pub dir: PathBuf,
    granted: Permissions,
    sandbox: Sandbox,
    api: Api,
    commands: Vec<PluginCommand>,
    /// Its translations (`None` under api 1: text is shown as written).
    locales: Option<Locales>,
    buttons: Vec<ButtonDef>,
    /// Its settings, text as written.
    settings: Vec<Setting>,
    exporters: Vec<FileActionDef>,
    importers: Vec<FileActionDef>,
    /// The current value of each setting.
    values: Map<String, Value>,
    /// The metatable of `ccad.settings`, whose `__index` holds the values.
    settings_meta: Table,
    /// What it printed while loading, until a host takes it.
    printed: Vec<String>,
}

impl std::fmt::Debug for Plugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Plugin").field("id", &self.manifest.id).field("dir", &self.dir).finish_non_exhaustive()
    }
}

/// Makes `ccad.settings` read-only: the values live in the metatable's
/// `__index`, which the host replaces before each command.
const SETTINGS_PRELUDE: &str = r#"
local mt = ...
local next, error = next, error
mt.__index = {}
mt.__newindex = function() error("ccad.settings is read-only", 2) end
mt.__pairs = function() return next, mt.__index, nil end
mt.__metatable = "ccad.settings"
"#;

/// `ccad.data`: the plugin's own data in the project.
const DATA_PRELUDE: &str = r#"
local ccad, id = ...
local run, null = ccad.run, ccad.null
local data = {}
function data.get(element) return run("plugin_data_get", { plugin = id, element = element }) end
function data.set(element, value)
  if value == nil then value = null end
  return run("plugin_data_set", { plugin = id, element = element, data = value })
end
function data.list() return run("plugin_data_list", { plugin = id }) end
ccad.data = data
"#;

/// Put `values` in `ccad.settings` (through its metatable).
fn fill_settings(lua: &Lua, meta: &Table, values: &Map<String, Value>) -> mlua::Result<()> {
    let inner = lua.create_table()?;
    for (k, v) in values {
        inner.raw_set(k.as_str(), to_lua(lua, v)?)?;
    }
    meta.raw_set("__index", inner)
}

impl Plugin {
    /// Load the plugin in `dir`: read its manifest, run its main file and
    /// keep the commands it registers. It is granted no permission yet, and
    /// its settings have their defaults.
    pub fn load(dir: &Path, limits: Limits) -> Result<Plugin, PluginError> {
        Plugin::load_with_settings(dir, limits, &Map::new())
    }

    /// [`Plugin::load`] with stored setting values (see
    /// [`crate::store::read_settings`]): those that fit their setting are
    /// kept, the others (and unknown keys) dropped.
    pub fn load_with_settings(dir: &Path, limits: Limits, stored: &Map<String, Value>) -> Result<Plugin, PluginError> {
        let fail = |message: String| PluginError { dir: dir.to_path_buf(), message };
        let manifest =
            Manifest::parse(&read_inside(dir, Path::new(MANIFEST), MAX_MANIFEST).map_err(fail)?).map_err(fail)?;
        let main = module_path(&manifest.main).ok_or_else(|| fail("bad main".into()))?;
        let source = read_inside(dir, &main, MAX_SOURCE).map_err(fail)?;
        let locales = if manifest.api >= 2 { Some(Locales::load(dir).map_err(fail)?) } else { None };
        let buttons = buttons(dir, &manifest).map_err(fail)?;
        let settings = manifest.setting_list().map_err(fail)?;
        let values = current_values(&settings, stored);
        let sandbox = Sandbox::new(limits).map_err(|e| fail(engine_error(&e).message))?;
        let (api, commands, settings_meta) =
            setup(&sandbox, dir, &manifest, &source, &values).map_err(|e| fail(engine_error(&e).message))?;
        if let Some(b) = buttons.iter().find(|b| !commands.iter().any(|c| c.short == b.command)) {
            return Err(fail(format!("a button runs the command \"{}\", which main.lua does not register", b.command)));
        }
        let exporters = file_actions(&manifest.exporters, &commands, "exporters", "save_path").map_err(fail)?;
        let importers = file_actions(&manifest.importers, &commands, "importers", "open_path").map_err(fail)?;
        let printed = sandbox.take_output();
        Ok(Plugin {
            manifest,
            dir: dir.to_path_buf(),
            granted: Permissions::NONE,
            sandbox,
            api,
            commands,
            locales,
            buttons,
            settings,
            exporters,
            importers,
            values,
            settings_meta,
            printed,
        })
    }

    /// Grant permissions (only those the manifest asks for are kept).
    pub fn grant(&mut self, permissions: Permissions) {
        self.granted = permissions.and(self.manifest.requested());
    }

    pub fn granted(&self) -> Permissions {
        self.granted
    }

    /// `text` in the current language when it is a `%key%` of its locales
    /// (api 2); as written otherwise.
    pub fn resolve(&self, text: &str) -> String {
        match &self.locales {
            Some(l) => l.resolve(text),
            None => text.to_string(),
        }
    }

    /// Its name, in the current language.
    pub fn display_name(&self) -> String {
        self.resolve(&self.manifest.name)
    }

    /// Its description, in the current language.
    pub fn display_description(&self) -> String {
        self.resolve(&self.manifest.description)
    }

    /// Its commands' catalog entries (full names), text in the current
    /// language.
    pub fn commands(&self) -> impl Iterator<Item = CommandInfo> + '_ {
        self.commands.iter().map(|c| self.info(c))
    }

    fn info(&self, c: &PluginCommand) -> CommandInfo {
        let mut description = self.resolve(&c.info.description);
        if description.is_empty()
            && let Some(title) = c.title.as_deref().filter(|t| !t.trim().is_empty())
        {
            description = self.resolve(title);
        }
        CommandInfo {
            description,
            input_schema: resolve_schema(&c.info.input_schema, |t| self.resolve(t)),
            ..c.info.clone()
        }
    }

    /// The title of its command `short` (without the plugin's id), in the
    /// current language; `None` when it has no such command or no title.
    pub fn command_title(&self, short: &str) -> Option<String> {
        let c = self.commands.iter().find(|c| c.short == short)?;
        c.title.as_deref().filter(|t| !t.trim().is_empty()).map(|t| self.resolve(t))
    }

    /// Its buttons, text in the current language.
    pub fn buttons(&self) -> Vec<Button> {
        self.buttons
            .iter()
            .map(|b| Button {
                command: format!("{}{SEPARATOR}{}", self.manifest.id, b.command),
                label: self.resolve(&b.label),
                panel: match &b.panel {
                    Some(p) => self.resolve(p),
                    None => self.display_name(),
                },
                icon: b.icon.clone(),
                size: b.size,
                when: b.when.clone(),
            })
            .collect()
    }

    /// Its exporters (the app's Export list), labels in the current language.
    pub fn exporters(&self) -> Vec<FileAction> {
        self.file_actions(&self.exporters)
    }

    /// Its importers (the app's Import list), labels in the current language.
    pub fn importers(&self) -> Vec<FileAction> {
        self.file_actions(&self.importers)
    }

    fn file_actions(&self, list: &[FileActionDef]) -> Vec<FileAction> {
        list.iter()
            .map(|a| FileAction {
                command: format!("{}{SEPARATOR}{}", self.manifest.id, a.command),
                label: self.resolve(&a.label),
                param: a.param.clone(),
                extensions: a.extensions.clone(),
            })
            .collect()
    }

    /// Its settings, text in the current language.
    pub fn settings(&self) -> Vec<Setting> {
        self.settings.iter().map(|s| s.resolved(|t| self.resolve(t))).collect()
    }

    /// The current value of each of its settings (defaults included).
    pub fn setting_values(&self) -> Map<String, Value> {
        self.values.clone()
    }

    /// Change the current value of its setting `key` (not stored; see
    /// [`crate::store::write_settings`]).
    pub fn set_setting(&mut self, key: &str, value: &Value) -> Result<(), String> {
        let setting = self
            .settings
            .iter()
            .find(|s| s.key == key)
            .ok_or_else(|| format!("the plugin {} has no setting \"{key}\"", self.manifest.id))?;
        let value = setting.validate(value)?;
        self.values.insert(key.to_string(), value);
        Ok(())
    }

    /// Run its command `short` (the name without the plugin's id).
    fn call(&mut self, session: &mut Session, short: &str, args: Value) -> Option<Result<Value, EngineError>> {
        let command = self.commands.iter().find(|c| c.short == short)?;
        let name = command.info.name.clone();
        let kind = command.info.kind;
        if kind == CommandKind::File && !self.granted.files {
            let message = format!(
                "{name} works with files: the plugin {} was not granted the \"files\" permission",
                self.manifest.id
            );
            return Some(Err(EngineError::rejected(message)));
        }
        let args = match args {
            Value::Null => json!({}),
            Value::Object(_) => args,
            _ => return Some(Err(EngineError::invalid(format!("{name}: the arguments must be an object")))),
        };
        if let Some(missing) = command.required.iter().find(|k| args.get(k.as_str()).is_none()) {
            return Some(Err(EngineError::invalid(format!("{name}: missing field `{missing}`"))));
        }
        let grants = match grants(&name, &command.info.input_schema, &args) {
            Ok(g) => g,
            Err(e) => return Some(Err(e)),
        };
        let scope = PluginScope {
            plugin: self.manifest.id.clone(),
            edits: kind != CommandKind::Query,
            files: if self.granted.files { FileAccess::All } else { FileAccess::Granted(grants.clone()) },
        };
        let func = command.func.clone();
        if let Err(e) = fill_settings(&self.sandbox.lua, &self.settings_meta, &self.values) {
            return Some(Err(engine_error(&e)));
        }
        self.sandbox.reset_budget();
        let files = RefCell::new(FileRun::new(grants));
        let (lua, api) = (&self.sandbox.lua, &self.api);
        let result = session.with_plugin_scope(scope, |session| {
            api.with_session(lua, session, Some(&files), |lua| {
                let out: LuaValue = func.call(to_lua(lua, &args)?)?;
                from_lua(lua, &out).map_err(|e| mlua::Error::runtime(format!("{name}: the result: {e}")))
            })
        });
        // What it wrote with ccad.file reaches the disk only when it succeeded.
        Some(result.map_err(|e| engine_error(&e)).and_then(|out| files.into_inner().commit().map(|()| out)))
    }
}

/// The files the top-level path arguments of the command `name` (whose
/// parameters are `schema`) grant: a `save_path` one to write, an
/// `open_path` one to read. Refused before the command runs: a path that is
/// not a string, is empty, has an extension its `extensions` do not list, or
/// cannot be resolved (a file to read that does not exist, a file to write in
/// a folder that does not exist).
fn grants(name: &str, schema: &Value, args: &Value) -> Result<Grants, EngineError> {
    let mut grants = Grants::new();
    for (param, ui, extensions) in path_params(schema) {
        let Some(value) = args.get(&param) else { continue };
        let bad = |m: String| EngineError::invalid(format!("{name}: `{param}`: {m}"));
        let text = value.as_str().ok_or_else(|| bad("a path, as a string".into()))?;
        if text.trim().is_empty() {
            return Err(bad("an empty path".into()));
        }
        let path = Path::new(text);
        if !extensions.is_empty() {
            let ext = cc_engine::files::extension_of(path);
            if !extensions.contains(&ext) {
                return Err(bad(format!("{text}: the file must be .{}", extensions.join(", ."))));
            }
        }
        if ui == "save_path" {
            grants.grant_write(path).map_err(bad)?;
        } else {
            grants.grant_read(path).map_err(bad)?;
        }
    }
    Ok(grants)
}

/// The manifest's exporters or importers (`key`), checked against the
/// commands: each needs exactly one parameter with `ui` = `ui`.
fn file_actions(
    list: &[FileActionSpec],
    commands: &[PluginCommand],
    key: &str,
    ui: &str,
) -> Result<Vec<FileActionDef>, String> {
    list.iter()
        .enumerate()
        .map(|(i, a)| {
            let err = |m: String| format!("{key}[{}]: {m}", i + 1);
            let c = commands
                .iter()
                .find(|c| c.short == a.command)
                .ok_or_else(|| err(format!("the command \"{}\", which main.lua does not register", a.command)))?;
            let params: Vec<_> = path_params(&c.info.input_schema).into_iter().filter(|(_, u, _)| u == ui).collect();
            match params.as_slice() {
                [(param, _, extensions)] => Ok(FileActionDef {
                    command: a.command.clone(),
                    label: a.label.clone(),
                    param: param.clone(),
                    extensions: extensions.clone(),
                }),
                _ => Err(err(format!(
                    "the command \"{}\" needs exactly one parameter with ui = \"{ui}\" (it has {})",
                    a.command,
                    params.len()
                ))),
            }
        })
        .collect()
}

/// The manifest's buttons, checked against the plugin's folder (icons).
fn buttons(dir: &Path, manifest: &Manifest) -> Result<Vec<ButtonDef>, String> {
    manifest
        .buttons
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let err = |e: String| format!("buttons[{}]: {e}", i + 1);
            let (size, when) = b.check().map_err(err)?;
            let icon = match &b.icon {
                Some(rel) => {
                    let path = plain_path(rel, "svg").ok_or_else(|| err(format!("icon \"{rel}\": an .svg file")))?;
                    Some(file_inside(dir, &path, MAX_ICON).map_err(|e| err(format!("icon: {e}")))?)
                }
                None => None,
            };
            Ok(ButtonDef {
                command: b.command.clone(),
                label: b.label.clone(),
                panel: b.panel.clone(),
                icon,
                size,
                when,
            })
        })
        .collect()
}

/// What `ccad.command` collected while the plugin loaded.
type Registered = Rc<RefCell<Vec<PluginCommand>>>;

/// Install the API, `require`, `ccad.settings`, `ccad.data` and
/// `ccad.command`, and run the main file. Returns the API, the commands and
/// the metatable of `ccad.settings`.
fn setup(
    sandbox: &Sandbox,
    dir: &Path,
    manifest: &Manifest,
    source: &str,
    values: &Map<String, Value>,
) -> mlua::Result<(Api, Vec<PluginCommand>, Table)> {
    let lua = &sandbox.lua;
    let api = Api::install_for_plugin(lua)?;
    let plugin = lua.create_table()?;
    plugin.raw_set("id", manifest.id.as_str())?;
    plugin.raw_set("name", manifest.name.as_str())?;
    plugin.raw_set("version", manifest.version.as_str())?;
    api.ccad.raw_set("plugin", plugin)?;
    lua.globals().raw_set("require", require_fn(lua, dir, &manifest.id)?)?;

    let meta = lua.create_table()?;
    lua.load(SETTINGS_PRELUDE).set_name("=ccad.settings").call::<()>(meta.clone())?;
    let settings = lua.create_table()?;
    settings.set_metatable(Some(meta.clone()))?;
    fill_settings(lua, &meta, values)?;
    api.ccad.raw_set("settings", settings)?;
    lua.load(DATA_PRELUDE).set_name("=ccad.data").call::<()>((api.ccad.clone(), manifest.id.as_str()))?;

    let registered: Registered = Rc::new(RefCell::new(Vec::new()));
    let sink = registered.clone();
    let id = manifest.id.clone();
    let requested = manifest.requested();
    let version = manifest.api;
    let command = lua.create_function(move |lua, spec: Table| {
        let c = register(lua, &id, version, requested, &spec)?;
        let mut list = sink.borrow_mut();
        if list.len() >= MAX_COMMANDS {
            return Err(mlua::Error::runtime(format!("a plugin registers at most {MAX_COMMANDS} commands")));
        }
        if list.iter().any(|o| o.short == c.short) {
            return Err(mlua::Error::runtime(format!("command \"{}\" is registered twice", c.short)));
        }
        list.push(c);
        Ok(())
    })?;
    api.ccad.raw_set("command", command)?;

    sandbox.reset_budget();
    let chunk_name = format!("@{}/{}", manifest.id, manifest.main);
    lua.load(source).set_name(chunk_name).set_mode(mlua::ChunkMode::Text).exec()?;

    // Commands are registered while the plugin loads only.
    let late = lua.create_function(|_, _: mlua::MultiValue| -> mlua::Result<()> {
        Err(mlua::Error::runtime("ccad.command can only be called while the plugin loads"))
    })?;
    api.ccad.raw_set("command", late)?;
    let commands = std::mem::take(&mut *registered.borrow_mut());
    Ok((api, commands, meta))
}

/// Read one `ccad.command { … }` table of a plugin written for `version` of
/// the plugin API.
fn register(lua: &Lua, id: &str, version: u32, requested: Permissions, spec: &Table) -> mlua::Result<PluginCommand> {
    let err = |m: String| mlua::Error::runtime(format!("ccad.command: {m}"));
    let short: String = spec.get::<Option<String>>("name")?.ok_or_else(|| err("needs a name".into()))?;
    let full = format!("{id}{SEPARATOR}{short}");
    if !valid_command_name(&short) || full.len() > MAX_NAME {
        return Err(err(format!(
            "name \"{short}\": lowercase letters, digits and single underscores, starting with a letter \
             ({MAX_NAME} characters at most with the plugin's id)"
        )));
    }
    let title: Option<String> = spec.get("title")?;
    let description: String = spec.get::<Option<String>>("description")?.unwrap_or_default();
    let kind = match spec.get::<Option<String>>("kind")?.as_deref() {
        None | Some("edit") => CommandKind::Edit,
        Some("query") => CommandKind::Query,
        Some("file") if requested.files => CommandKind::File,
        Some("file") => {
            return Err(err(format!("{short}: kind \"file\" needs the \"files\" permission in {MANIFEST}")));
        }
        Some(other) => return Err(err(format!("{short}: kind \"{other}\" (query, edit or file)"))),
    };
    let func: Function =
        spec.get::<Option<Function>>("run")?.ok_or_else(|| err(format!("{short}: needs a run function")))?;
    let mut schema = match spec.get::<LuaValue>("params")? {
        LuaValue::Nil => json!({ "type": "object", "properties": {} }),
        v => from_lua(lua, &v).map_err(|e| err(format!("{short}: params: {e}")))?,
    };
    let Some(obj) = schema.as_object_mut() else {
        return Err(err(format!("{short}: params must be a JSON Schema object")));
    };
    obj.insert("type".into(), json!("object"));
    obj.entry("properties").or_insert_with(|| json!({}));
    let required: Vec<String> = obj
        .get("required")
        .and_then(Value::as_array)
        .map(|r| r.iter().filter_map(Value::as_str).map(str::to_string).collect())
        .unwrap_or_default();
    if version >= 2 {
        check_param_hints(&schema).map_err(|e| err(format!("{short}: {e}")))?;
    }
    let info = CommandInfo {
        name: full,
        kind,
        group: format!("Plugin: {id}"),
        description,
        input_schema: schema,
        reads_files: false,
    };
    Ok(PluginCommand { short, title, info, required, func })
}

/// `require(name)`: the module `a.b` is `a/b.lua` in the plugin's folder,
/// run once (its result is kept).
fn require_fn(lua: &Lua, dir: &Path, id: &str) -> mlua::Result<Function> {
    let dir = dir.to_path_buf();
    let id = id.to_string();
    // `None` while the module loads (to catch a module requiring itself).
    let cache: Rc<RefCell<HashMap<String, Option<LuaValue>>>> = Rc::default();
    lua.create_function(move |lua, name: String| {
        if name.len() > 128 {
            return Err(mlua::Error::runtime("require: module name too long"));
        }
        if let Some(entry) = cache.borrow().get(&name) {
            return match entry {
                Some(v) => Ok(v.clone()),
                None => Err(mlua::Error::runtime(format!("require: \"{name}\" requires itself"))),
            };
        }
        let rel = format!("{}.lua", name.replace('.', "/"));
        let path =
            module_path(&rel).ok_or_else(|| mlua::Error::runtime(format!("require: bad module name \"{name}\"")))?;
        let source = read_inside(&dir, &path, MAX_SOURCE).map_err(|e| mlua::Error::runtime(format!("require: {e}")))?;
        cache.borrow_mut().insert(name.clone(), None);
        let loaded = lua
            .load(source)
            .set_name(format!("@{id}/{rel}"))
            .set_mode(mlua::ChunkMode::Text)
            .call::<LuaValue>(name.clone());
        let value = match loaded {
            Ok(LuaValue::Nil) => LuaValue::Boolean(true),
            Ok(v) => v,
            Err(e) => {
                cache.borrow_mut().remove(&name);
                return Err(e);
            }
        };
        cache.borrow_mut().insert(name, Some(value.clone()));
        Ok(value)
    })
}

/// The loaded plugins, as the session's [`Extension`].
#[derive(Debug, Default)]
pub struct PluginHost {
    plugins: Vec<Plugin>,
    messages: Vec<String>,
}

impl PluginHost {
    pub fn new() -> PluginHost {
        PluginHost::default()
    }

    /// Add a loaded plugin; refused if one with its id is already there.
    pub fn add(&mut self, plugin: Plugin) -> Result<(), PluginError> {
        if self.plugins.iter().any(|p| p.manifest.id == plugin.manifest.id) {
            return Err(PluginError {
                dir: plugin.dir.clone(),
                message: format!("another plugin with the id \"{}\" is already loaded", plugin.manifest.id),
            });
        }
        if self.plugins.len() >= MAX_PLUGINS {
            return Err(PluginError { dir: plugin.dir.clone(), message: format!("at most {MAX_PLUGINS} plugins") });
        }
        let id = plugin.manifest.id.clone();
        let mut plugin = plugin;
        for line in std::mem::take(&mut plugin.printed) {
            self.messages.push(format!("[{id}] {line}"));
        }
        self.plugins.push(plugin);
        Ok(())
    }

    /// Load the plugins of `folder`: the folder itself when it holds a
    /// manifest, else each of its subfolders that does (in name order),
    /// skipping hidden ones (an install's `.<id>.installing` and `.<id>.old`,
    /// the `.settings` folder) and refusing one whose name is not its
    /// plugin's id (installs are named so; permissions are granted by id).
    /// `grant` decides the permissions of each, or `None` to leave it out
    /// (turned off). In a folder of plugins, each starts with the settings
    /// stored in `.settings/<id>.json` ([`crate::store::read_settings`]); a
    /// single plugin's folder gives the defaults. Returns the plugins that
    /// could not be loaded.
    pub fn load_folder(
        &mut self,
        folder: &Path,
        limits: Limits,
        grant: impl Fn(&Manifest) -> Option<Permissions>,
    ) -> Vec<PluginError> {
        let mut errors = Vec::new();
        let single = folder.join(MANIFEST).is_file();
        let dirs: Vec<PathBuf> = if single {
            vec![folder.to_path_buf()]
        } else {
            match std::fs::read_dir(folder) {
                Ok(entries) => {
                    let mut dirs: Vec<PathBuf> = entries
                        .filter_map(|e| e.ok().map(|e| e.path()))
                        .filter(|p| !p.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.')))
                        .filter(|p| p.join(MANIFEST).is_file())
                        .take(MAX_PLUGINS)
                        .collect();
                    dirs.sort();
                    dirs
                }
                Err(e) => {
                    errors.push(PluginError { dir: folder.to_path_buf(), message: e.to_string() });
                    Vec::new()
                }
            }
        };
        for dir in dirs {
            // Read the manifest first: a plugin left out runs no code.
            let manifest = read_inside(&dir, Path::new(MANIFEST), MAX_MANIFEST).and_then(|t| Manifest::parse(&t));
            let (permissions, stored) = match &manifest {
                // A folder named after another id (a copy, a leftover of an
                // install) would take that id's place and its permissions.
                Ok(m) if !single && dir.file_name().is_none_or(|n| n.to_string_lossy() != m.id.as_str()) => {
                    let message = format!("the folder of the plugin \"{}\" must be named {}", m.id, m.id);
                    errors.push(PluginError { dir: dir.clone(), message });
                    continue;
                }
                Ok(m) => match grant(m) {
                    Some(p) if single => (p, Map::new()),
                    Some(p) => (p, crate::store::read_settings(folder, &m.id)),
                    None => continue,
                },
                Err(message) => {
                    errors.push(PluginError { dir: dir.clone(), message: message.clone() });
                    continue;
                }
            };
            match Plugin::load_with_settings(&dir, limits, &stored) {
                Ok(mut plugin) => {
                    plugin.grant(permissions);
                    if let Err(e) = self.add(plugin) {
                        errors.push(e);
                    }
                }
                Err(e) => errors.push(e),
            }
        }
        errors
    }

    pub fn plugins(&self) -> &[Plugin] {
        &self.plugins
    }

    /// The plugin `id`, if loaded.
    pub fn plugin(&self, id: &str) -> Option<&Plugin> {
        self.plugins.iter().find(|p| p.manifest.id == id)
    }

    /// Change the current value of the setting `key` of the plugin `id`
    /// (checked against its schema; not stored, see
    /// [`crate::store::write_settings`]).
    pub fn set_setting(&mut self, id: &str, key: &str, value: Value) -> Result<(), String> {
        let plugin =
            self.plugins.iter_mut().find(|p| p.manifest.id == id).ok_or_else(|| format!("no plugin \"{id}\""))?;
        plugin.set_setting(key, &value)
    }

    /// The title of the plugin command `full_name` (`<id>__<name>`) in the
    /// current language; `None` when there is no such command or it has no
    /// title.
    pub fn command_title(&self, full_name: &str) -> Option<String> {
        let (id, short) = full_name.split_once(SEPARATOR)?;
        self.plugin(id)?.command_title(short)
    }

    /// A summary of the plugins for listings (`ccad plugins`), text in the
    /// current language.
    pub fn summary(&self) -> Value {
        Value::Array(
            self.plugins
                .iter()
                .map(|p| {
                    let commands: Vec<Value> = p
                        .commands
                        .iter()
                        .map(|c| {
                            let info = p.info(c);
                            json!({
                                "name": info.name,
                                "kind": info.kind,
                                "title": p.command_title(&c.short),
                                "description": info.description,
                            })
                        })
                        .collect();
                    let settings = settings_json(&p.settings(), &p.values);
                    json!({
                        "id": p.manifest.id,
                        "name": p.display_name(),
                        "version": p.manifest.version,
                        "api": p.manifest.api,
                        "description": p.display_description(),
                        "dir": p.dir,
                        "requested": p.manifest.requested(),
                        "granted": p.granted,
                        "commands": commands,
                        "buttons": p.buttons(),
                        "settings": settings,
                        "exporters": p.exporters(),
                        "importers": p.importers(),
                    })
                })
                .collect(),
        )
    }
}

impl Extension for PluginHost {
    fn commands(&self) -> Vec<CommandInfo> {
        self.plugins.iter().flat_map(|p| p.commands()).collect()
    }

    fn run(&mut self, session: &mut Session, name: &str, args: Value) -> Option<cc_engine::Result<Value>> {
        let (id, short) = name.split_once(SEPARATOR)?;
        let plugin = self.plugins.iter_mut().find(|p| p.manifest.id == id)?;
        let result = plugin.call(session, short, args);
        for line in plugin.sandbox.take_output() {
            self.messages.push(format!("[{id}] {line}"));
        }
        result
    }

    fn take_messages(&mut self) -> Vec<String> {
        std::mem::take(&mut self.messages)
    }
}
