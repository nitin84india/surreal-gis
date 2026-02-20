use serde_json::Value;

use crate::adapter;

// #[surrealism]
/// Extract the X coordinate of a Point geometry.
pub fn st_x(geom: &Value) -> Result<Value, String> {
    let g = adapter::from_json_value(geom)?;
    surrealgis_functions::accessors::st_x(&g)
        .map(Value::from)
        .map_err(|e| e.to_string())
}

// #[surrealism]
/// Extract the Y coordinate of a Point geometry.
pub fn st_y(geom: &Value) -> Result<Value, String> {
    let g = adapter::from_json_value(geom)?;
    surrealgis_functions::accessors::st_y(&g)
        .map(Value::from)
        .map_err(|e| e.to_string())
}

// #[surrealism]
/// Extract the Z coordinate of a Point geometry (null if 2D).
pub fn st_z(geom: &Value) -> Result<Value, String> {
    let g = adapter::from_json_value(geom)?;
    surrealgis_functions::accessors::st_z(&g)
        .map(|opt| match opt {
            Some(z) => Value::from(z),
            None => Value::Null,
        })
        .map_err(|e| e.to_string())
}

// #[surrealism]
/// Return the SRID of a geometry.
pub fn st_srid(geom: &Value) -> Result<Value, String> {
    let g = adapter::from_json_value(geom)?;
    Ok(Value::from(surrealgis_functions::accessors::st_srid(&g)))
}

// #[surrealism]
/// Return the geometry type name as a string.
pub fn st_geometry_type(geom: &Value) -> Result<Value, String> {
    let g = adapter::from_json_value(geom)?;
    Ok(Value::from(surrealgis_functions::accessors::st_geometry_type(&g).to_string()))
}

// #[surrealism]
/// Return the total number of points in the geometry.
pub fn st_num_points(geom: &Value) -> Result<Value, String> {
    let g = adapter::from_json_value(geom)?;
    Ok(Value::from(surrealgis_functions::accessors::st_num_points(&g) as u64))
}

// #[surrealism]
/// Return the topological dimension (0=point, 1=line, 2=polygon).
pub fn st_dimension(geom: &Value) -> Result<Value, String> {
    let g = adapter::from_json_value(geom)?;
    Ok(Value::from(surrealgis_functions::accessors::st_dimension(&g) as u64))
}

// #[surrealism]
/// Return the first point of a LineString as a GeoJSON geometry.
pub fn st_start_point(geom: &Value) -> Result<Value, String> {
    let g = adapter::from_json_value(geom)?;
    let result = surrealgis_functions::accessors::st_start_point(&g)
        .map_err(|e| e.to_string())?;
    adapter::to_json_value(&result)
}

// #[surrealism]
/// Return the last point of a LineString as a GeoJSON geometry.
pub fn st_end_point(geom: &Value) -> Result<Value, String> {
    let g = adapter::from_json_value(geom)?;
    let result = surrealgis_functions::accessors::st_end_point(&g)
        .map_err(|e| e.to_string())?;
    adapter::to_json_value(&result)
}

// #[surrealism]
/// Check if the geometry is empty.
pub fn st_is_empty(geom: &Value) -> Result<Value, String> {
    let g = adapter::from_json_value(geom)?;
    Ok(Value::from(surrealgis_functions::accessors::st_is_empty(&g)))
}

// #[surrealism]
/// Check if the geometry is valid.
pub fn st_is_valid(geom: &Value) -> Result<Value, String> {
    let g = adapter::from_json_value(geom)?;
    surrealgis_functions::accessors::st_is_valid(&g)
        .map(Value::from)
        .map_err(|e| e.to_string())
}

// #[surrealism]
/// Check if a LineString is closed.
pub fn st_is_closed(geom: &Value) -> Result<Value, String> {
    let g = adapter::from_json_value(geom)?;
    surrealgis_functions::accessors::st_is_closed(&g)
        .map(Value::from)
        .map_err(|e| e.to_string())
}

// #[surrealism]
/// Check if a LineString is a ring (closed and >= 4 points).
pub fn st_is_ring(geom: &Value) -> Result<Value, String> {
    let g = adapter::from_json_value(geom)?;
    surrealgis_functions::accessors::st_is_ring(&g)
        .map(Value::from)
        .map_err(|e| e.to_string())
}

// #[surrealism]
/// Return the bounding box of a geometry as a Polygon.
pub fn st_envelope(geom: &Value) -> Result<Value, String> {
    let g = adapter::from_json_value(geom)?;
    let result = surrealgis_functions::accessors::st_envelope(&g)
        .map_err(|e| e.to_string())?;
    adapter::to_json_value(&result)
}

// #[surrealism]
/// Return the centroid of a geometry as a Point.
pub fn st_centroid(geom: &Value) -> Result<Value, String> {
    let g = adapter::from_json_value(geom)?;
    let result = surrealgis_functions::accessors::st_centroid(&g)
        .map_err(|e| e.to_string())?;
    adapter::to_json_value(&result)
}

// #[surrealism]
/// Return a point guaranteed to lie on the surface of the geometry.
pub fn st_point_on_surface(geom: &Value) -> Result<Value, String> {
    let g = adapter::from_json_value(geom)?;
    let result = surrealgis_functions::accessors::st_point_on_surface(&g)
        .map_err(|e| e.to_string())?;
    adapter::to_json_value(&result)
}

