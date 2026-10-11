//! NURBS関連の定数

use core::any::TypeId;

use crate::Scalar;

#[inline]
fn select_scalar_value<T: Scalar>(f32_value: f32, f64_value: f64) -> T {
    if TypeId::of::<T>() == TypeId::of::<f32>() {
        T::from_f32(f32_value)
    } else {
        // f32以外はf64側へフォールバックする（geo_contractsの既定値選択と整合）。
        T::from_f64(f64_value)
    }
}

/// 数値解法で使うしきい値群
pub mod solver {
    use crate::Scalar;

    use super::select_scalar_value;

    /// 境界付きニュートン法の収束許容誤差（f32）
    pub const NEWTON_TOLERANCE_F32: f32 = 1e-6;

    /// 境界付きニュートン法の収束許容誤差（f64）
    pub const NEWTON_TOLERANCE_F64: f64 = 1e-10;

    /// 境界付きニュートン法の数値微分ステップ幅（f32）
    pub const NEWTON_DIFF_STEP_F32: f32 = 1e-4;

    /// 境界付きニュートン法の数値微分ステップ幅（f64）
    pub const NEWTON_DIFF_STEP_F64: f64 = 1e-7;

    /// 導関数の中央差分近似で使う微小ステップ幅（f32）
    pub const DERIVATIVE_STEP_F32: f32 = 1e-4;

    /// 導関数の中央差分近似で使う微小ステップ幅（f64）
    pub const DERIVATIVE_STEP_F64: f64 = 1e-8;

    /// 境界付きニュートン法の収束許容誤差を型別に取得
    #[must_use]
    pub fn newton_tolerance<T: Scalar>() -> T {
        select_scalar_value::<T>(NEWTON_TOLERANCE_F32, NEWTON_TOLERANCE_F64)
    }

    /// 境界付きニュートン法の数値微分ステップ幅を型別に取得
    #[must_use]
    pub fn newton_diff_step<T: Scalar>() -> T {
        select_scalar_value::<T>(NEWTON_DIFF_STEP_F32, NEWTON_DIFF_STEP_F64)
    }

    /// 導関数の中央差分近似で使う微小ステップ幅を型別に取得
    #[must_use]
    pub fn derivative_step<T: Scalar>() -> T {
        select_scalar_value::<T>(DERIVATIVE_STEP_F32, DERIVATIVE_STEP_F64)
    }
}

/// 最大サポート次数
pub const MAX_DEGREE: usize = 10;

/// 最小制御点数（線形曲線の場合）
pub const MIN_CONTROL_POINTS: usize = 2;

/// NURBS最近接パラメータ探索で使う境界付きニュートン法の最大反復回数
pub const NEWTON_MAX_ITER: usize = 20;

/// 曲線長近似で使うデフォルト分割数
pub const CURVE_LENGTH_SUBDIVISIONS: usize = 100;

/// 曲面積近似で使うデフォルト分割数
pub const SURFACE_AREA_SUBDIVISIONS: usize = 20;

/// 弧長近似サンプリング数の最小値
pub const ARC_LENGTH_MIN_SAMPLES: usize = 10;

/// 弧長近似サンプリング数の最大値
pub const ARC_LENGTH_MAX_SAMPLES: usize = 1000;

/// 弧長->パラメータ逆算時の最大反復回数
pub const PARAMETER_AT_LENGTH_MAX_ITER: usize = 50;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solver_typed_constants_return_expected_values() {
        assert_eq!(
            solver::newton_tolerance::<f32>().to_bits(),
            solver::NEWTON_TOLERANCE_F32.to_bits()
        );
        assert_eq!(
            solver::newton_tolerance::<f64>().to_bits(),
            solver::NEWTON_TOLERANCE_F64.to_bits()
        );

        assert_eq!(
            solver::newton_diff_step::<f32>().to_bits(),
            solver::NEWTON_DIFF_STEP_F32.to_bits()
        );
        assert_eq!(
            solver::newton_diff_step::<f64>().to_bits(),
            solver::NEWTON_DIFF_STEP_F64.to_bits()
        );

        assert_eq!(
            solver::derivative_step::<f32>().to_bits(),
            solver::DERIVATIVE_STEP_F32.to_bits()
        );
        assert_eq!(
            solver::derivative_step::<f64>().to_bits(),
            solver::DERIVATIVE_STEP_F64.to_bits()
        );
    }
}
