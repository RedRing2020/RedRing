//! EllipseArc3D Intersection 実装
//!
//! BasicIntersection, MultipleIntersection, SelfIntersection トレイトの実装
//! 委譲パターンで Ellipse3D の実装を再利用し、角度範囲でフィルタリング

use crate::{Arc3D, Circle3D, Ellipse3D, EllipseArc3D, LineSegment3D, Point3D, Triangle3D};
use geo_foundation::{
    core::{arc_traits::Arc3DProperties, triangle_traits::Triangle3DProperties},
    extensions::{BasicCollision, BasicIntersection, MultipleIntersection, SelfIntersection},
    Circle3DProperties, Scalar,
};

// ============================================================================
// EllipseArc3D vs Point3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Point3D<T>> for EllipseArc3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, point: &Point3D<T>, tolerance: T) -> Option<Self::Point> {
        // 基底楕円との交差を確認し、角度範囲内かチェック
        if self.ellipse().intersection_with(point, tolerance).is_some()
            && self.point_in_angle_range(point, tolerance)
        {
            Some(*point)
        } else {
            None
        }
    }
}

// ============================================================================
// EllipseArc3D vs Circle3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Circle3D<T>> for EllipseArc3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, circle: &Circle3D<T>, tolerance: T) -> Option<Self::Point> {
        // 簡易実装: 円の中心を返す（厳密には交点計算が必要）
        let (cx, cy, cz) = circle.center();
        let center = Point3D::new(cx, cy, cz);
        if self.intersects(circle, tolerance) {
            Some(center)
        } else {
            None
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, Circle3D<T>> for EllipseArc3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, circle: &Circle3D<T>, tolerance: T) -> Vec<Self::Point> {
        // 基底楕円との交点を取得し、角度範囲でフィルタリング
        let ellipse_intersections = self.ellipse().intersections_with(circle, tolerance);

        ellipse_intersections
            .into_iter()
            .filter(|p| self.point_in_angle_range(p, tolerance))
            .collect()
    }
}

// ============================================================================
// EllipseArc3D vs Arc3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Arc3D<T>> for EllipseArc3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, arc: &Arc3D<T>, tolerance: T) -> Option<Self::Point> {
        // 簡易実装: 円弧の中心を返す
        let (cx, cy, cz) = <Arc3D<T> as Arc3DProperties<T>>::center(arc);
        let center = Point3D::new(cx, cy, cz);
        if self.intersects(arc, tolerance) {
            Some(center)
        } else {
            None
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, Arc3D<T>> for EllipseArc3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, _arc: &Arc3D<T>, _tolerance: T) -> Vec<Self::Point> {
        // 簡易実装: 交点計算は未実装
        Vec::new()
    }
}

// ============================================================================
// EllipseArc3D vs Ellipse3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Ellipse3D<T>> for EllipseArc3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, ellipse: &Ellipse3D<T>, tolerance: T) -> Option<Self::Point> {
        // 簡易実装: 楕円の中心を返す
        let center = ellipse.center();
        if self.intersects(ellipse, tolerance) {
            Some(center)
        } else {
            None
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, Ellipse3D<T>> for EllipseArc3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, ellipse: &Ellipse3D<T>, tolerance: T) -> Vec<Self::Point> {
        // 基底楕円との交点を取得し、角度範囲でフィルタリング
        let ellipse_intersections = self.ellipse().intersections_with(ellipse, tolerance);

        ellipse_intersections
            .into_iter()
            .filter(|p| self.point_in_angle_range(p, tolerance))
            .collect()
    }
}

// ============================================================================
// EllipseArc3D vs EllipseArc3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, EllipseArc3D<T>> for EllipseArc3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, other: &EllipseArc3D<T>, tolerance: T) -> Option<Self::Point> {
        // 簡易実装: 他方の中心を返す
        let center = other.center();
        if self.intersects(other, tolerance) {
            Some(center)
        } else {
            None
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, EllipseArc3D<T>> for EllipseArc3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, other: &EllipseArc3D<T>, tolerance: T) -> Vec<Self::Point> {
        // 基底楕円同士の交点を取得し、両方の角度範囲でフィルタリング
        let ellipse_intersections = self
            .ellipse()
            .intersections_with(other.ellipse(), tolerance);

        ellipse_intersections
            .into_iter()
            .filter(|p| {
                self.point_in_angle_range(p, tolerance) && other.point_in_angle_range(p, tolerance)
            })
            .collect()
    }
}

// ============================================================================
// EllipseArc3D vs LineSegment3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, LineSegment3D<T>> for EllipseArc3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, line: &LineSegment3D<T>, tolerance: T) -> Option<Self::Point> {
        // 基底楕円との交差を取得
        self.ellipse().intersection_with(line, tolerance)
    }
}

impl<T: Scalar> MultipleIntersection<T, LineSegment3D<T>> for EllipseArc3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, line: &LineSegment3D<T>, tolerance: T) -> Vec<Self::Point> {
        // 基底楕円との交点を取得し、角度範囲でフィルタリング
        let ellipse_intersections = self.ellipse().intersections_with(line, tolerance);

        ellipse_intersections
            .into_iter()
            .filter(|p| self.point_in_angle_range(p, tolerance))
            .collect()
    }
}

// ============================================================================
// EllipseArc3D vs Triangle3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Triangle3D<T>> for EllipseArc3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, triangle: &Triangle3D<T>, tolerance: T) -> Option<Self::Point> {
        // 簡易実装: 三角形の重心を返す
        let (ax, ay, az) = <Triangle3D<T> as Triangle3DProperties<T>>::vertex_a(triangle);
        let (bx, by, bz) = <Triangle3D<T> as Triangle3DProperties<T>>::vertex_b(triangle);
        let (cx, cy, cz) = <Triangle3D<T> as Triangle3DProperties<T>>::vertex_c(triangle);

        let center_x = (ax + bx + cx) / (T::ONE + T::ONE + T::ONE);
        let center_y = (ay + by + cy) / (T::ONE + T::ONE + T::ONE);
        let center_z = (az + bz + cz) / (T::ONE + T::ONE + T::ONE);
        let center = Point3D::new(center_x, center_y, center_z);

        if BasicCollision::<T, Triangle3D<T>>::intersects(self, triangle, tolerance) {
            Some(center)
        } else {
            None
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, Triangle3D<T>> for EllipseArc3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, _triangle: &Triangle3D<T>, _tolerance: T) -> Vec<Self::Point> {
        // 簡易実装: 交点計算は未実装
        Vec::new()
    }
}

// ============================================================================
// EllipseArc3D Self Intersection
// ============================================================================

impl<T: Scalar> SelfIntersection<T> for EllipseArc3D<T> {
    type Point = Point3D<T>;

    fn self_intersections(&self, tolerance: T) -> Vec<Self::Point> {
        // 楕円弧は自己交差しない
        let _ = tolerance;
        Vec::new()
    }
}
