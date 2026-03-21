//! 幾何プリミティブクレート
//!
//! 新しいトレイト設計に基づく実装への移行中
//! 旧実装は一時的にコンパイル除外

// 新実装用モジュール（次元中立設計）
// 共通型とエラー
// Transform trait とエラー型は geo_core を正規参照先とし、このクレートでも再公開する
pub use geo_core::{
    AnalysisTransform2D, AnalysisTransform3D, AnalysisTransformSupport, TransformError,
};

// 3D プリミティブ
pub mod arc_3d;
mod arc_3d_collision; // Arc3D の衝突判定
pub mod arc_3d_extensions; // Arc3D の拡張機能 (Extension)
pub mod arc_3d_foundation; // Arc3D のFoundation実装
mod arc_3d_intersection; // Arc3D の交差計算
pub mod circle_3d; // Circle3D の新実装
mod circle_3d_collision; // Circle3D の衝突判定
pub mod circle_3d_extensions; // Circle3D の拡張機能 (Extension)
pub mod circle_3d_foundation; // Circle3D のFoundation実装
mod circle_3d_intersection; // Circle3D の交差計算
pub mod circle_3d_tests; // Circle3D のテスト
pub mod conical_solid_3d; // ConicalSolid3D の新実装 (Core) - 完全ハイブリッドモデラー対応
pub mod conical_solid_3d_extensions; // ConicalSolid3D の拡張機能 (Extension)
pub mod conical_solid_3d_foundation; // ConicalSolid3D のFoundation実装
                                     // #[cfg(test)]
                                     // pub mod conical_solid_3d_tests; // ConicalSolid3D のテスト - 未実装Transform機能のため無効化
pub mod conical_surface_3d; // ConicalSurface3D の新実装 (Core) - 完全ハイブリッドモデラー対応
pub mod conical_surface_3d_extensions; // ConicalSurface3D の拡張機能 (Extension)
pub mod conical_surface_3d_foundation; // ConicalSurface3D のFoundation実装
                                       // #[cfg(test)]
                                       // pub mod conical_surface_3d_tests; // ConicalSurface3D のテスト - 未実装Transform機能のため無効化
