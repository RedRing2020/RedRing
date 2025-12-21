//! Point3D - 3次元点の完全実装
//!
//! geo_core における Point3D の完全実装。
//! 基本機能、Foundation トレイト、Analysis 変換、演算子オーバーロードを含む。

use geo_foundation::{
    core::{
        point_core_traits::{Point3DConstructor, Point3DCore, Point3DMeasure, Point3DProperties},
        point_traits,
    },
    Scalar,
};

/// 3次元空間の点
///
/// ## 設計方針
/// - geo_core の基本型として全機能を実装
/// - Foundation トレイト完全対応
/// - analysis::Point3 との相互変換対応
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3D<T: Scalar> {
    x: T,
    y: T,
    z: T,
}

// ============================================================================
// Core Implementation (基本機能のみ)
// ============================================================================

impl<T: Scalar> Point3D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================

    /// 新しい点を作成
    ///
    /// # Examples
    /// ```
    /// use geo_core::Point3D;
    ///
    /// let p = Point3D::new(1.0, 2.0, 3.0);
    /// assert_eq!(p.x(), 1.0);
    /// assert_eq!(p.y(), 2.0);
    /// assert_eq!(p.z(), 3.0);
    /// ```
    #[inline]
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }

    /// 原点を取得
    ///
    /// # Examples
    /// ```
    /// use geo_core::Point3D;
    ///
    /// let origin = Point3D::<f64>::origin();
    /// assert_eq!(origin.x(), 0.0);
    /// assert_eq!(origin.y(), 0.0);
    /// assert_eq!(origin.z(), 0.0);
    /// ```
    #[inline]
    pub fn origin() -> Self {
        Self::new(T::ZERO, T::ZERO, T::ZERO)
    }

    /// タプルから点を作成
    ///
    /// # Examples
    /// ```
    /// use geo_core::Point3D;
    ///
    /// let p = Point3D::from_tuple((1.0, 2.0, 3.0));
    /// assert_eq!(p.x(), 1.0);
    /// ```
    #[inline]
    pub fn from_tuple(coords: (T, T, T)) -> Self {
        Self::new(coords.0, coords.1, coords.2)
    }

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

    /// X座標を取得
    #[inline]
    pub fn x(&self) -> T {
        self.x
    }

    /// Y座標を取得
    #[inline]
    pub fn y(&self) -> T {
        self.y
    }

    /// Z座標を取得
    #[inline]
    pub fn z(&self) -> T {
        self.z
    }

    /// 座標を配列として取得
    #[inline]
    pub fn coords(&self) -> [T; 3] {
        [self.x, self.y, self.z]
    }

    // ========================================================================
    // Core Calculation Methods
    // ========================================================================

    /// 他の点との距離を計算
    ///
    /// # Examples
    /// ```
    /// use geo_core::Point3D;
    ///
    /// let p1 = Point3D::new(0.0, 0.0, 0.0);
    /// let p2 = Point3D::new(3.0, 4.0, 0.0);
    /// assert_eq!(p1.distance_to(&p2), 5.0);
    /// ```
    pub fn distance_to(&self, other: &Self) -> T {
        self.distance_squared_to(other).sqrt()
    }

    /// 他の点との距離の二乗を計算（sqrt回避で高速）
    ///
    /// # Examples
    /// ```
    /// use geo_core::Point3D;
    ///
    /// let p1 = Point3D::new(0.0, 0.0, 0.0);
    /// let p2 = Point3D::new(3.0, 4.0, 0.0);
    /// assert_eq!(p1.distance_squared_to(&p2), 25.0);
    /// ```
    #[inline]
    pub fn distance_squared_to(&self, other: &Self) -> T {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }

    /// 原点からの距離（ノルム）
    ///
    /// # Examples
    /// ```
    /// use geo_core::Point3D;
    ///
    /// let p = Point3D::new(3.0, 4.0, 0.0);
    /// assert_eq!(p.norm(), 5.0);
    /// ```
    #[inline]
    pub fn norm(&self) -> T {
        self.norm_squared().sqrt()
    }

    /// 原点からの距離の二乗
    #[inline]
    pub fn norm_squared(&self) -> T {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// 2点の中点を計算
    ///
    /// # Examples
    /// ```
    /// use geo_core::Point3D;
    ///
    /// let p1 = Point3D::new(0.0, 0.0, 0.0);
    /// let p2 = Point3D::new(2.0, 4.0, 6.0);
    /// let mid = p1.midpoint(&p2);
    /// assert_eq!(mid.x(), 1.0);
    /// assert_eq!(mid.y(), 2.0);
    /// assert_eq!(mid.z(), 3.0);
    /// ```
    pub fn midpoint(&self, other: &Self) -> Self {
        let two = T::ONE + T::ONE;
        Self::new(
            (self.x + other.x) / two,
            (self.y + other.y) / two,
            (self.z + other.z) / two,
        )
    }

    /// 別の点との線形補間（t=0で自分、t=1で相手）
    ///
    /// # Examples
    /// ```
    /// use geo_core::Point3D;
    ///
    /// let p1 = Point3D::new(0.0, 0.0, 0.0);
    /// let p2 = Point3D::new(10.0, 10.0, 10.0);
    /// let lerp = p1.lerp(&p2, 0.5);
    /// assert_eq!(lerp.x(), 5.0);
    /// ```
    pub fn lerp(&self, other: &Self, t: T) -> Self {
        Self::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
            self.z + (other.z - self.z) * t,
        )
    }

    // ========================================================================
    // Extended Methods (Phase 2)
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

    /// 点が境界上（許容誤差内）にあるかを判定
    pub fn on_boundary(&self, point: &Self, tolerance: T) -> bool {
        self.distance_to(point) <= tolerance
    }

    /// 点が自分自身と一致するかを判定
    pub fn contains_point(&self, point: &Self) -> bool {
        self == point
    }

    // ========================================================================
    // Analysis Integration (Analysis変換機能)
    // ========================================================================

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

    /// 位置ベクトルとして取得（原点からのベクトル）
    pub fn to_vector(&self) -> analysis::linalg::vector::Vector3<T> {
        self.to_analysis_vector3()
    }
}

