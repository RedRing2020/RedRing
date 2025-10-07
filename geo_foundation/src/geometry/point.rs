/// ジェネリックPoint実装
///
/// f32/f64両対応の2D/3D点管理。
/// Scalarトレイトによる型安全性と統一的なAPI提供。
///
/// # 設計方針
///
/// - **ジェネリック**: f32/f64両対応でゲーム・CAD用途に最適化
/// - **型安全**: Scalarトレイトによる数値型の統一
/// - **次元別**: Point2D/Point3Dによる特化機能
/// - **相互変換**: 2D↔3D、f32↔f64の柔軟な変換
use crate::abstract_types::Scalar;
use std::fmt::{Debug, Display};
use std::ops::{Add, Div, Mul, Sub};

/// 2D点の汎用実装
///
/// x, y座標を持つ2次元点。
/// f32/f64の数値型に対応。
///
/// # 例
///
/// ```rust
/// use geo_foundation::geometry::Point2D;
///
/// // f64での高精度計算
/// let p1 = Point2D::<f64>::new(1.0, 2.0);
/// let p2 = Point2D::<f64>::new(4.0, 6.0);
/// let distance = p1.distance_to(p2);
///
/// // f32での高速計算
/// let p3 = Point2D::<f32>::new(0.0, 0.0);
/// let p4 = Point2D::<f32>::origin();
/// assert_eq!(p3, p4);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D<T: Scalar> {
    x: T,
    y: T,
}

impl<T: Scalar> Point2D<T> {
    /// 新しい2D点を作成
    ///
    /// # Arguments
    ///
    /// * `x` - X座標
    /// * `y` - Y座標
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// 原点（0, 0）を作成
    pub fn origin() -> Self {
        Self::new(T::ZERO, T::ZERO)
    }

    /// X座標を取得
    pub fn x(self) -> T {
        self.x
    }

    /// Y座標を取得
    pub fn y(self) -> T {
        self.y
    }

    /// 座標をタプルで取得
    pub fn coords(self) -> (T, T) {
        (self.x, self.y)
    }

    /// X座標を設定した新しい点を作成
    pub fn with_x(self, x: T) -> Self {
        Self::new(x, self.y)
    }

    /// Y座標を設定した新しい点を作成
    pub fn with_y(self, y: T) -> Self {
        Self::new(self.x, y)
    }

    /// 他の点との距離を計算
    ///
    /// # Arguments
    ///
    /// * `other` - 距離計算対象の点
    ///
    /// # Returns
    ///
    /// ユークリッド距離
    pub fn distance_to(self, other: Self) -> T {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// 他の点との距離の2乗を計算（平方根計算を避ける高速版）
    pub fn distance_squared_to(self, other: Self) -> T {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    /// 原点からの距離（長さ）を計算
    pub fn magnitude(self) -> T {
        self.distance_to(Self::origin())
    }

    /// 原点からの距離の2乗を計算
    pub fn magnitude_squared(self) -> T {
        self.x * self.x + self.y * self.y
    }

    /// 単位ベクトルに正規化（長さ1にする）
    ///
    /// # Returns
    ///
    /// 正規化されたベクトル。ゼロベクトルの場合はNone
    pub fn normalize(self) -> Option<Self> {
        let mag = self.magnitude();
        if mag > T::TOLERANCE {
            Some(Self::new(self.x / mag, self.y / mag))
        } else {
            None
        }
    }

    /// 内積を計算
    pub fn dot(self, other: Self) -> T {
        self.x * other.x + self.y * other.y
    }

    /// 外積（2Dでは疑似外積のスカラー値）を計算
    pub fn cross(self, other: Self) -> T {
        self.x * other.y - self.y * other.x
    }

    /// 線形補間
    ///
    /// # Arguments
    ///
    /// * `other` - 補間先の点
    /// * `t` - 補間パラメータ（0.0で自分、1.0で相手）
    pub fn lerp(self, other: Self, t: T) -> Self {
        Self::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
        )
    }

    /// 指定角度で回転した新しい点を取得
    ///
    /// # Arguments
    ///
    /// * `angle` - 回転角度（ラジアン）
    pub fn rotated(self, angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self::new(
            self.x * cos_a - self.y * sin_a,
            self.x * sin_a + self.y * cos_a,
        )
    }

    /// 2つの点の中点を計算
    pub fn midpoint(self, other: Self) -> Self {
        Self::new(
            (self.x + other.x) / (T::ONE + T::ONE),
            (self.y + other.y) / (T::ONE + T::ONE),
        )
    }

    /// 近似的に等しいかを判定
    pub fn approx_eq(self, other: Self) -> bool {
        (self.x - other.x).abs() < T::TOLERANCE && (self.y - other.y).abs() < T::TOLERANCE
    }

    /// 3D点に変換（Z座標を0に設定）
    pub fn to_3d(self) -> Point3D<T> {
        Point3D::new(self.x, self.y, T::ZERO)
    }

    /// 異なる数値型に変換
    pub fn cast<U: Scalar>(self) -> Point2D<U> {
        Point2D::<U>::new(U::from_f64(self.x.to_f64()), U::from_f64(self.y.to_f64()))
    }
}

/// 3D点の汎用実装
///
/// x, y, z座標を持つ3次元点。
/// f32/f64の数値型に対応。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3D<T: Scalar> {
    x: T,
    y: T,
    z: T,
}

