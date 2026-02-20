use rstar::{PointDistance, RTree, AABB};
use surrealgis_core::bbox::BoundingBox;
use surrealgis_core::coordinate::Coordinate;
use surrealgis_core::geometry::SurrealGeometry;

use crate::indexed_geometry::IndexedGeometry;
use crate::spatial_index::{IndexError, SpatialIndex};

/// R*-tree backed spatial index.
///
/// Uses the `rstar` crate's R*-tree implementation for efficient spatial queries.
/// Geometries are stored as bounding box envelopes keyed by `usize` IDs.
pub struct RTreeSpatialIndex {
    tree: RTree<IndexedGeometry>,
}

impl RTreeSpatialIndex {
    /// Create a new empty spatial index.
    pub fn new() -> Self {
        Self {
            tree: RTree::new(),
        }
    }
}

impl Default for RTreeSpatialIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl SpatialIndex for RTreeSpatialIndex {
    fn insert(&mut self, id: usize, geom: &SurrealGeometry) -> Result<(), IndexError> {
        let bbox = geom.bbox().ok_or(IndexError::NoBoundingBox)?;
        let indexed = IndexedGeometry::new(id, bbox);
        self.tree.insert(indexed);
        Ok(())
    }

    fn bulk_load(entries: Vec<(usize, SurrealGeometry)>) -> Result<Self, IndexError> {
        let mut indexed = Vec::with_capacity(entries.len());
        for (id, geom) in &entries {
            let bbox = geom.bbox().ok_or(IndexError::NoBoundingBox)?;
            indexed.push(IndexedGeometry::new(*id, bbox));
        }

        Ok(Self {
            tree: RTree::bulk_load(indexed),
        })
    }

    fn query_bbox(&self, bbox: &BoundingBox) -> Vec<usize> {
        let envelope = AABB::from_corners(
            [bbox.min_x, bbox.min_y],
            [bbox.max_x, bbox.max_y],
        );
        self.tree
            .locate_in_envelope_intersecting(&envelope)
            .map(|entry| entry.id())
            .collect()
    }

    fn query_nearest(&self, point: &Coordinate, k: usize) -> Vec<(usize, f64)> {
        let pt = [point.x(), point.y()];
        self.tree
            .nearest_neighbor_iter(&pt)
            .take(k)
            .map(|entry| {
                let dist_sq = entry.distance_2(&pt);
                (entry.id(), dist_sq.sqrt())
            })
            .collect()
    }

    fn query_within_distance(&self, point: &Coordinate, distance: f64) -> Vec<usize> {
        let pt = [point.x(), point.y()];
        // IMPORTANT: rstar's locate_within_distance takes SQUARED distance
        let distance_sq = distance * distance;
        self.tree
            .locate_within_distance(pt, distance_sq)
            .map(|entry| entry.id())
            .collect()
    }

    fn remove(&mut self, id: usize) -> bool {
        // Find the entry with the given ID by iterating over the tree,
        // then remove it. We need to clone the entry because rstar::remove
        // requires an owned reference for comparison.
        let entry = self.tree.iter().find(|e| e.id() == id).cloned();
        match entry {
            Some(e) => self.tree.remove(&e).is_some(),
            None => false,
        }
    }

