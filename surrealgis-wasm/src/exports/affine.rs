use surrealism::surrealism;
use surrealdb_types::Geometry;

use crate::adapter;

#[surrealism]
fn st_translate(geom: Geometry, dx: f64, dy: f64) -> Result<Geometry, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    let result = surrealgis_functions::affine::st_translate(&g, dx, dy).map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}

#[surrealism]
fn st_rotate(geom: Geometry, angle_degrees: f64) -> Result<Geometry, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    let result = surrealgis_functions::affine::st_rotate(&g, angle_degrees).map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}

#[surrealism]
fn st_scale(geom: Geometry, sx: f64, sy: f64) -> Result<Geometry, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    let result = surrealgis_functions::affine::st_scale(&g, sx, sy).map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}

#[surrealism]
fn st_affine(geom: Geometry, a: f64, b: f64, d: f64, e: f64, xoff: f64, yoff: f64) -> Result<Geometry, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    let result = surrealgis_functions::affine::st_affine(&g, a, b, d, e, xoff, yoff).map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}
