//! Point3D - 3次元点の完全実装
//!
//! geo_core における Point3D の完全実装。
//! 基本機能、Foundation トレイト、Analysis 変換、演算子オーバーロードを含む。

#[cfg(test)]
mod tests;

use analysis::abstract_types::Scalar;

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

impl<T: Scalar> Point3D<T> {
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

    /// 他の点との距離を計算
    ///
    /// # Examples
    /// ```
    /// use geo_core::Point3D;
    ///
    /// let p1 = Point3D::new(0.0, 0.0, 0.0);
    /// let p2 = Point3D::new(3.0, 4.0, 0.0);
    /// assert_eq!(p1.distance_to(p2), 5.0);
    /// ```
    pub fn distance_to(&self, other: Self) -> T {
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
    /// assert_eq!(p1.distance_squared_to(p2), 25.0);
    /// ```
    #[inline]
    pub fn distance_squared_to(&self, other: Self) -> T {
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
    /// let mid = p1.midpoint(p2);
    /// assert_eq!(mid.x(), 1.0);
    /// assert_eq!(mid.y(), 2.0);
    /// assert_eq!(mid.z(), 3.0);
    /// ```
    pub fn midpoint(&self, other: Self) -> Self {
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
    /// let lerp = p1.lerp(p2, 0.5);
    /// assert_eq!(lerp.x(), 5.0);
    /// ```
    pub fn lerp(&self, other: Self, t: T) -> Self {
        Self::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
            self.z + (other.z - self.z) * t,
        )
    }

    /// 点が自分自身と一致するかを判定
    pub fn contains_point(&self, point: Self) -> bool {
        *self == point
    }
}

// Note: Vector3D は geo_primitives で定義されるため、
// 演算子オーバーロードは geo_primitives で実装します

/// タプルからの変換
impl<T: Scalar> From<(T, T, T)> for Point3D<T> {
    fn from(tuple: (T, T, T)) -> Self {
        Self::new(tuple.0, tuple.1, tuple.2)
    }
}

impl<T: Scalar> From<Point3D<T>> for analysis::linalg::vector::Vector3<T> {
    fn from(point: Point3D<T>) -> Self {
        analysis::linalg::vector::Vector3::new(point.x(), point.y(), point.z())
    }
}

impl<T: Scalar> From<analysis::linalg::vector::Vector3<T>> for Point3D<T> {
    fn from(vector: analysis::linalg::vector::Vector3<T>) -> Self {
        Point3D::new(vector.x(), vector.y(), vector.z())
    }
}
