//! Unit tests module
//! テストケースの整理されたモジュール構成

// テストユーティリティ（共通ツール）
pub mod test_utils;

mod geometry_bbox;

// 個別テストモジュール
mod arc_tests; // Arc テスト（分離型構造）
mod bbox2d_tests; // BBox2D テスト
mod bbox3d_tests; // BBox3D テスト
mod circle_parametric_tests; // Circle の型パラメータ化テスト
mod direction_tests; // Direction テスト
mod ellipse_tests; // Ellipse テスト（分離型構造）
mod point3d_tests; // Point3D テスト
mod point_parametric_tests; // Point の型パラメータ化テスト
mod sphere_tests; // Sphere テスト（分離型構造）
mod traits_tests; // トレイトテスト
mod vector2d_tests; // Vector2D テスト
mod vector3d_tests; // Vector3D テスト
mod vector_parametric_tests; // Vector の型パラメータ化テスト
mod vector_traits_tests; // Vector トレイトテスト
