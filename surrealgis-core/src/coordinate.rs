use serde::{Deserialize, Serialize};

use crate::error::GeometryError;

/// An immutable coordinate value object with x, y, and optional z, m fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coordinate {
    x: f64,
    y: f64,
    z: Option<f64>,
    m: Option<f64>,
}

impl Coordinate {
    /// Create a 2D coordinate.
    pub fn new(x: f64, y: f64) -> Result<Self, GeometryError> {
        Self::validate_finite(x, "x")?;
        Self::validate_finite(y, "y")?;
        Ok(Self {
            x,
            y,
            z: None,
            m: None,
        })
    }

    /// Create a 3D coordinate (with Z).
    pub fn new_3d(x: f64, y: f64, z: f64) -> Result<Self, GeometryError> {
        Self::validate_finite(x, "x")?;
        Self::validate_finite(y, "y")?;
        Self::validate_finite(z, "z")?;
        Ok(Self {
            x,
            y,
            z: Some(z),
            m: None,
        })
    }

    /// Create a 4D coordinate (with Z and M).
    pub fn new_4d(x: f64, y: f64, z: f64, m: f64) -> Result<Self, GeometryError> {
        Self::validate_finite(x, "x")?;
        Self::validate_finite(y, "y")?;
        Self::validate_finite(z, "z")?;
        Self::validate_finite(m, "m")?;
        Ok(Self {
            x,
            y,
            z: Some(z),
            m: Some(m),
        })
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn z(&self) -> Option<f64> {
        self.z
    }

    pub fn m(&self) -> Option<f64> {
        self.m
    }

    /// Check if the coordinate is a valid geographic coordinate
    /// (longitude in [-180, 180], latitude in [-90, 90]).
    pub fn is_geographic_valid(&self) -> bool {
        (-180.0..=180.0).contains(&self.x) && (-90.0..=90.0).contains(&self.y)
    }

    fn validate_finite(val: f64, name: &str) -> Result<(), GeometryError> {
        if !val.is_finite() {
            return Err(GeometryError::InvalidCoordinate(format!(
                "{name} must be finite, got {val}"
            )));
        }
        Ok(())
    }
}

impl From<Coordinate> for geo_types::Coord<f64> {
    fn from(c: Coordinate) -> Self {
        geo_types::Coord { x: c.x, y: c.y }
    }
}

impl From<&Coordinate> for geo_types::Coord<f64> {
    fn from(c: &Coordinate) -> Self {
        geo_types::Coord { x: c.x, y: c.y }
    }
}

impl From<geo_types::Coord<f64>> for Coordinate {
    fn from(c: geo_types::Coord<f64>) -> Self {
        // geo_types::Coord values are expected to be valid f64
        Self {
            x: c.x,
            y: c.y,
            z: None,
            m: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_2d_coordinate() {
        let c = Coordinate::new(1.0, 2.0).unwrap();
        assert_eq!(c.x(), 1.0);
        assert_eq!(c.y(), 2.0);
        assert_eq!(c.z(), None);
        assert_eq!(c.m(), None);
    }

    #[test]
    fn new_3d_coordinate() {
        let c = Coordinate::new_3d(1.0, 2.0, 3.0).unwrap();
        assert_eq!(c.x(), 1.0);
        assert_eq!(c.y(), 2.0);
        assert_eq!(c.z(), Some(3.0));
        assert_eq!(c.m(), None);
    }

    #[test]
    fn new_4d_coordinate() {
        let c = Coordinate::new_4d(1.0, 2.0, 3.0, 4.0).unwrap();
        assert_eq!(c.x(), 1.0);
        assert_eq!(c.y(), 2.0);
        assert_eq!(c.z(), Some(3.0));
        assert_eq!(c.m(), Some(4.0));
    }

    #[test]
    fn nan_x_rejected() {
        let result = Coordinate::new(f64::NAN, 1.0);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GeometryError::InvalidCoordinate(_)));
    }

    #[test]
    fn nan_y_rejected() {
        let result = Coordinate::new(1.0, f64::NAN);
        assert!(result.is_err());
    }

    #[test]
    fn infinity_rejected() {
        let result = Coordinate::new(f64::INFINITY, 1.0);
        assert!(result.is_err());
    }

    #[test]
    fn neg_infinity_rejected() {
        let result = Coordinate::new(1.0, f64::NEG_INFINITY);
        assert!(result.is_err());
    }

    #[test]
    fn nan_z_rejected() {
        let result = Coordinate::new_3d(1.0, 2.0, f64::NAN);
        assert!(result.is_err());
    }

    #[test]
    fn nan_m_rejected() {
        let result = Coordinate::new_4d(1.0, 2.0, 3.0, f64::NAN);
        assert!(result.is_err());
    }

    #[test]
    fn is_geographic_valid_in_range() {
        let c = Coordinate::new(45.0, 30.0).unwrap();
        assert!(c.is_geographic_valid());
    }

    #[test]
    fn is_geographic_valid_at_bounds() {
        let c = Coordinate::new(180.0, 90.0).unwrap();
        assert!(c.is_geographic_valid());
        let c = Coordinate::new(-180.0, -90.0).unwrap();
        assert!(c.is_geographic_valid());
    }

    #[test]
    fn is_geographic_valid_out_of_range() {
        let c = Coordinate::new(200.0, 30.0).unwrap();
        assert!(!c.is_geographic_valid());
        let c = Coordinate::new(45.0, 100.0).unwrap();
        assert!(!c.is_geographic_valid());
    }

    #[test]
    fn convert_to_geo_coord() {
        let c = Coordinate::new(1.5, 2.5).unwrap();
        let gc: geo_types::Coord<f64> = c.into();
        assert_eq!(gc.x, 1.5);
        assert_eq!(gc.y, 2.5);
    }

    #[test]
    fn convert_from_geo_coord() {
        let gc = geo_types::Coord { x: 3.0, y: 4.0 };
        let c: Coordinate = gc.into();
        assert_eq!(c.x(), 3.0);
        assert_eq!(c.y(), 4.0);
    }

    #[test]
    fn coordinate_serialization_roundtrip() {
        let c = Coordinate::new_3d(1.0, 2.0, 3.0).unwrap();
        let json = serde_json::to_string(&c).unwrap();
        let deserialized: Coordinate = serde_json::from_str(&json).unwrap();
        assert_eq!(c, deserialized);
    }
}
