//! TorusSurface3D 形状の契約定義
//!
//! geo_primitives が実装すべきトーラス面の公開 trait。

use crate::{Direction3D, Point3D, Scalar};

pub trait TorusSurface3DConstructor<T: Scalar>: Sized {
    /// 中心・軸・参照方向・大半径・小半径から作成する（STEP AXIS2_PLACEMENT_3D 形式）
    ///
    /// 参照方向は軸の成分を除いて正規化する。軸と平行な場合・半径が正でない場合は `None`。
    fn new(
        center: Point3D<T>,
        axis: Direction3D<T>,
        ref_direction: Direction3D<T>,
        major_radius: T,
        minor_radius: T,
    ) -> Option<Self>;

    /// 軸を Z 軸正方向、参照方向を X 軸正方向として作成する（半径が正でない場合は `None`）
    fn new_standard(center: Point3D<T>, major_radius: T, minor_radius: T) -> Option<Self>;
}

pub trait TorusSurface3DProperties<T: Scalar> {
    fn center(&self) -> Point3D<T>;
    fn axis(&self) -> Direction3D<T>;
    fn ref_direction(&self) -> Direction3D<T>;
    fn major_radius(&self) -> T;
    fn minor_radius(&self) -> T;
}

pub trait TorusSurface3DEvaluation<T: Scalar> {
    /// 軸回りの角度 `u` (`0 <= u <= 2π`)・断面の円の角度 `v` (`0 <= v <= 2π`) に対応するトーラス面上の点を返す
    fn point_at_uv(&self, u: T, v: T) -> Point3D<T>;
    /// `u`・`v` での外向きの法線
    fn normal_at(&self, u: T, v: T) -> Direction3D<T>;
}

pub trait TorusSurface3DDistance<T: Scalar> {
    fn distance_to_point(&self, point: Point3D<T>) -> T;
}
