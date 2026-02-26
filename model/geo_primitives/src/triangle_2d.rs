//! Triangle2D Core 実装
//!
//! Foundation統一システムに基づくTriangle2Dの必須機能のみ

use crate::{Point2D, Vector2D};
use geo_foundation::{
    core::triangle_traits::{Triangle2DConstructor, Triangle2DMeasure, Triangle2DProperties},
    Scalar,
};

/// 2次元三角形（Core実装）
///
/// Core機能のみ：
/// - 基本構築・検証
/// - アクセサメソッド
/// - 基本的な幾何プロパティ（面積、外心、重心）
/// - 辺の長さ計算
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Triangle2D<T: Scalar> {
    vertex_a: Point2D<T>,
    vertex_b: Point2D<T>,
    vertex_c: Point2D<T>,
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> Triangle2D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================

    /// 新しい2D三角形を作成
    ///
    /// 基本的な検証のみ実行（退化三角形チェック）
    pub fn new(vertex_a: Point2D<T>, vertex_b: Point2D<T>, vertex_c: Point2D<T>) -> Option<Self> {
        // 退化した三角形（3点が一直線上）を検証
        let ab = Vector2D::from_points(vertex_a, vertex_b);
        let ac = Vector2D::from_points(vertex_a, vertex_c);

        // 外積（2Dでは z成分のみ）の絶対値が非常に小さい場合、3点が一直線上
        let cross_z = ab.x() * ac.y() - ab.y() * ac.x();
        if cross_z.abs() < T::from_f64(1e-10) {
            return None;
        }

        Some(Self {
            vertex_a,
            vertex_b,
            vertex_c,
        })
    }

    /// 原点と単位ベクトルから正三角形を作成
    pub fn unit_triangle() -> Self {
        let h = T::from_f64(0.8660254037844387); // sqrt(3)/2
        Self::new(
            Point2D::new(T::ZERO, T::ONE),
            Point2D::new(-h, -T::ONE / (T::ONE + T::ONE)),
            Point2D::new(h, -T::ONE / (T::ONE + T::ONE)),
        )
        .expect("Unit triangle should always be valid")
    }

    /// 原点中心の正三角形を生成（辺の長さ指定）
    pub fn equilateral_at_origin(side_length: T) -> Self {
        let h = T::from_f64(0.8660254037844387); // sqrt(3)/2
        let half = side_length / (T::ONE + T::ONE);
        Self::new(
            Point2D::new(T::ZERO, side_length * h / T::from_f64(1.5)),
            Point2D::new(-half, -side_length * h / T::from_f64(3.0)),
            Point2D::new(half, -side_length * h / T::from_f64(3.0)),
        )
        .expect("Equilateral triangle should always be valid")
    }

    /// 直角二等辺三角形を生成（原点、x軸、y軸上）
    pub fn right_isosceles(leg_length: T) -> Self {
        Self::new(
            Point2D::new(T::ZERO, T::ZERO),
            Point2D::new(leg_length, T::ZERO),
            Point2D::new(T::ZERO, leg_length),
        )
        .expect("Right isosceles triangle should always be valid")
    }

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

    /// 頂点A取得（内部用）
    pub(crate) fn vertex_a_internal(&self) -> Point2D<T> {
        self.vertex_a
    }

    /// 頂点B取得（内部用）
    pub(crate) fn vertex_b_internal(&self) -> Point2D<T> {
        self.vertex_b
    }

    /// 頂点C取得（内部用）
    pub(crate) fn vertex_c_internal(&self) -> Point2D<T> {
        self.vertex_c
    }

    // ========================================================================
    // Core Geometric Properties
    // ========================================================================

    /// 辺AB取得
    pub fn edge_ab(&self) -> Vector2D<T> {
        Vector2D::from_points(self.vertex_a, self.vertex_b)
    }

    /// 辺BC取得
    pub fn edge_bc(&self) -> Vector2D<T> {
        Vector2D::from_points(self.vertex_b, self.vertex_c)
    }

    /// 辺CA取得
    pub fn edge_ca(&self) -> Vector2D<T> {
        Vector2D::from_points(self.vertex_c, self.vertex_a)
    }

    /// 辺の長さ配列取得
    pub fn edge_lengths(&self) -> [T; 3] {
        [
            self.edge_ab().length(),
            self.edge_bc().length(),
            self.edge_ca().length(),
        ]
    }

    /// 周囲長計算
    pub fn perimeter(&self) -> T {
        let lengths = self.edge_lengths();
        lengths[0] + lengths[1] + lengths[2]
    }

    /// 面積計算（外積の半分）
    pub fn area(&self) -> T {
        let ab = self.edge_ab();
        let ac = Vector2D::from_points(self.vertex_a, self.vertex_c);

        // 外積のZ成分の絶対値の半分
        let cross_z = ab.x() * ac.y() - ab.y() * ac.x();
        cross_z.abs() / (T::ONE + T::ONE)
    }

    /// 重心計算
    pub fn centroid(&self) -> Point2D<T> {
        let three = T::ONE + T::ONE + T::ONE;
        Point2D::new(
            (self.vertex_a.x() + self.vertex_b.x() + self.vertex_c.x()) / three,
            (self.vertex_a.y() + self.vertex_b.y() + self.vertex_c.y()) / three,
        )
    }

    /// 外心計算（外接円の中心）
    pub fn circumcenter(&self) -> Option<Point2D<T>> {
        let ax = self.vertex_a.x();
        let ay = self.vertex_a.y();
        let bx = self.vertex_b.x();
        let by = self.vertex_b.y();
        let cx = self.vertex_c.x();
        let cy = self.vertex_c.y();

        // 行列式計算
        let d = (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by)) * (T::ONE + T::ONE);
        if d.abs() < T::from_f64(1e-10) {
            return None; // 退化した三角形
        }

        // 外心座標計算
        let a_sq = ax * ax + ay * ay;
        let b_sq = bx * bx + by * by;
        let c_sq = cx * cx + cy * cy;

        let ux = (a_sq * (by - cy) + b_sq * (cy - ay) + c_sq * (ay - by)) / d;
        let uy = (a_sq * (cx - bx) + b_sq * (ax - cx) + c_sq * (bx - ax)) / d;

        Some(Point2D::new(ux, uy))
    }

    /// 外接円の半径
    pub fn circumradius(&self) -> Option<T> {
        let circumcenter = self.circumcenter()?;
        let distance_a = Vector2D::from_points(circumcenter, self.vertex_a).length();
        Some(distance_a)
    }

    /// 内心計算（内接円の中心）
    pub fn incenter(&self) -> Point2D<T> {
        let [a, b, c] = self.edge_lengths();
        let perimeter = a + b + c;

        // 辺長による重み付き重心
        let ix =
            (a * self.vertex_a.x() + b * self.vertex_b.x() + c * self.vertex_c.x()) / perimeter;
        let iy =
            (a * self.vertex_a.y() + b * self.vertex_b.y() + c * self.vertex_c.y()) / perimeter;

        Point2D::new(ix, iy)
    }

    /// 内接円の半径
    pub fn inradius(&self) -> T {
        let area = self.area();
        let perimeter = self.perimeter();
        (area * (T::ONE + T::ONE)) / perimeter
    }

    // ========================================================================
    // Core Query Methods
    // ========================================================================

    /// 点が三角形内部にあるかの判定（重心座標使用）
    pub fn contains_point(&self, point: &Point2D<T>) -> bool {
        let (u, v, w) = self.barycentric_coordinates(point);
        u >= T::ZERO && v >= T::ZERO && w >= T::ZERO
    }

    /// 重心座標計算
    pub fn barycentric_coordinates(&self, point: &Point2D<T>) -> (T, T, T) {
        let v0 = Vector2D::from_points(self.vertex_a, self.vertex_c);
        let v1 = Vector2D::from_points(self.vertex_a, self.vertex_b);
        let v2 = Vector2D::from_points(self.vertex_a, *point);

        let dot00 = v0.dot(&v0);
        let dot01 = v0.dot(&v1);
        let dot02 = v0.dot(&v2);
        let dot11 = v1.dot(&v1);
        let dot12 = v1.dot(&v2);

        let inv_denom = T::ONE / (dot00 * dot11 - dot01 * dot01);
        let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
        let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;
        let w = T::ONE - u - v;

        (u, v, w)
    }

    /// 三角形が時計回りかの判定
    pub fn is_clockwise(&self) -> bool {
        let ab = self.edge_ab();
        let ac = Vector2D::from_points(self.vertex_a, self.vertex_c);
        let cross_z = ab.x() * ac.y() - ab.y() * ac.x();
        cross_z < T::ZERO
    }

    /// 三角形の向きを反転
    pub fn reverse(&self) -> Self {
        Self {
            vertex_a: self.vertex_a,
            vertex_b: self.vertex_c, // B と C を入れ替え
            vertex_c: self.vertex_b,
        }
    }

    /// 点から三角形までの最短距離を計算
    pub fn distance_to_point(&self, point: &Point2D<T>) -> T {
        // 点が三角形内部にある場合、距離は0
        if self.contains_point(point) {
            return T::ZERO;
        }

        // 各辺から点までの距離の最小値を計算
        let dist_ab = self.distance_to_edge(point, self.vertex_a, self.vertex_b);
        let dist_bc = self.distance_to_edge(point, self.vertex_b, self.vertex_c);
        let dist_ca = self.distance_to_edge(point, self.vertex_c, self.vertex_a);

        dist_ab.min(dist_bc).min(dist_ca)
    }

    /// 点から線分までの距離を計算（ヘルパーメソッド）
    fn distance_to_edge(&self, point: &Point2D<T>, p1: Point2D<T>, p2: Point2D<T>) -> T {
        let edge = Vector2D::from_points(p1, p2);
        let to_point = Vector2D::from_points(p1, *point);

        let edge_length_sq = edge.dot(&edge);
        if edge_length_sq < T::from_f64(1e-10) {
            // 退化した辺の場合、点p1までの距離
            return to_point.length();
        }

        // パラメータt: 点からの最短距離を与える辺上の位置
        let t = (to_point.dot(&edge) / edge_length_sq)
            .max(T::ZERO)
            .min(T::ONE);

        // 辺上の最近点
        let closest = Point2D::new(p1.x() + t * edge.x(), p1.y() + t * edge.y());

        Vector2D::from_points(closest, *point).length()
    }
}

