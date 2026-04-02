//! Vector measure capability traits.

use crate::Scalar;

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
