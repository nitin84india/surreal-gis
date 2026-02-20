use geo::TriangulateEarcut;
use surrealgis_core::geometry::{GeometryType, SurrealGeometry};

use crate::FunctionError;

/// Compute the Delaunay triangulation of a geometry.
/// Extracts all points from the input geometry, creates a bounding polygon,
/// and triangulates it using the earcut algorithm.
/// Returns a GeometryCollection of triangle Polygons.
pub fn st_delaunay_triangles(
    geom: &SurrealGeometry,
) -> Result<SurrealGeometry, FunctionError> {
    let points = extract_all_coords(geom)?;
    if points.len() < 3 {
        return Err(FunctionError::InvalidArgument(
            "st_delaunay_triangles requires at least 3 points".to_string(),
        ));
    }

    // Build a polygon from the convex hull of the points for earcut triangulation
    let multi_point = geo_types::MultiPoint::new(
        points
            .iter()
            .map(|c| geo_types::Point::new(c.x, c.y))
            .collect(),
    );

    use geo::ConvexHull;
    let hull = multi_point.convex_hull();

    // Triangulate the convex hull polygon using earcut
    let triangles = hull.earcut_triangles();

    // Convert triangles to SurrealGeometry polygons
    let srid = *geom.srid();
    let triangle_geoms: Result<Vec<SurrealGeometry>, _> = triangles
        .into_iter()
        .map(|tri| {
            let poly = tri.to_polygon();
            let geo = geo_types::Geometry::Polygon(poly);
            SurrealGeometry::from_geo(&geo, srid).map_err(FunctionError::from)
        })
        .collect();

    SurrealGeometry::geometry_collection(triangle_geoms?, srid).map_err(FunctionError::from)
}

/// Extract all coordinates from any geometry type into a flat Vec.
fn extract_all_coords(geom: &SurrealGeometry) -> Result<Vec<geo_types::Coord<f64>>, FunctionError> {
    let mut coords = Vec::new();
    collect_coords(geom, &mut coords)?;
    Ok(coords)
}

fn collect_coords(
    geom: &SurrealGeometry,
    coords: &mut Vec<geo_types::Coord<f64>>,
) -> Result<(), FunctionError> {
    match geom.geometry_type() {
        GeometryType::Point(c) => {
            coords.push(geo_types::Coord { x: c.x(), y: c.y() });
        }
        GeometryType::LineString(cs) => {
            for c in cs {
                coords.push(geo_types::Coord { x: c.x(), y: c.y() });
            }
        }
        GeometryType::Polygon { exterior, holes } => {
            for c in exterior {
                coords.push(geo_types::Coord { x: c.x(), y: c.y() });
            }
            for hole in holes {
                for c in hole {
                    coords.push(geo_types::Coord { x: c.x(), y: c.y() });
                }
            }
        }
        GeometryType::MultiPoint(cs) => {
            for c in cs {
                coords.push(geo_types::Coord { x: c.x(), y: c.y() });
            }
        }
        GeometryType::MultiLineString(lines) => {
            for line in lines {
                for c in line {
                    coords.push(geo_types::Coord { x: c.x(), y: c.y() });
                }
            }
        }
        GeometryType::MultiPolygon(polygons) => {
            for poly in polygons {
                for c in &poly.exterior {
                    coords.push(geo_types::Coord { x: c.x(), y: c.y() });
                }
                for hole in &poly.holes {
                    for c in hole {
                        coords.push(geo_types::Coord { x: c.x(), y: c.y() });
                    }
                }
            }
        }
        GeometryType::GeometryCollection(geoms) => {
            for g in geoms {
                collect_coords(g, coords)?;
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
    fn delaunay_from_multipoint() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(4.0, 0.0).unwrap(),
            Coordinate::new(4.0, 4.0).unwrap(),
            Coordinate::new(0.0, 4.0).unwrap(),
        ];
        let mp = SurrealGeometry::multi_point(coords, Srid::WEB_MERCATOR).unwrap();
        let result = st_delaunay_triangles(&mp).unwrap();
        assert_eq!(result.type_name(), "GeometryCollection");

        // A square should produce 2 triangles
        if let GeometryType::GeometryCollection(geoms) = result.geometry_type() {
            assert_eq!(geoms.len(), 2);
            for g in geoms {
                assert_eq!(g.type_name(), "Polygon");
            }
        } else {
            panic!("Expected GeometryCollection");
        }
    }

    #[test]
    fn delaunay_from_polygon() {
        let exterior = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(4.0, 0.0).unwrap(),
            Coordinate::new(4.0, 4.0).unwrap(),
            Coordinate::new(0.0, 4.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        let poly = SurrealGeometry::polygon(exterior, vec![], Srid::WEB_MERCATOR).unwrap();
        let result = st_delaunay_triangles(&poly).unwrap();
        assert_eq!(result.type_name(), "GeometryCollection");
    }

    #[test]
    fn delaunay_too_few_points() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 0.0).unwrap(),
        ];
        let mp = SurrealGeometry::multi_point(coords, Srid::WEB_MERCATOR).unwrap();
        let result = st_delaunay_triangles(&mp);
        assert!(matches!(result, Err(FunctionError::InvalidArgument(_))));
    }

    #[test]
    fn delaunay_triangle() {
        // Three points form exactly one triangle
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(4.0, 0.0).unwrap(),
            Coordinate::new(2.0, 3.0).unwrap(),
        ];
        let mp = SurrealGeometry::multi_point(coords, Srid::WEB_MERCATOR).unwrap();
        let result = st_delaunay_triangles(&mp).unwrap();
        if let GeometryType::GeometryCollection(geoms) = result.geometry_type() {
            assert_eq!(geoms.len(), 1);
        } else {
            panic!("Expected GeometryCollection");
        }
    }

    #[test]
    fn delaunay_preserves_srid() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(4.0, 0.0).unwrap(),
            Coordinate::new(2.0, 3.0).unwrap(),
        ];
        let mp = SurrealGeometry::multi_point(coords, Srid::WEB_MERCATOR).unwrap();
        let result = st_delaunay_triangles(&mp).unwrap();
        assert_eq!(result.srid().code(), Srid::WEB_MERCATOR.code());
    }
}
