//! Vector contracts.
//! Core contract definitions only; transform implementations live outside contracts.

use crate::Scalar;

pub trait Vector2DConstructor<T: Scalar> {
    fn new(x: T, y: T) -> Self;
    fn zero() -> Self;
    fn unit_x() -> Self;
    fn unit_y() -> Self;
    fn from_tuple(components: (T, T)) -> Self;
    fn from_polar(magnitude: T, angle: T) -> Self;
    fn from_vector(other: &Self) -> Self;
}

pub trait Vector3DConstructor<T: Scalar> {
    fn new(x: T, y: T, z: T) -> Self;
    fn zero() -> Self;
    fn unit_x() -> Self;
    fn unit_y() -> Self;
    fn unit_z() -> Self;
    fn from_tuple(components: (T, T, T)) -> Self;
    fn from_spherical(magnitude: T, azimuth: T, elevation: T) -> Self;
    fn from_vector(other: &Self) -> Self;
}

pub trait Vector2DProperties<T: Scalar> {
    fn x(&self) -> T;
    fn y(&self) -> T;
    fn components(&self) -> [T; 2];
    fn to_tuple(&self) -> (T, T);
    fn dimension(&self) -> u32 {
        1
    }
}

pub trait Vector3DProperties<T: Scalar> {
    fn x(&self) -> T;
    fn y(&self) -> T;
    fn z(&self) -> T;
    fn components(&self) -> [T; 3];
    fn to_tuple(&self) -> (T, T, T);
    fn dimension(&self) -> u32 {
        1
    }
}

pub trait Vector2DMetric<T: Scalar> {
    fn length(&self) -> T;
    fn length_squared(&self) -> T;
    fn normalize(&self) -> Self;
    fn try_normalize(&self) -> Option<Self>
    where
        Self: Sized;
    fn magnitude(&self) -> T;
    fn distance_to(&self, other: &Self) -> T;
    fn distance_squared_to(&self, other: &Self) -> T;
    fn is_zero(&self) -> bool {
        self.length_squared().is_zero()
    }
}

pub trait Vector3DMetric<T: Scalar> {
    fn length(&self) -> T;
    fn length_squared(&self) -> T;
    fn normalize(&self) -> Self;
    fn try_normalize(&self) -> Option<Self>
    where
        Self: Sized;
    fn magnitude(&self) -> T;
    fn distance_to(&self, other: &Self) -> T;
    fn distance_squared_to(&self, other: &Self) -> T;
    fn is_zero(&self) -> bool {
        self.length_squared().is_zero()
    }
}

pub trait Vector2DProduct<T: Scalar> {
    fn dot(&self, other: &Self) -> T;
}

pub trait Vector3DProduct<T: Scalar> {
    fn dot(&self, other: &Self) -> T;
}

pub trait Vector2DRelation<T: Scalar> {
    fn angle_to(&self, other: &Self) -> Option<T>;
    fn is_parallel_to(&self, other: &Self) -> bool;
    fn is_perpendicular_to(&self, other: &Self) -> bool;
}

pub trait Vector3DRelation<T: Scalar> {
    fn angle_to(&self, other: &Self) -> Option<T>;
    fn is_parallel_to(&self, other: &Self) -> bool;
    fn is_perpendicular_to(&self, other: &Self) -> bool;
}

pub trait Vector2DProjection<T: Scalar> {
    fn project_onto(&self, other: &Self) -> Option<Self>
    where
        Self: Sized;
}

pub trait Vector3DProjection<T: Scalar> {
    fn project_onto(&self, other: &Self) -> Option<Self>
    where
        Self: Sized;
}
