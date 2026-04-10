use super::{
    arc3d_point3d_collides, circle3d_point3d_collides, cylindrical_solid3d_point3d_collides,
    cylindrical_surface3d_point3d_collides, ellipse3d_point3d_collides,
    infinite_line3d_plane3d_collides, infinite_line3d_point3d_collides,
    line_segment3d_plane3d_collides, line_segment3d_point3d_collides,
    line_segment3d_triangle3d_collides, plane3d_infinite_line3d_collides,
    plane3d_line_segment3d_collides, plane3d_point3d_collides, plane3d_ray3d_collides,
    ray3d_plane3d_collides, ray3d_point3d_collides, ray3d_triangle3d_collides,
    spherical_solid3d_point3d_collides, torus_solid3d_point3d_collides,
    torus_surface3d_point3d_collides, triangle3d_line_segment3d_collides,
    triangle3d_point3d_collides, triangle3d_ray3d_collides, triangle_mesh3d_point3d_collides,
};
use crate::{
    Angle, Arc3D, Circle3D, CylindricalSolid3D, CylindricalSurface3D, Direction3D, Ellipse3D,
    InfiniteLine3D, LineSegment3D, Plane3D, Point3D, Ray3D, SphericalSolid3D, TorusSolid3D,
    TorusSurface3D, Triangle3D, TriangleMesh3D, Vector3D,
};
use analysis::test_constants;

fn standard_distance_tol() -> f64 {
    test_constants::DISTANCE_TOLERANCE_F64
}

#[test]
fn arc_point_collision_checks_angle_range() {
    let tol = standard_distance_tol();
    let arc = Arc3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        2.0,
        Direction3D::new(0.0, 0.0, 1.0).unwrap(),
        Direction3D::new(1.0, 0.0, 0.0).unwrap(),
        Angle::from_radians(0.0),
        Angle::from_radians(std::f64::consts::FRAC_PI_2),
    )
    .unwrap();

    let on_arc = Point3D::new(2.0, 0.0, 0.0);
    let out_of_angle = Point3D::new(-2.0, 0.0, 0.0);

    assert!(arc3d_point3d_collides(&arc, &on_arc, tol));
    assert!(!arc3d_point3d_collides(&arc, &out_of_angle, tol));
}

#[test]
fn ellipse_point_collision_uses_distance() {
    let tol = standard_distance_tol();
    let ellipse = Ellipse3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        3.0,
        2.0,
        Vector3D::new(0.0, 0.0, 1.0),
        Vector3D::new(1.0, 0.0, 0.0),
    )
    .unwrap();

    let on_ellipse = Point3D::new(3.0, 0.0, 0.0);
    let outside_plane = Point3D::new(0.0, 0.0, 0.5);

    assert!(ellipse3d_point3d_collides(&ellipse, &on_ellipse, tol));
    assert!(!ellipse3d_point3d_collides(&ellipse, &outside_plane, tol));
}

#[test]
fn torus_surface_point_collision_uses_surface_distance() {
    let tol = standard_distance_tol();
    let torus = TorusSurface3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Direction3D::new(0.0, 0.0, 1.0).unwrap(),
        Direction3D::new(1.0, 0.0, 0.0).unwrap(),
        3.0,
        1.0,
    )
    .unwrap();

    let on_surface = Point3D::new(4.0, 0.0, 0.0);
    let inside_tube = Point3D::new(3.0, 0.0, 0.0);

    assert!(torus_surface3d_point3d_collides(&torus, &on_surface, tol));
    assert!(!torus_surface3d_point3d_collides(&torus, &inside_tube, tol));
}

#[test]
fn circle_plane_ray_line_point_collisions_use_geometric_checks() {
    let tol = standard_distance_tol();
    let circle = Circle3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Direction3D::new(0.0, 0.0, 1.0).unwrap(),
        2.0,
    )
    .unwrap();
    assert!(circle3d_point3d_collides(
        &circle,
        &Point3D::new(2.0, 0.0, 0.0),
        tol
    ));
    assert!(!circle3d_point3d_collides(
        &circle,
        &Point3D::new(1.0, 0.0, 0.0),
        tol
    ));

    let plane = Plane3D::xy_plane(0.0);
    assert!(plane3d_point3d_collides(
        &plane,
        &Point3D::new(0.0, 0.0, 0.0),
        tol
    ));
    assert!(!plane3d_point3d_collides(
        &plane,
        &Point3D::new(0.0, 0.0, 1.0),
        tol
    ));

    let ray = Ray3D::new(Point3D::new(0.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
    assert!(ray3d_point3d_collides(
        &ray,
        &Point3D::new(2.0, 0.0, 0.0),
        tol
    ));
    assert!(!ray3d_point3d_collides(
        &ray,
        &Point3D::new(-1.0, 0.0, 0.0),
        tol
    ));

    let segment =
        LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0)).unwrap();
    assert!(line_segment3d_point3d_collides(
        &segment,
        &Point3D::new(0.5, 0.0, 0.0),
        tol
    ));
    assert!(!line_segment3d_point3d_collides(
        &segment,
        &Point3D::new(2.0, 0.0, 0.0),
        tol
    ));

    let line =
        InfiniteLine3D::from_two_points(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0))
            .unwrap();
    assert!(infinite_line3d_point3d_collides(
        &line,
        &Point3D::new(10.0, 0.0, 0.0),
        tol
    ));
    assert!(!infinite_line3d_point3d_collides(
        &line,
        &Point3D::new(0.0, 1.0, 0.0),
        tol
    ));
}

