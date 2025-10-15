//! Circle2D の新しい階層Foundation実装例
//!
//! マーカーInterface + 段階的Foundation実装

use geo_foundation::core::{
    BasicContainment,
    BasicMeasurement,
    BasicParametric,
    // 特化Foundation
    CircularFoundation,
    // 段階的Foundation
    DataAccess,
    // マーカーInterface
    GeometryShape,
    ParametricShape,
    Shape2D,
    SurfaceShape,
};
use geo_foundation::Scalar;

// ============================================================================
// Circle2D の階層Foundation実装
// ============================================================================

impl<T: Scalar> GeometryShape<T> for Circle2D<T> {
    type Point = Point2D<T>;
    type Vector = Vector2D<T>;
    type BBox = BBox2D<T>;
}

// マーカーInterface群（空実装）
impl<T: Scalar> Shape2D<T> for Circle2D<T> {}
impl<T: Scalar> SurfaceShape<T> for Circle2D<T> {}
impl<T: Scalar> ParametricShape<T> for Circle2D<T> {}

// レベル1: データアクセス
impl<T: Scalar> DataAccess<T> for Circle2D<T> {
    fn bounding_box(&self) -> Self::BBox {
        let min_x = self.center.x() - self.radius;
        let min_y = self.center.y() - self.radius;
        let max_x = self.center.x() + self.radius;
        let max_y = self.center.y() + self.radius;
        BBox2D::new(Point2D::new(min_x, min_y), Point2D::new(max_x, max_y))
    }
}

// レベル2: 基本計量
impl<T: Scalar> BasicMeasurement<T> for Circle2D<T> {
    fn area(&self) -> Option<T> {
        Some(T::PI * self.radius * self.radius)
    }

    fn perimeter(&self) -> Option<T> {
        Some(T::TAU * self.radius)
    }
}

// レベル3: 基本包含
impl<T: Scalar> BasicContainment<T> for Circle2D<T> {
    fn contains_point(&self, point: &Self::Point) -> bool {
        let distance = self.center.distance_to(point);
        distance <= self.radius
    }

    fn distance_to_point(&self, point: &Self::Point) -> T {
        let center_distance = self.center.distance_to(point);
        if center_distance <= self.radius {
            T::ZERO
        } else {
            center_distance - self.radius
        }
    }
}

// レベル4: パラメトリック
impl<T: Scalar> BasicParametric<T> for Circle2D<T> {
    fn point_at_parameter(&self, t: T) -> Self::Point {
        let x = self.center.x() + self.radius * t.cos();
        let y = self.center.y() + self.radius * t.sin();
        Point2D::new(x, y)
    }

    fn tangent_at_parameter(&self, t: T) -> Self::Vector {
        let dx = -self.radius * t.sin();
        let dy = self.radius * t.cos();
        Vector2D::new(dx, dy)
    }
}

// 特化Foundation: 円特有の操作
impl<T: Scalar> CircularFoundation<T> for Circle2D<T> {
    fn center(&self) -> Self::Point {
        self.center
    }

    fn radius(&self) -> T {
        self.radius
    }
}
