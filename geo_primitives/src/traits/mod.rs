//! Traits module organization
//! トレイトとユーティリティの整理されたモジュール構成

// Common utilities - 共通ユーティリティと分類
pub mod common;

// Re-export traits directly from geo_foundation
pub use geo_foundation::abstract_types::geometry::{
    BBox, BBoxOps, Circle2D, Circle3D, CollisionBBox, Direction, Direction2D, Direction3D,
    Normalizable, StepCompatible, Vector, Vector2D, Vector2DExt, Vector3D, Vector3DExt,
};

// Re-export abstract primitive traits from geo_foundation
pub use geo_foundation::abstract_types::geometry::{
    DimensionClass, GeometricPrimitive, GeometryPrimitive, MeasurablePrimitive, PrimitiveKind,
    PrimitiveCollection, SpatialRelation, TransformablePrimitive,
};

// Re-export utility functions from geo_foundation
pub use geo_foundation::abstract_types::geometry::utils::{
    clamp, f64_max, f64_min, in_range, lerp, scalar_distance, scalar_max, scalar_min,
};

// Re-export local Arc2D trait from geometry2d module
pub use crate::geometry2d::Arc2D;

// Re-export geo_primitives specific implementations
pub use common::{GeometryUnion};
