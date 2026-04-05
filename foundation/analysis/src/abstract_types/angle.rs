//! Angle - 角度計算のユーティリティ

use crate::abstract_types::Scalar;
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
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
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
    /// 角度専用の許容誤差を取得
    /// crate::GeometricTolerance trait を使用し、角度として適切なスケールに調整
    pub fn tolerance() -> T
    where
        T: crate::GeometricTolerance,
    {
        T::ANGLE_TOLERANCE
    }

    /// 角度の等価性をチェック（周期性を考慮）
    pub fn is_equivalent(&self, other: &Self, tolerance: T) -> bool {
        let diff = (*self - *other).normalize_signed();
        diff.radians.abs() <= tolerance
    }

    /// デフォルト許容誤差での等価性チェック（GeometricTolerance実装が必要）
    pub fn is_equivalent_default(&self, other: &Self) -> bool
    where
        T: crate::GeometricTolerance,
    {
        self.is_equivalent(other, Self::tolerance())
    }

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

    /// 角度を署名付き正規化（-π <= angle < π）
    pub fn normalize_signed(self) -> Self {
        let normalized = self.normalize();
        if normalized.radians > T::PI {
            Self {
                radians: normalized.radians - T::TAU,
            }
        } else {
            normalized
        }
    }

    /// 0度
    pub fn zero() -> Self {
        Self { radians: T::ZERO }
    }

    /// 30度
    pub fn deg_30() -> Self {
        Self::from_radians(T::PI / T::from_f64(6.0))
    }

    /// 45度
    pub fn deg_45() -> Self {
        Self::from_radians(T::PI / T::from_f64(4.0))
    }

    /// 60度
    pub fn deg_60() -> Self {
        Self::from_radians(T::PI / T::from_f64(3.0))
    }

    /// 90度
    pub fn right_angle() -> Self {
        Self::from_radians(T::PI / T::from_f64(2.0))
    }

    /// 120度
    pub fn deg_120() -> Self {
        Self::from_radians(T::from_f64(2.0) * T::PI / T::from_f64(3.0))
    }

    /// 180度
    pub fn straight_angle() -> Self {
        Self::from_radians(T::PI)
    }

    /// 270度
    pub fn three_quarter_angle() -> Self {
        Self::from_radians(T::from_f64(3.0) * T::PI / T::from_f64(2.0))
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

/// 角度の複合代入（加算）
impl<T: Scalar> std::ops::AddAssign for Angle<T> {
    fn add_assign(&mut self, other: Self) {
        self.radians += other.radians;
    }
}

/// 角度の複合代入（減算）
impl<T: Scalar> std::ops::SubAssign for Angle<T> {
    fn sub_assign(&mut self, other: Self) {
        self.radians -= other.radians;
    }
}

/// 角度のスカラー倍複合代入
impl<T: Scalar> std::ops::MulAssign<T> for Angle<T> {
    fn mul_assign(&mut self, scalar: T) {
        self.radians *= scalar;
    }
}

/// 角度のスカラー除算複合代入
impl<T: Scalar> std::ops::DivAssign<T> for Angle<T> {
    fn div_assign(&mut self, scalar: T) {
        self.radians /= scalar;
    }
}

/// ラジアン値からの変換
impl<T: Scalar> From<T> for Angle<T> {
    fn from(radians: T) -> Self {
        Self::from_radians(radians)
    }
}
