use surrealism::surrealism;
use surrealdb_types::Geometry;

use crate::adapter;

#[surrealism]
fn st_transform(geom: Geometry, to_srid: i32) -> Result<Geometry, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    let result = surrealgis_functions::crs::st_transform(&g, to_srid)
        .map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}

#[surrealism]
fn st_set_srid(geom: Geometry, new_srid: i32) -> Result<Geometry, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    let result = surrealgis_functions::crs::st_set_srid(&g, new_srid)
        .map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}
