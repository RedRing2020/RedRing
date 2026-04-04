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

    fn unit_ellipsoid_surface() -> Self
    where
        Self: Sized;

    fn from_semi_axes(center: (T, T, T), a: T, b: T, c: T) -> Option<Self>
    where
        Self: Sized;

    fn from_bounding_box(min: (T, T, T), max: (T, T, T)) -> Option<Self>
    where
        Self: Sized;

    fn oblate_spheroid(center: (T, T, T), equatorial_radius: T, polar_radius: T) -> Option<Self>
    where
        Self: Sized;
}

pub trait EllipsoidalSurface3DProperties<T: Scalar> {
    fn center(&self) -> (T, T, T);
    fn semi_axis_a(&self) -> T;
    fn semi_axis_b(&self) -> T;
    fn semi_axis_c(&self) -> T;
    fn axis(&self) -> (T, T, T);
    fn ref_direction(&self) -> (T, T, T);
    fn eccentricity(&self) -> T;
    fn is_sphere(&self) -> bool;
    fn is_oblate(&self) -> bool;
}

pub trait EllipsoidalSurface3DEvaluation<T: Scalar> {
    fn point_at_uv(&self, u: T, v: T) -> (T, T, T);
    fn normal_at(&self, u: T, v: T) -> (T, T, T);
    fn point_at_spherical(&self, theta: T, phi: T) -> (T, T, T);
}

pub trait EllipsoidalSurface3DDerived<T: Scalar> {
    fn surface_area(&self) -> T;
    fn bounding_box(&self) -> ((T, T, T), (T, T, T));
    fn volume(&self) -> T;
    fn surface_area_knud_thomsen(&self) -> T;
}

pub trait EllipsoidalSurface3DDistance<T: Scalar> {
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}

pub trait EllipsoidalSurface3DCore<T: Scalar>:
    EllipsoidalSurface3DConstructor<T> + EllipsoidalSurface3DProperties<T>
{
}

impl<T: Scalar, Surface> EllipsoidalSurface3DCore<T> for Surface where
    Surface: EllipsoidalSurface3DConstructor<T> + EllipsoidalSurface3DProperties<T>
{
}
