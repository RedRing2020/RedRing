//! Circle 形状の契約定義
//!
//! geo_primitives が実装すべき Circle 系の公開 trait。

use crate::{Direction3D, Point2D, Point3D, PointClassification, Scalar};

pub trait Circle2DConstructor<T: Scalar> {
    /// 中心と半径から作成する（参照方向は X 軸正方向。半径が正でない場合は `None`）
    fn new(center: Point2D<T>, radius: T) -> Option<Self>
    where
        Self: Sized;

    /// 3点を通る円を作成する（3点が一直線上にある場合は `None`）
    fn from_three_points(p1: Point2D<T>, p2: Point2D<T>, p3: Point2D<T>) -> Option<Self>
    where
        Self: Sized;
}

pub trait Circle3DConstructor<T: Scalar> {
    /// 中心・平面の法線・半径から作成する（半径が正でない場合は `None`）
    fn new(center: Point3D<T>, axis: Direction3D<T>, radius: T) -> Option<Self>
    where
        Self: Sized;

    /// XY 平面上の円を作成する（法線は Z 軸正方向）
    fn new_xy_plane(center: Point3D<T>, radius: T) -> Option<Self>
    where
        Self: Sized;

    /// 3点を通る円を作成する（3点が一直線上にある場合は `None`）
    fn from_three_points(p1: Point3D<T>, p2: Point3D<T>, p3: Point3D<T>) -> Option<Self>
    where
        Self: Sized;
}

pub trait Circle2DProperties<T: Scalar> {
    fn center(&self) -> Point2D<T>;
    fn radius(&self) -> T;
}

pub trait Circle3DProperties<T: Scalar> {
    fn center(&self) -> Point3D<T>;
    fn radius(&self) -> T;
    /// 円が存在する平面の法線
    fn axis(&self) -> Direction3D<T>;
    /// 角度 0 の方向
    fn ref_direction(&self) -> Direction3D<T>;
}

pub trait Circle2DEvaluation<T: Scalar> {
    /// Primitive 局所座標系の local angle parameter `t` (`0 <= t <= 2π`) に対応する円周上の点を返す
    fn point_at_parameter(&self, t: T) -> Point2D<T>;
}

pub trait Circle2DContainment<T: Scalar> {
    /// 点が円周上にあるか（既定の距離トレランス）
    fn contains_point(&self, point: Point2D<T>) -> bool;

    /// 円が囲む領域に対する点の位置を分類する（既定の距離トレランス）
    fn classify_point(&self, point: Point2D<T>) -> PointClassification;
}

pub trait Circle2DDistance<T: Scalar> {
    fn distance_to_point(&self, point: Point2D<T>) -> T;
}

pub trait Circle2DProjection<T: Scalar> {
    /// 点に最も近い曲線上の点を返す
    fn closest_point(&self, point: Point2D<T>) -> Point2D<T>;
}

pub trait Circle3DEvaluation<T: Scalar> {
    /// Primitive 局所座標系の local angle parameter `t` (`0 <= t <= 2π`) に対応する円周上の点を返す
    fn point_at_parameter(&self, t: T) -> Point3D<T>;
}

pub trait Circle3DContainment<T: Scalar> {
    /// 点が円周上にあるか（既定の距離トレランス）
    fn contains_point(&self, point: Point3D<T>) -> bool;

    /// 円が囲む領域に対する点の位置を分類する（既定の距離トレランス）
    fn classify_point(&self, point: Point3D<T>) -> PointClassification;
}

pub trait Circle3DDistance<T: Scalar> {
    fn distance_to_point(&self, point: Point3D<T>) -> T;
}

pub trait Circle3DProjection<T: Scalar> {
    /// 点に最も近い曲線上の点を返す
    fn closest_point(&self, point: Point3D<T>) -> Point3D<T>;
}