// ============================================================================
// Default Implementations
// ============================================================================

// Default実装（unit triangle）
impl<T: Scalar> Default for Triangle2D<T> {
    fn default() -> Self {
        Self::unit_triangle()
    }
}

// ============================================================================
// Core Traits Implementation (Phase 1)
// ============================================================================

impl<T: Scalar> Triangle2DConstructor<T> for Triangle2D<T> {
    fn new(a: (T, T), b: (T, T), c: (T, T)) -> Option<Self> {
        let pa = Point2D::new(a.0, a.1);
        let pb = Point2D::new(b.0, b.1);
        let pc = Point2D::new(c.0, c.1);
        Self::new(pa, pb, pc)
    }

    fn unit_triangle() -> Self {
        Self::unit_triangle()
    }

    fn from_array(points: [(T, T); 3]) -> Option<Self> {
        let pa = Point2D::new(points[0].0, points[0].1);
        let pb = Point2D::new(points[1].0, points[1].1);
        let pc = Point2D::new(points[2].0, points[2].1);
        Self::new(pa, pb, pc)
    }

    fn equilateral_at_origin(side_length: T) -> Self {
        Self::equilateral_at_origin(side_length)
    }

    fn right_isosceles(leg_length: T) -> Self {
        Self::right_isosceles(leg_length)
    }

