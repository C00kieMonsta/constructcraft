//! Data plugins keep in the project, on elements and on the project itself
//! (like attribute dictionaries): one JSON value per plugin, keyed by the
//! plugin's id. Values are stored as JSON text, so the RON file never holds
//! a free-form value tree (whose depth and shape would be the plugin's).

use std::collections::BTreeMap;

/// Plugin id → its JSON value, as compact text.
pub type PluginData = BTreeMap<String, String>;

/// Largest JSON text one plugin keeps on one element or on the project, bytes.
pub const MAX_PLUGIN_DATA_BYTES: usize = 256 * 1024;

/// Deepest nesting of arrays and objects in one plugin's data (the bound of
/// Lua tables converted to JSON; well under what the JSON reader takes).
pub const MAX_PLUGIN_DATA_DEPTH: usize = 64;

/// Longest plugin id, bytes.
pub const MAX_PLUGIN_ID: usize = 32;

/// Whether `s` follows the naming rule of plugin ids and command names: a
/// lowercase ASCII letter, then lowercase letters, digits and single
/// underscores (no `__`), not ending in one. No length limit.
pub fn valid_plugin_name(s: &str) -> bool {
    s.chars().next().is_some_and(|c| c.is_ascii_lowercase())
        && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        && !s.contains("__")
        && !s.ends_with('_')
}

/// Whether `s` is a valid plugin id: [`valid_plugin_name`] and at most
/// [`MAX_PLUGIN_ID`] bytes.
pub fn valid_plugin_id(s: &str) -> bool {
    valid_plugin_name(s) && s.len() <= MAX_PLUGIN_ID
}

/// How deep arrays and objects nest in JSON `text` (brackets inside strings
/// do not count). Linear, with no allocation.
pub fn nesting(text: &str) -> usize {
    let (mut depth, mut deepest, mut in_string, mut escaped) = (0usize, 0usize, false, false);
    for b in text.bytes() {
        if in_string {
            match b {
                _ if escaped => escaped = false,
                b'\\' => escaped = true,
                b'"' => in_string = false,
                _ => {}
            }
            continue;
        }
        match b {
            b'"' => in_string = true,
            b'[' | b'{' => {
                depth += 1;
                deepest = deepest.max(depth);
            }
            b']' | b'}' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    deepest
}

/// Whether `text` may be kept as one plugin's data: one JSON value other
/// than `null` (which means no data), at most [`MAX_PLUGIN_DATA_BYTES`] and
/// nested at most [`MAX_PLUGIN_DATA_DEPTH`] deep.
pub fn valid_value(text: &str) -> bool {
    text.len() <= MAX_PLUGIN_DATA_BYTES
        && nesting(text) <= MAX_PLUGIN_DATA_DEPTH
        && text.trim() != "null"
        && serde_json::from_str::<serde::de::IgnoredAny>(text).is_ok()
}

/// Drop the entries of a bad plugin id or a bad value; returns how many.
pub fn repair(data: &mut PluginData) -> usize {
    let before = data.len();
    data.retain(|id, text| valid_plugin_id(id) && valid_value(text));
    before - data.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_ids() {
        for ok in ["a", "quick_rooms", "a1_b2", &"a".repeat(32)] {
            assert!(valid_plugin_id(ok), "{ok}");
        }
        for bad in ["", "1a", "_a", "a_", "a__b", "Ab", "a-b", "a b", "á", &"a".repeat(33)] {
            assert!(!valid_plugin_id(bad), "{bad}");
        }
        assert!(valid_plugin_name(&"a".repeat(64)));
    }

    #[test]
    fn repair_drops_bad_entries() {
        let mut data = PluginData::new();
        data.insert("good".into(), r#"{"a":[1,2]}"#.into());
        data.insert("Bad".into(), "1".into());
        data.insert("broken".into(), "{".into());
        data.insert("trailing".into(), "1 2".into());
        data.insert("empty".into(), "null".into());
        data.insert("huge".into(), format!("\"{}\"", "x".repeat(MAX_PLUGIN_DATA_BYTES)));
        data.insert("deep".into(), format!("{}{}", "[".repeat(1000), "]".repeat(1000)));
        let limit = MAX_PLUGIN_DATA_DEPTH;
        data.insert("at_limit".into(), format!("{}{}", "[".repeat(limit), "]".repeat(limit)));
        assert_eq!(repair(&mut data), 6);
        assert_eq!(data.keys().collect::<Vec<_>>(), ["at_limit", "good"]);
    }

    #[test]
    fn nesting_skips_strings() {
        assert_eq!(nesting("1"), 0);
        assert_eq!(nesting(r#"{"a":[1,{"b":[]}]}"#), 4);
        assert_eq!(nesting(r#"["[[[\"[{", {}]"#), 2);
        // Values at the limit read back as JSON values.
        let text = format!("{}{}", "[".repeat(MAX_PLUGIN_DATA_DEPTH), "]".repeat(MAX_PLUGIN_DATA_DEPTH));
        assert!(serde_json::from_str::<serde_json::Value>(&text).is_ok());
    }
}
