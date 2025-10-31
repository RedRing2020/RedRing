//! Cylinder3D の拡張機能
//!
//! メッシュ生成、変換、サンプリングなどの高度な操作

use crate::{Cylinder3D, Point3D, TriangleMesh3D};
use geo_foundation::Scalar;

/// Cylinder3D の拡張実装
impl<T: Scalar> Cylinder3D<T> {
    /// 円柱の上面の中心点を取得
    pub fn top_center(&self) -> Point3D<T> {
        Point3D::new(
            self.center().x() + self.axis().x() * self.height(),
            self.center().y() + self.axis().y() * self.height(),
            self.center().z() + self.axis().z() * self.height(),
        )
    }

    /// 簡易的な円柱メッシュを生成
    pub fn to_mesh(&self, radial_segments: usize) -> TriangleMesh3D<T> {
        let radial_segments = radial_segments.max(3);
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // 基本的な円形断面を作成
        for i in 0..radial_segments {
            let angle = T::from_f64(2.0 * std::f64::consts::PI * i as f64 / radial_segments as f64);
            let cos_val = angle.cos();
            let sin_val = angle.sin();

            // 底面の点
            vertices.push(Point3D::new(
                self.center().x() + cos_val * self.radius(),
                self.center().y() + sin_val * self.radius(),
                self.center().z(),
            ));

            // 上面の点
            vertices.push(Point3D::new(
                self.center().x() + cos_val * self.radius() + self.axis().x() * self.height(),
                self.center().y() + sin_val * self.radius() + self.axis().y() * self.height(),
                self.center().z() + self.axis().z() * self.height(),
            ));
        }

        // 側面の三角形を作成
        for i in 0..radial_segments {
            let bottom1 = i * 2;
            let top1 = i * 2 + 1;
            let bottom2 = ((i + 1) % radial_segments) * 2;
            let top2 = ((i + 1) % radial_segments) * 2 + 1;

            // 2つの三角形で四角形を構成
            indices.push([bottom1, top1, bottom2]);
            indices.push([bottom2, top1, top2]);
        }

        TriangleMesh3D::new(vertices, indices).unwrap_or_else(|_| TriangleMesh3D::empty())
    }
}
