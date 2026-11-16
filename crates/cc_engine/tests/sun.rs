//! The sun of a view: its settings through `view_set`/`view_get`, where it
//! stands (`sun_position`, `sun_frames`), the project's presets and the
//! site's offset from UTC.

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_engine::commands::sun::{sun_arg, sun_json};
use cc_engine::{ErrorCode, Session};
use serde_json::{Value, json};

fn run(s: &mut Session, name: &str, args: Value) -> Value {
    s.execute(name, args).unwrap_or_else(|e| panic!("{name}: {e:?}"))
}

/// The command fails and leaves no undo step.
fn refused(s: &mut Session, name: &str, args: Value) -> ErrorCode {
    let depth = s.doc.undo_depth();
    let code = s.execute(name, args.clone()).map(|v| panic!("{name} {args} should fail, gave {v}")).unwrap_err().code;
    assert_eq!(s.doc.undo_depth(), depth, "{name} {args} left an undo step");
    code
}

fn three_d(s: &mut Session) -> u64 {
    run(s, "view_create", json!({"kind": "three_d"}))["id"].as_u64().unwrap()
}

fn sun_of(s: &mut Session, view: u64) -> Value {
    run(s, "view_get", json!({"view": view}))["sun"].clone()
}

fn set_sun(s: &mut Session, view: u64, sun: Value) -> Value {
    run(s, "view_set", json!({"view": view, "sun": sun}))["sun"].clone()
}

#[test]
fn each_mode_round_trips() {
    let mut s = Session::default();
    let v = three_d(&mut s);
    assert_eq!(sun_of(&mut s, v), Value::Null);

    let still = set_sun(&mut s, v, json!({"mode": "still", "date": "2026-03-20", "time": "09:30"}));
    assert_eq!(still["mode"], "still");
    assert_eq!(still["date"], "2026-03-20");
    assert_eq!(still["time"], "09:30");
    assert!(still.get("from").is_none() && still.get("azimuth").is_none());
    assert_eq!(still["show_path"], true);
    assert_eq!(still["ground"], Value::Null);
    let derived = &still["derived"];
    assert_eq!(derived["date"], "2026-03-20");
    assert_eq!(derived["time"], "09:30");
    assert_eq!(derived["frames"], 1);
    assert_eq!(derived["site_known"], false);
    assert!(derived["sunrise"].is_string() && derived["sunset"].is_string(), "{derived}");
    assert_eq!(derived["above_horizon"], true);

    let single = json!({"mode": "single_day", "date": "2026-12-21", "from": "07:00", "to": "17:00",
        "sunrise_to_sunset": false, "interval": "minutes_30"});
    let got = set_sun(&mut s, v, single.clone());
    for (k, val) in single.as_object().unwrap() {
        assert_eq!(&got[k], val, "{k}");
    }
    assert_eq!(got["derived"]["frames"], 21);
    assert!(got.get("time").is_none());

    let multi = json!({"mode": "multi_day", "from_date": "2026-01-01", "to_date": "2026-12-31", "from": "12:00",
        "to": "12:00", "sunrise_to_sunset": false, "interval": "month"});
    let got = set_sun(&mut s, v, multi.clone());
    for (k, val) in multi.as_object().unwrap() {
        assert_eq!(&got[k], val, "{k}");
    }
    assert_eq!(got["derived"]["frames"], 12);

    let light = set_sun(
        &mut s,
        v,
        json!({"mode": "lighting", "azimuth": -45.0, "altitude": 30.0,
        "relative_to_view": false}),
    );
    assert_eq!(light["azimuth"], 315.0);
    assert_eq!(light["altitude"], 30.0);
    assert_eq!(light["relative_to_view"], false);
    assert_eq!(light["derived"]["azimuth"], 315.0);
    assert_eq!(light["derived"]["frames"], 0);
    assert!(light["derived"].get("time").is_none());
    assert_eq!(light["derived"]["sunrise"], Value::Null);
    assert!(light.get("date").is_none());

    // Null removes the sun.
    assert_eq!(set_sun(&mut s, v, Value::Null), Value::Null);
}

