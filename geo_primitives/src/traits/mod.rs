//! Traits module organization
//! トレイトとユーティリティの整理されたモジュール構成

// Vector traits - ベクトル操作のためのトレイト定義
pub mod vector_traits;
pub mod normalizable;

// Bounding box traits - バウンディングボックストレイト
pub mod bbox_trait;

// Geometry structures - 幾何構造体
pub mod geometry;

// Common utilities - 共通ユーティリティと分類
pub mod common;

// Re-export commonly used traits
pub use vector_traits::Vector;
pub use vector_traits::Vector2DExt;
pub use vector_traits::Vector3DExt;
pub use normalizable::Normalizable;

// Re-export bounding box traits
pub use bbox_trait::{BoundingBox, BoundingBoxOps, CollisionBounds};

// Re-export geometry structures
pub use geometry::Point;
pub use geometry::Vector as GeometryVector;

// Re-export common utilities
pub use common::{PrimitiveKind, DimensionClass, GeometryPrimitive, GeometryUnion};
