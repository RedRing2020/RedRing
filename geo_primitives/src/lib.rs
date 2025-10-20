//! 幾何プリミティブクレート
//!
//! 新しいトレイト設計に基づく実装への移行中
//! 旧実装は一時的にコンパイル除外

// 新実装用モジュール（次元中立設計）
// 共通型とエラー
// エラー型はgeo_foundationから公開されたものを使用
pub use geo_foundation::TransformError;

// 3D プリミティブ
pub mod arc_3d;
pub mod arc_3d_transform; // Arc3D の変換機能 (Transform)
pub mod bbox_3d; // BBox3D の新実装
pub mod bbox_3d_extensions; // BBox3D の拡張機能 (Extension)
pub mod bbox_3d_transform; // BBox3D の変換機能 (Transform)
pub mod circle_3d; // Circle3D の新実装
pub mod direction_3d; // Direction3D の新実装 (Core)
pub mod direction_3d_extensions;
pub mod ellipse_3d; // Ellipse3D の新実装 (Core)
pub mod ellipse_3d_extensions; // Ellipse3D の拡張機能 (Extension)
pub mod ellipse_3d_transform; // Ellipse3D の変換機能 (Transform)
pub mod ellipse_3d_transform_safe; // Ellipse3D の安全な変換機能 (Safe Transform)
pub mod infinite_line_3d; // InfiniteLine3D の新実装
pub mod infinite_line_3d_extensions; // InfiniteLine3D の拡張機能 (Extension)
pub mod infinite_line_3d_transform; // InfiniteLine3D の変換機能 (Transform)
pub mod line_segment_3d; // LineSegment3D の新実装 (Core)
pub mod line_segment_3d_extensions; // LineSegment3D の拡張機能 (Extension)
pub mod line_segment_3d_transform; // LineSegment3D の変換機能 (Transform)
pub mod point_3d; // Point3D の新実装 (Core)
pub mod point_3d_extensions; // Point3D の拡張機能 (Extension)
pub mod point_3d_transform; // Point3D の変換機能 (Transform)
pub mod point_3d_transform_safe; // Point3D の安全な変換機能 (Safe Transform)
pub mod ray_3d; // Ray3D の新実装 (Core)
pub mod ray_3d_extensions; // Ray3D の拡張機能 (Extension)
pub mod ray_3d_transform; // Ray3D の変換機能 (Transform)
pub mod vector_3d; // Vector3D の新実装
pub mod vector_3d_extensions; // Vector3D の拡張機能 (Extension)
pub mod vector_3d_transform; // Vector3D の変換機能 (Transform)

// Transform テストモジュール
#[cfg(test)]
pub mod ellipse_3d_transform_safe_tests;
#[cfg(test)]
pub mod ellipse_3d_transform_tests;
#[cfg(test)]
pub mod point_3d_transform_safe_tests;
#[cfg(test)]
pub mod ray_3d_tests;

