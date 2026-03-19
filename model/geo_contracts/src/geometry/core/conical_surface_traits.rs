//! ConicalSurface Core Traits - 円錐サーフェスの3つのCore機能統合
//!
//! Foundation ハイブリッド実装方針に基づく
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用

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

pub trait ConicalSurface3DMeasure<T: Scalar> {
    fn surface_area(&self) -> T;
    fn point_at_uv(&self, u: T, v: T) -> (T, T, T);
    fn normal_at(&self, u: T, v: T) -> (T, T, T);
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}

pub trait ConicalSurface3DCore<T: Scalar>:
    ConicalSurface3DConstructor<T> + ConicalSurface3DProperties<T> + ConicalSurface3DMeasure<T>
{
}