// ============================================================================
// Foundation Trait Implementations
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

    fn from_spherical(r: T, theta: T, phi: T) -> Self {
        Point3D::from_spherical(r, theta, phi)
    }

    fn from_analysis_vector(v: &analysis::linalg::vector::Vector3<T>) -> Self {
        Point3D::from_analysis_vector3(*v)
    }

    fn from_point(p: &Self) -> Self {
        *p
    }
}

impl<T: Scalar> Point3DProperties<T> for Point3D<T> {
    fn x(&self) -> T {
        self.x
    }

    fn y(&self) -> T {
        self.y
    }

    fn z(&self) -> T {
        self.z
    }

    fn coords(&self) -> [T; 3] {
        Point3D::coords(self)
    }

    fn to_tuple(&self) -> (T, T, T) {
        (self.x, self.y, self.z)
    }

    fn to_analysis_vector(&self) -> analysis::linalg::vector::Vector3<T> {
        self.to_analysis_vector3()
    }
}

impl<T: Scalar> Point3DMeasure<T> for Point3D<T> {
    fn distance_to(&self, other: &Self) -> T {
        Point3D::distance_to(self, other)
    }

    fn distance_squared_to(&self, other: &Self) -> T {
        Point3D::distance_squared_to(self, other)
    }

    fn norm_squared(&self) -> T {
        Point3D::norm_squared(self)
    }

    fn distance_from_origin(&self) -> T {
        self.norm()
    }

    fn midpoint(&self, other: &Self) -> Self {
        Point3D::midpoint(self, other)
    }

    fn lerp(&self, other: &Self, t: T) -> Self {
        Point3D::lerp(self, other, t)
    }

    fn manhattan_distance_to(&self, other: &Self) -> T {
        Point3D::manhattan_distance_to(self, other)
    }

    fn chebyshev_distance_to(&self, other: &Self) -> T {
        Point3D::chebyshev_distance_to(self, other)
    }
}

impl<T: Scalar> Point3DCore<T> for Point3D<T> {}

