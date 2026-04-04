//! EllipsoidalSolid trait定義を capability taxonomy に沿って分離する。

use analysis::abstract_types::Scalar;

pub trait EllipsoidalSolid3DConstructor<T: Scalar> {
    fn new(
        center: (T, T, T),
        axis: (T, T, T),
        ref_direction: (T, T, T),
        a_radius: T,
        b_radius: T,
        c_radius: T,
    ) -> Option<Self>
    where
        Self: Sized;

    fn new_standard(center: (T, T, T), a_radius: T, b_radius: T, c_radius: T) -> Option<Self>
    where
        Self: Sized;

    fn unit_ellipsoid() -> Self
    where
        Self: Sized;

    fn from_radii(center: (T, T, T), a: T, b: T, c: T) -> Option<Self>
    where
        Self: Sized;

    fn from_bounding_box(min: (T, T, T), max: (T, T, T)) -> Option<Self>
    where
        Self: Sized;

    fn new_sphere(
        center: (T, T, T),
        axis: (T, T, T),
        ref_direction: (T, T, T),
        radius: T,
    ) -> Option<Self>
    where
        Self: Sized;
}

pub trait EllipsoidalSolid3DProperties<T: Scalar> {
    fn center(&self) -> (T, T, T);
    fn a_radius(&self) -> T;
    fn b_radius(&self) -> T;
    fn c_radius(&self) -> T;
    fn axis(&self) -> (T, T, T);
    fn ref_direction(&self) -> (T, T, T);
    fn radii(&self) -> (T, T, T);
    fn is_sphere(&self) -> bool;
    fn is_unit_ellipsoid(&self) -> bool;
    fn is_centered_at_origin(&self) -> bool;
}

pub trait EllipsoidalSolid3DDerived<T: Scalar> {
    fn volume(&self) -> T;
    fn surface_area(&self) -> T;
    fn bounding_box(&self) -> ((T, T, T), (T, T, T));
    fn is_degenerate(&self) -> bool;
}

pub trait EllipsoidalSolid3DContainment<T: Scalar> {
    fn contains_point(&self, point: (T, T, T)) -> bool;
    fn is_on_surface(&self, point: (T, T, T)) -> bool;
}

pub trait EllipsoidalSolid3DDistance<T: Scalar> {
    fn distance_to_surface(&self, point: (T, T, T)) -> T;
}

pub trait EllipsoidalSolid3DProjection<T: Scalar> {
    fn closest_point_on_surface(&self, point: (T, T, T)) -> (T, T, T);
}

pub trait EllipsoidalSolid3DCore<T: Scalar>:
    EllipsoidalSolid3DConstructor<T> + EllipsoidalSolid3DProperties<T>
{
}

impl<T: Scalar, Solid> EllipsoidalSolid3DCore<T> for Solid where
    Solid: EllipsoidalSolid3DConstructor<T> + EllipsoidalSolid3DProperties<T>
{
}
