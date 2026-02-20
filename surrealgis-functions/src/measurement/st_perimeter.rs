use geo::{Euclidean, Geodesic, Length};
use surrealgis_core::geometry::{GeometryType, SurrealGeometry};

use crate::FunctionError;

/// Compute the perimeter of a Polygon (length of exterior ring).
/// For geographic SRIDs, returns geodesic perimeter in meters.
/// For projected SRIDs, returns Euclidean perimeter.
pub fn st_perimeter(geom: &SurrealGeometry) -> Result<f64, FunctionError> {
    let geo_geom = geom.to_geo()?;

    match (&geo_geom, geom.geometry_type()) {
        (geo_types::Geometry::Polygon(poly), _) => {
            let exterior = poly.exterior();
            if geom.srid().is_geographic() {
                Ok(exterior.length::<Geodesic>())
            } else {
                Ok(exterior.length::<Euclidean>())
            }
        }
        (geo_types::Geometry::MultiPolygon(mp), _) => {
            let mut total = 0.0;
            for poly in &mp.0 {
                let exterior = poly.exterior();
                if geom.srid().is_geographic() {
                    total += exterior.length::<Geodesic>();
                } else {
                    total += exterior.length::<Euclidean>();
                }
            }
            Ok(total)
        }
        (_, GeometryType::GeometryCollection(geoms)) => {
            let mut total = 0.0;
            for g in geoms {
                total += st_perimeter(g)?;
            }
            Ok(total)
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
    fn unit_square_perimeter() {
        let exterior = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
            Coordinate::new(0.0, 1.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        let poly = SurrealGeometry::polygon(exterior, vec![], Srid::WEB_MERCATOR).unwrap();
        let perimeter = st_perimeter(&poly).unwrap();
        assert!((perimeter - 4.0).abs() < 1e-6);
    }

    #[test]
    fn point_has_zero_perimeter() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        assert_eq!(st_perimeter(&p).unwrap(), 0.0);
    }
}
