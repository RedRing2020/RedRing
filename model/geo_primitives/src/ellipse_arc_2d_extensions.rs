//! EllipseArc2D Extension 機能
//!
//! Extension Foundation パターンに基づく EllipseArc2D の拡張実装

use crate::{BBox2D, Ellipse2D, EllipseArc2D, Point2D, Vector2D};
use geo_foundation::{Angle, Scalar};
// use geo_foundation::core::arc_traits::Arc2D; // 未使用のため一時的にコメントアウト

// ============================================================================
// Extension Methods Implementation
// ============================================================================

impl<T: Scalar> EllipseArc2D<T> {
    // ========================================================================
    // Advanced Construction Methods (Extension)
    // ========================================================================

    /// 楕円の一部分として楕円弧を作成（高度構築）
    pub fn from_ellipse_sector(
        center: Point2D<T>,
        semi_major: T,
        semi_minor: T,
        rotation: T,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Option<Self> {
        let ellipse = Ellipse2D::new(center, semi_major, semi_minor, rotation)?;
        Some(Self::new(ellipse, start_angle, end_angle))
    }

    // ========================================================================
    // Advanced Geometry Methods (Extension)
    // ========================================================================

    /// 楕円弧が円弧かどうかを判定
    pub fn is_circular_arc(&self, tolerance: T) -> bool {
        self.ellipse().is_circle(tolerance)
    }

    /// 円弧に変換（可能な場合）
    // 一時的にコメントアウト: Arc2Dはトレイトなので具象型が必要
    // pub fn to_arc(&self) -> Option<Arc2D<T>> {
    //     let circle = self.ellipse().to_circle()?;
    //     Arc2D::new(circle, self.start_angle(), self.end_angle())
    // }
    // ========================================================================
    // Transformation Methods (Extension)
    // ========================================================================
    /// 中心を移動
    pub fn translate(&self, offset: Vector2D<T>) -> Self {
        let new_center = Point2D::new(
            self.ellipse().center().x() + offset.x(),
            self.ellipse().center().y() + offset.y(),
        );

        let new_ellipse = Ellipse2D::new(
            new_center,
            self.ellipse().semi_major(),
            self.ellipse().semi_minor(),
            self.ellipse().rotation(),
        )
        .unwrap();

        Self::new(new_ellipse, self.start_angle(), self.end_angle())
    }

    /// 回転
    pub fn rotate(&self, _angle: T, _pivot: Point2D<T>) -> Self {
        // TODO: 楕円の回転変換を実装
        // 現在は簡易実装
        *self
    }

    /// スケール
    pub fn scale(&self, _scale_x: T, _scale_y: T, _origin: Point2D<T>) -> Self {
        // TODO: 楕円のスケール変換を実装
        // 現在は簡易実装
        *self
    }

    // ========================================================================
    // Utility Methods (Extension)
    // ========================================================================

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

    // ========================================================================
    // Advanced Analysis Methods (Extension)
    // ========================================================================

    /// より詳細な境界ボックス計算（高精度版）
    pub fn precise_bounding_box(&self, sample_points: usize) -> BBox2D<T> {
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

        BBox2D::new(Point2D::new(min_x, min_y), Point2D::new(max_x, max_y))
    }

    /// 楕円弧の曲率を計算
    pub fn curvature_at_parameter(&self, t: T) -> T {
        // 楕円弧における曲率の近似計算
        let angle = self.start_angle().to_radians()
            + (self.end_angle().to_radians() - self.start_angle().to_radians()) * t;

        let a = self.ellipse().semi_major();
        let b = self.ellipse().semi_minor();

        let cos_theta = angle.cos();
        let sin_theta = angle.sin();

        let numerator = a * b;
        let denominator =
            (a * a * sin_theta * sin_theta + b * b * cos_theta * cos_theta).powf(T::from_f64(1.5));

        if denominator.abs() > T::EPSILON {
            numerator / denominator
        } else {
            T::ZERO
        }
    }

    /// 楕円弧上の点での法線ベクトル
    pub fn normal_at_parameter(&self, t: T) -> Vector2D<T> {
        let tangent = self.tangent_at_parameter(t);
        Vector2D::new(-tangent.y(), tangent.x()).normalize()
    }
}
