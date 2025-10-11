//! Common utilities and classification
//! 共通ユーティリティと分類システム

pub mod classification;
pub mod geometry_utils;
pub mod primitive_trait;

pub use classification::{DimensionClass, GeometryPrimitive, GeometryUnion, PrimitiveKind};
pub use geometry_utils::*;
pub use primitive_trait::{
    GeometricPrimitive, MeasurablePrimitive, PrimitiveCollection, TransformablePrimitive,
};
