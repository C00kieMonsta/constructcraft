//! The shadows a view's sun casts in projected views and plans, as exact
//! polygons (see [`crate::projected::render`]).
//!
//! Every opaque face of the model that is not edge-on to the sun casts; a
//! face turned to the sun catches. The shadow on a face is the union of the
//! casters standing in front of its plane, slid along the sun's rays onto
//! that plane and kept within the face ([`cc_geom::region::covered`]); the
//! ground catches them the same way on a level plane. A closed convex
//! solid (a box, a round column) casts as one polygon, the hull of its
//! points.
//!
//! Which faces catch them is the view's ([`Catch`]), as drawings are read:
//!
//! - Elevations and 3D views: everything casts on everything shown.
//! - Floor plans read the site. The whole model the view shows casts, what
//!   stands above the cut plane too (upper storeys, roofs, eaves, the beams
//!   of a pergola), on every face seen from above and on the ground, as
//!   the sun does: under an eave, a porch or a pergola too. Only what is
//!   indoors catches nothing ([`Indoors`]): the part of a face, or of the
//!   ground, within a space indoors and under a floor, roof or ceiling, is
//!   taken out of its shadows ([`region::covered_except`]), so the rooms
//!   keep their plain shading. Ceiling plans draw no shadows.
//! - Sections: what the section cuts away (the model in front of its line)
//!   casts nothing, the rest casts as it is drawn, closed by its cut; and
//!   the faces indoors catch nothing, by the same rule, so the rooms cut
//!   show their plain shading while the fronts, roofs and ground seen
//!   beyond keep their shadows, an eave's on the wall under it among them.
//!
//! Bounded: a grid over the casters seen from the sun finds those over a
//! face, and the faces that cast, the casters looked at, the points of one
//! union and the points of a whole view are limited; past a limit the view
//! gets no shadows at all.

use crate::projected::Frame;
use cc_core::{Category, DetailLevel, ElementId, Project, ShadingSettings, VisualStyle};
use cc_geom::{ModelGeometry, RoomArrangement, Solid, clip_polygon, face_plane_segments, polygon, region};
use glam::{DVec2, DVec3};
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::ops::Range;

/// A face catches the sun's shadows when the cosine of its angle to the
/// sun is at least this (edge-on, shadows run without end).
const MIN_FACING: f64 = 0.01;
/// A face casts when the cosine of its angle to the sun is at least this
/// (edge-on, it covers nothing).
const MIN_CASTING: f64 = 1e-3;
/// What stands less than this in front of a face, meters, casts nothing on
/// it (faces touching it, its own solid's faces around it).
const LIFT: f64 = 1e-4;
/// A face less opaque than this casts nothing, as in the 3D view.
const OPAQUE: f64 = 0.999;
/// Faces smaller than this, square meters, catch nothing.
const MIN_AREA: f64 = 1e-8;
/// Most faces that cast in one view.
pub const MAX_CASTERS: usize = 200_000;
/// Most points of the casters over one face (or the ground).
pub const MAX_UNION_POINTS: usize = 60_000;
/// Most points of the casters over all the faces of one view.
pub const MAX_POINTS: usize = 600_000;
/// Most casters looked at over all the faces of one view.
pub const MAX_LOOKS: usize = 15_000_000;
/// Most cells along a side of the grid over the casters.
const GRID_SIDE: usize = 128;
/// A caster over more cells than this is kept apart, compared with every face.
const LARGE: usize = 256;
/// A solid of more faces than this is not tested for being convex.
const MAX_CONVEX_FACES: usize = 96;
/// Most comparisons made over one face to leave out polygons inside others.
const MAX_PRUNING: usize = 4_000_000;
/// More polygons than twice this over one face (or the ground) are united
/// this many at a time, then their unions together.
const UNION_GROUP: usize = 16;
/// A storey's walls are those standing this high above its level, meters
/// (or halfway up a lower storey).
const STOREY_PROBE: f64 = 1.0;
/// A space closed by walls is indoors when slabs cover more of it than this.
const INDOORS_COVER: f64 = 0.75;
/// A face is indoors when the point this far in front of it is, meters.
pub(crate) const FACE_PROBE: f64 = 0.05;
/// A face seen from above, and the ground, are in the storey of the point
/// this far above them, meters (a floor may lie a little under its level).
const PLAN_PROBE: f64 = 0.5;
/// Most storeys looked at for what is indoors.
const MAX_STOREYS: usize = 256;
/// Most sides of the outlines of a storey's walls (their arrangement grows with the square).
const MAX_WALL_SIDES: usize = 4_000;

/// The ground's shadows: a black veil this opaque (the 3D view's, `GROUND_SHADOW` in `cc_render`'s `sun.wgsl`).
const GROUND_SHADOW: f64 = 0.35;
/// What is left of an unlit face's color in a shadow, as displayed (the
/// consistent colors style; the 3D view's `UNLIT_SHADOW`, of linear light).
const UNLIT_SHADOW: f64 = 0.85;
/// A sun lower than this casts shadows on the ground no longer than it
/// would from this altitude, degrees (as the 3D view's ground).
const MIN_GROUND_ALTITUDE_DEG: f64 = 5.0;
/// Farthest the ground's shadows reach past the model, meters.
const MAX_GROUND_REACH: f64 = 5000.0;
/// The light a face of the textures style keeps turned away from the sun
/// at the default shading (a face square to it is fully lit).
pub(crate) const TEXTURES_AMBIENT: f64 = 0.62;

/// The elements a view's graphics keep out of its shadows.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Skip {
    /// Hidden in the view (by category, filter, element, phase, design
    /// option or temporarily): they neither cast nor catch shadows.
    pub hidden: BTreeSet<ElementId>,
    /// Made see-through by the view's graphics: they cast none.
    pub clear: BTreeSet<ElementId>,
}

/// The shadows the sun of a view casts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shadows<'a> {
    /// The height of the level plane that catches them around the model
    /// (see [`cc_core::Project::sun_ground_height`]); `None`: no ground.
    pub ground: Option<f64>,
    /// The elements left out of them.
    pub skip: Option<&'a Skip>,
    /// The faces that catch them.
    pub catch: Catch<'a>,
}

/// Which faces of a view catch its sun's shadows, and what casts them.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Catch<'a> {
    /// Every face shown; what the view cuts away casts nothing
    /// (elevations, 3D views).
    #[default]
    All,
    /// A floor plan's reading of the site: the whole model shown casts,
    /// above the cut plane too, on every face and on the ground; what is
    /// indoors of them catches nothing.
    Site(&'a Indoors),
    /// A section's: what it cuts away casts nothing, and the faces indoors
    /// catch nothing.
    Outdoors(&'a Indoors),
}

/// No space indoors: everything catches the sun's shadows.
pub static OPEN_AIR: Indoors = Indoors { storeys: Vec::new(), failed: false };

/// Whether solids of `category` cover what lies under them from the sky:
/// floors, roofs and ceilings (foundation slabs lie under everything, and
/// beams, such as a pergola's, leave the sky open).
fn covers(category: Category) -> bool {
    matches!(category, Category::Floor | Category::Roof | Category::Ceiling)
}

/// The spaces of a model that are indoors, storey by storey, for the
/// shadows of floor plans and sections.
///
/// A storey runs from a level up to the next one (the last one without
/// end), and its walls are those standing [`STOREY_PROBE`] above its level
/// (halfway up a lower storey). The spaces their outlines close, the walls'
/// own bodies among them ([`RoomArrangement::spaces`]: rooms, whether or
/// not a room was placed in them), are indoors when floors, roofs and
/// ceilings above cover more than [`INDOORS_COVER`] of them; a storey keeps
/// them united, as the outlines of what is indoors on it. So a room is
/// indoors; a courtyard, a walled garden with the eaves over its edge and a
/// terrace behind a parapet are not, nor is a porch or a garage that walls
/// do not close.
///
/// A point is indoors when it is in one of the spaces of the storey at its
/// height ([`Indoors::holds`]) and a floor, roof or ceiling of another
/// element lies above it ([`Scene::above`], [`Scene::covers`]): the top of
/// a roof is not, whatever the storey it falls in. A section takes a face
/// whole, by the point [`FACE_PROBE`] in front of it, so the face of an
/// outer wall under an eave, which looks onto the open, keeps the eave's
/// shadow; a floor plan takes the part of each face seen from above that
/// is indoors, in the storey [`PLAN_PROBE`] above it. Spaces closed by
/// something other than walls (columns, components) are not found: their
/// faces catch shadows.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Indoors {
    /// The storeys, from the lowest up.
    pub storeys: Vec<Storey>,
    /// A limit was passed finding them: the view draws no shadows.
    pub failed: bool,
}

/// A storey and its spaces indoors.
#[derive(Debug, Clone, PartialEq)]
pub struct Storey {
    /// From this height up to this one, meters (the second not included).
    pub z: (f64, f64),
    /// The outlines of what is indoors on it, in plan: its spaces indoors
    /// united (rings that do not overlap, see [`region::covered`]).
    pub spaces: Vec<Vec<DVec2>>,
}

impl Indoors {
    /// The spaces indoors of `geom` (the geometry a view shows, at its
    /// `detail` level, less what `skip` hides: hidden walls close nothing,
    /// hidden and see-through slabs cover nothing).
    pub fn of(project: &Project, geom: &ModelGeometry, skip: Option<&Skip>, detail: DetailLevel) -> Self {
        let failed = Self { storeys: Vec::new(), failed: true };
        let Some(mut cover) = Scene::covers(geom, skip, &|s| s.detail.shows(detail), None) else { return failed };
        let mut floors: Vec<f64> = project.levels.iter().map(|l| l.elevation).filter(|z| z.is_finite()).collect();
        floors.sort_by(f64::total_cmp);
        floors.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
        if floors.len() > MAX_STOREYS {
            return failed;
        }
        let hidden = |id: &ElementId| skip.is_some_and(|k| k.hidden.contains(id));
        let mut storeys: Vec<Storey> = Vec::new();
        // The walls of the storey before, to find the spaces of the same walls once.
        let mut before: Vec<ElementId> = Vec::new();
        for (i, &z0) in floors.iter().enumerate() {
            let z1 = floors.get(i + 1).copied().unwrap_or(f64::INFINITY);
            let at = z0 + ((z1 - z0) * 0.5).min(STOREY_PROBE);
            let standing = geom.walls.iter().filter(|(id, w)| !hidden(id) && w.base_z <= at && w.top_z >= at);
            let (ids, walls): (Vec<ElementId>, Vec<Vec<DVec2>>) = standing.map(|(id, w)| (*id, w.footprint())).unzip();
            if let Some(last) = storeys.last().filter(|_| !ids.is_empty() && ids == before) {
                let spaces = last.spaces.clone();
                storeys.push(Storey { z: (z0, z1), spaces });
                continue;
            }
            let mut spaces = Vec::new();
            let sound = walls.iter().all(|w| w.iter().all(|p| p.is_finite()));
            if sound && walls.iter().map(Vec::len).sum::<usize>() <= MAX_WALL_SIDES {
                for space in RoomArrangement::new(walls).spaces() {
                    let area = polygon::area(&space);
                    let outline: Vec<DVec3> = space.iter().map(|p| p.extend(at)).collect();
                    let under = cover.on_face(&outline, DVec3::Z, (usize::MAX, usize::MAX), None);
                    let covered: f64 = under.iter().map(|r| polygon::area(&flat(r))).sum();
                    if area > MIN_AREA && covered > INDOORS_COVER * area {
                        spaces.push(space);
                    }
                }
            }
            if cover.failed() {
                return failed;
            }
            // United: the rooms and the walls between them as one outline.
            if let Some(united) = region::covered(&spaces, None, MAX_UNION_POINTS) {
                spaces = united;
            }
            before = ids;
            storeys.push(Storey { z: (z0, z1), spaces });
        }
        Self { storeys, failed: false }
    }

    /// The spaces indoors of the storey at height `z`.
    fn spaces_at(&self, z: f64) -> impl Iterator<Item = &Vec<DVec2>> {
        self.storeys.iter().filter(move |s| z >= s.z.0 && z < s.z.1).flat_map(|s| &s.spaces)
    }

    /// Whether `p` is in a space indoors of the storey at its height.
    pub fn holds(&self, p: DVec3) -> bool {
        self.spaces_at(p.z).any(|space| polygon::contains(space, p.truncate()))
    }

    /// Whether no space is indoors.
    pub fn is_empty(&self) -> bool {
        self.storeys.iter().all(|s| s.spaces.is_empty())
    }
}

