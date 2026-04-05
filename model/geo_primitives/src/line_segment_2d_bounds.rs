//! LineSegment2D の Bounds 実装

use crate::LineSegment2D;
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb2D;

impl<T: Scalar> Bounded<T> for LineSegment2D<T> {
    type Aabb = Aabb2D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        Aabb2D::from_points(&[self.start_point(), self.end_point()])
    }
}
