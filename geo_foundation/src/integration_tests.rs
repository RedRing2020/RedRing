/// geo_foundation統合テスト/// geo_foundation統合テスト

//////

/// Scalar trait 実装の統合動作確認を行います。/// Scalar trait 実装の統合動作確認を行います。

/// f32/f64両方での動作と相互運用性をテストします。/// f32/f64両方での動作と相互運用性をテストします。

use crate::Scalar;use crate::{

    Scalar,

#[cfg(test)]};

mod integration_tests {

    use super::*;#[cfg(test)]

mod integration_tests {

    #[test]    use super::*;

    fn test_scalar_trait_f32_f64_compatibility() {

        // f32 Scalar trait テスト    #[test]

        let a_f32 = 3.0f32;    fn test_scalar_trait_f32_f64_compatibility() {

        let b_f32 = 4.0f32;        // f32 Scalar trait テスト

        let hypotenuse_f32 = (a_f32 * a_f32 + b_f32 * b_f32).sqrt();        let a_f32 = 3.0f32;

        assert!((hypotenuse_f32 - 5.0f32).abs() < f32::TOLERANCE);        let b_f32 = 4.0f32;

        let hypotenuse_f32 = (a_f32 * a_f32 + b_f32 * b_f32).sqrt();

        // f64 Scalar trait テスト        assert!((hypotenuse_f32 - 5.0f32).abs() < f32::TOLERANCE);

        let a_f64 = 3.0f64;

        let b_f64 = 4.0f64;        // f64 Scalar trait テスト

        let hypotenuse_f64 = (a_f64 * a_f64 + b_f64 * b_f64).sqrt();        let a_f64 = 3.0f64;

        assert!((hypotenuse_f64 - 5.0f64).abs() < f64::TOLERANCE);        let b_f64 = 4.0f64;

        let hypotenuse_f64 = (a_f64 * a_f64 + b_f64 * b_f64).sqrt();

        // 型変換テスト        assert!((hypotenuse_f64 - 5.0f64).abs() < f64::TOLERANCE);

        let f32_val = 3.141592_f32;

        let converted_to_f64 = f32_val.to_f64();        // 型変換テスト

        let back_to_f32 = f32::from_f64(converted_to_f64);        let f32_val = 3.141592_f32;

        assert!((f32_val - back_to_f32).abs() < f32::TOLERANCE);        let converted_to_f64 = f32_val.to_f64();

    }        let back_to_f32 = f32::from_f64(converted_to_f64);

        assert!((f32_val - back_to_f32).abs() < f32::TOLERANCE);

    #[test]    }

    fn test_mathematical_constants_consistency() {

        // PI定数の一貫性    #[test]

        assert!((f32::PI.to_f64() - f64::PI).abs() < 1e-6);    fn test_mathematical_constants_consistency() {

        assert!((f32::TAU.to_f64() - f64::TAU).abs() < 1e-6);        // PI定数の一貫性

        assert!((f32::PI.to_f64() - f64::PI).abs() < 1e-6);

        // 角度変換定数の一貫性        assert!((f32::TAU.to_f64() - f64::TAU).abs() < 1e-6);

        let degrees_f32 = 180.0f32;

        let radians_f32 = degrees_f32 * f32::DEG_TO_RAD;        // 角度変換定数の一貫性

        assert!((radians_f32 - f32::PI).abs() < 1e-6);        let degrees_f32 = 180.0f32;

        let radians_f32 = degrees_f32 * f32::DEG_TO_RAD;

        let degrees_f64 = 180.0f64;        assert!((radians_f32 - f32::PI).abs() < 1e-6);

        let radians_f64 = degrees_f64 * f64::DEG_TO_RAD;

        assert!((radians_f64 - f64::PI).abs() < 1e-10);        let degrees_f64 = 180.0f64;

    }        let radians_f64 = degrees_f64 * f64::DEG_TO_RAD;

        assert!((radians_f64 - f64::PI).abs() < 1e-10);

    #[test]    }

    fn test_tolerance_handling() {

        // f32許容誤差    #[test]

        let a_f32 = 1.0f32;    fn test_tolerance_handling() {

        let b_f32 = 1.0f32 + f32::TOLERANCE / 10.0;        // f32許容誤差

        assert!(a_f32.approx_eq(b_f32));        let a_f32 = 1.0f32;

        let b_f32 = 1.0f32 + f32::TOLERANCE / 10.0;

        let c_f32 = 1.0f32 + f32::TOLERANCE * 10.0;        assert!(a_f32.approx_eq(b_f32));

        assert!(!a_f32.approx_eq(c_f32));

        let c_f32 = 1.0f32 + f32::TOLERANCE * 10.0;

        // f64許容誤差        assert!(!a_f32.approx_eq(c_f32));

        let a_f64 = 1.0f64;

        let b_f64 = 1.0f64 + f64::TOLERANCE / 10.0;        // f64許容誤差

        assert!(a_f64.approx_eq(b_f64));        let a_f64 = 1.0f64;

        let b_f64 = 1.0f64 + f64::TOLERANCE / 10.0;

        let c_f64 = 1.0f64 + f64::TOLERANCE * 10.0;        assert!(a_f64.approx_eq(b_f64));

        assert!(!a_f64.approx_eq(c_f64));

    }        let c_f64 = 1.0f64 + f64::TOLERANCE * 10.0;

        assert!(!a_f64.approx_eq(c_f64));

    #[test]    }

    fn test_type_conversions() {

        // Scalar trait のメソッドテスト    #[test]

        let value_f32: f32 = 42.5;    fn test_type_conversions() {

        let value_f64: f64 = 42.5;        // Scalar trait のメソッドテスト

        let value_f32: f32 = 42.5;

        // 基本的な変換        let value_f64: f64 = 42.5;

        assert!((value_f32.to_f64() - value_f64).abs() < 1e-6);

        assert!((f32::from_f64(value_f64) - value_f32).abs() < f32::TOLERANCE);        // 基本的な変換

        assert!((value_f32.to_f64() - value_f64).abs() < 1e-6);

        // 近似等価        assert!((f32::from_f64(value_f64) - value_f32).abs() < f32::TOLERANCE);

        assert!(value_f32.approx_eq(42.5));

        assert!(value_f64.approx_eq(42.5));        // 近似等価

        assert!(value_f32.approx_eq(42.5));

        // ゼロ判定        assert!(value_f64.approx_eq(42.5));

        assert!(0.0f32.is_zero());

        assert!(0.0f64.is_zero());        // ゼロ判定

        assert!((f32::TOLERANCE / 10.0).is_zero());        assert!(0.0f32.is_zero());

        assert!((f64::TOLERANCE / 10.0).is_zero());        assert!(0.0f64.is_zero());

        assert!((f32::TOLERANCE / 10.0).is_zero());

        // 1か判定        assert!((f64::TOLERANCE / 10.0).is_zero());

        assert!(1.0f32.is_one());

        assert!(1.0f64.is_one());        // 1か判定

    }        assert!(1.0f32.is_one());

        assert!(1.0f64.is_one());

    #[test]    }

    fn test_precision_boundaries() {

        // 精度境界でのテスト    #[test]

        let small_f32 = f32::TOLERANCE * 0.1;    fn test_precision_boundaries() {

        let large_f32 = f32::TOLERANCE * 10.0;        // 精度境界でのテスト

        let small_f32 = f32::TOLERANCE * 0.1;

        assert!(small_f32.is_zero());        let large_f32 = f32::TOLERANCE * 10.0;

        assert!(!large_f32.is_zero());

        assert!(small_f32.is_zero());

        let small_f64 = f64::TOLERANCE * 0.1;        assert!(!large_f32.is_zero());

        let large_f64 = f64::TOLERANCE * 10.0;

        let small_f64 = f64::TOLERANCE * 0.1;

        assert!(small_f64.is_zero());        let large_f64 = f64::TOLERANCE * 10.0;

        assert!(!large_f64.is_zero());

    }        assert!(small_f64.is_zero());

}        assert!(!large_f64.is_zero());
    }
}

    #[test]
    fn test_vector_angle_operations() {
        // VectorとAngleの統合動作テスト

        let angle = Angle::<f64>::from_degrees(30.0);
        let vector = Vector2D::from_angle(angle.to_radians());

        // ベクトルの角度が元の角度と一致
        let calculated_angle = Angle::<f64>::from_radians(vector.angle());
        assert!((angle.to_degrees() - calculated_angle.to_degrees()).abs() < 1e-10);

        // ベクトル回転による角度操作
        let rotation_angle = Angle::<f64>::from_degrees(45.0);
        let rotated_vector = Vector2D::from_angle(angle.to_radians() + rotation_angle.to_radians());

        let final_angle = Angle::<f64>::from_radians(rotated_vector.angle());
        let expected_angle = angle + rotation_angle;
        assert!((final_angle.to_degrees() - expected_angle.to_degrees()).abs() < 1e-10);
    }

    #[test]
    fn test_3d_vector_integration() {
        // 3Dベクトルの統合動作テスト

        // 3D単位ベクトル
        let x_axis = Vector3D::<f64>::unit_x();
        let y_axis = Vector3D::<f64>::unit_y();
        let z_axis = Vector3D::<f64>::unit_z();

        // 外積で右手座標系を確認
        let z_from_cross = x_axis.cross(&y_axis);
        assert!((z_from_cross.distance_to(&z_axis)) < 1e-10);

        // 正規直交基底の構築
        let arbitrary_vector = Vector3D::new(1.0, 2.0, 3.0);
        let (basis_x, basis_y, basis_z) = arbitrary_vector.build_orthonormal_basis();

        // 基底ベクトルが正規直交していることを確認
        assert!((basis_x.length() - 1.0).abs() < 1e-10);
        assert!((basis_y.length() - 1.0).abs() < 1e-10);
        assert!((basis_z.length() - 1.0).abs() < 1e-10);
        assert!(basis_x.dot(&basis_y).abs() < 1e-10);
        assert!(basis_y.dot(&basis_z).abs() < 1e-10);
        assert!(basis_z.dot(&basis_x).abs() < 1e-10);
    }

    #[test]
    fn test_vector_point_conversion_workflow() {
        // Vector⇔Point変換ワークフローテスト

        // 原点から複数の点への位置ベクトル
        let points = vec![
            Point2D::new(1.0, 2.0),
            Point2D::new(3.0, 4.0),
            Point2D::new(-1.0, 1.0),
        ];

        // Point→Vector変換
        let vectors: Vec<Vector2D<f64>> = points.iter().map(|&p| p.into()).collect();

        // ベクトル演算（重心計算）
        let centroid_vector =
            vectors.iter().fold(Vector2D::zero(), |acc, &v| acc + v) / (vectors.len() as f64);

        // Vector→Point変換
        let centroid_point: Point2D<f64> = centroid_vector.into();

        // 重心の検証
        let expected_centroid = Point2D::new(1.0, 7.0 / 3.0);
        assert!((centroid_point.distance_to(expected_centroid)) < 1e-10);
    }

    #[test]
    fn test_mixed_precision_vector_operations() {
        // 異なる精度でのVector演算テスト

        // f32での高速ベクトル計算
        let v1_f32 = Vector2D::<f32>::new(1.0, 2.0);
        let v2_f32 = Vector2D::<f32>::new(3.0, 4.0);
        let result_f32 = v1_f32 + v2_f32;

        // f64での高精度計算
        let v1_f64 = v1_f32.to_f64();
        let v2_f64 = v2_f32.to_f64();
        let result_f64 = v1_f64 + v2_f64;

        // 精度を考慮した結果比較
        let result_f32_as_f64 = result_f32.to_f64();
        assert!((result_f64.distance_to(&result_f32_as_f64)) < 1e-6);

        // 長さ計算の精度比較
        let length_f32 = result_f32.length() as f64;
        let length_f64 = result_f64.length();
        assert!((length_f32 - length_f64).abs() < 1e-6);
    }

    #[test]
    fn test_bbox_point_vector_integration() {
        // BBoxとPoint、Vectorの統合動作テスト

        // 複数の点から境界ボックスを作成
        let points = vec![
            Point2D::new(1.0, 2.0),
            Point2D::new(5.0, 1.0),
            Point2D::new(3.0, 6.0),
            Point2D::new(2.0, 4.0),
        ];

        let bbox = BBox2D::from_points(&points).unwrap();

        // 境界ボックスの中心点
        let center = bbox.center();
        assert_eq!(center, Point2D::new(3.0, 3.5));

        // すべての点が境界ボックス内にあることを確認
        for &point in &points {
            assert!(bbox.contains_point(point));
        }

        // サイズベクトル
        let size_vector = bbox.size();
        assert_eq!(size_vector.x(), 4.0); // width
        assert_eq!(size_vector.y(), 5.0); // height

        // ベクトル移動
        let offset = Vector2D::new(2.0, -1.0);
        let translated = bbox.translate(offset);
        let new_center = translated.center();
        assert_eq!(new_center, Point2D::new(5.0, 2.5));
    }

    #[test]
    fn test_bbox_circle_integration() {
        // BBoxとCircleの統合動作テスト

        let circle = Circle2DImpl::<f64>::new(Point2D::new(3.0, 2.0), 2.0);

        // 円の境界ボックスを手動作成
        let circle_bbox = BBox2D::from_coords(1.0, 0.0, 5.0, 4.0);

        // 円の中心点が境界ボックス内にあることを確認
        assert!(circle_bbox.contains_point(circle.center()));

        // 円周上の点を生成して境界ボックステスト
        let angle_0 = Angle::<f64>::from_degrees(0.0);
        let angle_90 = Angle::<f64>::from_degrees(90.0);
        let angle_180 = Angle::<f64>::from_degrees(180.0);
        let angle_270 = Angle::<f64>::from_degrees(270.0);

        let point_0 = circle.point_at_angle(angle_0); // 右端
        let point_90 = circle.point_at_angle(angle_90); // 上端
        let point_180 = circle.point_at_angle(angle_180); // 左端
        let point_270 = circle.point_at_angle(angle_270); // 下端

        // 各端点が境界ボックス上にあることを確認
        assert!(circle_bbox.contains_point(point_0));
        assert!(circle_bbox.contains_point(point_90));
        assert!(circle_bbox.contains_point(point_180));
        assert!(circle_bbox.contains_point(point_270));
    }

    #[test]
    fn test_bbox_3d_integration() {
        // 3D BBoxの統合動作テスト

        let points_3d = vec![
            Point3D::new(1.0, 2.0, 1.0),
            Point3D::new(4.0, 1.0, 3.0),
            Point3D::new(2.0, 5.0, 2.0),
            Point3D::new(3.0, 3.0, 4.0),
        ];

        let bbox_3d = BBox3D::from_points(&points_3d).unwrap();

        // 3D→2D投影テスト
        let bbox_2d = bbox_3d.to_2d();
        assert_eq!(bbox_2d.min(), Point2D::new(1.0, 1.0));
        assert_eq!(bbox_2d.max(), Point2D::new(4.0, 5.0));

        // 3Dベクトルとの統合
        let _center_3d = bbox_3d.center();
        let size_3d = bbox_3d.size();

        let volume = bbox_3d.volume();
        let expected_volume = size_3d.x() * size_3d.y() * size_3d.z();
        assert_eq!(volume, expected_volume);

        // 3D境界ボックスの8つの角
        let corners = bbox_3d.corners();
        assert_eq!(corners.len(), 8);

        // すべての角が境界ボックス内にあることを確認
        for &corner in &corners {
            assert!(bbox_3d.contains_point(corner));
        }
    }

    #[test]
    fn test_bbox_precision_compatibility() {
        // f32/f64精度での境界ボックス動作テスト

        let bbox_f32 = BBox2D::<f32>::from_coords(1.0, 2.0, 5.0, 6.0);
        let bbox_f64 = bbox_f32.to_f64();

        // 基本プロパティの比較
        assert!((bbox_f32.area() as f64 - bbox_f64.area()).abs() < 1e-6);
        assert!((bbox_f32.perimeter() as f64 - bbox_f64.perimeter()).abs() < 1e-6);

        // 中心点の比較
        let center_f32 = bbox_f32.center();
        let center_f64 = bbox_f64.center();
        assert!((center_f32.x() as f64 - center_f64.x()).abs() < 1e-6);
        assert!((center_f32.y() as f64 - center_f64.y()).abs() < 1e-6);

        // 交差判定の一貫性
        let other_f32 = BBox2D::<f32>::from_coords(3.0, 4.0, 7.0, 8.0);
        let other_f64 = other_f32.to_f64();

        assert_eq!(
            bbox_f32.intersects(&other_f32),
            bbox_f64.intersects(&other_f64)
        );
    }

    #[test]
    fn test_bbox_union_intersection_complex() {
        // 複数境界ボックスの和集合・積集合テスト

        let bbox1 = BBox2D::from_coords(0.0, 0.0, 4.0, 3.0);
        let bbox2 = BBox2D::from_coords(2.0, 1.0, 6.0, 5.0);
        let bbox3 = BBox2D::from_coords(1.0, 2.0, 3.0, 4.0);

        // 2つの境界ボックスの和集合
        let union12 = bbox1.union(&bbox2);
        assert_eq!(union12.min(), Point2D::new(0.0, 0.0));
        assert_eq!(union12.max(), Point2D::new(6.0, 5.0));

        // 3つの境界ボックスの和集合（段階的）
        let union_all = union12.union(&bbox3);
        assert_eq!(union_all.min(), Point2D::new(0.0, 0.0));
        assert_eq!(union_all.max(), Point2D::new(6.0, 5.0));

        // 積集合
        let intersection12 = bbox1.intersection(&bbox2).unwrap();
        assert_eq!(intersection12.min(), Point2D::new(2.0, 1.0));
        assert_eq!(intersection12.max(), Point2D::new(4.0, 3.0));

        // 積集合が存在しない場合
        let bbox_disjoint = BBox2D::from_coords(7.0, 6.0, 9.0, 8.0);
        assert!(bbox1.intersection(&bbox_disjoint).is_none());
    }

    // Arc/Ellipse統合テスト（新規追加）
    #[test]
    fn test_arc_circle_integration() {
        use crate::{Arc2D, Arc2DImpl, Circle2DImpl};

        // 円から円弧を作成
        let circle = Circle2DImpl::<f64>::new(Point2D::new(0.0, 0.0), 2.0);
        let start_angle = Angle::<f64>::from_degrees(0.0);
        let end_angle = Angle::<f64>::from_degrees(90.0);

        let arc = Arc2DImpl::new(circle, start_angle, end_angle).expect("円弧作成に失敗");

        // 円弧と元の円の関係
        assert_eq!(arc.center(), Point2D::new(0.0, 0.0));
        assert_eq!(arc.radius(), 2.0);

        // 円弧の端点が正しいかチェック
        let start_point = arc.start_point();
        let end_point = arc.end_point();

        assert!((start_point.x() - 2.0).abs() < 1e-10);
        assert!(start_point.y().abs() < 1e-10);
        assert!(end_point.x().abs() < 1e-10);
        assert!((end_point.y() - 2.0).abs() < 1e-10);

        // 弧長計算（90度の弧 = 1/4円）
        let expected_arc_length = 2.0 * PI * 2.0 / 4.0; // quarter circle: 2πr/4
        assert!((arc.arc_length() - expected_arc_length).abs() < 1e-10);
    }

    #[test]
    fn test_arc_point_containment_integration() {
        use crate::{Arc2D, Arc2DImpl, Circle2DImpl};

        let circle = Circle2DImpl::<f64>::new(Point2D::new(0.0, 0.0), 3.0);
        let arc = Arc2DImpl::new(
            circle,
            Angle::<f64>::from_degrees(30.0),
            Angle::<f64>::from_degrees(120.0),
        )
        .expect("円弧作成に失敗");

        let tolerance = 1e-10;

        // 円弧上の点（角度範囲内）
        let angle_60 = Angle::<f64>::from_degrees(60.0);
        let point_on_arc = Point2D::new(3.0 * angle_60.cos(), 3.0 * angle_60.sin());
        assert!(arc.contains_point(&point_on_arc, tolerance));

        // 円周上だが角度範囲外の点
        let angle_0 = Angle::<f64>::from_degrees(0.0);
        let point_outside_arc = Point2D::new(3.0 * angle_0.cos(), 3.0 * angle_0.sin());
        assert!(!arc.contains_point(&point_outside_arc, tolerance));

        // 円弧への最短距離
        let far_point = Point2D::new(10.0, 0.0);
        let distance = arc.distance_to_point(&far_point);

        // 端点からの距離が最短のはず
        let start_point = arc.start_point();
        let expected_distance = far_point.distance_to(start_point);
        assert!((distance - expected_distance).abs() < 1e-10);
    }

    #[test]
    fn test_arc_circle_intersection() {
        use crate::{Arc2D, Arc2DImpl, Circle2DImpl};

        let arc = Arc2DImpl::from_center_radius_angles(
            Point2D::new(0.0, 0.0),
            2.0,
            Angle::<f64>::from_degrees(0.0),
            Angle::<f64>::from_degrees(180.0),
        )
        .expect("円弧作成に失敗");

        // 交差する円
        let intersecting_circle = Circle2DImpl::new(Point2D::new(1.0, 0.0), 1.5);
        assert!(arc.intersects_with_circle(&intersecting_circle));

        // 交差しない円
        let far_circle = Circle2DImpl::new(Point2D::new(10.0, 10.0), 1.0);
        assert!(!arc.intersects_with_circle(&far_circle));

        // 円弧同士の交差
        let other_arc = Arc2DImpl::from_center_radius_angles(
            Point2D::new(0.0, 0.0),
            2.0,
            Angle::<f64>::from_degrees(90.0),
            Angle::<f64>::from_degrees(270.0),
        )
        .expect("他の円弧作成に失敗");

        assert!(arc.intersects_with_arc(&other_arc));
    }

    #[test]
    fn test_ellipse_creation_integration() {
        use crate::{Ellipse, Ellipse2DImpl, EllipseError};

        // 軸平行楕円の作成
        let ellipse = Ellipse2DImpl::axis_aligned(
            Point2D::new(1.0, 2.0),
            3.0, // major radius
            2.0, // minor radius
        )
        .expect("楕円作成に失敗");

        assert_eq!(ellipse.center(), Point2D::new(1.0, 2.0));
        assert_eq!(ellipse.major_radius(), 3.0);
        assert_eq!(ellipse.minor_radius(), 2.0);

        // 面積計算
        let expected_area = PI * 3.0 * 2.0;
        assert!((ellipse.area() - expected_area).abs() < 1e-10);

        // 離心率計算
        let expected_eccentricity = (1.0 - (2.0 * 2.0) / (3.0 * 3.0)).sqrt();
        assert!((ellipse.eccentricity() - expected_eccentricity).abs() < 1e-10);

        // エラーケースのテスト
        let invalid_result = Ellipse2DImpl::axis_aligned(
            Point2D::new(0.0, 0.0),
            1.0, // minor radius
            2.0, // major radius（長軸 < 短軸）
        );
        assert!(matches!(
            invalid_result,
            Err(EllipseError::InvalidAxisOrder)
        ));
    }

    #[test]
    fn test_ellipse_point_operations() {
        use crate::{Ellipse, Ellipse2D, Ellipse2DImpl};

        let ellipse = Ellipse2DImpl::axis_aligned(
            Point2D::new(0.0, 0.0),
            4.0, // major radius
            3.0, // minor radius
        )
        .expect("楕円作成に失敗");

        // 角度による点計算
        let angle_0 = Angle::<f64>::from_degrees(0.0);
        let point_0 = ellipse.point_at_angle(angle_0);
        assert!((point_0.x() - 4.0).abs() < 1e-10);
        assert!(point_0.y().abs() < 1e-10);

        let angle_90 = Angle::<f64>::from_degrees(90.0);
        let point_90 = ellipse.point_at_angle(angle_90);
        assert!(point_90.x().abs() < 1e-10);
        assert!((point_90.y() - 3.0).abs() < 1e-10);

        // 包含判定
        assert!(ellipse.contains_point(&Point2D::new(0.0, 0.0))); // 中心
        assert!(ellipse.contains_point(&Point2D::new(2.0, 2.0))); // 内部
        assert!(!ellipse.contains_point(&Point2D::new(5.0, 0.0))); // 外部

        // 焦点計算
        let (f1, f2) = ellipse.foci();
        let c = (4.0 * 4.0 - 3.0 * 3.0).sqrt(); // focal distance
        assert!((f1.x() - c).abs() < 1e-10);
        assert!(f1.y().abs() < 1e-10);
        assert!((f2.x() + c).abs() < 1e-10);
        assert!(f2.y().abs() < 1e-10);
    }

    #[test]
    fn test_ellipse_3d_integration() {
        use crate::{Ellipse, Ellipse3D, Ellipse3DImpl};

        // XY平面上の3D楕円
        let ellipse_3d = Ellipse3DImpl::from_radii_xy(
            Point3D::new(0.0, 0.0, 0.0),
            3.0, // major radius
            2.0, // minor radius
            Angle::<f64>::from_degrees(45.0),
        )
        .expect("3D楕円作成に失敗");

        assert_eq!(ellipse_3d.center(), Point3D::new(0.0, 0.0, 0.0));
        assert!((ellipse_3d.major_radius() - 3.0).abs() < 1e-10);
        assert!((ellipse_3d.minor_radius() - 2.0).abs() < 1e-10);

        // 法線ベクトル（Z軸方向）
        let normal = ellipse_3d.normal();
        assert!(normal.x().abs() < 1e-10);
        assert!(normal.y().abs() < 1e-10);
        assert!((normal.z() - 1.0).abs() < 1e-10);

        // 局所座標系
        let (x_axis, y_axis, z_axis) = ellipse_3d.local_coordinate_system();

        // 直交性チェック
        assert!(x_axis.dot(&y_axis).abs() < 1e-10);
        assert!(y_axis.dot(&z_axis).abs() < 1e-10);
        assert!(z_axis.dot(&x_axis).abs() < 1e-10);

        // 平面上判定
        let point_on_plane = Point3D::new(1.0, 1.0, 0.0);
        assert!(ellipse_3d.point_on_plane(&point_on_plane, 1e-10));

        let point_off_plane = Point3D::new(1.0, 1.0, 1.0);
        assert!(!ellipse_3d.point_on_plane(&point_off_plane, 1e-10));
    }

    #[test]
    fn test_bbox_arc_ellipse_integration() {
        use crate::{Arc2D, Arc2DImpl, Ellipse, Ellipse2DImpl};

        // 円弧の境界ボックス（概算）
        let arc = Arc2DImpl::from_center_radius_angles(
            Point2D::new(0.0, 0.0),
            2.0,
            Angle::<f64>::from_degrees(0.0),
            Angle::<f64>::from_degrees(90.0),
        )
        .expect("円弧作成に失敗");

        // 円弧の端点とその周辺
        let start_point = arc.start_point();
        let end_point = arc.end_point();
        let mid_point = arc.midpoint();

        // これらの点を包含する境界ボックス
        let points = [start_point, end_point, mid_point];
        let bbox = BBox2D::from_points(&points).unwrap();

        // すべての点が境界ボックス内にあることを確認
        for &point in &points {
            assert!(bbox.contains_point(point));
        }

        // 楕円の境界ボックス
        let ellipse = Ellipse2DImpl::axis_aligned(
            Point2D::new(0.0, 0.0),
            3.0, // major radius
            2.0, // minor radius
        )
        .expect("楕円作成に失敗");

        let ellipse_bbox = ellipse.bounding_box();

        // 楕円の特徴点が境界ボックス内にあることを確認
        assert!(ellipse_bbox.contains_point(ellipse.center()));
        assert!(ellipse_bbox.contains_point(Point2D::new(3.0, 0.0))); // right
        assert!(ellipse_bbox.contains_point(Point2D::new(-3.0, 0.0))); // left
        assert!(ellipse_bbox.contains_point(Point2D::new(0.0, 2.0))); // top
        assert!(ellipse_bbox.contains_point(Point2D::new(0.0, -2.0))); // bottom
    }

    #[test]
    fn test_mixed_geometry_workflow() {
        use crate::{Arc2D, Arc2DImpl, Circle2DImpl, Ellipse, Ellipse2DImpl};

        // 複合的な幾何ワークフロー

        // 1. 円から始める
        let circle = Circle2DImpl::<f64>::new(Point2D::new(2.0, 3.0), 4.0);
        let circle_center = circle.center();
        let circle_radius = circle.radius();

        // 2. 円から円弧を作成
        let arc = Arc2DImpl::new(
            circle,
            Angle::<f64>::from_degrees(0.0),
            Angle::<f64>::from_degrees(180.0),
        )
        .expect("円弧作成に失敗");

        // 3. 楕円を作成（同じ中心、異なる軸長）
        // 円の半径は4.0なので、楕円の両軸を円より大きくする
        let ellipse = Ellipse2DImpl::axis_aligned(
            arc.center(),
            6.0, // major radius - 円の半径4.0より大きい
            5.0, // minor radius - 円の半径4.0より大きい
        )
        .expect("楕円作成に失敗");

        // 4. 各形状の関係性チェック

        // 共通中心
        assert_eq!(circle_center, arc.center());
        assert_eq!(arc.center(), ellipse.center());

        // 円弧の端点が楕円内にあるかチェック
        let start_point = arc.start_point();
        let end_point = arc.end_point();

        // 楕円が円より大きいので、円弧の端点は楕円内にある
        assert!(ellipse.contains_point(&start_point));
        assert!(ellipse.contains_point(&end_point));

        // 面積比較
        let circle_area = PI * circle_radius * circle_radius;
        let ellipse_area = ellipse.area();
        let arc_length = arc.arc_length();

        // 楕円の面積が円より大きい（長軸が円の半径より大きいため）
        assert!(ellipse_area > circle_area);

        // 半円の弧長はπr
        let expected_arc_length = PI * circle_radius;
        assert!((arc_length - expected_arc_length).abs() < 1e-10);

        // 5. 境界ボックスの統合
        let circle_bbox = BBox2D::from_coords(
            circle_center.x() - circle_radius,
            circle_center.y() - circle_radius,
            circle_center.x() + circle_radius,
            circle_center.y() + circle_radius,
        );
        let ellipse_bbox = ellipse.bounding_box();

        // 楕円の境界ボックスが円の境界ボックスを包含
        assert!(ellipse_bbox.contains_bbox(&circle_bbox));
    }

    #[test]
    fn test_precision_consistency_across_geometries() {
        use crate::{Arc2D, Arc2DImpl, Circle2DImpl, Ellipse, Ellipse2DImpl};

        // f32とf64での一貫性テスト

        // Circle
        let circle_f32 = Circle2DImpl::<f32>::new(Point2D::new(1.0, 2.0), 3.0);
        let circle_f64 = Circle2DImpl::<f64>::new(Point2D::new(1.0, 2.0), 3.0);

        let angle = Angle::<f64>::from_degrees(45.0);
        let angle_f32 = Angle::<f32>::from_degrees(45.0);

        let point_f32 = circle_f32.point_at_angle(angle_f32);
        let point_f64 = circle_f64.point_at_angle(angle);

        // 精度を考慮した比較
        assert!((point_f32.x() as f64 - point_f64.x()).abs() < 1e-6);
        assert!((point_f32.y() as f64 - point_f64.y()).abs() < 1e-6);

        // Arc
        let arc_f64 = Arc2DImpl::new(
            circle_f64,
            Angle::<f64>::from_degrees(0.0),
            Angle::<f64>::from_degrees(90.0),
        )
        .expect("f64円弧作成に失敗");

        let arc_length_f64 = arc_f64.arc_length();
        let expected_length = 3.0 * PI / 2.0;
        assert!((arc_length_f64 - expected_length).abs() < 1e-10);

        // Ellipse
        let ellipse_f64 =
            Ellipse2DImpl::axis_aligned(Point2D::new(1.0, 2.0), 4.0, 3.0).expect("楕円作成に失敗");

        let ellipse_area = ellipse_f64.area();
        let expected_area = PI * 4.0 * 3.0;
        assert!((ellipse_area - expected_area).abs() < 1e-10);
    }
}
