//! InfiniteLine 形状の契約定義
//!
//! geo_primitives が実装すべき InfiniteLine 系の公開 trait。

use crate::Scalar;

pub type TwoPoints3D<T> = ((T, T, T), (T, T, T));

pub trait InfiniteLine2DConstructor<T: Scalar> {
    fn new(point: (T, T), direction: (T, T)) -> Option<Self>
    where
        Self: Sized;

    fn from_two_points(p1: (T, T), p2: (T, T)) -> Option<Self>
    where
        Self: Sized;

    fn horizontal(y: T) -> Self
    where
        Self: Sized;
}

pub trait InfiniteLine3DConstructor<T: Scalar> {
    fn new(point: (T, T, T), direction: (T, T, T)) -> Option<Self>
    where
        Self: Sized;

    fn from_two_points(p1: (T, T, T), p2: (T, T, T)) -> Option<Self>
    where
        Self: Sized;
}

pub trait InfiniteLine2DProperties<T: Scalar> {
    fn point(&self) -> (T, T);
    fn direction(&self) -> (T, T);
    fn normal(&self) -> (T, T);
    fn dimension(&self) -> u32;
    fn angle(&self) -> T;
}

pub trait InfiniteLine3DProperties<T: Scalar> {
    fn point(&self) -> (T, T, T);
    fn direction(&self) -> (T, T, T);
    fn dimension(&self) -> u32;
}

pub trait InfiniteLine2DEvaluation<T: Scalar> {
    /// 基準点 `point()` を原点とし、正規化方向 `direction()` に沿う signed support parameter `t` の evaluation point を返す
    fn point_at_parameter(&self, t: T) -> (T, T);

    /// 点を同じ signed support parameter 系へ写像する
    fn parameter_for_point(&self, point: (T, T)) -> T;
}

pub trait InfiniteLine2DDistance<T: Scalar> {
    fn distance_to_point(&self, point: (T, T)) -> T;
}

pub trait InfiniteLine2DContainment<T: Scalar> {
    fn contains_point(&self, point: (T, T)) -> bool;
}

pub trait InfiniteLine2DProjection<T: Scalar> {
    /// 点に最も近い曲線上の点を返す
    fn closest_point(&self, point: (T, T)) -> (T, T);
}

pub trait InfiniteLine2DTransform<T: Scalar> {
    fn reverse(&self) -> Self
    where
        Self: Sized;
}

pub trait InfiniteLine3DEvaluation<T: Scalar> {
    /// 基準点 `point()` を原点とし、正規化方向 `direction()` に沿う signed support parameter `t` の evaluation point を返す
    fn point_at_parameter(&self, t: T) -> (T, T, T);

    /// 点を同じ signed support parameter 系へ写像する
    fn parameter_for_point(&self, point: (T, T, T)) -> T;
}

pub trait InfiniteLine3DDistance<T: Scalar> {
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}

pub trait InfiniteLine3DContainment<T: Scalar> {
    fn contains_point(&self, point: (T, T, T)) -> bool;
}

pub trait InfiniteLine3DProjection<T: Scalar> {
    /// 点に最も近い曲線上の点を返す
    fn closest_point(&self, point: (T, T, T)) -> (T, T, T);
}

pub trait InfiniteLine3DTransform<T: Scalar> {
    fn reverse(&self) -> Self
    where
        Self: Sized;
}
