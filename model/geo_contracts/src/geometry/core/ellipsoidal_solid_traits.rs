//! EllipsoidalSolid3D 形状の契約定義
//!
//! geo_primitives が実装すべき楕円体（立体）の公開 trait。

use crate::{Direction3D, Point3D, PointClassification, Scalar};

pub trait EllipsoidalSolid3DConstructor<T: Scalar>: Sized {
    /// 中心・軸・参照方向・3つの半径から作成する（STEP AXIS2_PLACEMENT_3D 形式）
    ///
    /// `a_radius` は参照方向、`b_radius` は軸と参照方向に直交する方向、`c_radius` は軸方向の半径。
    /// 参照方向は軸の成分を除いて正規化する。軸と平行な場合・半径が正でない場合は `None`。
    fn new(
        center: Point3D<T>,
        axis: Direction3D<T>,
        ref_direction: Direction3D<T>,
        a_radius: T,
        b_radius: T,
        c_radius: T,
    ) -> Option<Self>;

    /// 軸を Z 軸正方向、参照方向を X 軸正方向として作成する（半径が正でない場合は `None`）
    fn new_standard(center: Point3D<T>, a_radius: T, b_radius: T, c_radius: T) -> Option<Self>;
}

pub trait EllipsoidalSolid3DProperties<T: Scalar> {
    fn center(&self) -> Point3D<T>;
    fn axis(&self) -> Direction3D<T>;
    fn ref_direction(&self) -> Direction3D<T>;
    /// 参照方向の半径
    fn a_radius(&self) -> T;
    /// 軸と参照方向に直交する方向の半径
    fn b_radius(&self) -> T;
    /// 軸方向の半径
    fn c_radius(&self) -> T;
}

pub trait EllipsoidalSolid3DContainment<T: Scalar> {
    /// 点が立体の内部または表面上にあるか（表面を含む厳密な判定）
    fn contains_point(&self, point: Point3D<T>) -> bool;
    /// 領域に対する点の位置を分類する（既定の距離トレランス）
    fn classify_point(&self, point: Point3D<T>) -> PointClassification;
}

pub trait EllipsoidalSolid3DDistance<T: Scalar> {
    /// 立体（領域）までの距離（内部・表面上は 0）
    fn distance_to_point(&self, point: Point3D<T>) -> T;
}

pub trait EllipsoidalSolid3DProjection<T: Scalar> {
    /// 立体（領域）上で点に最も近い点（内部・表面上の点はその点）
    fn closest_point(&self, point: Point3D<T>) -> Point3D<T>;
}
