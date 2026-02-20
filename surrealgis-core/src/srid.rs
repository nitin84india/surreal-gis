use serde::{Deserialize, Serialize};

use crate::error::GeometryError;

/// SRID (Spatial Reference System Identifier) newtype.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Srid(i32);

impl Srid {
    /// WGS 84 geographic coordinate system.
    pub const WGS84: Srid = Srid(4326);
    /// Web Mercator projection.
    pub const WEB_MERCATOR: Srid = Srid(3857);
    /// NAD83 geographic coordinate system.
    pub const NAD83: Srid = Srid(4269);
    /// Default SRID (WGS 84).
    pub const DEFAULT: Srid = Srid(4326);

    /// Create a new SRID from a code. Code must be positive.
    pub fn new(code: i32) -> Result<Self, GeometryError> {
        if code <= 0 {
            return Err(GeometryError::InvalidSrid(format!(
                "SRID must be positive, got {code}"
            )));
        }
        Ok(Srid(code))
    }

    /// Return the numeric SRID code.
    pub fn code(&self) -> i32 {
        self.0
    }

    /// Returns true for well-known geographic (lon/lat) SRIDs.
    pub fn is_geographic(&self) -> bool {
        matches!(
            self.0,
            4326 | 4269 | 4267 | 4258 | 4148 | 4674 | 4283 | 4612 | 4490
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_valid_srid() {
        let srid = Srid::new(4326).unwrap();
        assert_eq!(srid.code(), 4326);
    }

    #[test]
    fn reject_zero_srid() {
        let result = Srid::new(0);
        assert!(result.is_err());
    }

    #[test]
    fn reject_negative_srid() {
        let result = Srid::new(-1);
        assert!(result.is_err());
    }

    #[test]
    fn wgs84_is_geographic() {
        assert!(Srid::WGS84.is_geographic());
    }

    #[test]
    fn web_mercator_is_not_geographic() {
        assert!(!Srid::WEB_MERCATOR.is_geographic());
    }

    #[test]
    fn nad83_is_geographic() {
        assert!(Srid::NAD83.is_geographic());
    }

    #[test]
    fn constants_have_expected_values() {
        assert_eq!(Srid::WGS84.code(), 4326);
        assert_eq!(Srid::WEB_MERCATOR.code(), 3857);
        assert_eq!(Srid::NAD83.code(), 4269);
        assert_eq!(Srid::DEFAULT.code(), 4326);
    }

    #[test]
    fn srid_serialization_roundtrip() {
        let srid = Srid::WGS84;
        let json = serde_json::to_string(&srid).unwrap();
        let deserialized: Srid = serde_json::from_str(&json).unwrap();
        assert_eq!(srid, deserialized);
    }
}
