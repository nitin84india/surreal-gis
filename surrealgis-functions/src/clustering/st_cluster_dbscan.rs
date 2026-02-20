use surrealgis_core::geometry::SurrealGeometry;

use crate::FunctionError;

/// DBSCAN clustering algorithm for geometries.
///
/// Groups geometries into clusters based on density. Points that are within
/// `eps` distance of at least `min_points` other points form dense regions
/// (clusters). Points not reachable from any dense region are noise.
///
/// Returns a GeometryCollection of MultiPoints (one per cluster).
/// Noise points are excluded from the result.
pub fn st_cluster_dbscan(
    geoms: &[SurrealGeometry],
    eps: f64,
    min_points: usize,
) -> Result<SurrealGeometry, FunctionError> {
    if geoms.is_empty() {
        return Err(FunctionError::InvalidArgument(
            "Empty geometry input".into(),
        ));
    }
    if eps < 0.0 {
        return Err(FunctionError::InvalidArgument(
            "eps must be non-negative".into(),
        ));
    }
    if min_points == 0 {
        return Err(FunctionError::InvalidArgument(
            "min_points must be at least 1".into(),
        ));
    }

    let centroids = super::extract_centroids(geoms)?;
    let points: Vec<[f64; 2]> = centroids.iter().map(|p| [p.x(), p.y()]).collect();
    let n = points.len();

    let mut assignments: Vec<Option<usize>> = vec![None; n];
    let mut visited = vec![false; n];
    let mut cluster_id = 0;

    let eps_squared = eps * eps;

    for i in 0..n {
        if visited[i] {
            continue;
        }
        visited[i] = true;

        let neighbors = region_query(&points, i, eps_squared);

        if neighbors.len() < min_points {
            // Noise point - leave assignment as None
            continue;
        }

        // Start a new cluster
        assignments[i] = Some(cluster_id);
        let mut queue = neighbors;
        let mut qi = 0;

        while qi < queue.len() {
            let j = queue[qi];
            qi += 1;

            if !visited[j] {
                visited[j] = true;
                let j_neighbors = region_query(&points, j, eps_squared);
                if j_neighbors.len() >= min_points {
                    // Expand the cluster
                    for &nb in &j_neighbors {
                        if !queue.contains(&nb) {
                            queue.push(nb);
                        }
                    }
                }
            }

            if assignments[j].is_none() {
                assignments[j] = Some(cluster_id);
            }
        }

        cluster_id += 1;
    }

    let srid = *geoms[0].srid();
    super::build_cluster_result(geoms, &assignments, srid)
}

/// Find all points within squared distance of the given point.
fn region_query(points: &[[f64; 2]], idx: usize, eps_squared: f64) -> Vec<usize> {
    let p = &points[idx];
    points
        .iter()
        .enumerate()
        .filter(|(_, q)| {
            let dx = p[0] - q[0];
            let dy = p[1] - q[1];
            dx * dx + dy * dy <= eps_squared
        })
        .map(|(i, _)| i)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::srid::Srid;

    fn make_point(x: f64, y: f64) -> SurrealGeometry {
        SurrealGeometry::point(x, y, Srid::WEB_MERCATOR).unwrap()
    }

    #[test]
    fn two_clear_clusters() {
        // Cluster A: (0,0), (1,0), (0,1)
        // Cluster B: (10,10), (11,10), (10,11)
        let geoms = vec![
            make_point(0.0, 0.0),
            make_point(1.0, 0.0),
            make_point(0.0, 1.0),
            make_point(10.0, 10.0),
            make_point(11.0, 10.0),
            make_point(10.0, 11.0),
        ];

        let result = st_cluster_dbscan(&geoms, 2.0, 2).unwrap();
        assert_eq!(result.type_name(), "GeometryCollection");

        // Should have 2 clusters
        let geo = result.to_geo().unwrap();
        if let geo_types::Geometry::GeometryCollection(gc) = geo {
            assert_eq!(gc.0.len(), 2);
            // Each cluster should be a MultiPoint with 3 points
            for item in &gc.0 {
                if let geo_types::Geometry::MultiPoint(mp) = item {
                    assert_eq!(mp.0.len(), 3);
                } else {
                    panic!("Expected MultiPoint in cluster result");
                }
            }
        } else {
            panic!("Expected GeometryCollection");
        }
    }

    #[test]
    fn noise_points_excluded() {
        // Two close points and one far away (noise)
        let geoms = vec![
            make_point(0.0, 0.0),
            make_point(1.0, 0.0),
            make_point(100.0, 100.0), // noise
        ];

        let result = st_cluster_dbscan(&geoms, 2.0, 2).unwrap();
        let geo = result.to_geo().unwrap();
        if let geo_types::Geometry::GeometryCollection(gc) = geo {
            assert_eq!(gc.0.len(), 1); // Only 1 cluster, noise excluded
            if let geo_types::Geometry::MultiPoint(mp) = &gc.0[0] {
                assert_eq!(mp.0.len(), 2);
            } else {
                panic!("Expected MultiPoint");
            }
        } else {
            panic!("Expected GeometryCollection");
        }
    }

    #[test]
    fn single_cluster_all_close() {
        let geoms = vec![
            make_point(0.0, 0.0),
            make_point(1.0, 0.0),
            make_point(0.0, 1.0),
            make_point(1.0, 1.0),
        ];

        let result = st_cluster_dbscan(&geoms, 2.0, 2).unwrap();
        let geo = result.to_geo().unwrap();
        if let geo_types::Geometry::GeometryCollection(gc) = geo {
            assert_eq!(gc.0.len(), 1);
            if let geo_types::Geometry::MultiPoint(mp) = &gc.0[0] {
                assert_eq!(mp.0.len(), 4);
            } else {
                panic!("Expected MultiPoint");
            }
        } else {
            panic!("Expected GeometryCollection");
        }
    }

    #[test]
    fn all_noise_returns_error() {
        // All points far apart with high min_points
        let geoms = vec![
            make_point(0.0, 0.0),
            make_point(100.0, 100.0),
            make_point(200.0, 200.0),
        ];

        let result = st_cluster_dbscan(&geoms, 1.0, 3);
        assert!(result.is_err());
    }

    #[test]
    fn empty_input_returns_error() {
        let result = st_cluster_dbscan(&[], 1.0, 2);
        assert!(result.is_err());
    }

    #[test]
    fn negative_eps_returns_error() {
        let geoms = vec![make_point(0.0, 0.0)];
        let result = st_cluster_dbscan(&geoms, -1.0, 2);
        assert!(result.is_err());
    }

    #[test]
    fn zero_min_points_returns_error() {
        let geoms = vec![make_point(0.0, 0.0)];
        let result = st_cluster_dbscan(&geoms, 1.0, 0);
        assert!(result.is_err());
    }

    #[test]
    fn srid_preserved() {
        let geoms = vec![
            SurrealGeometry::point(0.0, 0.0, Srid::WEB_MERCATOR).unwrap(),
            SurrealGeometry::point(1.0, 0.0, Srid::WEB_MERCATOR).unwrap(),
        ];

        let result = st_cluster_dbscan(&geoms, 2.0, 1).unwrap();
        assert_eq!(*result.srid(), Srid::WEB_MERCATOR);
    }
}
