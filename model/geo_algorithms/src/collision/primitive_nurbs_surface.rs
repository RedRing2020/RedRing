//! NurbsSurface3D × Primitives 衝突判定実装（Stage 1）
//!
//! Stage 1 では Point3D / Plane3D / Ray3D の最小セットを提供する。

use crate::{
    Circle3D, CylindricalSolid3D, EllipsoidalSolid3D, InfiniteLine3D, LineSegment3D, Plane3D,
    Point3D, Ray3D, SphericalSolid3D,
};
use geo_contracts::{
    BasicCollision, Circle3DProperties, CylindricalSolid3DMeasure, InfiniteLine3DProperties,
    Plane3DProperties, Scalar,
};
use geo_nurbs::NurbsSurface3D;

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct NurbsSurfaceCollider<T: Scalar>(pub NurbsSurface3D<T>);

impl<T: Scalar> NurbsSurfaceCollider<T> {
    pub fn new(surface: NurbsSurface3D<T>) -> Self {
        Self(surface)
    }

    fn nearest_sample_to_point(&self, point: &Point3D<T>) -> (Point3D<T>, T) {
        let samples_u = 24;
        let samples_v = 24;
        let ((u_min, u_max), (v_min, v_max)) = self.0.parameter_domain();
        let du = (u_max - u_min) / T::from_usize(samples_u);
        let dv = (v_max - v_min) / T::from_usize(samples_v);

        let mut best_point = Point3D::new(T::ZERO, T::ZERO, T::ZERO);
        let mut min_dist = T::INFINITY;

        for i in 0..=samples_u {
            for j in 0..=samples_v {
                let u = u_min + du * T::from_usize(i);
                let v = v_min + dv * T::from_usize(j);
                let p = self.0.evaluate_at(u, v);
                let surface_point = Point3D::new(p.x(), p.y(), p.z());

                let dx = surface_point.x() - point.x();
                let dy = surface_point.y() - point.y();
                let dz = surface_point.z() - point.z();
                let d = (dx * dx + dy * dy + dz * dz).sqrt();

                if d < min_dist {
                    min_dist = d;
                    best_point = surface_point;
                }
            }
        }

        (best_point, min_dist)
    }

    fn min_distance_to_plane(&self, plane: &Plane3D<T>) -> T {
        let samples_u = 24;
        let samples_v = 24;
        let ((u_min, u_max), (v_min, v_max)) = self.0.parameter_domain();
        let du = (u_max - u_min) / T::from_usize(samples_u);
        let dv = (v_max - v_min) / T::from_usize(samples_v);

        let origin = plane.origin();
        let (nx, ny, nz) = <Plane3D<T> as Plane3DProperties<T>>::normal(plane);

        let mut min_dist = T::INFINITY;

        for i in 0..=samples_u {
            for j in 0..=samples_v {
                let u = u_min + du * T::from_usize(i);
                let v = v_min + dv * T::from_usize(j);
                let p = self.0.evaluate_at(u, v);

                let dx = p.x() - origin.x();
                let dy = p.y() - origin.y();
                let dz = p.z() - origin.z();
                let d = (dx * nx + dy * ny + dz * nz).abs();
                min_dist = min_dist.min(d);
            }
        }

        min_dist
    }

    fn min_distance_to_ray(&self, ray: &Ray3D<T>) -> T {
        let samples_u = 24;
        let samples_v = 24;
        let ((u_min, u_max), (v_min, v_max)) = self.0.parameter_domain();
        let du = (u_max - u_min) / T::from_usize(samples_u);
        let dv = (v_max - v_min) / T::from_usize(samples_v);

        let origin = ray.origin();
        let direction = ray.direction_vector();

        let mut min_dist = T::INFINITY;

        for i in 0..=samples_u {
            for j in 0..=samples_v {
                let u = u_min + du * T::from_usize(i);
                let v = v_min + dv * T::from_usize(j);
                let p = self.0.evaluate_at(u, v);

                let to_px = p.x() - origin.x();
                let to_py = p.y() - origin.y();
                let to_pz = p.z() - origin.z();

                let t = (to_px * direction.x() + to_py * direction.y() + to_pz * direction.z())
                    .max(T::ZERO);

                let closest_x = origin.x() + t * direction.x();
                let closest_y = origin.y() + t * direction.y();
                let closest_z = origin.z() + t * direction.z();

                let dx = p.x() - closest_x;
                let dy = p.y() - closest_y;
                let dz = p.z() - closest_z;
                let d = (dx * dx + dy * dy + dz * dz).sqrt();

                min_dist = min_dist.min(d);
            }
        }

        min_dist
    }

