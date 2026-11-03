//! Ray 形状の契約定義
//!
//! geo_primitives が実装すべき Ray 系の公開 trait。

use crate::{Direction2D, Direction3D, Point2D, Point3D, Scalar, Vector2D, Vector3D};

pub trait Ray2DConstructor<T: Scalar> {
    /// 起点と方向ベクトルから作成する（方向ベクトルがゼロの場合は `None`）
    fn new(origin: Point2D<T>, direction: Vector2D<T>) -> Option<Self>
    where
        Self: Sized;

    /// 起点と通過点から作成する（2点が一致する場合は `None`）
    fn from_points(start: Point2D<T>, through: Point2D<T>) -> Option<Self>
    where
        Self: Sized;
}

pub trait Ray3DConstructor<T: Scalar> {
    /// 起点と方向ベクトルから作成する（方向ベクトルがゼロの場合は `None`）
    fn new(origin: Point3D<T>, direction: Vector3D<T>) -> Option<Self>
    where
        Self: Sized;

    /// 起点と通過点から作成する（2点が一致する場合は `None`）
    fn from_points(start: Point3D<T>, through: Point3D<T>) -> Option<Self>
    where
        Self: Sized;
}

pub trait Ray2DProperties<T: Scalar> {
    fn origin(&self) -> Point2D<T>;
    fn direction(&self) -> Direction2D<T>;
    fn is_valid(&self) -> bool;
    fn dimension(&self) -> usize {
        2
    }
}

pub trait Ray3DProperties<T: Scalar> {
    fn origin(&self) -> Point3D<T>;
    fn direction(&self) -> Direction3D<T>;
    fn is_valid(&self) -> bool;
    fn dimension(&self) -> usize {
        3
    }
}

pub trait Ray2DEvaluation<T: Scalar> {
    /// Ray origin を原点とし、正規化方向に沿う support parameter `t` の evaluation point を返す
    ///
    /// Ray の有効 domain は `t >= 0` だが、この API は support line evaluation を表し、clamp は行わない。
    fn point_at_parameter(&self, t: T) -> Point2D<T>;

    /// 点を同じ support parameter 系へ写像する
    fn parameter_for_point(&self, point: Point2D<T>) -> T;
}

pub trait Ray2DProjection<T: Scalar> {
    /// 点に最も近い曲線上の点を返す
    fn closest_point(&self, point: Point2D<T>) -> Point2D<T>;
}

pub trait Ray2DDistance<T: Scalar> {
    fn distance_to_point(&self, point: Point2D<T>) -> T;
}

pub trait Ray2DContainment<T: Scalar> {
    /// 既定の距離の許容誤差で、点が Ray 上にあるかを判定する
    fn contains_point(&self, point: Point2D<T>) -> bool;
}

pub trait Ray3DEvaluation<T: Scalar> {
    /// Ray origin を原点とし、正規化方向に沿う support parameter `t` の evaluation point を返す
    ///
    /// Ray の有効 domain は `t >= 0` だが、この API は support line evaluation を表し、clamp は行わない。
    fn point_at_parameter(&self, t: T) -> Point3D<T>;

    /// 点を同じ support parameter 系へ写像する
    fn parameter_for_point(&self, point: Point3D<T>) -> T;
}

pub trait Ray3DProjection<T: Scalar> {
    /// 点に最も近い曲線上の点を返す
    fn closest_point(&self, point: Point3D<T>) -> Point3D<T>;
}

pub trait Ray3DDistance<T: Scalar> {
    fn distance_to_point(&self, point: Point3D<T>) -> T;
}

pub trait Ray3DContainment<T: Scalar> {
    /// 既定の距離の許容誤差で、点が Ray 上にあるかを判定する
    fn contains_point(&self, point: Point3D<T>) -> bool;
}
