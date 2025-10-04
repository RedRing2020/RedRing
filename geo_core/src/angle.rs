/// 角度型抽象 (ラジアン表現)
///
/// 設計方針:
/// - 内部表現: f64 (常にラジアン)
/// - 正規化: デフォルトは (-PI, PI] に正規化
/// - 変換: 度 <-> ラジアン
/// - 加減算/スカラー倍/比較 (トレラント比較は TolerantEq 実装)
/// - 三角関数: sin/cos/tan を直接提供 (f64返す軽量API)
/// - 安全性: NaN/inf は生成時に抑止 (debug_assert)
/// - 将来拡張: 有向角/半角/角度差ユーティリティ、AngleInterval
///
/// Circle/Arc の f64 化準備として、角度領域の型安全化を提供する。

use crate::scalar::Scalar; // 必要に応じて Scalar 版インターフェースを段階的に削除予定
use crate::tolerance::{ToleranceContext, TolerantEq, TolerantOrd};
use std::fmt;
use std::ops::{Add, Sub, Mul, Div, Neg};

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub struct Angle {
    radians: f64,
}

impl Angle {
    /// ラジアン値から生成（正規化あり）
    pub fn from_radians(r: f64) -> Self {
        debug_assert!(r.is_finite(), "Angle must be finite");
        Self { radians: Self::normalize_pi(r) }
    }

    /// 正規化無しで内部生成（大量生成で事後一括正規化したいケース用）
    pub fn from_radians_unchecked(r: f64) -> Self { Self { radians: r } }

    /// 度数法から生成
    pub fn from_degrees(d: f64) -> Self { Self::from_radians(d.to_radians()) }

    /// f64 ラジアン値取得
    pub fn radians(&self) -> f64 { self.radians }
    /// 度数法取得
    pub fn degrees(&self) -> f64 { self.radians.to_degrees() }

    /// (-PI, PI] に正規化した新しい Angle
    pub fn normalized(self) -> Self { Self::from_radians(self.radians) }

    /// 現在の値を正規化 (in-place)
    pub fn normalize_in_place(&mut self) { self.radians = Self::normalize_pi(self.radians); }

    /// [0, 2PI) に正規化
    pub fn normalized_positive(self) -> Self {
        let two_pi = std::f64::consts::PI * 2.0;
        let mut r = self.radians % two_pi;
        if r < 0.0 { r += two_pi; }
        Angle { radians: r }
    }

    /// 符号付き差分 (self - other) を (-PI, PI] で返す
    pub fn delta(self, other: Angle) -> Self {
        Self::from_radians(self.radians - other.radians)
    }

    /// 絶対角度差 |self - other| を [0, PI] で返す
    pub fn abs_delta(self, other: Angle) -> Self { self.delta(other).rabs() }

    /// 反対向き (π 加算) を返す
    pub fn opposite(self) -> Self { Self::from_radians(self.radians + std::f64::consts::PI) }

    /// 角度の絶対値（幾何では向き用途が多いので正規化後に abs）
    pub fn rabs(self) -> Self { Self { radians: self.radians.abs() }.normalized() }

    /// 三角関数 (f64 直接返却)
    pub fn sin(self) -> f64 { self.radians.sin() }
    pub fn cos(self) -> f64 { self.radians.cos() }
    pub fn tan(self) -> f64 { self.radians.tan() }

    /// 2D ベクトルとして (cosθ, sinθ)
    pub fn unit_direction(self) -> (f64, f64) { (self.cos(), self.sin()) }

    /// 内部正規化関数: (-PI, PI]
    #[inline]
    fn normalize_pi(r: f64) -> f64 {
        let two_pi = std::f64::consts::PI * 2.0;
        let mut v = r % two_pi;
        if v <= -std::f64::consts::PI { v += two_pi; }
        else if v > std::f64::consts::PI { v -= two_pi; }
        v
    }

    /// 線形補間 (t in [0,1])
    pub fn lerp(a: Angle, b: Angle, t: f64) -> Self {
        debug_assert!((0.0..=1.0).contains(&t), "t must be in [0,1]");
        // 最短回転方向を考慮した補間
        let mut diff = b.radians - a.radians;
        if diff > std::f64::consts::PI { diff -= 2.0 * std::f64::consts::PI; }
        else if diff < -std::f64::consts::PI { diff += 2.0 * std::f64::consts::PI; }
        Self::from_radians(a.radians + diff * t)
    }

