//! Plane3D 形状の契約定義
//!
//! geo_primitives が実装すべき Plane3D の公開 trait。

use crate::Scalar;

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

    /// XY平面かどうか判定
    fn is_xy_plane(&self) -> bool;

    /// XZ平面かどうか判定
    fn is_xz_plane(&self) -> bool;

    /// YZ平面かどうか判定
    fn is_yz_plane(&self) -> bool;
}
