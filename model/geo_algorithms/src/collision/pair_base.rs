//! 衝突判定のペアbase実装
//!
//! 型ごとの trait実装とは分離し、形状ペア単位の衝突判定を集約する。

use crate::intersection::pair_base::{
    infinite_line3d_infinite_line3d_intersection, infinite_line3d_line_segment3d_intersection,
    infinite_line3d_ray3d_intersection, infinite_line3d_spherical_surface3d_intersections,
    line_segment3d_line_segment3d_intersection, line_segment3d_spherical_surface3d_intersections,
    plane3d_infinite_line3d_intersection, plane3d_line_segment3d_intersection,
    plane3d_ray3d_intersection, ray3d_infinite_line3d_intersection,
    ray3d_line_segment3d_intersection, ray3d_ray3d_intersection,
    ray3d_spherical_surface3d_intersections,
};
use crate::intersection::primitive_2d::{
    infinite_line2d_line_segment2d_intersection as infinite_line2d_line_segment2d_intersection_2d,
    infinite_line2d_ray2d_intersection as infinite_line2d_ray2d_intersection_2d,
    ray2d_line_segment2d_intersection as ray2d_line_segment2d_intersection_2d,
};
use crate::{
    InfiniteLine2D, InfiniteLine3D, LineSegment2D, LineSegment3D, Plane3D, Ray2D, Ray3D,
    SphericalSurface3D,
};
use geo_contracts::Scalar;

pub fn ray2d_line_segment2d_collides<T: Scalar>(
    ray: &Ray2D<T>,
    segment: &LineSegment2D<T>,
    tolerance: T,
) -> bool {
    ray2d_line_segment2d_intersection_2d(ray, segment, tolerance).intersects()
}

pub fn line_segment2d_ray2d_collides<T: Scalar>(
    segment: &LineSegment2D<T>,
    ray: &Ray2D<T>,
    tolerance: T,
) -> bool {
    ray2d_line_segment2d_collides(ray, segment, tolerance)
}

pub fn infinite_line2d_line_segment2d_collides<T: Scalar>(
    line: &InfiniteLine2D<T>,
    segment: &LineSegment2D<T>,
    tolerance: T,
) -> bool {
    infinite_line2d_line_segment2d_intersection_2d(line, segment, tolerance).intersects()
}

pub fn line_segment2d_infinite_line2d_collides<T: Scalar>(
    segment: &LineSegment2D<T>,
    line: &InfiniteLine2D<T>,
    tolerance: T,
) -> bool {
    infinite_line2d_line_segment2d_collides(line, segment, tolerance)
}

pub fn infinite_line2d_ray2d_collides<T: Scalar>(
    line: &InfiniteLine2D<T>,
    ray: &Ray2D<T>,
    tolerance: T,
) -> bool {
    infinite_line2d_ray2d_intersection_2d(line, ray, tolerance).intersects()
}

pub fn ray2d_infinite_line2d_collides<T: Scalar>(
    ray: &Ray2D<T>,
    line: &InfiniteLine2D<T>,
    tolerance: T,
) -> bool {
    infinite_line2d_ray2d_collides(line, ray, tolerance)
}

// collision 判定は intersection 側の判定ロジックを正本として再利用し、
// 幾何条件の二重実装を避ける。
pub fn line_segment3d_spherical_surface3d_collides<T: Scalar>(
    segment: &LineSegment3D<T>,
    sphere: &SphericalSurface3D<T>,
) -> bool {
    !line_segment3d_spherical_surface3d_intersections(segment, sphere).is_empty()
}

pub fn infinite_line3d_spherical_surface3d_collides<T: Scalar>(
    line: &InfiniteLine3D<T>,
    sphere: &SphericalSurface3D<T>,
) -> bool {
    !infinite_line3d_spherical_surface3d_intersections(line, sphere).is_empty()
}

pub fn ray3d_spherical_surface3d_collides<T: Scalar>(
    ray: &Ray3D<T>,
    sphere: &SphericalSurface3D<T>,
) -> bool {
    !ray3d_spherical_surface3d_intersections(ray, sphere).is_empty()
}

pub fn line_segment3d_line_segment3d_collides<T: Scalar>(
    segment1: &LineSegment3D<T>,
    segment2: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    // 交点が一意に求まるケースを衝突ありとみなす。
    line_segment3d_line_segment3d_intersection(segment1, segment2, tolerance).is_some()
}

pub fn infinite_line3d_infinite_line3d_collides<T: Scalar>(
    line1: &InfiniteLine3D<T>,
    line2: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    infinite_line3d_infinite_line3d_intersection(line1, line2, tolerance).is_some()
}

pub fn infinite_line3d_line_segment3d_collides<T: Scalar>(
    line: &InfiniteLine3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    infinite_line3d_line_segment3d_intersection(line, segment, tolerance).is_some()
}

