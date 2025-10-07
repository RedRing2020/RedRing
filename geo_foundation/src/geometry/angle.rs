/// 型安全な角度管理のためのAngle構造体
///
/// 角度の型安全性を保証し、度数・ラジアン変換、正規化、
/// 四則演算をサポートする汎用角度型です。
///
/// # 設計方針
///
/// - **型安全性**: 度数とラジアンの混同を防止
/// - **汎用性**: f32/f64両対応でゲーム・CAD用途に最適化
/// - **正規化**: 角度の正規化（0-2π、-π〜π）を自動化
/// - **演算**: 直感的な四則演算と比較演算
use crate::abstract_types::Scalar;
use std::fmt::{Debug, Display};

/// 型安全な角度管理構造体
///
/// 内部的にはラジアンで管理し、度数・ラジアン両方での
/// 作成・アクセスをサポートします。
///
/// # 例
///
/// ```rust
/// use geo_foundation::geometry::Angle;
///
/// // 度数から作成
/// let angle1 = Angle::<f64>::from_degrees(90.0);
///
/// // ラジアンから作成
/// let angle2 = Angle::<f64>::from_radians(std::f64::consts::PI / 2.0);
///
/// // 角度演算
/// let sum = angle1 + angle2;  // 180度
/// let normalized = sum.normalize_0_2pi();  // 0-2πに正規化
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Angle<T: Scalar> {
    /// 内部表現（ラジアン）
    radians: T,
}

impl<T: Scalar> Angle<T> {
    /// ラジアンから角度を作成
    ///
    /// # Arguments
    ///
    /// * `radians` - ラジアン値
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geo_foundation::geometry::Angle;
    ///
    /// let angle = Angle::<f64>::from_radians(std::f64::consts::PI);
    /// assert_eq!(angle.to_degrees(), 180.0);
    /// ```
    pub fn from_radians(radians: T) -> Self {
        Self { radians }
    }

    /// 度数から角度を作成
    ///
    /// # Arguments
    ///
    /// * `degrees` - 度数値
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geo_foundation::geometry::Angle;
    ///
    /// let angle = Angle::<f32>::from_degrees(90.0);
    /// assert!((angle.to_radians() - std::f32::consts::PI / 2.0).abs() < 1e-6);
    /// ```
    pub fn from_degrees(degrees: T) -> Self {
        Self {
            radians: degrees * T::DEG_TO_RAD,
        }
    }

    /// ラジアン値を取得
    ///
    /// # Returns
    ///
    /// 角度のラジアン値
    pub fn to_radians(self) -> T {
        self.radians
    }

    /// 度数値を取得
    ///
    /// # Returns
    ///
    /// 角度の度数値
    pub fn to_degrees(self) -> T {
        self.radians * T::RAD_TO_DEG
    }

    /// ゼロ角度（0ラジアン）
    pub const fn zero() -> Self {
        Self { radians: T::ZERO }
    }

    /// 直角（π/2ラジアン、90度）
    pub fn right_angle() -> Self {
        Self {
            radians: T::PI / (T::ONE + T::ONE),
        }
    }

    /// 平角（πラジアン、180度）
    pub fn straight_angle() -> Self {
        Self { radians: T::PI }
    }

    /// 全角（2πラジアン、360度）
    pub fn full_angle() -> Self {
        Self { radians: T::TAU }
    }

    /// 角度を0-2πの範囲に正規化
    ///
    /// # Returns
    ///
    /// 0-2πの範囲に正規化された角度
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geo_foundation::geometry::Angle;
    ///
    /// let angle = Angle::<f64>::from_degrees(450.0);  // 450度
    /// let normalized = angle.normalize_0_2pi();
    /// assert!((normalized.to_degrees() - 90.0).abs() < 1e-10);
    /// ```
    pub fn normalize_0_2pi(self) -> Self {
        let mut result = self.radians;

        while result < T::ZERO {
            result = result + T::TAU;
        }
        while result >= T::TAU {
            result = result - T::TAU;
        }

        Self { radians: result }
    }

    /// 角度を-π〜πの範囲に正規化
    ///
    /// # Returns
    ///
    /// -π〜πの範囲に正規化された角度
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geo_foundation::geometry::Angle;
    ///
    /// let angle = Angle::<f64>::from_degrees(270.0);  // 270度
    /// let normalized = angle.normalize_pi_pi();
    /// assert!((normalized.to_degrees() + 90.0).abs() < 1e-10);  // -90度
    /// ```
    pub fn normalize_pi_pi(self) -> Self {
        let normalized_0_2pi = self.normalize_0_2pi();
        if normalized_0_2pi.radians > T::PI {
            Self {
                radians: normalized_0_2pi.radians - T::TAU,
            }
        } else {
            normalized_0_2pi
        }
    }

    /// 絶対値を取得
    ///
    /// # Returns
    ///
    /// 角度の絶対値
    pub fn abs(self) -> Self {
        Self {
            radians: self.radians.abs(),
        }
    }

    /// 正弦値を計算
    ///
    /// # Returns
    ///
    /// sin(角度)
    pub fn sin(self) -> T {
        self.radians.sin()
    }

    /// 余弦値を計算
    ///
    /// # Returns
    ///
    /// cos(角度)
    pub fn cos(self) -> T {
        self.radians.cos()
    }

    /// 正接値を計算
    ///
    /// # Returns
    ///
    /// tan(角度)
    pub fn tan(self) -> T {
        self.radians.tan()
    }

