use surrealgis_core::geometry::SurrealGeometry;
use surrealgis_core::srid::Srid;

use crate::FunctionError;

/// Create a Point geometry from x and y coordinates with a given SRID.
pub fn st_point(x: f64, y: f64, srid: i32) -> Result<SurrealGeometry, FunctionError> {
    let srid = Srid::new(srid)?;
    let geom = SurrealGeometry::point(x, y, srid)?;
    Ok(geom)
}

/// Alias for st_point.
pub fn st_make_point(x: f64, y: f64, srid: i32) -> Result<SurrealGeometry, FunctionError> {
    st_point(x, y, srid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_point() {
        let p = st_point(1.0, 2.0, 4326).unwrap();
        assert_eq!(p.type_name(), "Point");
        assert_eq!(p.srid().code(), 4326);
    }

    #[test]
    fn create_point_web_mercator() {
        let p = st_point(500000.0, 4500000.0, 3857).unwrap();
        assert_eq!(p.srid().code(), 3857);
    }

    #[test]
    fn make_point_alias() {
        let p1 = st_point(1.0, 2.0, 4326).unwrap();
        let p2 = st_make_point(1.0, 2.0, 4326).unwrap();
        assert_eq!(p1.type_name(), p2.type_name());
    }

    #[test]
    fn invalid_srid() {
        let result = st_point(1.0, 2.0, 0);
        assert!(result.is_err());
    }

    #[test]
    fn nan_coordinate() {
        let result = st_point(f64::NAN, 2.0, 4326);
        assert!(result.is_err());
    }
}
