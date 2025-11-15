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
pub mod arc_3d_extensions; // Arc3D の拡張機能 (Extension)
pub mod arc_3d_foundation; // Arc3D のFoundation実装
pub mod arc_3d_transform; // Arc3D の変換機能 (Transform)
pub mod arc_3d_transform_safe; // Arc3D の安全な変換機能 (Safe Transform)
pub mod bbox_3d; // BBox3D の新実装
pub mod bbox_3d_extensions; // BBox3D の拡張機能 (Extension)
pub mod bbox_3d_foundation; // BBox3D のFoundation実装
pub mod bbox_3d_transform; // BBox3D の変換機能 (Transform)
pub mod bbox_3d_transform_safe; // BBox3D の安全な変換機能 (Safe Transform)
pub mod circle_3d; // Circle3D の新実装
pub mod circle_3d_analysis_transform; // Circle3D のAnalysis Transform実装
pub mod circle_3d_extensions; // Circle3D の拡張機能 (Extension)
pub mod circle_3d_foundation; // Circle3D のFoundation実装
pub mod circle_3d_tests; // Circle3D のテスト
pub mod circle_3d_transform; // Circle3D の変換機能 (Transform)
pub mod circle_3d_transform_safe; // Circle3D の安全な変換機能 (Safe Transform)
pub mod conical_solid_3d; // ConicalSolid3D の新実装 (Core) - 完全ハイブリッドモデラー対応
pub mod conical_solid_3d_extensions; // ConicalSolid3D の拡張機能 (Extension)
pub mod conical_solid_3d_foundation; // ConicalSolid3D のFoundation実装
#[cfg(test)]
pub mod conical_solid_3d_tests; // ConicalSolid3D のテスト
pub mod conical_solid_3d_transform; // ConicalSolid3D の変換機能 (Transform)
pub mod conical_solid_3d_transform_safe; // ConicalSolid3D の安全な変換機能 (SafeTransform)
#[cfg(test)]
pub mod conical_solid_3d_transform_safe_tests; // ConicalSolid3D の安全な変換機能のテスト
pub mod conical_surface_3d; // ConicalSurface3D の新実装 (Core) - 完全ハイブリッドモデラー対応
pub mod conical_surface_3d_extensions; // ConicalSurface3D の拡張機能 (Extension)
pub mod conical_surface_3d_foundation; // ConicalSurface3D のFoundation実装
#[cfg(test)]
pub mod conical_surface_3d_tests; // ConicalSurface3D のテスト
pub mod conical_surface_3d_transform; // ConicalSurface3D の変換機能 (Transform)
pub mod conical_surface_3d_transform_safe; // ConicalSurface3D の安全な変換機能 (SafeTransform)
#[cfg(test)]
pub mod conical_surface_3d_transform_safe_tests; // ConicalSurface3D の安全な変換機能のテスト
pub mod cylindrical_solid_3d; // CylindricalSolid3D の新実装 (Core) - 完全ハイブリッドモデラー対応
pub mod cylindrical_solid_3d_extensions; // CylindricalSolid3D の拡張機能 (Extension)
pub mod cylindrical_solid_3d_foundation; // CylindricalSolid3D のFoundation実装
#[cfg(test)]
pub mod cylindrical_solid_3d_tests; // CylindricalSolid3D のテスト
pub mod cylindrical_solid_3d_transform; // CylindricalSolid3D の変換機能 (Transform)
pub mod cylindrical_solid_3d_transform_safe; // CylindricalSolid3D の安全な変換機能 (Safe Transform)
#[cfg(test)]
pub mod cylindrical_solid_3d_transform_tests; // CylindricalSolid3D の変換テスト
pub mod cylindrical_surface_3d; // CylindricalSurface3D の新実装 (Core) - 完全ハイブリッドモデラー対応
pub mod cylindrical_surface_3d_extensions; // CylindricalSurface3D の拡張機能 (Extension)
pub mod cylindrical_surface_3d_foundation; // CylindricalSurface3D のFoundation実装
#[cfg(test)]
pub mod cylindrical_surface_3d_tests; // CylindricalSurface3D のテスト
pub mod cylindrical_surface_3d_transform; // CylindricalSurface3D の変換機能 (Transform)
#[cfg(test)]
pub mod cylindrical_surface_3d_transform_tests; // CylindricalSurface3D の変換テスト
pub mod direction_3d; // Direction3D の新実装 (Core)
pub mod direction_3d_extensions;
pub mod direction_3d_transform_safe; // Direction3D の安全な変換機能 (Safe Transform)
pub mod ellipse_3d; // Ellipse3D の新実装 (Core)
pub mod ellipse_3d_analysis_transform; // Ellipse3D の Analysis Matrix4x4 統合変換
pub mod ellipse_3d_extensions; // Ellipse3D の拡張機能 (Extension)
pub mod ellipse_3d_transform; // Ellipse3D の変換機能 (Transform)
pub mod ellipse_3d_transform_safe; // Ellipse3D の安全な変換機能 (Safe Transform)
pub mod ellipse_arc_3d; // EllipseArc3D の実装 (Core)
pub mod ellipse_arc_3d_analysis_transform; // EllipseArc3D のAnalysis Transform実装
pub mod ellipse_arc_3d_extensions; // EllipseArc3D の拡張機能 (Extension)
pub mod ellipse_arc_3d_tests; // EllipseArc3D のテスト
pub mod ellipse_arc_3d_transform; // EllipseArc3D の変換機能 (Transform)
pub mod ellipse_arc_3d_transform_safe; // EllipseArc3D の安全な変換機能 (Safe Transform)
pub mod infinite_line_3d; // InfiniteLine3D の新実装
pub mod infinite_line_3d_analysis_transform; // InfiniteLine3D の Analysis Matrix/Vector 統合変換
pub mod infinite_line_3d_extensions; // InfiniteLine3D の拡張機能 (Extension)
pub mod infinite_line_3d_transform; // InfiniteLine3D の変換機能 (Transform)
pub mod infinite_line_3d_transform_safe; // InfiniteLine3D の安全な変換機能 (Safe Transform)
pub mod line_segment_3d; // LineSegment3D の新実装 (Core)
pub mod line_segment_3d_analysis_transform; // LineSegment3D の Analysis Matrix/Vector 統合変換
pub mod line_segment_3d_extensions; // LineSegment3D の拡張機能 (Extension)
pub mod line_segment_3d_transform; // LineSegment3D の変換機能 (Transform)
pub mod line_segment_3d_transform_safe; // LineSegment3D の安全な変換機能 (Safe Transform)
pub mod plane_3d; // Plane3D の新実装 (Core)
pub mod plane_3d_analysis_transform; // Plane3D の Analysis Matrix4x4 統合変換
pub mod plane_3d_extensions; // Plane3D の拡張機能 (Extension)
pub mod plane_3d_foundation; // Plane3D のFoundation実装
                             // pub mod plane_3d_intersection; // 一時的にコメントアウト（機能過多）
