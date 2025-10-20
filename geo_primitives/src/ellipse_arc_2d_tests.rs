//! EllipseArc2D実装のテスト

#[cfg(test)]
mod tests {
    use crate::ellipse_2d::Ellipse2D;
    use crate::ellipse_arc_2d::EllipseArc2D;
    use crate::Point2D;
    // use analysis::linalg::Matrix3x3;
    // use geo_foundation::extensions::{AdvancedTransform, BasicTransform};
    use geo_foundation::Angle;

    #[test]
    fn test_ellipse_arc_creation() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 5.0, 3.0, 0.0).expect("楕円の作成に失敗");
        let arc = EllipseArc2D::new(
            ellipse,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::PI),
        );

        assert_eq!(arc.ellipse().center(), center);
        assert_eq!(arc.start_angle().to_radians(), 0.0);
        assert_eq!(arc.end_angle().to_radians(), std::f64::consts::PI);
    }

    #[test]
    fn test_arc_start_end_points() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");
        let arc = EllipseArc2D::new(
            ellipse,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::PI / 2.0),
        );

        let start_point = arc.start_point();
        let end_point = arc.end_point();

        // 開始点（t=0, 長軸の正の端）
        assert!((start_point.x() - 4.0).abs() < 1e-10);
        assert!(start_point.y().abs() < 1e-10);

        // 終了点（t=π/2, 短軸の正の端）
        assert!(end_point.x().abs() < 1e-10);
        assert!((end_point.y() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_arc_point_at_parameter() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");
        let arc = EllipseArc2D::new(
            ellipse,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::PI),
        );

        // パラメータ0.0での点（開始点）
        let point_start = arc.point_at_parameter(0.0);
        let expected_start = arc.start_point();
        assert!((point_start.x() - expected_start.x()).abs() < 1e-10);
        assert!((point_start.y() - expected_start.y()).abs() < 1e-10);

        // パラメータ1.0での点（終了点）
        let point_end = arc.point_at_parameter(1.0);
        let expected_end = arc.end_point();
        assert!((point_end.x() - expected_end.x()).abs() < 1e-10);
        assert!((point_end.y() - expected_end.y()).abs() < 1e-10);
    }

    #[test]
    fn test_arc_length_calculation() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");

        // 半楕円
        let half_arc = EllipseArc2D::new(
            ellipse,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::PI),
        );
        let half_length = half_arc.arc_length();

        // 全楕円
        let full_arc = EllipseArc2D::new(
            ellipse,
            Angle::from_radians(0.0),
            Angle::from_radians(2.0 * std::f64::consts::PI),
        );
        let full_length = full_arc.arc_length();

        // 半楕円の長さは全楕円の約半分
        assert!((full_length - 2.0 * half_length).abs() < 0.1);
        assert!(half_length > 0.0);
        assert!(full_length > 0.0);
    }

    #[test]
    fn test_arc_subdivision() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");
        let arc = EllipseArc2D::new(
            ellipse,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::PI / 2.0),
        );

        let points = arc.subdivide(4);
        assert_eq!(points.len(), 5); // 4セグメント = 5点

        // 最初と最後の点が開始点と終了点に一致
        let first_point = &points[0];
        let last_point = &points[points.len() - 1];
        let start_point = arc.start_point();
        let end_point = arc.end_point();

        assert!((first_point.x() - start_point.x()).abs() < 1e-10);
        assert!((first_point.y() - start_point.y()).abs() < 1e-10);
        assert!((last_point.x() - end_point.x()).abs() < 1e-10);
        assert!((last_point.y() - end_point.y()).abs() < 1e-10);
    }

    #[test]
    fn test_arc_angle_span() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");
        let arc = EllipseArc2D::new(
            ellipse,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::PI / 2.0),
        );

        let span = arc.angle_span();
        assert!((span.to_radians() - std::f64::consts::PI / 2.0).abs() < 1e-10);
    }
}

// ============================================================================
// Transform Tests
// ============================================================================

#[cfg(test)]
mod ellipse_arc_2d_transform_tests {
    use crate::{Ellipse2D, EllipseArc2D, LineSegment2D, Point2D, Vector2D};
    use analysis::linalg::Matrix3x3;
    use geo_foundation::extensions::{AdvancedTransform, BasicTransform};
    use geo_foundation::Angle;

