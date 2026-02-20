use surrealgis_core::coordinate::Coordinate;
use surrealgis_core::geometry::SurrealGeometry;
use surrealgis_core::srid::Srid;

use crate::FunctionError;

/// Create a rectangular Polygon from bounding box coordinates.
pub fn st_make_envelope(
    xmin: f64,
    ymin: f64,
    xmax: f64,
    ymax: f64,
    srid: i32,
) -> Result<SurrealGeometry, FunctionError> {
    if xmin > xmax {
        return Err(FunctionError::InvalidArgument(format!(
            "xmin ({xmin}) must be <= xmax ({xmax})"
        )));
    }
    if ymin > ymax {
        return Err(FunctionError::InvalidArgument(format!(
            "ymin ({ymin}) must be <= ymax ({ymax})"
        )));
    }

    let srid = Srid::new(srid)?;
    let exterior = vec![
        Coordinate::new(xmin, ymin)?,
        Coordinate::new(xmax, ymin)?,
        Coordinate::new(xmax, ymax)?,
        Coordinate::new(xmin, ymax)?,
        Coordinate::new(xmin, ymin)?,
    ];
    let geom = SurrealGeometry::polygon(exterior, vec![], srid)?;
    Ok(geom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_envelope() {
        let env = st_make_envelope(0.0, 0.0, 10.0, 10.0, 4326).unwrap();
        assert_eq!(env.type_name(), "Polygon");
        assert_eq!(env.num_points(), 5);
        let bb = env.bbox().unwrap();
        assert_eq!(bb.min_x, 0.0);
        assert_eq!(bb.min_y, 0.0);
        assert_eq!(bb.max_x, 10.0);
        assert_eq!(bb.max_y, 10.0);
    }

    #[test]
    fn make_envelope_invalid_range() {
        let result = st_make_envelope(10.0, 0.0, 0.0, 10.0, 4326);
        assert!(result.is_err());
    }

    #[test]
    fn make_envelope_degenerate_line() {
        // xmin == xmax creates a degenerate polygon
        let result = st_make_envelope(5.0, 0.0, 5.0, 10.0, 4326);
        // This should still work as a valid polygon (degenerate)
        assert!(result.is_ok());
    }
}
