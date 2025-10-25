//! Vector3D拡張機能の実装
//!
//! Core Foundation パターンに基づく Vector3D の拡張機能
//! transform機能は vector_3d_transform.rs を参照

use crate::Vector3D;
use geo_foundation::Scalar;

// ============================================================================
// Vector3D Extensions（その他の拡張機能）
// ============================================================================

impl<T: Scalar> Vector3D<T> {
    // ========================================================================
    // Advanced Vector Operations
    // ========================================================================

    /// ベクトルの線形補間
    ///
    /// # 引数
    /// * `other` - 補間先のベクトル
    /// * `t` - 補間パラメータ（0.0 = self, 1.0 = other）
    ///
    /// # 戻り値
    /// 補間されたベクトル
    pub fn lerp(&self, other: &Self, t: T) -> Self {
        *self + (*other - *self) * t
    }

    /// ベクトルの球面線形補間（SLERP）
    ///
    /// 単位ベクトル同士の補間に適用
    ///
    /// # 引数
    /// * `other` - 補間先のベクトル
    /// * `t` - 補間パラメータ（0.0 = self, 1.0 = other）
    ///
    /// # 戻り値
    /// 球面補間されたベクトル
    pub fn slerp(&self, other: &Self, t: T) -> Self {
        let dot = self.dot(other).clamp(-T::ONE, T::ONE);
        let angle = dot.acos();

        if angle.abs() < T::EPSILON {
            // ベクトルがほぼ同じ方向の場合は線形補間
            return self.lerp(other, t);
        }

        let sin_angle = angle.sin();
        let factor1 = ((T::ONE - t) * angle).sin() / sin_angle;
        let factor2 = (t * angle).sin() / sin_angle;

        *self * factor1 + *other * factor2
    }

    /// ベクトル間の角度を計算
    ///
    /// # 引数
    /// * `other` - 他のベクトル
    ///
    /// # 戻り値
    /// ベクトル間の角度（ラジアン）
    pub fn angle_between(&self, other: &Self) -> T {
        let dot = self.dot(other);
        let lengths = self.length() * other.length();

        if lengths <= T::ZERO {
            return T::ZERO;
        }

        (dot / lengths).clamp(-T::ONE, T::ONE).acos()
    }

    /// ベクトルを指定された長さにスケール
    ///
    /// # 引数
    /// * `new_length` - 新しい長さ
    ///
    /// # 戻り値
    /// スケールされたベクトル（ゼロベクトルの場合は元のまま）
    pub fn with_length(&self, new_length: T) -> Self {
        let current_length = self.length();
        if current_length <= T::ZERO {
            *self
        } else {
            *self * (new_length / current_length)
        }
    }

    /// ベクトルの投影
    ///
    /// selfをotherベクトルに投影したベクトルを返す
    ///
    /// # 引数
    /// * `other` - 投影先のベクトル
    ///
    /// # 戻り値
    /// 投影されたベクトル
    pub fn project_onto(&self, other: &Self) -> Self {
        let other_length_sq = other.length_squared();
        if other_length_sq <= T::ZERO {
            Self::zero()
        } else {
            *other * (self.dot(other) / other_length_sq)
        }
    }

    /// ベクトルの反射
    ///
    /// 指定された法線ベクトルで反射したベクトルを返す
    ///
    /// # 引数
    /// * `normal` - 反射面の法線ベクトル（正規化済みであることを想定）
    ///
    /// # 戻り値
    /// 反射されたベクトル
    pub fn reflect(&self, normal: &Self) -> Self {
        let two = T::ONE + T::ONE;
        *self - *normal * (two * self.dot(normal))
    }

    // ========================================================================
    // Geometric Analysis Methods
    // ========================================================================

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
