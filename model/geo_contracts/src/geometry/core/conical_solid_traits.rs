//! ConicalSolid Core Traits - 円錐ソリッドの3つのCore機能統合
//!
//! Foundation ハイブリッド実装方針に基づく
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用

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

pub trait ConicalSolid3DMeasure<T: Scalar> {
    fn volume(&self) -> T;
    fn surface_area(&self) -> T;
    fn contains_point(&self, point: (T, T, T)) -> bool;
    fn distance_to_point(&self, point: (T, T, T)) -> T;
    fn point_at_conical(&self, r_ratio: T, theta: T, h_ratio: T) -> (T, T, T);
    fn bounding_box(&self) -> ((T, T, T), (T, T, T));
    fn closest_point_on_surface(&self, point: (T, T, T)) -> (T, T, T);
    fn contains_point_tolerance(&self, point: (T, T, T), tolerance: T) -> bool;
}

pub trait ConicalSolid3DCore<T: Scalar>:
    ConicalSolid3DConstructor<T> + ConicalSolid3DProperties<T> + ConicalSolid3DMeasure<T>
{
}
