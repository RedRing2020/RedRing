//! InfiniteLine3D 交点計算実装
//!
//! BasicIntersection, MultipleIntersection, SelfIntersection トレイトの実装

use crate::{InfiniteLine3D, LineSegment3D, Point3D, Ray3D, SphericalSurface3D, Vector3D};
use geo_contracts::Scalar;
use geo_contracts::{
    BasicIntersection, MultipleIntersection, SelfIntersection, SphericalSurface3DProperties,
};

// ============================================================================
// BasicIntersection Implementations
// ============================================================================

// InfiniteLine3D vs Point3D
impl<T: Scalar> BasicIntersection<T, Point3D<T>> for InfiniteLine3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, point: &Point3D<T>, tolerance: T) -> Option<Self::Point> {
        if self.contains_point(point, tolerance) {
            Some(*point)
        } else {
            None
        }
    }
}

// InfiniteLine3D vs SphericalSurface3D（最初の交点のみ）
impl<T: Scalar> BasicIntersection<T, SphericalSurface3D<T>> for InfiniteLine3D<T> {
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

// InfiniteLine3D vs InfiniteLine3D（単一交点）
impl<T: Scalar> BasicIntersection<T, InfiniteLine3D<T>> for InfiniteLine3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, other: &InfiniteLine3D<T>, tolerance: T) -> Option<Self::Point> {
        calculate_infinite_line_intersection(self, other, tolerance)
    }
}

// ============================================================================
// MultipleIntersection Implementations
// ============================================================================

// InfiniteLine3D vs SphericalSurface3D（複数交点、最大2点）
impl<T: Scalar> MultipleIntersection<T, SphericalSurface3D<T>> for InfiniteLine3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(
        &self,
        sphere: &SphericalSurface3D<T>,
        _tolerance: T,
    ) -> Vec<Self::Point> {
        let center_tuple = sphere.center();
        let center = Point3D::new(center_tuple.0, center_tuple.1, center_tuple.2);
        let projected = self.project_point(&center);

        let dist_to_center = self.distance_to_point(&center);

        if dist_to_center > sphere.radius() {
            return Vec::new();
        }

        if (dist_to_center - sphere.radius()).abs() <= T::EPSILON {
            // 接する場合
            return vec![projected];
        }

        // 2点で交わる場合
        let half_chord =
            (sphere.radius() * sphere.radius() - dist_to_center * dist_to_center).sqrt();
        let dir = self.direction_internal();

        let p1 = Point3D::new(
            projected.x() + half_chord * dir.x(),
            projected.y() + half_chord * dir.y(),
            projected.z() + half_chord * dir.z(),
        );
        let p2 = Point3D::new(
            projected.x() - half_chord * dir.x(),
            projected.y() - half_chord * dir.y(),
            projected.z() - half_chord * dir.z(),
        );

        vec![p1, p2]
    }
}

// InfiniteLine3D vs LineSegment3D（単一交点）
impl<T: Scalar> BasicIntersection<T, LineSegment3D<T>> for InfiniteLine3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, segment: &LineSegment3D<T>, tolerance: T) -> Option<Self::Point> {
        calculate_infinite_line_line_segment_intersection(self, segment, tolerance)
    }
}

// InfiniteLine3D vs Ray3D（単一交点）
impl<T: Scalar> BasicIntersection<T, Ray3D<T>> for InfiniteLine3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, ray: &Ray3D<T>, tolerance: T) -> Option<Self::Point> {
        calculate_infinite_line_ray_intersection(self, ray, tolerance)
    }
}

// ============================================================================
// SelfIntersection Implementation
// ============================================================================

impl<T: Scalar> SelfIntersection<T> for InfiniteLine3D<T> {
    type Point = Point3D<T>;

    fn self_intersections(&self, _tolerance: T) -> Vec<Self::Point> {
        Vec::new() // 無限直線は自己交差しない
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// InfiniteLine と InfiniteLine の交点を計算
fn calculate_infinite_line_intersection<T: Scalar>(
    line1: &InfiniteLine3D<T>,
    line2: &InfiniteLine3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    // 平行な場合は交点なし
    if line1.is_parallel_to(line2) {
        return None;
    }

    // スキュー直線の場合は交点なし
    if !line1.is_coplanar_with(line2) {
        return None;
    }

    // 同一平面上で非平行な2直線の交点を計算
    let p1 = line1.point_internal();
    let p2 = line2.point_internal();
    let d1 = Vector3D::new(
        line1.direction_internal().x(),
        line1.direction_internal().y(),
        line1.direction_internal().z(),
    );
    let d2 = Vector3D::new(
        line2.direction_internal().x(),
        line2.direction_internal().y(),
        line2.direction_internal().z(),
    );

    let r = Vector3D::from_points(&p2, &p1);

    let a = d1.dot(&d1);
    let b = d1.dot(&d2);
    let c = d2.dot(&d2);
    let d = d1.dot(&r);
    let e = d2.dot(&r);

    let denom = a * c - b * b;

    if denom.abs() < T::EPSILON {
        return None;
    }

    let s = (b * e - c * d) / denom;
    let t = (a * e - b * d) / denom;

    let point1 = line1.point_at_parameter(s);
    let point2 = line2.point_at_parameter(t);

    // 2点が許容範囲内に近ければ交点とみなす
    if point1.distance_to(&point2) <= tolerance {
        Some(point1)
    } else {
        None
    }
}

/// InfiniteLine3D と LineSegment3D の交点を計算
fn calculate_infinite_line_line_segment_intersection<T: Scalar>(
    line: &InfiniteLine3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    // 線分を無限直線に変換
    let segment_line = segment.line();

    // 平行またはスキュー直線の場合は交点なし
    if line.is_parallel_to(segment_line) || !line.is_coplanar_with(segment_line) {
        return None;
    }

    // 無限直線同士の交点を計算
    if let Some(point) = line.intersection_with_line(segment_line) {
        // 交点が線分の範囲内かチェック
        if segment.contains_point(&point, tolerance) {
            return Some(point);
        }
    }

    None
}

/// InfiniteLine3D と Ray3D の交点を計算
fn calculate_infinite_line_ray_intersection<T: Scalar>(
    line: &InfiniteLine3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    // Rayを無限直線に変換
    let ray_direction = ray.direction_vector();
    let ray_line = InfiniteLine3D::new(ray.origin(), ray_direction)?;

    // 平行またはスキュー直線の場合は交点なし
    if line.is_parallel_to(&ray_line) || !line.is_coplanar_with(&ray_line) {
        return None;
    }

    // 無限直線同士の交点を計算
    if let Some(point) = line.intersection_with_line(&ray_line) {
        // 交点がRayの範囲内かチェック（t ≥ 0）
        if ray.contains_point(&point, tolerance) {
            return Some(point);
        }
    }

    None
}
