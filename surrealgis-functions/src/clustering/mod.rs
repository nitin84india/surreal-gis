mod st_cluster_dbscan;
mod st_cluster_kmeans;
mod st_cluster_within;

pub use st_cluster_dbscan::st_cluster_dbscan;
pub use st_cluster_kmeans::st_cluster_kmeans;
pub use st_cluster_within::st_cluster_within;

use geo::Centroid;
use surrealgis_core::geometry::SurrealGeometry;
use surrealgis_core::srid::Srid;

use crate::FunctionError;

/// Extract centroid points from a slice of SurrealGeometry.
pub(crate) fn extract_centroids(
    geoms: &[SurrealGeometry],
) -> Result<Vec<geo_types::Point<f64>>, FunctionError> {
    geoms
        .iter()
        .map(|g| {
            let geo = g.to_geo()?;
            geo.centroid().ok_or_else(|| {
                FunctionError::InvalidArgument(
                    "Cannot compute centroid for geometry".into(),
                )
            })
        })
        .collect()
}

/// Build a GeometryCollection of MultiPoints from cluster assignments.
/// Each cluster becomes one MultiPoint in the collection.
/// `assignments[i] == None` means the point is noise (excluded).
pub(crate) fn build_cluster_result(
    geoms: &[SurrealGeometry],
    assignments: &[Option<usize>],
    srid: Srid,
) -> Result<SurrealGeometry, FunctionError> {
    use std::collections::HashMap;

    let mut clusters: HashMap<usize, Vec<geo_types::Point<f64>>> = HashMap::new();

    for (i, assignment) in assignments.iter().enumerate() {
        if let Some(cluster_id) = assignment {
            let centroid = extract_centroids(&geoms[i..=i])?;
            clusters.entry(*cluster_id).or_default().push(centroid[0]);
        }
    }

    if clusters.is_empty() {
        return Err(FunctionError::InvalidArgument(
            "No clusters formed".into(),
        ));
    }

    // Convert clusters to GeometryCollection of MultiPoints, sorted by cluster id
    let mut keys: Vec<usize> = clusters.keys().copied().collect();
    keys.sort();

    let gc_items: Vec<geo_types::Geometry<f64>> = keys
        .iter()
        .map(|key| {
            let points = &clusters[key];
            let mp = geo_types::MultiPoint(points.clone());
            geo_types::Geometry::MultiPoint(mp)
        })
        .collect();

    let gc = geo_types::GeometryCollection(gc_items);
    let result = geo_types::Geometry::GeometryCollection(gc);
    SurrealGeometry::from_geo(&result, srid).map_err(FunctionError::from)
}
