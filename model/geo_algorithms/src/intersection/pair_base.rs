//! 交点計算のペアbase実装
//!
//! 型ごとの trait実装とは分離し、形状ペア単位の幾何計算を集約する。

use crate::{Arc2D, Circle2D, LineSegment2D, Point2D};
use geo_contracts::{Arc2DProperties, Circle2DProperties, Scalar};

#[cfg(test)]
const STANDARD_TEST_TOLERANCE_F64: f64 = analysis::test_constants::DISTANCE_TOLERANCE_F64;
#[cfg(test)]
const SMALL_GAP_WITHIN_TOLERANCE_F64: f64 = STANDARD_TEST_TOLERANCE_F64 / 10.0;

pub fn circle2d_circle2d_intersections<T: Scalar>(
    circle1: &Circle2D<T>,
    circle2: &Circle2D<T>,
    tolerance: T,
) -> Vec<Point2D<T>> {
    let mut result = Vec::new();

    let center1 = circle1.center();
    let center2 = circle2.center();
    let center1_point = Point2D::from_tuple(center1);
    let center2_point = Point2D::from_tuple(center2);
    let r1 = circle1.radius();
    let r2 = circle2.radius();

    let dx = center2.0 - center1.0;
    let dy = center2.1 - center1.1;
    let d = center1_point.distance_to(&center2_point);

    if d > r1 + r2 + tolerance || d + tolerance < (r1 - r2).abs() || d.abs() <= tolerance {
        return result;
    }

    let a = (r1 * r1 - r2 * r2 + d * d) / ((T::ONE + T::ONE) * d);
    let h_squared = r1 * r1 - a * a;
    if h_squared < -tolerance {
        return result;
    }

    let h_squared = if h_squared.abs() <= tolerance {
        T::ZERO
    } else {
        h_squared
    };

    let h = h_squared.sqrt();
    let px = center1.0 + a * dx / d;
    let py = center1.1 + a * dy / d;

    if h.abs() <= tolerance {
        result.push(Point2D::new(px, py));
    } else {
        result.push(Point2D::new(px + h * dy / d, py - h * dx / d));
        result.push(Point2D::new(px - h * dy / d, py + h * dx / d));
    }

    result
}

pub fn circle2d_line_segment2d_intersections<T: Scalar>(
    circle: &Circle2D<T>,
    segment: &LineSegment2D<T>,
    tolerance: T,
) -> Vec<Point2D<T>> {
    line_segment2d_circle2d_intersections(segment, circle, tolerance)
}

pub fn arc2d_circle2d_intersections<T: Scalar>(
    arc: &Arc2D<T>,
    circle: &Circle2D<T>,
    tolerance: T,
) -> Vec<Point2D<T>> {
    let (center_x, center_y) = <Arc2D<T> as Arc2DProperties<T>>::center(arc);
    let base_circle = Circle2D::new(
        Point2D::new(center_x, center_y),
        <Arc2D<T> as Arc2DProperties<T>>::radius(arc),
    );

    if let Some(base_circle) = base_circle {
        circle2d_circle2d_intersections(&base_circle, circle, tolerance)
            .into_iter()
            .filter(|p| arc.contains_point_angle(*p))
            .collect()
    } else {
        Vec::new()
    }
}

pub fn line_segment2d_circle2d_intersections<T: Scalar>(
    segment: &LineSegment2D<T>,
    circle: &Circle2D<T>,
    tolerance: T,
) -> Vec<Point2D<T>> {
    let mut result = Vec::new();

    let center = circle.center();
    let radius = circle.radius();
    let start = segment.start_point().to_tuple();
    let end = segment.end_point().to_tuple();

    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    let fx = start.0 - center.0;
    let fy = start.1 - center.1;

    let start_point = Point2D::from_tuple(start);
    let end_point = Point2D::from_tuple(end);
    let a = start_point.distance_squared_to(&end_point);
    if a.abs() <= tolerance {
        return result;
    }

    let b = (fx * dx + fy * dy) * (T::ONE + T::ONE);
    let c = fx * fx + fy * fy - radius * radius;

    let discriminant = b * b - (T::ONE + T::ONE + T::ONE + T::ONE) * a * c;
    if discriminant < -tolerance {
        return result;
    }

    let discriminant = if discriminant.abs() <= tolerance {
        T::ZERO
    } else {
        discriminant
    };

    let sqrt_discriminant = discriminant.sqrt();
    let two_a = (T::ONE + T::ONE) * a;

    let t1 = (-b - sqrt_discriminant) / two_a;
    let t2 = (-b + sqrt_discriminant) / two_a;

    if t1 >= -tolerance && t1 <= T::ONE + tolerance {
        let x = start.0 + t1 * dx;
        let y = start.1 + t1 * dy;
        result.push(Point2D::new(x, y));
    }

    if t2 >= -tolerance && t2 <= T::ONE + tolerance && (t2 - t1).abs() > tolerance {
        let x = start.0 + t2 * dx;
        let y = start.1 + t2 * dy;
        result.push(Point2D::new(x, y));
    }

    result
}

