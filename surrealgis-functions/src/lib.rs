pub mod constructors;
pub mod accessors;
pub mod relationships;
pub mod measurement;
pub mod output;
pub mod crs;
pub mod affine;
pub mod processing;
pub mod overlay;
pub mod editors;
pub mod linear_ref;
pub mod clustering;

use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum FunctionError {
    #[error("Geometry error: {0}")]
    GeometryError(#[from] surrealgis_core::error::GeometryError),
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
    #[error("CRS error: {0}")]
    CrsError(String),
}