#[test]
fn merge_keeps_untouched_fields_and_mode_switch_carries_the_date() {
    let mut s = Session::default();
    let v = three_d(&mut s);
    set_sun(&mut s, v, json!({"date": "2026-09-01", "time": "15:45", "shadows": true, "frame": 3}));
    let got = set_sun(&mut s, v, json!({"time": "08:00"}));
    assert_eq!(got["mode"], "still");
    assert_eq!(got["date"], "2026-09-01");
    assert_eq!(got["time"], "08:00");
    assert_eq!(got["shadows"], true);
    assert_eq!(got["frame"], 3);
    // Changing the name or scale leaves the sun alone.
    run(&mut s, "view_set", json!({"view": v, "name": "Sun view"}));
    assert_eq!(sun_of(&mut s, v)["time"], "08:00");

    // Still -> single day keeps the date and starts from the mode's defaults.
    let got = set_sun(&mut s, v, json!({"mode": "single_day"}));
    assert_eq!(got["date"], "2026-09-01");
    assert_eq!(got["from"], "06:00");
    assert_eq!(got["interval"], "hour");
    assert_eq!(got["shadows"], true);
    // -> several days: the date is the first one.
    let got = set_sun(&mut s, v, json!({"mode": "multi_day"}));
    assert_eq!(got["from_date"], "2026-09-01");
    assert_eq!(got["to_date"], "2026-12-31");
    // -> still: the first date again.
    let got = set_sun(&mut s, v, json!({"mode": "still"}));
    assert_eq!(got["date"], "2026-09-01");
    assert_eq!(got["time"], "10:00");
    // The same mode keeps its values.
    let got = set_sun(&mut s, v, json!({"mode": "still", "time": "11:11"}));
    assert_eq!(got["date"], "2026-09-01");

    // Undo restores the earlier sun.
    run(&mut s, "undo", json!({}));
    assert_eq!(sun_of(&mut s, v)["time"], "10:00");
}

#[test]
fn bad_input_is_refused_without_an_undo_step() {
    let mut s = Session::default();
    let v = three_d(&mut s);
    set_sun(&mut s, v, json!({"mode": "single_day", "from": "08:00", "to": "16:00"}));
    let bad = [
        json!({"mode": "dusk"}),
        json!({"interval": "fortnight"}),
        json!({"preset": "Nope"}),
        json!({"ground": "No such level"}),
        json!({"date": "2026-02-30"}),
        json!({"date": "21/06/2026"}),
        json!({"from": "8h"}),
        json!({"from": "24:00"}),
        json!({"to": "12:60"}),
        json!({"time": "10:00"}),
        json!({"azimuth": 10.0}),
        json!({"from_date": "2026-01-01"}),
        json!({"interval": "day"}),
        json!({"interval": "month"}),
        json!({"from": "17:00"}),
        json!({"mode": "multi_day", "from_date": "2026-05-01", "to_date": "2026-04-01"}),
        json!({"mode": "lighting", "altitude": 95.0}),
        json!({"mode": "lighting", "altitude": -1.0}),
        json!({"mode": "lighting", "date": "2026-01-01"}),
        json!({"unknown_field": 1}),
    ];
    for sun in bad {
        refused(&mut s, "view_set", json!({"view": v, "sun": sun}));
    }
    // Non-finite numbers cannot come through JSON; the arguments are refused all the same.
    let sun = sun_of(&mut s, v);
    assert_eq!(sun["from"], "08:00");
    assert_eq!(sun["to"], "16:00");

    // A schedule takes no sun.
    let schedule = run(&mut s, "view_create", json!({"kind": "schedule"}))["id"].as_u64().unwrap();
    refused(&mut s, "view_set", json!({"view": schedule, "sun": {"mode": "still"}}));
    refused(&mut s, "view_set", json!({"view": schedule, "sun": null}));
}

#[test]
fn presets() {
    let mut s = Session::default();
    let v = three_d(&mut s);
    assert_eq!(run(&mut s, "sun_presets_list", json!({})), json!([]));
    set_sun(&mut s, v, json!({"mode": "single_day", "date": "2026-12-21", "interval": "minutes_15", "shadows": true}));
    let saved = run(&mut s, "sun_preset_save", json!({"name": " Summer ", "view": v}));
    assert_eq!(saved["name"], "Summer");
    assert_eq!(saved["mode"], "single_day");
    assert_eq!(saved["interval"], "minutes_15");
    assert!(saved.get("shadows").is_none());
    run(&mut s, "sun_preset_save", json!({"name": "Low", "sun": {"mode": "lighting", "altitude": 10.0}}));
    let list = run(&mut s, "sun_presets_list", json!({}));
    assert_eq!(list.as_array().unwrap().len(), 2);
    assert_eq!(list[1]["altitude"], 10.0);

    // Names are unique; replacing must be asked for.
    refused(&mut s, "sun_preset_save", json!({"name": "Summer", "sun": {"mode": "still"}}));
    refused(&mut s, "sun_preset_save", json!({"name": "  ", "sun": {"mode": "still"}}));
    refused(&mut s, "sun_preset_save", json!({"name": "X"}));
    refused(&mut s, "sun_preset_save", json!({"name": "X", "view": v, "sun": {}}));
    let other = three_d(&mut s);
    refused(&mut s, "sun_preset_save", json!({"name": "X", "view": other}));
    run(
        &mut s,
        "sun_preset_save",
        json!({"name": "Summer", "sun": {"mode": "still", "time": "12:00"}, "replace": true}),
    );
    assert_eq!(run(&mut s, "sun_presets_list", json!({}))[0]["time"], "12:00");

    // A view takes a preset; other fields apply on top.
    let got = set_sun(&mut s, other, json!({"preset": "low", "azimuth": 200.0}));
    assert_eq!(got["mode"], "lighting");
    assert_eq!(got["altitude"], 10.0);
    assert_eq!(got["azimuth"], 200.0);
    refused(&mut s, "view_set", json!({"view": other, "sun": {"preset": "Low", "mode": "still"}}));

    // Rename and delete.
    refused(&mut s, "sun_preset_rename", json!({"name": "Low", "new_name": "Summer"}));
    refused(&mut s, "sun_preset_rename", json!({"name": "Nope", "new_name": "Y"}));
    let renamed = run(&mut s, "sun_preset_rename", json!({"name": "Low", "new_name": "Evening"}));
    assert_eq!(renamed["name"], "Evening");
    run(&mut s, "sun_preset_delete", json!({"name": "Summer"}));
    refused(&mut s, "sun_preset_delete", json!({"name": "Summer"}));
    let list = run(&mut s, "sun_presets_list", json!({}));
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["name"], "Evening");
    // One undo step each.
    run(&mut s, "undo", json!({}));
    assert_eq!(run(&mut s, "sun_presets_list", json!({})).as_array().unwrap().len(), 2);
}

