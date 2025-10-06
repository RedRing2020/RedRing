/// Common utilities and classification
/// 共通ユーティリティと分類システム

pub mod classification;
pub mod geometry_utils;
pub mod primitive_trait;

pub use classification::{PrimitiveKind, DimensionClass, GeometryPrimitive, GeometryUnion};
pub use geometry_utils::*;
pub use primitive_trait::{GeometricPrimitive, TransformablePrimitive, MeasurablePrimitive, PrimitiveCollection};
