//! Draco mesh compression (Google's open bitstream, the glTF extension
//! `KHR_draco_mesh_compression`) over the pure Rust `draco-core` crate:
//! [`encode`] compresses a triangle mesh with positions, normals and
//! texture coordinates, [`decode`] reads one back.
//!
//! Streams are hostile input: the decoder's counts are bounded
//! ([`MAX_POINTS`], [`MAX_FACES`], [`MAX_DECODED_BYTES`]) and both
//! directions run behind `catch_unwind`, so a crafted stream or a bug in the
//! codec is an error, never a crash.
//!
//! Decoded meshes can be kept in a local cache folder ([`set_cache_dir`];
//! the app uses `draco-cache` in its data folder): each entry is named by
//! the SHA-256 of its stream and holds that hash and the SHA-256 of its own
//! contents, checked before it is used, so a damaged or stale entry is
//! never read; it is decoded again and rewritten. The folder is trimmed to
//! [`CACHE_MAX_BYTES`], oldest entries first. The hashes guard against
//! damage, not against someone who can write the folder: entries are read
//! with the same bounds as streams.

use draco_core::{
    DataType, DecodeLimits, DecoderBuffer, EncoderBuffer, EncoderOptions, GeometryAttributeType, Mesh, MeshDecoder,
    MeshEncoder, PointAttribute,
};
use sha2::{Digest, Sha256};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::RwLock;

/// Most points a decoded stream may have.
pub const MAX_POINTS: u64 = 50_000_000;
/// Most triangles a decoded stream may have.
pub const MAX_FACES: u64 = 50_000_000;
/// Most bytes of decoded attribute values.
pub const MAX_DECODED_BYTES: u64 = 1 << 30;

/// Quantization bits of positions: over each mesh's extent, so a 100 m
/// element keeps positions to about 1.5 mm.
pub const POSITION_BITS: i32 = 16;
/// Quantization bits of normals (octahedral).
pub const NORMAL_BITS: i32 = 10;
/// Quantization bits of texture coordinates.
pub const UV_BITS: i32 = 14;
/// Encoder speed (0 smallest … 10 fastest): 4 gave the smallest files on
/// every mesh `draco-core` measured, and is what common exporters default to.
const SPEED: i32 = 4;

/// One attribute of a decoded mesh.
#[derive(Debug, Clone, PartialEq)]
struct Attribute {
    /// The stream's id of it (what a container refers to).
    unique_id: u32,
    /// Numbers per point (1 to 4).
    width: u8,
    /// What normalized integers are divided by (`None`: taken as they are).
    scale: Option<f64>,
    /// `width` values per point.
    values: Vec<f32>,
}

/// A mesh read from a stream.
#[derive(Debug, Clone, PartialEq)]
pub struct Decoded {
    points: usize,
    triangles: Vec<[u32; 3]>,
    attributes: Vec<Attribute>,
}

impl Decoded {
    /// Points (vertices) of the mesh; attributes have one value per point.
    pub fn points(&self) -> usize {
        self.points
    }

    /// The triangles, as point indices.
    pub fn triangles(&self) -> Vec<[usize; 3]> {
        self.triangles.iter().map(|t| t.map(|i| i as usize)).collect()
    }

    /// The values of the attribute with the stream's `unique_id`, `width`
    /// per point (normalized integers mapped to 0..1 or -1..1); `None` when
    /// the stream has no such attribute or it has another width.
    pub fn attribute(&self, unique_id: u32, width: usize) -> Option<Vec<f64>> {
        let att = self.attributes.iter().find(|a| a.unique_id == unique_id)?;
        if usize::from(att.width) != width {
            return None;
        }
        Some(
            att.values
                .iter()
                .map(|v| match att.scale {
                    Some(s) => (f64::from(*v) / s).max(-1.0),
                    None => f64::from(*v),
                })
                .collect(),
        )
    }

