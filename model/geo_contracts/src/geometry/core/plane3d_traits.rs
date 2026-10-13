//! Plane3D 形状の契約定義
//!
//! geo_primitives が実装すべき Plane3D の公開 trait。

use crate::{Direction3D, Point3D, Scalar};

/// Plane3D Constructor トレイト
pub trait Plane3DConstructor<T: Scalar>: Sized {
    /// 原点・法線・第一軸方向から平面を作成する（STEP AXIS2_PLACEMENT_3D 形式）
    ///
    /// 第一軸方向は法線成分を除いて正規化する。法線と平行な場合は `None`。
    fn from_origin_and_axes(
        origin: Point3D<T>,
        normal: Direction3D<T>,
        u_direction: Direction3D<T>,
    ) -> Option<Self>;

    /// 原点・U 軸方向の参照点・V 軸方向の参照点から平面を作成する（3点が一直線上にある場合は `None`）
    fn from_three_points(
        origin: Point3D<T>,
        point_u: Point3D<T>,
        point_v: Point3D<T>,
    ) -> Option<Self>;

    /// 点と法線から平面を作成する（第一軸方向は法線に直交する方向を選ぶ）
    fn from_point_and_normal(point: Point3D<T>, normal: Direction3D<T>) -> Option<Self>;
}

/// Plane3D 基本プロパティ取得 trait
pub trait Plane3DProperties<T: Scalar> {
    /// 原点座標を取得
    fn origin(&self) -> Point3D<T>;

    /// 法線（Z軸）方向を取得
    fn normal(&self) -> Direction3D<T>;

    /// U軸（X軸）方向を取得
    fn u_axis(&self) -> Direction3D<T>;

    /// V軸（Y軸）方向を取得
    fn v_axis(&self) -> Direction3D<T>;
}

pub trait Plane3DContainment<T: Scalar> {
    /// 既定の距離の許容誤差で、点が平面上にあるかを判定する
    fn contains_point(&self, point: Point3D<T>) -> bool;
}

pub trait Plane3DDistance<T: Scalar> {
    /// 点から平面への符号付き距離（法線の向きを正とする）
    fn distance_to_point(&self, point: Point3D<T>) -> T;
}

pub trait Plane3DProjection<T: Scalar> {
    /// 点を平面に投影
    fn project_point(&self, point: Point3D<T>) -> Point3D<T>;
}
