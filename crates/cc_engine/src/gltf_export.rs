//! glTF 2.0 export (binary `.glb`) of what a 3D view shows, for
//! visualization programs (real-time renderers such as Twinmotion, game
//! engines, web viewers).
//!
//! The file is laid out so that exporting the same model again gives the
//! same names: the scene holds one node per level (named after it), and
//! under it one node per element, named `<type> [<id>]`, with the element's
//! id, unique id, category, type, mark and level in its `extras`. Each
//! element's mesh has one primitive per material, and materials are named
//! after the project's (faces of no project material are named by their
//! color), so a program that reimports the file can keep the materials it
//! replaced. glTF is in meters with Y up: the model's (x, y, z) becomes
//! (x, z, -y).
//!
//! Materials are metallic-roughness from the material's appearance: its
//! color, its finish as roughness (the path tracer's table), metal for
//! class `Metal`, blended opacity for transparent ones, and its texture
//! (PNG and JPEG as they are, other pictures re-encoded as PNG). Every face
//! has texture coordinates: in repeats of the texture where it has one, in
//! meters along the face otherwise, so materials applied later are laid at
//! their real size.

use cc_core::material::{Finish, MaterialClass, MaterialTexture};
use cc_core::raster::RasterFormat;
use cc_core::{Category, ElementId, Project, View};
use cc_geom::{ModelGeometry, Solid};
use glam::DVec3;
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};

/// The most triangles a file takes; a view showing more is refused.
pub const MAX_TRIANGLES: usize = 5_000_000;
/// The most textures embedded; materials past them keep their color.
pub const MAX_TEXTURES: usize = 256;
/// Twice the area below which a triangle is dropped, square meters.
const MIN_DOUBLE_AREA: f64 = 1e-12;
/// A material at least this transparent counts as glass.
const GLASS_TRANSPARENCY: f64 = 0.5;
/// The opacity of glass with no transparency of its own.
const GLASS_OPACITY: f64 = 0.3;

/// What [`export`] wrote.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Exported {
    /// The `.glb` file.
    pub bytes: Vec<u8>,
    /// Element nodes written.
    pub elements: usize,
    pub materials: usize,
    pub textures: usize,
    pub triangles: usize,
}

/// What a face is made of: a project material, or a bare color.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum MaterialKey {
    Named(String),
    Color([u8; 3]),
}

/// A material of the file.
struct MaterialOut {
    json: Value,
    /// Its texture, to lay coordinates in repeats of.
    texture: Option<MaterialTexture>,
}

/// The triangles of one material in one element.
#[derive(Default)]
struct Primitive {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

/// The binary buffer and the JSON arrays that index it.
#[derive(Default)]
struct Writer {
    bin: Vec<u8>,
    views: Vec<Value>,
    accessors: Vec<Value>,
}

impl Writer {
    /// Append `bytes` as a buffer view (4-byte aligned); its index.
    fn view(&mut self, bytes: &[u8], target: Option<u32>) -> usize {
        while !self.bin.len().is_multiple_of(4) {
            self.bin.push(0);
        }
        let mut v = json!({ "buffer": 0, "byteOffset": self.bin.len(), "byteLength": bytes.len() });
        if let (Some(t), Some(obj)) = (target, v.as_object_mut()) {
            obj.insert("target".into(), json!(t));
        }
        self.bin.extend_from_slice(bytes);
        self.views.push(v);
        self.views.len() - 1
    }

    /// An accessor of float vectors; its index.
    fn floats<const N: usize>(&mut self, data: &[[f32; N]], kind: &str, bounds: bool) -> usize {
        let bytes: Vec<u8> = data.iter().flat_map(|v| v.iter().flat_map(|f| f.to_le_bytes())).collect();
        let view = self.view(&bytes, Some(34962));
        let mut a = json!({ "bufferView": view, "componentType": 5126, "count": data.len(), "type": kind });
        if bounds && let Some(obj) = a.as_object_mut() {
            let mut lo = [f32::INFINITY; N];
            let mut hi = [f32::NEG_INFINITY; N];
            for v in data {
                for k in 0..N {
                    lo[k] = lo[k].min(v[k]);
                    hi[k] = hi[k].max(v[k]);
                }
            }
            obj.insert("min".into(), json!(lo.to_vec()));
            obj.insert("max".into(), json!(hi.to_vec()));
        }
        self.accessors.push(a);
        self.accessors.len() - 1
    }

