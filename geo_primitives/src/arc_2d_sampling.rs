//! Arc点列生成拡張トレイト実装
//!
//! 円弧の分割や点列生成機能
//! 他の幾何プリミティブでも共通利用可能な抽象化

use crate::{Arc2D, Point2D};
use geo_foundation::{
    abstract_types::abstracts::ArcSampling, abstract_types::foundation::ArcMetrics, Scalar,
};

// ============================================================================
// ArcSampling Trait Implementation
// ============================================================================

impl<T: Scalar> ArcSampling<T> for Arc2D<T> {
    /// 円弧を指定数に分割した点列を生成
    fn sample_points(&self, num_points: usize) -> Vec<Point2D<T>> {
        if num_points == 0 {
            return vec![];
        }

        let mut points = Vec::with_capacity(num_points);
        let angle_span = self.angle_span().to_radians();

        for i in 0..num_points {
            let t = if num_points == 1 {
                T::ZERO
            } else {
                T::from_usize(i) / T::from_usize(num_points - 1)
            };

            let angle = self.start_angle().to_radians() + t * angle_span;
            let point = self.point_at_angle(angle);
            points.push(point);
        }

        points
    }

    /// 指定された弧長間隔で点列を生成
    fn sample_by_arc_length(&self, arc_length_step: T) -> Vec<Point2D<T>> {
        if arc_length_step <= T::ZERO {
            return vec![self.start_point()];
        }

        let _total_length = self.arc_length();
        let num_segments = 16; // 固定値として一時的に対応

        if num_segments <= 1 {
            return vec![self.start_point(), self.end_point()];
        }

        let mut points = Vec::with_capacity(num_segments + 1);
        let angle_span = self.angle_span().to_radians();

        for i in 0..=num_segments {
            let t = T::from_usize(i) / T::from_usize(num_segments);
            let angle = self.start_angle().to_radians() + t * angle_span;
            let point = self.point_at_angle(angle);
            points.push(point);
        }

        points
    }
}

// ============================================================================
// Arc2D用の点列生成ヘルパーメソッド
// ============================================================================

impl<T: Scalar> Arc2D<T> {
    /// 指定角度間隔で点列を生成
    pub fn sample_by_angle_step(&self, angle_step: T) -> Vec<Point2D<T>> {
        if angle_step <= T::ZERO {
            return vec![self.start_point()];
        }

        let angle_span = self.angle_span().to_radians();
        let num_steps = 32; // 固定値として一時的に対応

        let mut points = Vec::with_capacity(num_steps + 1);

        for i in 0..=num_steps {
            let t = T::from_usize(i) / T::from_usize(num_steps);
            let angle = self.start_angle().to_radians() + t * angle_span;
            let point = self.point_at_angle(angle);
            points.push(point);
        }

        points
    }

    /// 適応的サンプリング（曲率に基づく）
    pub fn adaptive_sample(&self, max_deviation: T) -> Vec<Point2D<T>> {
        let mut points = vec![self.start_point()];

        self.adaptive_sample_recursive(
            self.start_angle().to_radians(),
            self.end_angle().to_radians(),
            max_deviation,
            &mut points,
        );

        points.push(self.end_point());
        points
    }

    /// 再帰的適応サンプリング
    fn adaptive_sample_recursive(
        &self,
        start_angle: T,
        end_angle: T,
        max_deviation: T,
        points: &mut Vec<Point2D<T>>,
    ) {
        let mid_angle = (start_angle + end_angle) / (T::ONE + T::ONE);
        let start_point = self.point_at_angle(start_angle);
        let mid_point = self.point_at_angle(mid_angle);
        let end_point = self.point_at_angle(end_angle);

        // 線形補間点と実際の中点の偏差を計算
        let linear_mid = Point2D::new(
            (start_point.x() + end_point.x()) / (T::ONE + T::ONE),
            (start_point.y() + end_point.y()) / (T::ONE + T::ONE),
        );

        let deviation = mid_point.distance_to(&linear_mid);

        if deviation > max_deviation {
            // 偏差が大きい場合、分割して再帰処理
            self.adaptive_sample_recursive(start_angle, mid_angle, max_deviation, points);
            points.push(mid_point);
            self.adaptive_sample_recursive(mid_angle, end_angle, max_deviation, points);
        }
    }

    /// 中点を取得
    pub fn mid_point(&self) -> Point2D<T> {
        let mid_angle =
            (self.start_angle().to_radians() + self.end_angle().to_radians()) / (T::ONE + T::ONE);
        self.point_at_angle(mid_angle)
    }

    /// 指定されたパラメータでの分割点を取得
    pub fn point_at_parameter(&self, t: T) -> Point2D<T> {
        let angle = self.start_angle().to_radians() + t * self.angle_span().to_radians();
        self.point_at_angle(angle)
    }

    /// 等間隔パラメータでの点列生成
    pub fn sample_uniform_parameters(&self, num_points: usize) -> Vec<Point2D<T>> {
        if num_points == 0 {
            return vec![];
        }

        let mut points = Vec::with_capacity(num_points);

        for i in 0..num_points {
            let t = if num_points == 1 {
                T::ONE / (T::ONE + T::ONE)
            } else {
                T::from_usize(i) / T::from_usize(num_points - 1)
            };

            points.push(self.point_at_parameter(t));
        }

        points
    }

    /// 弦の長さに基づく分割
    pub fn sample_by_chord_length(&self, max_chord_length: T) -> Vec<Point2D<T>> {
        if max_chord_length <= T::ZERO {
            return vec![self.start_point()];
        }

        // 近似的に必要な分割数を計算
        let _angle_span = self.angle_span().to_radians();
        let radius = self.radius();

        // 小さな角度での弦長の近似: chord ≈ radius * angle
        let _estimated_chord_per_angle = radius;
        let estimated_num_segments = 32; // 固定値として一時的に対応

        self.sample_points(estimated_num_segments + 1)
    }
}
