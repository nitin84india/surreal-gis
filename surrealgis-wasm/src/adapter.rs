use serde_json::Value;
use surrealgis_core::geometry::SurrealGeometry;
use surrealgis_core::serialization::geojson;

/// Convert a GeoJSON-like JSON value into a domain SurrealGeometry.
pub fn from_json_value(val: &Value) -> Result<SurrealGeometry, String> {
    geojson::from_geojson(val).map_err(|e| e.to_string())
}

/// Convert a domain SurrealGeometry into a GeoJSON JSON value.
pub fn to_json_value(geom: &SurrealGeometry) -> Result<Value, String> {
    geojson::to_geojson(geom).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn roundtrip_point() {
        let input = json!({
            "type": "Point",
            "coordinates": [1.0, 2.0]
        });
        let geom = from_json_value(&input).unwrap();
        assert_eq!(geom.type_name(), "Point");
        let output = to_json_value(&geom).unwrap();
        assert_eq!(output["type"], "Point");
        let coords = output["coordinates"].as_array().unwrap();
        assert_eq!(coords[0].as_f64().unwrap(), 1.0);
        assert_eq!(coords[1].as_f64().unwrap(), 2.0);
    }

    #[test]
    fn roundtrip_linestring() {
        let input = json!({
            "type": "LineString",
            "coordinates": [[0.0, 0.0], [1.0, 1.0], [2.0, 0.0]]
        });
        let geom = from_json_value(&input).unwrap();
        assert_eq!(geom.type_name(), "LineString");
        let output = to_json_value(&geom).unwrap();
        assert_eq!(output["type"], "LineString");
    }

    #[test]
    fn roundtrip_polygon() {
        let input = json!({
            "type": "Polygon",
            "coordinates": [[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 0.0]]]
        });
        let geom = from_json_value(&input).unwrap();
        assert_eq!(geom.type_name(), "Polygon");
        let output = to_json_value(&geom).unwrap();
        assert_eq!(output["type"], "Polygon");
    }

    #[test]
    fn invalid_json_returns_error() {
        let input = json!({"foo": "bar"});
        assert!(from_json_value(&input).is_err());
    }

    #[test]
    fn missing_coordinates_returns_error() {
        let input = json!({"type": "Point"});
        assert!(from_json_value(&input).is_err());
    }
}
