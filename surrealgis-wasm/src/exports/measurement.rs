use surrealism::surrealism;
use surrealdb_types::Geometry;

use crate::adapter;

#[surrealism]
fn st_distance(a: Geometry, b: Geometry) -> Result<f64, String> {
    let ga = adapter::from_surreal_geometry(a)?;
    let gb = adapter::from_surreal_geometry(b)?;
    surrealgis_functions::measurement::st_distance(&ga, &gb).map_err(|e| e.to_string())
}

#[surrealism]
fn st_distance_sphere(a: Geometry, b: Geometry) -> Result<f64, String> {
    let ga = adapter::from_surreal_geometry(a)?;
    let gb = adapter::from_surreal_geometry(b)?;
    surrealgis_functions::measurement::st_distance_sphere(&ga, &gb).map_err(|e| e.to_string())
}

#[surrealism]
fn st_area(geom: Geometry) -> Result<f64, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    surrealgis_functions::measurement::st_area(&g).map_err(|e| e.to_string())
}

#[surrealism]
fn st_length(geom: Geometry) -> Result<f64, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    surrealgis_functions::measurement::st_length(&g).map_err(|e| e.to_string())
}

#[surrealism]
fn st_perimeter(geom: Geometry) -> Result<f64, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    surrealgis_functions::measurement::st_perimeter(&g).map_err(|e| e.to_string())
}

#[surrealism]
fn st_azimuth(a: Geometry, b: Geometry) -> Result<f64, String> {
    let ga = adapter::from_surreal_geometry(a)?;
    let gb = adapter::from_surreal_geometry(b)?;
    surrealgis_functions::measurement::st_azimuth(&ga, &gb).map_err(|e| e.to_string())
}

#[surrealism]
fn st_dwithin(a: Geometry, b: Geometry, distance: f64) -> Result<bool, String> {
    let ga = adapter::from_surreal_geometry(a)?;
    let gb = adapter::from_surreal_geometry(b)?;
    surrealgis_functions::measurement::st_dwithin(&ga, &gb, distance).map_err(|e| e.to_string())
}
