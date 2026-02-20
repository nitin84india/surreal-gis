use geo::{Bearing, Geodesic};
use surrealgis_core::geometry::{GeometryType, SurrealGeometry};

use crate::FunctionError;

/// Compute the azimuth (bearing) between two points.
/// Returns the angle in radians from north (clockwise).
pub fn st_azimuth(a: &SurrealGeometry, b: &SurrealGeometry) -> Result<f64, FunctionError> {
    let (pa, pb) = match (a.geometry_type(), b.geometry_type()) {
        (GeometryType::Point(ca), GeometryType::Point(cb)) => {
            (
                geo_types::Point::new(ca.x(), ca.y()),
                geo_types::Point::new(cb.x(), cb.y()),
            )
        }
        _ => {
            return Err(FunctionError::InvalidArgument(
                "st_azimuth requires two Point geometries".to_string(),
            ))
        }
    };

    let bearing_degrees = Geodesic::bearing(pa, pb);
    // Convert from degrees to radians
    let bearing_radians = bearing_degrees.to_radians();
    // Normalize to [0, 2*PI)
    let normalized = if bearing_radians < 0.0 {
        bearing_radians + 2.0 * std::f64::consts::PI
    } else {
        bearing_radians
    };
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::srid::Srid;
    use std::f64::consts::PI;

    #[test]
    fn azimuth_north() {
        // Point due north should have azimuth ~0
        let a = SurrealGeometry::point(0.0, 0.0, Srid::WGS84).unwrap();
        let b = SurrealGeometry::point(0.0, 1.0, Srid::WGS84).unwrap();
        let az = st_azimuth(&a, &b).unwrap();
        assert!(az.abs() < 0.01 || (az - 2.0 * PI).abs() < 0.01, "Azimuth was {az}");
    }

    #[test]
    fn azimuth_east() {
        // Point due east should have azimuth ~PI/2
        let a = SurrealGeometry::point(0.0, 0.0, Srid::WGS84).unwrap();
        let b = SurrealGeometry::point(1.0, 0.0, Srid::WGS84).unwrap();
        let az = st_azimuth(&a, &b).unwrap();
        assert!((az - PI / 2.0).abs() < 0.01, "Azimuth was {az}");
    }

    #[test]
    fn azimuth_requires_points() {
        let a = SurrealGeometry::point(0.0, 0.0, Srid::WGS84).unwrap();
        let coords = vec![
            surrealgis_core::coordinate::Coordinate::new(0.0, 0.0).unwrap(),
            surrealgis_core::coordinate::Coordinate::new(1.0, 1.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WGS84).unwrap();
        assert!(st_azimuth(&a, &ls).is_err());
    }
}
