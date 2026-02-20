use serde_json::Value;

use crate::adapter;

// #[surrealism]
/// Transform (reproject) a geometry from its current SRID to a target SRID.
pub fn st_transform(geom: &Value, to_srid: i32) -> Result<Value, String> {
    let g = adapter::from_json_value(geom)?;
    let result = surrealgis_functions::crs::st_transform(&g, to_srid)
        .map_err(|e| e.to_string())?;
    adapter::to_json_value(&result)
}

// #[surrealism]
/// Change the SRID metadata of a geometry without reprojecting coordinates.
pub fn st_set_srid(geom: &Value, new_srid: i32) -> Result<Value, String> {
    let g = adapter::from_json_value(geom)?;
    let result = surrealgis_functions::crs::st_set_srid(&g, new_srid)
        .map_err(|e| e.to_string())?;
    adapter::to_json_value(&result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn point_4326() -> Value {
        json!({"type": "Point", "coordinates": [-73.9857, 40.7484]})
    }

    #[test]
    fn test_st_transform_4326_to_3857() {
        let result = st_transform(&point_4326(), 3857).unwrap();
        assert_eq!(result["type"], "Point");
        let coords = result["coordinates"].as_array().unwrap();
        let x = coords[0].as_f64().unwrap();
        let y = coords[1].as_f64().unwrap();
        // Web Mercator coords should be large numbers
        assert!(x.abs() > 1_000_000.0, "x was {x}");
        assert!(y.abs() > 1_000_000.0, "y was {y}");
    }

    #[test]
    fn test_st_transform_same_srid_fails() {
        // from_geojson defaults to SRID 4326, so transforming to 4326 should fail
        assert!(st_transform(&point_4326(), 4326).is_err());
    }

    #[test]
    fn test_st_set_srid() {
        let result = st_set_srid(&point_4326(), 3857).unwrap();
        assert_eq!(result["type"], "Point");
        let coords = result["coordinates"].as_array().unwrap();
        // Coordinates should remain unchanged
        assert_eq!(coords[0].as_f64().unwrap(), -73.9857);
        assert_eq!(coords[1].as_f64().unwrap(), 40.7484);
    }

    #[test]
    fn test_st_set_srid_invalid() {
        assert!(st_set_srid(&point_4326(), 0).is_err());
    }

    #[test]
    fn invalid_geojson_fails() {
        assert!(st_transform(&json!(42), 3857).is_err());
        assert!(st_set_srid(&json!("bad"), 4326).is_err());
    }
}