    /// The mesh's data out of the codec's: every attribute of one to four
    /// numbers, and only when every face names points the attributes have.
    fn from_mesh(mesh: &Mesh) -> Option<Decoded> {
        let points = mesh.num_points();
        let triangles: Vec<[u32; 3]> = mesh.faces().iter().map(|f| f.map(|p| p.0)).collect();
        if !triangles.iter().flatten().all(|i| (*i as usize) < points) {
            return None;
        }
        let mut attributes = Vec::new();
        for id in 0..mesh.num_attributes() {
            let att = mesh.try_attribute(id).ok()?;
            let width = att.num_components();
            if !(1..=4).contains(&width) {
                continue;
            }
            let scale = match (att.normalized(), att.data_type()) {
                (true, DataType::Int8) => Some(127.0),
                (true, DataType::Uint8) => Some(255.0),
                (true, DataType::Int16) => Some(32767.0),
                (true, DataType::Uint16) => Some(65535.0),
                (true, DataType::Int32) => Some(2_147_483_647.0),
                (true, DataType::Uint32) => Some(4_294_967_295.0),
                _ => None,
            };
            let values = att.read_f32s(points, usize::from(width));
            attributes.push(Attribute { unique_id: att.unique_id(), width, scale, values });
        }
        Some(Decoded { points, triangles, attributes })
    }
}

/// Decode a Draco mesh stream; `None` when it is broken, not a triangle
/// mesh or past the limits. With a cache folder set ([`set_cache_dir`]),
/// a stream decoded before is read back from it (see [`decode_cached`]).
pub fn decode(bytes: &[u8]) -> Option<Decoded> {
    let dir = CACHE_DIR.read().ok().and_then(|d| d.clone());
    match dir {
        Some(dir) => decode_cached(bytes, &dir),
        None => decode_stream(bytes),
    }
}

/// Decode with the codec, no cache.
fn decode_stream(bytes: &[u8]) -> Option<Decoded> {
    let limits = DecodeLimits::default()
        .with_max_points(MAX_POINTS)
        .with_max_faces(MAX_FACES)
        .with_max_decoded_bytes(MAX_DECODED_BYTES);
    catch_unwind(AssertUnwindSafe(|| {
        let mut buffer = DecoderBuffer::new(bytes).with_limits(limits);
        let mut mesh = Mesh::new();
        MeshDecoder::new().decode(&mut buffer, &mut mesh).ok()?;
        Decoded::from_mesh(&mesh)
    }))
    .ok()
    .flatten()
}

// --- The local cache of decoded meshes ---------------------------------------

/// The folder decoded meshes are kept in (`None`: no cache).
static CACHE_DIR: RwLock<Option<PathBuf>> = RwLock::new(None);

/// What starts a cache entry (and its version: a new layout is a new magic).
const CACHE_MAGIC: &[u8; 8] = b"CCDRC\x00\x01\x00";
/// Extension of cache entries.
const CACHE_EXT: &str = "drcmesh";
/// Streams smaller than this decode faster than an entry is read: not kept.
const CACHE_MIN_STREAM: usize = 4 * 1024;
/// Total size the cache folder is trimmed to, oldest entries first.
pub const CACHE_MAX_BYTES: u64 = 512 * 1024 * 1024;

/// Keep decoded meshes in `dir` (created when needed), or keep none.
/// Process-wide, so imports running on other threads use it too.
pub fn set_cache_dir(dir: Option<PathBuf>) {
    if let Ok(mut d) = CACHE_DIR.write() {
        *d = dir;
    }
}

/// The SHA-256 of `bytes`.
fn sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn hex(hash: &[u8; 32]) -> String {
    hash.iter().map(|b| format!("{b:02x}")).collect()
}

/// Decode `bytes` through the cache in `dir`: the entry is named by the
/// SHA-256 of the stream and holds that hash again and the SHA-256 of its
/// contents; it is used only when both match and its contents read whole
/// and in bounds. Otherwise the stream is decoded and the entry (re)written.
/// The cache is best effort: a folder that cannot be written only means
/// decoding every time.
pub fn decode_cached(bytes: &[u8], dir: &Path) -> Option<Decoded> {
    if bytes.len() < CACHE_MIN_STREAM {
        return decode_stream(bytes);
    }
    let key = sha256(bytes);
    let path = dir.join(format!("{}.{CACHE_EXT}", hex(&key)));
    if let Some(hit) = read_entry(&path, &key) {
        return Some(hit);
    }
    let decoded = decode_stream(bytes)?;
    if write_entry(dir, &path, &key, &decoded).is_ok() {
        trim(dir, CACHE_MAX_BYTES);
    }
    Some(decoded)
}

