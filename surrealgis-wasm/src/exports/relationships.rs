use serde_json::Value;

use crate::adapter;

// #[surrealism]
/// Returns true if the two geometries spatially intersect.
pub fn st_intersects(a: &Value, b: &Value) -> Result<Value, String> {
    let ga = adapter::from_json_value(a)?;
    let gb = adapter::from_json_value(b)?;
    surrealgis_functions::relationships::st_intersects(&ga, &gb)
        .map(Value::from)
        .map_err(|e| e.to_string())
}

// #[surrealism]
/// Returns true if geometry A contains geometry B.
pub fn st_contains(a: &Value, b: &Value) -> Result<Value, String> {
    let ga = adapter::from_json_value(a)?;
    let gb = adapter::from_json_value(b)?;
    surrealgis_functions::relationships::st_contains(&ga, &gb)
        .map(Value::from)
        .map_err(|e| e.to_string())
}

// #[surrealism]
/// Returns true if geometry A is within geometry B.
pub fn st_within(a: &Value, b: &Value) -> Result<Value, String> {
    let ga = adapter::from_json_value(a)?;
    let gb = adapter::from_json_value(b)?;
    surrealgis_functions::relationships::st_within(&ga, &gb)
        .map(Value::from)
        .map_err(|e| e.to_string())
}

// #[surrealism]
/// Returns true if the geometries touch (share boundary but not interior).
pub fn st_touches(a: &Value, b: &Value) -> Result<Value, String> {
    let ga = adapter::from_json_value(a)?;
    let gb = adapter::from_json_value(b)?;
    surrealgis_functions::relationships::st_touches(&ga, &gb)
        .map(Value::from)
        .map_err(|e| e.to_string())
}

// #[surrealism]
/// Returns true if the geometries cross each other.
pub fn st_crosses(a: &Value, b: &Value) -> Result<Value, String> {
    let ga = adapter::from_json_value(a)?;
    let gb = adapter::from_json_value(b)?;
    surrealgis_functions::relationships::st_crosses(&ga, &gb)
        .map(Value::from)
        .map_err(|e| e.to_string())
}

// #[surrealism]
/// Returns true if the geometries overlap.
pub fn st_overlaps(a: &Value, b: &Value) -> Result<Value, String> {
    let ga = adapter::from_json_value(a)?;
    let gb = adapter::from_json_value(b)?;
    surrealgis_functions::relationships::st_overlaps(&ga, &gb)
        .map(Value::from)
        .map_err(|e| e.to_string())
}

// #[surrealism]
/// Returns true if the geometries are spatially disjoint.
pub fn st_disjoint(a: &Value, b: &Value) -> Result<Value, String> {
    let ga = adapter::from_json_value(a)?;
    let gb = adapter::from_json_value(b)?;
    surrealgis_functions::relationships::st_disjoint(&ga, &gb)
        .map(Value::from)
        .map_err(|e| e.to_string())
}

// #[surrealism]
/// Returns true if the geometries are topologically equal.
pub fn st_equals(a: &Value, b: &Value) -> Result<Value, String> {
    let ga = adapter::from_json_value(a)?;
    let gb = adapter::from_json_value(b)?;
    surrealgis_functions::relationships::st_equals(&ga, &gb)
        .map(Value::from)
        .map_err(|e| e.to_string())
}

// #[surrealism]
/// Returns true if geometry A covers geometry B.
pub fn st_covers(a: &Value, b: &Value) -> Result<Value, String> {
    let ga = adapter::from_json_value(a)?;
    let gb = adapter::from_json_value(b)?;
    surrealgis_functions::relationships::st_covers(&ga, &gb)
        .map(Value::from)
        .map_err(|e| e.to_string())
}

// #[surrealism]
/// Returns true if geometry A is covered by geometry B.
pub fn st_covered_by(a: &Value, b: &Value) -> Result<Value, String> {
    let ga = adapter::from_json_value(a)?;
    let gb = adapter::from_json_value(b)?;
    surrealgis_functions::relationships::st_covered_by(&ga, &gb)
        .map(Value::from)
        .map_err(|e| e.to_string())
}

