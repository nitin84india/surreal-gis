use geo::BooleanOps;
use surrealgis_core::geometry::SurrealGeometry;

use crate::FunctionError;

/// Compute the geometric intersection of two polygon geometries.
/// Returns the area shared by both input geometries.
pub fn st_intersection(
    a: &SurrealGeometry,
    b: &SurrealGeometry,
) -> Result<SurrealGeometry, FunctionError> {
    let (mp_a, mp_b) = super::extract_polygon_operands(a, b)?;
    let result = mp_a.intersection(&mp_b);
    let geo_geom = geo_types::Geometry::MultiPolygon(result);
    SurrealGeometry::from_geo(&geo_geom, *a.srid()).map_err(FunctionError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::coordinate::Coordinate;
    use surrealgis_core::geometry::{PolygonData, SurrealGeometry};
    use surrealgis_core::srid::Srid;

    fn rect_polygon(x1: f64, y1: f64, x2: f64, y2: f64, srid: Srid) -> SurrealGeometry {
        let exterior = vec![
            Coordinate::new(x1, y1).unwrap(),
            Coordinate::new(x2, y1).unwrap(),
            Coordinate::new(x2, y2).unwrap(),
            Coordinate::new(x1, y2).unwrap(),
            Coordinate::new(x1, y1).unwrap(),
        ];
        SurrealGeometry::polygon(exterior, vec![], srid).unwrap()
    }

    #[test]
    fn overlapping_rectangles() {
        let a = rect_polygon(0.0, 0.0, 2.0, 2.0, Srid::WEB_MERCATOR);
        let b = rect_polygon(1.0, 1.0, 3.0, 3.0, Srid::WEB_MERCATOR);
        let result = st_intersection(&a, &b).unwrap();
        // Intersection should be the [1,1,2,2] square = area 1
        let geo = result.to_geo().unwrap();
        let area = geo::Area::unsigned_area(&geo);
        assert!((area - 1.0).abs() < 1e-6, "area was {area}");
    }

    #[test]
    fn identical_polygons() {
        let a = rect_polygon(0.0, 0.0, 2.0, 2.0, Srid::WEB_MERCATOR);
        let b = rect_polygon(0.0, 0.0, 2.0, 2.0, Srid::WEB_MERCATOR);
        let result = st_intersection(&a, &b).unwrap();
        let geo = result.to_geo().unwrap();
        let area = geo::Area::unsigned_area(&geo);
        assert!((area - 4.0).abs() < 1e-6, "area was {area}");
    }

    #[test]
    fn non_overlapping_polygons() {
        let a = rect_polygon(0.0, 0.0, 1.0, 1.0, Srid::WEB_MERCATOR);
        let b = rect_polygon(5.0, 5.0, 6.0, 6.0, Srid::WEB_MERCATOR);
        let result = st_intersection(&a, &b).unwrap();
        let geo = result.to_geo().unwrap();
        let area = geo::Area::unsigned_area(&geo);
        assert!(area < 1e-10, "area was {area}");
    }

    #[test]
    fn rejects_point_input() {
        let a = SurrealGeometry::point(1.0, 2.0, Srid::WEB_MERCATOR).unwrap();
        let b = rect_polygon(0.0, 0.0, 2.0, 2.0, Srid::WEB_MERCATOR);
        let result = st_intersection(&a, &b);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FunctionError::UnsupportedOperation(_)));
    }

    #[test]
    fn rejects_linestring_input() {
        let ls = SurrealGeometry::line_string(
            vec![
                Coordinate::new(0.0, 0.0).unwrap(),
                Coordinate::new(1.0, 1.0).unwrap(),
            ],
            Srid::WEB_MERCATOR,
        )
        .unwrap();
        let b = rect_polygon(0.0, 0.0, 2.0, 2.0, Srid::WEB_MERCATOR);
        let result = st_intersection(&ls, &b);
        assert!(result.is_err());
    }

    #[test]
    fn srid_preservation() {
        let srid = Srid::new(32632).unwrap();
        let a = rect_polygon(0.0, 0.0, 2.0, 2.0, srid);
        let b = rect_polygon(1.0, 1.0, 3.0, 3.0, srid);
        let result = st_intersection(&a, &b).unwrap();
        assert_eq!(result.srid().code(), 32632);
    }

    #[test]
    fn multi_polygon_input() {
        let polys = vec![
            PolygonData {
                exterior: vec![
                    Coordinate::new(0.0, 0.0).unwrap(),
                    Coordinate::new(2.0, 0.0).unwrap(),
                    Coordinate::new(2.0, 2.0).unwrap(),
                    Coordinate::new(0.0, 2.0).unwrap(),
                    Coordinate::new(0.0, 0.0).unwrap(),
                ],
                holes: vec![],
            },
        ];
        let a = SurrealGeometry::multi_polygon(polys, Srid::WEB_MERCATOR).unwrap();
        let b = rect_polygon(1.0, 1.0, 3.0, 3.0, Srid::WEB_MERCATOR);
        let result = st_intersection(&a, &b).unwrap();
        let geo = result.to_geo().unwrap();
        let area = geo::Area::unsigned_area(&geo);
        assert!((area - 1.0).abs() < 1e-6, "area was {area}");
    }
}
