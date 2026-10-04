//! Transform operation error definitions for geo_core.
//!
//! `TransformError` の正本は `geo_contracts` にあり、ここでは再エクスポートする。

use analysis::abstract_types::{Angle, Scalar};

pub use geo_contracts::TransformError;

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
