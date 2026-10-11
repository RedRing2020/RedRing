//! Triangle3D Core 実装
//!
//! Foundation統一システムに基づくTriangle3Dの必須機能のみ

mod bounds;
#[cfg(test)]
mod tests;
mod transform;

use crate::{Point3D, Vector3D};
use geo_contracts::{
    default_distance_tolerance, default_kernel_numerical_zero_tolerance, PointClassification,
    Scalar, Triangle3DBoundaryAccess, Triangle3DConstructor, Triangle3DContainment,
    Triangle3DDerived, Triangle3DDistance,
};

/// 3次元三角形（Core実装）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Triangle3D<T: Scalar> {
    vertex_a: Point3D<T>,
    vertex_b: Point3D<T>,
    vertex_c: Point3D<T>,
}

impl<T: Scalar> geo_contracts::PrimitiveMetadata for Triangle3D<T> {
    fn primitive_kind(&self) -> geo_contracts::PrimitiveKind {
        geo_contracts::PrimitiveKind::Triangle
    }
}

impl<T: Scalar> geo_contracts::TolerantEq<T> for Triangle3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        let a_distance = self.vertex_a.distance_to(&other.vertex_a);
        let b_distance = self.vertex_b.distance_to(&other.vertex_b);
        let c_distance = self.vertex_c.distance_to(&other.vertex_c);

        a_distance <= tolerance && b_distance <= tolerance && c_distance <= tolerance
    }
}

impl<T: Scalar> Triangle3D<T> {
    /// 新しい3D三角形を作成
    ///
    /// 基本的な検証のみ実行（退化三角形チェック）
    pub fn new(vertex_a: Point3D<T>, vertex_b: Point3D<T>, vertex_c: Point3D<T>) -> Option<Self> {
        // 退化した三角形（3点が一直線上）を検証
        let ab = Vector3D::from_points(&vertex_a, &vertex_b);
        let ac = Vector3D::from_points(&vertex_a, &vertex_c);
        let zero_tol = default_kernel_numerical_zero_tolerance::<T>();

        // 外積の大きさが非常に小さい場合、3点が一直線上
        let cross = ab.cross(&ac);
        if cross.length() < zero_tol {
            return None;
        }
        Some(Self {
            vertex_a,
            vertex_b,
            vertex_c,
        })
    }

    /// タプルから三角形を作成
    pub fn from_points(points: (Point3D<T>, Point3D<T>, Point3D<T>)) -> Option<Self> {
        Self::new(points.0, points.1, points.2)
    }

    /// 辺ABのベクトルを取得
    pub fn edge_ab(&self) -> Vector3D<T> {
        Vector3D::from_points(&self.vertex_a, &self.vertex_b)
    }

    /// 辺BCのベクトルを取得
    pub fn edge_bc(&self) -> Vector3D<T> {
        Vector3D::from_points(&self.vertex_b, &self.vertex_c)
    }

    /// 辺CAのベクトルを取得
    pub fn edge_ca(&self) -> Vector3D<T> {
        Vector3D::from_points(&self.vertex_c, &self.vertex_a)
    }

    /// 法線ベクトルを計算（正規化済み）
    pub fn normal(&self) -> Option<Vector3D<T>> {
        let ab = self.edge_ab();
        let ac = Vector3D::from_points(&self.vertex_a, &self.vertex_c);

        let cross = ab.cross(&ac);
        if cross.length() < default_kernel_numerical_zero_tolerance::<T>() {
            None
        } else {
            Some(cross.normalize())
        }
    }

    /// 面積を計算
    pub fn area(&self) -> T {
        let ab = self.edge_ab();
        let ac = Vector3D::from_points(&self.vertex_a, &self.vertex_c);

        let cross = ab.cross(&ac);
        cross.length() / T::from_f64(2.0)
    }

    /// 三角形が退化していないかチェック
    pub fn is_valid(&self) -> bool {
        let area: T = self.area();
        let threshold = default_kernel_numerical_zero_tolerance::<T>();
        area > threshold
    }

