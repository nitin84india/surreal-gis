use geo::Rotate;
use surrealgis_core::geometry::SurrealGeometry;

use crate::FunctionError;

/// Rotate a geometry around its centroid by a given angle in degrees.
/// Positive angle rotates counter-clockwise.
pub fn st_rotate(
    geom: &SurrealGeometry,
    angle_degrees: f64,
) -> Result<SurrealGeometry, FunctionError> {
    let geo_geom = geom.to_geo()?;
    let result = geo_geom.rotate_around_centroid(angle_degrees);
    SurrealGeometry::from_geo(&result, *geom.srid()).map_err(FunctionError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::coordinate::Coordinate;
    use surrealgis_core::geometry::GeometryType;
    use surrealgis_core::srid::Srid;

    #[test]
    fn rotate_point_is_identity() {
        // Rotating a point around its centroid (itself) should not change it
        let p = SurrealGeometry::point(5.0, 10.0, Srid::WEB_MERCATOR).unwrap();
        let result = st_rotate(&p, 90.0).unwrap();
        if let GeometryType::Point(c) = result.geometry_type() {
            assert!((c.x() - 5.0).abs() < 1e-10);
            assert!((c.y() - 10.0).abs() < 1e-10);
        } else {
            panic!("Expected Point");
        }
    }

    #[test]
    fn rotate_zero_degrees() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        let poly = SurrealGeometry::polygon(coords, vec![], Srid::WEB_MERCATOR).unwrap();
        let result = st_rotate(&poly, 0.0).unwrap();
        if let GeometryType::Polygon { exterior, .. } = result.geometry_type() {
            assert!((exterior[0].x() - 0.0).abs() < 1e-10);
            assert!((exterior[0].y() - 0.0).abs() < 1e-10);
        } else {
            panic!("Expected Polygon");
        }
    }

    #[test]
    fn rotate_360_degrees_is_identity() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(2.0, 0.0).unwrap(),
            Coordinate::new(2.0, 2.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        let poly = SurrealGeometry::polygon(coords.clone(), vec![], Srid::WEB_MERCATOR).unwrap();
        let result = st_rotate(&poly, 360.0).unwrap();
        if let GeometryType::Polygon { exterior, .. } = result.geometry_type() {
            for (orig, rotated) in coords.iter().zip(exterior.iter()) {
                assert!((orig.x() - rotated.x()).abs() < 1e-8);
                assert!((orig.y() - rotated.y()).abs() < 1e-8);
            }
        } else {
            panic!("Expected Polygon");
        }
    }

    #[test]
    fn rotate_preserves_srid() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WEB_MERCATOR).unwrap();
        let result = st_rotate(&p, 45.0).unwrap();
        assert_eq!(result.srid().code(), Srid::WEB_MERCATOR.code());
    }
}
