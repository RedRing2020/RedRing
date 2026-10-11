//! Triangle2D の Bounds 実装

use crate::Triangle2D;
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb2D;

impl<T: Scalar> Bounded<T> for Triangle2D<T> {
    type Aabb = Aabb2D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        Aabb2D::from_points(&[self.vertex_a, self.vertex_b, self.vertex_c])
    }
}
