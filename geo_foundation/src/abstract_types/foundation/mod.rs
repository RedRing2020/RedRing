//! Foundation Pattern - Foundation統一システム
//!
//! 全幾何プリミティブで共通利用可能な Foundation パターン実装
//! Core Foundation と Extension Foundation の統合システム

// Core Foundation Systems (統一基盤システム)
pub mod collision; // Collision Foundation統一システム
pub mod core_foundation; // Core Foundation統一システム（新規追加）
pub mod intersection;
pub mod transform; // Transform Foundation統一システム // Intersection Foundation統一システム

// Geometry-specific Foundation (幾何専用Foundation) - 段階的移行中
pub mod arc_core; // Arc Core Foundation（円弧専用）
pub mod arc_extensions; // Arc Extension Foundation
pub mod circle_core; // Circle Core Foundation（円専用）
pub mod ellipse_arc_core; // EllipseArc Core Foundation（楕円弧専用）
pub mod point_extensions; // Point Extension Foundation

// Re-exports for convenience
pub use arc_core::*; // Arc Core Foundation
pub use arc_extensions::*;
pub use circle_core::*; // Circle Core Foundation
pub use collision::*;
pub use core_foundation::*; // Core Foundation追加
pub use ellipse_arc_core::*; // EllipseArc Core Foundation
pub use intersection::*;
pub use point_extensions::*;
pub use transform::*;
