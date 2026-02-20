use geo::Translate;
use surrealgis_core::geometry::SurrealGeometry;

use crate::FunctionError;

/// Translate (shift) a geometry by the given offsets.
/// Returns a new geometry with all coordinates shifted by (dx, dy).
pub fn st_translate(
    geom: &SurrealGeometry,
    dx: f64,
    dy: f64,
) -> Result<SurrealGeometry, FunctionError> {
    let geo_geom = geom.to_geo()?;
    let result = geo_geom.translate(dx, dy);
    SurrealGeometry::from_geo(&result, *geom.srid()).map_err(FunctionError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::geometry::GeometryType;
    use surrealgis_core::srid::Srid;

    #[test]
    fn translate_point() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WEB_MERCATOR).unwrap();
        let result = st_translate(&p, 10.0, 20.0).unwrap();
        if let GeometryType::Point(c) = result.geometry_type() {
            assert!((c.x() - 11.0).abs() < 1e-10);
            assert!((c.y() - 22.0).abs() < 1e-10);
        } else {
            panic!("Expected Point");
        }
    }

    #[test]
    fn translate_zero_offset() {
        let p = SurrealGeometry::point(5.0, 10.0, Srid::WEB_MERCATOR).unwrap();
        let result = st_translate(&p, 0.0, 0.0).unwrap();
        if let GeometryType::Point(c) = result.geometry_type() {
            assert!((c.x() - 5.0).abs() < 1e-10);
            assert!((c.y() - 10.0).abs() < 1e-10);
        } else {
            panic!("Expected Point");
        }
    }

    #[test]
    fn translate_negative_offset() {
        let p = SurrealGeometry::point(10.0, 20.0, Srid::WEB_MERCATOR).unwrap();
        let result = st_translate(&p, -5.0, -10.0).unwrap();
        if let GeometryType::Point(c) = result.geometry_type() {
            assert!((c.x() - 5.0).abs() < 1e-10);
            assert!((c.y() - 10.0).abs() < 1e-10);
        } else {
            panic!("Expected Point");
        }
    }

    #[test]
    fn translate_preserves_srid() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WEB_MERCATOR).unwrap();
        let result = st_translate(&p, 10.0, 20.0).unwrap();
        assert_eq!(result.srid().code(), Srid::WEB_MERCATOR.code());
    }

    #[test]
    fn translate_linestring() {
        use surrealgis_core::coordinate::Coordinate;
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
            Coordinate::new(2.0, 0.0).unwrap(),
        ];
        let line = SurrealGeometry::line_string(coords, Srid::WEB_MERCATOR).unwrap();
        let result = st_translate(&line, 10.0, 10.0).unwrap();
        if let GeometryType::LineString(cs) = result.geometry_type() {
            assert!((cs[0].x() - 10.0).abs() < 1e-10);
            assert!((cs[0].y() - 10.0).abs() < 1e-10);
            assert!((cs[1].x() - 11.0).abs() < 1e-10);
            assert!((cs[1].y() - 11.0).abs() < 1e-10);
        } else {
            panic!("Expected LineString");
        }
    }
}
