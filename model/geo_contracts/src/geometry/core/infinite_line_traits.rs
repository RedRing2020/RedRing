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

    fn vertical(x: T) -> Self
    where
        Self: Sized;

    fn x_axis() -> Self
    where
        Self: Sized;

    fn y_axis() -> Self
    where
        Self: Sized;

    fn through_origin(direction: (T, T)) -> Option<Self>
    where
        Self: Sized;

    fn from_angle(angle: T) -> Self
    where
        Self: Sized;

    fn from_point_and_angle(point: (T, T), angle: T) -> Self
    where
        Self: Sized;

    fn perpendicular_through(point: (T, T), other: &Self) -> Self
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

    fn x_parallel(point: (T, T, T)) -> Self
    where
        Self: Sized;

    fn y_parallel(point: (T, T, T)) -> Self
    where
        Self: Sized;

    fn z_parallel(point: (T, T, T)) -> Self
    where
        Self: Sized;

    fn x_axis() -> Self
    where
        Self: Sized;

    fn y_axis() -> Self
    where
        Self: Sized;

    fn z_axis() -> Self
    where
        Self: Sized;

    fn through_origin(direction: (T, T, T)) -> Option<Self>
    where
        Self: Sized;

    fn from_xy_angle(angle: T) -> Self
    where
        Self: Sized;

    fn from_point_and_xy_angle(point: (T, T, T), angle: T) -> Self
    where
        Self: Sized;

    fn perpendicular_in_plane(
        point: (T, T, T),
        other: &Self,
        plane_normal: (T, T, T),
    ) -> Option<Self>
    where
        Self: Sized;
}

pub trait InfiniteLine2DProperties<T: Scalar> {
    fn point(&self) -> (T, T);
    fn direction(&self) -> (T, T);
    fn normal(&self) -> (T, T);
    fn slope(&self) -> Option<T>;
    fn y_intercept(&self) -> Option<T>;
    fn x_intercept(&self) -> Option<T>;
    fn is_horizontal(&self) -> bool;
    fn is_vertical(&self) -> bool;
    fn passes_through_origin(&self) -> bool;
    fn dimension(&self) -> u32;
    fn angle(&self) -> T;
    fn is_above(&self, point: (T, T)) -> bool;
    fn is_below(&self, point: (T, T)) -> bool;
}

pub trait InfiniteLine3DProperties<T: Scalar> {
    fn point(&self) -> (T, T, T);
    fn direction(&self) -> (T, T, T);
    fn is_x_parallel(&self) -> bool;
    fn is_y_parallel(&self) -> bool;
    fn is_z_parallel(&self) -> bool;
    fn is_xy_parallel(&self) -> bool;
    fn is_xz_parallel(&self) -> bool;
    fn is_yz_parallel(&self) -> bool;
    fn passes_through_origin(&self) -> bool;
    fn dimension(&self) -> u32;
    fn xy_angle(&self) -> T;
    fn is_on_plane(&self, plane_normal: (T, T, T), plane_point: (T, T, T)) -> bool;
    fn is_axis_aligned(&self) -> bool;
}

pub trait InfiniteLine2DMeasure<T: Scalar> {
    fn point_at_parameter(&self, t: T) -> (T, T);
    fn distance_to_point(&self, point: (T, T)) -> T;
    fn contains_point(&self, point: (T, T)) -> bool;
    fn project_point(&self, point: (T, T)) -> (T, T);
    fn parameter_for_point(&self, point: (T, T)) -> T;
    fn intersection(&self, other: &Self) -> Option<(T, T)>;
    fn is_parallel_to(&self, other: &Self) -> bool;
    fn is_perpendicular_to(&self, other: &Self) -> bool;
    fn is_same_line(&self, other: &Self) -> bool;
    fn angle_to(&self, other: &Self) -> T;
    fn reverse(&self) -> Self
    where
        Self: Sized;
    fn mirror_point(&self, point: (T, T)) -> (T, T);
    fn offset(&self, distance: T) -> Self
    where
        Self: Sized;
    fn rotate_around_origin(&self, angle: T) -> Self
    where
        Self: Sized;
    fn rotate_around_point(&self, center: (T, T), angle: T) -> Self
    where
        Self: Sized;
}

pub trait InfiniteLine3DMeasure<T: Scalar> {
    fn point_at_parameter(&self, t: T) -> (T, T, T);
    fn distance_to_point(&self, point: (T, T, T)) -> T;
    fn contains_point(&self, point: (T, T, T)) -> bool;
    fn project_point(&self, point: (T, T, T)) -> (T, T, T);
    fn parameter_for_point(&self, point: (T, T, T)) -> T;
    fn closest_points(&self, other: &Self) -> Option<TwoPoints3D<T>>;
    fn is_parallel_to(&self, other: &Self) -> bool;
    fn is_perpendicular_to(&self, other: &Self) -> bool;
    fn is_same_line(&self, other: &Self) -> bool;
    fn intersects(&self, other: &Self) -> bool;
    fn is_skew_to(&self, other: &Self) -> bool;
    fn angle_to(&self, other: &Self) -> T;
    fn reverse(&self) -> Self
    where
        Self: Sized;
    fn mirror_point(&self, point: (T, T, T)) -> (T, T, T);
    fn rotate_around_axis(
        &self,
        axis_point: (T, T, T),
        axis_direction: (T, T, T),
        angle: T,
    ) -> Option<Self>
    where
        Self: Sized;
}

pub trait InfiniteLine2DCore<T: Scalar>:
    InfiniteLine2DConstructor<T> + InfiniteLine2DProperties<T> + InfiniteLine2DMeasure<T>
{
}

pub trait InfiniteLine3DCore<T: Scalar>:
    InfiniteLine3DConstructor<T> + InfiniteLine3DProperties<T> + InfiniteLine3DMeasure<T>
{
}

impl<T: Scalar, L> InfiniteLine2DCore<T> for L where
    L: InfiniteLine2DConstructor<T> + InfiniteLine2DProperties<T> + InfiniteLine2DMeasure<T>
{
}

impl<T: Scalar, L> InfiniteLine3DCore<T> for L where
    L: InfiniteLine3DConstructor<T> + InfiniteLine3DProperties<T> + InfiniteLine3DMeasure<T>
{
}
