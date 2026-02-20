use geo::algorithm::{BoundingRect, Centroid, InteriorPoint};
use surrealgis_core::coordinate::Coordinate;
use surrealgis_core::geometry::{GeometryType, SurrealGeometry};

use crate::FunctionError;

/// Return the bounding box of a geometry as a Polygon.
pub fn st_envelope(geom: &SurrealGeometry) -> Result<SurrealGeometry, FunctionError> {
    let geo_geom = geom.to_geo()?;
    let rect = geo_geom
        .bounding_rect()
        .ok_or_else(|| FunctionError::InvalidArgument("Cannot compute envelope".to_string()))?;

    let min = rect.min();
    let max = rect.max();
    let exterior = vec![
        Coordinate::new(min.x, min.y)?,
        Coordinate::new(max.x, min.y)?,
        Coordinate::new(max.x, max.y)?,
        Coordinate::new(min.x, max.y)?,
        Coordinate::new(min.x, min.y)?,
    ];
    Ok(SurrealGeometry::polygon(exterior, vec![], *geom.srid())?)
}

/// Return the centroid of a geometry as a Point.
pub fn st_centroid(geom: &SurrealGeometry) -> Result<SurrealGeometry, FunctionError> {
    let geo_geom = geom.to_geo()?;
    let centroid = geo_geom
        .centroid()
        .ok_or_else(|| FunctionError::InvalidArgument("Cannot compute centroid".to_string()))?;
    Ok(SurrealGeometry::point(centroid.x(), centroid.y(), *geom.srid())?)
}

/// Return a point guaranteed to lie on the surface of the geometry.
pub fn st_point_on_surface(geom: &SurrealGeometry) -> Result<SurrealGeometry, FunctionError> {
    let geo_geom = geom.to_geo()?;
    let interior_point = match &geo_geom {
        geo_types::Geometry::Polygon(p) => p.interior_point(),
        geo_types::Geometry::MultiPolygon(mp) => mp.interior_point(),
        geo_types::Geometry::LineString(ls) => ls.interior_point(),
        geo_types::Geometry::MultiLineString(mls) => mls.interior_point(),
        geo_types::Geometry::Point(p) => Some(*p),
        geo_types::Geometry::MultiPoint(mp) => mp.interior_point(),
        _ => {
            return Err(FunctionError::UnsupportedOperation(
                "st_point_on_surface not supported for this geometry type".to_string(),
            ))
        }
    };

    let pt = interior_point.ok_or_else(|| {
        FunctionError::InvalidArgument("Cannot compute interior point".to_string())
    })?;
    Ok(SurrealGeometry::point(pt.x(), pt.y(), *geom.srid())?)
}

/// Return the boundary of a geometry.
/// For a Polygon, the boundary is its exterior ring as a LineString.
/// For a LineString, the boundary is the start and end points as a MultiPoint.
pub fn st_boundary(geom: &SurrealGeometry) -> Result<SurrealGeometry, FunctionError> {
    match geom.geometry_type() {
        GeometryType::Polygon { exterior, .. } => {
            // Boundary of a polygon is its exterior ring
            Ok(SurrealGeometry::line_string(exterior.clone(), *geom.srid())?)
        }
        GeometryType::LineString(coords) => {
            if coords.is_empty() || coords.len() < 2 {
                return Err(FunctionError::InvalidArgument(
                    "Cannot compute boundary of empty LineString".to_string(),
                ));
            }
            // If closed, boundary is empty (MultiPoint with 0 points) -> return error
            if coords.first() == coords.last() {
                // Closed linestring has empty boundary; return first point
                let c = &coords[0];
                Ok(SurrealGeometry::point(c.x(), c.y(), *geom.srid())?)
            } else {
                // Open linestring boundary = start + end as MultiPoint
                let start = coords.first().unwrap().clone();
                let end = coords.last().unwrap().clone();
                Ok(SurrealGeometry::multi_point(vec![start, end], *geom.srid())?)
            }
        }
        _ => Err(FunctionError::UnsupportedOperation(
            "st_boundary only supports Polygon and LineString geometries".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::srid::Srid;

    fn make_polygon() -> SurrealGeometry {
        let exterior = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(10.0, 0.0).unwrap(),
            Coordinate::new(10.0, 10.0).unwrap(),
            Coordinate::new(0.0, 10.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        SurrealGeometry::polygon(exterior, vec![], Srid::WGS84).unwrap()
    }

    #[test]
    fn test_st_envelope() {
        let poly = make_polygon();
        let env = st_envelope(&poly).unwrap();
        assert_eq!(env.type_name(), "Polygon");
        let bb = env.bbox().unwrap();
        assert_eq!(bb.min_x, 0.0);
        assert_eq!(bb.max_x, 10.0);
    }

    #[test]
    fn test_st_centroid() {
        let poly = make_polygon();
        let center = st_centroid(&poly).unwrap();
        assert_eq!(center.type_name(), "Point");
        if let GeometryType::Point(c) = center.geometry_type() {
            assert!((c.x() - 5.0).abs() < 1e-10);
            assert!((c.y() - 5.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_st_centroid_linestring() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(10.0, 0.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WGS84).unwrap();
        let center = st_centroid(&ls).unwrap();
        if let GeometryType::Point(c) = center.geometry_type() {
            assert!((c.x() - 5.0).abs() < 1e-10);
            assert!((c.y() - 0.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_st_point_on_surface() {
        let poly = make_polygon();
        let pt = st_point_on_surface(&poly).unwrap();
        assert_eq!(pt.type_name(), "Point");
        // The interior point should be inside the polygon
        if let GeometryType::Point(c) = pt.geometry_type() {
            assert!(c.x() >= 0.0 && c.x() <= 10.0);
            assert!(c.y() >= 0.0 && c.y() <= 10.0);
        }
    }

    #[test]
    fn test_st_boundary_polygon() {
        let poly = make_polygon();
        let boundary = st_boundary(&poly).unwrap();
        // Boundary of a polygon is a LineString (or MultiLineString)
        assert!(
            boundary.type_name() == "LineString"
                || boundary.type_name() == "MultiLineString"
        );
    }

    #[test]
    fn test_st_envelope_point() {
        let p = SurrealGeometry::point(5.0, 10.0, Srid::WGS84).unwrap();
        let env = st_envelope(&p).unwrap();
        assert_eq!(env.type_name(), "Polygon");
    }
}
