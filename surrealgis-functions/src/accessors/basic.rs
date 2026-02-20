use surrealgis_core::geometry::{GeometryType, SurrealGeometry};

use crate::FunctionError;

/// Extract the X coordinate of a Point.
pub fn st_x(geom: &SurrealGeometry) -> Result<f64, FunctionError> {
    match geom.geometry_type() {
        GeometryType::Point(coord) => Ok(coord.x()),
        _ => Err(FunctionError::InvalidArgument(
            "st_x requires a Point geometry".to_string(),
        )),
    }
}

/// Extract the Y coordinate of a Point.
pub fn st_y(geom: &SurrealGeometry) -> Result<f64, FunctionError> {
    match geom.geometry_type() {
        GeometryType::Point(coord) => Ok(coord.y()),
        _ => Err(FunctionError::InvalidArgument(
            "st_y requires a Point geometry".to_string(),
        )),
    }
}

/// Extract the Z coordinate of a Point (returns None if 2D).
pub fn st_z(geom: &SurrealGeometry) -> Result<Option<f64>, FunctionError> {
    match geom.geometry_type() {
        GeometryType::Point(coord) => Ok(coord.z()),
        _ => Err(FunctionError::InvalidArgument(
            "st_z requires a Point geometry".to_string(),
        )),
    }
}

/// Return the SRID of a geometry.
pub fn st_srid(geom: &SurrealGeometry) -> i32 {
    geom.srid().code()
}

/// Return the geometry type name as a string.
pub fn st_geometry_type(geom: &SurrealGeometry) -> &str {
    geom.type_name()
}

/// Return the total number of points in the geometry.
pub fn st_num_points(geom: &SurrealGeometry) -> usize {
    geom.num_points()
}

/// Return the topological dimension (0=point, 1=line, 2=polygon).
pub fn st_dimension(geom: &SurrealGeometry) -> u8 {
    match geom.geometry_type() {
        GeometryType::Point(_) | GeometryType::MultiPoint(_) => 0,
        GeometryType::LineString(_) | GeometryType::MultiLineString(_) => 1,
        GeometryType::Polygon { .. } | GeometryType::MultiPolygon(_) => 2,
        GeometryType::GeometryCollection(geoms) => {
            geoms.iter().map(st_dimension).max().unwrap_or(0)
        }
    }
}

/// Return the first point of a LineString.
pub fn st_start_point(geom: &SurrealGeometry) -> Result<SurrealGeometry, FunctionError> {
    match geom.geometry_type() {
        GeometryType::LineString(coords) => {
            if coords.is_empty() {
                return Err(FunctionError::InvalidArgument(
                    "LineString is empty".to_string(),
                ));
            }
            let c = &coords[0];
            Ok(SurrealGeometry::point(c.x(), c.y(), *geom.srid())?)
        }
        _ => Err(FunctionError::InvalidArgument(
            "st_start_point requires a LineString geometry".to_string(),
        )),
    }
}

/// Return the last point of a LineString.
pub fn st_end_point(geom: &SurrealGeometry) -> Result<SurrealGeometry, FunctionError> {
    match geom.geometry_type() {
        GeometryType::LineString(coords) => {
            if coords.is_empty() {
                return Err(FunctionError::InvalidArgument(
                    "LineString is empty".to_string(),
                ));
            }
            let c = coords.last().unwrap();
            Ok(SurrealGeometry::point(c.x(), c.y(), *geom.srid())?)
        }
        _ => Err(FunctionError::InvalidArgument(
            "st_end_point requires a LineString geometry".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::coordinate::Coordinate;
    use surrealgis_core::srid::Srid;

    fn make_point() -> SurrealGeometry {
        SurrealGeometry::point(1.5, 2.5, Srid::WGS84).unwrap()
    }

    fn make_linestring() -> SurrealGeometry {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
            Coordinate::new(2.0, 0.0).unwrap(),
        ];
        SurrealGeometry::line_string(coords, Srid::WGS84).unwrap()
    }

    fn make_polygon() -> SurrealGeometry {
        let exterior = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        SurrealGeometry::polygon(exterior, vec![], Srid::WGS84).unwrap()
    }

    #[test]
    fn test_st_x() {
        assert_eq!(st_x(&make_point()).unwrap(), 1.5);
    }

    #[test]
    fn test_st_y() {
        assert_eq!(st_y(&make_point()).unwrap(), 2.5);
    }

    #[test]
    fn test_st_z_none() {
        assert_eq!(st_z(&make_point()).unwrap(), None);
    }

    #[test]
    fn test_st_x_on_linestring_fails() {
        assert!(st_x(&make_linestring()).is_err());
    }

    #[test]
    fn test_st_srid() {
        assert_eq!(st_srid(&make_point()), 4326);
    }

    #[test]
    fn test_st_geometry_type() {
        assert_eq!(st_geometry_type(&make_point()), "Point");
        assert_eq!(st_geometry_type(&make_linestring()), "LineString");
        assert_eq!(st_geometry_type(&make_polygon()), "Polygon");
    }

    #[test]
    fn test_st_num_points() {
        assert_eq!(st_num_points(&make_point()), 1);
        assert_eq!(st_num_points(&make_linestring()), 3);
        assert_eq!(st_num_points(&make_polygon()), 4);
    }

    #[test]
    fn test_st_dimension() {
        assert_eq!(st_dimension(&make_point()), 0);
        assert_eq!(st_dimension(&make_linestring()), 1);
        assert_eq!(st_dimension(&make_polygon()), 2);
    }

    #[test]
    fn test_st_start_point() {
        let start = st_start_point(&make_linestring()).unwrap();
        assert_eq!(st_x(&start).unwrap(), 0.0);
        assert_eq!(st_y(&start).unwrap(), 0.0);
    }

    #[test]
    fn test_st_end_point() {
        let end = st_end_point(&make_linestring()).unwrap();
        assert_eq!(st_x(&end).unwrap(), 2.0);
        assert_eq!(st_y(&end).unwrap(), 0.0);
    }

    #[test]
    fn test_start_point_on_point_fails() {
        assert!(st_start_point(&make_point()).is_err());
    }
}
