//! 2D Primitive distance algorithms
//!
//! `geo_primitives` に実装済みの各 2D 形状の距離計算メソッドを
//! `geo_algorithms` 層の公開 entrypoint としてまとめた薄いラッパー群。
//!
//! 命名規則: `{shape_a}_{shape_b}_distance`

use crate::{Circle2D, InfiniteLine2D, Point2D, Ray2D};
use geo_contracts::Scalar;

/// 無限直線-点 間の最短距離（垂直距離）
pub fn infinite_line2d_point2d_distance<T: Scalar>(
    line: &InfiniteLine2D<T>,
    point: &Point2D<T>,
) -> T {
    line.distance_to_point(point)
}

/// 逆向きラッパー: point-line
pub fn point2d_infinite_line2d_distance<T: Scalar>(
    point: &Point2D<T>,
    line: &InfiniteLine2D<T>,
) -> T {
    line.distance_to_point(point)
}

/// Ray2D-点 間の最短距離（Ray の有効範囲を考慮）
pub fn ray2d_point2d_distance<T: Scalar>(ray: &Ray2D<T>, point: &Point2D<T>) -> T {
    ray.distance_to_point(point)
}

/// 逆向きラッパー: point-ray
pub fn point2d_ray2d_distance<T: Scalar>(point: &Point2D<T>, ray: &Ray2D<T>) -> T {
    ray.distance_to_point(point)
}

/// Circle2D-点 間の最短距離（円周への距離）
pub fn circle2d_point2d_distance<T: Scalar>(circle: &Circle2D<T>, point: &Point2D<T>) -> T {
    circle.distance_to_point(*point)
}

/// 逆向きラッパー: point-circle
pub fn point2d_circle2d_distance<T: Scalar>(point: &Point2D<T>, circle: &Circle2D<T>) -> T {
    circle.distance_to_point(*point)
}