    /// 指定した点が三角形の平面上にあるかチェック（バリセントリック座標使用）
    pub fn contains_point_on_plane(&self, point: Point3D<T>) -> bool {
        // バリセントリック座標で計算
        let v0 = Vector3D::from_points(&self.vertex_c, &self.vertex_a);
        let v1 = Vector3D::from_points(&self.vertex_c, &self.vertex_b);
        let v2 = Vector3D::from_points(&self.vertex_c, &point);

        let dot00 = v0.dot(&v0);
        let dot01 = v0.dot(&v1);
        let dot02 = v0.dot(&v2);
        let dot11 = v1.dot(&v1);
        let dot12 = v1.dot(&v2);

        let inv_denom = T::ONE / (dot00 * dot11 - dot01 * dot01);
        let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
        let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;

        // 三角形内部の条件
        u >= T::ZERO && v >= T::ZERO && (u + v) <= T::ONE
    }

    /// 点が三角形の内部または境界上にあるか（`classify_point` が `Outside` でない）
    pub fn contains_point(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.classify_point(point, tolerance) != PointClassification::Outside
    }

    /// 三角形の領域に対する点の位置を分類する
    ///
    /// 辺までの距離が `tolerance` 以内なら、平面外でも `OnBoundary` とする。それ以外は、平面からの距離が
    /// `tolerance` 以内で、平面へ投影した点が三角形の内側にあれば `Inside`、それ以外を `Outside` とする。
    /// 退化した三角形（法線を持たない）は内部を持たない。
    pub fn classify_point(&self, point: &Point3D<T>, tolerance: T) -> PointClassification {
        let edge_distance = self
            .distance_to_edge(point, &self.vertex_a, &self.vertex_b)
            .min(self.distance_to_edge(point, &self.vertex_b, &self.vertex_c))
            .min(self.distance_to_edge(point, &self.vertex_c, &self.vertex_a));
        if edge_distance <= tolerance {
            return PointClassification::OnBoundary;
        }

        let Some(normal) = self.normal() else {
            return PointClassification::Outside;
        };
        let plane_distance = Vector3D::from_points(&self.vertex_a, point).dot(&normal);
        if plane_distance.abs() <= tolerance && self.contains_point_on_plane(*point) {
            PointClassification::Inside
        } else {
            PointClassification::Outside
        }
    }

    /// 頂点順序を反転（法線方向を反転）
    pub fn reverse(&self) -> Self {
        Self {
            vertex_a: self.vertex_a,
            vertex_b: self.vertex_c,
            vertex_c: self.vertex_b,
        }
    }

    /// 外心座標を計算（三角形を含む平面上）
    pub fn circumcenter(&self) -> Option<Point3D<T>> {
        // 3D空間での外心計算は複雑なため、2D投影して計算
        // 法線を取得
        let normal = self.normal()?;

        // 三角形の平面上で2D座標系を構築
        let ab = self.edge_ab();
        let x_axis = ab.normalize();
        let y_axis = normal.cross(&x_axis);

        // 各頂点を2D座標に変換
        let a_2d = (T::ZERO, T::ZERO);
        let b_2d = (ab.length(), T::ZERO);

        let ac = Vector3D::from_points(&self.vertex_a, &self.vertex_c);
        let c_x = ac.dot(&x_axis);
        let c_y = ac.dot(&y_axis);
        let c_2d = (c_x, c_y);

        // 2D外心を計算
        let d =
            (a_2d.0 * (b_2d.1 - c_2d.1) + b_2d.0 * (c_2d.1 - a_2d.1) + c_2d.0 * (a_2d.1 - b_2d.1))
                * (T::ONE + T::ONE);
        if d.abs() < default_kernel_numerical_zero_tolerance::<T>() {
            return None;
        }

        let a_sq = a_2d.0 * a_2d.0 + a_2d.1 * a_2d.1;
        let b_sq = b_2d.0 * b_2d.0 + b_2d.1 * b_2d.1;
        let c_sq = c_2d.0 * c_2d.0 + c_2d.1 * c_2d.1;

        let ux =
            (a_sq * (b_2d.1 - c_2d.1) + b_sq * (c_2d.1 - a_2d.1) + c_sq * (a_2d.1 - b_2d.1)) / d;
        let uy =
            (a_sq * (c_2d.0 - b_2d.0) + b_sq * (a_2d.0 - c_2d.0) + c_sq * (b_2d.0 - a_2d.0)) / d;

        // 2D座標を3D空間に戻す
        let circumcenter = Point3D::new(
            self.vertex_a.x() + ux * x_axis.x() + uy * y_axis.x(),
            self.vertex_a.y() + ux * x_axis.y() + uy * y_axis.y(),
            self.vertex_a.z() + ux * x_axis.z() + uy * y_axis.z(),
        );

        Some(circumcenter)
    }