pub fn line_segment2d_arc2d_intersections<T: Scalar>(
    segment: &LineSegment2D<T>,
    arc: &Arc2D<T>,
    tolerance: T,
) -> Vec<Point2D<T>> {
    let base_circle = Circle2D::new(
        Point2D::new(arc.center().0, arc.center().1),
        <Arc2D<T> as Arc2DProperties<T>>::radius(arc),
    );

    if let Some(base_circle) = base_circle {
        line_segment2d_circle2d_intersections(segment, &base_circle, tolerance)
            .into_iter()
            .filter(|p| arc.contains_point_angle(*p))
            .collect()
    } else {
        Vec::new()
    }
}

pub fn line_segment2d_line_segment2d_intersection<T: Scalar>(
    seg1: &LineSegment2D<T>,
    seg2: &LineSegment2D<T>,
    tolerance: T,
) -> Option<Point2D<T>> {
    let p1 = seg1.start_point().to_tuple();
    let p2 = seg1.end_point().to_tuple();
    let p3 = seg2.start_point().to_tuple();
    let p4 = seg2.end_point().to_tuple();

    let d1x = p2.0 - p1.0;
    let d1y = p2.1 - p1.1;
    let d2x = p4.0 - p3.0;
    let d2y = p4.1 - p3.1;

    let denominator = d1x * d2y - d1y * d2x;
    if denominator.abs() <= tolerance {
        return None;
    }

    let t1 = ((p3.0 - p1.0) * d2y - (p3.1 - p1.1) * d2x) / denominator;
    let t2 = ((p3.0 - p1.0) * d1y - (p3.1 - p1.1) * d1x) / denominator;

    if t1 >= -tolerance && t1 <= T::ONE + tolerance && t2 >= -tolerance && t2 <= T::ONE + tolerance
    {
        let x = p1.0 + t1 * d1x;
        let y = p1.1 + t1 * d1y;
        Some(Point2D::new(x, y))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{
        arc2d_circle2d_intersections, circle2d_circle2d_intersections,
        circle2d_line_segment2d_intersections, line_segment2d_arc2d_intersections,
        line_segment2d_circle2d_intersections, line_segment2d_line_segment2d_intersection,
    };
    use crate::{Angle, Arc2D, Circle2D, LineSegment2D, Point2D};

    #[test]
    fn circle_circle_returns_two_points() {
        let c1 = Circle2D::new(Point2D::new(0.0, 0.0), 2.0).unwrap();
        let c2 = Circle2D::new(Point2D::new(2.0, 0.0), 2.0).unwrap();

        let points = circle2d_circle2d_intersections(&c1, &c2, super::STANDARD_TEST_TOLERANCE_F64);
        assert_eq!(points.len(), 2);
    }

    #[test]
    fn circle_circle_tangent_with_small_gap_uses_tolerance() {
        let c1 = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        let c2 = Circle2D::new(
            Point2D::new(2.0 + super::SMALL_GAP_WITHIN_TOLERANCE_F64, 0.0),
            1.0,
        )
        .unwrap();

        let points = circle2d_circle2d_intersections(&c1, &c2, super::STANDARD_TEST_TOLERANCE_F64);
        assert_eq!(points.len(), 1);
    }

    #[test]
    fn circle_line_segment_returns_two_points() {
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        let segment = LineSegment2D::new(Point2D::new(-2.0, 0.0), Point2D::new(2.0, 0.0)).unwrap();

        let points = circle2d_line_segment2d_intersections(
            &circle,
            &segment,
            super::STANDARD_TEST_TOLERANCE_F64,
        );
        assert_eq!(points.len(), 2);
    }

    #[test]
    fn line_segment_circle_returns_two_points() {
        let segment = LineSegment2D::new(Point2D::new(-2.0, 0.0), Point2D::new(2.0, 0.0)).unwrap();
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();

        let points = line_segment2d_circle2d_intersections(
            &segment,
            &circle,
            super::STANDARD_TEST_TOLERANCE_F64,
        );
        assert_eq!(points.len(), 2);
    }

    #[test]
    fn line_segment_arc_filters_outside_angle_range() {
        let segment = LineSegment2D::new(Point2D::new(-2.0, 0.0), Point2D::new(2.0, 0.0)).unwrap();
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();
        let arc = Arc2D::new(
            circle,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::PI * 0.5),
        )
        .unwrap();

        let points =
            line_segment2d_arc2d_intersections(&segment, &arc, super::STANDARD_TEST_TOLERANCE_F64);
        assert_eq!(points.len(), 1);
    }

    #[test]
    fn arc_circle_filters_outside_angle_range() {
        let arc_circle = Circle2D::new(Point2D::new(0.0, 0.0), 2.0).unwrap();
        let arc = Arc2D::new(
            arc_circle,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::PI * 0.5),
        )
        .unwrap();
        let circle = Circle2D::new(Point2D::new(2.0, 0.0), 2.0).unwrap();

        let points =
            arc2d_circle2d_intersections(&arc, &circle, super::STANDARD_TEST_TOLERANCE_F64);
        assert_eq!(points.len(), 1);
    }

    #[test]
    fn line_segment_line_segment_returns_single_intersection() {
        let seg1 = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(2.0, 2.0)).unwrap();
        let seg2 = LineSegment2D::new(Point2D::new(0.0, 2.0), Point2D::new(2.0, 0.0)).unwrap();

        let p = line_segment2d_line_segment2d_intersection(
            &seg1,
            &seg2,
            super::STANDARD_TEST_TOLERANCE_F64,
        );
        assert!(p.is_some());
    }
}
