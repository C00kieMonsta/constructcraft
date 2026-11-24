//! glTF export (`gltf_export`): what a 3D view shows as a `.glb`, one node
//! per element under one per level, in meters with Y up, the same bytes
//! every time.

// Test helpers outside #[test] functions may fail loudly.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cc_engine::{ErrorCode, Session};
use serde_json::{Value, json};
use std::path::PathBuf;

fn run(s: &mut Session, name: &str, args: Value) -> Value {
    s.execute(name, args).unwrap_or_else(|e| panic!("{name}: {e:?}"))
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ccad-gltf-export-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

fn house() -> Session {
    let mut s = Session::default();
    let sample = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/casa_simples.ccad");
    run(&mut s, "project_open", json!({"path": sample}));
    s
}

/// The JSON and binary chunks of a `.glb`, its header checked.
fn parse(bytes: &[u8]) -> (Value, Vec<u8>) {
    let u32_at = |i: usize| u32::from_le_bytes(bytes[i..i + 4].try_into().unwrap()) as usize;
    assert_eq!(&bytes[0..4], b"glTF");
    assert_eq!(u32_at(4), 2);
    assert_eq!(u32_at(8), bytes.len());
    let json_len = u32_at(12);
    assert_eq!(&bytes[16..20], b"JSON");
    assert_eq!(json_len % 4, 0);
    let json: Value = serde_json::from_slice(&bytes[20..20 + json_len]).unwrap();
    let rest = 20 + json_len;
    let bin = if rest < bytes.len() {
        let len = u32_at(rest);
        assert_eq!(&bytes[rest + 4..rest + 8], b"BIN\0");
        bytes[rest + 8..rest + 8 + len].to_vec()
    } else {
        Vec::new()
    };
    (json, bin)
}

/// The smallest and largest position of a node's mesh, from the accessors' bounds.
fn bounds(gltf: &Value, node: &Value) -> ([f64; 3], [f64; 3]) {
    let mesh = &gltf["meshes"][node["mesh"].as_u64().unwrap() as usize];
    let mut lo = [f64::INFINITY; 3];
    let mut hi = [f64::NEG_INFINITY; 3];
    for p in mesh["primitives"].as_array().unwrap() {
        let a = &gltf["accessors"][p["attributes"]["POSITION"].as_u64().unwrap() as usize];
        for k in 0..3 {
            lo[k] = lo[k].min(a["min"][k].as_f64().unwrap());
            hi[k] = hi[k].max(a["max"][k].as_f64().unwrap());
        }
    }
    (lo, hi)
}

#[test]
fn exports_the_house_by_level_and_element_without_changing_it() {
    let mut s = house();
    let (depth, dirty) = (s.doc.undo_depth(), s.doc.is_dirty());
    let path = scratch("house.glb");
    let out = run(&mut s, "gltf_export", json!({"path": path}));
    assert_eq!(s.doc.undo_depth(), depth);
    assert_eq!(s.doc.is_dirty(), dirty);
    assert!(out["elements"].as_u64().unwrap() > 5, "{out}");
    assert!(out["triangles"].as_u64().unwrap() > 50, "{out}");

    let bytes = std::fs::read(&path).unwrap();
    let (gltf, bin) = parse(&bytes);
    assert_eq!(gltf["asset"]["version"], "2.0");
    assert_eq!(gltf["buffers"][0]["byteLength"].as_u64().unwrap() as usize, bin.len());
    // Every buffer view lies inside the binary chunk.
    for v in gltf["bufferViews"].as_array().unwrap() {
        let end = v["byteOffset"].as_u64().unwrap() + v["byteLength"].as_u64().unwrap();
        assert!(end as usize <= bin.len());
    }

    // The roots are the levels, named after them; their children the elements.
    let nodes = gltf["nodes"].as_array().unwrap();
    let levels: Vec<String> = s.project().levels.iter().map(|l| l.name.clone()).collect();
    let roots = gltf["scenes"][0]["nodes"].as_array().unwrap();
    let mut children = 0;
    for r in roots {
        let root = &nodes[r.as_u64().unwrap() as usize];
        assert!(levels.contains(&root["name"].as_str().unwrap().to_string()), "{}", root["name"]);
        for c in root["children"].as_array().unwrap() {
            let node = &nodes[c.as_u64().unwrap() as usize];
            let id = node["extras"]["id"].as_str().unwrap();
            assert!(node["name"].as_str().unwrap().ends_with(&format!("[{id}]")), "{}", node["name"]);
            assert_eq!(node["extras"]["uid"].as_str().unwrap().len(), 32);
            children += 1;
        }
    }
    assert_eq!(children, out["elements"].as_u64().unwrap());

    // Materials are named after the project's.
    let names: Vec<&str> = gltf["materials"].as_array().unwrap().iter().map(|m| m["name"].as_str().unwrap()).collect();
    assert!(names.iter().any(|n| s.project().material(n).is_some()), "{names:?}");
    assert_eq!(names.len() as u64, out["materials"].as_u64().unwrap());

    // The same model gives the same bytes.
    let again = scratch("house-again.glb");
    run(&mut s, "gltf_export", json!({"path": again, "view": "{3D}"}));
    assert!(std::fs::read(&again).unwrap() == bytes, "a second export differs");
}

#[test]
fn turns_the_model_y_up_in_meters() {
    let mut s = house();
    let column = run(&mut s, "column_create", json!({"position": [50.0, 30.0]}));
    let id = format!("E{}", column["id"]);
    let path = scratch("column.glb");
    run(&mut s, "gltf_export", json!({"path": path}));
    let (gltf, _) = parse(&std::fs::read(&path).unwrap());
    let node = gltf["nodes"].as_array().unwrap().iter().find(|n| n["extras"]["id"] == id.as_str()).unwrap();
    assert_eq!(node["extras"]["category"], "Column");
    let (lo, hi) = bounds(&gltf, node);
    // Plan (50, 30) → glTF x 50, z -30; the column stands up along Y.
    assert!((lo[0] + hi[0]) / 2.0 - 50.0 < 0.01 && (lo[0] + hi[0]) / 2.0 - 50.0 > -0.01, "{lo:?} {hi:?}");
    assert!(((lo[2] + hi[2]) / 2.0 + 30.0).abs() < 0.01, "{lo:?} {hi:?}");
    assert!(hi[1] - lo[1] > 2.0, "{lo:?} {hi:?}");
    assert!(hi[0] - lo[0] < 1.0 && hi[2] - lo[2] < 1.0, "{lo:?} {hi:?}");
}

#[test]
fn leaves_out_elements_the_view_hides() {
    let mut s = house();
    let path = scratch("all.glb");
    let all = run(&mut s, "gltf_export", json!({"path": path}));
    run(&mut s, "view_graphics_set", json!({"view": "{3D}", "category": "wall", "visible": false}));
    let path = scratch("no-walls.glb");
    let some = run(&mut s, "gltf_export", json!({"path": path}));
    assert!(some["elements"].as_u64() < all["elements"].as_u64(), "{all} {some}");
    let (gltf, _) = parse(&std::fs::read(&path).unwrap());
    assert!(gltf["nodes"].as_array().unwrap().iter().all(|n| n["extras"]["category"] != "Wall"));
}

#[test]
fn refuses_other_formats_and_views() {
    let mut s = house();
    let code = |s: &mut Session, args: Value| s.execute("gltf_export", args).unwrap_err().code;
    assert_eq!(code(&mut s, json!({"path": scratch("a.gltf")})), ErrorCode::InvalidArguments);
    let plan = run(&mut s, "views_list", json!({"category": "floor_plan"}))[0]["id"].clone();
    assert_eq!(code(&mut s, json!({"path": scratch("a.glb"), "view": plan})), ErrorCode::InvalidArguments);
}

#[test]
fn embeds_textures_as_png_and_lays_them_on_the_faces() {
    let mut s = house();
    let path = scratch("plain.glb");
    run(&mut s, "gltf_export", json!({"path": path}));
    let (plain, _) = parse(&std::fs::read(&path).unwrap());
    assert!(plain.get("images").is_none());
    let name = plain["materials"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["name"].as_str().unwrap().to_string())
        .find(|n| s.project().material(n).is_some_and(|m| m.identity.class != cc_core::material::MaterialClass::Glass))
        .unwrap();

    // A BMP: glTF takes only PNG and JPEG, so it is written as PNG.
    let mut bmp = Vec::new();
    image::RgbImage::from_pixel(4, 2, image::Rgb([200, 40, 10]))
        .write_to(&mut std::io::Cursor::new(&mut bmp), image::ImageFormat::Bmp)
        .unwrap();
    let data = cc_core::raster::base64_encode(&bmp);
    run(&mut s, "material_texture_set", json!({"material": name, "data": data, "width": 2.0}));
    let path = scratch("textured.glb");
    let out = run(&mut s, "gltf_export", json!({"path": path}));
    assert_eq!(out["textures"], 1, "{out}");
    let (gltf, bin) = parse(&std::fs::read(&path).unwrap());
    let material = gltf["materials"].as_array().unwrap().iter().find(|m| m["name"] == name.as_str()).unwrap();
    assert_eq!(material["pbrMetallicRoughness"]["baseColorTexture"]["index"], 0);
    // The texture replaces the color: a white base.
    assert_eq!(material["pbrMetallicRoughness"]["baseColorFactor"][0], 1.0);
    let image = &gltf["images"][0];
    assert_eq!(image["mimeType"], "image/png");
    let view = &gltf["bufferViews"][image["bufferView"].as_u64().unwrap() as usize];
    let start = view["byteOffset"].as_u64().unwrap() as usize;
    let len = view["byteLength"].as_u64().unwrap() as usize;
    let decoded = image::load_from_memory(&bin[start..start + len]).unwrap().to_rgb8();
    assert_eq!((decoded.width(), decoded.height()), (4, 2));
    assert_eq!(decoded.get_pixel(0, 0).0, [200, 40, 10]);
}

/// The bounds of every mesh piece of the project's component types.
fn mesh_bounds(s: &Session) -> ([f64; 3], [f64; 3]) {
    let (mut lo, mut hi) = ([f64::MAX; 3], [f64::MIN; 3]);
    for t in s.project().types.values() {
        let cc_core::TypeData::Component { def, .. } = &t.data else { continue };
        for item in &def.items {
            let cc_core::ComponentShape::Mesh { vertices, .. } = &item.shape else { continue };
            for v in vertices.iter() {
                for k in 0..3 {
                    lo[k] = lo[k].min(v[k]);
                    hi[k] = hi[k].max(v[k]);
                }
            }
        }
    }
    (lo, hi)
}

#[test]
fn draco_compresses_the_meshes_and_reads_back() {
    let mut s = house();
    let plain = scratch("house-plain.glb");
    let packed = scratch("house-draco.glb");
    let a = run(&mut s, "gltf_export", json!({"path": plain}));
    let b = run(&mut s, "gltf_export", json!({"path": packed, "draco": true}));
    assert_eq!(a["triangles"], b["triangles"]);
    assert!(b["bytes"].as_u64().unwrap() * 2 < a["bytes"].as_u64().unwrap(), "{a} {b}");
    let bytes = std::fs::read(&packed).unwrap();
    assert_eq!(bytes.len() as u64, b["bytes"].as_u64().unwrap());
    let again = scratch("house-draco-again.glb");
    run(&mut s, "gltf_export", json!({"path": again, "draco": true}));
    assert_eq!(std::fs::read(&again).unwrap(), bytes, "the same model gives the same bytes");

    let (gltf, bin) = parse(&bytes);
    assert_eq!(gltf["extensionsUsed"], json!(["KHR_draco_mesh_compression"]));
    assert_eq!(gltf["extensionsRequired"], json!(["KHR_draco_mesh_compression"]));
    let accessors = gltf["accessors"].as_array().unwrap();
    let views = gltf["bufferViews"].as_array().unwrap();
    for mesh in gltf["meshes"].as_array().unwrap() {
        for p in mesh["primitives"].as_array().unwrap() {
            let ext = &p["extensions"]["KHR_draco_mesh_compression"];
            let view = &views[ext["bufferView"].as_u64().unwrap() as usize];
            let start = view["byteOffset"].as_u64().unwrap() as usize;
            assert_eq!(&bin[start..start + 5], b"DRACO");
            for name in ["POSITION", "NORMAL", "TEXCOORD_0"] {
                assert!(ext["attributes"][name].is_u64(), "{name}: {ext}");
                let acc = &accessors[p["attributes"][name].as_u64().unwrap() as usize];
                assert!(acc.get("bufferView").is_none(), "compressed accessors have no view: {acc}");
            }
            let pos = &accessors[p["attributes"]["POSITION"].as_u64().unwrap() as usize];
            assert!(pos["min"].is_array() && pos["max"].is_array());
            let idx = &accessors[p["indices"].as_u64().unwrap() as usize];
            assert_eq!(idx["count"].as_u64().unwrap() % 3, 0);
        }
    }

    // Read back: the same triangles, at the same places to within the quantization.
    let mut one = Session::default();
    let mut two = Session::default();
    let x = run(&mut one, "gltf_import", json!({"path": plain, "merge": true}));
    let y = run(&mut two, "gltf_import", json!({"path": packed, "merge": true}));
    assert_eq!(x["faces"], y["faces"], "{x} {y}");
    let ((lo1, hi1), (lo2, hi2)) = (mesh_bounds(&one), mesh_bounds(&two));
    for k in 0..3 {
        assert!((lo1[k] - lo2[k]).abs() < 2e-3 && (hi1[k] - hi2[k]).abs() < 2e-3, "{lo1:?} {hi1:?} / {lo2:?} {hi2:?}");
    }
}