    /// 点から三角形までの最短距離を計算
    pub fn distance_to_point(&self, point: &Point3D<T>) -> T {
        // 点の平面への投影
        if let Some(normal) = self.normal() {
            let to_point = Vector3D::from_points(&self.vertex_a, point);
            let distance_to_plane = to_point.dot(&normal).abs();

            // 投影点を計算
            let projected = Point3D::new(
                point.x() - distance_to_plane * normal.x(),
                point.y() - distance_to_plane * normal.y(),
                point.z() - distance_to_plane * normal.z(),
            );

            // 投影点が三角形内部にある場合
            if self.contains_point_on_plane(projected) {
                return distance_to_plane;
            }

            // 三角形外部の場合、各辺からの最短距離を計算
            let dist_ab = self.distance_to_edge(point, &self.vertex_a, &self.vertex_b);
            let dist_bc = self.distance_to_edge(point, &self.vertex_b, &self.vertex_c);
            let dist_ca = self.distance_to_edge(point, &self.vertex_c, &self.vertex_a);

            dist_ab.min(dist_bc).min(dist_ca)
        } else {
            // 退化した三角形の場合、頂点までの最短距離
            let dist_a = Vector3D::from_points(&self.vertex_a, point).length();
            let dist_b = Vector3D::from_points(&self.vertex_b, point).length();
            let dist_c = Vector3D::from_points(&self.vertex_c, point).length();
            dist_a.min(dist_b).min(dist_c)
        }
    }

    /// 点から線分までの距離（ヘルパーメソッド）
    fn distance_to_edge(&self, point: &Point3D<T>, p1: &Point3D<T>, p2: &Point3D<T>) -> T {
        let edge = Vector3D::from_points(p1, p2);
        let to_point = Vector3D::from_points(p1, point);

        let edge_length_sq = edge.dot(&edge);
        if edge_length_sq < default_kernel_numerical_zero_tolerance::<T>() {
            return to_point.length();
        }

        let t = (to_point.dot(&edge) / edge_length_sq)
            .max(T::ZERO)
            .min(T::ONE);

        let closest = Point3D::new(
            p1.x() + t * edge.x(),
            p1.y() + t * edge.y(),
            p1.z() + t * edge.z(),
        );

        Vector3D::from_points(&closest, point).length()
    }
}

impl<T: Scalar> Triangle3DConstructor<T> for Triangle3D<T> {
    fn new(a: (T, T, T), b: (T, T, T), c: (T, T, T)) -> Option<Self> {
        let pa = Point3D::new(a.0, a.1, a.2);
        let pb = Point3D::new(b.0, b.1, b.2);
        let pc = Point3D::new(c.0, c.1, c.2);
        Self::new(pa, pb, pc)
    }
}

impl<T: Scalar> Triangle3DBoundaryAccess<T> for Triangle3D<T> {
    fn vertex_a(&self) -> (T, T, T) {
        let p = self.vertex_a;
        (p.x(), p.y(), p.z())
    }

    fn vertex_b(&self) -> (T, T, T) {
        let p = self.vertex_b;
        (p.x(), p.y(), p.z())
    }

    fn vertex_c(&self) -> (T, T, T) {
        let p = self.vertex_c;
        (p.x(), p.y(), p.z())
    }
}

impl<T: Scalar> Triangle3DDerived<T> for Triangle3D<T> {
    fn normal(&self) -> (T, T, T) {
        let n = Triangle3D::normal(self).unwrap_or(Vector3D::unit_z());
        (n.x(), n.y(), n.z())
    }
}

impl<T: Scalar> Triangle3DContainment<T> for Triangle3D<T> {
    fn contains_point(&self, point: (T, T, T)) -> bool {
        let p = Point3D::new(point.0, point.1, point.2);
        Triangle3D::contains_point(self, &p, default_distance_tolerance::<T>())
    }

    fn classify_point(&self, point: (T, T, T)) -> PointClassification {
        let p = Point3D::new(point.0, point.1, point.2);
        Triangle3D::classify_point(self, &p, default_distance_tolerance::<T>())
    }
}

impl<T: Scalar> Triangle3DDistance<T> for Triangle3D<T> {
    fn distance_to_point(&self, point: (T, T, T)) -> T {
        let p = Point3D::new(point.0, point.1, point.2);
        Triangle3D::distance_to_point(self, &p)
    }
}

impl<T: Scalar> std::fmt::Display for Triangle3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Triangle3D(A: {:?}, B: {:?}, C: {:?})",
            self.vertex_a, self.vertex_b, self.vertex_c
        )
    }
}
