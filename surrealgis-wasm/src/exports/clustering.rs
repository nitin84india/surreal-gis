use surrealism::surrealism;
use surrealdb_types::Geometry;

use crate::adapter;

#[surrealism]
fn st_cluster_dbscan(geoms: Vec<Geometry>, eps: f64, min_points: i64) -> Result<Geometry, String> {
    let gs: Result<Vec<_>, String> = geoms
        .into_iter()
        .map(adapter::from_surreal_geometry)
        .collect();
    let result =
        surrealgis_functions::clustering::st_cluster_dbscan(&gs?, eps, min_points as usize)
            .map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}

#[surrealism]
fn st_cluster_kmeans(geoms: Vec<Geometry>, k: i64) -> Result<Geometry, String> {
    let gs: Result<Vec<_>, String> = geoms
        .into_iter()
        .map(adapter::from_surreal_geometry)
        .collect();
    let result = surrealgis_functions::clustering::st_cluster_kmeans(&gs?, k as usize)
        .map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}

#[surrealism]
fn st_cluster_within(geoms: Vec<Geometry>, distance: f64) -> Result<Geometry, String> {
    let gs: Result<Vec<_>, String> = geoms
        .into_iter()
        .map(adapter::from_surreal_geometry)
        .collect();
    let result = surrealgis_functions::clustering::st_cluster_within(&gs?, distance)
        .map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&result)
}