    /// 2つの角度間の差を計算（最短経路）
    ///
    /// # Arguments
    ///
    /// * `other` - 比較対象の角度
    ///
    /// # Returns
    ///
    /// 最短経路での角度差（-π〜π）
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geo_foundation::geometry::Angle;
    ///
    /// let angle1 = Angle::<f64>::from_degrees(10.0);
    /// let angle2 = Angle::<f64>::from_degrees(350.0);
    /// let diff = angle1.difference(angle2);
    /// assert!((diff.to_degrees() - 20.0).abs() < 1e-10);  // 最短経路は20度
    /// ```
    pub fn difference(self, other: Self) -> Self {
        let diff = self - other;
        diff.normalize_pi_pi()
    }

    /// 角度が近似的に等しいかを判定
    ///
    /// # Arguments
    ///
    /// * `other` - 比較対象の角度
    ///
    /// # Returns
    ///
    /// 許容誤差内で等しければtrue
    pub fn approx_eq(self, other: Self) -> bool {
        let diff = self.difference(other);
        diff.radians.abs() < T::TOLERANCE
    }
}

// 四則演算の実装
impl<T: Scalar> std::ops::Add for Angle<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            radians: self.radians + rhs.radians,
        }
    }
}

impl<T: Scalar> std::ops::Sub for Angle<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            radians: self.radians - rhs.radians,
        }
    }
}

impl<T: Scalar> std::ops::Mul<T> for Angle<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self {
            radians: self.radians * rhs,
        }
    }
}

impl<T: Scalar> std::ops::Div<T> for Angle<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self {
            radians: self.radians / rhs,
        }
    }
}

impl<T: Scalar> std::ops::Neg for Angle<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            radians: -self.radians,
        }
    }
}

// 代入演算の実装
impl<T: Scalar> std::ops::AddAssign for Angle<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.radians += rhs.radians;
    }
}

impl<T: Scalar> std::ops::SubAssign for Angle<T> {
    fn sub_assign(&mut self, rhs: Self) {
        self.radians -= rhs.radians;
    }
}

impl<T: Scalar> std::ops::MulAssign<T> for Angle<T> {
    fn mul_assign(&mut self, rhs: T) {
        self.radians *= rhs;
    }
}

impl<T: Scalar> std::ops::DivAssign<T> for Angle<T> {
    fn div_assign(&mut self, rhs: T) {
        self.radians /= rhs;
    }
}

// 比較演算の実装
impl<T: Scalar> PartialOrd for Angle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.radians.partial_cmp(&other.radians)
    }
}

// 表示実装
impl<T: Scalar> Display for Angle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}°", self.to_degrees())
    }
}

// 便利な型エイリアス
pub type Angle32 = Angle<f32>;
pub type Angle64 = Angle<f64>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_angle_creation() {
        let angle_rad = Angle::<f64>::from_radians(std::f64::consts::PI);
        let angle_deg = Angle::<f64>::from_degrees(180.0);

        assert!(angle_rad.approx_eq(angle_deg));
    }

    #[test]
    fn test_angle_conversion() {
        let angle = Angle::<f32>::from_degrees(90.0);
        assert!((angle.to_radians() - std::f32::consts::PI / 2.0).abs() < 1e-6);
        assert!((angle.to_degrees() - 90.0).abs() < 1e-6);
    }

    #[test]
    fn test_angle_normalization() {
        let angle = Angle::<f64>::from_degrees(450.0);
        let normalized = angle.normalize_0_2pi();
        assert!((normalized.to_degrees() - 90.0).abs() < 1e-10);

        let angle2 = Angle::<f64>::from_degrees(270.0);
        let normalized2 = angle2.normalize_pi_pi();
        assert!((normalized2.to_degrees() + 90.0).abs() < 1e-10);
    }

    #[test]
    fn test_angle_arithmetic() {
        let angle1 = Angle::<f64>::from_degrees(30.0);
        let angle2 = Angle::<f64>::from_degrees(60.0);

        let sum = angle1 + angle2;
        assert!((sum.to_degrees() - 90.0).abs() < 1e-10);

        let diff = angle2 - angle1;
        assert!((diff.to_degrees() - 30.0).abs() < 1e-10);

        let scaled = angle1 * 2.0;
        assert!((scaled.to_degrees() - 60.0).abs() < 1e-10);
    }

    #[test]
    fn test_angle_difference() {
        let angle1 = Angle::<f64>::from_degrees(10.0);
        let angle2 = Angle::<f64>::from_degrees(350.0);
        let diff = angle1.difference(angle2);
        assert!((diff.to_degrees() - 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_angle_constants() {
        let right = Angle::<f64>::right_angle();
        assert!((right.to_degrees() - 90.0).abs() < 1e-10);

        let straight = Angle::<f64>::straight_angle();
        assert!((straight.to_degrees() - 180.0).abs() < 1e-10);

        let full = Angle::<f64>::full_angle();
        assert!((full.to_degrees() - 360.0).abs() < 1e-10);
    }

    #[test]
    fn test_trigonometry() {
        let angle = Angle::<f64>::from_degrees(90.0);
        assert!((angle.sin() - 1.0).abs() < 1e-10);
        assert!(angle.cos().abs() < 1e-10);

        let angle45 = Angle::<f64>::from_degrees(45.0);
        let sqrt2_over_2 = (2.0f64).sqrt() / 2.0;
        assert!((angle45.sin() - sqrt2_over_2).abs() < 1e-10);
        assert!((angle45.cos() - sqrt2_over_2).abs() < 1e-10);
    }
}
