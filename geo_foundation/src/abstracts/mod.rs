//! Abstract Traits - 最小責務抽象化
//!
//! 幾何プリミティブの最小責務原則に基づく抽象トレイト群
//! 機能は拡張トレイト（Foundation層）で分離し、コア機能のみを提供

// === 基本プリミティブ ===
pub mod direction_traits;
pub mod point_traits; // Point最小責務トレイト
pub mod vector_traits; // Vector最小責務トレイト // Direction最小責務トレイト

// === 線形プリミティブ ===
pub mod infinite_line_traits; // InfiniteLine最小責務トレイト
pub mod line_segment_traits;
pub mod ray_traits; // Ray最小責務トレイト // LineSegment最小責務トレイト

// === 曲線プリミティブ ===
pub mod arc_traits; // Arc最小責務トレイト
pub mod circle_traits; // Circle最小責務トレイト
pub mod ellipse_arc_traits;
pub mod ellipse_traits; // Ellipse最小責務トレイト // EllipseArc最小責務トレイト

// === 補助プリミティブ ===
pub mod bbox_traits; // BBox最小責務トレイト

// === Re-exports ===
pub use arc_traits::*;
pub use bbox_traits::*;
pub use circle_traits::*;
pub use direction_traits::*;
pub use ellipse_arc_traits::*;
pub use ellipse_traits::*;
pub use infinite_line_traits::*;
pub use line_segment_traits::*;
pub use point_traits::*;
pub use ray_traits::*;
pub use vector_traits::*;
