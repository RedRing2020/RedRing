//! Traits module organization
//! トレイトとユーティリティの整理されたモジュール構成

// Common utilities - 共通ユーティリティと分類
pub mod common;

// Re-export traits directly from geo_foundation
pub use geo_foundation::abstract_types::geometry::{
    Vector, Vector2D, Vector3D, Vector2DExt, Vector3DExt, Normalizable,
    BoundingBox, BoundingBoxOps, CollisionBounds
};

// Re-export common utilities
pub use common::{PrimitiveKind, DimensionClass, GeometryPrimitive, GeometryUnion};
