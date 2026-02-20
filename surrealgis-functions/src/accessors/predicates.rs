use surrealgis_core::geometry::{GeometryType, SurrealGeometry};

use crate::FunctionError;

/// Check if the geometry is empty (has no coordinates).
pub fn st_is_empty(geom: &SurrealGeometry) -> bool {
    geom.is_empty()
}

/// Check if the geometry is valid.
/// Uses geo crate's validation where available, otherwise does basic checks.
pub fn st_is_valid(geom: &SurrealGeometry) -> Result<bool, FunctionError> {
    match geom.geometry_type() {
        GeometryType::Point(_) => Ok(true),
        GeometryType::LineString(coords) => Ok(coords.len() >= 2),
        GeometryType::Polygon { exterior, holes } => {
            // Exterior must have at least 4 points (3 + closing)
            if exterior.len() < 4 {
                return Ok(false);
            }
            // Exterior must be closed
            if exterior.first() != exterior.last() {
                return Ok(false);
            }
            // Each hole must also be valid
            for hole in holes {
                if hole.len() < 4 {
                    return Ok(false);
                }
                if hole.first() != hole.last() {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        GeometryType::MultiPoint(coords) => Ok(!coords.is_empty()),
        GeometryType::MultiLineString(lines) => {
            Ok(!lines.is_empty() && lines.iter().all(|l| l.len() >= 2))
        }
        GeometryType::MultiPolygon(polygons) => {
            Ok(!polygons.is_empty()
                && polygons.iter().all(|p| {
                    p.exterior.len() >= 4
                        && p.exterior.first() == p.exterior.last()
                }))
        }
        GeometryType::GeometryCollection(geoms) => {
            for g in geoms {
                if !st_is_valid(g)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
    }
}

/// Check if a LineString is closed (first point == last point).
pub fn st_is_closed(geom: &SurrealGeometry) -> Result<bool, FunctionError> {
    match geom.geometry_type() {
        GeometryType::LineString(coords) => {
            if coords.len() < 2 {
                return Ok(false);
            }
            Ok(coords.first() == coords.last())
        }
        GeometryType::MultiLineString(lines) => {
            // All linestrings must be closed
            Ok(lines.iter().all(|l| l.len() >= 2 && l.first() == l.last()))
        }
        _ => Err(FunctionError::InvalidArgument(
            "st_is_closed requires a LineString or MultiLineString".to_string(),
        )),
    }
}

/// Check if a LineString is a ring (closed and simple).
/// For simplicity, we check closed and >= 4 points.
pub fn st_is_ring(geom: &SurrealGeometry) -> Result<bool, FunctionError> {
    match geom.geometry_type() {
        GeometryType::LineString(coords) => {
            if coords.len() < 4 {
                return Ok(false);
            }
            Ok(coords.first() == coords.last())
        }
        _ => Err(FunctionError::InvalidArgument(
            "st_is_ring requires a LineString geometry".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::coordinate::Coordinate;
    use surrealgis_core::srid::Srid;

    #[test]
    fn point_is_not_empty() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        assert!(!st_is_empty(&p));
    }

    #[test]
    fn point_is_valid() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        assert!(st_is_valid(&p).unwrap());
    }

    #[test]
    fn polygon_is_valid() {
        let exterior = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        let poly = SurrealGeometry::polygon(exterior, vec![], Srid::WGS84).unwrap();
        assert!(st_is_valid(&poly).unwrap());
    }

    #[test]
    fn closed_linestring() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WGS84).unwrap();
        assert!(st_is_closed(&ls).unwrap());
    }

    #[test]
    fn open_linestring() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
            Coordinate::new(2.0, 0.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WGS84).unwrap();
        assert!(!st_is_closed(&ls).unwrap());
    }

    #[test]
    fn ring_linestring() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WGS84).unwrap();
        assert!(st_is_ring(&ls).unwrap());
    }

    #[test]
    fn non_ring_too_few_points() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 0.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WGS84).unwrap();
        assert!(!st_is_ring(&ls).unwrap());
    }

    #[test]
    fn is_closed_on_point_fails() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        assert!(st_is_closed(&p).is_err());
    }
}
