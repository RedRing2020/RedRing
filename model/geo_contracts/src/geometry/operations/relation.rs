//! Geometry relation contracts.

use crate::Scalar;

pub trait Aabb2DRelation<T: Scalar> {
    type Point2D;

    fn contains_point(&self, point: &Self::Point2D) -> bool;
    fn contains_bbox(&self, other: &Self) -> bool;
    fn intersects(&self, other: &Self) -> bool;
}

pub trait Aabb3DRelation<T: Scalar> {
    type Point3D;

    fn contains_point(&self, point: &Self::Point3D) -> bool;
    fn contains_bbox(&self, other: &Self) -> bool;
    fn intersects(&self, other: &Self) -> bool;
}
