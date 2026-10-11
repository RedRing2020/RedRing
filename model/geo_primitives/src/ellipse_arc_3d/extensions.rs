//! EllipseArc3D Extension 機能
//!
//! Extension Foundation パターンに基づく EllipseArc3D の拡張実装

use crate::{EllipseArc3D, Point3D, Vector3D};
use geo_contracts::Scalar;

impl<T: Scalar> EllipseArc3D<T> {
    /// Arc trim-local parameter `t` (`0 <= t <= 1`) に対応する接線ベクトルを取得
    ///
    /// 基底楕円の local angle parameter へ線形写像して接線を評価する。
    pub fn tangent_at_parameter(&self, t: T) -> Vector3D<T> {
        let angle = self.start_angle().to_radians()
            + (self.end_angle().to_radians() - self.start_angle().to_radians()) * t;
        self.ellipse().tangent_at_parameter(angle)
    }

    /// 点から楕円弧への最短距離
    pub fn distance_to_point(&self, point: &Point3D<T>) -> T {
        // 点が角度範囲内にある場合
        if self.contains_point_angle(point) {
            let center = self.center();
            let translated = Vector3D::new(
                point.x() - center.x(),
                point.y() - center.y(),
                point.z() - center.z(),
            );

            let major_axis = self.major_axis_direction().as_vector();
            let minor_axis = self.minor_axis_direction().as_vector();
            let normal_axis = self.normal().as_vector();

            let x_local = translated.dot(&major_axis);
            let y_local = translated.dot(&minor_axis);
            let z_local = translated.dot(&normal_axis);

            return geo_commons::ellipse_3d_distance_to_point(
                x_local,
                y_local,
                z_local,
                self.semi_major(),
                self.semi_minor(),
            );
        }

        // 角度範囲外の場合は端点への距離
        let start_point = self.start_point();
        let end_point = self.end_point();

        let dist_to_start = point.distance_to(&start_point);
        let dist_to_end = point.distance_to(&end_point);

        dist_to_start.min(dist_to_end)
    }

    /// より詳細な境界ボックス計算（高精度版）
    pub fn precise_bounding_box(&self, sample_points: usize) -> geo_core::Aabb3D<T> {
        let mut min_x = T::MAX;
        let mut max_x = T::MIN;
        let mut min_y = T::MAX;
        let mut max_y = T::MIN;
        let mut min_z = T::MAX;
        let mut max_z = T::MIN;

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
            min_z = min_z.min(point.z());
            max_z = max_z.max(point.z());
        }

        geo_core::Aabb3D::new(
            geo_core::Point3D::new(min_x, min_y, min_z),
            geo_core::Point3D::new(max_x, max_y, max_z),
        )
    }

    /// 楕円弧を複数のセグメントに分割
    pub fn subdivide(&self, num_segments: usize) -> Vec<Point3D<T>> {
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
}
