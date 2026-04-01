//! 角度処理ユーティリティ
//!
//! すべて度数法（deg）で扱う。

pub const FULL_TURN_DEG: f64 = 360.0;
pub const HALF_TURN_DEG: f64 = 180.0;

/// 回転軸の実移動量を「周数 + 剰余角」で保持する。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AngularPosition {
    /// 完全回転の周数（負値は逆方向）
    pub full_rotations: i32,
    /// 剰余角 [deg], 常に 0 <= remainder_deg < 360
    pub remainder_deg: f64,
}

impl AngularPosition {
    /// 総角度 [deg] を返す。
    pub fn total_deg(self) -> f64 {
        self.full_rotations as f64 * FULL_TURN_DEG + self.remainder_deg
    }

    /// 総角度 [deg] から AngularPosition を構築する。
    pub fn from_total_deg(total_deg: f64) -> Self {
        let full_rotations = (total_deg / FULL_TURN_DEG).floor() as i32;
        let remainder = total_deg.rem_euclid(FULL_TURN_DEG);
        Self {
            full_rotations,
            remainder_deg: remainder,
        }
    }
}

/// [0, 360) に正規化する。
pub fn normalize_angle_deg(angle_deg: f64) -> f64 {
    let normalized = angle_deg.rem_euclid(FULL_TURN_DEG);
    if normalized == FULL_TURN_DEG {
        0.0
    } else {
        normalized
    }
}

/// [0, 360) に正規化する。
///
/// 設計側で使用する用語に合わせた公開名。挙動は `normalize_angle_deg` と同じ。
pub fn normalize_to_0_360(angle_deg: f64) -> f64 {
    normalize_angle_deg(angle_deg)
}

/// (-180, 180] に正規化する。
pub fn normalize_angle_signed_deg(angle_deg: f64) -> f64 {
    let mut normalized = normalize_angle_deg(angle_deg);
    if normalized > HALF_TURN_DEG {
        normalized -= FULL_TURN_DEG;
    }
    normalized
}

/// (-180, 180] に正規化する。
///
/// 設計側で使用する用語に合わせた公開名。挙動は `normalize_angle_signed_deg` と同じ。
pub fn normalize_to_minus180_180(angle_deg: f64) -> f64 {
    normalize_angle_signed_deg(angle_deg)
}

/// `from_deg` から `to_deg` への最短角差を返す。
///
/// 戻り値は [-180, 180]。
pub fn shortest_angular_delta_deg(from_deg: f64, to_deg: f64) -> f64 {
    normalize_angle_signed_deg(to_deg - from_deg)
}

/// from から to への最短角度差を返す。
///
/// 設計側で使用する用語に合わせた公開名。挙動は `shortest_angular_delta_deg` と同じ。
pub fn shortest_angle(from_deg: f64, to_deg: f64) -> f64 {
    shortest_angular_delta_deg(from_deg, to_deg)
}

/// 許容誤差付きで角度同値を判定する。
pub fn are_angles_equivalent_deg(a_deg: f64, b_deg: f64, tolerance_deg: f64) -> bool {
    shortest_angular_delta_deg(a_deg, b_deg).abs() <= tolerance_deg.abs()
}

/// 0/360 同値を許容誤差付きで判定する。
///
/// 設計側で使用する用語に合わせた公開名。挙動は `are_angles_equivalent_deg` と同じ。
pub fn is_equivalent_0_360(a_deg: f64, b_deg: f64, tolerance_deg: f64) -> bool {
    are_angles_equivalent_deg(a_deg, b_deg, tolerance_deg)
}

/// 巻き戻し（rewind）方針。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RewindPolicy {
    /// 現在角度から最短経路で到達する。
    Shortest,
    /// 正方向（CCW）のみで到達する。
    PositiveOnly,
    /// 負方向（CW）のみで到達する。
    NegativeOnly,
}

