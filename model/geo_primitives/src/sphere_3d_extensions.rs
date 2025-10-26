//! Sphere3D Extensions - 高度な幾何操作と分析機能

use crate::{Point3D, Sphere3D, TriangleMesh3D, Vector3D};
use geo_foundation::Scalar;

impl<T: Scalar> Sphere3D<T> {
    // ========================================================================
    // Mesh Generation
    // ========================================================================

    /// 球を三角形メッシュに変換
    ///
    /// # 引数
    /// * `u_subdivisions` - 経度方向の分割数（最小4）
    /// * `v_subdivisions` - 緯度方向の分割数（最小3）
    ///
    /// # 戻り値
    /// * `Some(TriangleMesh3D)` - 変換に成功した場合
    /// * `None` - 分割数が不適切な場合
    pub fn to_triangle_mesh(
        &self,
        u_subdivisions: usize,
        v_subdivisions: usize,
    ) -> Option<TriangleMesh3D<T>> {
        if u_subdivisions < 4 || v_subdivisions < 3 {
            return None;
        }

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // 頂点生成（球面座標系を使用）
        for v in 0..=v_subdivisions {
            let phi = T::PI * T::from_f64(v as f64 / v_subdivisions as f64); // 0 to π

            for u in 0..u_subdivisions {
                let theta =
                    T::from_f64(2.0) * T::PI * T::from_f64(u as f64 / u_subdivisions as f64); // 0 to 2π

                // 球面座標から直交座標に変換
                let x = self.center().x() + self.radius() * phi.sin() * theta.cos();
                let y = self.center().y() + self.radius() * phi.sin() * theta.sin();
                let z = self.center().z() + self.radius() * phi.cos();

                vertices.push(Point3D::new(x, y, z));
            }
        }

        // インデックス生成
        for v in 0..v_subdivisions {
            for u in 0..u_subdivisions {
                let current = v * u_subdivisions + u;
                let next_u = v * u_subdivisions + (u + 1) % u_subdivisions;
                let next_v = (v + 1) * u_subdivisions + u;
                let next_both = (v + 1) * u_subdivisions + (u + 1) % u_subdivisions;

                if v == 0 {
                    // 上極の三角形
                    indices.push([current, next_both, next_v]);
                } else if v == v_subdivisions - 1 {
                    // 下極の三角形
                    indices.push([current, next_u, next_both]);
                } else {
                    // 中間部分の四角形を2つの三角形に分割
                    indices.push([current, next_u, next_both]);
                    indices.push([current, next_both, next_v]);
                }
            }
        }

        TriangleMesh3D::new(vertices, indices).ok()
    }

    /// 簡易球メッシュ生成（正八面体の再分割）
    ///
    /// # 引数
    /// * `subdivision_level` - 再分割レベル（0-4推奨）
    pub fn to_icosphere_mesh(&self, subdivision_level: u32) -> Option<TriangleMesh3D<T>> {
        if subdivision_level > 6 {
            return None; // メモリ使用量を制限
        }

        // 正八面体の初期頂点
        let mut vertices = vec![
            Point3D::new(
                self.center().x(),
                self.center().y() + self.radius(),
                self.center().z(),
            ), // 上
            Point3D::new(
                self.center().x(),
                self.center().y() - self.radius(),
                self.center().z(),
            ), // 下
            Point3D::new(
                self.center().x() + self.radius(),
                self.center().y(),
                self.center().z(),
            ), // 右
            Point3D::new(
                self.center().x() - self.radius(),
                self.center().y(),
                self.center().z(),
            ), // 左
            Point3D::new(
                self.center().x(),
                self.center().y(),
                self.center().z() + self.radius(),
            ), // 前
            Point3D::new(
                self.center().x(),
                self.center().y(),
                self.center().z() - self.radius(),
            ), // 後
        ];

        // 正八面体の初期面（三角形）
        let mut indices = vec![
            [0, 2, 4],
            [0, 4, 3],
            [0, 3, 5],
            [0, 5, 2], // 上半分
            [1, 4, 2],
            [1, 3, 4],
            [1, 5, 3],
            [1, 2, 5], // 下半分
        ];

        // 再分割
        for _ in 0..subdivision_level {
            let mut new_indices = Vec::new();

            for triangle in &indices {
                // 各辺の中点を計算
                let mid01 =
                    self.get_sphere_point_between(vertices[triangle[0]], vertices[triangle[1]]);
                let mid12 =
                    self.get_sphere_point_between(vertices[triangle[1]], vertices[triangle[2]]);
                let mid20 =
                    self.get_sphere_point_between(vertices[triangle[2]], vertices[triangle[0]]);

                let idx0 = triangle[0];
                let idx1 = triangle[1];
                let idx2 = triangle[2];
                let idx01 = vertices.len();
                let idx12 = vertices.len() + 1;
                let idx20 = vertices.len() + 2;

                vertices.push(mid01);
                vertices.push(mid12);
                vertices.push(mid20);

                // 4つの小さな三角形を作成
                new_indices.push([idx0, idx01, idx20]);
                new_indices.push([idx1, idx12, idx01]);
                new_indices.push([idx2, idx20, idx12]);
                new_indices.push([idx01, idx12, idx20]);
            }

            indices = new_indices;
        }

        TriangleMesh3D::new(vertices, indices).ok()
    }

