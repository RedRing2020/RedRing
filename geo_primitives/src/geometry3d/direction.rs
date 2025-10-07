/// Direction3D - 3D方向ベクトルの実装
/// 
/// STEP互換のDirection3D実装。常に正規化されたベクトルとして管理され、
/// CAD操作に必要な方向性を持つ要素を表現する。

use crate::geometry3d::Vector3D;
use crate::traits::geometry::{Direction, Direction3D as Direction3DTrait, StepCompatible};
use std::f64::consts::PI;

/// 3D方向ベクトル
/// 
/// 常に長さが1に正規化されたベクトルを表現する。
/// STEPのDIRECTIONエンティティに対応。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Direction3D {
    /// 正規化されたベクトル
    vector: Vector3D,
}

impl Direction3D {
    /// 内部用：正規化されたベクトルから直接作成（事前に正規化済みを前提）
    fn from_normalized_vector(vector: Vector3D) -> Self {
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

    /// Z成分を取得
    pub fn z(&self) -> f64 {
        self.vector.z()
    }

    /// 長さを取得（常に1.0）
    pub fn length(&self) -> f64 {
        self.vector.length()
    }
}

impl Direction for Direction3D {
    type Vector = Vector3D;
    type Scalar = f64;

    fn from_vector(vector: Self::Vector) -> Option<Self> {
        let normalized = vector.normalize()?;
        Some(Self::from_normalized_vector(normalized))
    }

    fn from_components_2d(x: Self::Scalar, y: Self::Scalar) -> Option<Self> {
        let vector = Vector3D::new(x, y, 0.0);
        Self::from_vector(vector)
    }

    fn from_components_3d(x: Self::Scalar, y: Self::Scalar, z: Self::Scalar) -> Option<Self> {
        let vector = Vector3D::new(x, y, z);
        Self::from_vector(vector)
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

impl Direction3DTrait for Direction3D {
    fn cross(&self, other: &Self) -> Self::Vector {
        self.vector.cross(&other.vector)
    }

    fn rotate_around_axis(&self, axis: &Self, angle: Self::Scalar) -> Self {
        // Rodrigues回転公式を使用
        let cos_theta = angle.cos();
        let sin_theta = angle.sin();
        let axis_vec = axis.to_vector();
        let dot_product = self.dot(axis);
        
        let rotated = self.vector * cos_theta
            + axis_vec.cross(&self.vector) * sin_theta
            + axis_vec * dot_product * (1.0 - cos_theta);
            
        Self::from_normalized_vector(rotated)
    }

    fn any_perpendicular(&self) -> Self {
        // 最も小さい成分を持つ軸と外積を取る
        let abs_x = self.x().abs();
        let abs_y = self.y().abs();
        let abs_z = self.z().abs();
        
        let reference = if abs_x <= abs_y && abs_x <= abs_z {
            Vector3D::new(1.0, 0.0, 0.0)
        } else if abs_y <= abs_z {
            Vector3D::new(0.0, 1.0, 0.0)
        } else {
            Vector3D::new(0.0, 0.0, 1.0)
        };
        
        let cross = self.vector.cross(&reference);
        Self::from_normalized_vector(cross)
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
        format!("DIRECTION('',({:.6},{:.6},{:.6}))", self.x(), self.y(), self.z())
    }

    fn from_step_string(_step_str: &str) -> Result<Self, String> {
        // 将来実装予定
        Err("Not implemented yet".to_string())
    }
}

impl std::fmt::Display for Direction3D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Direction3D({:.3}, {:.3}, {:.3})", self.x(), self.y(), self.z())
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