//! Direction3D Extensions Tests
//!
//! Direction3D拡張機能のテスト

#[cfg(test)]
mod direction_3d_extensions_tests {
    use crate::{Direction3D, Vector3D};
    use geo_foundation::Scalar;

    type TestType = f64;

    #[test]
    fn test_azimuth_elevation_angles() {
        // Z軸方向のテスト
        let dir_z = Direction3D::<TestType>::positive_z();
        let elevation = dir_z.elevation_angle_radians();
        
        // Z軸方向の仰角は0ラジアン
        assert!((elevation - 0.0).abs() < TestType::EPSILON);
    }

    #[test]
    fn test_angle_between_3d() {
        let dir_x = Direction3D::<TestType>::positive_x();
        let dir_y = Direction3D::<TestType>::positive_y();
        
        let angle = dir_x.angle_between_radians(&dir_y);
        
        // X軸とY軸の間の角度は90度（π/2ラジアン）
        assert!((angle - std::f64::consts::FRAC_PI_2).abs() < TestType::EPSILON);
    }

    #[test]
    fn test_tolerance_based_comparisons_3d() {
        let dir1 = Direction3D::<TestType>::positive_x();
        
        // より現実的な角度誤差でテスト
        // 小さな角度誤差（約0.1度 ≈ 0.00175ラジアン）
        let small_error = 0.00175;
        let dir2 = Direction3D::new(
            small_error.cos(), 
            small_error.sin(), 
            0.0
        ).unwrap();
        
        // 新しいAngle型を使用したAPI（推奨）
        use geo_foundation::Angle;
        
        // ラジアン指定での角度許容誤差
        assert!(dir1.is_same_direction_within_angle(&dir2, Angle::from_radians(0.002))); // 0.002ラジアン許容誤差
        assert!(!dir1.is_same_direction_within_angle(&dir2, Angle::from_radians(0.001))); // 0.001ラジアン許容誤差
        
        // 度指定での角度許容誤差
        assert!(dir1.is_same_direction_within_angle(&dir2, Angle::from_degrees(0.2))); // 0.2度許容誤差
        assert!(!dir1.is_same_direction_within_angle(&dir2, Angle::from_degrees(0.05))); // 0.05度許容誤差
        
        // より大きな角度誤差（約1度 ≈ 0.0175ラジアン）
        let large_error = 0.0175;
        let dir3 = Direction3D::new(
            large_error.cos(),
            large_error.sin(),
            0.0
        ).unwrap();
        
        assert!(dir1.is_same_direction_within_angle(&dir3, Angle::from_degrees(2.0))); // 2度許容誤差
        assert!(!dir1.is_same_direction_within_angle(&dir3, Angle::from_degrees(0.5))); // 0.5度許容誤差
    }

    #[test]
    fn test_opposite_direction_detection_3d() {
        let positive_x = Direction3D::<TestType>::positive_x();
        let negative_x = Direction3D::<TestType>::negative_x();

        use geo_foundation::Angle;
        assert!(positive_x.is_opposite_direction_within_angle(&negative_x, Angle::from_degrees(1.0)));
    }

    #[test]
    fn test_elevation_angle_xy_plane() {
        // XY平面の方向（仰角90度 = π/2）
        let dir_xy = Direction3D::<TestType>::positive_x();
        let elevation = dir_xy.elevation_angle_radians();
        
        assert!((elevation - std::f64::consts::FRAC_PI_2).abs() < TestType::EPSILON);
    }

    #[test]
    fn test_azimuth_angle_variations() {
        // Y軸方向の方位角は90度（π/2）
        let dir_y = Direction3D::<TestType>::positive_y();
        let azimuth = dir_y.azimuth_angle_radians();
        
        assert!((azimuth - std::f64::consts::FRAC_PI_2).abs() < TestType::EPSILON);
    }

    #[test]
    fn test_direction_3d_construction() {
        // ベクトルから方向を作成
        let vec = Vector3D::new(1.0, 1.0, 1.0);
        let dir = Direction3D::from_vector(vec).unwrap();
        
        // 正規化されているかチェック（より緩い許容誤差）
        let length_squared = dir.x() * dir.x() + dir.y() * dir.y() + dir.z() * dir.z();
        assert!((length_squared - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_angle_based_construction() {
        // 方位角と仰角から方向を作成テスト用のヘルパー
        let azimuth = std::f64::consts::FRAC_PI_4; // 45度
        let elevation = std::f64::consts::FRAC_PI_3; // 60度
        
        let x = elevation.sin() * azimuth.cos();
        let y = elevation.sin() * azimuth.sin();
        let z = elevation.cos();
        
        let dir = Direction3D::new(x, y, z).unwrap();
        
        // 作成された方向の角度をチェック
        let computed_azimuth = dir.azimuth_angle_radians();
        let computed_elevation = dir.elevation_angle_radians();
        
        assert!((computed_azimuth - azimuth).abs() < TestType::EPSILON);
        assert!((computed_elevation - elevation).abs() < TestType::EPSILON);
    }

    #[test]
    fn test_direction_normalization() {
        // 異なる長さのベクトルから同じ方向を作成
        let vec1 = Vector3D::new(1.0, 0.0, 0.0);
        let vec2 = Vector3D::new(5.0, 0.0, 0.0);
        
        let dir1 = Direction3D::from_vector(vec1).unwrap();
        let dir2 = Direction3D::from_vector(vec2).unwrap();
        
        // 両方とも正規化されて同じ方向になる
        assert!((dir1.x() - dir2.x()).abs() < TestType::EPSILON);
        assert!((dir1.y() - dir2.y()).abs() < TestType::EPSILON);
        assert!((dir1.z() - dir2.z()).abs() < TestType::EPSILON);
    }
}