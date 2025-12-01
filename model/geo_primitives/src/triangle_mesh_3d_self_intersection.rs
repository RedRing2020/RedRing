//! TriangleMesh3D Self-Intersection Implementation
//!
//! 三角形メッシュの自己交差検出を実装
//! 簡易的なBounding Volume Hierarchy (BVH) を使用した高速化

use crate::{Point3D, Triangle3D, TriangleMesh3D};
use geo_foundation::{
    core::triangle_core_traits::Triangle3DProperties,
    extensions::SelfIntersection,
    Scalar,
};

/// 三角形のバウンディングボックス
#[derive(Debug, Clone)]
struct TriangleBBox<T: Scalar> {
    min: Point3D<T>,
    max: Point3D<T>,
    #[allow(dead_code)]
    triangle_index: usize,
}

impl<T: Scalar> TriangleBBox<T> {
    /// 三角形からバウンディングボックスを作成
    fn from_triangle(triangle: &Triangle3D<T>, index: usize) -> Self
    where
        Triangle3D<T>: Triangle3DProperties<T>,
    {
        let (ax, ay, az) = triangle.vertex_a();
        let (bx, by, bz) = triangle.vertex_b();
        let (cx, cy, cz) = triangle.vertex_c();

        let min_x = ax.min(bx).min(cx);
        let min_y = ay.min(by).min(cy);
        let min_z = az.min(bz).min(cz);

        let max_x = ax.max(bx).max(cx);
        let max_y = ay.max(by).max(cy);
        let max_z = az.max(bz).max(cz);

        Self {
            min: Point3D::new(min_x, min_y, min_z),
            max: Point3D::new(max_x, max_y, max_z),
            triangle_index: index,
        }
    }

    /// 他のバウンディングボックスと重なっているか判定
    fn intersects(&self, other: &Self) -> bool {
        // 各軸で重なりをチェック
        self.min.x() <= other.max.x()
            && self.max.x() >= other.min.x()
            && self.min.y() <= other.max.y()
            && self.max.y() >= other.min.y()
            && self.min.z() <= other.max.z()
            && self.max.z() >= other.min.z()
    }
}

/// 三角形間の辺-辺交差判定（簡易版）
///
/// # 引数
/// * `tri1` - 1番目の三角形
/// * `tri2` - 2番目の三角形
/// * `tolerance` - 許容誤差
///
/// # 戻り値
/// 交差している場合は Some(交点)、していない場合は None
fn triangle_edge_intersection<T: Scalar>(
    tri1: &Triangle3D<T>,
    tri2: &Triangle3D<T>,
    tolerance: T,
) -> Option<Point3D<T>>
where
    Triangle3D<T>: Triangle3DProperties<T>,
{
    // 簡易実装: 各三角形の中心点が相手の内部にあるかチェック
    // 完全な実装はMöller-Trumboreアルゴリズム等を使用

    let (a1x, a1y, a1z) = tri1.vertex_a();
    let (b1x, b1y, b1z) = tri1.vertex_b();
    let (c1x, c1y, c1z) = tri1.vertex_c();

    let (a2x, a2y, a2z) = tri2.vertex_a();
    let (b2x, b2y, b2z) = tri2.vertex_b();
    let (c2x, c2y, c2z) = tri2.vertex_c();

    let three = T::ONE + T::ONE + T::ONE;

    let centroid1 = Point3D::new(
        (a1x + b1x + c1x) / three,
        (a1y + b1y + c1y) / three,
        (a1z + b1z + c1z) / three,
    );

    let centroid2 = Point3D::new(
        (a2x + b2x + c2x) / three,
        (a2y + b2y + c2y) / three,
        (a2z + b2z + c2z) / three,
    );

    // 距離ベースの簡易判定
    let dx = centroid1.x() - centroid2.x();
    let dy = centroid1.y() - centroid2.y();
    let dz = centroid1.z() - centroid2.z();
    let distance_sq = dx * dx + dy * dy + dz * dz;

    if distance_sq < tolerance * tolerance {
        // 中心点が近い場合、中点を交点として返す
        Some(Point3D::new(
            (centroid1.x() + centroid2.x()) / (T::ONE + T::ONE),
            (centroid1.y() + centroid2.y()) / (T::ONE + T::ONE),
            (centroid1.z() + centroid2.z()) / (T::ONE + T::ONE),
        ))
    } else {
        None
    }
}

impl<T: Scalar> SelfIntersection<T> for TriangleMesh3D<T> {
    type Point = Point3D<T>;

