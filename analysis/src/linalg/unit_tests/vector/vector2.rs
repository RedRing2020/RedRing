use crate::linalg::vector::Vector2;

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_vector2_creation() {
        let v = Vector2::new(3.0, 4.0);
        assert_eq!(v.x(), 3.0);
        assert_eq!(v.y(), 4.0);
    }

    #[test]
    fn test_vector2_operations() {
        let v1 = Vector2::new(3.0, 4.0);
        let v2 = Vector2::new(1.0, 2.0);

        let dot = v1.dot(&v2);
        assert_eq!(dot, 11.0); // 3*1 + 4*2 = 11

        let cross = v1.cross(&v2);
        assert_eq!(cross, 2.0); // 3*2 - 4*1 = 2
    }

    #[test]
    fn test_vector2_norm() {
        let v = Vector2::new(3.0, 4.0);
        assert_eq!(v.norm(), 5.0); // 3-4-5 直角三角形
        assert_eq!(v.norm_squared(), 25.0);
    }

    #[test]
    fn test_vector2_rotation() {
        let v = Vector2::new(1.0, 0.0);
        let rotated = v.rotate(PI / 2.0);

        assert!((rotated.x() - 0.0).abs() < 1e-10);
        assert!((rotated.y() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_vector2_arithmetic() {
        let v1 = Vector2::new(1.0, 2.0);
        let v2 = Vector2::new(3.0, 4.0);

        let sum = v1 + v2;
        assert_eq!(sum, Vector2::new(4.0, 6.0));

        let diff = v2 - v1;
        assert_eq!(diff, Vector2::new(2.0, 2.0));

        let scaled = v1 * 2.0;
        assert_eq!(scaled, Vector2::new(2.0, 4.0));
    }
}
