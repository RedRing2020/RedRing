//! Arc 形状の契約定義
//!
//! geo_primitives が実装すべき Arc 系の公開 trait。
//! 角度範囲は、開始角から反時計回りに終了角までとする（終了角が開始角より小さい場合は 2π を加える）。

use crate::{Angle, Direction3D, Point2D, Point3D, Scalar};

pub trait Arc2DConstructor<T: Scalar> {
    /// 中心・半径・開始角・終了角から作成する（半径が正でない場合は `None`）
    fn new(
        center: Point2D<T>,
        radius: T,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Option<Self>
    where
        Self: Sized;

    /// 開始点・中間点・終了点を通る円弧を作成する（3点が一直線上にある場合は `None`）
    fn from_three_points(start: Point2D<T>, mid: Point2D<T>, end: Point2D<T>) -> Option<Self>
    where
        Self: Sized;
}

pub trait Arc3DConstructor<T: Scalar> {
    /// 中心・半径・法線・開始方向・開始角・終了角から作成する
    ///
    /// 半径が正でない場合と、開始方向が法線に直交しない場合は `None`。
    fn new(
        center: Point3D<T>,
        radius: T,
        normal: Direction3D<T>,
        start_dir: Direction3D<T>,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Option<Self>
    where
        Self: Sized;

    /// XY 平面上の円弧を作成する（法線は Z 軸正方向、開始方向は X 軸正方向）
    fn xy_arc(
        center: Point3D<T>,
        radius: T,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Option<Self>
    where
        Self: Sized;

    /// 開始点・中間点・終了点を通る円弧を作成する（3点が一直線上にある場合は `None`）
    fn from_three_points(start: Point3D<T>, mid: Point3D<T>, end: Point3D<T>) -> Option<Self>
    where
        Self: Sized;

    /// 1 周の円弧を作成する（半径が正でない場合は `None`）
    fn full_circle(center: Point3D<T>, normal: Direction3D<T>, radius: T) -> Option<Self>
    where
        Self: Sized;
}

pub trait Arc2DProperties<T: Scalar> {
    fn center(&self) -> Point2D<T>;
    fn radius(&self) -> T;
    fn start_angle(&self) -> Angle<T>;
    fn end_angle(&self) -> Angle<T>;
    /// 開始角から反時計回りに終了角までの掃引角
    fn angle_span(&self) -> Angle<T>;
    fn is_full_circle(&self) -> bool;
}

pub trait Arc3DProperties<T: Scalar> {
    fn center(&self) -> Point3D<T>;
    fn radius(&self) -> T;
    /// 円弧平面の法線
    fn normal(&self) -> Direction3D<T>;
    /// 角度 0 の方向
    fn start_direction(&self) -> Direction3D<T>;
    fn start_angle(&self) -> Angle<T>;
    fn end_angle(&self) -> Angle<T>;
    /// 開始角から反時計回りに終了角までの掃引角
    fn angle_span(&self) -> Angle<T>;
    fn is_full_circle(&self) -> bool;
}

pub trait Arc2DDerived<T: Scalar> {
    fn length(&self) -> T;
}

pub trait Arc2DEndpoint<T: Scalar> {
    /// 母曲線上の ideal な開始点を取得
    fn start_point(&self) -> Point2D<T>;

    /// 母曲線上の ideal な終了点を取得
    fn end_point(&self) -> Point2D<T>;
}

pub trait Arc2DEvaluation<T: Scalar> {
    /// Arc trim-local parameter `t` (`0 <= t <= 1`) に対応する ideal evaluation point を取得
    ///
    /// `t=0/1` は Arc のトリム区間の両端を指すが、拘束端点補間を意味しない。
    /// 評価時には母円の local angle parameter（開始角 + 掃引角 × t）へ写像される。
    fn point_at_parameter(&self, t: T) -> Point2D<T>;

    /// 母円の native angle parameter を直接指定して ideal evaluation point を取得
    ///
    /// 角度範囲内かどうかの判定は `contains_angle` 側の責務とし、この API は角度指定評価を表す。
    fn point_at_angle(&self, angle: Angle<T>) -> Point2D<T>;
}

pub trait Arc2DDistance<T: Scalar> {
    fn distance_to_point(&self, point: Point2D<T>) -> T;
}

pub trait Arc2DContainment<T: Scalar> {
    /// 既定の距離の許容誤差で、点が円弧上にあるかを判定する
    fn contains_point(&self, point: Point2D<T>) -> bool;
}

pub trait Arc2DTrimRange<T: Scalar> {
    /// 角度が角度範囲に含まれるかを判定する（範囲の両端を含み、既定の角度トレランスで判定する）
    fn contains_angle(&self, angle: Angle<T>) -> bool;

    /// 点の角度が角度範囲に含まれるかを判定する
    ///
    /// 点が曲線上にあるかではなく、角度範囲のみを判定する。点の角度は、点に最も近い母円上の点の角度（中心の点は角度 0）とする。
    fn contains_point_angle(&self, point: Point2D<T>) -> bool;
}

pub trait Arc3DDerived<T: Scalar> {
    fn length(&self) -> T;
}

pub trait Arc3DEndpoint<T: Scalar> {
    /// 母曲線上の ideal な開始点を取得
    fn start_point(&self) -> Point3D<T>;

    /// 母曲線上の ideal な終了点を取得
    fn end_point(&self) -> Point3D<T>;
}

pub trait Arc3DEvaluation<T: Scalar> {
    /// Arc trim-local parameter `t` (`0 <= t <= 1`) に対応する ideal evaluation point を取得
    ///
    /// `t=0/1` は Arc のトリム区間の両端を指すが、拘束端点補間を意味しない。
    /// 評価時には母円の local angle parameter（開始角 + 掃引角 × t）へ写像される。
    fn point_at_parameter(&self, t: T) -> Point3D<T>;

    /// 母円の native angle parameter を直接指定して ideal evaluation point を取得
    fn point_at_angle(&self, angle: Angle<T>) -> Point3D<T>;
}

pub trait Arc3DDistance<T: Scalar> {
    fn distance_to_point(&self, point: Point3D<T>) -> T;
}

pub trait Arc3DContainment<T: Scalar> {
    /// 既定の距離の許容誤差で、点が円弧上にあるかを判定する
    fn contains_point(&self, point: Point3D<T>) -> bool;
}

pub trait Arc3DTrimRange<T: Scalar> {
    /// 角度が角度範囲に含まれるかを判定する（範囲の両端を含み、既定の角度トレランスで判定する）
    fn contains_angle(&self, angle: Angle<T>) -> bool;

    /// 点の角度が角度範囲に含まれるかを判定する
    ///
    /// 点が曲線上にあるかではなく、角度範囲のみを判定する。点の角度は、円弧平面へ投影した点の、
    /// 開始方向を角度 0 とし法線まわりに反時計回りを正とする角度とする。中心軸上の点は角度 0 とする。
    fn contains_point_angle(&self, point: Point3D<T>) -> bool;
}

pub trait Arc3DSampling<T: Scalar> {
    /// 開始点から終了点まで、trim-local parameter で等間隔な `num_points` 個の点を返す
    ///
    /// `num_points` が 0 の場合は空、1 の場合は開始点のみを返す。
    fn sample_points(&self, num_points: usize) -> Vec<Point3D<T>>;
}
