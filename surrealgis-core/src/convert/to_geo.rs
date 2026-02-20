use geo_types;

use crate::coordinate::Coordinate;
use crate::error::GeometryError;
use crate::geometry::{GeometryType, SurrealGeometry};

impl SurrealGeometry {
    /// Convert this geometry to a geo_types::Geometry.
    pub fn to_geo(&self) -> Result<geo_types::Geometry<f64>, GeometryError> {
        match self.geometry_type() {
            GeometryType::Point(coord) => {
                let gc: geo_types::Coord<f64> = coord.into();
                Ok(geo_types::Geometry::Point(geo_types::Point(gc)))
            }
            GeometryType::LineString(coords) => {
                let line = coords_to_geo_linestring(coords);
                Ok(geo_types::Geometry::LineString(line))
            }
            GeometryType::Polygon { exterior, holes } => {
                let ext = coords_to_geo_linestring(exterior);
                let hole_lines: Vec<geo_types::LineString<f64>> =
                    holes.iter().map(|h| coords_to_geo_linestring(h)).collect();
                Ok(geo_types::Geometry::Polygon(geo_types::Polygon::new(
                    ext, hole_lines,
                )))
            }
            GeometryType::MultiPoint(coords) => {
                let points: Vec<geo_types::Point<f64>> = coords
                    .iter()
                    .map(|c| {
                        let gc: geo_types::Coord<f64> = c.into();
                        geo_types::Point(gc)
                    })
                    .collect();
                Ok(geo_types::Geometry::MultiPoint(geo_types::MultiPoint(
                    points,
                )))
            }
            GeometryType::MultiLineString(lines) => {
                let geo_lines: Vec<geo_types::LineString<f64>> =
                    lines.iter().map(|l| coords_to_geo_linestring(l)).collect();
                Ok(geo_types::Geometry::MultiLineString(
                    geo_types::MultiLineString(geo_lines),
                ))
            }
            GeometryType::MultiPolygon(polygons) => {
                let geo_polys: Vec<geo_types::Polygon<f64>> = polygons
                    .iter()
                    .map(|p| {
                        let ext = coords_to_geo_linestring(&p.exterior);
                        let holes: Vec<geo_types::LineString<f64>> =
                            p.holes.iter().map(|h| coords_to_geo_linestring(h)).collect();
                        geo_types::Polygon::new(ext, holes)
                    })
                    .collect();
                Ok(geo_types::Geometry::MultiPolygon(geo_types::MultiPolygon(
                    geo_polys,
                )))
            }
            GeometryType::GeometryCollection(geoms) => {
                let converted: Result<Vec<geo_types::Geometry<f64>>, GeometryError> =
                    geoms.iter().map(|g| g.to_geo()).collect();
                Ok(geo_types::Geometry::GeometryCollection(
                    geo_types::GeometryCollection(converted?),
                ))
            }
        }
    }
}

impl TryFrom<&SurrealGeometry> for geo_types::Geometry<f64> {
    type Error = GeometryError;

    fn try_from(geom: &SurrealGeometry) -> Result<Self, Self::Error> {
        geom.to_geo()
    }
}

fn coords_to_geo_linestring(coords: &[Coordinate]) -> geo_types::LineString<f64> {
    let geo_coords: Vec<geo_types::Coord<f64>> = coords.iter().map(|c| c.into()).collect();
    geo_types::LineString(geo_coords)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::Coordinate;
    use crate::srid::Srid;

    #[test]
    fn point_to_geo() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        let geo = p.to_geo().unwrap();
        match geo {
            geo_types::Geometry::Point(pt) => {
                assert_eq!(pt.x(), 1.0);
                assert_eq!(pt.y(), 2.0);
            }
            _ => panic!("Expected Point"),
        }
    }

    #[test]
    fn linestring_to_geo() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WGS84).unwrap();
        let geo = ls.to_geo().unwrap();
        assert!(matches!(geo, geo_types::Geometry::LineString(_)));
    }

    #[test]
    fn polygon_to_geo() {
        let exterior = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        let poly = SurrealGeometry::polygon(exterior, vec![], Srid::WGS84).unwrap();
        let geo = poly.to_geo().unwrap();
        assert!(matches!(geo, geo_types::Geometry::Polygon(_)));
    }

    #[test]
    fn multi_point_to_geo() {
        let coords = vec![
            Coordinate::new(1.0, 2.0).unwrap(),
            Coordinate::new(3.0, 4.0).unwrap(),
        ];
        let mp = SurrealGeometry::multi_point(coords, Srid::WGS84).unwrap();
        let geo = mp.to_geo().unwrap();
        assert!(matches!(geo, geo_types::Geometry::MultiPoint(_)));
    }

    #[test]
    fn try_from_works() {
        let p = SurrealGeometry::point(5.0, 10.0, Srid::WGS84).unwrap();
        let geo: geo_types::Geometry<f64> = (&p).try_into().unwrap();
        assert!(matches!(geo, geo_types::Geometry::Point(_)));
    }
}
