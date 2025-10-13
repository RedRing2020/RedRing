//! 交点計算の統一インターフェース
//!
//! 幾何プリミティブ間の交点計算を統一的に扱うためのトレイト定義

use crate::Scalar;

/// 基本交点計算トレイト
///
/// 2つの幾何オブジェクト間の交点を計算するための統一インターフェース
pub trait BasicIntersection<T: Scalar, Other> {
    /// 出力される点の型
    type Point;

    /// 他のオブジェクトとの交点を取得
    ///
    /// # 引数
    /// * `other` - 交点を計算する相手オブジェクト
    /// * `tolerance` - 計算精度の許容誤差
    ///
    /// # 戻り値
    /// 交点が存在する場合は Some(point)、存在しない場合は None
    fn intersection_with(&self, other: &Other, tolerance: T) -> Option<Self::Point>;
}

/// 複数交点計算トレイト
///
/// 複数の交点を返す可能性がある幾何オブジェクト間の交点計算
pub trait MultipleIntersection<T: Scalar, Other> {
    /// 出力される点の型
    type Point;

    /// 他のオブジェクトとの全交点を取得
    ///
    /// # 引数
    /// * `other` - 交点を計算する相手オブジェクト
    /// * `tolerance` - 計算精度の許容誤差
    ///
    /// # 戻り値
    /// 交点のベクトル（0個以上）
    fn intersections_with(&self, other: &Other, tolerance: T) -> Vec<Self::Point>;
}

/// 自己交差検出トレイト
///
/// 単一オブジェクトの自己交差を検出
pub trait SelfIntersection<T: Scalar> {
    /// 出力される点の型
    type Point;

    /// 自己交差点を取得
    ///
    /// # 引数
    /// * `tolerance` - 計算精度の許容誤差
    ///
    /// # 戻り値
    /// 自己交差点のベクトル
    fn self_intersections(&self, tolerance: T) -> Vec<Self::Point>;
}

/// 交点計算のヘルパートレイト
///
/// tolerance のデフォルト値を提供する便利機能
pub trait IntersectionHelpers<T: Scalar, Other>: BasicIntersection<T, Other> {
    /// デフォルト tolerance での交点計算
    fn intersection(&self, other: &Other) -> Option<Self::Point> {
        self.intersection_with(other, T::EPSILON)
    }
}

// ヘルパートレイトの自動実装
impl<T: Scalar, Other, U> IntersectionHelpers<T, Other> for U where U: BasicIntersection<T, Other> {}
