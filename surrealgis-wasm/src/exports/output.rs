use surrealism::surrealism;
use surrealdb_types::Geometry;

use crate::adapter;

#[surrealism]
fn st_as_text(geom: Geometry) -> Result<String, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    surrealgis_functions::output::st_as_text(&g).map_err(|e| e.to_string())
}

#[surrealism]
fn st_as_wkb(geom: Geometry) -> Result<String, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    surrealgis_functions::output::st_as_wkb(&g).map_err(|e| e.to_string())
}

#[surrealism]
fn st_as_geojson(geom: Geometry) -> Result<String, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    surrealgis_functions::output::st_as_geojson(&g).map_err(|e| e.to_string())
}

#[surrealism]
fn st_as_ewkt(geom: Geometry) -> Result<String, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    surrealgis_functions::output::st_as_ewkt(&g).map_err(|e| e.to_string())
}
