//! NurbsSurface3D × Primitives 交点計算実装（Stage 1）
//!
//! Stage 1 では Point3D / Plane3D / Ray3D の最小セットを提供する。

use crate::{Plane3D, Point3D, Ray3D};
use geo_contracts::Scalar;
use geo_nurbs::NurbsSurface3D;

fn point_intersection_if<T: Scalar>(point: &Point3D<T>, condition: bool) -> Option<Point3D<T>> {
    if condition {
        Some(Point3D::new(point.x(), point.y(), point.z()))
    } else {
        None
    }
}

pub fn nurbssurface3d_point3d_intersection<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let distance = crate::collision::nurbssurface3d_point3d_distance(surface, point);
    point_intersection_if(point, distance <= tolerance)
}

pub fn nurbssurface3d_plane3d_intersection<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let distance = crate::collision::nurbssurface3d_plane3d_distance(surface, plane);

    if distance <= tolerance {
        Some(plane.origin())
    } else {
        None
    }
}

pub fn nurbssurface3d_ray3d_intersection<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let distance = crate::collision::nurbssurface3d_ray3d_distance(surface, ray);

    if distance <= tolerance {
        Some(ray.origin())
    } else {
        None
    }
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
    fn test_nurbssurface3d_point3d_intersection_on_surface() {
        let surface = create_test_surface::<f64>();
        let point = Point3D::new(0.5, 0.5, 0.0);
        let result = nurbssurface3d_point3d_intersection(&surface, &point, 1e-6);
        assert!(result.is_some());
    }

    #[test]
    fn test_nurbssurface3d_plane3d_intersection_on_same_plane() {
        let surface = create_test_surface::<f64>();
        let plane = Plane3D::xy_plane(0.0);
        let result = nurbssurface3d_plane3d_intersection(&surface, &plane, 1e-6);
        assert!(result.is_some());
    }

    #[test]
    fn test_nurbssurface3d_ray3d_intersection_vertical_hit() {
        let surface = create_test_surface::<f64>();
        let ray = Ray3D::new(Point3D::new(0.5, 0.5, -1.0), Vector3D::new(0.0, 0.0, 1.0)).unwrap();
        let result = nurbssurface3d_ray3d_intersection(&surface, &ray, 1e-6);
        assert!(result.is_some());
    }
}
