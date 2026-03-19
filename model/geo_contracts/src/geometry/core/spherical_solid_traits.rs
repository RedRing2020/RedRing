//! SphericalSolid Core Traits - 球ソリッドの3つのCore機能統合
//!
//! Foundation ハイブリッド実装方針に基づく
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用

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

pub trait SphericalSolid3DMeasure<T: Scalar> {
    fn volume(&self) -> T;
    fn surface_area(&self) -> T;
    fn contains_point(&self, point: (T, T, T)) -> bool;
    fn distance_to_point(&self, point: (T, T, T)) -> T;
    fn point_at_latlong(&self, latitude: T, longitude: T) -> (T, T, T);
    fn bounding_box(&self) -> ((T, T, T), (T, T, T));
    fn closest_point_on_surface(&self, point: (T, T, T)) -> (T, T, T);
    fn intersects_sphere(&self, other_center: (T, T, T), other_radius: T) -> bool;
}

pub trait SphericalSolid3DCore<T: Scalar>:
    SphericalSolid3DConstructor<T> + SphericalSolid3DProperties<T> + SphericalSolid3DMeasure<T>
{
}
