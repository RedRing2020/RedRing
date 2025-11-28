//! Point3D Core 実装
//!
//! Foundation統一システムに基づくPoint3Dの必須機能のみ

use crate::Vector3D;
use analysis::linalg::vector::Vector3;
use geo_foundation::{
    core::{
        point_core_traits::{
            Point3DConstructor, Point3DCore, Point3DMeasure, Point3DProperties,
        },
        point_traits,
    },
    Scalar,
};

/// 3次元空間の点
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3D<T: Scalar> {
    x: T,
    y: T,
    z: T,
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> Point3D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================

    /// 新しい点を作成
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }

    /// 原点を取得
    pub fn origin() -> Self {
        Self::new(T::ZERO, T::ZERO, T::ZERO)
    }

    /// タプルから点を作成
    pub fn from_tuple(coords: (T, T, T)) -> Self {
        Self::new(coords.0, coords.1, coords.2)
    }

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

    /// X座標を取得
    pub fn x(&self) -> T {
        self.x
    }

    /// Y座標を取得
    pub fn y(&self) -> T {
        self.y
    }

    /// Z座標を取得
    pub fn z(&self) -> T {
        self.z
    }

    /// 座標を配列として取得
    pub fn coords(&self) -> [T; 3] {
        [self.x, self.y, self.z]
    }

    // ========================================================================
    // Core Calculation Methods
    // ========================================================================

    /// 他の点との距離を計算
    pub fn distance_to(&self, other: &Self) -> T {
        self.distance_squared_to(other).sqrt()
    }

    /// 他の点との距離の二乗を計算（sqrt回避で高速）
    pub fn distance_squared_to(&self, other: &Self) -> T {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }

    /// 原点からの距離（ノルム）
    pub fn norm(&self) -> T {
        self.norm_squared().sqrt()
    }

    /// 原点からの距離の二乗
    pub fn norm_squared(&self) -> T {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    // ========================================================================
    // Phase 2 Constructor Methods
    // ========================================================================

    /// 球面座標から点を作成（r: 半径, theta: 方位角, phi: 仰角）
    /// theta: xy平面での角度（0 = +x軸）
    /// phi: z軸からの角度（0 = +z軸, π/2 = xy平面）
    pub fn from_spherical(r: T, theta: T, phi: T) -> Self {
        let sin_phi = phi.sin();
        Self::new(
            r * sin_phi * theta.cos(),
            r * sin_phi * theta.sin(),
            r * phi.cos(),
        )
    }

    // ========================================================================
    // Phase 2 Measure Methods
    // ========================================================================

    /// 2点の中点を計算
    pub fn midpoint(&self, other: &Self) -> Self {
        let two = T::ONE + T::ONE;
        Self::new(
            (self.x + other.x) / two,
            (self.y + other.y) / two,
            (self.z + other.z) / two,
        )
    }

    /// 別の点との線形補間（t=0で自分、t=1で相手）
    pub fn lerp(&self, other: &Self, t: T) -> Self {
        Self::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
            self.z + (other.z - self.z) * t,
        )
    }

    /// マンハッタン距離（L1ノルム）
    pub fn manhattan_distance_to(&self, other: &Self) -> T {
        (self.x - other.x).abs() + (self.y - other.y).abs() + (self.z - other.z).abs()
    }

    /// チェビシェフ距離（L∞ノルム）
    pub fn chebyshev_distance_to(&self, other: &Self) -> T {
        let dx = (self.x - other.x).abs();
        let dy = (self.y - other.y).abs();
        let dz = (self.z - other.z).abs();
        dx.max(dy).max(dz)
    }

    // ========================================================================
    // Legacy Methods (kept for compatibility)
    // ========================================================================

    /// 点が境界上（許容誤差内）にあるかを判定
    pub fn on_boundary(&self, point: &Self, tolerance: T) -> bool {
        self.distance_to(point) <= tolerance
    }

    /// 点が自分自身と一致するかを判定
    pub fn contains_point(&self, point: &Self) -> bool {
        self == point
    }

    // ========================================================================
    // Conversion Methods
    // ========================================================================
}

// ============================================================================
// Operator Implementations
// ============================================================================

// Point - Point = Vector (2点間のベクトル)
impl<T: Scalar> std::ops::Sub for Point3D<T> {
    type Output = Vector3D<T>;

