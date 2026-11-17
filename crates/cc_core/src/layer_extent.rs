//! Per-layer vertical extents of compound walls: a layer of a wall type may
//! start higher or lower than the wall and stop short of its top, so an
//! insulation layer can start above the ground while the structure core
//! runs deeper into the foundation.
//!
//! Each [`crate::Layer`] of a wall type carries a `base_offset` (meters
//! from the wall's bottom at each point along it: positive starts higher,
//! negative runs below) and an optional `height` (from the layer's own
//! bottom; `None` reaches the wall's top). The wall's bottom and top are its
//! nominal base and top, its elevation profile's chains, or the top it is
//! attached to. A layer never rises above the wall's top: its top is the
//! lower of its own and the wall's, and where that leaves it no height it is
//! simply absent there. Floors and roofs share [`crate::Layer`] but take no
//! extents (repair clears them).
//!
//! The structure core still defines the wall for everything topological:
//! joins, rooms, the openings it hosts and the attachment of its top work
//! with the wall as a whole (its nominal extents, reaching down to its
//! lowest layer for joins).

use serde::{Deserialize, Serialize};

/// The furthest a layer's bottom lies above or below its wall's bottom, meters.
pub const MAX_OFFSET: f64 = 100.0;

/// The least height a layer may be given, meters.
pub const MIN_HEIGHT: f64 = 0.001;

/// The greatest height a layer may be given, meters.
pub const MAX_HEIGHT: f64 = 1000.0;

/// Below this a layer's span is no span at all, meters.
pub const EPS: f64 = 1e-9;

/// A layer's own vertical extent, relative to its wall (see the module).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Extent {
    /// Its bottom above (below, negative) the wall's bottom, meters.
    #[serde(default)]
    pub base_offset: f64,
    /// Its height from its own bottom (`None`: up to the wall's top).
    #[serde(default)]
    pub height: Option<f64>,
}

impl Extent {
    /// The extent a layer carries.
    pub fn of(layer: &crate::Layer) -> Self {
        Self { base_offset: layer.base_offset, height: layer.height }
    }

    /// Whether the layer spans the whole wall, as layers did before extents.
    pub fn is_default(&self) -> bool {
        self.base_offset == 0.0 && self.height.is_none()
    }

    /// Whether its values lie within the module's bounds.
    pub fn is_valid(&self) -> bool {
        self.base_offset.is_finite()
            && self.base_offset.abs() <= MAX_OFFSET
            && self.height.is_none_or(|h| h.is_finite() && (MIN_HEIGHT..=MAX_HEIGHT).contains(&h))
    }

    /// The layer's bottom and top where the wall runs from `bottom` to
    /// `top`: its own, its top no higher than the wall's; `None` where that
    /// leaves it no height.
    pub fn range(&self, bottom: f64, top: f64) -> Option<(f64, f64)> {
        let lo = bottom + self.base_offset;
        let hi = self.height.map_or(top, |h| (lo + h).min(top));
        (hi > lo + EPS).then_some((lo, hi))
    }
}

/// Why a layer's extent is refused, for the commands' messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtentError {
    /// `base_offset` is not a number or lies past [`MAX_OFFSET`].
    Offset,
    /// `height` is not a number or lies outside [`MIN_HEIGHT`]..=[`MAX_HEIGHT`].
    Height,
}

impl std::fmt::Display for ExtentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtentError::Offset => write!(f, "base_offset must be a number within ±{MAX_OFFSET} m"),
            ExtentError::Height => {
                write!(f, "height must be null or a number from {MIN_HEIGHT} to {MAX_HEIGHT} m")
            }
        }
    }
}

/// The first layer (its index) whose extent is out of bounds, and why.
pub fn check(layers: &[crate::Layer]) -> Result<(), (usize, ExtentError)> {
    for (i, l) in layers.iter().enumerate() {
        if !(l.base_offset.is_finite() && l.base_offset.abs() <= MAX_OFFSET) {
            return Err((i, ExtentError::Offset));
        }
        if !l.height.is_none_or(|h| h.is_finite() && (MIN_HEIGHT..=MAX_HEIGHT).contains(&h)) {
            return Err((i, ExtentError::Height));
        }
    }
    Ok(())
}

/// Whether any layer has an extent of its own.
pub fn any(layers: &[crate::Layer]) -> bool {
    layers.iter().any(|l| !Extent::of(l).is_default())
}

/// Reset the extent values of a wall layer that lie out of bounds; returns
/// the number of values reset.
pub fn repair(layer: &mut crate::Layer) -> usize {
    let mut n = 0;
    if !(layer.base_offset.is_finite() && layer.base_offset.abs() <= MAX_OFFSET) {
        layer.base_offset = 0.0;
        n += 1;
    }
    if layer.height.is_some_and(|h| !(h.is_finite() && (MIN_HEIGHT..=MAX_HEIGHT).contains(&h))) {
        layer.height = None;
        n += 1;
    }
    n
}

/// Clear the extent of a layer of a type that takes none (floors, roofs);
/// returns the number of values cleared.
pub fn clear(layer: &mut crate::Layer) -> usize {
    let n = usize::from(layer.base_offset != 0.0) + usize::from(layer.height.is_some());
    layer.base_offset = 0.0;
    layer.height = None;
    n
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Layer;

    fn layer(base_offset: f64, height: Option<f64>) -> Layer {
        Layer { material: "m".into(), thickness: 0.1, base_offset, height, ..Default::default() }
    }

    #[test]
    fn a_layer_stops_at_the_wall_top() {
        let e = Extent { base_offset: 1.0, height: None };
        assert_eq!(e.range(0.0, 3.0), Some((1.0, 3.0)));
        let e = Extent { base_offset: -0.5, height: Some(1.0) };
        assert_eq!(e.range(0.0, 3.0), Some((-0.5, 0.5)));
        // Taller than the wall: clamped to its top.
        let e = Extent { base_offset: 1.0, height: Some(9.0) };
        assert_eq!(e.range(0.0, 3.0), Some((1.0, 3.0)));
        // Starting above the top: absent there.
        let e = Extent { base_offset: 3.5, height: None };
        assert_eq!(e.range(0.0, 3.0), None);
        assert!(Extent::default().is_default());
    }

    #[test]
    fn hostile_values_are_reset() {
        let mut bad = [layer(f64::NAN, Some(f64::INFINITY)), layer(1e9, Some(-1.0)), layer(0.5, Some(1e6))];
        assert_eq!(check(&bad), Err((0, ExtentError::Offset)));
        let n: usize = bad.iter_mut().map(repair).sum();
        assert_eq!(n, 5);
        assert!(check(&bad).is_ok());
        assert_eq!(bad[2].base_offset, 0.5, "a value in bounds stays");
        assert!(bad.iter().all(|l| l.height.is_none()));
        assert_eq!(check(&[layer(0.0, Some(0.0))]), Err((0, ExtentError::Height)));
        let mut floor = layer(0.2, Some(1.0));
        assert_eq!(clear(&mut floor), 2);
        assert!(!any(&[floor]));
    }
}
