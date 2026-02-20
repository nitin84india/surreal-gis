use serde_json::Value;
use surrealgis_core::geometry::SurrealGeometry;

use crate::adapter;

/// Extract an f64 from a JSON value.
pub fn extract_f64(val: &Value) -> Result<f64, String> {
    val.as_f64()
        .ok_or_else(|| format!("Expected a number, got: {val}"))
}

/// Extract an i32 from a JSON value.
pub fn extract_i32(val: &Value) -> Result<i32, String> {
    val.as_i64()
        .and_then(|n| i32::try_from(n).ok())
        .ok_or_else(|| format!("Expected an integer, got: {val}"))
}

/// Extract a string from a JSON value.
pub fn extract_string(val: &Value) -> Result<String, String> {
    val.as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Expected a string, got: {val}"))
}

/// Extract a SurrealGeometry from a GeoJSON-like JSON value.
pub fn extract_geometry(val: &Value) -> Result<SurrealGeometry, String> {
    adapter::from_json_value(val)
}

/// Extract an optional i32 from an optional JSON value.
pub fn extract_optional_i32(val: Option<&Value>) -> Result<Option<i32>, String> {
    match val {
        None => Ok(None),
        Some(v) => extract_i32(v).map(Some),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extract_f64_from_number() {
        assert_eq!(extract_f64(&json!(3.14)).unwrap(), 3.14);
    }

    #[test]
    fn extract_f64_from_integer() {
        assert_eq!(extract_f64(&json!(42)).unwrap(), 42.0);
    }

    #[test]
    fn extract_f64_from_string_fails() {
        assert!(extract_f64(&json!("not a number")).is_err());
    }

    #[test]
    fn extract_i32_from_integer() {
        assert_eq!(extract_i32(&json!(42)).unwrap(), 42);
    }

    #[test]
    fn extract_i32_from_float_fails() {
        assert!(extract_i32(&json!(3.14)).is_err());
    }

    #[test]
    fn extract_i32_from_string_fails() {
        assert!(extract_i32(&json!("not a number")).is_err());
    }

    #[test]
    fn extract_string_from_string() {
        assert_eq!(extract_string(&json!("hello")).unwrap(), "hello");
    }

    #[test]
    fn extract_string_from_number_fails() {
        assert!(extract_string(&json!(42)).is_err());
    }

    #[test]
    fn extract_geometry_from_geojson() {
        let val = json!({"type": "Point", "coordinates": [1.0, 2.0]});
        let geom = extract_geometry(&val).unwrap();
        assert_eq!(geom.type_name(), "Point");
    }

    #[test]
    fn extract_geometry_from_invalid_fails() {
        assert!(extract_geometry(&json!(42)).is_err());
    }

    #[test]
    fn extract_optional_i32_none() {
        assert_eq!(extract_optional_i32(None).unwrap(), None);
    }

    #[test]
    fn extract_optional_i32_some() {
        assert_eq!(extract_optional_i32(Some(&json!(4326))).unwrap(), Some(4326));
    }

    #[test]
    fn extract_optional_i32_invalid() {
        assert!(extract_optional_i32(Some(&json!("nope"))).is_err());
    }
}
