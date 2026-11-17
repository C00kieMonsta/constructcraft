//! Walls drawn on a face or core line join at their corners like walls drawn
//! on their centerline (issue 165).

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_engine::Session;
use serde_json::{Value, json};

fn run(s: &mut Session, name: &str, args: Value) -> Value {
    s.execute(name, args).unwrap_or_else(|e| panic!("{name}: {e:?}"))
}

/// The template's 25 cm wall type: 2.5 cm finish, 20 cm core, 2.5 cm finish.
fn layered_type(s: &mut Session) -> u64 {
    let types = run(s, "types_list", json!({"category": "wall"}));
    for t in types.as_array().unwrap() {
        let id = t["id"].as_u64().unwrap();
        let layers = run(s, "type_get", json!({"type": id}))["params"]["layers"].clone();
        let thicknesses: Vec<f64> =
            layers.as_array().into_iter().flatten().map(|l| l["thickness"].as_f64().unwrap()).collect();
        if thicknesses.len() == 3 && (thicknesses.iter().sum::<f64>() - 0.25).abs() < 1e-9 {
            return id;
        }
    }
    panic!("no layered 25 cm wall type in {types}");
}

fn volume(s: &mut Session, id: u64) -> f64 {
    run(s, "element_get", json!({"id": id}))["derived"]["volume"].as_f64().unwrap()
}

#[test]
fn corners_close_whatever_the_location_line() {
    // Drawn clockwise, so the exterior (left) face is outside; the outer
    // face lies `outside` beyond the drawn line.
    let lines = [
        ("FinishExterior", 0.0),
        ("CoreExterior", 0.025),
        ("Centerline", 0.125),
        ("CoreCenter", 0.125),
        ("CoreInterior", 0.225),
        ("FinishInterior", 0.25),
    ];
    for (line, outside) in lines {
        let mut s = Session::default();
        let ty = layered_type(&mut s);
        let corners = [[0.0, 0.0], [5.0, 0.0], [5.0, -4.0], [0.0, -4.0]];
        let walls: Vec<u64> = (0..4)
            .map(|i| {
                let w = run(
                    &mut s,
                    "wall_create",
                    json!({"start": corners[i], "end": corners[(i + 1) % 4], "type": ty,
                           "location_line": line, "height": 3.0}),
                );
                w["id"].as_u64().unwrap()
            })
            .collect();
        // The ring between the outer rectangle and the inner one, 0.25 m in.
        let (w, d) = (5.0 + 2.0 * outside, 4.0 + 2.0 * outside);
        let area = w * d - (w - 0.5) * (d - 0.5);
        let total: f64 = walls.iter().map(|&id| volume(&mut s, id)).sum();
        assert!((total - area * 3.0).abs() < 1e-6, "{line}: {total} != {}", area * 3.0);
    }
}

#[test]
fn an_open_corner_on_the_core_line_is_mitered() {
    let mut s = Session::default();
    let ty = layered_type(&mut s);
    let a = run(
        &mut s,
        "wall_create",
        json!({"start": [0, 0], "end": [5, 0], "type": ty, "location_line": "CoreExterior", "height": 3.0}),
    );
    let b = run(
        &mut s,
        "wall_create",
        json!({"start": [5, 0], "end": [5, -4], "type": ty, "location_line": "CoreExterior", "height": 3.0}),
    );
    // Faces 0.025 m outside and 0.225 m inside the drawn lines; the corner
    // runs out to the outer faces: an L of arms 5.025 and 4.025 m, 0.25 m thick.
    let area = 0.25 * (5.025 + 4.025) - 0.25 * 0.25;
    let total = volume(&mut s, a["id"].as_u64().unwrap()) + volume(&mut s, b["id"].as_u64().unwrap());
    assert!((total - area * 3.0).abs() < 1e-6, "{total} != {}", area * 3.0);
}

