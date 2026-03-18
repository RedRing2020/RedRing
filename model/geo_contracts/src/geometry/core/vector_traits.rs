//! Vector contracts.
//! Core contract definitions only; transform implementations live outside contracts.

use crate::Scalar;
use analysis::linalg::vector::{Vector2, Vector3};

pub trait Vector2DConstructor<T: Scalar> {
    fn new(x: T, y: T) -> Self;
    fn zero() -> Self;
    fn unit_x() -> Self;
    fn unit_y() -> Self;
    fn from_tuple(components: (T, T)) -> Self;
    fn from_analysis_vector(vector: &Vector2<T>) -> Self;
    fn from_array(components: [T; 2]) -> Self;
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
    fn from_analysis_vector(vector: &Vector3<T>) -> Self;
    fn from_array(components: [T; 3]) -> Self;
    fn from_spherical(magnitude: T, azimuth: T, elevation: T) -> Self;
    fn from_cylindrical(radial_distance: T, azimuth: T, height: T) -> Self;
    fn from_vector(other: &Self) -> Self;
}

pub trait Vector2DProperties<T: Scalar> {
    fn x(&self) -> T;
    fn y(&self) -> T;
    fn components(&self) -> [T; 2];
    fn to_tuple(&self) -> (T, T);
    fn to_analysis_vector(&self) -> Vector2<T>;
    fn length(&self) -> T;
    fn length_squared(&self) -> T;
    fn normalize(&self) -> Self;
    fn try_normalize(&self) -> Option<Self>
    where
        Self: Sized;

    fn is_zero(&self) -> bool {
        self.length_squared().is_zero()
    }

    fn is_unit(&self) -> bool {
        let len_sq = self.length_squared();
        (len_sq - T::ONE).abs() < T::EPSILON
    }

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
    fn to_analysis_vector(&self) -> Vector3<T>;
    fn length(&self) -> T;
    fn length_squared(&self) -> T;
    fn normalize(&self) -> Self;
    fn try_normalize(&self) -> Option<Self>
    where
        Self: Sized;

    fn is_zero(&self) -> bool {
        self.length_squared().is_zero()
    }

    fn is_unit(&self) -> bool {
        let len_sq = self.length_squared();
        (len_sq - T::ONE).abs() < T::EPSILON
    }

    fn dimension(&self) -> u32 {
        1
    }
}

// TODO(#318): Measure contracts are temporarily colocated and will be split by responsibility.
pub trait Vector2DMeasure<T: Scalar> {
    fn dot(&self, other: &Self) -> T;
    fn cross_2d(&self, other: &Self) -> T;
    fn angle_to(&self, other: &Self) -> Option<T>;
    fn distance_to(&self, other: &Self) -> T;
    fn distance_squared_to(&self, other: &Self) -> T;
    fn magnitude(&self) -> T;
    fn manhattan_distance(&self) -> T;
    fn is_parallel_to(&self, other: &Self) -> bool;
    fn is_perpendicular_to(&self, other: &Self) -> bool;
    fn project_onto(&self, other: &Self) -> Option<Self>
    where
        Self: Sized;

    fn area(&self) -> Option<T> {
        None
    }

    fn length(&self) -> Option<T> {
        Some(self.magnitude())
    }
}

pub trait Vector3DMeasure<T: Scalar> {
    fn dot(&self, other: &Self) -> T;
    fn cross_3d(&self, other: &Self) -> Self;
    fn angle_to(&self, other: &Self) -> Option<T>;
    fn distance_to(&self, other: &Self) -> T;
    fn distance_squared_to(&self, other: &Self) -> T;
    fn magnitude(&self) -> T;
    fn manhattan_distance(&self) -> T;
    fn is_parallel_to(&self, other: &Self) -> bool;
    fn is_perpendicular_to(&self, other: &Self) -> bool;
    fn project_onto(&self, other: &Self) -> Option<Self>
    where
        Self: Sized;
    fn project_onto_plane(&self, normal: &Self) -> Option<Self>
    where
        Self: Sized;

    fn area(&self) -> Option<T> {
        None
    }

    fn volume(&self) -> Option<T> {
        None
    }

    fn length(&self) -> Option<T> {
        Some(self.magnitude())
    }
}

pub trait Vector2DCore<T: Scalar>:
    Vector2DConstructor<T> + Vector2DProperties<T> + Vector2DMeasure<T>
{
}

pub trait Vector3DCore<T: Scalar>:
    Vector3DConstructor<T> + Vector3DProperties<T> + Vector3DMeasure<T>
{
}
