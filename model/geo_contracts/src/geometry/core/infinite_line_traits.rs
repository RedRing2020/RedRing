//! InfiniteLine 形状の契約定義
//!
//! geo_primitives が実装すべき InfiniteLine 系の公開 trait。

use crate::Scalar;

/// InfiniteLine3D 基本プロパティ取得 trait
pub trait InfiniteLine3DProperties<T: Scalar> {
    /// 直線上の点を取得
    fn point(&self) -> (T, T, T);

    /// 方向ベクトルを取得（正規化済み）
    fn direction(&self) -> (T, T, T);

    /// X軸に平行かどうか判定
    fn is_x_parallel(&self) -> bool;

    /// Y軸に平行かどうか判定
    fn is_y_parallel(&self) -> bool;

    /// Z軸に平行かどうか判定
    fn is_z_parallel(&self) -> bool;

    /// XY平面に平行かどうか判定
    fn is_xy_parallel(&self) -> bool;

    /// XZ平面に平行かどうか判定
    fn is_xz_parallel(&self) -> bool;

    /// YZ平面に平行かどうか判定
    fn is_yz_parallel(&self) -> bool;

    /// 原点を通るかどうか判定
    fn passes_through_origin(&self) -> bool;

    /// XY平面上での方向角度（azimuth）を取得
    fn xy_angle(&self) -> T;

    /// 指定平面上にあるか判定
    fn is_on_plane(&self, plane_normal: (T, T, T), plane_point: (T, T, T)) -> bool;

    /// 座標軸に平行か判定（いずれかの軸）
    fn is_axis_aligned(&self) -> bool;
}
