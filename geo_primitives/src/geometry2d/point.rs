use crate::geometry2d::Vector; // ジェネリック Vector を使用
use geo_foundation::abstract_types::geometry::{Point as PointTrait, Point2D as Point2DTrait};
use geo_foundation::abstract_types::geometry::common::distance_operations::DistanceCalculation;
use geo_foundation::{Scalar, ToleranceContext, TolerantEq};

/// A 2D point represented by x and y coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point<T: Scalar> {
    pub x: T,
    pub y: T,
}

impl<T: Scalar> Point<T> {
    /// Creates a new Point.
    pub fn new(x: T, y: T) -> Point<T> {
        Point { x, y }
    }

    pub fn origin() -> Self {
        Self::new(T::ZERO, T::ZERO)
    }

    pub fn x(&self) -> T {
        self.x
    }
    pub fn y(&self) -> T {
        self.y
    }

    pub fn distance_to(&self, other: &Self) -> T {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Point間のベクトルを取得
    pub fn vector_to(&self, other: &Self) -> Vector<T> {
        Vector::new(other.x - self.x, other.y - self.y)
    }

    /// Vectorで平行移動
    pub fn translate(&self, vector: &Vector<T>) -> Self {
        Self::new(self.x + vector.x(), self.y + vector.y())
    }

    /// スカラー倍（原点からの拡大・縮小）
    pub fn scale(&self, scalar: T) -> Self {
        Self::new(self.x * scalar, self.y * scalar)
    }

    /// 原点を中心とした回転（角度はラジアン）
    pub fn rotate(&self, angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self::new(
            self.x * cos_a - self.y * sin_a,
            self.x * sin_a + self.y * cos_a,
        )
    }

    /// 中点を計算
    pub fn midpoint(&self, other: &Self) -> Self {
        Self::new(
            (self.x + other.x) / (T::ONE + T::ONE),
            (self.y + other.y) / (T::ONE + T::ONE),
        )
    }
}

// 演算子オーバーロード
impl<T: Scalar> std::ops::Add<Vector<T>> for Point<T> {
    type Output = Self;

    fn add(self, vector: Vector<T>) -> Self::Output {
        self.translate(&vector)
    }
}

impl<T: Scalar> std::ops::Sub<Vector<T>> for Point<T> {
    type Output = Self;

    fn sub(self, vector: Vector<T>) -> Self::Output {
        Self::new(self.x - vector.x(), self.y - vector.y())
    }
}

impl<T: Scalar> std::ops::Sub for Point<T> {
    type Output = Vector<T>;

    fn sub(self, other: Self) -> Self::Output {
        // self - other = Vector from other to self
        other.vector_to(&self)
    }
}

impl<T: Scalar> std::ops::Mul<T> for Point<T> {
    type Output = Self;

    fn mul(self, scalar: T) -> Self::Output {
        self.scale(scalar)
    }
}

impl<T: Scalar> std::ops::Div<T> for Point<T> {
    type Output = Self;

    fn div(self, scalar: T) -> Self::Output {
        Self::new(self.x / scalar, self.y / scalar)
    }
}

// geo_foundationトレイトの実装
impl<T: Scalar> TolerantEq for Point<T> {
    fn tolerant_eq(&self, other: &Self, context: &ToleranceContext) -> bool {
        let distance = self.distance_to(other);
        distance < T::from_f64(context.tolerance())
    }
}

impl<T: Scalar> PointTrait<T, 2> for Point<T> {
    type Vector = Vector<T>;

    fn origin() -> Self {
        Self::new(T::ZERO, T::ZERO)
    }

    fn distance_to(&self, other: &Self) -> T {
        self.distance_to(other)
    }

    fn translate(&self, vector: &Self::Vector) -> Self {
        Self::new(self.x + vector.x(), self.y + vector.y())
    }

    fn vector_to(&self, other: &Self) -> Self::Vector {
        Vector::new(other.x - self.x, other.y - self.y)
    }

    fn coords(&self) -> [T; 2] {
        [self.x, self.y]
    }
}

// Point2DTrait の実装
impl<T: Scalar> Point2DTrait<T> for Point<T> {
    fn x(&self) -> T {
        self.x
    }

    fn y(&self) -> T {
        self.y
    }

    fn from_components(x: T, y: T) -> Self {
        Self::new(x, y)
    }
}

// 統合された距離計算トレイトの実装
impl<T: Scalar> DistanceCalculation<T, Point<T>> for Point<T> {
    type DistanceResult = T;

    fn distance_to(&self, target: &Point<T>) -> Self::DistanceResult {
        let dx = self.x - target.x;
        let dy = self.y - target.y;
        (dx * dx + dy * dy).sqrt()
    }

    fn distance_squared_to(&self, target: &Point<T>) -> T {
        let dx = self.x - target.x;
        let dy = self.y - target.y;
        dx * dx + dy * dy
    }

    fn extract_scalar_distance(&self, result: Self::DistanceResult) -> T {
        result
    }
}

// 型エイリアス（後方互換性確保）
/// 2D Point用の型エイリアス
pub type Point2D<T> = Point<T>;

/// f64版の2D Point（デフォルト）
pub type Point2DF64 = Point<f64>;

/// f32版の2D Point（高速演算用）
pub type Point2DF32 = Point<f32>;
