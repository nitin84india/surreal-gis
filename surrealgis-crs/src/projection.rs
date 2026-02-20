use std::fmt;

use proj4rs::Proj;

use crate::error::CrsError;
use crate::registry;

/// A wrapper around a proj4rs projection with associated SRID metadata.
pub struct Projection {
    proj: Proj,
    srid: i32,
    is_geographic: bool,
}

impl Projection {
    /// Create a new projection from a known SRID.
    ///
    /// First attempts to use the built-in `crs-definitions` feature of proj4rs
    /// for the most accurate definition. Falls back to the local registry's
    /// proj4 string if the EPSG code is not found in proj4rs's built-in database.
    pub fn new(srid: i32) -> Result<Self, CrsError> {
        // Try proj4rs built-in EPSG definitions first (most accurate)
        let proj = if srid > 0 && srid <= u16::MAX as i32 {
            Proj::from_epsg_code(srid as u16).or_else(|_| {
                // Fall back to our local registry
                let proj4_str = registry::get_proj4_string(srid)
                    .ok_or(CrsError::UnknownSrid(srid))?;
                Proj::from_proj_string(proj4_str)
                    .map_err(|e| CrsError::ProjectionError(e.to_string()))
            })
        } else {
            // Negative or oversized SRIDs: only check local registry
            let proj4_str = registry::get_proj4_string(srid)
                .ok_or(CrsError::UnknownSrid(srid))?;
            Proj::from_proj_string(proj4_str)
                .map_err(|e| CrsError::ProjectionError(e.to_string()))
        }?;

        Ok(Self {
            proj,
            srid,
            is_geographic: registry::is_geographic(srid),
        })
    }

    /// Returns a reference to the underlying proj4rs Proj instance.
    pub fn proj(&self) -> &Proj {
        &self.proj
    }

    /// Returns the SRID code for this projection.
    pub fn srid(&self) -> i32 {
        self.srid
    }

    /// Returns true if this is a geographic (lon/lat degrees) CRS.
    pub fn is_geographic(&self) -> bool {
        self.is_geographic
    }
}

impl fmt::Debug for Projection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Projection")
            .field("srid", &self.srid)
            .field("is_geographic", &self.is_geographic)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_wgs84_projection() {
        let proj = Projection::new(4326).unwrap();
        assert_eq!(proj.srid(), 4326);
        assert!(proj.is_geographic());
    }

    #[test]
    fn create_web_mercator_projection() {
        let proj = Projection::new(3857).unwrap();
        assert_eq!(proj.srid(), 3857);
        assert!(!proj.is_geographic());
    }

    #[test]
    fn create_nad83_projection() {
        let proj = Projection::new(4269).unwrap();
        assert_eq!(proj.srid(), 4269);
        assert!(proj.is_geographic());
    }

    #[test]
    fn create_utm_zone_18n_projection() {
        let proj = Projection::new(32618).unwrap();
        assert_eq!(proj.srid(), 32618);
        assert!(!proj.is_geographic());
    }

    #[test]
    fn create_lambert_93_projection() {
        let proj = Projection::new(2154).unwrap();
        assert_eq!(proj.srid(), 2154);
        assert!(!proj.is_geographic());
    }

    #[test]
    fn unknown_srid_returns_error() {
        let result = Projection::new(99999);
        assert!(result.is_err());
        match result.unwrap_err() {
            CrsError::UnknownSrid(code) => assert_eq!(code, 99999),
            other => panic!("Expected UnknownSrid, got: {:?}", other),
        }
    }

    #[test]
    fn proj_accessor_returns_valid_ref() {
        let projection = Projection::new(4326).unwrap();
        // Verify we can access the underlying Proj
        let _proj_ref = projection.proj();
    }

    #[test]
    fn laea_europe_projection() {
        let proj = Projection::new(3035).unwrap();
        assert_eq!(proj.srid(), 3035);
        assert!(!proj.is_geographic());
    }
}
