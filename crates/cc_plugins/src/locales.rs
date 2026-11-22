//! A plugin's translations (plugin API 2): `locales/<code>.toml` in its
//! folder, `<code>` a ConstructCraft language code (`en`, `pt-BR`, `zh-CN`…;
//! files with other names are ignored). Each holds `key = "text"` lines, or
//! tables whose keys join with `.`:
//!
//! ```toml
//! name = "Quick rooms"
//! [settings]
//! height = "Default height"   # the key settings.height
//! ```
//!
//! A text shown to the user that is exactly `%key%` is looked up when it is
//! shown, in the current language ([`cc_engine::i18n::language`]): the file
//! of that exact code, then one of the same base language (`pt` for `pt-BR`),
//! then `en`; a key found nowhere shows as the key itself.

use crate::plugin::read_inside;
use cc_engine::i18n::Language;
use std::collections::HashMap;
use std::path::Path;

/// The folder of the locale files, inside the plugin's folder.
pub const LOCALES_DIR: &str = "locales";
/// Largest locale file, in bytes.
pub const MAX_LOCALE: u64 = 256 * 1024;
/// Deepest nesting of tables in a locale file.
const MAX_DEPTH: usize = 16;
/// Most entries of the locales folder looked at.
const MAX_ENTRIES: usize = 1000;

/// A plugin's translations, by language code.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Locales {
    /// `(code, key → text)`, sorted by code.
    files: Vec<(String, HashMap<String, String>)>,
}

/// The key of a text that is exactly `%key%`.
pub fn key_of(text: &str) -> Option<&str> {
    let key = text.strip_prefix('%')?.strip_suffix('%')?;
    (!key.is_empty() && !key.contains('%') && !key.chars().any(char::is_whitespace)).then_some(key)
}

/// The language part of a code (`pt` of `pt-BR`).
fn base(code: &str) -> &str {
    code.split('-').next().unwrap_or(code)
}

impl Locales {
    /// Read the locale files of the plugin in `dir` (none when it has no
    /// `locales` folder). A file that is too large, outside the folder
    /// (through a link) or not TOML of strings is an error.
    pub fn load(dir: &Path) -> Result<Locales, String> {
        let folder = dir.join(LOCALES_DIR);
        if !folder.is_dir() {
            return Ok(Locales::default());
        }
        let entries = std::fs::read_dir(&folder).map_err(|e| format!("{LOCALES_DIR}: {e}"))?;
        let mut files = Vec::new();
        for entry in entries.take(MAX_ENTRIES).filter_map(Result::ok) {
            let name = entry.file_name();
            let Some(code) = name.to_str().and_then(|n| n.strip_suffix(".toml")) else { continue };
            if !Language::ALL.iter().any(|l| l.code() == code) {
                continue;
            }
            let rel = Path::new(LOCALES_DIR).join(format!("{code}.toml"));
            let text = read_inside(dir, &rel, MAX_LOCALE)?;
            let shown = format!("{LOCALES_DIR}/{code}.toml");
            let table: toml::Table = toml::from_str(&text).map_err(|e| format!("{shown}: {e}"))?;
            let mut map = HashMap::new();
            flatten("", &table, &mut map, 0).map_err(|e| format!("{shown}: {e}"))?;
            files.push((code.to_string(), map));
        }
        files.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(Locales { files })
    }

    /// The language codes it has files for.
    pub fn codes(&self) -> impl Iterator<Item = &str> {
        self.files.iter().map(|(c, _)| c.as_str())
    }

    /// The translation of `key` for the language `code`: that code, then
    /// its base language, then English.
    pub fn lookup(&self, key: &str, code: &str) -> Option<&str> {
        let exact = self.files.iter().find(|(c, _)| c == code);
        let same_base = self.files.iter().filter(|(c, _)| base(c) == base(code));
        let english = self.files.iter().find(|(c, _)| c == "en");
        exact.into_iter().chain(same_base).chain(english).find_map(|(_, m)| m.get(key)).map(String::as_str)
    }

    /// `text`, or its translation in the language `code` when it is a
    /// `%key%` (the key itself when no file has it).
    pub fn resolve_for(&self, text: &str, code: &str) -> String {
        match key_of(text) {
            Some(key) => self.lookup(key, code).unwrap_or(key).to_string(),
            None => text.to_string(),
        }
    }

    /// [`Locales::resolve_for`] in the current language.
    pub fn resolve(&self, text: &str) -> String {
        self.resolve_for(text, cc_engine::i18n::language().code())
    }
}

/// Put the strings of `table` in `out`, nested keys joined with `.`.
fn flatten(prefix: &str, table: &toml::Table, out: &mut HashMap<String, String>, depth: usize) -> Result<(), String> {
    if depth > MAX_DEPTH {
        return Err(format!("tables nested more than {MAX_DEPTH} deep"));
    }
    for (k, v) in table {
        let key = if prefix.is_empty() { k.clone() } else { format!("{prefix}.{k}") };
        match v {
            toml::Value::String(s) => {
                out.insert(key, s.clone());
            }
            toml::Value::Table(t) => flatten(&key, t, out, depth + 1)?,
            _ => return Err(format!("{key}: not a string")),
        }
    }
    Ok(())
}
