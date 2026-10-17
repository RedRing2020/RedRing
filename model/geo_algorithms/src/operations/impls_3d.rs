//! 3D の形状をまたぐ演算の実装

use super::{CrossCollision, CrossDistance, CrossIntersection};
use crate::intersection::primitive_3d as intersection;
use crate::{
    Aabb3D, ConicalSolid3D, ConicalSurface3D, InfiniteLine3D, IntersectionResult, LineSegment3D,
    Plane3D, Point3D, Ray3D, Scalar, SphericalSolid3D, SphericalSurface3D, Triangle3D, Vector3D,
};
use geo_contracts::{
    default_parallel_cross_error_tolerance, InfiniteLine3DProperties, LineSegment3DProperties,
};

/// 交点の有無で衝突判定し、逆の向きは代表の向きを呼ぶ
macro_rules! collision_by_intersection {
    ($a:ident, $b:ident) => {
        impl<T: Scalar> CrossCollision<T, $b<T>> for $a<T> {
            fn collides(&self, other: &$b<T>, tolerance: T) -> bool {
                self.intersection(other, tolerance).intersects()
            }
        }

        impl<T: Scalar> CrossCollision<T, $a<T>> for $b<T> {
            fn collides(&self, other: &$a<T>, tolerance: T) -> bool {
                other.collides(self, tolerance)
            }
        }
    };
}

/// `$a` から `$b` への交差を `$function` で書く
macro_rules! intersection {
    ($a:ident, $b:ident, $function:path) => {
        impl<T: Scalar> CrossIntersection<T, $b<T>> for $a<T> {
            fn intersection(&self, other: &$b<T>, tolerance: T) -> IntersectionResult<T> {
                $function(self, other, tolerance)
            }
        }
    };
}

/// 代表の向き（`$a` と `$b`）の交差を `$function` で書き、逆の向きはそれを呼ぶ
macro_rules! symmetric_intersection {
    ($a:ident, $b:ident, $function:path) => {
        intersection!($a, $b, $function);

        impl<T: Scalar> CrossIntersection<T, $a<T>> for $b<T> {
            fn intersection(&self, other: &$a<T>, tolerance: T) -> IntersectionResult<T> {
                other.intersection(self, tolerance)
            }
        }
    };
}

collision_by_intersection!(InfiniteLine3D, LineSegment3D);
collision_by_intersection!(InfiniteLine3D, Ray3D);

impl<T: Scalar> CrossDistance<T, InfiniteLine3D<T>> for InfiniteLine3D<T> {
    fn distance_to(&self, other: &InfiniteLine3D<T>) -> T {
        let direction = self.direction().as_vector();
        let other_direction = other.direction().as_vector();
        let offset = Vector3D::from_points(self.point(), other.point());
        let normal = direction.cross(other_direction);
        // 単位方向ベクトル同士の外積の長さで平行かを判定する
        let parallel_tolerance = default_parallel_cross_error_tolerance::<T>();
        if normal.length_squared() <= parallel_tolerance * parallel_tolerance {
            return offset.cross(direction).length();
        }
        offset.dot(normal).abs() / normal.length()
    }
}

impl<T: Scalar> CrossDistance<T, Aabb3D<T>> for LineSegment3D<T> {
    fn distance_to(&self, aabb: &Aabb3D<T>) -> T {
        let start = self.start_point();
        let end = self.end_point();
        let (min, max) = (aabb.min(), aabb.max());
        geo_commons::line_segment_to_aabb_distance(
            (start.x(), start.y(), start.z()),
            (end.x(), end.y(), end.z()),
            (min.x(), min.y(), min.z()),
            (max.x(), max.y(), max.z()),
        )
    }
}

intersection!(
    InfiniteLine3D,
    LineSegment3D,
    intersection::infinite_line3d_line_segment3d_intersection
);
symmetric_intersection!(
    InfiniteLine3D,
    Ray3D,
    intersection::infinite_line3d_ray3d_intersection
);
intersection!(
    InfiniteLine3D,
    InfiniteLine3D,
    intersection::infinite_line3d_infinite_line3d_intersection
);
intersection!(
    LineSegment3D,
    LineSegment3D,
    intersection::line_segment3d_line_segment3d_intersection
);
intersection!(Ray3D, Ray3D, intersection::ray3d_ray3d_intersection);
intersection!(
    Plane3D,
    LineSegment3D,
    intersection::plane3d_line_segment3d_intersection
);
intersection!(Plane3D, Ray3D, intersection::plane3d_ray3d_intersection);
symmetric_intersection!(
    Triangle3D,
    LineSegment3D,
    intersection::triangle3d_line_segment3d_intersection
);
symmetric_intersection!(
    Triangle3D,
    Ray3D,
    intersection::triangle3d_ray3d_intersection
);
intersection!(
    ConicalSolid3D,
    Point3D,
    intersection::conical_solid3d_point3d_intersection
);
intersection!(
    ConicalSolid3D,
    InfiniteLine3D,
    intersection::conical_solid3d_infinite_line3d_intersection
);
intersection!(
    ConicalSolid3D,
    Ray3D,
    intersection::conical_solid3d_ray3d_intersection
);
intersection!(
    ConicalSolid3D,
    LineSegment3D,
    intersection::conical_solid3d_line_segment3d_intersection
);
intersection!(
    ConicalSurface3D,
    Point3D,
    intersection::conical_surface3d_point3d_intersection
);
intersection!(
    ConicalSurface3D,
    InfiniteLine3D,
    intersection::conical_surface3d_infinite_line3d_intersections
);
symmetric_intersection!(
    ConicalSurface3D,
    Ray3D,
    intersection::conical_surface3d_ray3d_intersections
);
intersection!(
    ConicalSurface3D,
    LineSegment3D,
    intersection::conical_surface3d_line_segment3d_intersections
);
intersection!(
    SphericalSolid3D,
    Point3D,
    intersection::spherical_solid3d_point3d_intersection
);
intersection!(
    SphericalSolid3D,
    InfiniteLine3D,
    intersection::spherical_solid3d_infinite_line3d_intersection
);
intersection!(
    SphericalSolid3D,
    Ray3D,
    intersection::spherical_solid3d_ray3d_intersection
);
symmetric_intersection!(
    Ray3D,
    SphericalSurface3D,
    intersection::ray3d_spherical_surface3d_intersections
);
intersection!(
    LineSegment3D,
    SphericalSurface3D,
    intersection::line_segment3d_spherical_surface3d_intersections
);
