//! Plugin event handlers (`ccad.on`): payloads, their scope, their budget,
//! and what happens when they fail.

use cc_engine::Session;
use cc_plugins::{Limits, Plugin, PluginHost};
use serde_json::{Value, json};
use std::path::PathBuf;

const TRACKER: &str = r#"
ccad.on("project_opened", function(e) print("opened " .. tostring(e.path) .. " " .. tostring(e.new)) end)
ccad.on("project_saving", function(e) print("saving " .. e.path) end)
ccad.on("project_saved", function(e) print("saved " .. e.path) end)
ccad.on("model_changed", function(e)
  print(string.format("changed %s +%d ~%d -%d%s", e.command, #e.added, #e.modified, #e.removed,
    e.other and " other" or ""))
  local _, err = ccad.try("element_set", { id = e.added[1] or e.modified[1] or 1, mark = "X" })
  print("edit " .. (err and err.code or "ok"))
  if e.command ~= "undo" and e.command ~= "redo" then
    ccad.data.set(nil, { last = e.command, added = e.added })
  end
end)
ccad.command { name = "walls", run = function()
  ccad.run("wall_create", { start = {0, 5}, ["end"] = {4, 5} })
  ccad.run("wall_create", { start = {0, 9}, ["end"] = {4, 9} })
end }
ccad.command { name = "late", kind = "query", run = function() ccad.on("model_changed", function() end) end }
"#;

fn plugin_dir(name: &str, id: &str, main: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ccad-events-{}-{name}", std::process::id())).join(id);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("plugin.toml"),
        format!("id = \"{id}\"\nname = \"{id} name\"\nversion = \"1.0\"\napi = 2\n"),
    )
    .unwrap();
    std::fs::write(dir.join("main.lua"), main).unwrap();
    dir
}

fn load(name: &str, id: &str, main: &str, limits: Limits) -> Result<Plugin, String> {
    Plugin::load(&plugin_dir(name, id, main), limits).map_err(|e| e.message)
}

fn session_with(plugins: Vec<Plugin>) -> Session {
    let mut host = PluginHost::new();
    for p in plugins {
        host.add(p).unwrap();
    }
    let mut s = Session::default();
    s.set_extension(Some(Box::new(host)));
    s
}

fn walls(s: &mut Session) -> Vec<u64> {
    let list = s.execute("elements_list", json!({"category": "wall"})).unwrap();
    list.as_array().unwrap().iter().map(|e| e["id"].as_u64().unwrap()).collect()
}

fn create_wall(s: &mut Session) {
    s.execute("wall_create", json!({"start": [0, 0], "end": [4, 0]})).unwrap();
}

