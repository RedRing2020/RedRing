//! 2D の形状をまたぐ演算の実装

use super::{CrossCollision, CrossIntersection};
use crate::intersection::pair_base::line_segment2d_arc2d_intersections;
use crate::intersection::primitive_2d as intersection;
use crate::{Arc2D, Circle2D, InfiniteLine2D, IntersectionResult, LineSegment2D, Ray2D, Scalar};
use geo_contracts::{Circle2DProperties, InfiniteLine2DDistance, Ray2DDistance};

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

collision_by_intersection!(Ray2D, LineSegment2D);
collision_by_intersection!(InfiniteLine2D, LineSegment2D);
collision_by_intersection!(InfiniteLine2D, Ray2D);

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
