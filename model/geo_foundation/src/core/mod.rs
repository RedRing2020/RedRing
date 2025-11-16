//! 幾何コア抽象化レイヤー - 基本機能の抽象化
//!
//! このモジュールは、幾何的プリミティブの基本的な抽象化を提供し、
//! 型安全性と責務分離を実現します。

// ============================================================================
// New Core Traits (ハイブリッド方針: Core3機能統合 + Transform共通)
// ============================================================================
pub mod point_core_traits;
pub mod transform; // extensionsから移動した共通Transformトレイト群
pub mod transform_error; // extensionsから移動したTransformError(段階的移行中)

// ============================================================================
// Legacy Traits (段階的移行中)
// ============================================================================
pub mod arc_traits;
pub mod circle_core;
pub mod circle_traits;
pub mod direction_traits;
pub mod nurbs_traits;
pub mod plane_traits;
pub mod point_traits;
pub mod triangle_traits;
pub mod vector_traits;
