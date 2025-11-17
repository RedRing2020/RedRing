//! 幾何コア抽象化レイヤー - 基本機能の抽象化
//!
//! このモジュールは、幾何的プリミティブの基本的な抽象化を提供し、
//! 型安全性と責務分離を実現します。

// ============================================================================
// New Core Traits (ハイブリッド方針: Core3機能統合 + Transform共通)
// ============================================================================
pub mod bbox_core_traits; // BBox Core traits (Constructor/Properties/Measure)
pub mod circle_core_traits; // Circle Core traits (Constructor/Properties/Measure)
pub mod direction_core_traits; // Direction Core traits (Constructor/Properties/Measure)
pub mod infinite_line_core_traits; // InfiniteLine Core traits (Constructor/Properties/Measure)
pub mod linesegment_core_traits; // LineSegment Core traits (Constructor/Properties/Measure)
pub mod point_core_traits;
pub mod ray_core_traits; // Ray Core traits (Constructor/Properties/Measure)
pub mod transform; // extensionsから移動した共通Transformトレイト群
pub mod transform_error; // extensionsから移動したTransformError(段階的移行中)
pub mod vector_core_traits; // Vector Core traits (Constructor/Properties/Measure)

// ============================================================================
// Legacy Traits (段階的移行中)
// ============================================================================
pub mod arc_traits;
pub mod circle_traits;
pub mod direction_traits;
pub mod nurbs_traits;
pub mod plane_traits;
pub mod point_traits;
pub mod triangle_traits;
pub mod vector_traits;
