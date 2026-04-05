//! Circle2D の Bounds 実装

use crate::Circle2D;
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb2D;

impl<T: Scalar> Bounded<T> for Circle2D<T> {
    type Aabb = Aabb2D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        let (min_point, max_point) = self.bounding_box();
        Some(Aabb2D::new(min_point, max_point))
    }
}
