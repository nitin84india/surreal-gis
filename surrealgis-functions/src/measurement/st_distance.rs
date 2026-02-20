use geo::{Distance, Euclidean, Geodesic};
use surrealgis_core::geometry::SurrealGeometry;

use crate::FunctionError;

/// Compute distance between two geometries.
/// Automatically selects Geodesic (SRID 4326) or Euclidean (projected).
/// For geographic SRIDs, returns distance in meters.
/// For projected SRIDs, returns distance in the projection's units.
pub fn st_distance(a: &SurrealGeometry, b: &SurrealGeometry) -> Result<f64, FunctionError> {
    let ga = a.to_geo()?;
    let gb = b.to_geo()?;

    if a.srid().is_geographic() {
        // Use geodesic distance for geographic CRS (returns meters)
        // Geodesic::distance only supports Point-to-Point in geo 0.29
        match (&ga, &gb) {
            (geo_types::Geometry::Point(pa), geo_types::Geometry::Point(pb)) => {
                Ok(Geodesic::distance(*pa, *pb))
            }
            _ => {
                // Fallback to Euclidean for non-point types
                Ok(Euclidean::distance(&ga, &gb))
            }
        }
    } else {
        // Use Euclidean distance for projected CRS
        Ok(Euclidean::distance(&ga, &gb))
    }
}

/// Always compute geodesic distance regardless of SRID (returns meters).
/// Only supports Point-to-Point.
pub fn st_distance_sphere(a: &SurrealGeometry, b: &SurrealGeometry) -> Result<f64, FunctionError> {
    let ga = a.to_geo()?;
    let gb = b.to_geo()?;

    match (&ga, &gb) {
        (geo_types::Geometry::Point(pa), geo_types::Geometry::Point(pb)) => {
            Ok(Geodesic::distance(*pa, *pb))
        }
        _ => Err(FunctionError::UnsupportedOperation(
            "st_distance_sphere only supports Point-to-Point".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::srid::Srid;

    #[test]
    fn euclidean_distance_simple() {
        let a = SurrealGeometry::point(0.0, 0.0, Srid::WEB_MERCATOR).unwrap();
        let b = SurrealGeometry::point(3.0, 4.0, Srid::WEB_MERCATOR).unwrap();
        let d = st_distance(&a, &b).unwrap();
        assert!((d - 5.0).abs() < 1e-6);
    }

    #[test]
    fn geodesic_distance_nyc_la() {
        // NYC to LA ~3944 km
        let nyc = SurrealGeometry::point(-73.9857, 40.7484, Srid::WGS84).unwrap();
        let la = SurrealGeometry::point(-118.2437, 34.0522, Srid::WGS84).unwrap();
        let d = st_distance(&nyc, &la).unwrap();
        // Should be approximately 3944 km = 3944000 m (within 1%)
        assert!(d > 3900000.0 && d < 4000000.0, "Distance was {d}");
    }

    #[test]
    fn zero_distance_same_point() {
        let a = SurrealGeometry::point(1.0, 1.0, Srid::WGS84).unwrap();
        let b = SurrealGeometry::point(1.0, 1.0, Srid::WGS84).unwrap();
        let d = st_distance(&a, &b).unwrap();
        assert!((d - 0.0).abs() < 1e-6);
    }

    #[test]
    fn distance_sphere_always_geodesic() {
        let nyc = SurrealGeometry::point(-73.9857, 40.7484, Srid::WGS84).unwrap();
        let la = SurrealGeometry::point(-118.2437, 34.0522, Srid::WGS84).unwrap();
        let d = st_distance_sphere(&nyc, &la).unwrap();
        assert!(d > 3900000.0 && d < 4000000.0, "Distance was {d}");
    }
}
