//! Plane3DCoordinateSystem の Foundation トレイト実装

// use crate::{BBox3D, Plane3DCoordinateSystem}; // 一時的にコメントアウト
use crate::BBox3D;
use geo_foundation::{extension_foundation::ExtensionFoundation, PrimitiveKind, Scalar};

impl<T: Scalar> ExtensionFoundation<T> for Plane3DCoordinateSystem<T> {
    type BBox = BBox3D<T>;

    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Plane
    }

    fn bounding_box(&self) -> Self::BBox {
        // 座標系付き平面も無限平面なので境界ボックスを持たない
        // 理論的には無限大の境界ボックスを返すべきだが、
        // 実用上は None を表現するため、原点の微小な境界ボックスを返す
        let origin = self.origin();
        let epsilon = T::EPSILON;

        let min_point = crate::Point3D::new(
            origin.x() - epsilon,
            origin.y() - epsilon,
            origin.z() - epsilon,
        );
        let max_point = crate::Point3D::new(
            origin.x() + epsilon,
            origin.y() + epsilon,
            origin.z() + epsilon,
        );

        BBox3D::from_points(&[min_point, max_point]).expect("Failed to create bounding box")
    }

    fn measure(&self) -> Option<T> {
        // 無限平面の測度（面積）は定義されない
        None
    }
}
