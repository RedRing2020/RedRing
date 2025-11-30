//! Ray3D 交点計算実装
//!
//! BasicIntersection, MultipleIntersection, SelfIntersection トレイトの実装

use crate::{InfiniteLine3D, LineSegment3D, Point3D, Ray3D, SphericalSurface3D, Vector3D};
use geo_foundation::{
    extensions::{BasicIntersection, MultipleIntersection, SelfIntersection},
    Scalar, SphericalSurface3DProperties,
};

// ============================================================================
// BasicIntersection Implementations
// ============================================================================

// Ray3D vs Point3D
impl<T: Scalar> BasicIntersection<T, Point3D<T>> for Ray3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, point: &Point3D<T>, tolerance: T) -> Option<Self::Point> {
        if self.contains_point(point, tolerance) {
            Some(*point)
        } else {
            None
        }
    }
}

// Ray3D vs SphericalSurface3D（最初の交点のみ）
impl<T: Scalar> BasicIntersection<T, SphericalSurface3D<T>> for Ray3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(
        &self,
        sphere: &SphericalSurface3D<T>,
        tolerance: T,
    ) -> Option<Self::Point> {
        let intersections =
            <Self as MultipleIntersection<T, SphericalSurface3D<T>>>::intersections_with(
                self, sphere, tolerance,
            );
        intersections.into_iter().next()
    }
}

// Ray3D vs Ray3D（単一交点）
impl<T: Scalar> BasicIntersection<T, Ray3D<T>> for Ray3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, other: &Ray3D<T>, tolerance: T) -> Option<Self::Point> {
        calculate_ray_intersection(self, other, tolerance)
    }
}

// ============================================================================
// MultipleIntersection Implementations
// ============================================================================

// Ray3D vs SphericalSurface3D（複数交点、最大2点）
impl<T: Scalar> MultipleIntersection<T, SphericalSurface3D<T>> for Ray3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(
        &self,
        sphere: &SphericalSurface3D<T>,
        _tolerance: T,
    ) -> Vec<Self::Point> {
        let center_tuple = sphere.center();
        let center = Point3D::new(center_tuple.0, center_tuple.1, center_tuple.2);
        let origin = self.origin();
        let direction = self.direction_vector();

        // Ray と球の交点計算
        let oc = origin - center;

        let a = direction.dot(&direction);
        let b = (oc.dot(&direction)) * (T::ONE + T::ONE);
        let c = oc.dot(&oc) - sphere.radius() * sphere.radius();

        let discriminant = b * b - (T::ONE + T::ONE + T::ONE + T::ONE) * a * c;

        if discriminant < T::ZERO {
            return Vec::new();
        }

        let sqrt_disc = discriminant.sqrt();
        let two_a = (T::ONE + T::ONE) * a;

        let t1 = (-b - sqrt_disc) / two_a;
        let t2 = (-b + sqrt_disc) / two_a;

        let mut intersections = Vec::new();

        if t1 >= T::ZERO {
            let point = self.point_at_parameter(t1);
            intersections.push(point);
        }

        if t2 >= T::ZERO && (t2 - t1).abs() > T::EPSILON {
            let point = self.point_at_parameter(t2);
            intersections.push(point);
        }

        intersections
    }
}

// Ray3D vs LineSegment3D（単一交点）
impl<T: Scalar> BasicIntersection<T, LineSegment3D<T>> for Ray3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, segment: &LineSegment3D<T>, tolerance: T) -> Option<Self::Point> {
        calculate_ray_line_segment_intersection(self, segment, tolerance)
    }
}

// Ray3D vs InfiniteLine3D（単一交点）
impl<T: Scalar> BasicIntersection<T, InfiniteLine3D<T>> for Ray3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, line: &InfiniteLine3D<T>, tolerance: T) -> Option<Self::Point> {
        calculate_ray_infinite_line_intersection(self, line, tolerance)
    }
}

// ============================================================================
// SelfIntersection Implementation
// ============================================================================

impl<T: Scalar> SelfIntersection<T> for Ray3D<T> {
    type Point = Point3D<T>;

    fn self_intersections(&self, _tolerance: T) -> Vec<Self::Point> {
        Vec::new() // Rayは自己交差しない
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Ray と Ray の交点を計算（3Dでは通常交差しない）
fn calculate_ray_intersection<T: Scalar>(
    ray1: &Ray3D<T>,
    ray2: &Ray3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let p1 = ray1.origin();
    let p2 = ray2.origin();
    let d1 = ray1.direction_vector();
    let d2 = ray2.direction_vector();

    let r = Vector3D::from_points(&p2, &p1);

    let a = d1.dot(&d1);
    let b = d1.dot(&d2);
    let c = d2.dot(&d2);
    let d = d1.dot(&r);
    let e = d2.dot(&r);

    let denom = a * c - b * b;

    if denom.abs() < T::EPSILON {
        // 平行
        return None;
    }

    let s = (b * e - c * d) / denom;
    let t = (a * e - b * d) / denom;

    if s >= T::ZERO && t >= T::ZERO {
        let point1 = ray1.point_at_parameter(s);
        let point2 = ray2.point_at_parameter(t);

        // 2点が許容範囲内に近ければ交点とみなす
        if point1.distance_to(&point2) <= tolerance {
            Some(point1)
        } else {
            None
        }
    } else {
        None
    }
}

/// Ray3D と LineSegment3D の交点を計算
fn calculate_ray_line_segment_intersection<T: Scalar>(
    ray: &Ray3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    // Rayを無限直線に変換
    let ray_direction = ray.direction_vector();
    let ray_line = InfiniteLine3D::new(ray.origin(), ray_direction)?;

    // 線分を無限直線に変換
    let segment_line = segment.line();

    // 平行またはスキュー直線の場合は交点なし
    if ray_line.is_parallel_to(segment_line) || !ray_line.is_coplanar_with(segment_line) {
        return None;
    }

    // 無限直線同士の交点を計算
    if let Some(point) = ray_line.intersection_with_line(segment_line) {
        // 交点がRayの範囲内かつ線分の範囲内かチェック
        if ray.contains_point(&point, tolerance) && segment.contains_point(&point, tolerance) {
            return Some(point);
        }
    }

    None
}

/// Ray3D と InfiniteLine3D の交点を計算
fn calculate_ray_infinite_line_intersection<T: Scalar>(
    ray: &Ray3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    // Rayを無限直線に変換
    let ray_direction = ray.direction_vector();
    let ray_line = InfiniteLine3D::new(ray.origin(), ray_direction)?;

    // 平行またはスキュー直線の場合は交点なし
    if ray_line.is_parallel_to(line) || !ray_line.is_coplanar_with(line) {
        return None;
    }

    // 無限直線同士の交点を計算
    if let Some(point) = ray_line.intersection_with_line(line) {
        // 交点がRayの範囲内かチェック（t ≥ 0）
        if ray.contains_point(&point, tolerance) {
            return Some(point);
        }
    }

    None
}