pub mod cylindrical_solid_3d; // CylindricalSolid3D の新実装 (Core) - 完全ハイブリッドモデラー対応
mod cylindrical_solid_3d_collision; // CylindricalSolid3D の衝突判定
pub mod cylindrical_solid_3d_extensions; // CylindricalSolid3D の拡張機能 (Extension)
pub mod cylindrical_solid_3d_foundation; // CylindricalSolid3D のFoundation実装
#[cfg(test)]
pub mod cylindrical_solid_3d_tests; // CylindricalSolid3D のテスト
pub mod cylindrical_surface_3d; // CylindricalSurface3D の新実装 (Core) - 完全ハイブリッドモデラー対応
mod cylindrical_surface_3d_collision; // CylindricalSurface3D の衝突判定・高度衝突判定
#[cfg(test)]
pub mod cylindrical_surface_3d_collision_tests; // CylindricalSurface3D の衝突判定テスト
pub mod cylindrical_surface_3d_extensions; // CylindricalSurface3D の拡張機能 (Extension)
pub mod cylindrical_surface_3d_foundation; // CylindricalSurface3D のFoundation実装
mod cylindrical_surface_3d_intersection; // CylindricalSurface3D の交差計算
#[cfg(test)]
pub mod cylindrical_surface_3d_tests; // CylindricalSurface3D のテスト
pub mod direction_3d; // Direction3D の新実装 (Core)
pub mod direction_3d_extensions;
pub mod ellipse_3d; // Ellipse3D の新実装 (Core)
mod ellipse_3d_collision; // Ellipse3D の衝突判定
#[cfg(test)]
pub mod ellipse_3d_collision_tests; // Ellipse3D の衝突判定テスト
pub mod ellipse_3d_extensions; // Ellipse3D の拡張機能 (Extension)
mod ellipse_3d_intersection; // Ellipse3D の交差計算
#[cfg(test)]
pub mod ellipse_3d_intersection_tests; // Ellipse3D の交差計算テスト
pub mod ellipse_arc_3d; // EllipseArc3D の実装 (Core)
pub mod ellipse_arc_3d_extensions; // EllipseArc3D の拡張機能 (Extension)
pub mod ellipse_arc_3d_foundation; // EllipseArc3D の Foundation 実装
pub mod ellipse_arc_3d_tests; // EllipseArc3D のテスト
pub mod ellipsoidal_solid_3d; // EllipsoidalSolid3D の新実装 (Core) - 完全ハイブリッドモデラー対応
mod ellipsoidal_solid_3d_collision; // EllipsoidalSolid3D の衝突判定
#[cfg(test)]
pub mod ellipsoidal_solid_3d_collision_tests; // EllipsoidalSolid3D の衝突判定テスト
pub mod ellipsoidal_solid_3d_foundation; // EllipsoidalSolid3D のFoundation実装
mod ellipsoidal_solid_3d_intersection; // EllipsoidalSolid3D の交差計算
#[cfg(test)]
pub mod ellipsoidal_solid_3d_intersection_tests; // EllipsoidalSolid3D の交差計算テスト
pub mod ellipsoidal_solid_3d_transform; // EllipsoidalSolid3D の Transform 実装
pub mod ellipsoidal_surface_3d; // EllipsoidalSurface3D の新実装 (Core) - 完全ハイブリッドモデラー対応
mod ellipsoidal_surface_3d_collision; // EllipsoidalSurface3D の衝突検出実装
mod ellipsoidal_surface_3d_intersection; // EllipsoidalSurface3D の交点計算実装
pub mod infinite_line_3d; // InfiniteLine3D の新実装
mod infinite_line_3d_collision; // InfiniteLine3D の衝突検出実装
pub mod infinite_line_3d_extensions; // InfiniteLine3D の拡張機能 (Extension)
pub mod infinite_line_3d_foundation; // InfiniteLine3D の Foundation 実装
mod infinite_line_3d_intersection; // InfiniteLine3D の交点計算実装
pub mod line_segment_3d; // LineSegment3D の新実装 (Core)
mod line_segment_3d_collision; // LineSegment3D の衝突検出実装
pub mod line_segment_3d_extensions; // LineSegment3D の拡張機能 (Extension)
pub mod line_segment_3d_foundation; // LineSegment3D の Foundation 実装
mod line_segment_3d_intersection; // LineSegment3D の交点計算実装
pub mod plane_3d; // Plane3D の新実装 (Core)
mod plane_3d_collision; // Plane3D の衝突検出実装
pub mod plane_3d_extensions; // Plane3D の拡張機能 (Extension)
pub mod plane_3d_foundation; // Plane3D のFoundation実装
mod plane_3d_intersection; // Plane3D の交点計算実装
#[cfg(test)]
pub mod plane_3d_tests; // Plane3D のテスト
                        // 削除: plane_coordinate_systemはPlane3Dに統合済み

// Point/Vector モジュール宣言
// Note: Point/Vector implementations are provided by geo_core
// TODO: Point2D implementation pending
// pub mod point_2d;
// pub mod point_2d_extensions;
// #[cfg(test)]
// pub mod point_2d_tests;
// pub mod point_2d_transform;
// pub mod point_3d;
// pub mod point_3d_extensions;
// pub mod point_3d_foundation;
// #[cfg(test)]
// pub mod point_3d_tests;
// pub mod vector_2d;
// pub mod vector_2d_extensions;
// #[cfg(test)]
// pub mod vector_2d_tests;
// pub mod vector_2d_transform;
// pub mod vector_3d;
// pub mod vector_3d_extensions;
// pub mod vector_3d_foundation;
// #[cfg(test)]
// pub mod vector_3d_tests;
// pub mod vector_3d_transform;
// #[cfg(test)]
// pub mod vector_3d_transform_tests;

