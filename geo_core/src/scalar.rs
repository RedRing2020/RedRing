//! スカラー値と許容誤差を統合した型システム
//!
//! 数値計算の精度を保証し、単位系の安全性を提供する。
use crate::tolerance::{ToleranceContext, TolerantEq, TolerantOrd};
use std::fmt;
use std::ops::{Add, Sub, Mul, Div, Neg};

/// 許容誤差を持つスカラー値
/// 
/// 注意: 長さに関する値はmm単位で格納される
#[derive(Debug, Clone, Copy)]
pub struct Scalar {
    value: f64,
    /// この値に関連する許容誤差（オプション）
    tolerance: Option<f64>,
}

impl Scalar {
    /// 新しいスカラー値を作成
    pub fn new(value: f64) -> Self {
        Self {
            value,
            tolerance: None,
        }
    }

    /// f64からスカラー値を作成（エイリアス）
    pub fn from_f64(value: f64) -> Self {
        Self::new(value)
    }

    /// 許容誤差付きスカラー値を作成
    pub fn with_tolerance(value: f64, tolerance: f64) -> Self {
        debug_assert!(tolerance >= 0.0, "Tolerance must be non-negative");
        Self {
            value,
            tolerance: Some(tolerance),
        }
    }

    /// 値を取得
    pub fn value(&self) -> f64 {
        self.value
    }

    /// 許容誤差を取得
    pub fn tolerance(&self) -> Option<f64> {
        self.tolerance
    }

    /// 有効許容誤差を取得（指定されていない場合はコンテキストから）
    pub fn effective_tolerance(&self, context: &ToleranceContext) -> f64 {
        self.tolerance.unwrap_or(context.linear)
    }

    /// 値の絶対値
    pub fn abs(&self) -> Self {
        Self {
            value: self.value.abs(),
            tolerance: self.tolerance,
        }
    }

    /// 平方根
    pub fn sqrt(&self) -> Self {
        debug_assert!(self.value >= 0.0, "Cannot take square root of negative number");
        Self {
            value: self.value.sqrt(),
            // 誤差伝播: δ(√x) ≈ δx/(2√x)
            tolerance: self.tolerance.map(|t| t / (2.0 * self.value.sqrt())),
        }
    }

    /// 自然対数
    pub fn ln(&self) -> Self {
        debug_assert!(self.value > 0.0, "Cannot take logarithm of non-positive number");
        Self {
            value: self.value.ln(),
            // 誤差伝播: δ(ln(x)) ≈ δx/x
            tolerance: self.tolerance.map(|t| t / self.value),
        }
    }

    /// 指数関数
    pub fn exp(&self) -> Self {
        let exp_val = self.value.exp();
        Self {
            value: exp_val,
            // 誤差伝播: δ(exp(x)) ≈ exp(x) * δx
            tolerance: self.tolerance.map(|t| exp_val * t),
        }
    }

    /// 正弦
    pub fn sin(&self) -> Self {
        Self {
            value: self.value.sin(),
            // 誤差伝播: δ(sin(x)) ≈ cos(x) * δx
            tolerance: self.tolerance.map(|t| self.value.cos().abs() * t),
        }
    }

    /// 余弦
    pub fn cos(&self) -> Self {
        Self {
            value: self.value.cos(),
            // 誤差伝播: δ(cos(x)) ≈ sin(x) * δx
            tolerance: self.tolerance.map(|t| self.value.sin().abs() * t),
        }
    }

    /// 正接
    pub fn tan(&self) -> Self {
        let cos_val = self.value.cos();
        debug_assert!(cos_val.abs() > 1e-15, "Tangent is undefined at π/2 + nπ");
        Self {
            value: self.value.tan(),
            // 誤差伝播: δ(tan(x)) ≈ sec²(x) * δx = δx/cos²(x)
            tolerance: self.tolerance.map(|t| t / (cos_val * cos_val)),
        }
    }

    /// べき乗
    pub fn powf(&self, exponent: f64) -> Self {
        debug_assert!(self.value > 0.0 || exponent.fract() == 0.0,
                     "Fractional power of negative number");
        let pow_val = self.value.powf(exponent);
        Self {
            value: pow_val,
            // 誤差伝播: δ(x^n) ≈ n * x^(n-1) * δx
            tolerance: self.tolerance.map(|t| {
                if self.value == 0.0 && exponent > 1.0 {
                    0.0
                } else {
                    exponent * self.value.powf(exponent - 1.0).abs() * t
                }
            }),
        }
    }

