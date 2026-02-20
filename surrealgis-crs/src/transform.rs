use surrealgis_core::coordinate::Coordinate;
use surrealgis_core::geometry::{GeometryType, PolygonData, SurrealGeometry};
use surrealgis_core::srid::Srid;

use crate::error::CrsError;
use crate::projection::Projection;

/// Transforms a geometry from one coordinate reference system to another.
///
/// This is the primary reprojection entry point. It handles:
/// 1. Looking up proj4 definitions for both SRIDs
/// 2. Converting geographic coordinates from degrees to radians before transform
/// 3. Invoking proj4rs for the actual coordinate transformation
/// 4. Converting geographic output from radians back to degrees
/// 5. Constructing a new geometry with the transformed coordinates and target SRID
pub fn transform_geometry(
    geom: &SurrealGeometry,
    from_srid: i32,
    to_srid: i32,
) -> Result<SurrealGeometry, CrsError> {
    if from_srid == to_srid {
        return Err(CrsError::SameSrid(from_srid));
    }

    let src_proj = Projection::new(from_srid)?;
    let dst_proj = Projection::new(to_srid)?;

    let target_srid = Srid::new(to_srid)
        .map_err(|e| CrsError::ProjectionError(e.to_string()))?;

    let transformed_type = transform_geometry_type(
        geom.geometry_type(),
        &src_proj,
        &dst_proj,
    )?;

    rebuild_geometry(transformed_type, target_srid)
}

/// Changes the SRID metadata of a geometry without reprojecting coordinates.
///
/// This is useful when you know coordinates are already in the target CRS
/// but the SRID metadata is wrong or missing.
pub fn set_srid(
    geom: &SurrealGeometry,
    new_srid: i32,
) -> Result<SurrealGeometry, CrsError> {
    let srid = Srid::new(new_srid)
        .map_err(|e| CrsError::ProjectionError(e.to_string()))?;

    let cloned_type = clone_geometry_type(geom.geometry_type());
    rebuild_geometry(cloned_type, srid)
}

// ── Coordinate conversion helpers ────────────────────────────────────────

fn degrees_to_radians(deg: f64) -> f64 {
    deg * std::f64::consts::PI / 180.0
}

fn radians_to_degrees(rad: f64) -> f64 {
    rad * 180.0 / std::f64::consts::PI
}

// ── Single-coordinate transform ──────────────────────────────────────────

fn transform_coordinate(
    coord: &Coordinate,
    src_proj: &Projection,
    dst_proj: &Projection,
) -> Result<Coordinate, CrsError> {
    let mut x = coord.x();
    let mut y = coord.y();
    let z = coord.z().unwrap_or(0.0);

    // proj4rs expects radians for geographic CRS
    if src_proj.is_geographic() {
        x = degrees_to_radians(x);
        y = degrees_to_radians(y);
    }

    // Use proj4rs Transform trait on a mutable point
    let mut point = TransformPoint { x, y, z };
    proj4rs::transform::transform(src_proj.proj(), dst_proj.proj(), &mut point)
        .map_err(|e| CrsError::ProjectionError(e.to_string()))?;

    // proj4rs outputs radians for geographic CRS
    if dst_proj.is_geographic() {
        point.x = radians_to_degrees(point.x);
        point.y = radians_to_degrees(point.y);
    }

    if coord.z().is_some() {
        Coordinate::new_3d(point.x, point.y, point.z)
            .map_err(|e| CrsError::InvalidCoordinate(e.to_string()))
    } else {
        Coordinate::new(point.x, point.y)
            .map_err(|e| CrsError::InvalidCoordinate(e.to_string()))
    }
}

/// Internal point type implementing proj4rs::transform::Transform.
struct TransformPoint {
    x: f64,
    y: f64,
    z: f64,
}

impl proj4rs::transform::Transform for TransformPoint {
    fn transform_coordinates<F: proj4rs::transform::TransformClosure>(
        &mut self,
        f: &mut F,
    ) -> proj4rs::errors::Result<()> {
        f(self.x, self.y, self.z).map(|(x, y, z)| {
            self.x = x;
            self.y = y;
            self.z = z;
        })
    }
}

// ── Batch coordinate transforms ──────────────────────────────────────────

fn transform_coords(
    coords: &[Coordinate],
    src: &Projection,
    dst: &Projection,
) -> Result<Vec<Coordinate>, CrsError> {
    coords
        .iter()
        .map(|c| transform_coordinate(c, src, dst))
        .collect()
}

fn transform_rings(
    rings: &[Vec<Coordinate>],
    src: &Projection,
    dst: &Projection,
) -> Result<Vec<Vec<Coordinate>>, CrsError> {
    rings
        .iter()
        .map(|ring| transform_coords(ring, src, dst))
        .collect()
}

// ── Geometry type transform (recursive for collections) ──────────────────