pub mod ray_3d; // Ray3D の新実装 (Core)
mod ray_3d_collision; // Ray3D の衝突検出実装
pub mod ray_3d_extensions; // Ray3D の拡張機能 (Extension)
pub mod ray_3d_foundation; // Ray3D のFoundation実装
mod ray_3d_intersection; // Ray3D の交点計算実装
pub mod rectangle_3d; // Rect3D の新実装 (Core)
pub mod rectangle_3d_foundation; // Rect3D のFoundation実装
pub mod rectangle_3d_transform; // Rect3D の変換実装
pub mod spherical_solid_3d; // SphericalSolid3D の新実装 (Core) - 完全ハイブリッドモデラー対応
mod spherical_solid_3d_collision; // SphericalSolid3D の衝突判定
#[cfg(test)]
pub mod spherical_solid_3d_collision_tests; // SphericalSolid3D 衝突判定テスト
pub mod spherical_solid_3d_foundation; // SphericalSolid3D のFoundation実装
pub mod spherical_surface_3d; // SphericalSurface3D の新実装 (Core) - 完全ハイブリッドモデラー対応
pub mod spherical_surface_3d_foundation; // SphericalSurface3D のFoundation実装
pub mod torus_solid_3d; // TorusSolid3D の新実装 (Core) - 3D CAM 固体加工対応
mod torus_solid_3d_collision; // TorusSolid3D の衝突判定実装
pub mod torus_solid_3d_extensions; // TorusSolid3D の拡張機能 (Extension)
pub mod torus_solid_3d_foundation; // TorusSolid3D のFoundation実装
mod torus_solid_3d_intersection; // TorusSolid3D の交差判定実装
pub mod torus_surface_3d; // TorusSurface3D の新実装 (Core) - 3D CAM 工具オフセット対応
mod torus_surface_3d_collision; // TorusSurface3D の衝突判定実装
pub mod torus_surface_3d_extensions; // TorusSurface3D の拡張機能 (Extension)
pub mod torus_surface_3d_foundation; // TorusSurface3D のFoundation実装
mod torus_surface_3d_intersection; // TorusSurface3D の交差判定実装
pub mod triangle_3d; // Triangle3D の新実装 (Core)
mod triangle_3d_collision; // Triangle3D の衝突検出実装
pub mod triangle_3d_foundation; // Triangle3D のFoundation実装
mod triangle_3d_intersection; // Triangle3D の交点計算実装
pub mod triangle_mesh_3d; // TriangleMesh3D の新実装 (Core)
mod triangle_mesh_3d_collision; // TriangleMesh3D の衝突検出実装
pub mod triangle_mesh_3d_foundation; // TriangleMesh3D のFoundation実装
mod triangle_mesh_3d_intersection; // TriangleMesh3D の交点計算実装
pub mod triangle_mesh_3d_transform; // TriangleMesh3D のAnalysisTransform実装

// Transform テストモジュール
#[cfg(test)]
pub mod ray_3d_tests;
#[cfg(test)]
pub mod triangle_3d_tests;
#[cfg(test)]
pub mod triangle_mesh_3d_tests;

// Vector3D テストモジュール
// pub mod vector_3d_transform_safe_tests; // 削除済み

// 2D プリミティブ
// Arc2D関連（ジェネリック実装完了により再有効化）
pub mod arc_2d; // Arc2D の新実装 (Core)
pub mod arc_2d_extensions; // Arc2D の拡張機能 (Extension)
pub mod arc_2d_foundation; // Arc2D のFoundation実装

pub mod circle_2d; // Circle2D の新実装 (Core)
pub mod circle_2d_extensions; // Circle2D の拡張機能 (Extension - Phase 2 対応)
                              // pub mod circle_2d_core_traits; // Moved to circle_2d.rs
                              // pub mod circle_3d_core_traits; // Moved to circle_3d.rs

