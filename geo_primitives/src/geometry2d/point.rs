use crate::geometry2d::Vector2D;
use geo_foundation::abstract_types::geometry::{Point as PointTrait, Point2D as Point2DTrait};
use geo_foundation::abstract_types::{ToleranceContext, TolerantEq};

/// A 2D point represented by x and y coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    /// Creates a new Point.
    pub fn new(x: f64, y: f64) -> Point {
        Point { x, y }
    }

    pub fn origin() -> Self {
        Self::new(0.0, 0.0)
    }

    pub fn x(&self) -> f64 {
        self.x
    }
    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn distance_to(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

// geo_foundationトレイトの実装
impl TolerantEq for Point {
    fn tolerant_eq(&self, other: &Self, context: &ToleranceContext) -> bool {
        let distance = self.distance_to(other);
        distance < context.tolerance()
    }
}

impl PointTrait<2> for Point {
    type Scalar = f64;
    type Vector = Vector2D;

    fn origin() -> Self {
        Self::new(0.0, 0.0)
    }

    fn distance_to(&self, other: &Self) -> Self::Scalar {
        Point::distance_to(self, other)
    }

    fn translate(&self, vector: &Self::Vector) -> Self {
        Self::new(self.x + vector.x(), self.y + vector.y())
    }

    fn vector_to(&self, other: &Self) -> Self::Vector {
        Vector2D::new(other.x - self.x, other.y - self.y)
    }

    fn coords(&self) -> [Self::Scalar; 2] {
        [self.x, self.y]
    }
}

impl Point2DTrait for Point {
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
