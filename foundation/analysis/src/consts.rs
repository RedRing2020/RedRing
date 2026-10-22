//! RedRing の数値解析・幾何計算で共有する定数群。
//!
//! 標準ライブラリの定数を優先し、
//! 用途別の許容誤差や数値計算閾値だけをこのモジュールで管理します。

/// 数値計算アルゴリズム用の閾値
pub mod numerical {
    /// ニュートン法で微分がゼロとみなされる閾値（f32）
    pub const DERIVATIVE_ZERO_THRESHOLD_F32: f32 = 1e-6;

    /// ニュートン法で微分がゼロとみなされる閾値（f64）
    pub const DERIVATIVE_ZERO_THRESHOLD_F64: f64 = 1e-12;

    /// 線形 solver の特異判定・ピボット判定に使う既定閾値（f32）
    pub const LINEAR_SOLVER_TOLERANCE_F32: f32 = 1e-6;

    /// 線形 solver の特異判定・ピボット判定に使う既定閾値
    pub const LINEAR_SOLVER_TOLERANCE_F64: f64 = 1e-15;

    /// カーネル内部のゼロ判定（ゼロベクトル長・分母ゼロ近傍）用固定閾値（f64）
    pub const KERNEL_NUMERICAL_ZERO_THRESHOLD_F64: f64 = 1e-12;

    /// カーネル内部のゼロ判定（ゼロベクトル長・分母ゼロ近傍）用固定閾値（f32）
    pub const KERNEL_NUMERICAL_ZERO_THRESHOLD_F32: f32 = 1e-6;

    /// 正規化済みベクトルの外積を 0 とみなす数値誤差閾値（f64）
    pub const PARALLEL_CROSS_ERROR_TOLERANCE_F64: f64 = 1e-10;

    /// 正規化済みベクトルの外積を 0 とみなす数値誤差閾値（f32）
    pub const PARALLEL_CROSS_ERROR_TOLERANCE_F32: f32 = 1e-6;

    /// 正規化済みベクトルの内積を 0 とみなす数値誤差閾値（f64）
    pub const ORTHOGONALITY_DOT_ERROR_TOLERANCE_F64: f64 = 1e-10;

    /// 正規化済みベクトルの内積を 0 とみなす数値誤差閾値（f32）
    pub const ORTHOGONALITY_DOT_ERROR_TOLERANCE_F32: f32 = 1e-6;
}

/// 特殊数学定数
pub mod special {
    /// 黄金比 φ
    pub const GOLDEN_RATIO_F64: f64 = std::f64::consts::GOLDEN_RATIO;
    pub const GOLDEN_RATIO_F32: f32 = std::f32::consts::GOLDEN_RATIO;

    /// 自然対数 ln(2)
    pub const LN_2_F64: f64 = std::f64::consts::LN_2;
    pub const LN_2_F32: f32 = std::f32::consts::LN_2;

    /// 自然対数 ln(10)
    pub const LN_10_F64: f64 = std::f64::consts::LN_10;
    pub const LN_10_F32: f32 = std::f32::consts::LN_10;

    /// 平方根 √3
    pub const SQRT_3_F64: f64 = 1.7320508075688772;
    pub const SQRT_3_F32: f32 = 1.7320508_f32;

    /// 平方根 √3 の半分
    pub const SQRT_3_OVER_2_F64: f64 = SQRT_3_F64 / 2.0;
    pub const SQRT_3_OVER_2_F32: f32 = SQRT_3_F32 / 2.0;
}

/// 幾何計算用の定数
pub mod game {
    /// 数学定数（f32）
    pub const PI: f32 = std::f32::consts::PI;
    pub const TAU: f32 = std::f32::consts::TAU;
    pub const E: f32 = std::f32::consts::E;

    /// 円・円弧関連の定数（f32）
    pub const PI_2: f32 = PI / 2.0; // π/2 (90度)
    pub const PI_4: f32 = PI / 4.0; // π/4 (45度)
    pub const PI_3: f32 = PI / 3.0; // π/3 (60度)
    pub const PI_6: f32 = PI / 6.0; // π/6 (30度)

