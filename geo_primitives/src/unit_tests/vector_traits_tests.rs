#[cfg(test)]
mod tests {
    use crate::geometry2d::Vector2D;
    use crate::geometry3d::Vector3D;
    use crate::traits::Vector;
    use geo_foundation::abstract_types::geometry::common::normalization_operations::Normalizable;

    #[test]
    fn test_vector_trait_with_vector2d() {
        let v1 = Vector2D::new(3.0, 4.0);
        let v2 = Vector2D::new(1.0, 0.0);

        // Vector トレイトのテスト
        assert_eq!(v1.norm(), 5.0);
        assert_eq!(v1.dot(&v2), 3.0);
        assert!(!v1.is_unit(1e-10));
        assert!(v2.is_unit(1e-10));

        // 成分アクセス
        assert_eq!(v1[0], 3.0);
        assert_eq!(v1[1], 4.0);

        // Vector2DExt トレイトのテスト
        let perp = v1.perpendicular();
        assert_eq!(perp, Vector2D::new(-4.0, 3.0));
        assert_eq!(v1.cross_2d(&v2), -4.0);

        // Normalizable トレイトのテスト
        let normalized = v1.normalize().unwrap();
        assert!((normalized.norm() - 1.0).abs() < 1e-10);
        assert!(v1.can_normalize(1e-10));

        let zero = Vector2D::zero();
        assert!(!zero.can_normalize(1e-10));
        assert_eq!(zero.normalize_or_zero(), Vector2D::zero());
    }

    #[test]
    fn test_vector_trait_with_vector3d() {
        let v1 = Vector3D::new(1.0, 2.0, 3.0);
        let v2 = Vector3D::unit_x();

        // Vector トレイトのテスト
        assert!((v1.norm() - (14.0_f64).sqrt()).abs() < 1e-10);
        assert_eq!(v1.dot(&v2), 1.0);
        assert!(v2.is_unit(1e-10));

        // 成分アクセス
        assert_eq!(v1[0], 1.0);
        assert_eq!(v1[1], 2.0);
        assert_eq!(v1[2], 3.0);

        // Vector3DExt トレイトのテスト
        let cross = v1.cross(&v2);
        assert_eq!(cross, Vector3D::new(0.0, 3.0, -2.0));

        // 平行性テスト
        let parallel = Vector3D::new(2.0, 4.0, 6.0);
        assert!(v1.is_parallel_to(&parallel, 1e-10));

        // 垂直性テスト（v1 = (1,2,3)に垂直なベクトルを作成）
        let perpendicular = Vector3D::new(2.0, -1.0, 0.0); // 1*2 + 2*(-1) + 3*0 = 0
        assert!(v1.is_perpendicular_to(&perpendicular, 1e-10));
    }
}