#[cfg(test)]
pub mod plane_3d_tests; // Plane3D のテスト
pub mod plane_3d_transform; // Plane3D の変換機能 (Transform)
pub mod plane_3d_transform_safe; // Plane3D の安全な変換機能 (Safe Transform)
#[cfg(test)]
pub mod plane_3d_transform_safe_tests; // Plane3D の安全な変換テスト
pub mod plane_coordinate_system; // STEP準拠の座標系付き平面（Core）
pub mod plane_coordinate_system_foundation; // Plane3DCoordinateSystem のFoundation実装
#[cfg(test)]
pub mod plane_coordinate_system_tests; // Plane3DCoordinateSystem のテスト
pub mod plane_coordinate_system_transform; // Plane3DCoordinateSystem の変換機能 (Transform)
#[cfg(test)]
pub mod plane_coordinate_system_transform_tests; // Plane3DCoordinateSystem の変換テスト
pub mod point_3d; // Point3D の新実装 (Core)
pub mod point_3d_analysis_transform; // Point3D の Analysis Matrix/Vector 統合変換
pub mod point_3d_extensions; // Point3D の拡張機能 (Extension)
pub mod point_3d_foundation; // Point3D のFoundation実装
pub mod point_3d_transform; // Point3D の変換機能 (Transform)
pub mod point_3d_transform_safe; // Point3D の安全な変換機能 (Safe Transform)
pub mod ray_3d; // Ray3D の新実装 (Core)
pub mod ray_3d_analysis_transform; // Ray3D の Analysis Matrix4x4 統合変換
pub mod ray_3d_extensions; // Ray3D の拡張機能 (Extension)
pub mod ray_3d_foundation; // Ray3D のFoundation実装
pub mod ray_3d_transform; // Ray3D の変換機能 (Transform)
pub mod ray_3d_transform_safe; // Ray3D の安全な変換機能 (Safe Transform)
pub mod spherical_solid_3d; // SphericalSolid3D の新実装 (Core) - 完全ハイブリッドモデラー対応
pub mod spherical_solid_3d_foundation; // SphericalSolid3D のFoundation実装
pub mod spherical_solid_3d_transform; // SphericalSolid3D の変換機能 (Transform)
pub mod spherical_solid_3d_transform_safe; // SphericalSolid3D の安全な変換機能 (Safe Transform)
pub mod spherical_surface_3d; // SphericalSurface3D の新実装 (Core) - 完全ハイブリッドモデラー対応
pub mod spherical_surface_3d_foundation; // SphericalSurface3D のFoundation実装
pub mod spherical_surface_3d_transform; // SphericalSurface3D の変換機能 (Transform)
pub mod spherical_surface_3d_transform_safe; // SphericalSurface3D の安全な変換機能 (Safe Transform)
pub mod torus_solid_3d; // TorusSolid3D の新実装 (Core) - 3D CAM 固体加工対応
pub mod torus_solid_3d_extensions; // TorusSolid3D の拡張機能 (Extension)
pub mod torus_solid_3d_foundation; // TorusSolid3D のFoundation実装
pub mod torus_solid_3d_transform; // TorusSolid3D の変換機能 (Transform)
pub mod torus_solid_3d_transform_safe; // TorusSolid3D の安全な変換機能 (Safe Transform)
pub mod torus_surface_3d; // TorusSurface3D の新実装 (Core) - 3D CAM 工具オフセット対応
pub mod torus_surface_3d_extensions; // TorusSurface3D の拡張機能 (Extension)
pub mod torus_surface_3d_foundation; // TorusSurface3D のFoundation実装
pub mod torus_surface_3d_transform; // TorusSurface3D の変換機能 (Transform)
pub mod torus_surface_3d_transform_safe; // TorusSurface3D の安全な変換機能 (Safe Transform)
pub mod triangle_3d; // Triangle3D の新実装 (Core)
pub mod triangle_3d_analysis_transform; // Triangle3D のAnalysis Transform実装
pub mod triangle_3d_foundation; // Triangle3D のFoundation実装
pub mod triangle_3d_transform; // Triangle3D の変換機能 (Transform)
pub mod triangle_mesh_3d; // TriangleMesh3D の新実装 (Core)
pub mod triangle_mesh_3d_foundation; // TriangleMesh3D のFoundation実装

