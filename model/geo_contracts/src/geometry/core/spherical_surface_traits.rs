//! SphericalSurface3D 形状の契約定義
//!
//! geo_primitives が実装すべき球面の公開 trait。

use crate::{Direction3D, Point3D, Scalar};

pub trait SphericalSurface3DConstructor<T: Scalar>: Sized {
    /// 中心・軸・参照方向・半径から作成する（STEP AXIS2_PLACEMENT_3D 形式）
    ///
    /// 参照方向は軸の成分を除いて正規化する。軸と平行な場合・半径が正でない場合は `None`。
    fn new(
        center: Point3D<T>,
        axis: Direction3D<T>,
        ref_direction: Direction3D<T>,
        radius: T,
    ) -> Option<Self>;

    /// 軸を Z 軸正方向、参照方向を X 軸正方向として作成する（半径が正でない場合は `None`）
    fn new_standard(center: Point3D<T>, radius: T) -> Option<Self>;
}

pub trait SphericalSurface3DProperties<T: Scalar> {
    fn center(&self) -> Point3D<T>;
    fn radius(&self) -> T;
    /// 緯度 π/2 の方向
    fn axis(&self) -> Direction3D<T>;
    /// 経度 0・緯度 0 の方向
    fn ref_direction(&self) -> Direction3D<T>;
}

pub trait SphericalSurface3DEvaluation<T: Scalar> {
    /// 経度 `u` (`0 <= u <= 2π`)・緯度 `v` (`-π/2 <= v <= π/2`) に対応する球面上の点を返す
    fn point_at_uv(&self, u: T, v: T) -> Point3D<T>;
    /// 経度 `u`・緯度 `v` での外向きの法線
    fn normal_at(&self, u: T, v: T) -> Direction3D<T>;
}

pub trait SphericalSurface3DDistance<T: Scalar> {
    fn distance_to_point(&self, point: Point3D<T>) -> T;
}
