use geo::{AffineOps, AffineTransform};
use surrealgis_core::geometry::SurrealGeometry;

use crate::FunctionError;

/// Apply a general 2D affine transformation to a geometry.
///
/// The transformation matrix is:
/// ```text
/// | a  b  xoff |
/// | d  e  yoff |
/// | 0  0  1    |
/// ```
///
/// New coordinates: x' = a*x + b*y + xoff, y' = d*x + e*y + yoff
pub fn st_affine(
    geom: &SurrealGeometry,
    a: f64,
    b: f64,
    d: f64,
    e: f64,
    xoff: f64,
    yoff: f64,
) -> Result<SurrealGeometry, FunctionError> {
    let geo_geom = geom.to_geo()?;
    let transform = AffineTransform::new(a, b, xoff, d, e, yoff);
    let result = geo_geom.affine_transform(&transform);
    SurrealGeometry::from_geo(&result, *geom.srid()).map_err(FunctionError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::geometry::GeometryType;
    use surrealgis_core::srid::Srid;

    #[test]
    fn affine_identity() {
        // Identity: a=1, b=0, d=0, e=1, xoff=0, yoff=0
        let p = SurrealGeometry::point(3.0, 4.0, Srid::WEB_MERCATOR).unwrap();
        let result = st_affine(&p, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0).unwrap();
        if let GeometryType::Point(c) = result.geometry_type() {
            assert!((c.x() - 3.0).abs() < 1e-10);
            assert!((c.y() - 4.0).abs() < 1e-10);
        } else {
            panic!("Expected Point");
        }
    }

    #[test]
    fn affine_translation() {
        // Pure translation: a=1, b=0, d=0, e=1, xoff=10, yoff=20
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WEB_MERCATOR).unwrap();
        let result = st_affine(&p, 1.0, 0.0, 0.0, 1.0, 10.0, 20.0).unwrap();
        if let GeometryType::Point(c) = result.geometry_type() {
            assert!((c.x() - 11.0).abs() < 1e-10);
            assert!((c.y() - 22.0).abs() < 1e-10);
        } else {
            panic!("Expected Point");
        }
    }

    #[test]
    fn affine_scale() {
        // Scale by 2: a=2, b=0, d=0, e=2, xoff=0, yoff=0
        let p = SurrealGeometry::point(3.0, 4.0, Srid::WEB_MERCATOR).unwrap();
        let result = st_affine(&p, 2.0, 0.0, 0.0, 2.0, 0.0, 0.0).unwrap();
        if let GeometryType::Point(c) = result.geometry_type() {
            assert!((c.x() - 6.0).abs() < 1e-10);
            assert!((c.y() - 8.0).abs() < 1e-10);
        } else {
            panic!("Expected Point");
        }
    }

    #[test]
    fn affine_preserves_srid() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WEB_MERCATOR).unwrap();
        let result = st_affine(&p, 1.0, 0.0, 0.0, 1.0, 5.0, 5.0).unwrap();
        assert_eq!(result.srid().code(), Srid::WEB_MERCATOR.code());
    }
}
