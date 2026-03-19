//! TorusSolid Core Traits - トーラスソリッドの3つのCore機能統合
//!
//! Foundation ハイブリッド実装方針に基づく
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用

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

pub trait TorusSolid3DMeasure<T: Scalar> {
    fn volume(&self) -> T;
    fn surface_area(&self) -> T;
    fn contains_point(&self, point: (T, T, T)) -> bool;
    fn distance_to_point(&self, point: (T, T, T)) -> T;
    fn point_at_toroidal(&self, u: T, v: T) -> (T, T, T);
    fn bounding_box(&self) -> ((T, T, T), (T, T, T));
    fn closest_point_on_surface(&self, point: (T, T, T)) -> (T, T, T);
    fn is_self_intersecting(&self) -> bool;
}

pub trait TorusSolid3DCore<T: Scalar>:
    TorusSolid3DConstructor<T> + TorusSolid3DProperties<T> + TorusSolid3DMeasure<T>
{
}
