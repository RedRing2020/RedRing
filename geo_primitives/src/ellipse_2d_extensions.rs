//! Ellipse2D拡張メソッド
//!
//! Core Foundation パターンに基づく Ellipse2D の拡張機能
//! 基本機能は ellipse_2d.rs を参照

use crate::{Circle2D, Ellipse2D, Point2D, Vector2D};
use geo_foundation::{tolerance_migration::DefaultTolerances, Scalar};

impl<T: Scalar> Ellipse2D<T> {
    // ========================================================================
    // Extension Construction Methods
    // ========================================================================

    /// 単位楕円を作成（中心が原点、a=1、b指定）
    pub fn unit_ellipse(semi_minor: T) -> Option<Self> {
        Self::axis_aligned(Point2D::origin(), T::ONE, semi_minor)
    }

    /// 5点から楕円を作成（楕円フィッティング）
    /// 実装は簡略化: とりあえず境界ボックスベースの近似
    pub fn from_five_points(points: [Point2D<T>; 5]) -> Option<Self> {
        // 点群の境界ボックスを計算
        let min_x = points
            .iter()
            .map(|p| p.x())
            .fold(points[0].x(), |min, x| min.min(x));
        let max_x = points
            .iter()
            .map(|p| p.x())
            .fold(points[0].x(), |max, x| max.max(x));
        let min_y = points
            .iter()
            .map(|p| p.y())
            .fold(points[0].y(), |min, y| min.min(y));
        let max_y = points
            .iter()
            .map(|p| p.y())
            .fold(points[0].y(), |max, y| max.max(y));

        let center = Point2D::new(
            (min_x + max_x) / (T::ONE + T::ONE),
            (min_y + max_y) / (T::ONE + T::ONE),
        );

        let width = max_x - min_x;
        let height = max_y - min_y;

        if width > height {
            Self::axis_aligned(
                center,
                width / (T::ONE + T::ONE),
                height / (T::ONE + T::ONE),
            )
        } else {
            Self::axis_aligned(
                center,
                height / (T::ONE + T::ONE),
                width / (T::ONE + T::ONE),
            )
        }
    }

    // ========================================================================
    // Extension Predicate Methods
    // ========================================================================

    /// 離心率を計算
    pub fn eccentricity(&self) -> T {
        if self.semi_major_axis() <= T::ZERO {
            return T::ZERO;
        }

        let a = self.semi_major_axis();
        let b = self.semi_minor_axis();
        let c_squared = a * a - b * b;
        if c_squared <= T::ZERO {
            T::ZERO // 円の場合
        } else {
            (c_squared / (a * a)).sqrt()
        }
    }

    /// 楕円が円かどうかを判定
    pub fn is_circle(&self) -> bool {
        let tolerance = DefaultTolerances::distance::<T>();
        (self.semi_major_axis() - self.semi_minor_axis()).abs() <= tolerance
    }

    /// 楕円が退化しているか（軸の長さが0に近い）を判定
    pub fn is_degenerate(&self) -> bool {
        let tolerance = DefaultTolerances::distance::<T>();
        self.semi_major_axis() <= tolerance || self.semi_minor_axis() <= tolerance
    }

    /// 点が楕円境界上にあるかを判定
    pub fn on_boundary(&self, point: &Point2D<T>) -> bool {
        let tolerance = DefaultTolerances::distance::<T>();
        let distance = self.distance_to_point(point);
        distance <= tolerance
    }

    // ========================================================================
    // Extension Geometric Methods
    // ========================================================================

    /// 指定角度での点を取得
    pub fn point_at_angle(&self, angle: T) -> Point2D<T> {
        self.point_at_parameter(angle)
    }

    // ========================================================================
    // Extension Transformation Methods
    // ========================================================================

    /// 楕円を平行移動
    pub fn translate(&self, offset: &Vector2D<T>) -> Self {
        Self::new(
            self.center() + *offset,
            self.semi_major_axis(),
            self.semi_minor_axis(),
            self.rotation(),
        )
        .unwrap() // Core で作成された楕円なので常に有効
    }

    /// 楕円を拡大縮小
    pub fn scale(&self, factor: T) -> Option<Self> {
        if factor > T::ZERO {
            Self::new(
                self.center(),
                self.semi_major_axis() * factor,
                self.semi_minor_axis() * factor,
                self.rotation(),
            )
        } else {
            None
        }
    }

    /// 楕円を回転
    pub fn rotate(&self, angle: T) -> Self {
        Self::new(
            self.center(),
            self.semi_major_axis(),
            self.semi_minor_axis(),
            self.rotation() + angle,
        )
        .unwrap() // Core で作成された楕円なので常に有効
    }

    /// 原点中心での回転
    pub fn rotate_around_origin(&self, angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        let new_center = Point2D::new(
            self.center().x() * cos_a - self.center().y() * sin_a,
            self.center().x() * sin_a + self.center().y() * cos_a,
        );

        Self::new(
            new_center,
            self.semi_major_axis(),
            self.semi_minor_axis(),
            self.rotation() + angle,
        )
        .unwrap() // Core で作成された楕円なので常に有効
    }

