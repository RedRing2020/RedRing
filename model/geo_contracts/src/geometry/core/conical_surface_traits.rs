//! ConicalSurface trait定義を capability taxonomy に沿って分離する。

use analysis::abstract_types::Scalar;

pub trait ConicalSurface3DConstructor<T: Scalar> {
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

    fn unit_cone_surface() -> Self
    where
        Self: Sized;
}

pub trait ConicalSurface3DProperties<T: Scalar> {
    fn apex(&self) -> (T, T, T);
    fn base_center(&self) -> (T, T, T);
    fn radius(&self) -> T;
    fn height(&self) -> T;
    fn axis(&self) -> (T, T, T);
    fn ref_direction(&self) -> (T, T, T);
    fn slant_height(&self) -> T;
}

pub trait ConicalSurface3DEvaluation<T: Scalar> {
    fn point_at_uv(&self, u: T, v: T) -> (T, T, T);
    fn normal_at(&self, u: T, v: T) -> (T, T, T);
}

pub trait ConicalSurface3DDerived<T: Scalar> {
    fn surface_area(&self) -> T;
}

pub trait ConicalSurface3DDistance<T: Scalar> {
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}

pub trait ConicalSurface3DCore<T: Scalar>:
    ConicalSurface3DConstructor<T> + ConicalSurface3DProperties<T>
{
}

impl<T: Scalar, Surface> ConicalSurface3DCore<T> for Surface where
    Surface: ConicalSurface3DConstructor<T> + ConicalSurface3DProperties<T>
{
}
