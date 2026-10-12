//! InfiniteLine 形状の契約定義
//!
//! geo_primitives が実装すべき InfiniteLine 系の公開 trait。

use crate::{Direction2D, Direction3D, Point2D, Point3D, Scalar, Vector2D, Vector3D};

pub trait InfiniteLine2DConstructor<T: Scalar> {
    /// 点と方向ベクトルから作成する（方向ベクトルがゼロの場合は `None`）
    fn new(point: &Point2D<T>, direction: Vector2D<T>) -> Option<Self>
    where
        Self: Sized;

    /// 2点を通る直線を作成する（2点が一致する場合は `None`）
    fn from_two_points(p1: &Point2D<T>, p2: &Point2D<T>) -> Option<Self>
    where
        Self: Sized;

    /// X軸に平行な直線（y = y0）を作成する
    fn horizontal(y: T) -> Self
    where
        Self: Sized;
}

pub trait InfiniteLine3DConstructor<T: Scalar> {
    /// 点と方向ベクトルから作成する（方向ベクトルがゼロの場合は `None`）
    fn new(point: &Point3D<T>, direction: Vector3D<T>) -> Option<Self>
    where
        Self: Sized;

    /// 2点を通る直線を作成する（2点が一致する場合は `None`）
    fn from_two_points(p1: &Point3D<T>, p2: &Point3D<T>) -> Option<Self>
    where
        Self: Sized;
}

pub trait InfiniteLine2DProperties<T: Scalar> {
    fn point(&self) -> Point2D<T>;
    fn direction(&self) -> Direction2D<T>;
    /// 方向を右回りに90度回転した法線
    fn normal(&self) -> Direction2D<T>;
    fn dimension(&self) -> u32;
}

pub trait InfiniteLine3DProperties<T: Scalar> {
    fn point(&self) -> Point3D<T>;
    fn direction(&self) -> Direction3D<T>;
    fn dimension(&self) -> u32;
}

pub trait InfiniteLine2DEvaluation<T: Scalar> {
    /// 基準点 `point()` を原点とし、正規化方向 `direction()` に沿う signed support parameter `t` の evaluation point を返す
    fn point_at_parameter(&self, t: T) -> Point2D<T>;

    /// 点を同じ signed support parameter 系へ写像する
    fn parameter_for_point(&self, point: &Point2D<T>) -> T;
}

pub trait InfiniteLine2DDistance<T: Scalar> {
    fn distance_to_point(&self, point: &Point2D<T>) -> T;
}

pub trait InfiniteLine2DContainment<T: Scalar> {
    /// 既定の距離の許容誤差で、点が直線上にあるかを判定する
    fn contains_point(&self, point: &Point2D<T>) -> bool;
}

pub trait InfiniteLine2DProjection<T: Scalar> {
    /// 点に最も近い曲線上の点を返す
    fn closest_point(&self, point: &Point2D<T>) -> Point2D<T>;
}

pub trait InfiniteLine2DTransform<T: Scalar> {
    fn reverse(&self) -> Self
    where
        Self: Sized;
}

pub trait InfiniteLine3DEvaluation<T: Scalar> {
    /// 基準点 `point()` を原点とし、正規化方向 `direction()` に沿う signed support parameter `t` の evaluation point を返す
    fn point_at_parameter(&self, t: T) -> Point3D<T>;

    /// 点を同じ signed support parameter 系へ写像する
    fn parameter_for_point(&self, point: &Point3D<T>) -> T;
}

pub trait InfiniteLine3DDistance<T: Scalar> {
    fn distance_to_point(&self, point: &Point3D<T>) -> T;
}

pub trait InfiniteLine3DContainment<T: Scalar> {
    /// 既定の距離の許容誤差で、点が直線上にあるかを判定する
    fn contains_point(&self, point: &Point3D<T>) -> bool;
}

pub trait InfiniteLine3DProjection<T: Scalar> {
    /// 点に最も近い曲線上の点を返す
    fn closest_point(&self, point: &Point3D<T>) -> Point3D<T>;
}

pub trait InfiniteLine3DTransform<T: Scalar> {
    fn reverse(&self) -> Self
    where
        Self: Sized;
}
