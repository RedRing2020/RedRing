//! NurbsSurface3D × Primitives 交点計算実装（Stage 1）
//!
//! Stage 1 では Point3D / Plane3D / Ray3D の最小セットを提供する。

use crate::{
    Circle3D, CylindricalSolid3D, EllipsoidalSolid3D, InfiniteLine3D, LineSegment3D, Plane3D,
    Point3D, Ray3D, SphericalSolid3D,
};
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

pub fn nurbssurface3d_line_segment3d_intersection<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let distance = crate::collision::nurbssurface3d_line_segment3d_distance(surface, segment);

    if distance <= tolerance {
        Some(segment.start())
    } else {
        None
    }
}

pub fn nurbssurface3d_infinite_line3d_intersection<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let distance = crate::collision::nurbssurface3d_infinite_line3d_distance(surface, line);

    if distance <= tolerance {
        let (px, py, pz) = geo_contracts::InfiniteLine3DProperties::point(line);
        Some(Point3D::new(px, py, pz))
    } else {
        None
    }
}

pub fn nurbssurface3d_circle3d_intersection<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    circle: &Circle3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let distance = crate::collision::nurbssurface3d_circle3d_distance(surface, circle);

    if distance <= tolerance {
        let (cx, cy, cz) = geo_contracts::Circle3DProperties::center(circle);
        Some(Point3D::new(cx, cy, cz))
    } else {
        None
    }
}

pub fn nurbssurface3d_spherical_solid3d_intersection<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    sphere: &SphericalSolid3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let distance = crate::collision::nurbssurface3d_spherical_solid3d_distance(surface, sphere);

    if distance <= tolerance {
        let ((u_min, _), (v_min, _)) = surface.parameter_domain();
        let p = surface.evaluate_at(u_min, v_min);
        Some(Point3D::new(p.x(), p.y(), p.z()))
    } else {
        None
    }
}

pub fn nurbssurface3d_ellipsoidal_solid3d_intersection<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    ellipsoid: &EllipsoidalSolid3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let distance =
        crate::collision::nurbssurface3d_ellipsoidal_solid3d_distance(surface, ellipsoid);

    if distance <= tolerance {
        let ((u_min, _), (v_min, _)) = surface.parameter_domain();
        let p = surface.evaluate_at(u_min, v_min);
        Some(Point3D::new(p.x(), p.y(), p.z()))
    } else {
        None
    }
}

pub fn nurbssurface3d_cylindrical_solid3d_intersection<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    cylinder: &CylindricalSolid3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let distance = crate::collision::nurbssurface3d_cylindrical_solid3d_distance(surface, cylinder);

    if distance <= tolerance {
        let ((u_min, _), (v_min, _)) = surface.parameter_domain();
        let p = surface.evaluate_at(u_min, v_min);
        Some(Point3D::new(p.x(), p.y(), p.z()))
    } else {
        None
    }
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

    #[test]
    fn test_nurbssurface3d_line_segment3d_intersection_vertical_hit() {
        let surface = create_test_surface::<f64>();
        let segment =
            LineSegment3D::new(Point3D::new(0.5, 0.5, -1.0), Point3D::new(0.5, 0.5, 1.0)).unwrap();
        let result = nurbssurface3d_line_segment3d_intersection(&surface, &segment, 1e-6);
        assert!(result.is_some());
    }

    #[test]
    fn test_nurbssurface3d_infinite_line3d_intersection_vertical_hit() {
        let surface = create_test_surface::<f64>();
        let line = InfiniteLine3D::from_two_points(
            Point3D::new(0.5, 0.5, -1.0),
            Point3D::new(0.5, 0.5, 1.0),
        )
        .unwrap();
        let result = nurbssurface3d_infinite_line3d_intersection(&surface, &line, 1e-6);
        assert!(result.is_some());
    }

    #[test]
    fn test_nurbssurface3d_circle3d_intersection_same_plane() {
        let surface = create_test_surface::<f64>();
        let circle = Circle3D::new(
            Point3D::new(0.5, 0.5, 0.0),
            Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap(),
            0.25,
        )
        .unwrap();
        let result = nurbssurface3d_circle3d_intersection(&surface, &circle, 1e-6);
        assert!(result.is_some());
    }

    #[test]
    fn test_nurbssurface3d_spherical_solid3d_intersection_enclosing() {
        let surface = create_test_surface::<f64>();
        let sphere = SphericalSolid3D::new(
            Point3D::new(0.5, 0.5, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            2.0,
        )
        .unwrap();
        let result = nurbssurface3d_spherical_solid3d_intersection(&surface, &sphere, 1e-6);
        assert!(result.is_some());
    }

    #[test]
    fn test_nurbssurface3d_ellipsoidal_solid3d_intersection_enclosing() {
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
        let result = nurbssurface3d_ellipsoidal_solid3d_intersection(&surface, &ellipsoid, 1e-6);
        assert!(result.is_some());
    }

    #[test]
    fn test_nurbssurface3d_cylindrical_solid3d_intersection_enclosing() {
        let surface = create_test_surface::<f64>();
        let cylinder = CylindricalSolid3D::new(
            Point3D::new(0.5, 0.5, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            2.0,
            4.0,
        )
        .unwrap();
        let result = nurbssurface3d_cylindrical_solid3d_intersection(&surface, &cylinder, 1e-6);
        assert!(result.is_some());
    }
}
