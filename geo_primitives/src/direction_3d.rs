//! Direction3D Core 実装
//!
//! Foundation統一システムに基づくDirection3Dの必須機能のみ
//! 拡張機能は direction_3d_extensions.rs を参照

use crate::Vector3D;
use geo_foundation::{core::direction_traits, Scalar};
use std::ops::{Deref, DerefMut, Mul, Neg};

/// 3次元方向ベクトル（正規化済み）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Direction3D<T: Scalar> {
    vector: Vector3D<T>,
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> Direction3D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================

    /// ベクトルから方向を作成（正規化）
    pub fn from_vector(vector: Vector3D<T>) -> Option<Self> {
        let len = vector.length();
        if len <= T::ZERO {
            None
        } else {
            let normalized = vector.normalize();
            Some(Self { vector: normalized })
        }
    }

    /// X、Y、Z成分から方向を作成
    pub fn new(x: T, y: T, z: T) -> Option<Self> {
        Self::from_vector(Vector3D::new(x, y, z))
    }

    /// X軸正方向の単位ベクトル
    pub fn positive_x() -> Self {
        Self {
            vector: Vector3D::unit_x(),
        }
    }

    /// Y軸正方向の単位ベクトル
    pub fn positive_y() -> Self {
        Self {
            vector: Vector3D::unit_y(),
        }
    }

    /// Z軸正方向の単位ベクトル
    pub fn positive_z() -> Self {
        Self {
            vector: Vector3D::unit_z(),
        }
    }

    /// X軸負方向の単位ベクトル
    pub fn negative_x() -> Self {
        Self {
            vector: -Vector3D::unit_x(),
        }
    }

    /// Y軸負方向の単位ベクトル
    pub fn negative_y() -> Self {
        Self {
            vector: -Vector3D::unit_y(),
        }
    }

    /// Z軸負方向の単位ベクトル
    pub fn negative_z() -> Self {
        Self {
            vector: -Vector3D::unit_z(),
        }
    }

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

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
    pub fn as_vector(&self) -> Vector3D<T> {
        self.vector
    }

    // ========================================================================
    // Core Calculation Methods
    // ========================================================================

    /// 他の方向との内積を計算
    pub fn dot(&self, other: &Self) -> T {
        self.vector.dot(&other.vector)
    }

    /// 3D外積を計算（結果も正規化される）
    pub fn cross(&self, other: &Self) -> Self {
        let cross_result = self.vector.cross(&other.vector);
        Self::from_vector(cross_result).unwrap_or_else(|| Self::positive_x())
    }

    /// 180度回転（反転）
    pub fn reverse(&self) -> Self {
        Self {
            vector: -self.vector,
        }
    }

    /// 他の方向と平行かどうかを判定
    pub fn is_parallel_to(&self, other: &Self) -> bool {
        self.vector.is_parallel(&other.vector)
    }

    /// 他の方向と垂直かどうかを判定
    pub fn is_perpendicular_to(&self, other: &Self) -> bool {
        let dot_product = self.dot(other).abs();
        dot_product <= T::ZERO + T::EPSILON
    }

    /// 長さ（常に1.0、正規化済みのため）
    pub fn length(&self) -> T {
        T::ONE
    }

    /// ベクトル正規化（既に正規化済みなので自身を返す）
    pub fn normalize(&self) -> Self {
        *self
    }

    /// 否定（-演算子のメソッド版）
    pub fn negate(&self) -> Self {
        -*self
    }
}

// ============================================================================
// geo_foundation abstracts trait implementations
// ============================================================================

/// geo_foundation::core::Direction2D<T> トレイト実装
impl<T: Scalar> direction_traits::Direction2D<T> for Direction3D<T> {
    type Vector = Vector3D<T>;

    fn x(&self) -> T {
        self.x()
    }

    fn y(&self) -> T {
        self.y()
    }

    fn as_vector(&self) -> Self::Vector {
        self.as_vector()
    }
}

/// geo_foundation::core::Direction3D<T> トレイト実装
impl<T: Scalar> direction_traits::Direction3D<T> for Direction3D<T> {
    fn z(&self) -> T {
        self.z()
    }
}

/// geo_foundation::core::DirectionRelations<T> トレイト実装
impl<T: Scalar> direction_traits::DirectionRelations<T> for Direction3D<T> {
    fn is_parallel_to(&self, other: &Self) -> bool {
        self.is_parallel_to(other)
    }

    fn is_perpendicular_to(&self, other: &Self) -> bool {
        self.is_perpendicular_to(other)
    }

    fn dot(&self, other: &Self) -> T {
        self.dot(other)
    }
}

// ============================================================================
// Deref implementations - Vector3Dメソッドを透過的に使用可能
// ============================================================================

/// Direction3DをVector3Dとして扱えるようにする
impl<T: Scalar> Deref for Direction3D<T> {
    type Target = Vector3D<T>;

    fn deref(&self) -> &Self::Target {
        &self.vector
    }
}

/// Direction3DをVector3Dとして変更可能にする（注意：正規化が破られる可能性）
impl<T: Scalar> DerefMut for Direction3D<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vector
    }
}

// ============================================================================
// From trait implementations
// ============================================================================

/// Vector3Dからの変換（失敗する可能性があるためOptionを返す）
impl<T: Scalar> TryFrom<Vector3D<T>> for Direction3D<T> {
    type Error = ();

    fn try_from(vector: Vector3D<T>) -> Result<Self, Self::Error> {
        Self::from_vector(vector).ok_or(())
    }
}

// ============================================================================
// Operator Implementations
// ============================================================================

/// スカラー倍（Vector3Dを返す）
impl<T: Scalar> Mul<T> for Direction3D<T> {
    type Output = Vector3D<T>;

    fn mul(self, scalar: T) -> Self::Output {
        self.vector * scalar
    }
}

/// 符号反転（Negation）
impl<T: Scalar> Neg for Direction3D<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            vector: -self.vector,
        }
    }
}
