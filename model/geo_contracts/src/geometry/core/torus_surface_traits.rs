//! TorusSurface trait定義を capability taxonomy に沿って分離する。

use analysis::abstract_types::Scalar;

pub trait TorusSurface3DConstructor<T: Scalar> {
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

    fn unit_torus_surface() -> Self
    where
        Self: Sized;
}

pub trait TorusSurface3DProperties<T: Scalar> {
    fn center(&self) -> (T, T, T);
    fn major_radius(&self) -> T;
    fn minor_radius(&self) -> T;
    fn axis(&self) -> (T, T, T);
    fn ref_direction(&self) -> (T, T, T);
    fn tube_diameter(&self) -> T;
}

pub trait TorusSurface3DEvaluation<T: Scalar> {
    fn point_at_uv(&self, u: T, v: T) -> (T, T, T);
    fn normal_at(&self, u: T, v: T) -> (T, T, T);
}

pub trait TorusSurface3DDerived<T: Scalar> {
    fn surface_area(&self) -> T;
}

pub trait TorusSurface3DDistance<T: Scalar> {
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}

pub trait TorusSurface3DCore<T: Scalar>:
    TorusSurface3DConstructor<T> + TorusSurface3DProperties<T>
{
}

impl<T: Scalar, Surface> TorusSurface3DCore<T> for Surface where
    Surface: TorusSurface3DConstructor<T> + TorusSurface3DProperties<T>
{
}
