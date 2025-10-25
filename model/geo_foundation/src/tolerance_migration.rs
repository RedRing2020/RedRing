//! 段階的移行のための許容誤差デフォルト値提供
//!
//! Scalar から許容誤差を分離するための過渡期的な仕組み

use crate::{Scalar, ToleranceSettings};

/// デフォルト許容誤差の提供
///
/// Scalar から許容誤差を分離する過渡期において、
/// 後方互換性を保ちながらデフォルト値を提供
pub struct DefaultTolerances;

impl DefaultTolerances {
    /// 距離許容誤差のデフォルト値（f64）
    pub const DISTANCE_F64: f64 = 1e-10;

    /// 角度許容誤差のデフォルト値（f64）
    pub const ANGLE_F64: f64 = 1e-8;

    /// 距離許容誤差のデフォルト値（f32）
    pub const DISTANCE_F32: f32 = 1e-6;

    /// 角度許容誤差のデフォルト値（f32）
    pub const ANGLE_F32: f32 = 1e-4;

    /// 型に応じた距離許容誤差を取得
    pub fn distance<T: Scalar>() -> T {
        if std::mem::size_of::<T>() == std::mem::size_of::<f64>() {
            T::from_f64(Self::DISTANCE_F64)
        } else {
            T::from_f64(Self::DISTANCE_F32 as f64)
        }
    }

    /// 型に応じた角度許容誤差を取得
    pub fn angle<T: Scalar>() -> T {
        if std::mem::size_of::<T>() == std::mem::size_of::<f64>() {
            T::from_f64(Self::ANGLE_F64)
        } else {
            T::from_f64(Self::ANGLE_F32 as f64)
        }
    }

    /// 型に応じた標準設定を取得
    pub fn standard<T: Scalar>() -> ToleranceSettings<T> {
        ToleranceSettings {
            distance_tolerance: Self::distance(),
            angle_tolerance: Self::angle(),
            area_tolerance: Self::distance(),
            length_tolerance: Self::distance(),
        }
    }
}

/// デフォルト許容誤差のアクセス用マクロ
///
/// 移行期間中の利便性のため
#[macro_export]
macro_rules! default_distance_tolerance {
    ($t:ty) => {
        $crate::tolerance::DefaultTolerances::distance::<$t>()
    };
}

#[macro_export]
macro_rules! default_angle_tolerance {
    ($t:ty) => {
        $crate::tolerance::DefaultTolerances::angle::<$t>()
    };
}

/// 移行ヘルパー：Scalarのメソッドを外部化
pub trait ScalarToleranceExt<T: Scalar> {
    /// 距離許容誤差を取得（移行用）
    fn default_distance_tolerance() -> T {
        DefaultTolerances::distance()
    }

    /// 角度許容誤差を取得（移行用）
    fn default_angle_tolerance() -> T {
        DefaultTolerances::angle()
    }
}

impl<T: Scalar> ScalarToleranceExt<T> for T {}
