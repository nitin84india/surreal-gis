use surrealdb_types::Geometry;
use surrealgis_core::geometry::SurrealGeometry;
use surrealgis_core::srid::Srid;

/// Convert a `surrealdb_types::Geometry` into a domain `SurrealGeometry`.
/// SurrealDB geometry has no SRID concept, so we default to WGS84.
pub fn from_surreal_geometry(g: Geometry) -> Result<SurrealGeometry, String> {
    let geo: geo_types::Geometry<f64> = surreal_geometry_to_geo(g);
    SurrealGeometry::from_geo(&geo, Srid::WGS84).map_err(|e| e.to_string())
}

/// Convert a domain `SurrealGeometry` into a `surrealdb_types::Geometry`.
pub fn to_surreal_geometry(g: &SurrealGeometry) -> Result<Geometry, String> {
    let geo = g.to_geo().map_err(|e| e.to_string())?;
    Ok(geo_to_surreal_geometry(geo))
}

/// Convert `surrealdb_types::Geometry` to `geo_types::Geometry<f64>`.
fn surreal_geometry_to_geo(g: Geometry) -> geo_types::Geometry<f64> {
    match g {
        Geometry::Point(p) => geo_types::Geometry::Point(p),
        Geometry::Line(l) => geo_types::Geometry::LineString(l),
        Geometry::Polygon(p) => geo_types::Geometry::Polygon(p),
        Geometry::MultiPoint(mp) => geo_types::Geometry::MultiPoint(mp),
        Geometry::MultiLine(ml) => geo_types::Geometry::MultiLineString(ml),
        Geometry::MultiPolygon(mp) => geo_types::Geometry::MultiPolygon(mp),
        Geometry::Collection(c) => {
            let geoms: Vec<geo_types::Geometry<f64>> =
                c.into_iter().map(surreal_geometry_to_geo).collect();
            geo_types::Geometry::GeometryCollection(geo_types::GeometryCollection(geoms))
        }
    }
}

/// Convert `geo_types::Geometry<f64>` to `surrealdb_types::Geometry`.
fn geo_to_surreal_geometry(g: geo_types::Geometry<f64>) -> Geometry {
    match g {
        geo_types::Geometry::Point(p) => Geometry::from_point(p),
        geo_types::Geometry::LineString(l) => Geometry::from_line(l),
        geo_types::Geometry::Polygon(p) => Geometry::from_polygon(p),
        geo_types::Geometry::MultiPoint(mp) => Geometry::from_multipoint(mp),
        geo_types::Geometry::MultiLineString(ml) => Geometry::from_multiline(ml),
        geo_types::Geometry::MultiPolygon(mp) => Geometry::from_multipolygon(mp),
        geo_types::Geometry::GeometryCollection(gc) => {
            let geoms: Vec<Geometry> = gc.0.into_iter().map(geo_to_surreal_geometry).collect();
            Geometry::from_collection(geoms)
        }
        geo_types::Geometry::Line(l) => {
            // Convert Line to LineString (2-point)
            Geometry::from_line(geo_types::LineString::from(l))
        }
        geo_types::Geometry::Rect(r) => {
            // Convert Rect to Polygon
            Geometry::from_polygon(r.to_polygon())
        }
        geo_types::Geometry::Triangle(t) => {
            // Convert Triangle to Polygon
            Geometry::from_polygon(t.to_polygon())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_point() {
        let surreal_geom = Geometry::from_point(geo_types::Point::new(1.0, 2.0));
        let domain = from_surreal_geometry(surreal_geom).unwrap();
        assert_eq!(domain.type_name(), "Point");
        let back = to_surreal_geometry(&domain).unwrap();
        assert!(back.is_point());
        let pt = back.into_point().unwrap();
        assert_eq!(pt.x(), 1.0);
        assert_eq!(pt.y(), 2.0);
    }

    #[test]
    fn roundtrip_linestring() {
        let line = geo_types::LineString::from(vec![(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)]);
        let surreal_geom = Geometry::from_line(line);
        let domain = from_surreal_geometry(surreal_geom).unwrap();
        assert_eq!(domain.type_name(), "LineString");
        let back = to_surreal_geometry(&domain).unwrap();
        assert!(back.is_line());
    }

    #[test]
    fn roundtrip_polygon() {
        let exterior =
            geo_types::LineString::from(vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 0.0)]);
        let polygon = geo_types::Polygon::new(exterior, vec![]);
        let surreal_geom = Geometry::from_polygon(polygon);
        let domain = from_surreal_geometry(surreal_geom).unwrap();
        assert_eq!(domain.type_name(), "Polygon");
        let back = to_surreal_geometry(&domain).unwrap();
        assert!(back.is_polygon());
    }

    #[test]
    fn roundtrip_multipoint() {
        let mp = geo_types::MultiPoint::new(vec![
            geo_types::Point::new(0.0, 0.0),
            geo_types::Point::new(1.0, 1.0),
        ]);
        let surreal_geom = Geometry::from_multipoint(mp);
        let domain = from_surreal_geometry(surreal_geom).unwrap();
        assert_eq!(domain.type_name(), "MultiPoint");
        let back = to_surreal_geometry(&domain).unwrap();
        assert!(back.is_multipoint());
    }

    #[test]
    fn roundtrip_collection() {
        let p = Geometry::from_point(geo_types::Point::new(1.0, 2.0));
        let line = geo_types::LineString::from(vec![(0.0, 0.0), (1.0, 1.0)]);
        let l = Geometry::from_line(line);
        let collection = Geometry::from_collection(vec![p, l]);
        let domain = from_surreal_geometry(collection).unwrap();
        assert_eq!(domain.type_name(), "GeometryCollection");
        let back = to_surreal_geometry(&domain).unwrap();
        assert!(back.is_collection());
    }
}