// ============================================================================
// Operator Implementations
// ============================================================================

// Note: Vector3D は geo_primitives で定義されるため、
// 演算子オーバーロードは geo_primitives で実装します

// ============================================================================
// From trait implementations
// ============================================================================

/// タプルからの変換
impl<T: Scalar> From<(T, T, T)> for Point3D<T> {
    fn from(tuple: (T, T, T)) -> Self {
        Self::new(tuple.0, tuple.1, tuple.2)
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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let p = Point3D::new(1.0, 2.0, 3.0);
        assert_eq!(p.x(), 1.0);
        assert_eq!(p.y(), 2.0);
        assert_eq!(p.z(), 3.0);
    }

    #[test]
    fn test_origin() {
        let origin = Point3D::<f64>::origin();
        assert_eq!(origin.x(), 0.0);
        assert_eq!(origin.y(), 0.0);
        assert_eq!(origin.z(), 0.0);
    }

    #[test]
    fn test_from_tuple() {
        let p = Point3D::from_tuple((1.0, 2.0, 3.0));
        assert_eq!(p.x(), 1.0);
        assert_eq!(p.y(), 2.0);
        assert_eq!(p.z(), 3.0);
    }

    #[test]
    fn test_coords() {
        let p = Point3D::new(1.0, 2.0, 3.0);
        assert_eq!(p.coords(), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_distance_to() {
        let p1 = Point3D::new(0.0, 0.0, 0.0);
        let p2 = Point3D::new(3.0, 4.0, 0.0);
        assert_eq!(p1.distance_to(&p2), 5.0);
    }

    #[test]
    fn test_distance_squared_to() {
        let p1 = Point3D::new(0.0, 0.0, 0.0);
        let p2 = Point3D::new(3.0, 4.0, 0.0);
        assert_eq!(p1.distance_squared_to(&p2), 25.0);
    }

    #[test]
    fn test_norm() {
        let p = Point3D::new(3.0, 4.0, 0.0);
        assert_eq!(p.norm(), 5.0);
    }

    #[test]
    fn test_norm_squared() {
        let p = Point3D::new(3.0, 4.0, 0.0);
        assert_eq!(p.norm_squared(), 25.0);
    }

    #[test]
    fn test_midpoint() {
        let p1 = Point3D::new(0.0, 0.0, 0.0);
        let p2 = Point3D::new(2.0, 4.0, 6.0);
        let mid = p1.midpoint(&p2);
        assert_eq!(mid.x(), 1.0);
        assert_eq!(mid.y(), 2.0);
        assert_eq!(mid.z(), 3.0);
    }

    #[test]
    fn test_lerp() {
        let p1 = Point3D::new(0.0, 0.0, 0.0);
        let p2 = Point3D::new(10.0, 10.0, 10.0);
        
        let lerp0 = p1.lerp(&p2, 0.0);
        assert_eq!(lerp0.x(), 0.0);
        
        let lerp_half = p1.lerp(&p2, 0.5);
        assert_eq!(lerp_half.x(), 5.0);
        assert_eq!(lerp_half.y(), 5.0);
        assert_eq!(lerp_half.z(), 5.0);
        
        let lerp1 = p1.lerp(&p2, 1.0);
        assert_eq!(lerp1.x(), 10.0);
    }

    #[test]
    fn test_from_trait() {
        let p: Point3D<f64> = (1.0, 2.0, 3.0).into();
        assert_eq!(p.x(), 1.0);
        assert_eq!(p.y(), 2.0);
        assert_eq!(p.z(), 3.0);
    }

    #[test]
    fn test_equality() {
        let p1 = Point3D::new(1.0, 2.0, 3.0);
        let p2 = Point3D::new(1.0, 2.0, 3.0);
        let p3 = Point3D::new(1.0, 2.0, 4.0);
        
        assert_eq!(p1, p2);
        assert_ne!(p1, p3);
    }

    #[test]
    fn test_clone() {
        let p1 = Point3D::new(1.0, 2.0, 3.0);
        let p2 = p1.clone();
        assert_eq!(p1, p2);
    }
}
