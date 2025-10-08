//! Point の型パラメータ化機能のテスト
//! f32/f64 両方の動作を検証

#[cfg(test)]
pub mod tests {
    use crate::geometry2d::point::{Point2D, Point2DF32, Point2DF64};
    use geo_foundation::abstract_types::Scalar;

    #[test]
    fn test_point_f64_creation() {
        // f64版（デフォルト）
        let point_f64: Point2D<f64> = Point2D::new(3.0, 4.0);
        assert_eq!(point_f64.x(), 3.0f64);
        assert_eq!(point_f64.y(), 4.0f64);

        // Point2DF64型エイリアス使用
        let point_alias: Point2DF64 = Point2D::new(1.0, 2.0);
        assert_eq!(point_alias.x(), 1.0f64);
        assert_eq!(point_alias.y(), 2.0f64);
    }

    #[test]
    fn test_point_f32_creation() {
        // f32版
        let point_f32: Point2D<f32> = Point2D::new(3.0f32, 4.0f32);
        assert_eq!(point_f32.x(), 3.0f32);
        assert_eq!(point_f32.y(), 4.0f32);

        // Point2DF32型エイリアス使用
        let point_alias: Point2DF32 = Point2D::new(1.0f32, 2.0f32);
        assert_eq!(point_alias.x(), 1.0f32);
        assert_eq!(point_alias.y(), 2.0f32);
    }

    #[test]
    fn test_point_distance_calculations() {
        // f64での距離計算
        let p1_f64: Point2D<f64> = Point2D::new(0.0, 0.0);
        let p2_f64: Point2D<f64> = Point2D::new(3.0, 4.0);
        let distance_f64 = p1_f64.distance_to(&p2_f64);
        assert!((distance_f64 - 5.0).abs() < f64::EPSILON);

        // f32での距離計算
        let p1_f32: Point2D<f32> = Point2D::new(0.0f32, 0.0f32);
        let p2_f32: Point2D<f32> = Point2D::new(3.0f32, 4.0f32);
        let distance_f32 = p1_f32.distance_to(&p2_f32);
        assert!((distance_f32 - 5.0f32).abs() < f32::EPSILON);
    }

    #[test]
    fn test_point_origins() {
        // f64原点
        let origin_f64: Point2D<f64> = Point2D::origin();
        assert_eq!(origin_f64.x(), 0.0f64);
        assert_eq!(origin_f64.y(), 0.0f64);

        // f32原点
        let origin_f32: Point2D<f32> = Point2D::origin();
        assert_eq!(origin_f32.x(), 0.0f32);
        assert_eq!(origin_f32.y(), 0.0f32);
    }

    #[test]
    fn test_scalar_constants() {
        // Scalarトレイトの定数を使用
        let point_pi_f64: Point2D<f64> = Point2D::new(f64::PI, f64::E);
        assert!((point_pi_f64.x() - std::f64::consts::PI).abs() < f64::EPSILON);
        assert!((point_pi_f64.y() - std::f64::consts::E).abs() < f64::EPSILON);

        let point_pi_f32: Point2D<f32> = Point2D::new(f32::PI, f32::E);
        assert!((point_pi_f32.x() - std::f32::consts::PI).abs() < f32::EPSILON);
        assert!((point_pi_f32.y() - std::f32::consts::E).abs() < f32::EPSILON);
    }

    #[test]
    fn test_type_conversion() {
        // f64からf32への変換（精度の損失に注意）
        let point_f64: Point2D<f64> = Point2D::new(std::f64::consts::PI, std::f64::consts::E);
        let point_f32: Point2D<f32> = Point2D::new(point_f64.x() as f32, point_f64.y() as f32);

        // 精度の違いを考慮した比較
        assert!((point_f32.x() - std::f32::consts::PI).abs() < 1e-6);
        assert!((point_f32.y() - std::f32::consts::E).abs() < 1e-6);
    }

    #[test]
    fn test_point_parametric_interoperability() {
        // Point2D<f64> と Point2D<f32> の相互運用性テスト
        let p_f64: Point2D<f64> = Point2D::new(1.5, 2.5);
        let p_f32: Point2D<f32> = Point2D::new(1.5f32, 2.5f32);

        // 型変換による比較
        let converted_f32 = Point2D::<f32>::new(p_f64.x() as f32, p_f64.y() as f32);
        assert!((converted_f32.x() - p_f32.x()).abs() < f32::EPSILON);
        assert!((converted_f32.y() - p_f32.y()).abs() < f32::EPSILON);
    }
}
