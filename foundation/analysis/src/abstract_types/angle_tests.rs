use super::Angle;
use crate::test_constants::{TOLERANCE_F32, TOLERANCE_F64};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_angle_creation() {
        let angle_rad = Angle::from_radians(1.0f64);
        let angle_deg = Angle::from_degrees(180.0f64);

        assert!((angle_rad.to_radians() - 1.0).abs() < TOLERANCE_F64);
        assert!((angle_deg.to_radians() - std::f64::consts::PI).abs() < TOLERANCE_F64);
    }

    #[test]
    fn test_angle_conversion() {
        let angle = Angle::from_degrees(90.0f64);
        assert!((angle.to_radians() - std::f64::consts::PI / 2.0).abs() < TOLERANCE_F64);
        assert!((angle.to_degrees() - 90.0).abs() < TOLERANCE_F64);
    }

    #[test]
    fn test_angle_arithmetic() {
        let a1 = Angle::from_degrees(45.0f64);
        let a2 = Angle::from_degrees(30.0f64);

        let sum = a1 + a2;
        assert!((sum.to_degrees() - 75.0).abs() < TOLERANCE_F64);

        let diff = a1 - a2;
        assert!((diff.to_degrees() - 15.0).abs() < TOLERANCE_F64);
    }

    #[test]
    fn test_trigonometric_functions() {
        let angle = Angle::from_degrees(90.0f64);
        assert!((angle.sin() - 1.0).abs() < TOLERANCE_F64);
        assert!(angle.cos().abs() < TOLERANCE_F64);

        let angle_45 = Angle::from_degrees(45.0f64);
        let sqrt2_over_2 = std::f64::consts::SQRT_2 / 2.0;
        assert!((angle_45.sin() - sqrt2_over_2).abs() < TOLERANCE_F64);
        assert!((angle_45.cos() - sqrt2_over_2).abs() < TOLERANCE_F64);
    }

    #[test]
    fn test_angle_normalization() {
        let angle1 = Angle::from_degrees(450.0f64);
        let normalized = angle1.normalize();
        assert!((normalized.to_degrees() - 90.0).abs() < TOLERANCE_F64);

        let angle2 = Angle::from_degrees(-90.0f64);
        let normalized2 = angle2.normalize();
        assert!((normalized2.to_degrees() - 270.0).abs() < TOLERANCE_F64);

        let angle3 = Angle::from_degrees(270.0f64);
        let signed_normalized = angle3.normalize_signed();
        assert!((signed_normalized.to_degrees() - (-90.0)).abs() < TOLERANCE_F64);

        let angle4 = Angle::from_degrees(-270.0f64);
        let signed_normalized2 = angle4.normalize_signed();
        assert!((signed_normalized2.to_degrees() - 90.0).abs() < TOLERANCE_F64);
    }

    #[test]
    fn test_angle_constants() {
        assert!((Angle::<f64>::deg_30().to_degrees() - 30.0).abs() < TOLERANCE_F64);
        assert!((Angle::<f64>::deg_45().to_degrees() - 45.0).abs() < TOLERANCE_F64);
        assert!((Angle::<f64>::deg_60().to_degrees() - 60.0).abs() < TOLERANCE_F64);
        assert!((Angle::<f64>::right_angle().to_degrees() - 90.0).abs() < TOLERANCE_F64);
        assert!((Angle::<f64>::deg_120().to_degrees() - 120.0).abs() < TOLERANCE_F64);
        assert!((Angle::<f64>::straight_angle().to_degrees() - 180.0).abs() < TOLERANCE_F64);
        assert!((Angle::<f64>::three_quarter_angle().to_degrees() - 270.0).abs() < TOLERANCE_F64);
        assert!((Angle::<f64>::full_angle().to_degrees() - 360.0).abs() < TOLERANCE_F64);

        assert!((Angle::<f32>::deg_45().to_degrees() - 45.0).abs() < TOLERANCE_F32);
        assert!((Angle::<f32>::right_angle().to_degrees() - 90.0).abs() < TOLERANCE_F32);
    }
}
