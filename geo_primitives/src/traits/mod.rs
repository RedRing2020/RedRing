/// Traits module organization
/// トレイトとユーティリティの整理されたモジュール構成

// Vector traits - ベクトル操作のためのトレイト定義
pub mod vector;
pub mod vector2d_ext;
pub mod vector3d_ext;
pub mod normalizable;

// Bounding box traits - バウンディングボックストレイト
pub mod bbox_trait;

// Geometry structures - 幾何構造体
pub mod geometry;

// Common utilities - 共通ユーティリティと分類
pub mod common;

// Re-export commonly used traits
pub use vector::Vector;
pub use vector2d_ext::Vector2DExt;
pub use vector3d_ext::Vector3DExt;
pub use normalizable::Normalizable;

// Re-export bounding box traits
pub use bbox_trait::{BoundingBox, BoundingBoxOps, CollisionBounds};

// Re-export geometry structures
pub use geometry::{CadPoint, CadVector};

// Re-export common utilities
pub use common::{PrimitiveKind, DimensionClass, GeometryPrimitive, GeometryUnion};