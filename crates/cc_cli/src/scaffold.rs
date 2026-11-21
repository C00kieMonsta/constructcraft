//! `ccad plugins new <folder> <id> [name=…]`: a new plugin's folder with a
//! command, a button, a setting, translations, an icon, a test and the
//! type definitions of the `ccad` table for the Lua language server.

use std::path::{Path, PathBuf};

/// The type definitions of the `ccad` table (LuaLS), copied into
/// `types/ccad.d.lua`.
pub const TYPES: &str = include_str!("../../../docs/lua/ccad.d.lua");

/// The id made readable: `room_tools` → `Room tools`.
fn readable(id: &str) -> String {
    let text = id.replace('_', " ");
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// `text` as a TOML basic string (with its quotes).
fn toml_string(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if c.is_control() => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The files of a new plugin `id` named `name`: relative path and content.
pub fn files(id: &str, name: &str) -> Vec<(String, String)> {
    let name_toml = toml_string(name);
    let manifest = format!(
        r#"# See docs/automation.md, "Plugins", and README.md.
id = "{id}"
name = "%name%"
version = "0.1.0"
api = 2
description = "%description%"
authors = []
license = "MIT"
main = "main.lua"

[[buttons]]
command = "wall_line"
label = "%button.wall_line%"
icon = "icons/{id}.svg"
panel = "%panel%"

[settings.wall_height]
type = "number"
ui = "length"
default = 2.8
minimum = 1.0
maximum = 10.0
title = "%settings.wall_height.title%"
description = "%settings.wall_height.description%"
"#
    );
    let main = r#"-- The plugin's code: it runs once when the plugin loads and registers its
-- commands. Types of the ccad table: types/ccad.d.lua (see .luarc.json).

ccad.command {
  name = "wall_line",
  title = "%wall_line.title%",
  description = "%wall_line.description%",
  params = {
    properties = {
      start = { type = "array", ui = "point", title = "%param.start.title%", description = "%param.start.description%" },
      length = { type = "number", ui = "length", minimum = 0.1, title = "%param.length.title%" },
      level = { type = "integer", ui = "level", title = "%param.level.title%" },
    },
    required = { "start", "length" },
  },
  run = function(args)
    if args.length <= 0 then
      error("the length must be positive", 0)
    end
    -- The setting stands in for a height the command is not given.
    local height = ccad.settings.wall_height
    local wall = ccad.run("wall_create", {
      start = args.start,
      ["end"] = { args.start[1] + args.length, args.start[2] },
      height = height,
      level = args.level,
    })
    -- The plugin's own data on the wall, saved with the project.
    ccad.data.set(wall.id, { length = args.length })
    return { wall = wall.id }
  end,
}
"#
    .to_string();
    let en = format!(
        r#"name = {name_toml}
description = "A wall along x from a point, as high as the plugin's setting."
panel = {name_toml}

[button]
wall_line = "Wall line"

[wall_line]
title = "Wall line"
description = "A wall along x from a point, with the height of the plugin's setting."

[param.start]
title = "Start"
description = "Where the wall starts."

[param.length]
title = "Length"

[param.level]
title = "Level"

[settings.wall_height]
title = "Wall height"
description = "The height of the walls it draws."
"#
    );
    let pt = format!(
        r#"name = {name_toml}
description = "Uma parede ao longo de x a partir de um ponto, com a altura da configuração do plugin."
panel = {name_toml}

[button]
wall_line = "Linha de parede"

[wall_line]
title = "Linha de parede"
description = "Uma parede ao longo de x a partir de um ponto, com a altura da configuração do plugin."

[param.start]
title = "Início"
description = "Onde a parede começa."

[param.length]
title = "Comprimento"

[param.level]
title = "Nível"

[settings.wall_height]
title = "Altura da parede"
description = "A altura das paredes que ele desenha."
"#
    );
    let icon = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#fff" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><path d="M3 9h18v6H3z" fill="#fff" fill-opacity=".2"/><path d="M3 19h18M3 17v4M21 17v4"/></svg>
"##
    .to_string();
    let test = format!(
        r#"-- Run with: ccad plugins test <this plugin's folder>
-- Each test_* function runs in a fresh project with the plugin loaded;
-- ccad.test has eq, near, fails and ok.
local t = ccad.test

function test_draws_a_wall_with_the_default_height()
  local out = ccad.run("{id}__wall_line", {{ start = {{ 1, 2 }}, length = 4 }})
  local wall = ccad.run("element_get", {{ id = out.wall }})
  t.near(wall.derived.length, 4, 1e-6, "length")
  t.near(wall.params.height, 2.8, 1e-6, "the wall_height setting")
  t.eq(ccad.run("plugin_data_get", {{ plugin = "{id}", element = out.wall }}), {{ length = 4 }})
end

function test_refuses_a_length_that_is_not_positive()
  t.fails(function()
    ccad.run("{id}__wall_line", {{ start = {{ 0, 0 }}, length = 0 }})
  end, "positive")
end
"#
    );
    let readme = format!(
        r#"# {name}

A ConstructCraft plugin (plugin API 2): the command `{id}__wall_line`, a ribbon
button, the setting `wall_height`, English and Portuguese texts.

- `plugin.toml`: the manifest (id, buttons, settings).
- `main.lua`: the code; `locales/*.toml`: the texts; `icons/`: the button's icon.
- `tests/*_test.lua`: tests, run with `ccad plugins test <this folder>`.
- `types/ccad.d.lua` and `.luarc.json`: completion and checks of the `ccad`
  table in editors with the Lua language server (LuaLS).

```sh
ccad plugins test .                                   # run the tests
ccad --plugins . run {id}__wall_line start=[0,0] length=5
```

In the app, set this folder as the development folder (Preferences →
Plugin developer mode): the plugin loads again whenever a file changes, and
the Plugin console shows what it prints. See "Developing a plugin" in
ConstructCraft's docs/automation.md.
"#
    );
    let luarc = r#"{
  "$schema": "https://raw.githubusercontent.com/LuaLS/vscode-lua/master/setting/schema.json",
  "runtime.version": "Lua 5.4",
  "workspace.library": ["types"],
  "diagnostics.globals": ["ccad"],
  "diagnostics.disable": ["lowercase-global"]
}
"#
    .to_string();
    vec![
        ("plugin.toml".into(), manifest),
        ("main.lua".into(), main),
        ("locales/en.toml".into(), en),
        ("locales/pt-BR.toml".into(), pt),
        (format!("icons/{id}.svg"), icon),
        ("tests/basic_test.lua".into(), test),
        ("README.md".into(), readme),
        (".luarc.json".into(), luarc),
        ("types/ccad.d.lua".into(), TYPES.to_string()),
    ]
}

/// Make the plugin `id` in `folder/<id>`; refused when the id is invalid,
/// the name is not 1 to 100 characters on one line, or the folder exists.
/// Returns the plugin's folder.
pub fn create(folder: &Path, id: &str, name: Option<&str>) -> Result<PathBuf, String> {
    if !cc_plugins::plugin::valid_id(id) {
        return Err(format!(
            "\"{id}\" is not a plugin id: up to 32 lowercase letters, digits and single underscores, starting \
             with a letter"
        ));
    }
    let name = name.map_or_else(|| readable(id), str::to_string);
    if name.trim().is_empty() || name.len() > 100 || name.chars().any(char::is_control) {
        return Err("name: 1 to 100 characters on one line".into());
    }
    let dir = folder.join(id);
    if dir.exists() {
        return Err(format!("{} already exists", dir.display()));
    }
    std::fs::create_dir_all(folder).map_err(|e| format!("{}: {e}", folder.display()))?;
    std::fs::create_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let written = files(id, &name).into_iter().try_for_each(|(rel, text)| {
        let path = dir.join(&rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))
    });
    if let Err(e) = written {
        // Nothing half made is left.
        let _ = std::fs::remove_dir_all(&dir);
        return Err(e);
    }
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_and_strings() {
        assert_eq!(readable("room_tools"), "Room tools");
        assert_eq!(toml_string("a \"b\" \\ c"), r#""a \"b\" \\ c""#);
    }
}
