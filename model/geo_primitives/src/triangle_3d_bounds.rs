//! Triangle3D の Bounds 実装

use crate::Triangle3D;
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb3D;

impl<T: Scalar> Bounded<T> for Triangle3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        // 3つの頂点の最小/最大座標を計算
        let vertices = [
            self.vertex_a_internal(),
            self.vertex_b_internal(),
            self.vertex_c_internal(),
        ];

        let min_x = vertices
            .iter()
            .map(|v| v.x())
            .fold(T::INFINITY, |a, b| a.min(b));
        let max_x = vertices
            .iter()
            .map(|v| v.x())
            .fold(-T::INFINITY, |a, b| a.max(b));

        let min_y = vertices
            .iter()
            .map(|v| v.y())
            .fold(T::INFINITY, |a, b| a.min(b));
        let max_y = vertices
            .iter()
            .map(|v| v.y())
            .fold(-T::INFINITY, |a, b| a.max(b));

        let min_z = vertices
            .iter()
            .map(|v| v.z())
            .fold(T::INFINITY, |a, b| a.min(b));
        let max_z = vertices
            .iter()
            .map(|v| v.z())
            .fold(-T::INFINITY, |a, b| a.max(b));

        let min_point = crate::Point3D::new(min_x, min_y, min_z);
        let max_point = crate::Point3D::new(max_x, max_y, max_z);

        Some(Aabb3D::new(min_point, max_point))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point3D;
    use geo_contracts::{PrimitiveKind, PrimitiveMetadata};

    #[test]
    fn test_metadata_and_bounds_capabilities() {
        let triangle = Triangle3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        )
        .unwrap();

        assert_eq!(triangle.primitive_kind(), PrimitiveKind::Triangle);
        assert_eq!(triangle.area(), 0.5);

        let aabb = triangle.aabb().expect("Triangle should have an AABB");
        assert_eq!(aabb.min().x(), 0.0);
        assert_eq!(aabb.max().x(), 1.0);
        assert_eq!(aabb.min().y(), 0.0);
        assert_eq!(aabb.max().y(), 1.0);
    }
}
