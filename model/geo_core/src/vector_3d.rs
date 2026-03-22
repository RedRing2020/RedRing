//! Vector3D - 3次元ベクトルの完全実装
//!
//! geo_core における Vector3D の完全実装。
//! 基本機能、Foundation トレイト、拡張機能、演算子オーバーロードを含む。

use crate::Point3D;
use analysis::abstract_types::{Scalar, TolerantEq};

/// 3次元ベクトル
///
/// ## 設計方針
/// - geo_core の基本型として全機能を実装
/// - Foundation トレイト完全対応
/// - Point3D との演算子オーバーロード対応
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3D<T: Scalar> {
    x: T,
    y: T,
    z: T,
}

// ============================================================================
// Core Implementation
// ============================================================================

impl<T: Scalar> Vector3D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================

    /// 新しいベクトルを作成
    ///
    /// # Examples
    /// ```
    /// use geo_core::Vector3D;
    ///
    /// let v = Vector3D::new(1.0, 2.0, 3.0);
    /// assert_eq!(v.x(), 1.0);
    /// assert_eq!(v.y(), 2.0);
    /// assert_eq!(v.z(), 3.0);
    /// ```
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
    ///
    /// # Examples
    /// ```
    /// use geo_core::Vector3D;
    ///
    /// let v = Vector3D::new(3.0, 4.0, 0.0);
    /// assert_eq!(v.length(), 5.0);
    /// ```
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

    /// 正規化を試行（ゼロベクトルの場合はNoneを返す）
    pub fn try_normalize(&self) -> Option<Self> {
        let len = self.length();
        if len <= T::ZERO {
            None
        } else {
            Some(Self::new(self.x / len, self.y / len, self.z / len))
        }
    }

    /// 内積
    ///
    /// # Examples
    /// ```
    /// use geo_core::Vector3D;
    ///
    /// let v1 = Vector3D::new(1.0, 0.0, 0.0);
    /// let v2 = Vector3D::new(0.0, 1.0, 0.0);
    /// assert_eq!(v1.dot(&v2), 0.0);
    /// ```
    pub fn dot(&self, other: &Self) -> T {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// 外積
    ///
    /// # Examples
    /// ```
    /// use geo_core::Vector3D;
    ///
    /// let v1 = Vector3D::new(1.0, 0.0, 0.0);
    /// let v2 = Vector3D::new(0.0, 1.0, 0.0);
    /// let cross = v1.cross(&v2);
    /// assert_eq!(cross.z(), 1.0);
    /// ```
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
        self.is_parallel_with_error_tolerance(other, T::PARALLEL_CROSS_ERROR_TOLERANCE)
    }

    /// 他のベクトルと垂直かどうかを判定
    pub fn is_perpendicular(&self, other: &Self) -> bool {
        self.is_perpendicular_with_error_tolerance(other, T::ORTHOGONALITY_DOT_ERROR_TOLERANCE)
    }

    /// 他のベクトルと平行かどうかを数値誤差閾値つきで判定
    pub fn is_parallel_with_error_tolerance(&self, other: &Self, tolerance: T) -> bool {
        let cross = self.cross(other);
        cross.length() <= tolerance
    }

    /// 他のベクトルと垂直かどうかを数値誤差閾値つきで判定
    pub fn is_perpendicular_with_error_tolerance(&self, other: &Self, tolerance: T) -> bool {
        self.dot(other).abs() <= tolerance
    }

    // ========================================================================
    // Extended Methods
    // ========================================================================

    /// ベクトルの線形補間
    pub fn lerp(&self, other: &Self, t: T) -> Self {
        *self + (*other - *self) * t
    }

    /// ベクトルの球面線形補間（SLERP）
    pub fn slerp(&self, other: &Self, t: T) -> Self {
        let dot = self.dot(other).clamp(-T::ONE, T::ONE);
        let angle = dot.acos();

        if angle.abs() < T::EPSILON {
            return self.lerp(other, t);
        }

        let sin_angle = angle.sin();
        let factor1 = ((T::ONE - t) * angle).sin() / sin_angle;
        let factor2 = (t * angle).sin() / sin_angle;

        *self * factor1 + *other * factor2
    }

    /// ベクトル間の角度を計算（ラジアン）
    pub fn angle_between(&self, other: &Self) -> T {
        let dot = self.dot(other);
        let lengths = self.length() * other.length();

        if lengths <= T::ZERO {
            return T::ZERO;
        }

        (dot / lengths).clamp(-T::ONE, T::ONE).acos()
    }

    /// ベクトルを指定された長さにスケール
    pub fn with_length(&self, new_length: T) -> Self {
        let current_length = self.length();
        if current_length <= T::ZERO {
            *self
        } else {
            *self * (new_length / current_length)
        }
    }

    /// ベクトルの投影
    pub fn project_onto(&self, other: &Self) -> Self {
        let other_length_sq = other.length_squared();
        if other_length_sq <= T::ZERO {
            Self::zero()
        } else {
            *other * (self.dot(other) / other_length_sq)
        }
    }

    /// ベクトルの反射
    pub fn reflect(&self, normal: &Self) -> Self {
        let two = T::ONE + T::ONE;
        *self - *normal * (two * self.dot(normal))
    }

    /// 他のベクトルとの関係性を判定
    pub fn relationship_with(&self, other: &Self) -> VectorRelationship {
        if self.is_parallel(other) {
            if self.dot(other) > T::ZERO {
                VectorRelationship::SameDirection
            } else {
                VectorRelationship::OppositeDirection
            }
        } else if self.is_perpendicular(other) {
            VectorRelationship::Perpendicular
        } else {
            VectorRelationship::Oblique
        }
    }

    /// ベクトルの符号付き体積（3つのベクトルのスカラー三重積）
    pub fn scalar_triple_product(&self, b: &Self, c: &Self) -> T {
        self.dot(&b.cross(c))
    }

    /// ベクトルの主要軸（最大成分）を取得
    pub fn dominant_axis(&self) -> DominantAxis {
        let abs_x = self.x().abs();
        let abs_y = self.y().abs();
        let abs_z = self.z().abs();

        if abs_x >= abs_y && abs_x >= abs_z {
            DominantAxis::X
        } else if abs_y >= abs_z {
            DominantAxis::Y
        } else {
            DominantAxis::Z
        }
    }
}

