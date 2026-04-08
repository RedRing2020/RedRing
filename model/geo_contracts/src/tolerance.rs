//! 幾何計算のための許容誤差管理
//!
//! geo_contracts で共有するトレランス設定。

use std::any::TypeId;

use analysis::consts::numerical::{
    KERNEL_NUMERICAL_ZERO_THRESHOLD_F32, KERNEL_NUMERICAL_ZERO_THRESHOLD_F64,
};

use crate::Scalar;

#[inline]
fn is_f32_scalar<T: Scalar>() -> bool {
    TypeId::of::<T>() == TypeId::of::<f32>()
}

#[inline]
fn precision_distance_tolerance<T: Scalar>() -> T {
    if is_f32_scalar::<T>() {
        T::DISTANCE_TOLERANCE
    } else {
        T::from_f64(1e-12)
    }
}

#[inline]
fn precision_angle_tolerance<T: Scalar>() -> T {
    if is_f32_scalar::<T>() {
        T::ANGLE_TOLERANCE
    } else {
        T::from_f64(1e-10)
    }
}

#[inline]
fn precision_length_tolerance<T: Scalar>() -> T {
    if is_f32_scalar::<T>() {
        T::DISTANCE_TOLERANCE
    } else {
        T::from_f64(1e-12)
    }
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
    /// 高精度設定（CAD/精密加工用）
    pub fn precision() -> Self {
        let distance_tolerance = precision_distance_tolerance();
        let angle_tolerance = precision_angle_tolerance();
        let length_tolerance = precision_length_tolerance();
        Self {
            distance_tolerance,
            angle_tolerance,
            area_tolerance: area_tolerance_from_length(length_tolerance),
            length_tolerance,
        }
    }

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

    /// カスタム設定
    pub fn custom(distance: T, angle: T, area: T, length: T) -> Self {
        Self {
            distance_tolerance: distance,
            angle_tolerance: angle,
            area_tolerance: area,
            length_tolerance: length,
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

/// 幾何計算コンテキスト
#[derive(Debug, Clone, Copy)]
pub struct GeometryContext<T: Scalar> {
    pub tolerances: ToleranceSettings<T>,
}

impl<T: Scalar> GeometryContext<T> {
    /// 新しいコンテキストを作成
    pub fn new(tolerances: ToleranceSettings<T>) -> Self {
        Self { tolerances }
    }

    /// 標準コンテキスト
    pub fn standard() -> Self {
        Self::new(ToleranceSettings::standard())
    }

    /// 高精度コンテキスト
    pub fn precision() -> Self {
        Self::new(ToleranceSettings::precision())
    }

    /// 緩いコンテキスト
    pub fn relaxed() -> Self {
        Self::new(ToleranceSettings::relaxed())
    }
}

impl<T: Scalar> Default for GeometryContext<T> {
    fn default() -> Self {
        Self::standard()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precision_profile_is_type_aware() {
        let f32_profile = ToleranceSettings::<f32>::precision();
        assert_eq!(f32_profile.distance_tolerance, 1e-6_f32);
        assert_eq!(f32_profile.angle_tolerance, 1e-6_f32);
        assert_eq!(f32_profile.area_tolerance, 1e-12_f32);
        assert_eq!(f32_profile.length_tolerance, 1e-6_f32);

        let f64_profile = ToleranceSettings::<f64>::precision();
        assert_eq!(f64_profile.distance_tolerance, 1e-12_f64);
        assert_eq!(f64_profile.angle_tolerance, 1e-10_f64);
        assert_eq!(f64_profile.area_tolerance, 1e-24_f64);
        assert_eq!(f64_profile.length_tolerance, 1e-12_f64);
    }

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
