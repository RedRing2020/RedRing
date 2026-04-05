//! 2次元ベクトル（Vector2D）の実装
//!
//! geo_core内部実装として、全機能を統合

use analysis::abstract_types::{Angle, Scalar};

use std::ops::{Add, Mul, Neg, Sub};

/// 2次元ベクトル
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2D<T: Scalar> {
    x: T,
    y: T,
}

impl<T: Scalar> Vector2D<T> {
    /// 新しいベクトルを作成
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// ゼロベクトルを取得
    pub fn zero() -> Self {
        Self::new(T::ZERO, T::ZERO)
    }

    /// X軸単位ベクトルを取得
    pub fn unit_x() -> Self {
        Self::new(T::ONE, T::ZERO)
    }

    /// Y軸単位ベクトルを取得
    pub fn unit_y() -> Self {
        Self::new(T::ZERO, T::ONE)
    }

    /// X成分を取得
    pub fn x(&self) -> T {
        self.x
    }

    /// Y成分を取得
    pub fn y(&self) -> T {
        self.y
    }

    /// 成分を配列として取得
    pub fn components(&self) -> [T; 2] {
        [self.x, self.y]
    }

    /// ベクトルの長さの二乗を取得
    pub fn length_squared(&self) -> T {
        self.x * self.x + self.y * self.y
    }

    /// ベクトルの長さ（ノルム）を計算
    pub fn length(&self) -> T {
        self.length_squared().sqrt()
    }

    /// ベクトルの大きさ（長さの別名）
    pub fn magnitude(&self) -> T {
        self.length()
    }

    /// ベクトルを正規化（長さを1にする）
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len <= T::ZERO {
            Self::zero()
        } else {
            Self::new(self.x / len, self.y / len)
        }
    }

    /// 内積を計算
    pub fn dot(&self, other: &Self) -> T {
        self.x * other.x + self.y * other.y
    }

    /// 外積のZ成分を計算（2Dでは実際の外積ではなく、Z成分のスカラー値）
    pub fn cross(&self, other: &Self) -> T {
        self.x * other.y - self.y * other.x
    }

    /// 角度からベクトルを作成（単位ベクトル）
    pub fn from_angle(angle: Angle<T>) -> Self {
        Self::new(angle.cos(), angle.sin())
    }

    /// 角度と長さからベクトルを作成
    pub fn from_angle_length(angle: Angle<T>, length: T) -> Self {
        Self::new(angle.cos() * length, angle.sin() * length)
    }

    /// 2つの点からベクトルを作成（to - from）
    pub fn from_points(from: crate::Point2D<T>, to: crate::Point2D<T>) -> Self {
        from.vector_to(&to)
    }

    /// Point2D から Vector2D を作成（原点からのベクトルとして）
    pub fn from_point(point: &crate::Point2D<T>) -> Self {
        Self::new(point.x(), point.y())
    }

    /// ベクトルが単位ベクトルかを判定
    pub fn is_unit(&self, tolerance: T) -> bool {
        let len = self.length();
        (len - T::ONE).abs() <= tolerance
    }

    /// ベクトルがゼロベクトルかを判定
    pub fn is_zero(&self, tolerance: T) -> bool {
        self.length() <= tolerance
    }

    /// 2つのベクトルが平行かを判定
    pub fn is_parallel(&self, other: &Self) -> bool {
        self.is_parallel_with_error_tolerance(other, T::PARALLEL_CROSS_ERROR_TOLERANCE)
    }

    /// 2つのベクトルが平行かを数値誤差閾値つきで判定
    pub fn is_parallel_with_error_tolerance(&self, other: &Self, tolerance: T) -> bool {
        self.cross(other).abs() <= tolerance
    }

    /// 2つのベクトルが垂直かを判定
    pub fn is_perpendicular(&self, other: &Self) -> bool {
        self.is_perpendicular_with_error_tolerance(other, T::ORTHOGONALITY_DOT_ERROR_TOLERANCE)
    }

    /// 2つのベクトルが垂直かを数値誤差閾値つきで判定
    pub fn is_perpendicular_with_error_tolerance(&self, other: &Self, tolerance: T) -> bool {
        self.dot(other).abs() <= tolerance
    }

    /// 正規化を試行（ゼロベクトルの場合はNoneを返す）
    pub fn try_normalize(&self) -> Option<Self> {
        let len = self.length();
        if len <= T::ZERO {
            None
        } else {
            Some(Self::new(self.x / len, self.y / len))
        }
    }

    /// ベクトルを指定長さにスケール
    pub fn with_length(&self, new_length: T) -> Option<Self> {
        self.try_normalize().map(|unit| unit * new_length)
    }

    /// ベクトルを指定角度回転
    pub fn rotate(&self, angle: Angle<T>) -> Self {
        let radians = angle.to_radians();
        let cos_a = radians.cos();
        let sin_a = radians.sin();
        Self::new(
            self.x * cos_a - self.y * sin_a,
            self.x * sin_a + self.y * cos_a,
        )
    }

    /// ベクトルを90度回転（反時計回り）
    pub fn rotate_90(&self) -> Self {
        Self::new(-self.y, self.x)
    }

    /// ベクトルを90度回転（時計回り）
    pub fn rotate_neg_90(&self) -> Self {
        Self::new(self.y, -self.x)
    }

    /// ベクトルを反射（指定ベクトル方向の法線で反射）
    pub fn reflect(&self, normal: &Self) -> Self {
        let normal_unit = normal.normalize();
        let two = T::ONE + T::ONE;
        *self - normal_unit * (self.dot(&normal_unit) * two)
    }

    /// ベクトルを反転（-selfと同じ）
    pub fn negate(&self) -> Self {
        Self::new(-self.x, -self.y)
    }

    /// ベクトル間の角度を取得
    pub fn angle_to(&self, other: &Self) -> Angle<T> {
        let dot = self.dot(other);
        let cross = self.cross(other);
        Angle::from_radians(cross.atan2(dot))
    }

    /// ベクトルのX軸からの角度を取得
    pub fn angle(&self) -> Angle<T> {
        Angle::from_radians(self.y.atan2(self.x))
    }

    /// 2つのベクトル間で線形補間
    pub fn lerp(&self, other: &Self, t: T) -> Self {
        Self::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
        )
    }

    /// 投影ベクトルを計算（このベクトルをotherに投影）
    pub fn project_onto(&self, other: &Self) -> Self {
        let other_len_sq = other.length_squared();
        if other_len_sq <= T::ZERO {
            Self::zero()
        } else {
            *other * (self.dot(other) / other_len_sq)
        }
    }

    /// 拒絶ベクトルを計算（投影の残り）
    pub fn reject_from(&self, other: &Self) -> Self {
        *self - self.project_onto(other)
    }

    /// 成分ごとの最小値
    pub fn min(&self, other: &Self) -> Self {
        Self::new(self.x.min(other.x), self.y.min(other.y))
    }

    /// 成分ごとの最大値
    pub fn max(&self, other: &Self) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y))
    }

    /// 成分ごとの絶対値
    pub fn abs(&self) -> Self {
        Self::new(self.x.abs(), self.y.abs())
    }

    /// Vector2D を Point2D に変換（ベクトルを位置として解釈）
    pub fn to_point(&self) -> crate::Point2D<T> {
        crate::Point2D::new(self.x, self.y)
    }

    /// 指定した点にこのベクトルを適用して新しい点を取得
    pub fn apply_to_point(&self, point: &crate::Point2D<T>) -> crate::Point2D<T> {
        crate::Point2D::new(point.x() + self.x, point.y() + self.y)
    }

    /// 3次元ベクトルに拡張（Z=0）
    pub fn to_3d(&self) -> crate::Vector3D<T> {
        crate::Vector3D::new(self.x, self.y, T::ZERO)
    }

    /// 3次元ベクトルに拡張（指定Z値）
    pub fn to_3d_with_z(&self, z: T) -> crate::Vector3D<T> {
        crate::Vector3D::new(self.x, self.y, z)
    }
}