    /// 安全な除算（ゼロ除算チェック付き）
    pub fn safe_div(&self, other: &Self, context: &ToleranceContext) -> Option<Self> {
        let other_tolerance = other.effective_tolerance(context);
        if other.value.abs() <= other_tolerance {
            None // ゼロ除算の可能性
        } else {
            Some(*self / *other)
        }
    }
}

impl PartialEq for Scalar {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.tolerance == other.tolerance
    }
}

impl TolerantEq for Scalar {
    fn tolerant_eq(&self, other: &Scalar, context: &ToleranceContext) -> bool {
        let self_tol = self.effective_tolerance(context);
        let other_tol = other.effective_tolerance(context);
        let combined_tolerance = (self_tol * self_tol + other_tol * other_tol).sqrt();

        (self.value - other.value).abs() <= combined_tolerance
    }
}

impl TolerantOrd for Scalar {
    fn tolerant_cmp(&self, other: &Scalar, context: &ToleranceContext) -> Option<std::cmp::Ordering> {
        if self.tolerant_eq(other, context) {
            Some(std::cmp::Ordering::Equal)
        } else if self.value < other.value {
            Some(std::cmp::Ordering::Less)
        } else {
            Some(std::cmp::Ordering::Greater)
        }
    }
}

// f64との比較
impl PartialEq<f64> for Scalar {
    fn eq(&self, other: &f64) -> bool {
        self.value == *other
    }
}

impl PartialOrd<f64> for Scalar {
    fn partial_cmp(&self, other: &f64) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(other)
    }
}

// 算術演算の実装
impl Add for Scalar {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            value: self.value + other.value,
            // 誤差伝播: δ(a + b) = √(δa² + δb²)
            tolerance: match (self.tolerance, other.tolerance) {
                (Some(a), Some(b)) => Some((a * a + b * b).sqrt()),
                (Some(a), None) | (None, Some(a)) => Some(a),
                (None, None) => None,
            },
        }
    }
}

impl Sub for Scalar {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            value: self.value - other.value,
            // 減算も加算と同じ誤差伝播
            tolerance: match (self.tolerance, other.tolerance) {
                (Some(a), Some(b)) => Some((a * a + b * b).sqrt()),
                (Some(a), None) | (None, Some(a)) => Some(a),
                (None, None) => None,
            },
        }
    }
}

impl Mul for Scalar {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        Self {
            value: self.value * other.value,
            // 誤差伝播: δ(ab) = √((b*δa)² + (a*δb)²)
            tolerance: match (self.tolerance, other.tolerance) {
                (Some(da), Some(db)) => {
                    let term1 = other.value * da;
                    let term2 = self.value * db;
                    Some((term1 * term1 + term2 * term2).sqrt())
                },
                (Some(da), None) => Some(other.value.abs() * da),
                (None, Some(db)) => Some(self.value.abs() * db),
                (None, None) => None,
            },
        }
    }
}

impl Div for Scalar {
    type Output = Self;

    fn div(self, other: Self) -> Self::Output {
        debug_assert!(other.value != 0.0, "Division by zero");
        Self {
            value: self.value / other.value,
            // 誤差伝播: δ(a/b) = √((δa/b)² + (a*δb/b²)²)
            tolerance: match (self.tolerance, other.tolerance) {
                (Some(da), Some(db)) => {
                    let term1 = da / other.value;
                    let term2 = self.value * db / (other.value * other.value);
                    Some((term1 * term1 + term2 * term2).sqrt())
                },
                (Some(da), None) => Some(da / other.value.abs()),
                (None, Some(db)) => Some(self.value.abs() * db / (other.value * other.value)),
                (None, None) => None,
            },
        }
    }
}

impl Neg for Scalar {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            value: -self.value,
            tolerance: self.tolerance,
        }
    }
}

// f64からの変換
impl From<f64> for Scalar {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for Scalar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.tolerance {
            Some(tol) => write!(f, "{:.6} ± {:.2e}", self.value, tol),
            None => write!(f, "{:.6}", self.value),
        }
    }
}

/// 数学定数
pub mod constants {
    use super::Scalar;

    pub const PI: Scalar = Scalar { value: std::f64::consts::PI, tolerance: None };
    pub const E: Scalar = Scalar { value: std::f64::consts::E, tolerance: None };
    pub const SQRT_2: Scalar = Scalar { value: std::f64::consts::SQRT_2, tolerance: None };
    pub const FRAC_1_SQRT_2: Scalar = Scalar { value: std::f64::consts::FRAC_1_SQRT_2, tolerance: None };
}

