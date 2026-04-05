//! Triangle3D Core 実装
//!
//! Foundation統一システムに基づくTriangle3Dの必須機能のみ

use crate::{Point3D, Vector3D};
use geo_contracts::{
    Scalar, Triangle3DBoundaryAccess, Triangle3DBoundaryQuantity, Triangle3DConstructor,
    Triangle3DContainment, Triangle3DDerived, Triangle3DDistance,
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
        let a_distance = self
            .vertex_a_internal()
            .distance_to(&other.vertex_a_internal());
        let b_distance = self
            .vertex_b_internal()
            .distance_to(&other.vertex_b_internal());
        let c_distance = self
            .vertex_c_internal()
            .distance_to(&other.vertex_c_internal());

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

        // 外積の大きさが非常に小さい場合、3点が一直線上
        let cross = ab.cross(&ac);
        if cross.length() < T::from_f64(1e-10) {
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

    /// 頂点Aを取得（内部用）
    pub(crate) fn vertex_a_internal(&self) -> Point3D<T> {
        self.vertex_a
    }

    /// 頂点Bを取得（内部用）
    pub(crate) fn vertex_b_internal(&self) -> Point3D<T> {
        self.vertex_b
    }

    /// 頂点Cを取得（内部用）
    pub(crate) fn vertex_c_internal(&self) -> Point3D<T> {
        self.vertex_c
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
        if cross.length() < T::from_f64(1e-10) {
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

    /// 重心を計算
    pub fn centroid(&self) -> Point3D<T> {
        let x = (self.vertex_a.x() + self.vertex_b.x() + self.vertex_c.x()) / T::from_f64(3.0);
        let y = (self.vertex_a.y() + self.vertex_b.y() + self.vertex_c.y()) / T::from_f64(3.0);
        let z = (self.vertex_a.z() + self.vertex_b.z() + self.vertex_c.z()) / T::from_f64(3.0);

        Point3D::new(x, y, z)
    }

    /// 周囲長を計算
    pub fn perimeter(&self) -> T {
        let ab_length: T = self.edge_ab().length();
        let bc_length: T = self.edge_bc().length();
        let ca_length: T = self.edge_ca().length();

        ab_length + bc_length + ca_length
    }

    /// 三角形が退化していないかチェック
    pub fn is_valid(&self) -> bool {
        let area: T = self.area();
        let threshold = T::from_f64(1e-10);
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

    /// xz平面上の単位正三角形を生成
    pub fn unit_triangle_xz() -> Self {
        let h = T::from_f64(0.8660254037844387); // sqrt(3)/2
        Self::new(
            Point3D::new(T::ZERO, T::ZERO, T::ONE),
            Point3D::new(-h, T::ZERO, -T::ONE / (T::ONE + T::ONE)),
            Point3D::new(h, T::ZERO, -T::ONE / (T::ONE + T::ONE)),
        )
        .expect("Unit triangle should always be valid")
    }

    /// yz平面上の単位正三角形を生成
    pub fn unit_triangle_yz() -> Self {
        let h = T::from_f64(0.8660254037844387); // sqrt(3)/2
        Self::new(
            Point3D::new(T::ZERO, T::ZERO, T::ONE),
            Point3D::new(T::ZERO, -h, -T::ONE / (T::ONE + T::ONE)),
            Point3D::new(T::ZERO, h, -T::ONE / (T::ONE + T::ONE)),
        )
        .expect("Unit triangle should always be valid")
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
        if d.abs() < T::from_f64(1e-10) {
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

    /// 外接円の半径を計算
    pub fn circumradius(&self) -> Option<T> {
        let circumcenter = self.circumcenter()?;
        let distance = Vector3D::from_points(&circumcenter, &self.vertex_a).length();
        Some(distance)
    }

    /// 内接円の半径を計算
    pub fn inradius(&self) -> T {
        let area = self.area();
        let perimeter = self.perimeter();
        (area * (T::ONE + T::ONE)) / perimeter
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
        if edge_length_sq < T::from_f64(1e-10) {
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

    /// 三角形が平面上にあるか判定（常に true）
    pub fn is_planar(&self) -> bool {
        true // 三角形は常に平面上
    }
}

impl<T: Scalar> Triangle3DConstructor<T> for Triangle3D<T> {
    fn new(a: (T, T, T), b: (T, T, T), c: (T, T, T)) -> Option<Self> {
        let pa = Point3D::new(a.0, a.1, a.2);
        let pb = Point3D::new(b.0, b.1, b.2);
        let pc = Point3D::new(c.0, c.1, c.2);
        Self::new(pa, pb, pc)
    }

    fn from_array(points: [(T, T, T); 3]) -> Option<Self> {
        let pa = Point3D::new(points[0].0, points[0].1, points[0].2);
        let pb = Point3D::new(points[1].0, points[1].1, points[1].2);
        let pc = Point3D::new(points[2].0, points[2].1, points[2].2);
        Self::new(pa, pb, pc)
    }

    fn unit_triangle_xy() -> Self {
        let h = T::from_f64(0.8660254037844387); // sqrt(3)/2
        let pa = Point3D::new(T::ZERO, T::ONE, T::ZERO);
        let pb = Point3D::new(-h, -T::ONE / (T::ONE + T::ONE), T::ZERO);
        let pc = Point3D::new(h, -T::ONE / (T::ONE + T::ONE), T::ZERO);
        Self::new(pa, pb, pc).expect("Unit triangle should always be valid")
    }

    fn unit_triangle_xz() -> Self {
        Self::unit_triangle_xz()
    }

    fn unit_triangle_yz() -> Self {
        Self::unit_triangle_yz()
    }

    fn reversed(&self) -> Self {
        self.reverse()
    }
}

impl<T: Scalar> Triangle3DBoundaryAccess<T> for Triangle3D<T> {
    fn vertex_a(&self) -> (T, T, T) {
        let p = self.vertex_a_internal();
        (p.x(), p.y(), p.z())
    }

    fn vertex_b(&self) -> (T, T, T) {
        let p = self.vertex_b_internal();
        (p.x(), p.y(), p.z())
    }

    fn vertex_c(&self) -> (T, T, T) {
        let p = self.vertex_c_internal();
        (p.x(), p.y(), p.z())
    }
}

impl<T: Scalar> Triangle3DDerived<T> for Triangle3D<T> {
    fn centroid(&self) -> (T, T, T) {
        let c = Triangle3D::centroid(self);
        (c.x(), c.y(), c.z())
    }

    fn normal(&self) -> (T, T, T) {
        let n = Triangle3D::normal(self).unwrap_or(Vector3D::unit_z());
        (n.x(), n.y(), n.z())
    }

    fn circumcenter(&self) -> Option<(T, T, T)> {
        Triangle3D::circumcenter(self).map(|c| (c.x(), c.y(), c.z()))
    }

    fn circumradius(&self) -> Option<T> {
        Triangle3D::circumradius(self)
    }

    fn inradius(&self) -> T {
        Triangle3D::inradius(self)
    }

    fn area(&self) -> T {
        Triangle3D::area(self)
    }

    fn is_planar(&self) -> bool {
        Triangle3D::is_planar(self)
    }
}

impl<T: Scalar> Triangle3DBoundaryQuantity<T> for Triangle3D<T> {
    fn edge_ab_length(&self) -> T {
        Triangle3D::edge_ab(self).length()
    }

    fn edge_bc_length(&self) -> T {
        Triangle3D::edge_bc(self).length()
    }

    fn edge_ca_length(&self) -> T {
        Triangle3D::edge_ca(self).length()
    }

    fn perimeter(&self) -> T {
        Triangle3D::perimeter(self)
    }
}

impl<T: Scalar> Triangle3DContainment<T> for Triangle3D<T> {
    fn contains_point(&self, point: (T, T, T)) -> bool {
        let p = Point3D::new(point.0, point.1, point.2);
        Triangle3D::contains_point_on_plane(self, p)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point3D;
    use geo_contracts::TolerantEq;

    #[test]
    fn test_tolerant_eq() {
        let triangle1 = Triangle3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        )
        .unwrap();
        let triangle2 = Triangle3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        )
        .unwrap();
        let triangle3 = Triangle3D::new(
            Point3D::new(1.0, 1.0, 1.0),
            Point3D::new(2.0, 1.0, 1.0),
            Point3D::new(1.0, 2.0, 1.0),
        )
        .unwrap();

        assert!(triangle1.tolerant_eq(&triangle2, 0.01));
        assert!(!triangle1.tolerant_eq(&triangle3, 0.01));
    }
}
