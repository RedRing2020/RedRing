//! Point3D - 3次元点の基本実装
//!
//! geo_core における低レイヤー基本型として、Point3D の基本機能のみを提供します。
//! Foundation パターンの拡張機能は geo_primitives で実装されます。

use analysis::abstract_types::Scalar;

/// 3次元空間の点
///
/// ## 設計方針
/// - geo_core の基本型として最小限の機能のみ実装
/// - Foundation トレイトは geo_primitives で実装
/// - analysis::Point3 との変換機能は geo_primitives で提供
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
}

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