    fn reversed(&self) -> Self {
        self.reverse()
    }
}

impl<T: Scalar> Triangle2DProperties<T> for Triangle2D<T> {
    fn vertex_a(&self) -> (T, T) {
        let p = self.vertex_a_internal();
        (p.x(), p.y())
    }

    fn vertex_b(&self) -> (T, T) {
        let p = self.vertex_b_internal();
        (p.x(), p.y())
    }

    fn vertex_c(&self) -> (T, T) {
        let p = self.vertex_c_internal();
        (p.x(), p.y())
    }

    fn centroid(&self) -> (T, T) {
        let c = self.centroid();
        (c.x(), c.y())
    }

    fn circumcenter(&self) -> Option<(T, T)> {
        self.circumcenter().map(|c| (c.x(), c.y()))
    }

    fn incenter(&self) -> (T, T) {
        let i = self.incenter();
        (i.x(), i.y())
    }

    fn circumradius(&self) -> Option<T> {
        self.circumradius()
    }

    fn inradius(&self) -> T {
        self.inradius()
    }
}

impl<T: Scalar> Triangle2DMeasure<T> for Triangle2D<T> {
    fn measure(&self) -> T {
        self.area()
    }

    fn edge_ab_length(&self) -> T {
        self.edge_ab().length()
    }

