//! CylindricalSolid3D 形状の契約定義
//!
//! geo_primitives が実装すべき円柱（立体）の公開 trait。

use crate::{Direction3D, Point3D, PointClassification, Scalar};

pub trait CylindricalSolid3DConstructor<T: Scalar>: Sized {
    /// 底面の中心・軸・参照方向・半径・高さから作成する（上面は底面の中心から軸方向に高さだけ離れた位置）
    ///
    /// 参照方向は軸の成分を除いて正規化する。軸と平行な場合・半径または高さが正でない場合は `None`。
    fn new(
        center: Point3D<T>,
        axis: Direction3D<T>,
        ref_direction: Direction3D<T>,
        radius: T,
        height: T,
    ) -> Option<Self>;

    /// 軸を Z 軸正方向、参照方向を X 軸正方向として作成する（半径または高さが正でない場合は `None`）
    fn new_standard(center: Point3D<T>, radius: T, height: T) -> Option<Self>;
}

pub trait CylindricalSolid3DProperties<T: Scalar> {
    /// 底面の中心
    fn center(&self) -> Point3D<T>;
    fn radius(&self) -> T;
    fn height(&self) -> T;
    /// 底面から上面へ向かう軸の方向
    fn axis(&self) -> Direction3D<T>;
    fn ref_direction(&self) -> Direction3D<T>;
}

pub trait CylindricalSolid3DContainment<T: Scalar> {
    /// 点が立体の内部または表面上にあるか（表面を含む厳密な判定）
    fn contains_point(&self, point: Point3D<T>) -> bool;
    /// 領域に対する点の位置を分類する（既定の距離トレランス）
    fn classify_point(&self, point: Point3D<T>) -> PointClassification;
}

pub trait CylindricalSolid3DDistance<T: Scalar> {
    /// 立体（領域）までの距離（内部・表面上は 0）
    fn distance_to_point(&self, point: Point3D<T>) -> T;
}

pub trait CylindricalSolid3DProjection<T: Scalar> {
    /// 立体（領域）上で点に最も近い点（内部・表面上の点はその点）
    fn closest_point(&self, point: Point3D<T>) -> Point3D<T>;
}