pub fn infinite_line3d_ray3d_collides<T: Scalar>(
    line: &InfiniteLine3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    infinite_line3d_ray3d_intersection(line, ray, tolerance).is_some()
}

pub fn ray3d_ray3d_collides<T: Scalar>(ray1: &Ray3D<T>, ray2: &Ray3D<T>, tolerance: T) -> bool {
    // Ray の有効範囲（t >= 0）は intersection 側で判定済み。
    ray3d_ray3d_intersection(ray1, ray2, tolerance).is_some()
}

pub fn ray3d_line_segment3d_collides<T: Scalar>(
    ray: &Ray3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    ray3d_line_segment3d_intersection(ray, segment, tolerance).is_some()
}

pub fn ray3d_infinite_line3d_collides<T: Scalar>(
    ray: &Ray3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    ray3d_infinite_line3d_intersection(ray, line, tolerance).is_some()
}

pub fn plane3d_line_segment3d_collides<T: Scalar>(
    plane: &Plane3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    // tolerance を伴う符号判定は intersection 側に統一する。
    plane3d_line_segment3d_intersection(plane, segment, tolerance).is_some()
}

pub fn plane3d_ray3d_collides<T: Scalar>(plane: &Plane3D<T>, ray: &Ray3D<T>, tolerance: T) -> bool {
    plane3d_ray3d_intersection(plane, ray, tolerance).is_some()
}

pub fn plane3d_infinite_line3d_collides<T: Scalar>(
    plane: &Plane3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    plane3d_infinite_line3d_intersection(plane, line, tolerance).is_some()
}

#[cfg(test)]
mod tests {
    use super::{
        infinite_line3d_ray3d_collides, line_segment3d_line_segment3d_collides,
        line_segment3d_spherical_surface3d_collides, plane3d_ray3d_collides, ray3d_ray3d_collides,
        ray3d_spherical_surface3d_collides,
    };
    use crate::{
        InfiniteLine3D, LineSegment3D, Plane3D, Point3D, Ray3D, SphericalSurface3D, Vector3D,
    };
    use analysis::test_constants;

    const STANDARD_TEST_TOLERANCE_F64: f64 = test_constants::DISTANCE_TOLERANCE_F64;

    #[test]
    fn line_segment3d_spherical_surface_collides_true() {
        let segment =
            LineSegment3D::new(Point3D::new(-2.0, 0.0, 0.0), Point3D::new(2.0, 0.0, 0.0)).unwrap();
        let sphere = SphericalSurface3D::new_standard(Point3D::origin(), 1.0).unwrap();
        assert!(line_segment3d_spherical_surface3d_collides(
            &segment, &sphere
        ));
    }

    #[test]
    fn ray3d_spherical_surface_collides_false() {
        let ray = Ray3D::new(Point3D::new(2.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let sphere = SphericalSurface3D::new_standard(Point3D::origin(), 1.0).unwrap();
        assert!(!ray3d_spherical_surface3d_collides(&ray, &sphere));
    }

    #[test]
    fn line_segment3d_line_segment3d_collides_true() {
        let segment1 =
            LineSegment3D::new(Point3D::new(-1.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0)).unwrap();
        let segment2 =
            LineSegment3D::new(Point3D::new(0.0, -1.0, 0.0), Point3D::new(0.0, 1.0, 0.0)).unwrap();
        assert!(line_segment3d_line_segment3d_collides(
            &segment1,
            &segment2,
            STANDARD_TEST_TOLERANCE_F64
        ));
    }

    #[test]
    fn ray3d_ray3d_collides_true() {
        let ray1 = Ray3D::new(Point3D::new(-1.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let ray2 = Ray3D::new(Point3D::new(0.0, -1.0, 0.0), Vector3D::new(0.0, 1.0, 0.0)).unwrap();
        assert!(ray3d_ray3d_collides(
            &ray1,
            &ray2,
            STANDARD_TEST_TOLERANCE_F64
        ));
    }

    #[test]
    fn infinite_line3d_ray3d_collides_false() {
        let line = InfiniteLine3D::from_two_points(
            Point3D::new(-1.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        let ray = Ray3D::new(Point3D::new(0.0, -2.0, 0.0), Vector3D::new(0.0, -1.0, 0.0)).unwrap();
        assert!(!infinite_line3d_ray3d_collides(
            &line,
            &ray,
            STANDARD_TEST_TOLERANCE_F64
        ));
    }

    #[test]
    fn plane3d_ray3d_collides_true() {
        let plane = Plane3D::xy_plane(0.0_f64);
        let ray = Ray3D::new(Point3D::new(0.0, 0.0, -2.0), Vector3D::new(0.0, 0.0, 1.0)).unwrap();
        assert!(plane3d_ray3d_collides(
            &plane,
            &ray,
            STANDARD_TEST_TOLERANCE_F64
        ));
    }
}
