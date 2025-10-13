//! Point2D拡張メソッド
//!
//! Core Foundation パターンに基づく Point2D の拡張機能
//! 基本機能は point_2d.rs を参照

use crate::{Point2D, Vector2D};
use geo_foundation::{Angle, Scalar};

impl<T: Scalar> Point2D<T> {
    // ========================================================================
    // Extension Interpolation Methods
    // ========================================================================

    /// 点を別の点に向かって線形補間
    pub fn lerp(&self, other: &Self, t: T) -> Self {
        let one_minus_t = T::ONE - t;
        Point2D::new(
            one_minus_t * self.x() + t * other.x(),
            one_minus_t * self.y() + t * other.y(),
        )
    }

    /// 他の点との中点を計算
    pub fn midpoint(&self, other: &Self) -> Self {
        let half = T::from_f64(0.5);
        self.lerp(other, half)
    }

    // ========================================================================
    // Extension Transformation Methods
    // ========================================================================

    /// ベクトルによる平行移動
    pub fn translate(&self, vector: Vector2D<T>) -> Self {
        *self + vector
    }

    /// 原点周りの回転
    pub fn rotate(&self, angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Point2D::new(
            self.x() * cos_a - self.y() * sin_a,
            self.x() * sin_a + self.y() * cos_a,
        )
    }

    /// 指定点周りの回転
    pub fn rotate_around(&self, center: &Self, angle: Angle<T>) -> Self {
        let offset = *self - *center;
        let rotated_offset = offset.rotate(angle);
        *center + rotated_offset
    }

    /// スケール変換
    pub fn scale(&self, scale_x: T, scale_y: T) -> Self {
        Point2D::new(self.x() * scale_x, self.y() * scale_y)
    }

    /// 均等スケール変換
    pub fn scale_uniform(&self, scale: T) -> Self {
        self.scale(scale, scale)
    }

    /// X軸に関する反射
    pub fn reflect_x(&self) -> Self {
        Point2D::new(-self.x(), self.y())
    }

    /// Y軸に関する反射
    pub fn reflect_y(&self) -> Self {
        Point2D::new(self.x(), -self.y())
    }

    /// 原点に関する反射
    pub fn reflect_origin(&self) -> Self {
        Point2D::new(-self.x(), -self.y())
    }

    // ========================================================================
    // Extension Predicate Methods
    // ========================================================================

    /// 原点かどうかを判定
    pub fn is_origin(&self) -> bool {
        self.x().abs() <= T::EPSILON && self.y().abs() <= T::EPSILON
    }

    /// 近似等価判定
    pub fn is_approximately_equal(&self, other: &Self, tolerance: T) -> bool {
        (self.x() - other.x()).abs() <= tolerance && (self.y() - other.y()).abs() <= tolerance
    }

    // ========================================================================
    // Extension Type Conversion Methods
    // ========================================================================

    /// Vector2Dに変換
    pub fn to_vector(&self) -> Vector2D<T> {
        Vector2D::new(self.x(), self.y())
    }

    /// 他の点へのベクトルを取得
    pub fn vector_to(&self, other: &Self) -> Vector2D<T> {
        Vector2D::new(other.x() - self.x(), other.y() - self.y())
    }

    /// ベクトルから点を作成
    pub fn from_vector(vector: Vector2D<T>) -> Self {
        Point2D::new(vector.x(), vector.y())
    }

    // ========================================================================
    // Extension Dimension Conversion Methods
    // ========================================================================

    /// 3D点に変換（Z座標は0）
    pub fn to_3d(&self) -> crate::Point3D<T> {
        crate::Point3D::new(self.x(), self.y(), T::ZERO)
    }

    /// 3D点に変換（Z座標を指定）
    pub fn to_3d_with_z(&self, z: T) -> crate::Point3D<T> {
        crate::Point3D::new(self.x(), self.y(), z)
    }
}
