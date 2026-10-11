//! SphericalSurface trait定義を capability taxonomy に沿って分離する。

use analysis::abstract_types::Scalar;

pub trait SphericalSurface3DConstructor<T: Scalar> {
    fn new(center: (T, T, T), axis: (T, T, T), ref_direction: (T, T, T), radius: T) -> Option<Self>
    where
        Self: Sized;

    fn new_standard(center: (T, T, T), radius: T) -> Option<Self>
    where
        Self: Sized;
}

pub trait SphericalSurface3DProperties<T: Scalar> {
    fn center(&self) -> (T, T, T);
    fn radius(&self) -> T;
    fn axis(&self) -> (T, T, T);
    fn ref_direction(&self) -> (T, T, T);
    fn diameter(&self) -> T;
}

pub trait SphericalSurface3DEvaluation<T: Scalar> {
    fn point_at_uv(&self, u: T, v: T) -> (T, T, T);
    fn normal_at(&self, u: T, v: T) -> (T, T, T);
}

pub trait SphericalSurface3DDerived<T: Scalar> {
    fn surface_area(&self) -> T;
    fn bounding_box(&self) -> ((T, T, T), (T, T, T));
}

pub trait SphericalSurface3DDistance<T: Scalar> {
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}

pub trait SphericalSurface3DProjection<T: Scalar> {
    fn closest_point(&self, point: (T, T, T)) -> (T, T, T);
}
