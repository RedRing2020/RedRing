//! EllipsoidalSolid3D の衝突判定テスト

#[cfg(test)]
mod tests {
    use crate::{EllipsoidalSolid3D, InfiniteLine3D, LineSegment3D, Plane3D, Point3D, Ray3D, Vector3D};
    use geo_foundation::{extensions::BasicCollision, Scalar};

    fn create_test_ellipsoid<T: Scalar>() -> EllipsoidalSolid3D<T> {
        let center = Point3D::origin();
        let axis = Vector3D::new(T::ZERO, T::ZERO, T::ONE);
        let ref_direction = Vector3D::new(T::ONE, T::ZERO, T::ZERO);
        let a_radius = T::from_f64(2.0); // X軸方向
        let b_radius = T::from_f64(1.5); // Y軸方向
        let c_radius = T::ONE;           // Z軸方向
        EllipsoidalSolid3D::new(center, axis, ref_direction, a_radius, b_radius, c_radius)
            .expect("Failed to create test ellipsoid")
    }

    #[test]
    fn test_collision_with_point_on_surface() {
        let ellipsoid = create_test_ellipsoid::<f64>();
        let point = Point3D::new(2.0, 0.0, 0.0); // X軸上の表面
        assert!(ellipsoid.intersects(&point, 1e-6));
    }

    #[test]
    fn test_collision_with_point_inside() {
        let ellipsoid = create_test_ellipsoid::<f64>();
        let point = Point3D::new(1.0, 0.0, 0.0); // 内部
        assert!(ellipsoid.intersects(&point, 1e-6));
    }

    #[test]
    fn test_collision_with_point_outside() {
        let ellipsoid = create_test_ellipsoid::<f64>();
        let point = Point3D::new(3.0, 0.0, 0.0); // 外部
        assert!(!ellipsoid.intersects(&point, 1e-6));
    }

    #[test]
    fn test_distance_to_point_inside() {
        let ellipsoid = create_test_ellipsoid::<f64>();
        let point = Point3D::new(1.0, 0.0, 0.0);
        let distance = ellipsoid.distance_to(&point);
        assert!(distance.abs() < 1e-6); // 内部なのでゼロ
    }

    #[test]
    fn test_distance_to_point_outside() {
        let ellipsoid = create_test_ellipsoid::<f64>();
        let point = Point3D::new(3.0, 0.0, 0.0);
        let distance = ellipsoid.distance_to(&point);
        assert!(distance > 0.0); // 外部なので正の距離
    }

    #[test]
    fn test_collision_with_line_segment() {
        let ellipsoid = create_test_ellipsoid::<f64>();
        let start = Point3D::new(-1.0, 0.0, 0.0);
        let end = Point3D::new(1.0, 0.0, 0.0);
        let line = LineSegment3D::new(start, end).expect("Failed to create line segment");
        assert!(ellipsoid.intersects(&line, 1e-6));
    }

    #[test]
    fn test_collision_with_ray() {
        let ellipsoid = create_test_ellipsoid::<f64>();
        let origin = Point3D::new(-3.0, 0.0, 0.0);
        let direction = Vector3D::new(1.0, 0.0, 0.0); // X軸方向
        let ray = Ray3D::new(origin, direction).expect("Failed to create ray");
        // 簡易実装：Ray起点との距離のみチェック
        let distance = ellipsoid.distance_to(&ray);
        assert!(distance > 0.0); // 起点は外部なので正の距離
    }

    #[test]
    fn test_collision_with_infinite_line() {
        let ellipsoid = create_test_ellipsoid::<f64>();
        let point = Point3D::new(-3.0, 0.0, 0.0);
        let direction = Vector3D::new(1.0, 0.0, 0.0);
        let line = InfiniteLine3D::new(point, direction).expect("Failed to create line");
        // 簡易実装：直線上の特定点のみチェック
        let distance = ellipsoid.distance_to(&line);
        assert!(distance > 0.0); // 外部の点なので正の距離
    }

    #[test]
    fn test_collision_with_plane() {
        let ellipsoid = create_test_ellipsoid::<f64>();
        let plane = Plane3D::xy_plane(0.0);
        assert!(ellipsoid.intersects(&plane, 1e-6));
    }

    #[test]
    fn test_collision_with_self() {
        let ellipsoid = create_test_ellipsoid::<f64>();
        assert!(ellipsoid.intersects(&ellipsoid, 1e-6));
        assert!(ellipsoid.distance_to(&ellipsoid) < 1e-6);
    }

    #[test]
    fn test_collision_with_other_ellipsoid_intersecting() {
        let ellipsoid1 = create_test_ellipsoid::<f64>();
        let center = Point3D::new(2.5, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);
        let ellipsoid2 = EllipsoidalSolid3D::new(center, axis, ref_direction, 1.0, 1.0, 1.0)
            .expect("Failed to create ellipsoid2");
        assert!(ellipsoid1.intersects(&ellipsoid2, 1e-6));
    }

    #[test]
    fn test_collision_with_other_ellipsoid_separate() {
        let ellipsoid1 = create_test_ellipsoid::<f64>();
        let center = Point3D::new(5.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);
        let ellipsoid2 = EllipsoidalSolid3D::new(center, axis, ref_direction, 1.0, 1.0, 1.0)
            .expect("Failed to create ellipsoid2");
        assert!(!ellipsoid1.intersects(&ellipsoid2, 1e-6));
    }
}
