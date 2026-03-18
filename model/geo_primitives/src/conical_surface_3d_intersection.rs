//! ConicalSurface3D - Intersection Implementation
//!
//! 3次元円錐サーフェスの交差判定実装

use crate::{
    Circle3D, ConicalSurface3D, InfiniteLine3D, LineSegment3D, Plane3D, Point3D, Triangle3D,
};
use geo_contracts::{BasicIntersection, MultipleIntersection, SelfIntersection};
use geo_foundation::{
    
    Scalar,
};

// ============================================================================
// BasicIntersection Implementations
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Point3D<T>> for ConicalSurface3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, point: &Point3D<T>, tolerance: T) -> Option<Point3D<T>> {
        use geo_contracts::BasicCollision;
        if self.intersects(point, tolerance) {
            Some(*point)
        } else {
            None
        }
    }
}

impl<T: Scalar> BasicIntersection<T, Circle3D<T>> for ConicalSurface3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, circle: &Circle3D<T>, tolerance: T) -> Option<Point3D<T>> {
        use geo_contracts::BasicCollision;
        use geo_foundation::Circle3DProperties;
        if self.intersects(circle, tolerance) {
            let (cx, cy, cz) = circle.center();
            Some(Point3D::new(cx, cy, cz))
        } else {
            None
        }
    }
}

impl<T: Scalar> BasicIntersection<T, LineSegment3D<T>> for ConicalSurface3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, segment: &LineSegment3D<T>, tolerance: T) -> Option<Point3D<T>> {
        use geo_contracts::BasicCollision;
        if self.intersects(&segment.start(), tolerance) {
            Some(segment.start())
        } else if self.intersects(&segment.end(), tolerance) {
            Some(segment.end())
        } else {
            None
        }
    }
}

impl<T: Scalar> BasicIntersection<T, InfiniteLine3D<T>> for ConicalSurface3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, line: &InfiniteLine3D<T>, tolerance: T) -> Option<Point3D<T>> {
        use geo_contracts::BasicCollision;
        let ref_point = line.point_at_parameter(T::ZERO);
        if self.intersects(&ref_point, tolerance) {
            Some(ref_point)
        } else {
            None
        }
    }
}

impl<T: Scalar> BasicIntersection<T, Triangle3D<T>> for ConicalSurface3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, triangle: &Triangle3D<T>, tolerance: T) -> Option<Point3D<T>> {
        use geo_contracts::BasicCollision;
        use geo_foundation::Triangle3DProperties;

        let (ax, ay, az) = triangle.vertex_a();
        let (bx, by, bz) = triangle.vertex_b();
        let (cx, cy, cz) = triangle.vertex_c();
        let va = Point3D::new(ax, ay, az);
        let vb = Point3D::new(bx, by, bz);
        let vc = Point3D::new(cx, cy, cz);

        if self.intersects(&va, tolerance) {
            Some(va)
        } else if self.intersects(&vb, tolerance) {
            Some(vb)
        } else if self.intersects(&vc, tolerance) {
            Some(vc)
        } else {
            None
        }
    }
}

impl<T: Scalar> BasicIntersection<T, Plane3D<T>> for ConicalSurface3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, plane: &Plane3D<T>, tolerance: T) -> Option<Point3D<T>> {
        use geo_contracts::BasicCollision;
        if self.intersects(plane, tolerance) {
            Some(self.center_internal())
        } else {
            None
        }
    }
}

// ============================================================================
// MultipleIntersection Implementations
// ============================================================================

impl<T: Scalar> MultipleIntersection<T, Point3D<T>> for ConicalSurface3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, point: &Point3D<T>, tolerance: T) -> Vec<Point3D<T>> {
        use geo_contracts::BasicIntersection;
        if let Some(pt) = self.intersection_with(point, tolerance) {
            vec![pt]
        } else {
            vec![]
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, Circle3D<T>> for ConicalSurface3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, circle: &Circle3D<T>, tolerance: T) -> Vec<Point3D<T>> {
        use geo_contracts::BasicIntersection;
        if let Some(pt) = self.intersection_with(circle, tolerance) {
            vec![pt]
        } else {
            vec![]
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, LineSegment3D<T>> for ConicalSurface3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, segment: &LineSegment3D<T>, tolerance: T) -> Vec<Point3D<T>> {
        use geo_contracts::BasicIntersection;
        if let Some(pt) = self.intersection_with(segment, tolerance) {
            vec![pt]
        } else {
            vec![]
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, InfiniteLine3D<T>> for ConicalSurface3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, line: &InfiniteLine3D<T>, tolerance: T) -> Vec<Point3D<T>> {
        use geo_contracts::BasicIntersection;
        if let Some(pt) = self.intersection_with(line, tolerance) {
            vec![pt]
        } else {
            vec![]
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, Triangle3D<T>> for ConicalSurface3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, triangle: &Triangle3D<T>, tolerance: T) -> Vec<Point3D<T>> {
        use geo_contracts::BasicCollision;
        use geo_foundation::Triangle3DProperties;

        let (ax, ay, az) = triangle.vertex_a();
        let (bx, by, bz) = triangle.vertex_b();
        let (cx, cy, cz) = triangle.vertex_c();
        let va = Point3D::new(ax, ay, az);
        let vb = Point3D::new(bx, by, bz);
        let vc = Point3D::new(cx, cy, cz);

        let mut results = Vec::new();
        if self.intersects(&va, tolerance) {
            results.push(va);
        }
        if self.intersects(&vb, tolerance) {
            results.push(vb);
        }
        if self.intersects(&vc, tolerance) {
            results.push(vc);
        }
        results
    }
}

impl<T: Scalar> MultipleIntersection<T, Plane3D<T>> for ConicalSurface3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, plane: &Plane3D<T>, tolerance: T) -> Vec<Point3D<T>> {
        use geo_contracts::BasicIntersection;
        if let Some(pt) = self.intersection_with(plane, tolerance) {
            vec![pt]
        } else {
            vec![]
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, ConicalSurface3D<T>> for ConicalSurface3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, other: &ConicalSurface3D<T>, tolerance: T) -> Vec<Point3D<T>> {
        use geo_contracts::BasicCollision;
        if self.intersects(other, tolerance) {
            vec![self.center_internal()]
        } else {
            vec![]
        }
    }
}

// ============================================================================
// SelfIntersection Implementation
// ============================================================================

impl<T: Scalar> SelfIntersection<T> for ConicalSurface3D<T> {
    type Point = Point3D<T>;

    fn self_intersections(&self, _tolerance: T) -> Vec<Point3D<T>> {
        // 単一の円錐サーフェスは自己交差しない
        vec![]
    }
}
