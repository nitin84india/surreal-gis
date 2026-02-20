use rand::Rng;
use surrealgis_core::geometry::SurrealGeometry;

use crate::FunctionError;

/// K-means++ clustering algorithm for geometries.
///
/// Groups geometries into exactly `k` clusters using Lloyd's algorithm
/// with k-means++ initialization. Each point is assigned to exactly one cluster.
///
/// Returns a GeometryCollection of MultiPoints (one per cluster).
pub fn st_cluster_kmeans(
    geoms: &[SurrealGeometry],
    k: usize,
) -> Result<SurrealGeometry, FunctionError> {
    if geoms.is_empty() {
        return Err(FunctionError::InvalidArgument(
            "Empty geometry input".into(),
        ));
    }
    if k == 0 {
        return Err(FunctionError::InvalidArgument(
            "k must be at least 1".into(),
        ));
    }

    let centroids = super::extract_centroids(geoms)?;
    let k = k.min(centroids.len()); // Can't have more clusters than points

    let points: Vec<[f64; 2]> = centroids.iter().map(|p| [p.x(), p.y()]).collect();

    // K-means++ initialization
    let mut rng = rand::thread_rng();
    let first = rng.gen_range(0..points.len());
    let mut centers: Vec<[f64; 2]> = vec![points[first]];

    for _ in 1..k {
        let distances: Vec<f64> = points
            .iter()
            .map(|p| {
                centers
                    .iter()
                    .map(|c| {
                        let dx = p[0] - c[0];
                        let dy = p[1] - c[1];
                        dx * dx + dy * dy
                    })
                    .fold(f64::MAX, f64::min)
            })
            .collect();

        let total: f64 = distances.iter().sum();
        if total <= 0.0 {
            break;
        }

        let threshold = rng.gen::<f64>() * total;
        let mut cumsum = 0.0;
        for (i, &d) in distances.iter().enumerate() {
            cumsum += d;
            if cumsum >= threshold {
                centers.push(points[i]);
                break;
            }
        }
    }

    // Lloyd's iteration (max 100 iterations)
    let mut assignments = vec![0usize; points.len()];
    for _ in 0..100 {
        let mut changed = false;

        // Assignment step
        for (i, p) in points.iter().enumerate() {
            let nearest = centers
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    let da = (p[0] - a[0]).powi(2) + (p[1] - a[1]).powi(2);
                    let db = (p[0] - b[0]).powi(2) + (p[1] - b[1]).powi(2);
                    da.partial_cmp(&db).unwrap()
                })
                .unwrap()
                .0;
            if assignments[i] != nearest {
                changed = true;
                assignments[i] = nearest;
            }
        }

        if !changed {
            break;
        }

        // Update step - recompute centers
        for (ci, center) in centers.iter_mut().enumerate() {
            let members: Vec<&[f64; 2]> = points
                .iter()
                .zip(&assignments)
                .filter(|(_, &a)| a == ci)
                .map(|(p, _)| p)
                .collect();
            if !members.is_empty() {
                let sx: f64 = members.iter().map(|p| p[0]).sum();
                let sy: f64 = members.iter().map(|p| p[1]).sum();
                let n = members.len() as f64;
                *center = [sx / n, sy / n];
            }
        }
    }

    let opt_assignments: Vec<Option<usize>> = assignments.into_iter().map(Some).collect();
    let srid = *geoms[0].srid();
    super::build_cluster_result(geoms, &opt_assignments, srid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::srid::Srid;

    fn make_point(x: f64, y: f64) -> SurrealGeometry {
        SurrealGeometry::point(x, y, Srid::WEB_MERCATOR).unwrap()
    }

    #[test]
    fn k_equals_two_clear_separation() {
        // Two well-separated groups
        let geoms = vec![
            make_point(0.0, 0.0),
            make_point(1.0, 0.0),
            make_point(0.0, 1.0),
            make_point(100.0, 100.0),
            make_point(101.0, 100.0),
            make_point(100.0, 101.0),
        ];

        let result = st_cluster_kmeans(&geoms, 2).unwrap();
        assert_eq!(result.type_name(), "GeometryCollection");

        let geo = result.to_geo().unwrap();
        if let geo_types::Geometry::GeometryCollection(gc) = geo {
            assert_eq!(gc.0.len(), 2);
            // Each cluster should have 3 points
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
            assert_eq!(sizes, vec![3, 3]);
        } else {
            panic!("Expected GeometryCollection");
        }
    }

    #[test]
    fn k_equals_one_returns_all() {
        let geoms = vec![
            make_point(0.0, 0.0),
            make_point(1.0, 0.0),
            make_point(100.0, 100.0),
        ];

        let result = st_cluster_kmeans(&geoms, 1).unwrap();
        let geo = result.to_geo().unwrap();
        if let geo_types::Geometry::GeometryCollection(gc) = geo {
            assert_eq!(gc.0.len(), 1);
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
    fn k_greater_than_points_clamped() {
        let geoms = vec![make_point(0.0, 0.0), make_point(1.0, 1.0)];

        let result = st_cluster_kmeans(&geoms, 10).unwrap();
        let geo = result.to_geo().unwrap();
        if let geo_types::Geometry::GeometryCollection(gc) = geo {
            // Should have at most 2 clusters (clamped to number of points)
            assert!(gc.0.len() <= 2);
        } else {
            panic!("Expected GeometryCollection");
        }
    }

    #[test]
    fn empty_input_returns_error() {
        let result = st_cluster_kmeans(&[], 2);
        assert!(result.is_err());
    }

    #[test]
    fn k_zero_returns_error() {
        let geoms = vec![make_point(0.0, 0.0)];
        let result = st_cluster_kmeans(&geoms, 0);
        assert!(result.is_err());
    }

    #[test]
    fn single_point_k_one() {
        let geoms = vec![make_point(5.0, 5.0)];
        let result = st_cluster_kmeans(&geoms, 1).unwrap();
        let geo = result.to_geo().unwrap();
        if let geo_types::Geometry::GeometryCollection(gc) = geo {
            assert_eq!(gc.0.len(), 1);
            if let geo_types::Geometry::MultiPoint(mp) = &gc.0[0] {
                assert_eq!(mp.0.len(), 1);
            } else {
                panic!("Expected MultiPoint");
            }
        } else {
            panic!("Expected GeometryCollection");
        }
    }

    #[test]
    fn srid_preserved() {
        let geoms = vec![
            SurrealGeometry::point(0.0, 0.0, Srid::WEB_MERCATOR).unwrap(),
            SurrealGeometry::point(1.0, 0.0, Srid::WEB_MERCATOR).unwrap(),
        ];

        let result = st_cluster_kmeans(&geoms, 1).unwrap();
        assert_eq!(*result.srid(), Srid::WEB_MERCATOR);
    }
}