// ============================================================================
// Helper Enums
// ============================================================================

/// ベクトル間の関係性
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorRelationship {
    /// 同じ方向
    SameDirection,
    /// 反対方向
    OppositeDirection,
    /// 垂直
    Perpendicular,
    /// 斜交
    Oblique,
}

/// 主要軸
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DominantAxis {
    /// X軸
    X,
    /// Y軸
    Y,
    /// Z軸
    Z,
}

// ============================================================================
// Operator Implementations
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
// Point3D との演算子オーバーロード
// ============================================================================

// Point - Point = Vector (2点間のベクトル)
impl<T: Scalar> std::ops::Sub for Point3D<T> {
    type Output = Vector3D<T>;

    fn sub(self, other: Self) -> Self::Output {
        Vector3D::new(
            self.x() - other.x(),
            self.y() - other.y(),
            self.z() - other.z(),
        )
    }
}

// Point + Vector = Point (点をベクトル分移動)
impl<T: Scalar> std::ops::Add<Vector3D<T>> for Point3D<T> {
    type Output = Point3D<T>;

    fn add(self, vector: Vector3D<T>) -> Self::Output {
        Point3D::new(
            self.x() + vector.x(),
            self.y() + vector.y(),
            self.z() + vector.z(),
        )
    }
}

// Point - Vector = Point (点をベクトル分逆移動)
impl<T: Scalar> std::ops::Sub<Vector3D<T>> for Point3D<T> {
    type Output = Point3D<T>;

    fn sub(self, vector: Vector3D<T>) -> Self::Output {
        Point3D::new(
            self.x() - vector.x(),
            self.y() - vector.y(),
            self.z() - vector.z(),
        )
    }
}

// ============================================================================
// Core Traits Implementation (Foundation Pattern)
// ============================================================================