// Circle Core Traits の公開 - Foundation Pattern実装完了
pub use geo_contracts::{Circle2DConstructor, Circle2DMeasure, Circle2DProperties};
pub use geo_contracts::{Circle3DConstructor, Circle3DMeasure, Circle3DProperties};

// Arc Core Traits の公開 - Foundation Pattern実装完了
pub use geo_contracts::{
    Arc2DConstructor, Arc2DMeasure, Arc2DProperties, Arc3DConstructor, Arc3DMeasure,
    Arc3DProperties,
};

pub mod circle_2d_metrics; // Circle2D 計量演算
pub mod direction_2d; // Direction2D の新実装 (Core)
pub mod direction_2d_extensions;
// pub mod direction_3d_core_traits; // Moved to direction_3d.rs
pub use geo_contracts::{Direction3DConstructor, Direction3DMeasure, Direction3DProperties}; // Direction3D の Core traits 公開

// InfiniteLine Core Traits の公開 - Foundation Pattern実装完了
pub use geo_contracts::{
    InfiniteLine2DConstructor, InfiniteLine2DMeasure, InfiniteLine2DProperties,
    InfiniteLine3DConstructor, InfiniteLine3DMeasure, InfiniteLine3DProperties,
};

// Ray Core Traits の公開 - Foundation Pattern実装完了
pub use geo_contracts::{
    Ray2DConstructor, Ray2DMeasure, Ray2DProperties, Ray3DConstructor, Ray3DMeasure,
    Ray3DProperties,
};
pub mod ellipse_2d; // Ellipse2D の実装 (新traitsシステム対応)
                    // pub mod ellipse_2d_additional_tests; // Ellipse2D の追加テスト
pub mod ellipse_2d_foundation; // Ellipse2D の Foundation 実装
                               // pub mod ellipse_2d_tests; // Ellipse2D のテスト
pub mod ellipse_2d_transform; // Ellipse2D の変換実装
pub mod ellipse_arc_2d; // EllipseArc2D の実装 (Core)
pub mod ellipse_arc_2d_extensions; // EllipseArc2D の拡張機能 (Extension)
pub mod ellipse_arc_2d_foundation; // EllipseArc2D の Foundation 実装
pub mod infinite_line_2d; // InfiniteLine2D の新実装
pub mod infinite_line_2d_extensions; // InfiniteLine2D の拡張機能 (Extension)
pub mod infinite_line_2d_foundation; // InfiniteLine2D の Foundation 実装
pub mod infinite_line_2d_transform; // InfiniteLine2D の変換実装
pub mod line_segment_2d; // LineSegment2D の新実装 (Core)
pub mod line_segment_2d_extensions; // LineSegment2D の拡張機能 (Extension)
pub mod line_segment_2d_foundation; // LineSegment2D のFoundation実装
pub mod ray_2d; // Ray2D の新実装 (Core)
pub mod ray_2d_extensions; // Ray2D の拡張機能 (Extension)
pub mod ray_2d_foundation; // Ray2D の Foundation 実装
pub mod ray_2d_transform; // Ray2D の変換実装
pub mod rectangle_2d; // Rect2D の新実装 (Core)
pub mod rectangle_2d_foundation; // Rect2D の Foundation 実装
pub mod rectangle_2d_transform; // Rect2D の変換実装
pub mod triangle_2d; // Triangle2D の新実装 (Core)
pub mod triangle_2d_foundation; // Triangle2D の Foundation 実装
pub mod triangle_2d_transform; // Triangle2D の変換実装

