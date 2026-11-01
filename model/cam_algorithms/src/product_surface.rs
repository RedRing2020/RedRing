//! 製品形状の面への問い合わせ（削り込み・削り残しの判定）
//!
//! 3 軸（工具の軸は +Z、工具は上方から近づく）とし、製品形状を上から見た最も高い面（上側の包絡）として扱う。
//! 逆オフセット法の工具の高さ（`InverseOffsetShape`）と同じ見方で、面の向き（法線）には頼らない（STL・単体の面は向きが
//! そろうとは限らないため）。

use geo_algorithms::{Point3D, TriangleMesh3D};

use crate::inverse_offset::{MeshTriangles, XyBucketIndex, contains_xy, cross, sub};
use crate::solver::CamSolverError;

/// 製品形状の三角形メッシュへの問い合わせ（上側の包絡の高さ・点からの最短距離）
#[derive(Debug, Clone)]
pub struct ProductSurface {
    vertices: Vec<[f64; 3]>,
    triangles: Vec<[usize; 3]>,
    index: XyBucketIndex,
}

impl ProductSurface {
    /// 製品形状の三角形メッシュから作る（面積のない三角形は使わない）
    ///
    /// # Errors
    ///
    /// 面積のある三角形がない場合。
    pub fn new(mesh: &TriangleMesh3D<f64>) -> Result<Self, CamSolverError> {
        let MeshTriangles {
            vertices,
            triangles,
            xy_min,
            xy_max,
        } = MeshTriangles::new(mesh)?;
        let index = XyBucketIndex::build(&vertices, &triangles, 0.0, xy_min, xy_max);
        Ok(Self {
            vertices,
            triangles,
            index,
        })
    }

    /// (x, y) の、製品形状の上側の包絡の高さ（その XY に製品形状がない場合は `None`）
    ///
    /// 鉛直な三角形は、XY で面積がないため高さを決めない（両側の面の高さで決まる）。
    pub fn height_at(&self, x: f64, y: f64) -> Option<f64> {
        self.index
            .candidates(x, y)
            .iter()
            .filter_map(|&triangle| {
                let [a, b, c] = self.corners(triangle);
                contains_xy(a, b, c, x, y)
                    .then(|| plane_height(a, b, c, x, y))
                    .flatten()
            })
            .reduce(f64::max)
    }

    /// 点 `point` から製品形状の三角形までの最短距離
    pub fn distance_to(&self, point: Point3D<f64>) -> f64 {
        let p = [point.x(), point.y(), point.z()];
        let mut best = f64::INFINITY;
        for (lower_bound, cells) in self.index.rings(p[0], p[1]) {
            // 外側の環の三角形は、今の最短距離より近くにない
            if lower_bound >= best {
                break;
            }
            for &triangle in cells.into_iter().flatten() {
                let [a, b, c] = self.corners(triangle);
                best = best.min(squared_distance_to_triangle(p, a, b, c).sqrt());
            }
        }
        best
    }

    fn corners(&self, triangle: usize) -> [[f64; 3]; 3] {
        self.triangles[triangle].map(|vertex| self.vertices[vertex])
    }
}

/// 三角形 `a`・`b`・`c` を含む平面の、(x, y) での高さ（鉛直な三角形は `None`）
fn plane_height(a: [f64; 3], b: [f64; 3], c: [f64; 3], x: f64, y: f64) -> Option<f64> {
    let n = cross(sub(b, a), sub(c, a));
    // XY で面積がない三角形は、`contains_xy` が含まないため、ここには来ない
    (n[2] != 0.0).then(|| a[2] - (n[0] * (x - a[0]) + n[1] * (y - a[1])) / n[2])
}

