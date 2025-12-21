//! CylindricalSolid3D - Intersection Implementation
//!
//! 3次元円柱ソリッドの交差判定実装

use crate::{
    Circle3D, CylindricalSolid3D, InfiniteLine3D, LineSegment3D, Plane3D, Point3D, Triangle3D,
};
use geo_foundation::{
    extensions::{BasicIntersection, MultipleIntersection, SelfIntersection},
    Scalar,
};

// ============================================================================
// BasicIntersection Implementations
// ============================================================================

impl<T: Scalar> BasicIntersection<T, Point3D<T>> for CylindricalSolid3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, point: &Point3D<T>, tolerance: T) -> Option<Point3D<T>> {
        use geo_foundation::extensions::BasicCollision;
        if self.intersects(point, tolerance) {
            Some(*point)
        } else {
            None
        }
    }
}

impl<T: Scalar> BasicIntersection<T, Circle3D<T>> for CylindricalSolid3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, circle: &Circle3D<T>, tolerance: T) -> Option<Point3D<T>> {
        use geo_foundation::{extensions::BasicCollision, Circle3DProperties};
        if self.intersects(circle, tolerance) {
            let (cx, cy, cz) = circle.center();
            Some(Point3D::new(cx, cy, cz))
        } else {
            None
        }
    }
}

impl<T: Scalar> BasicIntersection<T, LineSegment3D<T>> for CylindricalSolid3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, segment: &LineSegment3D<T>, tolerance: T) -> Option<Point3D<T>> {
        use geo_foundation::extensions::BasicCollision;
        if self.intersects(&segment.start(), tolerance) {
            Some(segment.start())
        } else if self.intersects(&segment.end(), tolerance) {
            Some(segment.end())
        } else {
            None
        }
    }
}

impl<T: Scalar> BasicIntersection<T, Triangle3D<T>> for CylindricalSolid3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, triangle: &Triangle3D<T>, tolerance: T) -> Option<Point3D<T>> {
        use geo_foundation::{extensions::BasicCollision, Triangle3DProperties};

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

impl<T: Scalar> BasicIntersection<T, InfiniteLine3D<T>> for CylindricalSolid3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, line: &InfiniteLine3D<T>, tolerance: T) -> Option<Point3D<T>> {
        use geo_foundation::extensions::BasicCollision;
        let ref_point = line.point_at_parameter(T::ZERO);
        if self.intersects(&ref_point, tolerance) {
            Some(ref_point)
        } else {
            None
        }
    }
}

impl<T: Scalar> BasicIntersection<T, Plane3D<T>> for CylindricalSolid3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, plane: &Plane3D<T>, tolerance: T) -> Option<Point3D<T>> {
        use geo_foundation::extensions::BasicCollision;
        if self.intersects(plane, tolerance) {
            // 簡易実装：円柱の中心点を返す
            Some(self.center_internal())
        } else {
            None
        }
    }
}

// ============================================================================
// MultipleIntersection Implementations
// ============================================================================

impl<T: Scalar> MultipleIntersection<T, Point3D<T>> for CylindricalSolid3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, point: &Point3D<T>, tolerance: T) -> Vec<Point3D<T>> {
        use geo_foundation::extensions::BasicIntersection;
        if let Some(pt) = self.intersection_with(point, tolerance) {
            vec![pt]
        } else {
            vec![]
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, Circle3D<T>> for CylindricalSolid3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, circle: &Circle3D<T>, tolerance: T) -> Vec<Point3D<T>> {
        use geo_foundation::extensions::BasicIntersection;
        if let Some(pt) = self.intersection_with(circle, tolerance) {
            vec![pt]
        } else {
            vec![]
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, LineSegment3D<T>> for CylindricalSolid3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, segment: &LineSegment3D<T>, tolerance: T) -> Vec<Point3D<T>> {
        use geo_foundation::extensions::BasicCollision;
        let mut results = Vec::new();

        if self.intersects(&segment.start(), tolerance) {
            results.push(segment.start());
        }
        if self.intersects(&segment.end(), tolerance) {
            results.push(segment.end());
        }

        results
    }
}

impl<T: Scalar> MultipleIntersection<T, Triangle3D<T>> for CylindricalSolid3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, triangle: &Triangle3D<T>, tolerance: T) -> Vec<Point3D<T>> {
        use geo_foundation::{extensions::BasicCollision, Triangle3DProperties};
        let mut results = Vec::new();

        let (ax, ay, az) = triangle.vertex_a();
        let (bx, by, bz) = triangle.vertex_b();
        let (cx, cy, cz) = triangle.vertex_c();
        let va = Point3D::new(ax, ay, az);
        let vb = Point3D::new(bx, by, bz);
        let vc = Point3D::new(cx, cy, cz);

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

impl<T: Scalar> MultipleIntersection<T, InfiniteLine3D<T>> for CylindricalSolid3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, line: &InfiniteLine3D<T>, tolerance: T) -> Vec<Point3D<T>> {
        use geo_foundation::extensions::BasicIntersection;
        if let Some(pt) = self.intersection_with(line, tolerance) {
            vec![pt]
        } else {
            vec![]
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, Plane3D<T>> for CylindricalSolid3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, plane: &Plane3D<T>, tolerance: T) -> Vec<Point3D<T>> {
        use geo_foundation::extensions::BasicIntersection;
        if let Some(pt) = self.intersection_with(plane, tolerance) {
            vec![pt]
        } else {
            vec![]
        }
    }
}

impl<T: Scalar> MultipleIntersection<T, CylindricalSolid3D<T>> for CylindricalSolid3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, other: &CylindricalSolid3D<T>, tolerance: T) -> Vec<Point3D<T>> {
        use geo_foundation::extensions::BasicCollision;
        if self.intersects(other, tolerance) {
            // 簡易実装：両方の中心点の中間点を返す
            let center1 = self.center_internal();
            let center2 = other.center_internal();
            let mid_x = (center1.x() + center2.x()) / T::from_f64(2.0);
            let mid_y = (center1.y() + center2.y()) / T::from_f64(2.0);
            let mid_z = (center1.z() + center2.z()) / T::from_f64(2.0);
            vec![Point3D::new(mid_x, mid_y, mid_z)]
        } else {
            vec![]
        }
    }
}

// ============================================================================
// SelfIntersection Implementation
// ============================================================================

impl<T: Scalar> SelfIntersection<T> for CylindricalSolid3D<T> {
    type Point = Point3D<T>;

    fn self_intersections(&self, _tolerance: T) -> Vec<Point3D<T>> {
        // 円柱ソリッドは自己交差しない
        vec![]
    }
}
