//! 衝突検出・距離計算の統一インターフェース
//!
//! 全幾何プリミティブで共通利用可能な衝突検出Foundation システム
//! メンテナンス効率向上のため、統一インターフェースを提供

use crate::Scalar;

/// 基本衝突検出インターフェース
///
/// 2つの幾何オブジェクト間の基本的な衝突検出・距離計算を提供
/// 全ての幾何プリミティブ間の組み合わせで実装される
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
    /// 衝突している場合は true
    fn intersects(&self, other: &Other, tolerance: T) -> bool;

    /// 重なりを持つかどうか
    ///
    /// 完全に内包している場合や部分的に重なっている場合を検出
    ///
    /// # 引数
    /// * `other` - 重なり判定を行う相手オブジェクト
    /// * `tolerance` - 判定精度の許容誤差
    ///
    /// # 戻り値
    /// 重なりがある場合は true
    fn overlaps(&self, other: &Other, tolerance: T) -> bool;

    /// 最短距離
    ///
    /// 2つのオブジェクト間の最短距離を計算
    /// 重なりがある場合は0を返す
    ///
    /// # 引数
    /// * `other` - 距離計算を行う相手オブジェクト
    ///
    /// # 戻り値
    /// 最短距離
    fn distance_to(&self, other: &Other) -> T;
}

/// 高度衝突検出インターフェース
///
/// より詳細な衝突情報や高度な衝突検出機能を提供
/// 最近点対、重なり測定、分離軸判定等を含む
pub trait AdvancedCollision<T: Scalar, Other>: BasicCollision<T, Other> {
    /// 最近点対型
    type PointPair;

    /// 2D ベクトル型
    type Vector2D;

    /// 最近点対を取得
    ///
    /// 2つのオブジェクト間で最も近い点の組み合わせを取得
    ///
    /// # 引数
    /// * `other` - 最近点計算を行う相手オブジェクト
    ///
    /// # 戻り値
    /// 最近点対（self側の点, other側の点）
    fn closest_points(&self, other: &Other) -> Self::PointPair;

    /// 重なり測定値
    ///
    /// 2つのオブジェクトの重なり部分の面積または長さを計算
    /// 重なりがない場合は None を返す
    ///
    /// # 引数
    /// * `other` - 重なり測定を行う相手オブジェクト
    ///
    /// # 戻り値
    /// 重なり測定値（面積、長さ、体積等）、重なりがない場合は None
    fn overlap_measure(&self, other: &Other) -> Option<T>;

    /// 分離軸判定（Separating Axis Theorem）
    ///
    /// 指定された軸での射影を比較して分離しているかを判定
    /// 凸形状の衝突検出で使用される高速アルゴリズム
    ///
    /// # 引数
    /// * `other` - 分離判定を行う相手オブジェクト
    /// * `axis` - 分離軸となるベクトル
    ///
    /// # 戻り値
    /// 指定軸で分離している場合は true
    fn separated_by_axis(&self, other: &Other, axis: Self::Vector2D) -> bool;

    /// 包含関係判定
    ///
    /// 一方のオブジェクトが他方を完全に包含しているかを判定
    ///
    /// # 引数
    /// * `other` - 包含判定を行う相手オブジェクト
    /// * `tolerance` - 判定精度の許容誤差
    ///
    /// # 戻り値
    /// (self が other を包含, other が self を包含)
    fn containment_relation(&self, other: &Other, tolerance: T) -> (bool, bool);
}

/// 点との特化距離計算
///
/// 幾何オブジェクトと点の間の距離・包含関係に特化したトレイト
/// 点は最も基本的な幾何要素のため専用トレイトで効率化
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
    /// 点が内部にある場合は true
    fn contains_point(&self, point: &Self::Point2D, tolerance: T) -> bool;

    /// 点が境界上にあるか
    ///
    /// # 引数
    /// * `point` - 判定対象の点
    /// * `tolerance` - 判定精度の許容誤差
    ///
    /// # 戻り値
    /// 点が境界上にある場合は true
    fn point_on_boundary(&self, point: &Self::Point2D, tolerance: T) -> bool;

    /// 最近点を取得
    ///
    /// オブジェクト上で指定点に最も近い点を取得
    ///
    /// # 引数
    /// * `point` - 基準となる点
    ///
    /// # 戻り値
    /// オブジェクト上の最近点
    fn closest_point(&self, point: &Self::Point2D) -> Self::Point2D;
}

