/// 許容誤差に関する抽象型とトレイト

use crate::common::constants::GEOMETRIC_TOLERANCE;

/// 許容誤差コンテキスト
#[derive(Debug, Clone, Copy)]
pub struct ToleranceContext {
    tolerance: f64,
}

impl ToleranceContext {
    /// 新しい許容誤差コンテキストを作成
    pub fn new(tolerance: f64) -> Self {
        Self { tolerance }
    }

    /// デフォルトの許容誤差コンテキスト
    pub fn default() -> Self {
        Self { tolerance: GEOMETRIC_TOLERANCE }
    }

    /// 許容誤差値を取得
    pub fn tolerance(&self) -> f64 {
        self.tolerance
    }
}

/// 許容誤差を考慮した等価比較トレイト
pub trait TolerantEq {
    /// 許容誤差を考慮した等価比較
    fn tolerant_eq(&self, other: &Self, context: &ToleranceContext) -> bool;

    /// デフォルト許容誤差での等価比較
    fn tolerant_eq_default(&self, other: &Self) -> bool {
        self.tolerant_eq(other, &ToleranceContext::default())
    }
}

// f64に対する実装
impl TolerantEq for f64 {
    fn tolerant_eq(&self, other: &f64, context: &ToleranceContext) -> bool {
        (self - other).abs() < context.tolerance()
    }
}

// Scalarに対する実装
impl TolerantEq for crate::Scalar {
    fn tolerant_eq(&self, other: &Self, context: &ToleranceContext) -> bool {
        self.value().tolerant_eq(&other.value(), context)
    }
}