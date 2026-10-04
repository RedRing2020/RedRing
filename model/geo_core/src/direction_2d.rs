//! Direction2D Core 実装
//!
//! Foundation統一システムに基づくDirection2Dの必須機能のみ
//! 拡張機能は direction_2d_extensions.rs を参照

use crate::Vector2D;
use analysis::linalg::vector::Vector2;
use geo_contracts::default_angle_tolerance;
use geo_contracts::geometry::core::direction_traits::{
    Direction2DConstructor, Direction2DProperties, Direction2DRelation, Direction2DTransform,
};
use geo_contracts::Scalar;
use std::ops::{Deref, DerefMut};

/// 2次元方向ベクトル（正規化済み）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Direction2D<T: Scalar> {
    vector: Vector2D<T>,
}

impl<T: Scalar> Direction2D<T> {
    fn is_parallel_with_angle_tolerance(&self, other: &Self, angle_tolerance: T) -> bool {
        let angle = self.angle_to(other);
        angle <= angle_tolerance || (T::PI - angle).abs() <= angle_tolerance
    }

    fn is_perpendicular_with_angle_tolerance(&self, other: &Self, angle_tolerance: T) -> bool {
        let right_angle = T::PI / (T::ONE + T::ONE);
        (self.angle_to(other) - right_angle).abs() <= angle_tolerance
    }

    /// ベクトルから方向を作成（正規化）
    pub fn from_vector(vector: Vector2D<T>) -> Option<Self> {
        let len = vector.length();
        if len <= T::ZERO {
            None
        } else {
            let normalized = vector.normalize();
            Some(Self { vector: normalized })
        }
    }

    /// X、Y成分から方向を作成
    pub fn new(x: T, y: T) -> Option<Self> {
        Self::from_vector(Vector2D::new(x, y))
    }

    /// X軸正方向の単位ベクトル
    pub fn positive_x() -> Self {
        Self {
            vector: Vector2D::unit_x(),
        }
    }

    /// Y軸正方向の単位ベクトル
    pub fn positive_y() -> Self {
        Self {
            vector: Vector2D::unit_y(),
        }
    }

    /// X軸負方向の単位ベクトル
    pub fn negative_x() -> Self {
        Self {
            vector: -Vector2D::unit_x(),
        }
    }

    /// Y軸負方向の単位ベクトル
    pub fn negative_y() -> Self {
        Self {
            vector: -Vector2D::unit_y(),
        }
    }

    /// X成分を取得
    pub fn x(&self) -> T {
        self.vector.x()
    }

    /// Y成分を取得
    pub fn y(&self) -> T {
        self.vector.y()
    }

    /// 内部ベクトルを取得
    pub fn as_vector(&self) -> Vector2D<T> {
        self.vector
    }

    /// 他の方向との内積を計算
    pub fn dot(&self, other: &Self) -> T {
        self.vector.dot(&other.vector)
    }

    /// 他の方向との角度を計算（0 ≤ angle ≤ π）
    pub fn angle_to(&self, other: &Self) -> T {
        let dot_product = self.dot(other);
        // dot_product を [-1, 1] にクランプ
        let clamped = dot_product.max(-T::ONE).min(T::ONE);
        clamped.acos()
    }

    /// 90度回転（反時計回り）
    pub fn rotate_90(&self) -> Self {
        Self {
            vector: self.vector.rotate_90(),
        }
    }

    /// 180度回転（反転）
    pub fn reverse(&self) -> Self {
        Self {
            vector: -self.vector,
        }
    }

    /// 他の方向と平行かどうかを判定
    pub fn is_parallel_to(&self, other: &Self) -> bool {
        self.is_parallel_with_angle_tolerance(other, default_angle_tolerance::<T>())
    }

    /// 他の方向と垂直かどうかを判定
    pub fn is_perpendicular_to(&self, other: &Self) -> bool {
        self.is_perpendicular_with_angle_tolerance(other, default_angle_tolerance::<T>())
    }
}

/// Direction2D Constructor Trait Implementation
impl<T: Scalar> Direction2DConstructor<T> for Direction2D<T> {
    fn from_vector(vector: Vector2<T>) -> Option<Self> {
        let geo_vector = Vector2D::new(vector.x(), vector.y());
        Self::from_vector(geo_vector)
    }

    fn new(x: T, y: T) -> Option<Self> {
        Self::new(x, y)
    }

    fn positive_x() -> Self {
        Self::positive_x()
    }