// Vector3D関連（Core, Extension, Transform, Safe Transform, Analysis）
pub mod vector_3d; // Vector3D の新実装
pub mod vector_3d_analysis_transform; // Vector3D の Analysis Matrix/Vector 統合変換
#[cfg(test)]
pub mod vector_3d_analysis_transform_tests; // Vector3D Analysis変換のテストスイート
pub mod vector_3d_extensions; // Vector3D の拡張機能 (Extension)
pub mod vector_3d_foundation; // Vector3D のFoundation実装
pub mod vector_3d_transform; // Vector3D の変換機能 (Transform)
pub mod vector_3d_transform_safe; // Vector3D の安全な変換機能 (Safe Transform)

// Transform テストモジュール
#[cfg(test)]
pub mod ellipse_3d_transform_safe_tests;
#[cfg(test)]
pub mod ellipse_3d_transform_tests;
#[cfg(test)]
pub mod point_3d_transform_safe_tests;
#[cfg(test)]
pub mod ray_3d_tests;
#[cfg(test)]
pub mod torus_solid_3d_tests;
#[cfg(test)]
pub mod torus_solid_3d_transform_safe_tests;
#[cfg(test)]
pub mod torus_surface_3d_tests;
#[cfg(test)]
pub mod torus_surface_3d_transform_safe_tests;
#[cfg(test)]
pub mod triangle_3d_tests;
#[cfg(test)]
pub mod triangle_mesh_3d_tests;

