//! EllipseArc3D の Foundation トレイト実装

use crate::EllipseArc3D;
use geo_contracts::{Bounded, MeasureFoundation, PrimitiveKind, PrimitiveMetadata, Scalar};
use geo_core::Aabb3D;

impl<T: Scalar> PrimitiveMetadata for EllipseArc3D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Arc
    }
}

impl<T: Scalar> MeasureFoundation<T> for EllipseArc3D<T> {
    fn measure(&self) -> Option<T> {
        // 楕円弧の測度は arc_length() メソッドが実装されていないため None を返す
        // TODO: EllipseArc3D に arc_length() メソッドを実装する必要がある
        None
    }
}

impl<T: Scalar> Bounded<T> for EllipseArc3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        // 楕円弧の中心と半径（長半軸）から包含する境界ボックスを計算
        let center = self.center();
        let semi_major = self.semi_major();

        // 簡易的な実装: 中心から長半軸分の範囲を境界ボックスとする
        // より正確な実装では、楕円弧の実際の範囲を考慮する必要がある
        let min_x = center.x() - semi_major;
        let max_x = center.x() + semi_major;
        let min_y = center.y() - semi_major;
        let max_y = center.y() + semi_major;
        let min_z = center.z() - semi_major;
        let max_z = center.z() + semi_major;

        Some(Aabb3D::new(
            crate::Point3D::new(min_x, min_y, min_z),
            crate::Point3D::new(max_x, max_y, max_z),
        ))
    }
}
