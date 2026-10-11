//! 交点計算のtrait定義
//!
//! 実装は `geo_algorithms::intersection` 以下に配置される。

use analysis::abstract_types::Scalar;

/// 基本交点計算トレイト
///
/// 2つの幾何オブジェクト間の交点を計算するための統一インターフェース。
pub trait BasicIntersection<T: Scalar, Other> {
    /// 出力される点の型
    type Point;

    /// 他のオブジェクトとの交点を取得
    ///
    /// 数値安定性（平行判定・共面判定等）のトレランスは各実装が内部既定値を使用する。
    /// アプリケーショントレランスが必要な場合は専用 trait を別途用意する。
    ///
    /// # 引数
    /// * `other` - 交点を計算する相手オブジェクト
    ///
    /// # 戻り値
    /// 交点が存在する場合は `Some(point)`、存在しない場合は `None`
    fn intersection_with(&self, other: &Other) -> Option<Self::Point>;
}