    /// 角度変換定数（f32）
    pub const DEG_TO_RAD: f32 = PI / 180.0;
    pub const RAD_TO_DEG: f32 = 180.0 / PI;

    /// 幾何計算用の許容誤差（f32用）
    pub const GEOMETRIC_TOLERANCE: f32 = 1e-6;

    /// 距離計算用の許容誤差（f32用）
    pub const GEOMETRIC_DISTANCE_TOLERANCE: f32 = 1e-6;

    /// 角度計算用の許容誤差（f32用、ラジアン）
    pub const GEOMETRIC_ANGLE_TOLERANCE: f32 = 1e-6;

    /// よく使われる角度（f32）
    pub const ANGLE_0: f32 = 0.0;
    pub const ANGLE_30: f32 = PI_6;
    pub const ANGLE_45: f32 = PI_4;
    pub const ANGLE_60: f32 = PI_3;
    pub const ANGLE_90: f32 = PI_2;
    pub const ANGLE_180: f32 = PI;
    pub const ANGLE_270: f32 = 3.0 * PI_2;
    pub const ANGLE_360: f32 = TAU;
}

/// 高精度幾何計算用の定数
pub mod precision {
    /// 数学定数（f64）
    pub const PI: f64 = std::f64::consts::PI;
    pub const TAU: f64 = std::f64::consts::TAU;
    pub const E: f64 = std::f64::consts::E;

    /// 円・円弧関連の定数（f64）
    pub const PI_2: f64 = PI / 2.0; // π/2 (90度)
    pub const PI_4: f64 = PI / 4.0; // π/4 (45度)
    pub const PI_3: f64 = PI / 3.0; // π/3 (60度)
    pub const PI_6: f64 = PI / 6.0; // π/6 (30度)

    /// 角度変換定数（f64）
    pub const DEG_TO_RAD: f64 = PI / 180.0;
    pub const RAD_TO_DEG: f64 = 180.0 / PI;

    /// 幾何計算用の許容誤差（f64用）
    pub const GEOMETRIC_TOLERANCE: f64 = 1e-10;

    /// 距離計算用の許容誤差（f64用）
    pub const GEOMETRIC_DISTANCE_TOLERANCE: f64 = 1e-10;

    /// 角度計算用の許容誤差（f64用、ラジアン）
    pub const GEOMETRIC_ANGLE_TOLERANCE: f64 = 1e-12;

    /// よく使われる角度（f64）
    pub const ANGLE_0: f64 = 0.0;
    pub const ANGLE_30: f64 = PI_6;
    pub const ANGLE_45: f64 = PI_4;
    pub const ANGLE_60: f64 = PI_3;
    pub const ANGLE_90: f64 = PI_2;
    pub const ANGLE_180: f64 = PI;
    pub const ANGLE_270: f64 = 3.0 * PI_2;
    pub const ANGLE_360: f64 = TAU;
}

/// 後方互換性のための precision エイリアス
pub const PI: f64 = precision::PI;
pub const TAU: f64 = precision::TAU;
pub const E: f64 = precision::E;
pub const PI_2: f64 = precision::PI_2;
pub const PI_4: f64 = precision::PI_4;
pub const PI_3: f64 = precision::PI_3;
pub const PI_6: f64 = precision::PI_6;
pub const DEG_TO_RAD: f64 = precision::DEG_TO_RAD;
pub const RAD_TO_DEG: f64 = precision::RAD_TO_DEG;
pub const GEOMETRIC_TOLERANCE: f64 = precision::GEOMETRIC_TOLERANCE;
pub const GEOMETRIC_DISTANCE_TOLERANCE: f64 = precision::GEOMETRIC_DISTANCE_TOLERANCE;
pub const GEOMETRIC_ANGLE_TOLERANCE: f64 = precision::GEOMETRIC_ANGLE_TOLERANCE;

/// 型別の幾何計算トレランス
pub trait GeometricTolerance {
    /// その型に適した幾何計算用の許容誤差（汎用）
    const TOLERANCE: Self;

