//! Vector - Scalar traitベースのベクトル実装
//!
//! f32/f64両対応の汎用2D/3Dベクトル実装
//! 既存のvector_extで定義されていた拡張機能もすべて統合済み

use crate::abstract_types::Scalar;
use std::fmt::{Debug, Display};
use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign};

/// 2次元ベクトル
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Vector2D<T: Scalar> {
    x: T,
    y: T,
}

impl<T: Scalar> Vector2D<T> {
    /// 新しい2Dベクトルを作成
    ///
    /// # Examples
    /// ```
    /// use geo_foundation::Vector2D;
    /// let v = Vector2D::new(3.0, 4.0);
    /// assert_eq!(v.x(), 3.0);
    /// assert_eq!(v.y(), 4.0);
    /// ```
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// X成分にアクセス
    pub fn x(&self) -> T {
        self.x
    }

    /// Y成分にアクセス
    pub fn y(&self) -> T {
        self.y
    }

    /// X成分を設定
    pub fn set_x(&mut self, x: T) {
        self.x = x;
    }

    /// Y成分を設定
    pub fn set_y(&mut self, y: T) {
        self.y = y;
    }

    /// ゼロベクトル
    pub fn zero() -> Self {
        Self::new(T::ZERO, T::ZERO)
    }

    /// 単位X軸ベクトル
    pub fn unit_x() -> Self {
        Self::new(T::ONE, T::ZERO)
    }

    /// 単位Y軸ベクトル
    pub fn unit_y() -> Self {
        Self::new(T::ZERO, T::ONE)
    }

    /// 内積
    pub fn dot(&self, other: &Self) -> T {
        self.x * other.x + self.y * other.y
    }

    /// 外積（2Dでは擬似スカラー）
    pub fn cross(&self, other: &Self) -> T {
        self.x * other.y - self.y * other.x
    }

    /// ベクトルの長さの二乗
    pub fn length_squared(&self) -> T {
        self.dot(self)
    }

    /// ベクトルの長さ
    pub fn length(&self) -> T {
        self.length_squared().sqrt()
    }

    /// 正規化（単位ベクトル化）
    pub fn normalize(&self) -> Option<Self> {
        let len = self.length();
        if len == T::ZERO {
            None
        } else {
            Some(Self::new(self.x / len, self.y / len))
        }
    }

    /// 正規化（ゼロベクトルの場合はゼロベクトルを返す）
    pub fn normalized(&self) -> Self {
        self.normalize().unwrap_or_else(Self::zero)
    }

    /// ゼロベクトルかどうかを判定
    pub fn is_zero(&self) -> bool {
        self.x == T::ZERO && self.y == T::ZERO
    }

    /// 近似的にゼロベクトルかどうかを判定
    pub fn is_zero_approx(&self, tolerance: T) -> bool {
        self.length() <= tolerance
    }

    /// ベクトルから原点までの距離（長さ）
    pub fn distance_to_origin(&self) -> T {
        self.length()
    }

    /// 他のベクトルとの距離
    pub fn distance_to(&self, other: &Self) -> T {
        (*self - *other).length()
    }

    /// 垂直ベクトル（時計回りに90度回転）
    pub fn perpendicular(&self) -> Self {
        Self::new(self.y, -self.x)
    }

    /// 反射ベクトル（法線に対する反射）
    pub fn reflect(&self, normal: &Self) -> Self {
        let normal = normal.normalized();
        *self - normal * (self.dot(&normal) * (T::ONE + T::ONE))
    }

    /// 角度（ラジアン）からベクトルを作成
    pub fn from_angle(angle: T) -> Self {
        Self::new(angle.cos(), angle.sin())
    }

    /// ベクトルの角度（ラジアン）を取得
    pub fn angle(&self) -> T {
        self.y.atan2(self.x)
    }

    /// 他のベクトルとの角度差
    pub fn angle_to(&self, other: &Self) -> T {
        let dot_product = self.dot(other);
        let cross_product = self.cross(other);
        cross_product.atan2(dot_product)
    }