// Vector3D テストモジュール
#[cfg(test)]
pub mod vector_3d_transform_safe_tests;

// 2D プリミティブ
// Arc2D関連（ジェネリック実装完了により再有効化）
pub mod arc_2d; // Arc2D の新実装 (Core)
pub mod arc_2d_collision; // Arc2D 衝突検出・距離計算Foundation実装
                          // pub mod arc_2d_transform; // Arc2D の変換機能 (Transform) - 一時無効化
pub mod arc_2d_transform_safe; // Arc2D の安全な変換機能 (Safe Transform)
pub mod bbox_2d; // BBox2D の新実装 (Core)
pub mod bbox_2d_extensions; // BBox2D の拡張機能 (Extension)
pub mod bbox_2d_transform; // BBox2D の変換機能 (Transform)
pub mod bbox_2d_transform_safe; // BBox2D の安全な変換機能 (Safe Transform)
pub mod circle_2d; // Circle2D の新実装 (Core)
pub mod circle_2d_analysis_transform; // Circle2D の Analysis Matrix/Vector 統合変換
pub mod circle_2d_metrics; // Circle2D 計量演算
pub mod circle_2d_transform; // Circle2D の変換機能 (Transform)
pub mod circle_2d_transform_safe; // Circle2D の安全な変換機能 (Safe Transform)
pub mod direction_2d; // Direction2D の新実装 (Core)
pub mod direction_2d_extensions;
pub mod direction_2d_transform_safe; // Direction2D の安全な変換機能 (Safe Transform)
pub mod ellipse_2d; // Ellipse2D の実装 (新traitsシステム対応)
pub mod ellipse_2d_analysis_transform; // Ellipse2D の Analysis Matrix3x3 統合変換
pub mod ellipse_2d_transform; // Ellipse2D の変換機能 (Transform)
pub mod ellipse_2d_transform_safe; // Ellipse2D の安全な変換機能 (Safe Transform)
pub mod ellipse_arc_2d; // EllipseArc2D の実装 (Core)
pub mod ellipse_arc_2d_analysis_transform; // EllipseArc2D のAnalysis Transform実装
pub mod ellipse_arc_2d_extensions; // EllipseArc2D の拡張機能 (Extension)
pub mod ellipse_arc_2d_transform; // EllipseArc2D の変換機能 (Transform)
pub mod ellipse_arc_2d_transform_safe; // EllipseArc2D の安全な変換機能 (Safe Transform)
pub mod infinite_line_2d; // InfiniteLine2D の新実装
pub mod infinite_line_2d_analysis_transform; // InfiniteLine2D の Analysis Matrix/Vector 統合変換
pub mod infinite_line_2d_extensions; // InfiniteLine2D の拡張機能 (Extension)
pub mod infinite_line_2d_transform; // InfiniteLine2D の変換機能 (Transform)
pub mod infinite_line_2d_transform_safe; // InfiniteLine2D の安全な変換機能 (Safe Transform)
pub mod line_segment_2d; // LineSegment2D の新実装 (Core)
pub mod line_segment_2d_analysis_transform; // LineSegment2D の Analysis Matrix/Vector 統合変換
pub mod line_segment_2d_extensions; // LineSegment2D の拡張機能 (Extension)
pub mod line_segment_2d_transform; // LineSegment2D の変換機能 (Transform)
pub mod line_segment_2d_transform_safe; // LineSegment2D の安全な変換機能 (Safe Transform)
pub mod point_2d; // Point2D の新実装
pub mod point_2d_analysis_transform; // Point2D の Analysis Matrix3x3 統合変換
pub mod point_2d_extensions; // Point2D の拡張機能 (Extension)
pub mod point_2d_transform; // Point2D の変換機能 (Transform)
pub mod point_2d_transform_safe; // Point2D の安全な変換機能 (Safe Transform)
pub mod ray_2d; // Ray2D の新実装 (Core)
pub mod ray_2d_analysis_transform; // Ray2D の Analysis Matrix3x3 統合変換
pub mod ray_2d_extensions; // Ray2D の拡張機能 (Extension)
pub mod ray_2d_transform; // Ray2D の変換機能 (Transform)
pub mod ray_2d_transform_safe; // Ray2D の安全な変換機能 (Safe Transform)
pub mod triangle_2d; // Triangle2D の新実装 (Core)
pub mod triangle_2d_analysis_transform; // Triangle2D のAnalysis Transform実装
pub mod triangle_2d_transform; // Triangle2D の変換機能 (Transform)

