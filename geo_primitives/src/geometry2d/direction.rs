//! Direction2D - ジェネリック2D方向ベクトルの実装
//!
//! 常に正規化されたベクトルとして管理され、型パラメータTをサポート

use crate::geometry2d::Vector;
use crate::traits::StepCompatible;
use geo_foundation::abstract_types::geometry::{Direction, Direction2D as Direction2DTrait};
use geo_foundation::abstract_types::Scalar;

/// ジェネリック2D方向ベクトル
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Direction2D<T: Scalar> {
    /// 正規化されたベクトル（内部的にはVectorを使用）
    vector: Vector<T>,
}

impl<T: Scalar> Direction2D<T> {
    /// 新しいDirection2Dを作成（内部で正規化）
    pub fn new(x: T, y: T) -> Option<Self> {
        let vector = Vector::new(x, y);
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

    /// 定数方向ベクトル - 正のX軸方向
    pub fn positive_x() -> Self {
        Self {
            vector: Vector::new(T::ONE, T::ZERO),
        }
    }

    /// 定数方向ベクトル - 正のY軸方向
    pub fn positive_y() -> Self {
        Self {
            vector: Vector::new(T::ZERO, T::ONE),
        }
    }

    /// 定数方向ベクトル - 負のX軸方向
    pub fn negative_x() -> Self {
        Self {
            vector: Vector::new(-T::ONE, T::ZERO),
        }
    }

    /// 定数方向ベクトル - 負のY軸方向
    pub fn negative_y() -> Self {
        Self {
            vector: Vector::new(T::ZERO, -T::ONE),
        }
    }
}

impl<T: Scalar> Direction<T> for Direction2D<T> {
    type Vector = Vector<T>;

    fn from_vector(vector: Self::Vector) -> Option<Self>
    where
        Self: Sized,
    {
        vector
            .normalize()
            .map(|normalized| Self { vector: normalized })
    }

    fn to_vector(&self) -> Self::Vector {
        self.vector
    }

    fn dot(&self, other: &Self) -> T {
        self.vector.dot(&other.vector)
    }

    fn reverse(&self) -> Self {
        Self {
            vector: -self.vector,
        }
    }

    fn is_parallel(&self, other: &Self, tolerance: T) -> bool {
        let cross = self.vector.cross_2d(&other.vector);
        cross.abs() < tolerance
    }

    fn is_perpendicular(&self, other: &Self, tolerance: T) -> bool {
        let dot = self.dot(other);
        dot.abs() < tolerance
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

impl<T: Scalar> Direction2DTrait<T> for Direction2D<T> {
    fn perpendicular(&self) -> Self {
        Self {
            vector: self.vector.perpendicular(),
        }
    }

    fn from_angle(angle: T) -> Self
    where
        Self: Sized,
    {
        // Scalarトレイトの角度メソッドを使用
        Self {
            vector: Vector::new(angle.cos(), angle.sin()),
        }
    }

    fn to_angle(&self) -> T {
        // Scalarトレイトのatan2メソッドを使用
        self.vector.y().atan2(self.vector.x())
    }

    fn x_axis() -> Self
    where
        Self: Sized,
    {
        Self::positive_x()
    }

    fn y_axis() -> Self
    where
        Self: Sized,
    {
        Self::positive_y()
    }
}

impl<T: Scalar> StepCompatible for Direction2D<T> {
    fn to_step_string(&self) -> String {
        format!(
            "DIRECTION(({:.6},{:.6}))",
            self.x().to_f64(),
            self.y().to_f64()
        )
    }

    fn from_step_string(_step_str: &str) -> Result<Self, String>
    where
        Self: Sized,
    {
        Err("STEP parsing not implemented yet".to_string())
    }
}

impl<T: Scalar> std::fmt::Display for Direction2D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Direction2D({:.3}, {:.3})",
            self.x().to_f64(),
            self.y().to_f64()
        )
    }
}

// 型エイリアス（後方互換性確保）
/// f64版の2D Direction（デフォルト）
pub type Direction2DF64 = Direction2D<f64>;

/// f32版の2D Direction（高速演算用）
pub type Direction2DF32 = Direction2D<f32>;