    fn min_distance_to_line_segment(&self, segment: &LineSegment3D<T>) -> T {
        let samples_u = 24;
        let samples_v = 24;
        let ((u_min, u_max), (v_min, v_max)) = self.0.parameter_domain();
        let du = (u_max - u_min) / T::from_usize(samples_u);
        let dv = (v_max - v_min) / T::from_usize(samples_v);

        let start = segment.start();
        let end = segment.end();
        let seg_dx = end.x() - start.x();
        let seg_dy = end.y() - start.y();
        let seg_dz = end.z() - start.z();
        let seg_len_sq = seg_dx * seg_dx + seg_dy * seg_dy + seg_dz * seg_dz;

        let mut min_dist = T::INFINITY;

        for i in 0..=samples_u {
            for j in 0..=samples_v {
                let u = u_min + du * T::from_usize(i);
                let v = v_min + dv * T::from_usize(j);
                let p = self.0.evaluate_at(u, v);

                let to_px = p.x() - start.x();
                let to_py = p.y() - start.y();
                let to_pz = p.z() - start.z();

                let t = if seg_len_sq.is_zero() {
                    T::ZERO
                } else {
                    ((to_px * seg_dx + to_py * seg_dy + to_pz * seg_dz) / seg_len_sq)
                        .clamp(T::ZERO, T::ONE)
                };

                let cx = start.x() + t * seg_dx;
                let cy = start.y() + t * seg_dy;
                let cz = start.z() + t * seg_dz;

                let dx = p.x() - cx;
                let dy = p.y() - cy;
                let dz = p.z() - cz;
                let d = (dx * dx + dy * dy + dz * dz).sqrt();
                min_dist = min_dist.min(d);
            }
        }

        min_dist
    }

    fn min_distance_to_infinite_line(&self, line: &InfiniteLine3D<T>) -> T {
        let samples_u = 24;
        let samples_v = 24;
        let ((u_min, u_max), (v_min, v_max)) = self.0.parameter_domain();
        let du = (u_max - u_min) / T::from_usize(samples_u);
        let dv = (v_max - v_min) / T::from_usize(samples_v);

        let (px, py, pz) = <InfiniteLine3D<T> as InfiniteLine3DProperties<T>>::point(line);
        let (dx, dy, dz) = <InfiniteLine3D<T> as InfiniteLine3DProperties<T>>::direction(line);

        let mut min_dist = T::INFINITY;

        for i in 0..=samples_u {
            for j in 0..=samples_v {
                let u = u_min + du * T::from_usize(i);
                let v = v_min + dv * T::from_usize(j);
                let p = self.0.evaluate_at(u, v);

                let tx = p.x() - px;
                let ty = p.y() - py;
                let tz = p.z() - pz;
                let t = tx * dx + ty * dy + tz * dz;

                let cx = px + t * dx;
                let cy = py + t * dy;
                let cz = pz + t * dz;

                let ddx = p.x() - cx;
                let ddy = p.y() - cy;
                let ddz = p.z() - cz;
                let d = (ddx * ddx + ddy * ddy + ddz * ddz).sqrt();
                min_dist = min_dist.min(d);
            }
        }

        min_dist
    }

    fn min_distance_to_circle(&self, circle: &Circle3D<T>) -> T {
        let samples_u = 24;
        let samples_v = 24;
        let ((u_min, u_max), (v_min, v_max)) = self.0.parameter_domain();
        let du = (u_max - u_min) / T::from_usize(samples_u);
        let dv = (v_max - v_min) / T::from_usize(samples_v);

        let (cx, cy, cz) = <Circle3D<T> as Circle3DProperties<T>>::center(circle);
        let radius = <Circle3D<T> as Circle3DProperties<T>>::radius(circle);

        let mut min_dist = T::INFINITY;

        for i in 0..=samples_u {
            for j in 0..=samples_v {
                let u = u_min + du * T::from_usize(i);
                let v = v_min + dv * T::from_usize(j);
                let p = self.0.evaluate_at(u, v);

                let dx = p.x() - cx;
                let dy = p.y() - cy;
                let dz = p.z() - cz;
                let distance_to_center = (dx * dx + dy * dy + dz * dz).sqrt();
                let d = (distance_to_center - radius).abs();
                min_dist = min_dist.min(d);
            }
        }

        min_dist
    }

