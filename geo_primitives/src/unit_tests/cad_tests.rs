#[cfg(test)]
mod cad_tests {
    use crate::{CadPoint, CadVector, CadDirection, CadCircle};

    #[test]
    fn test_cad_circle_basic() {
        let center = CadPoint::new(0.0, 0.0, 0.0);
        let normal = CadDirection::from_vector(CadVector::new(0.0, 0.0, 1.0)).unwrap();
        let circle = CadCircle::new(center, 1.0, normal);

        assert_eq!(circle.radius(), 1.0);
        assert!((circle.length() - 2.0 * std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_cad_circle_evaluation() {
        let center = CadPoint::new(0.0, 0.0, 0.0);
        let normal = CadDirection::from_vector(CadVector::new(0.0, 0.0, 1.0)).unwrap();
        let circle = CadCircle::new(center, 1.0, normal);

        let point = circle.evaluate(0.0); // t=0の点
        assert!((point.x() - 1.0).abs() < 1e-10);
        assert!(point.y().abs() < 1e-10);
        assert!(point.z().abs() < 1e-10);
    }

    #[test]
    fn test_cad_point_basic() {
        let point = CadPoint::new(1.0, 2.0, 3.0);
        assert_eq!(point.x(), 1.0);
        assert_eq!(point.y(), 2.0);
        assert_eq!(point.z(), 3.0);
    }

    #[test]
    fn test_cad_vector_basic() {
        let vector = CadVector::new(1.0, 2.0, 3.0);
        assert_eq!(vector.x(), 1.0);
        assert_eq!(vector.y(), 2.0);
        assert_eq!(vector.z(), 3.0);
        
        let length = vector.length();
        assert!((length - (1.0 + 4.0 + 9.0_f64).sqrt()).abs() < 1e-10);
    }

    #[test]
    fn test_cad_direction_creation() {
        let vector = CadVector::new(3.0, 4.0, 0.0);
        let direction = CadDirection::from_vector(vector);
        
        assert!(direction.is_some());
        let dir = direction.unwrap();
        assert!((dir.length() - 1.0).abs() < 1e-10); // 正規化されている
    }
}