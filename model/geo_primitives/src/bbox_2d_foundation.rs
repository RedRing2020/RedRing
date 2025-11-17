//! BBox2D Foundation トレイト実装
//!
//! BBox2D の Foundation Pattern 実装
//! Core Traits (Constructor/Properties/Measure) を実装

use crate::{BBox2D, Point2D};
use geo_foundation::{BBox2DConstructor, BBox2DMeasure, BBox2DProperties, Scalar};

// ============================================================================
// Constructor トレイト実装
// ============================================================================

impl<T: Scalar> BBox2DConstructor<T> for BBox2D<T> {
    fn new(min: (T, T), max: (T, T)) -> Self {
        // 座標を正規化（min <= max）
        let actual_min = Point2D::new(min.0.min(max.0), min.1.min(max.1));
        let actual_max = Point2D::new(min.0.max(max.0), min.1.max(max.1));
        Self::new(actual_min, actual_max)
    }

    fn from_point(point: (T, T)) -> Self {
        let p = Point2D::new(point.0, point.1);
        Self::from_point(p)
    }

    fn from_points(points: &[(T, T)]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let point_vec: Vec<Point2D<T>> = points.iter().map(|&(x, y)| Point2D::new(x, y)).collect();

        Self::from_points(&point_vec)
    }

    fn from_center_size(center: (T, T), width: T, height: T) -> Self {
        let center_point = Point2D::new(center.0, center.1);
        Self::from_center_size(center_point, width, height)
    }

    fn unit_box() -> Self {
        let half = T::ONE / (T::ONE + T::ONE);
        Self::new(Point2D::new(-half, -half), Point2D::new(half, half))
    }

    fn empty() -> Self {
        // 無効な境界ボックス（min > max）
        Self::new(Point2D::new(T::ONE, T::ONE), Point2D::new(T::ZERO, T::ZERO))
    }
}

// ============================================================================
// Properties トレイト実装
// ============================================================================

impl<T: Scalar> BBox2DProperties<T> for BBox2D<T> {
    fn min(&self) -> (T, T) {
        let min_point = BBox2D::min(self);
        (min_point.x(), min_point.y())
    }

    fn max(&self) -> (T, T) {
        let max_point = BBox2D::max(self);
        (max_point.x(), max_point.y())
    }

    fn center(&self) -> (T, T) {
        let center_point = BBox2D::center(self);
        (center_point.x(), center_point.y())
    }

    fn width(&self) -> T {
        BBox2D::width(self)
    }

    fn height(&self) -> T {
        BBox2D::height(self)
    }

    fn size(&self) -> (T, T) {
        BBox2D::size(self)
    }

    fn is_empty(&self) -> bool {
        !BBox2D::is_valid(self)
    }

    fn is_valid(&self) -> bool {
        BBox2D::is_valid(self)
    }

    fn is_point(&self) -> bool {
        BBox2D::is_point(self, T::EPSILON)
    }

    fn corners(&self) -> [(T, T); 4] {
        let min_point = BBox2D::min(self);
        let max_point = BBox2D::max(self);
        [
            (min_point.x(), min_point.y()), // 左下
            (max_point.x(), min_point.y()), // 右下
            (max_point.x(), max_point.y()), // 右上
            (min_point.x(), max_point.y()), // 左上
        ]
    }

    fn dimension(&self) -> u32 {
        2
    }
}

// ============================================================================
// Measure トレイト実装
// ============================================================================

impl<T: Scalar> BBox2DMeasure<T> for BBox2D<T> {
    fn area(&self) -> T {
        BBox2D::area(self)
    }

    fn perimeter(&self) -> T {
        BBox2D::perimeter(self)
    }

    fn diagonal_length(&self) -> T {
        let width = BBox2D::width(self);
        let height = BBox2D::height(self);
        (width * width + height * height).sqrt()
    }

    fn contains_point(&self, point: (T, T)) -> bool {
        let p = Point2D::new(point.0, point.1);
        BBox2D::contains_point(self, &p)
    }

    fn point_on_boundary(&self, point: (T, T)) -> bool {
        let p = Point2D::new(point.0, point.1);
        BBox2D::on_boundary(self, &p, T::EPSILON)
    }

    fn intersects(&self, other: &Self) -> bool {
        let self_min = <Self as BBox2DProperties<T>>::min(self);
        let self_max = <Self as BBox2DProperties<T>>::max(self);
        let other_min = <Self as BBox2DProperties<T>>::min(other);
        let other_max = <Self as BBox2DProperties<T>>::max(other);

        self_min.0 <= other_max.0
            && self_max.0 >= other_min.0
            && self_min.1 <= other_max.1
            && self_max.1 >= other_min.1
    }

