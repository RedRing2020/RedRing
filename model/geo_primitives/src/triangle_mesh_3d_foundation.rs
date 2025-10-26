//! TriangleMesh3D の Foundation トレイト実装

use crate::{BBox3D, TriangleMesh3D};
use geo_foundation::{
    extension_foundation::ExtensionFoundation, PrimitiveKind, Scalar, TolerantEq,
};

// ============================================================================
// Foundation Trait Implementation
// ============================================================================

impl<T: Scalar> ExtensionFoundation<T> for TriangleMesh3D<T> {
    type BBox = BBox3D<T>;

    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Mesh
    }

    fn bounding_box(&self) -> Self::BBox {
        if let Some((min_point, max_point)) = self.bounding_box() {
            BBox3D::new(min_point, max_point)
        } else {
            // 空のメッシュの場合は原点のバウンディングボックス
            let origin = crate::Point3D::new(T::ZERO, T::ZERO, T::ZERO);
            BBox3D::new(origin, origin)
        }
    }

    fn measure(&self) -> Option<T> {
        // 三角形の面積の合計を計算
        let mut total_area = T::ZERO;

        for triangle_indices in self.indices() {
            if let (Some(v0), Some(v1), Some(v2)) = (
                self.vertices().get(triangle_indices[0]),
                self.vertices().get(triangle_indices[1]),
                self.vertices().get(triangle_indices[2]),
            ) {
                // 三角形の面積を計算（外積の半分）
                let edge1 = crate::Vector3D::new(v1.x() - v0.x(), v1.y() - v0.y(), v1.z() - v0.z());
                let edge2 = crate::Vector3D::new(v2.x() - v0.x(), v2.y() - v0.y(), v2.z() - v0.z());
                let cross_product = edge1.cross(&edge2);
                let triangle_area = cross_product.length() / T::from_f64(2.0);
                total_area += triangle_area;
            }
        }

        Some(total_area)
    }
}

impl<T: Scalar> TolerantEq<T> for TriangleMesh3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        // 頂点数と三角形数が同じかチェック
        if self.vertex_count() != other.vertex_count()
            || self.triangle_count() != other.triangle_count()
        {
            return false;
        }

        // 表面積の差をチェック（measure()を使用）
        let self_area = self.measure().unwrap_or(T::ZERO);
        let other_area = other.measure().unwrap_or(T::ZERO);
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
        assert!(mesh.measure().is_some());
        // measure()はsurface_area()の代替実装

        if let Some((min_pt, max_pt)) = mesh.bounding_box() {
            assert_eq!(min_pt.x(), 0.0);
            assert_eq!(max_pt.x(), 1.0);
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
