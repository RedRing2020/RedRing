//! 2D Primitive distance algorithms
//!
//! 各 2D 形状間の距離計算エントリポイント。
//! 独自計算で実装し、geo_primitives への delegate は行わない（循環依存防止）。
//!
//! 命名規則: `{shape_a}_{shape_b}_distance`

use crate::{Circle2D, InfiniteLine2D, LineSegment2D, Point2D, Ray2D};
use geo_contracts::{
    Circle2DProperties, InfiniteLine2DProperties, LineSegment2DProperties, Ray2DProperties, Scalar,
};

/// LineSegment2D-点 間の最短距離（端点クランプあり）
pub fn line_segment2d_point2d_distance<T: Scalar>(
    segment: &LineSegment2D<T>,
    point: &Point2D<T>,
) -> T {
    let s1x = LineSegment2DProperties::start(segment).0;
    let s1y = LineSegment2DProperties::start(segment).1;
    let s2x = LineSegment2DProperties::end(segment).0;
    let s2y = LineSegment2DProperties::end(segment).1;
    let dx = s2x - s1x;
    let dy = s2y - s1y;
    let len_sq = dx * dx + dy * dy;
    if len_sq <= T::EPSILON {
        return ((point.x() - s1x) * (point.x() - s1x) + (point.y() - s1y) * (point.y() - s1y))
            .sqrt();
    }
    let t = ((point.x() - s1x) * dx + (point.y() - s1y) * dy) / len_sq;
    let t_clamped = t.max(T::ZERO).min(T::ONE);
    let proj_x = s1x + t_clamped * dx;
    let proj_y = s1y + t_clamped * dy;
    ((point.x() - proj_x) * (point.x() - proj_x) + (point.y() - proj_y) * (point.y() - proj_y))
        .sqrt()
}

/// 逆向きラッパー: point-segment
pub fn point2d_line_segment2d_distance<T: Scalar>(
    point: &Point2D<T>,
    segment: &LineSegment2D<T>,
) -> T {
    line_segment2d_point2d_distance(segment, point)
}

/// 無限直線-点 間の最短距離（垂直距離）
pub fn infinite_line2d_point2d_distance<T: Scalar>(
    line: &InfiniteLine2D<T>,
    point: &Point2D<T>,
) -> T {
    let (px, py) = InfiniteLine2DProperties::point(line);
    let (dx, dy) = InfiniteLine2DProperties::direction(line);
    // 法線方向への射影 = cross product / |dir| (dir は単位ベクトル)
    let to_x = point.x() - px;
    let to_y = point.y() - py;
    (to_x * dy - to_y * dx).abs()
}

/// 逆向きラッパー: point-line
pub fn point2d_infinite_line2d_distance<T: Scalar>(
    point: &Point2D<T>,
    line: &InfiniteLine2D<T>,
) -> T {
    infinite_line2d_point2d_distance(line, point)
}

/// Ray2D-点 間の最短距離（Ray の有効範囲を考慮）
pub fn ray2d_point2d_distance<T: Scalar>(ray: &Ray2D<T>, point: &Point2D<T>) -> T {
    let (ox, oy) = Ray2DProperties::origin(ray);
    let (dx, dy) = Ray2DProperties::direction(ray);
    let to_x = point.x() - ox;
    let to_y = point.y() - oy;
    let t = to_x * dx + to_y * dy;
    if t >= T::ZERO {
        // 点が Ray の有効範囲内: 直線上への射影距離
        (to_x * dy - to_y * dx).abs()
    } else {
        // 点が起点より後ろ: 起点への距離
        (to_x * to_x + to_y * to_y).sqrt()
    }
}

/// 逆向きラッパー: point-ray
pub fn point2d_ray2d_distance<T: Scalar>(point: &Point2D<T>, ray: &Ray2D<T>) -> T {
    ray2d_point2d_distance(ray, point)
}

/// Circle2D-点 間の最短距離（円周への距離）
pub fn circle2d_point2d_distance<T: Scalar>(circle: &Circle2D<T>, point: &Point2D<T>) -> T {
    let (cx, cy) = Circle2DProperties::center(circle);
    let r = Circle2DProperties::radius(circle);
    let dist = ((point.x() - cx) * (point.x() - cx) + (point.y() - cy) * (point.y() - cy)).sqrt();
    (dist - r).abs()
}

/// 逆向きラッパー: point-circle
pub fn point2d_circle2d_distance<T: Scalar>(point: &Point2D<T>, circle: &Circle2D<T>) -> T {
    circle2d_point2d_distance(circle, point)
}

/// Circle2D-Circle2D 間の最短距離（交差時は 0）
pub fn circle2d_circle2d_distance<T: Scalar>(c1: &Circle2D<T>, c2: &Circle2D<T>) -> T {
    let (cx1, cy1) = Circle2DProperties::center(c1);
    let (cx2, cy2) = Circle2DProperties::center(c2);
    let r1 = Circle2DProperties::radius(c1);
    let r2 = Circle2DProperties::radius(c2);
    let center_dist = ((cx2 - cx1) * (cx2 - cx1) + (cy2 - cy1) * (cy2 - cy1)).sqrt();
    let radii_sum = r1 + r2;
    let radii_diff = (r1 - r2).abs();
    if center_dist >= radii_sum {
        center_dist - radii_sum
    } else if center_dist <= radii_diff {
        radii_diff - center_dist
    } else {
        T::ZERO
    }
}