    /// スカラー倍
    pub fn scale(&self, scalar: T) -> Self {
        Self::new(self.x * scalar, self.y * scalar)
    }

    /// 線形補間
    pub fn lerp(&self, other: &Self, t: T) -> Self {
        *self + (*other - *self) * t
    }

    /// 成分ごとの最小値
    pub fn min(&self, other: &Self) -> Self {
        Self::new(
            if self.x < other.x { self.x } else { other.x },
            if self.y < other.y { self.y } else { other.y },
        )
    }

    /// 成分ごとの最大値
    pub fn max(&self, other: &Self) -> Self {
        Self::new(
            if self.x > other.x { self.x } else { other.x },
            if self.y > other.y { self.y } else { other.y },
        )
    }

    /// f32からf64への変換
    pub fn to_f64(self) -> Vector2D<f64>
    where
        T: Into<f64>,
    {
        Vector2D::new(self.x.into(), self.y.into())
    }

    /// f64からf32への変換
    pub fn to_f32(self) -> Vector2D<f32>
    where
        T: Into<f32>,
    {
        Vector2D::new(self.x.into(), self.y.into())
    }
}

/// 3次元ベクトル
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Vector3D<T: Scalar> {
    x: T,
    y: T,
    z: T,
}

impl<T: Scalar> Vector3D<T> {
    /// 新しい3Dベクトルを作成
    ///
    /// # Examples
    /// ```
    /// use geo_foundation::Vector3D;
    /// let v = Vector3D::new(1.0, 2.0, 3.0);
    /// assert_eq!(v.x(), 1.0);
    /// assert_eq!(v.y(), 2.0);
    /// assert_eq!(v.z(), 3.0);
    /// ```
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }

    /// X成分にアクセス
    pub fn x(&self) -> T {
        self.x
    }

    /// Y成分にアクセス
    pub fn y(&self) -> T {
        self.y
    }

    /// Z成分にアクセス
    pub fn z(&self) -> T {
        self.z
    }

    /// X成分を設定
    pub fn set_x(&mut self, x: T) {
        self.x = x;
    }

    /// Y成分を設定
    pub fn set_y(&mut self, y: T) {
        self.y = y;
    }

    /// Z成分を設定
    pub fn set_z(&mut self, z: T) {
        self.z = z;
    }

    /// ゼロベクトル
    pub fn zero() -> Self {
        Self::new(T::ZERO, T::ZERO, T::ZERO)
    }

    /// 単位X軸ベクトル
    pub fn unit_x() -> Self {
        Self::new(T::ONE, T::ZERO, T::ZERO)
    }

    /// 単位Y軸ベクトル
    pub fn unit_y() -> Self {
        Self::new(T::ZERO, T::ONE, T::ZERO)
    }

    /// 単位Z軸ベクトル
    pub fn unit_z() -> Self {
        Self::new(T::ZERO, T::ZERO, T::ONE)
    }

    /// 内積
    pub fn dot(&self, other: &Self) -> T {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// 外積
    pub fn cross(&self, other: &Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    /// ベクトルの長さの二乗
    pub fn length_squared(&self) -> T {
        self.dot(self)
    }

    /// ベクトルの長さ
    pub fn length(&self) -> T {
        self.length_squared().sqrt()
    }

    /// 正規化（単位ベクトル化）
    pub fn normalize(&self) -> Option<Self> {
        let len = self.length();
        if len == T::ZERO {
            None
        } else {
            Some(Self::new(self.x / len, self.y / len, self.z / len))
        }
    }

    /// 正規化（ゼロベクトルの場合はゼロベクトルを返す）
    pub fn normalized(&self) -> Self {
        self.normalize().unwrap_or_else(Self::zero)
    }

    /// ゼロベクトルかどうかを判定
    pub fn is_zero(&self) -> bool {
        self.x == T::ZERO && self.y == T::ZERO && self.z == T::ZERO
    }

    /// 近似的にゼロベクトルかどうかを判定
    pub fn is_zero_approx(&self, tolerance: T) -> bool {
        self.length() <= tolerance
    }

    /// ベクトルから原点までの距離（長さ）
    pub fn distance_to_origin(&self) -> T {
        self.length()
    }

    /// 他のベクトルとの距離
    pub fn distance_to(&self, other: &Self) -> T {
        (*self - *other).length()
    }

    /// 反射ベクトル（法線に対する反射）
    pub fn reflect(&self, normal: &Self) -> Self {
        let normal = normal.normalized();
        *self - normal * (self.dot(&normal) * (T::ONE + T::ONE))
    }

    /// 指定した軸周りの回転（ロドリゲスの回転公式）
    pub fn rotate_around_axis(&self, axis: &Self, angle: T) -> Self {
        let axis = axis.normalized();
        let cos_theta = angle.cos();
        let sin_theta = angle.sin();
        let one_minus_cos = T::ONE - cos_theta;

        // ロドリゲスの回転公式
        *self * cos_theta
            + axis.cross(self) * sin_theta
            + axis * (axis.dot(self) * one_minus_cos)
    }

    /// 任意の軸に対する垂直ベクトルを生成
    pub fn any_perpendicular(&self) -> Self {
        let abs_x = self.x.abs();
        let abs_y = self.y.abs();
        let abs_z = self.z.abs();

        // 最も小さい成分に対応する軸を選択
        if abs_x <= abs_y && abs_x <= abs_z {
            Self::unit_x().cross(self).normalized()
        } else if abs_y <= abs_z {
            Self::unit_y().cross(self).normalized()
        } else {
            Self::unit_z().cross(self).normalized()
        }
    }

    /// 正規直交基底を構築（このベクトルをZ軸とする）
    pub fn build_orthonormal_basis(&self) -> (Self, Self, Self) {
        let z = self.normalized();
        let x = z.any_perpendicular();
        let y = z.cross(&x);
        (x, y, z)
    }

    /// スカラー倍
    pub fn scale(&self, scalar: T) -> Self {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }

    /// 線形補間
    pub fn lerp(&self, other: &Self, t: T) -> Self {
        *self + (*other - *self) * t
    }

    /// 成分ごとの最小値
    pub fn min(&self, other: &Self) -> Self {
        Self::new(
            if self.x < other.x { self.x } else { other.x },
            if self.y < other.y { self.y } else { other.y },
            if self.z < other.z { self.z } else { other.z },
        )
    }

    /// 成分ごとの最大値
    pub fn max(&self, other: &Self) -> Self {
        Self::new(
            if self.x > other.x { self.x } else { other.x },
            if self.y > other.y { self.y } else { other.y },
            if self.z > other.z { self.z } else { other.z },
        )
    }

    /// 2Dベクトルに変換（Z成分を破棄）
    pub fn to_2d(&self) -> Vector2D<T> {
        Vector2D::new(self.x, self.y)
    }

    /// f32からf64への変換
    pub fn to_f64(self) -> Vector3D<f64>
    where
        T: Into<f64>,
    {
        Vector3D::new(self.x.into(), self.y.into(), self.z.into())
    }

    /// f64からf32への変換
    pub fn to_f32(self) -> Vector3D<f32>
    where
        T: Into<f32>,
    {
        Vector3D::new(self.x.into(), self.y.into(), self.z.into())
    }
}

// Vector2D用の演算子実装
impl<T: Scalar> Add for Vector2D<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl<T: Scalar> AddAssign for Vector2D<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.x = self.x + rhs.x;
        self.y = self.y + rhs.y;
    }
}

