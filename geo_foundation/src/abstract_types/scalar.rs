//! Scalar - 数値の抽象化型
//!
//! 幾何計算において、f32/f64の両方をサポートし、
//! 数値型の統一インターフェースを提供

use std::fmt::{Debug, Display};

/// 幾何計算用の汎用数値型トレイト
///
/// 角度、座標、距離など幾何要素で使用される数値型の
/// 共通操作を定義します。
///
/// # 用途
///
/// - **f32**: ゲーム開発、リアルタイム描画、GPUコンピューティング
/// - **f64**: CAD/CAM、科学技術計算、高精度数値解析
pub trait Scalar:
    Copy
    + Clone
    + Debug
    + Display
    + PartialEq
    + PartialOrd
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::Mul<Output = Self>
    + std::ops::Div<Output = Self>
    + std::ops::Neg<Output = Self>
    + std::ops::AddAssign
    + std::ops::SubAssign
    + std::ops::MulAssign
    + std::ops::DivAssign
    + 'static
{
    /// ゼロ値
    const ZERO: Self;

    /// 単位値（1）
    const ONE: Self;

    /// 数学定数π
    const PI: Self;

    /// 数学定数τ（2π）
    const TAU: Self;

    /// 数学定数e
    const E: Self;

    /// 幾何計算用の許容誤差
    const TOLERANCE: Self;

    /// 角度変換定数（度からラジアンへ）
    const DEG_TO_RAD: Self;

    /// 角度変換定数（ラジアンから度へ）
    const RAD_TO_DEG: Self;

    /// 絶対値を取得
    fn abs(self) -> Self;

    /// 平方根を計算
    fn sqrt(self) -> Self;

    /// 正弦値を計算
    fn sin(self) -> Self;

    /// 余弦値を計算
    fn cos(self) -> Self;

    /// 正接値を計算
    fn tan(self) -> Self;

    /// 逆正弦値を計算
    fn asin(self) -> Self;

    /// 逆余弦値を計算
    fn acos(self) -> Self;

    /// 逆正接値を計算
    fn atan(self) -> Self;

    /// 2つの値の逆正接値を計算
    fn atan2(self, other: Self) -> Self;

    /// 床関数（最大整数部）
    fn floor(self) -> Self;

    /// 天井関数（最小整数部）
    fn ceil(self) -> Self;

    /// 四捨五入
    fn round(self) -> Self;

    /// 最小値を取得
    fn min(self, other: Self) -> Self;

    /// 最大値を取得
    fn max(self, other: Self) -> Self;

    /// べき乗
    fn powi(self, exp: i32) -> Self;

    /// 値を指定範囲にクランプ
    fn clamp(self, min: Self, max: Self) -> Self;

    /// ほぼ等しいかを許容誤差で判定
    fn approx_eq(self, other: Self) -> bool {
        (self - other).abs() < Self::TOLERANCE
    }

    /// f64に変換
    fn to_f64(self) -> f64;

    /// f64から変換
    fn from_f64(value: f64) -> Self;
}

/// f32用のScalar実装
impl Scalar for f32 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const PI: Self = std::f32::consts::PI;
    const TAU: Self = std::f32::consts::TAU;
    const E: Self = std::f32::consts::E;
    const TOLERANCE: Self = 1e-6;
    const DEG_TO_RAD: Self = Self::PI / 180.0;
    const RAD_TO_DEG: Self = 180.0 / Self::PI;

    #[inline]
    fn abs(self) -> Self {
        self.abs()
    }

    #[inline]
    fn sqrt(self) -> Self {
        self.sqrt()
    }

    #[inline]
    fn sin(self) -> Self {
        self.sin()
    }

    #[inline]
    fn cos(self) -> Self {
        self.cos()
    }

    #[inline]
    fn tan(self) -> Self {
        self.tan()
    }

    #[inline]
    fn asin(self) -> Self {
        self.asin()
    }

    #[inline]
    fn acos(self) -> Self {
        self.acos()
    }

    #[inline]
    fn atan(self) -> Self {
        self.atan()
    }

    #[inline]
    fn atan2(self, other: Self) -> Self {
        self.atan2(other)
    }

    #[inline]
    fn floor(self) -> Self {
        self.floor()
    }

    #[inline]
    fn ceil(self) -> Self {
        self.ceil()
    }

    #[inline]
    fn round(self) -> Self {
        self.round()
    }

    #[inline]
    fn min(self, other: Self) -> Self {
        self.min(other)
    }

    #[inline]
    fn max(self, other: Self) -> Self {
        self.max(other)
    }

    #[inline]
    fn clamp(self, min: Self, max: Self) -> Self {
        self.clamp(min, max)
    }

    #[inline]
    fn to_f64(self) -> f64 {
        self as f64
    }

    #[inline]
    fn from_f64(value: f64) -> Self {
        value as f32
    }

    #[inline]
    fn powi(self, exp: i32) -> Self {
        self.powi(exp)
    }
}

