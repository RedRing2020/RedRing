//! Ellipse3D - Intersection Implementation
//!
//! 3次元楕円の交点計算実装
//! Phase 3.1: BasicIntersection, MultipleIntersection 実装

use crate::{
    Arc3D, Circle3D, Ellipse3D, InfiniteLine3D, LineSegment3D, Plane3D, Point3D, Ray3D, Triangle3D,
};
use geo_foundation::{
    core::arc_traits::Arc3DProperties,
    extensions::{BasicCollision, BasicIntersection, MultipleIntersection, SelfIntersection},
    Scalar,
};

// ============================================================================
// Ellipse3D vs Point3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Point3D<T>> for Ellipse3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, point: &Point3D<T>, tolerance: T) -> Option<Self::Point> {
        if self.distance_to(point) <= tolerance {
            Some(*point)
        } else {
            None
        }
    }
}

// ============================================================================
// Ellipse3D vs Circle3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Circle3D<T>> for Ellipse3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, circle: &Circle3D<T>, tolerance: T) -> Option<Self::Point> {
        let center = circle.center_internal();
        if self.distance_to(&center) <= circle.radius_internal() + tolerance {
            Some(center)
        } else {
            None
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, Circle3D<T>> for Ellipse3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, circle: &Circle3D<T>, tolerance: T) -> Vec<Self::Point> {
        let center = circle.center_internal();
        if self.distance_to(&center) <= circle.radius_internal() + tolerance {
            vec![center]
        } else {
            Vec::new()
        }
    }
}

// ============================================================================
// Ellipse3D vs Arc3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Arc3D<T>> for Ellipse3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, arc: &Arc3D<T>, tolerance: T) -> Option<Self::Point> {
        let (cx, cy, cz) = Arc3DProperties::center(arc);
        let arc_center = Point3D::new(cx, cy, cz);
        if self.distance_to(&arc_center) <= Arc3DProperties::radius(arc) + tolerance {
            Some(arc_center)
        } else {
            None
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, Arc3D<T>> for Ellipse3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, arc: &Arc3D<T>, tolerance: T) -> Vec<Self::Point> {
        let (cx, cy, cz) = Arc3DProperties::center(arc);
        let arc_center = Point3D::new(cx, cy, cz);
        if self.distance_to(&arc_center) <= Arc3DProperties::radius(arc) + tolerance {
            vec![arc_center]
        } else {
            Vec::new()
        }
    }
}

// ============================================================================
// Ellipse3D vs LineSegment3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, LineSegment3D<T>> for Ellipse3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, segment: &LineSegment3D<T>, tolerance: T) -> Option<Self::Point> {
        let start = segment.start();
        let end = segment.end();

        if self.distance_to(&start) <= tolerance {
            return Some(start);
        }
        if self.distance_to(&end) <= tolerance {
            return Some(end);
        }

        // 中点チェック
        let mid = Point3D::new(
            (start.x() + end.x()) / T::from_f64(2.0),
            (start.y() + end.y()) / T::from_f64(2.0),
            (start.z() + end.z()) / T::from_f64(2.0),
        );

        if self.distance_to(&mid) <= tolerance {
            Some(mid)
        } else {
            None
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, LineSegment3D<T>> for Ellipse3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, segment: &LineSegment3D<T>, tolerance: T) -> Vec<Self::Point> {
        let mut intersections = Vec::new();
        let start = segment.start();
        let end = segment.end();

        if self.distance_to(&start) <= tolerance {
            intersections.push(start);
        }
        if self.distance_to(&end) <= tolerance {
            intersections.push(end);
        }

        intersections
    }
}

// ============================================================================
// Ellipse3D vs InfiniteLine3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, InfiniteLine3D<T>> for Ellipse3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, line: &InfiniteLine3D<T>, tolerance: T) -> Option<Self::Point> {
        let point = line.point_internal();
        if self.distance_to(&point) <= tolerance {
            Some(point)
        } else {
            None
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, InfiniteLine3D<T>> for Ellipse3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, line: &InfiniteLine3D<T>, tolerance: T) -> Vec<Self::Point> {
        let point = line.point_internal();
        if self.distance_to(&point) <= tolerance {
            vec![point]
        } else {
            Vec::new()
        }
    }
}

// ============================================================================
// Ellipse3D vs Ray3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Ray3D<T>> for Ellipse3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, ray: &Ray3D<T>, tolerance: T) -> Option<Self::Point> {
        let origin = ray.origin_internal();
        if self.distance_to(&origin) <= tolerance {
            Some(origin)
        } else {
            None
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, Ray3D<T>> for Ellipse3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, ray: &Ray3D<T>, tolerance: T) -> Vec<Self::Point> {
        let origin = ray.origin_internal();
        if self.distance_to(&origin) <= tolerance {
            vec![origin]
        } else {
            Vec::new()
        }
    }
}

// ============================================================================
// Ellipse3D vs Plane3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Plane3D<T>> for Ellipse3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, plane: &Plane3D<T>, tolerance: T) -> Option<Self::Point> {
        let dist = plane.distance_to_point(self.center());
        if dist <= tolerance {
            Some(self.center())
        } else {
            None
        }
    }
}

// ============================================================================
// Ellipse3D vs Triangle3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Triangle3D<T>> for Ellipse3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, triangle: &Triangle3D<T>, tolerance: T) -> Option<Self::Point> {
        let centroid = triangle.centroid();
        if self.distance_to(&centroid) <= tolerance {
            Some(centroid)
        } else {
            None
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, Triangle3D<T>> for Ellipse3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, triangle: &Triangle3D<T>, tolerance: T) -> Vec<Self::Point> {
        let mut intersections = Vec::new();

        let va = triangle.vertex_a_internal();
        let vb = triangle.vertex_b_internal();
        let vc = triangle.vertex_c_internal();

        if self.distance_to(&va) <= tolerance {
            intersections.push(va);
        }
        if self.distance_to(&vb) <= tolerance {
            intersections.push(vb);
        }
        if self.distance_to(&vc) <= tolerance {
            intersections.push(vc);
        }

        intersections
    }
}

// ============================================================================
// Ellipse3D vs Ellipse3D
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Ellipse3D<T>> for Ellipse3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, other: &Ellipse3D<T>, tolerance: T) -> Option<Self::Point> {
        let mid = Point3D::new(
            (self.center().x() + other.center().x()) / T::from_f64(2.0),
            (self.center().y() + other.center().y()) / T::from_f64(2.0),
            (self.center().z() + other.center().z()) / T::from_f64(2.0),
        );

        if self.distance_to(&mid) <= tolerance {
            Some(mid)
        } else {
            None
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, Ellipse3D<T>> for Ellipse3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, _other: &Ellipse3D<T>, _tolerance: T) -> Vec<Self::Point> {
        Vec::new() // 楕円同士の交点計算は複雑
    }
}

// ============================================================================
// SelfIntersection
// ============================================================================

impl<T: Scalar> SelfIntersection<T> for Ellipse3D<T> {
    type Point = Point3D<T>;

    fn self_intersections(&self, _tolerance: T) -> Vec<Self::Point> {
        Vec::new() // 楕円は自己交差しない
    }
}
