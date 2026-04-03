//! AABB contracts.
//! Core contract definitions only; concrete types live outside contracts.

use crate::Scalar;

pub trait Aabb2DProperties<T: Scalar> {
    type Point2D;

    fn min(&self) -> Self::Point2D;
    fn max(&self) -> Self::Point2D;
}

pub trait Aabb2DDerived<T: Scalar>: Aabb2DProperties<T> {
    fn width(&self) -> T;
    fn height(&self) -> T;
    fn area(&self) -> T;
    fn center(&self) -> Self::Point2D;
    fn is_valid(&self) -> bool;
}

pub trait Aabb3DProperties<T: Scalar> {
    type Point3D;

    fn min(&self) -> Self::Point3D;
    fn max(&self) -> Self::Point3D;
}

pub trait Aabb3DDerived<T: Scalar>: Aabb3DProperties<T> {
    fn width(&self) -> T;
    fn height(&self) -> T;
    fn depth(&self) -> T;
    fn volume(&self) -> T;
    fn center(&self) -> Self::Point3D;
    fn is_valid(&self) -> bool;
}
