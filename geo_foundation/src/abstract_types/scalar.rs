/// Scalar - 数値の抽象化型
/// 
/// 幾何計算において、浮動小数点数の精度や許容誤差を
/// 統一的に扱うための抽象型

use std::fmt;
use std::ops::{Add, Sub, Mul, Div, Neg};

/// 幾何計算用の数値抽象型
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scalar {
    value: f64,
}

impl Scalar {
    /// 新しいScalarを作成
    pub fn new(value: f64) -> Self {
        Self { value }
    }

    /// 値を取得
    pub fn value(&self) -> f64 {
        self.value
    }

    /// ゼロ値
    pub const ZERO: Self = Self { value: 0.0 };
    
    /// 単位値
    pub const ONE: Self = Self { value: 1.0 };

    /// 絶対値
    pub fn abs(&self) -> Self {
        Self::new(self.value.abs())
    }

    /// 平方根
    pub fn sqrt(&self) -> Self {
        Self::new(self.value.sqrt())
    }

    /// 正弦
    pub fn sin(&self) -> Self {
        Self::new(self.value.sin())
    }

    /// 余弦
    pub fn cos(&self) -> Self {
        Self::new(self.value.cos())
    }
}

// 算術演算の実装
impl Add for Scalar {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.value + rhs.value)
    }
}

impl Sub for Scalar {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.value - rhs.value)
    }
}

impl Mul for Scalar {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.value * rhs.value)
    }
}

impl Div for Scalar {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self::new(self.value / rhs.value)
    }
}

impl Neg for Scalar {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.value)
    }
}

// f64からの変換
impl From<f64> for Scalar {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

impl From<Scalar> for f64 {
    fn from(scalar: Scalar) -> Self {
        scalar.value
    }
}

// 表示
impl fmt::Display for Scalar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}