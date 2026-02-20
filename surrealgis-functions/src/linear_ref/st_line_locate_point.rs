use geo::LineLocatePoint;
use surrealgis_core::geometry::SurrealGeometry;

use crate::FunctionError;

/// Returns a fraction (0.0 to 1.0) representing the location of the closest point
/// on a line to the given point, as a fraction of the line's total length.
pub fn st_line_locate_point(
    line_geom: &SurrealGeometry,
    point_geom: &SurrealGeometry,
) -> Result<f64, FunctionError> {
    let geo_line = line_geom.to_geo()?;
    let geo_point = point_geom.to_geo()?;

    match (&geo_line, &geo_point) {
        (geo_types::Geometry::LineString(line), geo_types::Geometry::Point(point)) => {
            let fraction = line.line_locate_point(point).ok_or_else(|| {
                FunctionError::InvalidArgument(
                    "Cannot locate point on empty line".into(),
                )
            })?;
            Ok(fraction)
        }
        _ => Err(FunctionError::UnsupportedOperation(
            "st_line_locate_point requires LineString and Point inputs".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::coordinate::Coordinate;
    use surrealgis_core::srid::Srid;

    fn make_line() -> SurrealGeometry {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(10.0, 0.0).unwrap(),
        ];
        SurrealGeometry::line_string(coords, Srid::WEB_MERCATOR).unwrap()
    }

    #[test]
    fn locate_point_at_start() {
        let line = make_line();
        let point = SurrealGeometry::point(0.0, 0.0, Srid::WEB_MERCATOR).unwrap();
        let fraction = st_line_locate_point(&line, &point).unwrap();
        assert!((fraction - 0.0).abs() < 1e-6);
    }

    #[test]
    fn locate_point_at_midpoint() {
        let line = make_line();
        let point = SurrealGeometry::point(5.0, 0.0, Srid::WEB_MERCATOR).unwrap();
        let fraction = st_line_locate_point(&line, &point).unwrap();
        assert!((fraction - 0.5).abs() < 1e-6);
    }

    #[test]
    fn locate_point_at_end() {
        let line = make_line();
        let point = SurrealGeometry::point(10.0, 0.0, Srid::WEB_MERCATOR).unwrap();
        let fraction = st_line_locate_point(&line, &point).unwrap();
        assert!((fraction - 1.0).abs() < 1e-6);
    }

    #[test]
    fn locate_point_off_line() {
        let line = make_line();
        // Point perpendicular to midpoint, should still project to 0.5
        let point = SurrealGeometry::point(5.0, 5.0, Srid::WEB_MERCATOR).unwrap();
        let fraction = st_line_locate_point(&line, &point).unwrap();
        assert!((fraction - 0.5).abs() < 1e-6);
    }

    #[test]
    fn locate_point_before_line() {
        let line = make_line();
        // Point before the start of line projects to 0.0
        let point = SurrealGeometry::point(-5.0, 0.0, Srid::WEB_MERCATOR).unwrap();
        let fraction = st_line_locate_point(&line, &point).unwrap();
        assert!((fraction - 0.0).abs() < 1e-6);
    }

    #[test]
    fn locate_point_after_line() {
        let line = make_line();
        // Point after end of line projects to 1.0
        let point = SurrealGeometry::point(15.0, 0.0, Srid::WEB_MERCATOR).unwrap();
        let fraction = st_line_locate_point(&line, &point).unwrap();
        assert!((fraction - 1.0).abs() < 1e-6);
    }

    #[test]
    fn locate_non_linestring_rejected() {
        let point1 = SurrealGeometry::point(1.0, 2.0, Srid::WEB_MERCATOR).unwrap();
        let point2 = SurrealGeometry::point(3.0, 4.0, Srid::WEB_MERCATOR).unwrap();
        let result = st_line_locate_point(&point1, &point2);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            FunctionError::UnsupportedOperation(_)
        ));
    }

    #[test]
    fn locate_non_point_rejected() {
        let line = make_line();
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
        ];
        let line2 = SurrealGeometry::line_string(coords, Srid::WEB_MERCATOR).unwrap();
        let result = st_line_locate_point(&line, &line2);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            FunctionError::UnsupportedOperation(_)
        ));
    }
}