    fn min_distance_to_spherical_solid(&self, sphere: &SphericalSolid3D<T>) -> T {
        let samples_u = 24;
        let samples_v = 24;
        let ((u_min, u_max), (v_min, v_max)) = self.0.parameter_domain();
        let du = (u_max - u_min) / T::from_usize(samples_u);
        let dv = (v_max - v_min) / T::from_usize(samples_v);

        let mut min_dist = T::INFINITY;

        for i in 0..=samples_u {
            for j in 0..=samples_v {
                let u = u_min + du * T::from_usize(i);
                let v = v_min + dv * T::from_usize(j);
                let p = self.0.evaluate_at(u, v);
                let surface_point = Point3D::new(p.x(), p.y(), p.z());
                min_dist = min_dist.min(sphere.distance_to_surface(surface_point));
            }
        }

        min_dist
    }

    fn min_distance_to_ellipsoidal_solid(&self, ellipsoid: &EllipsoidalSolid3D<T>) -> T {
        let samples_u = 24;
        let samples_v = 24;
        let ((u_min, u_max), (v_min, v_max)) = self.0.parameter_domain();
        let du = (u_max - u_min) / T::from_usize(samples_u);
        let dv = (v_max - v_min) / T::from_usize(samples_v);

        let mut min_dist = T::INFINITY;

        for i in 0..=samples_u {
            for j in 0..=samples_v {
                let u = u_min + du * T::from_usize(i);
                let v = v_min + dv * T::from_usize(j);
                let p = self.0.evaluate_at(u, v);
                let surface_point = Point3D::new(p.x(), p.y(), p.z());

                let d = if ellipsoid.contains_point(&surface_point) {
                    T::ZERO
                } else {
                    ellipsoid.distance_to_surface(&surface_point)
                };
                min_dist = min_dist.min(d);
            }
        }

        min_dist
    }

    fn min_distance_to_cylindrical_solid(&self, cylinder: &CylindricalSolid3D<T>) -> T {
        let samples_u = 24;
        let samples_v = 24;
        let ((u_min, u_max), (v_min, v_max)) = self.0.parameter_domain();
        let du = (u_max - u_min) / T::from_usize(samples_u);
        let dv = (v_max - v_min) / T::from_usize(samples_v);

        let mut min_dist = T::INFINITY;

        for i in 0..=samples_u {
            for j in 0..=samples_v {
                let u = u_min + du * T::from_usize(i);
                let v = v_min + dv * T::from_usize(j);
                let p = self.0.evaluate_at(u, v);

                let d = <CylindricalSolid3D<T> as CylindricalSolid3DMeasure<T>>::distance_to_point(
                    cylinder,
                    (p.x(), p.y(), p.z()),
                );
                min_dist = min_dist.min(d);
            }
        }

        min_dist
    }
}

impl<T: Scalar> BasicCollision<T, Point3D<T>> for NurbsSurfaceCollider<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.distance_to(point) <= tolerance
    }

    fn overlaps(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.intersects(point, tolerance)
    }

    fn distance_to(&self, point: &Point3D<T>) -> T {
        self.nearest_sample_to_point(point).1
    }
}

impl<T: Scalar> BasicCollision<T, Plane3D<T>> for NurbsSurfaceCollider<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, plane: &Plane3D<T>, tolerance: T) -> bool {
        self.distance_to(plane) <= tolerance
    }

    fn overlaps(&self, plane: &Plane3D<T>, tolerance: T) -> bool {
        self.intersects(plane, tolerance)
    }

    fn distance_to(&self, plane: &Plane3D<T>) -> T {
        self.min_distance_to_plane(plane)
    }
}

impl<T: Scalar> BasicCollision<T, Ray3D<T>> for NurbsSurfaceCollider<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, ray: &Ray3D<T>, tolerance: T) -> bool {
        self.distance_to(ray) <= tolerance
    }

    fn overlaps(&self, ray: &Ray3D<T>, tolerance: T) -> bool {
        self.intersects(ray, tolerance)
    }

    fn distance_to(&self, ray: &Ray3D<T>) -> T {
        self.min_distance_to_ray(ray)
    }
}

impl<T: Scalar> BasicCollision<T, LineSegment3D<T>> for NurbsSurfaceCollider<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, segment: &LineSegment3D<T>, tolerance: T) -> bool {
        self.distance_to(segment) <= tolerance
    }

    fn overlaps(&self, segment: &LineSegment3D<T>, tolerance: T) -> bool {
        self.intersects(segment, tolerance)
    }

    fn distance_to(&self, segment: &LineSegment3D<T>) -> T {
        self.min_distance_to_line_segment(segment)
    }
}

