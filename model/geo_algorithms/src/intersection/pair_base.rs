//! 2D の線分と円・円弧の交点

use crate::{Arc2D, Circle2D, LineSegment2D, Point2D, Vector2D};
use geo_contracts::{
    Arc2DProperties, Arc2DTrimRange, Circle2DConstructor, Circle2DProperties,
    LineSegment2DProperties, Scalar,
};

/// 線分と円の交点（線分の始点に近い順）と、接するか
///
/// 中心から線分を含む直線までの距離 h で判定する。h が半径より `tolerance` を超えて大きければ交わらず、半径との差が
/// `tolerance` 以内なら垂線の足で接する。線分の範囲は、端点から `tolerance` までの点を含む。
pub fn line_segment2d_circle2d_intersections<T: Scalar>(
    segment: &LineSegment2D<T>,
    circle: &Circle2D<T>,
    tolerance: T,
) -> (Vec<Point2D<T>>, bool) {
    let mut result = Vec::new();

    let center = circle.center();
    let radius = circle.radius();
    let start = segment.start_point();
    let direction = Vector2D::from_points(start, segment.end_point());
    let offset = Vector2D::from_points(center, start);

    let length_squared = direction.length_squared();
    let length = length_squared.sqrt();
    if length <= tolerance {
        return (result, false);
    }

    // 中心に最も近い直線上の点 t_c と、そこから中心までの距離 h を先に求め、t = t_c ± √(r² − h²) / |d| とする
    let closest = -offset.dot(direction) / length_squared;
    let nearest = offset + direction * closest;
    let distance = nearest.length();
    let slack = tolerance / length;
    let in_segment = |t: T| t >= -slack && t <= T::ONE + slack;
    if distance > radius + tolerance {
        return (result, false);
    }
    if (distance - radius).abs() <= tolerance {
        if in_segment(closest) {
            result.push(start + direction * closest);
        }
        return (result, true);
    }

    let half = ((radius - distance) * (radius + distance)).sqrt() / length;
    for t in [closest - half, closest + half] {
        if in_segment(t) {
            result.push(start + direction * t);
        }
    }

    (result, false)
}

pub fn line_segment2d_arc2d_intersections<T: Scalar>(
    segment: &LineSegment2D<T>,
    arc: &Arc2D<T>,
    tolerance: T,
) -> Vec<Point2D<T>> {
    let base_circle = Circle2D::new(arc.center(), <Arc2D<T> as Arc2DProperties<T>>::radius(arc));

    if let Some(base_circle) = base_circle {
        line_segment2d_circle2d_intersections(segment, &base_circle, tolerance)
            .0
            .into_iter()
            .filter(|p| arc.contains_point_angle(*p))
            .collect()
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{line_segment2d_arc2d_intersections, line_segment2d_circle2d_intersections};
    use crate::{Angle, Arc2D, Circle2D, LineSegment2D, Point2D};
    use geo_contracts::Arc2DConstructor;
    use geo_contracts::Circle2DConstructor;
    use geo_contracts::Circle2DProperties;
    use geo_contracts::LineSegment2DConstructor;

    const STANDARD_TEST_TOLERANCE_F64: f64 = analysis::test_constants::DISTANCE_TOLERANCE_F64;

    #[test]
    fn line_segment_circle_returns_two_points() {
        let segment = LineSegment2D::new(Point2D::new(-2.0, 0.0), Point2D::new(2.0, 0.0)).unwrap();
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();

        let (points, tangent) =
            line_segment2d_circle2d_intersections(&segment, &circle, STANDARD_TEST_TOLERANCE_F64);
        assert_eq!(points.len(), 2);
        assert!(!tangent);
    }

    #[test]
    fn line_segment_tangent_to_circle_returns_one_touching_point() {
        let segment = LineSegment2D::new(Point2D::new(-2.0, 1.0), Point2D::new(2.0, 1.0)).unwrap();
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();

        let (points, tangent) =
            line_segment2d_circle2d_intersections(&segment, &circle, STANDARD_TEST_TOLERANCE_F64);
        assert_eq!(points, vec![Point2D::new(0.0, 1.0)]);
        assert!(tangent);
    }

    #[test]
    fn line_segment_ending_inside_circle_crosses_once() {
        let segment = LineSegment2D::new(Point2D::new(-2.0, 0.0), Point2D::new(0.0, 0.0)).unwrap();
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();

        let (points, tangent) =
            line_segment2d_circle2d_intersections(&segment, &circle, STANDARD_TEST_TOLERANCE_F64);
        assert_eq!(points.len(), 1);
        assert!((points[0].x() + 1.0).abs() < STANDARD_TEST_TOLERANCE_F64);
        assert!(!tangent);
    }

    #[test]
    fn small_segment_and_circle_apart_do_not_intersect() {
        // 判別式（長さの 4 乗）を距離の許容誤差と比べると、小さな線分・円では離れていても交点ありになる
        let segment =
            LineSegment2D::new(Point2D::new(-0.0005, 0.0), Point2D::new(0.0005, 0.0)).unwrap();
        let circle = Circle2D::new(Point2D::new(0.0, -0.0002), 0.0001).unwrap();

        let (points, _) =
            line_segment2d_circle2d_intersections(&segment, &circle, STANDARD_TEST_TOLERANCE_F64);
        assert!(points.is_empty());
    }

    #[test]
    fn line_segment_arc_filters_outside_angle_range() {
        let segment = LineSegment2D::new(Point2D::new(-2.0, 0.0), Point2D::new(2.0, 0.0)).unwrap();
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        let arc = Arc2D::new(
            circle.center(),
            circle.radius(),
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::PI * 0.5),
        )
        .unwrap();

        let points =
            line_segment2d_arc2d_intersections(&segment, &arc, STANDARD_TEST_TOLERANCE_F64);
        assert_eq!(points.len(), 1);
    }
}
