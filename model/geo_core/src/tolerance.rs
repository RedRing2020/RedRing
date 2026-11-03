//! 幾何計算のための許容誤差管理
//!
//! 幾何のカーネルで共有する許容誤差の既定値と設定。

use std::any::TypeId;

use analysis::consts::numerical::{
    KERNEL_NUMERICAL_ZERO_THRESHOLD_F32, KERNEL_NUMERICAL_ZERO_THRESHOLD_F64,
};

use analysis::abstract_types::Scalar;

#[inline]
fn is_f32_scalar<T: Scalar>() -> bool {
    TypeId::of::<T>() == TypeId::of::<f32>()
}

#[inline]
fn area_tolerance_from_length<T: Scalar>(length_tolerance: T) -> T {
    length_tolerance * length_tolerance
}

/// アプリケーション固有の許容誤差設定
#[derive(Debug, Clone, Copy)]
pub struct ToleranceSettings<T: Scalar> {
    /// 距離計算用の許容誤差（点の包含判定、距離比較など）
    pub distance_tolerance: T,

    /// 角度計算用の許容誤差（平行・垂直判定など）
    pub angle_tolerance: T,

    /// 面積計算用の許容誤差（長さトレランスの二乗系）
    pub area_tolerance: T,

    /// 長さ計算用の許容誤差
    pub length_tolerance: T,
}

impl<T: Scalar> ToleranceSettings<T> {
    /// 標準設定（一般的な工学計算用）
    pub fn standard() -> Self {
        let distance_tolerance = T::from_f64(1e-6);
        let angle_tolerance = T::from_f64(1e-4);
        let length_tolerance = T::from_f64(1e-6);
        Self {
            distance_tolerance,
            angle_tolerance,
            area_tolerance: area_tolerance_from_length(length_tolerance),
            length_tolerance,
        }
    }

    /// 緩い設定（ゲーム・リアルタイム用）
    pub fn relaxed() -> Self {
        let distance_tolerance = T::from_f64(1e-3);
        let angle_tolerance = T::from_f64(1e-2);
        let length_tolerance = T::from_f64(1e-3);
        Self {
            distance_tolerance,
            angle_tolerance,
            area_tolerance: area_tolerance_from_length(length_tolerance),
            length_tolerance,
        }
    }
}

impl<T: Scalar> Default for ToleranceSettings<T> {
    fn default() -> Self {
        Self::standard()
    }
}

/// `standard` プロファイルの距離トレランスを返す。
pub fn default_distance_tolerance<T: Scalar>() -> T {
    ToleranceSettings::<T>::standard().distance_tolerance
}

/// `standard` プロファイルの角度トレランスを返す。
pub fn default_angle_tolerance<T: Scalar>() -> T {
    ToleranceSettings::<T>::standard().angle_tolerance
}

/// 型に応じた平行判定（外積）誤差閾値を返す。
pub fn default_parallel_cross_error_tolerance<T: Scalar>() -> T {
    T::PARALLEL_CROSS_ERROR_TOLERANCE
}

/// 型に応じた直交判定（内積）誤差閘値を返す。
pub fn default_orthogonality_dot_error_tolerance<T: Scalar>() -> T {
    T::ORTHOGONALITY_DOT_ERROR_TOLERANCE
}

/// カーネル内部の数値安定化ガード（ゼロ判定）に使う固定閾値を返す。
///
/// `ToleranceSettings` とは独立しており、アプリケーション設定で変更しない。
pub fn default_kernel_numerical_zero_tolerance<T: Scalar>() -> T {
    if is_f32_scalar::<T>() {
        T::from_f32(KERNEL_NUMERICAL_ZERO_THRESHOLD_F32)
    } else {
        T::from_f64(KERNEL_NUMERICAL_ZERO_THRESHOLD_F64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_and_relaxed_area_tolerance_follow_length_squared() {
        let standard = ToleranceSettings::<f64>::standard();
        assert_eq!(
            standard.area_tolerance,
            standard.length_tolerance * standard.length_tolerance
        );

        let relaxed = ToleranceSettings::<f32>::relaxed();
        assert_eq!(
            relaxed.area_tolerance,
            relaxed.length_tolerance * relaxed.length_tolerance
        );
    }

    #[test]
    fn test_default_kernel_numerical_zero_tolerance_is_type_aware() {
        assert_eq!(default_kernel_numerical_zero_tolerance::<f32>(), 1e-6_f32);
        assert_eq!(default_kernel_numerical_zero_tolerance::<f64>(), 1e-12_f64);
    }
}