impl<T: Scalar> Sub for Vector2D<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl<T: Scalar> SubAssign for Vector2D<T> {
    fn sub_assign(&mut self, rhs: Self) {
        self.x = self.x - rhs.x;
        self.y = self.y - rhs.y;
    }
}

impl<T: Scalar> Mul<T> for Vector2D<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl<T: Scalar> MulAssign<T> for Vector2D<T> {
    fn mul_assign(&mut self, rhs: T) {
        self.x = self.x * rhs;
        self.y = self.y * rhs;
    }
}

impl<T: Scalar> Div<T> for Vector2D<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl<T: Scalar> DivAssign<T> for Vector2D<T> {
    fn div_assign(&mut self, rhs: T) {
        self.x = self.x / rhs;
        self.y = self.y / rhs;
    }
}

impl<T: Scalar> Neg for Vector2D<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}

impl<T: Scalar> Index<usize> for Vector2D<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("Vector2D index out of range: {}", index),
        }
    }
}

impl<T: Scalar> IndexMut<usize> for Vector2D<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => panic!("Vector2D index out of range: {}", index),
        }
    }
}

// Vector3D用の演算子実装
impl<T: Scalar> Add for Vector3D<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl<T: Scalar> AddAssign for Vector3D<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.x = self.x + rhs.x;
        self.y = self.y + rhs.y;
        self.z = self.z + rhs.z;
    }
}

