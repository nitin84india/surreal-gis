use surrealism::surrealism;
use surrealdb_types::Geometry;

use crate::adapter;

#[surrealism]
fn st_intersects(a: Geometry, b: Geometry) -> Result<bool, String> {
    let ga = adapter::from_surreal_geometry(a)?;
    let gb = adapter::from_surreal_geometry(b)?;
    surrealgis_functions::relationships::st_intersects(&ga, &gb).map_err(|e| e.to_string())
}

#[surrealism]
fn st_contains(a: Geometry, b: Geometry) -> Result<bool, String> {
    let ga = adapter::from_surreal_geometry(a)?;
    let gb = adapter::from_surreal_geometry(b)?;
    surrealgis_functions::relationships::st_contains(&ga, &gb).map_err(|e| e.to_string())
}

#[surrealism]
fn st_within(a: Geometry, b: Geometry) -> Result<bool, String> {
    let ga = adapter::from_surreal_geometry(a)?;
    let gb = adapter::from_surreal_geometry(b)?;
    surrealgis_functions::relationships::st_within(&ga, &gb).map_err(|e| e.to_string())
}

#[surrealism]
fn st_touches(a: Geometry, b: Geometry) -> Result<bool, String> {
    let ga = adapter::from_surreal_geometry(a)?;
    let gb = adapter::from_surreal_geometry(b)?;
    surrealgis_functions::relationships::st_touches(&ga, &gb).map_err(|e| e.to_string())
}

#[surrealism]
fn st_crosses(a: Geometry, b: Geometry) -> Result<bool, String> {
    let ga = adapter::from_surreal_geometry(a)?;
    let gb = adapter::from_surreal_geometry(b)?;
    surrealgis_functions::relationships::st_crosses(&ga, &gb).map_err(|e| e.to_string())
}

#[surrealism]
fn st_overlaps(a: Geometry, b: Geometry) -> Result<bool, String> {
    let ga = adapter::from_surreal_geometry(a)?;
    let gb = adapter::from_surreal_geometry(b)?;
    surrealgis_functions::relationships::st_overlaps(&ga, &gb).map_err(|e| e.to_string())
}

#[surrealism]
fn st_disjoint(a: Geometry, b: Geometry) -> Result<bool, String> {
    let ga = adapter::from_surreal_geometry(a)?;
    let gb = adapter::from_surreal_geometry(b)?;
    surrealgis_functions::relationships::st_disjoint(&ga, &gb).map_err(|e| e.to_string())
}

#[surrealism]
fn st_equals(a: Geometry, b: Geometry) -> Result<bool, String> {
    let ga = adapter::from_surreal_geometry(a)?;
    let gb = adapter::from_surreal_geometry(b)?;
    surrealgis_functions::relationships::st_equals(&ga, &gb).map_err(|e| e.to_string())
}

#[surrealism]
fn st_covers(a: Geometry, b: Geometry) -> Result<bool, String> {
    let ga = adapter::from_surreal_geometry(a)?;
    let gb = adapter::from_surreal_geometry(b)?;
    surrealgis_functions::relationships::st_covers(&ga, &gb).map_err(|e| e.to_string())
}

#[surrealism]
fn st_covered_by(a: Geometry, b: Geometry) -> Result<bool, String> {
    let ga = adapter::from_surreal_geometry(a)?;
    let gb = adapter::from_surreal_geometry(b)?;
    surrealgis_functions::relationships::st_covered_by(&ga, &gb).map_err(|e| e.to_string())
}

#[surrealism]
fn st_relate(a: Geometry, b: Geometry) -> Result<String, String> {
    let ga = adapter::from_surreal_geometry(a)?;
    let gb = adapter::from_surreal_geometry(b)?;
    surrealgis_functions::relationships::st_relate(&ga, &gb).map_err(|e| e.to_string())
}
