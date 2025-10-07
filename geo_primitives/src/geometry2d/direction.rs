//! Direction2D - 2D方向ベクトルの実装
//!
//! STEP互換のDirection2D実装。常に正規化されたベクトルとして管理され、
//! CAD操作に必要な方向性を持つ要素を表現する。

use crate::geometry2d::Vector2D;
use geo_foundation::abstract_types::geometry::{Direction, Direction2D as Direction2DTrait, StepCompatible};

/// 2D方向ベクトル
///
/// 常に長さが1に正規化されたベクトルを表現する。
/// STEPのDIRECTIONエンティティに対応。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Direction2D {
    /// 正規化されたベクトル
    vector: Vector2D,
}

impl Direction2D {
    /// 内部用：正規化されたベクトルから直接作成（事前に正規化済みを前提）
    fn from_normalized_vector(vector: Vector2D) -> Self {
        Self { vector }
    }

    /// X成分を取得
    pub fn x(&self) -> f64 {
        self.vector.x()
    }

    /// Y成分を取得
    pub fn y(&self) -> f64 {
        self.vector.y()
    }

    /// 長さを取得（常に1.0）
    pub fn length(&self) -> f64 {
        self.vector.length()
    }
}

impl Direction for Direction2D {
    type Vector = Vector2D;
    type Scalar = f64;

    fn from_vector(vector: Self::Vector) -> Option<Self> {
        let normalized = vector.normalize()?;
        Some(Self::from_normalized_vector(normalized))
    }

    fn from_components_2d(x: Self::Scalar, y: Self::Scalar) -> Option<Self> {
        let vector = Vector2D::new(x, y);
        Self::from_vector(vector)
    }

    fn from_components_3d(_x: Self::Scalar, _y: Self::Scalar, _z: Self::Scalar) -> Option<Self> {
        // 2Dでは3D成分は使用しない
        None
    }

    fn to_vector(&self) -> Self::Vector {
        self.vector
    }

    fn dot(&self, other: &Self) -> Self::Scalar {
        self.vector.dot(&other.vector)
    }

    fn reverse(&self) -> Self {
        Self::from_normalized_vector(-self.vector)
    }

    fn is_parallel(&self, other: &Self, tolerance: Self::Scalar) -> bool {
        let dot = self.dot(other).abs();
        (dot - 1.0).abs() < tolerance
    }

    fn is_perpendicular(&self, other: &Self, tolerance: Self::Scalar) -> bool {
        self.dot(other).abs() < tolerance
    }

    fn is_same_direction(&self, other: &Self, tolerance: Self::Scalar) -> bool {
        let dot = self.dot(other);
        (dot - 1.0).abs() < tolerance
    }

    fn is_opposite_direction(&self, other: &Self, tolerance: Self::Scalar) -> bool {
        let dot = self.dot(other);
        (dot + 1.0).abs() < tolerance
    }
}

impl Direction2DTrait for Direction2D {
    fn perpendicular(&self) -> Self {
        let perp_vector = self.vector.perpendicular();
        Self::from_normalized_vector(perp_vector)
    }

    fn from_angle(angle: Self::Scalar) -> Self {
        let vector = Vector2D::new(angle.cos(), angle.sin());
        Self::from_normalized_vector(vector)
    }

    fn to_angle(&self) -> Self::Scalar {
        self.vector.y().atan2(self.vector.x())
    }

    fn x_axis() -> Self {
        Self::from_normalized_vector(Vector2D::new(1.0, 0.0))
    }

    fn y_axis() -> Self {
        Self::from_normalized_vector(Vector2D::new(0.0, 1.0))
    }
}

impl StepCompatible for Direction2D {
    fn to_step_string(&self) -> String {
        format!("DIRECTION('',({:.6},{:.6}))", self.x(), self.y())
    }

    fn from_step_string(_step_str: &str) -> Result<Self, String> {
        // 将来実装予定
        Err("Not implemented yet".to_string())
    }
}

impl std::fmt::Display for Direction2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Direction2D({:.3}, {:.3})", self.x(), self.y())
    }
}

// 便利な関数
impl Direction2D {
    /// 正のX軸方向を取得
    pub fn positive_x() -> Self {
        Self::from_normalized_vector(Vector2D::new(1.0, 0.0))
    }

    /// 正のY軸方向を取得
    pub fn positive_y() -> Self {
        Self::from_normalized_vector(Vector2D::new(0.0, 1.0))
    }

    /// 負のX軸方向を取得
    pub fn negative_x() -> Self {
        Self::from_normalized_vector(Vector2D::new(-1.0, 0.0))
    }

    /// 負のY軸方向を取得
    pub fn negative_y() -> Self {
        Self::from_normalized_vector(Vector2D::new(0.0, -1.0))
    }
}
