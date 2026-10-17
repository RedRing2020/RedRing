//! 2D の線分と円・円弧の交点

use crate::{Arc2D, Circle2D, LineSegment2D, Point2D, Vector2D};
use geo_contracts::{
    Arc2DProperties, Arc2DTrimRange, Circle2DConstructor, Circle2DProperties,
    LineSegment2DProperties, Scalar,
};

pub fn line_segment2d_circle2d_intersections<T: Scalar>(
    segment: &LineSegment2D<T>,
    circle: &Circle2D<T>,
    tolerance: T,
) -> Vec<Point2D<T>> {
    let mut result = Vec::new();

    let center = circle.center();
    let radius = circle.radius();
    let start = segment.start_point();
    let direction = Vector2D::from_points(start, segment.end_point());
    let offset = Vector2D::from_points(center, start);

    let a = direction.length_squared();
    if a.abs() <= tolerance {
        return result;
    }

    let b = offset.dot(direction) * (T::ONE + T::ONE);
    let c = offset.length_squared() - radius * radius;

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
        result.push(start + direction * t1);
    }

    if t2 >= -tolerance && t2 <= T::ONE + tolerance && (t2 - t1).abs() > tolerance {
        result.push(start + direction * t2);
    }

    result
}

pub fn line_segment2d_arc2d_intersections<T: Scalar>(
    segment: &LineSegment2D<T>,
    arc: &Arc2D<T>,
    tolerance: T,
) -> Vec<Point2D<T>> {
    let base_circle = Circle2D::new(arc.center(), <Arc2D<T> as Arc2DProperties<T>>::radius(arc));

    if let Some(base_circle) = base_circle {
        line_segment2d_circle2d_intersections(segment, &base_circle, tolerance)
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

        let points =
            line_segment2d_circle2d_intersections(&segment, &circle, STANDARD_TEST_TOLERANCE_F64);
        assert_eq!(points.len(), 2);
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