    fn len(&self) -> usize {
        self.tree.size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial_index::SpatialIndex;
    use surrealgis_core::coordinate::Coordinate;
    use surrealgis_core::geometry::SurrealGeometry;
    use surrealgis_core::srid::Srid;

    fn make_point(x: f64, y: f64) -> SurrealGeometry {
        SurrealGeometry::point(x, y, Srid::WGS84).unwrap()
    }

    fn make_coord(x: f64, y: f64) -> Coordinate {
        Coordinate::new(x, y).unwrap()
    }

    fn make_bbox(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> BoundingBox {
        BoundingBox::new(min_x, min_y, max_x, max_y).unwrap()
    }

    fn make_polygon_geom(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> SurrealGeometry {
        let exterior = vec![
            Coordinate::new(min_x, min_y).unwrap(),
            Coordinate::new(max_x, min_y).unwrap(),
            Coordinate::new(max_x, max_y).unwrap(),
            Coordinate::new(min_x, min_y).unwrap(),
        ];
        SurrealGeometry::polygon(exterior, vec![], Srid::WGS84).unwrap()
    }

    // ── Basic operations ──────────────────────────────────────────

    #[test]
    fn insert_single_point_and_query() {
        let mut index = RTreeSpatialIndex::new();
        index.insert(1, &make_point(5.0, 5.0)).unwrap();

        let results = index.query_bbox(&make_bbox(0.0, 0.0, 10.0, 10.0));
        assert_eq!(results, vec![1]);
    }

    #[test]
    fn insert_and_query_multiple_overlapping_bboxes() {
        let mut index = RTreeSpatialIndex::new();
        index.insert(1, &make_polygon_geom(0.0, 0.0, 5.0, 5.0)).unwrap();
        index.insert(2, &make_polygon_geom(3.0, 3.0, 8.0, 8.0)).unwrap();
        index.insert(3, &make_polygon_geom(6.0, 6.0, 10.0, 10.0)).unwrap();

        let mut results = index.query_bbox(&make_bbox(4.0, 4.0, 7.0, 7.0));
        results.sort();
        assert_eq!(results, vec![1, 2, 3]);
    }

    #[test]
    fn query_non_intersecting_bbox_returns_empty() {
        let mut index = RTreeSpatialIndex::new();
        index.insert(1, &make_polygon_geom(0.0, 0.0, 2.0, 2.0)).unwrap();
        index.insert(2, &make_polygon_geom(3.0, 3.0, 5.0, 5.0)).unwrap();

        let results = index.query_bbox(&make_bbox(10.0, 10.0, 20.0, 20.0));
        assert!(results.is_empty());
    }

    #[test]
    fn new_index_is_empty() {
        let index = RTreeSpatialIndex::new();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn len_after_inserts() {
        let mut index = RTreeSpatialIndex::new();
        index.insert(1, &make_point(0.0, 0.0)).unwrap();
        index.insert(2, &make_point(1.0, 1.0)).unwrap();
        assert_eq!(index.len(), 2);
        assert!(!index.is_empty());
    }

    // ── Bulk load ─────────────────────────────────────────────────

    #[test]
    fn bulk_load_100_points_all_queryable() {
        let entries: Vec<(usize, SurrealGeometry)> = (0..100)
            .map(|i| (i, make_point(i as f64, i as f64)))
            .collect();

        let index = RTreeSpatialIndex::bulk_load(entries).unwrap();
        assert_eq!(index.len(), 100);

        let results = index.query_bbox(&make_bbox(0.0, 0.0, 99.0, 99.0));
        assert_eq!(results.len(), 100);
    }

    #[test]
    fn bulk_load_vs_sequential_same_results() {
        let points: Vec<(usize, SurrealGeometry)> = vec![
            (0, make_point(1.0, 1.0)),
            (1, make_point(5.0, 5.0)),
            (2, make_point(9.0, 9.0)),
        ];

        let bulk_index = RTreeSpatialIndex::bulk_load(points.clone()).unwrap();

        let mut seq_index = RTreeSpatialIndex::new();
        for (id, geom) in &points {
            seq_index.insert(*id, geom).unwrap();
        }

        let query = make_bbox(0.0, 0.0, 10.0, 10.0);
        let mut bulk_results = bulk_index.query_bbox(&query);
        let mut seq_results = seq_index.query_bbox(&query);
        bulk_results.sort();
        seq_results.sort();
        assert_eq!(bulk_results, seq_results);
    }

    #[test]
    fn bulk_load_empty_creates_empty_index() {
        let index = RTreeSpatialIndex::bulk_load(vec![]).unwrap();
        assert!(index.is_empty());
    }

    // ── k-NN ──────────────────────────────────────────────────────

    #[test]
    fn knn_three_points_known_distances() {
        let entries = vec![
            (0, make_point(0.0, 0.0)),
            (1, make_point(3.0, 0.0)),
            (2, make_point(5.0, 0.0)),
        ];
        let index = RTreeSpatialIndex::bulk_load(entries).unwrap();
        let origin = make_coord(0.0, 0.0);

        let nearest_1 = index.query_nearest(&origin, 1);
        assert_eq!(nearest_1.len(), 1);
        assert_eq!(nearest_1[0].0, 0);
        assert!((nearest_1[0].1 - 0.0).abs() < 1e-10);

        let nearest_2 = index.query_nearest(&origin, 2);
        assert_eq!(nearest_2.len(), 2);
        assert_eq!(nearest_2[0].0, 0);
        assert_eq!(nearest_2[1].0, 1);
        assert!((nearest_2[1].1 - 3.0).abs() < 1e-10);

        let nearest_3 = index.query_nearest(&origin, 3);
        assert_eq!(nearest_3.len(), 3);
        assert_eq!(nearest_3[2].0, 2);
        assert!((nearest_3[2].1 - 5.0).abs() < 1e-10);
    }

    #[test]
    fn knn_k_larger_than_index_returns_all() {
        let entries = vec![
            (0, make_point(0.0, 0.0)),
            (1, make_point(1.0, 1.0)),
        ];
        let index = RTreeSpatialIndex::bulk_load(entries).unwrap();

        let results = index.query_nearest(&make_coord(0.0, 0.0), 100);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn knn_empty_index_returns_empty() {
        let index = RTreeSpatialIndex::new();
        let results = index.query_nearest(&make_coord(0.0, 0.0), 5);
        assert!(results.is_empty());
    }

    #[test]
    fn knn_returns_correct_euclidean_distance() {
        let entries = vec![
            (0, make_point(3.0, 4.0)), // dist from origin = 5.0
        ];
        let index = RTreeSpatialIndex::bulk_load(entries).unwrap();

        let results = index.query_nearest(&make_coord(0.0, 0.0), 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, 0);
        assert!((results[0].1 - 5.0).abs() < 1e-10);
    }

    #[test]
    fn knn_returns_in_distance_order() {
        let entries = vec![
            (0, make_point(10.0, 0.0)),
            (1, make_point(1.0, 0.0)),
            (2, make_point(5.0, 0.0)),
        ];
        let index = RTreeSpatialIndex::bulk_load(entries).unwrap();

        let results = index.query_nearest(&make_coord(0.0, 0.0), 3);
        assert_eq!(results[0].0, 1); // closest: dist 1
        assert_eq!(results[1].0, 2); // middle: dist 5
        assert_eq!(results[2].0, 0); // farthest: dist 10
    }

    // ── Within distance ───────────────────────────────────────────

    #[test]
    fn within_distance_known_points() {
        let entries = vec![
            (0, make_point(0.0, 0.0)),  // dist 0
            (1, make_point(2.0, 0.0)),  // dist 2
            (2, make_point(4.0, 0.0)),  // dist 4
            (3, make_point(6.0, 0.0)),  // dist 6
        ];
        let index = RTreeSpatialIndex::bulk_load(entries).unwrap();

        let mut results = index.query_within_distance(&make_coord(0.0, 0.0), 3.0);
        results.sort();
        assert_eq!(results, vec![0, 1]);
    }

    #[test]
    fn within_distance_boundary_inclusion() {
        let entries = vec![
            (0, make_point(3.0, 0.0)),  // dist = exactly 3.0
        ];
        let index = RTreeSpatialIndex::bulk_load(entries).unwrap();

        let results = index.query_within_distance(&make_coord(0.0, 0.0), 3.0);
        assert_eq!(results, vec![0]);
    }

    #[test]
    fn within_distance_squared_gotcha_handled() {
        // If we forgot to square the distance, distance=5.0 would be treated
        // as distance_sq=5.0 (actual distance ~2.236), missing the point at (3,0).
        let entries = vec![
            (0, make_point(3.0, 0.0)),  // dist = 3.0
            (1, make_point(4.0, 0.0)),  // dist = 4.0
        ];
        let index = RTreeSpatialIndex::bulk_load(entries).unwrap();

        let mut results = index.query_within_distance(&make_coord(0.0, 0.0), 5.0);
        results.sort();
        assert_eq!(results, vec![0, 1]);
    }

    #[test]
    fn within_distance_empty_index() {
        let index = RTreeSpatialIndex::new();
        let results = index.query_within_distance(&make_coord(0.0, 0.0), 100.0);
        assert!(results.is_empty());
    }

    #[test]
    fn within_distance_none_in_range() {
        let entries = vec![
            (0, make_point(10.0, 10.0)),
        ];
        let index = RTreeSpatialIndex::bulk_load(entries).unwrap();

        let results = index.query_within_distance(&make_coord(0.0, 0.0), 1.0);
        assert!(results.is_empty());
    }

    #[test]
    fn within_distance_diagonal() {
        // Point at (3, 4) is distance 5 from origin
        let entries = vec![
            (0, make_point(3.0, 4.0)),
        ];
        let index = RTreeSpatialIndex::bulk_load(entries).unwrap();

        let results = index.query_within_distance(&make_coord(0.0, 0.0), 4.9);
        assert!(results.is_empty());

        let results = index.query_within_distance(&make_coord(0.0, 0.0), 5.1);
        assert_eq!(results, vec![0]);
    }

    #[test]
    fn within_distance_with_bbox_geometry() {
        // Polygon bbox at (8,8)-(12,12) - nearest corner to origin is (8,8), dist ~11.31
        let entries = vec![
            (0, make_polygon_geom(8.0, 8.0, 12.0, 12.0)),
        ];
        let index = RTreeSpatialIndex::bulk_load(entries).unwrap();

        let results = index.query_within_distance(&make_coord(0.0, 0.0), 11.0);
        assert!(results.is_empty());

        let results = index.query_within_distance(&make_coord(0.0, 0.0), 12.0);
        assert_eq!(results, vec![0]);
    }

    // ── Remove ────────────────────────────────────────────────────

    #[test]
    fn remove_existing_entry() {
        let mut index = RTreeSpatialIndex::new();
        index.insert(0, &make_point(1.0, 1.0)).unwrap();
        index.insert(1, &make_point(5.0, 5.0)).unwrap();
        assert_eq!(index.len(), 2);

        let removed = index.remove(0);
        assert!(removed);
        assert_eq!(index.len(), 1);

        // Verify removed entry is gone
        let results = index.query_bbox(&make_bbox(0.0, 0.0, 2.0, 2.0));
        assert!(results.is_empty());

        // Verify other entry is still present
        let results = index.query_bbox(&make_bbox(4.0, 4.0, 6.0, 6.0));
        assert_eq!(results, vec![1]);
    }

    #[test]
    fn remove_nonexistent_entry_returns_false() {
        let mut index = RTreeSpatialIndex::new();
        index.insert(0, &make_point(1.0, 1.0)).unwrap();

        let removed = index.remove(999);
        assert!(!removed);
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn remove_from_empty_index_returns_false() {
        let mut index = RTreeSpatialIndex::new();
        let removed = index.remove(0);
        assert!(!removed);
    }

    #[test]
    fn remove_and_requery() {
        let entries = vec![
            (0, make_point(1.0, 0.0)),
            (1, make_point(2.0, 0.0)),
            (2, make_point(3.0, 0.0)),
        ];
        let mut index = RTreeSpatialIndex::bulk_load(entries).unwrap();

        // Remove middle point
        assert!(index.remove(1));
        assert_eq!(index.len(), 2);

        // kNN should skip removed entry
        let nearest = index.query_nearest(&make_coord(0.0, 0.0), 3);
        assert_eq!(nearest.len(), 2);
        let ids: Vec<usize> = nearest.iter().map(|(id, _)| *id).collect();
        assert!(ids.contains(&0));
        assert!(ids.contains(&2));
        assert!(!ids.contains(&1));

        // Within-distance should skip removed entry
        let mut within = index.query_within_distance(&make_coord(0.0, 0.0), 5.0);
        within.sort();
        assert_eq!(within, vec![0, 2]);
    }

    #[test]
    fn remove_all_entries_leaves_empty_index() {
        let mut index = RTreeSpatialIndex::new();
        index.insert(0, &make_point(1.0, 1.0)).unwrap();
        index.insert(1, &make_point(2.0, 2.0)).unwrap();

        assert!(index.remove(0));
        assert!(index.remove(1));
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);

        let results = index.query_bbox(&make_bbox(0.0, 0.0, 10.0, 10.0));
        assert!(results.is_empty());
    }

    // ── Scale tests ───────────────────────────────────────────────

    #[test]
    fn scale_100k_points_bbox_query_vs_brute_force() {
        // Create 100K points on a grid (100K = 316x316 approximately, use 100x1000)
        let entries: Vec<(usize, SurrealGeometry)> = (0..100_000)
            .map(|i| {
                let x = (i % 1000) as f64;
                let y = (i / 1000) as f64;
                (i, make_point(x, y))
            })
            .collect();

        let index = RTreeSpatialIndex::bulk_load(entries.clone()).unwrap();
        assert_eq!(index.len(), 100_000);

        // Query a sub-region: x in [10,19], y in [10,19]
        let query = make_bbox(10.0, 10.0, 19.0, 19.0);
        let mut rtree_results = index.query_bbox(&query);
        rtree_results.sort();

        // Brute force comparison
        let mut brute_results: Vec<usize> = entries
            .iter()
            .filter(|(_, geom)| {
                let bb = geom.bbox().unwrap();
                bb.min_x <= query.max_x
                    && bb.max_x >= query.min_x
                    && bb.min_y <= query.max_y
                    && bb.max_y >= query.min_y
            })
            .map(|(id, _)| *id)
            .collect();
        brute_results.sort();

        assert_eq!(rtree_results, brute_results);
        // 10 columns (10..19) x 10 rows (10..19) = 100 points
        assert_eq!(rtree_results.len(), 100);
    }

    // ── Edge cases ────────────────────────────────────────────────

    #[test]
    fn degenerate_point_bbox() {
        let mut index = RTreeSpatialIndex::new();
        index.insert(0, &make_point(5.0, 5.0)).unwrap();

        let results = index.query_bbox(&make_bbox(5.0, 5.0, 5.0, 5.0));
        assert_eq!(results, vec![0]);
    }

    #[test]
    fn overlapping_identical_bboxes() {
        let entries = vec![
            (0, make_polygon_geom(0.0, 0.0, 5.0, 5.0)),
            (1, make_polygon_geom(0.0, 0.0, 5.0, 5.0)),
            (2, make_polygon_geom(0.0, 0.0, 5.0, 5.0)),
        ];
        let index = RTreeSpatialIndex::bulk_load(entries).unwrap();

        let mut results = index.query_bbox(&make_bbox(2.0, 2.0, 3.0, 3.0));
        results.sort();
        assert_eq!(results, vec![0, 1, 2]);
    }

    #[test]
    fn very_large_coordinates() {
        let mut index = RTreeSpatialIndex::new();
        let big = 1e15;
        index.insert(0, &make_polygon_geom(-big, -big, big, big)).unwrap();

        let results = index.query_bbox(&make_bbox(-1.0, -1.0, 1.0, 1.0));
        assert_eq!(results, vec![0]);
    }

    #[test]
    fn very_small_coordinates() {
        let mut index = RTreeSpatialIndex::new();
        let small = 1e-15;
        index.insert(0, &make_point(small, small)).unwrap();

        let results = index.query_bbox(&make_bbox(-1.0, -1.0, 1.0, 1.0));
        assert_eq!(results, vec![0]);
    }

    #[test]
    fn touching_bboxes_intersect() {
        let mut index = RTreeSpatialIndex::new();
        index.insert(0, &make_polygon_geom(0.0, 0.0, 5.0, 5.0)).unwrap();
        index.insert(1, &make_polygon_geom(5.0, 0.0, 10.0, 5.0)).unwrap();

        let mut results = index.query_bbox(&make_bbox(5.0, 0.0, 5.0, 5.0));
        results.sort();
        assert_eq!(results, vec![0, 1]);
    }

    #[test]
    fn query_bbox_partial_overlap() {
        let mut index = RTreeSpatialIndex::new();
        index.insert(0, &make_polygon_geom(0.0, 0.0, 5.0, 5.0)).unwrap();
        index.insert(1, &make_polygon_geom(4.0, 4.0, 9.0, 9.0)).unwrap();
        index.insert(2, &make_polygon_geom(10.0, 10.0, 15.0, 15.0)).unwrap();

        let mut results = index.query_bbox(&make_bbox(3.0, 3.0, 6.0, 6.0));
        results.sort();
        assert_eq!(results, vec![0, 1]);
    }

    #[test]
    fn mixed_bbox_sizes_query() {
        let entries = vec![
            (0, make_polygon_geom(0.0, 0.0, 100.0, 100.0)),
            (1, make_polygon_geom(49.0, 49.0, 51.0, 51.0)),
            (2, make_point(50.0, 50.0)),
        ];
        let index = RTreeSpatialIndex::bulk_load(entries).unwrap();

        let mut results = index.query_bbox(&make_bbox(49.5, 49.5, 50.5, 50.5));
        results.sort();
        assert_eq!(results, vec![0, 1, 2]);
    }

    // ── Default trait ─────────────────────────────────────────────

    #[test]
    fn default_is_empty_index() {
        let index = RTreeSpatialIndex::default();
        assert!(index.is_empty());
    }

    // ── Insert with polygon (bbox extracted) ──────────────────────

    #[test]
    fn insert_polygon_queries_by_bbox() {
        let mut index = RTreeSpatialIndex::new();
        let exterior = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(10.0, 0.0).unwrap(),
            Coordinate::new(10.0, 10.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        let poly = SurrealGeometry::polygon(exterior, vec![], Srid::WGS84).unwrap();
        index.insert(0, &poly).unwrap();

        let results = index.query_bbox(&make_bbox(5.0, 5.0, 6.0, 6.0));
        assert_eq!(results, vec![0]);

        let results = index.query_bbox(&make_bbox(20.0, 20.0, 30.0, 30.0));
        assert!(results.is_empty());
    }
}
