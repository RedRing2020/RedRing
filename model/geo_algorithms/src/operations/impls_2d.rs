//! 2D の形状をまたぐ演算の実装

use super::{CrossCollision, CrossIntersection};
use crate::intersection::pair_base::line_segment2d_arc2d_intersections;
use crate::intersection::primitive_2d as intersection;
use crate::{
    Arc2D, Circle2D, InfiniteLine2D, IntersectionResult, LineSegment2D, Point2D, Ray2D, Scalar,
    Triangle2D, Vector2D,
};
use geo_contracts::{
    Arc2DEndpoint, Arc2DProperties, Arc2DTrimRange, Circle2DProperties, InfiniteLine2DDistance,
    Ray2DDistance, Triangle2DBoundaryAccess, Triangle2DDistance,
};

/// 逆の向き（`$b` から `$a`）の衝突判定は代表の向きを呼ぶ
macro_rules! reverse_collision {
    ($a:ident, $b:ident) => {
        impl<T: Scalar> CrossCollision<T, $a<T>> for $b<T> {
            fn collides(&self, other: &$a<T>, tolerance: T) -> bool {
                other.collides(self, tolerance)
            }
        }
    };
}

/// 交点の有無で衝突判定し、逆の向きは代表の向きを呼ぶ
macro_rules! collision_by_intersection {
    ($a:ident, $b:ident) => {
        impl<T: Scalar> CrossCollision<T, $b<T>> for $a<T> {
            fn collides(&self, other: &$b<T>, tolerance: T) -> bool {
                self.intersection(other, tolerance).intersects()
            }
        }

        reverse_collision!($a, $b);
    };
}

/// 代表の向き（`$a` から `$b`）の交差を `$function` で書き、逆の向きはそれを呼ぶ
macro_rules! symmetric_intersection {
    ($a:ident, $b:ident, $function:path) => {
        impl<T: Scalar> CrossIntersection<T, $b<T>> for $a<T> {
            fn intersection(&self, other: &$b<T>, tolerance: T) -> IntersectionResult<T> {
                $function(self, other, tolerance)
            }
        }

        impl<T: Scalar> CrossIntersection<T, $a<T>> for $b<T> {
            fn intersection(&self, other: &$a<T>, tolerance: T) -> IntersectionResult<T> {
                other.intersection(self, tolerance)
            }
        }
    };
}

impl<T: Scalar> CrossCollision<T, Circle2D<T>> for Ray2D<T> {
    fn collides(&self, circle: &Circle2D<T>, tolerance: T) -> bool {
        self.distance_to_point(circle.center()) <= circle.radius() + tolerance
    }
}
reverse_collision!(Ray2D, Circle2D);

impl<T: Scalar> CrossCollision<T, Circle2D<T>> for InfiniteLine2D<T> {
    fn collides(&self, circle: &Circle2D<T>, tolerance: T) -> bool {
        self.distance_to_point(circle.center()) <= circle.radius() + tolerance
    }
}
reverse_collision!(InfiniteLine2D, Circle2D);

impl<T: Scalar> CrossCollision<T, Arc2D<T>> for LineSegment2D<T> {
    fn collides(&self, arc: &Arc2D<T>, tolerance: T) -> bool {
        !line_segment2d_arc2d_intersections(self, arc, tolerance).is_empty()
    }
}
reverse_collision!(LineSegment2D, Arc2D);

/// 曲線上の点から `center` までの距離の範囲が、円の半径を `tolerance` の幅で含むか
///
/// 曲線はつながっているので、距離は最小値と最大値の間のすべての値をとる。
fn distance_range_reaches_radius<T: Scalar>(
    min_distance: T,
    max_distance: T,
    radius: T,
    tolerance: T,
) -> bool {
    min_distance <= radius + tolerance && max_distance >= radius - tolerance
}

impl<T: Scalar> CrossCollision<T, Circle2D<T>> for Arc2D<T> {
    /// 円弧上の点から円の中心までの距離の最小値・最大値で判定する
    ///
    /// 最小値・最大値をとる点の候補は、円弧の両端と、2 つの中心を結ぶ直線と円弧の交点。
    fn collides(&self, circle: &Circle2D<T>, tolerance: T) -> bool {
        let center = circle.center();
        let mut candidates: Vec<Point2D<T>> = vec![self.start_point(), self.end_point()];
        let offset = Vector2D::from_points(self.center(), center);
        let offset_length = offset.length();
        if offset_length > T::ZERO {
            let toward = offset * (self.radius() / offset_length);
            candidates.extend(
                [self.center() + toward, self.center() - toward]
                    .into_iter()
                    .filter(|point| self.contains_point_angle(*point)),
            );
        }
        let distances = candidates.iter().map(|point| point.distance_to(center));
        let min_distance = distances.clone().fold(T::INFINITY, T::min);
        let max_distance = distances.fold(T::ZERO, T::max);
        distance_range_reaches_radius(min_distance, max_distance, circle.radius(), tolerance)
    }
}
reverse_collision!(Arc2D, Circle2D);

impl<T: Scalar> CrossCollision<T, Circle2D<T>> for Triangle2D<T> {
    /// 円の中心から三角形（領域）までの距離の最小値と、最も遠い頂点までの距離で判定する
    fn collides(&self, circle: &Circle2D<T>, tolerance: T) -> bool {
        let center = circle.center();
        let max_distance = [self.vertex_a(), self.vertex_b(), self.vertex_c()]
            .into_iter()
            .map(|vertex| vertex.distance_to(center))
            .fold(T::ZERO, T::max);
        distance_range_reaches_radius(
            self.distance_to_point(center),
            max_distance,
            circle.radius(),
            tolerance,
        )
    }
}
reverse_collision!(Triangle2D, Circle2D);

collision_by_intersection!(Triangle2D, LineSegment2D);
collision_by_intersection!(Ray2D, LineSegment2D);
collision_by_intersection!(InfiniteLine2D, LineSegment2D);
collision_by_intersection!(InfiniteLine2D, Ray2D);

symmetric_intersection!(
    Triangle2D,
    LineSegment2D,
    intersection::triangle2d_line_segment2d_intersection
);
symmetric_intersection!(Ray2D, Circle2D, intersection::ray2d_circle2d_intersections);
symmetric_intersection!(
    Ray2D,
    LineSegment2D,
    intersection::ray2d_line_segment2d_intersection
);
symmetric_intersection!(
    InfiniteLine2D,
    Circle2D,
    intersection::infinite_line2d_circle2d_intersections
);
symmetric_intersection!(
    InfiniteLine2D,
    LineSegment2D,
    intersection::infinite_line2d_line_segment2d_intersection
);
symmetric_intersection!(
    InfiniteLine2D,
    Ray2D,
    intersection::infinite_line2d_ray2d_intersection
);
