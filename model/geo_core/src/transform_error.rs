//! Transform operation error definitions for geo_core.

use analysis::abstract_types::{Angle, Scalar};
use std::fmt;

/// Errors that can occur during transform operations.
#[derive(Debug, Clone, PartialEq)]
pub enum TransformError {
    /// Resulting geometry became invalid (for example, zero scale).
    InvalidGeometry(String),
    /// Zero-length vector was used where normalization is required.
    ZeroVector(String),
    /// Invalid scale factor was provided.
    InvalidScaleFactor(String),
    /// Invalid rotation parameters were provided.
    InvalidRotation(String),
}

impl fmt::Display for TransformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransformError::InvalidGeometry(msg) => {
                write!(f, "invalid geometry after transform: {msg}")
            }
            TransformError::ZeroVector(msg) => write!(f, "zero vector is not allowed: {msg}"),
            TransformError::InvalidScaleFactor(msg) => write!(f, "invalid scale factor: {msg}"),
            TransformError::InvalidRotation(msg) => write!(f, "invalid rotation: {msg}"),
        }
    }
}

impl std::error::Error for TransformError {}

/// Fallible transform operations.
pub trait SafeTransform<T: Scalar> {
    fn safe_translate(&self, offset: T) -> Result<Self, TransformError>
    where
        Self: Sized;

    fn safe_scale(&self, center: T, factor: T) -> Result<Self, TransformError>
    where
        Self: Sized;

    fn safe_rotate(&self, center: T, axis: T, angle: Angle<T>) -> Result<Self, TransformError>
    where
        Self: Sized;
}