    /// その型に適した距離計算用の許容誤差
    const DISTANCE_TOLERANCE: Self;

    /// その型に適した角度計算用の許容誤差（ラジアン単位）
    const ANGLE_TOLERANCE: Self;

    /// その型に適した平行判定（外積）誤差閾値
    const PARALLEL_CROSS_ERROR_TOLERANCE: Self;

    /// その型に適した直交判定（内積）誤差閾値
    const ORTHOGONALITY_DOT_ERROR_TOLERANCE: Self;
}

impl GeometricTolerance for f32 {
    const TOLERANCE: f32 = game::GEOMETRIC_TOLERANCE;
    const DISTANCE_TOLERANCE: f32 = game::GEOMETRIC_DISTANCE_TOLERANCE;
    const ANGLE_TOLERANCE: f32 = game::GEOMETRIC_ANGLE_TOLERANCE;
    const PARALLEL_CROSS_ERROR_TOLERANCE: f32 = numerical::PARALLEL_CROSS_ERROR_TOLERANCE_F32;
    const ORTHOGONALITY_DOT_ERROR_TOLERANCE: f32 = numerical::ORTHOGONALITY_DOT_ERROR_TOLERANCE_F32;
}

impl GeometricTolerance for f64 {
    const TOLERANCE: f64 = precision::GEOMETRIC_TOLERANCE;
    const DISTANCE_TOLERANCE: f64 = precision::GEOMETRIC_DISTANCE_TOLERANCE;
    const ANGLE_TOLERANCE: f64 = precision::GEOMETRIC_ANGLE_TOLERANCE;
    const PARALLEL_CROSS_ERROR_TOLERANCE: f64 = numerical::PARALLEL_CROSS_ERROR_TOLERANCE_F64;
    const ORTHOGONALITY_DOT_ERROR_TOLERANCE: f64 = numerical::ORTHOGONALITY_DOT_ERROR_TOLERANCE_F64;
}

// 数値計算定数の再エクスポート
pub const DERIVATIVE_ZERO_THRESHOLD_F32: f32 = numerical::DERIVATIVE_ZERO_THRESHOLD_F32;
pub const DERIVATIVE_ZERO_THRESHOLD_F64: f64 = numerical::DERIVATIVE_ZERO_THRESHOLD_F64;

/// テスト用の統一トレランス定数
pub mod test_constants {
    use super::GeometricTolerance;

    /// f64 テスト用の標準許容誤差
    pub const TOLERANCE_F64: f64 = <f64 as GeometricTolerance>::TOLERANCE;

    /// f32 テスト用の標準許容誤差
    pub const TOLERANCE_F32: f32 = <f32 as GeometricTolerance>::TOLERANCE;

    /// 角度テスト用の許容誤差（f64）
    pub const ANGLE_TOLERANCE_F64: f64 = <f64 as GeometricTolerance>::ANGLE_TOLERANCE;

    /// 角度テスト用の許容誤差（f32）
    pub const ANGLE_TOLERANCE_F32: f32 = <f32 as GeometricTolerance>::ANGLE_TOLERANCE;

    /// 距離テスト用の許容誤差（f64）
    pub const DISTANCE_TOLERANCE_F64: f64 = <f64 as GeometricTolerance>::DISTANCE_TOLERANCE;

    /// 距離テスト用の許容誤差（f32）
    pub const DISTANCE_TOLERANCE_F32: f32 = <f32 as GeometricTolerance>::DISTANCE_TOLERANCE;

    // 数値計算アルゴリズム専用の許容誤差

    /// ソルバー用高精度許容誤差（f32）
    pub const SOLVER_TOLERANCE_F32: f32 = super::numerical::LINEAR_SOLVER_TOLERANCE_F32;

    /// ソルバー用高精度許容誤差
    pub const SOLVER_TOLERANCE_F64: f64 = super::numerical::LINEAR_SOLVER_TOLERANCE_F64;

    /// 数値積分用許容誤差（標準精度）
    pub const INTEGRATION_TOLERANCE: f64 = 1e-4;