/// Points seen in plan.
fn flat(points: &[DVec3]) -> Vec<DVec2> {
    points.iter().map(|p| p.truncate()).collect()
}

/// A point of the face `verts` (with unit `normal`): the mean of its
/// corners, or when that lies outside it (a face that is not convex), the
/// middle of its largest triangle.
pub(crate) fn point_within(verts: &[DVec3], normal: DVec3) -> Option<DVec3> {
    let first = *verts.first()?;
    let mean = verts.iter().sum::<DVec3>() / verts.len() as f64;
    let (u, v) = basis(normal);
    let at = |p: DVec3| DVec2::new((p - first).dot(u), (p - first).dot(v));
    let outline: Vec<DVec2> = verts.iter().map(|p| at(*p)).collect();
    if !mean.is_finite() || verts.len() < 3 {
        return None;
    }
    if polygon::contains(&outline, at(mean)) {
        return Some(mean);
    }
    let size = |t: &[usize; 3]| (outline[t[1]] - outline[t[0]]).perp_dot(outline[t[2]] - outline[t[0]]).abs();
    let largest = polygon::triangulate(&outline).into_iter().max_by(|a, b| size(a).total_cmp(&size(b)))?;
    Some(largest.iter().filter_map(|i| verts.get(*i)).sum::<DVec3>() / 3.0)
}

/// The light of a face of the textures style at `facing` (the cosine of
/// its angle to the light): [`TEXTURES_AMBIENT`] turned away and full
/// square to it at the default shading; a view's ambient light moves the
/// first as it moves the shaded style's grey (up to full light at 1).
pub(crate) fn textures_light(shading: &ShadingSettings, facing: f64) -> f64 {
    let ambient = textures_ambient(shading);
    let cos = if facing.is_nan() { 0.0 } else { facing.clamp(0.0, 1.0) };
    ambient + (1.0 - ambient) * cos
}

/// The textures style's light of a face turned away (see [`textures_light`]).
fn textures_ambient(shading: &ShadingSettings) -> f64 {
    let a = shading.clamped().ambient;
    let k = (1.0 - TEXTURES_AMBIENT) / (1.0 - VisualStyle::SHADED_AMBIENT);
    (TEXTURES_AMBIENT + (a - VisualStyle::SHADED_AMBIENT) * k).clamp(0.0, 1.0)
}

/// The opacity (0 to 255) of the black veil that a cast shadow lays on a
/// face of `style` at `facing` (the cosine of its angle to the sun), so
/// that 2D views match the 3D view: in the shaded style the face shows
/// the shadow's grey ([`ShadingSettings::shadow_grey`]) under it, in the
/// textures style it loses the sun's light and darkens as much as the
/// shaded style's shadow does from its ambient light, in consistent colors
/// it darkens as the 3D view's unlit faces do. The shadow fades out where
/// the face turns edge-on to the sun, as in the 3D view. No shadows in the
/// other styles.
pub fn veil(style: VisualStyle, facing: f64, shading: &ShadingSettings) -> u8 {
    let cos = if facing.is_nan() { 0.0 } else { facing.clamp(0.0, 1.0) };
    let (lit, dark) = match style {
        VisualStyle::Shaded => {
            let lit = shading.grey(cos);
            (lit, shading.shadow_grey(lit))
        }
        VisualStyle::Textures => {
            let lit = textures_light(shading, cos);
            // The shaded style's shadow grey over its default ambient grey,
            // as a share of the textures style's default ambient light.
            let dark = (1.0 - shading.clamped().shadow) * TEXTURES_AMBIENT / VisualStyle::SHADED_AMBIENT;
            (lit, dark.min(lit))
        }
        VisualStyle::ConsistentColors => (1.0, 1.0 - shading.darkening(1.0 - UNLIT_SHADOW)),
        VisualStyle::Wireframe | VisualStyle::HiddenLine => return 0,
    };
    if lit <= 0.0 {
        return 0;
    }
    let t = (cos / 0.05).clamp(0.0, 1.0);
    let fade = t * t * (3.0 - 2.0 * t);
    ((1.0 - dark / lit) * fade * 255.0).round().clamp(0.0, 255.0) as u8
}

/// The opacity (0 to 255) of the black veil of a shadow on the ground:
/// [`GROUND_SHADOW`] at the default shading, none to black with the view's
/// shadow intensity.
pub fn ground_veil(shading: &ShadingSettings) -> u8 {
    (shading.darkening(GROUND_SHADOW) * 255.0).round().clamp(0.0, 255.0) as u8
}

/// A face that casts: its outline (what the view's cut leaves of it).
struct Caster<'a> {
    verts: Cow<'a, [DVec3]>,
    /// Its solid and face in the geometry (a cut's cap has no face).
    id: (usize, usize),
}

/// What casts as one: a face, or a closed convex solid, whose shadow is
/// the hull of its points.
struct Unit {
    /// The element it belongs to.
    element: ElementId,
    /// Its faces among the casters (one, or a convex solid's).
    faces: Range<usize>,
    /// A convex solid's points (empty for a face).
    points: Vec<DVec3>,
    /// Its box seen from the sun, and its point nearest the sun (along it).
    lo: DVec2,
    hi: DVec2,
    top: f64,
}

/// What casts shadows in a view, seen from its sun.
pub(crate) struct Scene<'a> {
    /// Unit vector towards the sun.
    sun: DVec3,
    /// Two unit vectors square to it and to each other.
    axes: (DVec3, DVec3),
    casters: Vec<Caster<'a>>,
    units: Vec<Unit>,
    /// The grid over the units' boxes: origin, cell size, cells along a
    /// side, the units over each cell and those over many.
    origin: DVec2,
    cell: DVec2,
    side: usize,
    cells: Vec<Vec<u32>>,
    large: Vec<u32>,
    /// Units met by the current lookup carry its number.
    seen: Vec<u32>,
    lookup: u32,
    /// Points given to unions, and casters looked at over faces, so far.
    points: usize,
    looks: usize,
    /// A limit was passed: no shadows.
    failed: bool,
}

/// Two unit vectors square to unit `n` and to each other, `n` their cross product.
fn basis(n: DVec3) -> (DVec3, DVec3) {
    let helper = if n.z.abs() < 0.9 { DVec3::Z } else { DVec3::X };
    let u = n.cross(helper).normalize_or(DVec3::X);
    (u, n.cross(u))
}

fn box2(points: impl Iterator<Item = DVec2>) -> (DVec2, DVec2) {
    points.fold((DVec2::INFINITY, DVec2::NEG_INFINITY), |(lo, hi), p| (lo.min(p), hi.max(p)))
}

fn apart(a: (DVec2, DVec2), b: (DVec2, DVec2)) -> bool {
    a.0.x > b.1.x || b.0.x > a.1.x || a.0.y > b.1.y || b.0.y > a.1.y
}

/// Whether `solid` is a closed convex body: every edge of a face is walked
/// the other way by another face, and no point lies in front of a face.
fn convex(solid: &Solid) -> bool {
    let faces = &solid.faces;
    if faces.len() < 4 || faces.len() > MAX_CONVEX_FACES {
        return false;
    }
    let key = |p: DVec3| ((p.x * 1e6).round() as i64, (p.y * 1e6).round() as i64, (p.z * 1e6).round() as i64);
    let mut edges = std::collections::HashMap::new();
    for face in faces {
        let n = face.verts.len();
        for i in 0..n {
            let (a, b) = (key(face.verts[i]), key(face.verts[(i + 1) % n]));
            if a != b {
                *edges.entry((a, b)).or_insert(0usize) += 1;
            }
        }
    }
    if !edges.iter().all(|((a, b), n)| *n == 1 && edges.get(&(*b, *a)) == Some(&1)) {
        return false;
    }
    faces.iter().all(|f| {
        let Some(p0) = f.verts.first() else { return false };
        faces.iter().flat_map(|g| g.verts.iter()).all(|p| (*p - *p0).dot(f.normal) <= 1e-6)
    })
}

/// Whether the convex polygon `poly` (counter-clockwise) holds every point of `points`.
fn holds_all(poly: &[DVec2], points: &[DVec2]) -> bool {
    let n = poly.len();
    n >= 3 && points.iter().all(|p| (0..n).all(|i| (poly[(i + 1) % n] - poly[i]).perp_dot(*p - poly[i]) >= 0.0))
}

/// `poly` counter-clockwise when it is convex (else `None`).
fn as_convex(poly: &[DVec2]) -> Option<Vec<DVec2>> {
    let n = poly.len();
    let turn = |i: usize| (poly[(i + 1) % n] - poly[i]).perp_dot(poly[(i + 2) % n] - poly[(i + 1) % n]);
    if n >= 3 && (0..n).all(|i| turn(i) >= 0.0) {
        Some(poly.to_vec())
    } else if n >= 3 && (0..n).all(|i| turn(i) <= 0.0) {
        Some(poly.iter().rev().copied().collect())
    } else {
        None
    }
}

/// Whether the polygon `outer` holds the whole of `inner`: its corners
/// (those on its outline too), a point within it, and no side of one
/// crosses a side of the other.
fn encloses(outer: &[DVec2], inner: &[DVec2]) -> bool {
    fn sides(poly: &[DVec2]) -> impl Iterator<Item = (DVec2, DVec2)> + '_ {
        (0..poly.len()).map(move |i| (poly[i], poly[(i + 1) % poly.len()]))
    }
    let on_outline = |p: DVec2| {
        sides(outer).any(|(a, b)| {
            let t = ((p - a).dot(b - a) / (b - a).length_squared().max(1e-300)).clamp(0.0, 1.0);
            a.lerp(b, t).distance_squared(p) < 1e-12
        })
    };
    let crosses = |(a, b): (DVec2, DVec2), (c, d): (DVec2, DVec2)| {
        let (ab, cd) = (b - a, d - c);
        let denom = ab.perp_dot(cd);
        if denom.abs() <= 1e-12 * ab.length() * cd.length() {
            return false;
        }
        let (t, u) = ((c - a).perp_dot(cd) / denom, (c - a).perp_dot(ab) / denom);
        t > 1e-9 && t < 1.0 - 1e-9 && u > 1e-9 && u < 1.0 - 1e-9
    };
    let within = polygon::triangulate(inner).first().map(|t| (inner[t[0]] + inner[t[1]] + inner[t[2]]) / 3.0);
    within.is_some_and(|p| polygon::contains(outer, p))
        && inner.iter().all(|p| polygon::contains(outer, *p) || on_outline(*p))
        && !sides(inner).any(|side| sides(outer).any(|other| crosses(side, other)))
}

/// A caster slid onto a plane: its outline there, whether it is convex
/// (then counter-clockwise) and its box.
#[derive(Clone)]
struct Polygon {
    pts: Vec<DVec2>,
    convex: bool,
    lo: DVec2,
    hi: DVec2,
}

/// What the casters lay on a plane.
enum Cast {
    /// These polygons (to unite).
    Polygons(Vec<Polygon>),
    /// One of them covers the whole face.
    All,
}

/// A face that catches shadows, laid flat in its own plane.
struct Flat {
    /// A point of the plane, its unit normal and two unit vectors along it.
    p0: DVec3,
    normal: DVec3,
    u: DVec3,
    v: DVec3,
    /// The face's outline in the plane.
    outline: Vec<DVec2>,
    /// The convex polygon (counter-clockwise) the casters are cut to: the
    /// face when it is convex (`cut_to_face`), else its box.
    window: Vec<DVec2>,
    cut_to_face: bool,
}

impl Flat {
    /// The face `verts` with unit `normal`; `None` when it is no polygon.
    fn of(verts: &[DVec3], normal: DVec3) -> Option<Self> {
        let p0 = *verts.first()?;
        let (u, v) = basis(normal);
        let outline: Vec<DVec2> = verts.iter().map(|p| DVec2::new((*p - p0).dot(u), (*p - p0).dot(v))).collect();
        if verts.len() < 3 || !outline.iter().all(|p| p.is_finite()) || polygon::area(&outline) < MIN_AREA {
            return None;
        }
        let (lo, hi) = box2(outline.iter().copied());
        let window = as_convex(&outline);
        let cut_to_face = window.is_some();
        let window = window.unwrap_or_else(|| vec![lo, DVec2::new(hi.x, lo.y), hi, DVec2::new(lo.x, hi.y)]);
        Some(Self { p0, normal, u, v, outline, window, cut_to_face })
    }

    /// A point of the model in the plane's coordinates.
    fn at(&self, p: DVec3) -> DVec2 {
        DVec2::new((p - self.p0).dot(self.u), (p - self.p0).dot(self.v))
    }

