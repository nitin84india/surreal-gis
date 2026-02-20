use geo_types::{Geometry, MultiLineString, MultiPoint, MultiPolygon};
use surrealgis_core::geometry::SurrealGeometry;

use crate::FunctionError;

/// Promote a single geometry to its Multi equivalent:
/// - Point -> MultiPoint(1)
/// - LineString -> MultiLineString(1)
/// - Polygon -> MultiPolygon(1)
/// - Already Multi types are returned as-is.
/// - GeometryCollection and other types are unsupported.
pub fn st_multi(geom: &SurrealGeometry) -> Result<SurrealGeometry, FunctionError> {
    let geo_geom = geom.to_geo()?;
    let result = match geo_geom {
        Geometry::Point(p) => Geometry::MultiPoint(MultiPoint(vec![p])),
        Geometry::LineString(l) => Geometry::MultiLineString(MultiLineString(vec![l])),
        Geometry::Polygon(p) => Geometry::MultiPolygon(MultiPolygon(vec![p])),
        Geometry::MultiPoint(_) | Geometry::MultiLineString(_) | Geometry::MultiPolygon(_) => {
            geo_geom
        }
        _ => {
            return Err(FunctionError::UnsupportedOperation(
                "st_multi: unsupported geometry type".to_string(),
            ))
        }
    };
    SurrealGeometry::from_geo(&result, *geom.srid()).map_err(FunctionError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::coordinate::Coordinate;
    use surrealgis_core::geometry::{GeometryType, PolygonData};
    use surrealgis_core::srid::Srid;

    #[test]
    fn multi_point() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        let result = st_multi(&p).unwrap();
        assert_eq!(result.type_name(), "MultiPoint");
        assert_eq!(result.num_points(), 1);
    }

    #[test]
    fn multi_linestring() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
        ];
        let line = SurrealGeometry::line_string(coords, Srid::WGS84).unwrap();
        let result = st_multi(&line).unwrap();
        assert_eq!(result.type_name(), "MultiLineString");
        if let GeometryType::MultiLineString(lines) = result.geometry_type() {
            assert_eq!(lines.len(), 1);
            assert_eq!(lines[0].len(), 2);
        } else {
            panic!("Expected MultiLineString");
        }
    }

    #[test]
    fn multi_polygon() {
        let exterior = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(10.0, 0.0).unwrap(),
            Coordinate::new(10.0, 10.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        let poly = SurrealGeometry::polygon(exterior, vec![], Srid::WGS84).unwrap();
        let result = st_multi(&poly).unwrap();
        assert_eq!(result.type_name(), "MultiPolygon");
    }

    #[test]
    fn multi_already_multi_point() {
        let coords = vec![
            Coordinate::new(1.0, 2.0).unwrap(),
            Coordinate::new(3.0, 4.0).unwrap(),
        ];
        let mp = SurrealGeometry::multi_point(coords, Srid::WGS84).unwrap();
        let result = st_multi(&mp).unwrap();
        assert_eq!(result.type_name(), "MultiPoint");
        assert_eq!(result.num_points(), 2);
    }

    #[test]
    fn multi_already_multi_polygon() {
        let polygons = vec![PolygonData {
            exterior: vec![
                Coordinate::new(0.0, 0.0).unwrap(),
                Coordinate::new(1.0, 0.0).unwrap(),
                Coordinate::new(1.0, 1.0).unwrap(),
                Coordinate::new(0.0, 0.0).unwrap(),
            ],
            holes: vec![],
        }];
        let mp = SurrealGeometry::multi_polygon(polygons, Srid::WGS84).unwrap();
        let result = st_multi(&mp).unwrap();
        assert_eq!(result.type_name(), "MultiPolygon");
    }

    #[test]
    fn multi_geometry_collection_rejected() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        let gc = SurrealGeometry::geometry_collection(vec![p], Srid::WGS84).unwrap();
        let result = st_multi(&gc);
        assert!(result.is_err());
    }

    #[test]
    fn multi_preserves_srid() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WEB_MERCATOR).unwrap();
        let result = st_multi(&p).unwrap();
        assert_eq!(result.srid().code(), Srid::WEB_MERCATOR.code());
    }
}