/// 点 `p` から三角形 `a`・`b`・`c` までの最短距離の 2 乗（三角形の面・辺・頂点のうち最も近い点まで）
fn squared_distance_to_triangle(p: [f64; 3], a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f64 {
    let dot = |u: [f64; 3], v: [f64; 3]| u[0] * v[0] + u[1] * v[1] + u[2] * v[2];
    let (ab, ac, ap) = (sub(b, a), sub(c, a), sub(p, a));
    let along = |origin: [f64; 3], edge: [f64; 3], t: f64| {
        [
            origin[0] + t * edge[0],
            origin[1] + t * edge[1],
            origin[2] + t * edge[2],
        ]
    };
    let squared = |q: [f64; 3]| {
        let d = sub(p, q);
        dot(d, d)
    };
    // 最も近い点がどの頂点・辺・面にあるかを、重心座標の符号で分ける
    let (d1, d2) = (dot(ab, ap), dot(ac, ap));
    if d1 <= 0.0 && d2 <= 0.0 {
        return squared(a);
    }
    let bp = sub(p, b);
    let (d3, d4) = (dot(ab, bp), dot(ac, bp));
    if d3 >= 0.0 && d4 <= d3 {
        return squared(b);
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        return squared(along(a, ab, d1 / (d1 - d3)));
    }
    let cp = sub(p, c);
    let (d5, d6) = (dot(ab, cp), dot(ac, cp));
    if d6 >= 0.0 && d5 <= d6 {
        return squared(c);
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        return squared(along(a, ac, d2 / (d2 - d6)));
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && d4 - d3 >= 0.0 && d5 - d6 >= 0.0 {
        return squared(along(b, sub(c, b), (d4 - d3) / ((d4 - d3) + (d5 - d6))));
    }
    // 面の中
    let denominator = va + vb + vc;
    let (v, w) = (vb / denominator, vc / denominator);
    squared([
        a[0] + v * ab[0] + w * ac[0],
        a[1] + v * ab[1] + w * ac[1],
        a[2] + v * ab[2] + w * ac[2],
    ])
}

#[cfg(test)]
mod tests {
    use geo_algorithms::{Point3D, TriangleMesh3D, TriangleMesh3DConstructor};

    use super::{ProductSurface, squared_distance_to_triangle};

    /// 底面 z = 0（20 × 20）・頂点 (10, 10, 10) の四角錐の側面
    fn pyramid() -> ProductSurface {
        let mesh = TriangleMesh3D::new(
            vec![
                Point3D::new(0.0, 0.0, 0.0),
                Point3D::new(20.0, 0.0, 0.0),
                Point3D::new(20.0, 20.0, 0.0),
                Point3D::new(0.0, 20.0, 0.0),
                Point3D::new(10.0, 10.0, 10.0),
            ],
            // 向きをそろえない（面の向きに頼らない）
            vec![[0, 1, 4], [2, 1, 4], [2, 3, 4], [0, 3, 4]],
        )
        .unwrap();
        ProductSurface::new(&mesh).unwrap()
    }

    #[test]
    fn height_is_the_upper_envelope_and_none_outside_the_shape() {
        let surface = pyramid();
        assert!((surface.height_at(10.0, 10.0).unwrap() - 10.0).abs() < 1e-12);
        assert!((surface.height_at(5.0, 10.0).unwrap() - 5.0).abs() < 1e-12);
        assert!((surface.height_at(10.0, 2.0).unwrap() - 2.0).abs() < 1e-12);
        assert_eq!(surface.height_at(-1.0, 10.0), None);
        assert_eq!(surface.height_at(10.0, 25.0), None);
    }

    #[test]
    fn distance_is_to_the_nearest_face_edge_or_vertex() {
        let surface = pyramid();
        // 側面 x = 5 の点の上下 1（側面の法線は (-1, 0, 1) / √2）
        let slope = 1.0 / 2.0_f64.sqrt();
        assert!((surface.distance_to(Point3D::new(5.0, 10.0, 6.0)) - slope).abs() < 1e-12);
        assert!((surface.distance_to(Point3D::new(5.0, 10.0, 4.0)) - slope).abs() < 1e-12);
        // 頂点の真上
        assert!((surface.distance_to(Point3D::new(10.0, 10.0, 13.0)) - 3.0).abs() < 1e-12);
        // 形状の外の下（最も近いのは底の辺 y = 0。XY で 3、高さ -4）
        assert!((surface.distance_to(Point3D::new(10.0, -3.0, -4.0)) - 5.0).abs() < 1e-12);
        // 索引の範囲から遠く離れた点
        assert!((surface.distance_to(Point3D::new(-100.0, 0.0, 0.0)) - 100.0).abs() < 1e-12);
    }

    #[test]
    fn squared_distance_covers_every_region_of_the_triangle() {
        let (a, b, c) = ([0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [0.0, 4.0, 0.0]);
        let cases = [
            ([1.0, 1.0, 2.0], 4.0),   // 面の中
            ([-1.0, -1.0, 0.0], 2.0), // 頂点 a
            ([5.0, -1.0, 0.0], 2.0),  // 頂点 b
            ([-1.0, 5.0, 0.0], 2.0),  // 頂点 c
            ([2.0, -3.0, 0.0], 9.0),  // 辺 ab
            ([-3.0, 2.0, 0.0], 9.0),  // 辺 ac
            ([3.0, 3.0, 0.0], 2.0),   // 辺 bc
        ];
        for (p, expected) in cases {
            let actual = squared_distance_to_triangle(p, a, b, c);
            assert!((actual - expected).abs() < 1e-12, "{p:?}: {actual}");
        }
    }
}
