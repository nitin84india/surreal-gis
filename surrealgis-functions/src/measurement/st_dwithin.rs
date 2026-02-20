use surrealgis_core::geometry::SurrealGeometry;

use crate::measurement::st_distance::st_distance;
use crate::FunctionError;

/// Returns true if the geometries are within the specified distance of each other.
pub fn st_dwithin(
    a: &SurrealGeometry,
    b: &SurrealGeometry,
    distance: f64,
) -> Result<bool, FunctionError> {
    if distance < 0.0 {
        return Err(FunctionError::InvalidArgument(
            "Distance must be non-negative".to_string(),
        ));
    }
    let d = st_distance(a, b)?;
    Ok(d <= distance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::srid::Srid;

    #[test]
    fn within_distance() {
        let a = SurrealGeometry::point(0.0, 0.0, Srid::WEB_MERCATOR).unwrap();
        let b = SurrealGeometry::point(3.0, 4.0, Srid::WEB_MERCATOR).unwrap();
        assert!(st_dwithin(&a, &b, 6.0).unwrap());
        assert!(st_dwithin(&a, &b, 5.0).unwrap());
        assert!(!st_dwithin(&a, &b, 4.9).unwrap());
    }

    #[test]
    fn negative_distance_fails() {
        let a = SurrealGeometry::point(0.0, 0.0, Srid::WGS84).unwrap();
        let b = SurrealGeometry::point(1.0, 1.0, Srid::WGS84).unwrap();
        assert!(st_dwithin(&a, &b, -1.0).is_err());
    }

    #[test]
    fn zero_distance_same_point() {
        let a = SurrealGeometry::point(1.0, 1.0, Srid::WGS84).unwrap();
        let b = SurrealGeometry::point(1.0, 1.0, Srid::WGS84).unwrap();
        assert!(st_dwithin(&a, &b, 0.0).unwrap());
    }
}