use crate::vector_traits::{
    Vector3DConstructor, Vector3DCore, Vector3DMeasure, Vector3DProperties,
};
use analysis::linalg::vector::Vector3;

impl<T: Scalar> Vector3DConstructor<T> for Vector3D<T> {
    fn new(x: T, y: T, z: T) -> Self {
        Self::new(x, y, z)
    }

    fn zero() -> Self {
        Self::zero()
    }

    fn unit_x() -> Self {
        Self::unit_x()
    }

    fn unit_y() -> Self {
        Self::unit_y()
    }

    fn unit_z() -> Self {
        Self::unit_z()
    }

    fn from_tuple(components: (T, T, T)) -> Self {
        Self::new(components.0, components.1, components.2)
    }

    fn from_analysis_vector(vector: &Vector3<T>) -> Self {
        Self::new(vector.x(), vector.y(), vector.z())
    }

    fn from_array(components: [T; 3]) -> Self {
        Self::new(components[0], components[1], components[2])
    }

    fn from_spherical(magnitude: T, azimuth: T, elevation: T) -> Self {
        let x = magnitude * elevation.cos() * azimuth.cos();
        let y = magnitude * elevation.cos() * azimuth.sin();
        let z = magnitude * elevation.sin();
        Self::new(x, y, z)
    }

    fn from_cylindrical(radial_distance: T, azimuth: T, height: T) -> Self {
        let x = radial_distance * azimuth.cos();
        let y = radial_distance * azimuth.sin();
        let z = height;
        Self::new(x, y, z)
    }

    fn from_vector(other: &Self) -> Self {
        *other
    }
}

impl<T: Scalar> Vector3DProperties<T> for Vector3D<T> {
    fn x(&self) -> T {
        self.x
    }

    fn y(&self) -> T {
        self.y
    }

    fn z(&self) -> T {
        self.z
    }

    fn components(&self) -> [T; 3] {
        [self.x, self.y, self.z]
    }

    fn to_tuple(&self) -> (T, T, T) {
        (self.x, self.y, self.z)
    }

    fn to_analysis_vector(&self) -> Vector3<T> {
        Vector3::new(self.x, self.y, self.z)
    }

    fn length(&self) -> T {
        Vector3D::length(self)
    }

    fn length_squared(&self) -> T {
        Vector3D::length_squared(self)
    }

    fn normalize(&self) -> Self {
        Vector3D::normalize(self)
    }

    fn try_normalize(&self) -> Option<Self> {
        Vector3D::try_normalize(self)
    }
}

impl<T: Scalar> Vector3DMeasure<T> for Vector3D<T> {
    fn dot(&self, other: &Self) -> T {
        self.dot(other)
    }

    fn cross_3d(&self, other: &Self) -> Self {
        self.cross(other)
    }

    fn angle_to(&self, other: &Self) -> Option<T> {
        Some(self.angle_between(other))
    }

    fn distance_to(&self, other: &Self) -> T {
        (*self - *other).length()
    }

    fn distance_squared_to(&self, other: &Self) -> T {
        (*self - *other).length_squared()
    }

    fn magnitude(&self) -> T {
        self.magnitude()
    }

    fn manhattan_distance(&self) -> T {
        self.x.abs() + self.y.abs() + self.z.abs()
    }

    fn is_parallel_to(&self, other: &Self) -> bool {
        self.is_parallel(other)
    }

    fn is_perpendicular_to(&self, other: &Self) -> bool {
        self.is_perpendicular(other)
    }

    fn project_onto(&self, other: &Self) -> Option<Self> {
        Some(self.project_onto(other))
    }

    fn project_onto_plane(&self, normal: &Self) -> Option<Self> {
        let proj = self.project_onto(normal);
        Some(*self - proj)
    }
}

impl<T: Scalar> Vector3DCore<T> for Vector3D<T> {}

impl<T: Scalar> TolerantEq<T> for Vector3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        let diff = *self - *other;
        let diff_magnitude = diff.magnitude();
        diff_magnitude <= tolerance
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