/// 方針に従い、連続角度系での目標角を返す。
///
/// 例: current=350, target=10, Shortest -> 370
pub fn rewound_target_deg(current_deg: f64, target_deg: f64, policy: RewindPolicy) -> f64 {
    let current_norm = normalize_angle_deg(current_deg);
    let target_norm = normalize_angle_deg(target_deg);

    match policy {
        RewindPolicy::Shortest => {
            current_deg + shortest_angular_delta_deg(current_norm, target_norm)
        }
        RewindPolicy::PositiveOnly => {
            let delta = (target_norm - current_norm).rem_euclid(FULL_TURN_DEG);
            current_deg + delta
        }
        RewindPolicy::NegativeOnly => {
            let delta = -((current_norm - target_norm).rem_euclid(FULL_TURN_DEG));
            current_deg + delta
        }
    }
}

/// 離散角列を連続角列へ変換する。
pub fn unwind_angles_deg(angles_deg: &[f64]) -> Vec<f64> {
    if angles_deg.is_empty() {
        return Vec::new();
    }

    let mut unwound = Vec::with_capacity(angles_deg.len());
    unwound.push(angles_deg[0]);

    for &angle in &angles_deg[1..] {
        let prev = *unwound.last().expect("unwound is never empty");
        unwound.push(rewound_target_deg(prev, angle, RewindPolicy::Shortest));
    }

    unwound
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_angle_deg_wraps_correctly() {
        assert!((normalize_angle_deg(370.0) - 10.0).abs() < 1e-12);
        assert!((normalize_angle_deg(-10.0) - 350.0).abs() < 1e-12);
    }

    #[test]
    fn shortest_angular_delta_prefers_short_path() {
        assert!((shortest_angular_delta_deg(350.0, 10.0) - 20.0).abs() < 1e-12);
        assert!((shortest_angular_delta_deg(10.0, 350.0) + 20.0).abs() < 1e-12);
    }

    #[test]
    fn angle_equivalence_works_across_wrap() {
        assert!(are_angles_equivalent_deg(0.0, 360.0, 1e-9));
        assert!(are_angles_equivalent_deg(-180.0, 180.0, 1e-9));
        assert!(!are_angles_equivalent_deg(0.0, 1.0, 0.1));
    }

    #[test]
    fn rewound_target_supports_policies() {
        assert!((rewound_target_deg(350.0, 10.0, RewindPolicy::Shortest) - 370.0).abs() < 1e-12);
        assert!(
            (rewound_target_deg(350.0, 10.0, RewindPolicy::PositiveOnly) - 370.0).abs() < 1e-12
        );
        assert!((rewound_target_deg(350.0, 10.0, RewindPolicy::NegativeOnly) - 10.0).abs() < 1e-12);
    }

    #[test]
    fn unwind_angles_creates_continuous_series() {
        let src = [350.0, 355.0, 2.0, 8.0, 355.0];
        let unwound = unwind_angles_deg(&src);

        assert_eq!(unwound.len(), 5);
        assert!((unwound[0] - 350.0).abs() < 1e-12);
        assert!((unwound[2] - 362.0).abs() < 1e-12);
        assert!((unwound[3] - 368.0).abs() < 1e-12);
        assert!((unwound[4] - 355.0).abs() < 1e-12);
    }

    #[test]
    fn angular_position_roundtrip_examples() {
        let p450 = AngularPosition::from_total_deg(450.0);
        assert_eq!(p450.full_rotations, 1);
        assert!((p450.remainder_deg - 90.0).abs() < 1e-12);
        assert!((p450.total_deg() - 450.0).abs() < 1e-12);

        let pneg90 = AngularPosition::from_total_deg(-90.0);
        assert_eq!(pneg90.full_rotations, -1);
        assert!((pneg90.remainder_deg - 270.0).abs() < 1e-12);
        assert!((pneg90.total_deg() - (-90.0)).abs() < 1e-12);

        let p720 = AngularPosition::from_total_deg(720.0);
        assert_eq!(p720.full_rotations, 2);
        assert!((p720.remainder_deg - 0.0).abs() < 1e-12);
        assert!((p720.total_deg() - 720.0).abs() < 1e-12);
    }

    #[test]
    fn design_named_helpers_delegate_correctly() {
        assert!((normalize_to_0_360(370.0) - 10.0).abs() < 1e-12);
        assert!((normalize_to_minus180_180(350.0) + 10.0).abs() < 1e-12);
        assert!((shortest_angle(350.0, 10.0) - 20.0).abs() < 1e-12);
        assert!(is_equivalent_0_360(0.0, 360.0, 1e-9));
    }
}
