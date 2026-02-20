use geo::BooleanOps;
use geo_types::{Geometry, MultiPolygon, Polygon};
use surrealgis_core::geometry::SurrealGeometry;

use crate::FunctionError;

/// Compute the union of all polygon components in a geometry.
/// Accepts MultiPolygon, Polygon, or GeometryCollection containing polygons.
/// Uses iterative BooleanOps::union() to merge all polygons.
pub fn st_unary_union(geom: &SurrealGeometry) -> Result<SurrealGeometry, FunctionError> {
    let geo_geom = geom.to_geo()?;
    let polygons = extract_polygons(geo_geom)?;

    if polygons.is_empty() {
        return Err(FunctionError::InvalidArgument(
            "st_unary_union: no polygons found in geometry".to_string(),
        ));
    }

    // Single polygon: return as-is
    if polygons.len() == 1 {
        let result = Geometry::Polygon(polygons.into_iter().next().unwrap());
        return SurrealGeometry::from_geo(&result, *geom.srid()).map_err(FunctionError::from);
    }

    // Iteratively union all polygons
    let mut result = MultiPolygon(vec![polygons[0].clone()]);
    for poly in &polygons[1..] {
        let mp = MultiPolygon(vec![poly.clone()]);
        result = result.union(&mp);
    }

    let geo_result = if result.0.len() == 1 {
        Geometry::Polygon(result.0.into_iter().next().unwrap())
    } else {
        Geometry::MultiPolygon(result)
    };

    SurrealGeometry::from_geo(&geo_result, *geom.srid()).map_err(FunctionError::from)
}

/// Extract all Polygon geometries from a Geometry, descending into Multi and Collection types.
fn extract_polygons(g: Geometry<f64>) -> Result<Vec<Polygon<f64>>, FunctionError> {
    match g {
        Geometry::Polygon(p) => Ok(vec![p]),
        Geometry::MultiPolygon(mp) => Ok(mp.0),
        Geometry::GeometryCollection(gc) => {
            let mut polys = Vec::new();
            for child in gc.0 {
                let mut child_polys = extract_polygons(child)?;
                polys.append(&mut child_polys);
            }
            if polys.is_empty() {
                Err(FunctionError::InvalidArgument(
                    "st_unary_union: GeometryCollection contains no polygons".to_string(),
                ))
            } else {
                Ok(polys)
            }
        }
        _ => Err(FunctionError::UnsupportedOperation(
            "st_unary_union requires Polygon, MultiPolygon, or GeometryCollection input"
                .to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::coordinate::Coordinate;
    use surrealgis_core::geometry::PolygonData;
    use surrealgis_core::srid::Srid;

    fn make_square(x: f64, y: f64, size: f64, srid: Srid) -> SurrealGeometry {
        let exterior = vec![
            Coordinate::new(x, y).unwrap(),
            Coordinate::new(x + size, y).unwrap(),
            Coordinate::new(x + size, y + size).unwrap(),
            Coordinate::new(x, y + size).unwrap(),
            Coordinate::new(x, y).unwrap(),
        ];
        SurrealGeometry::polygon(exterior, vec![], srid).unwrap()
    }

    #[test]
    fn unary_union_single_polygon() {
        let poly = make_square(0.0, 0.0, 10.0, Srid::WEB_MERCATOR);
        let result = st_unary_union(&poly).unwrap();
        assert_eq!(result.type_name(), "Polygon");
    }

    #[test]
    fn unary_union_overlapping_multipolygon() {
        // Two overlapping squares should merge into one polygon
        let polygons = vec![
            PolygonData {
                exterior: vec![
                    Coordinate::new(0.0, 0.0).unwrap(),
                    Coordinate::new(10.0, 0.0).unwrap(),
                    Coordinate::new(10.0, 10.0).unwrap(),
                    Coordinate::new(0.0, 10.0).unwrap(),
                    Coordinate::new(0.0, 0.0).unwrap(),
                ],
                holes: vec![],
            },
            PolygonData {
                exterior: vec![
                    Coordinate::new(5.0, 0.0).unwrap(),
                    Coordinate::new(15.0, 0.0).unwrap(),
                    Coordinate::new(15.0, 10.0).unwrap(),
                    Coordinate::new(5.0, 10.0).unwrap(),
                    Coordinate::new(5.0, 0.0).unwrap(),
                ],
                holes: vec![],
            },
        ];
        let mp = SurrealGeometry::multi_polygon(polygons, Srid::WEB_MERCATOR).unwrap();
        let result = st_unary_union(&mp).unwrap();
        // Overlapping polygons should merge into a single polygon
        assert_eq!(result.type_name(), "Polygon");
    }

    #[test]
    fn unary_union_disjoint_multipolygon() {
        // Two disjoint squares should remain as MultiPolygon
        let polygons = vec![
            PolygonData {
                exterior: vec![
                    Coordinate::new(0.0, 0.0).unwrap(),
                    Coordinate::new(1.0, 0.0).unwrap(),
                    Coordinate::new(1.0, 1.0).unwrap(),
                    Coordinate::new(0.0, 1.0).unwrap(),
                    Coordinate::new(0.0, 0.0).unwrap(),
                ],
                holes: vec![],
            },
            PolygonData {
                exterior: vec![
                    Coordinate::new(10.0, 10.0).unwrap(),
                    Coordinate::new(11.0, 10.0).unwrap(),
                    Coordinate::new(11.0, 11.0).unwrap(),
                    Coordinate::new(10.0, 11.0).unwrap(),
                    Coordinate::new(10.0, 10.0).unwrap(),
                ],
                holes: vec![],
            },
        ];
        let mp = SurrealGeometry::multi_polygon(polygons, Srid::WEB_MERCATOR).unwrap();
        let result = st_unary_union(&mp).unwrap();
        assert_eq!(result.type_name(), "MultiPolygon");
    }

    #[test]
    fn unary_union_point_rejected() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        let result = st_unary_union(&p);
        assert!(result.is_err());
    }

    #[test]
    fn unary_union_preserves_srid() {
        let poly = make_square(0.0, 0.0, 10.0, Srid::WEB_MERCATOR);
        let result = st_unary_union(&poly).unwrap();
        assert_eq!(result.srid().code(), Srid::WEB_MERCATOR.code());
    }
}
