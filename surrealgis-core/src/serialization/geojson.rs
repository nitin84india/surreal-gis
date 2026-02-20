use serde_json::{json, Value};

use crate::coordinate::Coordinate;
use crate::error::GeometryError;
use crate::geometry::{GeometryType, PolygonData, SurrealGeometry};
use crate::srid::Srid;

/// Convert a SurrealGeometry to a GeoJSON geometry object (serde_json::Value).
pub fn to_geojson(geom: &SurrealGeometry) -> Result<Value, GeometryError> {
    match geom.geometry_type() {
        GeometryType::Point(coord) => Ok(json!({
            "type": "Point",
            "coordinates": coord_to_array(coord),
        })),
        GeometryType::LineString(coords) => Ok(json!({
            "type": "LineString",
            "coordinates": coords_to_arrays(coords),
        })),
        GeometryType::Polygon { exterior, holes } => {
            let mut rings = vec![coords_to_arrays(exterior)];
            for hole in holes {
                rings.push(coords_to_arrays(hole));
            }
            Ok(json!({
                "type": "Polygon",
                "coordinates": rings,
            }))
        }
        GeometryType::MultiPoint(coords) => Ok(json!({
            "type": "MultiPoint",
            "coordinates": coords_to_arrays(coords),
        })),
        GeometryType::MultiLineString(lines) => {
            let arrays: Vec<Vec<Vec<f64>>> = lines.iter().map(|l| coords_to_arrays(l)).collect();
            Ok(json!({
                "type": "MultiLineString",
                "coordinates": arrays,
            }))
        }
        GeometryType::MultiPolygon(polygons) => {
            let poly_arrays: Vec<Vec<Vec<Vec<f64>>>> = polygons
                .iter()
                .map(|p| {
                    let mut rings = vec![coords_to_arrays(&p.exterior)];
                    for hole in &p.holes {
                        rings.push(coords_to_arrays(hole));
                    }
                    rings
                })
                .collect();
            Ok(json!({
                "type": "MultiPolygon",
                "coordinates": poly_arrays,
            }))
        }
        GeometryType::GeometryCollection(geoms) => {
            let geometries: Result<Vec<Value>, GeometryError> =
                geoms.iter().map(to_geojson).collect();
            Ok(json!({
                "type": "GeometryCollection",
                "geometries": geometries?,
            }))
        }
    }
}

