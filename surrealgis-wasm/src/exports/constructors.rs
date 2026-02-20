use surrealism::surrealism;
use surrealdb_types::Geometry;

use crate::adapter;

#[surrealism]
fn st_point(x: f64, y: f64) -> Result<Geometry, String> {
    let geom = surrealgis_functions::constructors::st_point(x, y, 4326)
        .map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&geom)
}

#[surrealism]
fn st_make_point(x: f64, y: f64) -> Result<Geometry, String> {
    let geom = surrealgis_functions::constructors::st_make_point(x, y, 4326)
        .map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&geom)
}

#[surrealism]
fn st_make_line(points: Vec<Geometry>) -> Result<Geometry, String> {
    let coords: Result<Vec<(f64, f64)>, String> = points
        .into_iter()
        .map(|g| {
            let pt = g
                .into_point()
                .map_err(|e| format!("st_make_line expects Point geometries: {e}"))?;
            Ok((pt.x(), pt.y()))
        })
        .collect();
    let geom = surrealgis_functions::constructors::st_make_line(&coords?, 4326)
        .map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&geom)
}

#[surrealism]
fn st_make_polygon(exterior: Geometry, holes: Vec<Geometry>) -> Result<Geometry, String> {
    let ext_line = exterior
        .into_line()
        .map_err(|e| format!("st_make_polygon exterior must be a LineString: {e}"))?;
    let ext_coords: Vec<(f64, f64)> = ext_line.into_points().into_iter().map(|p| (p.x(), p.y())).collect();

    let hole_rings: Result<Vec<Vec<(f64, f64)>>, String> = holes
        .into_iter()
        .map(|g| {
            let line = g
                .into_line()
                .map_err(|e| format!("st_make_polygon holes must be LineStrings: {e}"))?;
            Ok(line.into_points().into_iter().map(|p| (p.x(), p.y())).collect())
        })
        .collect();

    let geom =
        surrealgis_functions::constructors::st_make_polygon(&ext_coords, &hole_rings?, 4326)
            .map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&geom)
}

#[surrealism]
fn st_make_envelope(xmin: f64, ymin: f64, xmax: f64, ymax: f64) -> Result<Geometry, String> {
    let geom =
        surrealgis_functions::constructors::st_make_envelope(xmin, ymin, xmax, ymax, 4326)
            .map_err(|e| e.to_string())?;
    adapter::to_surreal_geometry(&geom)
}
