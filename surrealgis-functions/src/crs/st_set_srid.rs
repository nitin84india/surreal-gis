use surrealgis_core::geometry::SurrealGeometry;
use surrealgis_crs::transform;

use crate::FunctionError;

/// Change the SRID metadata of a geometry without reprojecting coordinates.
pub fn st_set_srid(geom: &SurrealGeometry, new_srid: i32) -> Result<SurrealGeometry, FunctionError> {
    transform::set_srid(geom, new_srid)
        .map_err(|e| FunctionError::CrsError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::geometry::GeometryType;
    use surrealgis_core::srid::Srid;

    #[test]
    fn set_srid_changes_metadata() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        let updated = st_set_srid(&p, 3857).unwrap();
        assert_eq!(updated.srid().code(), 3857);
        // Coordinates should remain unchanged
        if let GeometryType::Point(c) = updated.geometry_type() {
            assert_eq!(c.x(), 1.0);
            assert_eq!(c.y(), 2.0);
        }
    }

    #[test]
    fn set_srid_invalid_srid() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        let result = st_set_srid(&p, 0);
        assert!(result.is_err());
    }
}
