use surrealism::surrealism;
use surrealdb_types::Geometry;

use crate::adapter;

#[surrealism]
fn st_line_interpolate_point(geom: Geometry, fraction: f64) -> Result<Geometry, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    let result = surrealgis_functions::linear_ref::st_line_interpolate_point(&g, fraction)
        .map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}

#[surrealism]
fn st_line_locate_point(line: Geometry, point: Geometry) -> Result<f64, String> {
    let gl = adapter::from_surreal_geometry(line)?;
    let gp = adapter::from_surreal_geometry(point)?;
    surrealgis_functions::linear_ref::st_line_locate_point(&gl, &gp).map_err(|e| e.to_string())
}

#[surrealism]
fn st_line_substring(
    geom: Geometry,
    start_fraction: f64,
    end_fraction: f64,
) -> Result<Geometry, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    let result =
        surrealgis_functions::linear_ref::st_line_substring(&g, start_fraction, end_fraction)
            .map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}
