//! CylindricalSolid3D の拡張機能
//!
//! メッシュ生成、変換、サンプリングなどの高度な操作
//! ソリッド特有の体積的操作と表面的操作を含む

use crate::{CylindricalSolid3D, Point3D, TriangleMesh3D};
use geo_foundation::Scalar;

/// CylindricalSolid3D の拡張実装
impl<T: Scalar> CylindricalSolid3D<T> {
    /// 円柱ソリッドの上面の中心点を取得
    pub fn top_center(&self) -> Point3D<T> {
        Point3D::new(
            self.center().x() + self.axis().as_vector().x() * self.height(),
            self.center().y() + self.axis().as_vector().y() * self.height(),
            self.center().z() + self.axis().as_vector().z() * self.height(),
        )
    }

    /// 簡易的な円柱ソリッドメッシュを生成
    pub fn to_mesh(&self, radial_segments: usize) -> TriangleMesh3D<T> {
        let radial_segments = radial_segments.max(3);
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // 底面と上面の中心点
        let bottom_center = self.center();
        let top_center = self.top_center();

        // 円形断面の頂点を生成
        for i in 0..radial_segments {
            let angle = T::from_f64(2.0 * std::f64::consts::PI * i as f64 / radial_segments as f64);
            let cos_val = angle.cos();
            let sin_val = angle.sin();

            // 底面の点
            vertices.push(Point3D::new(
                bottom_center.x() + cos_val * self.radius(),
                bottom_center.y() + sin_val * self.radius(),
                bottom_center.z(),
            ));

            // 上面の点
            vertices.push(Point3D::new(
                top_center.x() + cos_val * self.radius(),
                top_center.y() + sin_val * self.radius(),
                top_center.z(),
            ));
        }

        // 中心点を追加
        vertices.push(bottom_center); // インデックス: radial_segments * 2
        vertices.push(top_center); // インデックス: radial_segments * 2 + 1

        let bottom_center_idx = radial_segments * 2;
        let top_center_idx = radial_segments * 2 + 1;

        // 底面の三角形
        for i in 0..radial_segments {
            let next_i = (i + 1) % radial_segments;
            indices.push([bottom_center_idx, i * 2, next_i * 2]);
        }

        // 上面の三角形
        for i in 0..radial_segments {
            let next_i = (i + 1) % radial_segments;
            indices.push([top_center_idx, next_i * 2 + 1, i * 2 + 1]);
        }

        // 側面の三角形
        for i in 0..radial_segments {
            let next_i = (i + 1) % radial_segments;
            let bottom_current = i * 2;
            let top_current = i * 2 + 1;
            let bottom_next = next_i * 2;
            let top_next = next_i * 2 + 1;

            // 側面の四角形を2つの三角形に分割
            indices.push([bottom_current, top_current, bottom_next]);
            indices.push([top_current, top_next, bottom_next]);
        }

        TriangleMesh3D::new(vertices, indices).unwrap_or_else(|_| {
            // エラーの場合は空のメッシュを返す
            TriangleMesh3D::new(Vec::new(), Vec::new()).unwrap()
        })
    }

    /// 体積計算の別実装（検証用）
    pub fn volume_alternative(&self) -> T {
        // V = πr²h
        let pi = T::PI;
        let radius_sq = self.radius() * self.radius();
        pi * radius_sq * self.height()
    }

    /// 質量特性計算（密度を指定）
    pub fn mass_properties(&self, density: T) -> MassProperties<T> {
        let volume = self.volume();
        let mass = volume * density;

        // 慣性モーメント（均質円柱）
        let radius_sq = self.radius() * self.radius();
        let height_sq = self.height() * self.height();

        // 円柱軸周りの慣性モーメント
        let ixx_local = mass * radius_sq / T::from_f64(2.0);

        // 円柱軸に垂直な軸周りの慣性モーメント
        let iyy_local = mass * (T::from_f64(3.0) * radius_sq + height_sq) / T::from_f64(12.0);
        let izz_local = iyy_local;

        MassProperties {
            mass,
            center_of_mass: self.center(),
            inertia_local: [ixx_local, iyy_local, izz_local],
        }
    }
}

/// 質量特性を表す構造体
#[derive(Debug, Clone, PartialEq)]
pub struct MassProperties<T: Scalar> {
    /// 質量
    pub mass: T,
    /// 質量中心
    pub center_of_mass: Point3D<T>,
    /// ローカル座標系での慣性モーメント [Ixx, Iyy, Izz]
    pub inertia_local: [T; 3],
}

// ============================================================================
// Backward Compatibility (移行期間中のみ) - 重複回避のためコメントアウト
// ============================================================================

/*
/// 旧Cylinder3D向けの拡張機能は既存のcylinder_3d_extensions.rsで提供されるため
/// 重複を避けるためここでは実装しない
*/