/// 便利メソッドを提供するヘルパートレイト
///
/// デフォルト tolerance での操作や、よく使用される組み合わせ操作を提供
/// `BasicCollision` を実装した型に対して自動的に実装される
pub trait CollisionHelpers<T: Scalar, Other>: BasicCollision<T, Other> {
    /// デフォルト tolerance での衝突判定
    ///
    /// # 引数
    /// * `other` - 衝突判定を行う相手オブジェクト
    ///
    /// # 戻り値
    /// 衝突している場合は true
    fn intersects_default(&self, other: &Other) -> bool {
        self.intersects(other, T::EPSILON)
    }

    /// デフォルト tolerance での重なり判定
    ///
    /// # 引数
    /// * `other` - 重なり判定を行う相手オブジェクト
    ///
    /// # 戻り値
    /// 重なりがある場合は true
    fn overlaps_default(&self, other: &Other) -> bool {
        self.overlaps(other, T::EPSILON)
    }

    /// 接触判定（重なりまたは境界接触）
    ///
    /// # 引数
    /// * `other` - 接触判定を行う相手オブジェクト
    /// * `tolerance` - 判定精度の許容誤差
    ///
    /// # 戻り値
    /// 接触している場合は true
    fn touches(&self, other: &Other, tolerance: T) -> bool {
        self.distance_to(other) <= tolerance
    }

    /// デフォルト tolerance での接触判定
    ///
    /// # 引数
    /// * `other` - 接触判定を行う相手オブジェクト
    ///
    /// # 戻り値
    /// 接触している場合は true
    fn touches_default(&self, other: &Other) -> bool {
        self.touches(other, T::EPSILON)
    }

    /// 分離判定（接触していない）
    ///
    /// # 引数
    /// * `other` - 分離判定を行う相手オブジェクト
    /// * `tolerance` - 判定精度の許容誤差
    ///
    /// # 戻り値
    /// 分離している場合は true
    fn separated(&self, other: &Other, tolerance: T) -> bool {
        !self.touches(other, tolerance)
    }
}

// CollisionHelpers の自動実装
// BasicCollision を実装している型に対して自動的に便利メソッドを提供
impl<T: Scalar, Other, U> CollisionHelpers<T, Other> for U where U: BasicCollision<T, Other> {}

/// 点との距離計算用ヘルパートレイト
///
/// `PointDistance` を実装した型に対する便利メソッドを提供
pub trait PointDistanceHelpers<T: Scalar>: PointDistance<T> {
    /// デフォルト tolerance での包含判定
    ///
    /// # 引数
    /// * `point` - 判定対象の点
    ///
    /// # 戻り値
    /// 点が内部にある場合は true
    fn contains_point_default(&self, point: &Self::Point2D) -> bool {
        self.contains_point(point, T::EPSILON)
    }

    /// デフォルト tolerance での境界判定
    ///
    /// # 引数
    /// * `point` - 判定対象の点
    ///
    /// # 戻り値
    /// 点が境界上にある場合は true
    fn point_on_boundary_default(&self, point: &Self::Point2D) -> bool {
        self.point_on_boundary(point, T::EPSILON)
    }

    /// 点が外部にあるか
    ///
    /// # 引数
    /// * `point` - 判定対象の点
    /// * `tolerance` - 判定精度の許容誤差
    ///
    /// # 戻り値
    /// 点が外部にある場合は true
    fn point_outside(&self, point: &Self::Point2D, tolerance: T) -> bool {
        !self.contains_point(point, tolerance) && !self.point_on_boundary(point, tolerance)
    }
}

// PointDistanceHelpers の自動実装
impl<T: Scalar, U> PointDistanceHelpers<T> for U where U: PointDistance<T> {}

/// Bounding Box による高速事前スクリーニング
///
/// 詳細な衝突検出の前に境界ボックスでの事前判定を行い、
/// 計算効率を向上させるためのトレイト
pub trait BoundingBoxCollision<T: Scalar> {
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
    /// 境界ボックスが重なっている場合は true
    fn bbox_intersects(&self, other_bbox: &Self::BBox) -> bool;
}
