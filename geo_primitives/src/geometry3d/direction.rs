//! Direction3D - ジェネリック3D方向ベクトルの実装
//!
//! Direction2Dと同様のパターンでジェネリック化されたDirection3D実装

use crate::geometry3d::Vector;
use analysis::abstract_types::angle::Angle;
use geo_foundation::abstract_types::geometry::common::normalization_operations::Normalizable;
use geo_foundation::abstract_types::geometry::{Direction, Direction3D as Direction3DTrait};
use geo_foundation::Scalar;

/// ジェネリック3D方向ベクトル
///
/// 常に長さが1に正規化されたベクトルを表現する。
/// Direction2D<T>と同様のパターンで実装。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Direction3D<T: Scalar> {
    /// 正規化されたベクトル
    vector: Vector<T>,
}

impl<T: Scalar> Direction3D<T> {
    /// 内部用：正規化されたベクトルから直接作成（事前に正規化済みを前提）
    fn from_normalized_vector(vector: Vector<T>) -> Self {
        Self { vector }
    }

    /// 新しいDirection3Dを作成（正規化チェック付き）
    pub fn new(x: T, y: T, z: T) -> Option<Self> {
        let vector = Vector::new(x, y, z);
        Self::from_vector(vector)
    }

    /// X成分を取得
    pub fn x(&self) -> T {
        self.vector.x()
    }

    /// Y成分を取得
    pub fn y(&self) -> T {
        self.vector.y()
    }

    /// Z成分を取得
    pub fn z(&self) -> T {
        self.vector.z()
    }

    /// 内部ベクトルを取得
    pub fn to_vector(&self) -> Vector<T> {
        self.vector
    }

    /// 基本方向ベクトル：正のX軸方向
    pub fn positive_x() -> Self {
        Self::from_normalized_vector(Vector::new(T::ONE, T::ZERO, T::ZERO))
    }

    /// 基本方向ベクトル：正のY軸方向
    pub fn positive_y() -> Self {
        Self::from_normalized_vector(Vector::new(T::ZERO, T::ONE, T::ZERO))
    }

    /// 基本方向ベクトル：正のZ軸方向
    pub fn positive_z() -> Self {
        Self::from_normalized_vector(Vector::new(T::ZERO, T::ZERO, T::ONE))
    }

    /// 基本方向ベクトル：負のX軸方向
    pub fn negative_x() -> Self {
        Self::from_normalized_vector(Vector::new(-T::ONE, T::ZERO, T::ZERO))
    }

    /// 基本方向ベクトル：負のY軸方向
    pub fn negative_y() -> Self {
        Self::from_normalized_vector(Vector::new(T::ZERO, -T::ONE, T::ZERO))
    }

    /// 基本方向ベクトル：負のZ軸方向
    pub fn negative_z() -> Self {
        Self::from_normalized_vector(Vector::new(T::ZERO, T::ZERO, -T::ONE))
    }
}

impl<T: Scalar> Direction<T> for Direction3D<T> {
    type Vector = Vector<T>;

    fn from_vector(vector: Self::Vector) -> Option<Self> {
        vector.normalize().map(Self::from_normalized_vector)
    }

    fn to_vector(&self) -> Self::Vector {
        self.vector
    }

    fn dot(&self, other: &Self) -> T {
        self.vector.dot(&other.vector)
    }

    fn reverse(&self) -> Self {
        Self::from_normalized_vector(-self.vector)
    }

    fn is_parallel(&self, other: &Self, tolerance: T) -> bool {
        let dot = self.dot(other).abs();
        (dot - T::ONE).abs() < tolerance
    }

    fn is_perpendicular(&self, other: &Self, tolerance: T) -> bool {
        self.dot(other).abs() < tolerance
    }

    fn is_same_direction(&self, other: &Self, tolerance: T) -> bool {
        let dot = self.dot(other);
        (dot - T::ONE).abs() < tolerance
    }

    fn is_opposite_direction(&self, other: &Self, tolerance: T) -> bool {
        let dot = self.dot(other);
        (dot + T::ONE).abs() < tolerance
    }
}

impl<T: Scalar> Direction3DTrait<T> for Direction3D<T> {
    fn cross(&self, other: &Self) -> Self::Vector {
        self.vector.cross(&other.vector)
    }
}

// 独立した高度な操作メソッド（トレイト外実装）
impl<T: Scalar> Direction3D<T> {
    /// 軸周りの回転（Rodrigues回転公式）
    pub fn rotate_around_axis(&self, axis: &Self, angle: T) -> Self {
        // Angle型を使用して三角関数を計算
        let angle_obj = Angle::from_radians(angle);
        let cos_theta = angle_obj.cos();
        let sin_theta = angle_obj.sin();
        let axis_vec = axis.to_vector();
        let dot_product = self.dot(axis);

        let rotated = self.vector * cos_theta
            + axis_vec.cross(&self.vector) * sin_theta
            + axis_vec * dot_product * (T::ONE - cos_theta);

        Self::from_vector(rotated).unwrap()
    }

    /// 垂直な方向ベクトルを一つ取得
    pub fn any_perpendicular(&self) -> Self {
        // 最も小さい成分を持つ軸と外積を取る
        let abs_x = self.x().abs();
        let abs_y = self.y().abs();
        let abs_z = self.z().abs();

        let reference = if abs_x <= abs_y && abs_x <= abs_z {
            Vector::new(T::ONE, T::ZERO, T::ZERO)
        } else if abs_y <= abs_z {
            Vector::new(T::ZERO, T::ONE, T::ZERO)
        } else {
            Vector::new(T::ZERO, T::ZERO, T::ONE)
        };

        let cross = self.vector.cross(&reference);
        Self::from_vector(cross).unwrap()
    }

    /// 正規直交基底を構築
    pub fn build_orthonormal_basis(&self) -> (Self, Self, Self) {
        let u = self.any_perpendicular();
        let v = self.cross(&u);
        let v_normalized = Self::from_vector(v).unwrap();
        (*self, u, v_normalized)
    }

    /// X軸方向ベクトル
    pub fn x_axis() -> Self {
        Self::positive_x()
    }

    /// Y軸方向ベクトル
    pub fn y_axis() -> Self {
        Self::positive_y()
    }

    /// Z軸方向ベクトル
    pub fn z_axis() -> Self {
        Self::positive_z()
    }
}

// 型エイリアス（後方互換性確保）
/// f64版の3D Direction（デフォルト）
pub type Direction3DF64 = Direction3D<f64>;

/// f32版の3D Direction（高速演算用）
pub type Direction3DF32 = Direction3D<f32>;
