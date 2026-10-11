//! Ray 形状の契約定義
//!
//! geo_primitives が実装すべき Ray 系の公開 trait。
//! analysis との結合を避けるため origin / direction はタプルで返す。

use crate::Scalar;

pub trait Ray2DConstructor<T: Scalar> {
    fn new(origin: (T, T), direction: (T, T)) -> Option<Self>
    where
        Self: Sized;

    fn from_points(start: (T, T), through: (T, T)) -> Option<Self>
    where
        Self: Sized;
}

pub trait Ray3DConstructor<T: Scalar> {
    fn new(origin: (T, T, T), direction: (T, T, T)) -> Option<Self>
    where
        Self: Sized;

    fn from_points(start: (T, T, T), through: (T, T, T)) -> Option<Self>
    where
        Self: Sized;

    fn from_spherical(origin: (T, T, T), azimuth: T, elevation: T) -> Self
    where
        Self: Sized;
}

pub trait Ray2DProperties<T: Scalar> {
    fn origin(&self) -> (T, T);
    fn direction(&self) -> (T, T);
    fn is_valid(&self) -> bool;
    fn dimension(&self) -> usize {
        2
    }
    fn angle(&self) -> T;
}

pub trait Ray3DProperties<T: Scalar> {
    fn origin(&self) -> (T, T, T);
    fn direction(&self) -> (T, T, T);
    fn is_valid(&self) -> bool;
    fn dimension(&self) -> usize {
        3
    }
}

pub trait Ray2DEvaluation<T: Scalar> {
    /// Ray origin を原点とし、正規化方向に沿う support parameter `t` の evaluation point を返す
    ///
    /// Ray の有効 domain は `t >= 0` だが、この API は support line evaluation を表し、clamp は行わない。
    fn point_at_parameter(&self, t: T) -> (T, T);

    /// 点を同じ support parameter 系へ写像する
    fn parameter_for_point(&self, point: (T, T)) -> T;
}

pub trait Ray2DProjection<T: Scalar> {
    /// 点に最も近い曲線上の点を返す
    fn closest_point(&self, point: (T, T)) -> (T, T);
}

pub trait Ray2DDistance<T: Scalar> {
    fn distance_to_point(&self, point: (T, T)) -> T;
}

pub trait Ray2DContainment<T: Scalar> {
    fn contains_point(&self, point: (T, T)) -> bool;
}

pub trait Ray2DTransform<T: Scalar> {
    fn reverse(&self) -> Self
    where
        Self: Sized;
}

pub trait Ray3DEvaluation<T: Scalar> {
    /// Ray origin を原点とし、正規化方向に沿う support parameter `t` の evaluation point を返す
    ///
    /// Ray の有効 domain は `t >= 0` だが、この API は support line evaluation を表し、clamp は行わない。
    fn point_at_parameter(&self, t: T) -> (T, T, T);

    /// 点を同じ support parameter 系へ写像する
    fn parameter_for_point(&self, point: (T, T, T)) -> T;
}

pub trait Ray3DProjection<T: Scalar> {
    /// 点に最も近い曲線上の点を返す
    fn closest_point(&self, point: (T, T, T)) -> (T, T, T);
}

pub trait Ray3DDistance<T: Scalar> {
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}

pub trait Ray3DContainment<T: Scalar> {
    fn contains_point(&self, point: (T, T, T)) -> bool;
}

pub trait Ray3DTransform<T: Scalar> {
    fn reverse(&self) -> Self
    where
        Self: Sized;
}
