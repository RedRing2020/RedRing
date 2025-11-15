//! 3次元点の数値計算実装
//!
//! 数学的な3次元点（位置）を表す型
//! Vector3との相互変換とトレイト共通化を提供

use crate::{linalg::vector::Vector3, Scalar};

/// 3次元点
///
/// 3次元空間内の位置を表す
/// Vector3とは概念的に異なるが、数値的には同じ構造
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3<T: Scalar> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: Scalar> Point3<T> {
    /// 新しい点を作成
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }

    /// 原点を作成
    pub fn origin() -> Self {
        Self::new(T::ZERO, T::ZERO, T::ZERO)
    }

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

    /// Vector3に変換
    pub fn to_vector(&self) -> Vector3<T> {
        Vector3::new(self.x, self.y, self.z)
    }

    /// Vector3から作成
    pub fn from_vector(v: Vector3<T>) -> Self {
        Self::new(v.x(), v.y(), v.z())
    }

    /// 別の点への距離
    pub fn distance_to(&self, other: &Point3<T>) -> T {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// 別の点への距離の二乗
    pub fn distance_squared_to(&self, other: &Point3<T>) -> T {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }

    /// 点間の中点
    pub fn midpoint(&self, other: &Point3<T>) -> Point3<T> {
        let two = T::ONE + T::ONE;
        Point3::new(
            (self.x + other.x) / two,
            (self.y + other.y) / two,
            (self.z + other.z) / two,
        )
    }

    /// 線形補間
    pub fn lerp(&self, other: &Point3<T>, t: T) -> Point3<T> {
        let one_minus_t = T::ONE - t;
        Point3::new(
            self.x * one_minus_t + other.x * t,
            self.y * one_minus_t + other.y * t,
            self.z * one_minus_t + other.z * t,
        )
    }

    /// 2次元に射影（z座標を無視）
    pub fn to_2d(&self) -> crate::linalg::point2::Point2<T> {
        crate::linalg::point2::Point2::new(self.x, self.y)
    }
}

/// Point3からVector3への変換
impl<T: Scalar> From<Point3<T>> for Vector3<T> {
    fn from(p: Point3<T>) -> Self {
        p.to_vector()
    }
}

/// Vector3からPoint3への変換
impl<T: Scalar> From<Vector3<T>> for Point3<T> {
    fn from(v: Vector3<T>) -> Self {
        Point3::from_vector(v)
    }
}

/// Point3とVector3の減算（点-点=ベクトル）
impl<T: Scalar> std::ops::Sub for Point3<T> {
    type Output = Vector3<T>;

    fn sub(self, rhs: Point3<T>) -> Self::Output {
        Vector3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

/// Point3とVector3の加算（点+ベクトル=点）
impl<T: Scalar> std::ops::Add<Vector3<T>> for Point3<T> {
    type Output = Point3<T>;

    fn add(self, rhs: Vector3<T>) -> Self::Output {
        Point3::new(self.x + rhs.x(), self.y + rhs.y(), self.z + rhs.z())
    }
}

/// Point3とVector3の減算（点-ベクトル=点）
impl<T: Scalar> std::ops::Sub<Vector3<T>> for Point3<T> {
    type Output = Point3<T>;

    fn sub(self, rhs: Vector3<T>) -> Self::Output {
        Point3::new(self.x - rhs.x(), self.y - rhs.y(), self.z - rhs.z())
    }
}

/// 3次元座標アクセスの共通トレイト
///
/// Point3とVector3で座標アクセスを統一
pub trait Coordinates3D<T: Scalar> {
    fn x(&self) -> T;
    fn y(&self) -> T;
    fn z(&self) -> T;
}

impl<T: Scalar> Coordinates3D<T> for Point3<T> {
    fn x(&self) -> T {
        self.x
    }
    fn y(&self) -> T {
        self.y
    }
    fn z(&self) -> T {
        self.z
    }
}

impl<T: Scalar> Coordinates3D<T> for Vector3<T> {
    fn x(&self) -> T {
        Vector3::x(self)
    }
    fn y(&self) -> T {
        Vector3::y(self)
    }
    fn z(&self) -> T {
        Vector3::z(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point3_creation() {
        let p = Point3::new(1.0, 2.0, 3.0);
        assert_eq!(p.x(), 1.0);
        assert_eq!(p.y(), 2.0);
        assert_eq!(p.z(), 3.0);
    }

    #[test]
    fn test_origin() {
        let origin = Point3::<f64>::origin();
        assert_eq!(origin.x(), 0.0);
        assert_eq!(origin.y(), 0.0);
        assert_eq!(origin.z(), 0.0);
    }

    #[test]
    fn test_vector_conversion() {
        let p = Point3::new(1.0, 2.0, 3.0);
        let v = p.to_vector();
        let p2 = Point3::from_vector(v);

        assert_eq!(p, p2);
    }

    #[test]
    fn test_distance() {
        let p1 = Point3::new(0.0, 0.0, 0.0);
        let p2 = Point3::new(3.0, 4.0, 0.0);

        assert_eq!(p1.distance_to(&p2), 5.0);
        assert_eq!(p1.distance_squared_to(&p2), 25.0);
    }

    #[test]
    fn test_midpoint() {
        let p1 = Point3::new(0.0, 0.0, 0.0);
        let p2 = Point3::new(4.0, 6.0, 8.0);
        let mid = p1.midpoint(&p2);

        assert_eq!(mid, Point3::new(2.0, 3.0, 4.0));
    }

    #[test]
    fn test_point_vector_operations() {
        let p = Point3::new(1.0, 2.0, 3.0);
        let v = Vector3::new(1.0, 1.0, 1.0);

        // 点 + ベクトル = 点
        let p_plus_v = p + v;
        assert_eq!(p_plus_v, Point3::new(2.0, 3.0, 4.0));

        // 点 - ベクトル = 点
        let p_minus_v = p - v;
        assert_eq!(p_minus_v, Point3::new(0.0, 1.0, 2.0));

        // 点 - 点 = ベクトル
        let p1 = Point3::new(3.0, 4.0, 5.0);
        let p2 = Point3::new(1.0, 2.0, 3.0);
        let diff = p1 - p2;
        assert_eq!(diff, Vector3::new(2.0, 2.0, 2.0));
    }

    #[test]
    fn test_coordinates_trait() {
        let p = Point3::new(1.0, 2.0, 3.0);
        let v = Vector3::new(4.0, 5.0, 6.0);

        // トレイト経由でのアクセス
        assert_eq!(Coordinates3D::x(&p), 1.0);
        assert_eq!(Coordinates3D::y(&v), 5.0);
    }

    #[test]
    fn test_to_2d() {
        let p3 = Point3::new(1.0, 2.0, 5.0);
        let p2 = p3.to_2d();

        assert_eq!(p2.x(), 1.0);
        assert_eq!(p2.y(), 2.0);
        // z座標は射影で失われる
    }
}
