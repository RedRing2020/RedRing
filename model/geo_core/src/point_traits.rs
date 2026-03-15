//! Point core traits defined in geo_core.

use analysis::abstract_types::Scalar;
use analysis::linalg::vector::{Vector2, Vector3};

pub trait Point2DConstructor<T: Scalar> {
    fn new(x: T, y: T) -> Self;
    fn origin() -> Self;
    fn from_tuple(coords: (T, T)) -> Self;
    fn from_analysis_vector(vector: &Vector2<T>) -> Self;
    fn from_point(other: &Self) -> Self;
    fn from_polar(r: T, theta: T) -> Self;
}

pub trait Point3DConstructor<T: Scalar> {
    fn new(x: T, y: T, z: T) -> Self;
    fn origin() -> Self;
    fn from_tuple(coords: (T, T, T)) -> Self;
    fn from_analysis_vector(vector: &Vector3<T>) -> Self;
    fn from_point(other: &Self) -> Self;
    fn from_spherical(r: T, theta: T, phi: T) -> Self;
}

pub trait Point2DProperties<T: Scalar> {
    fn x(&self) -> T;
    fn y(&self) -> T;
    fn coords(&self) -> [T; 2];
    fn to_tuple(&self) -> (T, T);
    fn to_analysis_vector(&self) -> Vector2<T>;
    fn polar_radius(&self) -> T;
}

pub trait Point3DProperties<T: Scalar> {
    fn x(&self) -> T;
    fn y(&self) -> T;
    fn z(&self) -> T;
    fn coords(&self) -> [T; 3];
    fn to_tuple(&self) -> (T, T, T);
    fn to_analysis_vector(&self) -> Vector3<T>;
}

pub trait Point2DMeasure<T: Scalar> {
    fn distance_to(&self, other: &Self) -> T;
    fn distance_squared_to(&self, other: &Self) -> T;
    fn distance_from_origin(&self) -> T;
    fn norm_squared(&self) -> T;
    fn midpoint(&self, other: &Self) -> Self;
    fn lerp(&self, other: &Self, t: T) -> Self;
    fn manhattan_distance_to(&self, other: &Self) -> T;
    fn chebyshev_distance_to(&self, other: &Self) -> T;
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
}

pub trait Point2DCore<T: Scalar>:
    Point2DConstructor<T> + Point2DProperties<T> + Point2DMeasure<T>
{
}

pub trait Point3DCore<T: Scalar>:
    Point3DConstructor<T> + Point3DProperties<T> + Point3DMeasure<T>
{
}
