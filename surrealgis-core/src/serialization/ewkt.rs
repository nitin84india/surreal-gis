use ::wkt::TryFromWkt;

use crate::error::GeometryError;
use crate::geometry::SurrealGeometry;
use crate::serialization::wkt as surreal_wkt;
use crate::srid::Srid;

/// Convert a SurrealGeometry to Extended WKT format: "SRID=4326;POINT(1 2)"
pub fn to_ewkt(geom: &SurrealGeometry) -> Result<String, GeometryError> {
    let wkt_str = surreal_wkt::to_wkt(geom)?;
    Ok(format!("SRID={};{}", geom.srid().code(), wkt_str))
}

/// Parse an EWKT string. Format: "SRID=4326;POINT(1 2)"
/// If no SRID prefix is present, falls back to plain WKT with default SRID.
pub fn from_ewkt(ewkt_str: &str) -> Result<SurrealGeometry, GeometryError> {
    if let Some(rest) = ewkt_str.strip_prefix("SRID=") {
        let semicolon_pos = rest.find(';').ok_or_else(|| {
            GeometryError::SerializationError(
                "EWKT: expected ';' after SRID value".to_string(),
            )
        })?;

        let srid_str = &rest[..semicolon_pos];
        let srid_code: i32 = srid_str.parse().map_err(|e| {
            GeometryError::SerializationError(format!("EWKT: invalid SRID value '{srid_str}': {e}"))
        })?;
        let srid = Srid::new(srid_code)?;

        let wkt_body = &rest[semicolon_pos + 1..];
        let geo: geo_types::Geometry<f64> =
            TryFromWkt::try_from_wkt_str(wkt_body).map_err(|e| {
                GeometryError::SerializationError(format!("EWKT WKT parse error: {e}"))
            })?;
        SurrealGeometry::from_geo(&geo, srid)
    } else {
        // No SRID prefix, treat as plain WKT
        surreal_wkt::from_wkt(ewkt_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::Coordinate;

    #[test]
    fn point_ewkt_roundtrip() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        let ewkt = to_ewkt(&p).unwrap();
        assert!(ewkt.starts_with("SRID=4326;"));
        let roundtripped = from_ewkt(&ewkt).unwrap();
        assert_eq!(roundtripped.type_name(), "Point");
        assert_eq!(roundtripped.srid().code(), 4326);
    }

    #[test]
    fn linestring_ewkt_roundtrip() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WGS84).unwrap();
        let ewkt = to_ewkt(&ls).unwrap();
        let roundtripped = from_ewkt(&ewkt).unwrap();
        assert_eq!(roundtripped.type_name(), "LineString");
        assert_eq!(roundtripped.srid().code(), 4326);
    }

    #[test]
    fn ewkt_with_custom_srid() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WEB_MERCATOR).unwrap();
        let ewkt = to_ewkt(&p).unwrap();
        assert!(ewkt.starts_with("SRID=3857;"));
        let roundtripped = from_ewkt(&ewkt).unwrap();
        assert_eq!(roundtripped.srid().code(), 3857);
    }

    #[test]
    fn ewkt_preserves_srid() {
        let srid = Srid::new(32632).unwrap();
        let p = SurrealGeometry::point(500000.0, 4649776.0, srid).unwrap();
        let ewkt = to_ewkt(&p).unwrap();
        assert!(ewkt.starts_with("SRID=32632;"));
        let roundtripped = from_ewkt(&ewkt).unwrap();
        assert_eq!(roundtripped.srid().code(), 32632);
    }

    #[test]
    fn ewkt_without_srid_prefix_falls_back() {
        let result = from_ewkt("POINT(5 10)").unwrap();
        assert_eq!(result.type_name(), "Point");
        assert_eq!(result.srid().code(), 4326);
    }

    #[test]
    fn ewkt_missing_semicolon_error() {
        let result = from_ewkt("SRID=4326 POINT(1 2)");
        assert!(result.is_err());
    }

    #[test]
    fn ewkt_invalid_srid_error() {
        let result = from_ewkt("SRID=abc;POINT(1 2)");
        assert!(result.is_err());
    }

    #[test]
    fn ewkt_negative_srid_error() {
        let result = from_ewkt("SRID=-1;POINT(1 2)");
        assert!(result.is_err());
    }
}
