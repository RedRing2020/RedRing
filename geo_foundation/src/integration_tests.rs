/// geo_foundation統合テスト
///
/// Scalar trait、Angle、Circle、Arc、Ellipse実装の統合動作確認を行います。
/// f32/f64両方での動作と相互運用性をテストします。
use crate::{Angle, BoundingBox2D, BoundingBox3D, Circle, Circle2D, Circle2DImpl, Point2D, Point3D, Scalar, Vector2D, Vector3D};

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_scalar_trait_f32_f64_compatibility() {
        // f32 Scalar trait テスト
        let a_f32 = 3.0f32;
        let b_f32 = 4.0f32;
        let hypotenuse_f32 = (a_f32 * a_f32 + b_f32 * b_f32).sqrt();
        assert!((hypotenuse_f32 - 5.0f32).abs() < f32::TOLERANCE);

        // f64 Scalar trait テスト
        let a_f64 = 3.0f64;
        let b_f64 = 4.0f64;
        let hypotenuse_f64 = (a_f64 * a_f64 + b_f64 * b_f64).sqrt();
        assert!((hypotenuse_f64 - 5.0f64).abs() < f64::TOLERANCE);

        // 型変換テスト
        let f32_val = 3.141592f32;
        let converted_to_f64 = f32_val.to_f64();
        let back_to_f32 = f32::from_f64(converted_to_f64);
        assert!((f32_val - back_to_f32).abs() < f32::TOLERANCE);
    }

    #[test]
    fn test_angle_f32_operations() {
        let angle1 = Angle::<f32>::from_degrees(30.0);
        let angle2 = Angle::<f32>::from_degrees(60.0);

        // 加算
        let sum = angle1 + angle2;
        assert!((sum.to_degrees() - 90.0).abs() < 1e-5);

        // 減算
        let diff = angle2 - angle1;
        assert!((diff.to_degrees() - 30.0).abs() < 1e-5);

        // 正規化
        let large_angle = Angle::<f32>::from_degrees(450.0);
        let normalized = large_angle.normalize_0_2pi();
        let normalized_degrees = normalized.to_degrees();
        eprintln!("450° normalized to 0-2π: {}°", normalized_degrees);
        assert!(
            (normalized_degrees - 90.0).abs() < 1e-4,
            "Expected ~90°, got {}°",
            normalized_degrees
        );

        // 三角関数
        let right_angle = Angle::<f32>::from_degrees(90.0);
        assert!((right_angle.sin() - 1.0).abs() < 1e-5);
        assert!(right_angle.cos().abs() < 1e-5);
    }

    #[test]
    fn test_angle_f64_operations() {
        let angle1 = Angle::<f64>::from_degrees(45.0);
        let angle2 = Angle::<f64>::from_radians(std::f64::consts::PI / 4.0);

        // 度数とラジアンの等価性
        assert!(angle1.approx_eq(angle2));

        // 角度差計算
        let angle_10 = Angle::<f64>::from_degrees(10.0);
        let angle_350 = Angle::<f64>::from_degrees(350.0);
        let shortest_diff = angle_10.difference(angle_350);
        assert!((shortest_diff.to_degrees() - 20.0).abs() < 1e-10);

        // π/2の三角関数値
        let quarter_turn = Angle::<f64>::from_radians(std::f64::consts::PI / 2.0);
        assert!((quarter_turn.sin() - 1.0).abs() < 1e-10);
        assert!(quarter_turn.cos().abs() < 1e-10);
    }

    #[test]
    fn test_circle2d_f32_basic_operations() {
        let circle = Circle2DImpl::<f32>::new(Point2D::new(0.0, 0.0), 5.0);

        // 基本プロパティ
        assert_eq!(circle.center(), Point2D::new(0.0, 0.0));
        assert_eq!(circle.radius(), 5.0);
        assert!(circle.is_valid());

        // 面積と円周
        let expected_area = f32::PI * 25.0;
        assert!((circle.area() - expected_area).abs() < 1e-5);

        let expected_circumference = f32::TAU * 5.0;
        assert!((circle.circumference() - expected_circumference).abs() < 1e-5);

        // 点の包含判定
        assert!(circle.contains_point(&Point2D::new(0.0, 0.0))); // 中心
        assert!(circle.contains_point(&Point2D::new(3.0, 4.0))); // 内部 (3-4-5三角形)
        assert!(!circle.contains_point(&Point2D::new(6.0, 0.0))); // 外部
        assert!(circle.point_on_circumference(&Point2D::new(5.0, 0.0))); // 円周上
    }

    #[test]
    fn test_circle2d_f64_advanced_operations() {
        let circle = Circle2DImpl::<f64>::new(Point2D::new(1.0, 1.0), 2.0);

        // 角度による点計算
        let angle_0 = Angle::<f64>::from_degrees(0.0);
        let point_0 = circle.point_at_angle(angle_0);
        let expected_point_0 = Point2D::new(3.0, 1.0); // center + radius * (cos(0), sin(0))
        assert!((point_0.x() - expected_point_0.x()).abs() < 1e-10);
        assert!((point_0.y() - expected_point_0.y()).abs() < 1e-10);

        let angle_90 = Angle::<f64>::from_degrees(90.0);
        let point_90 = circle.point_at_angle(angle_90);
        let expected_point_90 = Point2D::new(1.0, 3.0); // center + radius * (cos(90°), sin(90°))
        assert!((point_90.x() - expected_point_90.x()).abs() < 1e-10);
        assert!((point_90.y() - expected_point_90.y()).abs() < 1e-10);

        // 接線と法線
        let tangent = circle.tangent_at_angle(angle_0);
        let normal = circle.normal_at_angle(angle_0);

        // 接線と法線は垂直
        let dot_product = tangent.x() * normal.x() + tangent.y() * normal.y();
        assert!(dot_product.abs() < 1e-10);
    }

    #[test]
    fn test_circle_transformations() {
        let original = Circle2DImpl::<f64>::new(Point2D::new(2.0, 3.0), 4.0);

        // スケーリング
        let scaled = original.scaled(2.0);
        assert_eq!(scaled.center(), Point2D::new(2.0, 3.0));
        assert_eq!(scaled.radius(), 8.0);

        // 平行移動
        let translated = original.translated(&Point2D::new(1.0, -1.0));
        assert_eq!(translated.center(), Point2D::new(3.0, 2.0));
        assert_eq!(translated.radius(), 4.0);

        // 距離計算
        let distance_to_center = original.distance_to_point(&Point2D::new(2.0, 3.0));
        assert!((distance_to_center - 4.0).abs() < 1e-10); // 中心からの距離は半径

        let distance_to_circumference = original.distance_to_point(&Point2D::new(6.0, 3.0));
        assert!(distance_to_circumference.abs() < 1e-10); // 円周上の点
    }

    #[test]
    fn test_three_point_circle_construction() {
        // 直角三角形の3頂点
        let p1 = Point2D::new(0.0, 0.0);
        let p2 = Point2D::new(4.0, 0.0);
        let p3 = Point2D::new(0.0, 3.0);

        let circle = Circle2DImpl::<f64>::from_three_points(p1, p2, p3).unwrap();

        // すべての点が円周上にあることを確認
        assert!(circle.point_on_circumference(&p1));
        assert!(circle.point_on_circumference(&p2));
        assert!(circle.point_on_circumference(&p3));

        // 直角三角形の外接円の半径は斜辺の半分
        let hypotenuse = ((4.0 * 4.0 + 3.0 * 3.0) as f64).sqrt();
        let expected_radius = hypotenuse / 2.0;
        assert!((circle.radius() - expected_radius).abs() < 1e-10);
    }

    #[test]
    fn test_bounding_circle() {
        let points = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 0.0),
            Point2D::new(0.0, 1.0),
            Point2D::new(1.0, 1.0),
        ];

        let bounding = Circle2DImpl::<f64>::bounding_circle(&points).unwrap();

        // すべての点が円内に含まれることを確認
        for &point in &points {
            assert!(bounding.contains_point(&point) || bounding.point_on_circumference(&point));
        }
    }

    #[test]
    fn test_f32_f64_mixed_workflow() {
        // f32で高速計算
        let angle_f32 = Angle::<f32>::from_degrees(45.0);
        let circle_f32 = Circle2DImpl::<f32>::new(Point2D::new(0.0, 0.0), 1.0);
        let point_f32 = circle_f32.point_at_angle(angle_f32);

        // f64で高精度計算
        let angle_f64 = Angle::<f64>::from_degrees(45.0);
        let circle_f64 = Circle2DImpl::<f64>::new(Point2D::new(0.0, 0.0), 1.0);
        let point_f64 = circle_f64.point_at_angle(angle_f64);

        // 結果の比較（精度の違いを考慮）
        let diff_x = (point_f32.x().to_f64() - point_f64.x()).abs();
        let diff_y = (point_f32.y().to_f64() - point_f64.y()).abs();
        assert!(diff_x < 1e-6); // f32精度レベルで近似
        assert!(diff_y < 1e-6);
    }

    #[test]
    fn test_mathematical_constants_consistency() {
        // PI定数の一貫性
        assert!((f32::PI.to_f64() - f64::PI).abs() < 1e-6);
        assert!((f32::TAU.to_f64() - f64::TAU).abs() < 1e-6);

        // 角度変換定数の一貫性
        let degrees_f32 = 180.0f32;
        let radians_f32 = degrees_f32 * f32::DEG_TO_RAD;
        assert!((radians_f32 - f32::PI).abs() < 1e-6);

        let degrees_f64 = 180.0f64;
        let radians_f64 = degrees_f64 * f64::DEG_TO_RAD;
        assert!((radians_f64 - f64::PI).abs() < 1e-10);
    }

    #[test]
    fn test_tolerance_handling() {
        // f32許容誤差
        let a_f32 = 1.0f32;
        let b_f32 = 1.0f32 + f32::TOLERANCE / 10.0;
        assert!(a_f32.approx_eq(b_f32));

        let c_f32 = 1.0f32 + f32::TOLERANCE * 10.0;
        assert!(!a_f32.approx_eq(c_f32));

        // f64許容誤差
        let a_f64 = 1.0f64;
        let b_f64 = 1.0f64 + f64::TOLERANCE / 10.0;
        assert!(a_f64.approx_eq(b_f64));

        let c_f64 = 1.0f64 + f64::TOLERANCE * 10.0;
        assert!(!a_f64.approx_eq(c_f64));

        // 角度の許容誤差
        let angle1 = Angle::<f64>::from_degrees(0.0);
        let angle2 = Angle::<f64>::from_degrees(360.0);
        assert!(angle1.approx_eq(angle2.normalize_0_2pi()));
    }

    #[test]
    fn test_edge_cases() {
        // ゼロ半径円（無効）
        let zero_circle = Circle2DImpl::<f64>::new(Point2D::new(0.0, 0.0), 0.0);
        assert!(!zero_circle.is_valid());

        // 単位円判定
        let unit_circle = Circle2DImpl::<f64>::new(Point2D::new(0.0, 0.0), 1.0);
        assert!(unit_circle.is_unit_circle());

        // 一直線上の3点（外接円なし）
        let p1 = Point2D::new(0.0, 0.0);
        let p2 = Point2D::new(1.0, 0.0);
        let p3 = Point2D::new(2.0, 0.0);
        assert!(Circle2DImpl::<f64>::from_three_points(p1, p2, p3).is_none());

        // 空の点配列（境界円なし）
        assert!(Circle2DImpl::<f64>::bounding_circle(&[]).is_none());

        // 1点の境界円（ゼロ半径円）
        let single_point_circle =
            Circle2DImpl::<f64>::bounding_circle(&[Point2D::new(1.0, 2.0)]).unwrap();
        assert_eq!(single_point_circle.center(), Point2D::new(1.0, 2.0));
        assert_eq!(single_point_circle.radius(), 0.0);
    }

    #[test]
    fn test_angle_constant_functions() {
        // 角度定数関数のテスト
        let zero = Angle::<f64>::zero();
        assert_eq!(zero.to_degrees(), 0.0);

        let right = Angle::<f64>::right_angle();
        assert!((right.to_degrees() - 90.0).abs() < 1e-10);

        let straight = Angle::<f64>::straight_angle();
        assert!((straight.to_degrees() - 180.0).abs() < 1e-10);

        let full = Angle::<f64>::full_angle();
        assert!((full.to_degrees() - 360.0).abs() < 1e-10);
    }

    #[test]
    fn test_complex_geometric_scenario() {
        // 複雑なシナリオ：複数の円と角度を組み合わせた計算

        // 中心が異なる2つの円
        let circle1 = Circle2DImpl::<f64>::new(Point2D::new(0.0, 0.0), 5.0);
        let circle2 = Circle2DImpl::<f64>::new(Point2D::new(10.0, 0.0), 3.0);

        // 各円上の特定角度の点を計算
        let angle = Angle::<f64>::from_degrees(30.0);
        let point1 = circle1.point_at_angle(angle);
        let point2 = circle2.point_at_angle(angle);

        // 2点間の距離を計算
        let dx = point2.x() - point1.x();
        let dy = point2.y() - point1.y();
        let distance = (dx * dx + dy * dy).sqrt();

        // 期待値と比較（30度での各円上の点の距離）
        let expected_distance = {
            let (cos30, sin30) = (angle.cos(), angle.sin());
            let p1 = (5.0 * cos30, 5.0 * sin30);
            let p2 = (10.0 + 3.0 * cos30, 3.0 * sin30);
            let dx = p2.0 - p1.0;
            let dy = p2.1 - p1.1;
            (dx * dx + dy * dy).sqrt()
        };

        assert!((distance - expected_distance).abs() < 1e-10);
    }

    #[test]
    fn test_vector_point_circle_integration() {
        // Vector、Point、Circleの統合動作テスト

        // 中心点とベクトルから円を構築
        let center = Point2D::new(2.0, 3.0);
        let radius_vector = Vector2D::new(4.0, 0.0);
        let radius = radius_vector.length();
        let circle = Circle2DImpl::<f64>::new(center, radius);

        // ベクトルの方向にある点が円周上にあることを確認
        let point_on_circle = Point2D::from(Vector2D::from(center) + radius_vector);
        assert!(circle.point_on_circumference(&point_on_circle));

        // 円の中心からベクトル方向への接線ベクトル
        let angle = radius_vector.angle();
        let tangent_angle = angle + std::f64::consts::PI / 2.0;
        let tangent_vector = Vector2D::from_angle(tangent_angle);

        // 接線ベクトルと半径ベクトルは垂直
        assert!((radius_vector.dot(&tangent_vector)).abs() < 1e-10);
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
        // BoundingBoxとPoint、Vectorの統合動作テスト
        
        // 複数の点から境界ボックスを作成
        let points = vec![
            Point2D::new(1.0, 2.0),
            Point2D::new(5.0, 1.0),
            Point2D::new(3.0, 6.0),
            Point2D::new(2.0, 4.0),
        ];
        
        let bbox = BoundingBox2D::from_points(&points).unwrap();
        
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
        // BoundingBoxとCircleの統合動作テスト
        
        let circle = Circle2DImpl::<f64>::new(Point2D::new(3.0, 2.0), 2.0);
        
        // 円の境界ボックスを手動作成
        let circle_bbox = BoundingBox2D::from_coords(1.0, 0.0, 5.0, 4.0);
        
        // 円の中心点が境界ボックス内にあることを確認
        assert!(circle_bbox.contains_point(circle.center()));
        
        // 円周上の点を生成して境界ボックステスト
        let angle_0 = Angle::<f64>::from_degrees(0.0);
        let angle_90 = Angle::<f64>::from_degrees(90.0);
        let angle_180 = Angle::<f64>::from_degrees(180.0);
        let angle_270 = Angle::<f64>::from_degrees(270.0);
        
        let point_0 = circle.point_at_angle(angle_0);   // 右端
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
        // 3D BoundingBoxの統合動作テスト
        
        let points_3d = vec![
            Point3D::new(1.0, 2.0, 1.0),
            Point3D::new(4.0, 1.0, 3.0),
            Point3D::new(2.0, 5.0, 2.0),
            Point3D::new(3.0, 3.0, 4.0),
        ];
        
        let bbox_3d = BoundingBox3D::from_points(&points_3d).unwrap();
        
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
        
        let bbox_f32 = BoundingBox2D::<f32>::from_coords(1.0, 2.0, 5.0, 6.0);
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
        let other_f32 = BoundingBox2D::<f32>::from_coords(3.0, 4.0, 7.0, 8.0);
        let other_f64 = other_f32.to_f64();
        
        assert_eq!(bbox_f32.intersects(&other_f32), bbox_f64.intersects(&other_f64));
    }

    #[test]
    fn test_bbox_union_intersection_complex() {
        // 複数境界ボックスの和集合・積集合テスト
        
        let bbox1 = BoundingBox2D::from_coords(0.0, 0.0, 4.0, 3.0);
        let bbox2 = BoundingBox2D::from_coords(2.0, 1.0, 6.0, 5.0);
        let bbox3 = BoundingBox2D::from_coords(1.0, 2.0, 3.0, 4.0);
        
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
        let bbox_disjoint = BoundingBox2D::from_coords(7.0, 6.0, 9.0, 8.0);
        assert!(bbox1.intersection(&bbox_disjoint).is_none());
    }

    // Arc/Ellipse統合テスト（新規追加）
    #[test]
    fn test_arc_circle_integration() {
        use crate::{Arc2DImpl, Arc2D, Circle2DImpl};
        
        // 円から円弧を作成
        let circle = Circle2DImpl::<f64>::new(Point2D::new(0.0, 0.0), 2.0);
        let start_angle = Angle::<f64>::from_degrees(0.0);
        let end_angle = Angle::<f64>::from_degrees(90.0);
        
        let arc = Arc2DImpl::new(circle, start_angle, end_angle)
            .expect("円弧作成に失敗");
        
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
        let expected_arc_length = 2.0 * std::f64::consts::PI * 2.0 / 4.0; // quarter circle: 2πr/4
        assert!((arc.arc_length() - expected_arc_length).abs() < 1e-10);
    }

    #[test]
    fn test_arc_point_containment_integration() {
        use crate::{Arc2DImpl, Arc2D, Circle2DImpl};
        
        let circle = Circle2DImpl::<f64>::new(Point2D::new(0.0, 0.0), 3.0);
        let arc = Arc2DImpl::new(
            circle,
            Angle::<f64>::from_degrees(30.0),
            Angle::<f64>::from_degrees(120.0)
        ).expect("円弧作成に失敗");
        
        let tolerance = 1e-10;
        
        // 円弧上の点（角度範囲内）
        let angle_60 = Angle::<f64>::from_degrees(60.0);
        let point_on_arc = Point2D::new(
            3.0 * angle_60.cos(),
            3.0 * angle_60.sin()
        );
        assert!(arc.contains_point(&point_on_arc, tolerance));
        
        // 円周上だが角度範囲外の点
        let angle_0 = Angle::<f64>::from_degrees(0.0);
        let point_outside_arc = Point2D::new(
            3.0 * angle_0.cos(),
            3.0 * angle_0.sin()
        );
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
        use crate::{Arc2DImpl, Arc2D, Circle2DImpl};
        
        let arc = Arc2DImpl::from_center_radius_angles(
            Point2D::new(0.0, 0.0),
            2.0,
            Angle::<f64>::from_degrees(0.0),
            Angle::<f64>::from_degrees(180.0)
        ).expect("円弧作成に失敗");
        
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
            Angle::<f64>::from_degrees(270.0)
        ).expect("他の円弧作成に失敗");
        
        assert!(arc.intersects_with_arc(&other_arc));
    }

    #[test]
    fn test_ellipse_creation_integration() {
        use crate::{Ellipse2DImpl, Ellipse, EllipseError};
        
        // 軸平行楕円の作成
        let ellipse = Ellipse2DImpl::axis_aligned(
            Point2D::new(1.0, 2.0),
            3.0, // major radius
            2.0  // minor radius
        ).expect("楕円作成に失敗");
        
        assert_eq!(ellipse.center(), Point2D::new(1.0, 2.0));
        assert_eq!(ellipse.major_radius(), 3.0);
        assert_eq!(ellipse.minor_radius(), 2.0);
        
        // 面積計算
        let expected_area = std::f64::consts::PI * 3.0 * 2.0;
        assert!((ellipse.area() - expected_area).abs() < 1e-10);
        
        // 離心率計算
        let expected_eccentricity = (1.0 - (2.0 * 2.0) / (3.0 * 3.0)).sqrt();
        assert!((ellipse.eccentricity() - expected_eccentricity).abs() < 1e-10);
        
        // エラーケースのテスト
        let invalid_result = Ellipse2DImpl::axis_aligned(
            Point2D::new(0.0, 0.0),
            1.0, // minor radius
            2.0  // major radius（長軸 < 短軸）
        );
        assert!(matches!(invalid_result, Err(EllipseError::InvalidAxisOrder)));
    }

    #[test]
    fn test_ellipse_point_operations() {
        use crate::{Ellipse2DImpl, Ellipse, Ellipse2D};
        
        let ellipse = Ellipse2DImpl::axis_aligned(
            Point2D::new(0.0, 0.0),
            4.0, // major radius
            3.0  // minor radius
        ).expect("楕円作成に失敗");
        
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
        use crate::{Ellipse3DImpl, Ellipse, Ellipse3D};
        
        // XY平面上の3D楕円
        let ellipse_3d = Ellipse3DImpl::from_radii_xy(
            Point3D::new(0.0, 0.0, 0.0),
            3.0, // major radius
            2.0, // minor radius
            Angle::<f64>::from_degrees(45.0)
        ).expect("3D楕円作成に失敗");
        
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
        use crate::{Arc2DImpl, Arc2D, Ellipse2DImpl, Ellipse};
        
        // 円弧の境界ボックス（概算）
        let arc = Arc2DImpl::from_center_radius_angles(
            Point2D::new(0.0, 0.0),
            2.0,
            Angle::<f64>::from_degrees(0.0),
            Angle::<f64>::from_degrees(90.0)
        ).expect("円弧作成に失敗");
        
        // 円弧の端点とその周辺
        let start_point = arc.start_point();
        let end_point = arc.end_point();
        let mid_point = arc.midpoint();
        
        // これらの点を包含する境界ボックス
        let points = [start_point, end_point, mid_point];
        let bbox = BoundingBox2D::from_points(&points).unwrap();
        
        // すべての点が境界ボックス内にあることを確認
        for &point in &points {
            assert!(bbox.contains_point(point));
        }
        
        // 楕円の境界ボックス
        let ellipse = Ellipse2DImpl::axis_aligned(
            Point2D::new(0.0, 0.0),
            3.0, // major radius
            2.0  // minor radius
        ).expect("楕円作成に失敗");
        
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
        use crate::{Arc2DImpl, Arc2D, Circle2DImpl, Ellipse2DImpl, Ellipse};
        
        // 複合的な幾何ワークフロー
        
        // 1. 円から始める
        let circle = Circle2DImpl::<f64>::new(Point2D::new(2.0, 3.0), 4.0);
        let circle_center = circle.center();
        let circle_radius = circle.radius();
        
        // 2. 円から円弧を作成
        let arc = Arc2DImpl::new(
            circle,
            Angle::<f64>::from_degrees(0.0),
            Angle::<f64>::from_degrees(180.0)
        ).expect("円弧作成に失敗");
        
        // 3. 楕円を作成（同じ中心、異なる軸長）
        // 円の半径は4.0なので、楕円の両軸を円より大きくする
        let ellipse = Ellipse2DImpl::axis_aligned(
            arc.center(),
            6.0, // major radius - 円の半径4.0より大きい
            5.0  // minor radius - 円の半径4.0より大きい
        ).expect("楕円作成に失敗");
        
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
        let circle_area = std::f64::consts::PI * circle_radius * circle_radius;
        let ellipse_area = ellipse.area();
        let arc_length = arc.arc_length();
        
        // 楕円の面積が円より大きい（長軸が円の半径より大きいため）
        assert!(ellipse_area > circle_area);
        
        // 半円の弧長はπr
        let expected_arc_length = std::f64::consts::PI * circle_radius;
        assert!((arc_length - expected_arc_length).abs() < 1e-10);
        
        // 5. 境界ボックスの統合
        let circle_bbox = BoundingBox2D::from_coords(
            circle_center.x() - circle_radius,
            circle_center.y() - circle_radius,
            circle_center.x() + circle_radius,
            circle_center.y() + circle_radius
        );
        let ellipse_bbox = ellipse.bounding_box();
        
        // 楕円の境界ボックスが円の境界ボックスを包含
        assert!(ellipse_bbox.contains_bbox(&circle_bbox));
    }

    #[test]
    fn test_precision_consistency_across_geometries() {
        use crate::{Arc2DImpl, Arc2D, Circle2DImpl, Ellipse2DImpl, Ellipse};
        
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
            Angle::<f64>::from_degrees(90.0)
        ).expect("f64円弧作成に失敗");
        
        let arc_length_f64 = arc_f64.arc_length();
        let expected_length = 3.0 * std::f64::consts::PI / 2.0;
        assert!((arc_length_f64 - expected_length).abs() < 1e-10);
        
        // Ellipse
        let ellipse_f64 = Ellipse2DImpl::axis_aligned(
            Point2D::new(1.0, 2.0),
            4.0,
            3.0
        ).expect("楕円作成に失敗");
        
        let ellipse_area = ellipse_f64.area();
        let expected_area = std::f64::consts::PI * 4.0 * 3.0;
        assert!((ellipse_area - expected_area).abs() < 1e-10);
    }
}