impl<T: Scalar> BasicCollision<T, InfiniteLine3D<T>> for NurbsSurfaceCollider<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, line: &InfiniteLine3D<T>, tolerance: T) -> bool {
        self.distance_to(line) <= tolerance
    }

    fn overlaps(&self, line: &InfiniteLine3D<T>, tolerance: T) -> bool {
        self.intersects(line, tolerance)
    }

    fn distance_to(&self, line: &InfiniteLine3D<T>) -> T {
        self.min_distance_to_infinite_line(line)
    }
}

impl<T: Scalar> BasicCollision<T, Circle3D<T>> for NurbsSurfaceCollider<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, circle: &Circle3D<T>, tolerance: T) -> bool {
        self.distance_to(circle) <= tolerance
    }

    fn overlaps(&self, circle: &Circle3D<T>, tolerance: T) -> bool {
        self.intersects(circle, tolerance)
    }

    fn distance_to(&self, circle: &Circle3D<T>) -> T {
        self.min_distance_to_circle(circle)
    }
}

impl<T: Scalar> BasicCollision<T, SphericalSolid3D<T>> for NurbsSurfaceCollider<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, sphere: &SphericalSolid3D<T>, tolerance: T) -> bool {
        self.distance_to(sphere) <= tolerance
    }

    fn overlaps(&self, sphere: &SphericalSolid3D<T>, tolerance: T) -> bool {
        self.intersects(sphere, tolerance)
    }

    fn distance_to(&self, sphere: &SphericalSolid3D<T>) -> T {
        self.min_distance_to_spherical_solid(sphere)
    }
}

impl<T: Scalar> BasicCollision<T, EllipsoidalSolid3D<T>> for NurbsSurfaceCollider<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, ellipsoid: &EllipsoidalSolid3D<T>, tolerance: T) -> bool {
        self.distance_to(ellipsoid) <= tolerance
    }

    fn overlaps(&self, ellipsoid: &EllipsoidalSolid3D<T>, tolerance: T) -> bool {
        self.intersects(ellipsoid, tolerance)
    }

    fn distance_to(&self, ellipsoid: &EllipsoidalSolid3D<T>) -> T {
        self.min_distance_to_ellipsoidal_solid(ellipsoid)
    }
}

impl<T: Scalar> BasicCollision<T, CylindricalSolid3D<T>> for NurbsSurfaceCollider<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, cylinder: &CylindricalSolid3D<T>, tolerance: T) -> bool {
        self.distance_to(cylinder) <= tolerance
    }

    fn overlaps(&self, cylinder: &CylindricalSolid3D<T>, tolerance: T) -> bool {
        self.intersects(cylinder, tolerance)
    }

    fn distance_to(&self, cylinder: &CylindricalSolid3D<T>) -> T {
        self.min_distance_to_cylindrical_solid(cylinder)
    }
}

// Public distance functions shared by intersection module.

pub fn nurbssurface3d_point3d_distance<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    point: &Point3D<T>,
) -> T {
    NurbsSurfaceCollider::new(surface.clone()).distance_to(point)
}

pub fn nurbssurface3d_plane3d_distance<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    plane: &Plane3D<T>,
) -> T {
    NurbsSurfaceCollider::new(surface.clone()).distance_to(plane)
}

pub fn nurbssurface3d_ray3d_distance<T: Scalar>(surface: &NurbsSurface3D<T>, ray: &Ray3D<T>) -> T {
    NurbsSurfaceCollider::new(surface.clone()).distance_to(ray)
}

pub fn nurbssurface3d_line_segment3d_distance<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    segment: &LineSegment3D<T>,
) -> T {
    NurbsSurfaceCollider::new(surface.clone()).distance_to(segment)
}

pub fn nurbssurface3d_infinite_line3d_distance<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    line: &InfiniteLine3D<T>,
) -> T {
    NurbsSurfaceCollider::new(surface.clone()).distance_to(line)
}

pub fn nurbssurface3d_circle3d_distance<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    circle: &Circle3D<T>,
) -> T {
    NurbsSurfaceCollider::new(surface.clone()).distance_to(circle)
}

pub fn nurbssurface3d_spherical_solid3d_distance<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    sphere: &SphericalSolid3D<T>,
) -> T {
    NurbsSurfaceCollider::new(surface.clone()).distance_to(sphere)
}

