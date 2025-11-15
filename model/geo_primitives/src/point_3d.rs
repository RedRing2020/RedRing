//! Point3D Core 実装
//!
//! Foundation統一システムに基づくPoint3Dの必須機能のみ

use crate::Vector3D;
use geo_foundation::{core::point_traits, Scalar};

/// 3次元空間の点
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3D<T: Scalar> {
    x: T,
    y: T,
    z: T,
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> Point3D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================

    /// 新しい点を作成
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }

    /// 原点を取得
    pub fn origin() -> Self {
        Self::new(T::ZERO, T::ZERO, T::ZERO)
    }

    /// タプルから点を作成
    pub fn from_tuple(coords: (T, T, T)) -> Self {
        Self::new(coords.0, coords.1, coords.2)
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

    /// Z座標を取得
    pub fn z(&self) -> T {
        self.z
    }

    /// 座標を配列として取得
    pub fn coords(&self) -> [T; 3] {
        [self.x, self.y, self.z]
    }

    // ========================================================================
    // Core Calculation Methods
    // ========================================================================

    /// 他の点との距離を計算
    pub fn distance_to(&self, other: &Self) -> T {
        self.distance_squared_to(other).sqrt()
    }

    /// 他の点との距離の二乗を計算（sqrt回避で高速）
    pub fn distance_squared_to(&self, other: &Self) -> T {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }

    /// 原点からの距離（ノルム）
    pub fn norm(&self) -> T {
        self.norm_squared().sqrt()
    }

    /// 原点からの距離の二乗
    pub fn norm_squared(&self) -> T {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// 点が境界上（許容誤差内）にあるかを判定
    pub fn on_boundary(&self, point: &Self, tolerance: T) -> bool {
        self.distance_to(point) <= tolerance
    }

    /// 点が自分自身と一致するかを判定
    pub fn contains_point(&self, point: &Self) -> bool {
        self == point
    }

    // ========================================================================
    // Conversion Methods
    // ========================================================================
}

// ============================================================================
// Operator Implementations
// ============================================================================

// Point - Point = Vector (2点間のベクトル)
impl<T: Scalar> std::ops::Sub for Point3D<T> {
    type Output = Vector3D<T>;

    fn sub(self, other: Self) -> Self::Output {
        Vector3D::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

// Point + Vector = Point (点をベクトル分移動)
impl<T: Scalar> std::ops::Add<Vector3D<T>> for Point3D<T> {
    type Output = Point3D<T>;

    fn add(self, vector: Vector3D<T>) -> Self::Output {
        Point3D::new(
            self.x + vector.x(),
            self.y + vector.y(),
            self.z + vector.z(),
        )
    }
}

// Point - Vector = Point (点をベクトル分逆移動)
impl<T: Scalar> std::ops::Sub<Vector3D<T>> for Point3D<T> {
    type Output = Point3D<T>;

    fn sub(self, vector: Vector3D<T>) -> Self::Output {
        Point3D::new(
            self.x - vector.x(),
            self.y - vector.y(),
            self.z - vector.z(),
        )
    }
}

// 基本機能のみに集中 - 複雑な変換は将来のextensionトレイトで実装予定

// ============================================================================
// geo_foundation abstracts trait implementations
// ============================================================================

/// geo_foundation::core::Point2D<T> トレイト実装
impl<T: Scalar> point_traits::Point2D<T> for Point3D<T> {
    fn x(&self) -> T {
        self.x
    }

    fn y(&self) -> T {
        self.y
    }
}

/// geo_foundation::core::Point3D<T> トレイト実装
impl<T: Scalar> point_traits::Point3D<T> for Point3D<T> {
    fn z(&self) -> T {
        self.z
    }
}

// ============================================================================
// From trait implementations
// ============================================================================

/// タプルからの変換
impl<T: Scalar> From<(T, T, T)> for Point3D<T> {
    fn from(tuple: (T, T, T)) -> Self {
        Self::new(tuple.0, tuple.1, tuple.2)
    }
}