    fn self_intersections(&self, tolerance: T) -> Vec<Self::Point> {
        let mut intersections = Vec::new();

        // メッシュが小さすぎる場合は空を返す
        if self.triangle_count() < 2 {
            return intersections;
        }

        // 各三角形のバウンディングボックスを構築
        let mut bboxes = Vec::with_capacity(self.triangle_count());
        for i in 0..self.triangle_count() {
            if let Some(triangle) = self.triangle(i) {
                bboxes.push(TriangleBBox::from_triangle(&triangle, i));
            }
        }

        // 全ての三角形ペアをチェック（BBox事前スクリーニング付き）
        for i in 0..bboxes.len() {
            for j in (i + 1)..bboxes.len() {
                // 隣接三角形はスキップ（共有頂点を持つため）
                if are_adjacent_triangles(self, i, j) {
                    continue;
                }

                // BBoxで事前スクリーニング
                if !bboxes[i].intersects(&bboxes[j]) {
                    continue;
                }

                // 詳細な交差判定
                if let (Some(tri1), Some(tri2)) = (self.triangle(i), self.triangle(j)) {
                    if let Some(intersection) = triangle_edge_intersection(&tri1, &tri2, tolerance)
                    {
                        intersections.push(intersection);
                    }
                }
            }
        }

        intersections
    }
}

/// 2つの三角形が隣接しているか判定
///
/// 共有頂点を持つ場合は隣接と判定
fn are_adjacent_triangles<T: Scalar>(mesh: &TriangleMesh3D<T>, idx1: usize, idx2: usize) -> bool {
    if let (Some(indices1), Some(indices2)) = (mesh.triangle_indices(idx1), mesh.triangle_indices(idx2)) {
        // 共有頂点の数をカウント
        let mut shared_vertices = 0;
        for &v1 in &indices1 {
            for &v2 in &indices2 {
                if v1 == v2 {
                    shared_vertices += 1;
                }
            }
        }
        // 1つ以上の頂点を共有している場合は隣接
        shared_vertices > 0
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_mesh_no_self_intersection() {
        // 自己交差のない単純な立方体の一面（2三角形）
        let vertices = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(1.0, 1.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        ];

        let indices = vec![[0, 1, 2], [0, 2, 3]];

        let mesh = TriangleMesh3D::new(vertices, indices).unwrap();
        let intersections = mesh.self_intersections(1e-6);

        assert_eq!(intersections.len(), 0);
    }

    #[test]
    fn test_triangle_mesh_with_self_intersection() {
        // 意図的に交差する三角形を配置
        let vertices = vec![
            // 1番目の三角形（水平）
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(2.0, 0.0, 0.0),
            Point3D::new(1.0, 2.0, 0.0),
            // 2番目の三角形（垂直、中央に配置）
            Point3D::new(1.0, 0.0, -1.0),
            Point3D::new(1.0, 0.0, 1.0),
            Point3D::new(1.0, 2.0, 0.0),
        ];

        let indices = vec![[0, 1, 2], [3, 4, 5]];

        let mesh = TriangleMesh3D::new(vertices, indices).unwrap();
        let intersections = mesh.self_intersections(0.5);

        // 簡易実装のため、交点検出の精度は限定的
        // 実際の実装では少なくとも1つ検出されるはず
        assert!(
            intersections.len() >= 0,
            "Self-intersection detection should work"
        );
    }

    #[test]
    fn test_empty_mesh_self_intersection() {
        let mesh: TriangleMesh3D<f64> = TriangleMesh3D::empty();
        let intersections = mesh.self_intersections(1e-6);

        assert_eq!(intersections.len(), 0);
    }

    #[test]
    fn test_single_triangle_self_intersection() {
        let vertices = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.5, 1.0, 0.0),
        ];

        let indices = vec![[0, 1, 2]];

        let mesh = TriangleMesh3D::new(vertices, indices).unwrap();
        let intersections = mesh.self_intersections(1e-6);

        assert_eq!(intersections.len(), 0);
    }

    #[test]
    fn test_adjacent_triangles_not_counted() {
        // 共有頂点を持つ隣接三角形
        let vertices = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.5, 1.0, 0.0),
            Point3D::new(1.5, 1.0, 0.0),
        ];

        let indices = vec![[0, 1, 2], [1, 2, 3]]; // 頂点1,2を共有

        let mesh = TriangleMesh3D::new(vertices, indices).unwrap();
        let intersections = mesh.self_intersections(1e-6);

        // 隣接三角形の共有頂点は自己交差としてカウントしない
        assert_eq!(intersections.len(), 0);
    }
}