/// The entry's bytes: magic, the stream's hash, the contents' hash, contents.
fn entry_bytes(key: &[u8; 32], d: &Decoded) -> Option<Vec<u8>> {
    let mut body = Vec::new();
    let u32_of = |n: usize| u32::try_from(n).ok();
    body.extend(u32_of(d.points)?.to_le_bytes());
    body.extend(u32_of(d.triangles.len())?.to_le_bytes());
    for i in d.triangles.iter().flatten() {
        body.extend(i.to_le_bytes());
    }
    body.extend(u32_of(d.attributes.len())?.to_le_bytes());
    for a in &d.attributes {
        body.extend(a.unique_id.to_le_bytes());
        body.push(a.width);
        body.extend(a.scale.unwrap_or(0.0).to_le_bytes());
        for v in &a.values {
            body.extend(v.to_le_bytes());
        }
    }
    let mut out = Vec::with_capacity(72 + body.len());
    out.extend_from_slice(CACHE_MAGIC);
    out.extend_from_slice(key);
    out.extend_from_slice(&sha256(&body));
    out.extend(body);
    Some(out)
}

/// Write the entry beside its place, then move it there, so a reader never
/// sees half of one.
fn write_entry(dir: &Path, path: &Path, key: &[u8; 32], d: &Decoded) -> std::io::Result<()> {
    let bytes = entry_bytes(key, d).ok_or(std::io::ErrorKind::InvalidData)?;
    std::fs::create_dir_all(dir)?;
    let tmp = path.with_extension(format!("tmp{}", std::process::id()));
    std::fs::write(&tmp, &bytes)?;
    std::fs::rename(&tmp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })
}

/// The entry at `path` for the stream hashed `key`, when it verifies.
fn read_entry(path: &Path, key: &[u8; 32]) -> Option<Decoded> {
    let len = std::fs::metadata(path).ok().filter(|m| m.is_file())?.len();
    if len > MAX_DECODED_BYTES + (MAX_FACES * 12) + 1024 {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    let parsed = parse_entry(&bytes, key);
    if parsed.is_none() {
        // A broken or foreign entry is replaced by the next write.
        let _ = std::fs::remove_file(path);
    }
    parsed
}

/// Read an entry's bytes: `None` unless the magic, both hashes and every
/// count check out.
fn parse_entry(bytes: &[u8], key: &[u8; 32]) -> Option<Decoded> {
    let (magic, rest) = bytes.split_at_checked(8)?;
    let (stream_hash, rest) = rest.split_at_checked(32)?;
    let (body_hash, body) = rest.split_at_checked(32)?;
    if magic != CACHE_MAGIC || stream_hash != key || body_hash != sha256(body) {
        return None;
    }
    let mut r = Reader { bytes: body };
    let points = r.u32()? as usize;
    let faces = r.u32()? as usize;
    if points as u64 > MAX_POINTS || faces as u64 > MAX_FACES {
        return None;
    }
    let mut triangles = Vec::with_capacity(faces.min(r.bytes.len() / 12));
    for _ in 0..faces {
        let t = [r.u32()?, r.u32()?, r.u32()?];
        if t.iter().any(|i| *i as usize >= points) {
            return None;
        }
        triangles.push(t);
    }
    let count = r.u32()? as usize;
    let mut attributes = Vec::new();
    for _ in 0..count {
        let unique_id = r.u32()?;
        let width = *r.take(1)?.first()?;
        if !(1..=4).contains(&width) {
            return None;
        }
        let scale = f64::from_le_bytes(r.take(8)?.try_into().ok()?);
        let scale = (scale.is_finite() && scale > 0.0).then_some(scale);
        let n = points.checked_mul(usize::from(width))?;
        let raw = r.take(n.checked_mul(4)?)?;
        let values = raw.as_chunks::<4>().0.iter().map(|b| f32::from_le_bytes(*b)).collect();
        attributes.push(Attribute { unique_id, width, scale, values });
    }
    r.bytes.is_empty().then_some(Decoded { points, triangles, attributes })
}

/// Reads little-endian values off the front of a slice.
struct Reader<'a> {
    bytes: &'a [u8],
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let (head, rest) = self.bytes.split_at_checked(n)?;
        self.bytes = rest;
        Some(head)
    }

    fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }
}

