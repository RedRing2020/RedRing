//! 交点計算のペアbase実装
//!
//! 型ごとの trait実装とは分離し、形状ペア単位の幾何計算を集約する。

use geo_foundation::{Arc2DProperties, Circle2DProperties, LineSegment2DProperties, Scalar};
use geo_primitives::{Arc2D, Circle2D, LineSegment2D, Point2D};

pub fn line_segment2d_circle2d_intersections<T: Scalar>(
    segment: &LineSegment2D<T>,
    circle: &Circle2D<T>,
) -> Vec<Point2D<T>> {
    let mut result = Vec::new();

    let center = circle.center();
    let radius = circle.radius();
    let start = segment.start();
    let end = segment.end();

    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    let fx = start.0 - center.0;
    let fy = start.1 - center.1;

    let a = dx * dx + dy * dy;
    if a.abs() <= T::EPSILON {
        return result;
    }

    let b = (fx * dx + fy * dy) * (T::ONE + T::ONE);
    let c = fx * fx + fy * fy - radius * radius;

    let discriminant = b * b - (T::ONE + T::ONE + T::ONE + T::ONE) * a * c;
    if discriminant < T::ZERO {
        return result;
    }

    let sqrt_discriminant = discriminant.sqrt();
    let two_a = (T::ONE + T::ONE) * a;

    let t1 = (-b - sqrt_discriminant) / two_a;
    let t2 = (-b + sqrt_discriminant) / two_a;

    if t1 >= T::ZERO && t1 <= T::ONE {
        let x = start.0 + t1 * dx;
        let y = start.1 + t1 * dy;
        result.push(Point2D::new(x, y));
    }

    if t2 >= T::ZERO && t2 <= T::ONE && (t2 - t1).abs() > T::EPSILON {
        let x = start.0 + t2 * dx;
        let y = start.1 + t2 * dy;
        result.push(Point2D::new(x, y));
    }

    result
}

pub fn line_segment2d_arc2d_intersections<T: Scalar>(
    segment: &LineSegment2D<T>,
    arc: &Arc2D<T>,
) -> Vec<Point2D<T>> {
    let base_circle = Circle2D::new(
        Point2D::new(arc.center().0, arc.center().1),
        <Arc2D<T> as Arc2DProperties<T>>::radius(arc),
    );

    if let Some(base_circle) = base_circle {
        line_segment2d_circle2d_intersections(segment, &base_circle)
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
) -> Option<Point2D<T>> {
    let p1 = seg1.start();
    let p2 = seg1.end();
    let p3 = seg2.start();
    let p4 = seg2.end();

    let d1x = p2.0 - p1.0;
    let d1y = p2.1 - p1.1;
    let d2x = p4.0 - p3.0;
    let d2y = p4.1 - p3.1;

    let denominator = d1x * d2y - d1y * d2x;
    if denominator.abs() < T::EPSILON {
        return None;
    }

    let t1 = ((p3.0 - p1.0) * d2y - (p3.1 - p1.1) * d2x) / denominator;
    let t2 = ((p3.0 - p1.0) * d1y - (p3.1 - p1.1) * d1x) / denominator;

    if t1 >= T::ZERO && t1 <= T::ONE && t2 >= T::ZERO && t2 <= T::ONE {
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
        line_segment2d_arc2d_intersections, line_segment2d_circle2d_intersections,
        line_segment2d_line_segment2d_intersection,
    };
    use geo_primitives::{Angle, Arc2D, Circle2D, LineSegment2D, Point2D};

    #[test]
    fn line_segment_circle_returns_two_points() {
        let segment = LineSegment2D::new(Point2D::new(-2.0, 0.0), Point2D::new(2.0, 0.0)).unwrap();
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();

        let points = line_segment2d_circle2d_intersections(&segment, &circle);
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

        let points = line_segment2d_arc2d_intersections(&segment, &arc);
        assert_eq!(points.len(), 1);
    }

    #[test]
    fn line_segment_line_segment_returns_single_intersection() {
        let seg1 = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(2.0, 2.0)).unwrap();
        let seg2 = LineSegment2D::new(Point2D::new(0.0, 2.0), Point2D::new(2.0, 0.0)).unwrap();

        let p = line_segment2d_line_segment2d_intersection(&seg1, &seg2);
        assert!(p.is_some());
    }
}
