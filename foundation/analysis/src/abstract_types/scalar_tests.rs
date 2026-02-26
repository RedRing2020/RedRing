use super::Scalar;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f32_scalar_constants() {
        assert_eq!(f32::ZERO, 0.0f32);
        assert_eq!(f32::ONE, 1.0f32);
        assert_eq!(f32::PI, std::f32::consts::PI);
        assert_eq!(f32::TAU, std::f32::consts::TAU);
        assert_eq!(f32::E, std::f32::consts::E);
    }

    #[test]
    fn test_f64_scalar_constants() {
        assert_eq!(f64::ZERO, 0.0f64);
        assert_eq!(f64::ONE, 1.0f64);
        assert_eq!(f64::PI, std::f64::consts::PI);
        assert_eq!(f64::TAU, std::f64::consts::TAU);
        assert_eq!(f64::E, std::f64::consts::E);
    }

    #[test]
    fn test_scalar_operations() {
        let a = 3.0f32;
        let b = 4.0f32;

        assert_eq!((a * a + b * b).sqrt(), 5.0f32);
        assert!((f32::PI / 2.0).sin().approx_eq(1.0));
        assert!((f32::PI).cos().approx_eq(-1.0));
    }

    #[test]
    fn test_angle_conversion() {
        let degrees = 90.0f64;
        let radians = degrees * f64::DEG_TO_RAD;
        assert!(radians.approx_eq(f64::PI / 2.0));

        let back_to_degrees = radians * f64::RAD_TO_DEG;
        assert!(back_to_degrees.approx_eq(90.0));
    }

    #[test]
    fn test_approx_eq() {
        use crate::GeometricTolerance;
        let a = 1.0f32;
        let b = 1.0f32 + <f32 as GeometricTolerance>::TOLERANCE / 10.0;
        assert!(a.approx_eq(b));

        let c = 1.0f32 + <f32 as GeometricTolerance>::TOLERANCE * 10.0;
        assert!(!a.approx_eq(c));
    }

    #[test]
    fn test_type_conversion() {
        let f32_val = std::f32::consts::PI;
        let f64_val = f32_val.to_f64();
        let back_to_f32 = f32::from_f64(f64_val);

        assert!((f32_val - back_to_f32).abs() < f32::EPSILON);
    }

    #[test]
    fn test_scalar_arithmetic() {
        let a: f64 = 2.0;
        let b: f64 = 3.0;

        assert_eq!(a.powf(b), 8.0);
        assert_eq!(a.max(b), 3.0);
        assert_eq!(a.min(b), 2.0);
        assert!((a.sin().powi(2) + a.cos().powi(2) - 1.0).abs() < f64::EPSILON);
    }
}
