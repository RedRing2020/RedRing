use crate::linalg::vector::Vector;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_creation() {
        let v = Vector::<f64>::new(vec![1.0, 2.0, 3.0]);
        assert_eq!(v.len(), 3);
        assert_eq!(v.get(0), 1.0);
        assert_eq!(v.get(1), 2.0);
        assert_eq!(v.get(2), 3.0);
    }

    #[test]
    fn test_vector_operations() {
        let v1 = Vector::<f64>::new(vec![1.0, 2.0, 3.0]);
        let v2 = Vector::<f64>::new(vec![4.0, 5.0, 6.0]);

        let dot = v1.dot(&v2).unwrap();
        assert_eq!(dot, 32.0); // 1*4 + 2*5 + 3*6 = 32

        let norm = v1.norm();
        assert!((norm - (14.0_f64).sqrt()).abs() < 1e-10);
    }

    #[test]
    fn test_vector_normalize() {
        let v = Vector::<f64>::new(vec![3.0, 4.0]);
        let normalized = v.normalize().unwrap();

        assert!((normalized.norm() - 1.0_f64).abs() < 1e-10);
        assert!((normalized.get(0) - 0.6_f64).abs() < 1e-10);
        assert!((normalized.get(1) - 0.8_f64).abs() < 1e-10);
    }
}