//! Direction3D - Core Implementation
//!
//! 3次元方向ベクトルの基本実装とコンストラクタ、アクセサメソッド

use crate::Vector3D;
use geo_foundation::Scalar;
use std::ops::{Deref, DerefMut, Mul, Neg};

/// 3次元方向ベクトル（正規化済み）
#[derive(Debug, Clone, Copy, PartialEq)]
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

// ============================================================================
// Foundation Pattern Core Traits Implementation
// ============================================================================

use analysis::linalg::Vector3;
use geo_foundation::core::direction_core_traits::{
    Direction3DConstructor, Direction3DMeasure, Direction3DProperties,
};

impl<T: Scalar> Direction3DConstructor<T> for Direction3D<T> {
    fn from_vector(vector: Vector3<T>) -> Option<Self> {
        Direction3D::new(vector.x(), vector.y(), vector.z())
    }

    fn new(x: T, y: T, z: T) -> Option<Self> {
        Direction3D::new(x, y, z)
    }

    fn positive_x() -> Self {
        Direction3D::positive_x()
    }
    fn positive_y() -> Self {
        Direction3D::positive_y()
    }
    fn positive_z() -> Self {
        Direction3D::positive_z()
    }
    fn negative_x() -> Self {
        Direction3D::positive_x().reverse()
    }
    fn negative_y() -> Self {
        Direction3D::positive_y().reverse()
    }
    fn negative_z() -> Self {
        Direction3D::positive_z().reverse()
    }

    fn from_tuple(components: (T, T, T)) -> Option<Self> {
        Self::new(components.0, components.1, components.2)
    }

    fn from_analysis_vector(vector: &Vector3<T>) -> Option<Self> {
        Self::new(vector.x(), vector.y(), vector.z())
    }
}

impl<T: Scalar> Direction3DProperties<T> for Direction3D<T> {
    fn x(&self) -> T {
        self.x()
    }
    fn y(&self) -> T {
        self.y()
    }
    fn z(&self) -> T {
        self.z()
    }
    fn components(&self) -> [T; 3] {
        [self.x(), self.y(), self.z()]
    }
    fn to_tuple(&self) -> (T, T, T) {
        (self.x(), self.y(), self.z())
    }
    fn to_analysis_vector(&self) -> Vector3<T> {
        Vector3::new(self.x(), self.y(), self.z())
    }
    fn as_vector(&self) -> Vector3<T> {
        self.to_analysis_vector()
    }
    fn length(&self) -> T {
        T::ONE
    }
    fn is_normalized(&self) -> bool {
        true
    }
    fn dimension(&self) -> u32 {
        3
    }
}

impl<T: Scalar> Direction3DMeasure<T> for Direction3D<T> {
    fn dot(&self, other: &Self) -> T {
        self.dot(other)
    }

    fn angle_to(&self, other: &Self) -> T {
        let dot_product = self.dot(other).max(-T::ONE).min(T::ONE);
        dot_product.acos()
    }

    fn cross(&self, other: &Self) -> Self {
        let result_vector = self.as_vector().cross(&other.as_vector());
        let vec3d = Vector3D::new(result_vector.x(), result_vector.y(), result_vector.z());
        Direction3D::from_vector(vec3d).unwrap_or_else(|| Direction3D::positive_x())
    }

    fn is_parallel_to(&self, other: &Self) -> bool {
        (self.dot(other).abs() - T::ONE).abs() <= T::EPSILON
    }

    fn is_perpendicular_to(&self, other: &Self) -> bool {
        self.dot(other).abs() <= T::EPSILON
    }

    fn is_same_direction(&self, other: &Self) -> bool {
        self.dot(other) >= T::ONE - T::EPSILON
    }

    fn is_opposite_direction(&self, other: &Self) -> bool {
        self.dot(other) <= -T::ONE + T::EPSILON
    }

    fn reverse(&self) -> Self {
        self.reverse()
    }

    fn rotate_around_axis(&self, axis: &Self, angle: T) -> Self {
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();
        let one_minus_cos = T::ONE - cos_angle;

        let ax = axis.x();
        let ay = axis.y();
        let az = axis.z();
        let x = self.x();
        let y = self.y();
        let z = self.z();

        let new_x = (cos_angle + ax * ax * one_minus_cos) * x
            + (ax * ay * one_minus_cos - az * sin_angle) * y
            + (ax * az * one_minus_cos + ay * sin_angle) * z;

        let new_y = (ay * ax * one_minus_cos + az * sin_angle) * x
            + (cos_angle + ay * ay * one_minus_cos) * y
            + (ay * az * one_minus_cos - ax * sin_angle) * z;

        let new_z = (az * ax * one_minus_cos - ay * sin_angle) * x
            + (az * ay * one_minus_cos + ax * sin_angle) * y
            + (cos_angle + az * az * one_minus_cos) * z;

        Direction3D::new(new_x, new_y, new_z)
            .expect("回転後のベクトルは正規化可能でなければならない")
    }
}
