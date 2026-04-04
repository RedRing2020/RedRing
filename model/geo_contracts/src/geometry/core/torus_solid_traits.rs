//! TorusSolid trait定義を capability taxonomy に沿って分離する。

use analysis::abstract_types::Scalar;

pub trait TorusSolid3DConstructor<T: Scalar> {
    fn new(
        center: (T, T, T),
        axis: (T, T, T),
        ref_direction: (T, T, T),
        major_radius: T,
        minor_radius: T,
    ) -> Option<Self>
    where
        Self: Sized;

    fn new_standard(center: (T, T, T), major_radius: T, minor_radius: T) -> Option<Self>
    where
        Self: Sized;

    fn unit_torus() -> Self
    where
        Self: Sized;

    fn from_diameters(
        center: (T, T, T),
        axis: (T, T, T),
        major_diameter: T,
        minor_diameter: T,
    ) -> Option<Self>
    where
        Self: Sized;

    fn from_radii_and_axis(
        center: (T, T, T),
        axis: (T, T, T),
        major_radius: T,
        minor_radius: T,
    ) -> Option<Self>
    where
        Self: Sized;

    fn ring_torus(center: (T, T, T), axis: (T, T, T), radius: T) -> Option<Self>
    where
        Self: Sized;
}

pub trait TorusSolid3DProperties<T: Scalar> {
    fn center(&self) -> (T, T, T);
    fn major_radius(&self) -> T;
    fn minor_radius(&self) -> T;
    fn axis(&self) -> (T, T, T);
    fn ref_direction(&self) -> (T, T, T);
    fn tube_diameter(&self) -> T;
    fn aspect_ratio(&self) -> T;
    fn outer_radius(&self) -> T;
    fn inner_radius(&self) -> T;
}

pub trait TorusSolid3DDerived<T: Scalar> {
    fn volume(&self) -> T;
    fn surface_area(&self) -> T;
    fn bounding_box(&self) -> ((T, T, T), (T, T, T));
    fn is_self_intersecting(&self) -> bool;
}

pub trait TorusSolid3DContainment<T: Scalar> {
    fn contains_point(&self, point: (T, T, T)) -> bool;
}

pub trait TorusSolid3DDistance<T: Scalar> {
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}

pub trait TorusSolid3DEvaluation<T: Scalar> {
    fn point_at_toroidal(&self, u: T, v: T) -> (T, T, T);
}

pub trait TorusSolid3DProjection<T: Scalar> {
    fn closest_point_on_surface(&self, point: (T, T, T)) -> (T, T, T);
}

pub trait TorusSolid3DCore<T: Scalar>:
    TorusSolid3DConstructor<T> + TorusSolid3DProperties<T>
{
}

impl<T: Scalar, Solid> TorusSolid3DCore<T> for Solid where
    Solid: TorusSolid3DConstructor<T> + TorusSolid3DProperties<T>
{
}