impl<T: Scalar> Sub for Vector3D<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl<T: Scalar> SubAssign for Vector3D<T> {
    fn sub_assign(&mut self, rhs: Self) {
        self.x = self.x - rhs.x;
        self.y = self.y - rhs.y;
        self.z = self.z - rhs.z;
    }
}

impl<T: Scalar> Mul<T> for Vector3D<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl<T: Scalar> MulAssign<T> for Vector3D<T> {
    fn mul_assign(&mut self, rhs: T) {
        self.x = self.x * rhs;
        self.y = self.y * rhs;
        self.z = self.z * rhs;
    }
}

impl<T: Scalar> Div<T> for Vector3D<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl<T: Scalar> DivAssign<T> for Vector3D<T> {
    fn div_assign(&mut self, rhs: T) {
        self.x = self.x / rhs;
        self.y = self.y / rhs;
        self.z = self.z / rhs;
    }
}

impl<T: Scalar> Neg for Vector3D<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl<T: Scalar> Index<usize> for Vector3D<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Vector3D index out of range: {}", index),
        }
    }
}

impl<T: Scalar> IndexMut<usize> for Vector3D<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Vector3D index out of range: {}", index),
        }
    }
}

// Display実装
impl<T: Scalar + Display> Display for Vector2D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vector2D({}, {})", self.x, self.y)
    }
}

impl<T: Scalar + Display> Display for Vector3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vector3D({}, {}, {})", self.x, self.y, self.z)
    }
}

// PointからVectorへの変換
impl<T: Scalar> From<crate::geometry::Point2D<T>> for Vector2D<T> {
    fn from(point: crate::geometry::Point2D<T>) -> Self {
        Self::new(point.x(), point.y())
    }
}

impl<T: Scalar> From<crate::geometry::Point3D<T>> for Vector3D<T> {
    fn from(point: crate::geometry::Point3D<T>) -> Self {
        Self::new(point.x(), point.y(), point.z())
    }
}

// VectorからPointへの変換
impl<T: Scalar> From<Vector2D<T>> for crate::geometry::Point2D<T> {
    fn from(vector: Vector2D<T>) -> Self {
        Self::new(vector.x, vector.y)
    }
}

