use surrealism::surrealism;
use surrealdb_types::Geometry;

use crate::adapter;

#[surrealism]
fn st_x(geom: Geometry) -> Result<f64, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    surrealgis_functions::accessors::st_x(&g).map_err(|e| e.to_string())
}

#[surrealism]
fn st_y(geom: Geometry) -> Result<f64, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    surrealgis_functions::accessors::st_y(&g).map_err(|e| e.to_string())
}

#[surrealism]
fn st_z(geom: Geometry) -> Result<Option<f64>, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    surrealgis_functions::accessors::st_z(&g).map_err(|e| e.to_string())
}

#[surrealism]
fn st_srid(geom: Geometry) -> Result<i64, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    Ok(surrealgis_functions::accessors::st_srid(&g) as i64)
}

#[surrealism]
fn st_geometry_type(geom: Geometry) -> Result<String, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    Ok(surrealgis_functions::accessors::st_geometry_type(&g).to_string())
}

#[surrealism]
fn st_num_points(geom: Geometry) -> Result<i64, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    Ok(surrealgis_functions::accessors::st_num_points(&g) as i64)
}

#[surrealism]
fn st_dimension(geom: Geometry) -> Result<i64, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    Ok(surrealgis_functions::accessors::st_dimension(&g) as i64)
}

#[surrealism]
fn st_start_point(geom: Geometry) -> Result<Geometry, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    let result = surrealgis_functions::accessors::st_start_point(&g)
        .map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}

#[surrealism]
fn st_end_point(geom: Geometry) -> Result<Geometry, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    let result = surrealgis_functions::accessors::st_end_point(&g)
        .map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}

#[surrealism]
fn st_is_empty(geom: Geometry) -> Result<bool, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    Ok(surrealgis_functions::accessors::st_is_empty(&g))
}

#[surrealism]
fn st_is_valid(geom: Geometry) -> Result<bool, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    surrealgis_functions::accessors::st_is_valid(&g).map_err(|e| e.to_string())
}

#[surrealism]
fn st_is_closed(geom: Geometry) -> Result<bool, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    surrealgis_functions::accessors::st_is_closed(&g).map_err(|e| e.to_string())
}

#[surrealism]
fn st_is_ring(geom: Geometry) -> Result<bool, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    surrealgis_functions::accessors::st_is_ring(&g).map_err(|e| e.to_string())
}

#[surrealism]
fn st_envelope(geom: Geometry) -> Result<Geometry, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    let result = surrealgis_functions::accessors::st_envelope(&g)
        .map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}

#[surrealism]
fn st_centroid(geom: Geometry) -> Result<Geometry, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    let result = surrealgis_functions::accessors::st_centroid(&g)
        .map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}

#[surrealism]
fn st_point_on_surface(geom: Geometry) -> Result<Geometry, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    let result = surrealgis_functions::accessors::st_point_on_surface(&g)
        .map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}

#[surrealism]
fn st_boundary(geom: Geometry) -> Result<Geometry, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    let result = surrealgis_functions::accessors::st_boundary(&g)
        .map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}
