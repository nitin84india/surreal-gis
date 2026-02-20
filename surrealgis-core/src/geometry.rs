use crate::bbox::BoundingBox;
use crate::coordinate::Coordinate;
use crate::error::GeometryError;
use crate::flags::GeometryFlags;
use crate::srid::Srid;
use crate::validation;

/// Data for a single polygon (exterior ring + holes).
#[derive(Debug, Clone, PartialEq)]
pub struct PolygonData {
    pub exterior: Vec<Coordinate>,
    pub holes: Vec<Vec<Coordinate>>,
}

/// The specific geometry variant.
#[derive(Debug, Clone, PartialEq)]
pub enum GeometryType {
    Point(Coordinate),
    LineString(Vec<Coordinate>),
    Polygon {
        exterior: Vec<Coordinate>,
        holes: Vec<Vec<Coordinate>>,
    },
    MultiPoint(Vec<Coordinate>),
    MultiLineString(Vec<Vec<Coordinate>>),
    MultiPolygon(Vec<PolygonData>),
    GeometryCollection(Vec<SurrealGeometry>),
}

/// The aggregate root for all geometry operations.
#[derive(Debug, Clone, PartialEq)]
pub struct SurrealGeometry {
    geometry_type: GeometryType,
    srid: Srid,
    bbox: Option<BoundingBox>,
    flags: GeometryFlags,
}

impl SurrealGeometry {
    // ── Smart Constructors ──────────────────────────────────────────

    /// Create a Point geometry.
    pub fn point(x: f64, y: f64, srid: Srid) -> Result<Self, GeometryError> {
        let coord = Coordinate::new(x, y)?;
        let bbox = BoundingBox::from_coordinates(&[coord.clone()]);
        let mut flags = GeometryFlags::HAS_SRID;
        if bbox.is_some() {
            flags |= GeometryFlags::HAS_BBOX;
        }
        Ok(Self {
            geometry_type: GeometryType::Point(coord),
            srid,
            bbox,
            flags,
        })
    }

    /// Create a LineString geometry.
    pub fn line_string(
        coords: Vec<Coordinate>,
        srid: Srid,
    ) -> Result<Self, GeometryError> {
        validation::validate_linestring(&coords)?;
        let bbox = BoundingBox::from_coordinates(&coords);
        let mut flags = GeometryFlags::HAS_SRID;
        if bbox.is_some() {
            flags |= GeometryFlags::HAS_BBOX;
        }
        Ok(Self {
            geometry_type: GeometryType::LineString(coords),
            srid,
            bbox,
            flags,
        })
    }

    /// Create a Polygon geometry.
    pub fn polygon(
        exterior: Vec<Coordinate>,
        holes: Vec<Vec<Coordinate>>,
        srid: Srid,
    ) -> Result<Self, GeometryError> {
        validation::validate_polygon(&exterior, &holes)?;
        let bbox = BoundingBox::from_coordinates(&exterior);
        let mut flags = GeometryFlags::HAS_SRID;
        if bbox.is_some() {
            flags |= GeometryFlags::HAS_BBOX;
        }
        Ok(Self {
            geometry_type: GeometryType::Polygon { exterior, holes },
            srid,
            bbox,
            flags,
        })
    }

    /// Create a MultiPoint geometry.
    pub fn multi_point(
        coords: Vec<Coordinate>,
        srid: Srid,
    ) -> Result<Self, GeometryError> {
        if coords.is_empty() {
            return Err(GeometryError::EmptyGeometry);
        }
        let bbox = BoundingBox::from_coordinates(&coords);
        let mut flags = GeometryFlags::HAS_SRID;
        if bbox.is_some() {
            flags |= GeometryFlags::HAS_BBOX;
        }
        Ok(Self {
            geometry_type: GeometryType::MultiPoint(coords),
            srid,
            bbox,
            flags,
        })
    }

    /// Create a MultiLineString geometry.
    pub fn multi_line_string(
        lines: Vec<Vec<Coordinate>>,
        srid: Srid,
    ) -> Result<Self, GeometryError> {
        if lines.is_empty() {
            return Err(GeometryError::EmptyGeometry);
        }
        for line in &lines {
            validation::validate_linestring(line)?;
        }
        let all_coords: Vec<Coordinate> = lines.iter().flatten().cloned().collect();
        let bbox = BoundingBox::from_coordinates(&all_coords);
        let mut flags = GeometryFlags::HAS_SRID;
        if bbox.is_some() {
            flags |= GeometryFlags::HAS_BBOX;
        }
        Ok(Self {
            geometry_type: GeometryType::MultiLineString(lines),
            srid,
            bbox,
            flags,
        })
    }

