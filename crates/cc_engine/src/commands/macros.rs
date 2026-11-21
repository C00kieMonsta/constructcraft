//! Macros (scripts of [`crate::script`]): check one without running it, and
//! write one in its canonical form.

use crate::error::{EngineError, Result};
use crate::registry::{Command, CommandKind};
use crate::script;
use crate::session::Session;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Largest macro file read (the app reads macros with the same bound).
pub const MAX_MACRO_BYTES: u64 = 32 * 1024 * 1024;

/// Check a macro without running it: lines that are not steps, commands
/// this session does not have, argument names a command refuses and
/// required arguments left out, and `$step`/`$level` placeholders that
/// cannot resolve. Returns the number of steps and the problems found, each
/// with its line (from 1; null for the array format), its step (from 1) and
/// its severity (`error`, or `warning` for what runs but is likely a
/// mistake). Nothing runs.
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct MacroCheck {
    /// The macro itself: one `{"command": ..., "args": {...}}` object per
    /// line (`//` lines are comments), or a JSON array of them.
    #[serde(default)]
    pub text: Option<String>,
    /// Path of a macro file, instead of `text`.
    #[serde(default)]
    pub path: Option<String>,
}

impl Command for MacroCheck {
    const NAME: &'static str = "macro_check";
    const KIND: CommandKind = CommandKind::Query;
    const READS_FILES: bool = true;
    fn run(self, s: &mut Session) -> Result<Value> {
        let text = match (self.text, self.path) {
            (Some(text), _) => text,
            (None, Some(path)) => {
                s.require_read("path", std::path::Path::new(&path))?;
                cc_core::read_to_string_limited(std::path::Path::new(&path), MAX_MACRO_BYTES)
                    .map_err(|e| EngineError::invalid(format!("{path}: {e}")))?
            }
            (None, None) => return Err(EngineError::invalid("give the macro's `text` or its `path`")),
        };
        let steps =
            script::parse_lines(&text).iter().filter(|l| matches!(l.item, script::LineItem::Step { .. })).count();
        Ok(json!({ "steps": steps, "diagnostics": script::check(s, &text) }))
    }
}

/// Write a macro in its canonical form: one step per line, as compact JSON
/// the way recorded macros are saved, with comments and blank lines kept
/// where they are and lines that are not steps kept as they are. A macro in
/// the array format becomes one step per line. Returns `{text}`.
#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct MacroFormat {
    /// The macro to format.
    pub text: String,
}

impl Command for MacroFormat {
    const NAME: &'static str = "macro_format";
    const KIND: CommandKind = CommandKind::Query;
    fn run(self, _: &mut Session) -> Result<Value> {
        Ok(json!({ "text": script::format(&self.text)? }))
    }
}
