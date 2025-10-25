//! EllipseArc3D実装のテスト

#[cfg(test)]
mod tests {
    use crate::{Arc3D, Direction3D, Ellipse3D, EllipseArc3D, Point3D, Vector3D};
    use geo_foundation::Angle;

    fn create_test_ellipse_arc() -> EllipseArc3D<f64> {
        let center = Point3D::new(2.0, 3.0, 1.0);
        let normal = Vector3D::new(0.0, 0.0, 1.0); // Z軸方向
        let major_axis = Vector3D::new(1.0, 0.0, 0.0); // X軸方向
        let ellipse = Ellipse3D::new(center, 4.0, 2.0, normal, major_axis).unwrap();

        EllipseArc3D::new(
            ellipse,
            Angle::from_degrees(45.0),
            Angle::from_degrees(135.0),
        )
    }

    #[test]
    fn test_ellipse_arc_creation() {
        let ellipse_arc = create_test_ellipse_arc();

        let tolerance = 1e-10_f64; // 基礎レイヤーでは標準的な数値精度を使用
                                   // 基本プロパティのテスト
        assert!((ellipse_arc.center().x() - 2.0_f64).abs() < tolerance);
        assert!((ellipse_arc.center().y() - 3.0_f64).abs() < tolerance);
        assert!((ellipse_arc.center().z() - 1.0_f64).abs() < tolerance);
        assert!((ellipse_arc.semi_major() - 4.0_f64).abs() < tolerance);
        assert!((ellipse_arc.semi_minor() - 2.0_f64).abs() < tolerance);
        assert!((ellipse_arc.start_angle().to_degrees() - 45.0).abs() < tolerance);
        assert!((ellipse_arc.end_angle().to_degrees() - 135.0).abs() < tolerance);
    }

    #[test]
    fn test_from_arc() {
        // Arc3D を直接作成
        let center = Point3D::new(1.0, 2.0, 3.0);
        let normal = Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap();
        let start_dir = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();

        let arc = Arc3D::new(
            center,
            3.0,
            normal,
            start_dir,
            Angle::from_degrees(0.0),
            Angle::from_degrees(90.0),
        )
        .unwrap();

        let ellipse_arc = EllipseArc3D::from_arc(arc).unwrap();

        let tolerance = 1e-10_f64; // 基礎レイヤーでは標準的な数値精度を使用
        assert!((ellipse_arc.center().x() - 1.0_f64).abs() < tolerance);
        assert!((ellipse_arc.semi_major() - 3.0_f64).abs() < tolerance);
        assert!((ellipse_arc.semi_minor() - 3.0_f64).abs() < tolerance);
        assert!(ellipse_arc.is_circular());
    }

    #[test]
    fn test_angle_span() {
        let ellipse_arc = create_test_ellipse_arc(); // 45度-135度
        let expected_span = 90.0_f64.to_radians();
        let tolerance = 1e-10_f64; // 基礎レイヤーでは標準的な数値精度を使用
        assert!((ellipse_arc.angle_span() - expected_span).abs() < tolerance);
    }

    #[test]
    fn test_arc_points() {
        let ellipse_arc = create_test_ellipse_arc();

        let start_point = ellipse_arc.start_point();
        let end_point = ellipse_arc.end_point();
        let midpoint = ellipse_arc.midpoint();

        // 点が有限値であることを確認
        assert!(start_point.x().is_finite());
        assert!(start_point.y().is_finite());
        assert!(start_point.z().is_finite());
        assert!(end_point.x().is_finite());
        assert!(end_point.y().is_finite());
        assert!(end_point.z().is_finite());
        assert!(midpoint.x().is_finite());
        assert!(midpoint.y().is_finite());
        assert!(midpoint.z().is_finite());
    }

    #[test]
    fn test_basic_transforms() {
        let ellipse_arc = create_test_ellipse_arc();

        let tolerance = 1e-10_f64; // 基礎レイヤーでは標準的な数値精度を使用

        // 平行移動
        let translation = Vector3D::new(1.0, -1.0, 2.0);
        let translated = ellipse_arc.translate(translation);
        assert!((translated.center().x() - 3.0_f64).abs() < tolerance);
        assert!((translated.center().y() - 2.0_f64).abs() < tolerance);
        assert!((translated.center().z() - 3.0_f64).abs() < tolerance);

        // スケール（簡易版のため現在はそのまま）
        let scaled = ellipse_arc.scale(2.0).unwrap();
        assert!((scaled.semi_major() - 8.0_f64).abs() < tolerance);
        assert!((scaled.semi_minor() - 4.0_f64).abs() < tolerance);
    }

    #[test]
    fn test_arc_modifications() {
        let ellipse_arc = create_test_ellipse_arc();

        let tolerance = 1e-10_f64; // 基礎レイヤーでは標準的な数値精度を使用

        // 角度変更
        let new_arc = ellipse_arc.with_angles(Angle::from_degrees(0.0), Angle::from_degrees(180.0));
        assert!((new_arc.start_angle().to_degrees() - 0.0_f64).abs() < tolerance);
        assert!((new_arc.end_angle().to_degrees() - 180.0_f64).abs() < tolerance);

        // 向き反転
        let reversed = ellipse_arc.reverse();
        assert!((reversed.start_angle().to_degrees() - 135.0).abs() < tolerance);
        assert!((reversed.end_angle().to_degrees() - 45.0).abs() < tolerance);
    }

    #[test]
    fn test_sub_arc() {
        let ellipse_arc = create_test_ellipse_arc(); // 45度-135度

        // 有効な部分弧
        let sub_arc = ellipse_arc.sub_arc(Angle::from_degrees(60.0), Angle::from_degrees(120.0));
        assert!(sub_arc.is_some());
        let sub = sub_arc.unwrap();

        // 角度比較には適切な許容誤差を使用（基礎レイヤーなので標準ライブラリ定数ベース）
        let tolerance = 1e-10_f64; // 角度計算用の適切な許容誤差
        assert!((sub.start_angle().to_degrees() - 60.0_f64).abs() < tolerance);
        assert!((sub.end_angle().to_degrees() - 120.0_f64).abs() < tolerance);

        // 範囲外の部分弧
        let invalid_sub = ellipse_arc.sub_arc(
            Angle::from_degrees(30.0), // 45度未満
            Angle::from_degrees(120.0),
        );
        assert!(invalid_sub.is_none());
    }

    #[test]
    fn test_validation() {
        let ellipse_arc = create_test_ellipse_arc();
        assert!(ellipse_arc.is_valid());

        // デフォルト楕円弧の検証
        let default_arc = EllipseArc3D::<f64>::default();
        assert!(default_arc.is_valid());
    }

    #[test]
    fn test_bounding_box() {
        let ellipse_arc = create_test_ellipse_arc();
        let bbox = ellipse_arc.bounding_box();

        // バウンディングボックスが有効であることを確認
        assert!(bbox.min().x().is_finite());
        assert!(bbox.max().x().is_finite());
        assert!(bbox.min().y().is_finite());
        assert!(bbox.max().y().is_finite());
        assert!(bbox.min().z().is_finite());
        assert!(bbox.max().z().is_finite());
    }
}
