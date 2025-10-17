//! Foundation traitのテスト

use crate::{Point2D, Vector2D};
use geo_foundation::core_foundation::{BasicContainment, BasicDirectional, CoreFoundation};

/// Point2DのCoreFoundationテスト
#[test]
fn test_point2d_geometry_foundation() {
    let point = Point2D::new(3.0, 4.0);

    // 境界ボックス取得
    let bbox = point.bounding_box();
    assert_eq!(bbox.min(), point);
    assert_eq!(bbox.max(), point);
}

/// Point2DのBasicContainmentテスト
#[test]
fn test_point2d_basic_containment() {
    let point1 = Point2D::new(1.0, 2.0);
    let point2 = Point2D::new(1.0, 2.0);
    let point3 = Point2D::new(1.1, 2.0);

    // 同じ点は含む
    assert!(point1.contains_point(&point2));

    // 異なる点は含まない
    assert!(!point1.contains_point(&point3));

    // 境界判定（許容誤差内）
    assert!(point1.on_boundary(&point3, 0.2));
    assert!(!point1.on_boundary(&point3, 0.05));

    // 距離計算
    let distance = point1.distance_to_point(&point3);
    assert!((distance - 0.1f64).abs() < f64::EPSILON);
}

/// Vector2DのCoreFoundationテスト
#[test]
fn test_vector2d_geometry_foundation() {
    let vector = Vector2D::new(3.0, -2.0);

    // 境界ボックス取得（原点からベクトル終点まで）
    let bbox = vector.bounding_box();
    assert_eq!(bbox.min(), Point2D::new(0.0, -2.0));
    assert_eq!(bbox.max(), Point2D::new(3.0, 0.0));
}

/// Vector2DのBasicMetricsテスト
#[test]
fn test_vector2d_basic_metrics() {
    let vector = Vector2D::new(3.0f64, 4.0f64);

    // 長さ取得（BasicMetricsトレイトから）
    use geo_foundation::core_foundation::BasicMetrics;
    let length = BasicMetrics::length(&vector).unwrap();
    assert_eq!(length, 5.0f64);

    // 面積・体積・周長は定義されない
    assert!(vector.area().is_none());
    assert!(vector.volume().is_none());
    assert!(vector.perimeter().is_none());
}

/// Vector2DのBasicDirectionalテスト
#[test]
fn test_vector2d_basic_directional() {
    let vector = Vector2D::new(3.0, 4.0);

    // 方向取得（正規化）
    let direction = vector.direction();
    assert!((direction.length() - 1.0f64).abs() < f64::EPSILON);
    assert_eq!(direction.x(), 0.6);
    assert_eq!(direction.y(), 0.8);

    // 方向反転
    let reversed = vector.reverse_direction();
    assert_eq!(reversed.x(), -3.0);
    assert_eq!(reversed.y(), -4.0);
}

/// 型関連のテスト
#[test]
fn test_associated_types() {
    let point = Point2D::new(1.0, 2.0);
    let vector = Vector2D::new(3.0, 4.0);

    // 型の一貫性確認
    let bbox_from_point = point.bounding_box();
    let bbox_from_vector = vector.bounding_box();

    // 同じBBox2D型を返すことを確認
    assert_eq!(
        std::mem::size_of_val(&bbox_from_point),
        std::mem::size_of_val(&bbox_from_vector)
    );
}

/// f32でのテスト
#[test]
fn test_foundation_f32() {
    let point: Point2D<f32> = Point2D::new(1.0f32, 2.0f32);
    let vector: Vector2D<f32> = Vector2D::new(3.0f32, 4.0f32);

    // f32での基本機能確認
    assert!(point.contains_point(&point));

    // BasicMetricsトレイトのlength()を明示的に呼び出し
    use geo_foundation::core_foundation::BasicMetrics;
    let length_opt: Option<f32> = BasicMetrics::length(&vector);
    assert_eq!(length_opt.unwrap(), 5.0f32);

    let direction = vector.direction();
    assert!((direction.length() - 1.0f32).abs() < f32::EPSILON);
}