    /// 2点間の中点を球表面に投影
    fn get_sphere_point_between(&self, p1: Point3D<T>, p2: Point3D<T>) -> Point3D<T> {
        let mid_x = (p1.x() + p2.x()) / T::from_f64(2.0);
        let mid_y = (p1.y() + p2.y()) / T::from_f64(2.0);
        let mid_z = (p1.z() + p2.z()) / T::from_f64(2.0);

        let offset = Vector3D::new(
            mid_x - self.center().x(),
            mid_y - self.center().y(),
            mid_z - self.center().z(),
        );

        let normalized = offset.normalize();

        Point3D::new(
            self.center().x() + normalized.x() * self.radius(),
            self.center().y() + normalized.y() * self.radius(),
            self.center().z() + normalized.z() * self.radius(),
        )
    }

    // ========================================================================
    // Geometric Analysis
    // ========================================================================

    /// 他の球との交差判定
    pub fn intersects_sphere(&self, other: &Sphere3D<T>) -> bool {
        let center_distance = self.center().distance_to(&other.center());
        center_distance <= (self.radius() + other.radius())
    }

    /// 他の球を完全に含むかどうか
    pub fn contains_sphere(&self, other: &Sphere3D<T>) -> bool {
        let center_distance = self.center().distance_to(&other.center());
        center_distance + other.radius() <= self.radius()
    }

    /// 他の球の内部にあるかどうか
    pub fn is_inside_sphere(&self, other: &Sphere3D<T>) -> bool {
        other.contains_sphere(self)
    }

    /// 2つの球の最小包含球を計算
    pub fn bounding_sphere_with(&self, other: &Sphere3D<T>) -> Sphere3D<T> {
        let center_distance = self.center().distance_to(&other.center());

        // 一方が他方を完全に含む場合
        if self.contains_sphere(other) {
            return self.clone();
        }
        if other.contains_sphere(self) {
            return other.clone();
        }

        // 新しい包含球の半径と中心を計算
        let new_radius = (center_distance + self.radius() + other.radius()) / T::from_f64(2.0);

        let direction = Vector3D::new(
            other.center().x() - self.center().x(),
            other.center().y() - self.center().y(),
            other.center().z() - self.center().z(),
        )
        .normalize();

        let offset = (center_distance + other.radius() - self.radius()) / T::from_f64(2.0);

        let new_center = Point3D::new(
            self.center().x() + direction.x() * offset,
            self.center().y() + direction.y() * offset,
            self.center().z() + direction.z() * offset,
        );

        Sphere3D::new(new_center, new_radius).unwrap()
    }

    // ========================================================================
    // Sampling and Tessellation
    // ========================================================================

    /// 球表面の均等サンプリング点を生成
    pub fn uniform_surface_samples(&self, num_samples: usize) -> Vec<Point3D<T>> {
        let mut points = Vec::with_capacity(num_samples);

        for i in 0..num_samples {
            // フィボナッチ球面サンプリング
            let y = T::from_f64(1.0) - T::from_f64(2.0 * i as f64 / num_samples as f64);
            let radius_at_y = (T::from_f64(1.0) - y * y).sqrt();

            let golden_angle = T::PI * (T::from_f64(3.0) - T::from_f64(5.0).sqrt());
            let theta = golden_angle * T::from_f64(i as f64);

            let x = theta.cos() * radius_at_y;
            let z = theta.sin() * radius_at_y;

            let point = Point3D::new(
                self.center().x() + x * self.radius(),
                self.center().y() + y * self.radius(),
                self.center().z() + z * self.radius(),
            );

            points.push(point);
        }

        points
    }

    /// 球内部のランダムサンプリング点を生成（擬似乱数）
    pub fn volume_samples(&self, num_samples: usize, seed: u64) -> Vec<Point3D<T>> {
        let mut points = Vec::with_capacity(num_samples);
        let mut rng_state = seed;

        for _ in 0..num_samples {
            // 簡易線形合同法による擬似乱数生成
            rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
            let r1 = (rng_state as f64) / (u64::MAX as f64);

            rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
            let r2 = (rng_state as f64) / (u64::MAX as f64);

            rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
            let r3 = (rng_state as f64) / (u64::MAX as f64);

            // 球内部の均等分布
            let radius = self.radius() * T::from_f64(r1.powf(1.0 / 3.0));
            let theta = T::from_f64(2.0) * T::PI * T::from_f64(r2);
            let phi = T::from_f64(r3 * 2.0 - 1.0).acos();

            let x = self.center().x() + radius * phi.sin() * theta.cos();
            let y = self.center().y() + radius * phi.sin() * theta.sin();
            let z = self.center().z() + radius * phi.cos();

            points.push(Point3D::new(x, y, z));
        }

        points
    }
}
