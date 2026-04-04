//! CylindricalSurface trait定義を capability taxonomy に沿って分離する。

use analysis::abstract_types::Scalar;

pub trait CylindricalSurface3DConstructor<T: Scalar> {
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

    fn unit_cylinder_surface() -> Self
    where
        Self: Sized;
}

pub trait CylindricalSurface3DProperties<T: Scalar> {
    fn center(&self) -> (T, T, T);
    fn radius(&self) -> T;
    fn height(&self) -> T;
    fn axis(&self) -> (T, T, T);
    fn ref_direction(&self) -> (T, T, T);
    fn diameter(&self) -> T;
}

pub trait CylindricalSurface3DEvaluation<T: Scalar> {
    fn point_at_uv(&self, u: T, v: T) -> (T, T, T);
    fn normal_at(&self, u: T, v: T) -> (T, T, T);
}

pub trait CylindricalSurface3DDerived<T: Scalar> {
    fn surface_area(&self) -> T;
}

pub trait CylindricalSurface3DDistance<T: Scalar> {
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}

pub trait CylindricalSurface3DCore<T: Scalar>:
    CylindricalSurface3DConstructor<T> + CylindricalSurface3DProperties<T>
{
}

impl<T: Scalar, Surface> CylindricalSurface3DCore<T> for Surface where
    Surface: CylindricalSurface3DConstructor<T> + CylindricalSurface3DProperties<T>
{
}
