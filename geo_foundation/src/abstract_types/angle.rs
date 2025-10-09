//! Angle - 角度計算のユーティリティ
//!
//! CAD/CAM システムで使用される角度計算機能

use crate::abstract_types::Scalar;
use crate::constants::precision::PI;
use std::fmt;

/// 角度型の共通インターフェース
pub trait AngleType: Copy + Clone {
    type Scalar: Scalar;

    /// ラジアン値から角度を作成
    fn from_radians(radians: Self::Scalar) -> Self;

    /// 角度をラジアン値として取得
    fn to_radians(self) -> Self::Scalar;
}

/// 型安全な角度を表現する構造体
///
/// ラジアン値を内部で保持し、型安全な角度計算を提供します。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Angle<T: Scalar> {
    radians: T,
}

impl<T: Scalar> AngleType for Angle<T> {
    type Scalar = T;

    fn from_radians(radians: T) -> Self {
        Self { radians }
    }

    fn to_radians(self) -> T {
        self.radians
    }
}

impl<T: Scalar> Angle<T> {
    /// ラジアン値から角度を作成
    pub fn from_radians(radians: T) -> Self {
        Self { radians }
    }

    /// 度数から角度を作成
    pub fn from_degrees(degrees: T) -> Self {
        let radians = degrees * T::DEG_TO_RAD;
        Self { radians }
    }

    /// 角度をラジアン値として取得
    pub fn to_radians(self) -> T {
        self.radians
    }

    /// 角度を度数として取得
    pub fn to_degrees(self) -> T {
        self.radians * T::RAD_TO_DEG
    }

    /// 正弦値を計算
    pub fn sin(self) -> T {
        self.radians.sin()
    }

    /// 余弦値を計算
    pub fn cos(self) -> T {
        self.radians.cos()
    }

    /// 正接値を計算
    pub fn tan(self) -> T {
        self.radians.tan()
    }

    /// 角度を正規化（0 <= angle < 2π）
    pub fn normalize(self) -> Self {
        let tau = T::TAU;
        let mut rad = self.radians;
        while rad < T::ZERO {
            rad += tau;
        }
        while rad >= tau {
            rad -= tau;
        }
        Self { radians: rad }
    }

    /// 0度
    pub fn zero() -> Self {
        Self { radians: T::ZERO }
    }

    /// 90度
    pub fn right_angle() -> Self {
        Self::from_radians(T::PI / (T::ONE + T::ONE))
    }

    /// 180度
    pub fn straight_angle() -> Self {
        Self::from_radians(T::PI)
    }

    /// 270度
    pub fn three_quarter_angle() -> Self {
        Self::from_radians((T::ONE + T::ONE + T::ONE) * T::PI / (T::ONE + T::ONE))
    }

    /// 360度
    pub fn full_angle() -> Self {
        Self::from_radians(T::TAU)
    }
}

impl<T: Scalar> fmt::Display for Angle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}°", self.to_degrees())
    }
}

/// 角度の加算
impl<T: Scalar> std::ops::Add for Angle<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            radians: self.radians + other.radians,
        }
    }
}

/// 角度の減算
impl<T: Scalar> std::ops::Sub for Angle<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            radians: self.radians - other.radians,
        }
    }
}

/// 角度のスカラー倍
impl<T: Scalar> std::ops::Mul<T> for Angle<T> {
    type Output = Self;

    fn mul(self, scalar: T) -> Self::Output {
        Self {
            radians: self.radians * scalar,
        }
    }
}

/// 角度のスカラー除算
impl<T: Scalar> std::ops::Div<T> for Angle<T> {
    type Output = Self;

    fn div(self, scalar: T) -> Self::Output {
        Self {
            radians: self.radians / scalar,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_angle_creation() {
        let angle_rad = Angle::from_radians(1.0f64);
        let angle_deg = Angle::from_degrees(180.0f64);

        assert!((angle_rad.to_radians() - 1.0).abs() < 1e-10);
        assert!((angle_deg.to_radians() - PI).abs() < 1e-10);
    }

    #[test]
    fn test_angle_conversion() {
        let angle = Angle::from_degrees(90.0f64);
        assert!((angle.to_radians() - PI / 2.0).abs() < 1e-10);
        assert!((angle.to_degrees() - 90.0).abs() < 1e-10);
    }

    #[test]
    fn test_angle_arithmetic() {
        let a1 = Angle::from_degrees(45.0f64);
        let a2 = Angle::from_degrees(30.0f64);

        let sum = a1 + a2;
        assert!((sum.to_degrees() - 75.0).abs() < 1e-10);

        let diff = a1 - a2;
        assert!((diff.to_degrees() - 15.0).abs() < 1e-10);
    }

    #[test]
    fn test_trigonometric_functions() {
        let angle = Angle::from_degrees(90.0f64);
        assert!((angle.sin() - 1.0).abs() < 1e-10);
        assert!(angle.cos().abs() < 1e-10);

        let angle_45 = Angle::from_degrees(45.0f64);
        let sqrt2_over_2 = std::f64::consts::SQRT_2 / 2.0;
        assert!((angle_45.sin() - sqrt2_over_2).abs() < 1e-10);
        assert!((angle_45.cos() - sqrt2_over_2).abs() < 1e-10);
    }

    #[test]
    fn test_angle_normalization() {
        let angle1 = Angle::from_degrees(450.0f64); // 450° = 90°
        let normalized = angle1.normalize();
        assert!((normalized.to_degrees() - 90.0).abs() < 1e-10);

        let angle2 = Angle::from_degrees(-90.0f64); // -90° = 270°
        let normalized2 = angle2.normalize();
        assert!((normalized2.to_degrees() - 270.0).abs() < 1e-10);
    }
}