#[test]
fn preset_limit() {
    let mut s = Session::default();
    for i in 0..cc_core::sun::MAX_PRESETS {
        run(&mut s, "sun_preset_save", json!({"name": format!("P{i}"), "sun": {}}));
    }
    refused(&mut s, "sun_preset_save", json!({"name": "One more", "sun": {}}));
    // Replacing one is still allowed.
    run(&mut s, "sun_preset_save", json!({"name": "P0", "sun": {"mode": "lighting"}, "replace": true}));
}

#[test]
fn positions_and_frames() {
    let mut s = Session::default();
    // The equinox at the equator on Greenwich: solar noon is about 12:07
    // (the equation of time), the sun then nearly overhead.
    let at = |s: &mut Session, time: &str| {
        let args = json!({"date": "2026-03-20", "time": time, "latitude": 0.0, "longitude": 0.0, "utc_offset": 0.0});
        run(s, "sun_position", args)
    };
    assert!(at(&mut s, "12:00")["altitude"].as_f64().unwrap() > 88.0);
    let p = at(&mut s, "12:07");
    assert!(p["altitude"].as_f64().unwrap() > 89.0, "{p}");
    assert_eq!(p["above_horizon"], true);
    assert_eq!(p["site_known"], true);
    assert!(p["direction"][2].as_f64().unwrap() > 0.99, "{p}");
    assert!(p["sunrise"].is_string() && p["sunset"].is_string());
    // A polar night.
    let p =
        run(&mut s, "sun_position", json!({"date": "2026-12-21", "time": "12:00", "latitude": 80.0, "longitude": 0.0}));
    assert_eq!(p["above_horizon"], false);
    assert_eq!(p["polar"], "night");
    assert_eq!(p["sunrise"], Value::Null);
    // Without a site: the equator at Greenwich.
    assert_eq!(run(&mut s, "sun_position", json!({"date": "2026-03-20", "time": "12:00"}))["site_known"], false);
    refused(&mut s, "sun_position", json!({"date": "2026-03-20"}));
    refused(&mut s, "sun_position", json!({"date": "2026-03-20", "time": "12:00", "latitude": 91.0}));
    refused(&mut s, "sun_position", json!({"date": "2026-03-20", "time": "12:00", "utc_offset": 15.0}));

    let v = three_d(&mut s);
    refused(&mut s, "sun_position", json!({"view": v}));
    refused(&mut s, "sun_frames", json!({"view": v}));
    set_sun(
        &mut s,
        v,
        json!({"mode": "single_day", "date": "2026-03-20", "from": "06:00", "to": "18:00",
        "interval": "hour", "frame": 6}),
    );
    refused(&mut s, "sun_position", json!({"view": v, "date": "2026-03-20"}));
    let p = run(&mut s, "sun_position", json!({"view": v}));
    assert_eq!(p["time"], "12:00");
    let frames = run(&mut s, "sun_frames", json!({"view": v}));
    assert_eq!(frames["count"], 13);
    assert_eq!(frames["frames"].as_array().unwrap().len(), 13);
    assert_eq!(frames["frames"][6]["time"], "12:00");
    assert_eq!(frames["frames"][6]["altitude"], p["altitude"]);
    let few = run(&mut s, "sun_frames", json!({"view": v, "limit": 3}));
    assert_eq!(few["count"], 13);
    assert_eq!(few["frames"].as_array().unwrap().len(), 3);
    assert_eq!(few["frames"][2]["index"], 2);

    // Every day of a month, hourly from 8 to 10: three frames a day.
    set_sun(
        &mut s,
        v,
        json!({"mode": "multi_day", "from_date": "2026-01-01", "to_date": "2026-01-31",
        "from": "08:00", "to": "10:00", "interval": "hour"}),
    );
    assert_eq!(run(&mut s, "sun_frames", json!({"view": v}))["count"], 93);
}

