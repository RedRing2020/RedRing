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

pub trait Rect2DMeasure<T: Scalar>: Rect2DContainment<T> + Rect2DDerived<T> {
    fn contains_point(&self, point: (T, T)) -> bool {
        <Self as Rect2DContainment<T>>::contains_point(self, point)
    }

    fn area(&self) -> T {
        <Self as Rect2DDerived<T>>::area(self)
    }

    fn perimeter(&self) -> T {
        <Self as Rect2DDerived<T>>::perimeter(self)
    }

    fn is_valid(&self) -> bool {
        <Self as Rect2DDerived<T>>::is_valid(self)
    }
}

pub trait Rect2DContainment<T: Scalar> {
    fn contains_point(&self, point: (T, T)) -> bool;
}

pub trait Rect2DDerived<T: Scalar> {
    fn area(&self) -> T;
    fn perimeter(&self) -> T;
    fn is_valid(&self) -> bool;
}

impl<T: Scalar, R> Rect2DMeasure<T> for R where R: Rect2DContainment<T> + Rect2DDerived<T> {}

pub trait Rect2DCore<T: Scalar>: Rect2DConstructor<T> + Rect2DProperties<T> {}

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

pub trait Rect3DMeasure<T: Scalar>:
    Rect3DContainment<T> + Rect3DDerived<T> + Rect3DEvaluation<T>
{
    fn contains_point(&self, point: (T, T, T), tolerance: T) -> bool {
        <Self as Rect3DContainment<T>>::contains_point(self, point, tolerance)
    }

    fn area(&self) -> T {
        <Self as Rect3DDerived<T>>::area(self)
    }

    fn corners(&self) -> [(T, T, T); 4] {
        <Self as Rect3DDerived<T>>::corners(self)
    }

    fn distance_to_plane(&self, point: (T, T, T)) -> T {
        <Self as Rect3DEvaluation<T>>::distance_to_plane(self, point)
    }

    fn is_valid(&self) -> bool {
        <Self as Rect3DDerived<T>>::is_valid(self)
    }
}

pub trait Rect3DContainment<T: Scalar> {
    fn contains_point(&self, point: (T, T, T), tolerance: T) -> bool;
}

pub trait Rect3DDerived<T: Scalar> {
    fn area(&self) -> T;
    fn corners(&self) -> [(T, T, T); 4];
    fn is_valid(&self) -> bool;
}

pub trait Rect3DEvaluation<T: Scalar> {
    fn distance_to_plane(&self, point: (T, T, T)) -> T;
}

impl<T: Scalar, R> Rect3DMeasure<T> for R where
    R: Rect3DContainment<T> + Rect3DDerived<T> + Rect3DEvaluation<T>
{
}

pub trait Rect3DCore<T: Scalar>: Rect3DConstructor<T> + Rect3DProperties<T> {}
