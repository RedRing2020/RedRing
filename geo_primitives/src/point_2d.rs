//! 2次元点（Point2D）の新実装
//!
//! foundation.rs の基盤トレイトに基づく Point2D の実装
//!
//! ## メソッド分類
//!
//! ### 基本実装 (Core Implementation)
//! - `new()`, `origin()` - 基本的なコンストラクタ
//!
//! ### アクセサメソッド (Accessor Methods)
//! - `x()`, `y()`, `coords()`, `to_tuple()`, `from_tuple()` - 座標データへのアクセス
//!
//! ### 距離・測定関連 (Distance & Measurement)
//! - `distance_to()`, `distance_squared_to()`, `norm()`, `norm_squared()` - 距離計算
//!
//! ### 補間・中点計算 (Interpolation & Midpoint)
//! - `lerp()`, `midpoint()` - 点間の補間操作
//!
//! ### 座標変換 (Geometric Transformations)
//! - `translate()`, `rotate()`, `rotate_around()`, `scale()`, `scale_uniform()`
//! - `reflect_x()`, `reflect_y()`, `reflect_origin()` - 幾何学的変換
//!
//! ### 判定・比較関連 (Predicates & Comparisons)
//! - `is_origin()`, `is_approximately_equal()` - 状態判定
//!
//! ### 型変換・相互変換 (Type Conversions)
//! - `to_vector()`, `vector_to()`, `from_vector()` - Vector2D との相互変換
//!
//! ### 次元拡張 (Dimension Extension)
//! - `to_3d()`, `to_3d_with_z()` - 3次元への拡張
//!
//! この分類により、将来的に各カテゴリを独立したトレイトとして分離したり、
//! opt機能として条件コンパイルしたりすることが容易になります。

use geo_foundation::{
    abstract_types::geometry::foundation::{BasicContainment, GeometryFoundation},
    Scalar,
};

use std::ops::{Add, Mul, Neg, Sub};

/// 2次元空間の点
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D<T: Scalar> {
    x: T,
    y: T,
}

impl<T: Scalar> Point2D<T> {
    // ============================================================================
    // 基本実装 (Core Implementation)
    // ============================================================================

    /// 新しい点を作成
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// 原点を取得
    pub fn origin() -> Self {
        Self::new(T::ZERO, T::ZERO)
    }

    // ============================================================================
    // アクセサメソッド (Accessor Methods)
    // ============================================================================

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

    /// タプルから点を作成
    pub fn from_tuple(coords: (T, T)) -> Self {
        Self::new(coords.0, coords.1)
    }

    // ============================================================================
    // 距離・測定関連 (Distance & Measurement)
    // ============================================================================

