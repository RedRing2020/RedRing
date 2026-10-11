//! Ellipse2D の Bounds 実装

use crate::Ellipse2D;
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb2D;

impl<T: Scalar> Bounded<T> for Ellipse2D<T> {
    type Aabb = Aabb2D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        Some(self.bounding_box())
    }
}
