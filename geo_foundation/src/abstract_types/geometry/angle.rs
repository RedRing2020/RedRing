//! Angle (角度) 構造体とユーティリティ
//!
//! 型安全な角度表現と角度計算のためのモジュール

use std::fmt;
use std::ops::{Add, Sub, Mul, Div, Neg};

/// 数値型制約トレイト
pub trait Scalar: 
    Copy + Clone + PartialEq + PartialOrd + 
    Add<Output = Self> + Sub<Output = Self> + 
    Mul<Output = Self> + Div<Output = Self> +
    Neg<Output = Self> +
    fmt::Debug + fmt::Display +
    'static
{
    /// π定数
    fn pi() -> Self;
    /// 2π定数 
    fn tau() -> Self;
    /// ゼロ値
    fn zero() -> Self;
    /// 1値
    fn one() -> Self;
    /// 2値
    fn two() -> Self;
    /// 180値（度数変換用）
    fn one_hundred_eighty() -> Self;
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
    /// 逆正接（atan2）
    fn atan2(self, x: Self) -> Self;
    /// 有限値かチェック
    fn is_finite(self) -> bool;
    /// f64からの変換
    fn from_f64(value: f64) -> Self;
}

impl Scalar for f64 {
    fn pi() -> Self { std::f64::consts::PI }
    fn tau() -> Self { std::f64::consts::TAU }
    fn zero() -> Self { 0.0 }
    fn one() -> Self { 1.0 }
    fn two() -> Self { 2.0 }
    fn one_hundred_eighty() -> Self { 180.0 }
    fn abs(self) -> Self { self.abs() }
    fn sqrt(self) -> Self { self.sqrt() }
    fn sin(self) -> Self { self.sin() }
    fn cos(self) -> Self { self.cos() }
    fn tan(self) -> Self { self.tan() }
    fn asin(self) -> Self { self.asin() }
    fn acos(self) -> Self { self.acos() }
    fn atan2(self, x: Self) -> Self { self.atan2(x) }
    fn is_finite(self) -> Self { self.is_finite() }
    fn from_f64(value: f64) -> Self { value }
}

impl Scalar for f32 {
    fn pi() -> Self { std::f32::consts::PI }
    fn tau() -> Self { std::f32::consts::TAU }
    fn zero() -> Self { 0.0 }
    fn one() -> Self { 1.0 }
    fn two() -> Self { 2.0 }
    fn one_hundred_eighty() -> Self { 180.0 }
    fn abs(self) -> Self { self.abs() }
    fn sqrt(self) -> Self { self.sqrt() }
    fn sin(self) -> Self { self.sin() }
    fn cos(self) -> Self { self.cos() }
    fn tan(self) -> Self { self.tan() }
    fn asin(self) -> Self { self.asin() }
    fn acos(self) -> Self { self.acos() }
    fn atan2(self, x: Self) -> Self { self.atan2(x) }
    fn is_finite(self) -> Self { self.is_finite() }
    fn from_f64(value: f64) -> Self { value as f32 }
}

/// 型安全な角度を表現する構造体
/// 
/// # 特徴
/// - ラジアン/度数の混在エラーを防止
/// - 角度の正規化と演算をサポート
/// - f32/f64両対応のジェネリック設計
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Angle<T: Scalar> {
    radians: T,
}

impl<T: Scalar> Angle<T> {
    /// ラジアンから角度を作成
    /// 
    /// # Arguments
    /// * `radians` - ラジアン値
    /// 
    /// # Examples
    /// ```
    /// use geo_foundation::Angle;
    /// let angle = Angle::from_radians(std::f64::consts::PI);
    /// assert_eq!(angle.degrees(), 180.0);
    /// ```
    pub fn from_radians(radians: T) -> Self {
        Self { radians }
    }

    /// 度数から角度を作成
    /// 
    /// # Arguments
    /// * `degrees` - 度数値
    pub fn from_degrees(degrees: T) -> Self {
        let deg_to_rad = T::pi() / T::one_hundred_eighty();
        Self { radians: degrees * deg_to_rad }
    }

    /// ラジアン値を取得
    pub fn radians(&self) -> T {
        self.radians
    }