#[test]
fn site_utc_offset() {
    let mut s = Session::default();
    refused(&mut s, "site_set", json!({"utc_offset": -3.0}));
    let out = run(&mut s, "site_set", json!({"latitude": -23.5, "longitude": -46.6}));
    assert_eq!(out["site"]["utc_offset"], Value::Null);
    assert_eq!(out["site"]["utc_offset_used"], -3.0);
    let out = run(&mut s, "site_set", json!({"utc_offset": 5.5}));
    assert_eq!(out["site"]["utc_offset"], 5.5);
    assert_eq!(out["site"]["utc_offset_used"], 5.5);
    refused(&mut s, "site_set", json!({"utc_offset": 14.5}));
    refused(&mut s, "site_set", json!({"utc_offset": -12.5}));
    refused(&mut s, "site_set", json!({"clear": true, "utc_offset": null}));
    let out = run(&mut s, "site_set", json!({"utc_offset": null}));
    assert_eq!(out["site"]["utc_offset"], Value::Null);
    assert_eq!(run(&mut s, "site_get", json!({}))["site"]["utc_offset_used"], -3.0);
    // The views' suns follow the site.
    let v = three_d(&mut s);
    assert_eq!(set_sun(&mut s, v, json!({}))["derived"]["site_known"], true);
}

#[test]
fn deleting_the_ground_level_clears_it() {
    let mut s = Session::default();
    let v = three_d(&mut s);
    let level = run(&mut s, "level_create", json!({"name": "Terrace", "elevation": 9.0}))["id"].as_u64().unwrap();
    let got = set_sun(&mut s, v, json!({"shadows": true, "ground": "Terrace"}));
    assert_eq!(got["ground"], level);
    run(&mut s, "level_delete", json!({"level": level}));
    assert_eq!(sun_of(&mut s, v)["ground"], Value::Null);
    run(&mut s, "undo", json!({}));
    assert_eq!(sun_of(&mut s, v)["ground"], level);
    assert_eq!(set_sun(&mut s, v, json!({"ground": null}))["ground"], Value::Null);
}

#[test]
fn sun_arg_inverts_the_settings() {
    let mut s = Session::default();
    let a = three_d(&mut s);
    let b = three_d(&mut s);
    let level = run(&mut s, "level_create", json!({"name": "Roof", "elevation": 6.0}))["id"].as_u64().unwrap();
    let suns = [
        json!({"mode": "still", "date": "2027-02-03", "time": "16:20", "frame": 2, "show_path": false, "ground": level}),
        json!({"mode": "single_day", "date": "2026-07-04", "from": "05:15", "to": "19:45", "sunrise_to_sunset": true,
            "interval": "minutes_45", "shadows": true}),
        json!({"mode": "multi_day", "from_date": "2026-03-01", "to_date": "2027-03-01", "from": "09:00",
            "to": "15:00", "interval": "week", "frame": 40}),
        json!({"mode": "lighting", "azimuth": 359.5, "altitude": 0.0, "relative_to_view": false}),
    ];
    // Each mode, applied over every other one, comes back the same.
    for sun in &suns {
        set_sun(&mut s, a, sun.clone());
        let settings = s.project().views[&cc_core::ViewId(a)].sun.clone().unwrap();
        let arg = sun_arg(s.project(), &settings);
        for start in &suns {
            set_sun(&mut s, b, start.clone());
            run(&mut s, "view_set", json!({"view": b, "sun": serde_json::to_value(&arg).unwrap()}));
            let back = s.project().views[&cc_core::ViewId(b)].sun.clone().unwrap();
            assert_eq!(back, settings, "{sun} over {start}");
        }
        // The friendly form is what view_get shows, without `derived`.
        let mut shown = sun_of(&mut s, a);
        shown.as_object_mut().unwrap().remove("derived");
        assert_eq!(shown, sun_json(s.project(), &settings));
    }
    // From no sun at all.
    run(&mut s, "view_set", json!({"view": b, "sun": null}));
    let settings = s.project().views[&cc_core::ViewId(a)].sun.clone().unwrap();
    let arg = sun_arg(s.project(), &settings);
    run(&mut s, "view_set", json!({"view": b, "sun": serde_json::to_value(&arg).unwrap()}));
    assert_eq!(s.project().views[&cc_core::ViewId(b)].sun.clone().unwrap(), settings);
}