    /// The point of the plane over (or under) the point `p` of the plan,
    /// in the plane's coordinates (the plane does not stand upright).
    fn over_plan(&self, p: DVec2) -> DVec2 {
        let d = p - self.p0.truncate();
        self.at(p.extend(self.p0.z - d.dot(self.normal.truncate()) / self.normal.z))
    }

    /// The whole face as one polygon.
    fn whole(&self) -> Polygon {
        let pts = if self.cut_to_face { self.window.clone() } else { self.outline.clone() };
        let (lo, hi) = box2(pts.iter().copied());
        Polygon { pts, convex: self.cut_to_face, lo, hi }
    }
}

impl<'a> Scene<'a> {
    /// The casters of `geom` under a sun towards unit `sun`: the faces of
    /// the solids `shown` that `skip` leaves in, opaque and not edge-on to
    /// the sun, those of a closed convex solid together as one unit; with
    /// a `cut` plane (a point and the direction kept), only what lies
    /// beyond it, closed by the cut's caps. `None` when the sun is not a
    /// finite direction or more than [`MAX_CASTERS`] faces cast.
    pub fn new(
        geom: &'a ModelGeometry,
        sun: DVec3,
        skip: Option<&Skip>,
        shown: &dyn Fn(&Solid) -> bool,
        cut: Option<(DVec3, DVec3)>,
    ) -> Option<Self> {
        let sun = sun.try_normalize().filter(|s| s.is_finite())?;
        let axes = basis(sun);
        let mut casters: Vec<Caster<'a>> = Vec::new();
        let mut units: Vec<Unit> = Vec::new();
        let seen_from = |p: &DVec3| DVec2::new(p.dot(axes.0), p.dot(axes.1));
        let unit =
            |element: ElementId, faces: Range<usize>, points: Vec<DVec3>, verts: &mut dyn Iterator<Item = DVec3>| {
                let (mut lo, mut hi, mut top) = (DVec2::INFINITY, DVec2::NEG_INFINITY, f64::NEG_INFINITY);
                for p in verts {
                    let s = seen_from(&p);
                    (lo, hi, top) = (lo.min(s), hi.max(s), top.max(p.dot(sun)));
                }
                Unit { element, faces, points, lo, hi, top }
            };
        let good = |verts: &[DVec3]| verts.len() >= 3 && verts.iter().all(|p| p.is_finite());
        let casts = |s: &Solid| {
            shown(s) && !skip.is_some_and(|k| k.hidden.contains(&s.element) || k.clear.contains(&s.element))
        };
        for (index, solid) in geom.solids.iter().enumerate().filter(|(_, s)| casts(s)) {
            let opaque = |f: usize| 1.0 - solid.paint_of(f).map_or(solid.transparency, |p| p.transparency) >= OPAQUE;
            let cut_face = |verts: &'a [DVec3]| match cut {
                None => Cow::Borrowed(verts),
                Some((point, keep)) => Cow::Owned(clip_polygon(verts, point, keep)),
            };
            let start = casters.len();
            if (0..solid.faces.len()).all(opaque) && convex(solid) {
                // One unit: the hull of what the cut leaves of it (its cap's corners are among the points).
                for (f, face) in solid.faces.iter().enumerate() {
                    let verts = cut_face(&face.verts);
                    if good(&verts) {
                        casters.push(Caster { verts, id: (index, f) });
                    }
                }
                let mut points: Vec<DVec3> = casters[start..].iter().flat_map(|c| c.verts.iter().copied()).collect();
                points.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)).then(a.z.total_cmp(&b.z)));
                points.dedup_by(|a, b| a.distance_squared(*b) < 1e-18);
                if points.len() >= 4 {
                    let faces = start..casters.len();
                    let made = unit(solid.element, faces, points.clone(), &mut points.iter().copied());
                    units.push(made);
                } else {
                    casters.truncate(start);
                }
            } else {
                for (f, face) in solid.faces.iter().enumerate() {
                    if face.normal.dot(sun).abs() < MIN_CASTING || !opaque(f) {
                        continue;
                    }
                    let verts = cut_face(&face.verts);
                    if good(&verts) {
                        casters.push(Caster { verts, id: (index, f) });
                    }
                }
                if let Some((point, keep)) = cut.filter(|(_, keep)| keep.dot(sun).abs() >= MIN_CASTING)
                    && solid.transparency <= 1.0 - OPAQUE
                {
                    let caps = caps(solid, point, keep).into_iter().filter(|c| good(c));
                    casters.extend(caps.map(|cap| Caster { verts: Cow::Owned(cap), id: (index, usize::MAX) }));
                }
                for (i, caster) in casters.iter().enumerate().skip(start) {
                    units.push(unit(solid.element, i..i + 1, Vec::new(), &mut caster.verts.iter().copied()));
                }
            }
            if casters.len() > MAX_CASTERS {
                return None;
            }
        }
        // The grid: about a unit a cell, over their boxes.
        let (lo, hi) =
            units.iter().fold((DVec2::INFINITY, DVec2::NEG_INFINITY), |(lo, hi), c| (lo.min(c.lo), hi.max(c.hi)));
        let side = ((units.len() as f64).sqrt().ceil() as usize).clamp(1, GRID_SIDE);
        let size = if units.is_empty() { DVec2::ONE } else { (hi - lo).max(DVec2::splat(1e-6)) };
        let origin = if units.is_empty() { DVec2::ZERO } else { lo };
        let mut scene = Self {
            sun,
            axes,
            casters,
            seen: vec![0; units.len()],
            units,
            origin,
            cell: size / side as f64,
            side,
            cells: vec![Vec::new(); side * side],
            large: Vec::new(),
            lookup: 0,
            points: 0,
            looks: 0,
            failed: false,
        };
        for i in 0..scene.units.len() {
            let (a, z) = scene.span(scene.units[i].lo, scene.units[i].hi);
            if (z.0 - a.0 + 1) * (z.1 - a.1 + 1) > LARGE {
                scene.large.push(i as u32);
                continue;
            }
            for y in a.1..=z.1 {
                for x in a.0..=z.0 {
                    if let Some(c) = scene.cells.get_mut(y * side + x) {
                        c.push(i as u32);
                    }
                }
            }
        }
        Some(scene)
    }

    /// What covers the faces of `geom` from the sky: the floors, roofs and
    /// ceilings among the solids `shown` that `skip` leaves in, opaque, as
    /// the casters of a sun straight overhead. What they lay on a face is
    /// the part of it that lies under them, and they stand "towards the
    /// sun" of the points they lie above (see [`Scene::above`]).
    pub fn covers(
        geom: &'a ModelGeometry,
        skip: Option<&Skip>,
        shown: &dyn Fn(&Solid) -> bool,
        cut: Option<(DVec3, DVec3)>,
    ) -> Option<Self> {
        Self::new(geom, DVec3::Z, skip, &|s| covers(s.category) && shown(s), cut)
    }

    /// Whether a limit was passed: the view then draws no shadows.
    pub fn failed(&self) -> bool {
        self.failed
    }

    /// The cells from the one holding `lo` to the one holding `hi` (within the grid).
    fn span(&self, lo: DVec2, hi: DVec2) -> ((usize, usize), (usize, usize)) {
        let at = |p: DVec2| {
            let c = ((p - self.origin) / self.cell).floor();
            let clamp = |v: f64| if v.is_nan() { 0 } else { v.clamp(0.0, (self.side - 1) as f64) as usize };
            (clamp(c.x), clamp(c.y))
        };
        (at(lo), at(hi))
    }

    /// The units whose boxes, seen from the sun, may reach `lo`..`hi`: the
    /// large ones first.
    fn around(&mut self, lo: DVec2, hi: DVec2) -> Vec<usize> {
        self.lookup = self.lookup.wrapping_add(1);
        if self.lookup == 0 {
            self.seen.fill(0);
            self.lookup = 1;
        }
        let mut out: Vec<usize> = self.large.iter().map(|i| *i as usize).collect();
        let (a, z) = self.span(lo, hi);
        for y in a.1..=z.1 {
            for x in a.0..=z.0 {
                for &i in self.cells.get(y * self.side + x).into_iter().flatten() {
                    if let Some(s) = self.seen.get_mut(i as usize)
                        && *s != self.lookup
                    {
                        *s = self.lookup;
                        out.push(i as usize);
                    }
                }
            }
        }
        out
    }

    /// Count `points` given to a union; `false` (and no more shadows) past a limit.
    fn spend(&mut self, points: usize) -> bool {
        self.points = self.points.saturating_add(points);
        if points > MAX_UNION_POINTS || self.points > MAX_POINTS {
            self.failed = true;
        }
        !self.failed
    }

    /// What the units `around` lay on the plane through `p0` with unit
    /// `normal` (at `facing` to the sun), as `flat` maps it, of what stands
    /// in front of it and within the convex `window` (counter-clockwise),
    /// leaving out the face `own` (and, with an element, everything of that
    /// element); [`Cast::All`] as soon as one covers the whole `outline`.
    /// Polygons inside another are left out.
    fn cast(
        &self,
        around: &[usize],
        (p0, normal, facing): (DVec3, DVec3, f64),
        flat: &dyn Fn(DVec3) -> DVec2,
        (own, apart_from): ((usize, usize), Option<ElementId>),
        seen: Option<((DVec2, DVec2), f64)>,
        (window, outline): (&[DVec2], &[DVec2]),
    ) -> Cast {
        let sun = self.sun;
        let front = p0 + normal * LIFT;
        let bounds = box2(window.iter().copied());
        // Slid along the sun's rays onto the plane.
        let slid = |p: DVec3| flat(p - sun * ((p - p0).dot(normal) / facing));
        let mut polys: Vec<Polygon> = Vec::new();
        for unit in around.iter().filter_map(|i| self.units.get(*i)) {
            if seen.is_some_and(|(seen, low)| unit.top <= low + LIFT || apart((unit.lo, unit.hi), seen))
                || apart_from == Some(unit.element)
            {
                continue;
            }
            let faces = self.casters.get(unit.faces.clone()).unwrap_or(&[]);
            let poly = if unit.points.is_empty() {
                // A face: what of it stands in front of the plane.
                let Some(c) = faces.first().filter(|c| c.id != own) else { continue };
                let part = clip_polygon(&c.verts, front, normal);
                if part.len() < 3 {
                    continue;
                }
                part.iter().map(|p| slid(*p)).collect::<Vec<DVec2>>()
            } else {
                // A convex solid: the hull of what stands in front of the plane.
                let height = |p: &DVec3| (*p - p0).dot(normal);
                let (lowest, highest) = unit
                    .points
                    .iter()
                    .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), p| (lo.min(height(p)), hi.max(height(p))));
                if highest <= LIFT {
                    continue;
                }
                let points: Vec<DVec2> = if lowest >= LIFT {
                    unit.points.iter().map(|p| slid(*p)).collect()
                } else {
                    faces.iter().flat_map(|c| clip_polygon(&c.verts, front, normal)).map(slid).collect()
                };
                polygon::convex_hull(&points)
            };
            if poly.len() < 3 || !poly.iter().all(|p| p.is_finite()) || apart(box2(poly.iter().copied()), bounds) {
                continue;
            }
            let convex = as_convex(&poly);
            if !outline.is_empty() && convex.as_ref().is_some_and(|c| holds_all(c, outline)) {
                return Cast::All;
            }
            // Only what lies in the window matters (and crosses the other polygons).
            let pts = crate::projected::clip_convex(convex.as_deref().unwrap_or(&poly), window);
            if pts.len() >= 3 && polygon::area(&pts) > MIN_AREA {
                let (lo, hi) = box2(pts.iter().copied());
                polys.push(Polygon { pts, convex: convex.is_some(), lo, hi });
            }
        }
        // A polygon inside a convex one adds nothing: from the largest down,
        // each is compared with the convex ones kept so far (of two alike,
        // the first stays), while the work allows.
        let mut order: Vec<usize> = (0..polys.len()).collect();
        let areas: Vec<f64> = polys.iter().map(|p| polygon::area(&p.pts)).collect();
        order.sort_by(|a, b| areas[*b].total_cmp(&areas[*a]).then(a.cmp(b)));
        let mut kept: Vec<usize> = Vec::new();
        let mut convex: Vec<usize> = Vec::new();
        let mut work = 0usize;
        for i in order {
            let a = &polys[i];
            let inside = |j: &usize| {
                let b = &polys[*j];
                b.lo.cmple(a.lo).all() && b.hi.cmpge(a.hi).all() && holds_all(&b.pts, &a.pts)
            };
            work = work.saturating_add(convex.len());
            if work <= MAX_PRUNING && convex.iter().any(inside) {
                continue;
            }
            kept.push(i);
            if a.convex {
                convex.push(i);
            }
        }
        if kept.len() < polys.len() {
            kept.sort_unstable();
            let mut slots: Vec<Option<Polygon>> = polys.into_iter().map(Some).collect();
            polys = kept.into_iter().filter_map(|i| slots.get_mut(i).and_then(Option::take)).collect();
        }
        Cast::Polygons(polys)
    }

    /// The region `polys` cover and `less` do not, within `clip` (when they
    /// were not all cut to it already): rings. Convex polygons apart from
    /// one another, with nothing to leave out, are their own rings; the
    /// others are united ([`region::covered_except`]; a union that fails
    /// leaves no shadow here), many of them a few at a time
    /// ([`UNION_GROUP`]), which costs less than all at once.
    fn rings(&mut self, polys: Vec<Polygon>, less: Vec<Polygon>, clip: Option<&[DVec2]>) -> Vec<Vec<DVec2>> {
        if polys.is_empty() {
            return Vec::new();
        }
        // What lies apart from every polygon takes nothing out.
        let around =
            polys.iter().fold((DVec2::INFINITY, DVec2::NEG_INFINITY), |(lo, hi), p| (lo.min(p.lo), hi.max(p.hi)));
        let less: Vec<Vec<DVec2>> = less.into_iter().filter(|p| !apart((p.lo, p.hi), around)).map(|p| p.pts).collect();
        let simple = less.is_empty()
            && clip.is_none()
            && polys.iter().all(|p| p.convex)
            && (0..polys.len()).all(|i| (0..i).all(|j| apart((polys[i].lo, polys[i].hi), (polys[j].lo, polys[j].hi))));
        if simple {
            return polys.into_iter().map(|p| p.pts).collect();
        }
        let mut polys: Vec<(f64, Vec<DVec2>)> = polys.into_iter().map(|p| (p.lo.x, p.pts)).collect();
        let points = polys.iter().map(|p| p.1.len()).chain(less.iter().map(Vec::len)).sum::<usize>()
            + clip.map_or(0, <[DVec2]>::len);
        if !self.spend(points) {
            return Vec::new();
        }
        // Many polygons over one another are united a few neighbors at a
        // time, then those unions together, while that at least halves
        // them: the edges inside a union are gone before they would cross
        // the rest. Neighbors that cannot be united go on as they are.
        while polys.len() > 2 * UNION_GROUP {
            let before = polys.len();
            polys.sort_by(|a, b| a.0.total_cmp(&b.0));
            let mut united: Vec<(f64, Vec<DVec2>)> = Vec::new();
            let mut left = polys.into_iter().map(|p| p.1);
            loop {
                let group: Vec<Vec<DVec2>> = left.by_ref().take(UNION_GROUP).collect();
                if group.is_empty() {
                    break;
                }
                let rings = region::covered(&group, None, MAX_UNION_POINTS).unwrap_or(group);
                united.extend(rings.into_iter().map(|r| (r.iter().map(|p| p.x).fold(f64::INFINITY, f64::min), r)));
            }
            polys = united;
            if polys.len() * 2 > before {
                break;
            }
        }
        let polys: Vec<Vec<DVec2>> = polys.into_iter().map(|p| p.1).collect();
        region::covered_except(&polys, &less, clip, MAX_UNION_POINTS).unwrap_or_default()
    }

    /// What the casters lay on the face `flat` (with corners `verts`, the
    /// face `own` of the geometry), leaving out everything of the element
    /// `apart_from`; nothing on a face turned away from the sun or edge-on
    /// to it, or once a limit is passed.
    fn over(&mut self, verts: &[DVec3], flat: &Flat, own: (usize, usize), apart_from: Option<ElementId>) -> Cast {
        let facing = flat.normal.dot(self.sun);
        if self.failed || facing.is_nan() || facing < MIN_FACING {
            return Cast::Polygons(Vec::new());
        }
        let (e1, e2) = self.axes;
        let seen = box2(verts.iter().map(|p| DVec2::new(p.dot(e1), p.dot(e2))));
        let low = verts.iter().map(|p| p.dot(self.sun)).fold(f64::INFINITY, f64::min);
        let around = self.around(seen.0, seen.1);
        self.looks = self.looks.saturating_add(around.len());
        if self.looks > MAX_LOOKS {
            self.failed = true;
            return Cast::Polygons(Vec::new());
        }
        let plane = (flat.p0, flat.normal, facing);
        let on = (flat.window.as_slice(), flat.outline.as_slice());
        self.cast(&around, plane, &|p| flat.at(p), (own, apart_from), Some((seen, low)), on)
    }

    /// Whether a caster stands over the point `p`, towards the sun (for
    /// [`Scene::covers`]: whether a floor, roof or ceiling lies above it),
    /// leaving out everything of the element `apart_from`.
    pub fn above(&self, p: DVec3, apart_from: Option<ElementId>) -> bool {
        let seen = DVec2::new(p.dot(self.axes.0), p.dot(self.axes.1));
        let depth = p.dot(self.sun);
        if self.failed || !seen.is_finite() || !depth.is_finite() {
            return false;
        }
        let (cell, _) = self.span(seen, seen);
        let near = self.cells.get(cell.1 * self.side + cell.0).into_iter().flatten().chain(&self.large);
        near.filter_map(|i| self.units.get(*i as usize)).any(|unit| {
            let beside = seen.cmplt(unit.lo).any() || seen.cmpgt(unit.hi).any();
            if beside || unit.top <= depth + LIFT || apart_from == Some(unit.element) {
                return false;
            }
            self.casters.get(unit.faces.clone()).unwrap_or(&[]).iter().any(|c| {
                let outline: Vec<DVec2> =
                    c.verts.iter().map(|v| DVec2::new(v.dot(self.axes.0), v.dot(self.axes.1))).collect();
                // Where the sun's ray from the point meets the face's plane.
                let n = (1..c.verts.len().saturating_sub(1))
                    .map(|i| (c.verts[i] - c.verts[0]).cross(c.verts[i + 1] - c.verts[0]))
                    .fold(DVec3::ZERO, |sum, n| sum + n);
                let along = n.dot(self.sun);
                polygon::contains(&outline, seen)
                    && along.abs() > 1e-12 * n.length()
                    && (c.verts[0] - p).dot(n) / along > LIFT
            })
        })
    }

    /// The part of the face `flat` (with corners `verts`, the face `own`
    /// of `element`), seen from above, that is `indoors`: within a space
    /// of the storey just above it and under a floor, roof or ceiling of
    /// another element, as this scene casts them straight down
    /// ([`Scene::covers`]). `None`: all of it (or what is indoors of it
    /// could not be found: it catches no shadow).
    fn indoors_on(
        &mut self,
        verts: &[DVec3],
        flat: &Flat,
        (own, element): ((usize, usize), ElementId),
        indoors: &Indoors,
    ) -> Option<Vec<Polygon>> {
        // A face that stands upright has no part under anything.
        if flat.normal.z < MIN_FACING {
            return Some(Vec::new());
        }
        let plan: Vec<DVec2> = verts.iter().map(|p| p.truncate()).collect();
        let around = box2(plan.iter().copied());
        let height = verts.iter().map(|p| p.z).sum::<f64>() / verts.len().max(1) as f64 + PLAN_PROBE;
        let spaces: Vec<&Vec<DVec2>> =
            indoors.spaces_at(height).filter(|s| !apart(box2(s.iter().copied()), around)).collect();
        if spaces.is_empty() {
            return Some(Vec::new());
        }
        let (all, cover) = match self.over(verts, flat, own, Some(element)) {
            Cast::All => (true, vec![flat.whole()]),
            Cast::Polygons(polys) => (false, polys),
        };
        if cover.is_empty() {
            return Some(Vec::new());
        }
        // The whole face in one space: what is covered of it is indoors.
        if spaces.iter().any(|space| encloses(space, &plan)) {
            if all {
                return None;
            }
            let clip = (!flat.cut_to_face).then_some(flat.outline.as_slice());
            let under: f64 = self.rings(cover.clone(), Vec::new(), clip).iter().map(|r| polygon::area(r)).sum();
            return (under < polygon::area(&flat.outline) * (1.0 - 1e-9)).then_some(cover);
        }
        // Else what is covered within each space.
        let cover: Vec<Vec<DVec2>> = cover.into_iter().map(|p| p.pts).collect();
        let mut parts = Vec::new();
        for space in spaces {
            let space: Vec<DVec2> = space.iter().map(|p| flat.over_plan(*p)).collect();
            if !self.spend(cover.iter().map(Vec::len).sum::<usize>() + space.len()) {
                return None;
            }
            for pts in region::covered(&cover, Some(&space), MAX_UNION_POINTS)? {
                let (lo, hi) = box2(pts.iter().copied());
                parts.push(Polygon { pts, convex: false, lo, hi });
            }
        }
        Some(parts)
    }

    /// The shadows cast on the face `own` (its solid and face in the
    /// geometry) with outline `verts` and unit `normal`: rings in its
    /// plane (see [`region::covered`]). None on a face turned away from the
    /// sun or edge-on to it.
    ///
    /// With what is `indoors` (a floor plan's faces, seen from above): the
    /// scene of what covers from the sky ([`Scene::covers`]), the spaces
    /// indoors and the face's element, the part of the face that is
    /// indoors catches nothing: it is taken out of its shadows. A limit
    /// passed by the cover is passed by this scene too.
    pub fn on_face(
        &mut self,
        verts: &[DVec3],
        normal: DVec3,
        own: (usize, usize),
        indoors: Option<(&mut Scene<'_>, &Indoors, ElementId)>,
    ) -> Vec<Vec<DVec3>> {
        let facing = normal.dot(self.sun);
        if self.failed || facing.is_nan() || facing < MIN_FACING {
            return Vec::new();
        }
        let Some(flat) = Flat::of(verts, normal) else { return Vec::new() };
        let mut inside = Vec::new();
        if let Some((cover, indoors, element)) = indoors {
            let parts = cover.indoors_on(verts, &flat, (own, element), indoors);
            if cover.failed {
                self.failed = true;
            }
            match parts.filter(|_| !self.failed) {
                Some(parts) => inside = parts,
                None => return Vec::new(),
            }
        }
        let polys = match self.over(verts, &flat, own, None) {
            Cast::All if inside.is_empty() => return vec![verts.to_vec()],
            Cast::All => vec![flat.whole()],
            Cast::Polygons(polys) => polys,
        };
        // The casters are cut to the face when it is convex, else to its box (and the union to the face).
        let clip = (!flat.cut_to_face).then_some(flat.outline.as_slice());
        let rings = self.rings(polys, inside, clip);
        rings.into_iter().map(|r| r.into_iter().map(|q| flat.p0 + flat.u * q.x + flat.v * q.y).collect()).collect()
    }

    /// The shadows cast on the level plane at height `z` around a model
    /// within `bounds`: rings at that height, no farther from the model
    /// than the shadows of a sun [`MIN_GROUND_ALTITUDE_DEG`] up reach.
    /// With what is `indoors` (the scene of what covers from the sky,
    /// [`Scene::covers`], and the spaces indoors), none where the plane is
    /// indoors: within a space and under a floor, roof or ceiling.
    pub fn on_ground(
        &mut self,
        z: f64,
        bounds: (DVec3, DVec3),
        indoors: Option<(&mut Scene<'_>, &Indoors)>,
    ) -> Vec<Vec<DVec3>> {
        let height = bounds.1.z - z;
        let finite = z.is_finite() && bounds.0.is_finite() && bounds.1.is_finite();
        if self.failed || !finite || self.sun.z < MIN_FACING || height <= LIFT {
            return Vec::new();
        }
        let reach = (height / MIN_GROUND_ALTITUDE_DEG.to_radians().tan()).min(MAX_GROUND_REACH);
        let (lo, hi) = (bounds.0.truncate() - reach, bounds.1.truncate() + reach);
        let window = [lo, DVec2::new(hi.x, lo.y), hi, DVec2::new(lo.x, hi.y)];
        let none = ((usize::MAX, usize::MAX), None);
        let on = |scene: &Scene<'_>| {
            let all: Vec<usize> = (0..scene.units.len()).collect();
            let plane = (DVec3::new(0.0, 0.0, z), DVec3::Z, scene.sun.z);
            scene.cast(&all, plane, &|p| p.truncate(), none, None, (&window, &[]))
        };
        let Cast::Polygons(polys) = on(self) else { return Vec::new() };
        let mut inside = Vec::new();
        if let Some((cover, indoors)) = indoors {
            let spaces: Vec<&Vec<DVec2>> = indoors.spaces_at(z + PLAN_PROBE).collect();
            let covered = match on(&*cover) {
                Cast::Polygons(covered) if !spaces.is_empty() => covered,
                _ => Vec::new(),
            };
            let covered: Vec<Vec<DVec2>> = covered.into_iter().map(|p| p.pts).collect();
            for space in spaces.into_iter().filter(|_| !covered.is_empty()) {
                let spent = cover.spend(covered.iter().map(Vec::len).sum::<usize>() + space.len());
                // What is indoors could not be found: no shadow on the ground.
                let Some(rings) = region::covered(&covered, Some(space), MAX_UNION_POINTS).filter(|_| spent) else {
                    self.failed |= cover.failed;
                    return Vec::new();
                };
                inside.extend(rings.into_iter().map(|pts| {
                    let (lo, hi) = box2(pts.iter().copied());
                    Polygon { pts, convex: false, lo, hi }
                }));
            }
            if cover.failed {
                self.failed = true;
                return Vec::new();
            }
        }
        let rings = self.rings(polys, inside, None);
        rings.into_iter().map(|r| r.into_iter().map(|q| q.extend(z)).collect()).collect()
    }
}

