use serde_json::Value;

use crate::adapter;

// #[surrealism]
/// Create a Point geometry from x/y coordinates (default SRID 4326).
pub fn st_point(x: f64, y: f64) -> Result<Value, String> {
    let geom = surrealgis_functions::constructors::st_point(x, y, 4326)
        .map_err(|e| e.to_string())?;
    adapter::to_json_value(&geom)
}

// #[surrealism]
/// Alias for st_point.
pub fn st_make_point(x: f64, y: f64) -> Result<Value, String> {
    let geom = surrealgis_functions::constructors::st_make_point(x, y, 4326)
        .map_err(|e| e.to_string())?;
    adapter::to_json_value(&geom)
}

// #[surrealism]
/// Create a LineString from an array of coordinate pairs (as JSON array of [x,y] arrays).
pub fn st_make_line(coords: &Value) -> Result<Value, String> {
    let arr = coords
        .as_array()
        .ok_or_else(|| "st_make_line expects an array of coordinate pairs".to_string())?;

    let points: Result<Vec<(f64, f64)>, String> = arr
        .iter()
        .map(|v| {
            let pair = v
                .as_array()
                .ok_or_else(|| "Each coordinate must be an [x, y] array".to_string())?;
            if pair.len() < 2 {
                return Err("Each coordinate must have at least 2 values".to_string());
            }
            let x = pair[0]
                .as_f64()
                .ok_or_else(|| "x must be a number".to_string())?;
            let y = pair[1]
                .as_f64()
                .ok_or_else(|| "y must be a number".to_string())?;
            Ok((x, y))
        })
        .collect();

    let geom = surrealgis_functions::constructors::st_make_line(&points?, 4326)
        .map_err(|e| e.to_string())?;
    adapter::to_json_value(&geom)
}

// #[surrealism]
/// Create a Polygon from an exterior ring (JSON array of [x,y]) and optional holes.
pub fn st_make_polygon(exterior: &Value, holes: &Value) -> Result<Value, String> {
    let ext_arr = exterior
        .as_array()
        .ok_or_else(|| "exterior must be an array of coordinate pairs".to_string())?;

    let ext_coords: Result<Vec<(f64, f64)>, String> = ext_arr
        .iter()
        .map(parse_coord_pair)
        .collect();

    let hole_rings: Vec<Vec<(f64, f64)>> = if holes.is_null() || holes.is_array() && holes.as_array().unwrap().is_empty() {
        vec![]
    } else {
        let hole_arr = holes
            .as_array()
            .ok_or_else(|| "holes must be an array of rings".to_string())?;
        let mut rings = Vec::new();
        for ring_val in hole_arr {
            let ring_arr = ring_val
                .as_array()
                .ok_or_else(|| "Each hole ring must be an array of coordinate pairs".to_string())?;
            let ring_coords: Result<Vec<(f64, f64)>, String> = ring_arr
                .iter()
                .map(parse_coord_pair)
                .collect();
            rings.push(ring_coords?);
        }
        rings
    };

    let geom = surrealgis_functions::constructors::st_make_polygon(&ext_coords?, &hole_rings, 4326)
        .map_err(|e| e.to_string())?;
    adapter::to_json_value(&geom)
}

// #[surrealism]
/// Create a rectangular Polygon (envelope) from bounding box coordinates.
pub fn st_make_envelope(xmin: f64, ymin: f64, xmax: f64, ymax: f64) -> Result<Value, String> {
    let geom = surrealgis_functions::constructors::st_make_envelope(xmin, ymin, xmax, ymax, 4326)
        .map_err(|e| e.to_string())?;
    adapter::to_json_value(&geom)
}

fn parse_coord_pair(val: &Value) -> Result<(f64, f64), String> {
    let pair = val
        .as_array()
        .ok_or_else(|| "Each coordinate must be an [x, y] array".to_string())?;
    if pair.len() < 2 {
        return Err("Each coordinate must have at least 2 values".to_string());
    }
    let x = pair[0]
        .as_f64()
        .ok_or_else(|| "x must be a number".to_string())?;
    let y = pair[1]
        .as_f64()
        .ok_or_else(|| "y must be a number".to_string())?;
    Ok((x, y))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn create_point() {
        let result = st_point(1.0, 2.0).unwrap();
        assert_eq!(result["type"], "Point");
        let coords = result["coordinates"].as_array().unwrap();
        assert_eq!(coords[0].as_f64().unwrap(), 1.0);
        assert_eq!(coords[1].as_f64().unwrap(), 2.0);
    }

    #[test]
    fn create_make_point() {
        let result = st_make_point(3.0, 4.0).unwrap();
        assert_eq!(result["type"], "Point");
    }

    #[test]
    fn create_line() {
        let coords = json!([[0.0, 0.0], [1.0, 1.0], [2.0, 0.0]]);
        let result = st_make_line(&coords).unwrap();
        assert_eq!(result["type"], "LineString");
        let line_coords = result["coordinates"].as_array().unwrap();
        assert_eq!(line_coords.len(), 3);
    }

    #[test]
    fn create_line_too_few_points() {
        let coords = json!([[0.0, 0.0]]);
        assert!(st_make_line(&coords).is_err());
    }

    #[test]
    fn create_polygon() {
        let exterior = json!([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 0.0]]);
        let holes = json!([]);
        let result = st_make_polygon(&exterior, &holes).unwrap();
        assert_eq!(result["type"], "Polygon");
    }

    #[test]
    fn create_polygon_with_hole() {
        let exterior = json!([
            [0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0], [0.0, 0.0]
        ]);
        let holes = json!([
            [[2.0, 2.0], [8.0, 2.0], [8.0, 8.0], [2.0, 8.0], [2.0, 2.0]]
        ]);
        let result = st_make_polygon(&exterior, &holes).unwrap();
        assert_eq!(result["type"], "Polygon");
    }

    #[test]
    fn create_envelope() {
        let result = st_make_envelope(0.0, 0.0, 10.0, 10.0).unwrap();
        assert_eq!(result["type"], "Polygon");
    }

    #[test]
    fn create_envelope_invalid_range() {
        assert!(st_make_envelope(10.0, 0.0, 0.0, 10.0).is_err());
    }

    #[test]
    fn point_nan_coordinate_fails() {
        assert!(st_point(f64::NAN, 2.0).is_err());
    }
}
