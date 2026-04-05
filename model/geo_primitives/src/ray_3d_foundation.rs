//! Ray3D の Foundation トレイト実装

use crate::Ray3D;
use geo_contracts::Scalar;
use geo_contracts::{PrimitiveKind, PrimitiveMetadata, TolerantEq};

impl<T: Scalar> PrimitiveMetadata for Ray3D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Ray
    }
}

// Note: Ray3D は無限に延びるため、Bounded トレイトは実装しません
// 境界ボックスが必要な場合は ray_3d_extensions.rs の bounding_box() メソッドを使用してください

impl<T: Scalar> TolerantEq<T> for Ray3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        // 起点の距離をチェック
        let origin_distance = self.origin.distance_to(&other.origin);

        // 方向ベクトルの類似性をチェック（内積）
        let direction_dot = self.direction.dot(&other.direction).abs();
        let direction_similar = direction_dot >= T::from_f64(0.999); // 約2.5度以内

        // 許容誤差として単一のスカラー値を使用
        origin_distance <= tolerance && direction_similar
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Direction3D, Point3D, Vector3D};

    #[test]
    fn test_extension_foundation() {
        let origin = Point3D::new(0.0, 0.0, 0.0);
        let direction = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let ray = Ray3D::new(origin, direction.as_vector()).unwrap();

        assert_eq!(ray.primitive_kind(), PrimitiveKind::Ray);

        // Ray3D は Bounded を実装しないため、aabb() はありません
        // 必要な場合は ray_3d_extensions.rs の bounding_box() を使用
    }

    #[test]
    fn test_tolerant_eq() {
        let origin = Point3D::new(0.0, 0.0, 0.0);
        let direction = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();

        let ray1 = Ray3D::new(origin, direction.as_vector()).unwrap();
        let ray2 = Ray3D::new(origin, direction.as_vector()).unwrap();
        let ray3 = Ray3D::new(Point3D::new(1.0, 0.0, 0.0), direction.as_vector()).unwrap();

        let tolerance = 0.01; // スカラー値の許容誤差

        assert!(ray1.tolerant_eq(&ray2, tolerance));
        assert!(!ray1.tolerant_eq(&ray3, tolerance));
    }
}