    // ========================================================================
    // Extension Type Conversion Methods
    // ========================================================================

    /// 楕円を円に変換（可能な場合）
    pub fn to_circle(&self) -> Option<Circle2D<T>> {
        if self.is_circle() {
            Circle2D::new(self.center(), self.semi_major_axis())
        } else {
            None
        }
    }

    // TODO: 3D楕円への変換は将来の実装予定
    // pub fn to_3d(&self) -> crate::geometry3d::ellipse::Ellipse3D<T> {
    //     // Z=0平面上の楕円として3D楕円を作成
    //     // 実装は3D楕円の実装後に追加予定
    // }

    // ========================================================================
    // Foundation Integration Methods (Extension)
    // ========================================================================

    /// Foundation Transform統合での高度な楕円変換
    pub fn foundation_scale_from_point(&self, point: Point2D<T>, factor: T) -> Option<Self> {
        if factor <= T::ZERO {
            return None;
        }

        // アフィン変換: center' = point + (center - point) * factor
        let offset = Vector2D::from_points(point, self.center());
        let new_center = point + (offset * factor);
        let new_major = self.semi_major_axis() * factor;
        let new_minor = self.semi_minor_axis() * factor;

        Self::new(new_center, new_major, new_minor, self.rotation())
    }

    /// Foundation Collision統合での楕円同士の衝突解決
    pub fn foundation_resolve_collision(&self, other: &Self) -> Option<(Self, Self)> {
        let center_distance = self.center().distance_to(&other.center());
        let self_avg_radius = (self.semi_major_axis() + self.semi_minor_axis()) / (T::ONE + T::ONE);
        let other_avg_radius = (other.semi_major_axis() + other.semi_minor_axis()) / (T::ONE + T::ONE);
        let required_distance = self_avg_radius + other_avg_radius;

        if center_distance >= required_distance {
            return None; // 衝突していない
        }

        if center_distance < T::EPSILON {
            // 同心楕円の場合は少しずらす
            let offset = Vector2D::new(required_distance / (T::ONE + T::ONE), T::ZERO);
            return Some((
                self.translate(&offset.negate()),
                other.translate(&offset),
            ));
        }

        let direction = Vector2D::from_points(self.center(), other.center()).normalize();
        let separation = required_distance - center_distance;
        let half_separation = separation / (T::ONE + T::ONE);

        let self_offset = direction * (-half_separation);
        let other_offset = direction * half_separation;

        Some((
            self.translate(&self_offset),
            other.translate(&other_offset),
        ))
    }

    /// Foundation Intersection統合での楕円群の重心計算
    pub fn foundation_weighted_center(&self, others: &[Self], weights: &[T]) -> Option<Point2D<T>> {
        if others.len() != weights.len() {
            return None;
        }

        let mut total_weight = T::ZERO;
        let mut weighted_x = T::ZERO;
        let mut weighted_y = T::ZERO;

        // 自分の重み（面積に基づく）
        let self_weight = self.area();
        total_weight = total_weight + self_weight;
        weighted_x = weighted_x + (self.center().x() * self_weight);
        weighted_y = weighted_y + (self.center().y() * self_weight);

        // 他の楕円の重み付き中心
        for (ellipse, &weight) in others.iter().zip(weights) {
            total_weight = total_weight + weight;
            weighted_x = weighted_x + (ellipse.center().x() * weight);
            weighted_y = weighted_y + (ellipse.center().y() * weight);
        }

        if total_weight > T::EPSILON {
            Some(Point2D::new(
                weighted_x / total_weight,
                weighted_y / total_weight,
            ))
        } else {
            None
        }
    }

    /// Foundation系統での楕円の軸変換
    pub fn foundation_swap_axes(&self) -> Option<Self> {
        // 長軸と短軸を入れ替える（90度回転も含む）
        Self::new(
            self.center(),
            self.semi_minor_axis(), // 短軸が新しい長軸
            self.semi_major_axis(), // 長軸が新しい短軸
            self.rotation() + T::PI / (T::ONE + T::ONE), // 90度回転
        )
    }

    /// Foundation系統での楕円の離心率調整
    pub fn foundation_adjust_eccentricity(&self, target_eccentricity: T) -> Option<Self> {
        if target_eccentricity < T::ZERO || target_eccentricity >= T::ONE {
            return None; // 無効な離心率
        }

        // e = sqrt(1 - (b/a)^2) から b = a * sqrt(1 - e^2) を計算
        let new_minor = self.semi_major_axis() * (T::ONE - target_eccentricity * target_eccentricity).sqrt();
        
        Self::new(
            self.center(),
            self.semi_major_axis(),
            new_minor,
            self.rotation(),
        )
    }
}
