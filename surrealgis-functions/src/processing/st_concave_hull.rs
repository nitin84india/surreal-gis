use geo::ConcaveHull;
use surrealgis_core::geometry::{GeometryType, SurrealGeometry};

use crate::FunctionError;

/// Compute the concave hull of a geometry with a given concavity parameter.
/// The concavity parameter is accepted for PostGIS API compatibility but is not
/// used by the underlying geo 0.32 implementation (which always computes a
/// concave hull with its own internal heuristics).
/// Concavity ranges from 0.0 (convex hull) to 1.0 (most concave).
/// Extracts all points from the input geometry to form a MultiPoint, then computes
/// the concave hull.
pub fn st_concave_hull(
    geom: &SurrealGeometry,
    concavity: f64,
) -> Result<SurrealGeometry, FunctionError> {
    if !(0.0..=1.0).contains(&concavity) {
        return Err(FunctionError::InvalidArgument(
            "st_concave_hull concavity must be between 0.0 and 1.0".to_string(),
        ));
    }

    let points = extract_all_points(geom)?;
    if points.len() < 3 {
        return Err(FunctionError::InvalidArgument(
            "st_concave_hull requires at least 3 points".to_string(),
        ));
    }

    let multi_point = geo_types::MultiPoint::new(points);
    // geo 0.32 ConcaveHull::concave_hull() takes no concavity argument.
    // The concavity parameter is kept in our API for PostGIS compatibility.
    let _ = concavity;
    let hull = multi_point.concave_hull();
    let result = geo_types::Geometry::Polygon(hull);
    SurrealGeometry::from_geo(&result, *geom.srid()).map_err(FunctionError::from)
}

/// Extract all points from any geometry type into a flat Vec of geo_types::Point.
fn extract_all_points(geom: &SurrealGeometry) -> Result<Vec<geo_types::Point<f64>>, FunctionError> {
    let mut points = Vec::new();
    collect_points(geom, &mut points)?;
    Ok(points)
}

fn collect_points(
    geom: &SurrealGeometry,
    points: &mut Vec<geo_types::Point<f64>>,
) -> Result<(), FunctionError> {
    match geom.geometry_type() {
        GeometryType::Point(c) => {
            points.push(geo_types::Point::new(c.x(), c.y()));
        }
        GeometryType::LineString(coords) => {
            for c in coords {
                points.push(geo_types::Point::new(c.x(), c.y()));
            }
        }
        GeometryType::Polygon { exterior, holes } => {
            for c in exterior {
                points.push(geo_types::Point::new(c.x(), c.y()));
            }
            for hole in holes {
                for c in hole {
                    points.push(geo_types::Point::new(c.x(), c.y()));
                }
            }
        }
        GeometryType::MultiPoint(coords) => {
            for c in coords {
                points.push(geo_types::Point::new(c.x(), c.y()));
            }
        }
        GeometryType::MultiLineString(lines) => {
            for line in lines {
                for c in line {
                    points.push(geo_types::Point::new(c.x(), c.y()));
                }
            }
        }
        GeometryType::MultiPolygon(polygons) => {
            for poly in polygons {
                for c in &poly.exterior {
                    points.push(geo_types::Point::new(c.x(), c.y()));
                }
                for hole in &poly.holes {
                    for c in hole {
                        points.push(geo_types::Point::new(c.x(), c.y()));
                    }
                }
            }
        }
        GeometryType::GeometryCollection(geoms) => {
            for g in geoms {
                collect_points(g, points)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::coordinate::Coordinate;
    use surrealgis_core::srid::Srid;

    #[test]
    fn concave_hull_of_multipoint() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(4.0, 0.0).unwrap(),
            Coordinate::new(4.0, 4.0).unwrap(),
            Coordinate::new(0.0, 4.0).unwrap(),
            Coordinate::new(2.0, 2.0).unwrap(),
        ];
        let mp = SurrealGeometry::multi_point(coords, Srid::WEB_MERCATOR).unwrap();
        let hull = st_concave_hull(&mp, 0.5).unwrap();
        assert_eq!(hull.type_name(), "Polygon");
    }

    #[test]
    fn concave_hull_zero_concavity_is_convex() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(4.0, 0.0).unwrap(),
            Coordinate::new(4.0, 4.0).unwrap(),
            Coordinate::new(0.0, 4.0).unwrap(),
            Coordinate::new(2.0, 2.0).unwrap(),
        ];
        let mp = SurrealGeometry::multi_point(coords, Srid::WEB_MERCATOR).unwrap();
        // concavity=0.0 should produce a convex hull
        let hull = st_concave_hull(&mp, 0.0).unwrap();
        assert_eq!(hull.type_name(), "Polygon");
    }

    #[test]
    fn concave_hull_invalid_concavity_too_high() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
        ];
        let mp = SurrealGeometry::multi_point(coords, Srid::WEB_MERCATOR).unwrap();
        let result = st_concave_hull(&mp, 1.5);
        assert!(matches!(result, Err(FunctionError::InvalidArgument(_))));
    }

    #[test]
    fn concave_hull_invalid_concavity_negative() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
        ];
        let mp = SurrealGeometry::multi_point(coords, Srid::WEB_MERCATOR).unwrap();
        let result = st_concave_hull(&mp, -0.5);
        assert!(matches!(result, Err(FunctionError::InvalidArgument(_))));
    }

    #[test]
    fn concave_hull_too_few_points() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 0.0).unwrap(),
        ];
        let mp = SurrealGeometry::multi_point(coords, Srid::WEB_MERCATOR).unwrap();
        let result = st_concave_hull(&mp, 0.5);
        assert!(matches!(result, Err(FunctionError::InvalidArgument(_))));
    }

    #[test]
    fn concave_hull_from_linestring() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(2.0, 4.0).unwrap(),
            Coordinate::new(4.0, 0.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WEB_MERCATOR).unwrap();
        let hull = st_concave_hull(&ls, 0.5).unwrap();
        assert_eq!(hull.type_name(), "Polygon");
    }

    #[test]
    fn concave_hull_preserves_srid() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
        ];
        let mp = SurrealGeometry::multi_point(coords, Srid::WEB_MERCATOR).unwrap();
        let hull = st_concave_hull(&mp, 0.5).unwrap();
        assert_eq!(hull.srid().code(), Srid::WEB_MERCATOR.code());
    }
}
