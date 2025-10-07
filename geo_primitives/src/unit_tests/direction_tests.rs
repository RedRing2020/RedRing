//! Direction2D and Direction3D のユニットテスト
#[cfg(test)]
mod direction2d_tests {
    use crate::geometry2d::Direction2D;
    use crate::traits::{Direction, Direction2D as Direction2DTrait, StepCompatible};
    use std::f64::consts::PI;

    #[test]
    fn test_direction2d_from_vector() {
        use crate::geometry2d::Vector2D;

        let vector = Vector2D::new(3.0, 4.0);
        let direction = Direction2D::from_vector(vector).unwrap();

        // 正規化されているかテスト
        assert!((direction.length() - 1.0).abs() < 1e-10);
        assert!((direction.x() - 0.6).abs() < 1e-10);
        assert!((direction.y() - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_direction2d_from_zero_vector() {
        use crate::geometry2d::Vector2D;

        let vector = Vector2D::new(0.0, 0.0);
        let direction = Direction2D::from_vector(vector);

        assert!(direction.is_none());
    }

    #[test]
    fn test_direction2d_from_angle() {
        let direction = Direction2D::from_angle(PI / 4.0);

        assert!((direction.x() - (PI / 4.0).cos()).abs() < 1e-10);
        assert!((direction.y() - (PI / 4.0).sin()).abs() < 1e-10);
    }

    #[test]
    fn test_direction2d_to_angle() {
        let direction = Direction2D::from_angle(PI / 3.0);
        let angle = direction.to_angle();

        assert!((angle - PI / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_direction2d_perpendicular() {
        let direction = Direction2D::x_axis();
        let perp = direction.perpendicular();

        assert!((perp.x() - 0.0).abs() < 1e-10);
        assert!((perp.y() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_direction2d_dot_product() {
        let dir1 = Direction2D::x_axis();
        let dir2 = Direction2D::y_axis();

        assert!((dir1.dot(&dir2) - 0.0).abs() < 1e-10);
        assert!((dir1.dot(&dir1) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_direction2d_parallel_perpendicular() {
        let dir1 = Direction2D::x_axis();
        let dir2 = Direction2D::x_axis();
        let dir3 = Direction2D::y_axis();

        assert!(dir1.is_parallel(&dir2, 1e-10));
        assert!(dir1.is_perpendicular(&dir3, 1e-10));
        assert!(!dir1.is_parallel(&dir3, 1e-10));
    }

    #[test]
    fn test_direction2d_reverse() {
        let direction = Direction2D::x_axis();
        let reversed = direction.reverse();

        assert!((reversed.x() + 1.0).abs() < 1e-10);
        assert!((reversed.y() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_direction2d_step_string() {
        let direction = Direction2D::x_axis();
        let step_str = direction.to_step_string();

        assert!(step_str.contains("DIRECTION"));
        assert!(step_str.contains("1.000000"));
        assert!(step_str.contains("0.000000"));
    }

    #[test]
    fn test_direction2d_constants() {
        let pos_x = Direction2D::positive_x();
        let pos_y = Direction2D::positive_y();
        let neg_x = Direction2D::negative_x();
        let neg_y = Direction2D::negative_y();

        assert_eq!(pos_x.x(), 1.0);
        assert_eq!(pos_x.y(), 0.0);
        assert_eq!(pos_y.x(), 0.0);
        assert_eq!(pos_y.y(), 1.0);
        assert_eq!(neg_x.x(), -1.0);
        assert_eq!(neg_x.y(), 0.0);
        assert_eq!(neg_y.x(), 0.0);
        assert_eq!(neg_y.y(), -1.0);
    }
}

#[cfg(test)]
mod direction3d_tests {
    use crate::geometry3d::Direction3D;
    use crate::traits::{Direction, Direction3D as Direction3DTrait, StepCompatible};
    use std::f64::consts::PI;

    #[test]
    fn test_direction3d_from_vector() {
        use crate::geometry3d::Vector3D;

        let vector = Vector3D::new(1.0, 2.0, 2.0);
        let direction = Direction3D::from_vector(vector).unwrap();

        // 正規化されているかテスト
        assert!((direction.length() - 1.0).abs() < 1e-10);

        let expected_length = (1.0 + 4.0 + 4.0_f64).sqrt();
        assert!((direction.x() - 1.0 / expected_length).abs() < 1e-10);
        assert!((direction.y() - 2.0 / expected_length).abs() < 1e-10);
        assert!((direction.z() - 2.0 / expected_length).abs() < 1e-10);
    }

    #[test]
    fn test_direction3d_from_zero_vector() {
        use crate::geometry3d::Vector3D;

        let vector = Vector3D::new(0.0, 0.0, 0.0);
        let direction = Direction3D::from_vector(vector);

        assert!(direction.is_none());
    }

    #[test]
    fn test_direction3d_cross_product() {
        let dir1 = Direction3D::x_axis();
        let dir2 = Direction3D::y_axis();
        let cross = dir1.cross(&dir2);

        assert!((cross.x() - 0.0).abs() < 1e-10);
        assert!((cross.y() - 0.0).abs() < 1e-10);
        assert!((cross.z() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_direction3d_any_perpendicular() {
        let direction = Direction3D::z_axis();
        let perp = direction.any_perpendicular();

        // 垂直であることを確認
        assert!(direction.is_perpendicular(&perp, 1e-10));
        assert!((perp.length() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_direction3d_build_orthonormal_basis() {
        let z_axis = Direction3D::z_axis();
        let (u, v, w) = z_axis.build_orthonormal_basis();

        // 正規直交基底の性質をテスト
        assert!((u.length() - 1.0).abs() < 1e-10);
        assert!((v.length() - 1.0).abs() < 1e-10);
        assert!((w.length() - 1.0).abs() < 1e-10);

        // 互いに直交していることを確認
        assert!(u.is_perpendicular(&v, 1e-10));
        assert!(v.is_perpendicular(&w, 1e-10));
        assert!(w.is_perpendicular(&u, 1e-10));
    }

    #[test]
    fn test_direction3d_rotate_around_axis() {
        let x_axis = Direction3D::x_axis();
        let z_axis = Direction3D::z_axis();

        // Z軸周りに90度回転
        let rotated = x_axis.rotate_around_axis(&z_axis, PI / 2.0);

        // Y軸方向になるはず
        assert!((rotated.x() - 0.0).abs() < 1e-10);
        assert!((rotated.y() - 1.0).abs() < 1e-10);
        assert!((rotated.z() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_direction3d_parallel_perpendicular() {
        let dir1 = Direction3D::x_axis();
        let dir2 = Direction3D::x_axis();
        let dir3 = Direction3D::y_axis();

        assert!(dir1.is_parallel(&dir2, 1e-10));
        assert!(dir1.is_perpendicular(&dir3, 1e-10));
        assert!(!dir1.is_parallel(&dir3, 1e-10));
    }

    #[test]
    fn test_direction3d_same_opposite() {
        let dir1 = Direction3D::x_axis();
        let dir2 = Direction3D::x_axis();
        let dir3 = dir1.reverse();

        assert!(dir1.is_same_direction(&dir2, 1e-10));
        assert!(dir1.is_opposite_direction(&dir3, 1e-10));
        assert!(!dir1.is_same_direction(&dir3, 1e-10));
    }

    #[test]
    fn test_direction3d_step_string() {
        let direction = Direction3D::z_axis();
        let step_str = direction.to_step_string();

        assert!(step_str.contains("DIRECTION"));
        assert!(step_str.contains("0.000000"));
        assert!(step_str.contains("1.000000"));
    }

    #[test]
    fn test_direction3d_constants() {
        let pos_x = Direction3D::positive_x();
        let pos_y = Direction3D::positive_y();
        let pos_z = Direction3D::positive_z();
        let neg_x = Direction3D::negative_x();
        let neg_y = Direction3D::negative_y();
        let neg_z = Direction3D::negative_z();

        assert_eq!(pos_x.x(), 1.0);
        assert_eq!(pos_x.y(), 0.0);
        assert_eq!(pos_x.z(), 0.0);

        assert_eq!(pos_y.x(), 0.0);
        assert_eq!(pos_y.y(), 1.0);
        assert_eq!(pos_y.z(), 0.0);

        assert_eq!(pos_z.x(), 0.0);
        assert_eq!(pos_z.y(), 0.0);
        assert_eq!(pos_z.z(), 1.0);

        assert_eq!(neg_x.x(), -1.0);
        assert_eq!(neg_y.y(), -1.0);
        assert_eq!(neg_z.z(), -1.0);
    }
}
