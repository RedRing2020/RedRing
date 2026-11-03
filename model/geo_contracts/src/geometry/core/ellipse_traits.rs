//! Ellipse 形状の契約定義
//!
//! Ellipse の trait定義を capability taxonomy に沿って分離する。

use crate::{Angle, Direction3D, Point2D, Point3D, PointClassification, Scalar};

/// Ellipse2D の生成 trait
pub trait Ellipse2DConstructor<T: Scalar>: Sized {
    /// 中心・長半軸・短半軸・回転角（X 軸から長軸までの角度）から楕円を作成する
    ///
    /// 半軸が正でない場合と、長半軸が短半軸より短い場合は `None`。
    fn new(center: Point2D<T>, semi_major: T, semi_minor: T, rotation: Angle<T>) -> Option<Self>;
}

/// Ellipse2D の定義パラメータ
pub trait Ellipse2DProperties<T: Scalar> {
    /// 中心座標を取得
    fn center(&self) -> Point2D<T>;

    /// 長半軸の長さを取得
    fn semi_major_axis(&self) -> T;

    /// 短半軸の長さを取得
    fn semi_minor_axis(&self) -> T;

    /// 回転角（X 軸から長軸までの角度）を取得
    fn rotation(&self) -> Angle<T>;
}

/// Ellipse2D の評価
pub trait Ellipse2DEvaluation<T: Scalar> {
    /// Primitive 局所座標系の周期 angle parameter `t` (`0 <= t < 2π`) に対応する evaluation point を返す
    fn point_at_parameter(&self, t: T) -> Point2D<T>;
}

/// Ellipse2D の包含判定
pub trait Ellipse2DContainment<T: Scalar> {
    /// 点が楕円上にあるか（既定の距離トレランス）
    fn contains_point(&self, point: Point2D<T>) -> bool;

    /// 楕円が囲む領域に対する点の位置を分類する（既定の距離トレランス）
    fn classify_point(&self, point: Point2D<T>) -> PointClassification;
}

/// Ellipse2D の距離計算
pub trait Ellipse2DDistance<T: Scalar> {
    fn distance_to_point(&self, point: Point2D<T>) -> T;
}

pub trait Ellipse2DProjection<T: Scalar> {
    /// 点に最も近い曲線上の点を返す
    fn closest_point(&self, point: Point2D<T>) -> Point2D<T>;
}

/// Ellipse3D の生成 trait
pub trait Ellipse3DConstructor<T: Scalar>: Sized {
    /// 中心・長半軸・短半軸・平面の法線・長軸方向から楕円を作成する
    ///
    /// 半軸が正でない場合、長半軸が短半軸より短い場合、長軸方向が法線に直交しない場合は `None`。
    fn new(
        center: Point3D<T>,
        semi_major_axis: T,
        semi_minor_axis: T,
        normal: Direction3D<T>,
        major_axis_direction: Direction3D<T>,
    ) -> Option<Self>;

    /// XY 平面上の楕円を作成する（回転角は X 軸から長軸までの角度）
    fn new_xy_plane(
        center: Point3D<T>,
        semi_major_axis: T,
        semi_minor_axis: T,
        rotation: Angle<T>,
    ) -> Option<Self>;
}

/// Ellipse3D の定義パラメータ
pub trait Ellipse3DProperties<T: Scalar> {
    /// 中心座標を取得
    fn center(&self) -> Point3D<T>;

    /// 楕円平面の法線を取得
    fn normal(&self) -> Direction3D<T>;

    /// 長軸方向を取得
    fn major_axis_direction(&self) -> Direction3D<T>;

    /// 短軸方向（法線 × 長軸方向）を取得
    fn minor_axis_direction(&self) -> Direction3D<T>;

    /// 長半軸の長さを取得
    fn semi_major_axis(&self) -> T;

    /// 短半軸の長さを取得
    fn semi_minor_axis(&self) -> T;
}

/// Ellipse3D の評価
pub trait Ellipse3DEvaluation<T: Scalar> {
    /// Primitive 局所座標系の周期 angle parameter `t` (`0 <= t < 2π`) に対応する evaluation point を返す
    fn point_at_parameter(&self, t: T) -> Point3D<T>;
}

/// Ellipse3D の包含判定
pub trait Ellipse3DContainment<T: Scalar> {
    /// 点が楕円上にあるか（既定の距離トレランス）
    fn contains_point(&self, point: Point3D<T>) -> bool;

    /// 楕円が囲む領域に対する点の位置を分類する（既定の距離トレランス）
    fn classify_point(&self, point: Point3D<T>) -> PointClassification;
}

/// Ellipse3D の距離計算
pub trait Ellipse3DDistance<T: Scalar> {
    fn distance_to_point(&self, point: Point3D<T>) -> T;
}

/// Ellipse3D の射影
pub trait Ellipse3DProjection<T: Scalar> {
    /// 点に最も近い曲線上の点を返す
    fn closest_point(&self, point: Point3D<T>) -> Point3D<T>;
}
