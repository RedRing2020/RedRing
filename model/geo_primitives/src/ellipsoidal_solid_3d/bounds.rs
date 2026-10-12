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
        let a = self.a_radius;
        let b = self.b_radius;
        let c = self.c_radius;

        // ローカル座標軸
        let x_axis = self.ref_direction.as_vector();
        let y_axis = self.y_axis_internal().as_vector();
        let z_axis = self.axis.as_vector();
        let center = self.center;

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
}
