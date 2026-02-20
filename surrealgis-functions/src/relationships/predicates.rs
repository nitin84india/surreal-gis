use geo::algorithm::Relate;
use surrealgis_core::geometry::SurrealGeometry;

use crate::FunctionError;

/// Pre-filter using bounding boxes for fast rejection.
fn bbox_pre_filter(a: &SurrealGeometry, b: &SurrealGeometry) -> Option<bool> {
    if let (Some(bbox_a), Some(bbox_b)) = (a.bbox(), b.bbox()) {
        if !bbox_a.intersects(bbox_b) {
            return Some(false);
        }
    }
    None // Cannot determine, must do full check
}

/// Pre-filter that returns true only if bboxes DON'T intersect (for disjoint).
fn bbox_pre_filter_disjoint(a: &SurrealGeometry, b: &SurrealGeometry) -> Option<bool> {
    if let (Some(bbox_a), Some(bbox_b)) = (a.bbox(), b.bbox()) {
        if !bbox_a.intersects(bbox_b) {
            return Some(true);
        }
    }
    None
}

/// Returns true if the two geometries spatially intersect.
pub fn st_intersects(a: &SurrealGeometry, b: &SurrealGeometry) -> Result<bool, FunctionError> {
    if let Some(result) = bbox_pre_filter(a, b) {
        return Ok(result);
    }
    let ga = a.to_geo()?;
    let gb = b.to_geo()?;
    Ok(ga.relate(&gb).is_intersects())
}

/// Returns true if geometry A contains geometry B.
pub fn st_contains(a: &SurrealGeometry, b: &SurrealGeometry) -> Result<bool, FunctionError> {
    if let Some(false) = bbox_pre_filter(a, b) {
        return Ok(false);
    }
    let ga = a.to_geo()?;
    let gb = b.to_geo()?;
    Ok(ga.relate(&gb).is_contains())
}

/// Returns true if geometry A is within geometry B.
pub fn st_within(a: &SurrealGeometry, b: &SurrealGeometry) -> Result<bool, FunctionError> {
    if let Some(false) = bbox_pre_filter(a, b) {
        return Ok(false);
    }
    let ga = a.to_geo()?;
    let gb = b.to_geo()?;
    Ok(ga.relate(&gb).is_within())
}

/// Returns true if the geometries touch (share boundary but not interior).
pub fn st_touches(a: &SurrealGeometry, b: &SurrealGeometry) -> Result<bool, FunctionError> {
    if let Some(false) = bbox_pre_filter(a, b) {
        return Ok(false);
    }
    let ga = a.to_geo()?;
    let gb = b.to_geo()?;
    Ok(ga.relate(&gb).is_touches())
}

/// Returns true if the geometries cross each other.
pub fn st_crosses(a: &SurrealGeometry, b: &SurrealGeometry) -> Result<bool, FunctionError> {
    if let Some(false) = bbox_pre_filter(a, b) {
        return Ok(false);
    }
    let ga = a.to_geo()?;
    let gb = b.to_geo()?;
    Ok(ga.relate(&gb).is_crosses())
}

/// Returns true if the geometries overlap.
pub fn st_overlaps(a: &SurrealGeometry, b: &SurrealGeometry) -> Result<bool, FunctionError> {
    if let Some(false) = bbox_pre_filter(a, b) {
        return Ok(false);
    }
    let ga = a.to_geo()?;
    let gb = b.to_geo()?;
    Ok(ga.relate(&gb).is_overlaps())
}

/// Returns true if the geometries are spatially disjoint.
pub fn st_disjoint(a: &SurrealGeometry, b: &SurrealGeometry) -> Result<bool, FunctionError> {
    if let Some(result) = bbox_pre_filter_disjoint(a, b) {
        return Ok(result);
    }
    let ga = a.to_geo()?;
    let gb = b.to_geo()?;
    Ok(!ga.relate(&gb).is_intersects())
}

/// Returns true if the geometries are topologically equal.
pub fn st_equals(a: &SurrealGeometry, b: &SurrealGeometry) -> Result<bool, FunctionError> {
    let ga = a.to_geo()?;
    let gb = b.to_geo()?;
    let matrix = ga.relate(&gb);
    Ok(matrix.is_within() && matrix.is_contains())
}

/// Returns true if geometry A covers geometry B.
pub fn st_covers(a: &SurrealGeometry, b: &SurrealGeometry) -> Result<bool, FunctionError> {
    if let Some(false) = bbox_pre_filter(a, b) {
        return Ok(false);
    }
    let ga = a.to_geo()?;
    let gb = b.to_geo()?;
    Ok(ga.relate(&gb).is_covers())
}

