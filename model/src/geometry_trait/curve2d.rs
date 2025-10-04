use std::any::Any;
use crate::geometry_kind::CurveKind2D;
use geo_core::{Point2D as Point, Vector2D as Vector};

/// Curve2D: 2次元曲線の抽象トレイト
///
/// 各曲線型（Line, Arc, Ellipse など）が共通で実装するためのインターフェース。
/// CurveKind2D による分類と、評価・微分・長さ取得などの基本操作を提供する。
pub trait Curve2D: Any {
    /// 型判定のためのダウンキャスト
    fn as_any(&self) -> &dyn Any;

    /// 曲線の分類を返す
    fn kind(&self) -> CurveKind2D;

    /// パラメータ t に対応する点を返す（通常 t ∈ [0, 1]）
    fn evaluate(&self, t: f64) -> Point;

    /// パラメータ t における接線方向（1階微分）を返す
    fn derivative(&self, t: f64) -> Vector;

    /// 曲線の長さ（t ∈ [0, 1] 区間における定義長）
    fn length(&self) -> f64;

    /// 指定点に対するパラメータ初期推定（数値解析用）
    fn parameter_hint(&self, pt: &Point) -> f64 {
        // デフォルト実装は中心方向など、構造体ごとにオーバーライド
        0.5
    }

    /// 有効なパラメータ範囲（通常 [0, 1]）
    fn domain(&self) -> (f64, f64) {
        (0.0, 1.0)
    }
}
