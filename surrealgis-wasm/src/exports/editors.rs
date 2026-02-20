use surrealism::surrealism;
use surrealdb_types::Geometry;

use crate::adapter;

#[surrealism]
fn st_reverse(geom: Geometry) -> Result<Geometry, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    let result = surrealgis_functions::editors::st_reverse(&g).map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}

#[surrealism]
fn st_force_2d(geom: Geometry) -> Result<Geometry, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    let result = surrealgis_functions::editors::st_force_2d(&g).map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}

#[surrealism]
fn st_snap_to_grid(geom: Geometry, size: f64) -> Result<Geometry, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    let result =
        surrealgis_functions::editors::st_snap_to_grid(&g, size).map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}

#[surrealism]
fn st_collect(geoms: Vec<Geometry>) -> Result<Geometry, String> {
    let domain_geoms: Result<Vec<_>, _> = geoms
        .into_iter()
        .map(adapter::from_surreal_geometry)
        .collect();
    let result =
        surrealgis_functions::editors::st_collect(&domain_geoms?).map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}

#[surrealism]
fn st_multi(geom: Geometry) -> Result<Geometry, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    let result = surrealgis_functions::editors::st_multi(&g).map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}

#[surrealism]
fn st_line_merge(geom: Geometry) -> Result<Geometry, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    let result = surrealgis_functions::editors::st_line_merge(&g).map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}

#[surrealism]
fn st_unary_union(geom: Geometry) -> Result<Geometry, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    let result =
        surrealgis_functions::editors::st_unary_union(&g).map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}
