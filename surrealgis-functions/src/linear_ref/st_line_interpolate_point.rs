use geo::{Euclidean, InterpolateLine};
use surrealgis_core::geometry::SurrealGeometry;

use crate::FunctionError;

/// Returns a point interpolated along a line at a given fraction.
/// Fraction 0.0 returns the start point, 1.0 returns the end point.
pub fn st_line_interpolate_point(
    geom: &SurrealGeometry,
    fraction: f64,
) -> Result<SurrealGeometry, FunctionError> {
    if !(0.0..=1.0).contains(&fraction) {
        return Err(FunctionError::InvalidArgument(format!(
            "Fraction must be between 0.0 and 1.0, got {fraction}"
        )));
    }

    let geo_geom = geom.to_geo()?;
    match geo_geom {
        geo_types::Geometry::LineString(ref line) => {
            let pt = Euclidean.point_at_ratio_from_start(line, fraction).ok_or_else(|| {
                FunctionError::InvalidArgument(
                    "Cannot interpolate point on empty line".into(),
                )
            })?;
            let result = geo_types::Geometry::Point(pt);
            SurrealGeometry::from_geo(&result, *geom.srid()).map_err(FunctionError::from)
        }
        _ => Err(FunctionError::UnsupportedOperation(
            "st_line_interpolate_point requires a LineString input".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::coordinate::Coordinate;
    use surrealgis_core::geometry::GeometryType;
    use surrealgis_core::srid::Srid;

    fn make_line() -> SurrealGeometry {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(10.0, 0.0).unwrap(),
        ];
        SurrealGeometry::line_string(coords, Srid::WEB_MERCATOR).unwrap()
    }

    #[test]
    fn interpolate_at_start() {
        let line = make_line();
        let result = st_line_interpolate_point(&line, 0.0).unwrap();
        match result.geometry_type() {
            GeometryType::Point(c) => {
                assert!((c.x() - 0.0).abs() < 1e-6);
                assert!((c.y() - 0.0).abs() < 1e-6);
            }
            _ => panic!("Expected Point"),
        }
    }

    #[test]
    fn interpolate_at_midpoint() {
        let line = make_line();
        let result = st_line_interpolate_point(&line, 0.5).unwrap();
        match result.geometry_type() {
            GeometryType::Point(c) => {
                assert!((c.x() - 5.0).abs() < 1e-6);
                assert!((c.y() - 0.0).abs() < 1e-6);
            }
            _ => panic!("Expected Point"),
        }
    }

    #[test]
    fn interpolate_at_end() {
        let line = make_line();
        let result = st_line_interpolate_point(&line, 1.0).unwrap();
        match result.geometry_type() {
            GeometryType::Point(c) => {
                assert!((c.x() - 10.0).abs() < 1e-6);
                assert!((c.y() - 0.0).abs() < 1e-6);
            }
            _ => panic!("Expected Point"),
        }
    }

    #[test]
    fn interpolate_preserves_srid() {
        let line = make_line();
        let result = st_line_interpolate_point(&line, 0.5).unwrap();
        assert_eq!(result.srid().code(), Srid::WEB_MERCATOR.code());
    }

    #[test]
    fn interpolate_fraction_below_zero_rejected() {
        let line = make_line();
        let result = st_line_interpolate_point(&line, -0.1);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FunctionError::InvalidArgument(_)));
    }

    #[test]
    fn interpolate_fraction_above_one_rejected() {
        let line = make_line();
        let result = st_line_interpolate_point(&line, 1.1);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FunctionError::InvalidArgument(_)));
    }

    #[test]
    fn interpolate_non_linestring_rejected() {
        let point = SurrealGeometry::point(1.0, 2.0, Srid::WEB_MERCATOR).unwrap();
        let result = st_line_interpolate_point(&point, 0.5);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            FunctionError::UnsupportedOperation(_)
        ));
    }

    #[test]
    fn interpolate_multi_segment_line() {
        // Line: (0,0) -> (10,0) -> (10,10), total length = 20
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(10.0, 0.0).unwrap(),
            Coordinate::new(10.0, 10.0).unwrap(),
        ];
        let line = SurrealGeometry::line_string(coords, Srid::WEB_MERCATOR).unwrap();
        // fraction=0.5 should be at (10,0) - midpoint of total length 20
        let result = st_line_interpolate_point(&line, 0.5).unwrap();
        match result.geometry_type() {
            GeometryType::Point(c) => {
                assert!((c.x() - 10.0).abs() < 1e-6);
                assert!((c.y() - 0.0).abs() < 1e-6);
            }
            _ => panic!("Expected Point"),
        }
    }
}
