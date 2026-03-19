//! SphericalSolid3D の交差判定実装
//!
//! 球ソリッドとの交点計算を提供する。

use crate::{InfiniteLine3D, Plane3D, Point3D, Ray3D, SphericalSolid3D};
use geo_contracts::{BasicIntersection, MultipleIntersection, SelfIntersection};
use geo_foundation::{
    
    Scalar,
};

// ============================================================================
// BasicIntersection implementations for SphericalSolid3D
// ============================================================================

/// SphericalSolid3D と Point3D の交差判定
impl<T: Scalar> BasicIntersection<T, Point3D<T>> for SphericalSolid3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, point: &Point3D<T>, _tolerance: T) -> Option<Self::Point> {
        let center = self.center_internal();
        let to_point = crate::Vector3D::from_points(&center, point);
        let distance = to_point.magnitude();
        let radius = self.radius_internal();

        if distance <= radius {
            // 点が球ソリッド内または表面上にある場合、その点を返す
            Some(*point)
        } else {
            None
        }
    }
}

/// SphericalSolid3D と Plane3D の交差判定
impl<T: Scalar> BasicIntersection<T, Plane3D<T>> for SphericalSolid3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, plane: &Plane3D<T>, _tolerance: T) -> Option<Self::Point> {
        // 簡易実装：中心と平面の交点のみ考慮
        // TODO: 平面と球ソリッドの円交差を返す
        let center = self.center_internal();
        let distance = plane.distance_to_point(center).abs();
        let radius = self.radius_internal();

        if distance <= radius {
            Some(center)
        } else {
            None
        }
    }
}

// ============================================================================
// MultipleIntersection implementations for SphericalSolid3D
// ============================================================================

/// SphericalSolid3D と InfiniteLine3D の複数交差判定
impl<T: Scalar> MultipleIntersection<T, InfiniteLine3D<T>> for SphericalSolid3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, line: &InfiniteLine3D<T>, _tolerance: T) -> Vec<Self::Point> {
        use geo_foundation::InfiniteLine3DProperties;

        let center = self.center_internal();
        let radius = self.radius_internal();
        let (px, py, pz) = line.point();
        let point_on_line = Point3D::new(px, py, pz);
        let (dx, dy, dz) = line.direction();
        let direction = crate::Vector3D::new(dx, dy, dz);

        // 直線のパラメトリック表現: P = point_on_line + t * direction
        let to_center = crate::Vector3D::from_points(&point_on_line, &center);

        let a = direction.dot(&direction);
        let b = direction.dot(&to_center) * (-T::ONE - T::ONE);
        let c = to_center.dot(&to_center) - radius * radius;

        let discriminant = b * b - (T::ONE + T::ONE + T::ONE + T::ONE) * a * c;

        if discriminant < T::ZERO {
            vec![]
        } else if discriminant.is_zero() {
            let t = -b / ((T::ONE + T::ONE) * a);
            let point = point_on_line + direction * t;
            vec![point]
        } else {
            let sqrt_disc = discriminant.sqrt();
            let t1 = (-b - sqrt_disc) / ((T::ONE + T::ONE) * a);
            let t2 = (-b + sqrt_disc) / ((T::ONE + T::ONE) * a);

            let point1 = point_on_line + direction * t1;
            let point2 = point_on_line + direction * t2;

            vec![point1, point2]
        }
    }
}

/// SphericalSolid3D と Ray3D の複数交差判定
impl<T: Scalar> MultipleIntersection<T, Ray3D<T>> for SphericalSolid3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, ray: &Ray3D<T>, _tolerance: T) -> Vec<Self::Point> {
        let center = self.center_internal();
        let radius = self.radius_internal();
        let origin = ray.origin();
        let direction = ray.direction_vector();

        let to_center = crate::Vector3D::from_points(&origin, &center);

        let a = direction.dot(&direction);
        let b = direction.dot(&to_center) * (-T::ONE - T::ONE);
        let c = to_center.dot(&to_center) - radius * radius;

        let discriminant = b * b - (T::ONE + T::ONE + T::ONE + T::ONE) * a * c;

        if discriminant < T::ZERO {
            return vec![];
        }

        let sqrt_disc = discriminant.sqrt();
        let t1 = (-b - sqrt_disc) / ((T::ONE + T::ONE) * a);
        let t2 = (-b + sqrt_disc) / ((T::ONE + T::ONE) * a);

        let mut points = Vec::new();

        if t1 >= T::ZERO {
            let point1 = origin + direction * t1;
            points.push(point1);
        }

        if discriminant.is_zero() {
            return points;
        }

        if t2 >= T::ZERO {
            let point2 = origin + direction * t2;
            points.push(point2);
        }

        points
    }
}

// ============================================================================
// SelfIntersection implementations for SphericalSolid3D
// ============================================================================

/// SphericalSolid3D と SphericalSolid3D の自己交差判定
impl<T: Scalar> SelfIntersection<T> for SphericalSolid3D<T> {
    type Point = Point3D<T>;

    fn self_intersections(&self, _tolerance: T) -> Vec<Self::Point> {
        // 球ソリッド単体では自己交差は発生しない
        vec![]
    }
}
