//! EllipseArc2D Extension 機能
//!
//! Extension Foundation パターンに基づく EllipseArc2D の拡張実装

use crate::{Ellipse2D, EllipseArc2D, Point2D, Vector2D};
use geo_contracts::{Ellipse2DProperties, Scalar};

impl<T: Scalar> EllipseArc2D<T> {
    /// 中心を移動
    pub fn translate(&self, offset: Vector2D<T>) -> Self {
        let center = Point2D::from_tuple(Ellipse2DProperties::center(self.ellipse()));
        let new_center = Point2D::new(center.x() + offset.x(), center.y() + offset.y());

        let new_ellipse = Ellipse2D::new(
            new_center,
            Ellipse2DProperties::semi_major_axis(self.ellipse()),
            Ellipse2DProperties::semi_minor_axis(self.ellipse()),
            self.ellipse().rotation(),
        )
        .unwrap();

        Self::new(new_ellipse, self.start_angle(), self.end_angle())
    }

    /// 楕円弧の方向を反転
    pub fn reverse(&self) -> Self {
        Self::new(*self.ellipse(), self.end_angle(), self.start_angle())
    }

    /// 楕円弧を複数のセグメントに分割
    pub fn subdivide(&self, num_segments: usize) -> Vec<Point2D<T>> {
        let mut points = Vec::with_capacity(num_segments + 1);

        for i in 0..=num_segments {
            let t = if num_segments == 0 {
                T::ZERO
            } else {
                T::from_f64(i as f64) / T::from_f64(num_segments as f64)
            };
            points.push(self.point_at_parameter(t));
        }

        points
    }

    /// より詳細な境界ボックス計算（高精度版）
    pub fn precise_bounding_box(&self, sample_points: usize) -> geo_core::Aabb2D<T> {
        use geo_core::Point2D;
        let mut min_x = T::MAX;
        let mut max_x = T::MIN;
        let mut min_y = T::MAX;
        let mut max_y = T::MIN;

        // サンプリング点での詳細計算
        for i in 0..=sample_points {
            let t = if sample_points == 0 {
                T::ZERO
            } else {
                T::from_f64(i as f64) / T::from_f64(sample_points as f64)
            };
            let point = self.point_at_parameter(t);

            min_x = min_x.min(point.x());
            max_x = max_x.max(point.x());
            min_y = min_y.min(point.y());
            max_y = max_y.max(point.y());
        }

        geo_core::Aabb2D::new(Point2D::new(min_x, min_y), Point2D::new(max_x, max_y))
    }
}