    /// An accessor of triangle indices; its index.
    fn indices(&mut self, data: &[u32]) -> usize {
        let bytes: Vec<u8> = data.iter().flat_map(|i| i.to_le_bytes()).collect();
        let view = self.view(&bytes, Some(34963));
        self.accessors
            .push(json!({ "bufferView": view, "componentType": 5125, "count": data.len(), "type": "SCALAR" }));
        self.accessors.len() - 1
    }
}

/// An sRGB channel as linear light.
fn linear(c: u8) -> f64 {
    let c = f64::from(c) / 255.0;
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

/// The roughness of a finish (as the path tracer shades it).
fn roughness(finish: Finish, metal: bool) -> f64 {
    match (finish, metal) {
        (Finish::Matte, false) => 0.90,
        (Finish::Satin, false) => 0.55,
        (Finish::Glossy, false) => 0.30,
        (Finish::Polished, false) => 0.12,
        (Finish::Matte, true) => 0.65,
        (Finish::Satin, true) => 0.40,
        (Finish::Glossy, true) => 0.22,
        (Finish::Polished, true) => 0.06,
    }
}

/// A picture as glTF takes it: PNG and JPEG as they are, others decoded and
/// written as PNG; `None` when it cannot be read.
fn image_bytes(texture: &MaterialTexture) -> Option<(Vec<u8>, &'static str)> {
    if !texture.image_ok() {
        return None;
    }
    let image = &texture.image;
    match image.format {
        RasterFormat::Png => Some((image.data.clone(), "image/png")),
        RasterFormat::Jpeg => Some((image.data.clone(), "image/jpeg")),
        _ => {
            let rgba = cc_views::raster::decode(image, u32::MAX)?;
            let mut out = Vec::new();
            let mut encoder = png::Encoder::new(&mut out, rgba.width, rgba.height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().ok()?;
            writer.write_image_data(&rgba.pixels).ok()?;
            writer.finish().ok()?;
            Some((out, "image/png"))
        }
    }
}

/// The materials of the file and the textures they use, in the order first met.
#[derive(Default)]
struct Materials {
    list: Vec<MaterialOut>,
    index: HashMap<MaterialKey, usize>,
    images: Vec<Value>,
    textures: Vec<Value>,
    /// Texture of each picture already embedded, by its bytes' address.
    embedded: Vec<(std::sync::Arc<cc_core::raster::RasterImage>, Option<usize>)>,
}

impl Materials {
    /// The index of the material of `key`, made on first use; `transparency`
    /// is the model's for the face (0 to 1).
    fn slot(&mut self, project: &Project, key: &MaterialKey, transparency: f64, w: &mut Writer) -> usize {
        if let Some(i) = self.index.get(key) {
            return *i;
        }
        let transparency = if transparency.is_finite() { transparency.clamp(0.0, 1.0) } else { 0.0 };
        let out = match key {
            MaterialKey::Named(name) => match project.material(name) {
                Some(m) => {
                    let class = m.identity.class;
                    let glass = class == MaterialClass::Glass || transparency >= GLASS_TRANSPARENCY;
                    let metal = class == MaterialClass::Metal;
                    let opacity = match (glass, transparency > 0.0) {
                        (_, true) => 1.0 - transparency,
                        (true, false) => GLASS_OPACITY,
                        (false, false) => 1.0,
                    };
                    let texture = m.appearance.texture.as_ref().filter(|_| !glass).cloned();
                    let tex_index = texture.as_ref().and_then(|t| self.texture(t, w));
                    let color = if tex_index.is_some() { [255, 255, 255] } else { m.appearance.color };
                    let json = pbr(name, color, opacity, metal, roughness(m.appearance.finish, metal), tex_index);
                    MaterialOut { json, texture: texture.filter(|_| tex_index.is_some()) }
                }
                None => MaterialOut {
                    json: pbr(name, [200, 200, 200], 1.0 - transparency, false, 0.9, None),
                    texture: None,
                },
            },
            MaterialKey::Color(c) => {
                let name = format!("Color {:02X}{:02X}{:02X}", c[0], c[1], c[2]);
                MaterialOut { json: pbr(&name, *c, 1.0 - transparency, false, 0.9, None), texture: None }
            }
        };
        self.list.push(out);
        self.index.insert(key.clone(), self.list.len() - 1);
        self.list.len() - 1
    }

    /// The texture of `t`'s picture, embedded on first use; `None` when it
    /// cannot be read or [`MAX_TEXTURES`] are taken.
    fn texture(&mut self, t: &MaterialTexture, w: &mut Writer) -> Option<usize> {
        if let Some((_, i)) =
            self.embedded.iter().find(|(image, _)| std::sync::Arc::ptr_eq(image, &t.image) || **image == *t.image)
        {
            return *i;
        }
        let index = (self.textures.len() < MAX_TEXTURES).then(|| image_bytes(t)).flatten().map(|(bytes, mime)| {
            let view = w.view(&bytes, None);
            self.images.push(json!({ "bufferView": view, "mimeType": mime }));
            self.textures.push(json!({ "source": self.images.len() - 1, "sampler": 0 }));
            self.textures.len() - 1
        });
        self.embedded.push((t.image.clone(), index));
        index
    }
}

/// A metallic-roughness material.
fn pbr(name: &str, color: [u8; 3], opacity: f64, metal: bool, roughness: f64, texture: Option<usize>) -> Value {
    let opacity = if opacity.is_finite() { opacity.clamp(0.0, 1.0) } else { 1.0 };
    let mut base = json!({
        "baseColorFactor": [linear(color[0]), linear(color[1]), linear(color[2]), opacity],
        "metallicFactor": if metal { 1.0 } else { 0.0 },
        "roughnessFactor": roughness,
    });
    if let (Some(t), Some(obj)) = (texture, base.as_object_mut()) {
        obj.insert("baseColorTexture".into(), json!({ "index": t }));
    }
    let mut m = json!({ "name": name, "pbrMetallicRoughness": base, "doubleSided": true });
    if opacity < 1.0
        && let Some(obj) = m.as_object_mut()
    {
        obj.insert("alphaMode".into(), json!("BLEND"));
    }
    m
}

/// A model point in glTF's frame (Y up).
fn point(p: DVec3) -> [f32; 3] {
    [p.x as f32, p.z as f32, (-p.y) as f32]
}

/// Add the faces of `solid` to its element's primitives, by material.
fn add_solid(
    project: &Project,
    solid: &Solid,
    materials: &mut Materials,
    w: &mut Writer,
    prims: &mut BTreeMap<usize, Primitive>,
    triangles: &mut usize,
) -> Result<(), String> {
    let key_of = |material: Option<&str>, appearance: [u8; 3]| match material.filter(|n| project.material(n).is_some())
    {
        Some(n) => MaterialKey::Named(n.to_string()),
        None => MaterialKey::Color(appearance),
    };
    let own = key_of(solid.material.as_deref(), solid.appearance);
    let normals = solid.vertex_normals();
    let shifts = solid.facet_shifts();
    for (fi, face) in solid.faces.iter().enumerate() {
        if face.verts.len() < 3 || !face.verts.iter().all(|p| p.is_finite()) || !face.normal.is_finite() {
            continue;
        }
        let (key, transparency) = match solid.painted.get(&fi) {
            Some(look) => (key_of(look.material.as_deref(), look.appearance), look.transparency),
            None => (own.clone(), solid.transparency),
        };
        let slot = materials.slot(project, &key, transparency, w);
        let texture = materials.list.get(slot).and_then(|m| m.texture.clone());
        let prim = prims.entry(slot).or_default();
        let (u, v) = cc_geom::plane_axes(face.normal);
        let [du, dv] = shifts.get(fi).copied().unwrap_or_default();
        let face_normals = normals.get(fi);
        let Ok(start) = u32::try_from(prim.positions.len()) else {
            return Err("an element has too many vertices".into());
        };
        // Whole repeats are taken off per face so the coordinates stay precise as f32.
        let mut shift: Option<[f64; 2]> = None;
        for (k, p) in face.verts.iter().enumerate() {
            let n = face_normals.and_then(|ns| ns.get(k)).copied().filter(|n| n.is_finite()).unwrap_or(face.normal);
            let plane = [p.dot(u) + du, p.dot(v) + dv];
            let [x, y] = match &texture {
                Some(t) => cc_raytracing::materials::texture_uv(t, plane),
                None => [plane[0], -plane[1]],
            };
            let s = *shift.get_or_insert([x.floor(), y.floor()]);
            let uv = [(x - s[0]) as f32, (y - s[1]) as f32];
            prim.positions.push(point(*p));
            let n = n.normalize_or_zero();
            prim.normals.push(if n == DVec3::ZERO { point(face.normal) } else { point(n) });
            prim.uvs.push(if uv.iter().all(|c| c.is_finite()) { uv } else { [0.0, 0.0] });
        }
        for [a, b, c] in face.triangulate() {
            let (Some(pa), Some(pb), Some(pc)) = (face.verts.get(a), face.verts.get(b), face.verts.get(c)) else {
                continue;
            };
            let cross = (*pb - *pa).cross(*pc - *pa);
            if cross.length_squared() <= MIN_DOUBLE_AREA * MIN_DOUBLE_AREA {
                continue;
            }
            // Wound counter-clockwise seen from outside, as glTF expects.
            let (b, c) = if cross.dot(face.normal) < 0.0 { (c, b) } else { (b, c) };
            let index = |i: usize| u32::try_from(i).ok().and_then(|i| start.checked_add(i));
            let (Some(ia), Some(ib), Some(ic)) = (index(a), index(b), index(c)) else { continue };
            prim.indices.extend([ia, ib, ic]);
            *triangles += 1;
        }
        if *triangles > MAX_TRIANGLES {
            return Err(format!("the view shows more than {MAX_TRIANGLES} triangles"));
        }
    }
    Ok(())
}

/// The name of an element's node: its type and id.
fn element_name(project: &Project, id: ElementId, category: Category) -> String {
    let element = project.element(id);
    let type_name = element.and_then(|e| project.element_type(e.type_id)).map(|t| t.name.trim()).unwrap_or_default();
    match type_name {
        "" => format!("{category:?} [{id}]"),
        t => format!("{t} [{id}]"),
    }
}

/// The `.glb` of what `view` shows of the model: `geom` is the geometry of
/// the view (its phase and design options), and the elements its graphics
/// hide are left out, as are conceptual masses and pieces not shown at its
/// detail level. Errors past [`MAX_TRIANGLES`].
pub fn export(project: &Project, geom: &ModelGeometry, view: &View) -> Result<Exported, String> {
    let mut resolver = cc_views::graphics::Resolver::new(project, geom, view, None);
    let mut shown: HashMap<ElementId, bool> = HashMap::new();
    let mut w = Writer::default();
    let mut materials = Materials::default();
    let mut triangles = 0;
    // Element → its primitives by material, in the order the elements are met.
    let mut order: Vec<(ElementId, Category)> = Vec::new();
    let mut elements: HashMap<ElementId, BTreeMap<usize, Primitive>> = HashMap::new();
    for solid in &geom.solids {
        if solid.category == Category::Mass || !solid.detail.shows(view.detail) {
            continue;
        }
        if !*shown.entry(solid.element).or_insert_with(|| resolver.style(solid.element).is_some()) {
            continue;
        }
        let prims = elements.entry(solid.element).or_insert_with(|| {
            order.push((solid.element, solid.category));
            BTreeMap::new()
        });
        add_solid(project, solid, &mut materials, &mut w, prims, &mut triangles)?;
    }

    // Meshes and element nodes, under one node per level.
    let mut meshes = Vec::new();
    let mut nodes: Vec<Value> = Vec::new();
    let mut by_level: BTreeMap<(i64, String), Vec<usize>> = BTreeMap::new();
    for (id, category) in &order {
        let Some(prims) = elements.remove(id) else { continue };
        let primitives: Vec<Value> = prims
            .into_iter()
            .filter(|(_, p)| !p.indices.is_empty())
            .map(|(material, p)| {
                let position = w.floats(&p.positions, "VEC3", true);
                let normal = w.floats(&p.normals, "VEC3", false);
                let uv = w.floats(&p.uvs, "VEC2", false);
                let indices = w.indices(&p.indices);
                json!({
                    "attributes": { "POSITION": position, "NORMAL": normal, "TEXCOORD_0": uv },
                    "indices": indices,
                    "material": material,
                })
            })
            .collect();
        if primitives.is_empty() {
            continue;
        }
        let name = element_name(project, *id, *category);
        meshes.push(json!({ "name": name, "primitives": primitives }));
        let element = project.element(*id);
        let level = element.and_then(|e| project.level(e.level));
        let mut extras = json!({ "id": id.to_string(), "category": format!("{category:?}") });
        if let (Some(e), Some(obj)) = (element, extras.as_object_mut()) {
            obj.insert("uid".into(), json!(e.uid.to_string()));
            if let Some(t) = project.element_type(e.type_id) {
                obj.insert("type".into(), json!(t.name));
            }
            if !e.mark.is_empty() {
                obj.insert("mark".into(), json!(e.mark));
            }
            if let Some(l) = level {
                obj.insert("level".into(), json!(l.name));
            }
        }
        nodes.push(json!({ "name": name, "mesh": meshes.len() - 1, "extras": extras }));
        // Levels in order of elevation (millimeters, so the key is ordered).
        let key = match level {
            Some(l) if l.elevation.is_finite() => ((l.elevation * 1000.0).round() as i64, l.name.clone()),
            _ => (i64::MIN, String::new()),
        };
        by_level.entry(key).or_default().push(nodes.len() - 1);
    }
    let count = nodes.len();
    let mut roots = Vec::new();
    for ((_, name), children) in by_level {
        let name = if name.is_empty() { "No level".to_string() } else { name };
        nodes.push(json!({ "name": name, "children": children }));
        roots.push(nodes.len() - 1);
    }

    let materials_json: Vec<Value> = materials.list.iter().map(|m| m.json.clone()).collect();
    let mut gltf = json!({
        "asset": { "version": "2.0", "generator": concat!("ConstructCraft ", env!("CARGO_PKG_VERSION")) },
        "scene": 0,
        "scenes": [{ "name": view.name, "nodes": roots }],
        "nodes": nodes,
        "meshes": meshes,
        "materials": materials_json,
    });
    let textures = materials.textures.len();
    if let Some(obj) = gltf.as_object_mut() {
        if !materials.textures.is_empty() {
            obj.insert("textures".into(), json!(materials.textures));
            obj.insert("images".into(), json!(materials.images));
            // Repeating, mipmapped.
            obj.insert(
                "samplers".into(),
                json!([{ "magFilter": 9729, "minFilter": 9987, "wrapS": 10497, "wrapT": 10497 }]),
            );
        }
        while !w.bin.len().is_multiple_of(4) {
            w.bin.push(0);
        }
        if !w.bin.is_empty() {
            obj.insert("buffers".into(), json!([{ "byteLength": w.bin.len() }]));
            obj.insert("bufferViews".into(), json!(w.views));
            obj.insert("accessors".into(), json!(w.accessors));
        }
    }
    let bytes = glb(&gltf, &w.bin)?;
    Ok(Exported { bytes, elements: count, materials: materials.list.len(), textures, triangles })
}

/// The binary container: header, JSON chunk (padded with spaces) and binary
/// chunk (padded with zeros).
fn glb(gltf: &Value, bin: &[u8]) -> Result<Vec<u8>, String> {
    let mut json = serde_json::to_vec(gltf).map_err(|e| e.to_string())?;
    while !json.len().is_multiple_of(4) {
        json.push(b' ');
    }
    let len = |n: usize| u32::try_from(n).map_err(|_| "the file would be larger than 4 GB".to_string());
    let bin_chunk = if bin.is_empty() { 0 } else { 8 + bin.len() };
    let total = len(12 + 8 + json.len() + bin_chunk)?;
    let mut out = Vec::with_capacity(total as usize);
    out.extend_from_slice(b"glTF");
    out.extend_from_slice(&2u32.to_le_bytes());
    out.extend_from_slice(&total.to_le_bytes());
    out.extend_from_slice(&len(json.len())?.to_le_bytes());
    out.extend_from_slice(b"JSON");
    out.extend_from_slice(&json);
    if !bin.is_empty() {
        out.extend_from_slice(&len(bin.len())?.to_le_bytes());
        out.extend_from_slice(b"BIN\0");
        out.extend_from_slice(bin);
    }
    Ok(out)
}