/// The wall tool previews the body at the type's `location_offset`, so it must
/// be where the created wall's body ends up (issue 198).
#[test]
fn the_type_offset_places_the_body_like_the_wall() {
    use cc_core::{ElementId, ElementKind, LocationLine};
    for line in LocationLine::ALL {
        let mut s = Session::default();
        let ty = layered_type(&mut s);
        let name = serde_json::to_value(line).unwrap();
        let w = run(
            &mut s,
            "wall_create",
            json!({"start": [0, 0], "end": [5, 0], "type": ty, "location_line": name, "height": 3.0}),
        );
        let project = s.project();
        let element = project.element(ElementId(w["id"].as_u64().unwrap())).unwrap();
        let ElementKind::Wall(wall) = &element.kind else { panic!("not a wall") };
        let offset = project.element_type(element.type_id).unwrap().location_offset(line);
        // Drawn along +x, so the centerline is `offset` to the right (−y).
        let (start, end) = project.wall_axis(element, wall);
        assert!((start.y + offset).abs() < 1e-9 && (end.y + offset).abs() < 1e-9, "{line:?}: {start} {end}");
        if line == LocationLine::FinishExterior {
            assert!((offset - 0.125).abs() < 1e-9, "the exterior face on the line: {offset}");
        }
    }
}

/// Two rings of walls drawn on the same corners, one wholly below the other (a
/// foundation wall on its centerline under a parapet on its exterior face): each
/// ring closes its own corners, as if the other were not there (issue 387).
#[test]
fn rings_one_above_the_other_join_each_on_its_own() {
    let corners = [[0.0, 0.0], [5.0, 0.0], [5.0, -4.0], [0.0, -4.0]];
    let ring = |s: &mut Session, ty: u64, line: &str, base: f64, height: f64| -> Vec<u64> {
        (0..4)
            .map(|i| {
                let args = json!({"start": corners[i], "end": corners[(i + 1) % 4], "type": ty,
                                  "location_line": line, "base_offset": base, "height": height});
                run(s, "wall_create", args)["id"].as_u64().unwrap()
            })
            .collect()
    };
    let total = |s: &mut Session, walls: &[u64]| walls.iter().map(|&id| volume(s, id)).sum::<f64>();
    // The volumes of each ring alone.
    let alone = |line: &str, base: f64, height: f64| {
        let mut s = Session::default();
        let ty = layered_type(&mut s);
        let walls = ring(&mut s, ty, line, base, height);
        total(&mut s, &walls)
    };
    let (low, high) = (alone("Centerline", -1.0, 0.6), alone("FinishExterior", -0.15, 0.17));

    let mut s = Session::default();
    let ty = layered_type(&mut s);
    let below = ring(&mut s, ty, "Centerline", -1.0, 0.6);
    let above = ring(&mut s, ty, "FinishExterior", -0.15, 0.17);
    assert!((total(&mut s, &below) - low).abs() < 1e-9, "{} != {low}", total(&mut s, &below));
    assert!((total(&mut s, &above) - high).abs() < 1e-9, "{} != {high}", total(&mut s, &above));
    // The closed ring of 25 cm walls on the centerline: 5.25 x 4.25 less 4.75 x 3.75.
    assert!((low - (5.25 * 4.25 - 4.75 * 3.75) * 0.6).abs() < 1e-6, "{low}");
}

/// A wall type may make its walls structural; `wall_create` takes the type's
/// value unless told otherwise, and each wall keeps its own (issue 387).
#[test]
fn a_wall_type_can_make_its_walls_structural() {
    let mut s = Session::default();
    let ty = layered_type(&mut s);
    let structural =
        |s: &mut Session, id: u64| run(s, "element_get", json!({"id": id}))["params"]["structural"].clone();
    let wall = |s: &mut Session, extra: Value| {
        let mut args = json!({"start": [0, 0], "end": [5, 0], "type": ty});
        args.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        run(s, "wall_create", args)["id"].as_u64().unwrap()
    };
    let plain = wall(&mut s, json!({}));
    assert_eq!(structural(&mut s, plain), json!(false));
    run(&mut s, "type_set", json!({"type": ty, "params": {"structural": true}}));
    assert_eq!(run(&mut s, "type_get", json!({"type": ty}))["params"]["structural"], json!(true));
    let made = wall(&mut s, json!({}));
    assert_eq!(structural(&mut s, made), json!(true));
    let told = wall(&mut s, json!({"structural": false}));
    assert_eq!(structural(&mut s, told), json!(false));
    // The walls already made keep their value.
    assert_eq!(structural(&mut s, plain), json!(false));
}
