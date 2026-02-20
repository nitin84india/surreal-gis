use surrealgis_core::bbox::BoundingBox;
use surrealgis_core::geometry::SurrealGeometry;

/// Pre-filter two bounding boxes for potential intersection.
///
/// Returns `true` if the bboxes intersect (potential match), `false` for definite non-match.
/// This is the primary entry point for bbox-based spatial filtering.
pub fn bbox_pre_filter(a: &BoundingBox, b: &BoundingBox) -> bool {
    bbox_intersects(a, b)
}

/// Check if two bounding boxes intersect (share any area, including edges/corners).
pub fn bbox_intersects(a: &BoundingBox, b: &BoundingBox) -> bool {
    a.min_x <= b.max_x
        && a.max_x >= b.min_x
        && a.min_y <= b.max_y
        && a.max_y >= b.min_y
}

/// Check if the outer bounding box fully contains the inner bounding box.
pub fn bbox_contains(outer: &BoundingBox, inner: &BoundingBox) -> bool {
    outer.min_x <= inner.min_x
        && outer.max_x >= inner.max_x
        && outer.min_y <= inner.min_y
        && outer.max_y >= inner.max_y
}

/// Expand a bounding box by a distance in all four directions.
pub fn expand_bbox(bbox: &BoundingBox, distance: f64) -> BoundingBox {
    BoundingBox::new(
        bbox.min_x - distance,
        bbox.min_y - distance,
        bbox.max_x + distance,
        bbox.max_y + distance,
    )
    .expect("Expanding a valid bbox by a non-negative distance should produce a valid bbox")
}

