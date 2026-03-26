use crate::angle_utils::normalize_angle_deg;

/// 制約検証結果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintViolation {
    None,
    NotFinite,
    InvalidRange,
    BelowMinimum,
    AboveMaximum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidationResult {
    pub violation: ConstraintViolation,
}

impl ValidationResult {
    pub fn ok() -> Self {
        Self {
            violation: ConstraintViolation::None,
        }
    }

    pub fn is_valid(self) -> bool {
        self.violation == ConstraintViolation::None
    }
}

/// 線形軸の移動範囲（mm）を検証する。
pub fn validate_linear_travel_mm(
    requested_mm: f64,
    min_mm: f64,
    max_mm: f64,
    tolerance_mm: f64,
) -> ValidationResult {
    validate_closed_range(requested_mm, min_mm, max_mm, tolerance_mm)
}

/// 線形速度（mm/min）を検証する。
pub fn validate_linear_speed_mm_per_min(
    requested: f64,
    min_limit: f64,
    max_limit: f64,
    tolerance: f64,
) -> ValidationResult {
    validate_closed_range(requested, min_limit, max_limit, tolerance)
}

/// 回転速度（deg/min）を検証する。
pub fn validate_rotary_speed_deg_per_min(
    requested: f64,
    min_limit: f64,
    max_limit: f64,
    tolerance: f64,
) -> ValidationResult {
    validate_closed_range(requested, min_limit, max_limit, tolerance)
}

/// 線形加速度（mm/s^2）を検証する。
pub fn validate_linear_acceleration_mm_per_s2(
    requested: f64,
    min_limit: f64,
    max_limit: f64,
    tolerance: f64,
) -> ValidationResult {
    validate_closed_range(requested, min_limit, max_limit, tolerance)
}

/// 回転加速度（deg/s^2）を検証する。
pub fn validate_rotary_acceleration_deg_per_s2(
    requested: f64,
    min_limit: f64,
    max_limit: f64,
    tolerance: f64,
) -> ValidationResult {
    validate_closed_range(requested, min_limit, max_limit, tolerance)
}

/// 回転軸角度（deg）の範囲検証。
///
/// `min_deg > max_deg` の場合は 0 度跨ぎの範囲として扱う。
pub fn validate_rotary_angle_deg(
    requested_deg: f64,
    min_deg: f64,
    max_deg: f64,
    tolerance_deg: f64,
) -> ValidationResult {
    if !requested_deg.is_finite() || !min_deg.is_finite() || !max_deg.is_finite() {
        return ValidationResult {
            violation: ConstraintViolation::NotFinite,
        };
    }

    let tol = tolerance_deg.abs();
    let requested = normalize_angle_deg(requested_deg);
    let min_norm = normalize_angle_deg(min_deg);
    let max_norm = normalize_angle_deg(max_deg);

    if (min_norm - max_norm).abs() <= tol {
        return ValidationResult::ok();
    }

    let in_range = if min_norm < max_norm {
        requested >= min_norm - tol && requested <= max_norm + tol
    } else {
        requested >= min_norm - tol || requested <= max_norm + tol
    };

    if in_range {
        ValidationResult::ok()
    } else {
        ValidationResult {
            violation: ConstraintViolation::InvalidRange,
        }
    }
}

fn validate_closed_range(value: f64, min: f64, max: f64, tolerance: f64) -> ValidationResult {
    if !value.is_finite() || !min.is_finite() || !max.is_finite() {
        return ValidationResult {
            violation: ConstraintViolation::NotFinite,
        };
    }

    if min > max {
        return ValidationResult {
            violation: ConstraintViolation::InvalidRange,
        };
    }

    let tol = tolerance.abs();

    if value < min - tol {
        return ValidationResult {
            violation: ConstraintViolation::BelowMinimum,
        };
    }

    if value > max + tol {
        return ValidationResult {
            violation: ConstraintViolation::AboveMaximum,
        };
    }

    ValidationResult::ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_travel_accepts_within_tolerance() {
        let result = validate_linear_travel_mm(10.0001, 0.0, 10.0, 0.001);
        assert!(result.is_valid());
    }

    #[test]
    fn linear_travel_detects_violations() {
        let low = validate_linear_travel_mm(-0.1, 0.0, 100.0, 0.0);
        let high = validate_linear_travel_mm(100.1, 0.0, 100.0, 0.0);

        assert_eq!(low.violation, ConstraintViolation::BelowMinimum);
        assert_eq!(high.violation, ConstraintViolation::AboveMaximum);
    }

    #[test]
    fn linear_speed_and_acceleration_validate_like_ranges() {
        assert!(validate_linear_speed_mm_per_min(1200.0, 0.0, 2000.0, 0.0).is_valid());
        assert!(validate_rotary_speed_deg_per_min(360.0, 0.0, 720.0, 0.0).is_valid());
        assert!(validate_linear_acceleration_mm_per_s2(30.0, 0.0, 50.0, 0.0).is_valid());
        assert!(validate_rotary_acceleration_deg_per_s2(100.0, 0.0, 200.0, 0.0).is_valid());
    }

    #[test]
    fn rotary_angle_supports_wrapped_ranges() {
        let ok = validate_rotary_angle_deg(350.0, 300.0, 30.0, 0.0);
        let ng = validate_rotary_angle_deg(120.0, 300.0, 30.0, 0.0);

        assert!(ok.is_valid());
        assert_eq!(ng.violation, ConstraintViolation::InvalidRange);
    }

    #[test]
    fn invalid_ranges_and_non_finite_are_rejected() {
        let invalid = validate_linear_travel_mm(10.0, 20.0, 0.0, 0.0);
        let non_finite = validate_linear_travel_mm(f64::NAN, 0.0, 1.0, 0.0);

        assert_eq!(invalid.violation, ConstraintViolation::InvalidRange);
        assert_eq!(non_finite.violation, ConstraintViolation::NotFinite);
    }
}