    /// 度数値を取得
    pub fn degrees(&self) -> T {
        let rad_to_deg = T::one_hundred_eighty() / T::pi();
        self.radians * rad_to_deg
    }

    /// 角度を正規化（0-2π範囲）
    /// 
    /// # Returns
    /// 0 ≤ θ < 2π の範囲に正規化された角度
    pub fn normalize(&self) -> Self {
        let tau = T::tau();
        let mut normalized = self.radians;
        
        // 負の角度を正の角度に変換
        while normalized < T::zero() {
            normalized = normalized + tau;
        }
        
        // 2π以上の角度を0-2π範囲に変換
        while normalized >= tau {
            normalized = normalized - tau;
        }
        
        Self { radians: normalized }
    }

    /// 角度を±π範囲に正規化
    /// 
    /// # Returns
    /// -π ≤ θ < π の範囲に正規化された角度
    pub fn normalize_signed(&self) -> Self {
        let pi = T::pi();
        let tau = T::tau();
        let mut normalized = self.radians;
        
        // 2π周期で正規化
        while normalized >= pi {
            normalized = normalized - tau;
        }
        while normalized < -pi {
            normalized = normalized + tau;
        }
        
        Self { radians: normalized }
    }

    /// 2つの角度の差を計算（最短経路）
    /// 
    /// # Arguments
    /// * `other` - 目標角度
    /// 
    /// # Returns
    /// この角度から目標角度への最短回転角度
    pub fn difference(&self, other: &Self) -> Self {
        let diff = other.radians - self.radians;
        Self::from_radians(diff).normalize_signed()
    }

    /// 角度の線形補間
    /// 
    /// # Arguments
    /// * `other` - 目標角度
    /// * `t` - 補間パラメータ（0.0〜1.0）
    /// 
    /// # Returns
    /// 補間された角度
    pub fn lerp(&self, other: &Self, t: T) -> Self {
        let diff = self.difference(other);
        Self::from_radians(self.radians + diff.radians * t)
    }

    /// 角度の球面線形補間（最短経路）
    /// 
    /// # Arguments
    /// * `other` - 目標角度
    /// * `t` - 補間パラメータ（0.0〜1.0）
    pub fn slerp(&self, other: &Self, t: T) -> Self {
        // 単純な角度の場合はlerpと同じ
        self.lerp(other, t)
    }

    /// 角度が指定範囲内にあるかを判定
    /// 
    /// # Arguments
    /// * `start` - 開始角度
    /// * `end` - 終了角度
    /// 
    /// # Returns
    /// 角度が範囲内にある場合true
    pub fn is_within_range(&self, start: &Self, end: &Self) -> bool {
        let normalized_self = self.normalize();
        let normalized_start = start.normalize();
        let normalized_end = end.normalize();
        
        if normalized_start.radians <= normalized_end.radians {
            // 通常の範囲（0度から180度など）
            normalized_self.radians >= normalized_start.radians && 
            normalized_self.radians <= normalized_end.radians
        } else {
            // 跨ぎ範囲（300度から60度など）
            normalized_self.radians >= normalized_start.radians || 
            normalized_self.radians <= normalized_end.radians
        }
    }

    /// 角度の絶対値
    pub fn abs(&self) -> Self {
        Self::from_radians(self.radians.abs())
    }

    /// 2つの角度の最小角度を返す
    pub fn min(&self, other: &Self) -> Self {
        if self.radians <= other.radians {
            *self
        } else {
            *other
        }
    }

    /// 2つの角度の最大角度を返す
    pub fn max(&self, other: &Self) -> Self {
        if self.radians >= other.radians {
            *self
        } else {
            *other
        }
    }

    /// 角度がほぼ等しいかを判定
    /// 
    /// # Arguments
    /// * `other` - 比較対象の角度
    /// * `tolerance` - 許容誤差（ラジアン）
    pub fn is_approximately_equal(&self, other: &Self, tolerance: T) -> bool {
        self.difference(other).radians.abs() <= tolerance
    }
}

// 演算子オーバーロード
impl<T: Scalar> Add for Angle<T> {
    type Output = Self;
    