// Vector2D関連（Core, Extension, Transform, Safe Transform）
pub mod vector_2d; // Vector2D の新実装 (Core)
pub mod vector_2d_analysis_transform; // Vector2D の Analysis Matrix3x3 統合変換
pub mod vector_2d_extensions; // Vector2D の拡張機能 (Extension)
pub mod vector_2d_transform; // Vector2D の変換機能 (Transform)
pub mod vector_2d_transform_safe; // Vector2D の安全な変換機能 (Safe Transform)

// テストモジュール（次元中立設計）
// 3D テスト
#[cfg(test)]
mod bbox_3d_tests;
#[cfg(test)]
mod ellipse_3d_tests;
#[cfg(test)]
mod point_3d_tests;
#[cfg(test)]
mod spherical_solid_3d_tests;
#[cfg(test)]
mod spherical_solid_3d_transform_safe_tests;
#[cfg(test)]
mod spherical_surface_3d_tests;
#[cfg(test)]
mod spherical_surface_3d_transform_safe_tests;
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

// Vector2D テストモジュール
#[cfg(test)]
pub mod vector_2d_transform_safe_tests;

// 最小限の基盤のみ残す
pub use geo_foundation::{Angle, Scalar};

// Foundation システム統一トレイト
pub use geo_foundation::extensions::{
    AdvancedCollision,
    AdvancedTransform,
    BBoxCollision,
    // Collision Foundation
    BasicCollision,
    // Intersection Foundation
    BasicIntersection,
    // Transform Foundation
    BasicTransform,
    MultipleIntersection,
    PointDistance,
    SelfIntersection,
};

// 新実装の公開（次元中立設計）
// 3D プリミティブ
pub use arc_3d::Arc3D;
pub use bbox_3d::BBox3D;
pub use circle_3d::Circle3D;
pub use conical_solid_3d::{Cone3D, ConicalSolid3D}; // 新式円錐ソリッド + 互換エイリアス
pub use conical_surface_3d::{ConeRim3D, ConicalSurface3D}; // 新式円錐サーフェス + 互換エイリアス
pub use cylindrical_solid_3d::CylindricalSolid3D; // 新式ソリッド
pub use cylindrical_solid_3d_transform_safe::{
    CylindricalSolid3DTransformError, SafeTransform, TransformResult,
}; // 安全な変換操作とエラー型
pub use cylindrical_surface_3d::CylindricalSurface3D; // 新式サーフェス
pub use direction_3d::Direction3D;
pub use ellipse_3d::Ellipse3D;
pub use ellipse_arc_3d::EllipseArc3D;
pub use infinite_line_3d::InfiniteLine3D;
pub use line_segment_3d::LineSegment3D;
pub use plane_3d::Plane3D;
pub use plane_coordinate_system::Plane3DCoordinateSystem;
pub use point_3d::Point3D;
pub use ray_3d::Ray3D;
pub use spherical_solid_3d::SphericalSolid3D; // 新式球ソリッド
pub use spherical_surface_3d::SphericalSurface3D; // 新式球サーフェス
pub use torus_solid_3d::TorusSolid3D; // 新式トーラスソリッド (3D CAM対応)
pub use torus_surface_3d::TorusSurface3D; // 新式トーラスサーフェス (3D CAM対応)
pub use triangle_3d::Triangle3D;
pub use triangle_mesh_3d::TriangleMesh3D;
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
pub use triangle_2d::Triangle2D;
pub use vector_2d::Vector2D;

// ============================================================================
// Test Modules
// ============================================================================
