use surrealgis_core::geometry::SurrealGeometry;

use crate::FunctionError;

/// Distance-based clustering using Union-Find.
///
/// Groups geometries such that any two geometries within `distance` of each
/// other end up in the same cluster (transitive closure). Every point belongs
/// to exactly one cluster.
///
/// Returns a GeometryCollection of MultiPoints (one per cluster).
pub fn st_cluster_within(
    geoms: &[SurrealGeometry],
    distance: f64,
) -> Result<SurrealGeometry, FunctionError> {
    if geoms.is_empty() {
        return Err(FunctionError::InvalidArgument(
            "Empty geometry input".into(),
        ));
    }
    if distance < 0.0 {
        return Err(FunctionError::InvalidArgument(
            "distance must be non-negative".into(),
        ));
    }

    let centroids = super::extract_centroids(geoms)?;
    let points: Vec<[f64; 2]> = centroids.iter().map(|p| [p.x(), p.y()]).collect();
    let n = points.len();

    // Union-Find data structure
    let mut parent: Vec<usize> = (0..n).collect();
    let mut rank: Vec<usize> = vec![0; n];

    let dist_sq = distance * distance;

    // For each pair of points, union if within distance
    for i in 0..n {
        for j in (i + 1)..n {
            let dx = points[i][0] - points[j][0];
            let dy = points[i][1] - points[j][1];
            if dx * dx + dy * dy <= dist_sq {
                union(&mut parent, &mut rank, i, j);
            }
        }
    }

    // Build cluster assignments from Union-Find roots
    let mut cluster_map: std::collections::HashMap<usize, usize> =
        std::collections::HashMap::new();
    let mut next_id = 0;
    let assignments: Vec<Option<usize>> = (0..n)
        .map(|i| {
            let root = find(&mut parent, i);
            let id = *cluster_map.entry(root).or_insert_with(|| {
                let id = next_id;
                next_id += 1;
                id
            });
            Some(id)
        })
        .collect();

    let srid = *geoms[0].srid();
    super::build_cluster_result(geoms, &assignments, srid)
}

/// Find root with path compression.
fn find(parent: &mut [usize], i: usize) -> usize {
    if parent[i] != i {
        parent[i] = find(parent, parent[i]);
    }
    parent[i]
}

/// Union by rank.
fn union(parent: &mut [usize], rank: &mut [usize], a: usize, b: usize) {
    let ra = find(parent, a);
    let rb = find(parent, b);
    if ra != rb {
        if rank[ra] < rank[rb] {
            parent[ra] = rb;
        } else if rank[ra] > rank[rb] {
            parent[rb] = ra;
        } else {
            parent[rb] = ra;
            rank[ra] += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::srid::Srid;

    fn make_point(x: f64, y: f64) -> SurrealGeometry {
        SurrealGeometry::point(x, y, Srid::WEB_MERCATOR).unwrap()
    }

    #[test]
    fn nearby_points_grouped() {
        // All within distance 2 of each other
        let geoms = vec![
            make_point(0.0, 0.0),
            make_point(1.0, 0.0),
            make_point(0.0, 1.0),
        ];

        let result = st_cluster_within(&geoms, 2.0).unwrap();
        let geo = result.to_geo().unwrap();
        if let geo_types::Geometry::GeometryCollection(gc) = geo {
            assert_eq!(gc.0.len(), 1); // All in one cluster
            if let geo_types::Geometry::MultiPoint(mp) = &gc.0[0] {
                assert_eq!(mp.0.len(), 3);
            } else {
                panic!("Expected MultiPoint");
            }
        } else {
            panic!("Expected GeometryCollection");
        }
    }

    #[test]
    fn far_points_separate_clusters() {
        let geoms = vec![
            make_point(0.0, 0.0),
            make_point(1.0, 0.0),
            make_point(100.0, 100.0),
            make_point(101.0, 100.0),
        ];

        let result = st_cluster_within(&geoms, 2.0).unwrap();
        let geo = result.to_geo().unwrap();
        if let geo_types::Geometry::GeometryCollection(gc) = geo {
            assert_eq!(gc.0.len(), 2);
            let mut sizes: Vec<usize> = gc
                .0
                .iter()
                .map(|item| {
                    if let geo_types::Geometry::MultiPoint(mp) = item {
                        mp.0.len()
                    } else {
                        panic!("Expected MultiPoint");
                    }
                })
                .collect();
            sizes.sort();
            assert_eq!(sizes, vec![2, 2]);
        } else {
            panic!("Expected GeometryCollection");
        }
    }

    #[test]
    fn transitive_clustering() {
        // Chain: A-B within distance, B-C within distance, so A-B-C in same cluster
        // D far away
        let geoms = vec![
            make_point(0.0, 0.0),   // A
            make_point(1.5, 0.0),   // B (close to A)
            make_point(3.0, 0.0),   // C (close to B, far from A)
            make_point(100.0, 0.0), // D (far from all)
        ];

        let result = st_cluster_within(&geoms, 2.0).unwrap();
        let geo = result.to_geo().unwrap();
        if let geo_types::Geometry::GeometryCollection(gc) = geo {
            assert_eq!(gc.0.len(), 2);
            let mut sizes: Vec<usize> = gc
                .0
                .iter()
                .map(|item| {
                    if let geo_types::Geometry::MultiPoint(mp) = item {
                        mp.0.len()
                    } else {
                        panic!("Expected MultiPoint");
                    }
                })
                .collect();
            sizes.sort();
            assert_eq!(sizes, vec![1, 3]); // A-B-C together, D alone
        } else {
            panic!("Expected GeometryCollection");
        }
    }

    #[test]
    fn zero_distance_each_point_is_cluster() {
        let geoms = vec![
            make_point(0.0, 0.0),
            make_point(1.0, 0.0),
            make_point(2.0, 0.0),
        ];

        let result = st_cluster_within(&geoms, 0.0).unwrap();
        let geo = result.to_geo().unwrap();
        if let geo_types::Geometry::GeometryCollection(gc) = geo {
            assert_eq!(gc.0.len(), 3); // Each point in its own cluster
        } else {
            panic!("Expected GeometryCollection");
        }
    }

    #[test]
    fn single_point() {
        let geoms = vec![make_point(5.0, 5.0)];
        let result = st_cluster_within(&geoms, 1.0).unwrap();
        let geo = result.to_geo().unwrap();
        if let geo_types::Geometry::GeometryCollection(gc) = geo {
            assert_eq!(gc.0.len(), 1);
        } else {
            panic!("Expected GeometryCollection");
        }
    }

    #[test]
    fn empty_input_returns_error() {
        let result = st_cluster_within(&[], 1.0);
        assert!(result.is_err());
    }

    #[test]
    fn negative_distance_returns_error() {
        let geoms = vec![make_point(0.0, 0.0)];
        let result = st_cluster_within(&geoms, -1.0);
        assert!(result.is_err());
    }

    #[test]
    fn srid_preserved() {
        let geoms = vec![
            SurrealGeometry::point(0.0, 0.0, Srid::WEB_MERCATOR).unwrap(),
            SurrealGeometry::point(1.0, 0.0, Srid::WEB_MERCATOR).unwrap(),
        ];

        let result = st_cluster_within(&geoms, 2.0).unwrap();
        assert_eq!(*result.srid(), Srid::WEB_MERCATOR);
    }
}
