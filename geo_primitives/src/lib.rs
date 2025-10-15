//! 幾何プリミティブクレート
//!
//! 新しいトレイト設計に基づく実装への移行中
//! 旧実装は一時的にコンパイル除外

// 新実装用モジュール（次元中立設計）
// 3D プリミティブ
pub mod arc_3d;
pub mod bbox_3d; // BBox3D の新実装
pub mod circle_3d; // Circle3D の新実装
pub mod ellipse_3d; // Ellipse3D の新実装
pub mod infinite_line_3d; // InfiniteLine3D の新実装
pub mod line_segment_3d; // LineSegment3D の新実装
pub mod point_3d; // Point3D の新実装
pub mod vector_3d; // Vector3D の新実装

// 2D プリミティブ
// Arc2D関連（ジェネリック実装完了により再有効化）
pub mod arc_2d; // Arc2D の新実装 (Core)
pub mod arc_2d_collision; // Arc2D 衝突検出・距離計算Foundation実装
                          // pub mod arc_2d_containment; // Arc2D 包含判定拡張（未対応）
                          // pub mod arc_2d_extensions; // Arc2D の拡張機能 (Extension)（未対応）
                          // pub mod arc_2d_intersection; // Arc2D 交点計算Foundation実装（未対応）
                          // pub mod arc_2d_metrics; // Arc2D 計量演算拡張（未対応）
                          // pub mod arc_2d_sampling; // Arc2D 点列生成拡張（未対応）
                          // pub mod arc_2d_transform; // Arc2D 変換操作拡張（未対応）
pub mod bbox_2d; // BBox2D の新実装 (Core)
pub mod bbox_2d_extensions; // BBox2D の拡張機能 (Extension)
pub mod circle_2d; // Circle2D の新実装 (Core)
pub mod circle_2d_metrics; // Circle2D 計量演算
                           // pub mod circle_2d_extensions; // Circle2D の拡張機能 (Extension) - 一時無効化
pub mod ellipse_2d; // Ellipse2D の実装 (新traitsシステム対応)
pub mod ellipse_arc_2d; // EllipseArc2D の実装 (楕円弧)
                        // pub mod ellipse_2d_extensions; // Ellipse2D の拡張機能 (Extension) - 一時無効化
pub mod infinite_line_2d; // InfiniteLine2D の新実装
pub mod line_segment_2d; // LineSegment2D の新実装 (Core)
pub mod line_segment_2d_extensions; // LineSegment2D の拡張機能 (Extension)
pub mod point_2d; // Point2D の新実装
pub mod point_2d_extensions; // Point2D の拡張機能 (Extension)
pub mod ray_2d; // Ray2D の新実装 (Core)
pub mod ray_2d_extensions; // Ray2D の拡張機能 (Extension)
pub mod vector_2d; // Vector2D の新実装 (Core)
pub mod vector_2d_extensions; // Vector2D の拡張機能 (Extension)

// テストモジュール（次元中立設計）
// 3D テスト
// #[cfg(test)]
// mod arc_3d_tests; // 一時無効化
// #[cfg(test)]
// mod circle_3d_tests; // 一時無効化：古いFoundationトレイト使用
#[cfg(test)]
mod ellipse_3d_tests;
// #[cfg(test)]
// mod infinite_line_3d_tests; // 一時無効化：古いFoundationトレイト使用
// #[cfg(test)]
// mod line_segment_3d_tests; // 一時無効化：古いFoundationトレイト使用
// #[cfg(test)]
// mod point_3d_tests; // 一時無効化：古いFoundationトレイト使用
// #[cfg(test)]
// mod vector_3d_tests; // 一時無効化：古いFoundationトレイト使用

// 2D テスト
// #[cfg(test)]
// mod arc_2d_tests; // 一時無効化：古いFoundationトレイト使用
#[cfg(test)]
mod bbox_2d_tests;
// #[cfg(test)]
// mod circle_2d_tests; // 一時無効化：古いFoundationトレイト使用
// #[cfg(test)]
// mod ellipse_2d_tests; // 一時無効化：古いFoundationトレイト使用
#[cfg(test)]
mod ellipse_arc_new_tests;
#[cfg(test)]
mod ellipse_new_tests; // 新しいEllipse2D実装のテスト // 新しいEllipseArc2D実装のテスト
                       // #[cfg(test)]
                       // mod foundation_2d_tests; // 一時無効化：古いFoundationトレイト使用
                       // #[cfg(test)]
                       // mod infinite_line_2d_tests; // 一時無効化：古いFoundationトレイト使用
                       // #[cfg(test)]
                       // mod line_segment_2d_tests; // 一時無効化：古いFoundationトレイト使用
#[cfg(test)]
mod point_2d_tests;
#[cfg(test)]
mod ray_2d_tests;
#[cfg(test)]
mod vector_2d_tests;

// 最小限の基盤のみ残す
pub use geo_foundation::{Angle, Scalar};

// Foundation システム統一トレイト
pub use geo_foundation::abstract_types::foundation::{
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
pub use ellipse_3d::Ellipse3D;
pub use infinite_line_3d::InfiniteLine3D;
pub use line_segment_3d::LineSegment3D;
pub use point_3d::Point3D;
pub use vector_3d::Vector3D;

// 2D プリミティブ
pub use arc_2d::Arc2D; // ジェネリック実装完了により再有効化
pub use bbox_2d::BBox2D;
pub use circle_2d::Circle2D;
pub use ellipse_2d::Ellipse2D;
pub use ellipse_arc_2d::EllipseArc2D; // 楕円弧
pub use infinite_line_2d::InfiniteLine2D;
pub use line_segment_2d::LineSegment2D;
pub use point_2d::Point2D;
pub use ray_2d::Ray2D;
pub use vector_2d::Vector2D;
