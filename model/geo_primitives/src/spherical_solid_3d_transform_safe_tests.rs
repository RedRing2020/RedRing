//! SphericalSolid3D の安全な変換操作のテスト
//!
//! スケール制限、トレランスベースの検証、エラーケースのテスト
//!
//! **作成日: 2025年11月8日**
//! **最終更新: 2025年11月8日**

#[cfg(test)]
mod tests {
    use crate::{Point3D, SphericalSolid3D, Vector3D};
    use geo_foundation::{extensions::transform_error::TransformError, GeometricTolerance};

    /// テスト用の標準球固体を作成
    fn create_test_sphere() -> SphericalSolid3D<f64> {
        SphericalSolid3D::new(
            Point3D::new(2.0, 3.0, 4.0),
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            5.0,
        )
        .unwrap()
    }

    /// 最小サイズ球固体を作成（トレランス近く）
    fn create_minimal_sphere() -> SphericalSolid3D<f64> {
        let tolerance = <f64 as GeometricTolerance>::DISTANCE_TOLERANCE;
        let radius = tolerance * 10.0; // トレランスの10倍
        SphericalSolid3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            radius,
        )
        .unwrap()
    }

    // ========================================================================
    // 原点中心スケールのテスト
    // ========================================================================

    #[test]
    fn test_safe_scale_origin_success() {
        let sphere = create_test_sphere();
        let result = sphere.safe_scale_origin(2.0).unwrap();

        let tolerance = 1e-10;
        // 中心と半径がともに2倍
        assert!((result.center().x() - 4.0).abs() < tolerance);
        assert!((result.center().y() - 6.0).abs() < tolerance);
        assert!((result.center().z() - 8.0).abs() < tolerance);
        assert!((result.radius() - 10.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_scale_origin_zero_factor() {
        let sphere = create_test_sphere();
        let result = sphere.safe_scale_origin(0.0);

        assert!(matches!(result, Err(TransformError::InvalidScaleFactor(_))));
    }

    #[test]
    fn test_safe_scale_origin_negative_factor() {
        let sphere = create_test_sphere();
        let result = sphere.safe_scale_origin(-2.0);

        assert!(matches!(result, Err(TransformError::InvalidScaleFactor(_))));
    }

    #[test]
    fn test_safe_scale_origin_infinite_factor() {
        let sphere = create_test_sphere();
        let result = sphere.safe_scale_origin(f64::INFINITY);

        assert!(matches!(result, Err(TransformError::InvalidScaleFactor(_))));
    }

    #[test]
    fn test_safe_scale_origin_nan_factor() {
        let sphere = create_test_sphere();
        let result = sphere.safe_scale_origin(f64::NAN);

        assert!(matches!(result, Err(TransformError::InvalidScaleFactor(_))));
    }

    // ========================================================================
    // 指定点中心スケールのテスト
    // ========================================================================

    #[test]
    fn test_safe_scale_success() {
        let sphere = create_test_sphere();
        let scale_center = Point3D::new(0.0, 0.0, 0.0);
        let result = sphere.safe_scale(scale_center, 1.5).unwrap();

        let tolerance = 1e-10;
        // 中心が1.5倍、半径が1.5倍
        assert!((result.center().x() - 3.0).abs() < tolerance);
        assert!((result.center().y() - 4.5).abs() < tolerance);
        assert!((result.center().z() - 6.0).abs() < tolerance);
        assert!((result.radius() - 7.5).abs() < tolerance);
    }

    #[test]
    fn test_safe_scale_invalid_center() {
        let sphere = create_test_sphere();
        let invalid_center = Point3D::new(f64::NAN, 0.0, 0.0);
        let result = sphere.safe_scale(invalid_center, 2.0);

        assert!(matches!(result, Err(TransformError::InvalidGeometry(_))));
    }

    // ========================================================================
    // 半径のみスケールのテスト
    // ========================================================================

    #[test]
    fn test_safe_scale_radius_success() {
        let sphere = create_test_sphere();
        let result = sphere.safe_scale_radius(1.5).unwrap();

        let tolerance = 1e-10;
        // 中心は変わらず、半径のみ1.5倍
        assert!((result.center().x() - 2.0).abs() < tolerance);
        assert!((result.center().y() - 3.0).abs() < tolerance);
        assert!((result.center().z() - 4.0).abs() < tolerance);
        assert!((result.radius() - 7.5).abs() < tolerance);
    }

    #[test]
    fn test_safe_scale_radius_negative_factor() {
        let sphere = create_test_sphere();
        let result = sphere.safe_scale_radius(-1.0);

        assert!(matches!(result, Err(TransformError::InvalidScaleFactor(_))));
    }

    // ========================================================================
    // トレランスベース制限のテスト
    // ========================================================================

    #[test]
    fn test_scale_below_tolerance_rejected() {
        let sphere = create_minimal_sphere();
        let tolerance = <f64 as GeometricTolerance>::DISTANCE_TOLERANCE;

        // 半径をトレランス以下にするスケール倍率
        let bad_factor = tolerance / (sphere.radius() * 2.0);
        let result = sphere.safe_scale_radius(bad_factor);

        assert!(matches!(result, Err(TransformError::InvalidGeometry(_))));
    }

    #[test]
    fn test_scale_just_above_tolerance_succeeds() {
        let sphere = create_minimal_sphere();
        let tolerance = <f64 as GeometricTolerance>::DISTANCE_TOLERANCE;

        // 半径をトレランスより少し大きくするスケール倍率
        let good_factor = (tolerance * 2.0) / sphere.radius();
        let result = sphere.safe_scale_radius(good_factor);

        assert!(result.is_ok());
        assert!(result.unwrap().radius() > tolerance);
    }

    // ========================================================================
    // 新しい半径設定のテスト
    // ========================================================================

    #[test]
    fn test_safe_with_radius_success() {
        let sphere = create_test_sphere();
        let result = sphere.safe_with_radius(8.0).unwrap();

        let tolerance = 1e-10;
        // 中心・座標系は変わらず、半径のみ変更
        assert!((result.center().x() - 2.0).abs() < tolerance);
        assert!((result.center().y() - 3.0).abs() < tolerance);
        assert!((result.center().z() - 4.0).abs() < tolerance);
        assert!((result.radius() - 8.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_with_radius_below_tolerance() {
        let sphere = create_test_sphere();
        let tolerance = <f64 as GeometricTolerance>::DISTANCE_TOLERANCE;
        let result = sphere.safe_with_radius(tolerance / 2.0);

        assert!(matches!(result, Err(TransformError::InvalidGeometry(_))));
    }

    #[test]
    fn test_safe_with_radius_negative() {
        let sphere = create_test_sphere();
        let result = sphere.safe_with_radius(-5.0);

        assert!(matches!(result, Err(TransformError::InvalidScaleFactor(_))));
    }

    // ========================================================================
    // 検証メソッドのテスト
    // ========================================================================

    #[test]
    fn test_validate_scale_factor() {
        let sphere = create_test_sphere();

        // 有効なスケール倍率
        assert!(sphere.validate_scale_factor(2.0));
        assert!(sphere.validate_scale_factor(0.5));

        // 無効なスケール倍率
        assert!(!sphere.validate_scale_factor(0.0));
        assert!(!sphere.validate_scale_factor(-1.0));
        assert!(!sphere.validate_scale_factor(f64::NAN));
        assert!(!sphere.validate_scale_factor(f64::INFINITY));
    }

    #[test]
    fn test_validate_scale_factor_tolerance_limit() {
        let sphere = create_minimal_sphere();
        let tolerance = <f64 as GeometricTolerance>::DISTANCE_TOLERANCE;

        // トレランス制限を下回るスケール倍率は無効
        let bad_factor = tolerance / (sphere.radius() * 2.0);
        assert!(!sphere.validate_scale_factor(bad_factor));

        // トレランス制限を上回るスケール倍率は有効
        let good_factor = (tolerance * 2.0) / sphere.radius();
        assert!(sphere.validate_scale_factor(good_factor));
    }

    #[test]
    fn test_minimum_scale_factor() {
        let sphere = create_test_sphere();
        let min_factor = sphere.minimum_scale_factor();

        // 最小スケール倍率は正の値
        assert!(min_factor > 0.0);

        // 最小スケール倍率は有効
        assert!(sphere.validate_scale_factor(min_factor));

        // 最小スケール倍率より少し小さい値は無効
        let slightly_smaller = min_factor * 0.99;
        assert!(!sphere.validate_scale_factor(slightly_smaller));
    }

    #[test]
    fn test_minimum_scale_factor_minimal_sphere() {
        let sphere = create_minimal_sphere();
        let min_factor = sphere.minimum_scale_factor();
        let tolerance = <f64 as GeometricTolerance>::DISTANCE_TOLERANCE;

        // 最小球の場合、最小スケール倍率は比較的大きくなる
        // 浮動小数点精度を考慮して少し小さい値で検証
        assert!(min_factor >= 0.099); // 約10分の1

        // 最小スケール倍率を適用した結果の半径はトレランスより大きい
        let scaled_radius = sphere.radius() * min_factor;
        assert!(scaled_radius >= tolerance);
    }

    // ========================================================================
    // エラーメッセージのテスト
    // ========================================================================

    #[test]
    fn test_error_messages() {
        let sphere = create_test_sphere();

        // ゼロスケール倍率のエラーメッセージ
        if let Err(TransformError::InvalidScaleFactor(msg)) = sphere.safe_scale_origin(0.0) {
            assert!(msg.contains("正の有限値"));
        } else {
            panic!("Expected InvalidScaleFactor error");
        }

        // トレランス以下のエラーメッセージ
        let tolerance = <f64 as GeometricTolerance>::DISTANCE_TOLERANCE;
        if let Err(TransformError::InvalidGeometry(msg)) = sphere.safe_with_radius(tolerance / 2.0)
        {
            assert!(msg.contains("トレランス"));
        } else {
            panic!("Expected InvalidGeometry error");
        }
    }
}
