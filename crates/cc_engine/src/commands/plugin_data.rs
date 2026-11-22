//! Data plugins keep in the project, per element and per project, namespaced
//! by plugin id (see [`cc_core::plugin_data`]).

use super::edit;
use crate::error::{EngineError, ErrorCode, Result};
use crate::refs;
use crate::registry::{Command, CommandKind};
use crate::session::Session;
use crate::t;
use cc_core::ElementId;
use cc_core::plugin_data::{MAX_PLUGIN_DATA_BYTES, MAX_PLUGIN_DATA_DEPTH, PluginData, nesting, valid_plugin_id};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// The plugin id, refused unless valid.
fn plugin_id(id: &str) -> Result<&str> {
    if valid_plugin_id(id) {
        Ok(id)
    } else {
        Err(EngineError::invalid(format!(
            "plugin id \"{id}\": up to 32 lowercase letters, digits and single underscores, starting with a letter"
        )))
    }
}

/// The data of the element (when given; it must exist) or of the project.
fn data_of(s: &Session, element: Option<ElementId>) -> Result<&PluginData> {
    let p = s.project();
    match element {
        Some(id) => p.element(id).map(|e| &e.plugin_data).ok_or_else(|| EngineError::not_found("element")),
        None => Ok(&p.plugin_data),
    }
}

/// The JSON value a plugin keeps on an element or on the project, or `null`
/// when it keeps none.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PluginDataGet {
    /// Plugin id (lowercase letters, digits and single underscores, starting with a letter; at most 32).
    pub plugin: String,
    /// Element id (default: the project's own data).
    #[serde(default)]
    pub element: Option<u64>,
}

impl Command for PluginDataGet {
    const NAME: &'static str = "plugin_data_get";
    const KIND: CommandKind = CommandKind::Query;
    fn run(self, s: &mut Session) -> Result<Value> {
        let plugin = plugin_id(&self.plugin)?;
        let element = self.element.map(|e| refs::element(s.project(), e)).transpose()?;
        match data_of(s, element)?.get(plugin) {
            None => Ok(Value::Null),
            Some(text) => serde_json::from_str(text)
                .map_err(|e| EngineError::new(ErrorCode::Internal, format!("stored data of plugin \"{plugin}\": {e}"))),
        }
    }
}

/// Keep a JSON value for a plugin on an element or on the project, replacing
/// what that plugin kept there (`data: null` removes it). Saved in the
/// project file; one undo step.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PluginDataSet {
    /// Plugin id (lowercase letters, digits and single underscores, starting with a letter; at most 32).
    pub plugin: String,
    /// Element id (default: the project's own data).
    #[serde(default)]
    pub element: Option<u64>,
    /// The value to keep (any JSON, at most 256 KiB as compact JSON text and nested at most 64 deep); `null` removes the plugin's data.
    pub data: Value,
}

impl Command for PluginDataSet {
    const NAME: &'static str = "plugin_data_set";
    const KIND: CommandKind = CommandKind::Edit;
    fn run(self, s: &mut Session) -> Result<Value> {
        let plugin = plugin_id(&self.plugin)?.to_string();
        let element = self.element.map(|e| refs::element(s.project(), e)).transpose()?;
        let text = match &self.data {
            Value::Null => None,
            data => {
                let text = serde_json::to_string(data).map_err(|e| EngineError::invalid(format!("data: {e}")))?;
                if text.len() > MAX_PLUGIN_DATA_BYTES {
                    return Err(EngineError::invalid(format!(
                        "data: {} bytes of JSON, at most {MAX_PLUGIN_DATA_BYTES}",
                        text.len()
                    )));
                }
                if nesting(&text) > MAX_PLUGIN_DATA_DEPTH {
                    return Err(EngineError::invalid(format!(
                        "data: arrays and objects nested more than {MAX_PLUGIN_DATA_DEPTH} deep"
                    )));
                }
                Some(text)
            }
        };
        let stored = text.is_some();
        edit(s, t!("tx-plugin-data"), |e| {
            let data = match element {
                Some(id) => &mut e.element_mut(id).ok_or_else(|| EngineError::not_found("element"))?.plugin_data,
                None => e.plugin_data_mut(),
            };
            match text {
                Some(text) => data.insert(plugin.clone(), text),
                None => data.remove(&plugin),
            };
            Ok(())
        })?;
        Ok(json!({ "plugin": plugin, "element": element.map(|id| id.0), "stored": stored }))
    }
}

/// Where a plugin keeps data: whether the project has some (`project`) and
/// the ids of the elements that do (`elements`, ascending).
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PluginDataList {
    /// Plugin id (lowercase letters, digits and single underscores, starting with a letter; at most 32).
    pub plugin: String,
}

impl Command for PluginDataList {
    const NAME: &'static str = "plugin_data_list";
    const KIND: CommandKind = CommandKind::Query;
    fn run(self, s: &mut Session) -> Result<Value> {
        let plugin = plugin_id(&self.plugin)?;
        let p = s.project();
        let elements: Vec<u64> =
            p.elements.values().filter(|e| e.plugin_data.contains_key(plugin)).map(|e| e.id.0).collect();
        Ok(json!({ "project": p.plugin_data.contains_key(plugin), "elements": elements }))
    }
}