/// The caps that close `solid` where the plane through `point` cuts it,
/// keeping the side `keep` points to: the loops of its section, each outer
/// one with the loops inside it as holes ([`polygon::keyhole`]).
fn caps(solid: &Solid, point: DVec3, keep: DVec3) -> Vec<Vec<DVec3>> {
    let frame = Frame::looking(point, keep);
    let segs: Vec<(DVec2, DVec2)> = solid
        .faces
        .iter()
        .flat_map(|f| face_plane_segments(f, point, keep))
        .map(|(a, b)| (frame.project(a), frame.project(b)))
        .collect();
    if segs.is_empty() {
        return Vec::new();
    }
    let loops: Vec<Vec<DVec2>> =
        crate::projected::chain_loops(&segs).into_iter().filter(|l| polygon::area(l) > MIN_AREA).collect();
    // A loop inside an odd number of others is a hole of the smallest of them.
    let depth: Vec<Vec<usize>> = (0..loops.len())
        .map(|i| (0..loops.len()).filter(|&j| j != i && polygon::contains(&loops[j], loops[i][0])).collect())
        .collect();
    let mut out = Vec::new();
    for (i, outer) in loops.iter().enumerate().filter(|(i, _)| depth[*i].len().is_multiple_of(2)) {
        let holes: Vec<Vec<DVec2>> = (0..loops.len())
            .filter(|&h| depth[h].len() == depth[i].len() + 1 && depth[h].contains(&i))
            .map(|h| loops[h].clone())
            .collect();
        let ring = if holes.is_empty() { Some(outer.clone()) } else { polygon::keyhole(outer, &holes) };
        // Holes that cannot be joined: the outline alone (a shadow where light passes).
        let ring = ring.unwrap_or_else(|| outer.clone());
        out.push(ring.into_iter().map(|p| frame.origin + frame.right * p.x + frame.up * p.y).collect());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Look, Prim};
    use cc_core::Category;

    fn solid(id: u64, lo: DVec3, hi: DVec3) -> Solid {
        solid_of(id, Category::Wall, lo.into(), hi.into())
    }

    /// A box of `category` from corner to corner.
    fn solid_of(id: u64, category: Category, lo: (f64, f64, f64), hi: (f64, f64, f64)) -> Solid {
        let mut s = Solid::new(ElementId(id), category, [200; 3]);
        s.add_box(DVec2::ZERO, DVec2::X, DVec2::Y, lo.0, hi.0, lo.1, hi.1, lo.2, hi.2);
        s
    }

    fn model(solids: Vec<Solid>) -> ModelGeometry {
        ModelGeometry { solids, ..Default::default() }
    }

    fn look(sun: DVec3, ground: Option<f64>) -> Look<'static> {
        let shadows = Some(Shadows { ground, skip: None, catch: Catch::All });
        Look { light: Some(sun.normalize()), shadows, ..Look::plain() }
    }

    /// The shadows of a drawing: the black veils that belong to no element.
    fn veils(prims: &[Prim]) -> Vec<(Vec<DVec2>, u8)> {
        prims
            .iter()
            .filter_map(|p| match p {
                Prim::Fill { pts, color: [0, 0, 0, a], element: None, cut: false } => Some((pts.clone(), *a)),
                _ => None,
            })
            .collect()
    }

    fn bounds(pts: &[DVec2]) -> (DVec2, DVec2) {
        box2(pts.iter().copied())
    }

    /// A floor plan's underlay: it takes the plan's own rule ([`Catch::Uncovered`]).
    fn plan(geom: &ModelGeometry, cut_z: f64, look: Look<'_>) -> Vec<Prim> {
        crate::plan::style_underlay(geom, cut_z, false, look, &Default::default())
    }

    /// Looking north from the line y = `y`, the model in front of it cut away.
    fn section(geom: &ModelGeometry, y: f64, look: Look<'_>) -> Vec<Prim> {
        let mut frame = Frame::looking(DVec3::new(0.0, y, 0.0), DVec3::Y);
        frame.near = Some(0.0);
        crate::projected::render(geom, &frame, None, look, &Default::default()).prims
    }

    /// The veils of `prims` of opacity `alpha`, and the area they cover.
    fn veils_of(prims: &[Prim], alpha: u8) -> (Vec<Vec<DVec2>>, f64) {
        let rings: Vec<Vec<DVec2>> = veils(prims).into_iter().filter(|(_, a)| *a == alpha).map(|(r, _)| r).collect();
        let area = rings.iter().map(|r| polygon::area(r)).sum();
        (rings, area)
    }

    /// Whether one of `rings` runs from `lo` to `hi`.
    fn spans(rings: &[Vec<DVec2>], lo: (f64, f64), hi: (f64, f64)) -> bool {
        rings.iter().any(|r| {
            let (a, b) = bounds(r);
            a.distance(DVec2::new(lo.0, lo.1)) < 1e-3 && b.distance(DVec2::new(hi.0, hi.1)) < 1e-3
        })
    }

    fn south(geom: &ModelGeometry, look: Look<'_>) -> Vec<Prim> {
        let frame = Frame::looking(DVec3::ZERO, DVec3::Y);
        crate::projected::render(geom, &frame, None, look, &Default::default()).prims
    }

    /// The fill of the face of `element` holding `p`, as painted (the last one).
    fn face_at(prims: &[Prim], element: u64, p: DVec2) -> [u8; 4] {
        prims
            .iter()
            .rev()
            .find_map(|prim| match prim {
                Prim::Fill { pts, color, element: Some(e), .. } if e.0 == element && polygon::contains(pts, p) => {
                    Some(*color)
                }
                _ => None,
            })
            .unwrap()
    }

    /// A box 2 m high on the ground under a sun 45° up in the west: in a
    /// plan its shadow on the ground runs 2 m east of it, one ring with the
    /// box's width, in the ground's veil; under everything else.
    #[test]
    fn a_box_casts_its_shadow_on_the_ground_of_a_plan() {
        let geom = model(vec![solid(1, DVec3::ZERO, DVec3::new(1.0, 1.0, 2.0))]);
        let prims = plan(&geom, 10.0, look(DVec3::new(-1.0, 0.0, 1.0), Some(0.0)));
        let shadows = veils(&prims);
        assert_eq!(shadows.len(), 1, "{shadows:?}");
        let (ring, alpha) = &shadows[0];
        assert_eq!(*alpha, ground_veil(&ShadingSettings::default()));
        assert_eq!(ring.len(), 4, "{ring:?}");
        let (lo, hi) = bounds(ring);
        assert!(lo.distance(DVec2::new(LIFT, 0.0)) < 1e-6 && hi.distance(DVec2::new(3.0, 1.0)) < 1e-6, "{lo} {hi}");
        assert!((polygon::area(ring) - 3.0).abs() < 1e-3);
        assert!(matches!(&prims[0], Prim::Fill { element: None, .. }), "the ground's shadow comes first");
        // A lower ground: the shadow is longer and starts farther away.
        let low = veils(&plan(&geom, 10.0, look(DVec3::new(-1.0, 0.0, 1.0), Some(-1.0))));
        let (lo, hi) = bounds(&low[0].0);
        assert!(lo.distance(DVec2::new(1.0, 0.0)) < 1e-6 && hi.distance(DVec2::new(4.0, 1.0)) < 1e-6, "{lo} {hi}");
        // No ground plane, a ground above the plan's cut: no ground shadow.
        assert!(veils(&plan(&geom, 10.0, look(DVec3::new(-1.0, 0.0, 1.0), None))).is_empty());
        assert!(veils(&plan(&geom, 1.0, look(DVec3::new(-1.0, 0.0, 1.0), Some(1.5)))).is_empty());
    }

    /// A floor plan reads the site: a box 3 m high cut at 1.2 m casts its
    /// whole shadow, 3 m long. A view that casts from what its cut leaves
    /// (a section; here the same view from above catching as one) casts
    /// the shadow of a box 1.2 m high, closed by its cut.
    #[test]
    fn a_plan_casts_from_the_whole_model_and_a_cut_view_from_what_its_cut_leaves() {
        let sun = DVec3::new(-1.0, 0.0, 1.0);
        let cut_view = |geom: &ModelGeometry| {
            let mut frame = Frame::looking(DVec3::new(0.0, 0.0, 1.2), -DVec3::Z);
            frame.near = Some(0.0);
            veils(&crate::projected::render(geom, &frame, None, look(sun, Some(0.0)), &Default::default()).prims)
        };
        let geom = model(vec![solid(1, DVec3::ZERO, DVec3::new(1.0, 1.0, 3.0))]);
        let shadows = veils(&plan(&geom, 1.2, look(sun, Some(0.0))));
        assert_eq!(shadows.len(), 1, "{shadows:?}");
        let (lo, hi) = bounds(&shadows[0].0);
        assert!(lo.distance(DVec2::new(LIFT, 0.0)) < 1e-6 && hi.distance(DVec2::new(4.0, 1.0)) < 1e-6, "{lo} {hi}");
        let shadows = cut_view(&geom);
        assert_eq!(shadows.len(), 1, "{shadows:?}");
        let (lo, hi) = bounds(&shadows[0].0);
        assert!(lo.distance(DVec2::new(LIFT, 0.0)) < 1e-6 && hi.distance(DVec2::new(2.2, 1.0)) < 1e-6, "{lo} {hi}");
        // The box is one caster, the hull of what is left of it: its corners at the cut among its points.
        let cut = Some((DVec3::new(0.0, 0.0, 1.2), -DVec3::Z));
        let scene = Scene::new(&geom, DVec3::Z, None, &|_| true, cut).unwrap();
        assert_eq!(scene.units.len(), 1);
        let top = scene.units[0].points.iter().filter(|p| (p.z - 1.2).abs() < 1e-9).count();
        assert_eq!((scene.units[0].points.len(), top), (8, 4));
        // A solid that is not convex (an L of two boxes) casts face by face, closed by the cut's cap.
        let mut l = solid(1, DVec3::ZERO, DVec3::new(1.0, 1.0, 3.0));
        l.faces.extend(solid(1, DVec3::new(1.0, 0.0, 0.0), DVec3::new(2.0, 0.5, 3.0)).faces);
        let geom = model(vec![l]);
        let scene = Scene::new(&geom, sun, None, &|_| true, cut).unwrap();
        let caps: Vec<_> = scene.casters.iter().filter(|c| c.id.1 == usize::MAX).collect();
        assert!(!caps.is_empty() && scene.units.iter().all(|u| u.points.is_empty()));
        assert!(caps.iter().flat_map(|c| c.verts.iter()).all(|p| (p.z - 1.2).abs() < 1e-9));
        let flat = |c: &&Caster| polygon::area(&c.verts.iter().map(|p| p.truncate()).collect::<Vec<_>>());
        assert!((caps.iter().map(flat).sum::<f64>() - 1.5).abs() < 1e-9);
        let shadows = cut_view(&geom);
        assert_eq!(shadows.len(), 1, "{shadows:?}");
        // The L's outline swept 1.2 m east: 3.2 m long along its foot, 2.2 m along its stem.
        assert!((polygon::area(&shadows[0].0) - 2.7).abs() < 1e-3, "{}", polygon::area(&shadows[0].0));
        // In the plan, swept its whole 3 m.
        let shadows = veils(&plan(&geom, 1.2, look(sun, Some(0.0))));
        assert_eq!(shadows.len(), 1, "{shadows:?}");
        assert!((polygon::area(&shadows[0].0) - 4.5).abs() < 1e-3, "{}", polygon::area(&shadows[0].0));
    }

    /// A rectangle in plan.
    fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<DVec2> {
        vec![DVec2::new(x0, y0), DVec2::new(x1, y0), DVec2::new(x1, y1), DVec2::new(x0, y1)]
    }

    /// These spaces indoors, on one storey from under the ground up.
    fn indoors(spaces: Vec<Vec<DVec2>>) -> Indoors {
        Indoors { storeys: vec![Storey { z: (-10.0, f64::INFINITY), spaces }], failed: false }
    }

    /// A house of two storeys 6 x 4 m (floors, walls, a flat roof with
    /// eaves of 0.6 m) with an open terrace beside it and the two beams of
    /// a pergola over the terrace, under a sun 45 degrees up in the south,
    /// in the plan of the ground floor cut at 1.2 m. The whole house casts
    /// on the ground, the upper storey and the roof too, 7 m long, and
    /// behind the house the shadow starts at the wall, under the eave; the
    /// beams cast on the terrace, under the pergola. The floor indoors gets
    /// none, though the south wall stands between it and the sun. Under a
    /// roof the terrace catches the roof's shadow, as a porch does, unless
    /// it is a space indoors; and a space whose roof is hidden is open to
    /// the sky.
    #[test]
    fn a_plan_casts_on_all_but_what_is_indoors() {
        use Category::{Beam, Floor, Roof, Wall};
        let mut solids = vec![
            solid_of(1, Floor, (0.0, 0.0, -0.2), (6.0, 4.0, 0.0)),
            solid_of(2, Floor, (6.0, 0.0, -0.2), (10.0, 4.0, 0.0)),
            solid_of(7, Floor, (0.0, 0.0, 3.0), (6.0, 4.0, 3.2)),
            solid_of(12, Roof, (-0.6, -0.6, 6.0), (6.6, 4.6, 6.2)),
            solid_of(13, Beam, (6.0, 0.0, 2.4), (10.0, 0.2, 2.5)),
            solid_of(14, Beam, (6.0, 1.0, 2.4), (10.0, 1.2, 2.5)),
        ];
        for (storey, z) in [(0, (0.0, 3.0)), (1, (3.2, 6.0))] {
            let id = 3 + storey * 5;
            solids.push(solid_of(id, Wall, (0.0, 0.0, z.0), (6.0, 0.2, z.1)));
            solids.push(solid_of(id + 1, Wall, (0.0, 3.8, z.0), (6.0, 4.0, z.1)));
            solids.push(solid_of(id + 2, Wall, (0.0, 0.2, z.0), (0.2, 3.8, z.1)));
            solids.push(solid_of(id + 3, Wall, (5.8, 0.2, z.0), (6.0, 3.8, z.1)));
        }
        let geom = model(solids.clone());
        let sun = DVec3::new(0.0, -1.0, 1.0).normalize();
        let on_face = veil(VisualStyle::Shaded, sun.z, &ShadingSettings::default());
        assert_ne!(on_face, ground_veil(&ShadingSettings::default()));
        fn site<'a>(sun: DVec3, indoors: &'a Indoors, skip: Option<&'a Skip>) -> Look<'a> {
            let shadows = Some(Shadows { ground: Some(-0.2), skip, catch: Catch::Site(indoors) });
            Look { shadows, ..look(sun, None) }
        }
        let house = &indoors(vec![rect(0.0, 0.0, 6.0, 4.0)]);
        let holds = |rings: &[Vec<DVec2>], x: f64, y: f64| rings.iter().any(|r| polygon::contains(r, DVec2::new(x, y)));
        let prims = plan(&geom, 1.2, site(sun, house, None));
        // The ground: 6 x 7 m north of the house, the eaves' 0.6 x 5.4 m
        // either side of it, and the terrace's own 4 x 4.2 m.
        let (ground, area) = veils_of(&prims, ground_veil(&ShadingSettings::default()));
        assert!((area - 65.28).abs() < 1e-2, "{area}: {ground:?}");
        let all: Vec<DVec2> = ground.iter().flatten().copied().collect();
        let (lo, hi) = bounds(&all);
        assert!(lo.distance(DVec2::new(-0.6, 0.0)) < 1e-3 && hi.distance(DVec2::new(10.0, 11.0)) < 1e-3, "{lo} {hi}");
        // Behind the house, under its eave, the shadow starts at the wall;
        // in front of it the sun reaches under the eave; none indoors.
        assert!(holds(&ground, 3.0, 4.01) && holds(&ground, 3.0, 4.3) && holds(&ground, 3.0, 4.59));
        assert!(!holds(&ground, 3.0, -0.3) && !holds(&ground, 3.0, 2.0) && !holds(&ground, 3.0, 3.99));
        // The terrace: the beams' shadows, 2.4 m north of each; nothing indoors.
        let (rings, area) = veils_of(&prims, on_face);
        assert_eq!(rings.len(), 2, "{rings:?}");
        assert!((area - 2.4).abs() < 1e-6, "{area}");
        assert!(spans(&rings, (6.0, 2.4), (10.0, 2.7)) && spans(&rings, (6.0, 3.4), (10.0, 3.7)), "{rings:?}");
        assert_eq!(veils(&prims).len(), rings.len() + ground.len());
        // With no space indoors the south wall's shadow lies on the floor of the room.
        let open = plan(&geom, 1.2, look(sun, Some(-0.2)));
        assert!(holds(&veils_of(&open, on_face).0, 3.0, 1.0));
        assert!(
            holds(&veils_of(&open, ground_veil(&ShadingSettings::default())).0, 3.0, 2.0),
            "and on the ground under it"
        );
        assert!(!holds(&rings, 3.0, 1.0));
        // A roof over the terrace, which no walls close: a porch. It catches
        // the roof's shadow, from the first beam's to its far side.
        solids.push(solid_of(20, Roof, (6.0, 0.0, 2.5), (10.0, 4.0, 2.6)));
        let roofed = model(solids);
        let prims = plan(&roofed, 1.2, site(sun, house, None));
        let (rings, area) = veils_of(&prims, on_face);
        assert!(rings.len() == 1 && spans(&rings, (6.0, 2.4), (10.0, 4.0)) && (area - 6.4).abs() < 1e-6, "{rings:?}");
        assert!((veils_of(&prims, ground_veil(&ShadingSettings::default())).1 - 74.96).abs() < 1e-2);
        // The terrace a space indoors under that roof: no shadow on it, nor on the ground under it.
        let rooms = &indoors(vec![rect(0.0, 0.0, 6.0, 4.0), rect(6.0, 0.0, 10.0, 4.0)]);
        let prims = plan(&roofed, 1.2, site(sun, rooms, None));
        assert!(veils_of(&prims, on_face).0.is_empty());
        let (ground, area) = veils_of(&prims, ground_veil(&ShadingSettings::default()));
        assert!((area - 58.96).abs() < 1e-2 && !holds(&ground, 8.0, 2.0), "{area}");
        // Its roof hidden, the space is open to the sky (but for the 0.6 m
        // under the house's eave): the beams' shadows again.
        let skip = Skip { hidden: BTreeSet::from([ElementId(20)]), ..Default::default() };
        let prims = plan(&roofed, 1.2, site(sun, rooms, Some(&skip)));
        let (rings, area) = veils_of(&prims, on_face);
        assert!(spans(&rings, (6.6, 2.4), (10.0, 2.7)) && spans(&rings, (6.6, 3.4), (10.0, 3.7)), "{rings:?}");
        assert!(rings.len() == 2 && (area - 2.04).abs() < 1e-6, "{area}");
        assert!((veils_of(&prims, ground_veil(&ShadingSettings::default())).1 - 49.28).abs() < 1e-2);
        // A ceiling plan draws no shadows.
        let up = crate::plan::style_underlay(&geom, 1.2, true, site(sun, house, None), &Default::default());
        assert!(!up.is_empty() && veils(&up).is_empty());
    }

    /// A polygon holds another: its corners on the outline too, but not one
    /// that crosses it, lies beside it or fills a notch of it.
    #[test]
    fn a_polygon_holds_another() {
        let outer = rect(0.0, 0.0, 6.0, 4.0);
        assert!(encloses(&outer, &rect(1.0, 1.0, 2.0, 2.0)));
        assert!(encloses(&outer, &outer) && encloses(&outer, &rect(0.0, 0.0, 3.0, 4.0)));
        assert!(!encloses(&outer, &rect(5.0, 1.0, 7.0, 2.0)), "across a side");
        assert!(!encloses(&outer, &rect(6.0, 0.0, 10.0, 4.0)), "beside it");
        assert!(!encloses(&outer, &rect(-1.0, -1.0, 7.0, 5.0)), "around it");
        // A U: the rectangle in its notch has every corner on its outline.
        let u: Vec<DVec2> =
            [(0.0, 0.0), (3.0, 0.0), (3.0, 3.0), (2.0, 3.0), (2.0, 1.0), (1.0, 1.0), (1.0, 3.0), (0.0, 3.0)]
                .iter()
                .map(|&(x, y)| DVec2::new(x, y))
                .collect();
        assert!(!encloses(&u, &rect(1.0, 1.0, 2.0, 3.0)));
        assert!(encloses(&u, &rect(0.0, 0.0, 3.0, 1.0)) && !encloses(&u, &rect(0.5, 0.5, 2.5, 2.0)));
        assert!(!encloses(&outer, &[]));
    }

    /// A room 6 x 4 m under a roof with eaves of 0.6 m, cut by a section
    /// that looks north, and beyond it a wall under its own eave; the sun
    /// in the south-west, as high as it is far along each axis. The room's
    /// far wall is indoors: no shadow, though the roof and the west wall
    /// stand between it and the sun. The wall beyond is outdoors, under an
    /// eave or not: it keeps the eave's shadow, 0.6 m down from its top.
    /// An elevation casts on everything: the eaves' shadows on both fronts.
    #[test]
    fn a_section_casts_on_what_is_outdoors() {
        use Category::{Floor, Roof, Wall};
        let geom = model(vec![
            solid_of(1, Floor, (0.0, 0.0, -0.2), (6.0, 4.0, 0.0)),
            solid_of(2, Wall, (0.0, 0.0, 0.0), (6.0, 0.2, 3.0)),
            solid_of(3, Wall, (0.0, 3.8, 0.0), (6.0, 4.0, 3.0)),
            solid_of(4, Wall, (0.0, 0.2, 0.0), (0.2, 3.8, 3.0)),
            solid_of(5, Wall, (5.8, 0.2, 0.0), (6.0, 3.8, 3.0)),
            solid_of(6, Roof, (-0.6, -0.6, 3.0), (6.6, 4.6, 3.2)),
            solid_of(7, Wall, (8.0, 12.0, 0.0), (14.0, 12.2, 3.0)),
            solid_of(8, Roof, (7.4, 11.4, 3.0), (14.6, 12.8, 3.2)),
        ]);
        let sun = DVec3::new(-1.0, -1.0, 1.0).normalize();
        let room = vec![DVec2::ZERO, DVec2::new(6.0, 0.0), DVec2::new(6.0, 4.0), DVec2::new(0.0, 4.0)];
        let indoors = Indoors { storeys: vec![Storey { z: (0.0, f64::INFINITY), spaces: vec![room] }], failed: false };
        assert!(indoors.holds(DVec3::new(3.0, 3.75, 1.5)) && !indoors.holds(DVec3::new(11.0, 11.95, 1.5)));
        assert!(!indoors.holds(DVec3::new(3.0, 3.75, -0.5)), "below the storey");
        let catching = |catch| Look { shadows: Some(Shadows { ground: None, skip: None, catch }), ..look(sun, None) };
        let alpha = veil(VisualStyle::Shaded, sun.dot(-DVec3::Y), &ShadingSettings::default());
        let prims = section(&geom, 2.0, catching(Catch::Outdoors(&indoors)));
        let (rings, area) = veils_of(&prims, alpha);
        assert_eq!(rings.len(), 1, "{rings:?}");
        assert!(spans(&rings, (8.0, 2.4), (14.0, 3.0)) && (area - 3.6).abs() < 1e-6, "{rings:?}");
        assert_eq!(veils(&prims).len(), 1);
        // Catching as any view: the room's far wall is shadowed too.
        let (all, _) = veils_of(&section(&geom, 2.0, catching(Catch::All)), alpha);
        assert!(all.len() > 1 && spans(&all, (8.0, 2.4), (14.0, 3.0)), "{all:?}");
        assert!(all.iter().any(|r| r.iter().all(|p| p.x < 6.0 + 1e-6)), "{all:?}");
        // A space is indoors under a roof: with the house's roof gone, the
        // sun gets into the room (and the wall beyond keeps its eave's shadow).
        let mut open = geom.solids.clone();
        open.retain(|s| s.element != ElementId(6));
        let (rings, _) = veils_of(&section(&model(open), 2.0, catching(Catch::Outdoors(&indoors))), alpha);
        assert!(rings.len() > 1 && spans(&rings, (8.0, 2.4), (14.0, 3.0)), "{rings:?}");
        // What is indoors could not be found: no shadows at all.
        let lost = Indoors { storeys: Vec::new(), failed: true };
        assert!(veils(&section(&geom, 2.0, catching(Catch::Outdoors(&lost)))).is_empty());
        // Over a point: the roof over the room and the eave beyond the wall, nothing over the roof.
        let cover = Scene::covers(&geom, None, &|_| true, None).unwrap();
        assert!(cover.above(DVec3::new(3.0, 3.75, 1.5), None) && cover.above(DVec3::new(3.0, -0.3, 1.5), None));
        assert!(!cover.above(DVec3::new(3.0, 2.0, 3.3), None) && !cover.above(DVec3::new(7.0, 2.0, 1.5), None));
        assert!(!cover.above(DVec3::new(3.0, 2.0, 1.5), Some(ElementId(6))), "but for the roof itself");
        assert!(cover.above(DVec3::new(3.0, 2.0, -0.5), Some(ElementId(6))), "the floor is over the ground");
        // The south elevation: the eave's shadow on the house's front and on the wall beyond.
        let (rings, _) = veils_of(&south(&geom, catching(Catch::All)), alpha);
        assert!(spans(&rings, (0.0, 2.4), (6.0, 3.0)) && spans(&rings, (8.0, 2.4), (14.0, 3.0)), "{rings:?}");
    }

    /// Many polygons over one another are united a few at a time: the
    /// region is the one uniting them all at once gives, with what is left
    /// out of it, and the points count once.
    #[test]
    fn many_polygons_are_united_in_groups() {
        let mut seed = 0x9E37_79B9_7F4A_7C15u64;
        let mut rand = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            (seed >> 11) as f64 / (1u64 << 53) as f64
        };
        let plain: Vec<Vec<DVec2>> = (0..150)
            .map(|_| {
                let (lo, size) = (DVec2::new(rand() * 9.0, rand() * 9.0), DVec2::new(0.2 + rand(), 0.2 + rand()));
                vec![lo, lo + DVec2::new(size.x, 0.0), lo + size, lo + DVec2::new(0.0, size.y)]
            })
            .collect();
        assert!(plain.len() > 2 * UNION_GROUP);
        let polygon = |pts: &Vec<DVec2>| {
            let (lo, hi) = box2(pts.iter().copied());
            Polygon { pts: pts.clone(), convex: true, lo, hi }
        };
        let hole = vec![DVec2::new(3.0, 3.0), DVec2::new(5.5, 3.0), DVec2::new(5.5, 5.5), DVec2::new(3.0, 5.5)];
        let geom = model(Vec::new());
        let mut scene = Scene::new(&geom, DVec3::Z, None, &|_| true, None).unwrap();
        for less in [Vec::new(), vec![hole]] {
            let at_once = region::covered_except(&plain, &less, None, usize::MAX).unwrap();
            let spent = scene.points;
            let rings = scene.rings(plain.iter().map(polygon).collect(), less.iter().map(polygon).collect(), None);
            assert_eq!(scene.points - spent, 4 * (plain.len() + less.len()));
            let area = |rings: &[Vec<DVec2>]| rings.iter().map(|r| polygon::signed_area(r)).sum::<f64>();
            assert!((area(&rings) - area(&at_once)).abs() < 1e-9, "{} against {}", area(&rings), area(&at_once));
            assert!(area(&rings) > 30.0);
            for i in 0..80 {
                for j in 0..80 {
                    let p = DVec2::new((i as f64 + 0.37) * 0.14, (j as f64 + 0.61) * 0.14);
                    let inside =
                        plain.iter().any(|r| polygon::contains(r, p)) && !less.iter().any(|r| polygon::contains(r, p));
                    assert_eq!(rings.iter().filter(|r| polygon::contains(r, p)).count(), usize::from(inside), "{p}");
                }
            }
        }
        assert!(!scene.failed());
    }

    /// A point within a face: the mean of its corners, or of its largest
    /// triangle's when the face bends around that mean.
    #[test]
    fn a_point_within_a_face() {
        let square = [DVec3::ZERO, DVec3::X, DVec3::new(1.0, 1.0, 0.0), DVec3::Y];
        assert_eq!(point_within(&square, DVec3::Z), Some(DVec3::new(0.5, 0.5, 0.0)));
        // A U: the mean of its corners lies in its opening.
        let u: Vec<DVec3> =
            [(0.0, 0.0), (3.0, 0.0), (3.0, 3.0), (2.0, 3.0), (2.0, 1.0), (1.0, 1.0), (1.0, 3.0), (0.0, 3.0)]
                .iter()
                .map(|&(x, z)| DVec3::new(x, 5.0, z))
                .collect();
        let outline: Vec<DVec2> = u.iter().map(|p| DVec2::new(p.x, p.z)).collect();
        let mean = u.iter().sum::<DVec3>() / 8.0;
        assert!(!polygon::contains(&outline, DVec2::new(mean.x, mean.z)));
        let p = point_within(&u, -DVec3::Y).unwrap();
        assert!(polygon::contains(&outline, DVec2::new(p.x, p.z)) && (p.y - 5.0).abs() < 1e-12, "{p}");
        assert_eq!(point_within(&square[..2], DVec3::Z), None);
        assert_eq!(point_within(&[], DVec3::Z), None);
    }

    /// A slab under a box: the shadow on the slab's top is a veil that
    /// leaves the shaded style's shadow grey of the lit face, the 3D
    /// view's; the ground's shadow lies under the slab.
    #[test]
    fn a_shadow_on_a_face_shows_the_shadow_grey() {
        let slab = solid(1, DVec3::new(-5.0, -5.0, -0.2), DVec3::new(5.0, 5.0, 0.0));
        let geom = model(vec![slab, solid(2, DVec3::ZERO, DVec3::new(1.0, 1.0, 2.0))]);
        let sun = DVec3::new(-1.0, 0.0, 1.0).normalize();
        let prims = plan(&geom, 10.0, look(sun, Some(-0.2)));
        let lit = face_at(&prims, 1, DVec2::new(-3.0, -3.0));
        let grey = (VisualStyle::shaded_grey(sun.z) * 255.0).round();
        assert_eq!(lit, [grey as u8, grey as u8, grey as u8, 255]);
        let on_slab: Vec<_> = veils(&prims)
            .into_iter()
            .filter(|(_, a)| *a == veil(VisualStyle::Shaded, sun.z, &ShadingSettings::default()))
            .collect();
        assert_eq!(on_slab.len(), 1, "{:?}", veils(&prims));
        let (lo, hi) = bounds(&on_slab[0].0);
        assert!(lo.distance(DVec2::new(LIFT, 0.0)) < 1e-6 && hi.distance(DVec2::new(3.0, 1.0)) < 1e-6, "{lo} {hi}");
        let shown = grey * (1.0 - f64::from(on_slab[0].1) / 255.0);
        assert!((shown - VisualStyle::SHADED_SHADOW * 255.0).abs() <= 1.0, "{shown}");
        // After the slab's fill and before the box's faces, which cover its start.
        let at = |f: &dyn Fn(&Prim) -> bool| prims.iter().position(f).unwrap();
        let veil_at = at(&|p| matches!(p, Prim::Fill { pts, element: None, .. } if *pts == on_slab[0].0));
        let slab_at = at(&|p| matches!(p, Prim::Fill { element: Some(e), .. } if e.0 == 1));
        let box_at = at(&|p| matches!(p, Prim::Fill { element: Some(e), .. } if e.0 == 2));
        assert!(slab_at < veil_at && veil_at < box_at, "{slab_at} {veil_at} {box_at}");
        // The other styles: darker faces, each by its own veil; none in hidden line and wireframe.
        for (style, some) in [
            (VisualStyle::Textures, true),
            (VisualStyle::ConsistentColors, true),
            (VisualStyle::HiddenLine, false),
            (VisualStyle::Wireframe, false),
        ] {
            let found = veils(&plan(&geom, 10.0, Look { style, ..look(sun, Some(-0.2)) }));
            assert_eq!(!found.is_empty(), some, "{style:?}");
            assert!(
                found.iter().all(|(_, a)| *a == veil(style, sun.z, &ShadingSettings::default())
                    || *a == ground_veil(&ShadingSettings::default())),
                "{style:?}"
            );
            assert_eq!(veil(style, sun.z, &ShadingSettings::default()) > 0, some);
        }
    }

    /// A box 3 m high, 1 m in front of a wall, under a sun in the
    /// south-west as high as it is far along each axis, seen in the south
    /// elevation: its shadow on the wall is the pentagon the box's faces
    /// cast together, 3.5 m².
    #[test]
    fn a_box_casts_its_shadow_on_a_wall_in_an_elevation() {
        let wall = solid(1, DVec3::new(0.0, 1.0, 0.0), DVec3::new(10.0, 1.2, 5.0));
        let geom = model(vec![wall, solid(2, DVec3::new(4.0, -1.0, 0.0), DVec3::new(5.0, 0.0, 3.0))]);
        let sun = DVec3::new(-1.0, -1.0, 1.0).normalize();
        let prims = south(&geom, look(sun, Some(0.0)));
        let shadows = veils(&prims);
        assert_eq!(shadows.len(), 1, "{shadows:?}");
        let (ring, alpha) = &shadows[0];
        assert_eq!(*alpha, veil(VisualStyle::Shaded, sun.dot(-DVec3::Y), &ShadingSettings::default()));
        assert_eq!(ring.len(), 5, "{ring:?}");
        assert!((polygon::area(ring) - 3.5).abs() < 1e-3, "{}", polygon::area(ring));
        for corner in [(5.0, 0.0), (7.0, 0.0), (7.0, 1.0), (6.0, 2.0), (5.0, 2.0)] {
            let c = DVec2::new(corner.0, corner.1);
            assert!(ring.iter().any(|p| p.distance(c) < 1e-3), "{c} in {ring:?}");
        }
        // The veil belongs to no element: the wall is picked through it.
        let d = crate::Drawing2D { prims, ..Default::default() };
        assert_eq!(d.hit_test(DVec2::new(6.5, 0.5), 0.01), Some(ElementId(1)));
    }

    /// The conventional sun of façades in the south elevation (looking
    /// north): a box standing 1 to 2 m in front of a wall casts its shadow
    /// on the wall down and to the right of itself, each corner as far
    /// across and as far down as it stands in front of the wall.
    #[test]
    fn the_conventional_sun_drops_a_shadow_as_deep_as_the_caster_stands_out() {
        use cc_core::sun::{SunMode, SunSettings, SunSite};
        let wall = solid(1, DVec3::new(0.0, 1.0, 0.0), DVec3::new(10.0, 1.2, 5.0));
        let geom = model(vec![wall, solid(2, DVec3::new(4.0, -1.0, 0.0), DVec3::new(5.0, 0.0, 3.0))]);
        let settings = SunSettings { mode: SunMode::conventional(), shadows: true, ..Default::default() };
        let sun = settings.state(&SunSite::default(), Some(DVec2::Y)).direction;
        let shadows = veils(&south(&geom, look(sun, Some(0.0))));
        assert_eq!(shadows.len(), 1, "{shadows:?}");
        let ring = &shadows[0].0;
        // The top right corners: 2 m out at (5, 3) falls at (7, 1), 1 m out at (6, 2).
        for corner in [(5.0, 0.0), (7.0, 0.0), (7.0, 1.0), (6.0, 2.0), (5.0, 2.0)] {
            let c = DVec2::new(corner.0, corner.1);
            assert!(ring.iter().any(|p| p.distance(c) < 1e-3), "{c} in {ring:?}");
        }
        assert_eq!(ring.len(), 5, "{ring:?}");
    }

    /// A view's shading: its ambient light greys the faces turned away,
    /// its shadow intensity makes shadows from none to black.
    #[test]
    fn the_view_shading_sets_the_greys_and_the_shadows() {
        let wall = solid(1, DVec3::new(0.0, 1.0, 0.0), DVec3::new(10.0, 1.2, 5.0));
        let geom = model(vec![wall, solid(2, DVec3::new(4.0, -1.0, 0.0), DVec3::new(5.0, 0.0, 3.0))]);
        let away = |ambient: f64| {
            let shading = ShadingSettings { ambient, ..Default::default() };
            face_at(&south(&geom, Look { shading, ..look(DVec3::new(1.0, 1.0, 1.0), Some(0.0)) }), 1, DVec2::ONE)[0]
        };
        assert_eq!(away(0.6), 153);
        assert_eq!(away(0.8), 204);
        assert_eq!(away(1.0), 255);
        let sun = DVec3::new(-1.0, -1.0, 1.0).normalize();
        let lit = ShadingSettings::default().grey(sun.dot(-DVec3::Y));
        for style in [VisualStyle::Shaded, VisualStyle::Textures, VisualStyle::ConsistentColors] {
            let alpha = |shadow: f64| {
                let shading = ShadingSettings { shadow, ..Default::default() };
                let found = veils(&south(&geom, Look { style, shading, ..look(sun, Some(0.0)) }));
                found.iter().map(|(_, a)| *a).max().unwrap_or(0)
            };
            assert_eq!(alpha(0.0), 0, "{style:?}: no shadow");
            assert!(alpha(0.3) < alpha(ShadingSettings::DEFAULT_SHADOW), "{style:?}");
            assert!(alpha(ShadingSettings::DEFAULT_SHADOW) < alpha(1.0), "{style:?}");
            assert_eq!(alpha(1.0), 255, "{style:?}: black");
        }
        // The shaded style's shadow shows `1 - shadow` over a lit face.
        let shading = ShadingSettings { shadow: 0.3, ..Default::default() };
        let shown = lit * (1.0 - f64::from(veil(VisualStyle::Shaded, sun.dot(-DVec3::Y), &shading)) / 255.0);
        assert!((shown - 0.7).abs() < 0.01, "{shown}");
        assert_eq!(ground_veil(&ShadingSettings { shadow: 0.0, ..Default::default() }), 0);
        assert_eq!(ground_veil(&ShadingSettings { shadow: 1.0, ..Default::default() }), 255);
    }

    /// In the textures style a shadow lies over the face's color and under
    /// its surface pattern, whose lines stay drawn over it.
    #[test]
    fn surface_patterns_show_over_shadows() {
        let mut wall = solid(1, DVec3::new(0.0, 1.0, 0.0), DVec3::new(10.0, 1.2, 5.0));
        wall.surface.foreground = cc_core::Pattern::Diagonal;
        let geom = model(vec![wall, solid(2, DVec3::new(4.0, -1.0, 0.0), DVec3::new(5.0, 0.0, 3.0))]);
        let sun = DVec3::new(-1.0, -1.0, 1.0).normalize();
        let prims = south(&geom, Look { style: VisualStyle::Textures, ..look(sun, Some(0.0)) });
        let shadow = veils(&prims);
        assert_eq!(shadow.len(), 1);
        let veil_at =
            prims.iter().position(|p| matches!(p, Prim::Fill { pts, element: None, .. } if *pts == shadow[0].0));
        let veil_at = veil_at.unwrap();
        let fill_at = prims.iter().position(|p| matches!(p, Prim::Fill { element: Some(e), .. } if e.0 == 1)).unwrap();
        let pattern: Vec<usize> = (0..prims.len())
            .filter(|i| matches!(&prims[*i], Prim::Line { element: Some(e), pen: Some(_), .. } if e.0 == 1))
            .collect();
        assert!(!pattern.is_empty());
        assert!(fill_at < veil_at && pattern.iter().all(|i| *i > veil_at), "{fill_at} {veil_at} {pattern:?}");
    }

    /// Faces turned away from the sun keep their shade and get no shadow;
    /// no sun, a sun below the horizon and shadows off cast none.
    #[test]
    fn no_shadows_without_a_sun_that_casts_them() {
        let wall = solid(1, DVec3::new(0.0, 1.0, 0.0), DVec3::new(10.0, 1.2, 5.0));
        let geom = model(vec![wall, solid(2, DVec3::new(4.0, -1.0, 0.0), DVec3::new(5.0, 0.0, 3.0))]);
        // From the north: the south faces the elevation shows are turned away.
        let north = south(&geom, look(DVec3::new(1.0, 1.0, 1.0), Some(0.0)));
        assert!(veils(&north).is_empty());
        let ambient = (VisualStyle::SHADED_AMBIENT * 255.0).round() as u8;
        assert_eq!(face_at(&north, 1, DVec2::new(1.0, 1.0)), [ambient, ambient, ambient, 255]);
        let sun = DVec3::new(-1.0, -1.0, 1.0).normalize();
        let cases = [
            Look { shadows: None, ..look(sun, Some(0.0)) },
            Look { light: None, ..look(sun, Some(0.0)) },
            Look { light: Some(DVec3::ZERO), ..look(sun, Some(0.0)) },
        ];
        for case in cases {
            assert!(veils(&south(&geom, case)).is_empty(), "{case:?}");
            assert!(veils(&plan(&geom, 10.0, case)).is_empty(), "{case:?}");
        }
        assert_eq!(veil(VisualStyle::Shaded, 0.0, &ShadingSettings::default()), 0);
        assert_eq!(veil(VisualStyle::Shaded, -0.5, &ShadingSettings::default()), 0);
        assert_eq!(veil(VisualStyle::Shaded, f64::NAN, &ShadingSettings::default()), 0);
    }

    /// What the view hides neither casts nor catches; what it makes
    /// see-through, and glass, cast nothing.
    #[test]
    fn hidden_and_see_through_elements_cast_no_shadow() {
        let wall = solid(1, DVec3::new(0.0, 1.0, 0.0), DVec3::new(10.0, 1.2, 5.0));
        let front = solid(2, DVec3::new(4.0, -1.0, 0.0), DVec3::new(5.0, 0.0, 3.0));
        let geom = model(vec![wall.clone(), front.clone()]);
        let sun = DVec3::new(-1.0, -1.0, 1.0);
        let with = |skip: &Skip| {
            let base = look(sun, Some(0.0));
            let shadows = Some(Shadows { ground: Some(0.0), skip: Some(skip), catch: Catch::All });
            veils(&south(&geom, Look { shadows, ..base })).len()
        };
        assert_eq!(with(&Skip::default()), 1);
        let only = |id: u64| BTreeSet::from([ElementId(id)]);
        assert_eq!(with(&Skip { hidden: only(2), ..Default::default() }), 0, "the caster is hidden");
        assert_eq!(with(&Skip { clear: only(2), ..Default::default() }), 0, "the caster is see-through");
        assert_eq!(with(&Skip { hidden: only(1), ..Default::default() }), 0, "the face that catches is hidden");
        let glass = Solid { transparency: 0.6, ..front };
        assert!(veils(&south(&model(vec![wall, glass]), look(sun, Some(0.0)))).is_empty());
    }

    /// Past a limit there are no shadows at all, and nothing panics: too
    /// many faces make no scene, too many points stop it.
    #[test]
    fn past_a_limit_there_are_no_shadows() {
        let mut many = Solid::new(ElementId(1), Category::Wall, [200; 3]);
        let face = cc_geom::Face::new(vec![DVec3::ZERO, DVec3::X, DVec3::Y], DVec3::Z);
        many.faces = vec![face; MAX_CASTERS + 1];
        let geom = model(vec![many]);
        assert!(Scene::new(&geom, DVec3::Z, None, &|_| true, None).is_none());
        assert!(Scene::new(&geom, DVec3::new(f64::NAN, 0.0, 1.0), None, &|_| true, None).is_none());
        assert!(Scene::new(&geom, DVec3::ZERO, None, &|_| true, None).is_none());

        let slab = solid(1, DVec3::new(-5.0, -5.0, -0.2), DVec3::new(5.0, 5.0, 0.0));
        let geom = model(vec![slab, solid(2, DVec3::ZERO, DVec3::new(1.0, 1.0, 2.0))]);
        let top: Vec<DVec3> = geom.solids[0].faces[0].verts.clone();
        let mut scene = Scene::new(&geom, DVec3::new(-1.0, 0.0, 1.0), None, &|_| true, None).unwrap();
        assert_eq!(scene.on_face(&top, DVec3::Z, (0, 0), None).len(), 1);
        assert!(!scene.failed());
        assert!(!scene.spend(MAX_UNION_POINTS + 1));
        assert!(scene.failed());
        assert!(scene.on_face(&top, DVec3::Z, (0, 0), None).is_empty());
        assert!(scene.on_ground(-0.2, geom.bounds().unwrap(), None).is_empty());
        // Too many casters looked at over the faces of a view.
        let mut scene = Scene::new(&geom, DVec3::new(-1.0, 0.0, 1.0), None, &|_| true, None).unwrap();
        scene.looks = MAX_LOOKS;
        assert!(scene.on_face(&top, DVec3::Z, (0, 0), None).is_empty() && scene.failed());
        // Faces that are no polygons, and an empty model, catch and cast nothing.
        let mut scene = Scene::new(&geom, DVec3::new(-1.0, 0.0, 1.0), None, &|_| true, None).unwrap();
        assert!(scene.on_face(&top[..2], DVec3::Z, (0, 0), None).is_empty());
        let bad = [DVec3::ZERO, DVec3::X, DVec3::new(f64::NAN, 0.0, 0.0)];
        assert!(scene.on_face(&bad, DVec3::Z, (9, 9), None).is_empty());
        let empty = model(Vec::new());
        let mut scene = Scene::new(&empty, DVec3::Z, None, &|_| true, None).unwrap();
        assert!(scene.on_face(&top, DVec3::Z, (0, 0), None).is_empty());
        // A limit passed finding what is indoors of a face is passed by
        // the view: no shadows on the face, on the ground or anywhere after.
        let mut solids = geom.solids.clone();
        solids.push(solid_of(3, Category::Floor, (-5.0, -5.0, 5.0), (5.0, 5.0, 5.2)));
        let geom = model(solids);
        let sky = || Scene::covers(&geom, None, &|_| true, None).unwrap();
        let sunny = || Scene::new(&geom, DVec3::new(-1.0, 0.0, 1.0), None, &|_| true, None).unwrap();
        assert_eq!(sky().units.len(), 1, "the floor alone covers");
        // The west half of the slab is indoors: the floor's shadow lies on the east half.
        let west = Indoors {
            storeys: vec![Storey {
                z: (-10.0, f64::INFINITY),
                spaces: vec![vec![
                    DVec2::new(-5.0, -5.0),
                    DVec2::new(0.0, -5.0),
                    DVec2::new(0.0, 5.0),
                    DVec2::new(-5.0, 5.0),
                ]],
            }],
            failed: false,
        };
        let (mut scene, mut cover) = (sunny(), sky());
        let rings = scene.on_face(&top, DVec3::Z, (0, 0), Some((&mut cover, &west, ElementId(1))));
        assert_eq!(rings.len(), 1);
        let (lo, hi) = bounds(&rings[0].iter().map(|p| p.truncate()).collect::<Vec<_>>());
        assert!(lo.distance(DVec2::new(0.0, -5.0)) < 1e-6 && hi.distance(DVec2::new(5.0, 5.0)) < 1e-6, "{lo} {hi}");
        let rings = scene.on_ground(-0.2, geom.bounds().unwrap(), Some((&mut cover, &west)));
        let (lo, hi) = bounds(&rings.iter().flatten().map(|p| p.truncate()).collect::<Vec<_>>());
        assert!(
            rings.len() == 1 && lo.distance(DVec2::new(0.0, -5.0)) < 1e-6 && (hi.x - 10.4).abs() < 1e-6,
            "{lo} {hi}"
        );
        assert!(!scene.failed() && !cover.failed());
        cover.looks = MAX_LOOKS;
        assert!(scene.on_face(&top, DVec3::Z, (0, 0), Some((&mut cover, &west, ElementId(1)))).is_empty());
        assert!(scene.failed() && cover.failed());
        let (mut scene, mut cover) = (sunny(), sky());
        cover.failed = true;
        assert!(scene.on_ground(-0.2, geom.bounds().unwrap(), Some((&mut cover, &west))).is_empty() && scene.failed());
    }
}
