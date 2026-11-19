use crate::polygon;
use cc_core::{Category, ElementId};
use glam::{DVec2, DVec3};

/// A planar polygon face with an outward normal.
#[derive(Debug, Clone, PartialEq)]
pub struct Face {
    pub verts: Vec<DVec3>,
    pub normal: DVec3,
    /// A facet of a curved surface: edges shared with neighboring smooth
    /// facets are not outlines (only silhouettes, see [`Solid::smooth_edges`]).
    pub smooth: bool,
}

/// Unit normal of a polygon's winding (Newell's method); zero when it has no area.
fn newell(verts: &[DVec3]) -> DVec3 {
    let n = verts.len();
    (0..n)
        .map(|i| {
            let (a, b) = (verts[i], verts[(i + 1) % n]);
            DVec3::new((a.y - b.y) * (a.z + b.z), (a.z - b.z) * (a.x + b.x), (a.x - b.x) * (a.y + b.y))
        })
        .sum::<DVec3>()
        .normalize_or_zero()
}

/// Smooth facets meeting at less than this angle (cosine) read as one surface.
pub(crate) const SMOOTH_COS: f64 = 0.85;

/// Faces whose normals are closer than this (cosine) lie in one plane.
const COPLANAR_COS: f64 = 0.9999;

impl Face {
    pub fn new(verts: Vec<DVec3>, normal: DVec3) -> Self {
        Self { verts, normal: normal.normalize_or_zero(), smooth: false }
    }

    /// A polygon with the normal of its winding (Newell's method); `None`
    /// when it has fewer than three points or no area.
    pub fn of_polygon(verts: Vec<DVec3>) -> Option<Self> {
        let n = verts.len();
        let normal: DVec3 = verts
            .iter()
            .zip(verts.iter().cycle().skip(1))
            .map(|(a, b)| DVec3::new((a.y - b.y) * (a.z + b.z), (a.z - b.z) * (a.x + b.x), (a.x - b.x) * (a.y + b.y)))
            .sum();
        (n >= 3 && normal.length_squared() > 1e-20 && normal.is_finite()).then(|| Face::new(verts, normal))
    }

    /// Orthonormal in-plane basis `(u, v)`.
    pub(crate) fn basis(&self) -> (DVec3, DVec3) {
        // A face built without a normal (its points on one line, or a normal
        // that was zero) gets the normal of its points; with none, any plane.
        let n = if self.normal.length_squared() > 0.5 { self.normal } else { newell(&self.verts) };
        let n = if n.is_finite() && n != DVec3::ZERO { n } else { DVec3::Z };
        let helper = if n.z.abs() < 0.9 { DVec3::Z } else { DVec3::X };
        let u = n.cross(helper).normalize();
        (u, n.cross(u))
    }

    /// Triangles as index triples into `verts`.
    pub fn triangulate(&self) -> Vec<[usize; 3]> {
        let (u, v) = self.basis();
        let flat: Vec<DVec2> = self.verts.iter().map(|p| DVec2::new(p.dot(u), p.dot(v))).collect();
        // A quad splits along a diagonal inside it (merged faces need not be convex).
        if let [a, b, c, d] = flat[..] {
            let turn = |p: DVec2, q: DVec2, r: DVec2| (q - p).perp_dot(r - p);
            let ccw = turn(a, b, c) + turn(a, c, d) >= 0.0;
            let inside = |p: DVec2, q: DVec2, r: DVec2| (turn(p, q, r) >= 0.0) == ccw;
            return if inside(a, b, c) && inside(a, c, d) {
                vec![[0, 1, 2], [0, 2, 3]]
            } else {
                vec![[1, 2, 3], [1, 3, 0]]
            };
        }
        polygon::triangulate(&flat)
    }

    pub fn centroid(&self) -> DVec3 {
        self.verts.iter().copied().sum::<DVec3>() / self.verts.len().max(1) as f64
    }

    /// Area of the planar polygon.
    pub fn area(&self) -> f64 {
        let n = self.verts.len();
        let sum: DVec3 = (0..n).map(|i| self.verts[i].cross(self.verts[(i + 1) % n])).sum();
        sum.dot(self.normal).abs() * 0.5
    }
}

/// Boundary representation of one element (or a part of one).
#[derive(Debug, Clone)]
pub struct Solid {
    pub element: ElementId,
    pub category: Category,
    pub color: [u8; 3],
    /// 0 (opaque) to 1 (invisible), from the material.
    pub transparency: f64,
    pub faces: Vec<Face>,
    /// A part the element carries that is not its body (a roof's framing):
    /// drawn, but left out of its volume, joins and exported body.
    pub secondary: bool,
    /// The detail levels of the views that draw it (a component's piece).
    pub detail: cc_core::component::DetailLevels,
    /// The material's appearance color and surface pattern (the textures style).
    pub appearance: [u8; 3],
    pub surface: cc_core::PatternFill,
    /// The material the solid is made of (its texture in the textures style).
    pub material: Option<String>,
    /// Faces painted with another material, by face index (see [`crate::paint`]).
    pub painted: std::collections::BTreeMap<usize, Look>,
    /// Faces of the pieces this one runs on into (see [`join_pieces`]). They
    /// are not its own and no view draws them, but they hide the edges it
    /// shares with them, and shade and make silhouettes with its facets, as
    /// its own faces do.
    pub joined: Vec<Face>,
}

/// The axes a texture is laid along on a face with normal `n`: plan X and Y
/// on faces turned up or down, else level across the face (to the right,
/// seen from outside) and up its slope. A point's plane coordinates are its
/// dot products with them, in meters (see [`Solid::facet_shifts`]).
pub fn plane_axes(n: DVec3) -> (DVec3, DVec3) {
    if n.z.abs() > 0.9 {
        return (DVec3::X, if n.z >= 0.0 { DVec3::Y } else { -DVec3::Y });
    }
    let u = DVec3::new(-n.y, n.x, 0.0).normalize_or(DVec3::X);
    (u, n.cross(u).normalize_or(DVec3::Z))
}

/// How a material shows on a face: shading color, appearance color, surface
/// pattern and transparency, and the material's name (for its texture).
#[derive(Debug, Clone, PartialEq)]
pub struct Look {
    pub material: Option<String>,
    pub color: [u8; 3],
    pub appearance: [u8; 3],
    pub surface: cc_core::PatternFill,
    pub transparency: f64,
}

