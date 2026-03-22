//! NurbsSurface3D × Primitives 衝突判定実装（Stage 1）
//!
//! Stage 1 では Point3D / Plane3D / Ray3D の最小セットを提供する。

use crate::{Plane3D, Point3D, Ray3D};
use geo_contracts::{BasicCollision, Plane3DProperties, Scalar};
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Vector3D;
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
}