/// Delete the oldest entries (by modification time) until the cache's
/// entries take at most `max` bytes.
fn trim(dir: &Path, max: u64) {
    let Ok(read) = std::fs::read_dir(dir) else { return };
    let mut entries: Vec<(std::time::SystemTime, u64, PathBuf)> = read
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|x| x == CACHE_EXT))
        .filter_map(|e| {
            let m = e.metadata().ok().filter(std::fs::Metadata::is_file)?;
            Some((m.modified().unwrap_or(std::time::UNIX_EPOCH), m.len(), e.path()))
        })
        .collect();
    let mut total: u64 = entries.iter().map(|(_, len, _)| len).sum();
    entries.sort();
    for (_, len, path) in entries {
        if total <= max {
            break;
        }
        if std::fs::remove_file(&path).is_ok() {
            total = total.saturating_sub(len);
        }
    }
}

/// A compressed mesh: the stream and what a container needs to describe it.
#[derive(Debug, Clone, PartialEq)]
pub struct Encoded {
    pub bytes: Vec<u8>,
    /// Points the decoder gives back (the accessors' count).
    pub points: usize,
    /// Triangles the decoder gives back.
    pub faces: usize,
    /// The stream's unique ids of the position, normal and texture
    /// coordinate attributes (those that were given).
    pub position: u32,
    pub normal: Option<u32>,
    pub uv: Option<u32>,
}

/// An attribute of `width` floats per point.
fn attribute(kind: GeometryAttributeType, width: u8, values: &[f32]) -> Result<PointAttribute, String> {
    let points = values.len() / usize::from(width.max(1));
    let mut att = PointAttribute::new();
    att.try_init(kind, width, DataType::Float32, false, points).map_err(|e| e.to_string())?;
    att.buffer_mut().update_f32s_le(0, values);
    Ok(att)
}

