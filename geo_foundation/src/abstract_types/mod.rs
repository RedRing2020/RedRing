//! 抽象型定義モジュール - Foundation統一アーキテクチャ
//!
//! 設計統一アプローチによる構造化された幾何Foundation システム

// Foundation Pattern - 統一基盤システム
pub mod foundation;

// Abstract Traits - 最小責務抽象化
pub mod abstracts;

// Legacy geometry module (段階的移行中)
pub mod geometry;

// Re-exports for convenience (specific to avoid ambiguity)
pub use abstracts::Arc2D;
pub use foundation::{
    AdvancedCollision, AdvancedTransform, BasicCollision, BasicIntersection, BasicTransform,
    MultipleIntersection, PointDistance, SelfIntersection, TransformHelpers,
};