    fn add(self, rhs: Self) -> Self::Output {
        Self::from_radians(self.radians + rhs.radians)
    }
}

impl<T: Scalar> Sub for Angle<T> {
    type Output = Self;
    
    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_radians(self.radians - rhs.radians)
    }
}

impl<T: Scalar> Mul<T> for Angle<T> {
    type Output = Self;
    
    fn mul(self, rhs: T) -> Self::Output {
        Self::from_radians(self.radians * rhs)
    }
}

impl<T: Scalar> Div<T> for Angle<T> {
    type Output = Self;
    
    fn div(self, rhs: T) -> Self::Output {
        Self::from_radians(self.radians / rhs)
    }
}

impl<T: Scalar> Neg for Angle<T> {
    type Output = Self;
    
    fn neg(self) -> Self::Output {
        Self::from_radians(-self.radians)
    }
}

// 表示用実装
impl<T: Scalar> fmt::Display for Angle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}°", self.degrees())
    }
}

// 便利な型エイリアス
pub type Angle32 = Angle<f32>;
pub type Angle64 = Angle<f64>;

// 便利な定数
impl<T: Scalar> Angle<T> {
    /// 0度
    pub const fn zero() -> Self {
        Self { radians: T::zero() }
    }

    /// 90度
    pub fn right_angle() -> Self {
        Self::from_radians(T::pi() / T::two())
    }

    /// 180度
    pub fn straight_angle() -> Self {
        Self::from_radians(T::pi())
    }

    /// 360度
    pub fn full_angle() -> Self {
        Self::from_radians(T::tau())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_angle_creation() {
        let angle_deg = Angle64::from_degrees(90.0);
        let angle_rad = Angle64::from_radians(std::f64::consts::PI / 2.0);
        
        assert!((angle_deg.radians() - angle_rad.radians()).abs() < 1e-10);
        assert!((angle_deg.degrees() - 90.0).abs() < 1e-10);
    }

    #[test]
    fn test_angle_normalization() {
        let angle = Angle64::from_degrees(450.0); // 450度 = 90度
        let normalized = angle.normalize();
        
        assert!((normalized.degrees() - 90.0).abs() < 1e-10);
    }

    #[test]
    fn test_angle_difference() {
        let a1 = Angle64::from_degrees(10.0);
        let a2 = Angle64::from_degrees(350.0);
        let diff = a1.difference(&a2);
        
        // 最短経路は-20度（または340度）
        assert!((diff.degrees() + 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_angle_lerp() {
        let start = Angle64::from_degrees(0.0);
        let end = Angle64::from_degrees(90.0);
        let mid = start.lerp(&end, 0.5);
        
        assert!((mid.degrees() - 45.0).abs() < 1e-10);
    }

    #[test]
    fn test_angle_operations() {
        let a1 = Angle64::from_degrees(30.0);
        let a2 = Angle64::from_degrees(60.0);
        
        let sum = a1 + a2;
        assert!((sum.degrees() - 90.0).abs() < 1e-10);
        
        let diff = a2 - a1;
        assert!((diff.degrees() - 30.0).abs() < 1e-10);
        
        let scaled = a1 * 2.0;
        assert!((scaled.degrees() - 60.0).abs() < 1e-10);
    }

    #[test]
    fn test_angle_range_check() {
        let angle = Angle64::from_degrees(45.0);
        let start = Angle64::from_degrees(0.0);
        let end = Angle64::from_degrees(90.0);
        
        assert!(angle.is_within_range(&start, &end));
        
        let outside = Angle64::from_degrees(120.0);
        assert!(!outside.is_within_range(&start, &end));
    }

    #[test]
    fn test_cross_boundary_range() {
        let angle = Angle64::from_degrees(30.0);
        let start = Angle64::from_degrees(300.0);
        let end = Angle64::from_degrees(60.0);
        
        assert!(angle.is_within_range(&start, &end));
        
        let angle2 = Angle64::from_degrees(330.0);
        assert!(angle2.is_within_range(&start, &end));
    }

    #[test]
    fn test_f32_compatibility() {
        let angle = Angle32::from_degrees(45.0f32);
        assert!((angle.degrees() - 45.0f32).abs() < 1e-6);
    }
}