//! Direction2D のジェネリック動作テスト
//!
//! Scalarトレイトの角度メソッド（sin, cos, atan2）の動作確認

#[cfg(test)]
mod tests {
    use crate::geometry2d::Direction2D;
    use geo_foundation::abstract_types::geometry::Direction2D as Direction2DTrait;
    use geo_foundation::abstract_types::Scalar;

    #[test]
    fn test_direction2d_generic_f64() {
        // f64でのDirection2D動作確認
        let dir = Direction2D::<f64>::new(1.0, 0.0).unwrap();
        assert_eq!(dir.x(), 1.0);
        assert_eq!(dir.y(), 0.0);

        // 角度関連のテスト（Scalarトレイトの角度メソッド使用）
        let angle = dir.to_angle();
        assert!((angle - 0.0).abs() < f64::TOLERANCE);

        // 角度からDirection2Dを作成
        let dir_from_angle = Direction2D::<f64>::from_angle(geo_foundation::constants::precision::PI / 2.0);
        assert!((dir_from_angle.x() - 0.0).abs() < f64::TOLERANCE);
        assert!((dir_from_angle.y() - 1.0).abs() < f64::TOLERANCE);
    }

    #[test]
    fn test_direction2d_generic_f32() {
        // f32でのDirection2D動作確認
        let dir = Direction2D::<f32>::new(0.0, 1.0).unwrap();
        assert_eq!(dir.x(), 0.0);
        assert_eq!(dir.y(), 1.0);

        // 角度関連のテスト（Scalarトレイトの角度メソッド使用）
        let angle = dir.to_angle();
        assert!((angle - geo_foundation::constants::game::PI / 2.0).abs() < f32::TOLERANCE);

        // 角度からDirection2Dを作成
        let dir_from_angle = Direction2D::<f32>::from_angle(0.0);
        assert!((dir_from_angle.x() - 1.0).abs() < f32::TOLERANCE);
        assert!((dir_from_angle.y() - 0.0).abs() < f32::TOLERANCE);
    }

    #[test]
    fn test_direction2d_perpendicular() {
        // 垂直ベクトルのテスト
        let dir = Direction2D::<f64>::positive_x();
        let perp = dir.perpendicular();
        assert!((perp.x() - 0.0).abs() < f64::TOLERANCE);
        assert!((perp.y() - 1.0).abs() < f64::TOLERANCE);
    }

    #[test]
    fn test_direction2d_constants() {
        // 定数方向ベクトルのテスト
        let x_axis = Direction2D::<f64>::positive_x();
        assert_eq!(x_axis.x(), 1.0);
        assert_eq!(x_axis.y(), 0.0);

        let y_axis = Direction2D::<f64>::positive_y();
        assert_eq!(y_axis.x(), 0.0);
        assert_eq!(y_axis.y(), 1.0);

        let neg_x = Direction2D::<f64>::negative_x();
        assert_eq!(neg_x.x(), -1.0);
        assert_eq!(neg_x.y(), 0.0);

        let neg_y = Direction2D::<f64>::negative_y();
        assert_eq!(neg_y.x(), 0.0);
        assert_eq!(neg_y.y(), -1.0);
    }

    #[test]
    fn test_direction2d_scalar_angle_methods() {
        // Scalarトレイトの角度メソッドの直接テスト
        let half_pi = f64::PI / 2.0;

        // cos/sinメソッドのテスト - angle の cos/sin が dir の x/y と等しいかテスト
        let dir = Direction2D::<f64>::from_angle(half_pi);
        assert!((half_pi.cos() - dir.x()).abs() < f64::TOLERANCE); // cos(π/2) ≈ 0 = dir.x()
        assert!((half_pi.sin() - dir.y()).abs() < f64::TOLERANCE); // sin(π/2) = 1 = dir.y()

        // atan2メソッドのテスト
        let calculated_angle = dir.y().atan2(dir.x());
        assert!((calculated_angle - half_pi).abs() < f64::TOLERANCE);
    }
}