    /// 他の点との距離を計算
    pub fn distance_to(&self, other: &Self) -> T {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
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

    // ============================================================================
    // 補間・中点計算 (Interpolation & Midpoint)
    // ============================================================================

    /// 点を別の点に向かって線形補間
    pub fn lerp(&self, other: &Self, t: T) -> Self {
        Self::new(
            self.x + t * (other.x - self.x),
            self.y + t * (other.y - self.y),
        )
    }

    /// 中点を計算
    pub fn midpoint(&self, other: &Self) -> Self {
        Self::new(
            (self.x + other.x) / (T::ONE + T::ONE),
            (self.y + other.y) / (T::ONE + T::ONE),
        )
    }

    // ============================================================================
    // 座標変換 (Geometric Transformations)
    // ============================================================================

    /// 点を指定ベクトルで平行移動
    pub fn translate(&self, dx: T, dy: T) -> Self {
        Self::new(self.x + dx, self.y + dy)
    }

    /// 原点中心で回転（角度はラジアン）
    pub fn rotate(&self, angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self::new(
            self.x * cos_a - self.y * sin_a,
            self.x * sin_a + self.y * cos_a,
        )
    }

    /// 指定点中心で回転
    pub fn rotate_around(&self, center: &Self, angle: T) -> Self {
        let translated = Self::new(self.x - center.x, self.y - center.y);
        let rotated = translated.rotate(angle);
        Self::new(rotated.x + center.x, rotated.y + center.y)
    }

    /// 原点中心でスケーリング
    pub fn scale(&self, scale_x: T, scale_y: T) -> Self {
        Self::new(self.x * scale_x, self.y * scale_y)
    }

    /// 均等スケーリング
    pub fn scale_uniform(&self, scale: T) -> Self {
        Self::new(self.x * scale, self.y * scale)
    }

    /// X軸に対する反転
    pub fn reflect_x(&self) -> Self {
        Self::new(self.x, -self.y)
    }

    /// Y軸に対する反転
    pub fn reflect_y(&self) -> Self {
        Self::new(-self.x, self.y)
    }

    /// 原点に対する反転
    pub fn reflect_origin(&self) -> Self {
        Self::new(-self.x, -self.y)
    }

    // ============================================================================
    // 判定・比較関連 (Predicates & Comparisons)
    // ============================================================================

    /// 点が原点かどうかを判定
    pub fn is_origin(&self, tolerance: T) -> bool {
        self.norm() <= tolerance
    }

    /// 他の点との近似等価判定
    pub fn is_approximately_equal(&self, other: &Self, tolerance: T) -> bool {
        self.distance_to(other) <= tolerance
    }

    // ============================================================================
    // 型変換・相互変換 (Type Conversions)
    // ============================================================================

    /// Point2D を Vector2D に変換（原点からのベクトル）
    pub fn to_vector(&self) -> crate::Vector2D<T> {
        crate::Vector2D::new(self.x, self.y)
    }

    /// 2点間のベクトルを計算（明示的なVector2D作成）
    pub fn vector_to(&self, other: &Self) -> crate::Vector2D<T> {
        crate::Vector2D::new(other.x - self.x, other.y - self.y)
    }

    /// Vector2D から Point2D を作成（ベクトルの終点として）
    pub fn from_vector(vector: &crate::Vector2D<T>) -> Self {
        Self::new(vector.x(), vector.y())
    }

    // ============================================================================
    // 次元拡張 (Dimension Extension)
    // ============================================================================

    /// 3次元空間に拡張（Z=0）
    pub fn to_3d(&self) -> crate::Point3D<T> {
        crate::Point3D::new(self.x, self.y, T::ZERO)
    }

    /// 3次元空間に拡張（指定Z値）
    pub fn to_3d_with_z(&self, z: T) -> crate::Point3D<T> {
        crate::Point3D::new(self.x, self.y, z)
    }
}

// ============================================================================
// Foundation Trait Implementations
// ============================================================================

impl<T: Scalar> GeometryFoundation<T> for Point2D<T> {
    type Point = Point2D<T>;
    type Vector = crate::Vector2D<T>;
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
// 基本演算子実装 (Basic Operator Implementations)
// ============================================================================

/// 加算演算子の実装
impl<T: Scalar> Add for Point2D<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

/// Point2D - Point2D = Point2D (座標成分ごとの減算)
impl<T: Scalar> Sub for Point2D<T> {
    type Output = Point2D<T>;

    fn sub(self, other: Self) -> Self::Output {
        Point2D::new(self.x - other.x, self.y - other.y)
    }
}

/// スカラー乗算演算子の実装
impl<T: Scalar> Mul<T> for Point2D<T> {
    type Output = Self;

    fn mul(self, scalar: T) -> Self::Output {
        Self::new(self.x * scalar, self.y * scalar)
    }
}

/// 負号演算子の実装
impl<T: Scalar> Neg for Point2D<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}

// ============================================================================
// Vector2D 相互演算子実装 (Vector2D Interop Operators)
// ============================================================================

/// Point2D + Vector2D = Point2D (点をベクトル分移動)
impl<T: Scalar> Add<crate::Vector2D<T>> for Point2D<T> {
    type Output = Point2D<T>;

    fn add(self, vector: crate::Vector2D<T>) -> Self::Output {
        Point2D::new(self.x + vector.x(), self.y + vector.y())
    }
}

/// Point2D - Vector2D = Point2D (点をベクトル分逆移動)
impl<T: Scalar> Sub<crate::Vector2D<T>> for Point2D<T> {
    type Output = Point2D<T>;

    fn sub(self, vector: crate::Vector2D<T>) -> Self::Output {
        Point2D::new(self.x - vector.x(), self.y - vector.y())
    }
}
