use geo_types::{
    Geometry, GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon, Polygon,
};
use surrealgis_core::geometry::SurrealGeometry;

use crate::FunctionError;

/// Reverse the coordinate order of a geometry.
/// For Point: no-op. For LineString: reverses the coord vector.
/// For Polygon: reverses exterior and each hole ring.
/// For Multi types: reverses each sub-geometry.
/// For GeometryCollection: reverses each child.
pub fn st_reverse(geom: &SurrealGeometry) -> Result<SurrealGeometry, FunctionError> {
    let geo_geom = geom.to_geo()?;
    let reversed = reverse_geometry(geo_geom);
    SurrealGeometry::from_geo(&reversed, *geom.srid()).map_err(FunctionError::from)
}

fn reverse_geometry(g: Geometry<f64>) -> Geometry<f64> {
    match g {
        Geometry::Point(p) => Geometry::Point(p),
        Geometry::LineString(ls) => Geometry::LineString(reverse_linestring(ls)),
        Geometry::Polygon(p) => {
            let ext = reverse_linestring(p.exterior().clone());
            let holes: Vec<LineString<f64>> =
                p.interiors().iter().map(|h| reverse_linestring(h.clone())).collect();
            Geometry::Polygon(Polygon::new(ext, holes))
        }
        Geometry::MultiPoint(mp) => {
            // Reverse the order of points in the collection
            let mut points = mp.0;
            points.reverse();
            Geometry::MultiPoint(MultiPoint(points))
        }
        Geometry::MultiLineString(mls) => {
            let lines: Vec<LineString<f64>> =
                mls.0.into_iter().map(reverse_linestring).collect();
            Geometry::MultiLineString(MultiLineString(lines))
        }
        Geometry::MultiPolygon(mp) => {
            let polys: Vec<Polygon<f64>> = mp.0.into_iter().map(|p| {
                let ext = reverse_linestring(p.exterior().clone());
                let holes: Vec<LineString<f64>> =
                    p.interiors().iter().map(|h| reverse_linestring(h.clone())).collect();
                Polygon::new(ext, holes)
            }).collect();
            Geometry::MultiPolygon(MultiPolygon(polys))
        }
        Geometry::GeometryCollection(gc) => {
            let geoms: Vec<Geometry<f64>> =
                gc.0.into_iter().map(reverse_geometry).collect();
            Geometry::GeometryCollection(GeometryCollection(geoms))
        }
        other => other,
    }
}

fn reverse_linestring(mut ls: LineString<f64>) -> LineString<f64> {
    ls.0.reverse();
    ls
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::coordinate::Coordinate;
    use surrealgis_core::geometry::GeometryType;
    use surrealgis_core::srid::Srid;

    #[test]
    fn reverse_point_is_noop() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        let result = st_reverse(&p).unwrap();
        if let GeometryType::Point(c) = result.geometry_type() {
            assert!((c.x() - 1.0).abs() < 1e-10);
            assert!((c.y() - 2.0).abs() < 1e-10);
        } else {
            panic!("Expected Point");
        }
    }

    #[test]
    fn reverse_linestring() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
            Coordinate::new(2.0, 2.0).unwrap(),
        ];
        let line = SurrealGeometry::line_string(coords, Srid::WGS84).unwrap();
        let result = st_reverse(&line).unwrap();
        if let GeometryType::LineString(cs) = result.geometry_type() {
            assert!((cs[0].x() - 2.0).abs() < 1e-10);
            assert!((cs[0].y() - 2.0).abs() < 1e-10);
            assert!((cs[2].x() - 0.0).abs() < 1e-10);
            assert!((cs[2].y() - 0.0).abs() < 1e-10);
        } else {
            panic!("Expected LineString");
        }
    }

    #[test]
    fn reverse_polygon() {
        let exterior = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(10.0, 0.0).unwrap(),
            Coordinate::new(10.0, 10.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        let poly = SurrealGeometry::polygon(exterior, vec![], Srid::WGS84).unwrap();
        let result = st_reverse(&poly).unwrap();
        if let GeometryType::Polygon { exterior, .. } = result.geometry_type() {
            // Original: (0,0), (10,0), (10,10), (0,0)
            // Reversed: (0,0), (10,10), (10,0), (0,0)
            assert!((exterior[0].x() - 0.0).abs() < 1e-10);
            assert!((exterior[0].y() - 0.0).abs() < 1e-10);
            assert!((exterior[1].x() - 10.0).abs() < 1e-10);
            assert!((exterior[1].y() - 10.0).abs() < 1e-10);
            assert!((exterior[2].x() - 10.0).abs() < 1e-10);
            assert!((exterior[2].y() - 0.0).abs() < 1e-10);
        } else {
            panic!("Expected Polygon");
        }
    }

    #[test]
    fn reverse_preserves_srid() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WEB_MERCATOR).unwrap();
        let result = st_reverse(&p).unwrap();
        assert_eq!(result.srid().code(), Srid::WEB_MERCATOR.code());
    }

    #[test]
    fn reverse_multi_linestring() {
        let lines = vec![
            vec![
                Coordinate::new(0.0, 0.0).unwrap(),
                Coordinate::new(1.0, 1.0).unwrap(),
            ],
            vec![
                Coordinate::new(2.0, 2.0).unwrap(),
                Coordinate::new(3.0, 3.0).unwrap(),
            ],
        ];
        let mls = SurrealGeometry::multi_line_string(lines, Srid::WGS84).unwrap();
        let result = st_reverse(&mls).unwrap();
        if let GeometryType::MultiLineString(ls) = result.geometry_type() {
            // First line reversed: (1,1), (0,0)
            assert!((ls[0][0].x() - 1.0).abs() < 1e-10);
            assert!((ls[0][0].y() - 1.0).abs() < 1e-10);
            assert!((ls[0][1].x() - 0.0).abs() < 1e-10);
        } else {
            panic!("Expected MultiLineString");
        }
    }
}
