//! geo_core - 幾何計算基盤クレート
//!
//! geo_primitives と geo_nurbs が共通利用する低レベル実装を提供します。
//! AABB（軸平行境界ボックス）などの基本幾何型を実装。
//!
//! ## 主要機能
//! - **aabb2d/aabb3d**: 軸平行境界ボックス（AABB）実装
//!
//! ## Foundation パターンでの役割
//! ```text
//! geo_foundation → geo_commons → geo_core → geo_primitives, geo_nurbs
//! ```
//!
//! geo_core は低レベル共通実装、geo_commons は形状横断的な高レベル機能を提供
//!
//! ---
//! © RedRing Project

// AABB型実装
pub mod aabb_2d;
pub mod aabb_3d;

// 公開API
pub use aabb_2d::Aabb2D;
pub use aabb_3d::Aabb3D;
