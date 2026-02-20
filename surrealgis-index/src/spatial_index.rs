use surrealgis_core::bbox::BoundingBox;
use surrealgis_core::coordinate::Coordinate;
use surrealgis_core::geometry::SurrealGeometry;

/// Errors that can occur during spatial index operations.
#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum IndexError {
    #[error("Index error: {0}")]
    IndexError(String),
    #[error("Geometry has no bounding box")]
    NoBoundingBox,
}

/// Repository-pattern trait for spatial indexing.
///
/// Implementations provide efficient spatial queries over a collection
/// of geometries identified by `usize` IDs.
pub trait SpatialIndex: Sized {
    /// Insert a geometry by its ID. The bounding box is extracted automatically.
    fn insert(&mut self, id: usize, geom: &SurrealGeometry) -> Result<(), IndexError>;

    /// Bulk load geometries using the STR packing algorithm for better query performance.
    fn bulk_load(entries: Vec<(usize, SurrealGeometry)>) -> Result<Self, IndexError>;

    /// Query all geometry IDs whose bounding box intersects the given bounding box.
    fn query_bbox(&self, bbox: &BoundingBox) -> Vec<usize>;

    /// Find the k nearest geometries to a point, returning (id, distance) pairs.
    fn query_nearest(&self, point: &Coordinate, k: usize) -> Vec<(usize, f64)>;

    /// Find all geometry IDs within a given Euclidean distance of a point.
    ///
    /// Note: `distance` is the actual distance, NOT squared.
    fn query_within_distance(&self, point: &Coordinate, distance: f64) -> Vec<usize>;

    /// Remove a geometry by its ID. Returns true if it was found and removed.
    fn remove(&mut self, id: usize) -> bool;

    /// Number of entries in the index.
    fn len(&self) -> usize;

    /// Whether the index contains no entries.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
