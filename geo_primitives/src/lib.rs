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
pub mod arc_2d; // Arc2D の新実装 (Core)
pub mod arc_2d_extensions; // Arc2D の拡張機能 (Extension)
pub mod bbox_2d; // BBox2D の新実装 (Core)
pub mod bbox_2d_extensions; // BBox2D の拡張機能 (Extension)
pub mod circle_2d; // Circle2D の新実装 (Core)
pub mod circle_2d_extensions; // Circle2D の拡張機能 (Extension)
pub mod ellipse_2d; // Ellipse2D の新実装 (Core)
pub mod ellipse_2d_extensions; // Ellipse2D の拡張機能 (Extension)
pub mod infinite_line_2d; // InfiniteLine2D の新実装
pub mod line_segment_2d; // LineSegment2D の新実装 (Core)
pub mod line_segment_2d_extensions; // LineSegment2D の拡張機能 (Extension)
pub mod point_2d; // Point2D の新実装 (Core)
pub mod point_2d_extensions; // Point2D の拡張機能 (Extension)
pub mod vector_2d; // Vector2D の新実装 (Core)
pub mod vector_2d_extensions; // Vector2D の拡張機能 (Extension)

// テストモジュール（次元中立設計）
// 3D テスト
#[cfg(test)]
mod arc_3d_tests;
#[cfg(test)]
mod circle_3d_tests;
#[cfg(test)]
mod ellipse_3d_tests;
#[cfg(test)]
mod infinite_line_3d_tests;
#[cfg(test)]
mod line_segment_3d_tests;
#[cfg(test)]
mod point_3d_tests;
#[cfg(test)]
mod vector_3d_tests;

// 2D テスト
#[cfg(test)]
mod arc_2d_tests;
#[cfg(test)]
mod bbox_2d_tests;
#[cfg(test)]
mod circle_2d_tests;
#[cfg(test)]
mod ellipse_2d_tests;
#[cfg(test)]
mod foundation_2d_tests;
#[cfg(test)]
mod infinite_line_2d_tests;
#[cfg(test)]
mod line_segment_2d_tests;
#[cfg(test)]
mod point_2d_tests;
#[cfg(test)]
mod vector_2d_tests;

// 旧実装（一時除外）
// pub mod geometry2d;  // 旧実装 - 一時除外
// pub mod geometry3d;  // 旧実装 - 一時除外

// 最小限の基盤のみ残す
pub use geo_foundation::{Angle, Scalar};

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
pub use arc_2d::Arc2D;
pub use bbox_2d::BBox2D;
pub use circle_2d::Circle2D;
pub use ellipse_2d::Ellipse2D;
pub use infinite_line_2d::InfiniteLine2D;
pub use line_segment_2d::LineSegment2D;
pub use point_2d::Point2D;
pub use vector_2d::Vector2D;
