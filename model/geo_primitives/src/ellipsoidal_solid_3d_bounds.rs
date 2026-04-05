//! EllipsoidalSolid3D Bounds Implementation
//!
//! `Bounded` trait と境界計算補助を提供する。

use crate::{EllipsoidalSolid3D, Point3D};
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb3D;

impl<T: Scalar> Bounded<T> for EllipsoidalSolid3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        Some(self.bounding_box())
    }
}

impl<T: Scalar> EllipsoidalSolid3D<T> {
    /// 楕円体の境界ボックスを計算
    ///
    /// # Returns
    /// 楕円体を包む最小の軸平行境界ボックス（AABB）
    pub fn bounding_box(&self) -> Aabb3D<T> {
        // ローカル座標系での半径を取得
        let a = self.a_radius_internal();
        let b = self.b_radius_internal();
        let c = self.c_radius_internal();

        // ローカル座標軸
        let x_axis = self.ref_direction_internal().as_vector();
        let y_axis = self.y_axis_internal().as_vector();
        let z_axis = self.axis_internal().as_vector();
        let center = self.center_internal();

        // ローカル座標系での8つの頂点（楕円体を包む直方体）をワールド座標に変換
        let local_vertices = [
            (a, b, c),
            (a, b, -c),
            (a, -b, c),
            (a, -b, -c),
            (-a, b, c),
            (-a, b, -c),
            (-a, -b, c),
            (-a, -b, -c),
        ];

        // ワールド座標系に変換
        let world_vertices: Vec<Point3D<T>> = local_vertices
            .iter()
            .map(|(lx, ly, lz)| center + x_axis * *lx + y_axis * *ly + z_axis * *lz)
            .collect();

        // 最小・最大点を計算
        let mut min_x = world_vertices[0].x();
        let mut min_y = world_vertices[0].y();
        let mut min_z = world_vertices[0].z();
        let mut max_x = world_vertices[0].x();
        let mut max_y = world_vertices[0].y();
        let mut max_z = world_vertices[0].z();

        for v in &world_vertices {
            min_x = min_x.min(v.x());
            min_y = min_y.min(v.y());
            min_z = min_z.min(v.z());
            max_x = max_x.max(v.x());
            max_y = max_y.max(v.y());
            max_z = max_z.max(v.z());
        }

        Aabb3D::new(
            Point3D::new(min_x, min_y, min_z),
            Point3D::new(max_x, max_y, max_z),
        )
    }

    /// 退化した楕円体かどうかを判定
    ///
    /// # Returns
    /// いずれかの半径が許容誤差以下の場合は `true`
    pub fn is_degenerate(&self) -> bool {
        let epsilon = T::EPSILON * T::from_f64(10.0);
        self.a_radius_internal() < epsilon
            || self.b_radius_internal() < epsilon
            || self.c_radius_internal() < epsilon
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Vector3D;
    use geo_contracts::{PrimitiveKind, PrimitiveMetadata};

    #[test]
    fn test_ellipsoidal_solid_foundation() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let solid = EllipsoidalSolid3D::new_standard(center, 2.0, 3.0, 4.0).unwrap();

        // Foundation トレイトのテスト
        assert_eq!(solid.primitive_kind(), PrimitiveKind::EllipsoidalSolid);

        let bbox = solid.aabb().expect("should have aabb");
        // 標準方向なので、中心 ± 各半径
        assert_eq!(bbox.min(), Point3D::new(-1.0, -1.0, -1.0));
        assert_eq!(bbox.max(), Point3D::new(3.0, 5.0, 7.0));

        let volume = solid.volume();
        // V = (4/3)π × 2 × 3 × 4 = 32π
        let expected_volume = (4.0 / 3.0) * std::f64::consts::PI * 2.0 * 3.0 * 4.0;
        assert!((volume - expected_volume).abs() < 1e-10);
    }

    #[test]
    fn test_ellipsoidal_solid_bbox_rotated() {
        // 45度回転した楕円体
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::new(1.0, 1.0, 0.0); // 45度傾き
        let ref_dir = Vector3D::new(1.0, -1.0, 0.0);
        let solid = EllipsoidalSolid3D::new(center, axis, ref_dir, 2.0, 1.0, 1.0).unwrap();

        let bbox = solid.aabb().expect("should have aabb");

        // 回転しているので境界ボックスは単純な ±radius ではない
        assert!(bbox.min().x() < 0.0);
        assert!(bbox.min().y() < 0.0);
        assert!(bbox.max().x() > 0.0);
        assert!(bbox.max().y() > 0.0);
    }

    #[test]
    fn test_ellipsoidal_solid_degenerate() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let solid = EllipsoidalSolid3D::new_standard(center, f64::EPSILON / 2.0, 1.0, 1.0).unwrap();

        assert!(solid.is_degenerate());

        let bbox = solid.aabb().expect("should have aabb");
        assert!(!bbox.is_empty());
    }

    #[test]
    fn test_sphere_special_case_foundation() {
        // a = b = c の球の特殊ケース
        let center = Point3D::new(1.0, 2.0, 3.0);
        let solid = EllipsoidalSolid3D::new_sphere(
            center,
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            5.0,
        )
        .unwrap();

        assert_eq!(solid.primitive_kind(), PrimitiveKind::EllipsoidalSolid);

        // 球の体積: V = (4/3)π × r³
        let expected_volume = (4.0 / 3.0) * std::f64::consts::PI * 5.0_f64.powi(3);
        let volume = solid.volume();
        assert!((volume - expected_volume).abs() < 1e-10);

        // 境界ボックスは中心 ± 半径
        let bbox = solid.aabb().unwrap();
        assert_eq!(bbox.min(), Point3D::new(-4.0, -3.0, -2.0));
        assert_eq!(bbox.max(), Point3D::new(6.0, 7.0, 8.0));
    }
}
