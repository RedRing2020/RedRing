//! 2D Primitive distance algorithms
//!
//! 各 2D 形状間の距離計算エントリポイント。
//! 独自計算で実装し、geo_primitives への delegate は行わない（循環依存防止）。
//!
//! 命名規則: `{shape_a}_{shape_b}_distance`

use crate::{Circle2D, InfiniteLine2D, LineSegment2D, Point2D, Ray2D};
use geo_contracts::{
    default_kernel_numerical_zero_tolerance, default_parallel_cross_error_tolerance,
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
    let zero_tol = default_kernel_numerical_zero_tolerance::<T>();
    if len_sq <= zero_tol * zero_tol {
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
    let s1x = LineSegment2DProperties::start(seg1).0;
    let s1y = LineSegment2DProperties::start(seg1).1;
    let s2x = LineSegment2DProperties::end(seg1).0;
    let s2y = LineSegment2DProperties::end(seg1).1;
    let p1x = LineSegment2DProperties::start(seg2).0;
    let p1y = LineSegment2DProperties::start(seg2).1;
    let p2x = LineSegment2DProperties::end(seg2).0;
    let p2y = LineSegment2DProperties::end(seg2).1;

    let d1x = s2x - s1x;
    let d1y = s2y - s1y;
    let d2x = p2x - p1x;
    let d2y = p2y - p1y;

    // 交差チェック: 線分内部で交差していれば距離 0
    // ゼロ除算を防ぐため絶対値がカーネルゼロ閾値を超える場合のみ t/s を計算する
    let denom = d1x * d2y - d1y * d2x;
    let zero_tol = default_kernel_numerical_zero_tolerance::<T>();
    if denom.abs() > zero_tol {
        let dp_x = p1x - s1x;
        let dp_y = p1y - s1y;
        let t = (dp_x * d2y - dp_y * d2x) / denom;
        let s = (dp_x * d1y - dp_y * d1x) / denom;
        if t >= T::ZERO && t <= T::ONE && s >= T::ZERO && s <= T::ONE {
            return T::ZERO;
        }
    }

    // 各端点からもう一方の線分への距離の最小値
    line_segment2d_point2d_distance(seg1, &Point2D::new(p1x, p1y))
        .min(line_segment2d_point2d_distance(
            seg1,
            &Point2D::new(p2x, p2y),
        ))
        .min(line_segment2d_point2d_distance(
            seg2,
            &Point2D::new(s1x, s1y),
        ))
        .min(line_segment2d_point2d_distance(
            seg2,
            &Point2D::new(s2x, s2y),
        ))
}

/// Ray2D-Ray2D 間の最短距離
pub fn ray2d_ray2d_distance<T: Scalar>(ray1: &Ray2D<T>, ray2: &Ray2D<T>) -> T {
    let (ox1, oy1) = Ray2DProperties::origin(ray1);
    let (dx1, dy1) = Ray2DProperties::direction(ray1);
    let (ox2, oy2) = Ray2DProperties::origin(ray2);
    let (dx2, dy2) = Ray2DProperties::direction(ray2);

    let denom = dx1 * dy2 - dy1 * dx2;
    let par_tol = default_parallel_cross_error_tolerance::<T>();
    if denom.abs() > par_tol {
        // 非平行: 交点パラメータを計算
        let dp_x = ox2 - ox1;
        let dp_y = oy2 - oy1;
        let t1 = (dp_x * dy2 - dp_y * dx2) / denom;
        let t2 = (dp_x * dy1 - dp_y * dx1) / denom;
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

    let denom = rdx * sdy - rdy * sdx;
    let seg_len_sq = sdx * sdx + sdy * sdy;
    let par_tol = default_parallel_cross_error_tolerance::<T>();
    if denom * denom > par_tol * par_tol * seg_len_sq {
        let dp_x = s1x - ox;
        let dp_y = s1y - oy;
        let t_ray = (dp_x * sdy - dp_y * sdx) / denom;
        let t_seg = (dp_x * rdy - dp_y * rdx) / denom;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Circle2D, LineSegment2D, Point2D, Ray2D, Vector2D};
    use analysis::test_constants;

    const TOL: f64 = test_constants::DISTANCE_TOLERANCE_F64;

    // --- line_segment2d_line_segment2d_distance ---

    #[test]
    fn seg_seg_distance_crossing_is_zero() {
        // 十字交差: 距離は 0 でなければならない
        let seg1 = LineSegment2D::new(Point2D::new(-1.0, 0.0), Point2D::new(1.0, 0.0)).unwrap();
        let seg2 = LineSegment2D::new(Point2D::new(0.0, -1.0), Point2D::new(0.0, 1.0)).unwrap();
        let d = line_segment2d_line_segment2d_distance(&seg1, &seg2);
        assert!(d < TOL, "crossing segments must have distance 0, got {d}");
    }

    #[test]
    fn seg_seg_distance_parallel_non_touching() {
        let seg1 = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(1.0, 0.0)).unwrap();
        let seg2 = LineSegment2D::new(Point2D::new(0.0, 2.0), Point2D::new(1.0, 2.0)).unwrap();
        let d = line_segment2d_line_segment2d_distance(&seg1, &seg2);
        assert!(
            (d - 2.0).abs() < TOL,
            "parallel segments distance should be 2, got {d}"
        );
    }

    #[test]
    fn seg_seg_distance_t_shape_endpoint_touch() {
        // seg1 の端点が seg2 の中点に触れる T 字形
        let seg1 = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(0.0, 1.0)).unwrap();
        let seg2 = LineSegment2D::new(Point2D::new(-1.0, 1.0), Point2D::new(1.0, 1.0)).unwrap();
        let d = line_segment2d_line_segment2d_distance(&seg1, &seg2);
        assert!(
            d < TOL,
            "T-shape touching segments must have distance 0, got {d}"
        );
    }

    #[test]
    fn ray_ray_distance_crossing_is_zero() {
        let ray1 = Ray2D::new(Point2D::new(0.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
        let ray2 = Ray2D::new(Point2D::new(1.0, -1.0), Vector2D::new(0.0, 1.0)).unwrap();
        let d = ray2d_ray2d_distance(&ray1, &ray2);
        assert!(d < TOL, "crossing rays must have distance 0, got {d}");
    }

    #[test]
    fn ray_ray_distance_non_intersecting_positive() {
        let ray1 = Ray2D::new(Point2D::new(2.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
        let ray2 = Ray2D::new(Point2D::new(0.0, 1.0), Vector2D::new(0.0, 1.0)).unwrap();
        let d = ray2d_ray2d_distance(&ray1, &ray2);
        assert!(
            (d - 5.0_f64.sqrt()).abs() < TOL,
            "expected sqrt(5), got {d}"
        );
    }

    #[test]
    fn ray_segment_distance_crossing_is_zero() {
        let ray = Ray2D::new(Point2D::new(0.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
        let segment = LineSegment2D::new(Point2D::new(1.0, -1.0), Point2D::new(1.0, 1.0)).unwrap();
        let d = ray2d_line_segment2d_distance(&ray, &segment);
        assert!(
            d < TOL,
            "crossing ray/segment must have distance 0, got {d}"
        );
    }

    #[test]
    fn ray_segment_distance_origin_to_segment() {
        let ray = Ray2D::new(Point2D::new(0.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();
        let segment = LineSegment2D::new(Point2D::new(-2.0, 3.0), Point2D::new(-2.0, 5.0)).unwrap();
        let d = ray2d_line_segment2d_distance(&ray, &segment);
        assert!(
            (d - 13.0_f64.sqrt()).abs() < TOL,
            "expected sqrt(13), got {d}"
        );
    }

    // --- circle2d_circle2d_distance ---

    #[test]
    fn circle_circle_distance_external_is_positive() {
        // 外接より外: 距離 = 中心間距離 - 半径の和
        let c1 = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        let c2 = Circle2D::new(Point2D::new(5.0, 0.0), 1.0).unwrap();
        let d = circle2d_circle2d_distance(&c1, &c2);
        assert!((d - 3.0).abs() < TOL, "expected 3.0, got {d}");
    }

    #[test]
    fn circle_circle_distance_intersecting_is_zero() {
        // 交差: 距離は 0
        let c1 = Circle2D::new(Point2D::new(0.0, 0.0), 2.0).unwrap();
        let c2 = Circle2D::new(Point2D::new(2.0, 0.0), 2.0).unwrap();
        let d = circle2d_circle2d_distance(&c1, &c2);
        assert!(
            d < TOL,
            "intersecting circles must have distance 0, got {d}"
        );
    }

    #[test]
    fn circle_circle_distance_one_inside_other_is_positive() {
        // 内包: 距離 = 半径差 - 中心間距離
        let c1 = Circle2D::new(Point2D::new(0.0, 0.0), 5.0).unwrap();
        let c2 = Circle2D::new(Point2D::new(1.0, 0.0), 1.0).unwrap();
        let d = circle2d_circle2d_distance(&c1, &c2);
        assert!((d - 3.0).abs() < TOL, "expected 3.0, got {d}");
    }

    // --- triangle3d_point3d_distance は primitive_3d テストで網羅 ---
    // --- infinite_line2d_infinite_line2d_intersection / ray2d_ray2d_intersection は intersection テストで網羅 ---
}