impl<T: Scalar> Point3D<T> {
    /// 新しい3D点を作成
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }

    /// 原点（0, 0, 0）を作成
    pub fn origin() -> Self {
        Self::new(T::ZERO, T::ZERO, T::ZERO)
    }

    /// X座標を取得
    pub fn x(self) -> T {
        self.x
    }

    /// Y座標を取得
    pub fn y(self) -> T {
        self.y
    }

    /// Z座標を取得
    pub fn z(self) -> T {
        self.z
    }

    /// 座標をタプルで取得
    pub fn coords(self) -> (T, T, T) {
        (self.x, self.y, self.z)
    }

    /// 他の点との距離を計算
    pub fn distance_to(self, other: Self) -> T {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// 他の点との距離の2乗を計算
    pub fn distance_squared_to(self, other: Self) -> T {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }

    /// 原点からの距離（長さ）を計算
    pub fn magnitude(self) -> T {
        self.distance_to(Self::origin())
    }

    /// 原点からの距離の2乗を計算
    pub fn magnitude_squared(self) -> T {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// 単位ベクトルに正規化
    pub fn normalize(self) -> Option<Self> {
        let mag = self.magnitude();
        if mag > T::TOLERANCE {
            Some(Self::new(self.x / mag, self.y / mag, self.z / mag))
        } else {
            None
        }
    }

    /// 内積を計算
    pub fn dot(self, other: Self) -> T {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// 外積を計算
    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    /// 線形補間
    pub fn lerp(self, other: Self, t: T) -> Self {
        Self::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
            self.z + (other.z - self.z) * t,
        )
    }

    /// 2つの点の中点を計算
    pub fn midpoint(self, other: Self) -> Self {
        Self::new(
            (self.x + other.x) / (T::ONE + T::ONE),
            (self.y + other.y) / (T::ONE + T::ONE),
            (self.z + other.z) / (T::ONE + T::ONE),
        )
    }

    /// 近似的に等しいかを判定
    pub fn approx_eq(self, other: Self) -> bool {
        (self.x - other.x).abs() < T::TOLERANCE
            && (self.y - other.y).abs() < T::TOLERANCE
            && (self.z - other.z).abs() < T::TOLERANCE
    }

    /// 2D点に投影（Z座標を破棄）
    pub fn to_2d(self) -> Point2D<T> {
        Point2D::new(self.x, self.y)
    }

    /// XY平面での距離計算
    pub fn xy_distance_to(self, other: Self) -> T {
        self.to_2d().distance_to(other.to_2d())
    }

    /// 異なる数値型に変換
    pub fn cast<U: Scalar>(self) -> Point3D<U> {
        Point3D::<U>::new(
            U::from_f64(self.x.to_f64()),
            U::from_f64(self.y.to_f64()),
            U::from_f64(self.z.to_f64()),
        )
    }
}

// 四則演算の実装（Point2D）
impl<T: Scalar> Add for Point2D<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl<T: Scalar> Sub for Point2D<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl<T: Scalar> Mul<T> for Point2D<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl<T: Scalar> Div<T> for Point2D<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

// 四則演算の実装（Point3D）
impl<T: Scalar> Add for Point3D<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl<T: Scalar> Sub for Point3D<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl<T: Scalar> Mul<T> for Point3D<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl<T: Scalar> Div<T> for Point3D<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

// 表示実装
impl<T: Scalar> Display for Point2D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl<T: Scalar> Display for Point3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

// 便利な型エイリアス
pub type Point2D32 = Point2D<f32>;
pub type Point2D64 = Point2D<f64>;
pub type Point3D32 = Point3D<f32>;
pub type Point3D64 = Point3D<f64>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point2d_creation() {
        let p = Point2D::<f64>::new(3.0, 4.0);
        assert_eq!(p.x(), 3.0);
        assert_eq!(p.y(), 4.0);
        assert_eq!(p.coords(), (3.0, 4.0));
    }

    #[test]
    fn test_point2d_distance() {
        let p1 = Point2D::<f64>::new(0.0, 0.0);
        let p2 = Point2D::<f64>::new(3.0, 4.0);
        assert!((p1.distance_to(p2) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_point2d_operations() {
        let p1 = Point2D::<f32>::new(1.0, 2.0);
        let p2 = Point2D::<f32>::new(3.0, 4.0);

        let sum = p1 + p2;
        assert_eq!(sum.coords(), (4.0, 6.0));

        let diff = p2 - p1;
        assert_eq!(diff.coords(), (2.0, 2.0));

        let scaled = p1 * 2.0;
        assert_eq!(scaled.coords(), (2.0, 4.0));
    }

    #[test]
    fn test_point3d_creation() {
        let p = Point3D::<f64>::new(1.0, 2.0, 3.0);
        assert_eq!(p.coords(), (1.0, 2.0, 3.0));
    }

    #[test]
    fn test_point3d_distance() {
        let p1 = Point3D::<f64>::new(0.0, 0.0, 0.0);
        let p2 = Point3D::<f64>::new(1.0, 2.0, 2.0);
        assert!((p1.distance_to(p2) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_2d_3d_conversion() {
        let p2d = Point2D::<f64>::new(3.0, 4.0);
        let p3d = p2d.to_3d();
        assert_eq!(p3d.coords(), (3.0, 4.0, 0.0));

        let back_to_2d = p3d.to_2d();
        assert_eq!(back_to_2d.coords(), (3.0, 4.0));
    }

    #[test]
    fn test_type_casting() {
        let p_f64 = Point2D::<f64>::new(3.14159, 2.71828);
        let p_f32 = p_f64.cast::<f32>();

        assert!((p_f32.x() - 3.14159f32).abs() < 1e-5);
        assert!((p_f32.y() - 2.71828f32).abs() < 1e-5);
    }

    #[test]
    fn test_normalization() {
        let p = Point2D::<f64>::new(3.0, 4.0);
        let normalized = p.normalize().unwrap();
        assert!((normalized.magnitude() - 1.0).abs() < 1e-10);

        let zero = Point2D::<f64>::origin();
        assert!(zero.normalize().is_none());
    }

    #[test]
    fn test_cross_product_3d() {
        let i = Point3D::<f64>::new(1.0, 0.0, 0.0);
        let j = Point3D::<f64>::new(0.0, 1.0, 0.0);
        let k = i.cross(j);
        assert!(k.approx_eq(Point3D::new(0.0, 0.0, 1.0)));
    }
}