#[test]
fn handlers_get_each_event_with_its_payload() {
    let mut s = session_with(vec![load("payloads", "ev", TRACKER, Limits::default()).unwrap()]);
    assert_eq!(s.take_extension_messages(), vec!["[ev] opened nil true"]);
    let depth = s.doc.undo_depth();
    create_wall(&mut s);
    let id = walls(&mut s)[0];
    // The handler may not edit the model, but keeps its own data, in the
    // command's undo step; its own edit fires nothing more.
    assert_eq!(s.take_extension_messages(), vec!["[ev] changed wall_create +1 ~0 -0", "[ev] edit rejected"]);
    assert_eq!(s.doc.undo_depth(), depth + 1);
    assert_eq!(
        s.execute("plugin_data_get", json!({"plugin": "ev"})).unwrap(),
        json!({"last": "wall_create", "added": [id]})
    );
    assert_eq!(s.execute("element_get", json!({"id": id})).unwrap()["mark"], json!(""));
    s.execute("element_set", json!({"id": id, "mark": "A"})).unwrap();
    assert_eq!(s.take_extension_messages()[0], "[ev] changed element_set +0 ~1 -0");
    s.execute("elements_delete", json!({"ids": [id]})).unwrap();
    assert_eq!(s.take_extension_messages()[0], "[ev] changed elements_delete +0 ~0 -1");
    // One undo takes back the delete and the data the handler kept for it.
    s.execute("undo", json!({})).unwrap();
    assert_eq!(s.take_extension_messages()[0], "[ev] changed undo +1 ~0 -0 other");
    assert_eq!(s.execute("plugin_data_get", json!({"plugin": "ev"})).unwrap()["last"], json!("element_set"));
    s.execute("redo", json!({})).unwrap();
    assert_eq!(s.take_extension_messages()[0], "[ev] changed redo +0 ~0 -1 other");
    // A plugin command fires once, under its full name.
    s.execute("ev__walls", json!({})).unwrap();
    assert_eq!(s.take_extension_messages()[0], "[ev] changed ev__walls +2 ~0 -0");
    // Failed commands and queries fire nothing.
    assert!(s.execute("wall_create", json!({"start": [0, 0]})).is_err());
    s.execute("elements_list", json!({})).unwrap();
    assert!(s.take_extension_messages().is_empty());
    // Saving: before and after, with the path.
    let dir = std::env::temp_dir().join(format!("ccad-events-{}-save", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("p.ccad");
    s.execute("project_save", json!({"path": path})).unwrap();
    let shown = path.display().to_string();
    assert_eq!(s.take_extension_messages(), vec![format!("[ev] saving {shown}"), format!("[ev] saved {shown}")]);
    s.execute("project_open", json!({"path": path})).unwrap();
    assert_eq!(s.take_extension_messages(), vec![format!("[ev] opened {shown} false")]);
    s.execute("project_new", json!({})).unwrap();
    assert_eq!(s.take_extension_messages(), vec!["[ev] opened nil true"]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn handler_edits_stay_out_of_macros() {
    let mut s = session_with(vec![load("macro", "ev", TRACKER, Limits::default()).unwrap()]);
    s.start_recording();
    let depth = s.doc.undo_depth();
    create_wall(&mut s);
    let steps = s.stop_recording();
    assert_eq!(steps.iter().map(|st| st.command.as_str()).collect::<Vec<_>>(), vec!["wall_create"]);
    assert_eq!(s.doc.undo_depth(), depth + 1);
    assert!(s.execute("plugin_data_get", json!({"plugin": "ev"})).unwrap().is_object());
}

#[test]
fn a_failing_handler_leaves_the_command_and_is_turned_off() {
    let main = r#"
ccad.on("model_changed", function(e) ccad.data.set(nil, 1); error("boom") end)
ccad.on("model_changed", function(e) print("second ran") end)
"#;
    let mut s = session_with(vec![load("failing", "bad", main, Limits::default()).unwrap()]);
    for round in 1..=3 {
        let depth = s.doc.undo_depth();
        create_wall(&mut s);
        assert_eq!(walls(&mut s).len(), round);
        // Its edits went with it: only the wall's step is left.
        assert_eq!(s.doc.undo_depth(), depth + 1);
        assert!(!s.doc.can_redo());
        let messages = s.take_extension_messages();
        assert!(
            messages[0].starts_with("[bad] model_changed handler 1: ") && messages[0].contains("boom"),
            "{messages:?}"
        );
        if round == 3 {
            assert!(messages[1].contains("failed 3 times in a row"), "{messages:?}");
        }
        assert_eq!(messages.last().unwrap(), "[bad] second ran");
    }
    assert_eq!(s.execute("plugin_data_get", json!({"plugin": "bad"})).unwrap(), Value::Null);
    create_wall(&mut s);
    assert_eq!(s.take_extension_messages(), vec!["[bad] second ran"]);
}

#[test]
fn a_success_resets_the_failure_count() {
    let main = r#"
local n = 0
ccad.on("model_changed", function(e) n = n + 1; if n % 3 ~= 0 then error("odd " .. n) end end)
"#;
    let mut s = session_with(vec![load("reset", "flaky", main, Limits::default()).unwrap()]);
    for _ in 0..7 {
        create_wall(&mut s);
    }
    let messages = s.take_extension_messages();
    assert!(messages.iter().all(|m| !m.contains("in a row")), "{messages:?}");
    assert_eq!(messages.len(), 5);
}

#[test]
fn a_handler_over_its_budget_is_reported() {
    let main = r#"ccad.on("model_changed", function() while true do end end)"#;
    let limits = Limits { instructions: 2_000_000, ..Limits::default() };
    let mut s = session_with(vec![load("budget", "slow", main, limits).unwrap()]);
    create_wall(&mut s);
    assert_eq!(walls(&mut s).len(), 1);
    let messages = s.take_extension_messages();
    assert!(messages[0].contains("ran too long"), "{messages:?}");
}

#[test]
fn handlers_run_in_load_then_registration_order() {
    let a = r#"
ccad.on("model_changed", function() print("a1") end)
ccad.on("project_saved", function() print("a saved") end)
ccad.on("model_changed", function() print("a2") end)
"#;
    let b = r#"ccad.on("model_changed", function() print("b1") end)"#;
    let mut s = session_with(vec![
        load("order", "zeta", a, Limits::default()).unwrap(),
        load("order", "alpha", b, Limits::default()).unwrap(),
    ]);
    create_wall(&mut s);
    assert_eq!(s.take_extension_messages(), vec!["[zeta] a1", "[zeta] a2", "[alpha] b1"]);
}

#[test]
fn ccad_on_is_checked_and_only_works_while_loading() {
    let e = load("unknown", "u", r#"ccad.on("model_changd", function() end)"#, Limits::default()).unwrap_err();
    assert!(e.contains("unknown event \"model_changd\"") && e.contains("model_changed"), "{e}");
    let e = load("notfn", "u", r#"ccad.on("model_changed", 3)"#, Limits::default()).unwrap_err();
    assert!(e.contains("not a function"), "{e}");
    let many = "for i = 1, 33 do ccad.on(\"model_changed\", function() end) end";
    let e = load("many", "u", many, Limits::default()).unwrap_err();
    assert!(e.contains("at most 32"), "{e}");
    let mut s = session_with(vec![load("late", "ev", TRACKER, Limits::default()).unwrap()]);
    let e = s.execute("ev__late", json!({})).unwrap_err();
    assert!(e.message.contains("only be called while the plugin loads"), "{}", e.message);
}

#[test]
fn a_plugin_without_handlers_costs_no_journal() {
    let mut s = session_with(vec![load("none", "quiet", "", Limits::default()).unwrap()]);
    create_wall(&mut s);
    assert!(s.take_extension_messages().is_empty());
    assert!(s.doc.take_journal().is_none());
}
