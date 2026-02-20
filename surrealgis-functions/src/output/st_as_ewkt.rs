use surrealgis_core::geometry::SurrealGeometry;
use surrealgis_core::serialization::ewkt;

use crate::FunctionError;

/// Convert a geometry to Extended WKT (with SRID prefix).
/// Example output: "SRID=4326;POINT(1 2)"
pub fn st_as_ewkt(geom: &SurrealGeometry) -> Result<String, FunctionError> {
    ewkt::to_ewkt(geom).map_err(FunctionError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::srid::Srid;

    #[test]
    fn point_to_ewkt() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        let ewkt_str = st_as_ewkt(&p).unwrap();
        assert!(ewkt_str.starts_with("SRID=4326;"));
        assert!(ewkt_str.contains("POINT"));
    }

    #[test]
    fn web_mercator_ewkt() {
        let p = SurrealGeometry::point(500000.0, 4500000.0, Srid::WEB_MERCATOR).unwrap();
        let ewkt_str = st_as_ewkt(&p).unwrap();
        assert!(ewkt_str.starts_with("SRID=3857;"));
    }
}
