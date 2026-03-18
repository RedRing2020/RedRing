//! Circle contracts.

use crate::Scalar;

pub trait Circle2DConstructor<T: Scalar> {
    fn new(center: (T, T), radius: T) -> Option<Self>
    where
        Self: Sized;

    fn new_with_ref_direction(center: (T, T), radius: T, ref_direction: (T, T)) -> Option<Self>
    where
        Self: Sized;

    fn unit_circle() -> Self
    where
        Self: Sized;

    fn from_center_and_point(center: (T, T), point_on_circle: (T, T)) -> Option<Self>
    where
        Self: Sized;

    fn from_three_points(p1: (T, T), p2: (T, T), p3: (T, T)) -> Option<Self>
    where
        Self: Sized;

    fn centered_at_origin(radius: T) -> Option<Self>
    where
        Self: Sized;
}

pub trait Circle3DConstructor<T: Scalar> {
    fn new(center: (T, T, T), axis: (T, T, T), radius: T) -> Option<Self>
    where
        Self: Sized;

    fn new_xy_plane(center: (T, T, T), radius: T) -> Option<Self>
    where
        Self: Sized;

    fn unit_circle_xy() -> Self
    where
        Self: Sized;

    fn from_center_and_point(
        center: (T, T, T),
        axis: (T, T, T),
        point_on_circle: (T, T, T),
    ) -> Option<Self>
    where
        Self: Sized;

    fn from_three_points(p1: (T, T, T), p2: (T, T, T), p3: (T, T, T)) -> Option<Self>
    where
        Self: Sized;

    fn centered_at_origin_xy(radius: T) -> Option<Self>
    where
        Self: Sized;

    fn new_xz_plane(center: (T, T, T), radius: T) -> Option<Self>
    where
        Self: Sized;

    fn new_yz_plane(center: (T, T, T), radius: T) -> Option<Self>
    where
        Self: Sized;
}

pub trait Circle2DProperties<T: Scalar> {
    fn center(&self) -> (T, T);
    fn radius(&self) -> T;
    fn ref_direction(&self) -> (T, T);
    fn diameter(&self) -> T;
    fn dimension(&self) -> u32;
    fn is_unit_circle(&self) -> bool;
    fn is_centered_at_origin(&self) -> bool;
    fn is_degenerate(&self) -> bool;
}

pub trait Circle3DProperties<T: Scalar> {
    fn center(&self) -> (T, T, T);
    fn radius(&self) -> T;
    fn axis(&self) -> (T, T, T);
    fn ref_direction(&self) -> (T, T, T);
    fn dimension(&self) -> u32;
    fn is_unit_circle(&self) -> bool;
    fn is_centered_at_origin(&self) -> bool;
    fn is_degenerate(&self) -> bool;
    fn is_on_xy_plane(&self) -> bool;
}

// TODO(#318): Measure 契約は責務分離フェーズで shape definition から分離する。
pub trait Circle2DMeasure<T: Scalar> {
    fn circumference(&self) -> T;
    fn area(&self) -> T;
    fn contains_point(&self, point: (T, T)) -> bool;
    fn distance_to_point(&self, point: (T, T)) -> T;
    fn point_on_circumference(&self, point: (T, T)) -> bool;
    fn closest_point_to(&self, point: (T, T)) -> (T, T);
    fn point_at_parameter(&self, t: T) -> (T, T);
    fn distance_to_circle(&self, other: &Self) -> T;
}

pub trait Circle3DMeasure<T: Scalar> {
    fn circumference(&self) -> T;
    fn area(&self) -> T;
    fn contains_point(&self, point: (T, T, T)) -> bool;
    fn distance_to_point(&self, point: (T, T, T)) -> T;
    fn point_on_circumference(&self, point: (T, T, T)) -> bool;
    fn closest_point_to(&self, point: (T, T, T)) -> (T, T, T);
    fn point_at_parameter(&self, t: T) -> (T, T, T);
    fn distance_to_circle(&self, other: &Self) -> T;
}

pub trait Circle2DCore<T: Scalar>:
    Circle2DConstructor<T> + Circle2DProperties<T> + Circle2DMeasure<T>
{
}

pub trait Circle3DCore<T: Scalar>:
    Circle3DConstructor<T> + Circle3DProperties<T> + Circle3DMeasure<T>
{
}

impl<T: Scalar, C> Circle2DCore<T> for C where
    C: Circle2DConstructor<T> + Circle2DProperties<T> + Circle2DMeasure<T>
{
}

impl<T: Scalar, C> Circle3DCore<T> for C where
    C: Circle3DConstructor<T> + Circle3DProperties<T> + Circle3DMeasure<T>
{
}
