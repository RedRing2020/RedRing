//! Boolean Operations - ブール演算の統一インターフェース
//!
//! 幾何プリミティブ間のブール演算（和集合、積集合、差集合等）を提供

use crate::Scalar;

/// Boolean Operations トレイト
///
/// 2つの幾何オブジェクト間のブール演算を提供
/// 結果は新しい幾何オブジェクトとして返される
pub trait BooleanOperations<T: Scalar>: Sized {
    /// エラー型
    type Error;

    /// 和集合（Union）
    ///
    /// 2つのオブジェクトを統合した新しいオブジェクトを生成
    ///
    /// # 引数
    /// * `other` - 和集合を取る相手オブジェクト
    ///
    /// # 戻り値
    /// 和集合の結果、または演算が不可能な場合はエラー
    ///
    /// # 例
    /// ```text
    /// A ∪ B = 両方を含む領域
    /// ```
    fn union(&self, other: &Self) -> Result<Self, Self::Error>;

    /// 積集合（Intersection）
    ///
    /// 2つのオブジェクトの重なり部分を新しいオブジェクトとして生成
    ///
    /// # 引数
    /// * `other` - 積集合を取る相手オブジェクト
    ///
    /// # 戻り値
    /// 積集合の結果、または重なりがない場合はエラー
    ///
    /// # 例
    /// ```text
    /// A ∩ B = 両方に含まれる領域
    /// ```
    fn intersection(&self, other: &Self) -> Result<Self, Self::Error>;

    /// 差集合（Difference）
    ///
    /// 自分から相手を除いた部分を新しいオブジェクトとして生成
    ///
    /// # 引数
    /// * `other` - 差し引く相手オブジェクト
    ///
    /// # 戻り値
    /// 差集合の結果、または演算が不可能な場合はエラー
    ///
    /// # 例
    /// ```text
    /// A - B = A にあって B にない領域
    /// ```
    fn difference(&self, other: &Self) -> Result<Self, Self::Error>;

    /// 対称差（Symmetric Difference / XOR）
    ///
    /// 片方にのみ含まれる部分を新しいオブジェクトとして生成
    ///
    /// # 引数
    /// * `other` - 対称差を取る相手オブジェクト
    ///
    /// # 戻り値
    /// 対称差の結果、または演算が不可能な場合はエラー
    ///
    /// # 例
    /// ```text
    /// A ⊕ B = (A - B) ∪ (B - A) = 片方にのみ含まれる領域
    /// ```
    fn symmetric_difference(&self, other: &Self) -> Result<Self, Self::Error>;
}

/// Boolean Operations with Tolerance - 許容誤差付きブール演算
///
/// 数値誤差を考慮したブール演算を提供
pub trait TolerantBooleanOperations<T: Scalar>: Sized {
    /// エラー型
    type Error;

    /// 許容誤差付き和集合
    ///
    /// # 引数
    /// * `other` - 和集合を取る相手オブジェクト
    /// * `tolerance` - 許容誤差
    fn union_with_tolerance(&self, other: &Self, tolerance: T) -> Result<Self, Self::Error>;

    /// 許容誤差付き積集合
    ///
    /// # 引数
    /// * `other` - 積集合を取る相手オブジェクト
    /// * `tolerance` - 許容誤差
    fn intersection_with_tolerance(&self, other: &Self, tolerance: T) -> Result<Self, Self::Error>;

    /// 許容誤差付き差集合
    ///
    /// # 引数
    /// * `other` - 差し引く相手オブジェクト
    /// * `tolerance` - 許容誤差
    fn difference_with_tolerance(&self, other: &Self, tolerance: T) -> Result<Self, Self::Error>;

    /// 許容誤差付き対称差
    ///
    /// # 引数
    /// * `other` - 対称差を取る相手オブジェクト
    /// * `tolerance` - 許容誤差
    fn symmetric_difference_with_tolerance(
        &self,
        other: &Self,
        tolerance: T,
    ) -> Result<Self, Self::Error>;
}

/// Boolean Operations Error - ブール演算エラー型
///
/// ブール演算時に発生する可能性のあるエラー
#[derive(Debug, Clone, PartialEq)]
pub enum BooleanError {
    /// 結果が空集合
    EmptyResult,

    /// 演算不可能（形状の種類が異なる等）
    IncompatibleShapes,

    /// 数値計算エラー
    NumericalError,

    /// 退化した形状（点、線分等）
    DegenerateShape,

    /// 実装されていない演算
    NotImplemented,

    /// その他のエラー
    Other(String),
}

impl std::fmt::Display for BooleanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyResult => write!(f, "Boolean operation resulted in empty set"),
            Self::IncompatibleShapes => write!(f, "Incompatible shapes for boolean operation"),
            Self::NumericalError => write!(f, "Numerical error in boolean operation"),
            Self::DegenerateShape => write!(f, "Degenerate shape in boolean operation"),
            Self::NotImplemented => write!(f, "Boolean operation not implemented"),
            Self::Other(msg) => write!(f, "Boolean operation error: {}", msg),
        }
    }
}

impl std::error::Error for BooleanError {}

/// Multiple Boolean Operations - 複数オブジェクトのブール演算
///
/// 3つ以上のオブジェクトに対するブール演算を効率的に実行
pub trait MultipleBooleanOperations<T: Scalar>: Sized {
    /// エラー型
    type Error;

    /// 複数オブジェクトの和集合
    ///
    /// # 引数
    /// * `others` - 和集合を取るオブジェクトのスライス
    ///
    /// # 戻り値
    /// 全ての和集合、または演算が不可能な場合はエラー
    fn union_multiple(objects: &[Self]) -> Result<Self, Self::Error>;

    /// 複数オブジェクトの積集合
    ///
    /// # 引数
    /// * `others` - 積集合を取るオブジェクトのスライス
    ///
    /// # 戻り値
    /// 全ての積集合、または重なりがない場合はエラー
    fn intersection_multiple(objects: &[Self]) -> Result<Self, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean_error_display() {
        let error = BooleanError::EmptyResult;
        assert_eq!(error.to_string(), "Boolean operation resulted in empty set");

        let error = BooleanError::IncompatibleShapes;
        assert_eq!(
            error.to_string(),
            "Incompatible shapes for boolean operation"
        );

        let error = BooleanError::Other("custom error".to_string());
        assert_eq!(error.to_string(), "Boolean operation error: custom error");
    }
}
