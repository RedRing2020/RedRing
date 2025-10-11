//! ToleranceSettings と GeometryContext のテスト

use crate::{GeometryContext, ToleranceSettings};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tolerance_settings() {
        // 異なる精度設定の作成
        let precision = ToleranceSettings::<f64>::precision();
        let standard = ToleranceSettings::<f64>::standard();
        let relaxed = ToleranceSettings::<f64>::relaxed();

        // 精度順序の確認
        assert!(precision.distance_tolerance < standard.distance_tolerance);
        assert!(standard.distance_tolerance < relaxed.distance_tolerance);

        assert!(precision.angle_tolerance < standard.angle_tolerance);
        assert!(standard.angle_tolerance < relaxed.angle_tolerance);

        // カスタム設定
        let custom = ToleranceSettings::custom(1e-8, 1e-6, 1e-7, 1e-9);
        assert_eq!(custom.distance_tolerance, 1e-8);
        assert_eq!(custom.angle_tolerance, 1e-6);
    }

    #[test]
    fn test_geometry_context() {
        let context = GeometryContext::<f64>::standard();
        assert!(context.tolerances.distance_tolerance > 0.0);
        assert!(context.tolerances.angle_tolerance > 0.0);

        // デフォルトは標準設定
        let default_context = GeometryContext::<f64>::default();
        assert_eq!(
            default_context.tolerances.distance_tolerance,
            context.tolerances.distance_tolerance
        );
    }

    #[test]
    fn test_f32_compatibility() {
        // f32での使用例
        let context = GeometryContext::<f32>::standard();
        assert!(context.tolerances.distance_tolerance > 0.0);
        assert!(context.tolerances.angle_tolerance > 0.0);

        // 精度順序
        let precision = GeometryContext::<f32>::precision();
        let relaxed = GeometryContext::<f32>::relaxed();

        assert!(precision.tolerances.distance_tolerance < context.tolerances.distance_tolerance);
        assert!(context.tolerances.distance_tolerance < relaxed.tolerances.distance_tolerance);
    }
}
