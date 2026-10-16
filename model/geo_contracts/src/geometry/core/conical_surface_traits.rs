//! ConicalSurface3D 形状の契約定義
//!
//! geo_primitives が実装すべき円錐面（頂点の両側に無限に続く面）の公開 trait。
//! STEP の CONICAL_SURFACE と同じく、軸上の基準点・そこでの半径・半頂角で定義する。

use crate::{Angle, Direction3D, Point3D, Scalar};

pub trait ConicalSurface3DConstructor<T: Scalar>: Sized {
    /// 軸上の基準点・軸・参照方向・基準点での半径・半頂角から作成する（STEP AXIS2_PLACEMENT_3D 形式）
    ///
    /// 半径は軸方向に `tan(semi_angle)` の割合で増える。参照方向は軸の成分を除いて正規化する。
    /// 軸と平行な場合・半径が正でない場合・半頂角が `0 < semi_angle < π/2` でない場合は `None`。
    fn new(
        center: Point3D<T>,
        axis: Direction3D<T>,
        ref_direction: Direction3D<T>,
        radius: T,
        semi_angle: Angle<T>,
    ) -> Option<Self>;

    /// 軸を Z 軸正方向、参照方向を X 軸正方向として作成する
    fn new_standard(center: Point3D<T>, radius: T, semi_angle: Angle<T>) -> Option<Self>;
}

pub trait ConicalSurface3DProperties<T: Scalar> {
    /// 軸上の基準点（v = 0 の位置）
    fn center(&self) -> Point3D<T>;
    /// 基準点での半径
    fn radius(&self) -> T;
    /// 半頂角（軸と母線のなす角）
    fn semi_angle(&self) -> Angle<T>;
    /// 半径が増える向きの軸の方向
    fn axis(&self) -> Direction3D<T>;
    /// 円周方向の角度 0 の方向
    fn ref_direction(&self) -> Direction3D<T>;
    /// 頂点（基準点から軸の逆方向に `radius / tan(semi_angle)` 離れた位置）
    fn apex(&self) -> Point3D<T>;
}

pub trait ConicalSurface3DEvaluation<T: Scalar> {
    /// 円周方向の角度 `u` (`0 <= u <= 2π`)・軸方向の基準点からの距離 `v` に対応する円錐面上の点を返す
    fn point_at_uv(&self, u: T, v: T) -> Point3D<T>;
    /// `u`・`v` での法線（∂u × ∂v の向き）。頂点では定まらないため `None`
    fn normal_at(&self, u: T, v: T) -> Option<Direction3D<T>>;
}

pub trait ConicalSurface3DDistance<T: Scalar> {
    fn distance_to_point(&self, point: Point3D<T>) -> T;
}