/// LineSegment2D-LineSegment2D 間の最短距離
pub fn line_segment2d_line_segment2d_distance<T: Scalar>(
    seg1: &LineSegment2D<T>,
    seg2: &LineSegment2D<T>,
) -> T {
    let d1 = line_segment2d_point2d_distance(
        seg1,
        &Point2D::new(
            LineSegment2DProperties::start(seg2).0,
            LineSegment2DProperties::start(seg2).1,
        ),
    );
    let d2 = line_segment2d_point2d_distance(
        seg1,
        &Point2D::new(
            LineSegment2DProperties::end(seg2).0,
            LineSegment2DProperties::end(seg2).1,
        ),
    );
    let d3 = line_segment2d_point2d_distance(
        seg2,
        &Point2D::new(
            LineSegment2DProperties::start(seg1).0,
            LineSegment2DProperties::start(seg1).1,
        ),
    );
    let d4 = line_segment2d_point2d_distance(
        seg2,
        &Point2D::new(
            LineSegment2DProperties::end(seg1).0,
            LineSegment2DProperties::end(seg1).1,
        ),
    );
    d1.min(d2).min(d3).min(d4)
}

/// Ray2D-Ray2D 間の最短距離
pub fn ray2d_ray2d_distance<T: Scalar>(ray1: &Ray2D<T>, ray2: &Ray2D<T>) -> T {
    let (ox1, oy1) = Ray2DProperties::origin(ray1);
    let (dx1, dy1) = Ray2DProperties::direction(ray1);
    let (ox2, oy2) = Ray2DProperties::origin(ray2);
    let (dx2, dy2) = Ray2DProperties::direction(ray2);

    let denominator = dx1 * dy2 - dy1 * dx2;
    if denominator.abs() > T::EPSILON {
        // 非平行: 交点パラメータを計算
        let dp_x = ox2 - ox1;
        let dp_y = oy2 - oy1;
        let t1 = (dp_x * dy2 - dp_y * dx2) / denominator;
        let t2 = (dp_x * dy1 - dp_y * dx1) / denominator;
        if t1 >= T::ZERO && t2 >= T::ZERO {
            // 両Ray の有効範囲内で交差: 距離 0
            return T::ZERO;
        }
        // 有効範囲外: 端点間の最短距離
        let p1 = Point2D::new(ox1, oy1);
        let p2 = Point2D::new(ox2, oy2);
        let p1_on_r2 = if t2 >= T::ZERO {
            Point2D::new(ox2 + t2 * dx2, oy2 + t2 * dy2)
        } else {
            p2
        };
        let p2_on_r1 = if t1 >= T::ZERO {
            Point2D::new(ox1 + t1 * dx1, oy1 + t1 * dy1)
        } else {
            p1
        };
        let d1 = ray2d_point2d_distance(ray1, &p1_on_r2);
        let d2 = ray2d_point2d_distance(ray2, &p2_on_r1);
        d1.min(d2)
    } else {
        // 平行: 各端点からもう一方のRayへの最短距離
        let p_orig2 = Point2D::new(ox2, oy2);
        let p_orig1 = Point2D::new(ox1, oy1);
        ray2d_point2d_distance(ray1, &p_orig2).min(ray2d_point2d_distance(ray2, &p_orig1))
    }
}

/// Ray2D-LineSegment2D 間の最短距離
pub fn ray2d_line_segment2d_distance<T: Scalar>(ray: &Ray2D<T>, segment: &LineSegment2D<T>) -> T {
    let (ox, oy) = Ray2DProperties::origin(ray);
    let (rdx, rdy) = Ray2DProperties::direction(ray);
    let s1x = LineSegment2DProperties::start(segment).0;
    let s1y = LineSegment2DProperties::start(segment).1;
    let s2x = LineSegment2DProperties::end(segment).0;
    let s2y = LineSegment2DProperties::end(segment).1;
    let sdx = s2x - s1x;
    let sdy = s2y - s1y;

    let denominator = rdx * sdy - rdy * sdx;
    if denominator.abs() > T::EPSILON {
        let dp_x = s1x - ox;
        let dp_y = s1y - oy;
        let t_ray = (dp_x * sdy - dp_y * sdx) / denominator;
        let t_seg = (dp_x * rdy - dp_y * rdx) / denominator;
        if t_ray >= T::ZERO && t_seg >= T::ZERO && t_seg <= T::ONE {
            return T::ZERO;
        }
    }
    // 各端点からもう一方の形状への最短距離の最小値
    let seg_start = Point2D::new(s1x, s1y);
    let seg_end = Point2D::new(s2x, s2y);
    let ray_origin = Point2D::new(ox, oy);
    ray2d_point2d_distance(ray, &seg_start)
        .min(ray2d_point2d_distance(ray, &seg_end))
        .min(line_segment2d_point2d_distance(segment, &ray_origin))
}
