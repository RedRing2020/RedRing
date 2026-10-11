//! 衝突検出・距離計算のtrait定義
//!
//! 実装は `geo_algorithms::collision` 以下に配置される。

use analysis::abstract_types::Scalar;

/// 基本衝突検出trait
///
/// 2つの幾何オブジェクト間の基本的な衝突検出・距離計算を提供。
/// 全ての幾何プリミティブ間の組み合わせに対して実装される。
pub trait BasicCollision<T: Scalar, Other> {
    /// 2D 点型
    type Point2D;

    /// 衝突しているかどうか
    ///
    /// # 引数
    /// * `other` - 衝突判定を行う相手オブジェクト
    /// * `tolerance` - 判定精度の許容誤差
    ///
    /// # 戻り値
    /// 衝突している場合は `true`
    fn intersects(&self, other: &Other, tolerance: T) -> bool;

    /// 重なりを持つかどうか
    ///
    /// 完全に内包している場合や部分的に重なっている場合を検出。
    ///
    /// # 引数
    /// * `other` - 重なり判定を行う相手オブジェクト
    /// * `tolerance` - 判定精度の許容誤差
    ///
    /// # 戻り値
    /// 重なりがある場合は `true`
    fn overlaps(&self, other: &Other, tolerance: T) -> bool;

    /// 最短距離
    ///
    /// 2つのオブジェクト間の最短距離を計算。
    /// 重なりがある場合は `0` を返す。
    ///
    /// # 引数
    /// * `other` - 距離計算を行う相手オブジェクト
    ///
    /// # 戻り値
    /// 最短距離
    fn distance_to(&self, other: &Other) -> T;
}

/// 点との特化距離計算
///
/// 幾何オブジェクトと点の間の距離・包含関係に特化したトレイト。
/// 点は最も基本的な幾何要素のため専用トレイトで効率化。
pub trait PointDistance<T: Scalar> {
    /// 2D 点型
    type Point2D;

    /// 点までの距離
    ///
    /// # 引数
    /// * `point` - 距離計算対象の点
    ///
    /// # 戻り値
    /// 点までの最短距離
    fn distance_to_point(&self, point: &Self::Point2D) -> T;

    /// 点が内部にあるか
    ///
    /// # 引数
    /// * `point` - 判定対象の点
    /// * `tolerance` - 判定精度の許容誤差
    ///
    /// # 戻り値
    /// 点が内部にある場合は `true`
    fn contains_point(&self, point: &Self::Point2D, tolerance: T) -> bool;

    /// 点が境界上にあるか
    ///
    /// # 引数
    /// * `point` - 判定対象の点
    /// * `tolerance` - 判定精度の許容誤差
    ///
    /// # 戻り値
    /// 点が境界上にある場合は `true`
    fn point_on_boundary(&self, point: &Self::Point2D, tolerance: T) -> bool;

    /// 最近点を取得
    ///
    /// オブジェクト上で指定点に最も近い点を取得。
    ///
    /// # 引数
    /// * `point` - 基準となる点
    ///
    /// # 戻り値
    /// オブジェクト上の最近点
    fn closest_point(&self, point: &Self::Point2D) -> Self::Point2D;
}

/// BBox による高速事前スクリーニング
///
/// 詳細な衝突検出の前に境界ボックスでの事前判定を行い、
/// 計算効率を向上させるためのトレイト。
pub trait BBoxCollision<T: Scalar> {
    /// 境界ボックス型
    type BBox;

    /// 境界ボックスを取得
    fn bounding_box(&self) -> Self::BBox;

    /// 境界ボックス同士の衝突判定
    ///
    /// # 引数
    /// * `other_bbox` - 相手の境界ボックス
    ///
    /// # 戻り値
    /// 境界ボックスが重なっている場合は `true`
    fn bbox_intersects(&self, other_bbox: &Self::BBox) -> bool;
}
