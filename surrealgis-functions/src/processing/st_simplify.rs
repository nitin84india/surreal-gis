use geo::Simplify;
use surrealgis_core::geometry::SurrealGeometry;

use crate::FunctionError;

/// Simplify a geometry using the Ramer-Douglas-Peucker algorithm.
/// The tolerance parameter controls the maximum distance a point can deviate
/// from the simplified line.
/// Supported types: LineString, MultiLineString, Polygon, MultiPolygon.
/// Point and MultiPoint are returned unchanged (nothing to simplify).
pub fn st_simplify(
    geom: &SurrealGeometry,
    tolerance: f64,
) -> Result<SurrealGeometry, FunctionError> {
    if tolerance < 0.0 {
        return Err(FunctionError::InvalidArgument(
            "st_simplify tolerance must be non-negative".to_string(),
        ));
    }

    let geo_geom = geom.to_geo()?;
    let simplified = simplify_geometry(&geo_geom, tolerance)?;
    SurrealGeometry::from_geo(&simplified, *geom.srid()).map_err(FunctionError::from)
}

fn simplify_geometry(
    geom: &geo_types::Geometry<f64>,
    tolerance: f64,
) -> Result<geo_types::Geometry<f64>, FunctionError> {
    match geom {
        geo_types::Geometry::Point(_) | geo_types::Geometry::MultiPoint(_) => {
            // Points cannot be simplified, return as-is
            Ok(geom.clone())
        }
        geo_types::Geometry::LineString(ls) => {
            Ok(geo_types::Geometry::LineString(ls.simplify(tolerance)))
        }
        geo_types::Geometry::MultiLineString(mls) => {
            Ok(geo_types::Geometry::MultiLineString(mls.simplify(tolerance)))
        }
        geo_types::Geometry::Polygon(poly) => {
            Ok(geo_types::Geometry::Polygon(poly.simplify(tolerance)))
        }
        geo_types::Geometry::MultiPolygon(mp) => {
            Ok(geo_types::Geometry::MultiPolygon(mp.simplify(tolerance)))
        }
        geo_types::Geometry::GeometryCollection(gc) => {
            let simplified: Result<Vec<geo_types::Geometry<f64>>, FunctionError> = gc
                .0
                .iter()
                .map(|g| simplify_geometry(g, tolerance))
                .collect();
            Ok(geo_types::Geometry::GeometryCollection(
                geo_types::GeometryCollection(simplified?),
            ))
        }
        _ => Err(FunctionError::UnsupportedOperation(
            "st_simplify does not support this geometry type".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::coordinate::Coordinate;
    use surrealgis_core::srid::Srid;

    #[test]
    fn simplify_linestring() {
        // Create a linestring with a point very close to the line between start and end
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(0.5, 0.01).unwrap(), // nearly collinear
            Coordinate::new(1.0, 0.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WEB_MERCATOR).unwrap();

        // With a large tolerance, the middle point should be removed
        let simplified = st_simplify(&ls, 1.0).unwrap();
        assert_eq!(simplified.type_name(), "LineString");
        assert!(simplified.num_points() <= 3);
    }

    #[test]
    fn simplify_polygon() {
        let exterior = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(5.0, 0.01).unwrap(), // nearly collinear
            Coordinate::new(10.0, 0.0).unwrap(),
            Coordinate::new(10.0, 10.0).unwrap(),
            Coordinate::new(0.0, 10.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        let poly = SurrealGeometry::polygon(exterior, vec![], Srid::WEB_MERCATOR).unwrap();
        let simplified = st_simplify(&poly, 1.0).unwrap();
        assert_eq!(simplified.type_name(), "Polygon");
        assert!(simplified.num_points() <= 6);
    }

    #[test]
    fn simplify_zero_tolerance_preserves_all() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
            Coordinate::new(2.0, 0.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WEB_MERCATOR).unwrap();
        let simplified = st_simplify(&ls, 0.0).unwrap();
        assert_eq!(simplified.num_points(), 3);
    }

    #[test]
    fn simplify_negative_tolerance_rejected() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WEB_MERCATOR).unwrap();
        let result = st_simplify(&ls, -1.0);
        assert!(matches!(result, Err(FunctionError::InvalidArgument(_))));
    }

    #[test]
    fn simplify_point_unchanged() {
        let pt = SurrealGeometry::point(1.0, 2.0, Srid::WEB_MERCATOR).unwrap();
        let simplified = st_simplify(&pt, 1.0).unwrap();
        assert_eq!(simplified.type_name(), "Point");
    }

    #[test]
    fn simplify_preserves_srid() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
            Coordinate::new(2.0, 0.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WEB_MERCATOR).unwrap();
        let simplified = st_simplify(&ls, 0.1).unwrap();
        assert_eq!(simplified.srid().code(), Srid::WEB_MERCATOR.code());
    }
}
