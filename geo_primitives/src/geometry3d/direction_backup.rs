//! Direction3D - ジェネリック3D方向ベクトルの実装
//!
//! Direction2Dと同様のパターンでジェネリック化されたDirection3D実装

use crate::geometry3d::Vector;
use geo_foundation::abstract_types::geometry::{Direction, Direction3D as Direction3DTrait};
use geo_foundation::abstract_types::Scalar;

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

    fn from_components_3d(x: T, y: T, z: T) -> Option<Self> {
        Self::new(x, y, z)
    }
}

// 型エイリアス（後方互換性確保）
/// f64版の3D Direction（デフォルト）
pub type Direction3DF64 = Direction3D<f64>;

/// f32版の3D Direction（高速演算用）
pub type Direction3DF32 = Direction3D<f32>;
    }

    fn build_orthonormal_basis(&self) -> (Self, Self, Self) {
        let w = *self; // Z軸
        let u = self.any_perpendicular(); // X軸
        let v_vec = w.vector.cross(&u.vector); // Y軸
        let v = Self::from_normalized_vector(v_vec);

        (u, v, w)
    }

    fn x_axis() -> Self {
        Self::from_normalized_vector(Vector3D::new(1.0, 0.0, 0.0))
    }

    fn y_axis() -> Self {
        Self::from_normalized_vector(Vector3D::new(0.0, 1.0, 0.0))
    }

    fn z_axis() -> Self {
        Self::from_normalized_vector(Vector3D::new(0.0, 0.0, 1.0))
    }
}

impl StepCompatible for Direction3D {
    fn to_step_string(&self) -> String {
        format!(
            "DIRECTION('',({:.6},{:.6},{:.6}))",
            self.x(),
            self.y(),
            self.z()
        )
    }

    fn from_step_string(_step_str: &str) -> Result<Self, String> {
        // 将来実装予定
        Err("Not implemented yet".to_string())
    }
}

impl std::fmt::Display for Direction3D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Direction3D({:.3}, {:.3}, {:.3})",
            self.x(),
            self.y(),
            self.z()
        )
    }
}

// 便利な定数
impl Direction3D {
    /// 正のX軸方向
    /// 正のX軸方向を取得
    pub fn positive_x() -> Self {
        Self::from_normalized_vector(Vector3D::new(1.0, 0.0, 0.0))
    }

    /// 正のY軸方向を取得
    pub fn positive_y() -> Self {
        Self::from_normalized_vector(Vector3D::new(0.0, 1.0, 0.0))
    }

    /// 正のZ軸方向を取得
    pub fn positive_z() -> Self {
        Self::from_normalized_vector(Vector3D::new(0.0, 0.0, 1.0))
    }

    /// 負のX軸方向を取得
    pub fn negative_x() -> Self {
        Self::from_normalized_vector(Vector3D::new(-1.0, 0.0, 0.0))
    }

    /// 負のY軸方向を取得
    pub fn negative_y() -> Self {
        Self::from_normalized_vector(Vector3D::new(0.0, -1.0, 0.0))
    }

    /// 負のZ軸方向を取得
    pub fn negative_z() -> Self {
        Self::from_normalized_vector(Vector3D::new(0.0, 0.0, -1.0))
    }
}