/// Returns true if geometry A is covered by geometry B.
/// Equivalent to st_covers(b, a).
pub fn st_covered_by(a: &SurrealGeometry, b: &SurrealGeometry) -> Result<bool, FunctionError> {
    st_covers(b, a)
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::coordinate::Coordinate;
    use surrealgis_core::srid::Srid;

    fn poly_a() -> SurrealGeometry {
        let exterior = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(2.0, 0.0).unwrap(),
            Coordinate::new(2.0, 2.0).unwrap(),
            Coordinate::new(0.0, 2.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        SurrealGeometry::polygon(exterior, vec![], Srid::WGS84).unwrap()
    }

    fn poly_b() -> SurrealGeometry {
        let exterior = vec![
            Coordinate::new(1.0, 1.0).unwrap(),
            Coordinate::new(3.0, 1.0).unwrap(),
            Coordinate::new(3.0, 3.0).unwrap(),
            Coordinate::new(1.0, 3.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
        ];
        SurrealGeometry::polygon(exterior, vec![], Srid::WGS84).unwrap()
    }

    fn poly_far() -> SurrealGeometry {
        let exterior = vec![
            Coordinate::new(50.0, 50.0).unwrap(),
            Coordinate::new(51.0, 50.0).unwrap(),
            Coordinate::new(51.0, 51.0).unwrap(),
            Coordinate::new(50.0, 51.0).unwrap(),
            Coordinate::new(50.0, 50.0).unwrap(),
        ];
        SurrealGeometry::polygon(exterior, vec![], Srid::WGS84).unwrap()
    }

    fn point_inside_a() -> SurrealGeometry {
        SurrealGeometry::point(0.5, 0.5, Srid::WGS84).unwrap()
    }

    #[test]
    fn overlapping_polygons_intersect() {
        assert!(st_intersects(&poly_a(), &poly_b()).unwrap());
    }

    #[test]
    fn far_polygons_dont_intersect() {
        assert!(!st_intersects(&poly_a(), &poly_far()).unwrap());
    }

    #[test]
    fn polygon_contains_point() {
        assert!(st_contains(&poly_a(), &point_inside_a()).unwrap());
    }

    #[test]
    fn point_within_polygon() {
        assert!(st_within(&point_inside_a(), &poly_a()).unwrap());
    }

    #[test]
    fn point_not_within_far_polygon() {
        assert!(!st_within(&point_inside_a(), &poly_far()).unwrap());
    }

    #[test]
    fn far_polygons_disjoint() {
        assert!(st_disjoint(&poly_a(), &poly_far()).unwrap());
    }

    #[test]
    fn overlapping_not_disjoint() {
        assert!(!st_disjoint(&poly_a(), &poly_b()).unwrap());
    }

    #[test]
    fn touching_polygons() {
        let a = SurrealGeometry::polygon(
            vec![
                Coordinate::new(0.0, 0.0).unwrap(),
                Coordinate::new(1.0, 0.0).unwrap(),
                Coordinate::new(1.0, 1.0).unwrap(),
                Coordinate::new(0.0, 1.0).unwrap(),
                Coordinate::new(0.0, 0.0).unwrap(),
            ],
            vec![],
            Srid::WGS84,
        ).unwrap();
        let b = SurrealGeometry::polygon(
            vec![
                Coordinate::new(1.0, 0.0).unwrap(),
                Coordinate::new(2.0, 0.0).unwrap(),
                Coordinate::new(2.0, 1.0).unwrap(),
                Coordinate::new(1.0, 1.0).unwrap(),
                Coordinate::new(1.0, 0.0).unwrap(),
            ],
            vec![],
            Srid::WGS84,
        ).unwrap();
        assert!(st_touches(&a, &b).unwrap());
    }

    #[test]
    fn crossing_lines() {
        let coords_a = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(2.0, 2.0).unwrap(),
        ];
        let coords_b = vec![
            Coordinate::new(0.0, 2.0).unwrap(),
            Coordinate::new(2.0, 0.0).unwrap(),
        ];
        let line_a = SurrealGeometry::line_string(coords_a, Srid::WGS84).unwrap();
        let line_b = SurrealGeometry::line_string(coords_b, Srid::WGS84).unwrap();
        assert!(st_crosses(&line_a, &line_b).unwrap());
    }

    #[test]
    fn self_equals() {
        assert!(st_equals(&poly_a(), &poly_a()).unwrap());
    }

    #[test]
    fn polygon_covers_interior_point() {
        assert!(st_covers(&poly_a(), &point_inside_a()).unwrap());
    }

    #[test]
    fn point_covered_by_polygon() {
        assert!(st_covered_by(&point_inside_a(), &poly_a()).unwrap());
    }

    #[test]
    fn bbox_prefilter_rejects_far_intersects() {
        // Far polygons should be rejected by bbox pre-filter
        assert!(!st_intersects(&poly_a(), &poly_far()).unwrap());
    }
}