    fn positive_y() -> Self {
        Self::positive_y()
    }

    fn negative_x() -> Self {
        Self::negative_x()
    }

    fn negative_y() -> Self {
        Self::negative_y()
    }

    fn from_tuple(components: (T, T)) -> Option<Self> {
        Self::new(components.0, components.1)
    }

    fn from_analysis_vector(vector: &Vector2<T>) -> Option<Self> {
        let geo_vector = Vector2D::new(vector.x(), vector.y());
        Self::from_vector(geo_vector)
    }
}

/// Direction2D Properties Trait Implementation
impl<T: Scalar> Direction2DProperties<T> for Direction2D<T> {
    fn x(&self) -> T {
        self.vector.x()
    }

    fn y(&self) -> T {
        self.vector.y()
    }

    fn components(&self) -> [T; 2] {
        [self.vector.x(), self.vector.y()]
    }

    fn to_tuple(&self) -> (T, T) {
        (self.vector.x(), self.vector.y())
    }

    fn to_analysis_vector(&self) -> Vector2<T> {
        Vector2::new(self.vector.x(), self.vector.y())
    }

    fn as_vector(&self) -> Vector2<T> {
        Vector2::new(self.vector.x(), self.vector.y())
    }

    fn length(&self) -> T {
        T::ONE // Direction は常に正規化済み
    }

    fn is_normalized(&self) -> bool {
        true // Direction は常に正規化済み
    }

    fn dimension(&self) -> u32 {
        2
    }
}

/// Direction2D Relation Trait Implementation
impl<T: Scalar> Direction2DRelation<T> for Direction2D<T> {
    fn dot(&self, other: &Self) -> T {
        self.vector.dot(&other.vector)
    }

    fn angle_to(&self, other: &Self) -> T {
        let dot_product = self.vector.dot(&other.vector);
        let clamped = dot_product.max(-T::ONE).min(T::ONE);
        clamped.acos()
    }

    fn is_parallel_to(&self, other: &Self) -> bool {
        self.is_parallel_with_angle_tolerance(other, default_angle_tolerance::<T>())
    }

    fn is_perpendicular_to(&self, other: &Self) -> bool {
        self.is_perpendicular_with_angle_tolerance(other, default_angle_tolerance::<T>())
    }

    fn is_same_direction(&self, other: &Self) -> bool {
        self.angle_to(other) <= default_angle_tolerance::<T>()
    }

    fn is_opposite_direction(&self, other: &Self) -> bool {
        (self.angle_to(other) - T::PI).abs() <= default_angle_tolerance::<T>()
    }
}

/// Direction2D Transform Trait Implementation
impl<T: Scalar> Direction2DTransform<T> for Direction2D<T> {
    fn reverse(&self) -> Self {
        Self::from_vector(Vector2D::new(-self.x(), -self.y())).unwrap()
    }

    fn rotate_90(&self) -> Self {
        Self::from_vector(Vector2D::new(-self.y(), self.x())).unwrap()
    }

    fn rotate(&self, angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let new_x = self.x() * cos_a - self.y() * sin_a;
        let new_y = self.x() * sin_a + self.y() * cos_a;
        Self::from_vector(Vector2D::new(new_x, new_y)).unwrap()
    }
}

/// Direction2DをVector2Dとして扱えるようにする
impl<T: Scalar> Deref for Direction2D<T> {
    type Target = Vector2D<T>;

    fn deref(&self) -> &Self::Target {
        &self.vector
    }
}

/// Direction2DをVector2Dとして変更可能にする（注意：正規化が破られる可能性）
impl<T: Scalar> DerefMut for Direction2D<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vector
    }
}

/// Vector2Dからの変換（失敗する可能性があるためOptionを返す）
impl<T: Scalar> TryFrom<Vector2D<T>> for Direction2D<T> {
    type Error = ();

    fn try_from(vector: Vector2D<T>) -> Result<Self, Self::Error> {
        Self::from_vector(vector).ok_or(())
    }
}

/// スカラー乗算（Direction2D * T = Vector2D）
impl<T: Scalar> std::ops::Mul<T> for Direction2D<T> {
    type Output = Vector2D<T>;

    fn mul(self, scalar: T) -> Self::Output {
        self.vector * scalar
    }
}

/// 否定（-Direction2D = Direction2D）
impl<T: Scalar> std::ops::Neg for Direction2D<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            vector: -self.vector,
        }
    }
}
