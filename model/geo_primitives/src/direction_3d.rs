//! Direction3D - Core Implementation
//!
//! 3次元方向ベクトルの基本実装とコンストラクタ、アクセサメソッド

use crate::Vector3D;
use geo_foundation::Scalar;
use std::ops::{Deref, DerefMut, Mul, Neg};

/// 3次元方向ベクトル（正規化済み）
pub struct Direction3D<T: Scalar> {
    vector: Vector3D<T>,
}

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

    /// `to_vector` は既存コードで使われる命名なのでエイリアスを提供
    /// 内部のベクトルをコピーして返す（無駄な再計算を避ける）
    pub fn to_vector(&self) -> Vector3D<T> {
        self.vector
    }

    // ========================================================================
    // Core Basic Operations
    // ========================================================================

    /// 他の方向との内積を計算
    pub fn dot(&self, other: &Self) -> T {
        self.vector.dot(&other.vector)
    }

    /// 180度回転（反転）
    pub fn reverse(&self) -> Self {
        Self {
            vector: -self.vector,
        }
    }

    /// 長さを取得（常に1.0）
    pub fn length(&self) -> T {
        T::ONE
    }

    /// 正規化（既に正規化済みなのでselfを返す）
    pub fn normalize(&self) -> Self {
        *self
    }

    /// 符号反転（reverse()と同じ）
    pub fn negate(&self) -> Self {
        self.reverse()
    }
}

// ============================================================================
// Core Trait Implementations
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

/// Vector3Dからの変換（失敗する可能性があるためOptionを返す）
impl<T: Scalar> TryFrom<Vector3D<T>> for Direction3D<T> {
    type Error = ();

    fn try_from(vector: Vector3D<T>) -> Result<Self, Self::Error> {
        Self::from_vector(vector).ok_or(())
    }
}

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
