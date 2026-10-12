//! Arc3D の基本テスト
//!
//! 基本機能のみテスト：作成、アクセサ、基本プロパティ

use crate::{Arc3D, Direction3D, Point3D, Vector3D};
use analysis::test_constants::{TOLERANCE_F32, TOLERANCE_F64};
use geo_contracts::{
    Angle, Arc3DConstructor, Arc3DDerived, Arc3DEndpoint, Arc3DEvaluation, Arc3DProperties,
    Arc3DSampling, Arc3DTrimRange, Bounded, PrimitiveKind, PrimitiveMetadata, TolerantEq,
};

// ヘルパー関数：ラジアンから Angle を作成
fn angle(radians: f64) -> Angle<f64> {
    Angle::from_radians(radians)
}

fn angle_f32(radians: f32) -> Angle<f32> {
    Angle::from_radians(radians)
}

#[test]
fn test_basic_creation() {
    // XY平面上の基本的な円弧作成
    let center = Point3D::new(0.0_f64, 0.0_f64, 0.0_f64);
    let arc = Arc3D::xy_arc(center, 5.0_f64, angle(0.0), angle(std::f64::consts::PI)).unwrap();

    assert_eq!(arc.center(), center);
    assert_eq!(arc.radius(), 5.0);
    assert_eq!(arc.normal().as_vector(), Vector3D::unit_z());
    assert_eq!(arc.start_direction().as_vector(), Vector3D::unit_x());
    assert_eq!(arc.start_angle(), angle(0.0));
    assert_eq!(arc.end_angle(), angle(std::f64::consts::PI));

    // 不正な円弧（負の半径）
    let invalid = Arc3D::xy_arc(center, -1.0_f64, angle(0.0), angle(std::f64::consts::PI));
    assert!(invalid.is_none());

    // ゼロ半径
    let invalid2 = Arc3D::xy_arc(center, 0.0_f64, angle(0.0), angle(std::f64::consts::PI));
    assert!(invalid2.is_none());
}

#[test]
fn test_from_three_points_over_half_circle_and_offset_center() {
    use std::f64::consts::PI;
    let offset = Vector3D::new(1.0_f64, 2.0, 3.0);
    // 0° → 180° → 270° の順に反時計回りに通る（掃引角 270°）
    let start = Point3D::new(1.0_f64, 0.0, 0.0) + offset;
    let middle = Point3D::new(-1.0_f64, 0.0, 0.0) + offset;
    let end = Point3D::new(0.0_f64, -1.0, 0.0) + offset;

    let arc = Arc3D::from_three_points(start, middle, end).unwrap();

    assert!(arc.center().distance_to(Point3D::new(1.0, 2.0, 3.0)) < TOLERANCE_F64);
    assert!((arc.radius() - 1.0).abs() < TOLERANCE_F64);
    let sweep = arc.end_angle().to_radians() - arc.start_angle().to_radians();
    assert!((sweep - 1.5 * PI).abs() < TOLERANCE_F64);

    // 中間点は 180° の位置にある
    assert!(arc.point_at_angle(angle(PI)).distance_to(middle) < TOLERANCE_F64);
}

#[test]
fn test_from_three_points() {
    // XY平面上の3点を通る円弧
    let start = Point3D::new(1.0_f64, 0.0_f64, 0.0_f64);
    let middle = Point3D::new(0.0_f64, 1.0_f64, 0.0_f64);
    let end = Point3D::new(-1.0_f64, 0.0_f64, 0.0_f64);

    let arc = Arc3D::from_three_points(start, middle, end).unwrap();

    // 中心は原点、半径 1、反時計回りに 180°
    assert!(arc.center().distance_to(Point3D::origin()) < TOLERANCE_F64);
    assert!((arc.radius() - 1.0_f64).abs() < TOLERANCE_F64);
    let sweep = arc.end_angle().to_radians() - arc.start_angle().to_radians();
    assert!((sweep - std::f64::consts::PI).abs() < TOLERANCE_F64);

    // 一直線上の点では作成不可
    let collinear_start = Point3D::new(0.0_f64, 0.0_f64, 0.0_f64);
    let collinear_middle = Point3D::new(1.0_f64, 0.0_f64, 0.0_f64);
    let collinear_end = Point3D::new(2.0_f64, 0.0_f64, 0.0_f64);

    let invalid = Arc3D::from_three_points(collinear_start, collinear_middle, collinear_end);
    assert!(invalid.is_none());
}

#[test]
fn test_basic_properties() {
    let center = Point3D::origin();
    let arc = Arc3D::xy_arc(center, 2.0_f64, angle(0.0), angle(std::f64::consts::PI)).unwrap();

    // 角度範囲
    let span = arc.angle_span();
    let expected_span = angle(std::f64::consts::PI);
    assert!(span.is_equivalent_default(&expected_span));

    // 円弧長
    let length = arc.length();
    let expected_length = 2.0 * std::f64::consts::PI; // 半円
    assert!((length - expected_length).abs() < TOLERANCE_F64);

    // 完全円判定
    assert!(!arc.is_full_circle());
}