    /// Create a MultiPolygon geometry.
    pub fn multi_polygon(
        polygons: Vec<PolygonData>,
        srid: Srid,
    ) -> Result<Self, GeometryError> {
        if polygons.is_empty() {
            return Err(GeometryError::EmptyGeometry);
        }
        for poly in &polygons {
            validation::validate_polygon(&poly.exterior, &poly.holes)?;
        }
        let all_coords: Vec<Coordinate> = polygons
            .iter()
            .flat_map(|p| p.exterior.iter())
            .cloned()
            .collect();
        let bbox = BoundingBox::from_coordinates(&all_coords);
        let mut flags = GeometryFlags::HAS_SRID;
        if bbox.is_some() {
            flags |= GeometryFlags::HAS_BBOX;
        }
        Ok(Self {
            geometry_type: GeometryType::MultiPolygon(polygons),
            srid,
            bbox,
            flags,
        })
    }

    /// Create a GeometryCollection.
    pub fn geometry_collection(
        geometries: Vec<SurrealGeometry>,
        srid: Srid,
    ) -> Result<Self, GeometryError> {
        if geometries.is_empty() {
            return Err(GeometryError::EmptyGeometry);
        }
        // Compute bbox as union of all children
        let bbox = geometries.iter().fold(None::<BoundingBox>, |acc, g| {
            match (acc, g.bbox()) {
                (None, Some(b)) => Some(b.clone()),
                (Some(a), Some(b)) => Some(a.expand(b)),
                (a, None) => a,
            }
        });
        let mut flags = GeometryFlags::HAS_SRID;
        if bbox.is_some() {
            flags |= GeometryFlags::HAS_BBOX;
        }
        Ok(Self {
            geometry_type: GeometryType::GeometryCollection(geometries),
            srid,
            bbox,
            flags,
        })
    }

    // ── Internal constructor (for conversions) ──────────────────────

    /// Build a SurrealGeometry directly from parts (used by conversion code).
    pub(crate) fn from_parts(
        geometry_type: GeometryType,
        srid: Srid,
    ) -> Self {
        let bbox = Self::compute_bbox_for(&geometry_type);
        let mut flags = GeometryFlags::HAS_SRID;
        if bbox.is_some() {
            flags |= GeometryFlags::HAS_BBOX;
        }
        Self {
            geometry_type,
            srid,
            bbox,
            flags,
        }
    }

    // ── Accessors ───────────────────────────────────────────────────

    pub fn geometry_type(&self) -> &GeometryType {
        &self.geometry_type
    }

    pub fn srid(&self) -> &Srid {
        &self.srid
    }

    pub fn bbox(&self) -> Option<&BoundingBox> {
        self.bbox.as_ref()
    }

    pub fn flags(&self) -> GeometryFlags {
        self.flags
    }

    pub fn type_name(&self) -> &str {
        match &self.geometry_type {
            GeometryType::Point(_) => "Point",
            GeometryType::LineString(_) => "LineString",
            GeometryType::Polygon { .. } => "Polygon",
            GeometryType::MultiPoint(_) => "MultiPoint",
            GeometryType::MultiLineString(_) => "MultiLineString",
            GeometryType::MultiPolygon(_) => "MultiPolygon",
            GeometryType::GeometryCollection(_) => "GeometryCollection",
        }
    }

    pub fn is_empty(&self) -> bool {
        self.flags.contains(GeometryFlags::IS_EMPTY)
    }

    pub fn num_points(&self) -> usize {
        match &self.geometry_type {
            GeometryType::Point(_) => 1,
            GeometryType::LineString(coords) => coords.len(),
            GeometryType::Polygon { exterior, holes } => {
                exterior.len() + holes.iter().map(|h| h.len()).sum::<usize>()
            }
            GeometryType::MultiPoint(coords) => coords.len(),
            GeometryType::MultiLineString(lines) => {
                lines.iter().map(|l| l.len()).sum()
            }
            GeometryType::MultiPolygon(polygons) => polygons
                .iter()
                .map(|p| {
                    p.exterior.len()
                        + p.holes.iter().map(|h| h.len()).sum::<usize>()
                })
                .sum(),
            GeometryType::GeometryCollection(geoms) => {
                geoms.iter().map(|g| g.num_points()).sum()
            }
        }
    }

    /// Returns the coordinate dimension (2 for XY, 3 for XYZ, 4 for XYZM).
    pub fn dimension(&self) -> u8 {
        if self.flags.contains(GeometryFlags::HAS_Z) && self.flags.contains(GeometryFlags::HAS_M) {
            4
        } else if self.flags.contains(GeometryFlags::HAS_Z) {
            3
        } else {
            2
        }
    }

    /// Recompute the bounding box from coordinates.
    pub fn compute_bbox(&mut self) {
        self.bbox = Self::compute_bbox_for(&self.geometry_type);
        if self.bbox.is_some() {
            self.flags |= GeometryFlags::HAS_BBOX;
        } else {
            self.flags.remove(GeometryFlags::HAS_BBOX);
        }
    }

