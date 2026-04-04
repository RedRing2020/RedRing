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

pub trait Circle2DDerived<T: Scalar> {
    fn circumference(&self) -> T;
    fn area(&self) -> T;
}

pub trait Circle2DEvaluation<T: Scalar> {
    fn point_at_parameter(&self, t: T) -> (T, T);
}

pub trait Circle2DContainment<T: Scalar> {
    fn contains_point(&self, point: (T, T)) -> bool;
    fn point_on_circumference(&self, point: (T, T)) -> bool;
}

pub trait Circle2DDistance<T: Scalar> {
    fn distance_to_point(&self, point: (T, T)) -> T;
}

pub trait Circle2DProjection<T: Scalar> {
    fn closest_point_to(&self, point: (T, T)) -> (T, T);
}

pub trait Circle3DDerived<T: Scalar> {
    fn circumference(&self) -> T;
    fn area(&self) -> T;
}

pub trait Circle3DEvaluation<T: Scalar> {
    fn point_at_parameter(&self, t: T) -> (T, T, T);
}

pub trait Circle3DContainment<T: Scalar> {
    fn contains_point(&self, point: (T, T, T)) -> bool;
    fn point_on_circumference(&self, point: (T, T, T)) -> bool;
}

pub trait Circle3DDistance<T: Scalar> {
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}

pub trait Circle3DProjection<T: Scalar> {
    fn closest_point_to(&self, point: (T, T, T)) -> (T, T, T);
}

pub trait Circle2DMeasure<T: Scalar>:
    Circle2DDerived<T>
    + Circle2DEvaluation<T>
    + Circle2DContainment<T>
    + Circle2DDistance<T>
    + Circle2DProjection<T>
{
    fn circumference(&self) -> T {
        <Self as Circle2DDerived<T>>::circumference(self)
    }

    fn area(&self) -> T {
        <Self as Circle2DDerived<T>>::area(self)
    }

    fn contains_point(&self, point: (T, T)) -> bool {
        <Self as Circle2DContainment<T>>::contains_point(self, point)
    }

    fn distance_to_point(&self, point: (T, T)) -> T {
        <Self as Circle2DDistance<T>>::distance_to_point(self, point)
    }

    fn point_on_circumference(&self, point: (T, T)) -> bool {
        <Self as Circle2DContainment<T>>::point_on_circumference(self, point)
    }

    fn closest_point_to(&self, point: (T, T)) -> (T, T) {
        <Self as Circle2DProjection<T>>::closest_point_to(self, point)
    }

    fn point_at_parameter(&self, t: T) -> (T, T) {
        <Self as Circle2DEvaluation<T>>::point_at_parameter(self, t)
    }
}

pub trait Circle3DMeasure<T: Scalar>:
    Circle3DDerived<T>
    + Circle3DEvaluation<T>
    + Circle3DContainment<T>
    + Circle3DDistance<T>
    + Circle3DProjection<T>
{
    fn circumference(&self) -> T {
        <Self as Circle3DDerived<T>>::circumference(self)
    }

    fn area(&self) -> T {
        <Self as Circle3DDerived<T>>::area(self)
    }

    fn contains_point(&self, point: (T, T, T)) -> bool {
        <Self as Circle3DContainment<T>>::contains_point(self, point)
    }

    fn distance_to_point(&self, point: (T, T, T)) -> T {
        <Self as Circle3DDistance<T>>::distance_to_point(self, point)
    }

    fn point_on_circumference(&self, point: (T, T, T)) -> bool {
        <Self as Circle3DContainment<T>>::point_on_circumference(self, point)
    }

    fn closest_point_to(&self, point: (T, T, T)) -> (T, T, T) {
        <Self as Circle3DProjection<T>>::closest_point_to(self, point)
    }

    fn point_at_parameter(&self, t: T) -> (T, T, T) {
        <Self as Circle3DEvaluation<T>>::point_at_parameter(self, t)
    }
}

impl<T: Scalar, C> Circle2DMeasure<T> for C where
    C: Circle2DDerived<T>
        + Circle2DEvaluation<T>
        + Circle2DContainment<T>
        + Circle2DDistance<T>
        + Circle2DProjection<T>
{
}

impl<T: Scalar, C> Circle3DMeasure<T> for C where
    C: Circle3DDerived<T>
        + Circle3DEvaluation<T>
        + Circle3DContainment<T>
        + Circle3DDistance<T>
        + Circle3DProjection<T>
{
}

pub trait Circle2DCore<T: Scalar>: Circle2DConstructor<T> + Circle2DProperties<T> {}

pub trait Circle3DCore<T: Scalar>: Circle3DConstructor<T> + Circle3DProperties<T> {}

impl<T: Scalar, C> Circle2DCore<T> for C where C: Circle2DConstructor<T> + Circle2DProperties<T> {}

impl<T: Scalar, C> Circle3DCore<T> for C where C: Circle3DConstructor<T> + Circle3DProperties<T> {}