impl<T: Scalar> Add for Vector2D<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl<T: Scalar> Sub for Vector2D<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl<T: Scalar> Mul<T> for Vector2D<T> {
    type Output = Self;

    fn mul(self, scalar: T) -> Self::Output {
        Self::new(self.x * scalar, self.y * scalar)
    }
}

impl<T: Scalar> std::ops::Div<T> for Vector2D<T> {
    type Output = Self;

    fn div(self, scalar: T) -> Self::Output {
        Self::new(self.x / scalar, self.y / scalar)
    }
}

impl<T: Scalar> Neg for Vector2D<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}

impl<T: Scalar> From<(T, T)> for Vector2D<T> {
    fn from(tuple: (T, T)) -> Self {
        Self::new(tuple.0, tuple.1)
    }
}

use analysis::linalg::vector::Vector2;
use geo_contracts::{Vector2DConstructor, Vector2DCore, Vector2DMeasure, Vector2DProperties};

impl<T: Scalar> Vector2DConstructor<T> for Vector2D<T> {
    fn new(x: T, y: T) -> Self {
        Self::new(x, y)
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

    fn from_tuple(components: (T, T)) -> Self {
        Self::new(components.0, components.1)
    }

    fn from_analysis_vector(vector: &Vector2<T>) -> Self {
        Self::new(vector.x(), vector.y())
    }

    fn from_array(components: [T; 2]) -> Self {
        Self::new(components[0], components[1])
    }

    fn from_polar(magnitude: T, angle: T) -> Self {
        Self::new(magnitude * angle.cos(), magnitude * angle.sin())
    }

    fn from_vector(other: &Self) -> Self {
        *other
    }
}

impl<T: Scalar> Vector2DProperties<T> for Vector2D<T> {
    fn x(&self) -> T {
        self.x
    }

    fn y(&self) -> T {
        self.y
    }

    fn components(&self) -> [T; 2] {
        [self.x, self.y]
    }

    fn to_tuple(&self) -> (T, T) {
        (self.x, self.y)
    }

    fn to_analysis_vector(&self) -> Vector2<T> {
        Vector2::new(self.x, self.y)
    }

    fn length(&self) -> T {
        Vector2D::length(self)
    }

    fn length_squared(&self) -> T {
        Vector2D::length_squared(self)
    }

    fn normalize(&self) -> Self {
        Vector2D::normalize(self)
    }

    fn try_normalize(&self) -> Option<Self> {
        Vector2D::try_normalize(self)
    }
}

impl<T: Scalar> Vector2DMeasure<T> for Vector2D<T> {
    fn dot(&self, other: &Self) -> T {
        self.dot(other)
    }

    fn cross_2d(&self, other: &Self) -> T {
        self.cross(other)
    }

    fn angle_to(&self, other: &Self) -> Option<T> {
        Some(self.angle_to(other).to_radians())
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
        self.x.abs() + self.y.abs()
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
}

impl<T: Scalar> Vector2DCore<T> for Vector2D<T> {}
