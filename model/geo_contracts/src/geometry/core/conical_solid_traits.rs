//! ConicalSolid trait定義を capability taxonomy に沿って分離する。

use analysis::abstract_types::Scalar;

pub trait ConicalSolid3DConstructor<T: Scalar> {
    fn new(
        apex: (T, T, T),
        base_center: (T, T, T),
        axis: (T, T, T),
        ref_direction: (T, T, T),
        radius: T,
        height: T,
    ) -> Option<Self>
    where
        Self: Sized;

    fn new_standard(base_center: (T, T, T), radius: T, height: T) -> Option<Self>
    where
        Self: Sized;

    fn unit_cone() -> Self
    where
        Self: Sized;

    fn from_apex_and_base_circle(
        apex: (T, T, T),
        base_center: (T, T, T),
        base_radius: T,
    ) -> Option<Self>
    where
        Self: Sized;

    fn from_apex_angle(apex: (T, T, T), axis: (T, T, T), height: T, half_angle: T) -> Option<Self>
    where
        Self: Sized;

    fn frustum(
        base_center: (T, T, T),
        top_center: (T, T, T),
        base_radius: T,
        top_radius: T,
    ) -> Option<Self>
    where
        Self: Sized;
}

pub trait ConicalSolid3DProperties<T: Scalar> {
    fn apex(&self) -> (T, T, T);
    fn base_center(&self) -> (T, T, T);
    fn radius(&self) -> T;
    fn height(&self) -> T;
    fn axis(&self) -> (T, T, T);
    fn ref_direction(&self) -> (T, T, T);
    fn slant_height(&self) -> T;
    fn half_angle(&self) -> T;
    fn lateral_surface_area(&self) -> T;
    fn base_area(&self) -> T;
}

pub trait ConicalSolid3DDerived<T: Scalar> {
    fn volume(&self) -> T;
    fn surface_area(&self) -> T;
    fn bounding_box(&self) -> ((T, T, T), (T, T, T));
}

pub trait ConicalSolid3DContainment<T: Scalar> {
    fn contains_point(&self, point: (T, T, T)) -> bool;
    fn contains_point_tolerance(&self, point: (T, T, T), tolerance: T) -> bool;
}

pub trait ConicalSolid3DDistance<T: Scalar> {
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}

pub trait ConicalSolid3DEvaluation<T: Scalar> {
    fn point_at_conical(&self, r_ratio: T, theta: T, h_ratio: T) -> (T, T, T);
}

pub trait ConicalSolid3DProjection<T: Scalar> {
    fn closest_point_on_surface(&self, point: (T, T, T)) -> (T, T, T);
}

pub trait ConicalSolid3DCore<T: Scalar>:
    ConicalSolid3DConstructor<T> + ConicalSolid3DProperties<T>
{
}

impl<T: Scalar, Solid> ConicalSolid3DCore<T> for Solid where
    Solid: ConicalSolid3DConstructor<T> + ConicalSolid3DProperties<T>
{
}
