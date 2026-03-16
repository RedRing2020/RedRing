//! Circle2D 拡張機能
//!
//! Extension Foundation パターンに基づく Circle2D の拡張実装

use crate::{Circle2D, Point2D, Vector2D};
use geo_contracts::Scalar;

// ============================================================================
// Extension Methods Implementation
// ============================================================================

impl<T: Scalar> Circle2D<T> {
    // ========================================================================
    // Advanced Construction Methods (Extension)
    // ========================================================================

    /// 3点から円を作成（外接円）
    pub fn from_three_points(p1: Point2D<T>, p2: Point2D<T>, p3: Point2D<T>) -> Option<Self> {
        // 3点が一直線上にある場合は円を作れない
        let v1 = Vector2D::from_points(p1, p2);
        let v2 = Vector2D::from_points(p1, p3);

        let cross = v1.cross(&v2);
        if cross.abs() <= T::EPSILON {
            return None; // 共線点
        }

        // 外接円の中心計算（複素な幾何計算）
        let d1 = p1.x() * p1.x() + p1.y() * p1.y();
        let d2 = p2.x() * p2.x() + p2.y() * p2.y();
        let d3 = p3.x() * p3.x() + p3.y() * p3.y();

        let two = T::ONE + T::ONE;
        let aux1 = d1 * (p2.y() - p3.y()) + d2 * (p3.y() - p1.y()) + d3 * (p1.y() - p2.y());
        let aux2 = d1 * (p3.x() - p2.x()) + d2 * (p1.x() - p3.x()) + d3 * (p2.x() - p1.x());
        let div = two
            * (p1.x() * (p2.y() - p3.y())
                + p2.x() * (p3.y() - p1.y())
                + p3.x() * (p1.y() - p2.y()));

        if div.abs() <= T::EPSILON {
            return None;
        }

        let center = Point2D::new(aux1 / div, aux2 / div);
        let radius = center.distance_to(&p1);

        Self::new(center, radius)
    }

    /// 単位円を作成（原点中心、半径1）
    pub fn unit_circle() -> Self {
        Self::new(Point2D::new(T::ZERO, T::ZERO), T::ONE).unwrap()
    }

    // ========================================================================
    // Convenience Methods (Extension)
    // ========================================================================

    /// 指定角度での点を取得（ラジアン）
    pub fn point_at_angle(&self, angle: T) -> Point2D<T> {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Point2D::new(
            self.center_internal().x() + self.radius_internal() * cos_a,
            self.center_internal().y() + self.radius_internal() * sin_a,
        )
    }

    // ========================================================================
    // Transformation Methods (Extension)
    // ========================================================================

    /// 円を指定倍率でスケール
    pub fn scale(&self, factor: T) -> Option<Self> {
        if factor > T::ZERO {
            Self::new(self.center_internal(), self.radius_internal() * factor)
        } else {
            None
        }
    }

    /// 円を指定ベクトルで平行移動
    pub fn translate(&self, offset: Vector2D<T>) -> Self {
        Self::new(self.center_internal() + offset, self.radius_internal()).unwrap()
    }

    /// 円を指定点に移動
    pub fn move_to(&self, new_center: Point2D<T>) -> Self {
        Self::new(new_center, self.radius_internal()).unwrap()
    }

    // ========================================================================
    // Spatial Relationship Methods (Extension)
    // ========================================================================

    /// 他の円と交差するかを判定
    pub fn intersects_circle(&self, other: &Self) -> bool {
        let distance = self.center_internal().distance_to(&other.center_internal());
        let sum_radii = self.radius_internal() + other.radius_internal();
        let diff_radii = (self.radius_internal() - other.radius_internal()).abs();

        distance <= sum_radii && distance >= diff_radii
    }

    /// 他の円を完全に含むかを判定
    pub fn contains_circle(&self, other: &Self) -> bool {
        let distance = self.center_internal().distance_to(&other.center_internal());
        distance + other.radius_internal() <= self.radius_internal()
    }

    // ========================================================================
    // Dimension Extension Methods (Extension)
    // ========================================================================

    /// 3次元円に拡張（Z=0平面）
    pub fn to_3d(&self) -> crate::Circle3D<T> {
        use crate::Direction3D;
        crate::Circle3D::new(
            self.center_internal().to_3d(),
            Direction3D::positive_z(), // Z軸法線
            self.radius_internal(),
        )
        .unwrap()
    }

    /// 3次元円に拡張（指定Z値平面）
    pub fn to_3d_at_z(&self, z: T) -> crate::Circle3D<T> {
        use crate::Direction3D;
        crate::Circle3D::new(
            self.center_internal().to_3d_with_z(z),
            Direction3D::positive_z(), // Z軸法線
            self.radius_internal(),
        )
        .unwrap()
    }

    // ========================================================================
    // Foundation Integration Methods (Extension)
    // ========================================================================

    /// Foundation Transform統合での高度な変換
    pub fn foundation_scale_from_point(&self, point: Point2D<T>, factor: T) -> Option<Self> {
        if factor <= T::ZERO {
            return None;
        }

        // アフィン変換: center' = point + (center - point) * factor
        let offset = Vector2D::from_points(point, self.center_internal());
        let new_center = point + (offset * factor);
        let new_radius = self.radius_internal() * factor;

        Self::new(new_center, new_radius)
    }

    /// Foundation Collision統合での円同士の衝突解決
    pub fn foundation_resolve_collision(&self, other: &Self) -> Option<(Self, Self)> {
        if !self.intersects_circle(other) {
            return None;
        }

        let center_distance = self.center_internal().distance_to(&other.center_internal());
        let required_distance = self.radius_internal() + other.radius_internal();

        if center_distance < T::EPSILON {
            // 同心円の場合は少しずらす
            let offset = Vector2D::new(required_distance / (T::ONE + T::ONE), T::ZERO);
            return Some((self.translate(offset.negate()), other.translate(offset)));
        }

        let direction =
            Vector2D::from_points(self.center_internal(), other.center_internal()).normalize();
        let separation = required_distance - center_distance;
        let half_separation = separation / (T::ONE + T::ONE);

        let self_offset = direction * (-half_separation);
        let other_offset = direction * half_separation;

        Some((self.translate(self_offset), other.translate(other_offset)))
    }

    /// Foundation Intersection統合での複数円の中心計算
    pub fn foundation_weighted_center(&self, others: &[Self], weights: &[T]) -> Option<Point2D<T>> {
        if others.len() != weights.len() {
            return None;
        }

        let mut total_weight = T::ZERO;
        let mut weighted_x = T::ZERO;
        let mut weighted_y = T::ZERO;

        // 自分の重み（半径に基づく）
        let self_weight = self.radius_internal();
        total_weight += self_weight;
        weighted_x += self.center_internal().x() * self_weight;
        weighted_y += self.center_internal().y() * self_weight;

        // 他の円の重み付き中心
        for (circle, &weight) in others.iter().zip(weights) {
            total_weight += weight;
            weighted_x += circle.center_internal().x() * weight;
            weighted_y += circle.center_internal().y() * weight;
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
}
