//! 線形形状（Ray / 線分 / 無限直線）を含む衝突判定
//!
//! 線形形状同士および球面との衝突判定は、対応する交点計算の公開エントリポイントへ委譲し、
//! 交点が存在するかどうかで判定する。

use super::planar_and_mesh_family::{
    plane3d_infinite_line3d_collides, plane3d_line_segment3d_collides, plane3d_ray3d_collides,
};
use crate::intersection::primitive_3d::{
    infinite_line3d_infinite_line3d_intersection, infinite_line3d_line_segment3d_intersection,
    infinite_line3d_ray3d_intersection, infinite_line3d_spherical_surface3d_intersections,
    line_segment3d_infinite_line3d_intersection, line_segment3d_line_segment3d_intersection,
    line_segment3d_ray3d_intersection, line_segment3d_spherical_surface3d_intersections,
    ray3d_infinite_line3d_intersection, ray3d_line_segment3d_intersection,
    ray3d_ray3d_intersection, ray3d_spherical_surface3d_intersections,
};
use crate::{InfiniteLine3D, LineSegment3D, Plane3D, Point3D, Ray3D, SphericalSurface3D};
use geo_contracts::Scalar;

pub fn ray3d_point3d_collides<T: Scalar>(ray: &Ray3D<T>, point: &Point3D<T>, tolerance: T) -> bool {
    ray.contains_point(point, tolerance)
}

pub fn ray3d_spherical_surface3d_collides<T: Scalar>(
    ray: &Ray3D<T>,
    sphere: &SphericalSurface3D<T>,
    tolerance: T,
) -> bool {
    ray3d_spherical_surface3d_intersections(ray, sphere, tolerance).intersects()
}

pub fn ray3d_ray3d_collides<T: Scalar>(ray_a: &Ray3D<T>, ray_b: &Ray3D<T>, tolerance: T) -> bool {
    ray3d_ray3d_intersection(ray_a, ray_b, tolerance).intersects()
}

pub fn ray3d_line_segment3d_collides<T: Scalar>(
    ray: &Ray3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    ray3d_line_segment3d_intersection(ray, segment, tolerance).intersects()
}

pub fn ray3d_infinite_line3d_collides<T: Scalar>(
    ray: &Ray3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    ray3d_infinite_line3d_intersection(ray, line, tolerance).intersects()
}

pub fn ray3d_plane3d_collides<T: Scalar>(ray: &Ray3D<T>, plane: &Plane3D<T>, tolerance: T) -> bool {
    plane3d_ray3d_collides(plane, ray, tolerance)
}

pub fn line_segment3d_point3d_collides<T: Scalar>(
    segment: &LineSegment3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    segment.contains_point(point, tolerance)
}

pub fn line_segment3d_spherical_surface3d_collides<T: Scalar>(
    segment: &LineSegment3D<T>,
    sphere: &SphericalSurface3D<T>,
    tolerance: T,
) -> bool {
    line_segment3d_spherical_surface3d_intersections(segment, sphere, tolerance).intersects()
}

pub fn line_segment3d_line_segment3d_collides<T: Scalar>(
    seg_a: &LineSegment3D<T>,
    seg_b: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    line_segment3d_line_segment3d_intersection(seg_a, seg_b, tolerance).intersects()
}

pub fn line_segment3d_ray3d_collides<T: Scalar>(
    segment: &LineSegment3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    line_segment3d_ray3d_intersection(segment, ray, tolerance).intersects()
}

pub fn line_segment3d_infinite_line3d_collides<T: Scalar>(
    segment: &LineSegment3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    line_segment3d_infinite_line3d_intersection(segment, line, tolerance).intersects()
}

pub fn line_segment3d_plane3d_collides<T: Scalar>(
    segment: &LineSegment3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> bool {
    plane3d_line_segment3d_collides(plane, segment, tolerance)
}

pub fn infinite_line3d_point3d_collides<T: Scalar>(
    line: &InfiniteLine3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    line.contains_point(point, tolerance)
}

pub fn infinite_line3d_spherical_surface3d_collides<T: Scalar>(
    line: &InfiniteLine3D<T>,
    sphere: &SphericalSurface3D<T>,
    tolerance: T,
) -> bool {
    infinite_line3d_spherical_surface3d_intersections(line, sphere, tolerance).intersects()
}

pub fn infinite_line3d_infinite_line3d_collides<T: Scalar>(
    line_a: &InfiniteLine3D<T>,
    line_b: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    infinite_line3d_infinite_line3d_intersection(line_a, line_b, tolerance).intersects()
}

pub fn infinite_line3d_line_segment3d_collides<T: Scalar>(
    line: &InfiniteLine3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    infinite_line3d_line_segment3d_intersection(line, segment, tolerance).intersects()
}

pub fn infinite_line3d_ray3d_collides<T: Scalar>(
    line: &InfiniteLine3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    infinite_line3d_ray3d_intersection(line, ray, tolerance).intersects()
}

pub fn infinite_line3d_plane3d_collides<T: Scalar>(
    line: &InfiniteLine3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> bool {
    plane3d_infinite_line3d_collides(plane, line, tolerance)
}
