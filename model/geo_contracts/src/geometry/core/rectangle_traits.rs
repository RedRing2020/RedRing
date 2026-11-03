//! Rectangle 形状の契約定義
//!
//! geo_primitives が実装すべき Rect2D / Rect3D の公開 trait。

use crate::{Direction3D, Point2D, Point3D, PointClassification, Scalar};

pub trait Rect2DConstructor<T: Scalar>: Sized {
    /// 原点（最小の角）・幅・高さから軸平行の矩形を作成する（幅・高さが負の場合は `None`）
    fn new(origin: Point2D<T>, width: T, height: T) -> Option<Self>;
}

pub trait Rect2DProperties<T: Scalar> {
    fn origin(&self) -> Point2D<T>;
    fn width(&self) -> T;
    fn height(&self) -> T;
    fn center(&self) -> Point2D<T>;
}

pub trait Rect2DContainment<T: Scalar> {
    /// 点が矩形の内部または境界上にあるか（境界を含む厳密な判定）
    fn contains_point(&self, point: Point2D<T>) -> bool;
    /// 領域に対する点の位置を分類する（既定の距離トレランス）
    fn classify_point(&self, point: Point2D<T>) -> PointClassification;
}

pub trait Rect3DConstructor<T: Scalar>: Sized {
    /// 原点・U 軸方向・V 軸方向・幅・高さから矩形を作成する
    ///
    /// V 軸方向は U 軸に直交するよう補正する。幅・高さが負の場合と、U 軸と V 軸が平行な場合は `None`。
    fn new(
        origin: Point3D<T>,
        u_axis: Direction3D<T>,
        v_axis: Direction3D<T>,
        width: T,
        height: T,
    ) -> Option<Self>;
}

pub trait Rect3DProperties<T: Scalar> {
    fn origin(&self) -> Point3D<T>;
    fn u_axis(&self) -> Direction3D<T>;
    fn v_axis(&self) -> Direction3D<T>;
    /// U 軸 × V 軸
    fn normal(&self) -> Direction3D<T>;
    fn width(&self) -> T;
    fn height(&self) -> T;
    fn center(&self) -> Point3D<T>;
}

pub trait Rect3DContainment<T: Scalar> {
    /// 既定の距離の許容誤差で、点が矩形の内部または境界上にあるか（`classify_point` が `Outside` でない）
    fn contains_point(&self, point: Point3D<T>) -> bool;

    /// 矩形の領域に対する点の位置を分類する（既定の距離トレランス）
    fn classify_point(&self, point: Point3D<T>) -> PointClassification;
}
