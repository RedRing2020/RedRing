//! LineSegment3D 交点計算実装
//!
//! BasicIntersection, MultipleIntersection, SelfIntersection トレイトの実装
//! LineSegment3D と他の幾何形状との組み合わせを実装

use crate::{LineSegment3D, Point3D, SphericalSurface3D, Vector3D};
use geo_contracts::{BasicIntersection, MultipleIntersection, SelfIntersection};
use geo_foundation::{Scalar, SphericalSurface3DProperties};

// ============================================================================
// BasicIntersection Implementations
// ============================================================================

// LineSegment3D vs Point3D（単一交点判定）
impl<T: Scalar> BasicIntersection<T, Point3D<T>> for LineSegment3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, point: &Point3D<T>, tolerance: T) -> Option<Self::Point> {
        // 点が線分上にあれば、その点を交点として返す
        if self.contains_point(point, tolerance) {
            Some(*point)
        } else {
            None
        }
    }
}

// LineSegment3D vs SphericalSurface3D（最初の交点のみ）
impl<T: Scalar> BasicIntersection<T, SphericalSurface3D<T>> for LineSegment3D<T> {
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

// LineSegment3D vs LineSegment3D（単一交点）
impl<T: Scalar> BasicIntersection<T, LineSegment3D<T>> for LineSegment3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, other: &LineSegment3D<T>, tolerance: T) -> Option<Self::Point> {
        calculate_line_segment_intersection(self, other, tolerance)
    }
}

// ============================================================================
// MultipleIntersection Implementations
// ============================================================================

// LineSegment3D vs SphericalSurface3D（複数交点、最大2点）
impl<T: Scalar> MultipleIntersection<T, SphericalSurface3D<T>> for LineSegment3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(
        &self,
        sphere: &SphericalSurface3D<T>,
        _tolerance: T,
    ) -> Vec<Self::Point> {
        calculate_line_segment_sphere_intersections(self, sphere)
    }
}

// ============================================================================
// SelfIntersection Implementation
// ============================================================================

impl<T: Scalar> SelfIntersection<T> for LineSegment3D<T> {
    type Point = Point3D<T>;

    fn self_intersections(&self, _tolerance: T) -> Vec<Self::Point> {
        // 線分は自己交差しない
        Vec::new()
    }
}

// ============================================================================
// 幾何計算ヘルパー関数
// ============================================================================

/// 線分と球の交点を計算（最大2点）
fn calculate_line_segment_sphere_intersections<T: Scalar>(
    segment: &LineSegment3D<T>,
    sphere: &SphericalSurface3D<T>,
) -> Vec<Point3D<T>> {
    let mut result = Vec::new();

    let center_tuple = sphere.center();
    let center = Point3D::new(center_tuple.0, center_tuple.1, center_tuple.2);
    let radius = sphere.radius();
    let start = segment.start();
    let end = segment.end();

    let d = Vector3D::from_points(&start, &end);
    let f = Vector3D::from_points(&center, &start);

    // 2次方程式の係数: a*t^2 + b*t + c = 0
    let a = d.dot(&d);
    let b = (f.dot(&d)) * (T::ONE + T::ONE);
    let c = f.dot(&f) - radius * radius;

    let discriminant = b * b - (T::ONE + T::ONE + T::ONE + T::ONE) * a * c;

    if discriminant < T::ZERO {
        // 交点なし
        return result;
    }

    let sqrt_discriminant = discriminant.sqrt();
    let two_a = (T::ONE + T::ONE) * a;

    // t1, t2 を計算（線分上のパラメータ）
    let t1 = (-b - sqrt_discriminant) / two_a;
    let t2 = (-b + sqrt_discriminant) / two_a;

    // t が [0, 1] の範囲内にある交点のみを追加
    if t1 >= T::ZERO && t1 <= T::ONE {
        let point = Point3D::new(
            start.x() + t1 * d.x(),
            start.y() + t1 * d.y(),
            start.z() + t1 * d.z(),
        );
        result.push(point);
    }

    if t2 >= T::ZERO && t2 <= T::ONE && (t2 - t1).abs() > T::EPSILON {
        let point = Point3D::new(
            start.x() + t2 * d.x(),
            start.y() + t2 * d.y(),
            start.z() + t2 * d.z(),
        );
        result.push(point);
    }

    result
}

/// 線分と線分の交点を計算（最大1点、3Dでは通常交差しない）
fn calculate_line_segment_intersection<T: Scalar>(
    seg1: &LineSegment3D<T>,
    seg2: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    // 3Dでは一般に2つの線分は交差しない（同一平面上にある場合のみ）
    // 簡易実装: 最近接点が許容範囲内にあれば交点とみなす

    let p1 = seg1.start();
    let p2 = seg1.end();
    let p3 = seg2.start();
    let p4 = seg2.end();

    let d1 = Vector3D::from_points(&p1, &p2);
    let d2 = Vector3D::from_points(&p3, &p4);
    let r = Vector3D::from_points(&p3, &p1);

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

    if s >= T::ZERO && s <= T::ONE && t >= T::ZERO && t <= T::ONE {
        let point1 = Point3D::new(
            p1.x() + s * d1.x(),
            p1.y() + s * d1.y(),
            p1.z() + s * d1.z(),
        );
        let point2 = Point3D::new(
            p3.x() + t * d2.x(),
            p3.y() + t * d2.y(),
            p3.z() + t * d2.z(),
        );

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
