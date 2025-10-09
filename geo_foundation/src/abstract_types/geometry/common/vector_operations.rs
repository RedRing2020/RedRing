//! ベクトル操作の共通トレイト
//!
//! 正規化、内積、外積等のベクトル演算の統一インターフェース

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

/// ベクトル基本操作トレイト（将来の拡張用）
pub trait VectorOperations<T> {
    /// 内積計算
    fn dot(&self, other: &Self) -> T;

    /// ベクトルの長さ（ノルム）を取得
    fn length(&self) -> T;

    /// ベクトルの長さの二乗を取得（平方根計算を避けるため）
    fn length_squared(&self) -> T;

    /// ベクトルがゼロベクトルかどうかを判定
    fn is_zero(&self, tolerance: T) -> bool;

    /// ベクトルの単位化が可能かどうかを判定
    fn is_normalizable(&self, tolerance: T) -> bool;
}

/// 3Dベクトル固有の操作トレイト
pub trait Vector3DOperations<T>: VectorOperations<T> {
    /// 外積計算
    fn cross(&self, other: &Self) -> Self;

    /// スカラー三重積 (a · (b × c))
    fn scalar_triple_product(&self, b: &Self, c: &Self) -> T;

    /// ベクトル三重積 (a × (b × c))
    fn vector_triple_product(&self, b: &Self, c: &Self) -> Self;
}

/// 2Dベクトル固有の操作トレイト
pub trait Vector2DOperations<T>: VectorOperations<T> {
    /// 2D外積（スカラー値）
    fn cross_2d(&self, other: &Self) -> T;

    /// 垂直ベクトルを取得（90度回転）
    fn perpendicular(&self) -> Self;

    /// 時計回りに90度回転
    fn rotate_90_cw(&self) -> Self;

    /// 反時計回りに90度回転
    fn rotate_90_ccw(&self) -> Self;
}
