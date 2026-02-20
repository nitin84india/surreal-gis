use surrealgis_core::geometry::SurrealGeometry;

use crate::FunctionError;

/// Collect a set of geometries into a GeometryCollection.
/// Uses the SRID of the first geometry for the result.
pub fn st_collect(geoms: &[SurrealGeometry]) -> Result<SurrealGeometry, FunctionError> {
    if geoms.is_empty() {
        return Err(FunctionError::InvalidArgument(
            "st_collect requires at least one geometry".to_string(),
        ));
    }
    let srid = *geoms[0].srid();
    let geo_geoms: Result<Vec<geo_types::Geometry<f64>>, _> =
        geoms.iter().map(|g| g.to_geo()).collect();
    let gc = geo_types::GeometryCollection(geo_geoms?);
    let result = geo_types::Geometry::GeometryCollection(gc);
    SurrealGeometry::from_geo(&result, srid).map_err(FunctionError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::coordinate::Coordinate;
    use surrealgis_core::geometry::GeometryType;
    use surrealgis_core::srid::Srid;

    #[test]
    fn collect_single_point() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        let result = st_collect(&[p]).unwrap();
        assert_eq!(result.type_name(), "GeometryCollection");
        assert_eq!(result.num_points(), 1);
    }

    #[test]
    fn collect_multiple_points() {
        let p1 = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        let p2 = SurrealGeometry::point(3.0, 4.0, Srid::WGS84).unwrap();
        let p3 = SurrealGeometry::point(5.0, 6.0, Srid::WGS84).unwrap();
        let result = st_collect(&[p1, p2, p3]).unwrap();
        assert_eq!(result.type_name(), "GeometryCollection");
        assert_eq!(result.num_points(), 3);
    }

    #[test]
    fn collect_mixed_types() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
        ];
        let line = SurrealGeometry::line_string(coords, Srid::WGS84).unwrap();
        let result = st_collect(&[p, line]).unwrap();
        assert_eq!(result.type_name(), "GeometryCollection");
        assert_eq!(result.num_points(), 3);
    }

    #[test]
    fn collect_empty_rejected() {
        let result = st_collect(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn collect_preserves_first_srid() {
        let p1 = SurrealGeometry::point(1.0, 2.0, Srid::WEB_MERCATOR).unwrap();
        let p2 = SurrealGeometry::point(3.0, 4.0, Srid::WGS84).unwrap();
        let result = st_collect(&[p1, p2]).unwrap();
        assert_eq!(result.srid().code(), Srid::WEB_MERCATOR.code());
    }

    #[test]
    fn collect_returns_geometry_collection_children() {
        let p1 = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        let p2 = SurrealGeometry::point(3.0, 4.0, Srid::WGS84).unwrap();
        let result = st_collect(&[p1, p2]).unwrap();
        if let GeometryType::GeometryCollection(children) = result.geometry_type() {
            assert_eq!(children.len(), 2);
            assert_eq!(children[0].type_name(), "Point");
            assert_eq!(children[1].type_name(), "Point");
        } else {
            panic!("Expected GeometryCollection");
        }
    }
}