// テストモジュール（次元中立設計）
#[cfg(test)]
mod ellipse_3d_tests;
// mod spherical_solid_3d_tests; // 未実装position機能のため無効化
// mod spherical_solid_3d_transform_safe_tests; // 削除済み
// mod spherical_surface_3d_tests; // 未実装position機能のため無効化
// mod spherical_surface_3d_transform_safe_tests; // 削除済み
// 2D テスト
#[cfg(test)]
mod direction_2d_extensions_tests;
#[cfg(test)]
mod direction_3d_extensions_tests;
// mod ellipse_arc_2d_tests; // 未実装Transform機能のため無効化
#[cfg(test)]
mod foundation_tests;
// mod infinite_line_2d_tests; // 未実装Transform機能のため無効化
// mod infinite_line_3d_tests; // 未実装Transform機能のため無効化
// Point2D/Point3D/Vector2D/Vector3D関連のモジュールは geo_core に移動済み

// 最小限の基盤のみ残す
pub use geo_contracts::{Angle, Scalar};

// Foundation システム統一トレイト
pub use geo_contracts::{
    AdvancedCollision,
    BBoxCollision,
    // Collision Foundation
    BasicCollision,
    // Intersection Foundation
    BasicIntersection,
    MultipleIntersection,
    PointDistance,
    SelfIntersection,
};

// 新実装の公開（次元中立設計）
// 3D プリミティブ
pub use arc_3d::Arc3D;
pub use circle_3d::Circle3D;
pub use conical_solid_3d::{Cone3D, ConicalSolid3D}; // 新式円錐ソリッド + 互換エイリアス
pub use conical_surface_3d::{ConeRim3D, ConicalSurface3D}; // 新式円錐サーフェス + 互換エイリアス
pub use cylindrical_solid_3d::CylindricalSolid3D; // 新式ソリッド
                                                  // cylindrical_solid_3d_transform_safe は削除済み
pub use cylindrical_surface_3d::CylindricalSurface3D; // 新式サーフェス
pub use direction_3d::Direction3D;
pub use ellipse_3d::Ellipse3D;
pub use ellipse_arc_3d::EllipseArc3D;
pub use ellipsoidal_solid_3d::EllipsoidalSolid3D; // 3D楕円体ソリッド
pub use ellipsoidal_surface_3d::EllipsoidalSurface3D; // 3D楕円体サーフェス
pub use infinite_line_3d::InfiniteLine3D;
pub use line_segment_3d::LineSegment3D;
pub use plane_3d::Plane3D;
// 削除: Plane3DCoordinateSystemはPlane3Dに統合済み
pub use ray_3d::Ray3D;
pub use rectangle_3d::Rect3D;
pub use spherical_solid_3d::SphericalSolid3D; // 新式球ソリッド
pub use spherical_surface_3d::SphericalSurface3D; // 新式球サーフェス
pub use torus_solid_3d::TorusSolid3D; // 新式トーラスソリッド (3D CAM対応)
pub use torus_surface_3d::TorusSurface3D; // 新式トーラスサーフェス (3D CAM対応)
pub use triangle_3d::Triangle3D;
pub use triangle_mesh_3d::TriangleMesh3D;

// 2D プリミティブ
pub use arc_2d::Arc2D;
pub use circle_2d::Circle2D;
pub use direction_2d::Direction2D;
pub use ellipse_2d::Ellipse2D;
pub use ellipse_arc_2d::EllipseArc2D; // 楕円弧
                                      // Point2D/Vector2D/Point3D/Vector3D は geo_core から提供
pub use geo_core::{Point2D, Point3D, Vector2D, Vector3D};
pub use infinite_line_2d::InfiniteLine2D;
pub use line_segment_2d::LineSegment2D;
pub use ray_2d::Ray2D;
pub use rectangle_2d::Rect2D;
pub use triangle_2d::Triangle2D;

// Core Traits統合エクスポート（Foundation経由）
pub use geo_contracts::{InfiniteLine2DCore, InfiniteLine3DCore};
pub use geo_contracts::{Ray2DCore, Ray3DCore};

// ============================================================================
// Test Modules
// ============================================================================
