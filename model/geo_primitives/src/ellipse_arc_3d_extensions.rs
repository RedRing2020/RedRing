//! EllipseArc3D Extension 機能
//!
//! Extension Foundation パターンに基づく EllipseArc3D の拡張実装

use crate::{Arc3D, BBox3D, Circle3D, Ellipse3D, EllipseArc3D, Point3D, Vector3D};
use geo_foundation::{Angle, Scalar};

// ============================================================================
// Extension Methods Implementation
// ============================================================================

impl<T: Scalar> EllipseArc3D<T> {
    // ========================================================================
    // Advanced Construction Methods (Extension)
    // ========================================================================

    /// 楕円の一部分として3D楕円弧を作成（高度構築）
    pub fn from_ellipse_sector(
        center: Point3D<T>,
        semi_major: T,
        semi_minor: T,
        normal: Vector3D<T>,
        major_axis: Vector3D<T>,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Option<Self> {
        let ellipse = Ellipse3D::new(center, semi_major, semi_minor, normal, major_axis)?;
        Some(Self::new(ellipse, start_angle, end_angle))
    }

    /// 円から楕円弧を作成（完全な円弧）
    pub fn from_circle_arc(
        center: Point3D<T>,
        radius: T,
        normal: Vector3D<T>,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Option<Self> {
        let normal_dir = crate::Direction3D::from_vector(normal)?;
        let circle = Circle3D::new(center, normal_dir, radius)?;
        let ellipse = Ellipse3D::from_circle(&circle)?;
        Some(Self::new(ellipse, start_angle, end_angle))
    }

    // ========================================================================
    // Advanced Geometry Methods (Extension)
    // ========================================================================

    /// 楕円弧から円弧に変換（可能な場合）
    pub fn to_arc(&self) -> Option<Arc3D<T>> {
        if !self.is_circular() {
            return None;
        }

        // Arc3D の構築に必要な情報を取得
        let center = self.center();
        let radius = self.semi_major(); // 円の場合、長軸と短軸は同じ
        let normal = self.normal();

        // Arc3D は Direction3D を必要とするので、major_axis から作成
        let _start_direction = self.major_axis_direction();

        // 開始方向は長軸方向を使用
        let start_dir = self.ellipse().major_axis_direction();

        Arc3D::new(
            center,
            radius,
            crate::Direction3D::from_vector(normal.as_vector())?,
            start_dir,
            self.start_angle(),
            self.end_angle(),
        )
    }

    /// より詳細な弧長計算（高精度版）
    pub fn precise_arc_length(&self, segments: usize) -> T {
        if segments == 0 {
            return T::ZERO;
        }

        let mut length = T::ZERO;
        let step = T::ONE / T::from_f64(segments as f64);

        for i in 0..segments {
            let t1 = T::from_f64(i as f64) * step;
            let t2 = t1 + step;

            let p1 = self.point_at_parameter(t1);
            let p2 = self.point_at_parameter(t2);

            length += p1.distance_to(&p2);
        }

        length
    }

    /// パラメータ t での点での曲率を計算
    pub fn curvature_at_parameter(&self, t: T) -> T {
        let angle = self.start_angle().to_radians()
            + (self.end_angle().to_radians() - self.start_angle().to_radians()) * t;

        // 楕円の曲率計算（3D版）
        let a = self.semi_major();
        let b = self.semi_minor();

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

    /// 楕円弧上の点での接線ベクトル（平面内）
    pub fn normal_at_parameter(&self, t: T) -> Vector3D<T> {
        // 基底楕円から接線ベクトルを取得
        let angle = self.start_angle().to_radians()
            + (self.end_angle().to_radians() - self.start_angle().to_radians()) * t;
        let tangent = self.ellipse().tangent_at_parameter(angle);
        let plane_normal = self.normal();

        // 平面内の法線：接線と平面法線の外積
        tangent.cross(&plane_normal.as_vector()).normalize()
    }

    /// 楕円弧上の点での双法線ベクトル
    pub fn binormal_at_parameter(&self, t: T) -> Vector3D<T> {
        let angle = self.start_angle().to_radians()
            + (self.end_angle().to_radians() - self.start_angle().to_radians()) * t;
        let tangent = self.ellipse().tangent_at_parameter(angle);
        let normal = self.normal_at_parameter(t);

        // 双法線：接線と法線の外積
        tangent.cross(&normal).normalize()
    }

    // ========================================================================
    // Advanced Analysis Methods (Extension)
    // ========================================================================

    /// より詳細な境界ボックス計算（高精度版）
    pub fn precise_bounding_box(&self, sample_points: usize) -> BBox3D<T> {
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

        BBox3D::new(
            Point3D::new(min_x, min_y, min_z),
            Point3D::new(max_x, max_y, max_z),
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

    /// 楕円弧の接線ベクトル群を取得
    pub fn tangent_vectors(&self, num_points: usize) -> Vec<Vector3D<T>> {
        let mut tangents = Vec::with_capacity(num_points);

        for i in 0..num_points {
            let t = if num_points <= 1 {
                T::ZERO
            } else {
                T::from_f64(i as f64) / T::from_f64((num_points - 1) as f64)
            };
            let angle = self.start_angle().to_radians()
                + (self.end_angle().to_radians() - self.start_angle().to_radians()) * t;
            tangents.push(self.ellipse().tangent_at_parameter(angle));
        }

        tangents
    }

    // ========================================================================
    // Utility Methods (Extension)
    // ========================================================================

    /// 楕円弧の方向を反転（拡張版）
    pub fn reverse_advanced(&self) -> Self {
        // 基本的な reverse と同じだが、将来的に追加処理を含む可能性
        self.reverse()
    }

    /// 楕円弧の角度を正規化
    pub fn normalize_angles(&self) -> Self {
        let mut start_rad = self.start_angle().to_radians();
        let mut end_rad = self.end_angle().to_radians();

        // 角度を [0, 2π) の範囲に正規化
        while start_rad < T::ZERO {
            start_rad += T::TAU;
        }
        while start_rad >= T::TAU {
            start_rad -= T::TAU;
        }

        while end_rad < T::ZERO {
            end_rad += T::TAU;
        }
        while end_rad >= T::TAU {
            end_rad -= T::TAU;
        }

        // 終了角度が開始角度より小さい場合は一周分を追加
        if end_rad < start_rad {
            end_rad += T::TAU;
        }

        Self::new(
            self.ellipse().clone(),
            Angle::from_radians(start_rad),
            Angle::from_radians(end_rad),
        )
    }

    /// 楕円弧の品質評価
    pub fn quality_score(&self) -> T {
        // 簡易的な品質スコア（0.0-1.0）
        let angle_span = self.angle_span().abs();
        let aspect_ratio = self.semi_major() / self.semi_minor().max(T::EPSILON);

        // 角度スパンが適切で、アスペクト比が極端でない場合に高スコア
        let angle_score = if angle_span <= T::TAU {
            T::ONE
        } else {
            T::TAU / angle_span
        };

        let aspect_score = if aspect_ratio <= T::from_f64(10.0) {
            T::ONE
        } else {
            T::from_f64(10.0) / aspect_ratio
        };

        (angle_score + aspect_score) / (T::ONE + T::ONE) // 平均
    }

    /// 2つの楕円弧の類似度を計算
    pub fn similarity_to(&self, other: &Self, tolerance: T) -> T {
        // 中心点の距離
        let center_distance = self.center().distance_to(&other.center());
        let center_score = if center_distance <= tolerance {
            T::ONE
        } else {
            tolerance / center_distance
        };

        // 軸長の類似度
        let major_diff = (self.semi_major() - other.semi_major()).abs();
        let minor_diff = (self.semi_minor() - other.semi_minor()).abs();
        let axis_score = T::ONE
            - (major_diff + minor_diff) / (self.semi_major() + self.semi_minor()).max(T::EPSILON);

        // 角度範囲の類似度
        let angle_diff = (self.angle_span() - other.angle_span()).abs();
        let angle_score = T::ONE - angle_diff / T::TAU;

        // 重み付き平均
        let weights = [T::from_f64(0.4), T::from_f64(0.3), T::from_f64(0.3)]; // [center, axis, angle]
        center_score * weights[0] + axis_score * weights[1] + angle_score * weights[2]
    }
}