/// Parse a GeoJSON geometry object into a SurrealGeometry.
pub fn from_geojson(value: &Value) -> Result<SurrealGeometry, GeometryError> {
    let type_str = value
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            GeometryError::SerializationError("GeoJSON: missing 'type' field".to_string())
        })?;

    match type_str {
        "Point" => {
            let coords = get_coordinates(value)?;
            let arr = coords
                .as_array()
                .ok_or_else(|| geojson_err("Point coordinates must be an array"))?;
            let coord = parse_coord(arr)?;
            Ok(SurrealGeometry::from_parts(
                GeometryType::Point(coord),
                Srid::DEFAULT,
            ))
        }
        "LineString" => {
            let coords = get_coordinates(value)?;
            let arr = coords
                .as_array()
                .ok_or_else(|| geojson_err("LineString coordinates must be an array"))?;
            let coordinates = parse_coord_array(arr)?;
            Ok(SurrealGeometry::from_parts(
                GeometryType::LineString(coordinates),
                Srid::DEFAULT,
            ))
        }
        "Polygon" => {
            let coords = get_coordinates(value)?;
            let rings = coords
                .as_array()
                .ok_or_else(|| geojson_err("Polygon coordinates must be an array"))?;
            if rings.is_empty() {
                return Err(GeometryError::EmptyGeometry);
            }
            let exterior = parse_coord_array(
                rings[0]
                    .as_array()
                    .ok_or_else(|| geojson_err("Polygon ring must be an array"))?,
            )?;
            let mut holes = Vec::new();
            for ring in rings.iter().skip(1) {
                holes.push(parse_coord_array(
                    ring.as_array()
                        .ok_or_else(|| geojson_err("Polygon ring must be an array"))?,
                )?);
            }
            Ok(SurrealGeometry::from_parts(
                GeometryType::Polygon { exterior, holes },
                Srid::DEFAULT,
            ))
        }
        "MultiPoint" => {
            let coords = get_coordinates(value)?;
            let arr = coords
                .as_array()
                .ok_or_else(|| geojson_err("MultiPoint coordinates must be an array"))?;
            let coordinates = parse_coord_array(arr)?;
            Ok(SurrealGeometry::from_parts(
                GeometryType::MultiPoint(coordinates),
                Srid::DEFAULT,
            ))
        }
        "MultiLineString" => {
            let coords = get_coordinates(value)?;
            let lines = coords
                .as_array()
                .ok_or_else(|| geojson_err("MultiLineString coordinates must be an array"))?;
            let mut result = Vec::new();
            for line in lines {
                result.push(parse_coord_array(
                    line.as_array()
                        .ok_or_else(|| geojson_err("MultiLineString line must be an array"))?,
                )?);
            }
            Ok(SurrealGeometry::from_parts(
                GeometryType::MultiLineString(result),
                Srid::DEFAULT,
            ))
        }
        "MultiPolygon" => {
            let coords = get_coordinates(value)?;
            let polygons = coords
                .as_array()
                .ok_or_else(|| geojson_err("MultiPolygon coordinates must be an array"))?;
            let mut result = Vec::new();
            for poly in polygons {
                let rings = poly
                    .as_array()
                    .ok_or_else(|| geojson_err("MultiPolygon polygon must be an array"))?;
                if rings.is_empty() {
                    return Err(GeometryError::EmptyGeometry);
                }
                let exterior = parse_coord_array(
                    rings[0]
                        .as_array()
                        .ok_or_else(|| geojson_err("MultiPolygon ring must be an array"))?,
                )?;
                let mut holes = Vec::new();
                for ring in rings.iter().skip(1) {
                    holes.push(parse_coord_array(
                        ring.as_array()
                            .ok_or_else(|| geojson_err("MultiPolygon ring must be an array"))?,
                    )?);
                }
                result.push(PolygonData { exterior, holes });
            }
            Ok(SurrealGeometry::from_parts(
                GeometryType::MultiPolygon(result),
                Srid::DEFAULT,
            ))
        }
        "GeometryCollection" => {
            let geometries = value
                .get("geometries")
                .and_then(|v| v.as_array())
                .ok_or_else(|| {
                    geojson_err("GeometryCollection: missing 'geometries' array")
                })?;
            let geoms: Result<Vec<SurrealGeometry>, GeometryError> =
                geometries.iter().map(from_geojson).collect();
            Ok(SurrealGeometry::from_parts(
                GeometryType::GeometryCollection(geoms?),
                Srid::DEFAULT,
            ))
        }
        other => Err(GeometryError::UnsupportedGeometryType(other.to_string())),
    }
}

fn coord_to_array(coord: &Coordinate) -> Vec<f64> {
    let mut arr = vec![coord.x(), coord.y()];
    if let Some(z) = coord.z() {
        arr.push(z);
    }
    arr
}

fn coords_to_arrays(coords: &[Coordinate]) -> Vec<Vec<f64>> {
    coords.iter().map(coord_to_array).collect()
}

fn get_coordinates(value: &Value) -> Result<&Value, GeometryError> {
    value
        .get("coordinates")
        .ok_or_else(|| geojson_err("Missing 'coordinates' field"))
}

fn parse_coord(arr: &[Value]) -> Result<Coordinate, GeometryError> {
    if arr.len() < 2 {
        return Err(geojson_err("Coordinate must have at least 2 values"));
    }
    let x = arr[0]
        .as_f64()
        .ok_or_else(|| geojson_err("Coordinate x must be a number"))?;
    let y = arr[1]
        .as_f64()
        .ok_or_else(|| geojson_err("Coordinate y must be a number"))?;

    if arr.len() >= 3 {
        if let Some(z) = arr[2].as_f64() {
            return Coordinate::new_3d(x, y, z);
        }
    }
    Coordinate::new(x, y)
}

