use surrealgis_core::coordinate::Coordinate;
use surrealgis_core::geometry::SurrealGeometry;
use surrealgis_core::srid::Srid;

use crate::FunctionError;

/// Create a Polygon from exterior ring coordinates and optional hole rings.
/// Each ring is a slice of (x, y) pairs. Rings must be closed (first == last).
pub fn st_make_polygon(
    exterior: &[(f64, f64)],
    holes: &[Vec<(f64, f64)>],
    srid: i32,
) -> Result<SurrealGeometry, FunctionError> {
    let srid = Srid::new(srid)?;

    let ext_coords: Result<Vec<Coordinate>, _> = exterior
        .iter()
        .map(|(x, y)| Coordinate::new(*x, *y).map_err(FunctionError::from))
        .collect();

    let hole_coords: Result<Vec<Vec<Coordinate>>, _> = holes
        .iter()
        .map(|ring| {
            ring.iter()
                .map(|(x, y)| Coordinate::new(*x, *y).map_err(FunctionError::from))
                .collect()
        })
        .collect();

    let geom = SurrealGeometry::polygon(ext_coords?, hole_coords?, srid)?;
    Ok(geom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_simple_polygon() {
        let exterior = vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 0.0)];
        let poly = st_make_polygon(&exterior, &[], 4326).unwrap();
        assert_eq!(poly.type_name(), "Polygon");
        assert_eq!(poly.num_points(), 4);
    }

    #[test]
    fn make_polygon_with_hole() {
        let exterior = vec![
            (0.0, 0.0),
            (10.0, 0.0),
            (10.0, 10.0),
            (0.0, 10.0),
            (0.0, 0.0),
        ];
        let hole = vec![(2.0, 2.0), (8.0, 2.0), (8.0, 8.0), (2.0, 8.0), (2.0, 2.0)];
        let poly = st_make_polygon(&exterior, &[hole], 4326).unwrap();
        assert_eq!(poly.type_name(), "Polygon");
    }

    #[test]
    fn make_polygon_unclosed_fails() {
        let exterior = vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0)];
        let result = st_make_polygon(&exterior, &[], 4326);
        assert!(result.is_err());
    }
}
