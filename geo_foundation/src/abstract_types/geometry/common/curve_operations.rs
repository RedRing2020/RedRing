//! 曲線の共通操作トレイト
//!
//! 曲線型全般に適用される基本的な操作（点取得、長さ計算等）

use crate::abstract_types::Scalar;

/// 曲線の共通型定義
pub trait CurveTypes<T: Scalar> {
    /// 点の型
    type Point;
    /// ベクトルの型
    type Vector;
    /// 境界ボックスの型
    type BoundingBox;
}

/// 曲線の基本的な点取得操作
///
/// 開始点、終了点、中点等の共通操作を定義
pub trait CurvePoints<T: Scalar>: CurveTypes<T> {
    /// 曲線の開始点を取得
    fn start_point(&self) -> Self::Point;

    /// 曲線の終了点を取得
    fn end_point(&self) -> Self::Point;

    /// 曲線の中点を取得（パラメータt=0.5での点）
    fn midpoint(&self) -> Self::Point;

    /// 指定されたパラメータ位置での点を取得（t: 0.0〜1.0）
    fn point_at_parameter(&self, t: T) -> Self::Point;

    /// 曲線を等分割した点列を取得
    fn sample_points(&self, num_points: usize) -> Vec<Self::Point> {
        if num_points < 2 {
            return vec![self.start_point()];
        }

        let mut points = Vec::with_capacity(num_points);
        for i in 0..num_points {
            let t = T::from_f64(i as f64 / (num_points - 1) as f64);
            points.push(self.point_at_parameter(t));
        }
        points
    }
}

/// 曲線の計量操作（長さ、距離等）
pub trait CurveMetrics<T: Scalar>: CurveTypes<T> {
    /// 曲線の全長を取得
    fn length(&self) -> T;

    /// 曲線の開始から指定パラメータまでの弧長
    fn arc_length_to_parameter(&self, t: T) -> T;

    /// 指定された弧長に対応するパラメータを取得（弧長パラメータ化）
    fn parameter_at_arc_length(&self, length: T) -> T;

    /// 曲線が閉じているかどうかを判定
    fn is_closed(&self) -> bool;
}

/// 曲線の包含関係操作
pub trait CurveContainment<T: Scalar>: CurveTypes<T> {
    /// 指定された点が曲線上にあるかを判定
    fn contains_point(&self, point: &Self::Point) -> bool;

    /// 指定された点が曲線上にあるかを許容誤差付きで判定
    fn contains_point_with_tolerance(&self, point: &Self::Point, tolerance: T) -> bool;

    /// 曲線の境界ボックスを取得
    fn bounding_box(&self) -> Self::BoundingBox;

    /// 指定された点から曲線への最短距離を計算
    fn distance_to_point(&self, point: &Self::Point) -> T;

    /// 指定された点に最も近い曲線上の点を取得
    fn closest_point(&self, point: &Self::Point) -> Self::Point;
}

/// 曲線の変換操作
pub trait CurveTransformation<T: Scalar>: CurveTypes<T> {
    /// 変換後の曲線の型（通常は Self）
    type TransformedCurve;

    /// 曲線を反転（開始点と終了点を入れ替え）
    fn reverse(&self) -> Self::TransformedCurve;

    /// 曲線をスケール変換
    fn scale(&self, factor: T) -> Self::TransformedCurve;

    /// 曲線を平行移動（ベクトル指定）
    fn translate(&self, offset: &Self::Vector) -> Self::TransformedCurve;

    /// 曲線を指定した中心点を基準にスケール
    fn scale_about_point(&self, center: &Self::Point, factor: T) -> Self::TransformedCurve
    where
        Self: CurvePoints<T>;
}

/// 角度ベースの曲線操作（円弧、楕円弧等用）
pub trait AngularCurve<T: Scalar>: CurveTypes<T> {
    /// 角度の型
    type Angle;

    /// 開始角度を取得
    fn start_angle(&self) -> Self::Angle;

    /// 終了角度を取得
    fn end_angle(&self) -> Self::Angle;

    /// 角度範囲を取得（終了角度 - 開始角度）
    fn angle_span(&self) -> Self::Angle;

    /// 指定角度での点を取得
    fn point_at_angle(&self, angle: Self::Angle) -> Self::Point;

    /// 指定された角度が曲線の角度範囲内にあるかを判定
    fn contains_angle(&self, angle: Self::Angle) -> bool;

    /// 角度を正規化（0〜2πまたは-π〜π等）
    fn normalize_angle(&self, angle: Self::Angle) -> Self::Angle;
}

/// 中心を持つ曲線操作（円、楕円等用）
pub trait CenteredCurve<T: Scalar>: CurveTypes<T> {
    /// 曲線の中心点を取得
    fn center(&self) -> Self::Point;

    /// 中心から指定点への距離を計算
    fn distance_from_center(&self, point: &Self::Point) -> T;

    /// 中心を基準とした回転
    fn rotate_about_center(&self, angle: T) -> Self
    where
        Self: Clone;
}

/// 半径を持つ曲線操作（円、円弧等用）
pub trait RadialCurve<T: Scalar>: CurveTypes<T> {
    /// 曲線の半径を取得
    fn radius(&self) -> T;

    /// 半径を変更した新しい曲線を作成
    fn with_radius(&self, new_radius: T) -> Self
    where
        Self: Clone;

    /// 指定点が曲線の半径内にあるかを判定
    fn point_within_radius(&self, point: &Self::Point) -> bool
    where
        Self: CenteredCurve<T>;
}
