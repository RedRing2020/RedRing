/// 許容誤差に関する抽象型とトレイト

/// デフォルトの幾何計算許容誤差
pub const GEOMETRIC_TOLERANCE: f64 = 1e-10;

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

    /// 許容誤差値を取得
    pub fn tolerance(&self) -> f64 {
        self.tolerance
    }
}

impl Default for ToleranceContext {
    fn default() -> Self {
        Self {
            tolerance: GEOMETRIC_TOLERANCE,
        }
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

// Scalarトレイトを実装する型に対する汎用実装
impl<T: crate::abstract_types::Scalar> TolerantEq for T {
    fn tolerant_eq(&self, other: &Self, context: &ToleranceContext) -> bool {
        (*self - *other).abs() < T::from_f64(context.tolerance())
    }
}