    /// 数値積分用許容誤差（緩い精度）
    pub const INTEGRATION_TOLERANCE_LOOSE: f64 = 1e-3;

    /// 数値積分用許容誤差（高精度）
    pub const INTEGRATION_TOLERANCE_STRICT: f64 = 1e-6;

    /// 導関数を確かめる中心差分の刻み幅（f64）
    pub const FINITE_DIFFERENCE_STEP_F64: f64 = 1e-6;

    /// 導関数と中心差分を比べるときの許容誤差（f64。刻み幅の 1 次の誤差が残る、2 階導関数が不連続な点も含む）
    pub const FINITE_DIFFERENCE_TOLERANCE_F64: f64 = 1e-5;
}

#[cfg(test)]
mod tests {
    use super::{special, GeometricTolerance};
    use crate::consts::test_constants::{
        SOLVER_TOLERANCE_F32, SOLVER_TOLERANCE_F64, TOLERANCE_F32, TOLERANCE_F64,
    };

    #[test]
    fn test_special_constants() {
        // 黄金比のテスト
        let golden_ratio_f64 = special::GOLDEN_RATIO_F64;
        let expected_golden_ratio = (1.0 + 5.0_f64.sqrt()) / 2.0;
        assert!((golden_ratio_f64 - expected_golden_ratio).abs() < TOLERANCE_F64);

        let golden_ratio_f32 = special::GOLDEN_RATIO_F32;
        let expected_golden_ratio_f32 = (1.0 + 5.0_f32.sqrt()) / 2.0;
        assert!((golden_ratio_f32 - expected_golden_ratio_f32).abs() < TOLERANCE_F32);

        // ln(2)のテスト
        let ln_2_f64 = special::LN_2_F64;
        assert!((ln_2_f64 - std::f64::consts::LN_2).abs() < SOLVER_TOLERANCE_F64);

        // ln(10)のテスト
        let ln_10_f64 = special::LN_10_F64;
        assert!((ln_10_f64 - std::f64::consts::LN_10).abs() < SOLVER_TOLERANCE_F64);

        // √3のテスト
        let sqrt_3_f64 = special::SQRT_3_F64;
        assert!((sqrt_3_f64 - 3.0_f64.sqrt()).abs() < SOLVER_TOLERANCE_F64);

        let sqrt_3_over_2_f64 = special::SQRT_3_OVER_2_F64;
        assert!((sqrt_3_over_2_f64 - (3.0_f64.sqrt() / 2.0)).abs() < SOLVER_TOLERANCE_F64);
    }

    #[test]
    fn test_tolerance_constants() {
        // GeometricToleranceトレイトのテスト
        let geometric_f64 = <f64 as GeometricTolerance>::TOLERANCE;
        let geometric_f32 = <f32 as GeometricTolerance>::TOLERANCE;

        // f64は高精度、f32は低精度であることを確認
        assert!(geometric_f64 < geometric_f32 as f64);
        assert_eq!(geometric_f64, 1e-10);
        assert_eq!(geometric_f32, 1e-6);

        // 角度・距離許容誤差もテスト
        let angle_f64 = <f64 as GeometricTolerance>::ANGLE_TOLERANCE;
        let distance_f64 = <f64 as GeometricTolerance>::DISTANCE_TOLERANCE;
        assert_eq!(angle_f64, 1e-12);
        assert_eq!(distance_f64, 1e-10);

        let angle_f32 = <f32 as GeometricTolerance>::ANGLE_TOLERANCE;
        let distance_f32 = <f32 as GeometricTolerance>::DISTANCE_TOLERANCE;
        assert_eq!(angle_f32, 1e-6);
        assert_eq!(distance_f32, 1e-6);

        assert_eq!(SOLVER_TOLERANCE_F32, 1e-6_f32);
        assert_eq!(SOLVER_TOLERANCE_F64, 1e-15_f64);

        assert_eq!(super::numerical::DERIVATIVE_ZERO_THRESHOLD_F32, 1e-6_f32);
        assert_eq!(super::numerical::DERIVATIVE_ZERO_THRESHOLD_F64, 1e-12_f64);
    }
}
