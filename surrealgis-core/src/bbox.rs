use serde::{Deserialize, Serialize};

use crate::coordinate::Coordinate;
use crate::error::GeometryError;

/// Axis-aligned bounding box value object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl BoundingBox {
    /// Create a new bounding box. Validates that min <= max.
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Result<Self, GeometryError> {
        if min_x > max_x {
            return Err(GeometryError::InvalidGeometry(format!(
                "min_x ({min_x}) must be <= max_x ({max_x})"
            )));
        }
        if min_y > max_y {
            return Err(GeometryError::InvalidGeometry(format!(
                "min_y ({min_y}) must be <= max_y ({max_y})"
            )));
        }
        Ok(Self {
            min_x,
            min_y,
            max_x,
            max_y,
        })
    }

    /// Compute a bounding box from a slice of coordinates.
    /// Returns None if the slice is empty.
    pub fn from_coordinates(coords: &[Coordinate]) -> Option<Self> {
        if coords.is_empty() {
            return None;
        }

        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        for c in coords {
            if c.x() < min_x {
                min_x = c.x();
            }
            if c.y() < min_y {
                min_y = c.y();
            }
            if c.x() > max_x {
                max_x = c.x();
            }
            if c.y() > max_y {
                max_y = c.y();
            }
        }

        Some(Self {
            min_x,
            min_y,
            max_x,
            max_y,
        })
    }

    /// Check if this bounding box intersects another.
    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.min_x <= other.max_x
            && self.max_x >= other.min_x
            && self.min_y <= other.max_y
            && self.max_y >= other.min_y
    }

    /// Check if this bounding box fully contains another.
    pub fn contains(&self, other: &BoundingBox) -> bool {
        self.min_x <= other.min_x
            && self.max_x >= other.max_x
            && self.min_y <= other.min_y
            && self.max_y >= other.max_y
    }

    /// Check if this bounding box contains a coordinate.
    pub fn contains_coordinate(&self, coord: &Coordinate) -> bool {
        coord.x() >= self.min_x
            && coord.x() <= self.max_x
            && coord.y() >= self.min_y
            && coord.y() <= self.max_y
    }

    /// Compute the union of this bounding box with another.
    pub fn expand(&self, other: &BoundingBox) -> BoundingBox {
        BoundingBox {
            min_x: self.min_x.min(other.min_x),
            min_y: self.min_y.min(other.min_y),
            max_x: self.max_x.max(other.max_x),
            max_y: self.max_y.max(other.max_y),
        }
    }

    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    pub fn area(&self) -> f64 {
        self.width() * self.height()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_valid_bbox() {
        let bb = BoundingBox::new(0.0, 0.0, 10.0, 10.0).unwrap();
        assert_eq!(bb.min_x, 0.0);
        assert_eq!(bb.max_x, 10.0);
    }

    #[test]
    fn reject_invalid_min_max() {
        let result = BoundingBox::new(10.0, 0.0, 0.0, 10.0);
        assert!(result.is_err());
    }

    #[test]
    fn reject_invalid_min_max_y() {
        let result = BoundingBox::new(0.0, 10.0, 10.0, 0.0);
        assert!(result.is_err());
    }

    #[test]
    fn from_coordinates_computes_bbox() {
        let coords = vec![
            Coordinate::new(1.0, 2.0).unwrap(),
            Coordinate::new(5.0, 8.0).unwrap(),
            Coordinate::new(3.0, 4.0).unwrap(),
        ];
        let bb = BoundingBox::from_coordinates(&coords).unwrap();
        assert_eq!(bb.min_x, 1.0);
        assert_eq!(bb.min_y, 2.0);
        assert_eq!(bb.max_x, 5.0);
        assert_eq!(bb.max_y, 8.0);
    }

    #[test]
    fn from_coordinates_empty_returns_none() {
        let result = BoundingBox::from_coordinates(&[]);
        assert!(result.is_none());
    }

    #[test]
    fn intersects_overlapping() {
        let a = BoundingBox::new(0.0, 0.0, 5.0, 5.0).unwrap();
        let b = BoundingBox::new(3.0, 3.0, 8.0, 8.0).unwrap();
        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
    }

    #[test]
    fn intersects_non_overlapping() {
        let a = BoundingBox::new(0.0, 0.0, 2.0, 2.0).unwrap();
        let b = BoundingBox::new(5.0, 5.0, 8.0, 8.0).unwrap();
        assert!(!a.intersects(&b));
    }

    #[test]
    fn contains_bbox() {
        let outer = BoundingBox::new(0.0, 0.0, 10.0, 10.0).unwrap();
        let inner = BoundingBox::new(2.0, 2.0, 5.0, 5.0).unwrap();
        assert!(outer.contains(&inner));
        assert!(!inner.contains(&outer));
    }

    #[test]
    fn contains_coordinate_inside() {
        let bb = BoundingBox::new(0.0, 0.0, 10.0, 10.0).unwrap();
        let c = Coordinate::new(5.0, 5.0).unwrap();
        assert!(bb.contains_coordinate(&c));
    }

    #[test]
    fn contains_coordinate_outside() {
        let bb = BoundingBox::new(0.0, 0.0, 10.0, 10.0).unwrap();
        let c = Coordinate::new(15.0, 5.0).unwrap();
        assert!(!bb.contains_coordinate(&c));
    }

    #[test]
    fn expand_union() {
        let a = BoundingBox::new(0.0, 0.0, 5.0, 5.0).unwrap();
        let b = BoundingBox::new(3.0, 3.0, 8.0, 8.0).unwrap();
        let u = a.expand(&b);
        assert_eq!(u.min_x, 0.0);
        assert_eq!(u.min_y, 0.0);
        assert_eq!(u.max_x, 8.0);
        assert_eq!(u.max_y, 8.0);
    }

    #[test]
    fn width_height_area() {
        let bb = BoundingBox::new(1.0, 2.0, 5.0, 7.0).unwrap();
        assert_eq!(bb.width(), 4.0);
        assert_eq!(bb.height(), 5.0);
        assert_eq!(bb.area(), 20.0);
    }

    #[test]
    fn degenerate_point_bbox() {
        let bb = BoundingBox::new(5.0, 5.0, 5.0, 5.0).unwrap();
        assert_eq!(bb.width(), 0.0);
        assert_eq!(bb.height(), 0.0);
        assert_eq!(bb.area(), 0.0);
    }
}
