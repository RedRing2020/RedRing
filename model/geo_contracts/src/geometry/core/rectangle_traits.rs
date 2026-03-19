//! Rectangle Core Traits - 矩形の基本機能トレイト
//!
//! Foundation Pattern Phase 1 実装

use analysis::abstract_types::Scalar;

pub trait Rect2DConstructor<T: Scalar>: Sized {
    fn new(origin: (T, T), width: T, height: T) -> Option<Self>;
    fn from_corners(min: (T, T), max: (T, T)) -> Option<Self>;
    fn resize(&mut self, width: T, height: T) -> bool;
}

pub trait Rect2DProperties<T: Scalar> {
    fn origin(&self) -> (T, T);
    fn width(&self) -> T;
    fn height(&self) -> T;
    fn max_corner(&self) -> (T, T);
    fn center(&self) -> (T, T);
}

pub trait Rect2DMeasure<T: Scalar> {
    fn contains_point(&self, point: (T, T)) -> bool;
    fn area(&self) -> T;
    fn perimeter(&self) -> T;
    fn is_valid(&self) -> bool;
}

pub trait Rect2DCore<T: Scalar>:
    Rect2DConstructor<T> + Rect2DProperties<T> + Rect2DMeasure<T>
{
}

pub trait Rect3DConstructor<T: Scalar>: Sized {
    fn new(
        origin: (T, T, T),
        u_axis: (T, T, T),
        v_axis: (T, T, T),
        width: T,
        height: T,
    ) -> Option<Self>;
    fn resize(&mut self, width: T, height: T) -> bool;
}

pub trait Rect3DProperties<T: Scalar> {
    fn origin(&self) -> (T, T, T);
    fn u_axis(&self) -> (T, T, T);
    fn v_axis(&self) -> (T, T, T);
    fn normal(&self) -> (T, T, T);
    fn width(&self) -> T;
    fn height(&self) -> T;
    fn center(&self) -> (T, T, T);
}

pub trait Rect3DMeasure<T: Scalar> {
    fn contains_point(&self, point: (T, T, T), tolerance: T) -> bool;
    fn area(&self) -> T;
    fn corners(&self) -> [(T, T, T); 4];
    fn distance_to_plane(&self, point: (T, T, T)) -> T;
    fn is_valid(&self) -> bool;
}

pub trait Rect3DCore<T: Scalar>:
    Rect3DConstructor<T> + Rect3DProperties<T> + Rect3DMeasure<T>
{
}
