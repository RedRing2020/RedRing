//! geo_core - 幾何計算基盤クレート
//!
//! geo_primitives と geo_nurbs が共通利用する低レベル実装を提供します。
//! AABB（軸平行境界ボックス）などの基本幾何型を実装。
//!
//! ## 主要機能
//! - **aabb2d/aabb3d**: 軸平行境界ボックス（AABB）実装
//!
//! ## レイヤーでの役割
//! ```text
//! analysis → geo_core → geo_primitives, geo_nurbs
//! ```
//!
//! geo_core は低レベル共通実装を提供
//!
//! ---
//! © RedRing Project

mod aabb_2d;
mod aabb_3d;
mod direction_2d;
mod direction_3d;
mod point_2d;
mod point_3d;
mod similarity_transform_2d;
mod similarity_transform_3d;
mod vector_2d;
mod vector_3d;

pub use aabb_2d::Aabb2D;
pub use aabb_3d::Aabb3D;
pub use direction_2d::Direction2D;
pub use direction_3d::Direction3D;
pub use geo_contracts::TransformError;
pub use point_2d::Point2D;
pub use point_3d::Point3D;
pub use similarity_transform_2d::SimilarityTransform2D;
pub use similarity_transform_3d::SimilarityTransform3D;
pub use vector_2d::Vector2D;
pub use vector_3d::Vector3D;
