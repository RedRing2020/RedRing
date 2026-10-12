//! Point contracts.
//! Core contract definitions only; transform implementations live outside contracts.

use crate::Scalar;

pub trait Point2DConstructor<T: Scalar> {
    fn new(x: T, y: T) -> Self;
    fn origin() -> Self;
    fn from_tuple(coords: (T, T)) -> Self;

    fn from_polar(r: T, theta: T) -> Self;
}

pub trait Point3DConstructor<T: Scalar> {
    fn new(x: T, y: T, z: T) -> Self;
    fn origin() -> Self;
    fn from_tuple(coords: (T, T, T)) -> Self;

    fn from_spherical(r: T, theta: T, phi: T) -> Self;
}

pub trait Point2DProperties<T: Scalar> {
    fn x(&self) -> T;
    fn y(&self) -> T;
    fn to_tuple(&self) -> (T, T);
    fn position(&self) -> Self
    where
        Self: Sized + Clone,
    {
        self.clone()
    }

    fn dimension(&self) -> u32 {
        0
    }
}

pub trait Point3DProperties<T: Scalar> {
    fn x(&self) -> T;
    fn y(&self) -> T;
    fn z(&self) -> T;
    fn to_tuple(&self) -> (T, T, T);

    fn position(&self) -> Self
    where
        Self: Sized + Clone,
    {
        self.clone()
    }

    fn dimension(&self) -> u32 {
        0
    }
}

pub trait Point2DDistance<T: Scalar> {
    fn distance_to(&self, other: &Self) -> T;
    fn distance_squared_to(&self, other: &Self) -> T;
    fn norm_squared(&self) -> T;
    fn manhattan_distance_to(&self, other: &Self) -> T;
    fn chebyshev_distance_to(&self, other: &Self) -> T;
}

pub trait Point3DDistance<T: Scalar> {
    fn distance_to(&self, other: &Self) -> T;
    fn distance_squared_to(&self, other: &Self) -> T;
    fn norm_squared(&self) -> T;
    fn manhattan_distance_to(&self, other: &Self) -> T;
    fn chebyshev_distance_to(&self, other: &Self) -> T;
}

pub trait Point2DInterpolation<T: Scalar> {
    fn midpoint(&self, other: &Self) -> Self;
    fn lerp(&self, other: &Self, t: T) -> Self;
}

pub trait Point3DInterpolation<T: Scalar> {
    fn midpoint(&self, other: &Self) -> Self;
    fn lerp(&self, other: &Self, t: T) -> Self;
}
