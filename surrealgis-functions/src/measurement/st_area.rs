use geo::algorithm::Area;
use surrealgis_core::geometry::{GeometryType, SurrealGeometry};

use crate::FunctionError;

/// Compute the area of a geometry.
/// Returns unsigned area. For projected CRS, returns area in projection units squared.
/// For geographic CRS, returns approximate area (use with caution).
pub fn st_area(geom: &SurrealGeometry) -> Result<f64, FunctionError> {
    let geo_geom = geom.to_geo()?;
    match &geo_geom {
        geo_types::Geometry::Polygon(p) => Ok(p.unsigned_area()),
        geo_types::Geometry::MultiPolygon(mp) => Ok(mp.unsigned_area()),
        geo_types::Geometry::Rect(r) => Ok(r.unsigned_area()),
        geo_types::Geometry::Triangle(t) => Ok(t.unsigned_area()),
        _ => match geom.geometry_type() {
            GeometryType::Point(_)
            | GeometryType::LineString(_)
            | GeometryType::MultiPoint(_)
            | GeometryType::MultiLineString(_) => Ok(0.0),
            GeometryType::GeometryCollection(geoms) => {
                let mut total = 0.0;
                for g in geoms {
                    total += st_area(g)?;
                }
                Ok(total)
            }
            _ => Ok(0.0),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::coordinate::Coordinate;
    use surrealgis_core::srid::Srid;

    #[test]
    fn unit_square_area() {
        let exterior = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
            Coordinate::new(0.0, 1.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        let poly = SurrealGeometry::polygon(exterior, vec![], Srid::WEB_MERCATOR).unwrap();
        let area = st_area(&poly).unwrap();
        assert!((area - 1.0).abs() < 1e-6);
    }

    #[test]
    fn triangle_area() {
        let exterior = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(4.0, 0.0).unwrap(),
            Coordinate::new(2.0, 3.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        let poly = SurrealGeometry::polygon(exterior, vec![], Srid::WEB_MERCATOR).unwrap();
        let area = st_area(&poly).unwrap();
        assert!((area - 6.0).abs() < 1e-6);
    }

    #[test]
    fn polygon_with_hole_area() {
        let exterior = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(10.0, 0.0).unwrap(),
            Coordinate::new(10.0, 10.0).unwrap(),
            Coordinate::new(0.0, 10.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        let hole = vec![
            Coordinate::new(2.0, 2.0).unwrap(),
            Coordinate::new(8.0, 2.0).unwrap(),
            Coordinate::new(8.0, 8.0).unwrap(),
            Coordinate::new(2.0, 8.0).unwrap(),
            Coordinate::new(2.0, 2.0).unwrap(),
        ];
        let poly = SurrealGeometry::polygon(exterior, vec![hole], Srid::WEB_MERCATOR).unwrap();
        let area = st_area(&poly).unwrap();
        // 10*10 - 6*6 = 100 - 36 = 64
        assert!((area - 64.0).abs() < 1e-6);
    }

    #[test]
    fn point_has_zero_area() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        assert_eq!(st_area(&p).unwrap(), 0.0);
    }

    #[test]
    fn linestring_has_zero_area() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WGS84).unwrap();
        assert_eq!(st_area(&ls).unwrap(), 0.0);
    }
}
