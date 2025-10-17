//! 特殊数学定数管理（黄金比など）
//!
//! ## 使い分け
//!
//! - **基本数学定数 (π, e, τ)**: `T::PI`, `T::E`, `T::TAU` を使用
//! - **よく使う定数 (√2, 1/√2)**: `T::SQRT_2`, `T::INV_SQRT_2` を使用
//! - **固定型定数**: `analysis::consts::game::PI` (f32), `analysis::consts::precision::PI` (f64)
//! - **特殊数学定数**: このモジュールの `MathConstants` を使用

use crate::abstract_types::Scalar;

/// 特殊数学定数を提供
///
/// 基本定数 (π, e, τ, √2) は `Scalar` トレイトを使用してください
pub struct MathConstants;

impl MathConstants {
    /// 黄金比 φ = (1 + √5) / 2
    pub fn golden_ratio<T: Scalar>() -> T {
        (T::ONE + T::from_f64(5.0).sqrt()) / T::from_f64(2.0)
    }

    /// 自然対数の底 ln(2)
    pub fn ln_2<T: Scalar>() -> T {
        T::from_f64(std::f64::consts::LN_2)
    }

    /// 自然対数の底 ln(10)
    pub fn ln_10<T: Scalar>() -> T {
        T::from_f64(std::f64::consts::LN_10)
    }

    /// 平方根 √3
    pub fn sqrt_3<T: Scalar>() -> T {
        T::from_f64(3.0).sqrt()
    }
}

/// 許容誤差定数の統一管理
///
/// 型に応じて最適な許容誤差を提供します。
/// `analysis::consts` の固定型定数との統合ポイントです。
pub struct ToleranceConstants;

impl ToleranceConstants {
    /// 一般的な幾何学計算用許容誤差
    ///
    /// - f32相当: 1e-6 (ゲーム・リアルタイム用)
    /// - f64相当: 1e-10 (CAD・高精度用)
    pub fn geometric<T: Scalar>() -> T {
        if std::mem::size_of::<T>() == 4 {
            T::from_f64(1e-6) // f32 equivalent
        } else {
            T::from_f64(1e-10) // f64 equivalent
        }
    }

    /// 角度計算用許容誤差
    pub fn angular<T: Scalar>() -> T {
        if std::mem::size_of::<T>() == 4 {
            T::from_f64(1e-6) // f32 equivalent
        } else {
            T::from_f64(1e-8) // f64 equivalent
        }
    }

    /// 長さ計算用許容誤差
    pub fn distance<T: Scalar>() -> T {
        if std::mem::size_of::<T>() == 4 {
            T::from_f64(1e-6) // f32 equivalent
        } else {
            T::from_f64(1e-12) // f64 equivalent
        }
    }

    /// 面積計算用許容誤差
    pub fn area<T: Scalar>() -> T {
        if std::mem::size_of::<T>() == 4 {
            T::from_f64(1e-5) // f32 equivalent (area = length²)
        } else {
            T::from_f64(1e-10) // f64 equivalent
        }
    }
}
