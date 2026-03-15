//! AABB traits defined in geo_core.

use analysis::abstract_types::Scalar;

pub trait Aabb2DTrait<T: Scalar> {
    type Point2D;

    fn min(&self) -> Self::Point2D;
    fn max(&self) -> Self::Point2D;
    fn width(&self) -> T;
    fn height(&self) -> T;
    fn area(&self) -> T;
    fn center(&self) -> Self::Point2D;
    fn contains_point(&self, point: &Self::Point2D) -> bool;
    fn contains_bbox(&self, other: &Self) -> bool;
    fn is_valid(&self) -> bool;
}

pub trait Aabb3DTrait<T: Scalar> {
    type Point3D;

    fn min(&self) -> Self::Point3D;
    fn max(&self) -> Self::Point3D;
    fn width(&self) -> T;
    fn height(&self) -> T;
    fn depth(&self) -> T;
    fn volume(&self) -> T;
    fn center(&self) -> Self::Point3D;
    fn contains_point(&self, point: &Self::Point3D) -> bool;
    fn contains_bbox(&self, other: &Self) -> bool;
    fn intersects(&self, other: &Self) -> bool;
    fn is_valid(&self) -> bool;
}
