//! TriangleMesh3D の Foundation トレイト実装

use crate::TriangleMesh3D;
use geo_contracts::{Bounded, PrimitiveKind, PrimitiveMetadata, Scalar, TolerantEq};
use geo_core::Aabb3D;

impl<T: Scalar> PrimitiveMetadata for TriangleMesh3D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Mesh
    }
}

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

fn total_mesh_area<T: Scalar>(mesh: &TriangleMesh3D<T>) -> T {
    let mut total_area = T::ZERO;

    for triangle_indices in mesh.indices() {
        if let (Some(v0), Some(v1), Some(v2)) = (
            mesh.vertices().get(triangle_indices[0]),
            mesh.vertices().get(triangle_indices[1]),
            mesh.vertices().get(triangle_indices[2]),
        ) {
            let edge1 = crate::Vector3D::new(v1.x() - v0.x(), v1.y() - v0.y(), v1.z() - v0.z());
            let edge2 = crate::Vector3D::new(v2.x() - v0.x(), v2.y() - v0.y(), v2.z() - v0.z());
            let cross_product = edge1.cross(&edge2);
            let triangle_area = cross_product.length() / T::from_f64(2.0);
            total_area += triangle_area;
        }
    }

    total_area
}

impl<T: Scalar> TolerantEq<T> for TriangleMesh3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        // 頂点数と三角形数が同じかチェック
        if self.vertex_count() != other.vertex_count()
            || self.triangle_count() != other.triangle_count()
        {
            return false;
        }

        let self_area = total_mesh_area(self);
        let other_area = total_mesh_area(other);
        let area_diff = (self_area - other_area).abs();

        // 面積の差が許容誤差内かチェック
        area_diff <= tolerance
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point3D;

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
        assert_eq!(total_mesh_area(&mesh), 0.5);

        if let Some(bbox) = mesh.aabb() {
            assert_eq!(bbox.min().x(), 0.0);
            assert_eq!(bbox.max().x(), 1.0);
        }
    }

    #[test]
    fn test_tolerant_eq() {
        let vertices1 = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        ];
        let indices1 = vec![[0, 1, 2]];

        let vertices2 = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        ];
        let indices2 = vec![[0, 1, 2]];

        let mesh1 = TriangleMesh3D::new(vertices1, indices1).unwrap();
        let mesh2 = TriangleMesh3D::new(vertices2, indices2).unwrap();

        let tolerance = 0.01; // スカラー値の許容誤差

        assert!(mesh1.tolerant_eq(&mesh2, tolerance));
    }
}
