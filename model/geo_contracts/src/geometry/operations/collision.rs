//! 点との距離・境界ボックスの trait定義
//!
//! 形状をまたぐ距離・衝突判定・交差の trait定義は `geo_algorithms::operations` に置く。

use analysis::abstract_types::Scalar;

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
}
