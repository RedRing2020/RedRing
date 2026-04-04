//! Plane3D 形状の契約定義
//!
//! geo_primitives が実装すべき Plane3D の公開 trait。

use crate::Scalar;

/// Plane3D Constructor トレイト
pub trait Plane3DConstructor<T: Scalar>: Sized {
    /// 原点と軸から平面を作成（STEP AXIS2_PLACEMENT_3D形式）
    fn from_origin_and_axes(
        origin: (T, T, T),
        normal: (T, T, T),
        u_direction: (T, T, T),
    ) -> Option<Self>;

    /// 3点から平面を作成
    fn from_three_points(p1: (T, T, T), p2: (T, T, T), p3: (T, T, T)) -> Option<Self>;

    /// XY平面を作成（Z=0）
    fn xy_plane() -> Self;

    /// 点と法線から平面を作成
    fn from_point_and_normal(point: (T, T, T), normal: (T, T, T)) -> Option<Self>;

    /// XZ平面を作成（Y=0）
    fn xz_plane() -> Self;

    /// YZ平面を作成（X=0）
    fn yz_plane() -> Self;
}

/// Plane3D 基本プロパティ取得 trait
pub trait Plane3DProperties<T: Scalar> {
    /// 原点座標を取得
    fn origin(&self) -> (T, T, T);

    /// 法線（Z軸）方向を取得
    fn normal(&self) -> (T, T, T);

    /// U軸（X軸）方向を取得
    fn u_axis(&self) -> (T, T, T);

    /// V軸（Y軸）方向を取得
    fn v_axis(&self) -> (T, T, T);

    /// 形状の次元数（2: 2次元多様体）
    fn dimension(&self) -> u32;

    /// XY平面かどうか判定
    fn is_xy_plane(&self) -> bool;

    /// XZ平面かどうか判定
    fn is_xz_plane(&self) -> bool;

    /// YZ平面かどうか判定
    fn is_yz_plane(&self) -> bool;
}

pub trait Plane3DContainment<T: Scalar> {
    /// 点が平面上にあるか判定
    fn contains_point(&self, point: (T, T, T)) -> bool;
}

pub trait Plane3DDistance<T: Scalar> {
    /// 点から平面への距離
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}

pub trait Plane3DProjection<T: Scalar> {
    /// 点を平面に投影
    fn project_point(&self, point: (T, T, T)) -> (T, T, T);
}

pub trait Plane3DDerived<T: Scalar> {
    /// 平面の方程式係数を取得（Ax + By + Cz + D = 0）
    fn equation_coefficients(&self) -> (T, T, T, T);
}

pub trait Plane3DEvaluation<T: Scalar> {
    /// 点の平面座標（UV座標）を取得
    fn point_to_uv(&self, point: (T, T, T)) -> (T, T);

    /// UV座標から3D点を取得
    fn uv_to_point(&self, u: T, v: T) -> (T, T, T);
}

pub trait Plane3DTransform<T: Scalar> {
    /// 点を平面に対して鏡面反射
    fn mirror_point(&self, point: (T, T, T)) -> (T, T, T);
}

/// Plane3D Core トレイト（統合インターフェース）
pub trait Plane3DCore<T: Scalar>: Plane3DConstructor<T> + Plane3DProperties<T> {}
