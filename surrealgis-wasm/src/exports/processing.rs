use surrealism::surrealism;
use surrealdb_types::Geometry;

use crate::adapter;

#[surrealism]
fn st_buffer(geom: Geometry, distance: f64) -> Result<Geometry, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    let result = surrealgis_functions::processing::st_buffer(&g, distance)
        .map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}

#[surrealism]
fn st_convex_hull(geom: Geometry) -> Result<Geometry, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    let result = surrealgis_functions::processing::st_convex_hull(&g)
        .map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}

#[surrealism]
fn st_concave_hull(geom: Geometry, concavity: f64) -> Result<Geometry, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    let result = surrealgis_functions::processing::st_concave_hull(&g, concavity)
        .map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}

#[surrealism]
fn st_simplify(geom: Geometry, tolerance: f64) -> Result<Geometry, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    let result = surrealgis_functions::processing::st_simplify(&g, tolerance)
        .map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}

#[surrealism]
fn st_simplify_preserve_topology(geom: Geometry, tolerance: f64) -> Result<Geometry, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    let result =
        surrealgis_functions::processing::st_simplify_preserve_topology(&g, tolerance)
            .map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}

#[surrealism]
fn st_delaunay_triangles(geom: Geometry) -> Result<Geometry, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    let result = surrealgis_functions::processing::st_delaunay_triangles(&g)
        .map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}

#[surrealism]
fn st_voronoi_polygons(geom: Geometry) -> Result<Geometry, String> {
    let g = adapter::from_surreal_geometry(geom)?;
    let result = surrealgis_functions::processing::st_voronoi_polygons(&g)
        .map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}
