use std::collections::HashMap;

use geo_types::{Coord, Geometry, LineString, MultiLineString};
use surrealgis_core::geometry::SurrealGeometry;

use crate::FunctionError;

/// Merge consecutive LineStrings within a MultiLineString that share endpoints
/// into longer LineStrings. Non-MultiLineString inputs return an error.
///
/// Algorithm:
/// 1. Build an adjacency map from endpoints to line indices
/// 2. Walk chains from degree-1 endpoints, collecting consecutive segments
/// 3. Return merged result as MultiLineString (or LineString if single result)
pub fn st_line_merge(geom: &SurrealGeometry) -> Result<SurrealGeometry, FunctionError> {
    let geo_geom = geom.to_geo()?;
    let lines = match geo_geom {
        Geometry::MultiLineString(mls) => mls.0,
        Geometry::LineString(ls) => {
            // Single LineString is already merged
            let result = Geometry::LineString(ls);
            return SurrealGeometry::from_geo(&result, *geom.srid())
                .map_err(FunctionError::from);
        }
        _ => {
            return Err(FunctionError::UnsupportedOperation(
                "st_line_merge requires MultiLineString or LineString input".to_string(),
            ))
        }
    };

    if lines.is_empty() {
        return Err(FunctionError::InvalidArgument(
            "st_line_merge: empty MultiLineString".to_string(),
        ));
    }

    let merged = merge_lines(lines);

    let result = if merged.len() == 1 {
        Geometry::LineString(merged.into_iter().next().unwrap())
    } else {
        Geometry::MultiLineString(MultiLineString(merged))
    };

    SurrealGeometry::from_geo(&result, *geom.srid()).map_err(FunctionError::from)
}

/// Canonical key for a coordinate: use ordered bit patterns for approximate comparison.
/// We use integer bit representation to avoid floating-point comparison issues.
fn coord_key(c: &Coord<f64>) -> (i64, i64) {
    (c.x.to_bits() as i64, c.y.to_bits() as i64)
}

