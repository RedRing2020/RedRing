//! Vector2D拡張メソッド
//!
//! Core Foundation パターンに基づく Vector2D の拡張機能
//! 基本機能は vector_2d.rs を参照

use crate::{Point2D, Vector2D};
use geo_foundation::{Angle, Scalar};

impl<T: Scalar> Vector2D<T> {
    // ========================================================================
    // Extension Construction Methods
    // ========================================================================

    /// 角度からベクトルを作成（単位ベクトル）
    pub fn from_angle(angle: T) -> Self {
        Self::new(angle.cos(), angle.sin())
    }

    /// 角度と長さからベクトルを作成
    pub fn from_angle_length(angle: T, length: T) -> Self {
        Self::new(angle.cos() * length, angle.sin() * length)
    }

    /// 2つの点からベクトルを作成（to - from）
    pub fn from_points(from: Point2D<T>, to: Point2D<T>) -> Self {
        from.vector_to(&to)
    }

    /// Point2D から Vector2D を作成（原点からのベクトルとして）
    pub fn from_point(point: &Point2D<T>) -> Self {
        Self::new(point.x(), point.y())
    }

    // ========================================================================
    // Extension Predicate Methods
    // ========================================================================

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
    pub fn is_parallel(&self, other: &Self, tolerance: T) -> bool {
        self.cross(other).abs() <= tolerance
    }

    /// 2つのベクトルが垂直かを判定
    pub fn is_perpendicular(&self, other: &Self, tolerance: T) -> bool {
        self.dot(other).abs() <= tolerance
    }

    // ========================================================================
    // Extension Normalization Methods
    // ========================================================================

    /// 正規化を試行（ゼロベクトルの場合はNoneを返す）
    pub fn try_normalize(&self) -> Option<Self> {
        let len = self.length();
        if len <= T::ZERO {
            None
        } else {
            Some(Self::new(self.x() / len, self.y() / len))
        }
    }

    /// ベクトルを指定長さにスケール
    pub fn with_length(&self, new_length: T) -> Option<Self> {
        self.try_normalize().map(|unit| unit * new_length)
    }

    // ========================================================================
    // Extension Transformation Methods
    // ========================================================================

    /// ベクトルを指定角度回転
    pub fn rotate(&self, angle: Angle<T>) -> Self {
        let radians = angle.to_radians();
        let cos_a = radians.cos();
        let sin_a = radians.sin();
        Self::new(
            self.x() * cos_a - self.y() * sin_a,
            self.x() * sin_a + self.y() * cos_a,
        )
    }

    /// ベクトルを90度回転（反時計回り）
    pub fn rotate_90(&self) -> Self {
        Self::new(-self.y(), self.x())
    }

    /// ベクトルを90度回転（時計回り）
    pub fn rotate_neg_90(&self) -> Self {
        Self::new(self.y(), -self.x())
    }

    /// ベクトルを反射（指定ベクトル方向の法線で反射）
    pub fn reflect(&self, normal: &Self) -> Self {
        let normal_unit = normal.normalize();
        let two = T::ONE + T::ONE;
        *self - normal_unit * (self.dot(&normal_unit) * two)
    }

    /// ベクトルを反転（-selfと同じ）
    pub fn negate(&self) -> Self {
        Self::new(-self.x(), -self.y())
    }

    // ========================================================================
    // Extension Angle Methods
    // ========================================================================

    /// ベクトル間の角度を取得（ラジアン）
    pub fn angle_to(&self, other: &Self) -> T {
        let dot = self.dot(other);
        let cross = self.cross(other);
        cross.atan2(dot)
    }

    /// ベクトルのX軸からの角度を取得（ラジアン）
    pub fn angle(&self) -> T {
        self.y().atan2(self.x())
    }

    // ========================================================================
    // Extension Linear Operations
    // ========================================================================

    /// 2つのベクトル間で線形補間
    pub fn lerp(&self, other: &Self, t: T) -> Self {
        Self::new(
            self.x() + (other.x() - self.x()) * t,
            self.y() + (other.y() - self.y()) * t,
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

    // ========================================================================
    // Extension Component Operations
    // ========================================================================

    /// 成分ごとの最小値
    pub fn min(&self, other: &Self) -> Self {
        Self::new(self.x().min(other.x()), self.y().min(other.y()))
    }

    /// 成分ごとの最大値
    pub fn max(&self, other: &Self) -> Self {
        Self::new(self.x().max(other.x()), self.y().max(other.y()))
    }

    /// 成分ごとの絶対値
    pub fn abs(&self) -> Self {
        Self::new(self.x().abs(), self.y().abs())
    }

    // ========================================================================
    // Extension Type Conversion Methods
    // ========================================================================

    /// Vector2D を Point2D に変換（ベクトルを位置として解釈）
    pub fn to_point(&self) -> Point2D<T> {
        Point2D::new(self.x(), self.y())
    }

    /// 指定した点にこのベクトルを適用して新しい点を取得
    pub fn apply_to_point(&self, point: &Point2D<T>) -> Point2D<T> {
        Point2D::new(point.x() + self.x(), point.y() + self.y())
    }

    // ========================================================================
    // Extension Dimension Conversion Methods
    // ========================================================================

    /// 3次元ベクトルに拡張（Z=0）
    pub fn to_3d(&self) -> crate::Vector3D<T> {
        crate::Vector3D::new(self.x(), self.y(), T::ZERO)
    }

    /// 3次元ベクトルに拡張（指定Z値）
    pub fn to_3d_with_z(&self, z: T) -> crate::Vector3D<T> {
        crate::Vector3D::new(self.x(), self.y(), z)
    }
}
