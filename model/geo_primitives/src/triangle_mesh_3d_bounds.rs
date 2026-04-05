//! TriangleMesh3D の Foundation トレイト実装

use crate::TriangleMesh3D;
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb3D;

impl<T: Scalar> Bounded<T> for TriangleMesh3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        if let Some((min_point, max_point)) = self.bounding_box() {
            Some(Aabb3D::new(min_point, max_point))
        } else {
            // 空のメッシュの場合は None
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point3D;
    use geo_contracts::{PrimitiveKind, PrimitiveMetadata};

    #[test]
    fn test_extension_foundation() {
        let vertices = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        ];
        let indices = vec![[0, 1, 2]];

        let mesh = TriangleMesh3D::new(vertices, indices).unwrap();

        assert_eq!(mesh.primitive_kind(), PrimitiveKind::Mesh);

        if let Some(bbox) = mesh.aabb() {
            assert_eq!(bbox.min().x(), 0.0);
            assert_eq!(bbox.max().x(), 1.0);
        }
    }
}
