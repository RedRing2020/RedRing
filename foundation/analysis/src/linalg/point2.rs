//! 2次元点の数値計算実装
//!
//! 数学的な2次元点（位置）を表す型
//! Vector2との相互変換とトレイト共通化を提供

use crate::{linalg::vector::Vector2, Scalar};

/// 2次元点
///
/// 2次元平面内の位置を表す
/// Vector2とは概念的に異なるが、数値的には同じ構造
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2<T: Scalar> {
    pub x: T,
    pub y: T,
}

impl<T: Scalar> Point2<T> {
    /// 新しい点を作成
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// 原点を作成
    pub fn origin() -> Self {
        Self::new(T::ZERO, T::ZERO)
    }

    /// X座標を取得
    pub fn x(&self) -> T {
        self.x
    }

    /// Y座標を取得
    pub fn y(&self) -> T {
        self.y
    }

    /// Vector2に変換
    pub fn to_vector(&self) -> Vector2<T> {
        Vector2::new(self.x, self.y)
    }

    /// Vector2から作成
    pub fn from_vector(v: Vector2<T>) -> Self {
        Self::new(v.x(), v.y())
    }

    /// 別の点への距離
    pub fn distance_to(&self, other: &Point2<T>) -> T {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// 別の点への距離の二乗
    pub fn distance_squared_to(&self, other: &Point2<T>) -> T {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    /// 点間の中点
    pub fn midpoint(&self, other: &Point2<T>) -> Point2<T> {
        let two = T::ONE + T::ONE;
        Point2::new((self.x + other.x) / two, (self.y + other.y) / two)
    }

    /// 線形補間
    pub fn lerp(&self, other: &Point2<T>, t: T) -> Point2<T> {
        let one_minus_t = T::ONE - t;
        Point2::new(
            self.x * one_minus_t + other.x * t,
            self.y * one_minus_t + other.y * t,
        )
    }

    /// 3次元に拡張（z = 0）
    pub fn to_3d(&self) -> crate::linalg::point3::Point3<T> {
        crate::linalg::point3::Point3::new(self.x, self.y, T::ZERO)
    }

    /// 3次元に拡張（z座標指定）
    pub fn to_3d_with_z(&self, z: T) -> crate::linalg::point3::Point3<T> {
        crate::linalg::point3::Point3::new(self.x, self.y, z)
    }
}

/// Point2からVector2への変換
impl<T: Scalar> From<Point2<T>> for Vector2<T> {
    fn from(p: Point2<T>) -> Self {
        p.to_vector()
    }
}

/// Vector2からPoint2への変換
impl<T: Scalar> From<Vector2<T>> for Point2<T> {
    fn from(v: Vector2<T>) -> Self {
        Point2::from_vector(v)
    }
}

/// Point2とPoint2の減算（点-点=ベクトル）
impl<T: Scalar> std::ops::Sub for Point2<T> {
    type Output = Vector2<T>;

    fn sub(self, rhs: Point2<T>) -> Self::Output {
        Vector2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

/// Point2とVector2の加算（点+ベクトル=点）
impl<T: Scalar> std::ops::Add<Vector2<T>> for Point2<T> {
    type Output = Point2<T>;

    fn add(self, rhs: Vector2<T>) -> Self::Output {
        Point2::new(self.x + rhs.x(), self.y + rhs.y())
    }
}

/// Point2とVector2の減算（点-ベクトル=点）
impl<T: Scalar> std::ops::Sub<Vector2<T>> for Point2<T> {
    type Output = Point2<T>;

    fn sub(self, rhs: Vector2<T>) -> Self::Output {
        Point2::new(self.x - rhs.x(), self.y - rhs.y())
    }
}

/// 2次元座標アクセスの共通トレイト
///
/// Point2とVector2で座標アクセスを統一
pub trait Coordinates2D<T: Scalar> {
    fn x(&self) -> T;
    fn y(&self) -> T;
}

impl<T: Scalar> Coordinates2D<T> for Point2<T> {
    fn x(&self) -> T {
        self.x
    }
    fn y(&self) -> T {
        self.y
    }
}

impl<T: Scalar> Coordinates2D<T> for Vector2<T> {
    fn x(&self) -> T {
        Vector2::x(self)
    }
    fn y(&self) -> T {
        Vector2::y(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point2_creation() {
        let p = Point2::new(1.0, 2.0);
        assert_eq!(p.x(), 1.0);
        assert_eq!(p.y(), 2.0);
    }

    #[test]
    fn test_origin() {
        let origin = Point2::<f64>::origin();
        assert_eq!(origin.x(), 0.0);
        assert_eq!(origin.y(), 0.0);
    }

    #[test]
    fn test_vector_conversion() {
        let p = Point2::new(1.0, 2.0);
        let v = p.to_vector();
        let p2 = Point2::from_vector(v);

        assert_eq!(p, p2);
    }

    #[test]
    fn test_distance() {
        let p1 = Point2::new(0.0, 0.0);
        let p2 = Point2::new(3.0, 4.0);

        assert_eq!(p1.distance_to(&p2), 5.0);
        assert_eq!(p1.distance_squared_to(&p2), 25.0);
    }

    #[test]
    fn test_midpoint() {
        let p1 = Point2::new(0.0, 0.0);
        let p2 = Point2::new(4.0, 6.0);
        let mid = p1.midpoint(&p2);

        assert_eq!(mid, Point2::new(2.0, 3.0));
    }

    #[test]
    fn test_point_vector_operations() {
        let p = Point2::new(1.0, 2.0);
        let v = Vector2::new(1.0, 1.0);

        // 点 + ベクトル = 点
        let p_plus_v = p + v;
        assert_eq!(p_plus_v, Point2::new(2.0, 3.0));

        // 点 - ベクトル = 点
        let p_minus_v = p - v;
        assert_eq!(p_minus_v, Point2::new(0.0, 1.0));

        // 点 - 点 = ベクトル
        let p1 = Point2::new(3.0, 4.0);
        let p2 = Point2::new(1.0, 2.0);
        let diff = p1 - p2;
        assert_eq!(diff, Vector2::new(2.0, 2.0));
    }

    #[test]
    fn test_to_3d() {
        let p2 = Point2::new(1.0, 2.0);
        let p3_default = p2.to_3d();
        let p3_with_z = p2.to_3d_with_z(5.0);

        // to_3d()のテスト（z=0）
        assert_eq!(p3_default.x(), 1.0);
        assert_eq!(p3_default.y(), 2.0);
        assert_eq!(p3_default.z(), 0.0);

        // to_3d_with_z()のテスト
        assert_eq!(p3_with_z.x(), 1.0);
        assert_eq!(p3_with_z.y(), 2.0);
        assert_eq!(p3_with_z.z(), 5.0);
    }

    #[test]
    fn test_coordinates_trait() {
        let p = Point2::new(1.0, 2.0);
        let v = Vector2::new(4.0, 5.0);

        // トレイト経由でのアクセス
        assert_eq!(Coordinates2D::x(&p), 1.0);
        assert_eq!(Coordinates2D::y(&v), 5.0);
    }
}