fn parse_coord_array(arr: &[Value]) -> Result<Vec<Coordinate>, GeometryError> {
    arr.iter()
        .map(|v| {
            let inner = v
                .as_array()
                .ok_or_else(|| geojson_err("Expected coordinate array"))?;
            parse_coord(inner)
        })
        .collect()
}

fn geojson_err(msg: &str) -> GeometryError {
    GeometryError::SerializationError(format!("GeoJSON: {msg}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::Coordinate;

    #[test]
    fn point_geojson_roundtrip() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        let gjson = to_geojson(&p).unwrap();
        assert_eq!(gjson["type"], "Point");
        let coords = gjson["coordinates"].as_array().unwrap();
        assert_eq!(coords[0].as_f64().unwrap(), 1.0);
        assert_eq!(coords[1].as_f64().unwrap(), 2.0);
        let roundtripped = from_geojson(&gjson).unwrap();
        assert_eq!(roundtripped.type_name(), "Point");
    }

    #[test]
    fn linestring_geojson_roundtrip() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WGS84).unwrap();
        let gjson = to_geojson(&ls).unwrap();
        assert_eq!(gjson["type"], "LineString");
        let roundtripped = from_geojson(&gjson).unwrap();
        assert_eq!(roundtripped.type_name(), "LineString");
        assert_eq!(roundtripped.num_points(), 2);
    }

    #[test]
    fn polygon_geojson_roundtrip() {
        let exterior = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        let poly = SurrealGeometry::polygon(exterior, vec![], Srid::WGS84).unwrap();
        let gjson = to_geojson(&poly).unwrap();
        assert_eq!(gjson["type"], "Polygon");
        let roundtripped = from_geojson(&gjson).unwrap();
        assert_eq!(roundtripped.type_name(), "Polygon");
        assert_eq!(roundtripped.num_points(), 4);
    }

    #[test]
    fn multi_point_geojson_roundtrip() {
        let coords = vec![
            Coordinate::new(1.0, 2.0).unwrap(),
            Coordinate::new(3.0, 4.0).unwrap(),
        ];
        let mp = SurrealGeometry::multi_point(coords, Srid::WGS84).unwrap();
        let gjson = to_geojson(&mp).unwrap();
        assert_eq!(gjson["type"], "MultiPoint");
        let roundtripped = from_geojson(&gjson).unwrap();
        assert_eq!(roundtripped.type_name(), "MultiPoint");
    }

    #[test]
    fn geometry_collection_geojson_roundtrip() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WGS84).unwrap();
        let gc = SurrealGeometry::geometry_collection(vec![p, ls], Srid::WGS84).unwrap();
        let gjson = to_geojson(&gc).unwrap();
        assert_eq!(gjson["type"], "GeometryCollection");
        let roundtripped = from_geojson(&gjson).unwrap();
        assert_eq!(roundtripped.type_name(), "GeometryCollection");
    }

    #[test]
    fn geojson_missing_type_returns_error() {
        let value = json!({"coordinates": [1, 2]});
        assert!(from_geojson(&value).is_err());
    }

    #[test]
    fn geojson_unsupported_type_returns_error() {
        let value = json!({"type": "Feature"});
        assert!(from_geojson(&value).is_err());
    }

    #[test]
    fn geojson_missing_coordinates_returns_error() {
        let value = json!({"type": "Point"});
        assert!(from_geojson(&value).is_err());
    }

    #[test]
    fn from_geojson_uses_default_srid() {
        let value = json!({
            "type": "Point",
            "coordinates": [5.0, 10.0]
        });
        let sg = from_geojson(&value).unwrap();
        assert_eq!(sg.srid().code(), 4326);
    }
}