impl Look {
    /// The look of material `name`; `None` when the project has no such material.
    pub fn of(project: &cc_core::Project, name: &str) -> Option<Self> {
        let m = project.material(name)?;
        Some(Self {
            material: Some(m.name.clone()),
            color: m.shading_color(),
            appearance: m.appearance.color,
            surface: m.graphics.surface,
            transparency: project.material_transparency(name),
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: DVec3,
    pub dir: DVec3,
}

impl Solid {
    pub fn new(element: ElementId, category: Category, color: [u8; 3]) -> Self {
        Self {
            element,
            category,
            color,
            transparency: 0.0,
            faces: Vec::new(),
            secondary: false,
            detail: Default::default(),
            appearance: color,
            surface: Default::default(),
            material: None,
            painted: Default::default(),
            joined: Vec::new(),
        }
    }

    /// Face `i` of the solid, or of [`Self::joined`] (numbered after its own).
    fn face_or_joined(&self, i: usize) -> Option<&Face> {
        self.faces.get(i).or_else(|| self.joined.get(i.checked_sub(self.faces.len())?))
    }

    /// The boundary edges of the solid's faces, then of [`Self::joined`]
    /// (their faces numbered after its own).
    fn edges_with_joined(&self) -> Vec<(usize, DVec3, DVec3)> {
        let own = self.faces.len();
        let mut edges = boundary_edges(&self.faces);
        edges.extend(boundary_edges(&self.joined).into_iter().map(|(f, a, b)| (f + own, a, b)));
        edges
    }

    /// Which faces lie against the face of a joined piece (see
    /// [`join_pieces`]): they are inside the body the pieces make, so a view
    /// has nothing of them to draw.
    pub fn inner_faces(&self) -> Vec<bool> {
        if self.joined.is_empty() {
            return vec![false; self.faces.len()];
        }
        let points = |face: &Face| {
            let mut keys: Vec<(i64, i64, i64)> = face
                .verts
                .iter()
                .map(|p| {
                    let q = (*p * 1e5).round();
                    (q.x as i64, q.y as i64, q.z as i64)
                })
                .collect();
            keys.sort_unstable();
            keys.dedup();
            keys
        };
        let mut against: std::collections::HashMap<_, Vec<DVec3>> = std::collections::HashMap::new();
        for face in &self.joined {
            against.entry(points(face)).or_default().push(face.normal);
        }
        self.faces
            .iter()
            .map(|face| {
                let others = against.get(&points(face));
                others.is_some_and(|normals| normals.iter().any(|n| n.dot(face.normal) <= -COPLANAR_COS))
            })
            .collect()
    }

    /// Made of the same thing and shown by the same views: pieces that can
    /// read as one body (see [`join_pieces`]).
    fn alike(&self, other: &Solid) -> bool {
        self.element == other.element
            && self.category == other.category
            && self.color == other.color
            && self.transparency == other.transparency
            && self.secondary == other.secondary
            && self.detail == other.detail
            && self.appearance == other.appearance
            && self.surface == other.surface
            && self.material == other.material
            && self.painted.is_empty()
            && other.painted.is_empty()
    }

    /// An empty solid colored and made transparent like material `name`.
    pub fn of_material(element: ElementId, category: Category, project: &cc_core::Project, name: &str) -> Self {
        let mut s = Self {
            transparency: project.material_transparency(name),
            ..Self::new(element, category, project.material_color(name))
        };
        s.set_appearance(project, name);
        s
    }

    /// Take the appearance color and surface pattern of material `name`
    /// (what the textures style shows).
    pub fn set_appearance(&mut self, project: &cc_core::Project, name: &str) {
        if let Some(m) = project.material(name) {
            self.appearance = m.appearance.color;
            self.surface = m.graphics.surface;
            self.material = Some(m.name.clone());
        }
    }

    /// How the solid's own material shows (its faces that are not painted).
    pub fn look(&self) -> Look {
        Look {
            material: self.material.clone(),
            color: self.color,
            appearance: self.appearance,
            surface: self.surface,
            transparency: self.transparency,
        }
    }

    /// The paint of face `face`, if it is painted.
    pub fn paint_of(&self, face: usize) -> Option<&Look> {
        self.painted.get(&face)
    }

    /// The nearest face along `ray`: its distance and index.
    pub fn raycast_face(&self, ray: Ray) -> Option<(f64, usize)> {
        let mut best: Option<(f64, usize)> = None;
        for (i, face) in self.faces.iter().enumerate() {
            for [a, b, c] in face.triangulate() {
                let (Some(&pa), Some(&pb), Some(&pc)) = (face.verts.get(a), face.verts.get(b), face.verts.get(c))
                else {
                    continue;
                };
                if let Some(t) = ray_triangle(ray, pa, pb, pc)
                    && best.is_none_or(|(bt, _)| t < bt)
                {
                    best = Some((t, i));
                }
            }
        }
        best
    }

    pub fn bounds(&self) -> Option<(DVec3, DVec3)> {
        let mut it = self.faces.iter().flat_map(|f| f.verts.iter().copied());
        let first = it.next()?;
        Some(it.fold((first, first), |(lo, hi), p| (lo.min(p), hi.max(p))))
    }

    /// Volume via the divergence theorem (faces must be closed and outward).
    pub fn volume(&self) -> f64 {
        self.faces.iter().map(|f| f.centroid().dot(f.normal) * f.area()).sum::<f64>() / 3.0
    }

    /// Visible outline edges grouped by face: the parts of each face boundary
    /// not shared with a coplanar neighbor face (so split faces read as one),
    /// of the solid or of the pieces it is joined to ([`Self::joined`]).
    pub fn feature_edges(&self) -> Vec<Vec<(DVec3, DVec3)>> {
        let all = self.edges_with_joined();
        let own = self.faces.len();
        // Edges of other faces with the same normal (or smooth neighbors) hide what they overlap.
        let joins = |fi: usize, fj: usize| {
            // Only the solid's own edges are drawn.
            if fi >= own {
                return false;
            }
            let (Some(face), Some(other)) = (self.face_or_joined(fi), self.face_or_joined(fj)) else { return false };
            let cos = other.normal.dot(face.normal);
            // A face lying against the face of a joined piece is inside the body: none of its edges show.
            cos >= COPLANAR_COS
                || (face.smooth && other.smooth && cos > SMOOTH_COS)
                || (fj >= own && cos <= -COPLANAR_COS)
        };
        let mut covers: Vec<Vec<(f64, f64)>> = vec![Vec::new(); all.len()];
        for_each_overlap(&all, joins, |i, _, lo, hi| {
            if let Some(list) = covers.get_mut(i) {
                list.push((lo, hi));
            }
        });
        let mut out = vec![Vec::new(); self.faces.len()];
        for (&(fi, a, b), mut covered) in all.iter().zip(covers) {
            let Some(face_out) = out.get_mut(fi) else { continue };
            let ab = b - a;
            if ab.length_squared() < 1e-12 {
                continue;
            }
            covered.sort_by(|x, y| x.0.total_cmp(&y.0));
            let mut t = 0.0;
            for (lo, hi) in covered {
                if lo > t + 1e-9 {
                    face_out.push((a + ab * t, a + ab * lo));
                }
                t = t.max(hi);
            }
            if t < 1.0 - 1e-9 {
                face_out.push((a + ab * t, b));
            }
        }
        out
    }

    /// The shading normal at each vertex of each face: a flat face's own
    /// normal, and on a smooth facet the average of the normals of the
    /// smooth facets meeting at that point (those that read as one surface
    /// with it), each weighted by its angle there, so a curved surface
    /// shades without steps however its facets were split.
    pub fn vertex_normals(&self) -> Vec<Vec<DVec3>> {
        // Facets meet at the same points; look them up on a fine grid.
        let key = |p: DVec3| {
            let q = (p * 1e5).round();
            (q.x as i64, q.y as i64, q.z as i64)
        };
        let mut at: std::collections::HashMap<(i64, i64, i64), Vec<(usize, f64)>> = std::collections::HashMap::new();
        // The facets of joined pieces shade with the solid's own.
        for (fi, face) in self.faces.iter().chain(&self.joined).enumerate().filter(|(_, f)| f.smooth) {
            for (p, angle) in face.verts.iter().zip(corner_angles(&face.verts)) {
                at.entry(key(*p)).or_default().push((fi, angle));
            }
        }
        self.faces
            .iter()
            .map(|face| {
                if !face.smooth {
                    return vec![face.normal; face.verts.len()];
                }
                face.verts
                    .iter()
                    .map(|p| {
                        let sum: DVec3 = at
                            .get(&key(*p))
                            .into_iter()
                            .flatten()
                            .filter_map(|(fj, angle)| Some((self.face_or_joined(*fj)?.normal, *angle)))
                            .filter(|(n, _)| n.dot(face.normal) > SMOOTH_COS)
                            .map(|(n, angle)| n * angle)
                            .sum();
                        let n = sum.normalize_or_zero();
                        if n == DVec3::ZERO || !n.is_finite() { face.normal } else { n }
                    })
                    .collect()
            })
            .collect()
    }

    /// What to add to the plane coordinates of each face (see [`plane_axes`])
    /// so they run on from one facet of a curved surface to the next: every
    /// facet has its own axes, so without it patterns and textures would
    /// start again at each one. Facets are reached from the first of their
    /// surface across shared edges and agree with the facet they were reached
    /// from at the middle of that edge (exactly along it when it runs up the
    /// surface, as on an arc wall); a surface that closes on itself keeps one
    /// seam.
    pub fn facet_shifts(&self) -> Vec<[f64; 2]> {
        let neighbors = self.smooth_neighbors();
        let coords = |fi: usize, p: DVec3| {
            let (u, v) = plane_axes(self.faces.get(fi)?.normal);
            Some([p.dot(u), p.dot(v)])
        };
        let mut shifts = vec![[0.0; 2]; self.faces.len()];
        let mut seen = vec![false; self.faces.len()];
        let mut queue = std::collections::VecDeque::new();
        for seed in 0..self.faces.len() {
            if seen.get(seed) != Some(&false) || neighbors.get(seed).is_none_or(|n| n.is_empty()) {
                continue;
            }
            if let Some(s) = seen.get_mut(seed) {
                *s = true;
            }
            queue.push_back(seed);
            while let Some(fi) = queue.pop_front() {
                let base = shifts.get(fi).copied().unwrap_or_default();
                for &(fj, mid) in neighbors.get(fi).into_iter().flatten() {
                    if seen.get(fj) != Some(&false) {
                        continue;
                    }
                    let (Some(here), Some(there)) = (coords(fi, mid), coords(fj, mid)) else { continue };
                    let shift = [base[0] + here[0] - there[0], base[1] + here[1] - there[1]];
                    if let (Some(s), Some(slot)) = (seen.get_mut(fj), shifts.get_mut(fj)) {
                        *s = true;
                        if shift.iter().all(|c| c.is_finite()) {
                            *slot = shift;
                        }
                        queue.push_back(fj);
                    }
                }
            }
        }
        shifts
    }

    /// Per face, the smooth facets that read as one surface with it across a
    /// shared edge, each with that edge's midpoint (none for a flat face).
    pub fn smooth_neighbors(&self) -> Vec<Vec<(usize, DVec3)>> {
        // Facets share their edges' ends; look them up on a fine grid.
        let key = |p: DVec3| {
            let q = (p * 1e5).round();
            (q.x as i64, q.y as i64, q.z as i64)
        };
        let edge = |a: DVec3, b: DVec3| {
            let (a, b) = (key(a), key(b));
            if a <= b { (a, b) } else { (b, a) }
        };
        let sides = |face: &Face| -> Vec<(DVec3, DVec3)> {
            face.verts.iter().copied().zip(face.verts.iter().copied().cycle().skip(1)).collect()
        };
        let mut at: std::collections::HashMap<_, Vec<usize>> = std::collections::HashMap::new();
        for (fi, face) in self.faces.iter().enumerate().filter(|(_, f)| f.smooth) {
            for (a, b) in sides(face) {
                at.entry(edge(a, b)).or_default().push(fi);
            }
        }
        self.faces
            .iter()
            .enumerate()
            .map(|(fi, face)| {
                if !face.smooth {
                    return Vec::new();
                }
                let mut out = Vec::new();
                for (a, b) in sides(face) {
                    for &fj in at.get(&edge(a, b)).into_iter().flatten() {
                        let joins = self.faces.get(fj).is_some_and(|other| other.normal.dot(face.normal) > SMOOTH_COS);
                        if fj != fi && joins {
                            out.push((fj, (a + b) * 0.5));
                        }
                    }
                }
                out
            })
            .collect()
    }

    /// Edges between smooth facets (hidden by [`Self::feature_edges`]), per
    /// face, with the neighbor's normal: a view draws one as a silhouette
    /// where the face is seen and its neighbor is not.
    pub fn smooth_edges(&self) -> Vec<Vec<(DVec3, DVec3, DVec3)>> {
        let mut out = vec![Vec::new(); self.faces.len()];
        if !self.faces.iter().any(|f| f.smooth) {
            return out;
        }
        let edges = self.edges_with_joined();
        let own = self.faces.len();
        // Facets in one plane (the pieces of a split facet) make no silhouette.
        let turns = |fi: usize, fj: usize| {
            if fi >= own {
                return false;
            }
            let (Some(face), Some(other)) = (self.face_or_joined(fi), self.face_or_joined(fj)) else { return false };
            let cos = other.normal.dot(face.normal);
            face.smooth && other.smooth && cos > SMOOTH_COS && cos < COPLANAR_COS
        };
        // The pieces of facets split by a boolean (meeting in T-junctions)
        // still find each other.
        for_each_neighbor(&edges, turns, |i, j, lo, hi| {
            let (Some(&(fi, a, b)), Some(&(fj, ..))) = (edges.get(i), edges.get(j)) else { return };
            let (Some(other), Some(list)) = (self.face_or_joined(fj), out.get_mut(fi)) else { return };
            list.push((a.lerp(b, lo), a.lerp(b, hi), other.normal));
        });
        out
    }

    /// Nearest hit distance along `ray`, if any.
    pub fn raycast(&self, ray: Ray) -> Option<f64> {
        self.raycast_within(ray, f64::INFINITY)
    }

    /// Like [`Self::raycast`], for a caller that already has a hit at
    /// `limit`: faces whose boxes the ray meets beyond it are skipped (a
    /// result beyond `limit` is not then the nearest).
    pub fn raycast_within(&self, ray: Ray, limit: f64) -> Option<f64> {
        let mut best: Option<f64> = None;
        for face in &self.faces {
            let Some(first) = face.verts.first() else { continue };
            let (lo, hi) = face.verts.iter().fold((*first, *first), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
            if ray_box(ray, lo, hi).is_none_or(|t| t > best.map_or(limit, |b| b.min(limit))) {
                continue;
            }
            for [a, b, c] in face.triangulate() {
                if let Some(t) = ray_triangle(ray, face.verts[a], face.verts[b], face.verts[c]) {
                    best = Some(best.map_or(t, |bt| bt.min(t)));
                }
            }
        }
        best
    }

    /// Append planar polygons, each with the normal of its winding (Newell's
    /// method); degenerate polygons are dropped.
    pub fn add_polygons(&mut self, polygons: impl IntoIterator<Item = Vec<DVec3>>) {
        self.faces.extend(polygons.into_iter().filter_map(Face::of_polygon));
    }

    /// Mark smooth the facets of curved surfaces among the faces from index
    /// `from` on (see [`curved_facets`]), for solids whose builder does not
    /// know which they are (component pieces, imported meshes).
    pub fn smooth_curved(&mut self, from: usize) {
        let Some(faces) = self.faces.get_mut(from..) else { return };
        let flags = curved(faces);
        for (face, curved) in faces.iter_mut().zip(flags) {
            face.smooth |= curved;
        }
    }

    /// Append an axis-aligned (in the local frame) box.
    ///
    /// `origin` is the frame origin, `u`/`v` horizontal unit axes, Z is up.
    #[allow(clippy::too_many_arguments)]
    pub fn add_box(&mut self, origin: DVec2, u: DVec2, v: DVec2, u0: f64, u1: f64, v0: f64, v1: f64, z0: f64, z1: f64) {
        if u1 - u0 < 1e-6 || v1 - v0 < 1e-6 || z1 - z0 < 1e-6 {
            return;
        }
        let p = |a: f64, b: f64, z: f64| {
            let q = origin + u * a + v * b;
            DVec3::new(q.x, q.y, z)
        };
        let u3 = DVec3::new(u.x, u.y, 0.0);
        let v3 = DVec3::new(v.x, v.y, 0.0);
        let f = &mut self.faces;
        f.push(Face::new(vec![p(u0, v0, z1), p(u1, v0, z1), p(u1, v1, z1), p(u0, v1, z1)], DVec3::Z));
        f.push(Face::new(vec![p(u0, v0, z0), p(u0, v1, z0), p(u1, v1, z0), p(u1, v0, z0)], -DVec3::Z));
        f.push(Face::new(vec![p(u0, v0, z0), p(u1, v0, z0), p(u1, v0, z1), p(u0, v0, z1)], -v3));
        f.push(Face::new(vec![p(u1, v1, z0), p(u0, v1, z0), p(u0, v1, z1), p(u1, v1, z1)], v3));
        f.push(Face::new(vec![p(u0, v1, z0), p(u0, v0, z0), p(u0, v0, z1), p(u0, v1, z1)], -u3));
        f.push(Face::new(vec![p(u1, v0, z0), p(u1, v1, z0), p(u1, v1, z1), p(u1, v0, z1)], u3));
    }

    /// Append a prism like [`Self::add_prism`] with openings: `holes` are
    /// outlines inside `footprint` (either winding).
    ///
    /// Faces are pushed caps first (top and bottom triangles), then one side
    /// per edge, last: the edges of `polygon::ccw(footprint)`, then those of
    /// each hole of three points or more, in the given order, as
    /// `polygon::ccw(hole)` reversed (clockwise). Without holes it is
    /// [`Self::add_prism`]. `section::add_column_body` relies on this order.
    pub fn add_prism_with_holes(
        &mut self,
        footprint: &[DVec2],
        holes: &[Vec<DVec2>],
        bottom: impl Fn(DVec2) -> f64,
        top: impl Fn(DVec2) -> f64,
    ) {
        if holes.is_empty() {
            self.add_prism(footprint, bottom, top);
            return;
        }
        let fp = polygon::ccw(footprint);
        if fp.len() < 3 {
            return;
        }
        let holes: Vec<Vec<DVec2>> = holes
            .iter()
            .filter(|h| h.len() >= 3)
            .map(|h| {
                let mut h = polygon::ccw(h);
                h.reverse();
                h
            })
            .collect();
        let lift = |p: DVec2, z: f64| DVec3::new(p.x, p.y, z);
        for t in polygon::triangulate_with_holes(&fp, &holes) {
            let top_pts: Vec<DVec3> = t.iter().map(|&p| lift(p, top(p))).collect();
            let bottom_pts: Vec<DVec3> = t.iter().rev().map(|&p| lift(p, bottom(p))).collect();
            let n = |pts: &[DVec3]| (pts[1] - pts[0]).cross(pts[2] - pts[0]);
            let (tn, bn) = (n(&top_pts), n(&bottom_pts));
            self.faces.push(Face::new(top_pts, tn));
            self.faces.push(Face::new(bottom_pts, bn));
        }
        // Sides: the outline counter-clockwise, the holes clockwise, all facing out of the solid.
        for ring in std::iter::once(&fp).chain(holes.iter()) {
            let n = ring.len();
            for i in 0..n {
                let (a, b) = (ring[i], ring[(i + 1) % n]);
                let out = -(b - a).normalize_or_zero().perp();
                self.faces.push(Face::new(
                    vec![lift(a, bottom(a)), lift(b, bottom(b)), lift(b, top(b)), lift(a, top(a))],
                    DVec3::new(out.x, out.y, 0.0),
                ));
            }
        }
    }

    /// Append a prism between two copies of a polygon: `bottom(p)`/`top(p)` give
    /// the z of each footprint vertex (planar surfaces only).
    ///
    /// Faces are pushed top, bottom, then one side per edge, last, in the
    /// order of the footprint wound counter-clockwise ([`polygon::ccw`]).
    pub fn add_prism(&mut self, footprint: &[DVec2], bottom: impl Fn(DVec2) -> f64, top: impl Fn(DVec2) -> f64) {
        let fp = polygon::ccw(footprint);
        let n = fp.len();
        if n < 3 {
            return;
        }
        let lift = |p: DVec2, z: f64| DVec3::new(p.x, p.y, z);
        let plane_normal = |pts: &[DVec3]| -> DVec3 {
            let sum: DVec3 = (0..pts.len()).map(|i| pts[i].cross(pts[(i + 1) % pts.len()])).sum();
            sum.normalize_or_zero()
        };
        let top_pts: Vec<DVec3> = fp.iter().map(|&p| lift(p, top(p))).collect();
        let mut bottom_pts: Vec<DVec3> = fp.iter().map(|&p| lift(p, bottom(p))).collect();
        let top_n = plane_normal(&top_pts);
        bottom_pts.reverse();
        let bottom_n = plane_normal(&bottom_pts);
        self.faces.push(Face::new(top_pts, top_n));
        self.faces.push(Face::new(bottom_pts, bottom_n));
        for i in 0..n {
            let (a, b) = (fp[i], fp[(i + 1) % n]);
            let out = -(b - a).normalize_or_zero().perp();
            self.faces.push(Face::new(
                vec![lift(a, bottom(a)), lift(b, bottom(b)), lift(b, top(b)), lift(a, top(a))],
                DVec3::new(out.x, out.y, 0.0),
            ));
        }
    }
}

/// Which of `polygons` (the faces of one closed piece, such as a component
/// piece) are facets of curved surfaces (see [`Solid::smooth_curved`]), one
/// flag per polygon; degenerate polygons are not.
pub fn curved_facets(polygons: &[Vec<DVec3>]) -> Vec<bool> {
    let faces: Vec<Face> = polygons
        .iter()
        .map(|p| Face::of_polygon(p.clone()).unwrap_or_else(|| Face::new(Vec::new(), DVec3::ZERO)))
        .collect();
    curved(&faces)
}

/// The facets of curved surfaces among `faces`: those meeting another face
/// along an edge at less than the angle of [`SMOOTH_COS`] without lying in
/// its plane, and every face joined to one of them in one plane (the pieces
/// a boolean cut a facet into). Faces meeting all their neighbors at a
/// sharper angle, as those of a box, are not; so the edges between facets
/// are hidden and the creases sharper than that angle stay.
fn curved(faces: &[Face]) -> Vec<bool> {
    let edges = boundary_edges(faces);
    let mut parent: Vec<usize> = (0..faces.len()).collect();
    let mut bent = vec![false; faces.len()];
    let near = |fi: usize, fj: usize| {
        let (Some(a), Some(b)) = (faces.get(fi), faces.get(fj)) else { return false };
        a.normal.dot(b.normal) > SMOOTH_COS
    };
    // Pieces meeting in T-junctions join too.
    for_each_neighbor(&edges, near, |i, j, _, _| {
        let (Some(&(fi, ..)), Some(&(fj, ..))) = (edges.get(i), edges.get(j)) else { return };
        let (Some(a), Some(b)) = (faces.get(fi), faces.get(fj)) else { return };
        if a.normal.dot(b.normal) >= COPLANAR_COS {
            join(&mut parent, fi, fj);
        } else {
            for f in [fi, fj] {
                if let Some(slot) = bent.get_mut(f) {
                    *slot = true;
                }
            }
        }
    });
    let mut region = vec![false; faces.len()];
    for (f, &b) in bent.iter().enumerate() {
        if b && let Some(slot) = region.get_mut(root(&mut parent, f)) {
            *slot = true;
        }
    }
    (0..faces.len()).map(|f| region.get(root(&mut parent, f)).copied().unwrap_or(false)).collect()
}

/// Let the pieces that are slices of one curved body read as that body.
///
/// Pieces of one element, alike in material and detail levels, that lie
/// against each other on a whole face (the same points, turned against each
/// other) are joined when the body they make bends at one of those seams:
/// faces of the two pieces meet along it at less than the angle of
/// [`SMOOTH_COS`] without lying in one plane, as the slices of an oval lid
/// do. Every piece reached over such seams from a bent one is joined too
/// (the straight stretches of the body). Pieces that only stand side by
/// side (boards, doors, tiles) meet in one plane everywhere and keep the
/// lines between them.
///
/// Joined pieces keep their own faces. Each gets in [`Solid::joined`] the
/// faces of its neighbors at the seams, so no view draws the seams, and the
/// faces that bend there become smooth.
pub fn join_pieces(solids: &mut [Solid]) {
    type Key = (i64, i64, i64);
    if solids.len() < 2 {
        return;
    }
    let key = |p: DVec3| {
        let q = (p * 1e5).round();
        (q.x as i64, q.y as i64, q.z as i64)
    };
    let points = |face: &Face| -> Vec<Key> {
        let mut keys: Vec<Key> = face.verts.iter().map(|p| key(*p)).collect();
        keys.sort_unstable();
        keys.dedup();
        keys
    };
    let shared = |a: &[Key], b: &[Key]| a.iter().filter(|k| b.binary_search(k).is_ok()).count();
    for s in solids.iter_mut() {
        s.joined.clear();
    }

    // Faces at the same points, turned against each other, of two pieces alike.
    let mut at: std::collections::HashMap<Vec<Key>, Vec<(usize, usize)>> = std::collections::HashMap::new();
    for (si, s) in solids.iter().enumerate() {
        for (fi, face) in s.faces.iter().enumerate() {
            let keys = points(face);
            if keys.len() >= 3 && face.verts.iter().all(|p| p.is_finite()) {
                at.entry(keys).or_default().push((si, fi));
            }
        }
    }
    let mut seams: Vec<(usize, usize, usize, usize)> = Vec::new();
    for group in at.values().filter(|g| g.len() >= 2 && g.len() <= MAX_SHARED) {
        for (i, &(sa, fa)) in group.iter().enumerate() {
            for &(sb, fb) in group.iter().skip(i + 1) {
                let (Some(a), Some(b)) = (solids.get(sa), solids.get(sb)) else { continue };
                let (Some(f), Some(g)) = (a.faces.get(fa), b.faces.get(fb)) else { continue };
                if sa != sb && a.alike(b) && f.normal.dot(g.normal) <= -COPLANAR_COS {
                    seams.push(if sa < sb { (sa, fa, sb, fb) } else { (sb, fb, sa, fa) });
                }
            }
        }
    }
    if seams.is_empty() {
        return;
    }
    seams.sort_unstable();

    // The faces of a piece around one of its faces: those sharing an edge with it.
    let around = |solid: &Solid, face: usize| -> Vec<(usize, Vec<Key>)> {
        let Some(seam) = solid.faces.get(face).map(points) else { return Vec::new() };
        let ring = solid.faces.iter().enumerate().filter(|(fi, _)| *fi != face).map(|(fi, f)| (fi, points(f)));
        ring.filter(|(_, keys)| shared(keys, &seam) >= 2).collect()
    };
    // Where the body bends: the faces of the two pieces that meet along a seam at a small angle.
    let mut rings = Vec::with_capacity(seams.len());
    let mut bends: Vec<Vec<(usize, usize)>> = Vec::with_capacity(seams.len());
    let mut parent: Vec<usize> = (0..solids.len()).collect();
    for &(sa, fa, sb, fb) in &seams {
        let (ring_a, ring_b) = match (solids.get(sa), solids.get(sb)) {
            (Some(a), Some(b)) => (around(a, fa), around(b, fb)),
            _ => (Vec::new(), Vec::new()),
        };
        let mut bent = Vec::new();
        for (ha, keys_a) in &ring_a {
            for (hb, keys_b) in &ring_b {
                let normal = |s: usize, h: usize| solids.get(s).and_then(|s| s.faces.get(h)).map(|f| f.normal);
                let (Some(n), Some(m)) = (normal(sa, *ha), normal(sb, *hb)) else { continue };
                let cos = n.dot(m);
                if cos > SMOOTH_COS && cos < COPLANAR_COS && shared(keys_a, keys_b) >= 2 {
                    bent.push((*ha, *hb));
                }
            }
        }
        join(&mut parent, sa, sb);
        rings.push((ring_a, ring_b));
        bends.push(bent);
    }
    let mut curved = vec![false; solids.len()];
    for (&(sa, ..), bent) in seams.iter().zip(&bends) {
        if !bent.is_empty()
            && let Some(slot) = curved.get_mut(root(&mut parent, sa))
        {
            *slot = true;
        }
    }

    // The facets first, so the faces handed to the neighbors are smooth already.
    let mut kept = Vec::new();
    for ((&(sa, fa, sb, fb), bent), (ring_a, ring_b)) in seams.iter().zip(&bends).zip(rings) {
        if curved.get(root(&mut parent, sa)) != Some(&true) {
            continue;
        }
        for &(ha, hb) in bent {
            for (s, h) in [(sa, ha), (sb, hb)] {
                if let Some(face) = solids.get_mut(s).and_then(|s| s.faces.get_mut(h)) {
                    face.smooth = true;
                }
            }
        }
        kept.push((sa, sb, fb, ring_b));
        kept.push((sb, sa, fa, ring_a));
    }
    for (to, from, face, ring) in kept {
        let Some(from) = solids.get(from) else { continue };
        let faces = std::iter::once(face).chain(ring.into_iter().map(|(fi, _)| fi));
        let faces: Vec<Face> = faces.filter_map(|fi| from.faces.get(fi).cloned()).collect();
        if let Some(to) = solids.get_mut(to) {
            to.joined.extend(faces);
        }
    }
}

/// The representative of `i`'s set in a union-find forest whose parents
/// never exceed their children (see [`join`]), halving the path on the way.
fn root(parent: &mut [usize], mut i: usize) -> usize {
    loop {
        let Some(&p) = parent.get(i) else { return i };
        if p >= i {
            return i;
        }
        let up = parent.get(p).copied().unwrap_or(p);
        if let Some(slot) = parent.get_mut(i) {
            *slot = up;
        }
        i = up;
    }
}

/// Join the sets of `a` and `b` (the larger root under the smaller, so
/// every parent index is below its child's and [`root`] ends).
fn join(parent: &mut [usize], a: usize, b: usize) {
    let (ra, rb) = (root(parent, a), root(parent, b));
    if let Some(slot) = parent.get_mut(ra.max(rb)) {
        *slot = ra.min(rb);
    }
}

/// Every boundary edge of `faces`: its face's index and its ends.
fn boundary_edges(faces: &[Face]) -> Vec<(usize, DVec3, DVec3)> {
    faces
        .iter()
        .enumerate()
        .flat_map(|(fi, f)| f.verts.iter().zip(f.verts.iter().cycle().skip(1)).map(move |(a, b)| (fi, *a, *b)))
        .collect()
}

/// The angle of a polygon at each of its points (radians; 0 where a side
/// has no length).
fn corner_angles(verts: &[DVec3]) -> Vec<f64> {
    let n = verts.len();
    verts
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let (Some(prev), Some(next)) = (verts.get((i + n - 1) % n), verts.get((i + 1) % n)) else { return 0.0 };
            let angle = (*prev - *p).angle_between(*next - *p);
            if angle.is_finite() { angle } else { 0.0 }
        })
        .collect()
}

/// Up to this many edges at the same two points are paired directly (more
/// are matched along their lines, see [`for_each_neighbor`]).
const MAX_SHARED: usize = 8;

/// [`for_each_overlap`], quick for meshes whose faces meet edge to edge:
/// an edge with others at the same two points (within 0.01 mm) is paired
/// with those, wholly covered, and only the edges left (the pieces of a
/// boolean, meeting in T-junctions) are matched along their lines.
fn for_each_neighbor(
    edges: &[(usize, DVec3, DVec3)],
    pair: impl Fn(usize, usize) -> bool,
    mut visit: impl FnMut(usize, usize, f64, f64),
) {
    let key = |p: DVec3| {
        let q = (p * 1e5).round();
        (q.x as i64, q.y as i64, q.z as i64)
    };
    let mut at: std::collections::HashMap<_, Vec<usize>> = std::collections::HashMap::new();
    for (i, &(_, a, b)) in edges.iter().enumerate() {
        let (ka, kb) = (key(a), key(b));
        if ka != kb && a.is_finite() && b.is_finite() {
            at.entry(if ka < kb { (ka, kb) } else { (kb, ka) }).or_default().push(i);
        }
    }
    let mut paired = vec![false; edges.len()];
    for group in at.values().filter(|g| g.len() >= 2 && g.len() <= MAX_SHARED) {
        for &i in group {
            for &j in group {
                let (Some(&(fi, ..)), Some(&(fj, ..))) = (edges.get(i), edges.get(j)) else { continue };
                if fi == fj {
                    continue;
                }
                if let Some(slot) = paired.get_mut(i) {
                    *slot = true;
                }
                if pair(fi, fj) {
                    visit(i, j, 0.0, 1.0);
                }
            }
        }
    }
    let rest: Vec<usize> = (0..edges.len()).filter(|i| paired.get(*i) == Some(&false)).collect();
    let loose: Vec<(usize, DVec3, DVec3)> = rest.iter().filter_map(|i| edges.get(*i).copied()).collect();
    for_each_overlap(&loose, pair, |i, j, lo, hi| {
        if let (Some(&i), Some(&j)) = (rest.get(i), rest.get(j)) {
            visit(i, j, lo, hi);
        }
    });
}

/// Calls `visit(i, j, lo, hi)` for every edge `j` of `edges` (each a face
/// index and the edge's ends) lying along edge `i` of another face, the two
/// faces accepted by `pair`: `lo..hi` is the stretch of edge `i` (0 at its
/// start, 1 at its end) that edge `j` covers. Small sets compare every
/// pair of edges; larger ones only edges whose boxes touch (see `EdgeGrid`).
fn for_each_overlap(
    edges: &[(usize, DVec3, DVec3)],
    pair: impl Fn(usize, usize) -> bool,
    mut visit: impl FnMut(usize, usize, f64, f64),
) {
    let grid = (edges.len() > GRID_MIN_EDGES).then(|| EdgeGrid::new(edges));
    let mut seen = vec![usize::MAX; edges.len()];
    let mut candidates = Vec::new();
    for (i, &(fi, a, b)) in edges.iter().enumerate() {
        let ab = b - a;
        let len2 = ab.length_squared();
        if len2 < 1e-12 {
            continue;
        }
        let mut check = |j: usize| {
            let Some(&(fj, c, d)) = edges.get(j) else { return };
            if fj == fi || !pair(fi, fj) {
                return;
            }
            let off = |p: DVec3| (p - a).cross(ab).length_squared() / len2;
            if off(c) > 1e-10 || off(d) > 1e-10 {
                return;
            }
            let (tc, td) = ((c - a).dot(ab) / len2, (d - a).dot(ab) / len2);
            let (lo, hi) = (tc.min(td).max(0.0), tc.max(td).min(1.0));
            if hi - lo > 1e-9 {
                visit(i, j, lo, hi);
            }
        };
        match &grid {
            Some(grid) => {
                grid.near(a, b, i, &mut seen, &mut candidates);
                candidates.iter().for_each(|&j| check(j));
            }
            None => (0..edges.len()).for_each(check),
        }
    }
}

/// Edges of a solid binned in a uniform grid by their bounding boxes, grown
/// by a margin above the collinearity tolerance of `for_each_overlap`:
/// only edges sharing a cell can overlap, so dense meshes are not quadratic.
struct EdgeGrid {
    cell: f64,
    cells: std::collections::HashMap<[i64; 3], Vec<usize>>,
    /// Edges spanning too many cells, compared with every edge.
    big: Vec<usize>,
    count: usize,
}

const GRID_MARGIN: f64 = 1e-4;
const GRID_MAX_CELLS: f64 = 64.0;
/// Solids with up to this many edges compare every pair of faces (cheaper than binning).
const GRID_MIN_EDGES: usize = 1500;

impl EdgeGrid {
    fn new(edges: &[(usize, DVec3, DVec3)]) -> Self {
        let total: f64 = edges.iter().map(|(_, a, b)| a.distance(*b)).filter(|l| l.is_finite()).sum();
        let cell = (total / edges.len().max(1) as f64).max(1e-3);
        let mut grid = Self { cell, cells: Default::default(), big: Vec::new(), count: edges.len() };
        for (i, &(_, a, b)) in edges.iter().enumerate() {
            match grid.range(a, b) {
                Some((lo, hi)) => {
                    for key in cell_keys(lo, hi) {
                        grid.cells.entry(key).or_default().push(i);
                    }
                }
                None => grid.big.push(i),
            }
        }
        grid
    }

    /// Cells covered by the grown box of `a`-`b`; `None` when it spans too
    /// many (or its coordinates are not usable).
    fn range(&self, a: DVec3, b: DVec3) -> Option<([i64; 3], [i64; 3])> {
        let lo = ((a.min(b) - DVec3::splat(GRID_MARGIN)) / self.cell).floor();
        let hi = ((a.max(b) + DVec3::splat(GRID_MARGIN)) / self.cell).floor();
        let usable = |v: DVec3| v.is_finite() && v.abs().max_element() < 1e15;
        let span = hi - lo + DVec3::ONE;
        if !usable(lo) || !usable(hi) || span.x * span.y * span.z > GRID_MAX_CELLS {
            return None;
        }
        Some(([lo.x as i64, lo.y as i64, lo.z as i64], [hi.x as i64, hi.y as i64, hi.z as i64]))
    }

    /// The edges that may overlap edge `query` (`a`-`b`), into `out`;
    /// `seen` (one slot per edge) marks those already listed for it.
    fn near(&self, a: DVec3, b: DVec3, query: usize, seen: &mut [usize], out: &mut Vec<usize>) {
        out.clear();
        let Some((lo, hi)) = self.range(a, b) else {
            out.extend(0..self.count);
            return;
        };
        let cells = cell_keys(lo, hi).filter_map(|key| self.cells.get(&key));
        for &i in cells.flatten().chain(&self.big) {
            if let Some(mark) = seen.get_mut(i)
                && *mark != query
            {
                *mark = query;
                out.push(i);
            }
        }
    }
}

fn cell_keys(lo: [i64; 3], hi: [i64; 3]) -> impl Iterator<Item = [i64; 3]> {
    (lo[0]..=hi[0]).flat_map(move |x| (lo[1]..=hi[1]).flat_map(move |y| (lo[2]..=hi[2]).map(move |z| [x, y, z])))
}

/// Distance along `ray` at which it enters the box `lo`-`hi` (grown by a
/// small margin; 0 from inside), if it meets the box ahead.
pub(crate) fn ray_box(ray: Ray, lo: DVec3, hi: DVec3) -> Option<f64> {
    const MARGIN: f64 = 1e-6;
    let (lo, hi) = (lo - DVec3::splat(MARGIN), hi + DVec3::splat(MARGIN));
    let (mut near, mut far) = (0.0_f64, f64::INFINITY);
    for axis in 0..3 {
        let (o, d) = (ray.origin[axis], ray.dir[axis]);
        if d == 0.0 {
            if o < lo[axis] || o > hi[axis] {
                return None;
            }
            continue;
        }
        let (a, b) = ((lo[axis] - o) / d, (hi[axis] - o) / d);
        near = near.max(a.min(b));
        far = far.min(a.max(b));
    }
    (near <= far).then_some(near)
}

/// Möller–Trumbore, two-sided.
fn ray_triangle(ray: Ray, a: DVec3, b: DVec3, c: DVec3) -> Option<f64> {
    let e1 = b - a;
    let e2 = c - a;
    let p = ray.dir.cross(e2);
    let det = e1.dot(p);
    if det.abs() < 1e-12 {
        return None;
    }
    let inv = 1.0 / det;
    let s = ray.origin - a;
    let u = s.dot(p) * inv;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = s.cross(e1);
    let v = ray.dir.dot(q) * inv;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = e2.dot(q) * inv;
    (t > 1e-9).then_some(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A concave quad (a dart) splits along the diagonal inside it: the
    /// triangles cover its area exactly, whichever corner is the reflex one.
    #[test]
    fn concave_quads_split_inside() {
        let dart = [DVec2::new(0.0, 0.0), DVec2::new(2.0, 1.0), DVec2::new(0.0, 2.0), DVec2::new(0.5, 1.0)];
        for shift in 0..4 {
            let verts: Vec<DVec3> = (0..4).map(|k| dart[(k + shift) % 4].extend(0.0)).collect();
            let face = Face::new(verts.clone(), DVec3::Z);
            let area: f64 = face
                .triangulate()
                .iter()
                .map(|t| (verts[t[1]] - verts[t[0]]).cross(verts[t[2]] - verts[t[0]]).z.abs() / 2.0)
                .sum();
            assert!((area - face.area()).abs() < 1e-12, "shift {shift}: {area} vs {}", face.area());
        }
    }

    /// Regression (issue #51): a prism over a footprint with no area (its
    /// points on one line) has caps without a normal; triangulating them for
    /// the 3D view or a pick in it panicked inside earcutr.
    #[test]
    fn degenerate_faces_triangulate_without_panicking() {
        let line: Vec<DVec2> = (0..5).map(|i| DVec2::new(f64::from(i), 0.0)).collect();
        let mut s = Solid::new(ElementId(1), Category::Floor, [0; 3]);
        s.add_prism(&line, |_| 0.0, |_| 1.0);
        for f in &s.faces {
            f.triangulate();
        }
        let ray = Ray { origin: DVec3::new(2.0, 0.0, 5.0), dir: -DVec3::Z };
        s.raycast(ray);
        // A face built with no normal triangulates in the plane of its points.
        let pentagon: Vec<DVec3> = (0..5)
            .map(|i| f64::from(i) * std::f64::consts::TAU / 5.0)
            .map(|a| DVec3::new(a.cos(), a.sin(), 1.0))
            .collect();
        assert_eq!(Face::new(pentagon, DVec3::ZERO).triangulate().len(), 3);
        assert!(Face::new(vec![DVec3::splat(f64::NAN); 6], DVec3::Z).triangulate().is_empty());
    }

    #[test]
    fn box_volume_and_raycast() {
        let mut s = Solid::new(ElementId(1), Category::Wall, [0; 3]);
        s.add_box(DVec2::ZERO, DVec2::X, DVec2::Y, 0.0, 2.0, 0.0, 3.0, 0.0, 4.0);
        assert!((s.volume() - 24.0).abs() < 1e-9);
        let hit = s.raycast(Ray { origin: DVec3::new(1.0, -5.0, 1.0), dir: DVec3::Y }).unwrap();
        assert!((hit - 5.0).abs() < 1e-9);
    }

    #[test]
    fn split_faces_hide_shared_edges() {
        let mut s = Solid::new(ElementId(1), Category::Wall, [0; 3]);
        // Two boxes side by side form one 2x1x1 block: its outline has 12 edges.
        s.add_box(DVec2::ZERO, DVec2::X, DVec2::Y, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0);
        s.add_box(DVec2::ZERO, DVec2::X, DVec2::Y, 1.0, 2.0, 0.0, 1.0, 0.0, 1.0);
        // Drop the two internal faces at x = 1 (+u of the first box, -u of the second).
        s.faces.remove(10);
        s.faces.remove(5);
        let len: f64 = s.feature_edges().iter().flatten().map(|(a, b)| a.distance(*b)).sum();
        // Each visible edge is reported by both adjacent faces: 2 * (4*2 + 8*1).
        assert!((len - 2.0 * (8.0 + 8.0)).abs() < 1e-9, "{len}");
    }

    /// [`Solid::feature_edges`] comparing every pair of edges.
    fn feature_edges_brute(s: &Solid) -> Vec<Vec<(DVec3, DVec3)>> {
        let edges = |f: &Face| {
            let n = f.verts.len();
            (0..n).map(move |i| (f.verts[i], f.verts[(i + 1) % n])).collect::<Vec<_>>()
        };
        let all: Vec<Vec<(DVec3, DVec3)>> = s.faces.iter().map(edges).collect();
        let mut out = Vec::new();
        for (fi, face) in s.faces.iter().enumerate() {
            let mut face_out = Vec::new();
            for &(a, b) in &all[fi] {
                let ab = b - a;
                let len2 = ab.length_squared();
                if len2 < 1e-12 {
                    continue;
                }
                let mut covered: Vec<(f64, f64)> = Vec::new();
                for (fj, other) in s.faces.iter().enumerate() {
                    let cos = other.normal.dot(face.normal);
                    let smooth = face.smooth && other.smooth && cos > SMOOTH_COS;
                    if fj == fi || (cos < 0.9999 && !smooth) {
                        continue;
                    }
                    for &(c, d) in &all[fj] {
                        let off = |p: DVec3| (p - a).cross(ab).length_squared() / len2;
                        if off(c) > 1e-10 || off(d) > 1e-10 {
                            continue;
                        }
                        let (tc, td) = ((c - a).dot(ab) / len2, (d - a).dot(ab) / len2);
                        let (lo, hi) = (tc.min(td).max(0.0), tc.max(td).min(1.0));
                        if hi - lo > 1e-9 {
                            covered.push((lo, hi));
                        }
                    }
                }
                covered.sort_by(|x, y| x.0.total_cmp(&y.0));
                let mut t = 0.0;
                for (lo, hi) in covered {
                    if lo > t + 1e-9 {
                        face_out.push((a + ab * t, a + ab * lo));
                    }
                    t = t.max(hi);
                }
                if t < 1.0 - 1e-9 {
                    face_out.push((a + ab * t, b));
                }
            }
            out.push(face_out);
        }
        out
    }

    #[test]
    fn feature_edges_match_the_pairwise_comparison() {
        // A faceted sphere (dense, small edges) on a long split slab (edges
        // too long for the grid, and T-junctions along them).
        let mut s = Solid::new(ElementId(1), Category::Wall, [0; 3]);
        let n = 24;
        let p = |i: usize, j: usize| {
            let (th, ph) = (i as f64 / n as f64 * std::f64::consts::TAU, j as f64 / n as f64 * std::f64::consts::PI);
            DVec3::new(ph.sin() * th.cos(), ph.sin() * th.sin(), 1.0 + ph.cos())
        };
        let mut polys = vec![];
        for i in 0..n {
            for j in 0..n {
                polys.push(vec![p(i, j), p(i + 1, j), p(i + 1, j + 1)]);
                polys.push(vec![p(i, j), p(i + 1, j + 1), p(i, j + 1)]);
            }
        }
        s.add_polygons(polys);
        s.add_box(DVec2::new(-40.0, -1.0), DVec2::X, DVec2::Y, 0.0, 30.0, 0.0, 2.0, -0.2, 0.0);
        s.add_box(DVec2::new(-40.0, -1.0), DVec2::X, DVec2::Y, 30.0, 80.0, 0.0, 2.0, -0.2, 0.0);
        for f in s.faces.iter_mut().skip(n * n) {
            f.smooth = true;
        }
        assert_eq!(s.feature_edges(), feature_edges_brute(&s));
    }

    /// Issue #22: the facets of a curved surface shade as one surface (the
    /// normal at a shared point is their average), while flat faces and
    /// facets meeting at a sharp angle keep their own normals.
    #[test]
    fn smooth_facets_share_their_vertex_normals() {
        // Two vertical facets of a cylinder of radius 1, 10° apart, and a flat top.
        let at = |deg: f64, z: f64| DVec3::new(deg.to_radians().cos(), deg.to_radians().sin(), z);
        let facet = |a: f64, b: f64| {
            let mid = ((a + b) / 2.0).to_radians();
            let mut f =
                Face::new(vec![at(a, 0.0), at(b, 0.0), at(b, 1.0), at(a, 1.0)], DVec3::new(mid.cos(), mid.sin(), 0.0));
            f.smooth = true;
            f
        };
        let mut s = Solid::new(ElementId(1), Category::Wall, [0; 3]);
        s.faces = vec![
            facet(0.0, 10.0),
            facet(10.0, 20.0),
            Face::new(vec![at(0.0, 1.0), at(10.0, 1.0), at(20.0, 1.0)], DVec3::Z),
        ];
        let normals = s.vertex_normals();
        // On the shared edge (at 10°) both facets get the radial direction there.
        let radial = DVec3::new(10f64.to_radians().cos(), 10f64.to_radians().sin(), 0.0);
        assert!(normals[0][1].distance(radial) < 1e-9, "{:?}", normals[0][1]);
        assert!(normals[1][0].distance(radial) < 1e-9, "{:?}", normals[1][0]);
        // At the outer edges each keeps its own facet's normal; the top stays flat.
        assert!(normals[0][0].distance(s.faces[0].normal) < 1e-9);
        assert!(normals[2].iter().all(|n| *n == DVec3::Z));
        // A sharp corner between smooth facets is not smoothed across.
        s.faces[1] = facet(10.0, 80.0);
        let normals = s.vertex_normals();
        assert!(normals[0][1].distance(s.faces[0].normal) < 1e-9);
    }

    /// Prisms over regular polygons: sides 15° and 30° apart read as one
    /// curved surface, 45° apart (an octagon) and 90° (a box) stay creases.
    #[test]
    fn curved_facets_stop_at_creases() {
        let smooth_sides = |n: usize| {
            let ring: Vec<DVec2> =
                (0..n).map(|i| DVec2::from_angle(i as f64 / n as f64 * std::f64::consts::TAU)).collect();
            let mut s = Solid::new(ElementId(1), Category::Component, [0; 3]);
            s.add_box(DVec2::new(5.0, 0.0), DVec2::X, DVec2::Y, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0);
            let start = s.faces.len();
            s.add_prism(&ring, |_| 0.0, |_| 1.0);
            s.smooth_curved(start);
            let (box_faces, prism) = s.faces.split_at(start);
            assert!(box_faces.iter().all(|f| !f.smooth), "faces before `start` are left alone");
            assert!(prism.iter().filter(|f| f.normal.z.abs() > 0.5).all(|f| !f.smooth), "caps");
            prism.iter().filter(|f| f.normal.z.abs() < 0.5).all(|f| f.smooth)
        };
        assert!(smooth_sides(24) && smooth_sides(12));
        assert!(!smooth_sides(8) && !smooth_sides(4));
        // Pieces of one plane join the surface their pieces touch.
        let p = |x: f64, y: f64, z: f64| DVec3::new(x, y, z);
        let flags = curved_facets(&[
            vec![p(0.0, 0.0, 0.0), p(1.0, 0.0, 0.0), p(1.0, 0.0, 1.0)],
            vec![p(0.0, 0.0, 0.0), p(1.0, 0.0, 1.0), p(0.0, 0.0, 1.0)],
            // Turned 10° about the edge x = 1 from the first.
            vec![p(1.0, 0.0, 0.0), p(1.0 + 10f64.to_radians().cos(), 10f64.to_radians().sin(), 0.0), p(1.0, 0.0, 1.0)],
            vec![p(5.0, 5.0, 5.0)],
        ]);
        assert_eq!(flags, [true, true, true, false]);
    }

    /// A slice of a plate 0.1 thick between `x0` and `x1`, as wide as twice `h0` at `x0` and twice `h1` at `x1`.
    fn slice(x0: f64, x1: f64, h0: f64, h1: f64) -> Solid {
        let mut s = Solid::new(ElementId(1), Category::Component, [0; 3]);
        let outline = [DVec2::new(x0, -h0), DVec2::new(x1, -h1), DVec2::new(x1, h1), DVec2::new(x0, h0)];
        s.add_prism(&outline, |_| 0.0, |_| 0.1);
        s
    }

    /// The length of the outline of `s` drawn in the plane `x`.
    fn lines_at(s: &Solid, x: f64) -> f64 {
        let on = |p: &DVec3| (p.x - x).abs() < 1e-9;
        s.feature_edges().iter().flatten().filter(|(a, b)| on(a) && on(b)).map(|(a, b)| a.distance(*b)).sum()
    }

    #[test]
    fn slices_of_a_curved_body_read_as_one() {
        // A plate that widens in two bends, then runs straight in two slices.
        let mut pieces = [
            slice(0.0, 1.0, 1.0, 2.0),
            slice(1.0, 2.0, 2.0, 2.5),
            slice(2.0, 3.0, 2.5, 2.5),
            slice(3.0, 4.0, 2.5, 2.5),
        ];
        assert!(lines_at(&pieces[0], 1.0) > 8.0, "apart, a slice draws the face it ends on");
        join_pieces(&mut pieces);
        for (i, s) in pieces.iter().enumerate() {
            for x in [1.0, 2.0, 3.0] {
                assert_eq!(lines_at(s, x), 0.0, "slice {i} draws no seam at x = {x}");
            }
            assert_eq!(s.faces.len(), 6, "the slices keep their faces");
        }
        // The ends of the body stay: each edge of the face is the boundary of two faces.
        assert!((lines_at(&pieces[0], 0.0) - 2.0 * (2.0 * 2.0 + 2.0 * 0.1)).abs() < 1e-9);
        assert!((lines_at(&pieces[3], 4.0) - 2.0 * (2.0 * 5.0 + 2.0 * 0.1)).abs() < 1e-9);
        // The faces between the slices are inside the body.
        let inner: Vec<usize> = pieces.iter().map(|s| s.inner_faces().iter().filter(|i| **i).count()).collect();
        assert_eq!(inner, [1, 2, 2, 1]);
        // The sides that bend are facets of one surface: smooth, shaded together, a silhouette between them.
        let side = |f: &&Face| f.normal.y.abs() > 0.1;
        assert!(pieces[0].faces.iter().filter(side).all(|f| f.smooth));
        assert!(pieces[0].faces.iter().filter(|f| !side(f)).all(|f| !f.smooth), "the flat faces are not");
        let seams: f64 = pieces[0].smooth_edges().iter().flatten().map(|(a, b, _)| a.distance(*b)).sum();
        assert!((seams - 2.0 * 0.1).abs() < 1e-9, "one edge on each side, at x = 1: {seams}");
        let blended = pieces[0].faces.iter().zip(pieces[0].vertex_normals()).filter(|(f, _)| side(f)).all(|(f, ns)| {
            f.verts.iter().zip(ns).all(|(p, n)| ((p.x - 1.0).abs() < 1e-9) == (n.dot(f.normal) < 1.0 - 1e-6))
        });
        assert!(blended, "normals turn towards the next slice where they meet, and nowhere else");
    }

    #[test]
    fn pieces_side_by_side_keep_their_lines() {
        // Two boards in one plane on every side: nothing bends.
        let mut boards = [slice(0.0, 1.0, 1.0, 1.0), slice(1.0, 2.0, 1.0, 1.0)];
        let before = lines_at(&boards[0], 1.0);
        join_pieces(&mut boards);
        assert!(boards.iter().all(|s| s.joined.is_empty() && s.faces.iter().all(|f| !f.smooth)));
        assert_eq!(lines_at(&boards[0], 1.0), before);
        // Slices of two materials are two bodies.
        let mut two = [slice(0.0, 1.0, 1.0, 2.0), slice(1.0, 2.0, 2.0, 2.5)];
        two[1].color = [9; 3];
        join_pieces(&mut two);
        assert!(two.iter().all(|s| s.joined.is_empty()));
        // Neither are slices that do not end on the same face.
        let mut stepped = [slice(0.0, 1.0, 1.0, 2.0), slice(1.0, 2.0, 2.2, 2.5)];
        join_pieces(&mut stepped);
        assert!(stepped.iter().all(|s| s.joined.is_empty()));
    }

    #[test]
    fn prism_volume() {
        let mut s = Solid::new(ElementId(1), Category::Floor, [0; 3]);
        let sq = [DVec2::new(0.0, 0.0), DVec2::new(0.0, 2.0), DVec2::new(2.0, 2.0), DVec2::new(2.0, 0.0)];
        s.add_prism(&sq, |_| -0.5, |_| 0.0);
        assert!((s.volume() - 2.0).abs() < 1e-9);
    }
}
