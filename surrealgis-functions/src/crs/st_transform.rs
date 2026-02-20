use surrealgis_core::geometry::SurrealGeometry;
use surrealgis_crs::transform;

use crate::FunctionError;

/// Transform (reproject) a geometry from its current SRID to a target SRID.
/// Performs actual coordinate reprojection using proj4rs.
pub fn st_transform(
    geom: &SurrealGeometry,
    to_srid: i32,
) -> Result<SurrealGeometry, FunctionError> {
    let from_srid = geom.srid().code();
    transform::transform_geometry(geom, from_srid, to_srid)
        .map_err(|e| FunctionError::CrsError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::geometry::GeometryType;
    use surrealgis_core::srid::Srid;

    #[test]
    fn transform_4326_to_3857() {
        let p = SurrealGeometry::point(-73.9857, 40.7484, Srid::WGS84).unwrap();
        let transformed = st_transform(&p, 3857).unwrap();
        assert_eq!(transformed.srid().code(), 3857);
        // Web Mercator coords should be large numbers
        if let GeometryType::Point(c) = transformed.geometry_type() {
            assert!(c.x().abs() > 1_000_000.0, "x was {}", c.x());
            assert!(c.y().abs() > 1_000_000.0, "y was {}", c.y());
        }
    }

    #[test]
    fn transform_same_srid_fails() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        let result = st_transform(&p, 4326);
        assert!(result.is_err());
    }
}
