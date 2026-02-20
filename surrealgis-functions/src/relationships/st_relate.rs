use geo::algorithm::Relate;
use geo::relate::IntersectionMatrix;
use surrealgis_core::geometry::SurrealGeometry;

use crate::FunctionError;

/// Format an IntersectionMatrix as a 9-character DE-9IM string.
fn matrix_to_string(matrix: &IntersectionMatrix) -> String {
    // The IntersectionMatrix Debug format is "IntersectionMatrix(FF2F11212)"
    // We need to extract just the 9-char matrix string
    let debug_str = format!("{matrix:?}");
    // Try to extract the part between parentheses
    if let Some(start) = debug_str.find('(') {
        if let Some(end) = debug_str.find(')') {
            return debug_str[start + 1..end].to_string();
        }
    }
    // Fallback: just use the debug output
    debug_str
}

/// Returns the DE-9IM intersection matrix string (9 characters like "FF2F11212").
pub fn st_relate(a: &SurrealGeometry, b: &SurrealGeometry) -> Result<String, FunctionError> {
    let ga = a.to_geo()?;
    let gb = b.to_geo()?;
    let matrix = ga.relate(&gb);
    Ok(matrix_to_string(&matrix))
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::coordinate::Coordinate;
    use surrealgis_core::srid::Srid;

    #[test]
    fn relate_returns_9_char_matrix() {
        let a = SurrealGeometry::point(0.0, 0.0, Srid::WGS84).unwrap();
        let b = SurrealGeometry::point(1.0, 1.0, Srid::WGS84).unwrap();
        let matrix = st_relate(&a, &b).unwrap();
        assert_eq!(matrix.len(), 9, "Matrix was: {matrix}");
    }

    #[test]
    fn relate_identical_points() {
        let a = SurrealGeometry::point(1.0, 1.0, Srid::WGS84).unwrap();
        let b = SurrealGeometry::point(1.0, 1.0, Srid::WGS84).unwrap();
        let matrix = st_relate(&a, &b).unwrap();
        assert_eq!(matrix.len(), 9, "Matrix was: {matrix}");
        // Two identical points: "0FFFFFFF2"
        assert_eq!(matrix, "0FFFFFFF2");
    }

    #[test]
    fn relate_overlapping_polygons() {
        let poly_a = SurrealGeometry::polygon(
            vec![
                Coordinate::new(0.0, 0.0).unwrap(),
                Coordinate::new(2.0, 0.0).unwrap(),
                Coordinate::new(2.0, 2.0).unwrap(),
                Coordinate::new(0.0, 2.0).unwrap(),
                Coordinate::new(0.0, 0.0).unwrap(),
            ],
            vec![],
            Srid::WGS84,
        )
        .unwrap();
        let poly_b = SurrealGeometry::polygon(
            vec![
                Coordinate::new(1.0, 1.0).unwrap(),
                Coordinate::new(3.0, 1.0).unwrap(),
                Coordinate::new(3.0, 3.0).unwrap(),
                Coordinate::new(1.0, 3.0).unwrap(),
                Coordinate::new(1.0, 1.0).unwrap(),
            ],
            vec![],
            Srid::WGS84,
        )
        .unwrap();
        let matrix = st_relate(&poly_a, &poly_b).unwrap();
        assert_eq!(matrix.len(), 9, "Matrix was: {matrix}");
        // Overlapping polygons should have "2" in the first position (interior-interior)
        assert_eq!(&matrix[0..1], "2");
    }
}
