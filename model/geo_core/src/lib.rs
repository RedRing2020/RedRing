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

// 基本型実装
pub mod point_2d;
pub mod point_traits;
pub mod point_2d_transform;
pub mod point_3d;
pub mod point_3d_transform;
pub mod transform_error;
pub mod transform_traits;
pub mod vector_2d;
pub mod vector_2d_transform;
pub mod vector_3d;
pub mod vector_3d_transform;
pub mod vector_traits;

// テストモジュール
#[cfg(test)]
mod point_2d_tests;
#[cfg(test)]
mod point_3d_tests;
#[cfg(test)]
mod vector_2d_tests;
#[cfg(test)]
mod vector_3d_tests;

// AABB型実装
pub mod aabb_traits;
pub mod aabb_2d;
pub mod aabb_3d;

// 公開API
pub use aabb_2d::Aabb2D;
pub use aabb_3d::Aabb3D;
pub use point_2d::Point2D;
pub use point_3d::Point3D;
pub use transform_error::{SafeTransform, TransformError};
pub use transform_traits::{AnalysisTransform2D, AnalysisTransform3D, AnalysisTransformSupport};
pub use vector_2d::Vector2D;
pub use vector_3d::{DominantAxis, Vector3D, VectorRelationship};
