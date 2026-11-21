//! Lua scripts and plugins for ConstructCraft.
//!
//! Lua 5.4 runs sandboxed ([`sandbox`], from `cc_script`): no files, no loading of code, an
//! instruction budget and a memory limit. Lua code reaches the project only
//! through the engine's commands (`ccad.run`, see [`api`]), the same ones the
//! GUI, the command line and the MCP server use.
//!
//! - [`run_script`] runs a script once (`ccad lua file.lua`).
//! - [`plugin`] loads plugins (a folder with `plugin.toml` and `main.lua`)
//!   whose commands join the session's catalog through [`PluginHost`], an
//!   engine [`cc_engine::Extension`]: each plugin command is one undo step
//!   and leaves nothing behind when it fails.
//! - Plugins written for API 2 also declare ribbon buttons and settings in
//!   their manifest ([`contributions`]), translate their text
//!   ([`locales`]), give command parameters UI hints, and keep their own
//!   data in the project (`ccad.data`).
//! - [`store`] installs plugins from the community index and keeps the
//!   values of their settings.

pub mod api;
pub mod contributions;
pub mod locales;
pub mod plugin;
pub mod script;
pub mod store;

pub use cc_script::{convert, sandbox};

pub use api::{API_VERSION, FileRun};
pub use contributions::{Button, ButtonSize, FileAction, Setting, SettingKind, When};
pub use locales::Locales;
pub use plugin::{Manifest, Permissions, Plugin, PluginError, PluginHost};
pub use sandbox::Limits;
pub use script::{ScriptOptions, ScriptOutput, run_script};
