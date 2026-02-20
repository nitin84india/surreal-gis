use geo::{Centroid, Scale};
use surrealgis_core::geometry::SurrealGeometry;

use crate::FunctionError;

/// Scale a geometry by the given x and y factors relative to its centroid.
/// A factor of 1.0 keeps the dimension unchanged.
pub fn st_scale(
    geom: &SurrealGeometry,
    sx: f64,
    sy: f64,
) -> Result<SurrealGeometry, FunctionError> {
    let geo_geom = geom.to_geo()?;
    let result = match geo_geom.centroid() {
        Some(centroid) => geo_geom.scale_around_point(sx, sy, centroid),
        None => geo_geom.scale_xy(sx, sy),
    };
    SurrealGeometry::from_geo(&result, *geom.srid()).map_err(FunctionError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::coordinate::Coordinate;
    use surrealgis_core::geometry::GeometryType;
    use surrealgis_core::srid::Srid;

    #[test]
    fn scale_point_identity() {
        // Scaling a point around its centroid (itself) should not change it
        let p = SurrealGeometry::point(5.0, 10.0, Srid::WEB_MERCATOR).unwrap();
        let result = st_scale(&p, 2.0, 3.0).unwrap();
        if let GeometryType::Point(c) = result.geometry_type() {
            assert!((c.x() - 5.0).abs() < 1e-10);
            assert!((c.y() - 10.0).abs() < 1e-10);
        } else {
            panic!("Expected Point");
        }
    }

    #[test]
    fn scale_by_one_is_identity() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(4.0, 0.0).unwrap(),
            Coordinate::new(4.0, 4.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        let poly = SurrealGeometry::polygon(coords.clone(), vec![], Srid::WEB_MERCATOR).unwrap();
        let result = st_scale(&poly, 1.0, 1.0).unwrap();
        if let GeometryType::Polygon { exterior, .. } = result.geometry_type() {
            for (orig, scaled) in coords.iter().zip(exterior.iter()) {
                assert!((orig.x() - scaled.x()).abs() < 1e-8);
                assert!((orig.y() - scaled.y()).abs() < 1e-8);
            }
        } else {
            panic!("Expected Polygon");
        }
    }

    #[test]
    fn scale_doubles_size() {
        // A line from (0,0) to (2,0) has centroid at (1,0).
        // Scaling by 2x around centroid: (0,0) -> (-1,0), (2,0) -> (3,0)
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(2.0, 0.0).unwrap(),
        ];
        let line = SurrealGeometry::line_string(coords, Srid::WEB_MERCATOR).unwrap();
        let result = st_scale(&line, 2.0, 2.0).unwrap();
        if let GeometryType::LineString(cs) = result.geometry_type() {
            assert!((cs[0].x() - (-1.0)).abs() < 1e-8);
            assert!((cs[1].x() - 3.0).abs() < 1e-8);
        } else {
            panic!("Expected LineString");
        }
    }

    #[test]
    fn scale_preserves_srid() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WEB_MERCATOR).unwrap();
        let result = st_scale(&p, 2.0, 2.0).unwrap();
        assert_eq!(result.srid().code(), Srid::WEB_MERCATOR.code());
    }
}
