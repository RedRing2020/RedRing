//! SphericalSolid trait定義を capability taxonomy に沿って分離する。

use analysis::abstract_types::Scalar;

pub trait SphericalSolid3DConstructor<T: Scalar> {
    fn new(center: (T, T, T), axis: (T, T, T), ref_direction: (T, T, T), radius: T) -> Option<Self>
    where
        Self: Sized;

    fn new_standard(center: (T, T, T), radius: T) -> Option<Self>
    where
        Self: Sized;

    fn unit_sphere() -> Self
    where
        Self: Sized;

    fn from_diameter(center: (T, T, T), diameter: T) -> Option<Self>
    where
        Self: Sized;

    fn from_bounding_box(min: (T, T, T), max: (T, T, T)) -> Option<Self>
    where
        Self: Sized;

    fn from_four_points(p1: (T, T, T), p2: (T, T, T), p3: (T, T, T), p4: (T, T, T)) -> Option<Self>
    where
        Self: Sized;
}

pub trait SphericalSolid3DProperties<T: Scalar> {
    fn center(&self) -> (T, T, T);
    fn radius(&self) -> T;
    fn axis(&self) -> (T, T, T);
    fn ref_direction(&self) -> (T, T, T);
    fn diameter(&self) -> T;
    fn is_unit_sphere(&self) -> bool;
    fn circumference(&self) -> T;
    fn is_centered_at_origin(&self) -> bool;
}

pub trait SphericalSolid3DDerived<T: Scalar> {
    fn volume(&self) -> T;
    fn surface_area(&self) -> T;
    fn bounding_box(&self) -> ((T, T, T), (T, T, T));
}

pub trait SphericalSolid3DContainment<T: Scalar> {
    fn contains_point(&self, point: (T, T, T)) -> bool;
}

pub trait SphericalSolid3DDistance<T: Scalar> {
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}

pub trait SphericalSolid3DEvaluation<T: Scalar> {
    fn point_at_latlong(&self, latitude: T, longitude: T) -> (T, T, T);
}

pub trait SphericalSolid3DProjection<T: Scalar> {
    fn closest_point_on_surface(&self, point: (T, T, T)) -> (T, T, T);
}

pub trait SphericalSolid3DCore<T: Scalar>:
    SphericalSolid3DConstructor<T> + SphericalSolid3DProperties<T>
{
}

impl<T: Scalar, Solid> SphericalSolid3DCore<T> for Solid where
    Solid: SphericalSolid3DConstructor<T> + SphericalSolid3DProperties<T>
{
}
