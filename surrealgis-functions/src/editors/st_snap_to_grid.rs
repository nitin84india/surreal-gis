use geo::MapCoords;
use surrealgis_core::geometry::SurrealGeometry;

use crate::FunctionError;

/// Snap all coordinates of a geometry to a regular grid of the given cell size.
/// Each coordinate is rounded to the nearest grid point:
///   snapped_x = round(x / size) * size
///   snapped_y = round(y / size) * size
pub fn st_snap_to_grid(
    geom: &SurrealGeometry,
    size: f64,
) -> Result<SurrealGeometry, FunctionError> {
    if size <= 0.0 {
        return Err(FunctionError::InvalidArgument(
            "Grid size must be positive".to_string(),
        ));
    }
    let geo_geom = geom.to_geo()?;
    let snapped = geo_geom.map_coords(|coord| geo_types::Coord {
        x: (coord.x / size).round() * size,
        y: (coord.y / size).round() * size,
    });
    SurrealGeometry::from_geo(&snapped, *geom.srid()).map_err(FunctionError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::coordinate::Coordinate;
    use surrealgis_core::geometry::GeometryType;
    use surrealgis_core::srid::Srid;

    #[test]
    fn snap_point_to_grid() {
        let p = SurrealGeometry::point(1.3, 2.7, Srid::WEB_MERCATOR).unwrap();
        let result = st_snap_to_grid(&p, 1.0).unwrap();
        if let GeometryType::Point(c) = result.geometry_type() {
            assert!((c.x() - 1.0).abs() < 1e-10);
            assert!((c.y() - 3.0).abs() < 1e-10);
        } else {
            panic!("Expected Point");
        }
    }

    #[test]
    fn snap_point_to_fine_grid() {
        let p = SurrealGeometry::point(1.37, 2.74, Srid::WEB_MERCATOR).unwrap();
        let result = st_snap_to_grid(&p, 0.5).unwrap();
        if let GeometryType::Point(c) = result.geometry_type() {
            assert!((c.x() - 1.5).abs() < 1e-10);
            assert!((c.y() - 2.5).abs() < 1e-10);
        } else {
            panic!("Expected Point");
        }
    }

    #[test]
    fn snap_linestring_to_grid() {
        let coords = vec![
            Coordinate::new(0.3, 0.7).unwrap(),
            Coordinate::new(1.6, 1.2).unwrap(),
            Coordinate::new(2.8, 0.4).unwrap(),
        ];
        let line = SurrealGeometry::line_string(coords, Srid::WEB_MERCATOR).unwrap();
        let result = st_snap_to_grid(&line, 1.0).unwrap();
        if let GeometryType::LineString(cs) = result.geometry_type() {
            assert!((cs[0].x() - 0.0).abs() < 1e-10);
            assert!((cs[0].y() - 1.0).abs() < 1e-10);
            assert!((cs[1].x() - 2.0).abs() < 1e-10);
            assert!((cs[1].y() - 1.0).abs() < 1e-10);
            assert!((cs[2].x() - 3.0).abs() < 1e-10);
            assert!((cs[2].y() - 0.0).abs() < 1e-10);
        } else {
            panic!("Expected LineString");
        }
    }

    #[test]
    fn snap_negative_size_rejected() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WEB_MERCATOR).unwrap();
        let result = st_snap_to_grid(&p, -1.0);
        assert!(result.is_err());
    }

    #[test]
    fn snap_zero_size_rejected() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WEB_MERCATOR).unwrap();
        let result = st_snap_to_grid(&p, 0.0);
        assert!(result.is_err());
    }

    #[test]
    fn snap_preserves_srid() {
        let p = SurrealGeometry::point(1.3, 2.7, Srid::WEB_MERCATOR).unwrap();
        let result = st_snap_to_grid(&p, 1.0).unwrap();
        assert_eq!(result.srid().code(), Srid::WEB_MERCATOR.code());
    }

    #[test]
    fn snap_already_on_grid() {
        let p = SurrealGeometry::point(2.0, 4.0, Srid::WEB_MERCATOR).unwrap();
        let result = st_snap_to_grid(&p, 1.0).unwrap();
        if let GeometryType::Point(c) = result.geometry_type() {
            assert!((c.x() - 2.0).abs() < 1e-10);
            assert!((c.y() - 4.0).abs() < 1e-10);
        } else {
            panic!("Expected Point");
        }
    }
}
