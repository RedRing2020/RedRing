/// 数値型の抽象化トレイト
/// 
/// f32（グラフィックス用）とf64（高精度計算用）を統一的に扱うための抽象化

use std::fmt::{Debug, Display};
use std::ops::{Add, Sub, Mul, Div, Neg};

/// 数値計算で使用可能なスカラー型の抽象化
pub trait Scalar: 
    Copy + Clone + Debug + Display + PartialEq + PartialOrd +
    Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self> + Div<Output = Self> + Neg<Output = Self>
{
    /// ゼロ値
    const ZERO: Self;
    /// 単位値
    const ONE: Self;
    /// 機械イプシロン（数値精度の限界）
    const EPSILON: Self;
    /// 円周率
    const PI: Self;
    /// 2π
    const TAU: Self;
    /// e (自然対数の底)
    const E: Self;

    /// 絶対値
    fn abs(self) -> Self;
    /// 平方根
    fn sqrt(self) -> Self;
    /// 正弦
    fn sin(self) -> Self;
    /// 余弦
    fn cos(self) -> Self;
    /// 正接
    fn tan(self) -> Self;
    /// 逆正弦
    fn asin(self) -> Self;
    /// 逆余弦
    fn acos(self) -> Self;
    /// 逆正接
    fn atan(self) -> Self;
    /// 2引数逆正接
    fn atan2(self, other: Self) -> Self;
    /// 指数関数
    fn exp(self) -> Self;
    /// 自然対数
    fn ln(self) -> Self;
    /// べき乗
    fn powf(self, exp: Self) -> Self;
    /// 床関数
    fn floor(self) -> Self;
    /// 天井関数
    fn ceil(self) -> Self;
    /// 四捨五入
    fn round(self) -> Self;
    /// 最小値
    fn min(self, other: Self) -> Self;
    /// 最大値
    fn max(self, other: Self) -> Self;
    /// 範囲内にクランプ
    fn clamp(self, min: Self, max: Self) -> Self;

    /// f64に変換
    fn to_f64(self) -> f64;
    /// f32に変換
    fn to_f32(self) -> f32;
    /// f64から変換
    fn from_f64(value: f64) -> Self;
    /// f32から変換
    fn from_f32(value: f32) -> Self;

    /// 許容誤差内での等価判定
    fn approx_eq(self, other: Self, epsilon: Self) -> bool {
        (self - other).abs() <= epsilon
    }

    /// ゼロに近いかの判定
    fn is_zero(self) -> bool {
        self.abs() <= Self::EPSILON
    }

    /// 有限値かの判定
    fn is_finite(self) -> bool;
}

/// f64に対するScalarトレイトの実装
impl Scalar for f64 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const EPSILON: Self = f64::EPSILON;
    const PI: Self = std::f64::consts::PI;
    const TAU: Self = std::f64::consts::TAU;
    const E: Self = std::f64::consts::E;

    fn abs(self) -> Self { self.abs() }
    fn sqrt(self) -> Self { self.sqrt() }
    fn sin(self) -> Self { self.sin() }
    fn cos(self) -> Self { self.cos() }
    fn tan(self) -> Self { self.tan() }
    fn asin(self) -> Self { self.asin() }
    fn acos(self) -> Self { self.acos() }
    fn atan(self) -> Self { self.atan() }
    fn atan2(self, other: Self) -> Self { self.atan2(other) }
    fn exp(self) -> Self { self.exp() }
    fn ln(self) -> Self { self.ln() }
    fn powf(self, exp: Self) -> Self { self.powf(exp) }
    fn floor(self) -> Self { self.floor() }
    fn ceil(self) -> Self { self.ceil() }
    fn round(self) -> Self { self.round() }
    fn min(self, other: Self) -> Self { self.min(other) }
    fn max(self, other: Self) -> Self { self.max(other) }
    fn clamp(self, min: Self, max: Self) -> Self { self.clamp(min, max) }

    fn to_f64(self) -> f64 { self }
    fn to_f32(self) -> f32 { self as f32 }
    fn from_f64(value: f64) -> Self { value }
    fn from_f32(value: f32) -> Self { value as f64 }

    fn is_finite(self) -> bool { self.is_finite() }
}

/// f32に対するScalarトレイトの実装
impl Scalar for f32 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const EPSILON: Self = f32::EPSILON;
    const PI: Self = std::f32::consts::PI;
    const TAU: Self = std::f32::consts::TAU;
    const E: Self = std::f32::consts::E;

    fn abs(self) -> Self { self.abs() }
    fn sqrt(self) -> Self { self.sqrt() }
    fn sin(self) -> Self { self.sin() }
    fn cos(self) -> Self { self.cos() }
    fn tan(self) -> Self { self.tan() }
    fn asin(self) -> Self { self.asin() }
    fn acos(self) -> Self { self.acos() }
    fn atan(self) -> Self { self.atan() }
    fn atan2(self, other: Self) -> Self { self.atan2(other) }
    fn exp(self) -> Self { self.exp() }
    fn ln(self) -> Self { self.ln() }
    fn powf(self, exp: Self) -> Self { self.powf(exp) }
    fn floor(self) -> Self { self.floor() }
    fn ceil(self) -> Self { self.ceil() }
    fn round(self) -> Self { self.round() }
    fn min(self, other: Self) -> Self { self.min(other) }
    fn max(self, other: Self) -> Self { self.max(other) }
    fn clamp(self, min: Self, max: Self) -> Self { self.clamp(min, max) }

    fn to_f64(self) -> f64 { self as f64 }
    fn to_f32(self) -> f32 { self }
    fn from_f64(value: f64) -> Self { value as f32 }
    fn from_f32(value: f32) -> Self { value }

    fn is_finite(self) -> bool { self.is_finite() }
}

/// 型エイリアス（使いやすさのため）
pub type F32 = f32;
pub type F64 = f64;

// テストは unit_tests/scalar_tests.rs に移動