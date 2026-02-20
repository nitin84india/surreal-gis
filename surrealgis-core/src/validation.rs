use crate::coordinate::Coordinate;
use crate::error::GeometryError;
use crate::geometry::{GeometryType, SurrealGeometry};

/// Validate that a linestring has at least 2 points.
pub fn validate_linestring(coords: &[Coordinate]) -> Result<(), GeometryError> {
    if coords.len() < 2 {
        return Err(GeometryError::InvalidGeometry(format!(
            "LineString requires at least 2 points, got {}",
            coords.len()
        )));
    }
    Ok(())
}

/// Validate a polygon: exterior ring must have at least 4 points and be closed.
/// Each hole must also be a valid ring.
pub fn validate_polygon(
    exterior: &[Coordinate],
    holes: &[Vec<Coordinate>],
) -> Result<(), GeometryError> {
    validate_ring(exterior)?;
    for (i, hole) in holes.iter().enumerate() {
        validate_ring(hole).map_err(|e| {
            GeometryError::InvalidGeometry(format!("Hole {i}: {e}"))
        })?;
    }
    Ok(())
}

/// Validate that a ring has at least 4 points and is closed
/// (first == last).
pub fn validate_ring(ring: &[Coordinate]) -> Result<(), GeometryError> {
    if ring.len() < 4 {
        return Err(GeometryError::InvalidGeometry(format!(
            "Ring requires at least 4 points, got {}",
            ring.len()
        )));
    }
    let first = &ring[0];
    let last = &ring[ring.len() - 1];
    if (first.x() - last.x()).abs() > f64::EPSILON
        || (first.y() - last.y()).abs() > f64::EPSILON
    {
        return Err(GeometryError::InvalidGeometry(
            "Ring is not closed (first and last points must be equal)".to_string(),
        ));
    }
    Ok(())
}

/// Check if a geometry is valid (delegates to type-specific validation).
pub fn is_valid_geometry(geom: &SurrealGeometry) -> bool {
    match geom.geometry_type() {
        GeometryType::Point(_) => true,
        GeometryType::LineString(coords) => validate_linestring(coords).is_ok(),
        GeometryType::Polygon { exterior, holes } => {
            validate_polygon(exterior, holes).is_ok()
        }
        GeometryType::MultiPoint(coords) => !coords.is_empty(),
        GeometryType::MultiLineString(lines) => {
            !lines.is_empty() && lines.iter().all(|l| validate_linestring(l).is_ok())
        }
        GeometryType::MultiPolygon(polygons) => {
            !polygons.is_empty()
                && polygons
                    .iter()
                    .all(|p| validate_polygon(&p.exterior, &p.holes).is_ok())
        }
        GeometryType::GeometryCollection(geoms) => {
            !geoms.is_empty() && geoms.iter().all(is_valid_geometry)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::srid::Srid;

    fn coord(x: f64, y: f64) -> Coordinate {
        Coordinate::new(x, y).unwrap()
    }

    #[test]
    fn valid_linestring() {
        let coords = vec![coord(0.0, 0.0), coord(1.0, 1.0)];
        assert!(validate_linestring(&coords).is_ok());
    }

    #[test]
    fn linestring_too_few_points() {
        let coords = vec![coord(0.0, 0.0)];
        assert!(validate_linestring(&coords).is_err());
    }

    #[test]
    fn linestring_empty() {
        let coords: Vec<Coordinate> = vec![];
        assert!(validate_linestring(&coords).is_err());
    }

    #[test]
    fn valid_ring() {
        let ring = vec![
            coord(0.0, 0.0),
            coord(1.0, 0.0),
            coord(1.0, 1.0),
            coord(0.0, 0.0),
        ];
        assert!(validate_ring(&ring).is_ok());
    }

    #[test]
    fn ring_too_few_points() {
        let ring = vec![coord(0.0, 0.0), coord(1.0, 0.0), coord(0.0, 0.0)];
        assert!(validate_ring(&ring).is_err());
    }

    #[test]
    fn ring_not_closed() {
        let ring = vec![
            coord(0.0, 0.0),
            coord(1.0, 0.0),
            coord(1.0, 1.0),
            coord(0.5, 0.5),
        ];
        assert!(validate_ring(&ring).is_err());
    }

    #[test]
    fn valid_polygon_no_holes() {
        let exterior = vec![
            coord(0.0, 0.0),
            coord(1.0, 0.0),
            coord(1.0, 1.0),
            coord(0.0, 0.0),
        ];
        assert!(validate_polygon(&exterior, &[]).is_ok());
    }

    #[test]
    fn polygon_with_valid_hole() {
        let exterior = vec![
            coord(0.0, 0.0),
            coord(10.0, 0.0),
            coord(10.0, 10.0),
            coord(0.0, 0.0),
        ];
        let hole = vec![
            coord(2.0, 2.0),
            coord(3.0, 2.0),
            coord(3.0, 3.0),
            coord(2.0, 2.0),
        ];
        assert!(validate_polygon(&exterior, &[hole]).is_ok());
    }

    #[test]
    fn polygon_with_invalid_hole() {
        let exterior = vec![
            coord(0.0, 0.0),
            coord(10.0, 0.0),
            coord(10.0, 10.0),
            coord(0.0, 0.0),
        ];
        let bad_hole = vec![coord(2.0, 2.0), coord(3.0, 2.0)]; // too few
        assert!(validate_polygon(&exterior, &[bad_hole]).is_err());
    }

    #[test]
    fn is_valid_point() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        assert!(is_valid_geometry(&p));
    }

    #[test]
    fn is_valid_linestring_geom() {
        let ls = SurrealGeometry::line_string(
            vec![coord(0.0, 0.0), coord(1.0, 1.0)],
            Srid::WGS84,
        )
        .unwrap();
        assert!(is_valid_geometry(&ls));
    }

    #[test]
    fn is_valid_polygon_geom() {
        let poly = SurrealGeometry::polygon(
            vec![
                coord(0.0, 0.0),
                coord(1.0, 0.0),
                coord(1.0, 1.0),
                coord(0.0, 0.0),
            ],
            vec![],
            Srid::WGS84,
        )
        .unwrap();
        assert!(is_valid_geometry(&poly));
    }
}