fn transform_geometry_type(
    gt: &GeometryType,
    src: &Projection,
    dst: &Projection,
) -> Result<GeometryType, CrsError> {
    match gt {
        GeometryType::Point(coord) => {
            let new_coord = transform_coordinate(coord, src, dst)?;
            Ok(GeometryType::Point(new_coord))
        }
        GeometryType::LineString(coords) => {
            let new_coords = transform_coords(coords, src, dst)?;
            Ok(GeometryType::LineString(new_coords))
        }
        GeometryType::Polygon { exterior, holes } => {
            let new_exterior = transform_coords(exterior, src, dst)?;
            let new_holes = transform_rings(holes, src, dst)?;
            Ok(GeometryType::Polygon {
                exterior: new_exterior,
                holes: new_holes,
            })
        }
        GeometryType::MultiPoint(coords) => {
            let new_coords = transform_coords(coords, src, dst)?;
            Ok(GeometryType::MultiPoint(new_coords))
        }
        GeometryType::MultiLineString(lines) => {
            let new_lines = transform_rings(lines, src, dst)?;
            Ok(GeometryType::MultiLineString(new_lines))
        }
        GeometryType::MultiPolygon(polygons) => {
            let new_polygons = polygons
                .iter()
                .map(|p| {
                    let exterior = transform_coords(&p.exterior, src, dst)?;
                    let holes = transform_rings(&p.holes, src, dst)?;
                    Ok(PolygonData { exterior, holes })
                })
                .collect::<Result<Vec<_>, CrsError>>()?;
            Ok(GeometryType::MultiPolygon(new_polygons))
        }
        GeometryType::GeometryCollection(geoms) => {
            let new_geoms = geoms
                .iter()
                .map(|g| {
                    let new_type = transform_geometry_type(g.geometry_type(), src, dst)?;
                    let target_srid = Srid::new(dst.srid())
                        .map_err(|e| CrsError::ProjectionError(e.to_string()))?;
                    rebuild_geometry(new_type, target_srid)
                })
                .collect::<Result<Vec<_>, CrsError>>()?;
            Ok(GeometryType::GeometryCollection(new_geoms))
        }
    }
}

// ── Deep clone of geometry type ──────────────────────────────────────────

fn clone_geometry_type(gt: &GeometryType) -> GeometryType {
    gt.clone()
}

// ── Rebuild a SurrealGeometry from a GeometryType and Srid ───────────────

