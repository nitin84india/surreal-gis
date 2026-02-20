use geo::SimplifyVwPreserve;
use surrealgis_core::geometry::SurrealGeometry;

use crate::FunctionError;

/// Simplify a geometry using the Visvalingam-Whyatt algorithm while preserving topology.
/// The tolerance parameter controls the minimum area threshold for point removal.
/// Supported types: LineString, MultiLineString, Polygon, MultiPolygon.
/// Point and MultiPoint are returned unchanged (nothing to simplify).
pub fn st_simplify_preserve_topology(
    geom: &SurrealGeometry,
    tolerance: f64,
) -> Result<SurrealGeometry, FunctionError> {
    if tolerance < 0.0 {
        return Err(FunctionError::InvalidArgument(
            "st_simplify_preserve_topology tolerance must be non-negative".to_string(),
        ));
    }

    let geo_geom = geom.to_geo()?;
    let simplified = simplify_vw_geometry(&geo_geom, tolerance)?;
    SurrealGeometry::from_geo(&simplified, *geom.srid()).map_err(FunctionError::from)
}

fn simplify_vw_geometry(
    geom: &geo_types::Geometry<f64>,
    tolerance: f64,
) -> Result<geo_types::Geometry<f64>, FunctionError> {
    match geom {
        geo_types::Geometry::Point(_) | geo_types::Geometry::MultiPoint(_) => {
            Ok(geom.clone())
        }
        geo_types::Geometry::LineString(ls) => {
            Ok(geo_types::Geometry::LineString(ls.simplify_vw_preserve(tolerance)))
        }
        geo_types::Geometry::MultiLineString(mls) => {
            Ok(geo_types::Geometry::MultiLineString(mls.simplify_vw_preserve(tolerance)))
        }
        geo_types::Geometry::Polygon(poly) => {
            Ok(geo_types::Geometry::Polygon(poly.simplify_vw_preserve(tolerance)))
        }
        geo_types::Geometry::MultiPolygon(mp) => {
            Ok(geo_types::Geometry::MultiPolygon(mp.simplify_vw_preserve(tolerance)))
        }
        geo_types::Geometry::GeometryCollection(gc) => {
            let simplified: Result<Vec<geo_types::Geometry<f64>>, FunctionError> = gc
                .0
                .iter()
                .map(|g| simplify_vw_geometry(g, tolerance))
                .collect();
            Ok(geo_types::Geometry::GeometryCollection(
                geo_types::GeometryCollection(simplified?),
            ))
        }
        _ => Err(FunctionError::UnsupportedOperation(
            "st_simplify_preserve_topology does not support this geometry type".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::coordinate::Coordinate;
    use surrealgis_core::srid::Srid;

    #[test]
    fn simplify_vw_linestring() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(0.5, 0.01).unwrap(), // nearly collinear
            Coordinate::new(1.0, 0.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WEB_MERCATOR).unwrap();
        let simplified = st_simplify_preserve_topology(&ls, 1.0).unwrap();
        assert_eq!(simplified.type_name(), "LineString");
    }

    #[test]
    fn simplify_vw_polygon() {
        let exterior = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(5.0, 0.01).unwrap(),
            Coordinate::new(10.0, 0.0).unwrap(),
            Coordinate::new(10.0, 10.0).unwrap(),
            Coordinate::new(0.0, 10.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        let poly = SurrealGeometry::polygon(exterior, vec![], Srid::WEB_MERCATOR).unwrap();
        let simplified = st_simplify_preserve_topology(&poly, 5.0).unwrap();
        assert_eq!(simplified.type_name(), "Polygon");
    }

    #[test]
    fn simplify_vw_zero_tolerance_preserves_all() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
            Coordinate::new(2.0, 0.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WEB_MERCATOR).unwrap();
        let simplified = st_simplify_preserve_topology(&ls, 0.0).unwrap();
        assert_eq!(simplified.num_points(), 3);
    }

    #[test]
    fn simplify_vw_negative_tolerance_rejected() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WEB_MERCATOR).unwrap();
        let result = st_simplify_preserve_topology(&ls, -1.0);
        assert!(matches!(result, Err(FunctionError::InvalidArgument(_))));
    }

    #[test]
    fn simplify_vw_point_unchanged() {
        let pt = SurrealGeometry::point(1.0, 2.0, Srid::WEB_MERCATOR).unwrap();
        let simplified = st_simplify_preserve_topology(&pt, 1.0).unwrap();
        assert_eq!(simplified.type_name(), "Point");
    }

    #[test]
    fn simplify_vw_preserves_srid() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
            Coordinate::new(2.0, 0.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WEB_MERCATOR).unwrap();
        let simplified = st_simplify_preserve_topology(&ls, 0.1).unwrap();
        assert_eq!(simplified.srid().code(), Srid::WEB_MERCATOR.code());
    }
}
