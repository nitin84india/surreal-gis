use serde_json::Value;

use crate::adapter;

// #[surrealism]
/// Compute distance between two geometries.
/// For geographic SRIDs, returns geodesic distance in meters.
/// For projected SRIDs, returns Euclidean distance in projection units.
pub fn st_distance(a: &Value, b: &Value) -> Result<Value, String> {
    let ga = adapter::from_json_value(a)?;
    let gb = adapter::from_json_value(b)?;
    surrealgis_functions::measurement::st_distance(&ga, &gb)
        .map(Value::from)
        .map_err(|e| e.to_string())
}

// #[surrealism]
/// Always compute geodesic distance regardless of SRID (returns meters).
/// Only supports Point-to-Point.
pub fn st_distance_sphere(a: &Value, b: &Value) -> Result<Value, String> {
    let ga = adapter::from_json_value(a)?;
    let gb = adapter::from_json_value(b)?;
    surrealgis_functions::measurement::st_distance_sphere(&ga, &gb)
        .map(Value::from)
        .map_err(|e| e.to_string())
}

// #[surrealism]
/// Compute the area of a geometry.
pub fn st_area(geom: &Value) -> Result<Value, String> {
    let g = adapter::from_json_value(geom)?;
    surrealgis_functions::measurement::st_area(&g)
        .map(Value::from)
        .map_err(|e| e.to_string())
}

// #[surrealism]
/// Compute the length of a geometry.
pub fn st_length(geom: &Value) -> Result<Value, String> {
    let g = adapter::from_json_value(geom)?;
    surrealgis_functions::measurement::st_length(&g)
        .map(Value::from)
        .map_err(|e| e.to_string())
}

// #[surrealism]
/// Compute the perimeter of a Polygon.
pub fn st_perimeter(geom: &Value) -> Result<Value, String> {
    let g = adapter::from_json_value(geom)?;
    surrealgis_functions::measurement::st_perimeter(&g)
        .map(Value::from)
        .map_err(|e| e.to_string())
}

// #[surrealism]
/// Compute the azimuth (bearing) between two points in radians.
pub fn st_azimuth(a: &Value, b: &Value) -> Result<Value, String> {
    let ga = adapter::from_json_value(a)?;
    let gb = adapter::from_json_value(b)?;
    surrealgis_functions::measurement::st_azimuth(&ga, &gb)
        .map(Value::from)
        .map_err(|e| e.to_string())
}

// #[surrealism]
/// Returns true if the geometries are within the specified distance.
pub fn st_dwithin(a: &Value, b: &Value, distance: f64) -> Result<Value, String> {
    let ga = adapter::from_json_value(a)?;
    let gb = adapter::from_json_value(b)?;
    surrealgis_functions::measurement::st_dwithin(&ga, &gb, distance)
        .map(Value::from)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn point_a() -> Value {
        json!({"type": "Point", "coordinates": [0.0, 0.0]})
    }

    fn point_b() -> Value {
        json!({"type": "Point", "coordinates": [3.0, 4.0]})
    }

    fn unit_square() -> Value {
        json!({
            "type": "Polygon",
            "coordinates": [[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.0, 0.0]]]
        })
    }

    fn linestring() -> Value {
        json!({
            "type": "LineString",
            "coordinates": [[0.0, 0.0], [3.0, 4.0]]
        })
    }

    #[test]
    fn test_st_distance() {
        let result = st_distance(&point_a(), &point_b()).unwrap();
        let d = result.as_f64().unwrap();
        // Points at (0,0) and (3,4) with SRID 4326 (geographic) -
        // From_geojson defaults to 4326 which is geographic, so geodesic distance is used
        // for point-to-point; for non-point, euclidean fallback.
        // The actual distance depends on the geodesic calculation.
        assert!(d > 0.0);
    }

    #[test]
    fn test_st_distance_same_point() {
        let result = st_distance(&point_a(), &point_a()).unwrap();
        let d = result.as_f64().unwrap();
        assert!(d.abs() < 1e-6);
    }

    #[test]
    fn test_st_distance_sphere() {
        let nyc = json!({"type": "Point", "coordinates": [-73.9857, 40.7484]});
        let la = json!({"type": "Point", "coordinates": [-118.2437, 34.0522]});
        let result = st_distance_sphere(&nyc, &la).unwrap();
        let d = result.as_f64().unwrap();
        // ~3944 km
        assert!(d > 3900000.0 && d < 4000000.0, "Distance was {d}");
    }

    #[test]
    fn test_st_area_polygon() {
        let result = st_area(&unit_square()).unwrap();
        let area = result.as_f64().unwrap();
        // For geographic SRID, area in degree-squared; the unsigned_area for
        // a unit square in degrees is 1.0
        assert!((area - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_st_area_point() {
        let result = st_area(&point_a()).unwrap();
        assert_eq!(result.as_f64().unwrap(), 0.0);
    }

    #[test]
    fn test_st_length_linestring() {
        let result = st_length(&linestring()).unwrap();
        let len = result.as_f64().unwrap();
        // Geographic length should be > 0 (geodesic for SRID 4326)
        assert!(len > 0.0);
    }

    #[test]
    fn test_st_length_point() {
        let result = st_length(&point_a()).unwrap();
        assert_eq!(result.as_f64().unwrap(), 0.0);
    }

    #[test]
    fn test_st_perimeter_polygon() {
        let result = st_perimeter(&unit_square()).unwrap();
        let perimeter = result.as_f64().unwrap();
        // Geographic perimeter in meters (geodesic)
        assert!(perimeter > 0.0);
    }

    #[test]
    fn test_st_perimeter_point() {
        let result = st_perimeter(&point_a()).unwrap();
        assert_eq!(result.as_f64().unwrap(), 0.0);
    }

    #[test]
    fn test_st_azimuth() {
        let a = json!({"type": "Point", "coordinates": [0.0, 0.0]});
        let b = json!({"type": "Point", "coordinates": [0.0, 1.0]});
        let result = st_azimuth(&a, &b).unwrap();
        let az = result.as_f64().unwrap();
        // Due north should be ~0 radians
        assert!(az.abs() < 0.01 || (az - 2.0 * std::f64::consts::PI).abs() < 0.01);
    }

    #[test]
    fn test_st_azimuth_east() {
        let a = json!({"type": "Point", "coordinates": [0.0, 0.0]});
        let b = json!({"type": "Point", "coordinates": [1.0, 0.0]});
        let result = st_azimuth(&a, &b).unwrap();
        let az = result.as_f64().unwrap();
        // Due east should be ~PI/2 radians
        assert!((az - std::f64::consts::FRAC_PI_2).abs() < 0.01);
    }

    #[test]
    fn test_st_azimuth_non_point_fails() {
        assert!(st_azimuth(&linestring(), &point_a()).is_err());
    }

    #[test]
    fn test_st_dwithin_within() {
        let a = json!({"type": "Point", "coordinates": [0.0, 0.0]});
        let b = json!({"type": "Point", "coordinates": [0.0, 0.001]});
        let result = st_dwithin(&a, &b, 200.0).unwrap();
        assert_eq!(result.as_bool().unwrap(), true);
    }

    #[test]
    fn test_st_dwithin_negative_distance_fails() {
        assert!(st_dwithin(&point_a(), &point_b(), -1.0).is_err());
    }

    #[test]
    fn invalid_geojson_fails() {
        assert!(st_distance(&json!(42), &json!(43)).is_err());
        assert!(st_area(&json!("bad")).is_err());
    }
}