#[test]
fn test_full_circle_detection() {
    let center = Point3D::origin();
    let full_arc = Arc3D::xy_arc(
        center,
        3.0_f64,
        angle(0.0),
        angle(2.0 * std::f64::consts::PI),
    )
    .unwrap();

    assert!(full_arc.is_full_circle());

    let arc_length = full_arc.length();
    let expected_circumference = 2.0 * std::f64::consts::PI * 3.0;
    assert!((arc_length - expected_circumference).abs() < TOLERANCE_F64);
}

#[test]
fn test_parametric_points() {
    let center = Point3D::origin();
    let arc = Arc3D::xy_arc(center, 4.0_f64, angle(0.0), angle(std::f64::consts::PI)).unwrap();

    // 開始点 (t=0)
    let start = arc.point_at_parameter(0.0);
    assert!((start.x() - 4.0_f64).abs() < TOLERANCE_F64);
    assert!((start.y() - 0.0_f64).abs() < TOLERANCE_F64);
    assert!((start.z() - 0.0_f64).abs() < TOLERANCE_F64);

    // 中点 (t=0.5)
    let mid = arc.point_at_parameter(0.5);
    assert!((mid.x() - 0.0_f64).abs() < TOLERANCE_F64);
    assert!((mid.y() - 4.0_f64).abs() < TOLERANCE_F64);
    assert!((mid.z() - 0.0_f64).abs() < TOLERANCE_F64);

    // 終了点 (t=1.0)
    let end = arc.point_at_parameter(1.0);
    assert!((end.x() - (-4.0_f64)).abs() < TOLERANCE_F64);
    assert!((end.y() - 0.0_f64).abs() < TOLERANCE_F64);
    assert!((end.z() - 0.0_f64).abs() < TOLERANCE_F64);
}

#[test]
fn test_endpoint_methods() {
    let center = Point3D::origin();
    let arc = Arc3D::xy_arc(
        center,
        3.0_f64,
        angle(0.0),
        angle(std::f64::consts::PI / 2.0),
    )
    .unwrap();

    // 開始点
    let start = arc.start_point();
    assert!((start.x() - 3.0_f64).abs() < TOLERANCE_F64);
    assert!((start.y() - 0.0_f64).abs() < TOLERANCE_F64);

    // 終了点
    let end = arc.end_point();
    assert!((end.x() - 0.0_f64).abs() < TOLERANCE_F64);
    assert!((end.y() - 3.0_f64).abs() < TOLERANCE_F64);

    // 中点
    let mid = <Arc3D<f64> as Arc3DEvaluation<f64>>::point_at_parameter(&arc, 0.5);
    let expected_mid_angle = std::f64::consts::PI / 4.0;
    let expected_x = 3.0 * expected_mid_angle.cos();
    let expected_y = 3.0 * expected_mid_angle.sin();
    assert!((mid.x() - expected_x).abs() < TOLERANCE_F64);
    assert!((mid.y() - expected_y).abs() < TOLERANCE_F64);
}

#[test]
fn test_parameter_and_angle_apis_are_distinct_but_consistent() {
    let center = Point3D::origin();
    let arc = Arc3D::xy_arc(
        center,
        3.0_f64,
        angle(0.0),
        angle(std::f64::consts::PI / 2.0),
    )
    .unwrap();

    let from_parameter = arc.point_at_parameter(0.5);
    let from_angle = arc.point_at_angle(angle(std::f64::consts::PI / 4.0));

    assert!((from_parameter.x() - from_angle.x()).abs() < TOLERANCE_F64);
    assert!((from_parameter.y() - from_angle.y()).abs() < TOLERANCE_F64);
    assert!((from_parameter.z() - from_angle.z()).abs() < TOLERANCE_F64);
}

#[test]
fn test_angle_containment() {
    let center = Point3D::origin();
    let arc = Arc3D::xy_arc(center, 2.0_f64, angle(0.0), angle(std::f64::consts::PI)).unwrap();

    // 範囲内の角度
    assert!(arc.contains_angle(angle(std::f64::consts::PI / 2.0)));
    assert!(arc.contains_angle(angle(0.0)));
    assert!(arc.contains_angle(angle(std::f64::consts::PI)));

    // 範囲外の角度
    assert!(!arc.contains_angle(angle(3.0 * std::f64::consts::PI / 2.0)));
    assert!(!arc.contains_angle(angle(-std::f64::consts::PI / 2.0)));
}

#[test]
fn test_angle_span_calculation() {
    let center = Point3D::origin();

    // 正の角度範囲
    let arc1 = Arc3D::xy_arc(center, 2.0_f64, angle(0.0), angle(std::f64::consts::PI)).unwrap();
    let expected_span1 = angle(std::f64::consts::PI);
    assert!(arc1.angle_span().is_equivalent_default(&expected_span1));

    // 0度をまたぐ角度範囲
    let arc2 = Arc3D::xy_arc(
        center,
        2.0_f64,
        angle(3.0 * std::f64::consts::PI / 2.0),
        angle(std::f64::consts::PI / 2.0),
    )
    .unwrap();
    let expected_span2 = angle(std::f64::consts::PI);
    assert!(arc2.angle_span().is_equivalent_default(&expected_span2));

    // 評価点も開始角から反時計回りに進む（t=0.5 は角度 0 の点）
    assert!(
        arc2.point_at_parameter(0.5)
            .distance_to(Point3D::new(2.0, 0.0, 0.0))
            < TOLERANCE_F64
    );
}

