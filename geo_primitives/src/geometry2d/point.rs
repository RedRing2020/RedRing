use crate::geometry2d::Vector2D;
use geo_foundation::abstract_types::geometry::{Point as PointTrait, Point2D as Point2DTrait};
use geo_foundation::abstract_types::{Scalar, ToleranceContext, TolerantEq};

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
}

// geo_foundationトレイトの実装
impl<T: Scalar> TolerantEq for Point<T> {
    fn tolerant_eq(&self, other: &Self, context: &ToleranceContext) -> bool {
        let distance = self.distance_to(other);
        distance < T::from_f64(context.tolerance())
    }
}

impl<T: Scalar> PointTrait<2> for Point<T> {
    type Scalar = T;
    type Vector = Vector2D; // TODO: Vector2D<T> に変更が必要

    fn origin() -> Self {
        Self::new(T::ZERO, T::ZERO)
    }

    fn distance_to(&self, other: &Self) -> Self::Scalar {
        Point::distance_to(self, other)
    }

    fn translate(&self, vector: &Self::Vector) -> Self {
        Self::new(self.x + T::from_f64(vector.x()), self.y + T::from_f64(vector.y()))
    }

    fn vector_to(&self, other: &Self) -> Self::Vector {
        Vector2D::new((other.x - self.x).to_f64(), (other.y - self.y).to_f64())
    }

    fn coords(&self) -> [Self::Scalar; 2] {
        [self.x, self.y]
    }
}

impl<T: Scalar> Point2DTrait for Point<T> {
    fn x(&self) -> Self::Scalar {
        self.x
    }

    fn y(&self) -> Self::Scalar {
        self.y
    }

    fn from_components(x: Self::Scalar, y: Self::Scalar) -> Self {
        Self::new(x, y)
    }
}

// 型エイリアス（後方互換性確保）
/// f64版の2D Point（デフォルト）
pub type Point2D = Point<f64>;

/// f32版の2D Point（高速演算用）
pub type Point2Df = Point<f32>;
