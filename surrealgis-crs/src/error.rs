use thiserror::Error;

/// CRS-specific errors for projection and coordinate reference system operations.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum CrsError {
    #[error("Unknown SRID: {0}")]
    UnknownSrid(i32),

    #[error("Projection error: {0}")]
    ProjectionError(String),

    #[error("Invalid coordinate for transform: {0}")]
    InvalidCoordinate(String),

    #[error("Same source and target SRID: {0}")]
    SameSrid(i32),

    #[error("Geometry construction error: {0}")]
    GeometryError(String),
}

impl From<surrealgis_core::error::GeometryError> for CrsError {
    fn from(err: surrealgis_core::error::GeometryError) -> Self {
        CrsError::GeometryError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_unknown_srid() {
        let err = CrsError::UnknownSrid(99999);
        assert_eq!(err.to_string(), "Unknown SRID: 99999");
    }

    #[test]
    fn error_display_projection_error() {
        let err = CrsError::ProjectionError("invalid proj string".to_string());
        assert_eq!(err.to_string(), "Projection error: invalid proj string");
    }

    #[test]
    fn error_display_invalid_coordinate() {
        let err = CrsError::InvalidCoordinate("NaN value".to_string());
        assert_eq!(err.to_string(), "Invalid coordinate for transform: NaN value");
    }

    #[test]
    fn error_display_same_srid() {
        let err = CrsError::SameSrid(4326);
        assert_eq!(err.to_string(), "Same source and target SRID: 4326");
    }

    #[test]
    fn error_display_geometry_error() {
        let err = CrsError::GeometryError("empty geometry".to_string());
        assert_eq!(err.to_string(), "Geometry construction error: empty geometry");
    }

    #[test]
    fn error_clone_and_eq() {
        let err1 = CrsError::UnknownSrid(4326);
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }

    #[test]
    fn from_geometry_error() {
        let geom_err = surrealgis_core::error::GeometryError::EmptyGeometry;
        let crs_err: CrsError = geom_err.into();
        assert_eq!(crs_err, CrsError::GeometryError("Empty geometry".to_string()));
    }
}
