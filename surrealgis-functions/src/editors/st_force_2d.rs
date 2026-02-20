use surrealgis_core::geometry::SurrealGeometry;

use crate::FunctionError;

/// Force a geometry to 2D by stripping any Z/M coordinates.
/// Since our domain model is already 2D (only x, y in Coordinate),
/// this is effectively a roundtrip through geo to normalize the geometry.
/// Implemented for PostGIS compatibility.
pub fn st_force_2d(geom: &SurrealGeometry) -> Result<SurrealGeometry, FunctionError> {
    let geo_geom = geom.to_geo()?;
    SurrealGeometry::from_geo(&geo_geom, *geom.srid()).map_err(FunctionError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::coordinate::Coordinate;
    use surrealgis_core::geometry::GeometryType;
    use surrealgis_core::srid::Srid;

    #[test]
    fn force_2d_point() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        let result = st_force_2d(&p).unwrap();
        if let GeometryType::Point(c) = result.geometry_type() {
            assert!((c.x() - 1.0).abs() < 1e-10);
            assert!((c.y() - 2.0).abs() < 1e-10);
        } else {
            panic!("Expected Point");
        }
    }

    #[test]
    fn force_2d_linestring() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
            Coordinate::new(2.0, 0.0).unwrap(),
        ];
        let line = SurrealGeometry::line_string(coords, Srid::WGS84).unwrap();
        let result = st_force_2d(&line).unwrap();
        assert_eq!(result.type_name(), "LineString");
        assert_eq!(result.num_points(), 3);
    }

    #[test]
    fn force_2d_polygon() {
        let exterior = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(10.0, 0.0).unwrap(),
            Coordinate::new(10.0, 10.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        let poly = SurrealGeometry::polygon(exterior, vec![], Srid::WGS84).unwrap();
        let result = st_force_2d(&poly).unwrap();
        assert_eq!(result.type_name(), "Polygon");
        assert_eq!(result.num_points(), 4);
    }

    #[test]
    fn force_2d_preserves_srid() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WEB_MERCATOR).unwrap();
        let result = st_force_2d(&p).unwrap();
        assert_eq!(result.srid().code(), Srid::WEB_MERCATOR.code());
    }

    #[test]
    fn force_2d_preserves_dimension() {
        let p = SurrealGeometry::point(5.0, 10.0, Srid::WGS84).unwrap();
        let result = st_force_2d(&p).unwrap();
        assert_eq!(result.dimension(), 2);
    }
}