#[test]
fn test_sample_points() {
    let arc = Arc3D::xy_arc(
        Point3D::origin(),
        2.0_f64,
        angle(0.0),
        angle(std::f64::consts::PI / 2.0),
    )
    .unwrap();

    assert!(arc.sample_points(0).is_empty());
    assert_eq!(arc.sample_points(1), vec![arc.start_point()]);

    let points = arc.sample_points(3);
    assert_eq!(points.len(), 3);
    assert!(points[0].distance_to(arc.start_point()) < TOLERANCE_F64);
    assert!(points[1].distance_to(arc.point_at_parameter(0.5)) < TOLERANCE_F64);
    assert!(points[2].distance_to(arc.end_point()) < TOLERANCE_F64);
}

#[test]
fn test_f32_compatibility() {
    // f32での基本操作
    let center = Point3D::new(0.0f32, 0.0f32, 0.0f32);
    let arc = Arc3D::xy_arc(
        center,
        3.0f32,
        angle_f32(0.0f32),
        angle_f32(std::f32::consts::PI),
    )
    .unwrap();

    assert_eq!(arc.radius(), 3.0f32);

    let length = arc.length();
    let expected_length = 3.0f32 * std::f32::consts::PI;
    assert!((length - expected_length).abs() < TOLERANCE_F32);

    let start = arc.start_point();
    assert!((start.x() - 3.0f32).abs() < TOLERANCE_F32);
    assert!((start.y() - 0.0f32).abs() < TOLERANCE_F32);
}

#[test]
fn test_tolerant_eq() {
    let center = Point3D::new(0.0, 0.0, 0.0);
    let normal = Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap();
    let start_dir = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();
    let start_angle = geo_contracts::Angle::from_radians(0.0);
    let end_angle = geo_contracts::Angle::from_radians(std::f64::consts::PI);

    let arc1 = Arc3D::new(center, 5.0, normal, start_dir, start_angle, end_angle).unwrap();
    let arc2 = Arc3D::new(center, 5.0, normal, start_dir, start_angle, end_angle).unwrap();
    let arc3 = Arc3D::new(
        Point3D::new(1.0, 0.0, 0.0),
        5.0,
        normal,
        start_dir,
        start_angle,
        end_angle,
    )
    .unwrap();

    assert!(arc1.tolerant_eq(&arc2, 0.01));
    assert!(!arc1.tolerant_eq(&arc3, 0.01));
}

#[test]
fn test_metadata_and_bounds_capabilities() {
    let center = Point3D::new(0.0, 0.0, 0.0);
    let normal = Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap();
    let start_dir = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();
    let start_angle = geo_contracts::Angle::from_radians(0.0);
    let end_angle = geo_contracts::Angle::from_radians(std::f64::consts::PI);
    let arc = Arc3D::new(center, 5.0, normal, start_dir, start_angle, end_angle).unwrap();

    assert_eq!(arc.primitive_kind(), PrimitiveKind::Arc);
    assert_eq!(arc.length(), 5.0 * std::f64::consts::PI);

    let aabb = arc.aabb().expect("Arc should have an AABB");
    assert!((aabb.min().x() + 5.0).abs() < 1e-10);
    assert!((aabb.max().x() - 5.0).abs() < 1e-10);
    assert!((aabb.min().y() - 0.0).abs() < 1e-10);
    assert!((aabb.max().y() - 5.0).abs() < 1e-10);
    assert!((aabb.min().z() - 0.0).abs() < 1e-10);
    assert!((aabb.max().z() - 0.0).abs() < 1e-10);
}

#[test]
fn test_arc_3d_aabb_respects_trimmed_range() {
    let center = Point3D::new(0.0, 0.0, 0.0);
    let normal = Direction3D::from_vector(Vector3D::new(0.0, 1.0, 0.0)).unwrap();
    let start_dir = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();
    let arc = Arc3D::new(
        center,
        2.0,
        normal,
        start_dir,
        geo_contracts::Angle::from_radians(0.0),
        geo_contracts::Angle::from_radians(std::f64::consts::PI),
    )
    .unwrap();

    let aabb = arc.aabb().expect("Arc should have an AABB");
    assert!((aabb.min().x() + 2.0).abs() < 1e-10);
    assert!((aabb.max().x() - 2.0).abs() < 1e-10);
    assert!((aabb.min().y() - 0.0).abs() < 1e-10);
    assert!((aabb.max().y() - 0.0).abs() < 1e-10);
    assert!((aabb.min().z() + 2.0).abs() < 1e-10);
    assert!((aabb.max().z() - 0.0).abs() < 1e-10);
}
