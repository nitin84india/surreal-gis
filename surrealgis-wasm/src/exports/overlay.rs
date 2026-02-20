use surrealism::surrealism;
use surrealdb_types::Geometry;

use crate::adapter;

#[surrealism]
fn st_intersection(a: Geometry, b: Geometry) -> Result<Geometry, String> {
    let ga = adapter::from_surreal_geometry(a)?;
    let gb = adapter::from_surreal_geometry(b)?;
    let result =
        surrealgis_functions::overlay::st_intersection(&ga, &gb).map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}

#[surrealism]
fn st_union(a: Geometry, b: Geometry) -> Result<Geometry, String> {
    let ga = adapter::from_surreal_geometry(a)?;
    let gb = adapter::from_surreal_geometry(b)?;
    let result =
        surrealgis_functions::overlay::st_union(&ga, &gb).map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}

#[surrealism]
fn st_difference(a: Geometry, b: Geometry) -> Result<Geometry, String> {
    let ga = adapter::from_surreal_geometry(a)?;
    let gb = adapter::from_surreal_geometry(b)?;
    let result =
        surrealgis_functions::overlay::st_difference(&ga, &gb).map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}

#[surrealism]
fn st_sym_difference(a: Geometry, b: Geometry) -> Result<Geometry, String> {
    let ga = adapter::from_surreal_geometry(a)?;
    let gb = adapter::from_surreal_geometry(b)?;
    let result =
        surrealgis_functions::overlay::st_sym_difference(&ga, &gb).map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}
