use geo::line_measures::LengthMeasurable;
use geo::Euclidean;
use geo_types::{Coord, LineString, Point};
use surrealgis_core::geometry::SurrealGeometry;

use crate::FunctionError;

/// Interpolate a coordinate at a given distance along a LineString.
fn interpolate_along(line: &LineString<f64>, target_dist: f64) -> Coord<f64> {
    let mut accumulated = 0.0;
    for window in line.0.windows(2) {
        let seg_start = window[0];
        let seg_end = window[1];
        let seg_len =
            ((seg_end.x - seg_start.x).powi(2) + (seg_end.y - seg_start.y).powi(2)).sqrt();
        let next_accumulated = accumulated + seg_len;
        if target_dist <= next_accumulated {
            let t = if seg_len > 0.0 {
                (target_dist - accumulated) / seg_len
            } else {
                0.0
            };
            return Coord {
                x: seg_start.x + t * (seg_end.x - seg_start.x),
                y: seg_start.y + t * (seg_end.y - seg_start.y),
            };
        }
        accumulated = next_accumulated;
    }
    // Fallback: return last coordinate
    *line.0.last().unwrap_or(&Coord { x: 0.0, y: 0.0 })
}

/// Returns a substring of a line between two fractions of its total length.
/// Both fractions must be between 0.0 and 1.0, and start_fraction must be <= end_fraction.
/// If start_fraction == end_fraction, returns a Point at that location.
pub fn st_line_substring(
    geom: &SurrealGeometry,
    start_fraction: f64,
    end_fraction: f64,
) -> Result<SurrealGeometry, FunctionError> {
    if !(0.0..=1.0).contains(&start_fraction) || !(0.0..=1.0).contains(&end_fraction) {
        return Err(FunctionError::InvalidArgument(
            "Fractions must be between 0.0 and 1.0".into(),
        ));
    }
    if start_fraction > end_fraction {
        return Err(FunctionError::InvalidArgument(
            "Start fraction must be <= end fraction".into(),
        ));
    }

    let geo_geom = geom.to_geo()?;
    match geo_geom {
        geo_types::Geometry::LineString(ref line) => {
            let total_length = line.length(&Euclidean);
            if total_length == 0.0 {
                return Err(FunctionError::InvalidArgument(
                    "Cannot substring a zero-length line".into(),
                ));
            }

            // Degenerate case: equal fractions produce a single point
            if (start_fraction - end_fraction).abs() < f64::EPSILON {
                let dist = start_fraction * total_length;
                let pt = interpolate_along(line, dist);
                let result = geo_types::Geometry::Point(Point::new(pt.x, pt.y));
                return SurrealGeometry::from_geo(&result, *geom.srid())
                    .map_err(FunctionError::from);
            }

            let start_dist = start_fraction * total_length;
            let end_dist = end_fraction * total_length;

            let mut coords: Vec<Coord<f64>> = Vec::new();
            let mut accumulated = 0.0;
            let mut started = false;

            for window in line.0.windows(2) {
                let seg_start = window[0];
                let seg_end = window[1];
                let seg_len = ((seg_end.x - seg_start.x).powi(2)
                    + (seg_end.y - seg_start.y).powi(2))
                .sqrt();
                let next_accumulated = accumulated + seg_len;

                // Check if start point is in this segment
                if !started && accumulated <= start_dist && start_dist <= next_accumulated {
                    let t = if seg_len > 0.0 {
                        (start_dist - accumulated) / seg_len
                    } else {
                        0.0
                    };
                    coords.push(Coord {
                        x: seg_start.x + t * (seg_end.x - seg_start.x),
                        y: seg_start.y + t * (seg_end.y - seg_start.y),
                    });
                    started = true;
                }

                // Check if end point is in this segment
                if started && accumulated <= end_dist && end_dist <= next_accumulated {
                    let t = if seg_len > 0.0 {
                        (end_dist - accumulated) / seg_len
                    } else {
                        0.0
                    };
                    coords.push(Coord {
                        x: seg_start.x + t * (seg_end.x - seg_start.x),
                        y: seg_start.y + t * (seg_end.y - seg_start.y),
                    });
                    break;
                }

                // If started and haven't reached end, add segment endpoint
                if started {
                    coords.push(seg_end);
                }

                accumulated = next_accumulated;
            }

            if coords.len() < 2 {
                return Err(FunctionError::InvalidArgument(
                    "Could not compute substring".into(),
                ));
            }

            let result = geo_types::Geometry::LineString(LineString(coords));
            SurrealGeometry::from_geo(&result, *geom.srid()).map_err(FunctionError::from)
        }
        _ => Err(FunctionError::UnsupportedOperation(
            "st_line_substring requires a LineString input".into(),
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

    fn make_multi_segment_line() -> SurrealGeometry {
        // Line: (0,0) -> (10,0) -> (10,10), total length = 20
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(10.0, 0.0).unwrap(),
            Coordinate::new(10.0, 10.0).unwrap(),
        ];
        SurrealGeometry::line_string(coords, Srid::WEB_MERCATOR).unwrap()
    }

    #[test]
    fn substring_first_half() {
        let line = make_line();
        let result = st_line_substring(&line, 0.0, 0.5).unwrap();
        match result.geometry_type() {
            GeometryType::LineString(coords) => {
                assert_eq!(coords.len(), 2);
                assert!((coords[0].x() - 0.0).abs() < 1e-6);
                assert!((coords[1].x() - 5.0).abs() < 1e-6);
            }
            _ => panic!("Expected LineString"),
        }
    }

    #[test]
    fn substring_second_half() {
        let line = make_line();
        let result = st_line_substring(&line, 0.5, 1.0).unwrap();
        match result.geometry_type() {
            GeometryType::LineString(coords) => {
                assert_eq!(coords.len(), 2);
                assert!((coords[0].x() - 5.0).abs() < 1e-6);
                assert!((coords[1].x() - 10.0).abs() < 1e-6);
            }
            _ => panic!("Expected LineString"),
        }
    }

    #[test]
    fn substring_full_line() {
        let line = make_line();
        let result = st_line_substring(&line, 0.0, 1.0).unwrap();
        match result.geometry_type() {
            GeometryType::LineString(coords) => {
                assert_eq!(coords.len(), 2);
                assert!((coords[0].x() - 0.0).abs() < 1e-6);
                assert!((coords[1].x() - 10.0).abs() < 1e-6);
            }
            _ => panic!("Expected LineString"),
        }
    }

    #[test]
    fn substring_middle_quarter() {
        let line = make_line();
        let result = st_line_substring(&line, 0.25, 0.75).unwrap();
        match result.geometry_type() {
            GeometryType::LineString(coords) => {
                assert_eq!(coords.len(), 2);
                assert!((coords[0].x() - 2.5).abs() < 1e-6);
                assert!((coords[1].x() - 7.5).abs() < 1e-6);
            }
            _ => panic!("Expected LineString"),
        }
    }

    #[test]
    fn substring_same_fraction_returns_point() {
        let line = make_line();
        let result = st_line_substring(&line, 0.5, 0.5).unwrap();
        match result.geometry_type() {
            GeometryType::Point(c) => {
                assert!((c.x() - 5.0).abs() < 1e-6);
                assert!((c.y() - 0.0).abs() < 1e-6);
            }
            _ => panic!("Expected Point for equal fractions"),
        }
    }

    #[test]
    fn substring_multi_segment() {
        let line = make_multi_segment_line();
        // 0.0 to 0.5 = first 10 units = (0,0) -> (10,0)
        let result = st_line_substring(&line, 0.0, 0.5).unwrap();
        match result.geometry_type() {
            GeometryType::LineString(coords) => {
                assert_eq!(coords.len(), 2);
                assert!((coords[0].x() - 0.0).abs() < 1e-6);
                assert!((coords[0].y() - 0.0).abs() < 1e-6);
                assert!((coords[1].x() - 10.0).abs() < 1e-6);
                assert!((coords[1].y() - 0.0).abs() < 1e-6);
            }
            _ => panic!("Expected LineString"),
        }
    }

    #[test]
    fn substring_across_segments() {
        let line = make_multi_segment_line();
        // 0.25 to 0.75: start at (5,0), through (10,0), end at (10,5)
        let result = st_line_substring(&line, 0.25, 0.75).unwrap();
        match result.geometry_type() {
            GeometryType::LineString(coords) => {
                assert_eq!(coords.len(), 3);
                assert!((coords[0].x() - 5.0).abs() < 1e-6);
                assert!((coords[0].y() - 0.0).abs() < 1e-6);
                assert!((coords[1].x() - 10.0).abs() < 1e-6);
                assert!((coords[1].y() - 0.0).abs() < 1e-6);
                assert!((coords[2].x() - 10.0).abs() < 1e-6);
                assert!((coords[2].y() - 5.0).abs() < 1e-6);
            }
            _ => panic!("Expected LineString"),
        }
    }

    #[test]
    fn substring_preserves_srid() {
        let line = make_line();
        let result = st_line_substring(&line, 0.0, 0.5).unwrap();
        assert_eq!(result.srid().code(), Srid::WEB_MERCATOR.code());
    }

    #[test]
    fn substring_invalid_start_fraction() {
        let line = make_line();
        let result = st_line_substring(&line, -0.1, 0.5);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FunctionError::InvalidArgument(_)));
    }

    #[test]
    fn substring_invalid_end_fraction() {
        let line = make_line();
        let result = st_line_substring(&line, 0.0, 1.5);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FunctionError::InvalidArgument(_)));
    }

    #[test]
    fn substring_start_greater_than_end() {
        let line = make_line();
        let result = st_line_substring(&line, 0.7, 0.3);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FunctionError::InvalidArgument(_)));
    }

    #[test]
    fn substring_non_linestring_rejected() {
        let point = SurrealGeometry::point(1.0, 2.0, Srid::WEB_MERCATOR).unwrap();
        let result = st_line_substring(&point, 0.0, 0.5);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            FunctionError::UnsupportedOperation(_)
        ));
    }
}
