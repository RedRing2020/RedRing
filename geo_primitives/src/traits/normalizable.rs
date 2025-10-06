/// 正規化可能なベクトルトレイト

/// 正規化可能なベクトルトレイト
pub trait Normalizable {
    type Output;

    /// 正規化（エラーハンドリング付き）
    fn normalize(&self) -> Option<Self::Output>;

    /// 正規化（ゼロベクトルの場合はゼロベクトルを返す）
    fn normalize_or_zero(&self) -> Self::Output;

    /// 正規化できるかどうかをチェック
    fn can_normalize(&self, tolerance: f64) -> bool;
}
