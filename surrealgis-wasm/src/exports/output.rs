use serde_json::Value;

use crate::adapter;

// #[surrealism]
/// Convert a geometry to WKT text representation.
pub fn st_as_text(geom: &Value) -> Result<Value, String> {
    let g = adapter::from_json_value(geom)?;
    surrealgis_functions::output::st_as_text(&g)
        .map(Value::from)
        .map_err(|e| e.to_string())
}

// #[surrealism]
/// Convert a geometry to WKB binary representation (as hex string).
pub fn st_as_wkb(geom: &Value) -> Result<Value, String> {
    let g = adapter::from_json_value(geom)?;
    surrealgis_functions::output::st_as_wkb(&g)
        .map(Value::from)
        .map_err(|e| e.to_string())
}

// #[surrealism]
/// Convert a geometry to GeoJSON string.
pub fn st_as_geojson(geom: &Value) -> Result<Value, String> {
    let g = adapter::from_json_value(geom)?;
    surrealgis_functions::output::st_as_geojson(&g)
        .map(Value::from)
        .map_err(|e| e.to_string())
}

// #[surrealism]
/// Convert a geometry to Extended WKT (with SRID prefix).
pub fn st_as_ewkt(geom: &Value) -> Result<Value, String> {
    let g = adapter::from_json_value(geom)?;
    surrealgis_functions::output::st_as_ewkt(&g)
        .map(Value::from)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn point_json() -> Value {
        json!({"type": "Point", "coordinates": [1.0, 2.0]})
    }

    fn linestring_json() -> Value {
        json!({
            "type": "LineString",
            "coordinates": [[0.0, 0.0], [1.0, 1.0]]
        })
    }

    fn polygon_json() -> Value {
        json!({
            "type": "Polygon",
            "coordinates": [[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 0.0]]]
        })
    }

    #[test]
    fn test_st_as_text_point() {
        let result = st_as_text(&point_json()).unwrap();
        let wkt = result.as_str().unwrap();
        assert!(wkt.contains("POINT"));
        assert!(wkt.contains("1"));
        assert!(wkt.contains("2"));
    }

    #[test]
    fn test_st_as_text_linestring() {
        let result = st_as_text(&linestring_json()).unwrap();
        let wkt = result.as_str().unwrap();
        assert!(wkt.contains("LINESTRING"));
    }

    #[test]
    fn test_st_as_text_polygon() {
        let result = st_as_text(&polygon_json()).unwrap();
        let wkt = result.as_str().unwrap();
        assert!(wkt.contains("POLYGON"));
    }

    #[test]
    fn test_st_as_wkb() {
        let result = st_as_wkb(&point_json()).unwrap();
        let hex = result.as_str().unwrap();
        assert!(!hex.is_empty());
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_st_as_geojson() {
        let result = st_as_geojson(&point_json()).unwrap();
        let json_str = result.as_str().unwrap();
        assert!(json_str.contains("Point"));
        assert!(json_str.contains("coordinates"));
    }

    #[test]
    fn test_st_as_geojson_roundtrip() {
        let result = st_as_geojson(&point_json()).unwrap();
        let json_str = result.as_str().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
        assert_eq!(parsed["type"], "Point");
    }

    #[test]
    fn test_st_as_ewkt() {
        let result = st_as_ewkt(&point_json()).unwrap();
        let ewkt = result.as_str().unwrap();
        assert!(ewkt.starts_with("SRID=4326;"));
        assert!(ewkt.contains("POINT"));
    }

    #[test]
    fn invalid_geojson_fails() {
        assert!(st_as_text(&json!(42)).is_err());
        assert!(st_as_wkb(&json!("bad")).is_err());
        assert!(st_as_geojson(&json!(null)).is_err());
        assert!(st_as_ewkt(&json!([])).is_err());
    }
}
