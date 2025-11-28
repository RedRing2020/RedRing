//! Triangle3D Core 実装
//!
//! Foundation統一システムに基づくTriangle3Dの必須機能のみ

use crate::{Point3D, Vector3D};
use geo_foundation::{
    core::triangle_core_traits::{
        Triangle3DConstructor, Triangle3DMeasure, Triangle3DProperties,
    },
    Scalar,
};

/// 3次元三角形（Core実装）
///
/// Core機能のみ：
/// - 基本構築・検証
/// - アクセサメソッド
/// - 基本的な幾何プロパティ（面積、法線）
/// - 重心計算
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Triangle3D<T: Scalar> {
    vertex_a: Point3D<T>,
    vertex_b: Point3D<T>,
    vertex_c: Point3D<T>,
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> Triangle3D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================

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

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

    /// 頂点Aを取得
    pub fn vertex_a(&self) -> Point3D<T> {
        self.vertex_a
    }

    /// 頂点Bを取得
    pub fn vertex_b(&self) -> Point3D<T> {
        self.vertex_b
    }

    /// 頂点Cを取得
    pub fn vertex_c(&self) -> Point3D<T> {
        self.vertex_c
    }

    /// 全頂点を配列として取得
    pub fn vertices(&self) -> [Point3D<T>; 3] {
        [self.vertex_a, self.vertex_b, self.vertex_c]
    }

    // ========================================================================
    // Core Geometric Properties
    // ========================================================================

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

    // ========================================================================
    // Core Validation Methods
    // ========================================================================

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
}

// ============================================================================
// Core Traits Implementation (Phase 1)
// ============================================================================

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
        Self::new(pa, pb, pc)
            .expect("Unit triangle should always be valid")
    }
}

impl<T: Scalar> Triangle3DProperties<T> for Triangle3D<T> {
    fn vertex_a(&self) -> (T, T, T) {
        let p = self.vertex_a();
        (p.x(), p.y(), p.z())
    }

    fn vertex_b(&self) -> (T, T, T) {
        let p = self.vertex_b();
        (p.x(), p.y(), p.z())
    }

    fn vertex_c(&self) -> (T, T, T) {
        let p = self.vertex_c();
        (p.x(), p.y(), p.z())
    }

    fn centroid(&self) -> (T, T, T) {
        let c = self.centroid();
        (c.x(), c.y(), c.z())
    }

    fn normal(&self) -> (T, T, T) {
        let n = self.normal().unwrap_or(Vector3D::unit_z());
        (n.x(), n.y(), n.z())
    }
}

impl<T: Scalar> Triangle3DMeasure<T> for Triangle3D<T> {
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
}

// ============================================================================
// Display Implementation
// ============================================================================

impl<T: Scalar> std::fmt::Display for Triangle3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Triangle3D(A: {:?}, B: {:?}, C: {:?})",
            self.vertex_a, self.vertex_b, self.vertex_c
        )
    }
}
