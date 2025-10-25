//! 3次元ベクトル（Vector3D）のCore実装
//!
//! Core Foundation パターンに基づく Vector3D の必須機能のみ
//! 拡張機能は vector_3d_extensions.rs を参照

use crate::{BBox3D, Point3D};
use geo_foundation::{core::vector_traits, Scalar};

/// 3次元ベクトル
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3D<T: Scalar> {
    x: T,
    y: T,
    z: T,
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> Vector3D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================

    /// 新しいベクトルを作成
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }

    /// ゼロベクトルを取得
    pub fn zero() -> Self {
        Self::new(T::ZERO, T::ZERO, T::ZERO)
    }

    /// X軸単位ベクトルを取得
    pub fn unit_x() -> Self {
        Self::new(T::ONE, T::ZERO, T::ZERO)
    }

    /// Y軸単位ベクトルを取得
    pub fn unit_y() -> Self {
        Self::new(T::ZERO, T::ONE, T::ZERO)
    }

    /// Z軸単位ベクトルを取得
    pub fn unit_z() -> Self {
        Self::new(T::ZERO, T::ZERO, T::ONE)
    }

    /// タプルから作成
    pub fn from_tuple(components: (T, T, T)) -> Self {
        Self::new(components.0, components.1, components.2)
    }

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

    /// x成分を取得
    pub fn x(&self) -> T {
        self.x
    }

    /// y成分を取得
    pub fn y(&self) -> T {
        self.y
    }

    /// z成分を取得
    pub fn z(&self) -> T {
        self.z
    }

    /// 成分を配列として取得
    pub fn components(&self) -> [T; 3] {
        [self.x, self.y, self.z]
    }

    // ========================================================================
    // Core Calculation Methods
    // ========================================================================

    /// ベクトルの長さの二乗
    pub fn length_squared(&self) -> T {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// ベクトルの長さ
    pub fn length(&self) -> T {
        self.length_squared().sqrt()
    }

    /// ベクトルの大きさ（lengthのエイリアス）
    pub fn magnitude(&self) -> T {
        self.length()
    }

    /// ベクトルを正規化
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len == T::ZERO {
            Self::zero()
        } else {
            Self::new(self.x / len, self.y / len, self.z / len)
        }
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

    /// ベクトルの反転
    pub fn negate(&self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }

    /// 2点間のベクトルを作成
    pub fn from_points(from: &Point3D<T>, to: &Point3D<T>) -> Self {
        Self::new(to.x() - from.x(), to.y() - from.y(), to.z() - from.z())
    }

    /// ベクトルを点として解釈（原点からの位置ベクトル）
    pub fn to_point(&self) -> Point3D<T> {
        Point3D::new(self.x, self.y, self.z)
    }

    /// ゼロベクトルかどうかを判定
    pub fn is_zero(&self) -> bool {
        self.length() <= T::EPSILON
    }

    /// 他のベクトルと平行かどうかを判定
    pub fn is_parallel(&self, other: &Self) -> bool {
        let cross = self.cross(other);
        cross.length() <= T::EPSILON
    }

    /// 他のベクトルと垂直かどうかを判定
    pub fn is_perpendicular(&self, other: &Self) -> bool {
        self.dot(other).abs() <= T::EPSILON
    }

    /// ベクトルの境界ボックス（原点と終点を含む）
    pub fn bounding_box(&self) -> BBox3D<T> {
        let origin = Point3D::<T>::origin();
        let end_point = self.to_point();

        let min_x = origin.x().min(end_point.x());
        let max_x = origin.x().max(end_point.x());
        let min_y = origin.y().min(end_point.y());
        let max_y = origin.y().max(end_point.y());
        let min_z = origin.z().min(end_point.z());
        let max_z = origin.z().max(end_point.z());

        BBox3D::new(
            Point3D::new(min_x, min_y, min_z),
            Point3D::new(max_x, max_y, max_z),
        )
    }
}

// ============================================================================
// Core Operator implementations
// ============================================================================

impl<T: Scalar> std::ops::Add for Vector3D<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl<T: Scalar> std::ops::Sub for Vector3D<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl<T: Scalar> std::ops::Mul<T> for Vector3D<T> {
    type Output = Self;

    fn mul(self, scalar: T) -> Self::Output {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

impl<T: Scalar> std::ops::Div<T> for Vector3D<T> {
    type Output = Self;

    fn div(self, scalar: T) -> Self::Output {
        Self::new(self.x / scalar, self.y / scalar, self.z / scalar)
    }
}

impl<T: Scalar> std::ops::Neg for Vector3D<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.negate()
    }
}

// ============================================================================
// geo_foundation trait implementations
// ============================================================================

/// geo_foundation::core::Vector2D<T> トレイト実装
impl<T: Scalar> vector_traits::Vector2D<T> for Vector3D<T> {
    fn x(&self) -> T {
        self.x
    }

    fn y(&self) -> T {
        self.y
    }
}

/// geo_foundation::core::Vector3D<T> トレイト実装
impl<T: Scalar> vector_traits::Vector3D<T> for Vector3D<T> {
    fn z(&self) -> T {
        self.z
    }
}

/// geo_foundation::core::VectorMetrics<T> トレイト実装
impl<T: Scalar> vector_traits::VectorMetrics<T> for Vector3D<T> {
    fn length(&self) -> T {
        self.length()
    }

    fn length_squared(&self) -> T {
        self.length_squared()
    }

    fn normalize(&self) -> Self {
        self.normalize()
    }
}

/// geo_foundation::core::VectorOps<T> トレイト実装
impl<T: Scalar> vector_traits::VectorOps<T> for Vector3D<T> {
    fn add(&self, other: &Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    fn subtract(&self, other: &Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    fn scale(&self, scalar: T) -> Self {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }

    fn dot(&self, other: &Self) -> T {
        self.dot(other)
    }
}

// ============================================================================
// From trait implementations
// ============================================================================

/// タプルからの変換
impl<T: Scalar> From<(T, T, T)> for Vector3D<T> {
    fn from(tuple: (T, T, T)) -> Self {
        Self::new(tuple.0, tuple.1, tuple.2)
    }
}