// #[surrealism]
/// Return the boundary of a geometry.
pub fn st_boundary(geom: &Value) -> Result<Value, String> {
    let g = adapter::from_json_value(geom)?;
    let result = surrealgis_functions::accessors::st_boundary(&g)
        .map_err(|e| e.to_string())?;
    adapter::to_json_value(&result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn point_json() -> Value {
        json!({"type": "Point", "coordinates": [1.5, 2.5]})
    }

    fn linestring_json() -> Value {
        json!({
            "type": "LineString",
            "coordinates": [[0.0, 0.0], [1.0, 1.0], [2.0, 0.0]]
        })
    }

    fn polygon_json() -> Value {
        json!({
            "type": "Polygon",
            "coordinates": [[[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0], [0.0, 0.0]]]
        })
    }

    fn closed_linestring_json() -> Value {
        json!({
            "type": "LineString",
            "coordinates": [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 0.0]]
        })
    }

    #[test]
    fn test_st_x() {
        let result = st_x(&point_json()).unwrap();
        assert_eq!(result.as_f64().unwrap(), 1.5);
    }

    #[test]
    fn test_st_y() {
        let result = st_y(&point_json()).unwrap();
        assert_eq!(result.as_f64().unwrap(), 2.5);
    }

    #[test]
    fn test_st_z_2d() {
        let result = st_z(&point_json()).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_st_srid() {
        let result = st_srid(&point_json()).unwrap();
        assert_eq!(result.as_i64().unwrap(), 4326);
    }

    #[test]
    fn test_st_geometry_type_point() {
        let result = st_geometry_type(&point_json()).unwrap();
        assert_eq!(result.as_str().unwrap(), "Point");
    }

    #[test]
    fn test_st_geometry_type_linestring() {
        let result = st_geometry_type(&linestring_json()).unwrap();
        assert_eq!(result.as_str().unwrap(), "LineString");
    }

    #[test]
    fn test_st_num_points_point() {
        let result = st_num_points(&point_json()).unwrap();
        assert_eq!(result.as_u64().unwrap(), 1);
    }

    #[test]
    fn test_st_num_points_linestring() {
        let result = st_num_points(&linestring_json()).unwrap();
        assert_eq!(result.as_u64().unwrap(), 3);
    }

    #[test]
    fn test_st_dimension_point() {
        let result = st_dimension(&point_json()).unwrap();
        assert_eq!(result.as_u64().unwrap(), 0);
    }

    #[test]
    fn test_st_dimension_polygon() {
        let result = st_dimension(&polygon_json()).unwrap();
        assert_eq!(result.as_u64().unwrap(), 2);
    }

    #[test]
    fn test_st_start_point() {
        let result = st_start_point(&linestring_json()).unwrap();
        assert_eq!(result["type"], "Point");
        let coords = result["coordinates"].as_array().unwrap();
        assert_eq!(coords[0].as_f64().unwrap(), 0.0);
        assert_eq!(coords[1].as_f64().unwrap(), 0.0);
    }

    #[test]
    fn test_st_end_point() {
        let result = st_end_point(&linestring_json()).unwrap();
        assert_eq!(result["type"], "Point");
        let coords = result["coordinates"].as_array().unwrap();
        assert_eq!(coords[0].as_f64().unwrap(), 2.0);
        assert_eq!(coords[1].as_f64().unwrap(), 0.0);
    }

    #[test]
    fn test_st_start_point_on_point_fails() {
        assert!(st_start_point(&point_json()).is_err());
    }

    #[test]
    fn test_st_x_on_linestring_fails() {
        assert!(st_x(&linestring_json()).is_err());
    }

    #[test]
    fn test_st_is_empty() {
        let result = st_is_empty(&point_json()).unwrap();
        assert_eq!(result.as_bool().unwrap(), false);
    }

    #[test]
    fn test_st_is_valid_point() {
        let result = st_is_valid(&point_json()).unwrap();
        assert_eq!(result.as_bool().unwrap(), true);
    }

    #[test]
    fn test_st_is_valid_polygon() {
        let result = st_is_valid(&polygon_json()).unwrap();
        assert_eq!(result.as_bool().unwrap(), true);
    }

    #[test]
    fn test_st_is_closed() {
        let result = st_is_closed(&closed_linestring_json()).unwrap();
        assert_eq!(result.as_bool().unwrap(), true);
    }

    #[test]
    fn test_st_is_closed_open() {
        let result = st_is_closed(&linestring_json()).unwrap();
        assert_eq!(result.as_bool().unwrap(), false);
    }

    #[test]
    fn test_st_is_ring() {
        let result = st_is_ring(&closed_linestring_json()).unwrap();
        assert_eq!(result.as_bool().unwrap(), true);
    }

    #[test]
    fn test_st_envelope() {
        let result = st_envelope(&polygon_json()).unwrap();
        assert_eq!(result["type"], "Polygon");
    }

    #[test]
    fn test_st_centroid() {
        let result = st_centroid(&polygon_json()).unwrap();
        assert_eq!(result["type"], "Point");
        let coords = result["coordinates"].as_array().unwrap();
        let x = coords[0].as_f64().unwrap();
        let y = coords[1].as_f64().unwrap();
        assert!((x - 5.0).abs() < 1e-10);
        assert!((y - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_st_point_on_surface() {
        let result = st_point_on_surface(&polygon_json()).unwrap();
        assert_eq!(result["type"], "Point");
    }

    #[test]
    fn test_st_boundary_polygon() {
        let result = st_boundary(&polygon_json()).unwrap();
        assert_eq!(result["type"], "LineString");
    }

    #[test]
    fn test_st_boundary_open_linestring() {
        let result = st_boundary(&linestring_json()).unwrap();
        assert_eq!(result["type"], "MultiPoint");
    }

    #[test]
    fn invalid_geojson_fails() {
        let bad = json!(42);
        assert!(st_x(&bad).is_err());
        assert!(st_y(&bad).is_err());
        assert!(st_srid(&bad).is_err());
    }
}
