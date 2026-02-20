use geozero::{CoordDimensions, ToWkb};

use crate::error::GeometryError;
use crate::geometry::SurrealGeometry;
use crate::srid::Srid;

/// Convert a SurrealGeometry to WKB bytes.
pub fn to_wkb(geom: &SurrealGeometry) -> Result<Vec<u8>, GeometryError> {
    let geo = geom.to_geo()?;
    let wkb_bytes = geo
        .to_wkb(CoordDimensions::xy())
        .map_err(|e| GeometryError::SerializationError(format!("WKB encode error: {e}")))?;
    Ok(wkb_bytes)
}

/// Parse WKB bytes into a SurrealGeometry.
pub fn from_wkb(wkb_bytes: &[u8]) -> Result<SurrealGeometry, GeometryError> {
    use geozero::wkb::Wkb;
    use geozero::ToGeo;

    let wkb = Wkb(wkb_bytes.to_vec());
    let geo: geo_types::Geometry<f64> = wkb
        .to_geo()
        .map_err(|e| GeometryError::SerializationError(format!("WKB decode error: {e}")))?;
    SurrealGeometry::from_geo(&geo, Srid::DEFAULT)
}

/// Convert a SurrealGeometry to hex-encoded WKB string.
pub fn to_wkb_hex(geom: &SurrealGeometry) -> Result<String, GeometryError> {
    let bytes = to_wkb(geom)?;
    Ok(hex_encode(&bytes))
}

/// Parse a hex-encoded WKB string into a SurrealGeometry.
pub fn from_wkb_hex(hex_str: &str) -> Result<SurrealGeometry, GeometryError> {
    let bytes = hex_decode(hex_str)
        .map_err(|e| GeometryError::SerializationError(format!("Invalid hex: {e}")))?;
    from_wkb(&bytes)
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex_decode(hex: &str) -> Result<Vec<u8>, String> {
    if hex.len() % 2 != 0 {
        return Err("Hex string must have even length".to_string());
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| format!("Invalid hex at position {i}: {e}"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::Coordinate;

    #[test]
    fn point_wkb_roundtrip() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        let wkb_bytes = to_wkb(&p).unwrap();
        assert!(!wkb_bytes.is_empty());
        let roundtripped = from_wkb(&wkb_bytes).unwrap();
        assert_eq!(roundtripped.type_name(), "Point");
    }

    #[test]
    fn linestring_wkb_roundtrip() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
            Coordinate::new(2.0, 0.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WGS84).unwrap();
        let wkb_bytes = to_wkb(&ls).unwrap();
        let roundtripped = from_wkb(&wkb_bytes).unwrap();
        assert_eq!(roundtripped.type_name(), "LineString");
        assert_eq!(roundtripped.num_points(), 3);
    }

    #[test]
    fn polygon_wkb_roundtrip() {
        let exterior = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        let poly = SurrealGeometry::polygon(exterior, vec![], Srid::WGS84).unwrap();
        let wkb_bytes = to_wkb(&poly).unwrap();
        let roundtripped = from_wkb(&wkb_bytes).unwrap();
        assert_eq!(roundtripped.type_name(), "Polygon");
    }

    #[test]
    fn point_wkb_hex_roundtrip() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        let hex = to_wkb_hex(&p).unwrap();
        assert!(!hex.is_empty());
        let roundtripped = from_wkb_hex(&hex).unwrap();
        assert_eq!(roundtripped.type_name(), "Point");
    }

    #[test]
    fn invalid_wkb_returns_error() {
        let result = from_wkb(&[0x00, 0x01, 0x02]);
        assert!(result.is_err());
    }

    #[test]
    fn invalid_hex_returns_error() {
        let result = from_wkb_hex("ZZZZ");
        assert!(result.is_err());
    }

    #[test]
    fn odd_hex_returns_error() {
        let result = from_wkb_hex("abc");
        assert!(result.is_err());
    }
}