// 2D プリミティブ
// Arc2D関連（ジェネリック実装完了により再有効化）
pub mod arc_2d; // Arc2D の新実装 (Core)
pub mod arc_2d_collision; // Arc2D 衝突検出・距離計算Foundation実装
pub mod bbox_2d; // BBox2D の新実装 (Core)
pub mod bbox_2d_extensions; // BBox2D の拡張機能 (Extension)
pub mod bbox_2d_transform; // BBox2D の変換機能 (Transform)
pub mod circle_2d; // Circle2D の新実装 (Core)
pub mod circle_2d_metrics; // Circle2D 計量演算
pub mod circle_2d_transform; // Circle2D の変換機能 (Transform)
pub mod direction_2d; // Direction2D の新実装 (Core)
pub mod direction_2d_extensions;
pub mod ellipse_2d; // Ellipse2D の実装 (新traitsシステム対応)
pub mod ellipse_2d_transform; // Ellipse2D の変換機能 (Transform)
pub mod ellipse_arc_2d; // EllipseArc2D の実装 (Core)
pub mod ellipse_arc_2d_extensions; // EllipseArc2D の拡張機能 (Extension)
pub mod ellipse_arc_2d_transform; // EllipseArc2D の変換機能 (Transform)
pub mod infinite_line_2d; // InfiniteLine2D の新実装
pub mod infinite_line_2d_extensions; // InfiniteLine2D の拡張機能 (Extension)
pub mod infinite_line_2d_transform; // InfiniteLine2D の変換機能 (Transform)
pub mod line_segment_2d; // LineSegment2D の新実装 (Core)
pub mod line_segment_2d_extensions; // LineSegment2D の拡張機能 (Extension)
pub mod line_segment_2d_transform; // LineSegment2D の変換機能 (Transform)
pub mod point_2d; // Point2D の新実装
pub mod point_2d_extensions; // Point2D の拡張機能 (Extension)
pub mod point_2d_transform; // Point2D の変換機能 (Transform)
pub mod point_2d_transform_safe; // Point2D の安全な変換機能 (Safe Transform)
pub mod ray_2d; // Ray2D の新実装 (Core)
pub mod ray_2d_extensions; // Ray2D の拡張機能 (Extension)
pub mod ray_2d_transform; // Ray2D の変換機能 (Transform)
pub mod vector_2d; // Vector2D の新実装 (Core)
pub mod vector_2d_extensions; // Vector2D の拡張機能 (Extension)
pub mod vector_2d_transform; // Vector2D の変換機能 (Transform)

// テストモジュール（次元中立設計）
// 3D テスト
#[cfg(test)]
mod bbox_3d_tests;
#[cfg(test)]
mod ellipse_3d_tests;
#[cfg(test)]
mod point_3d_tests;
#[cfg(test)]
mod vector_3d_tests;

// 2D テスト
#[cfg(test)]
mod bbox_2d_tests;
#[cfg(test)]
mod direction_2d_extensions_tests;
#[cfg(test)]
mod direction_3d_extensions_tests;
#[cfg(test)]
mod ellipse_arc_2d_tests;
#[cfg(test)]
mod foundation_tests;
#[cfg(test)]
mod infinite_line_2d_tests;
#[cfg(test)]
mod infinite_line_3d_tests;
#[cfg(test)]
mod point_2d_tests;
#[cfg(test)]
pub mod point_2d_transform_safe_tests;
#[cfg(test)]
mod ray_2d_tests;
#[cfg(test)]
mod vector_2d_tests; // Foundation traitの動作確認テスト

// 最小限の基盤のみ残す
pub use geo_foundation::{Angle, Scalar};

// Foundation システム統一トレイト
pub use geo_foundation::extensions::{
    AdvancedCollision,
    AdvancedTransform,
    // Collision Foundation
    BasicCollision,
    // Intersection Foundation
    BasicIntersection,
    // Transform Foundation
    BasicTransform,
    BoundingBoxCollision,
    CollisionHelpers,
    IntersectionHelpers,
    MultipleIntersection,
    PointDistance,
    PointDistanceHelpers,
    SelfIntersection,
    TransformHelpers,
};

// 新実装の公開（次元中立設計）
// 3D プリミティブ
pub use arc_3d::Arc3D;
pub use bbox_3d::BBox3D;
pub use circle_3d::Circle3D;
pub use direction_3d::Direction3D;
pub use ellipse_3d::Ellipse3D;
pub use infinite_line_3d::InfiniteLine3D;
pub use line_segment_3d::LineSegment3D;
pub use point_3d::Point3D;
pub use ray_3d::Ray3D;
pub use vector_3d::Vector3D;

// 2D プリミティブ
pub use arc_2d::Arc2D; // ジェネリック実装完了により再有効化
pub use bbox_2d::BBox2D;
pub use circle_2d::Circle2D;
pub use direction_2d::Direction2D;
pub use ellipse_2d::Ellipse2D;
pub use ellipse_arc_2d::EllipseArc2D; // 楕円弧
pub use infinite_line_2d::InfiniteLine2D;
pub use line_segment_2d::LineSegment2D;
pub use point_2d::Point2D;
pub use ray_2d::Ray2D;
pub use vector_2d::Vector2D;

// ============================================================================
// Test Modules
// ============================================================================