    fn sub(self, other: Self) -> Self::Output {
        Vector3D::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

// Point + Vector = Point (点をベクトル分移動)
impl<T: Scalar> std::ops::Add<Vector3D<T>> for Point3D<T> {
    type Output = Point3D<T>;

    fn add(self, vector: Vector3D<T>) -> Self::Output {
        Point3D::new(
            self.x + vector.x(),
            self.y + vector.y(),
            self.z + vector.z(),
        )
    }
}

// Point - Vector = Point (点をベクトル分逆移動)
impl<T: Scalar> std::ops::Sub<Vector3D<T>> for Point3D<T> {
    type Output = Point3D<T>;

    fn sub(self, vector: Vector3D<T>) -> Self::Output {
        Point3D::new(
            self.x - vector.x(),
            self.y - vector.y(),
            self.z - vector.z(),
        )
    }
}

// 基本機能のみに集中 - 複雑な変換は将来のextensionトレイトで実装予定

// ============================================================================
// geo_foundation abstracts trait implementations
// ============================================================================

/// geo_foundation::core::Point2D<T> トレイト実装
impl<T: Scalar> point_traits::Point2D<T> for Point3D<T> {
    fn x(&self) -> T {
        self.x
    }

    fn y(&self) -> T {
        self.y
    }
}

/// geo_foundation::core::Point3D<T> トレイト実装
impl<T: Scalar> point_traits::Point3D<T> for Point3D<T> {
    fn z(&self) -> T {
        self.z
    }
}

// ============================================================================
// Core Traits Implementation (Foundation Pattern)
// ============================================================================

impl<T: Scalar> Point3DConstructor<T> for Point3D<T> {
    fn new(x: T, y: T, z: T) -> Self {
        Point3D::new(x, y, z)
    }

    fn origin() -> Self {
        Point3D::origin()
    }

    fn from_tuple(coords: (T, T, T)) -> Self {
        Point3D::from_tuple(coords)
    }

    fn from_analysis_vector(vector: &Vector3<T>) -> Self {
        Point3D::new(vector.x(), vector.y(), vector.z())
    }

    fn from_point(other: &Self) -> Self {
        *other
    }

    fn from_spherical(r: T, theta: T, phi: T) -> Self {
        Point3D::from_spherical(r, theta, phi)
    }
}

impl<T: Scalar> Point3DProperties<T> for Point3D<T> {
    fn x(&self) -> T {
        self.x()
    }

    fn y(&self) -> T {
        self.y()
    }

    fn z(&self) -> T {
        self.z()
    }

    fn coords(&self) -> [T; 3] {
        self.coords()
    }

    fn to_tuple(&self) -> (T, T, T) {
        (self.x(), self.y(), self.z())
    }

    fn to_analysis_vector(&self) -> Vector3<T> {
        Vector3::new(self.x(), self.y(), self.z())
    }
}

impl<T: Scalar> Point3DMeasure<T> for Point3D<T> {
    fn distance_to(&self, other: &Self) -> T {
        self.distance_to(other)
    }

    fn distance_squared_to(&self, other: &Self) -> T {
        self.distance_squared_to(other)
    }

    fn distance_from_origin(&self) -> T {
        let origin = Point3D::origin();
        self.distance_to(&origin)
    }

    fn norm_squared(&self) -> T {
        let origin = Point3D::origin();
        self.distance_squared_to(&origin)
    }

    fn midpoint(&self, other: &Self) -> Self {
        self.midpoint(other)
    }

    fn lerp(&self, other: &Self, t: T) -> Self {
        self.lerp(other, t)
    }

    fn manhattan_distance_to(&self, other: &Self) -> T {
        self.manhattan_distance_to(other)
    }

    fn chebyshev_distance_to(&self, other: &Self) -> T {
        self.chebyshev_distance_to(other)
    }
}

impl<T: Scalar> Point3DCore<T> for Point3D<T> {}

// ============================================================================
// From trait implementations
// ============================================================================

/// タプルからの変換
impl<T: Scalar> From<(T, T, T)> for Point3D<T> {
    fn from(tuple: (T, T, T)) -> Self {
        Self::new(tuple.0, tuple.1, tuple.2)
    }
}

// ============================================================================
// Analysis Integration (Analysis変換機能)
// ============================================================================

impl<T: Scalar> Point3D<T> {
    /// analysis::Point3に変換
    pub fn to_analysis_point3(&self) -> analysis::linalg::point3::Point3<T> {
        analysis::linalg::point3::Point3::new(self.x, self.y, self.z)
    }

    /// analysis::Point3から作成
    pub fn from_analysis_point3(p: analysis::linalg::point3::Point3<T>) -> Self {
        Self::new(p.x(), p.y(), p.z())
    }

    /// analysis::Vector3に変換（位置ベクトルとして）
    pub fn to_analysis_vector3(&self) -> analysis::linalg::vector::Vector3<T> {
        self.to_analysis_point3().to_vector()
    }

    /// analysis::Vector3から作成（位置ベクトルから）
    pub fn from_analysis_vector3(v: analysis::linalg::vector::Vector3<T>) -> Self {
        let point = analysis::linalg::point3::Point3::from_vector(v);
        Self::from_analysis_point3(point)
    }
}

/// analysis::Point3からの変換
impl<T: Scalar> From<analysis::linalg::point3::Point3<T>> for Point3D<T> {
    fn from(p: analysis::linalg::point3::Point3<T>) -> Self {
        Self::from_analysis_point3(p)
    }
}

/// analysis::Point3への変換
impl<T: Scalar> From<Point3D<T>> for analysis::linalg::point3::Point3<T> {
    fn from(p: Point3D<T>) -> Self {
        p.to_analysis_point3()
    }
}