    fn contains_bbox(&self, other: &Self) -> bool {
        BBox2D::contains_bbox(self, other)
    }

    fn intersection(&self, other: &Self) -> Option<Self> {
        if !<Self as BBox2DMeasure<T>>::intersects(self, other) {
            return None;
        }

        let self_min = <Self as BBox2DProperties<T>>::min(self);
        let self_max = <Self as BBox2DProperties<T>>::max(self);
        let other_min = <Self as BBox2DProperties<T>>::min(other);
        let other_max = <Self as BBox2DProperties<T>>::max(other);

        let min_x = self_min.0.max(other_min.0);
        let min_y = self_min.1.max(other_min.1);
        let max_x = self_max.0.min(other_max.0);
        let max_y = self_max.1.min(other_max.1);

        Some(<Self as BBox2DConstructor<T>>::new(
            (min_x, min_y),
            (max_x, max_y),
        ))
    }

    fn union(&self, other: &Self) -> Self {
        let self_min = <Self as BBox2DProperties<T>>::min(self);
        let self_max = <Self as BBox2DProperties<T>>::max(self);
        let other_min = <Self as BBox2DProperties<T>>::min(other);
        let other_max = <Self as BBox2DProperties<T>>::max(other);

        let min_x = self_min.0.min(other_min.0);
        let min_y = self_min.1.min(other_min.1);
        let max_x = self_max.0.max(other_max.0);
        let max_y = self_max.1.max(other_max.1);

        <Self as BBox2DConstructor<T>>::new((min_x, min_y), (max_x, max_y))
    }

    fn distance_to_point(&self, point: (T, T)) -> T {
        let p = Point2D::new(point.0, point.1);
        BBox2D::distance_to_point(self, &p)
    }

    fn closest_point_to(&self, point: (T, T)) -> (T, T) {
        let min = <Self as BBox2DProperties<T>>::min(self);
        let max = <Self as BBox2DProperties<T>>::max(self);
        let clamped_x = point.0.max(min.0).min(max.0);
        let clamped_y = point.1.max(min.1).min(max.1);
        (clamped_x, clamped_y)
    }

    fn distance_to_bbox(&self, other: &Self) -> T {
        if <Self as BBox2DMeasure<T>>::intersects(self, other) {
            return T::ZERO;
        }

        let self_center = <Self as BBox2DProperties<T>>::center(self);
        let other_center = <Self as BBox2DProperties<T>>::center(other);

        // 簡易実装: 中心間距離から各境界ボックスのサイズを引く
        let dx = (self_center.0 - other_center.0).abs()
            - (<Self as BBox2DProperties<T>>::width(self)
                + <Self as BBox2DProperties<T>>::width(other))
                / (T::ONE + T::ONE);
        let dy = (self_center.1 - other_center.1).abs()
            - (<Self as BBox2DProperties<T>>::height(self)
                + <Self as BBox2DProperties<T>>::height(other))
                / (T::ONE + T::ONE);

        let dx = dx.max(T::ZERO);
        let dy = dy.max(T::ZERO);

        (dx * dx + dy * dy).sqrt()
    }

    fn expand(&self, margin: T) -> Self {
        let min = <Self as BBox2DProperties<T>>::min(self);
        let max = <Self as BBox2DProperties<T>>::max(self);
        <Self as BBox2DConstructor<T>>::new(
            (min.0 - margin, min.1 - margin),
            (max.0 + margin, max.1 + margin),
        )
    }

    fn shrink(&self, margin: T) -> Option<Self> {
        let min = <Self as BBox2DProperties<T>>::min(self);
        let max = <Self as BBox2DProperties<T>>::max(self);
        let new_min = (min.0 + margin, min.1 + margin);
        let new_max = (max.0 - margin, max.1 - margin);

        if new_min.0 <= new_max.0 && new_min.1 <= new_max.1 {
            Some(<Self as BBox2DConstructor<T>>::new(new_min, new_max))
        } else {
            None
        }
    }

    fn extend_to_include_point(&self, point: (T, T)) -> Self {
        let min = <Self as BBox2DProperties<T>>::min(self);
        let max = <Self as BBox2DProperties<T>>::max(self);
        <Self as BBox2DConstructor<T>>::new(
            (min.0.min(point.0), min.1.min(point.1)),
            (max.0.max(point.0), max.1.max(point.1)),
        )
    }

    fn extend_to_include_bbox(&self, other: &Self) -> Self {
        <Self as BBox2DMeasure<T>>::union(self, other)
    }
}
