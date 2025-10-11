//! 距離計算の共通トレイト
//!
//! 点-点、点-線、点-曲線等のすべての距離計算を統一したインターフェース

use crate::Scalar;

/// 距離計算の統一インターフェース
/// 
/// 点-点、点-線、点-曲線等のすべての距離計算を統一したトレイト。
/// 型安全性と計算効率を重視した設計。
pub trait DistanceCalculation<T: Scalar, Target> {
    /// 距離計算の結果（通常はT、複雑な場合は構造体）
    type DistanceResult;

    /// 指定されたターゲットまでの距離を計算
    fn distance_to(&self, target: &Target) -> Self::DistanceResult;

    /// 距離の二乗を計算（平方根計算の回避）
    fn distance_squared_to(&self, target: &Target) -> T {
        // デフォルト実装：distance_to を使用（実装側でオーバーライド推奨）
        let dist = self.distance_to(target);
        let scalar_dist = self.extract_scalar_distance(dist);
        scalar_dist * scalar_dist
    }

    /// DistanceResult からスカラー距離を抽出（ヘルパーメソッド）
    fn extract_scalar_distance(&self, result: Self::DistanceResult) -> T;

    /// 指定した距離以内にあるかの高速判定
    fn is_within_distance(&self, target: &Target, max_distance: T) -> bool {
        self.distance_squared_to(target) <= max_distance * max_distance
    }
}

/// 最近点を含む距離計算の拡張
/// 
/// 距離だけでなく最近点も同時に取得する場合に使用
pub trait DistanceWithClosestPoint<T: Scalar, Target>: DistanceCalculation<T, Target> {
    /// 最近点の型
    type ClosestPoint;

    /// 距離と最近点を同時に計算
    fn distance_and_closest_point(&self, target: &Target) -> (Self::DistanceResult, Self::ClosestPoint);

    /// 最近点のみを取得
    fn closest_point(&self, target: &Target) -> Self::ClosestPoint {
        self.distance_and_closest_point(target).1
    }
}

/// 点集合間の距離計算（将来の拡張用）
pub trait CollectionDistanceCalculation<T: Scalar, TargetCollection> {
    /// 結果のコレクション型
    type DistanceCollection;

    /// 複数のターゲットに対する距離を一括計算
    fn distances_to_collection(&self, targets: &TargetCollection) -> Self::DistanceCollection;

    /// 最も近いターゲットとその距離を取得
    fn nearest_in_collection(&self, targets: &TargetCollection) -> Option<(T, usize)>;
}