/// f64用のScalar実装
impl Scalar for f64 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const PI: Self = std::f64::consts::PI;
    const TAU: Self = std::f64::consts::TAU;
    const E: Self = std::f64::consts::E;
    const TOLERANCE: Self = 1e-10;
    const DEG_TO_RAD: Self = Self::PI / 180.0;
    const RAD_TO_DEG: Self = 180.0 / Self::PI;

    #[inline]
    fn abs(self) -> Self {
        self.abs()
    }

    #[inline]
    fn sqrt(self) -> Self {
        self.sqrt()
    }

    #[inline]
    fn sin(self) -> Self {
        self.sin()
    }

    #[inline]
    fn cos(self) -> Self {
        self.cos()
    }

    #[inline]
    fn tan(self) -> Self {
        self.tan()
    }

    #[inline]
    fn asin(self) -> Self {
        self.asin()
    }

    #[inline]
    fn acos(self) -> Self {
        self.acos()
    }

    #[inline]
    fn atan(self) -> Self {
        self.atan()
    }

    #[inline]
    fn atan2(self, other: Self) -> Self {
        self.atan2(other)
    }

    #[inline]
    fn floor(self) -> Self {
        self.floor()
    }

    #[inline]
    fn ceil(self) -> Self {
        self.ceil()
    }

    #[inline]
    fn round(self) -> Self {
        self.round()
    }

    #[inline]
    fn min(self, other: Self) -> Self {
        self.min(other)
    }

    #[inline]
    fn max(self, other: Self) -> Self {
        self.max(other)
    }

    #[inline]
    fn clamp(self, min: Self, max: Self) -> Self {
        self.clamp(min, max)
    }

    #[inline]
    fn to_f64(self) -> f64 {
        self
    }

    #[inline]
    fn from_f64(value: f64) -> Self {
        value
    }

    #[inline]
    fn powi(self, exp: i32) -> Self {
        self.powi(exp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f32_scalar_constants() {
        assert_eq!(f32::ZERO, 0.0f32);
        assert_eq!(f32::ONE, 1.0f32);
        assert_eq!(f32::PI, std::f32::consts::PI);
        assert_eq!(f32::TAU, std::f32::consts::TAU);
        assert_eq!(f32::E, std::f32::consts::E);
    }

    #[test]
    fn test_f64_scalar_constants() {
        assert_eq!(f64::ZERO, 0.0f64);
        assert_eq!(f64::ONE, 1.0f64);
        assert_eq!(f64::PI, std::f64::consts::PI);
        assert_eq!(f64::TAU, std::f64::consts::TAU);
        assert_eq!(f64::E, std::f64::consts::E);
    }

    #[test]
    fn test_scalar_operations() {
        let a = 3.0f32;
        let b = 4.0f32;

        assert_eq!((a * a + b * b).sqrt(), 5.0f32);
        assert!((f32::PI / 2.0).sin().approx_eq(1.0));
        assert!((f32::PI).cos().approx_eq(-1.0));
    }

    #[test]
    fn test_angle_conversion() {
        let degrees = 90.0f64;
        let radians = degrees * f64::DEG_TO_RAD;
        assert!(radians.approx_eq(f64::PI / 2.0));

        let back_to_degrees = radians * f64::RAD_TO_DEG;
        assert!(back_to_degrees.approx_eq(90.0));
    }

    #[test]
    fn test_approx_eq() {
        let a = 1.0f32;
        let b = 1.0f32 + f32::TOLERANCE / 10.0;
        assert!(a.approx_eq(b));

        let c = 1.0f32 + f32::TOLERANCE * 10.0;
        assert!(!a.approx_eq(c));
    }

    #[test]
    fn test_type_conversion() {
        let f32_val = std::f32::consts::PI;
        let f64_val = f32_val.to_f64();
        let back_to_f32 = f32::from_f64(f64_val);

        assert!((f32_val - back_to_f32).abs() < f32::TOLERANCE);
    }
}
