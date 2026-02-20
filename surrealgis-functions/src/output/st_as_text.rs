use surrealgis_core::geometry::SurrealGeometry;
use surrealgis_core::serialization::wkt;

use crate::FunctionError;

/// Convert a geometry to WKT text representation.
pub fn st_as_text(geom: &SurrealGeometry) -> Result<String, FunctionError> {
    wkt::to_wkt(geom).map_err(FunctionError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::coordinate::Coordinate;
    use surrealgis_core::srid::Srid;

    #[test]
    fn point_to_wkt() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        let wkt = st_as_text(&p).unwrap();
        assert!(wkt.contains("POINT"));
        assert!(wkt.contains("1"));
        assert!(wkt.contains("2"));
    }

    #[test]
    fn linestring_to_wkt() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WGS84).unwrap();
        let wkt = st_as_text(&ls).unwrap();
        assert!(wkt.contains("LINESTRING"));
    }

    #[test]
    fn polygon_to_wkt() {
        let exterior = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        let poly = SurrealGeometry::polygon(exterior, vec![], Srid::WGS84).unwrap();
        let wkt = st_as_text(&poly).unwrap();
        assert!(wkt.contains("POLYGON"));
    }
}