#[test]
fn triangle_ellipsoid_and_torus_solid_point_collisions() {
    let tol = standard_distance_tol();
    let tri = Triangle3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0),
        Point3D::new(0.0, 1.0, 0.0),
    )
    .unwrap();
    assert!(triangle3d_point3d_collides(
        &tri,
        &Point3D::new(0.2, 0.2, 0.0),
        tol
    ));
    assert!(!triangle3d_point3d_collides(
        &tri,
        &Point3D::new(0.2, 0.2, 0.4),
        tol
    ));

    let torus_solid = TorusSolid3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Direction3D::new(0.0, 0.0, 1.0).unwrap(),
        Direction3D::new(1.0, 0.0, 0.0).unwrap(),
        3.0,
        1.0,
    )
    .unwrap();
    assert!(torus_solid3d_point3d_collides(
        &torus_solid,
        &Point3D::new(3.0, 0.0, 0.0),
        tol
    ));
    assert!(!torus_solid3d_point3d_collides(
        &torus_solid,
        &Point3D::new(0.0, 0.0, 0.0),
        tol
    ));
}

#[test]
fn spherical_and_cylindrical_point_collisions_use_distance_based_checks() {
    let tol = standard_distance_tol();
    let sphere = SphericalSolid3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Vector3D::new(0.0, 0.0, 1.0),
        Vector3D::new(1.0, 0.0, 0.0),
        2.0,
    )
    .unwrap();
    assert!(spherical_solid3d_point3d_collides(
        &sphere,
        &Point3D::new(1.0, 0.0, 0.0),
        tol
    ));
    assert!(!spherical_solid3d_point3d_collides(
        &sphere,
        &Point3D::new(4.0, 0.0, 0.0),
        tol
    ));

    let cyl_solid = CylindricalSolid3D::new_z_axis(Point3D::new(0.0, 0.0, 0.0), 1.0, 2.0).unwrap();
    assert!(cylindrical_solid3d_point3d_collides(
        &cyl_solid,
        &Point3D::new(0.5, 0.0, 1.0),
        tol
    ));
    assert!(!cylindrical_solid3d_point3d_collides(
        &cyl_solid,
        &Point3D::new(2.0, 0.0, 1.0),
        tol
    ));

    let cyl_surface = CylindricalSurface3D::new_z_axis(Point3D::new(0.0, 0.0, 0.0), 1.0).unwrap();
    assert!(cylindrical_surface3d_point3d_collides(
        &cyl_surface,
        &Point3D::new(1.0, 0.0, 0.5),
        tol
    ));
    assert!(!cylindrical_surface3d_point3d_collides(
        &cyl_surface,
        &Point3D::new(2.0, 0.0, 0.5),
        tol
    ));
}

#[test]
fn triangle_mesh_point_collision_checks_member_triangles() {
    let tol = standard_distance_tol();
    let mesh = TriangleMesh3D::new(
        vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        ],
        vec![[0, 1, 2]],
    )
    .unwrap();

    assert!(triangle_mesh3d_point3d_collides(
        &mesh,
        &Point3D::new(0.2, 0.2, 0.0),
        tol
    ));
    assert!(!triangle_mesh3d_point3d_collides(
        &mesh,
        &Point3D::new(0.2, 0.2, 0.3),
        tol
    ));
}

#[test]
fn symmetric_triangle_collision_wrappers_match_base_functions() {
    let tol = standard_distance_tol();
    let tri = Triangle3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0),
        Point3D::new(0.0, 1.0, 0.0),
    )
    .unwrap();
    let seg =
        LineSegment3D::new(Point3D::new(0.2, 0.2, -1.0), Point3D::new(0.2, 0.2, 1.0)).unwrap();
    let ray = Ray3D::new(Point3D::new(0.2, 0.2, 1.0), Vector3D::new(0.0, 0.0, -1.0)).unwrap();

    assert_eq!(
        line_segment3d_triangle3d_collides(&seg, &tri, tol),
        triangle3d_line_segment3d_collides(&tri, &seg, tol)
    );
    assert_eq!(
        ray3d_triangle3d_collides(&ray, &tri, tol),
        triangle3d_ray3d_collides(&tri, &ray, tol)
    );
}

#[test]
fn symmetric_plane_collision_wrappers_match_base_functions() {
    let tol = standard_distance_tol();
    let plane = Plane3D::xy_plane(0.0_f64);
    let ray = Ray3D::new(Point3D::new(0.0, 0.0, 1.0), Vector3D::new(0.0, 0.0, -1.0)).unwrap();
    let seg =
        LineSegment3D::new(Point3D::new(0.0, 0.0, -1.0), Point3D::new(0.0, 0.0, 1.0)).unwrap();
    let line =
        InfiniteLine3D::from_two_points(Point3D::new(0.0, 0.0, -1.0), Point3D::new(0.0, 0.0, 1.0))
            .unwrap();

    assert_eq!(
        ray3d_plane3d_collides(&ray, &plane, tol),
        plane3d_ray3d_collides(&plane, &ray, tol)
    );
    assert_eq!(
        line_segment3d_plane3d_collides(&seg, &plane, tol),
        plane3d_line_segment3d_collides(&plane, &seg, tol)
    );
    assert_eq!(
        infinite_line3d_plane3d_collides(&line, &plane, tol),
        plane3d_infinite_line3d_collides(&plane, &line, tol)
    );
}
