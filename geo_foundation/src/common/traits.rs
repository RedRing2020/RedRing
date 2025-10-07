/// 共通トレイト定義
/// 
/// 異なるクレート間で共有される基本的なトレイト
/// 正規化可能なトレイト
pub trait Normalizable {
    /// 正規化を行う
    fn normalize(&self) -> Self;
    
    /// 正規化可能かどうかを判定
    fn is_normalizable(&self) -> bool;
}

/// 測定可能なトレイト
pub trait Measurable {
    /// 測定値の型
    type Measure;
    
    /// 測定を行う
    fn measure(&self) -> Self::Measure;
}

/// 変換可能なトレイト
pub trait Transformable<T> {
    /// 変換を適用
    fn transform(&self, transformation: &T) -> Self;
}

/// 境界を持つトレイト
pub trait Bounded {
    /// 境界の型
    type Bounds;
    
    /// 境界を取得
    fn bounds(&self) -> Self::Bounds;
}