    /// Scalar ラッパ (暫定; 将来削除予定) - ラジアン値をそのまま格納
    pub fn to_scalar(self) -> Scalar { Scalar::new(self.radians) }
    pub fn from_scalar(s: Scalar) -> Self { Self::from_radians(s.value()) }
}

// 算術演算 (角度 + 角度, 角度 - 角度, 角度 * スカラー, 角度 / スカラー)
impl Add for Angle { type Output = Self; fn add(self, rhs: Self) -> Self::Output { Self::from_radians(self.radians + rhs.radians) } }
impl Sub for Angle { type Output = Self; fn sub(self, rhs: Self) -> Self::Output { Self::from_radians(self.radians - rhs.radians) } }
impl Mul<f64> for Angle { type Output = Self; fn mul(self, rhs: f64) -> Self::Output { Self::from_radians(self.radians * rhs) } }
impl Mul<Angle> for f64 { type Output = Angle; fn mul(self, rhs: Angle) -> Self::Output { Angle::from_radians(self * rhs.radians) } }
impl Div<f64> for Angle { type Output = Self; fn div(self, rhs: f64) -> Self::Output { Self::from_radians(self.radians / rhs) } }
impl Neg for Angle { type Output = Self; fn neg(self) -> Self::Output { Self::from_radians(-self.radians) } }

impl fmt::Display for Angle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{:.6} rad ({:.6}°)", self.radians, self.degrees()) }
}

impl TolerantEq for Angle {
    fn tolerant_eq(&self, other: &Self, context: &ToleranceContext) -> bool {
        // 角度許容値を使用 (context.angular)
        (self.delta(*other).radians().abs()) <= context.angular
    }
}

impl TolerantOrd for Angle {
    fn tolerant_cmp(&self, other: &Self, context: &ToleranceContext) -> Option<std::cmp::Ordering> {
        if self.tolerant_eq(other, context) { Some(std::cmp::Ordering::Equal) }
        else if self.radians < other.radians { Some(std::cmp::Ordering::Less) }
        else { Some(std::cmp::Ordering::Greater) }
    }
}

// 代表的な定数
impl Angle {
    pub const ZERO: Angle = Angle { radians: 0.0 };
    pub const PI: Angle = Angle { radians: std::f64::consts::PI };
    pub const TAU: Angle = Angle { radians: std::f64::consts::PI * 2.0 };
    pub const HALF_PI: Angle = Angle { radians: std::f64::consts::PI / 2.0 };
    pub const QUARTER_PI: Angle = Angle { radians: std::f64::consts::PI / 4.0 };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalization_pi_range() {
        let a = Angle::from_radians(3.5 * std::f64::consts::PI); // 1.5π -> -0.5π
        assert!((a.radians() + std::f64::consts::PI / 2.0).abs() < 1e-12);
    }

    #[test]
    fn positive_normalization() {
        let a = Angle::from_radians(-0.25 * std::f64::consts::PI);
        let p = a.normalized_positive();
        assert!((p.radians() - (1.75 * std::f64::consts::PI)).abs() < 1e-12);
    }

    #[test]
    fn delta_and_abs_delta() {
        let a = Angle::from_degrees(10.0);
        let b = Angle::from_degrees(-170.0); // 差分は 180° -> π
        assert!((a.delta(b).radians().abs() - std::f64::consts::PI).abs() < 1e-12);
        assert!((a.abs_delta(b).radians() - std::f64::consts::PI).abs() < 1e-12);
    }

    #[test]
    fn lerp_shortest_path() {
        let a = Angle::from_degrees(170.0);
        let b = Angle::from_degrees(-170.0);
        let mid = Angle::lerp(a, b, 0.5);
        // 最短経路なら -180 <-> 180 を跨がず 180 の差分を -20 度方向 (実際は 180-340= -160?)
        // 170 -> -170 は差分 -340° → +20° の最短補正, 中間は 170 + 10 = 180° -> π
        assert!((mid.radians() - std::f64::consts::PI).abs() < 1e-8);
    }

    #[test]
    fn trig_consistency() {
        let a = Angle::from_degrees(60.0);
        assert!((a.sin() - (std::f64::consts::PI / 3.0).sin()).abs() < 1e-12);
        assert!((a.cos() - 0.5).abs() < 1e-12);
    }
}
