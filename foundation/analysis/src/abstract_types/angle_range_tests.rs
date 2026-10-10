use super::angle::Angle;
use super::angle_range::AngleRange;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_constants::TOLERANCE_F64;

    fn deg(value: f64) -> Angle<f64> {
        Angle::from_degrees(value)
    }

    #[test]
    fn contains_angles_in_simple_range() {
        let range = AngleRange::from_ccw_bounds(deg(30.0), deg(120.0)).unwrap();

        assert!(range.contains(deg(30.0), 0.0));
        assert!(range.contains(deg(75.0), 0.0));
        assert!(range.contains(deg(120.0), 0.0));
        assert!(!range.contains(deg(150.0), 0.0));
        assert!(!range.contains(deg(10.0), 0.0));

        // 周期の違う同じ角度も含む
        assert!(range.contains(deg(75.0 + 360.0), 0.0));
        assert!(range.contains(deg(75.0 - 720.0), 0.0));
    }

    #[test]
    fn contains_angles_in_range_crossing_zero() {
        let range = AngleRange::from_ccw_bounds(deg(300.0), deg(60.0)).unwrap();

        assert!((range.span().to_degrees() - 120.0).abs() < TOLERANCE_F64);
        assert!(range.contains(deg(330.0), 0.0));
        assert!(range.contains(deg(0.0), 0.0));
        assert!(range.contains(deg(360.0), 0.0));
        assert!(range.contains(deg(-30.0), 0.0));
        assert!(range.contains(deg(60.0), 0.0));
        assert!(!range.contains(deg(90.0), 0.0));
        assert!(!range.contains(deg(270.0), 0.0));
    }

    #[test]
    fn tolerance_extends_both_ends() {
        let range = AngleRange::from_ccw_bounds(deg(30.0), deg(120.0)).unwrap();
        let tolerance = 1.0_f64.to_radians();

        assert!(range.contains(deg(29.5), tolerance));
        assert!(range.contains(deg(120.5), tolerance));
        assert!(!range.contains(deg(28.0), tolerance));
        assert!(!range.contains(deg(122.0), tolerance));

        // 0° を跨いで開始角の手前を判定する
        let range = AngleRange::from_ccw_bounds(deg(0.0), deg(90.0)).unwrap();
        assert!(range.contains(deg(359.5), tolerance));
        assert!(!range.contains(deg(358.0), tolerance));
    }

    #[test]
    fn full_range_contains_every_angle() {
        let full = AngleRange::full(deg(45.0)).unwrap();
        assert!(full.is_full());
        assert!(full.contains(deg(44.0), 0.0));
        assert!(full.contains(deg(-1000.0), 0.0));

        // 差が 2π の整数倍の上下限は全周
        let full = AngleRange::from_ccw_bounds(deg(0.0), deg(360.0)).unwrap();
        assert!(full.is_full());
        assert!(full.contains(deg(180.0), 0.0));

        // 2π を超える幅は全周
        assert!(AngleRange::new(deg(0.0), deg(720.0)).unwrap().is_full());
    }

    #[test]
    fn rejects_empty_and_non_finite_ranges() {
        assert!(AngleRange::from_ccw_bounds(deg(30.0), deg(30.0)).is_none());
        assert!(AngleRange::new(deg(0.0), deg(0.0)).is_none());
        assert!(AngleRange::new(deg(0.0), deg(-10.0)).is_none());
        assert!(AngleRange::new(Angle::from_radians(f64::NAN), deg(10.0)).is_none());
        assert!(AngleRange::new(deg(0.0), Angle::from_radians(f64::INFINITY)).is_none());

        let range = AngleRange::from_ccw_bounds(deg(0.0), deg(90.0)).unwrap();
        assert!(!range.contains(Angle::from_radians(f64::NAN), 0.0));
    }

    #[test]
    fn start_and_end_are_normalized() {
        let range = AngleRange::from_ccw_bounds(deg(-60.0), deg(420.0)).unwrap();
        assert!((range.start().to_degrees() - 300.0).abs() < TOLERANCE_F64);
        assert!((range.span().to_degrees() - 120.0).abs() < TOLERANCE_F64);
        assert!((range.end().to_degrees() - 60.0).abs() < TOLERANCE_F64);
    }
}
