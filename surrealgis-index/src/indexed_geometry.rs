use rstar::{PointDistance, RTreeObject, AABB};
use surrealgis_core::bbox::BoundingBox;

/// Wrapper around a geometry ID and its bounding box envelope for use in an R*-tree.
///
/// `PartialEq` compares by `id` only, which is required for rstar's `remove` to work
/// correctly when locating an entry by ID.
#[derive(Debug, Clone)]
pub struct IndexedGeometry {
    id: usize,
    envelope: AABB<[f64; 2]>,
}

impl IndexedGeometry {
    /// Create a new indexed geometry from an ID and bounding box.
    pub fn new(id: usize, bbox: &BoundingBox) -> Self {
        let envelope = AABB::from_corners(
            [bbox.min_x, bbox.min_y],
            [bbox.max_x, bbox.max_y],
        );
        Self { id, envelope }
    }

    /// Returns the geometry ID.
    pub fn id(&self) -> usize {
        self.id
    }
}

impl PartialEq for IndexedGeometry {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl RTreeObject for IndexedGeometry {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.envelope
    }
}

impl PointDistance for IndexedGeometry {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        self.envelope.distance_2(point)
    }
}