/// Compress a triangle mesh: `positions` (and `normals`, `uvs` when not
/// empty, one per position) and `indices`, three per triangle. Positions,
/// normals and texture coordinates are quantized ([`POSITION_BITS`],
/// [`NORMAL_BITS`], [`UV_BITS`]); the same mesh always gives the same bytes.
pub fn encode(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    uvs: &[[f32; 2]],
    indices: &[u32],
) -> Result<Encoded, String> {
    let n = positions.len();
    if n == 0 || indices.is_empty() || !indices.len().is_multiple_of(3) {
        return Err("Draco: an empty or broken mesh".into());
    }
    if indices.iter().any(|i| *i as usize >= n)
        || (!normals.is_empty() && normals.len() != n)
        || (!uvs.is_empty() && uvs.len() != n)
    {
        return Err("Draco: attribute counts do not match".into());
    }
    if !positions.iter().flatten().chain(normals.iter().flatten()).chain(uvs.iter().flatten()).all(|v| v.is_finite()) {
        return Err("Draco: a value is not finite".into());
    }
    let flat3 = |v: &[[f32; 3]]| v.iter().flatten().copied().collect::<Vec<f32>>();
    let run = || -> Result<Encoded, String> {
        let mut mesh = Mesh::new();
        mesh.set_num_points(n);
        mesh.try_set_num_faces(indices.len() / 3).map_err(|e| format!("Draco: {e}"))?;
        mesh.set_faces_from_flat_indices(indices);
        let mut options = EncoderOptions::new();
        options.set_global_int("encoding_speed", SPEED);
        options.set_global_int("decoding_speed", SPEED);
        let position = mesh.add_attribute(attribute(GeometryAttributeType::Position, 3, &flat3(positions))?);
        options.set_attribute_quantization(position, POSITION_BITS);
        let normal = if normals.is_empty() {
            None
        } else {
            let id = mesh.add_attribute(attribute(GeometryAttributeType::Normal, 3, &flat3(normals))?);
            options.set_attribute_quantization(id, NORMAL_BITS);
            Some(id)
        };
        let uv = if uvs.is_empty() {
            None
        } else {
            let flat: Vec<f32> = uvs.iter().flatten().copied().collect();
            let id = mesh.add_attribute(attribute(GeometryAttributeType::TexCoord, 2, &flat)?);
            options.set_attribute_quantization(id, UV_BITS);
            Some(id)
        };
        let mut encoder = MeshEncoder::new();
        encoder.set_mesh(mesh);
        let mut buffer = EncoderBuffer::new();
        let info = encoder.encode_with_info(&options, &mut buffer).map_err(|e| format!("Draco: {e}"))?;
        let unique = |id: i32| {
            info.attributes.iter().find(|a| a.source_attribute_id == id).map(|a| a.unique_id).ok_or_else(|| {
                "Draco: an attribute was not encoded".to_string() //
            })
        };
        Ok(Encoded {
            bytes: buffer.data().to_vec(),
            points: info.num_encoded_points,
            faces: info.num_encoded_faces,
            position: unique(position)?,
            normal: normal.map(unique).transpose()?,
            uv: uv.map(unique).transpose()?,
        })
    };
    catch_unwind(AssertUnwindSafe(run)).map_err(|_| "Draco: the encoder failed".to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Positions, normals, texture coordinates and indices.
    type Parts = (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>);

    /// A unit cube, a vertex per face corner (as glTF meshes are).
    fn cube() -> Parts {
        let (mut p, mut n, mut t, mut i) = (vec![], vec![], vec![], vec![]);
        for axis in 0..3 {
            for side in [0.0f32, 1.0] {
                let base = p.len() as u32;
                for (u, v) in [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)] {
                    let mut q = [0.0; 3];
                    q[axis] = side;
                    q[(axis + 1) % 3] = u;
                    q[(axis + 2) % 3] = v;
                    p.push(q);
                    let mut normal = [0.0; 3];
                    normal[axis] = if side == 0.0 { -1.0 } else { 1.0 };
                    n.push(normal);
                    t.push([u, v]);
                }
                i.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
            }
        }
        (p, n, t, i)
    }

    #[test]
    fn a_mesh_survives_a_round_trip() {
        let (p, n, t, i) = cube();
        let out = encode(&p, &n, &t, &i).unwrap();
        assert_eq!(out.faces, 12);
        assert_eq!(encode(&p, &n, &t, &i).unwrap(), out, "the same mesh gives the same bytes");
        let back = decode(&out.bytes).unwrap();
        assert_eq!(back.points(), out.points);
        let positions = back.attribute(out.position, 3).unwrap();
        let normals = back.attribute(out.normal.unwrap(), 3).unwrap();
        let uvs = back.attribute(out.uv.unwrap(), 2).unwrap();
        assert!(back.attribute(out.position, 2).is_none(), "the width must match");
        let triangles = back.triangles();
        assert_eq!(triangles.len(), 12);
        // Each decoded triangle is one of the cube's, with its normal and corners.
        let corners = |k: usize| [positions[3 * k], positions[3 * k + 1], positions[3 * k + 2]];
        let mut area = 0.0;
        for [a, b, c] in triangles {
            let (a, b, c) = (corners(a), corners(b), corners(c));
            for v in a.iter().chain(&b).chain(&c) {
                assert!((v - v.round()).abs() < 1e-4, "{v} is a cube corner");
            }
            let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let w = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let cross = [u[1] * w[2] - u[2] * w[1], u[2] * w[0] - u[0] * w[2], u[0] * w[1] - u[1] * w[0]];
            area += (cross.iter().map(|x| x * x).sum::<f64>()).sqrt() / 2.0;
        }
        assert!((area - 6.0).abs() < 1e-3, "the cube's area is {area}");
        assert!(normals.chunks(3).all(|n| (n.iter().map(|x| x * x).sum::<f64>() - 1.0).abs() < 1e-2));
        assert!(uvs.iter().all(|v| (-1e-3..=1.001).contains(v)));
    }

    #[test]
    fn bad_meshes_are_refused() {
        let (p, n, t, i) = cube();
        assert!(encode(&[], &[], &[], &[]).is_err());
        assert!(encode(&p, &n, &t, &[0, 1]).is_err());
        assert!(encode(&p, &n, &t, &[0, 1, 99]).is_err());
        assert!(encode(&p, &n[..3], &t, &i).is_err());
        let mut nan = p.clone();
        nan[0][0] = f32::NAN;
        assert!(encode(&nan, &n, &t, &i).is_err());
    }

    /// A grid of `n`×`n` squares, big enough for the cache to keep.
    fn grid(n: u32) -> Vec<u8> {
        let (mut p, mut i) = (vec![], vec![]);
        for y in 0..=n {
            for x in 0..=n {
                p.push([x as f32 * 0.1, y as f32 * 0.1, ((x * 7919 + y * 104_729) % 997) as f32 * 0.001]);
            }
        }
        for y in 0..n {
            for x in 0..n {
                let a = y * (n + 1) + x;
                i.extend([a, a + 1, a + n + 2, a, a + n + 2, a + n + 1]);
            }
        }
        encode(&p, &[], &[], &i).unwrap().bytes
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ccad-draco-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    fn entries(dir: &Path) -> Vec<PathBuf> {
        let mut v: Vec<PathBuf> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
        v.sort();
        v
    }

    #[test]
    fn decoded_meshes_are_kept_by_their_hash() {
        let dir = scratch("keep");
        let stream = grid(100);
        assert!(stream.len() >= CACHE_MIN_STREAM, "the grid's stream is {} bytes", stream.len());
        let fresh = decode_stream(&stream).unwrap();
        assert_eq!(decode_cached(&stream, &dir).unwrap(), fresh);
        let files = entries(&dir);
        assert_eq!(files.len(), 1);
        let name = files[0].file_stem().unwrap().to_str().unwrap().to_string();
        assert_eq!(name, hex(&sha256(&stream)), "the entry is named by the stream's SHA-256");
        // The second time it is read back from the entry.
        assert_eq!(read_entry(&files[0], &sha256(&stream)).unwrap(), fresh);
        assert_eq!(decode_cached(&stream, &dir).unwrap(), fresh);
        // Small streams are not kept.
        let (p, n, t, i) = cube();
        let small = encode(&p, &n, &t, &i).unwrap().bytes;
        assert!(decode_cached(&small, &dir).is_some());
        assert_eq!(entries(&dir).len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn damaged_or_foreign_entries_are_not_used() {
        let dir = scratch("damaged");
        let stream = grid(100);
        let fresh = decode_stream(&stream).unwrap();
        decode_cached(&stream, &dir).unwrap();
        let path = entries(&dir).remove(0);
        let good = std::fs::read(&path).unwrap();
        let key = sha256(&stream);
        // A changed value, a cut file, a wrong magic: refused, and the next decode rewrites the entry.
        for at in [good.len() - 1, 80, 3] {
            let mut bad = good.clone();
            bad[at] ^= 0x01;
            std::fs::write(&path, &bad).unwrap();
            assert!(read_entry(&path, &key).is_none(), "a flipped byte at {at} is caught");
            assert_eq!(decode_cached(&stream, &dir).unwrap(), fresh);
            assert_eq!(std::fs::read(&path).unwrap(), good, "the entry is written again");
        }
        std::fs::write(&path, &good[..good.len() / 2]).unwrap();
        assert_eq!(decode_cached(&stream, &dir).unwrap(), fresh);
        // Another stream's entry under this name: its stream hash does not match.
        let other = grid(101);
        let other_entry = entry_bytes(&sha256(&other), &decode_stream(&other).unwrap()).unwrap();
        std::fs::write(&path, &other_entry).unwrap();
        assert!(read_entry(&path, &key).is_none());
        assert_eq!(decode_cached(&stream, &dir).unwrap(), fresh);
        // Every truncation of an entry parses or not, without panicking.
        for len in (0..good.len()).step_by(97) {
            let _ = parse_entry(&good[..len], &key);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_cache_is_trimmed_oldest_first() {
        let dir = scratch("trim");
        let (a, b) = (grid(100), grid(102));
        decode_cached(&a, &dir).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(30));
        decode_cached(&b, &dir).unwrap();
        let newest = dir.join(format!("{}.{CACHE_EXT}", hex(&sha256(&b))));
        trim(&dir, std::fs::metadata(&newest).unwrap().len());
        assert_eq!(entries(&dir), vec![newest]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn broken_streams_are_none() {
        assert!(decode(b"").is_none());
        assert!(decode(b"DRACO\x02\x02\x01\x01\0\0\xff\xff\xff\xff").is_none());
        let (p, n, t, i) = cube();
        let bytes = encode(&p, &n, &t, &i).unwrap().bytes;
        // Every truncation and a flipped byte at every place: no panic.
        for len in 0..bytes.len() {
            let _ = decode(&bytes[..len]);
        }
        for at in 0..bytes.len() {
            let mut b = bytes.clone();
            b[at] ^= 0x5A;
            let _ = decode(&b);
        }
    }
}
