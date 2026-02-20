mod st_intersection;
mod st_union;
mod st_difference;
mod st_sym_difference;

pub use st_intersection::st_intersection;
pub use st_union::st_union;
pub use st_difference::st_difference;
pub use st_sym_difference::st_sym_difference;

use geo_types::{Geometry as GeoGeometry, MultiPolygon};
use surrealgis_core::geometry::SurrealGeometry;

use crate::FunctionError;

/// Extract polygon operands from two SurrealGeometry values, converting
/// Polygon to MultiPolygon for uniform BooleanOps handling.
pub(crate) fn extract_polygon_operands(
    a: &SurrealGeometry,
    b: &SurrealGeometry,
) -> Result<(MultiPolygon<f64>, MultiPolygon<f64>), FunctionError> {
    let ga = a.to_geo()?;
    let gb = b.to_geo()?;
    let mp_a = to_multi_polygon(ga)?;
    let mp_b = to_multi_polygon(gb)?;
    Ok((mp_a, mp_b))
}

fn to_multi_polygon(g: GeoGeometry<f64>) -> Result<MultiPolygon<f64>, FunctionError> {
    match g {
        GeoGeometry::Polygon(p) => Ok(MultiPolygon(vec![p])),
        GeoGeometry::MultiPolygon(mp) => Ok(mp),
        _ => Err(FunctionError::UnsupportedOperation(
            "Overlay operations require Polygon or MultiPolygon inputs".to_string(),
        )),
    }
}