// #[surrealism]
/// Returns the DE-9IM intersection matrix string.
pub fn st_relate(a: &Value, b: &Value) -> Result<Value, String> {
    let ga = adapter::from_json_value(a)?;
    let gb = adapter::from_json_value(b)?;
    surrealgis_functions::relationships::st_relate(&ga, &gb)
        .map(Value::from)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn poly_a() -> Value {
        json!({
            "type": "Polygon",
            "coordinates": [[[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0], [0.0, 0.0]]]
        })
    }

    fn poly_b() -> Value {
        json!({
            "type": "Polygon",
            "coordinates": [[[1.0, 1.0], [3.0, 1.0], [3.0, 3.0], [1.0, 3.0], [1.0, 1.0]]]
        })
    }

    fn poly_far() -> Value {
        json!({
            "type": "Polygon",
            "coordinates": [[[50.0, 50.0], [51.0, 50.0], [51.0, 51.0], [50.0, 51.0], [50.0, 50.0]]]
        })
    }

    fn point_inside_a() -> Value {
        json!({"type": "Point", "coordinates": [0.5, 0.5]})
    }

    #[test]
    fn overlapping_intersect() {
        let result = st_intersects(&poly_a(), &poly_b()).unwrap();
        assert_eq!(result.as_bool().unwrap(), true);
    }

    #[test]
    fn far_dont_intersect() {
        let result = st_intersects(&poly_a(), &poly_far()).unwrap();
        assert_eq!(result.as_bool().unwrap(), false);
    }

    #[test]
    fn contains_point() {
        let result = st_contains(&poly_a(), &point_inside_a()).unwrap();
        assert_eq!(result.as_bool().unwrap(), true);
    }

    #[test]
    fn point_within_polygon() {
        let result = st_within(&point_inside_a(), &poly_a()).unwrap();
        assert_eq!(result.as_bool().unwrap(), true);
    }

    #[test]
    fn far_disjoint() {
        let result = st_disjoint(&poly_a(), &poly_far()).unwrap();
        assert_eq!(result.as_bool().unwrap(), true);
    }

    #[test]
    fn overlapping_not_disjoint() {
        let result = st_disjoint(&poly_a(), &poly_b()).unwrap();
        assert_eq!(result.as_bool().unwrap(), false);
    }

    #[test]
    fn self_equals() {
        let result = st_equals(&poly_a(), &poly_a()).unwrap();
        assert_eq!(result.as_bool().unwrap(), true);
    }

    #[test]
    fn covers_point() {
        let result = st_covers(&poly_a(), &point_inside_a()).unwrap();
        assert_eq!(result.as_bool().unwrap(), true);
    }

    #[test]
    fn covered_by_polygon() {
        let result = st_covered_by(&point_inside_a(), &poly_a()).unwrap();
        assert_eq!(result.as_bool().unwrap(), true);
    }

    #[test]
    fn relate_returns_string() {
        let result = st_relate(&poly_a(), &poly_b()).unwrap();
        let matrix = result.as_str().unwrap();
        assert_eq!(matrix.len(), 9);
    }

    #[test]
    fn touching_polygons() {
        let a = json!({
            "type": "Polygon",
            "coordinates": [[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.0, 0.0]]]
        });
        let b = json!({
            "type": "Polygon",
            "coordinates": [[[1.0, 0.0], [2.0, 0.0], [2.0, 1.0], [1.0, 1.0], [1.0, 0.0]]]
        });
        let result = st_touches(&a, &b).unwrap();
        assert_eq!(result.as_bool().unwrap(), true);
    }

    #[test]
    fn crossing_lines() {
        let line_a = json!({
            "type": "LineString",
            "coordinates": [[0.0, 0.0], [2.0, 2.0]]
        });
        let line_b = json!({
            "type": "LineString",
            "coordinates": [[0.0, 2.0], [2.0, 0.0]]
        });
        let result = st_crosses(&line_a, &line_b).unwrap();
        assert_eq!(result.as_bool().unwrap(), true);
    }

    #[test]
    fn invalid_geojson_fails() {
        assert!(st_intersects(&json!(42), &json!(43)).is_err());
    }
}
