use wkt::ToWkt;

use crate::error::GeometryError;
use crate::geometry::SurrealGeometry;
use crate::srid::Srid;

/// Convert a SurrealGeometry to WKT string.
pub fn to_wkt(geom: &SurrealGeometry) -> Result<String, GeometryError> {
    let geo = geom.to_geo()?;
    Ok(geo.wkt_string())
}

/// Parse a WKT string into a SurrealGeometry with default SRID 4326.
pub fn from_wkt(wkt_str: &str) -> Result<SurrealGeometry, GeometryError> {
    let geo: geo_types::Geometry<f64> = wkt::TryFromWkt::try_from_wkt_str(wkt_str)
        .map_err(|e| GeometryError::SerializationError(format!("WKT parse error: {e}")))?;
    SurrealGeometry::from_geo(&geo, Srid::DEFAULT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::Coordinate;

    #[test]
    fn point_wkt_roundtrip() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        let wkt_str = to_wkt(&p).unwrap();
        assert!(wkt_str.contains("POINT"));
        let roundtripped = from_wkt(&wkt_str).unwrap();
        assert_eq!(roundtripped.type_name(), "Point");
    }

    #[test]
    fn linestring_wkt_roundtrip() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
            Coordinate::new(2.0, 0.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WGS84).unwrap();
        let wkt_str = to_wkt(&ls).unwrap();
        assert!(wkt_str.contains("LINESTRING"));
        let roundtripped = from_wkt(&wkt_str).unwrap();
        assert_eq!(roundtripped.type_name(), "LineString");
        assert_eq!(roundtripped.num_points(), 3);
    }

    #[test]
    fn polygon_wkt_roundtrip() {
        let exterior = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        let poly = SurrealGeometry::polygon(exterior, vec![], Srid::WGS84).unwrap();
        let wkt_str = to_wkt(&poly).unwrap();
        assert!(wkt_str.contains("POLYGON"));
        let roundtripped = from_wkt(&wkt_str).unwrap();
        assert_eq!(roundtripped.type_name(), "Polygon");
    }

    #[test]
    fn invalid_wkt_returns_error() {
        let result = from_wkt("NOT_A_WKT");
        assert!(result.is_err());
    }

    #[test]
    fn from_wkt_uses_default_srid() {
        let p = from_wkt("POINT(5 10)").unwrap();
        assert_eq!(p.srid().code(), 4326);
    }
}
