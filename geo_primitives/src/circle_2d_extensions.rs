//! Circle2D 拡張機能
//!
//! Extension Foundation パターンに基づく Circle2D の拡張実装

use crate::{Circle2D, Point2D, Vector2D};
use geo_foundation::Scalar;

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

        Some(Self::new(center, radius)?)
    }

    /// 単位円を作成（原点中心、半径1）
    pub fn unit_circle() -> Self {
        Self::new(Point2D::new(T::ZERO, T::ZERO), T::ONE).unwrap()
    }

    // ========================================================================
    // Convenience Methods (Extension)
    // ========================================================================

    /// 直径を取得
    pub fn diameter(&self) -> T {
        let two = T::ONE + T::ONE;
        self.radius() * two
    }

    /// 指定角度での点を取得（ラジアン）
    pub fn point_at_angle(&self, angle: T) -> Point2D<T> {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Point2D::new(
            self.center().x() + self.radius() * cos_a,
            self.center().y() + self.radius() * sin_a,
        )
    }

    /// 点が円上にあるかを判定
    pub fn contains_point_on_circle(&self, point: &Point2D<T>, tolerance: T) -> bool {
        let distance = self.center().distance_to(point);
        (distance - self.radius()).abs() <= tolerance
    }

    // ========================================================================
    // Transformation Methods (Extension)
    // ========================================================================

    /// 円を指定倍率でスケール
    pub fn scale(&self, factor: T) -> Option<Self> {
        if factor > T::ZERO {
            Self::new(self.center(), self.radius() * factor)
        } else {
            None
        }
    }

    /// 円を指定ベクトルで平行移動
    pub fn translate(&self, offset: Vector2D<T>) -> Self {
        Self::new(self.center() + offset, self.radius()).unwrap()
    }

    /// 円を指定点に移動
    pub fn move_to(&self, new_center: Point2D<T>) -> Self {
        Self::new(new_center, self.radius()).unwrap()
    }

    // ========================================================================
    // Spatial Relationship Methods (Extension)
    // ========================================================================

    /// 他の円と交差するかを判定
    pub fn intersects_circle(&self, other: &Self) -> bool {
        let distance = self.center().distance_to(&other.center());
        let sum_radii = self.radius() + other.radius();
        let diff_radii = (self.radius() - other.radius()).abs();

        distance <= sum_radii && distance >= diff_radii
    }

    /// 他の円を完全に含むかを判定
    pub fn contains_circle(&self, other: &Self) -> bool {
        let distance = self.center().distance_to(&other.center());
        distance + other.radius() <= self.radius()
    }

    // ========================================================================
    // Dimension Extension Methods (Extension)
    // ========================================================================

    /// 3次元円に拡張（Z=0平面）
    pub fn to_3d(&self) -> crate::Circle3D<T> {
        crate::Circle3D::new(
            self.center().to_3d(),
            Vector2D::new(T::ZERO, T::ZERO).to_3d_with_z(T::ONE), // Z軸法線
            self.radius(),
        )
        .unwrap()
    }

    /// 3次元円に拡張（指定Z値平面）
    pub fn to_3d_at_z(&self, z: T) -> crate::Circle3D<T> {
        crate::Circle3D::new(
            self.center().to_3d_with_z(z),
            Vector2D::new(T::ZERO, T::ZERO).to_3d_with_z(T::ONE), // Z軸法線
            self.radius(),
        )
        .unwrap()
    }
}
