use std::f64::consts::PI;

use surrealgis_core::geometry::SurrealGeometry;

use crate::FunctionError;

const BUFFER_SEGMENTS: usize = 64;

/// Create a buffer around a geometry at a given distance.
/// Currently only supports Point geometry (creates a circle polygon approximation).
/// For other geometry types, returns UnsupportedOperation.
pub fn st_buffer(geom: &SurrealGeometry, distance: f64) -> Result<SurrealGeometry, FunctionError> {
    if distance < 0.0 {
        return Err(FunctionError::InvalidArgument(
            "st_buffer distance must be non-negative".to_string(),
        ));
    }

    let geo_geom = geom.to_geo()?;

    match &geo_geom {
        geo_types::Geometry::Point(pt) => {
            let circle = point_buffer_circle(pt.x(), pt.y(), distance, BUFFER_SEGMENTS);
            let result = geo_types::Geometry::Polygon(circle);
            SurrealGeometry::from_geo(&result, *geom.srid()).map_err(FunctionError::from)
        }
        _ => Err(FunctionError::UnsupportedOperation(
            "st_buffer currently only supports Point geometry".to_string(),
        )),
    }
}

/// Generate a circle polygon approximation centered at (cx, cy) with given radius and segments.
fn point_buffer_circle(
    cx: f64,
    cy: f64,
    radius: f64,
    num_segments: usize,
) -> geo_types::Polygon<f64> {
    let mut coords = Vec::with_capacity(num_segments + 1);
    for i in 0..num_segments {
        let angle = 2.0 * PI * (i as f64) / (num_segments as f64);
        let x = cx + radius * angle.cos();
        let y = cy + radius * angle.sin();
        coords.push(geo_types::Coord { x, y });
    }
    // Close the ring
    coords.push(coords[0]);
    geo_types::Polygon::new(geo_types::LineString(coords), vec![])
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::srid::Srid;

    #[test]
    fn buffer_point_creates_polygon() {
        let pt = SurrealGeometry::point(0.0, 0.0, Srid::WEB_MERCATOR).unwrap();
        let result = st_buffer(&pt, 10.0).unwrap();
        assert_eq!(result.type_name(), "Polygon");
        // Circle should have BUFFER_SEGMENTS + 1 coords (closed ring)
        assert_eq!(result.num_points(), BUFFER_SEGMENTS + 1);
    }

    #[test]
    fn buffer_point_radius_check() {
        let pt = SurrealGeometry::point(5.0, 5.0, Srid::WEB_MERCATOR).unwrap();
        let result = st_buffer(&pt, 3.0).unwrap();
        let geo = result.to_geo().unwrap();
        if let geo_types::Geometry::Polygon(poly) = geo {
            // All vertices should be approximately distance 3.0 from center (5,5)
            for coord in poly.exterior().coords() {
                let dx = coord.x - 5.0;
                let dy = coord.y - 5.0;
                let dist = (dx * dx + dy * dy).sqrt();
                assert!(
                    (dist - 3.0).abs() < 1e-10,
                    "Vertex at ({}, {}) has distance {} from center, expected 3.0",
                    coord.x,
                    coord.y,
                    dist
                );
            }
        } else {
            panic!("Expected Polygon");
        }
    }

    #[test]
    fn buffer_zero_distance() {
        let pt = SurrealGeometry::point(1.0, 2.0, Srid::WEB_MERCATOR).unwrap();
        let result = st_buffer(&pt, 0.0).unwrap();
        assert_eq!(result.type_name(), "Polygon");
    }

    #[test]
    fn buffer_negative_distance_rejected() {
        let pt = SurrealGeometry::point(1.0, 2.0, Srid::WEB_MERCATOR).unwrap();
        let result = st_buffer(&pt, -1.0);
        assert!(result.is_err());
    }

    #[test]
    fn buffer_linestring_unsupported() {
        let coords = vec![
            surrealgis_core::coordinate::Coordinate::new(0.0, 0.0).unwrap(),
            surrealgis_core::coordinate::Coordinate::new(1.0, 1.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WEB_MERCATOR).unwrap();
        let result = st_buffer(&ls, 1.0);
        assert!(matches!(result, Err(FunctionError::UnsupportedOperation(_))));
    }

    #[test]
    fn buffer_preserves_srid() {
        let pt = SurrealGeometry::point(0.0, 0.0, Srid::WEB_MERCATOR).unwrap();
        let result = st_buffer(&pt, 5.0).unwrap();
        assert_eq!(result.srid().code(), Srid::WEB_MERCATOR.code());
    }
}
