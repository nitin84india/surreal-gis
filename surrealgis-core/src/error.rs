use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum GeometryError {
    #[error("Invalid coordinate: {0}")]
    InvalidCoordinate(String),
    #[error("Invalid SRID: {0}")]
    InvalidSrid(String),
    #[error("Invalid geometry: {0}")]
    InvalidGeometry(String),
    #[error("Empty geometry")]
    EmptyGeometry,
    #[error("Conversion error: {0}")]
    ConversionError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Unsupported geometry type: {0}")]
    UnsupportedGeometryType(String),
    #[error("Dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: String, got: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_invalid_coordinate() {
        let err = GeometryError::InvalidCoordinate("NaN value".to_string());
        assert_eq!(err.to_string(), "Invalid coordinate: NaN value");
    }

    #[test]
    fn error_display_invalid_srid() {
        let err = GeometryError::InvalidSrid("negative SRID".to_string());
        assert_eq!(err.to_string(), "Invalid SRID: negative SRID");
    }

    #[test]
    fn error_display_dimension_mismatch() {
        let err = GeometryError::DimensionMismatch {
            expected: "2D".to_string(),
            got: "3D".to_string(),
        };
        assert_eq!(err.to_string(), "Dimension mismatch: expected 2D, got 3D");
    }

    #[test]
    fn error_display_empty_geometry() {
        let err = GeometryError::EmptyGeometry;
        assert_eq!(err.to_string(), "Empty geometry");
    }

    #[test]
    fn error_clone_and_eq() {
        let err1 = GeometryError::InvalidGeometry("bad".to_string());
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }
}