fn rebuild_geometry(
    gt: GeometryType,
    srid: Srid,
) -> Result<SurrealGeometry, CrsError> {
    match gt {
        GeometryType::Point(coord) => {
            SurrealGeometry::point(coord.x(), coord.y(), srid)
                .map_err(CrsError::from)
        }
        GeometryType::LineString(coords) => {
            SurrealGeometry::line_string(coords, srid)
                .map_err(CrsError::from)
        }
        GeometryType::Polygon { exterior, holes } => {
            SurrealGeometry::polygon(exterior, holes, srid)
                .map_err(CrsError::from)
        }
        GeometryType::MultiPoint(coords) => {
            SurrealGeometry::multi_point(coords, srid)
                .map_err(CrsError::from)
        }
        GeometryType::MultiLineString(lines) => {
            SurrealGeometry::multi_line_string(lines, srid)
                .map_err(CrsError::from)
        }
        GeometryType::MultiPolygon(polygons) => {
            SurrealGeometry::multi_polygon(polygons, srid)
                .map_err(CrsError::from)
        }
        GeometryType::GeometryCollection(geoms) => {
            SurrealGeometry::geometry_collection(geoms, srid)
                .map_err(CrsError::from)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    // ── Helper to extract point coordinate ───────────────────────────────

    fn point_coords(geom: &SurrealGeometry) -> (f64, f64) {
        match geom.geometry_type() {
            GeometryType::Point(c) => (c.x(), c.y()),
            _ => panic!("Expected Point geometry"),
        }
    }

    // ── WGS 84 -> Web Mercator (NYC) ────────────────────────────────────

    #[test]
    fn transform_point_4326_to_3857_nyc() {
        let nyc = SurrealGeometry::point(-73.9857, 40.7484, Srid::WGS84).unwrap();
        let result = transform_geometry(&nyc, 4326, 3857).unwrap();

        let (x, y) = point_coords(&result);
        assert_eq!(result.srid().code(), 3857);

        // Expected Web Mercator coordinates for NYC
        // x should be roughly -8,235,851 meters
        // y should be roughly 4,975,293 meters
        assert_abs_diff_eq!(x, -8_235_851.0, epsilon = 500.0);
        assert_abs_diff_eq!(y, 4_975_293.0, epsilon = 500.0);
    }

    // ── Round-trip 4326 -> 3857 -> 4326 ─────────────────────────────────

    #[test]
    fn round_trip_4326_3857_4326() {
        let original = SurrealGeometry::point(-73.9857, 40.7484, Srid::WGS84).unwrap();

        let mercator = transform_geometry(&original, 4326, 3857).unwrap();
        let back = transform_geometry(&mercator, 3857, 4326).unwrap();

        let (x, y) = point_coords(&back);
        assert_abs_diff_eq!(x, -73.9857, epsilon = 1e-6);
        assert_abs_diff_eq!(y, 40.7484, epsilon = 1e-6);
    }

    // ── Paris coordinates: 4326 -> 2154 (Lambert-93) ────────────────────

    #[test]
    fn transform_point_4326_to_2154_paris() {
        let paris = SurrealGeometry::point(2.3522, 48.8566, Srid::WGS84).unwrap();
        let result = transform_geometry(&paris, 4326, 2154).unwrap();

        let (x, y) = point_coords(&result);
        assert_eq!(result.srid().code(), 2154);

        // Lambert-93 coordinates for Paris center should be approximately:
        // x ~ 652,000 m, y ~ 6,862,000 m
        assert_abs_diff_eq!(x, 652_000.0, epsilon = 2000.0);
        assert_abs_diff_eq!(y, 6_862_000.0, epsilon = 2000.0);
    }

    // ── UTM zone transform ──────────────────────────────────────────────

    #[test]
    fn transform_point_4326_to_utm_zone_18n() {
        // NYC is in UTM zone 18N
        let nyc = SurrealGeometry::point(-73.9857, 40.7484, Srid::WGS84).unwrap();
        let result = transform_geometry(&nyc, 4326, 32618).unwrap();

        let (x, y) = point_coords(&result);
        assert_eq!(result.srid().code(), 32618);

        // UTM Zone 18N coordinates for NYC should be approximately:
        // x ~ 585,628 m (easting), y ~ 4,512,000 m (northing)
        assert_abs_diff_eq!(x, 585_628.0, epsilon = 1000.0);
        assert_abs_diff_eq!(y, 4_512_000.0, epsilon = 2000.0);
    }

    // ── set_srid only changes metadata ──────────────────────────────────

    #[test]
    fn set_srid_changes_only_metadata() {
        let point = SurrealGeometry::point(-73.9857, 40.7484, Srid::WGS84).unwrap();
        let result = set_srid(&point, 3857).unwrap();

        // SRID changed
        assert_eq!(result.srid().code(), 3857);

        // Coordinates unchanged
        let (x, y) = point_coords(&result);
        assert_abs_diff_eq!(x, -73.9857, epsilon = 1e-10);
        assert_abs_diff_eq!(y, 40.7484, epsilon = 1e-10);
    }

    // ── Error cases ─────────────────────────────────────────────────────

    #[test]
    fn same_srid_returns_error() {
        let point = SurrealGeometry::point(0.0, 0.0, Srid::WGS84).unwrap();
        let result = transform_geometry(&point, 4326, 4326);
        assert!(result.is_err());
        match result.unwrap_err() {
            CrsError::SameSrid(code) => assert_eq!(code, 4326),
            other => panic!("Expected SameSrid, got: {:?}", other),
        }
    }

    #[test]
    fn unknown_source_srid_returns_error() {
        let point = SurrealGeometry::point(0.0, 0.0, Srid::WGS84).unwrap();
        let result = transform_geometry(&point, 99999, 4326);
        assert!(result.is_err());
        match result.unwrap_err() {
            CrsError::UnknownSrid(code) => assert_eq!(code, 99999),
            other => panic!("Expected UnknownSrid, got: {:?}", other),
        }
    }

    #[test]
    fn unknown_target_srid_returns_error() {
        let point = SurrealGeometry::point(0.0, 0.0, Srid::WGS84).unwrap();
        let result = transform_geometry(&point, 4326, 99999);
        assert!(result.is_err());
        match result.unwrap_err() {
            CrsError::UnknownSrid(code) => assert_eq!(code, 99999),
            other => panic!("Expected UnknownSrid, got: {:?}", other),
        }
    }

    // ── LineString transform ────────────────────────────────────────────

    #[test]
    fn transform_linestring_4326_to_3857() {
        let coords = vec![
            Coordinate::new(-73.9857, 40.7484).unwrap(),
            Coordinate::new(-74.0060, 40.7128).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WGS84).unwrap();
        let result = transform_geometry(&ls, 4326, 3857).unwrap();

        assert_eq!(result.srid().code(), 3857);
        assert_eq!(result.type_name(), "LineString");
        assert_eq!(result.num_points(), 2);
    }

    // ── Polygon transform ───────────────────────────────────────────────

    #[test]
    fn transform_polygon_4326_to_3857() {
        let exterior = vec![
            Coordinate::new(-74.0, 40.7).unwrap(),
            Coordinate::new(-73.9, 40.7).unwrap(),
            Coordinate::new(-73.9, 40.8).unwrap(),
            Coordinate::new(-74.0, 40.7).unwrap(),
        ];
        let poly = SurrealGeometry::polygon(exterior, vec![], Srid::WGS84).unwrap();
        let result = transform_geometry(&poly, 4326, 3857).unwrap();

        assert_eq!(result.srid().code(), 3857);
        assert_eq!(result.type_name(), "Polygon");
        assert_eq!(result.num_points(), 4);
    }

    // ── MultiPoint transform ────────────────────────────────────────────

    #[test]
    fn transform_multipoint_4326_to_3857() {
        let coords = vec![
            Coordinate::new(-73.9857, 40.7484).unwrap(),
            Coordinate::new(2.3522, 48.8566).unwrap(),
        ];
        let mp = SurrealGeometry::multi_point(coords, Srid::WGS84).unwrap();
        let result = transform_geometry(&mp, 4326, 3857).unwrap();

        assert_eq!(result.srid().code(), 3857);
        assert_eq!(result.type_name(), "MultiPoint");
        assert_eq!(result.num_points(), 2);
    }

    // ── GeometryCollection transform ────────────────────────────────────

    #[test]
    fn transform_geometry_collection() {
        let p = SurrealGeometry::point(-73.9857, 40.7484, Srid::WGS84).unwrap();
        let coords = vec![
            Coordinate::new(-74.0, 40.7).unwrap(),
            Coordinate::new(-73.9, 40.8).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WGS84).unwrap();
        let gc = SurrealGeometry::geometry_collection(vec![p, ls], Srid::WGS84).unwrap();

        let result = transform_geometry(&gc, 4326, 3857).unwrap();
        assert_eq!(result.srid().code(), 3857);
        assert_eq!(result.type_name(), "GeometryCollection");
    }

    // ── Degree/radian conversion helpers ────────────────────────────────

    #[test]
    fn degrees_radians_roundtrip() {
        let deg = 45.0;
        let rad = degrees_to_radians(deg);
        let back = radians_to_degrees(rad);
        assert_abs_diff_eq!(back, deg, epsilon = 1e-12);
    }

    #[test]
    fn degrees_to_radians_known_values() {
        assert_abs_diff_eq!(degrees_to_radians(0.0), 0.0, epsilon = 1e-12);
        assert_abs_diff_eq!(degrees_to_radians(90.0), std::f64::consts::FRAC_PI_2, epsilon = 1e-12);
        assert_abs_diff_eq!(degrees_to_radians(180.0), std::f64::consts::PI, epsilon = 1e-12);
        assert_abs_diff_eq!(degrees_to_radians(360.0), 2.0 * std::f64::consts::PI, epsilon = 1e-12);
    }

    // ── Round-trip for non-WGS84 geographic CRS ─────────────────────────

    #[test]
    fn round_trip_4269_3857_4269() {
        // NAD83 -> Web Mercator -> NAD83
        let original = SurrealGeometry::point(-73.9857, 40.7484, Srid::NAD83).unwrap();

        let mercator = transform_geometry(&original, 4269, 3857).unwrap();
        let back = transform_geometry(&mercator, 3857, 4269).unwrap();

        let (x, y) = point_coords(&back);
        assert_abs_diff_eq!(x, -73.9857, epsilon = 1e-4);
        assert_abs_diff_eq!(y, 40.7484, epsilon = 1e-4);
    }

    // ── LAEA Europe transform ───────────────────────────────────────────

    #[test]
    fn transform_point_4326_to_3035_berlin() {
        let berlin = SurrealGeometry::point(13.405, 52.52, Srid::WGS84).unwrap();
        let result = transform_geometry(&berlin, 4326, 3035).unwrap();

        let (x, y) = point_coords(&result);
        assert_eq!(result.srid().code(), 3035);

        // LAEA Europe coordinates for Berlin should be approximately:
        // x ~ 4,552,036 m, y ~ 3,280,000 m
        assert_abs_diff_eq!(x, 4_552_036.0, epsilon = 1000.0);
        assert_abs_diff_eq!(y, 3_280_000.0, epsilon = 30000.0);
    }

    // ── set_srid with invalid SRID ──────────────────────────────────────

    #[test]
    fn set_srid_invalid_code() {
        let point = SurrealGeometry::point(0.0, 0.0, Srid::WGS84).unwrap();
        let result = set_srid(&point, 0);
        assert!(result.is_err());
    }

    #[test]
    fn set_srid_preserves_geometry_type() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WGS84).unwrap();
        let result = set_srid(&ls, 3857).unwrap();

        assert_eq!(result.type_name(), "LineString");
        assert_eq!(result.num_points(), 2);
        assert_eq!(result.srid().code(), 3857);
    }
}
