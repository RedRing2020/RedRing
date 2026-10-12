//! 2次元円（Circle2D）の実装
//!
//! STEP (ISO 10303) 準拠の ref_direction フィールドを持つ

mod bounds;
#[cfg(test)]
mod tests;
mod transform;

use crate::{Direction2D, Point2D};
use geo_contracts::{
    default_distance_tolerance, default_kernel_numerical_zero_tolerance, Circle2DConstructor,
    Circle2DContainment, Circle2DDistance, Circle2DEvaluation, Circle2DProjection,
    Circle2DProperties, CrossDistance, PointClassification, PrimitiveKind, PrimitiveMetadata,
    Scalar,
};

/// 2次元円
///
/// STEP準拠でref_direction（参照方向）を持つ
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle2D<T: Scalar> {
    center: Point2D<T>,
    radius: T,
    /// 参照方向（STEP準拠）- 角度0度の方向を定義
    ref_direction: Direction2D<T>,
}

impl<T: Scalar> PrimitiveMetadata for Circle2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Circle
    }
}

impl<T: Scalar> Circle2D<T> {
    /// 参照方向を指定して円を作成（半径が正でない場合は `None`）
    fn new_with_ref_direction(
        center: Point2D<T>,
        radius: T,
        ref_direction: Direction2D<T>,
    ) -> Option<Self> {
        (radius > T::ZERO).then_some(Self {
            center,
            radius,
            ref_direction,
        })
    }

    /// 点に最も近い円周上の点の local angle parameter（`0 <= t < 2π`）を取得
    ///
    /// 円弧が点の角度を求めるため pub(crate)。中心の点は、`closest_point` と同じく角度 0 の点を最近点とする。
    pub(crate) fn parameter_for_point(&self, point: Point2D<T>) -> T {
        let closest = self.closest_point(point);
        let angle = (closest.y() - self.center.y()).atan2(closest.x() - self.center.x());
        if angle < T::ZERO {
            angle + T::TAU
        } else {
            angle
        }
    }
}

impl<T: Scalar> Circle2DConstructor<T> for Circle2D<T> {
    fn new(center: Point2D<T>, radius: T) -> Option<Self> {
        Self::new_with_ref_direction(center, radius, Direction2D::positive_x())
    }

    fn from_three_points(p1: Point2D<T>, p2: Point2D<T>, p3: Point2D<T>) -> Option<Self> {
        // 3点が一直線上にある場合は円が定義できない
        let dx1 = p2.x() - p1.x();
        let dy1 = p2.y() - p1.y();
        let dx2 = p3.x() - p2.x();
        let dy2 = p3.y() - p2.y();

        let cross = dx1 * dy2 - dy1 * dx2;
        if cross.abs() <= default_kernel_numerical_zero_tolerance::<T>() {
            return None;
        }

        // 外心を計算
        let d = T::from_f64(2.0)
            * (p1.x() * (p2.y() - p3.y())
                + p2.x() * (p3.y() - p1.y())
                + p3.x() * (p1.y() - p2.y()));

        if d.abs() <= default_kernel_numerical_zero_tolerance::<T>() {
            return None;
        }

        let p1_sq = p1.x() * p1.x() + p1.y() * p1.y();
        let p2_sq = p2.x() * p2.x() + p2.y() * p2.y();
        let p3_sq = p3.x() * p3.x() + p3.y() * p3.y();

        let cx =
            (p1_sq * (p2.y() - p3.y()) + p2_sq * (p3.y() - p1.y()) + p3_sq * (p1.y() - p2.y())) / d;
        let cy =
            (p1_sq * (p3.x() - p2.x()) + p2_sq * (p1.x() - p3.x()) + p3_sq * (p2.x() - p1.x())) / d;

        let center = Point2D::new(cx, cy);
        <Self as Circle2DConstructor<T>>::new(center, center.distance_to(p1))
    }
}

impl<T: Scalar> Circle2DProperties<T> for Circle2D<T> {
    fn center(&self) -> Point2D<T> {
        self.center
    }

    fn radius(&self) -> T {
        self.radius
    }
}

impl<T: Scalar> Circle2DEvaluation<T> for Circle2D<T> {
    fn point_at_parameter(&self, t: T) -> Point2D<T> {
        Point2D::new(
            self.center.x() + self.radius * t.cos(),
            self.center.y() + self.radius * t.sin(),
        )
    }
}

impl<T: Scalar> Circle2DContainment<T> for Circle2D<T> {
    fn contains_point(&self, point: Point2D<T>) -> bool {
        self.distance_to_point(point) <= default_distance_tolerance::<T>()
    }

    /// 円周までの距離が既定の許容誤差以内なら `OnBoundary`、それ以外は中心からの距離で内側・外側に分ける。
    fn classify_point(&self, point: Point2D<T>) -> PointClassification {
        let center_distance = point.distance_to(self.center);
        if (center_distance - self.radius).abs() <= default_distance_tolerance::<T>() {
            PointClassification::OnBoundary
        } else if center_distance < self.radius {
            PointClassification::Inside
        } else {
            PointClassification::Outside
        }
    }
}

impl<T: Scalar> Circle2DDistance<T> for Circle2D<T> {
    fn distance_to_point(&self, point: Point2D<T>) -> T {
        (point.distance_to(self.center) - self.radius).abs()
    }
}

impl<T: Scalar> Circle2DProjection<T> for Circle2D<T> {
    /// 中心の点は、角度 0 の点を最近点とする
    fn closest_point(&self, point: Point2D<T>) -> Point2D<T> {
        let dx = point.x() - self.center.x();
        let dy = point.y() - self.center.y();
        let distance = point.distance_to(self.center);

        if distance <= default_kernel_numerical_zero_tolerance::<T>() {
            Point2D::new(self.center.x() + self.radius, self.center.y())
        } else {
            let scale = self.radius / distance;
            Point2D::new(self.center.x() + dx * scale, self.center.y() + dy * scale)
        }
    }
}

impl<T: Scalar> CrossDistance<T, Self> for Circle2D<T> {
    fn distance_to(&self, other: &Self) -> T {
        let center_distance = self.center.distance_to(other.center);
        let radii_sum = self.radius + other.radius;

        if center_distance >= radii_sum {
            // 円が外部にある
            center_distance - radii_sum
        } else if center_distance <= (self.radius - other.radius).abs() {
            // 一方が他方の内部にある
            (self.radius - other.radius).abs() - center_distance
        } else {
            // 円が交差している
            T::ZERO
        }
    }
}
