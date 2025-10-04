use std::any::Any;
use crate::geometry_kind::CurveKind3D;

/// NOTE: 旧 model::geometry 依存を除去。Point / Vector は関連型化。

/// Curve3D: 3次元曲線の抽象トレイト
///
/// 各曲線型（Line, Ellipse, Nurbs など）が共通で実装するためのインターフェース。
/// CurveKind3D による分類と、評価・微分・長さ取得などの基本操作を提供する。
pub trait Curve3D: Any {
    type Point;
    type Vector;
    /// 型判定のためのダウンキャスト
    fn as_any(&self) -> &dyn Any;

    /// 曲線の分類を返す
    fn kind(&self) -> CurveKind3D;

    /// パラメータ t に対応する点を返す（通常 t ∈ [0, 1]）
    fn evaluate(&self, t: f64) -> Self::Point;

    /// パラメータ t における接線ベクトル（1階微分）を返す
    fn derivative(&self, t: f64) -> Self::Vector;

    /// 曲線の長さ（t ∈ [0, 1] 区間における定義長）
    fn length(&self) -> f64;

    /// 指定点に対するパラメータ初期推定（数値解析用）
    fn parameter_hint(&self, _pt: &Self::Point) -> f64 {
        0.5
    }

    /// 有効なパラメータ範囲（通常 [0, 1]）
    fn domain(&self) -> (f64, f64) {
        (0.0, 1.0)
    }
}
