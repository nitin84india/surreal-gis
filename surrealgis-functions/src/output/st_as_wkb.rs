use surrealgis_core::geometry::SurrealGeometry;
use surrealgis_core::serialization::wkb;

use crate::FunctionError;

/// Convert a geometry to WKB binary representation (as hex string).
pub fn st_as_wkb(geom: &SurrealGeometry) -> Result<String, FunctionError> {
    wkb::to_wkb_hex(geom).map_err(FunctionError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::srid::Srid;

    #[test]
    fn point_to_wkb_hex() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        let hex = st_as_wkb(&p).unwrap();
        // WKB hex should be non-empty
        assert!(!hex.is_empty());
        // Should only contain hex characters
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
