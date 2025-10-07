#[cfg(test)]
use crate::geometry2d::Vector2D;    #[test]
    fn test_vector2d_creation() {
        let v = Vector2D::new(1.0, 2.0);
        assert_eq!(v.x(), 1.0);
        assert_eq!(v.y(), 2.0);
    }

    #[test]
    fn test_vector2d_constants() {
        let zero = Vector2D::zero();
        assert_eq!(zero.x(), 0.0);
        assert_eq!(zero.y(), 0.0);

        let unit_x = Vector2D::unit_x();
        assert_eq!(unit_x.x(), 1.0);
        assert_eq!(unit_x.y(), 0.0);
    }

    #[test]
    fn test_vector2d_length() {
        let v = Vector2D::new(3.0, 4.0);
        assert!((v.length() - 5.0).abs() < 1e-10);
        assert!((v.length_squared() - 25.0).abs() < 1e-10);
    }

    #[test]
    fn test_vector2d_normalize() {
        let v = Vector2D::new(3.0, 4.0);
        let normalized = v.normalize().unwrap();
        assert!((normalized.length() - 1.0).abs() < 1e-10);

        let zero = Vector2D::zero();
        assert!(zero.normalize().is_none());
    }

    #[test]
    fn test_vector2d_dot_product() {
        let v1 = Vector2D::new(1.0, 2.0);
        let v2 = Vector2D::new(3.0, 4.0);
        let dot = v1.dot(&v2);
        assert_eq!(dot, 11.0); // 1*3 + 2*4 = 11
    }

    #[test]
    fn test_vector2d_perpendicular() {
        let v = Vector2D::new(1.0, 0.0);
        let perp = v.perpendicular();
        assert_eq!(perp, Vector2D::new(0.0, 1.0));
    }

    #[test]
    fn test_vector2d_arithmetic() {
        let v1 = Vector2D::new(1.0, 2.0);
        let v2 = Vector2D::new(3.0, 4.0);

        let add = v1 + v2;
        assert_eq!(add, Vector2D::new(4.0, 6.0));

        let sub = v2 - v1;
        assert_eq!(sub, Vector2D::new(2.0, 2.0));

        let mul = v1 * 2.0;
        assert_eq!(mul, Vector2D::new(2.0, 4.0));

        let neg = -v1;
        assert_eq!(neg, Vector2D::new(-1.0, -2.0));
    }

    #[test]
    fn test_vector2d_cross_2d() {
        let v1 = Vector2D::new(1.0, 2.0);
        let v2 = Vector2D::new(3.0, 4.0);
        let cross = v1.cross_2d(&v2);
        assert_eq!(cross, -2.0); // 1*4 - 2*3 = -2
    }

    #[test]
    fn test_vector2d_rotate() {
        let v = Vector2D::new(1.0, 0.0);
        let rotated = v.rotate(std::f64::consts::PI / 2.0); // 90度回転
        assert!((rotated.x() - 0.0).abs() < 1e-10);
        assert!((rotated.y() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_vector2d_add_scaled() {
        let v1 = Vector2D::new(1.0, 2.0);
        let v2 = Vector2D::new(3.0, 4.0);
        let result = v1.add_scaled(&v2, 2.0);
        assert_eq!(result, Vector2D::new(7.0, 10.0)); // (1,2) + 2*(3,4) = (7,10)
    }

    // vector_traits.rsから移動したトレイトテスト
    #[test]
    fn test_vector_trait_with_vector2d() {
        use crate::traits::{Vector, Normalizable};
        
        let v1 = Vector2D::new(3.0, 4.0);
        let v2 = Vector2D::new(1.0, 0.0);

        // Vector トレイトのテスト
        assert_eq!(v1.length(), 5.0);
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
        assert!((normalized.length() - 1.0).abs() < 1e-10);
        assert!(v1.can_normalize(1e-10));

        let zero = Vector2D::zero();
        assert!(!zero.can_normalize(1e-10));
        assert_eq!(zero.normalize_or_zero(), Vector2D::zero());
    }