pub fn nurbssurface3d_ellipsoidal_solid3d_distance<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    ellipsoid: &EllipsoidalSolid3D<T>,
) -> T {
    NurbsSurfaceCollider::new(surface.clone()).distance_to(ellipsoid)
}

pub fn nurbssurface3d_cylindrical_solid3d_distance<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    cylinder: &CylindricalSolid3D<T>,
) -> T {
    NurbsSurfaceCollider::new(surface.clone()).distance_to(cylinder)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Direction3D, Vector3D};
    use geo_contracts::NurbsSurface3DConstructor;

    fn create_test_surface<T: Scalar>() -> NurbsSurface3D<T> {
        <NurbsSurface3D<T> as NurbsSurface3DConstructor<T>>::unit_plane()
    }

    #[test]
    fn test_nurbssurface3d_point3d_distance_zero_on_surface() {
        let surface = create_test_surface::<f64>();
        let point = Point3D::new(0.5, 0.5, 0.0);
        let d = nurbssurface3d_point3d_distance(&surface, &point);
        assert!(d <= 1e-6);
    }

    #[test]
    fn test_nurbssurface3d_plane3d_distance_zero_for_xy_plane() {
        let surface = create_test_surface::<f64>();
        let plane = Plane3D::xy_plane(0.0);
        let d = nurbssurface3d_plane3d_distance(&surface, &plane);
        assert!(d <= 1e-6);
    }

    #[test]
    fn test_nurbssurface3d_ray3d_distance_zero_for_vertical_hit() {
        let surface = create_test_surface::<f64>();
        let ray = Ray3D::new(Point3D::new(0.5, 0.5, -1.0), Vector3D::new(0.0, 0.0, 1.0)).unwrap();
        let d = nurbssurface3d_ray3d_distance(&surface, &ray);
        assert!(d <= 1e-6);
    }

    #[test]
    fn test_nurbssurface3d_line_segment3d_distance_zero_for_vertical_hit() {
        let surface = create_test_surface::<f64>();
        let segment =
            LineSegment3D::new(Point3D::new(0.5, 0.5, -1.0), Point3D::new(0.5, 0.5, 1.0)).unwrap();
        let d = nurbssurface3d_line_segment3d_distance(&surface, &segment);
        assert!(d <= 1e-6);
    }

    #[test]
    fn test_nurbssurface3d_infinite_line3d_distance_zero_for_vertical_hit() {
        let surface = create_test_surface::<f64>();
        let line = InfiniteLine3D::from_two_points(
            Point3D::new(0.5, 0.5, -1.0),
            Point3D::new(0.5, 0.5, 1.0),
        )
        .unwrap();
        let d = nurbssurface3d_infinite_line3d_distance(&surface, &line);
        assert!(d <= 1e-6);
    }

    #[test]
    fn test_nurbssurface3d_circle3d_distance_zero_for_same_plane_circle() {
        let surface = create_test_surface::<f64>();
        let circle = Circle3D::new(
            Point3D::new(0.5, 0.5, 0.0),
            Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap(),
            0.25,
        )
        .unwrap();
        let d = nurbssurface3d_circle3d_distance(&surface, &circle);
        assert!(d <= 1e-6);
    }

    #[test]
    fn test_nurbssurface3d_spherical_solid3d_distance_zero_for_enclosing_sphere() {
        let surface = create_test_surface::<f64>();
        let sphere = SphericalSolid3D::new(
            Point3D::new(0.5, 0.5, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            2.0,
        )
        .unwrap();
        let d = nurbssurface3d_spherical_solid3d_distance(&surface, &sphere);
        assert!(d <= 1e-6);
    }

    #[test]
    fn test_nurbssurface3d_ellipsoidal_solid3d_distance_zero_for_enclosing_ellipsoid() {
        let surface = create_test_surface::<f64>();
        let ellipsoid = EllipsoidalSolid3D::new(
            Point3D::new(0.5, 0.5, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            2.0,
            2.0,
            2.0,
        )
        .unwrap();
        let d = nurbssurface3d_ellipsoidal_solid3d_distance(&surface, &ellipsoid);
        assert!(d <= 1e-6);
    }

    #[test]
    fn test_nurbssurface3d_cylindrical_solid3d_distance_zero_for_enclosing_cylinder() {
        let surface = create_test_surface::<f64>();
        let cylinder = CylindricalSolid3D::new(
            Point3D::new(0.5, 0.5, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            2.0,
            4.0,
        )
        .unwrap();
        let d = nurbssurface3d_cylindrical_solid3d_distance(&surface, &cylinder);
        assert!(d <= 1e-6);
    }
}
