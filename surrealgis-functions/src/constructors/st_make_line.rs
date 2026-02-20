use surrealgis_core::coordinate::Coordinate;
use surrealgis_core::geometry::SurrealGeometry;
use surrealgis_core::srid::Srid;

use crate::FunctionError;

/// Create a LineString from a vector of (x, y) coordinate pairs.
pub fn st_make_line(points: &[(f64, f64)], srid: i32) -> Result<SurrealGeometry, FunctionError> {
    if points.len() < 2 {
        return Err(FunctionError::InvalidArgument(
            "st_make_line requires at least 2 points".to_string(),
        ));
    }
    let srid = Srid::new(srid)?;
    let coords: Result<Vec<Coordinate>, _> = points
        .iter()
        .map(|(x, y)| Coordinate::new(*x, *y).map_err(FunctionError::from))
        .collect();
    let geom = SurrealGeometry::line_string(coords?, srid)?;
    Ok(geom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_line_from_coords() {
        let line = st_make_line(&[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)], 4326).unwrap();
        assert_eq!(line.type_name(), "LineString");
        assert_eq!(line.num_points(), 3);
    }

    #[test]
    fn make_line_too_few_points() {
        let result = st_make_line(&[(0.0, 0.0)], 4326);
        assert!(result.is_err());
    }
}
