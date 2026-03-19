//! TorusSurface Core Traits - トーラスサーフェスの3つのCore機能統合
//!
//! Foundation ハイブリッド実装方針に基づく
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用

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

pub trait TorusSurface3DMeasure<T: Scalar> {
    fn surface_area(&self) -> T;
    fn point_at_uv(&self, u: T, v: T) -> (T, T, T);
    fn normal_at(&self, u: T, v: T) -> (T, T, T);
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}

pub trait TorusSurface3DCore<T: Scalar>:
    TorusSurface3DConstructor<T> + TorusSurface3DProperties<T> + TorusSurface3DMeasure<T>
{
}
