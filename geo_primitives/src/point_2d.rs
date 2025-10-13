//! Point2D Core 実装
//!
//! Foundation統一システムに基づくPoint2Dの必須機能のみ

use crate::Vector2D;
use geo_foundation::{
    abstract_types::abstracts::point_traits::Point2D as Point2DTrait,
    abstract_types::geometry::core_foundation::{BasicContainment, CoreFoundation},
    Scalar,
};

use std::ops::{Add, Mul, Neg, Sub};

/// 2次元空間の点
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D<T: Scalar> {
    x: T,
    y: T,
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> Point2D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================

    /// 新しい点を作成
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// 原点を取得
    pub fn origin() -> Self {
        Self::new(T::ZERO, T::ZERO)
    }

    /// タプルから点を作成
    pub fn from_tuple(coords: (T, T)) -> Self {
        Self::new(coords.0, coords.1)
    }

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

    /// X座標を取得
    pub fn x(&self) -> T {
        self.x
    }

    /// Y座標を取得
    pub fn y(&self) -> T {
        self.y
    }

    /// 座標を配列として取得
    pub fn coords(&self) -> [T; 2] {
        [self.x, self.y]
    }

    /// 座標をタプルとして取得
    pub fn to_tuple(&self) -> (T, T) {
        (self.x, self.y)
    }

    // ========================================================================
    // Core Distance Methods (基本距離計算)
    // ========================================================================

    /// 他の点との距離を計算
    pub fn distance_to(&self, other: &Self) -> T {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// 2点間の距離を計算（static版）
    pub fn distance(point1: &Self, point2: &Self) -> T {
        point1.distance_to(point2)
    }

    /// 他の点との距離の二乗を計算（sqrt回避で高速）
    pub fn distance_squared_to(&self, other: &Self) -> T {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    /// 原点からの距離（ノルム）
    pub fn norm(&self) -> T {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// 原点からの距離の二乗
    pub fn norm_squared(&self) -> T {
        self.x * self.x + self.y * self.y
    }
}

// ============================================================================
// Foundation Abstract Trait Implementation
// ============================================================================

impl<T: Scalar> Point2DTrait<T> for Point2D<T> {
    fn x(&self) -> T {
        self.x
    }

    fn y(&self) -> T {
        self.y
    }
}

// ============================================================================
// Core Foundation Trait Implementations
// ============================================================================

impl<T: Scalar> CoreFoundation<T> for Point2D<T> {
    type Point = Point2D<T>;
    type Vector = Vector2D<T>;
    type BBox = crate::BBox2D<T>;

    fn bounding_box(&self) -> Self::BBox {
        crate::BBox2D::from_point(*self)
    }
}

impl<T: Scalar> BasicContainment<T> for Point2D<T> {
    fn contains_point(&self, point: &Self::Point) -> bool {
        *self == *point
    }

    fn on_boundary(&self, point: &Self::Point, tolerance: T) -> bool {
        self.distance_to(point) <= tolerance
    }

    fn distance_to_point(&self, point: &Self::Point) -> T {
        self.distance_to(point)
    }
}

// ============================================================================
// Extension Methods (既存コード互換性のため)
// ============================================================================

impl<T: Scalar> Point2D<T> {
    /// 他の点へのベクトル
    pub fn vector_to(&self, other: &Self) -> Vector2D<T> {
        Vector2D::new(other.x - self.x, other.y - self.y)
    }

    /// 指定点周りの回転（T型角度）
    pub fn rotate_around(&self, center: &Self, angle: T) -> Self {
        let offset = Vector2D::new(self.x - center.x, self.y - center.y);
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let rotated_x = offset.x() * cos_a - offset.y() * sin_a;
        let rotated_y = offset.x() * sin_a + offset.y() * cos_a;
        Point2D::new(center.x + rotated_x, center.y + rotated_y)
    }

    /// 指定点周りの回転（Angle<T>型角度）
    pub fn rotate_around_angle(&self, center: &Self, angle: geo_foundation::Angle<T>) -> Self {
        self.rotate_around(center, angle.to_radians())
    }

    /// 原点周りの回転（T型角度）
    pub fn rotate(&self, angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let rotated_x = self.x * cos_a - self.y * sin_a;
        let rotated_y = self.x * sin_a + self.y * cos_a;
        Point2D::new(rotated_x, rotated_y)
    }

    /// 3D点に変換（Z=0）
    pub fn to_3d(&self) -> crate::Point3D<T> {
        crate::Point3D::new(self.x, self.y, T::ZERO)
    }

    /// 3D点に変換（Z指定）
    pub fn to_3d_with_z(&self, z: T) -> crate::Point3D<T> {
        crate::Point3D::new(self.x, self.y, z)
    }

    /// Vector2Dに変換  
    pub fn to_vector(&self) -> Vector2D<T> {
        Vector2D::new(self.x, self.y)
    }

    /// ベクトルから点を作成
    pub fn from_vector(vector: Vector2D<T>) -> Self {
        Point2D::new(vector.x(), vector.y())
    }

    /// 原点判定
    pub fn is_origin(&self) -> bool {
        self.x.abs() <= T::EPSILON && self.y.abs() <= T::EPSILON
    }

    /// 近似等価判定
    pub fn is_approximately_equal(&self, other: &Self, tolerance: T) -> bool {
        (self.x - other.x).abs() <= tolerance && (self.y - other.y).abs() <= tolerance
    }

    /// 線形補間
    pub fn lerp(&self, other: &Self, t: T) -> Self {
        let one_minus_t = T::ONE - t;
        Point2D::new(
            one_minus_t * self.x + t * other.x,
            one_minus_t * self.y + t * other.y,
        )
    }

    /// 中点計算
    pub fn midpoint(&self, other: &Self) -> Self {
        let half = T::from_f64(0.5);
        self.lerp(other, half)
    }

    /// X軸反射
    pub fn reflect_x(&self) -> Self {
        Point2D::new(-self.x, self.y)
    }

    /// Y軸反射
    pub fn reflect_y(&self) -> Self {
        Point2D::new(self.x, -self.y)
    }

    /// 原点反射
    pub fn reflect_origin(&self) -> Self {
        Point2D::new(-self.x, -self.y)
    }

    /// 平行移動
    pub fn translate(&self, vector: Vector2D<T>) -> Self {
        *self + vector
    }

    /// スケール（非均等）
    pub fn scale(&self, scale_x: T, scale_y: T) -> Self {
        Point2D::new(self.x * scale_x, self.y * scale_y)
    }

    /// 均等スケール
    pub fn scale_uniform(&self, scale: T) -> Self {
        self.scale(scale, scale)
    }
}

// ============================================================================
// 基本演算子実装 (Basic Operator Implementations)
// ============================================================================

impl<T: Scalar> Add<Vector2D<T>> for Point2D<T> {
    type Output = Self;

    fn add(self, rhs: Vector2D<T>) -> Self::Output {
        Self::new(self.x + rhs.x(), self.y + rhs.y())
    }
}

impl<T: Scalar> Sub<Vector2D<T>> for Point2D<T> {
    type Output = Self;

    fn sub(self, rhs: Vector2D<T>) -> Self::Output {
        Self::new(self.x - rhs.x(), self.y - rhs.y())
    }
}

impl<T: Scalar> Sub<Point2D<T>> for Point2D<T> {
    type Output = Vector2D<T>;

    fn sub(self, rhs: Point2D<T>) -> Self::Output {
        Vector2D::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl<T: Scalar> Mul<T> for Point2D<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl<T: Scalar> Neg for Point2D<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}

impl<T: Scalar> Default for Point2D<T> {
    fn default() -> Self {
        Self::origin()
    }
}
