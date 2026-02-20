use surrealgis_core::geometry::{GeometryType, SurrealGeometry};
use voronoice::{BoundingBox, Point, VoronoiBuilder};

use crate::FunctionError;

/// Compute the Voronoi diagram for a geometry.
/// Extracts all points from the input geometry and generates Voronoi cells.
/// Returns a GeometryCollection of Polygon cells.
pub fn st_voronoi_polygons(
    geom: &SurrealGeometry,
) -> Result<SurrealGeometry, FunctionError> {
    let points = extract_all_points(geom)?;
    if points.len() < 3 {
        return Err(FunctionError::InvalidArgument(
            "st_voronoi_polygons requires at least 3 non-collinear points".to_string(),
        ));
    }

    // Compute bounding box with some padding for the Voronoi diagram
    let (min_x, min_y, max_x, max_y) = compute_bounds(&points);
    let width = max_x - min_x;
    let height = max_y - min_y;
    // Ensure non-zero dimensions for the bounding box (handle collinear points)
    let extent = width.max(height).max(1.0);
    let padding = extent * 0.5;
    let cx = (min_x + max_x) / 2.0;
    let cy = (min_y + max_y) / 2.0;
    let bbox_width = (width + padding * 2.0).max(1.0);
    let bbox_height = (height + padding * 2.0).max(1.0);

    let voronoi_sites: Vec<Point> = points
        .iter()
        .map(|p| Point { x: p.x, y: p.y })
        .collect();

    let voronoi = VoronoiBuilder::default()
        .set_sites(voronoi_sites)
        .set_bounding_box(BoundingBox::new(
            Point { x: cx, y: cy },
            bbox_width,
            bbox_height,
        ))
        .build();

    let voronoi = voronoi.ok_or_else(|| {
        FunctionError::InvalidArgument("Failed to build Voronoi diagram".to_string())
    })?;

    let srid = *geom.srid();
    let mut cell_geoms = Vec::new();

    for cell_idx in 0..voronoi.sites().len() {
        let cell = voronoi.cell(cell_idx);
        let vertices: Vec<geo_types::Coord<f64>> = cell
            .iter_vertices()
            .map(|v| geo_types::Coord { x: v.x, y: v.y })
            .collect();

        if vertices.len() < 3 {
            continue;
        }

        // Close the ring
        let mut ring = vertices;
        ring.push(ring[0]);

        let polygon = geo_types::Polygon::new(geo_types::LineString(ring), vec![]);
        let geo = geo_types::Geometry::Polygon(polygon);
        let sg = SurrealGeometry::from_geo(&geo, srid).map_err(FunctionError::from)?;
        cell_geoms.push(sg);
    }

    if cell_geoms.is_empty() {
        return Err(FunctionError::InvalidArgument(
            "Voronoi diagram produced no valid cells".to_string(),
        ));
    }

    SurrealGeometry::geometry_collection(cell_geoms, srid).map_err(FunctionError::from)
}

/// Extract all coordinate points from any geometry type.
fn extract_all_points(
    geom: &SurrealGeometry,
) -> Result<Vec<geo_types::Coord<f64>>, FunctionError> {
    let mut points = Vec::new();
    collect_points(geom, &mut points)?;
    Ok(points)
}

fn collect_points(
    geom: &SurrealGeometry,
    points: &mut Vec<geo_types::Coord<f64>>,
) -> Result<(), FunctionError> {
    match geom.geometry_type() {
        GeometryType::Point(c) => {
            points.push(geo_types::Coord { x: c.x(), y: c.y() });
        }
        GeometryType::LineString(coords) => {
            for c in coords {
                points.push(geo_types::Coord { x: c.x(), y: c.y() });
            }
        }
        GeometryType::Polygon { exterior, holes } => {
            for c in exterior {
                points.push(geo_types::Coord { x: c.x(), y: c.y() });
            }
            for hole in holes {
                for c in hole {
                    points.push(geo_types::Coord { x: c.x(), y: c.y() });
                }
            }
        }
        GeometryType::MultiPoint(coords) => {
            for c in coords {
                points.push(geo_types::Coord { x: c.x(), y: c.y() });
            }
        }
        GeometryType::MultiLineString(lines) => {
            for line in lines {
                for c in line {
                    points.push(geo_types::Coord { x: c.x(), y: c.y() });
                }
            }
        }
        GeometryType::MultiPolygon(polygons) => {
            for poly in polygons {
                for c in &poly.exterior {
                    points.push(geo_types::Coord { x: c.x(), y: c.y() });
                }
                for hole in &poly.holes {
                    for c in hole {
                        points.push(geo_types::Coord { x: c.x(), y: c.y() });
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

fn compute_bounds(points: &[geo_types::Coord<f64>]) -> (f64, f64, f64, f64) {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    for p in points {
        min_x = min_x.min(p.x);
        min_y = min_y.min(p.y);
        max_x = max_x.max(p.x);
        max_y = max_y.max(p.y);
    }
    (min_x, min_y, max_x, max_y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::coordinate::Coordinate;
    use surrealgis_core::srid::Srid;

    #[test]
    fn voronoi_from_multipoint() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(4.0, 0.0).unwrap(),
            Coordinate::new(4.0, 4.0).unwrap(),
            Coordinate::new(0.0, 4.0).unwrap(),
        ];
        let mp = SurrealGeometry::multi_point(coords, Srid::WEB_MERCATOR).unwrap();
        let result = st_voronoi_polygons(&mp).unwrap();
        assert_eq!(result.type_name(), "GeometryCollection");

        // 4 sites should produce 4 Voronoi cells
        if let GeometryType::GeometryCollection(geoms) = result.geometry_type() {
            assert_eq!(geoms.len(), 4);
            for g in geoms {
                assert_eq!(g.type_name(), "Polygon");
            }
        } else {
            panic!("Expected GeometryCollection");
        }
    }

    #[test]
    fn voronoi_two_points_rejected() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(4.0, 0.0).unwrap(),
        ];
        let mp = SurrealGeometry::multi_point(coords, Srid::WEB_MERCATOR).unwrap();
        let result = st_voronoi_polygons(&mp);
        assert!(matches!(result, Err(FunctionError::InvalidArgument(_))));
    }

    #[test]
    fn voronoi_too_few_points() {
        let pt = SurrealGeometry::point(1.0, 2.0, Srid::WEB_MERCATOR).unwrap();
        let result = st_voronoi_polygons(&pt);
        assert!(matches!(result, Err(FunctionError::InvalidArgument(_))));
    }

    #[test]
    fn voronoi_from_linestring() {
        // Use non-collinear points to form a valid Voronoi diagram
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(2.0, 3.0).unwrap(),
            Coordinate::new(4.0, 0.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WEB_MERCATOR).unwrap();
        let result = st_voronoi_polygons(&ls).unwrap();
        assert_eq!(result.type_name(), "GeometryCollection");
    }

    #[test]
    fn voronoi_preserves_srid() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(4.0, 0.0).unwrap(),
            Coordinate::new(2.0, 3.0).unwrap(),
        ];
        let mp = SurrealGeometry::multi_point(coords, Srid::WEB_MERCATOR).unwrap();
        let result = st_voronoi_polygons(&mp).unwrap();
        assert_eq!(result.srid().code(), Srid::WEB_MERCATOR.code());
    }
}
