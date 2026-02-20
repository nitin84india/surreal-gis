use geo::line_measures::LengthMeasurable;
use geo::{Euclidean, Geodesic};
use surrealgis_core::geometry::{GeometryType, SurrealGeometry};

use crate::FunctionError;

/// Compute the length of a geometry.
/// For geographic SRID (4326), returns geodesic length in meters.
/// For projected SRID, returns Euclidean length in projection units.
pub fn st_length(geom: &SurrealGeometry) -> Result<f64, FunctionError> {
    let geo_geom = geom.to_geo()?;

    match geom.geometry_type() {
        GeometryType::LineString(_) | GeometryType::MultiLineString(_) => {
            if geom.srid().is_geographic() {
                match &geo_geom {
                    geo_types::Geometry::LineString(ls) => Ok(ls.length(&Geodesic)),
                    geo_types::Geometry::MultiLineString(mls) => Ok(mls.length(&Geodesic)),
                    _ => unreachable!(),
                }
            } else {
                match &geo_geom {
                    geo_types::Geometry::LineString(ls) => Ok(ls.length(&Euclidean)),
                    geo_types::Geometry::MultiLineString(mls) => Ok(mls.length(&Euclidean)),
                    _ => unreachable!(),
                }
            }
        }
        _ => Ok(0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::coordinate::Coordinate;
    use surrealgis_core::srid::Srid;

    #[test]
    fn euclidean_length_simple() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(3.0, 4.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WEB_MERCATOR).unwrap();
        let length = st_length(&ls).unwrap();
        assert!((length - 5.0).abs() < 1e-6);
    }

    #[test]
    fn geodesic_length_short_line() {
        // ~111km for 1 degree at equator
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 0.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WGS84).unwrap();
        let length = st_length(&ls).unwrap();
        // Should be approximately 111195 meters (1 degree longitude at equator)
        assert!(length > 111000.0 && length < 112000.0, "Length was {length}");
    }

    #[test]
    fn point_has_zero_length() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        assert_eq!(st_length(&p).unwrap(), 0.0);
    }

    #[test]
    fn polygon_has_zero_length() {
        let exterior = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        let poly = SurrealGeometry::polygon(exterior, vec![], Srid::WGS84).unwrap();
        assert_eq!(st_length(&poly).unwrap(), 0.0);
    }
}
