use surrealgis_core::geometry::SurrealGeometry;
use surrealgis_core::serialization::geojson;

use crate::FunctionError;

/// Convert a geometry to GeoJSON string.
pub fn st_as_geojson(geom: &SurrealGeometry) -> Result<String, FunctionError> {
    let value = geojson::to_geojson(geom).map_err(FunctionError::from)?;
    serde_json::to_string(&value).map_err(|e| FunctionError::InvalidArgument(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::srid::Srid;

    #[test]
    fn point_to_geojson() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        let json = st_as_geojson(&p).unwrap();
        assert!(json.contains("Point"));
        assert!(json.contains("coordinates"));
    }

    #[test]
    fn geojson_roundtrip() {
        let p = SurrealGeometry::point(-73.9857, 40.7484, Srid::WGS84).unwrap();
        let json = st_as_geojson(&p).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["type"], "Point");
    }
}