/// The sun's shadows in a view's drawing: black veils that belong to no element.
fn shadows(s: &mut Session, view: u64) -> usize {
    let d = s.drawing(cc_core::ViewId(view)).unwrap();
    let veil =
        |p: &&cc_views::Prim| matches!(p, cc_views::Prim::Fill { color: [0, 0, 0, a], element: None, .. } if *a > 0);
    d.prims.iter().filter(veil).count()
}

/// A sun set to cast shadows casts them in plans and elevations drawn in
/// the shaded style, not in hidden line; what the view hides, for good or
/// for now, casts none; exports carry them.
#[test]
fn shadows_are_cast_in_plans_and_elevations() {
    let mut s = Session::default();
    // A house of four walls on a floor, and a screen wall 2 m off each side.
    for (a, b) in [([0, 0], [6, 0]), ([6, 0], [6, 4]), ([6, 4], [0, 4]), ([0, 4], [0, 0])] {
        run(&mut s, "wall_create", json!({"start": a, "end": b}));
    }
    run(&mut s, "floor_create", json!({"boundary": [[0, 0], [6, 0], [6, 4], [0, 4]]}));
    for (a, b) in [([2, -2], [4, -2]), ([8, 1], [8, 3]), ([4, 6], [2, 6]), ([-2, 3], [-2, 1])] {
        run(&mut s, "wall_create", json!({"start": a, "end": b}));
    }
    let first = |s: &mut Session, category: &str| {
        run(s, "views_list", json!({"category": category}))[0]["id"].as_u64().unwrap()
    };
    let (plan, elevation) = (first(&mut s, "floor_plan"), first(&mut s, "elevation"));
    // A light from behind the viewer's left shoulder (in the plan, from the south-west).
    let sun = json!({"mode": "lighting", "azimuth": 200, "altitude": 40, "relative_to_view": true, "show_path": false});
    for view in [plan, elevation] {
        run(&mut s, "view_set", json!({"view": view, "visual_style": "shaded", "sun": sun}));
        assert_eq!(shadows(&mut s, view), 0, "shadows are off by default");
        let set = set_sun(&mut s, view, json!({"shadows": true}));
        assert_eq!(set["shadows"], true);
        let cast = shadows(&mut s, view);
        assert!(cast > 0, "view {view}");
        // Hidden line draws none; back in the shaded style they return.
        run(&mut s, "view_set", json!({"view": view, "visual_style": "hidden_line"}));
        assert_eq!(shadows(&mut s, view), 0);
        run(&mut s, "view_set", json!({"view": view, "visual_style": "shaded"}));
        assert_eq!(shadows(&mut s, view), cast);
        // The walls hidden for now, then for good: nothing casts (the floor lies on the ground).
        run(&mut s, "view_temporary", json!({"view": view, "action": "hide_category", "categories": ["wall"]}));
        assert_eq!(shadows(&mut s, view), 0, "walls hidden for now");
        run(&mut s, "view_temporary", json!({"view": view, "action": "reset"}));
        assert_eq!(shadows(&mut s, view), cast);
        let walls: Vec<u64> = run(&mut s, "elements_list", json!({"category": "wall"}))
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["id"].as_u64().unwrap())
            .collect();
        run(&mut s, "element_hide_in_view", json!({"view": view, "ids": walls}));
        assert_eq!(shadows(&mut s, view), 0, "walls hidden");
        run(&mut s, "element_hide_in_view", json!({"view": view, "ids": walls, "hidden": false}));
        assert_eq!(shadows(&mut s, view), cast);
        // A sun at the horizon casts none.
        set_sun(&mut s, view, json!({"altitude": 0}));
        assert_eq!(shadows(&mut s, view), 0);
        set_sun(&mut s, view, json!({"altitude": 40}));
    }
    // The exported drawing carries them: black fills that let the faces show through.
    let dir = std::env::temp_dir().join(format!("cc_shadows_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("plan.svg");
    run(&mut s, "view_export", json!({"view": plan, "path": path.to_str().unwrap()}));
    let svg = std::fs::read_to_string(&path).unwrap();
    assert!(svg.contains(r#"fill="rgb(0,0,0)" fill-opacity="#), "no veil in the SVG");
    let _ = std::fs::remove_dir_all(&dir);
}

/// The elements the sun's shadows of a view's drawing lie on: a veil comes
/// right after the fill of its face (`None`: on the ground, before any face).
fn shadowed(s: &mut Session, view: u64) -> Vec<Option<u64>> {
    let d = s.drawing(cc_core::ViewId(view)).unwrap();
    let mut on = None;
    let mut out = Vec::new();
    for p in &d.prims {
        match p {
            cc_views::Prim::Fill { color: [0, 0, 0, a], element: None, .. } if *a > 0 => out.push(on),
            cc_views::Prim::Fill { element: Some(e), .. } => on = Some(e.0),
            _ => {}
        }
    }
    out
}

/// Each view casts its shadows as it is read. A house with eaves, an open
/// terrace beside it and an annex behind, inside a garden wall: the plan
/// casts on the ground and on the terrace, not on the floor indoors; the
/// section through the house leaves its room alone and casts the annex's
/// eave on the annex's front, though the garden wall closes the garden
/// around both; the elevation casts the house's eave on its front; the
/// ceiling plan casts nothing.
#[test]
fn shadows_follow_how_each_view_is_read() {
    let mut s = Session::default();
    let id = |v: Value| v["id"].as_u64().unwrap();
    let loop_of = |s: &mut Session, (x0, y0, x1, y1): (f64, f64, f64, f64)| -> Vec<u64> {
        let corners = [[x0, y0], [x1, y0], [x1, y1], [x0, y1]];
        (0..4).map(|i| id(run(s, "wall_create", json!({"start": corners[i], "end": corners[(i + 1) % 4]})))).collect()
    };
    let house = loop_of(&mut s, (0.0, 0.0, 6.0, 4.0));
    let annex = loop_of(&mut s, (12.0, 6.0, 16.0, 9.0));
    let garden = loop_of(&mut s, (-4.0, -4.0, 20.0, 13.0));
    let rect = |x0: f64, y0: f64, x1: f64, y1: f64| json!([[x0, y0], [x1, y0], [x1, y1], [x0, y1]]);
    let floor = id(run(&mut s, "floor_create", json!({"boundary": rect(0.0, 0.0, 6.0, 4.0)})));
    let terrace = id(run(&mut s, "floor_create", json!({"boundary": rect(6.0, 0.0, 9.0, 4.0)})));
    run(&mut s, "roof_create", json!({"boundary": rect(0.0, 0.0, 6.0, 4.0), "overhang": 0.6}));
    run(&mut s, "roof_create", json!({"boundary": rect(12.0, 6.0, 16.0, 9.0), "overhang": 0.6}));

    // What is indoors: the rooms under their roofs; not the garden, under an eave or not.
    let (project, geom) = s.project_and_geometry();
    let indoors = cc_views::Indoors::of(project, geom, None, cc_core::DetailLevel::Medium);
    assert!(!indoors.failed);
    let at = |x: f64, y: f64| glam::DVec3::new(x, y, 1.0);
    assert!(indoors.holds(at(3.0, 2.0)) && indoors.holds(at(14.0, 7.5)), "the rooms");
    assert!(!indoors.holds(at(3.0, -0.4)) && !indoors.holds(at(14.0, 5.6)), "under the eaves");
    assert!(!indoors.holds(at(-2.0, 2.0)) && !indoors.holds(at(7.5, 2.0)), "the garden, the terrace");
    assert!(!indoors.holds(at(30.0, 2.0)), "outside the garden wall");

    let first = |s: &mut Session, category: &str| {
        run(s, "views_list", json!({"category": category}))[0]["id"].as_u64().unwrap()
    };
    let plan = first(&mut s, "floor_plan");
    let elevation = id(run(&mut s, "view_create", json!({"kind": "elevation", "direction": "south"})));
    // Looking north from a line through the house's room.
    let section =
        id(run(&mut s, "view_create", json!({"kind": "section", "start": [22, 2], "end": [-6, 2], "depth": 14})));
    let ceiling = id(run(&mut s, "view_create", json!({"kind": "ceiling_plan"})));
    // The sun in the south-south-west, low.
    let sun = json!({"mode": "lighting", "azimuth": 200, "altitude": 25, "relative_to_view": false,
        "shadows": true, "show_path": false});
    for view in [plan, elevation, section, ceiling] {
        run(&mut s, "view_set", json!({"view": view, "visual_style": "shaded", "sun": sun}));
    }
    let count = |on: &[Option<u64>], ids: &[u64]| on.iter().filter(|e| e.is_some_and(|e| ids.contains(&e))).count();

    let on = shadowed(&mut s, plan);
    assert!(on.contains(&None), "on the ground");
    assert!(count(&on, &[terrace]) > 0, "the house's shadow on the terrace: {on:?}");
    assert_eq!(count(&on, &[floor]), 0, "none indoors: {on:?}");

    let on = shadowed(&mut s, section);
    assert_eq!(count(&on, &house), 0, "the room cut keeps its plain shading: {on:?}");
    assert!(count(&on, &annex) > 0, "the annex's eave on its front: {on:?}");
    assert!(count(&on, &garden) > 0, "the house's shadow on the garden wall: {on:?}");

    let on = shadowed(&mut s, elevation);
    assert!(count(&on, &house) > 0, "the eave on the house's front: {on:?}");
    assert!(count(&on, &annex) > 0);

    assert!(shadowed(&mut s, ceiling).is_empty(), "a ceiling plan draws none");
    // The ground under the house's eave is no room: in the plan it keeps
    // the shadow, right up to the wall on the side away from the sun.
    let d = s.drawing(cc_core::ViewId(plan)).unwrap();
    let shaded = |x: f64, y: f64| {
        d.prims.iter().any(|p| {
            matches!(p, cc_views::Prim::Fill { pts, color: [0, 0, 0, a], element: None, .. }
                if *a > 0 && cc_geom::polygon::contains(pts, glam::DVec2::new(x, y)))
        })
    };
    assert!(shaded(3.0, 4.3) && !shaded(3.0, 2.0), "behind the house, not in it");
    // With the roofs hidden in the section, nothing covers the rooms:
    // they are no longer indoors and the sun gets in.
    let roofs: Vec<u64> = run(&mut s, "elements_list", json!({"category": "roof"}))
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["id"].as_u64().unwrap())
        .collect();
    run(&mut s, "element_hide_in_view", json!({"view": section, "ids": roofs}));
    let on = shadowed(&mut s, section);
    assert!(count(&on, &house) > 0, "{on:?}");
}

/// The bounds of points: (min, max).
fn bounds_of(pts: &[glam::DVec2]) -> (glam::DVec2, glam::DVec2) {
    let lo = pts.iter().fold(glam::DVec2::splat(f64::INFINITY), |a, p| a.min(*p));
    let hi = pts.iter().fold(glam::DVec2::splat(f64::NEG_INFINITY), |a, p| a.max(*p));
    (lo, hi)
}

/// The conventional sun of façade drawings: in the south elevation, a
/// short wall standing in front of a long one casts its shadow on it down
/// and to the right, the shadow's right edge as far right of the short
/// wall as its front stands in front of the long wall, and its top as far
/// down as its back stands out.
#[test]
fn the_conventional_sun_casts_down_and_to_the_right() {
    let mut s = Session::default();
    let id = |v: Value| v["id"].as_u64().unwrap();
    let back = id(run(&mut s, "wall_create", json!({"start": [0, 0], "end": [10, 0]})));
    let front = id(run(&mut s, "wall_create", json!({"start": [4, -2], "end": [5, -2]})));
    let elevation = id(run(&mut s, "view_create", json!({"kind": "elevation", "direction": "south"})));
    let sun = set_sun(&mut s, elevation, json!({"mode": "conventional", "shadows": true, "show_path": false}));
    assert_eq!(sun["mode"], "lighting");
    assert_eq!(sun["azimuth"], 225.0);
    assert!((sun["altitude"].as_f64().unwrap() - 35.264_389_682_754_654).abs() < 1e-9);
    assert_eq!(sun["relative_to_view"], true);
    assert_eq!(sun["derived"]["conventional"], true);
    // Seen from the south looking north, the sun stands in the south-west.
    assert_eq!(sun["derived"]["azimuth"], 225.0);
    run(&mut s, "view_set", json!({"view": elevation, "visual_style": "shaded"}));

    let thickness = |s: &mut Session, wall: u64| {
        run(s, "element_get", json!({"id": wall}))["derived"]["thickness"].as_f64().unwrap()
    };
    let (t_back, t_front) = (thickness(&mut s, back), thickness(&mut s, front));
    let d = s.drawing(cc_core::ViewId(elevation)).unwrap();
    let fills_of = |e: u64| -> Vec<glam::DVec2> {
        d.prims
            .iter()
            .filter_map(|p| match p {
                cc_views::Prim::Fill { pts, element: Some(x), .. } if x.0 == e => Some(pts.clone()),
                _ => None,
            })
            .flatten()
            .collect()
    };
    let (lo, hi) = bounds_of(&fills_of(front));
    // Drawing units per meter, from the short wall's 1 m length.
    let k = hi.x - lo.x;
    assert!(k > 0.0);
    let veils: Vec<glam::DVec2> = d
        .prims
        .iter()
        .filter_map(|p| match p {
            cc_views::Prim::Fill { pts, color: [0, 0, 0, a], element: None, .. } if *a > 0 => Some(pts.clone()),
            _ => None,
        })
        .flatten()
        .collect();
    assert!(!veils.is_empty(), "a shadow");
    let (_, shadow_hi) = bounds_of(&veils);
    // The front of the short wall stands 2 m less the half thicknesses out.
    let out_front = 2.0 + t_front / 2.0 - t_back / 2.0;
    let out_back = out_front - t_front;
    let across = (shadow_hi.x - hi.x) / k;
    let down = (hi.y - shadow_hi.y) / k;
    assert!((across - out_front).abs() < 1e-3, "right by {across}, not {out_front}");
    assert!((down - out_back).abs() < 1e-3, "down by {down}, not {out_back}");

    // Back to another mode and to the conventional sun again: its direction is reset.
    set_sun(&mut s, elevation, json!({"azimuth": 100}));
    assert_eq!(sun_of(&mut s, elevation)["derived"]["conventional"], false);
    let again = set_sun(&mut s, elevation, json!({"mode": "Conventional"}));
    assert_eq!(again["azimuth"], 225.0);
    assert_eq!(again["derived"]["conventional"], true);
    assert_eq!(
        refused(&mut s, "view_set", json!({"view": elevation, "sun": {"mode": "conventional", "date": "2026-01-01"}})),
        ErrorCode::InvalidArguments
    );
}

/// A view's shading: `view_get` shows it, `view_set` patches it in one
/// undo step and refuses values out of range, unknown fields and views
/// without a sun; it lightens the shade and the shadows of the drawing.
#[test]
fn shading_is_set_per_view() {
    let mut s = Session::default();
    let id = |v: Value| v["id"].as_u64().unwrap();
    run(&mut s, "wall_create", json!({"start": [0, 0], "end": [10, 0]}));
    run(&mut s, "wall_create", json!({"start": [4, -2], "end": [5, -2]}));
    let elevation = id(run(&mut s, "view_create", json!({"kind": "elevation", "direction": "south"})));
    let view = run(&mut s, "view_get", json!({"view": elevation}));
    assert_eq!(view["shading"], json!({"ambient": 0.6, "shadow": 0.55}));
    let sun = json!({"mode": "conventional", "shadows": true, "show_path": false});
    run(&mut s, "view_set", json!({"view": elevation, "visual_style": "shaded", "sun": sun}));
    let darkest = |s: &mut Session| {
        let d = s.drawing(cc_core::ViewId(elevation)).unwrap();
        d.prims
            .iter()
            .filter_map(|p| match p {
                cc_views::Prim::Fill { color: [0, 0, 0, a], element: None, .. } => Some(*a),
                _ => None,
            })
            .max()
            .unwrap_or(0)
    };
    let before = darkest(&mut s);
    assert!(before > 0);
    let depth = s.doc.undo_depth();
    let out = run(&mut s, "view_set", json!({"view": elevation, "shading": {"shadow": 0.3}}));
    assert_eq!(out["shading"], json!({"ambient": 0.6, "shadow": 0.3}));
    assert_eq!(s.doc.undo_depth(), depth + 1);
    let lighter = darkest(&mut s);
    assert!(0 < lighter && lighter < before, "{lighter} < {before}");
    run(&mut s, "view_set", json!({"view": elevation, "shading": {"ambient": 0.8}}));
    assert_eq!(run(&mut s, "view_get", json!({"view": elevation}))["shading"], json!({"ambient": 0.8, "shadow": 0.3}));
    run(&mut s, "view_set", json!({"view": elevation, "shading": {"shadow": 0}}));
    assert_eq!(darkest(&mut s), 0, "no visible shadow");
    for bad in [json!({"ambient": 0.2}), json!({"shadow": 1.5}), json!({"ambient": "x"}), json!({"light": 1})] {
        assert_eq!(
            refused(&mut s, "view_set", json!({"view": elevation, "shading": bad})),
            ErrorCode::InvalidArguments
        );
    }
    let schedule = id(run(&mut s, "view_create", json!({"kind": "schedule", "category": "wall"})));
    assert_eq!(run(&mut s, "view_get", json!({"view": schedule}))["shading"], Value::Null);
    refused(&mut s, "view_set", json!({"view": schedule, "shading": {"ambient": 0.8}}));
    // Undone with its step; saved and loaded with the project, repaired when out of range.
    s.doc.undo(); // the schedule
    assert_eq!(s.project().views[&cc_core::ViewId(elevation)].shading.shadow, 0.0);
    s.doc.undo();
    assert_eq!(s.project().views[&cc_core::ViewId(elevation)].shading.shadow, 0.3);
    let text = cc_core::to_ron(s.project()).unwrap();
    let (back, repairs) = cc_core::from_ron_with_repairs(&text).unwrap();
    assert_eq!(repairs, 0);
    assert_eq!(back.views[&cc_core::ViewId(elevation)].shading, cc_core::ShadingSettings { ambient: 0.8, shadow: 0.3 });
    let broken = text.replace("ambient: 0.8", "ambient: 5.0");
    assert_ne!(broken, text, "the file holds the shading");
    let (back, repairs) = cc_core::from_ron_with_repairs(&broken).unwrap();
    assert!(repairs >= 1);
    assert_eq!(back.views[&cc_core::ViewId(elevation)].shading.ambient, 1.0);
}
