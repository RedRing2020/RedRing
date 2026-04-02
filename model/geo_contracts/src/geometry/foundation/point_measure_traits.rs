//! Point measure capability traits.

use crate::Scalar;

pub trait Point2DMeasure<T: Scalar> {
    fn distance_to(&self, other: &Self) -> T;
    fn distance_squared_to(&self, other: &Self) -> T;
    fn distance_from_origin(&self) -> T;
    fn norm_squared(&self) -> T;

    fn midpoint(&self, other: &Self) -> Self;
    fn lerp(&self, other: &Self, t: T) -> Self;
    fn manhattan_distance_to(&self, other: &Self) -> T;
    fn chebyshev_distance_to(&self, other: &Self) -> T;

    fn area(&self) -> Option<T> {
        None
    }

    fn length(&self) -> Option<T> {
        None
    }
}

pub trait Point3DMeasure<T: Scalar> {
    fn distance_to(&self, other: &Self) -> T;
    fn distance_squared_to(&self, other: &Self) -> T;
    fn distance_from_origin(&self) -> T;
    fn norm_squared(&self) -> T;

    fn midpoint(&self, other: &Self) -> Self;
    fn lerp(&self, other: &Self, t: T) -> Self;
    fn manhattan_distance_to(&self, other: &Self) -> T;
    fn chebyshev_distance_to(&self, other: &Self) -> T;

    fn area(&self) -> Option<T> {
        None
    }

    fn volume(&self) -> Option<T> {
        None
    }

    fn length(&self) -> Option<T> {
        None
    }
}
