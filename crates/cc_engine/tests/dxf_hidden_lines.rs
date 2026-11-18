//! DXF export of elevations, sections and 3D views as their visible
//! linework (issue #394): lines hidden behind opaque surfaces removed,
//! surface fills left out, lines joined into polylines.

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_engine::Session;
use cc_views::Prim;
use cc_views::dxf::{DxfScale, default_layer, to_dxf_with};
use glam::DVec2;
use serde_json::{Value, json};

fn run(s: &mut Session, name: &str, args: Value) -> Value {
    s.execute(name, args).unwrap_or_else(|e| panic!("{name}: {e:?}"))
}

fn id(v: &Value) -> u64 {
    v["id"].as_u64().unwrap()
}

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("ccad-dxf-hidden-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

/// The group code / value pairs of a DXF document.
fn pairs(bytes: &[u8]) -> Vec<(i32, String)> {
    let s: String = bytes.iter().map(|&b| char::from(b)).collect();
    let lines: Vec<&str> = s.lines().collect();
    lines.chunks(2).map(|c| (c[0].trim().parse().unwrap(), c[1].trim().to_string())).collect()
}

fn count(p: &[(i32, String)], what: &str) -> usize {
    p.iter().filter(|(c, v)| *c == 0 && v == what).count()
}

/// The segments of the entities section's lines and polylines, in millimeters.
fn segments(p: &[(i32, String)]) -> Vec<(DVec2, DVec2)> {
    let start = p.iter().position(|e| *e == (2, "ENTITIES".into())).unwrap();
    let mut out = Vec::new();
    let mut i = start;
    let real = |code: i32, from: usize| -> f64 {
        p[from + 1..].iter().take_while(|(c, _)| *c != 0).find(|(c, _)| *c == code).unwrap().1.parse().unwrap()
    };
    while i < p.len() {
        match (p[i].0, p[i].1.as_str()) {
            (0, "LINE") => {
                out.push((DVec2::new(real(10, i), real(20, i)), DVec2::new(real(11, i), real(21, i))));
            }
            (0, "POLYLINE") => {
                let closed = real(70, i) as i32 & 1 == 1;
                let mut pts = Vec::new();
                let mut j = i + 1;
                while j < p.len() && p[j] != (0, "SEQEND".into()) {
                    if p[j] == (0, "VERTEX".into()) {
                        pts.push(DVec2::new(real(10, j), real(20, j)));
                    }
                    j += 1;
                }
                for w in pts.windows(2) {
                    out.push((w[0], w[1]));
                }
                if closed && pts.len() > 2 {
                    out.push((*pts.last().unwrap(), pts[0]));
                }
                i = j;
            }
            _ => {}
        }
        i += 1;
    }
    out
}

/// Whether the segments cover `a`–`b` (millimeters): some segment holds both ends on its line.
fn covered(segs: &[(DVec2, DVec2)], a: DVec2, b: DVec2) -> bool {
    let on = |p: DVec2, (s, e): (DVec2, DVec2)| cc_geom::polygon::closest_on_segment(s, e, p).0.distance(p) < 0.01;
    segs.iter().any(|&seg| on(a, seg) && on(b, seg))
}

/// Whether any segment passes through `p` (millimeters).
fn touches(segs: &[(DVec2, DVec2)], p: DVec2) -> bool {
    segs.iter().any(|&(s, e)| cc_geom::polygon::closest_on_segment(s, e, p).0.distance(p) < 0.01)
}

/// The DXF as the export wrote it before hidden lines were removed: the
/// printed drawing with the export's layers.
fn raw_dxf(s: &mut Session, view: u64) -> Vec<u8> {
    let id = cc_core::ViewId(view);
    let scale = s.project().view(id).unwrap().scale;
    let drawing = s.printed_drawing(id).unwrap();
    let project = s.project();
    let blocks =
        cc_views::detail_component::dxf_blocks(project, id, &drawing, scale, &|e| format!("{:?}", e.category()));
    let layer = |prim: &Prim| match prim.element().and_then(|e| project.element(e)) {
        Some(cc_core::Element { kind: cc_core::ElementKind::DetailLine(l), .. }) => l.style.code(&project.line_styles),
        Some(cc_core::Element {
            kind: cc_core::ElementKind::FilledRegion(r) | cc_core::ElementKind::MaskingRegion(r),
            ..
        }) if r.boundary.is_some() => r.boundary.map(|b| b.code(&project.line_styles)).unwrap_or_default(),
        Some(e) => format!("{:?}", e.category()),
        None => default_layer(prim).to_string(),
    };
    to_dxf_with(&drawing, DxfScale::model(scale), &layer, &blocks)
}

/// The vertical edges of an element's faces in a view's drawing (meters).
fn vertical_lines(s: &mut Session, view: u64, element: u64) -> Vec<(DVec2, DVec2)> {
    let d = s.printed_drawing(cc_core::ViewId(view)).unwrap();
    d.prims
        .iter()
        .filter_map(|p| match p {
            Prim::Line { a, b, element: Some(e), .. } if e.0 == element && (a.x - b.x).abs() < 1e-9 => Some((*a, *b)),
            _ => None,
        })
        .collect()
}

#[test]
fn elevations_are_written_as_their_visible_linework() {
    let mut s = Session::default();
    // A tall wall in front, a low one behind it, seen from the south.
    let front = id(&run(&mut s, "wall_create", json!({"start": [0, 0], "end": [4, 0], "height": 3.0})));
    let back = id(&run(&mut s, "wall_create", json!({"start": [1, 3], "end": [3, 3], "height": 1.5})));
    let elevation = id(&run(&mut s, "view_create", json!({"kind": "elevation", "direction": "south"})));
    run(&mut s, "text_create", json!({"view": elevation, "position": [8.0, 1.0], "text": "Facade"}));
    run(
        &mut s,
        "masking_region_create",
        json!({"view": elevation, "curves": [{"kind": "rectangle", "corner": [6, 0], "opposite": [7, 1]}]}),
    );

    let clean_path = scratch("clean.dxf");
    let out = run(&mut s, "view_export", json!({"view": elevation, "path": clean_path}));
    assert_eq!(out["hidden_lines"], true, "{out}");
    assert!(out.get("warning").is_none());
    let raw_path = scratch("raw.dxf");
    let out = run(&mut s, "view_export", json!({"view": elevation, "path": raw_path, "hidden_lines": false}));
    assert_eq!(out["hidden_lines"], false);
    let clean = std::fs::read(&clean_path).unwrap();
    let raw = std::fs::read(&raw_path).unwrap();
    assert_eq!(raw, raw_dxf(&mut s, elevation), "written as before");

    let (c, r) = (pairs(&clean), pairs(&raw));
    let entities = |p: &[(i32, String)]| count(p, "LINE") + count(p, "POLYLINE");
    assert!(entities(&c) < entities(&r), "{} lines and polylines, {} lines before", entities(&c), entities(&r));
    assert!(count(&r, "SOLID") > 0);
    // Only the masking region's fill is left: two triangles of its rectangle.
    assert_eq!(count(&c, "SOLID"), 2, "surface fills are left out");
    assert!(c.contains(&(1, "Facade".into())), "the note stays");

    // The back wall's ends stand behind the front wall: gone.
    let segs = segments(&c);
    let behind = vertical_lines(&mut s, elevation, back);
    assert!(!behind.is_empty());
    for (a, b) in behind {
        let mid = (a + b) * 0.5 * 1000.0;
        assert!(!touches(&segs, mid), "{mid} is behind the front wall");
        assert!(touches(&segments(&r), mid), "and was written before");
    }
    // The front wall's outline is there, whole.
    let ends = vertical_lines(&mut s, elevation, front);
    assert_eq!(ends.len(), 2, "{ends:?}");
    for (a, b) in &ends {
        assert!(covered(&segs, *a * 1000.0, *b * 1000.0), "{a} {b}");
    }
    let (left, right) = (ends[0].0.x.min(ends[1].0.x), ends[0].0.x.max(ends[1].0.x));
    assert!((right - left - 4.0).abs() < 1e-6);
    let top = ends[0].0.y.max(ends[0].1.y);
    assert!(covered(&segs, DVec2::new(left, top) * 1000.0, DVec2::new(right, top) * 1000.0));
    // The outline is one closed polyline.
    assert!(c.windows(2).any(|w| w[0].0 == 66 && w[1].0 == 10) && c.contains(&(70, "1".into())));

    // The polylines read back as CAD primitives.
    let read = cc_cad::read(&clean, &cc_cad::Limits::default()).unwrap();
    let closed = read
        .drawing
        .prims
        .iter()
        .filter(|p| matches!(p, cc_core::cad::CadPrim::Polyline { closed: true, pts, .. } if pts.len() >= 4))
        .count();
    assert!(closed >= 1, "{:?}", read.drawing.prims);
    let wall = read.drawing.layers.iter().position(|l| l.name == "WALL").unwrap();
    assert!(
        read.drawing
            .prims
            .iter()
            .any(|p| matches!(p, cc_core::cad::CadPrim::Polyline { layer, .. } if *layer as usize == wall))
    );
}

#[test]
fn plans_are_written_as_before_unless_asked() {
    let mut s = Session::default();
    run(&mut s, "wall_create", json!({"start": [0, 0], "end": [4, 0]}));
    run(&mut s, "wall_create", json!({"start": [4, 0], "end": [4, 3]}));
    let plan = run(&mut s, "views_list", json!({"category": "floor_plan"}))[0]["id"].as_u64().unwrap();
    let path = scratch("plan.dxf");
    let out = run(&mut s, "view_export", json!({"view": plan, "path": path}));
    assert_eq!(out["hidden_lines"], false);
    assert_eq!(std::fs::read(&path).unwrap(), raw_dxf(&mut s, plan));
    // Asked, a plan is cleaned too.
    let out = run(&mut s, "view_export", json!({"view": plan, "path": path, "hidden_lines": true}));
    assert_eq!(out["hidden_lines"], true);
    // Other formats ignore it.
    let png = scratch("plan.png");
    let out = run(&mut s, "view_export", json!({"view": plan, "path": png, "dpi": 20, "hidden_lines": true}));
    assert!(out.get("hidden_lines").is_none());
}

#[test]
fn the_sample_houses_elevations_and_sections_clean_within_their_limits() {
    let mut s = Session::default();
    let sample = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/casa_simples.ccad");
    run(&mut s, "project_open", json!({"path": sample}));
    let mut views = Vec::new();
    for category in ["elevation", "section"] {
        for v in run(&mut s, "views_list", json!({"category": category})).as_array().unwrap() {
            views.push((v["id"].as_u64().unwrap(), v["name"].as_str().unwrap_or_default().to_string()));
        }
    }
    assert!(views.len() >= 4, "{views:?}");
    for (view, name) in views {
        let drawing = s.printed_drawing(cc_core::ViewId(view)).unwrap();
        let lines = drawing.prims.iter().filter(|p| matches!(p, Prim::Line { .. })).count();
        let fills = drawing.prims.iter().filter(|p| matches!(p, Prim::Fill { .. })).count();
        let skip = Default::default();
        let options =
            cc_views::linework::Options { layer: &|p| default_layer(p).into(), view_owned: &|_| false, skip: &skip };
        let started = std::time::Instant::now();
        let clean = cc_views::linework::clean(&drawing, &options).unwrap();
        let took = started.elapsed();
        let segments: usize = clean.lines.iter().map(|l| l.pts.len() - usize::from(!l.closed)).sum();
        let polylines = clean.lines.iter().filter(|l| l.pts.len() > 2).count();
        eprintln!(
            "{name}: {lines} lines, {fills} fills -> {} runs ({polylines} polylines, {segments} segments), {} fills kept, {took:?}",
            clean.lines.len(),
            clean.prims.iter().filter(|p| matches!(p, Prim::Fill { .. })).count(),
        );
        assert!(clean.lines.len() < lines, "{name}: {} runs from {lines} lines", clean.lines.len());
        // The export says so too.
        let out = run(&mut s, "view_export", json!({"view": view, "path": scratch("house.dxf")}));
        assert_eq!(out["hidden_lines"], true, "{name}: {out}");
    }
}
