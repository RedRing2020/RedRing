//! Circle2D の Bounds 実装

use crate::{Circle2D, Point2D};
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb2D;

impl<T: Scalar> Bounded<T> for Circle2D<T> {
    type Aabb = Aabb2D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        let (c, r) = (self.center, self.radius);
        Some(Aabb2D::new(
            Point2D::new(c.x() - r, c.y() - r),
            Point2D::new(c.x() + r, c.y() + r),
        ))
    }
}
