//! 正規化操作の共通トレイト
//!
//! Vector、Direction等の正規化可能な型に対する統一インターフェース

use crate::Scalar;

/// 正規化可能なオブジェクトの統一トレイト
/// 
/// Vector、Direction等の正規化可能な型すべてが実装すべき共通インターフェース。
/// 型安全性とエラーハンドリングを重視した設計。
pub trait Normalizable<T: Scalar> {
    /// 正規化の結果型（通常は Self）
    type Output;

    /// 正規化（エラーハンドリング付き）
    /// 
    /// ゼロベクトル等、正規化できない場合は None を返す
    fn normalize(&self) -> Option<Self::Output>;

    /// 正規化（フォールバック付き）
    /// 
    /// ゼロベクトルの場合はゼロベクトルを返す（安全なフォールバック）
    fn normalize_or_zero(&self) -> Self::Output;

    /// 正規化できるかどうかをチェック
    fn can_normalize(&self, tolerance: T) -> bool;

    /// 安全な正規化（デフォルト実装）
    /// 
    /// normalize()のラッパーで、エラーハンドリングを簡素化
    fn try_normalize(&self) -> Result<Self::Output, NormalizationError> {
        self.normalize().ok_or(NormalizationError::ZeroLength)
    }
}

/// 正規化エラーの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalizationError {
    /// ゼロ長（正規化不可能）
    ZeroLength,
    /// 数値的不安定
    NumericalInstability,
}

impl std::fmt::Display for NormalizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NormalizationError::ZeroLength => write!(f, "ゼロ長のため正規化できません"),
            NormalizationError::NumericalInstability => write!(f, "数値的不安定のため正規化できません"),
        }
    }
}

impl std::error::Error for NormalizationError {}

/// 条件付き正規化トレイト（将来の拡張用）
/// 
/// 特定の条件下でのみ正規化を行う場合に使用
pub trait ConditionalNormalizable<T: Scalar>: Normalizable<T> {
    /// 条件を満たす場合のみ正規化
    fn normalize_if<F>(&self, condition: F) -> Option<Self::Output>
    where
        F: FnOnce(&Self) -> bool;

    /// 条件を満たす場合のみ正規化、満たさない場合は元の値を返す
    fn normalize_if_or_self<F>(&self, condition: F) -> Self::Output
    where
        F: FnOnce(&Self) -> bool,
        Self::Output: From<Self>,
        Self: Clone;
}