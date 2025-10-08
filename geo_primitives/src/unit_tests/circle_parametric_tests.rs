//! Circle type parametrization tests
//! Verify Circle<T> functionality with both f32 and f64 precision

use crate::geometry2d::{Circle, Point2D};
use geo_foundation::abstract_types::Scalar;

#[cfg(test)]
mod tests {
    use super::*;

    // Type aliases
    type Circlef = Circle<f32>;
    type CircleTest = Circle<f64>;
    type Pointf = Point2D<f32>;
    type PointTest = Point2D<f64>;

    #[test]
    fn test_circle_basic_creation() {
        let center_f32 = Pointf::new(1.0, 2.0);
        let circle_f32 = Circlef::new(center_f32, 3.0);

        // center()は Pointを返すので直接比較
        let retrieved_center_f32 = circle_f32.center();
        assert_eq!(retrieved_center_f32.x(), center_f32.x());
        assert_eq!(retrieved_center_f32.y(), center_f32.y());
        assert_eq!(circle_f32.radius(), 3.0);

        let center_f64 = PointTest::new(1.0, 2.0);
        let circle_f64 = CircleTest::new(center_f64, 3.0);

        let retrieved_center_f64 = circle_f64.center();
        assert_eq!(retrieved_center_f64.x(), center_f64.x());
        assert_eq!(retrieved_center_f64.y(), center_f64.y());
        assert_eq!(circle_f64.radius(), 3.0);
    }

    #[test]
    fn test_circle_basic_operations() {
        let circle_f32 = Circlef::new(Pointf::new(0.0, 0.0), 3.0);
        let circle_f64 = CircleTest::new(PointTest::new(0.0, 0.0), 3.0);

        // Basic properties test
        assert_eq!(circle_f32.radius(), 3.0f32);
        assert_eq!(circle_f64.radius(), 3.0f64);

        // Degenerate test
        let degenerate_f32 = Circlef::new(Pointf::new(0.0, 0.0), 0.0);
        let degenerate_f64 = CircleTest::new(PointTest::new(0.0, 0.0), 0.0);

        assert!(degenerate_f32.is_degenerate());
        assert!(degenerate_f64.is_degenerate());
    }

    #[test]
    fn test_circle_scale() {
        let original_f32 = Circlef::new(Pointf::new(1.0, 1.0), 2.0);
        let scaled_f32 = original_f32.scale(2.0);

        assert_eq!(scaled_f32.center(), original_f32.center());
        assert_eq!(scaled_f32.radius(), 4.0);

        let original_f64 = CircleTest::new(PointTest::new(1.0, 1.0), 2.0);
        let scaled_f64 = original_f64.scale(2.0);

        assert_eq!(scaled_f64.center(), original_f64.center());
        assert_eq!(scaled_f64.radius(), 4.0);
    }

    #[test]
    fn test_scalar_constants() {
        // Scalar trait constants test
        assert_eq!(f32::ZERO, 0.0f32);
        assert_eq!(f32::ONE, 1.0f32);
        assert_eq!(f64::ZERO, 0.0f64);
        assert_eq!(f64::ONE, 1.0f64);

        // PI constants test
        assert!((f32::PI - std::f32::consts::PI).abs() < f32::EPSILON);
        assert!((f64::PI - std::f64::consts::PI).abs() < f64::EPSILON);
    }

    #[test]
    fn test_type_conversion() {
        // Type conversion test
        let value_f64 = std::f64::consts::PI;
        let value_f32 = f32::from_f64(value_f64);

        assert!((value_f32 - std::f32::consts::PI).abs() < f32::EPSILON);

        let value_f32_orig = std::f32::consts::E;
        let value_f64_conv = f64::from(value_f32_orig);

        assert!((value_f64_conv - std::f64::consts::E).abs() < 1e-6);
    }

    #[test]
    fn test_circle_from_three_points() {
        // Create circle from three points
        let p1 = PointTest::new(1.0, 0.0);
        let p2 = PointTest::new(0.0, 1.0);
        let p3 = PointTest::new(-1.0, 0.0);

        let circle_opt = CircleTest::from_three_points(p1, p2, p3);
        assert!(circle_opt.is_some());

        let circle = circle_opt.unwrap();
        assert!((circle.center().x() - 0.0).abs() < f64::EPSILON);
        assert!((circle.center().y() - 0.0).abs() < f64::EPSILON);
        assert!((circle.radius() - 1.0).abs() < f64::EPSILON);
    }
}