    fn compute_bbox_for(gt: &GeometryType) -> Option<BoundingBox> {
        match gt {
            GeometryType::Point(c) => BoundingBox::from_coordinates(&[c.clone()]),
            GeometryType::LineString(coords) => BoundingBox::from_coordinates(coords),
            GeometryType::Polygon { exterior, .. } => {
                BoundingBox::from_coordinates(exterior)
            }
            GeometryType::MultiPoint(coords) => BoundingBox::from_coordinates(coords),
            GeometryType::MultiLineString(lines) => {
                let all: Vec<Coordinate> = lines.iter().flatten().cloned().collect();
                BoundingBox::from_coordinates(&all)
            }
            GeometryType::MultiPolygon(polygons) => {
                let all: Vec<Coordinate> = polygons
                    .iter()
                    .flat_map(|p| p.exterior.iter())
                    .cloned()
                    .collect();
                BoundingBox::from_coordinates(&all)
            }
            GeometryType::GeometryCollection(geoms) => {
                geoms.iter().fold(None::<BoundingBox>, |acc, g| {
                    match (acc, g.bbox()) {
                        (None, Some(b)) => Some(b.clone()),
                        (Some(a), Some(b)) => Some(a.expand(b)),
                        (a, None) => a,
                    }
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_point() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        assert_eq!(p.type_name(), "Point");
        assert_eq!(p.num_points(), 1);
        assert_eq!(p.srid().code(), 4326);
        assert!(p.bbox().is_some());
    }

    #[test]
    fn create_linestring() {
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
            Coordinate::new(2.0, 0.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WGS84).unwrap();
        assert_eq!(ls.type_name(), "LineString");
        assert_eq!(ls.num_points(), 3);
    }

    #[test]
    fn create_polygon() {
        let exterior = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(10.0, 0.0).unwrap(),
            Coordinate::new(10.0, 10.0).unwrap(),
            Coordinate::new(0.0, 0.0).unwrap(),
        ];
        let poly = SurrealGeometry::polygon(exterior, vec![], Srid::WGS84).unwrap();
        assert_eq!(poly.type_name(), "Polygon");
        assert_eq!(poly.num_points(), 4);
        let bb = poly.bbox().unwrap();
        assert_eq!(bb.min_x, 0.0);
        assert_eq!(bb.max_x, 10.0);
    }

    #[test]
    fn create_multi_point() {
        let coords = vec![
            Coordinate::new(1.0, 2.0).unwrap(),
            Coordinate::new(3.0, 4.0).unwrap(),
        ];
        let mp = SurrealGeometry::multi_point(coords, Srid::WGS84).unwrap();
        assert_eq!(mp.type_name(), "MultiPoint");
        assert_eq!(mp.num_points(), 2);
    }

    #[test]
    fn create_multi_linestring() {
        let lines = vec![
            vec![
                Coordinate::new(0.0, 0.0).unwrap(),
                Coordinate::new(1.0, 1.0).unwrap(),
            ],
            vec![
                Coordinate::new(2.0, 2.0).unwrap(),
                Coordinate::new(3.0, 3.0).unwrap(),
            ],
        ];
        let mls = SurrealGeometry::multi_line_string(lines, Srid::WGS84).unwrap();
        assert_eq!(mls.type_name(), "MultiLineString");
        assert_eq!(mls.num_points(), 4);
    }

    #[test]
    fn create_multi_polygon() {
        let polygons = vec![PolygonData {
            exterior: vec![
                Coordinate::new(0.0, 0.0).unwrap(),
                Coordinate::new(1.0, 0.0).unwrap(),
                Coordinate::new(1.0, 1.0).unwrap(),
                Coordinate::new(0.0, 0.0).unwrap(),
            ],
            holes: vec![],
        }];
        let mp = SurrealGeometry::multi_polygon(polygons, Srid::WGS84).unwrap();
        assert_eq!(mp.type_name(), "MultiPolygon");
    }

    #[test]
    fn create_geometry_collection() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        let coords = vec![
            Coordinate::new(0.0, 0.0).unwrap(),
            Coordinate::new(1.0, 1.0).unwrap(),
        ];
        let ls = SurrealGeometry::line_string(coords, Srid::WGS84).unwrap();
        let gc = SurrealGeometry::geometry_collection(vec![p, ls], Srid::WGS84).unwrap();
        assert_eq!(gc.type_name(), "GeometryCollection");
        assert_eq!(gc.num_points(), 3);
    }

    #[test]
    fn empty_multi_point_rejected() {
        let result = SurrealGeometry::multi_point(vec![], Srid::WGS84);
        assert!(matches!(result.unwrap_err(), GeometryError::EmptyGeometry));
    }

    #[test]
    fn empty_geometry_collection_rejected() {
        let result = SurrealGeometry::geometry_collection(vec![], Srid::WGS84);
        assert!(matches!(result.unwrap_err(), GeometryError::EmptyGeometry));
    }

    #[test]
    fn compute_bbox_updates() {
        let mut p = SurrealGeometry::point(5.0, 10.0, Srid::WGS84).unwrap();
        p.compute_bbox();
        let bb = p.bbox().unwrap();
        assert_eq!(bb.min_x, 5.0);
        assert_eq!(bb.min_y, 10.0);
    }

    #[test]
    fn dimension_default_is_2d() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        assert_eq!(p.dimension(), 2);
    }

    #[test]
    fn is_empty_is_false_for_point() {
        let p = SurrealGeometry::point(1.0, 2.0, Srid::WGS84).unwrap();
        assert!(!p.is_empty());
    }
}
