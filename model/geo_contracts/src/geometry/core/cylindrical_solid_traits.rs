//! CylindricalSolid trait定義を capability taxonomy に沿って分離する。

use analysis::abstract_types::Scalar;

pub trait CylindricalSolid3DConstructor<T: Scalar> {
    fn new(
        center: (T, T, T),
        axis: (T, T, T),
        ref_direction: (T, T, T),
        radius: T,
        height: T,
    ) -> Option<Self>
    where
        Self: Sized;

    fn new_standard(center: (T, T, T), radius: T, height: T) -> Option<Self>
    where
        Self: Sized;

    fn unit_cylinder() -> Self
    where
        Self: Sized;

    fn from_axis_and_radius(
        start_point: (T, T, T),
        end_point: (T, T, T),
        radius: T,
    ) -> Option<Self>
    where
        Self: Sized;

    fn from_diameter(center: (T, T, T), axis: (T, T, T), diameter: T, height: T) -> Option<Self>
    where
        Self: Sized;

    fn from_two_points_and_radius(p1: (T, T, T), p2: (T, T, T), radius: T) -> Option<Self>
    where
        Self: Sized;
}

pub trait CylindricalSolid3DProperties<T: Scalar> {
    fn center(&self) -> (T, T, T);
    fn radius(&self) -> T;
    fn height(&self) -> T;
    fn axis(&self) -> (T, T, T);
    fn ref_direction(&self) -> (T, T, T);
    fn diameter(&self) -> T;
    fn top_center(&self) -> (T, T, T);
    fn lateral_surface_area(&self) -> T;
    fn base_area(&self) -> T;
}

pub trait CylindricalSolid3DDerived<T: Scalar> {
    fn volume(&self) -> T;
    fn surface_area(&self) -> T;
    fn bounding_box(&self) -> ((T, T, T), (T, T, T));
}

pub trait CylindricalSolid3DContainment<T: Scalar> {
    fn contains_point(&self, point: (T, T, T)) -> bool;
}

pub trait CylindricalSolid3DDistance<T: Scalar> {
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}

pub trait CylindricalSolid3DEvaluation<T: Scalar> {
    fn point_at_cylindrical(&self, r: T, theta: T, z: T) -> (T, T, T);
}

pub trait CylindricalSolid3DProjection<T: Scalar> {
    fn closest_point_on_surface(&self, point: (T, T, T)) -> (T, T, T);
}

pub trait CylindricalSolid3DCore<T: Scalar>:
    CylindricalSolid3DConstructor<T> + CylindricalSolid3DProperties<T>
{
}

impl<T: Scalar, Solid> CylindricalSolid3DCore<T> for Solid where
    Solid: CylindricalSolid3DConstructor<T> + CylindricalSolid3DProperties<T>
{
}