/// Compute the bounding box of a SurrealGeometry, if available.
pub fn compute_bbox(geom: &SurrealGeometry) -> Option<BoundingBox> {
    geom.bbox().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::bbox::BoundingBox;
    use surrealgis_core::coordinate::Coordinate;
    use surrealgis_core::geometry::SurrealGeometry;
    use surrealgis_core::srid::Srid;

    fn make_bbox(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> BoundingBox {
        BoundingBox::new(min_x, min_y, max_x, max_y).unwrap()
    }

    // ── bbox_pre_filter ─────────────────────────────────────────

    #[test]
    fn pre_filter_returns_true_for_overlapping() {
        let a = make_bbox(0.0, 0.0, 5.0, 5.0);
        let b = make_bbox(3.0, 3.0, 8.0, 8.0);
        assert!(bbox_pre_filter(&a, &b));
    }

    #[test]
    fn pre_filter_returns_false_for_disjoint() {
        let a = make_bbox(0.0, 0.0, 2.0, 2.0);
        let b = make_bbox(5.0, 5.0, 8.0, 8.0);
        assert!(!bbox_pre_filter(&a, &b));
    }

    // ── bbox_intersects ───────────────────────────────────────────

    #[test]
    fn intersects_overlapping() {
        let a = make_bbox(0.0, 0.0, 5.0, 5.0);
        let b = make_bbox(3.0, 3.0, 8.0, 8.0);
        assert!(bbox_intersects(&a, &b));
        assert!(bbox_intersects(&b, &a));
    }

    #[test]
    fn intersects_disjoint() {
        let a = make_bbox(0.0, 0.0, 2.0, 2.0);
        let b = make_bbox(5.0, 5.0, 8.0, 8.0);
        assert!(!bbox_intersects(&a, &b));
    }

    #[test]
    fn intersects_touching_edge() {
        let a = make_bbox(0.0, 0.0, 5.0, 5.0);
        let b = make_bbox(5.0, 0.0, 10.0, 5.0);
        assert!(bbox_intersects(&a, &b));
    }

    #[test]
    fn intersects_touching_corner() {
        let a = make_bbox(0.0, 0.0, 5.0, 5.0);
        let b = make_bbox(5.0, 5.0, 10.0, 10.0);
        assert!(bbox_intersects(&a, &b));
    }

    #[test]
    fn intersects_contained() {
        let a = make_bbox(0.0, 0.0, 10.0, 10.0);
        let b = make_bbox(2.0, 2.0, 5.0, 5.0);
        assert!(bbox_intersects(&a, &b));
    }

    #[test]
    fn intersects_identical() {
        let a = make_bbox(1.0, 1.0, 5.0, 5.0);
        assert!(bbox_intersects(&a, &a));
    }

    // ── bbox_contains ─────────────────────────────────────────────

    #[test]
    fn contains_fully() {
        let outer = make_bbox(0.0, 0.0, 10.0, 10.0);
        let inner = make_bbox(2.0, 2.0, 5.0, 5.0);
        assert!(bbox_contains(&outer, &inner));
    }

    #[test]
    fn contains_not_reversed() {
        let outer = make_bbox(0.0, 0.0, 10.0, 10.0);
        let inner = make_bbox(2.0, 2.0, 5.0, 5.0);
        assert!(!bbox_contains(&inner, &outer));
    }

    #[test]
    fn contains_identical() {
        let a = make_bbox(0.0, 0.0, 5.0, 5.0);
        assert!(bbox_contains(&a, &a));
    }

    #[test]
    fn contains_on_boundary() {
        let outer = make_bbox(0.0, 0.0, 10.0, 10.0);
        let inner = make_bbox(0.0, 0.0, 10.0, 10.0);
        assert!(bbox_contains(&outer, &inner));
    }

    #[test]
    fn contains_partially_outside() {
        let outer = make_bbox(0.0, 0.0, 5.0, 5.0);
        let inner = make_bbox(3.0, 3.0, 8.0, 8.0);
        assert!(!bbox_contains(&outer, &inner));
    }

    // ── expand_bbox ───────────────────────────────────────────────

    #[test]
    fn expand_by_positive_distance() {
        let bbox = make_bbox(5.0, 5.0, 10.0, 10.0);
        let expanded = expand_bbox(&bbox, 2.0);
        assert_eq!(expanded.min_x, 3.0);
        assert_eq!(expanded.min_y, 3.0);
        assert_eq!(expanded.max_x, 12.0);
        assert_eq!(expanded.max_y, 12.0);
    }

    #[test]
    fn expand_by_zero() {
        let bbox = make_bbox(1.0, 2.0, 3.0, 4.0);
        let expanded = expand_bbox(&bbox, 0.0);
        assert_eq!(expanded, bbox);
    }

    // ── compute_bbox ──────────────────────────────────────────────

    #[test]
    fn compute_bbox_from_point() {
        let p = SurrealGeometry::point(5.0, 10.0, Srid::WGS84).unwrap();
        let bbox = compute_bbox(&p).unwrap();
        assert_eq!(bbox.min_x, 5.0);
        assert_eq!(bbox.min_y, 10.0);
        assert_eq!(bbox.max_x, 5.0);
        assert_eq!(bbox.max_y, 10.0);
    }

    #[test]
    fn compute_bbox_from_polygon() {
        let exterior = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(10.0, 0.0).unwrap(),
            Coordinate::new(10.0, 10.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        let poly = SurrealGeometry::polygon(exterior, vec![], Srid::WGS84).unwrap();
        let bbox = compute_bbox(&poly).unwrap();
        assert_eq!(bbox.min_x, 0.0);
        assert_eq!(bbox.min_y, 0.0);
        assert_eq!(bbox.max_x, 10.0);
        assert_eq!(bbox.max_y, 10.0);
    }

    #[test]
    fn compute_bbox_from_linestring() {
        let coords = vec![
            Coordinate::new(-5.0, -3.0).unwrap(),
            Coordinate::new(7.0, 11.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WGS84).unwrap();
        let bbox = compute_bbox(&ls).unwrap();
        assert_eq!(bbox.min_x, -5.0);
        assert_eq!(bbox.min_y, -3.0);
        assert_eq!(bbox.max_x, 7.0);
        assert_eq!(bbox.max_y, 11.0);
    }
}
