//! Foundation トレイトの動作確認テスト
//!
//! Point2D と Vector2D を使用して実装済みの基本機能が
//! 正しく動作しているかを確認する最小限のテスト

use crate::{Point2D, Vector2D};

/// Point2D の基本機能テスト
#[test]
fn test_point2d_basic_functionality() {
    let point1 = Point2D::new(1.0, 2.0);
    let point2 = Point2D::new(1.0, 2.0);
    let point3 = Point2D::new(1.1, 2.0);

    // 同じ点は等しい
    assert_eq!(point1, point2);

    // 異なる点は等しくない
    assert_ne!(point1, point3);

    // 距離計算（現在実装されている機能）
    let distance = point1.distance_to(&point3);
    assert!((distance - 0.1f64).abs() < f64::EPSILON);
}

/// Vector2D の基本機能テスト
#[test]
fn test_vector2d_basic_functionality() {
    let vector = Vector2D::new(3.0f64, -2.0f64);

    // 基本的な属性アクセス
    assert_eq!(vector.x(), 3.0);
    assert_eq!(vector.y(), -2.0);

    // 長さ計算
    let length = vector.length();
    assert!((length - 3.605551275463989f64).abs() < f64::EPSILON);

    // 正規化
    let normalized = vector.normalize();
    assert!((normalized.length() - 1.0f64).abs() < f64::EPSILON);
}

/// 型関連のテスト
#[test]
fn test_type_consistency() {
    let point = Point2D::new(1.0, 2.0);
    let vector = Vector2D::new(3.0, 4.0);

    // 基本的な型の一貫性確認
    assert_eq!(std::mem::size_of_val(&point), 16); // f64 x 2
    assert_eq!(std::mem::size_of_val(&vector), 16); // f64 x 2
}

/// f32でのテスト
#[test]
fn test_f32_compatibility() {
    let point: Point2D<f32> = Point2D::new(1.0f32, 2.0f32);
    let vector: Vector2D<f32> = Vector2D::new(3.0f32, 4.0f32);

    // f32での基本機能確認
    assert_eq!(point.x(), 1.0f32);
    assert_eq!(vector.length(), 5.0f32);

    let normalized = vector.normalize();
    assert!((normalized.length() - 1.0f32).abs() < f32::EPSILON);
}