fn merge_lines(lines: Vec<LineString<f64>>) -> Vec<LineString<f64>> {
    if lines.is_empty() {
        return vec![];
    }

    let n = lines.len();
    // Build adjacency: endpoint -> list of (line_index, is_start)
    let mut adjacency: HashMap<(i64, i64), Vec<(usize, bool)>> = HashMap::new();

    for (i, line) in lines.iter().enumerate() {
        if line.0.is_empty() {
            continue;
        }
        let start = coord_key(&line.0[0]);
        let end = coord_key(line.0.last().unwrap());
        adjacency.entry(start).or_default().push((i, true));
        adjacency.entry(end).or_default().push((i, false));
    }

    let mut used = vec![false; n];
    let mut result = Vec::new();

    for i in 0..n {
        if used[i] || lines[i].0.is_empty() {
            continue;
        }

        // Start a chain from this line
        used[i] = true;
        let mut chain: Vec<Coord<f64>> = lines[i].0.clone();

        // Extend forward (from chain's end)
        loop {
            let end_key = coord_key(chain.last().unwrap());
            let next = adjacency.get(&end_key).and_then(|entries| {
                entries.iter().find(|(idx, _)| !used[*idx]).copied()
            });
            match next {
                Some((idx, is_start)) => {
                    used[idx] = true;
                    if is_start {
                        // The next line's start matches our end: append in order (skip first)
                        chain.extend_from_slice(&lines[idx].0[1..]);
                    } else {
                        // The next line's end matches our end: append in reverse (skip first)
                        let mut rev_coords: Vec<Coord<f64>> = lines[idx].0.clone();
                        rev_coords.reverse();
                        chain.extend_from_slice(&rev_coords[1..]);
                    }
                }
                None => break,
            }
        }

        // Extend backward (from chain's start)
        loop {
            let start_key = coord_key(&chain[0]);
            let prev = adjacency.get(&start_key).and_then(|entries| {
                entries.iter().find(|(idx, _)| !used[*idx]).copied()
            });
            match prev {
                Some((idx, is_start)) => {
                    used[idx] = true;
                    if is_start {
                        // The prev line's start matches our start: prepend in reverse (skip last)
                        let mut rev_coords: Vec<Coord<f64>> = lines[idx].0.clone();
                        rev_coords.reverse();
                        rev_coords.pop(); // Remove the duplicate endpoint
                        rev_coords.extend(chain);
                        chain = rev_coords;
                    } else {
                        // The prev line's end matches our start: prepend in order (skip last)
                        let mut pre_coords: Vec<Coord<f64>> = lines[idx].0.clone();
                        pre_coords.pop(); // Remove the duplicate endpoint
                        pre_coords.extend(chain);
                        chain = pre_coords;
                    }
                }
                None => break,
            }
        }

        result.push(LineString(chain));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealgis_core::coordinate::Coordinate;
    use surrealgis_core::geometry::GeometryType;
    use surrealgis_core::srid::Srid;

    #[test]
    fn merge_connected_lines() {
        // Two lines sharing endpoint: (0,0)-(1,1) and (1,1)-(2,2)
        let lines = vec![
            vec![
                Coordinate::new(0.0, 0.0).unwrap(),
                Coordinate::new(1.0, 1.0).unwrap(),
            ],
            vec![
                Coordinate::new(1.0, 1.0).unwrap(),
                Coordinate::new(2.0, 2.0).unwrap(),
            ],
        ];
        let mls = SurrealGeometry::multi_line_string(lines, Srid::WGS84).unwrap();
        let result = st_line_merge(&mls).unwrap();
        // Should merge into single LineString
        assert_eq!(result.type_name(), "LineString");
        assert_eq!(result.num_points(), 3);
    }

    #[test]
    fn merge_disconnected_lines() {
        // Two lines that don't share endpoints
        let lines = vec![
            vec![
                Coordinate::new(0.0, 0.0).unwrap(),
                Coordinate::new(1.0, 1.0).unwrap(),
            ],
            vec![
                Coordinate::new(5.0, 5.0).unwrap(),
                Coordinate::new(6.0, 6.0).unwrap(),
            ],
        ];
        let mls = SurrealGeometry::multi_line_string(lines, Srid::WGS84).unwrap();
        let result = st_line_merge(&mls).unwrap();
        // Should remain as MultiLineString
        assert_eq!(result.type_name(), "MultiLineString");
        if let GeometryType::MultiLineString(ls) = result.geometry_type() {
            assert_eq!(ls.len(), 2);
        } else {
            panic!("Expected MultiLineString");
        }
    }

    #[test]
    fn merge_three_connected_lines() {
        // Three lines: (0,0)-(1,1), (1,1)-(2,2), (2,2)-(3,3)
        let lines = vec![
            vec![
                Coordinate::new(0.0, 0.0).unwrap(),
                Coordinate::new(1.0, 1.0).unwrap(),
            ],
            vec![
                Coordinate::new(1.0, 1.0).unwrap(),
                Coordinate::new(2.0, 2.0).unwrap(),
            ],
            vec![
                Coordinate::new(2.0, 2.0).unwrap(),
                Coordinate::new(3.0, 3.0).unwrap(),
            ],
        ];
        let mls = SurrealGeometry::multi_line_string(lines, Srid::WGS84).unwrap();
        let result = st_line_merge(&mls).unwrap();
        assert_eq!(result.type_name(), "LineString");
        assert_eq!(result.num_points(), 4);
    }

    #[test]
    fn merge_single_linestring_passthrough() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
        ];
        let line = SurrealGeometry::line_string(coords, Srid::WGS84).unwrap();
        let result = st_line_merge(&line).unwrap();
        assert_eq!(result.type_name(), "LineString");
        assert_eq!(result.num_points(), 2);
    }

    #[test]
    fn merge_point_rejected() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        let result = st_line_merge(&p);
        assert!(result.is_err());
    }

    #[test]
    fn merge_preserves_srid() {
        let lines = vec![
            vec![
                Coordinate::new(0.0, 0.0).unwrap(),
                Coordinate::new(1.0, 1.0).unwrap(),
            ],
            vec![
                Coordinate::new(1.0, 1.0).unwrap(),
                Coordinate::new(2.0, 2.0).unwrap(),
            ],
        ];
        let mls = SurrealGeometry::multi_line_string(lines, Srid::WEB_MERCATOR).unwrap();
        let result = st_line_merge(&mls).unwrap();
        assert_eq!(result.srid().code(), Srid::WEB_MERCATOR.code());
    }
}
