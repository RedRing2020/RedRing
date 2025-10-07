//! Traits module organization
//! トレイトとユーティリティの整理されたモジュール構成

// Common utilities - 共通ユーティリティと分類
pub mod common;

// Re-export traits directly from geo_foundation
pub use geo_foundation::abstract_types::geometry::{
    BoundingBox, BoundingBoxOps, CollisionBounds, Direction, Direction2D, Direction3D,
    Normalizable, StepCompatible, Vector, Vector2D, Vector2DExt, Vector3D, Vector3DExt,
};

// Re-export common utilities
pub use common::{DimensionClass, GeometryPrimitive, GeometryUnion, PrimitiveKind};
