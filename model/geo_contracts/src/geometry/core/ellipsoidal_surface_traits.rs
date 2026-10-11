//! EllipsoidalSurface trait定義を capability taxonomy に沿って分離する。

use analysis::abstract_types::Scalar;

pub trait EllipsoidalSurface3DConstructor<T: Scalar> {
    fn new(
        center: (T, T, T),
        axis: (T, T, T),
        ref_direction: (T, T, T),
        semi_axis_a: T,
        semi_axis_b: T,
        semi_axis_c: T,
    ) -> Option<Self>
    where
        Self: Sized;

    fn new_standard(
        center: (T, T, T),
        semi_axis_a: T,
        semi_axis_b: T,
        semi_axis_c: T,
    ) -> Option<Self>
    where
        Self: Sized;
}

pub trait EllipsoidalSurface3DProperties<T: Scalar> {
    fn center(&self) -> (T, T, T);
    fn axis(&self) -> (T, T, T);
    fn ref_direction(&self) -> (T, T, T);
}

pub trait EllipsoidalSurface3DEvaluation<T: Scalar> {
    fn point_at_uv(&self, u: T, v: T) -> (T, T, T);
    fn normal_at(&self, u: T, v: T) -> (T, T, T);
}

pub trait EllipsoidalSurface3DDerived<T: Scalar> {
    fn surface_area(&self) -> T;
    fn bounding_box(&self) -> ((T, T, T), (T, T, T));
    fn volume(&self) -> T;
}

pub trait EllipsoidalSurface3DDistance<T: Scalar> {
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}
