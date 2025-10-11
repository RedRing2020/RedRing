//! Direction - 最小責務原則による2D方向ベクトルの実装
//!
//! 基本機能 + 必要な拡張トレイトを組み合わせた実装

use crate::geometry2d::Vector;
use crate::traits::StepCompatible;
use analysis::abstract_types::angle::Angle;
use geo_foundation::abstract_types::geometry::common::normalization_operations::Normalizable;
use geo_foundation::abstract_types::geometry::{
    Direction as DirectionTrait, Direction2D as Direction2DTrait, DirectionConstants,
};
use geo_foundation::Scalar;

/// ジェネリック2D方向ベクトル
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Direction<T: Scalar> {
    /// 正規化されたベクトル（内部的にはVectorを使用）
    vector: Vector<T>,
}

impl<T: Scalar> Direction<T> {
    /// 新しいDirectionを作成（内部で正規化）
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

impl<T: Scalar> DirectionTrait<T> for Direction<T> {
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

impl<T: Scalar> Direction2DTrait<T> for Direction<T> {
    fn perpendicular(&self) -> Self {
        Self {
            vector: self.vector.perpendicular(),
        }
    }
}

// DirectionAngular実装（Angle型を使用）- トレイトが存在しないため独自実装
impl<T: Scalar> Direction<T> {
    /// 角度から方向ベクトルを作成
    pub fn from_angle(angle: T) -> Self {
        let angle_obj = Angle::from_radians(angle);
        Self {
            vector: Vector::new(angle_obj.cos(), angle_obj.sin()),
        }
    }

    /// 方向ベクトルから角度を取得
    pub fn to_angle(&self) -> T {
        self.vector.y().atan2(self.vector.x())
    }
}

// DirectionConstants実装
impl<T: Scalar> DirectionConstants<T> for Direction<T> {
    fn positive_x() -> Self {
        Self::from_vector(Vector::unit_x()).unwrap()
    }

    fn positive_y() -> Self {
        Self::from_vector(Vector::unit_y()).unwrap()
    }

    fn negative_x() -> Self {
        Self::from_vector(-Vector::unit_x()).unwrap()
    }

    fn negative_y() -> Self {
        Self::from_vector(-Vector::unit_y()).unwrap()
    }
}

impl<T: Scalar> StepCompatible for Direction<T> {
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

// Direction2DAngular実装を後回しに
/*
impl<T: Scalar> Direction2DAngular<T> for Direction<T> {
    fn from_angle(angle: T) -> Self {
        let cos_val = Angle::from_radians(angle).cos();
        let sin_val = Angle::from_radians(angle).sin();
        Self::from_vector(Vector::new(cos_val, sin_val)).unwrap()
    }

    fn to_angle(&self) -> T {
        self.vector.y().atan2(self.vector.x())
    }
}
*/

impl<T: Scalar> std::fmt::Display for Direction<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Direction({:.3}, {:.3})",
            self.x().to_f64(),
            self.y().to_f64()
        )
    }
}

// 型エイリアス（後方互換性確保）
/// 2D Direction用の型エイリアス
pub type Direction2D<T> = Direction<T>;

/// f64版の2D Direction（デフォルト）
pub type Direction2DF64 = Direction<f64>;

/// f32版の2D Direction（高速演算用）
pub type Direction2DF32 = Direction<f32>;
