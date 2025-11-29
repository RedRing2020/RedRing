//! geo_core - 幾何計算基盤クレート
//!
//! Foundation パターンにおける共通幾何型とトレイト実装を提供します。
//! AABB（軸平行境界ボックス）などの基本幾何型を実装。
//!
//! ## 主要機能
//! - **aabb2d/aabb3d**: 軸平行境界ボックス（AABB）実装
//! - **intersections**: 交差判定・交線計算（予定）
//! - **collisions**: 衝突検出・距離計算（予定）
//!
//! ## Foundation パターンでの役割
//! ```text
//! geo_foundation → geo_core → geo_primitives, geo_nurbs
//! ```
//!
//! ---
//! © RedRing Project

// AABB型実装
pub mod aabb_2d;
pub mod aabb_3d;

// 公開API
pub use aabb_2d::Aabb2D;
pub use aabb_3d::Aabb3D;
