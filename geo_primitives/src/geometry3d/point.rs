use geo_foundation::abstract_types::{TolerantEq, ToleranceContext};
use geo_foundation::abstract_types::geometry::{Point as PointTrait, Point3D as Point3DTrait};
use crate::geometry3d::Vector3D;

/// f64ベース3D点
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    x: f64,
    y: f64,
    z: f64,
}

impl Point {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn origin() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn x(&self) -> f64 { self.x }
    pub fn y(&self) -> f64 { self.y }
    pub fn z(&self) -> f64 { self.z }

    pub fn distance_to(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// 2D点への投影（Z座標を破棄）
    pub fn to_point2d(&self) -> crate::geometry2d::Point2D {
        crate::geometry2d::Point2D::new(self.x, self.y)
    }

    /// XY平面上での距離計算
    pub fn xy_distance_to(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl Default for Point {
    fn default() -> Self {
        Self::origin()
    }
}

// 算術演算の実装
impl std::ops::Add<crate::geometry3d::Vector3D> for Point {
    type Output = Point;

    fn add(self, vector: crate::geometry3d::Vector3D) -> Self::Output {
        Point::new(self.x + vector.x(), self.y + vector.y(), self.z + vector.z())
    }
}

impl std::ops::Sub<Point> for Point {
    type Output = crate::geometry3d::Vector3D;

    fn sub(self, other: Point) -> Self::Output {
        crate::geometry3d::Vector3D::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

// geo_foundationトレイトの実装
impl TolerantEq for Point {
    fn tolerant_eq(&self, other: &Self, context: &ToleranceContext) -> bool {
        let distance = self.distance_to(other);
        distance < context.tolerance()
    }
}

impl PointTrait<3> for Point {
    type Scalar = f64;
    type Vector = Vector3D;

    fn origin() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    fn distance_to(&self, other: &Self) -> Self::Scalar {
        Point::distance_to(self, other)
    }

    fn translate(&self, vector: &Self::Vector) -> Self {
        Self::new(self.x + vector.x(), self.y + vector.y(), self.z + vector.z())
    }

    fn vector_to(&self, other: &Self) -> Self::Vector {
        Vector3D::new(other.x - self.x, other.y - self.y, other.z - self.z)
    }

    fn coords(&self) -> [Self::Scalar; 3] {
        [self.x, self.y, self.z]
    }
}

impl Point3DTrait for Point {
    fn x(&self) -> Self::Scalar {
        self.x
    }

    fn y(&self) -> Self::Scalar {
        self.y
    }

    fn z(&self) -> Self::Scalar {
        self.z
    }

    fn from_components(x: Self::Scalar, y: Self::Scalar, z: Self::Scalar) -> Self {
        Self::new(x, y, z)
    }
}

// Displayは別クレートで実装

// テストはunit_tests/point3d_tests.rsに移動