impl<T: Scalar> From<Vector3D<T>> for crate::geometry::Point3D<T> {
    fn from(vector: Vector3D<T>) -> Self {
        Self::new(vector.x, vector.y, vector.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector2d_creation() {
        let v = Vector2D::new(3.0, 4.0);
        assert_eq!(v.x(), 3.0);
        assert_eq!(v.y(), 4.0);
    }

    #[test]
    fn test_vector2d_operations() {
        let v1 = Vector2D::new(1.0, 2.0);
        let v2 = Vector2D::new(3.0, 4.0);
        
        let sum = v1 + v2;
        assert_eq!(sum, Vector2D::new(4.0, 6.0));
        
        let diff = v2 - v1;
        assert_eq!(diff, Vector2D::new(2.0, 2.0));
        
        let scaled = v1 * 2.0;
        assert_eq!(scaled, Vector2D::new(2.0, 4.0));
    }

    #[test]
    fn test_vector2d_dot_product() {
        let v1 = Vector2D::new(1.0, 0.0);
        let v2 = Vector2D::new(0.0, 1.0);
        assert_eq!(v1.dot(&v2), 0.0);
        
        let v3 = Vector2D::new(3.0, 4.0);
        let v4 = Vector2D::new(1.0, 0.0);
        assert_eq!(v3.dot(&v4), 3.0);
    }

    #[test]
    fn test_vector2d_cross_product() {
        let v1 = Vector2D::new(1.0, 0.0);
        let v2 = Vector2D::new(0.0, 1.0);
        assert_eq!(v1.cross(&v2), 1.0);
    }

    #[test]
    fn test_vector2d_length() {
        let v = Vector2D::new(3.0, 4.0);
        assert_eq!(v.length(), 5.0);
        assert_eq!(v.length_squared(), 25.0);
    }

    #[test]
    fn test_vector2d_normalization() {
        let v = Vector2D::new(3.0, 4.0);
        let normalized = v.normalize().unwrap();
        assert!((normalized.length() - 1.0).abs() < 1e-10);
        assert_eq!(normalized, Vector2D::new(0.6, 0.8));
    }

    #[test]
    fn test_vector2d_zero_normalization() {
        let zero = Vector2D::<f64>::zero();
        assert!(zero.normalize().is_none());
        assert_eq!(zero.normalized(), Vector2D::<f64>::zero());
    }

    #[test]
    fn test_vector3d_creation() {
        let v = Vector3D::new(1.0, 2.0, 3.0);
        assert_eq!(v.x(), 1.0);
        assert_eq!(v.y(), 2.0);
        assert_eq!(v.z(), 3.0);
    }

    #[test]
    fn test_vector3d_cross_product() {
        let v1 = Vector3D::new(1.0, 0.0, 0.0);
        let v2 = Vector3D::new(0.0, 1.0, 0.0);
        let cross = v1.cross(&v2);
        assert_eq!(cross, Vector3D::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_vector3d_normalization() {
        let v = Vector3D::new(1.0, 2.0, 2.0);
        let normalized = v.normalize().unwrap();
        assert!((normalized.length() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_f32_f64_compatibility() {
        let v_f32 = Vector2D::<f32>::new(1.0, 2.0);
        let v_f64 = v_f32.to_f64();
        assert_eq!(v_f64.x(), 1.0f64);
        assert_eq!(v_f64.y(), 2.0f64);
    }

    #[test]
    fn test_vector_constants() {
        let zero2d = Vector2D::<f64>::zero();
        assert!(zero2d.is_zero());
        
        let unit_x = Vector2D::<f64>::unit_x();
        assert_eq!(unit_x.length(), 1.0);
        
        let unit_y = Vector2D::<f64>::unit_y();
        assert_eq!(unit_y.length(), 1.0);
    }

    #[test]
    fn test_angle_operations() {
        use std::f64::consts::PI;
        
        let v = Vector2D::from_angle(PI / 4.0); // 45度
        assert!((v.x() - v.y()).abs() < 1e-10); // x ≈ y
        assert!((v.length() - 1.0).abs() < 1e-10); // 単位ベクトル
        
        let angle = v.angle();
        assert!((angle - PI / 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_point_vector_conversion() {
        use crate::geometry::Point2D;
        
        let point = Point2D::new(3.0, 4.0);
        let vector: Vector2D<f64> = point.into();
        assert_eq!(vector.x(), 3.0);
        assert_eq!(vector.y(), 4.0);
        
        let back_to_point: Point2D<f64> = vector.into();
        assert_eq!(back_to_point.x(), 3.0);
        assert_eq!(back_to_point.y(), 4.0);
    }
}