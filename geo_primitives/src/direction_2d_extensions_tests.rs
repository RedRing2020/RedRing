//! Direction2D Extensions Tests
//!
//! Direction2D拡張機能のテスト

#[cfg(test)]
mod direction_2d_extensions_tests {
    use crate::{Direction2D, Vector2D};

    type TestType = f64;

    #[test]
    fn test_angle_operations() {
        // 基本的な角度操作をテスト
        let _right = Direction2D::<TestType>::positive_x();
        let _up = Direction2D::<TestType>::positive_y();

        // 角度から方向を作成
        let from_radians = Direction2D::from_angle_radians(std::f64::consts::PI / 2.0);
        assert!((from_radians.x() - 0.0).abs() < TestType::EPSILON);
        assert!((from_radians.y() - 1.0).abs() < TestType::EPSILON);
    }

    #[test]
    fn test_advanced_rotations() {
        let dir = Direction2D::<TestType>::positive_x();

        // 45度回転
        let rotated = dir.rotated_by_angle(std::f64::consts::PI / 4.0);
        let expected_x = (std::f64::consts::PI / 4.0).cos();
        let expected_y = (std::f64::consts::PI / 4.0).sin();

        assert!((rotated.x() - expected_x).abs() < TestType::EPSILON);
        assert!((rotated.y() - expected_y).abs() < TestType::EPSILON);
    }

    #[test]
    fn test_angle_between_directions() {
        let right = Direction2D::<TestType>::positive_x();
        let up = Direction2D::<TestType>::positive_y();

        let angle = right.angle_between(&up);
        assert!((angle - std::f64::consts::PI / 2.0).abs() < TestType::EPSILON);
    }

    #[test]
    fn test_slerp_interpolation() {
        let start = Direction2D::<TestType>::positive_x();
        let end = Direction2D::<TestType>::positive_y();

        // 中点での補間
        let mid = start.slerp(&end, 0.5);
        let expected_angle = std::f64::consts::PI / 4.0; // 45度
        let actual_angle = mid.to_angle_radians();

        assert!((actual_angle - expected_angle).abs() < TestType::EPSILON);
    }

    #[test]
    fn test_tolerance_based_comparisons() {
        let dir1 = Direction2D::<TestType>::positive_x();
        let dir2 = Direction2D::from_angle_radians(0.001); // 0.001ラジアン ≈ 0.057度の誤差

        // 新しいAngle型を使用したAPI（推奨）
        use geo_foundation::Angle;

        // ラジアン指定での角度許容誤差
        assert!(dir1.is_same_direction_within_angle(&dir2, Angle::from_radians(0.002))); // 0.002ラジアン許容誤差
        assert!(!dir1.is_same_direction_within_angle(&dir2, Angle::from_radians(0.0005))); // 0.0005ラジアン許容誤差

        // 度指定での角度許容誤差
        assert!(dir1.is_same_direction_within_angle(&dir2, Angle::from_degrees(0.2))); // 0.2度許容誤差
        assert!(!dir1.is_same_direction_within_angle(&dir2, Angle::from_degrees(0.02))); // 0.02度許容誤差

        // より大きな角度誤差をテスト
        let dir3 = Direction2D::from_angle_radians(0.01); // 0.01ラジアン ≈ 0.57度の誤差
        assert!(dir1.is_same_direction_within_angle(&dir3, Angle::from_degrees(1.0))); // 1度許容誤差
        assert!(!dir1.is_same_direction_within_angle(&dir3, Angle::from_degrees(0.1)));
        // 0.1度許容誤差
    }

    #[test]
    fn test_opposite_direction_detection() {
        let right = Direction2D::<TestType>::positive_x();
        let left = Direction2D::<TestType>::negative_x();

        use geo_foundation::Angle;
        assert!(right.is_opposite_direction_within_angle(&left, Angle::from_degrees(1.0)));
    }

    #[test]
    fn test_perpendicular_direction() {
        let dir = Direction2D::<TestType>::positive_x();
        let perp = dir.perpendicular();

        // 垂直方向は Y軸正方向
        assert!((perp.x() - 0.0).abs() < TestType::EPSILON);
        assert!((perp.y() - 1.0).abs() < TestType::EPSILON);
    }

    #[test]
    fn test_direction_from_vector() {
        let dir = Direction2D::from_angle_radians(std::f64::consts::PI / 6.0); // 30度
        let vec = Vector2D::new(dir.x() * 5.0, dir.y() * 5.0);
        let dir_from_vec = Direction2D::from_vector(vec).unwrap();

        // 正規化されているので元の方向と一致すべき
        assert!((dir.x() - dir_from_vec.x()).abs() < TestType::EPSILON);
        assert!((dir.y() - dir_from_vec.y()).abs() < TestType::EPSILON);
    }

    #[test]
    fn test_angle_api_usage_examples() {
        // 新しいAngle型APIの使用例をテスト
        use geo_foundation::Angle;

        let dir1 = Direction2D::<TestType>::positive_x();
        let dir2 = Direction2D::from_angle_radians(0.017453); // 約1度の誤差

        // 使いやすい度数指定
        assert!(dir1.is_same_direction_within_angle(&dir2, Angle::from_degrees(2.0)));
        assert!(!dir1.is_same_direction_within_angle(&dir2, Angle::from_degrees(0.5)));

        // 精密なラジアン指定
        assert!(dir1.is_same_direction_within_angle(&dir2, Angle::from_radians(0.02)));
        assert!(!dir1.is_same_direction_within_angle(&dir2, Angle::from_radians(0.01)));

        // ユーザーは内積計算を気にする必要がない！
        // 直感的な角度指定で判定可能
    }
}
