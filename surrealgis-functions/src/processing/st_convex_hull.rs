use geo::ConvexHull;
use surrealgis_core::geometry::SurrealGeometry;

use crate::FunctionError;

/// Compute the convex hull of a geometry.
/// Returns the smallest convex polygon that contains all points of the input geometry.
pub fn st_convex_hull(geom: &SurrealGeometry) -> Result<SurrealGeometry, FunctionError> {
    let geo_geom = geom.to_geo()?;
    let hull = geo_geom.convex_hull();
    let result = geo_types::Geometry::Polygon(hull);
    SurrealGeometry::from_geo(&result, *geom.srid()).map_err(FunctionError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::coordinate::Coordinate;
    use surrealgis_core::srid::Srid;

    #[test]
    fn convex_hull_of_multipoint() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(4.0, 0.0).unwrap(),
            Coordinate::new(4.0, 4.0).unwrap(),
            Coordinate::new(0.0, 4.0).unwrap(),
            Coordinate::new(2.0, 2.0).unwrap(), // interior point
        ];
        let mp = SurrealGeometry::multi_point(coords, Srid::WEB_MERCATOR).unwrap();
        let hull = st_convex_hull(&mp).unwrap();
        assert_eq!(hull.type_name(), "Polygon");
        // The hull should only include the 4 corner points (+ closing), not the interior point
        assert_eq!(hull.num_points(), 5);
    }

    #[test]
    fn convex_hull_of_polygon() {
        let exterior = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(2.0, 4.0).unwrap(),
            Coordinate::new(4.0, 0.0).unwrap(),
            Coordinate::new(3.0, 1.0).unwrap(), // concavity
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        let poly = SurrealGeometry::polygon(exterior, vec![], Srid::WEB_MERCATOR).unwrap();
        let hull = st_convex_hull(&poly).unwrap();
        assert_eq!(hull.type_name(), "Polygon");
    }

    #[test]
    fn convex_hull_of_point() {
        let pt = SurrealGeometry::point(1.0, 2.0, Srid::WEB_MERCATOR).unwrap();
        let hull = st_convex_hull(&pt).unwrap();
        // Convex hull of a single point is a degenerate polygon
        assert_eq!(hull.type_name(), "Polygon");
    }

    #[test]
    fn convex_hull_of_linestring() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
            Coordinate::new(2.0, 0.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WEB_MERCATOR).unwrap();
        let hull = st_convex_hull(&ls).unwrap();
        assert_eq!(hull.type_name(), "Polygon");
    }

    #[test]
    fn convex_hull_preserves_srid() {
        let pt = SurrealGeometry::point(1.0, 2.0, Srid::WEB_MERCATOR).unwrap();
        let hull = st_convex_hull(&pt).unwrap();
        assert_eq!(hull.srid().code(), Srid::WEB_MERCATOR.code());
    }
}
