//! Ellipse Core Traits - Ellipse形状のCore機能統合
//!
//! Ellipse の trait定義を capability taxonomy に沿って分離する。

use crate::PointClassification;
use analysis::abstract_types::Scalar;

/// Ellipse2D の生成 trait
pub trait Ellipse2DConstructor<T: Scalar>: Sized {
    /// 中心、長軸、短軸、回転角から楕円を作成
    fn new(center: (T, T), semi_major: T, semi_minor: T, rotation: T) -> Option<Self>;
}

/// Ellipse2D の定義パラメータ
pub trait Ellipse2DProperties<T: Scalar> {
    /// 中心座標を取得
    fn center(&self) -> (T, T);

    /// 長半軸の長さを取得
    fn semi_major_axis(&self) -> T;

    /// 短半軸の長さを取得
    fn semi_minor_axis(&self) -> T;

    /// 回転角を取得（ラジアン）
    fn rotation(&self) -> T;
}

/// Ellipse2D の評価
pub trait Ellipse2DEvaluation<T: Scalar> {
    /// Primitive 局所座標系の周期 angle parameter `t` (`0 <= t < 2π`) に対応する evaluation point を返す
    fn point_at_parameter(&self, t: T) -> (T, T);
}

/// Ellipse2D の包含判定
pub trait Ellipse2DContainment<T: Scalar> {
    /// 点が楕円上にあるか（既定の距離トレランス）
    fn contains_point(&self, point: (T, T)) -> bool;

    /// 楕円が囲む領域に対する点の位置を分類する（既定の距離トレランス）
    fn classify_point(&self, point: (T, T)) -> PointClassification;
}

/// Ellipse2D の距離計算
pub trait Ellipse2DDistance<T: Scalar> {
    fn distance_to_point(&self, point: (T, T)) -> T;
}

pub trait Ellipse2DProjection<T: Scalar> {
    /// 点に最も近い曲線上の点を返す
    fn closest_point(&self, point: (T, T)) -> (T, T);
}

/// Ellipse3D の生成 trait
pub trait Ellipse3DConstructor<T: Scalar>: Sized {
    /// 中心、法線、長軸、短軸、長軸方向から楕円を作成
    fn new(
        center: (T, T, T),
        normal: (T, T, T),
        semi_major_axis: T,
        semi_minor_axis: T,
        major_axis_direction: (T, T, T),
    ) -> Option<Self>;

    /// XY平面上の楕円を作成
    fn new_xy_plane(
        center: (T, T, T),
        semi_major_axis: T,
        semi_minor_axis: T,
        rotation: T,
    ) -> Option<Self>;
}

/// Ellipse3D の定義パラメータ
pub trait Ellipse3DProperties<T: Scalar> {
    /// 楕円平面の法線ベクトルを取得
    fn normal(&self) -> (T, T, T);

    /// 長軸方向ベクトルを取得
    fn major_axis_direction(&self) -> (T, T, T);

    /// 短軸方向ベクトルを取得
    fn minor_axis_direction(&self) -> (T, T, T);

    /// 長半軸の長さを取得
    fn semi_major_axis(&self) -> T;

    /// 短半軸の長さを取得
    fn semi_minor_axis(&self) -> T;
}

/// Ellipse3D の評価
pub trait Ellipse3DEvaluation<T: Scalar> {
    /// Primitive 局所座標系の周期 angle parameter `t` (`0 <= t < 2π`) に対応する evaluation point を返す
    fn point_at_parameter(&self, t: T) -> (T, T, T);
}

/// Ellipse3D の包含判定
pub trait Ellipse3DContainment<T: Scalar> {
    /// 点が楕円上にあるか（既定の距離トレランス）
    fn contains_point(&self, point: (T, T, T)) -> bool;

    /// 楕円が囲む領域に対する点の位置を分類する（既定の距離トレランス）
    fn classify_point(&self, point: (T, T, T)) -> PointClassification;
}

/// Ellipse3D の距離計算
pub trait Ellipse3DDistance<T: Scalar> {
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}

/// Ellipse3D の射影
pub trait Ellipse3DProjection<T: Scalar> {
    /// 点に最も近い曲線上の点を返す
    fn closest_point(&self, point: (T, T, T)) -> (T, T, T);
}