    fn edge_bc_length(&self) -> T {
        self.edge_bc().length()
    }

    fn edge_ca_length(&self) -> T {
        self.edge_ca().length()
    }

    fn perimeter(&self) -> T {
        self.perimeter()
    }

    fn contains_point(&self, point: (T, T)) -> bool {
        let p = Point2D::new(point.0, point.1);
        self.contains_point(&p)
    }

    fn is_clockwise(&self) -> bool {
        self.is_clockwise()
    }

    fn distance_to_point(&self, point: (T, T)) -> T {
        let p = Point2D::new(point.0, point.1);
        self.distance_to_point(&p)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_creation() {
        let triangle = Triangle2D::new(
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 0.0),
            Point2D::new(0.5, 1.0),
        );
        assert!(triangle.is_some());

        // 退化した三角形（一直線上の点）
        let degenerate = Triangle2D::new(
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 0.0),
            Point2D::new(2.0, 0.0),
        );
        assert!(degenerate.is_none());
    }

    #[test]
    fn test_area_calculation() {
        let triangle = Triangle2D::new(
            Point2D::new(0.0, 0.0),
            Point2D::new(2.0, 0.0),
            Point2D::new(1.0, 2.0),
        )
        .unwrap();

        assert_eq!(triangle.area(), 2.0);
    }

    #[test]
    fn test_centroid() {
        let triangle = Triangle2D::new(
            Point2D::new(0.0, 0.0),
            Point2D::new(3.0, 0.0),
            Point2D::new(0.0, 3.0),
        )
        .unwrap();

        let centroid = triangle.centroid();
        assert_eq!(centroid, Point2D::new(1.0, 1.0));
    }

    #[test]
    fn test_contains_point() {
        let triangle = Triangle2D::new(
            Point2D::new(0.0, 0.0),
            Point2D::new(2.0, 0.0),
            Point2D::new(1.0, 2.0),
        )
        .unwrap();

        // 内部の点
        assert!(triangle.contains_point(&Point2D::new(1.0, 0.5)));

        // 外部の点
        assert!(!triangle.contains_point(&Point2D::new(3.0, 3.0)));

        // 重心は必ず内部
        assert!(triangle.contains_point(&triangle.centroid()));
    }

    #[test]
    fn test_orientation() {
        let ccw_triangle = Triangle2D::new(
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 0.0),
            Point2D::new(0.5, 1.0),
        )
        .unwrap();

        assert!(!ccw_triangle.is_clockwise());

        let cw_triangle = ccw_triangle.reverse();
        assert!(cw_triangle.is_clockwise());
    }

    #[test]
    fn test_circumcenter() {
        // 正三角形の外心は重心と一致
        let triangle: Triangle2D<f64> = Triangle2D::unit_triangle();
        let circumcenter = triangle.circumcenter().unwrap();
        let centroid = triangle.centroid();

        assert!((circumcenter.x() - centroid.x()).abs() < 1e-10);
        assert!((circumcenter.y() - centroid.y()).abs() < 1e-10);
    }
}