    #[test]
    fn test_basic_translate() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");
        let arc = EllipseArc2D::new(
            ellipse,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::PI / 2.0),
        );

        let translation = Vector2D::new(3.0, 2.0);
        let translated_arc = BasicTransform::translate(&arc, translation);

        // 基底楕円の中心が移動していることを確認
        let expected_center = Point2D::new(3.0, 2.0);
        assert!((translated_arc.ellipse().center().x() - expected_center.x()).abs() < 1e-10);
        assert!((translated_arc.ellipse().center().y() - expected_center.y()).abs() < 1e-10);

        // 角度範囲は変わらない
        assert!(
            (translated_arc.start_angle().to_radians() - arc.start_angle().to_radians()).abs()
                < 1e-10
        );
        assert!(
            (translated_arc.end_angle().to_radians() - arc.end_angle().to_radians()).abs() < 1e-10
        );

        // 軸の長さは変わらない
        assert!((translated_arc.ellipse().semi_major() - 4.0).abs() < 1e-10);
        assert!((translated_arc.ellipse().semi_minor() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_basic_rotate() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");
        let arc = EllipseArc2D::new(
            ellipse,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::PI / 2.0),
        );

        let rotation_center = Point2D::new(0.0, 0.0);
        let rotation_angle = Angle::from_radians(std::f64::consts::PI / 4.0);
        let rotated_arc = BasicTransform::rotate(&arc, rotation_center, rotation_angle);

        // 基底楕円の回転が適用されていることを確認
        let expected_rotation = std::f64::consts::PI / 4.0;
        assert!((rotated_arc.ellipse().rotation() - expected_rotation).abs() < 1e-10);

        // 角度範囲が回転していることを確認
        let expected_start = std::f64::consts::PI / 4.0;
        let expected_end = std::f64::consts::PI / 2.0 + std::f64::consts::PI / 4.0;
        assert!((rotated_arc.start_angle().to_radians() - expected_start).abs() < 1e-10);
        assert!((rotated_arc.end_angle().to_radians() - expected_end).abs() < 1e-10);
    }

    #[test]
    fn test_basic_scale() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");
        let arc = EllipseArc2D::new(
            ellipse,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::PI / 2.0),
        );

        let scale_center = Point2D::new(0.0, 0.0);
        let scale_factor = 2.0;
        let scaled_arc = BasicTransform::scale(&arc, scale_center, scale_factor);

        // 軸の長さがスケールされていることを確認
        assert!((scaled_arc.ellipse().semi_major() - 8.0).abs() < 1e-10);
        assert!((scaled_arc.ellipse().semi_minor() - 4.0).abs() < 1e-10);

        // 角度範囲は変わらない
        assert!(
            (scaled_arc.start_angle().to_radians() - arc.start_angle().to_radians()).abs() < 1e-10
        );
        assert!((scaled_arc.end_angle().to_radians() - arc.end_angle().to_radians()).abs() < 1e-10);
    }

    #[test]
    fn test_advanced_mirror() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");
        let arc = EllipseArc2D::new(
            ellipse,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::PI / 2.0),
        );

        // Y軸に対して鏡像反転
        let axis_start = Point2D::new(0.0, -1.0);
        let axis_end = Point2D::new(0.0, 1.0);
        let axis = LineSegment2D::new(axis_start, axis_end).expect("軸の作成に失敗");

        let mirrored_arc = AdvancedTransform::mirror(&arc, axis);

        // 鏡像反転後の中心はY軸に対して対称
        assert!((mirrored_arc.ellipse().center().x() - 0.0).abs() < 1e-10);
        assert!((mirrored_arc.ellipse().center().y() - 0.0).abs() < 1e-10);

        // 軸の長さは変わらない
        assert!((mirrored_arc.ellipse().semi_major() - 4.0).abs() < 1e-10);
        assert!((mirrored_arc.ellipse().semi_minor() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_advanced_scale_non_uniform() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");
        let arc = EllipseArc2D::new(
            ellipse,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::PI / 2.0),
        );

        let scale_center = Point2D::new(0.0, 0.0);
        let scale_x = 2.0;
        let scale_y = 0.5;
        let scaled_arc = AdvancedTransform::scale_non_uniform(&arc, scale_center, scale_x, scale_y);

        // 非等方スケール後は楕円の形状が変化
        // 詳細な検証は楕円の変換ロジックに依存
        assert!(scaled_arc.is_valid_after_transform());
    }

    #[test]
    fn test_advanced_reverse() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");
        let arc = EllipseArc2D::new(
            ellipse,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::PI / 2.0),
        );

        let reversed_arc = AdvancedTransform::reverse(&arc);

        // 開始角度と終了角度が交換されている
        assert!(
            (reversed_arc.start_angle().to_radians() - std::f64::consts::PI / 2.0).abs() < 1e-10
        );
        assert!((reversed_arc.end_angle().to_radians() - 0.0).abs() < 1e-10);

        // 基底楕円は変わらない
        assert!((reversed_arc.ellipse().center().x() - center.x()).abs() < 1e-10);
        assert!((reversed_arc.ellipse().center().y() - center.y()).abs() < 1e-10);
    }

    #[test]
    fn test_matrix_transform() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");
        let arc = EllipseArc2D::new(
            ellipse,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::PI / 2.0),
        );

        // 45度回転 + (1, 1)平行移動の変換行列
        let cos45 = (std::f64::consts::PI / 4.0).cos();
        let sin45 = (std::f64::consts::PI / 4.0).sin();
        let matrix = Matrix3x3::new(cos45, -sin45, 1.0, sin45, cos45, 1.0, 0.0, 0.0, 1.0);

        let transformed_arc = AdvancedTransform::transform_matrix(&arc, &matrix);

        // 変換後の妥当性確認
        assert!(transformed_arc.is_valid_after_transform());

        // 中心点が適切に変換されている
        let expected_center = Point2D::new(1.0, 1.0);
        let actual_center = transformed_arc.ellipse().center();
        assert!((actual_center.x() - expected_center.x()).abs() < 1e-10);
        assert!((actual_center.y() - expected_center.y()).abs() < 1e-10);
    }

    #[test]
    fn test_composite_transforms() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");
        let arc = EllipseArc2D::new(
            ellipse,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::PI / 2.0),
        );

        // 複合変換：平行移動 + 回転
        let translation = Vector2D::new(2.0, 1.0);
        let rotation_center = Point2D::new(2.0, 1.0);
        let rotation_angle = Angle::from_radians(std::f64::consts::PI / 6.0);

        let transformed_arc =
            arc.translate_and_rotate(translation, rotation_center, rotation_angle);

        // 変換後の妥当性確認
        assert!(transformed_arc.is_valid_after_transform());

        // 中心が移動している
        let actual_center = transformed_arc.ellipse().center();
        assert!((actual_center.x() - 2.0).abs() < 1e-10);
        assert!((actual_center.y() - 1.0).abs() < 1e-10);

        // 回転が適用されている
        let expected_rotation = std::f64::consts::PI / 6.0;
        assert!((transformed_arc.ellipse().rotation() - expected_rotation).abs() < 1e-10);
    }

    #[test]
    fn test_fast_transforms() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");
        let arc = EllipseArc2D::new(
            ellipse,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::PI / 2.0),
        );

        // 高速平行移動
        let translation = Vector2D::new(3.0, 2.0);
        let fast_translated = arc.fast_translate(translation);
        let regular_translated = BasicTransform::translate(&arc, translation);

        // 結果が同じであることを確認
        assert!(fast_translated.transform_equivalent(&regular_translated, 1e-10));

        // 高速等方スケール
        let scale_center = Point2D::new(0.0, 0.0);
        let scale_factor = 1.5;
        let fast_scaled = arc.fast_uniform_scale(scale_center, scale_factor);
        let regular_scaled = BasicTransform::scale(&arc, scale_center, scale_factor);

        // 結果が同じであることを確認
        assert!(fast_scaled.transform_equivalent(&regular_scaled, 1e-10));
    }

    #[test]
    fn test_transform_validation() {
        let center = Point2D::new(0.0, 0.0);
        let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");
        let arc = EllipseArc2D::new(
            ellipse,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::PI / 2.0),
        );

        // 正常な変換
        let translation = Vector2D::new(1.0, 1.0);
        let translated_arc = BasicTransform::translate(&arc, translation);
        assert!(translated_arc.is_valid_after_transform());

        // 等価性チェック
        let translated_arc2 = BasicTransform::translate(&arc, translation);
        assert!(translated_arc.transform_equivalent(&translated_arc2, 1e-10));
    }
}
