//! 衝突判定のペアbase実装
//!
//! 型ごとの trait実装とは分離し、形状ペア単位の衝突判定を集約する。

use crate::intersection::primitive_2d::{
    infinite_line2d_line_segment2d_intersection as infinite_line2d_line_segment2d_intersection_2d,
    infinite_line2d_ray2d_intersection as infinite_line2d_ray2d_intersection_2d,
    ray2d_line_segment2d_intersection as ray2d_line_segment2d_intersection_2d,
};
use crate::{InfiniteLine2D, LineSegment2D, Ray2D};
use geo_contracts::Scalar;

pub fn ray2d_line_segment2d_collides<T: Scalar>(
    ray: &Ray2D<T>,
    segment: &LineSegment2D<T>,
    tolerance: T,
) -> bool {
    ray2d_line_segment2d_intersection_2d(ray, segment, tolerance).intersects()
}

pub fn infinite_line2d_line_segment2d_collides<T: Scalar>(
    line: &InfiniteLine2D<T>,
    segment: &LineSegment2D<T>,
    tolerance: T,
) -> bool {
    infinite_line2d_line_segment2d_intersection_2d(line, segment, tolerance).intersects()
}

pub fn infinite_line2d_ray2d_collides<T: Scalar>(
    line: &InfiniteLine2D<T>,
    ray: &Ray2D<T>,
    tolerance: T,
) -> bool {
    infinite_line2d_ray2d_intersection_2d(line, ray, tolerance).intersects()
}

// collision 判定は intersection 側の判定ロジックを正本として再利用し、
// 幾何条件の二重実装を避ける。
