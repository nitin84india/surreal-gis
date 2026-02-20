use geo_types;

use crate::coordinate::Coordinate;
use crate::error::GeometryError;
use crate::geometry::{GeometryType, PolygonData, SurrealGeometry};
use crate::srid::Srid;

impl SurrealGeometry {
    /// Create a SurrealGeometry from a geo_types::Geometry with a specified SRID.
    pub fn from_geo(
        geom: &geo_types::Geometry<f64>,
        srid: Srid,
    ) -> Result<Self, GeometryError> {
        let geometry_type = match geom {
            geo_types::Geometry::Point(pt) => {
                let coord = Coordinate::new(pt.x(), pt.y())?;
                GeometryType::Point(coord)
            }
            geo_types::Geometry::LineString(ls) => {
                let coords = geo_linestring_to_coords(ls)?;
                GeometryType::LineString(coords)
            }
            geo_types::Geometry::Polygon(poly) => {
                let exterior = geo_linestring_to_coords(poly.exterior())?;
                let holes: Result<Vec<Vec<Coordinate>>, GeometryError> = poly
                    .interiors()
                    .iter()
                    .map(geo_linestring_to_coords)
                    .collect();
                GeometryType::Polygon {
                    exterior,
                    holes: holes?,
                }
            }
            geo_types::Geometry::MultiPoint(mp) => {
                let coords: Result<Vec<Coordinate>, GeometryError> = mp
                    .0
                    .iter()
                    .map(|pt| Coordinate::new(pt.x(), pt.y()))
                    .collect();
                GeometryType::MultiPoint(coords?)
            }
            geo_types::Geometry::MultiLineString(mls) => {
                let lines: Result<Vec<Vec<Coordinate>>, GeometryError> =
                    mls.0.iter().map(geo_linestring_to_coords).collect();
                GeometryType::MultiLineString(lines?)
            }
            geo_types::Geometry::MultiPolygon(mp) => {
                let polygons: Result<Vec<PolygonData>, GeometryError> = mp
                    .0
                    .iter()
                    .map(|poly| {
                        let exterior = geo_linestring_to_coords(poly.exterior())?;
                        let holes: Result<Vec<Vec<Coordinate>>, GeometryError> = poly
                            .interiors()
                            .iter()
                            .map(geo_linestring_to_coords)
                            .collect();
                        Ok(PolygonData {
                            exterior,
                            holes: holes?,
                        })
                    })
                    .collect();
                GeometryType::MultiPolygon(polygons?)
            }
            geo_types::Geometry::GeometryCollection(gc) => {
                let geoms: Result<Vec<SurrealGeometry>, GeometryError> =
                    gc.0.iter().map(|g| SurrealGeometry::from_geo(g, srid)).collect();
                GeometryType::GeometryCollection(geoms?)
            }
            geo_types::Geometry::Line(line) => {
                let start = Coordinate::new(line.start.x, line.start.y)?;
                let end = Coordinate::new(line.end.x, line.end.y)?;
                GeometryType::LineString(vec![start, end])
            }
            geo_types::Geometry::Rect(rect) => {
                // Convert Rect to a polygon
                let min = rect.min();
                let max = rect.max();
                let coords = vec![
                    Coordinate::new(min.x, min.y)?,
                    Coordinate::new(max.x, min.y)?,
                    Coordinate::new(max.x, max.y)?,
                    Coordinate::new(min.x, max.y)?,
                    Coordinate::new(min.x, min.y)?,
                ];
                GeometryType::Polygon {
                    exterior: coords,
                    holes: vec![],
                }
            }
            geo_types::Geometry::Triangle(tri) => {
                let coords = vec![
                    Coordinate::new(tri.0.x, tri.0.y)?,
                    Coordinate::new(tri.1.x, tri.1.y)?,
                    Coordinate::new(tri.2.x, tri.2.y)?,
                    Coordinate::new(tri.0.x, tri.0.y)?,
                ];
                GeometryType::Polygon {
                    exterior: coords,
                    holes: vec![],
                }
            }
        };

        Ok(SurrealGeometry::from_parts(geometry_type, srid))
    }
}

impl From<geo_types::Geometry<f64>> for SurrealGeometry {
    fn from(geom: geo_types::Geometry<f64>) -> Self {
        // Use default SRID 4326; panics on invalid coordinates (which shouldn't happen from valid geo_types)
        SurrealGeometry::from_geo(&geom, Srid::DEFAULT)
            .expect("Valid geo_types::Geometry should convert without error")
    }
}

fn geo_linestring_to_coords(
    ls: &geo_types::LineString<f64>,
) -> Result<Vec<Coordinate>, GeometryError> {
    ls.0.iter()
        .map(|c| Coordinate::new(c.x, c.y))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_types::{Coord, LineString, Point, Polygon};

    #[test]
    fn from_geo_point() {
        let geo_pt = geo_types::Geometry::Point(Point::new(1.0, 2.0));
        let sg = SurrealGeometry::from_geo(&geo_pt, Srid::WGS84).unwrap();
        assert_eq!(sg.type_name(), "Point");
    }

    #[test]
    fn from_geo_linestring() {
        let ls = LineString(vec![Coord { x: 0.0, y: 0.0 }, Coord { x: 1.0, y: 1.0 }]);
        let geo = geo_types::Geometry::LineString(ls);
        let sg = SurrealGeometry::from_geo(&geo, Srid::WGS84).unwrap();
        assert_eq!(sg.type_name(), "LineString");
    }

    #[test]
    fn from_geo_polygon() {
        let ext = LineString(vec![
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 1.0, y: 0.0 },
            Coord { x: 1.0, y: 1.0 },
            Coord { x: 0.0, y: 0.0 },
        ]);
        let poly = Polygon::new(ext, vec![]);
        let geo = geo_types::Geometry::Polygon(poly);
        let sg = SurrealGeometry::from_geo(&geo, Srid::WGS84).unwrap();
        assert_eq!(sg.type_name(), "Polygon");
    }

    #[test]
    fn from_geo_via_from_trait() {
        let geo_pt = geo_types::Geometry::Point(Point::new(5.0, 10.0));
        let sg: SurrealGeometry = geo_pt.into();
        assert_eq!(sg.type_name(), "Point");
        assert_eq!(sg.srid().code(), 4326);
    }

    #[test]
    fn roundtrip_point() {
        let original = SurrealGeometry::point(3.0, 4.0, Srid::WGS84).unwrap();
        let geo = original.to_geo().unwrap();
        let roundtripped = SurrealGeometry::from_geo(&geo, Srid::WGS84).unwrap();
        assert_eq!(original.type_name(), roundtripped.type_name());
        assert_eq!(original.num_points(), roundtripped.num_points());
    }

    #[test]
    fn roundtrip_linestring() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
            Coordinate::new(2.0, 0.0).unwrap(),
        ];
        let original = SurrealGeometry::line_string(coords, Srid::WGS84).unwrap();
        let geo = original.to_geo().unwrap();
        let roundtripped = SurrealGeometry::from_geo(&geo, Srid::WGS84).unwrap();
        assert_eq!(original.num_points(), roundtripped.num_points());
    }

    #[test]
    fn roundtrip_polygon() {
        let exterior = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(10.0, 0.0).unwrap(),
            Coordinate::new(10.0, 10.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        let original = SurrealGeometry::polygon(exterior, vec![], Srid::WGS84).unwrap();
        let geo = original.to_geo().unwrap();
        let roundtripped = SurrealGeometry::from_geo(&geo, Srid::WGS84).unwrap();
        assert_eq!(original.num_points(), roundtripped.num_points());
    }
}
