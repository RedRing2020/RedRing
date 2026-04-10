use super::shared::point_matches_either_segment_endpoint;
use super::{
    arc3d_infinite_line3d_intersection, arc3d_line_segment3d_intersection,
    arc3d_point3d_intersection, arc3d_ray3d_intersection, circle3d_infinite_line3d_intersection,
    circle3d_line_segment3d_intersection, circle3d_point3d_intersection,
    circle3d_ray3d_intersection, cylindrical_surface3d_point3d_intersection,
    ellipse3d_point3d_intersection, infinite_line3d_line_segment3d_intersection,
    infinite_line3d_point3d_intersection, infinite_line3d_spherical_surface3d_intersections,
    line_segment3d_infinite_line3d_intersection, line_segment3d_plane3d_intersection,
    line_segment3d_point3d_intersection, line_segment3d_ray3d_intersection,
    line_segment3d_spherical_surface3d_intersections, line_segment3d_triangle3d_intersection,
    plane3d_line_segment3d_intersection, plane3d_point3d_intersection, plane3d_ray3d_intersection,
    ray3d_line_segment3d_intersection, ray3d_plane3d_intersection, ray3d_point3d_intersection,
    ray3d_ray3d_intersection, ray3d_spherical_surface3d_intersections,
    ray3d_triangle3d_intersection, torus_surface3d_point3d_intersection,
    triangle3d_line_segment3d_intersection, triangle3d_point3d_intersection,
    triangle3d_ray3d_intersection, triangle_mesh3d_point3d_intersection,
};
use crate::{
    Angle, Arc3D, Circle3D, CylindricalSurface3D, Direction3D, Ellipse3D, InfiniteLine3D,
    IntersectionGeometry, IntersectionTopology, LineSegment3D, Plane3D, Point3D, Ray3D,
    SphericalSurface3D, TorusSurface3D, Triangle3D, TriangleMesh3D, Vector3D,
};
use analysis::test_constants;

fn standard_distance_tol() -> f64 {
    test_constants::DISTANCE_TOLERANCE_F64
}

#[test]
fn plane_point_intersection_returns_same_point() {
    let plane = Plane3D::xy_plane(0.0_f64);
    let point = Point3D::new(1.0, -2.0, 0.0);

    let result = plane3d_point3d_intersection(&plane, &point, standard_distance_tol());

    assert_eq!(result.topology, IntersectionTopology::Crossing);
    assert!(matches!(result.geometry, IntersectionGeometry::Point(p) if p == point));
}

#[test]
fn ray_point_intersection_respects_ray_direction() {
    let ray = Ray3D::new(Point3D::new(0.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
    let on_ray = Point3D::new(2.0, 0.0, 0.0);
    let behind_ray = Point3D::new(-1.0, 0.0, 0.0);

    let on = ray3d_point3d_intersection(&ray, &on_ray, standard_distance_tol());
    let behind = ray3d_point3d_intersection(&ray, &behind_ray, standard_distance_tol());
    assert_eq!(on.topology, IntersectionTopology::Crossing);
    assert!(matches!(on.geometry, IntersectionGeometry::Point(p) if p == on_ray));
    assert_eq!(behind.topology, IntersectionTopology::Disjoint);
}

#[test]
fn line_segment_point_intersection_checks_segment_bounds() {
    let segment =
        LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(2.0, 0.0, 0.0)).unwrap();
    let on_segment = Point3D::new(1.0, 0.0, 0.0);
    let outside_segment = Point3D::new(3.0, 0.0, 0.0);

    let on = line_segment3d_point3d_intersection(&segment, &on_segment, standard_distance_tol());
    let outside =
        line_segment3d_point3d_intersection(&segment, &outside_segment, standard_distance_tol());
    assert_eq!(on.topology, IntersectionTopology::Crossing);
    assert!(matches!(on.geometry, IntersectionGeometry::Point(p) if p == on_segment));
    assert_eq!(outside.topology, IntersectionTopology::Disjoint);
}

#[test]
fn infinite_line_point_intersection_checks_collinearity() {
    let line =
        InfiniteLine3D::from_two_points(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0))
            .unwrap();
    let on_line = Point3D::new(5.0, 0.0, 0.0);
    let off_line = Point3D::new(0.0, 1.0, 0.0);

    let on = infinite_line3d_point3d_intersection(&line, &on_line, standard_distance_tol());
    let off = infinite_line3d_point3d_intersection(&line, &off_line, standard_distance_tol());
    assert_eq!(on.topology, IntersectionTopology::Crossing);
    assert!(matches!(on.geometry, IntersectionGeometry::Point(p) if p == on_line));
    assert_eq!(off.topology, IntersectionTopology::Disjoint);
}

#[test]
fn triangle_point_intersection_uses_distance_based_test() {
    let triangle = Triangle3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0),
        Point3D::new(0.0, 1.0, 0.0),
    )
    .unwrap();
    let on_triangle = Point3D::new(0.2, 0.2, 0.0);
    let off_triangle = Point3D::new(0.2, 0.2, 0.5);

    let on = triangle3d_point3d_intersection(&triangle, &on_triangle, standard_distance_tol());
    let off = triangle3d_point3d_intersection(&triangle, &off_triangle, standard_distance_tol());
    assert_eq!(on.topology, IntersectionTopology::Crossing);
    assert!(matches!(on.geometry, IntersectionGeometry::Point(p) if p == on_triangle));
    assert_eq!(off.topology, IntersectionTopology::Disjoint);
}

#[test]
fn circle_point_intersection_requires_point_on_circumference() {
    let circle = Circle3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Direction3D::new(0.0, 0.0, 1.0).unwrap(),
        2.0,
    )
    .unwrap();
    let on_circle = Point3D::new(2.0, 0.0, 0.0);
    let inside_disk = Point3D::new(1.0, 0.0, 0.0);

    assert_eq!(
        circle3d_point3d_intersection(&circle, &on_circle, standard_distance_tol()).topology,
        IntersectionTopology::Crossing
    );
    assert!(matches!(
        circle3d_point3d_intersection(&circle, &on_circle, standard_distance_tol()).geometry,
        IntersectionGeometry::Point(point) if point == on_circle
    ));
    assert_eq!(
        circle3d_point3d_intersection(&circle, &inside_disk, standard_distance_tol()).topology,
        IntersectionTopology::Disjoint
    );
    assert!(matches!(
        circle3d_point3d_intersection(&circle, &inside_disk, standard_distance_tol()).geometry,
        IntersectionGeometry::None
    ));
}

#[test]
fn circle_point_intersection_result_converts_to_topology() {
    let circle = Circle3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Direction3D::new(0.0, 0.0, 1.0).unwrap(),
        2.0,
    )
    .unwrap();
    let on_circle = Point3D::new(2.0, 0.0, 0.0);
    let off_circle = Point3D::new(1.0, 0.0, 0.0);

    let hit = circle3d_point3d_intersection(&circle, &on_circle, standard_distance_tol());
    let miss = circle3d_point3d_intersection(&circle, &off_circle, standard_distance_tol());

    assert_eq!(hit.topology, IntersectionTopology::Crossing);
    assert!(matches!(hit.geometry, IntersectionGeometry::Point(_)));
    assert_eq!(miss.topology, IntersectionTopology::Disjoint);
    assert!(matches!(miss.geometry, IntersectionGeometry::None));
}

#[test]
fn arc_point_intersection_checks_angle_range() {
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

    assert_eq!(
        arc3d_point3d_intersection(&arc, &on_arc, standard_distance_tol()).topology,
        IntersectionTopology::Crossing
    );
    assert!(matches!(
        arc3d_point3d_intersection(&arc, &on_arc, standard_distance_tol()).geometry,
        IntersectionGeometry::Point(point) if point == on_arc
    ));
    assert_eq!(
        arc3d_point3d_intersection(&arc, &out_of_angle, standard_distance_tol()).topology,
        IntersectionTopology::Disjoint
    );
    assert!(matches!(
        arc3d_point3d_intersection(&arc, &out_of_angle, standard_distance_tol()).geometry,
        IntersectionGeometry::None
    ));
}

#[test]
fn arc_point_intersection_result_converts_to_topology() {
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
    let off_arc = Point3D::new(-2.0, 0.0, 0.0);
    let hit = arc3d_point3d_intersection(&arc, &on_arc, standard_distance_tol());
    let miss = arc3d_point3d_intersection(&arc, &off_arc, standard_distance_tol());

    assert_eq!(hit.topology, IntersectionTopology::Crossing);
    assert!(matches!(hit.geometry, IntersectionGeometry::Point(_)));
    assert_eq!(miss.topology, IntersectionTopology::Disjoint);
    assert!(matches!(miss.geometry, IntersectionGeometry::None));
}

#[test]
fn arc_linear_input_result_variants_convert_to_topology() {
    let arc = Arc3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        2.0,
        Direction3D::new(0.0, 0.0, 1.0).unwrap(),
        Direction3D::new(1.0, 0.0, 0.0).unwrap(),
        Angle::from_radians(0.0),
        Angle::from_radians(std::f64::consts::FRAC_PI_2),
    )
    .unwrap();

    let segment_hit =
        LineSegment3D::new(Point3D::new(2.0, 0.0, 0.0), Point3D::new(3.0, 0.0, 0.0)).unwrap();
    let segment_miss =
        LineSegment3D::new(Point3D::new(5.0, 0.0, 0.0), Point3D::new(6.0, 0.0, 0.0)).unwrap();
    let ray_hit = Ray3D::new(Point3D::new(2.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
    let ray_miss = Ray3D::new(Point3D::new(5.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
    let line_hit =
        InfiniteLine3D::from_two_points(Point3D::new(2.0, 0.0, 0.0), Point3D::new(2.0, 1.0, 0.0))
            .unwrap();
    let line_miss =
        InfiniteLine3D::from_two_points(Point3D::new(5.0, 0.0, 0.0), Point3D::new(5.0, 1.0, 0.0))
            .unwrap();

    let hit_segment =
        arc3d_line_segment3d_intersection(&arc, &segment_hit, standard_distance_tol());
    let miss_segment =
        arc3d_line_segment3d_intersection(&arc, &segment_miss, standard_distance_tol());
    let hit_ray = arc3d_ray3d_intersection(&arc, &ray_hit, standard_distance_tol());
    let miss_ray = arc3d_ray3d_intersection(&arc, &ray_miss, standard_distance_tol());
    let hit_line = arc3d_infinite_line3d_intersection(&arc, &line_hit, standard_distance_tol());
    let miss_line = arc3d_infinite_line3d_intersection(&arc, &line_miss, standard_distance_tol());

    assert_eq!(hit_segment.topology, IntersectionTopology::Crossing);
    assert!(matches!(
        hit_segment.geometry,
        IntersectionGeometry::Point(_)
    ));
    assert_eq!(miss_segment.topology, IntersectionTopology::Disjoint);
    assert_eq!(hit_ray.topology, IntersectionTopology::Crossing);
    assert_eq!(miss_ray.topology, IntersectionTopology::Disjoint);
    assert_eq!(hit_line.topology, IntersectionTopology::Crossing);
    assert_eq!(miss_line.topology, IntersectionTopology::Disjoint);
}

#[test]
fn segment_endpoint_match_helper_is_symmetric() {
    let segment =
        LineSegment3D::new(Point3D::new(2.0, 0.0, 0.0), Point3D::new(3.0, 0.0, 0.0)).unwrap();

    assert!(point_matches_either_segment_endpoint(
        Point3D::new(2.0, 0.0, 0.0),
        &segment,
        standard_distance_tol(),
    ));
    assert!(point_matches_either_segment_endpoint(
        Point3D::new(3.0, 0.0, 0.0),
        &segment,
        standard_distance_tol(),
    ));
    assert!(!point_matches_either_segment_endpoint(
        Point3D::new(4.0, 0.0, 0.0),
        &segment,
        standard_distance_tol(),
    ));
}

#[test]
fn ellipse_point_intersection_uses_distance() {
    let ellipse = Ellipse3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        3.0,
        2.0,
        Vector3D::new(0.0, 0.0, 1.0),
        Vector3D::new(1.0, 0.0, 0.0),
    )
    .unwrap();

    let on_ellipse = Point3D::new(3.0, 0.0, 0.0);
    let inside_ellipse = Point3D::new(1.0, 0.0, 0.0);
    let outside_plane = Point3D::new(0.0, 0.0, 0.5);

    let on = ellipse3d_point3d_intersection(&ellipse, &on_ellipse, standard_distance_tol());
    let inside = ellipse3d_point3d_intersection(&ellipse, &inside_ellipse, standard_distance_tol());
    let outside = ellipse3d_point3d_intersection(&ellipse, &outside_plane, standard_distance_tol());
    assert_eq!(on.topology, IntersectionTopology::Crossing);
    assert!(matches!(on.geometry, IntersectionGeometry::Point(p) if p == on_ellipse));
    assert_eq!(inside.topology, IntersectionTopology::Crossing);
    assert!(matches!(inside.geometry, IntersectionGeometry::Point(p) if p == inside_ellipse));
    assert_eq!(outside.topology, IntersectionTopology::Disjoint);
}

#[test]
fn circle_linear_input_result_variants_convert_to_topology() {
    let circle = Circle3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Direction3D::new(0.0, 0.0, 1.0).unwrap(),
        2.0,
    )
    .unwrap();

    let segment_hit =
        LineSegment3D::new(Point3D::new(2.0, 0.0, 0.0), Point3D::new(3.0, 0.0, 0.0)).unwrap();
    let segment_miss =
        LineSegment3D::new(Point3D::new(5.0, 0.0, 0.0), Point3D::new(6.0, 0.0, 0.0)).unwrap();
    let ray_hit = Ray3D::new(Point3D::new(2.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
    let ray_miss = Ray3D::new(Point3D::new(5.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
    let line_hit =
        InfiniteLine3D::from_two_points(Point3D::new(2.0, 0.0, 0.0), Point3D::new(2.0, 1.0, 0.0))
            .unwrap();
    let line_miss =
        InfiniteLine3D::from_two_points(Point3D::new(5.0, 0.0, 0.0), Point3D::new(5.0, 1.0, 0.0))
            .unwrap();

    let hit_segment =
        circle3d_line_segment3d_intersection(&circle, &segment_hit, standard_distance_tol());
    let miss_segment =
        circle3d_line_segment3d_intersection(&circle, &segment_miss, standard_distance_tol());
    let hit_ray = circle3d_ray3d_intersection(&circle, &ray_hit, standard_distance_tol());
    let miss_ray = circle3d_ray3d_intersection(&circle, &ray_miss, standard_distance_tol());
    let hit_line =
        circle3d_infinite_line3d_intersection(&circle, &line_hit, standard_distance_tol());
    let miss_line =
        circle3d_infinite_line3d_intersection(&circle, &line_miss, standard_distance_tol());

    assert_eq!(hit_segment.topology, IntersectionTopology::Crossing);
    assert!(matches!(
        hit_segment.geometry,
        IntersectionGeometry::Point(_)
    ));
    assert_eq!(miss_segment.topology, IntersectionTopology::Disjoint);
    assert_eq!(hit_ray.topology, IntersectionTopology::Crossing);
    assert_eq!(miss_ray.topology, IntersectionTopology::Disjoint);
    assert_eq!(hit_line.topology, IntersectionTopology::Crossing);
    assert_eq!(miss_line.topology, IntersectionTopology::Disjoint);
}

#[test]
fn torus_surface_point_intersection_uses_surface_distance() {
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

    let on = torus_surface3d_point3d_intersection(&torus, &on_surface, standard_distance_tol());
    let inside =
        torus_surface3d_point3d_intersection(&torus, &inside_tube, standard_distance_tol());
    assert_eq!(on.topology, IntersectionTopology::Crossing);
    assert!(matches!(on.geometry, IntersectionGeometry::Point(p) if p == on_surface));
    assert_eq!(inside.topology, IntersectionTopology::Disjoint);
}

#[test]
fn cylindrical_surface_point_intersection_uses_surface_distance() {
    let cyl = CylindricalSurface3D::new_z_axis(Point3D::new(0.0, 0.0, 0.0), 1.0).unwrap();

    let on_surface = Point3D::new(1.0, 0.0, 0.5);
    let off_surface = Point3D::new(2.0, 0.0, 0.5);

    let on = cylindrical_surface3d_point3d_intersection(&cyl, &on_surface, standard_distance_tol());
    let off =
        cylindrical_surface3d_point3d_intersection(&cyl, &off_surface, standard_distance_tol());
    assert_eq!(on.topology, IntersectionTopology::Crossing);
    assert!(matches!(on.geometry, IntersectionGeometry::Point(p) if p == on_surface));
    assert_eq!(off.topology, IntersectionTopology::Disjoint);
}

#[test]
fn triangle_mesh_point_intersection_checks_member_triangles() {
    let mesh = TriangleMesh3D::new(
        vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        ],
        vec![[0, 1, 2]],
    )
    .unwrap();

    let on_triangle = Point3D::new(0.2, 0.2, 0.0);
    let off_triangle = Point3D::new(0.2, 0.2, 0.3);

    let on = triangle_mesh3d_point3d_intersection(&mesh, &on_triangle, standard_distance_tol());
    let off = triangle_mesh3d_point3d_intersection(&mesh, &off_triangle, standard_distance_tol());
    assert_eq!(on.topology, IntersectionTopology::Crossing);
    assert!(matches!(on.geometry, IntersectionGeometry::Point(p) if p == on_triangle));
    assert_eq!(off.topology, IntersectionTopology::Disjoint);
}

#[test]
fn symmetric_triangle_intersection_wrappers_match_base_functions() {
    let tri = Triangle3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0),
        Point3D::new(0.0, 1.0, 0.0),
    )
    .unwrap();
    let seg =
        LineSegment3D::new(Point3D::new(0.2, 0.2, -1.0), Point3D::new(0.2, 0.2, 1.0)).unwrap();
    let ray = Ray3D::new(Point3D::new(0.2, 0.2, 1.0), Vector3D::new(0.0, 0.0, -1.0)).unwrap();

    let tol = standard_distance_tol();
    assert_eq!(
        line_segment3d_triangle3d_intersection(&seg, &tri, tol).topology,
        triangle3d_line_segment3d_intersection(&tri, &seg, tol).topology
    );
    assert_eq!(
        ray3d_triangle3d_intersection(&ray, &tri, tol).topology,
        triangle3d_ray3d_intersection(&tri, &ray, tol).topology
    );
}

#[test]
fn symmetric_plane_intersection_wrappers_match_base_functions() {
    let tol = standard_distance_tol();
    let plane = Plane3D::xy_plane(0.0_f64);
    let ray = Ray3D::new(Point3D::new(0.0, 0.0, 1.0), Vector3D::new(0.0, 0.0, -1.0)).unwrap();
    let seg =
        LineSegment3D::new(Point3D::new(0.0, 0.0, -1.0), Point3D::new(0.0, 0.0, 1.0)).unwrap();

    assert_eq!(
        ray3d_plane3d_intersection(&ray, &plane, tol).topology,
        plane3d_ray3d_intersection(&plane, &ray, tol).topology
    );
    assert_eq!(
        line_segment3d_plane3d_intersection(&seg, &plane, tol).topology,
        plane3d_line_segment3d_intersection(&plane, &seg, tol).topology
    );
}

#[test]
fn symmetric_line_segment_intersection_wrappers_match_base_functions() {
    let tol = standard_distance_tol();
    let seg = LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0)).unwrap();
    let ray = Ray3D::new(Point3D::new(0.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
    let line =
        InfiniteLine3D::from_two_points(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0))
            .unwrap();

    assert_eq!(
        line_segment3d_ray3d_intersection(&seg, &ray, tol).topology,
        ray3d_line_segment3d_intersection(&ray, &seg, tol).topology
    );
    assert_eq!(
        line_segment3d_infinite_line3d_intersection(&seg, &line, tol).topology,
        infinite_line3d_line_segment3d_intersection(&line, &seg, tol).topology
    );
}

#[test]
fn spherical_surface_line_like_intersections_return_expected_points() {
    let tolerance = 1e-9;
    let sphere = SphericalSurface3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Vector3D::new(0.0, 0.0, 1.0),
        Vector3D::new(1.0, 0.0, 0.0),
        1.0,
    )
    .unwrap();

    let segment =
        LineSegment3D::new(Point3D::new(-2.0, 0.0, 0.0), Point3D::new(2.0, 0.0, 0.0)).unwrap();
    let ray = Ray3D::new(Point3D::new(-2.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
    let line =
        InfiniteLine3D::from_two_points(Point3D::new(-2.0, 0.0, 0.0), Point3D::new(2.0, 0.0, 0.0))
            .unwrap();

    let seg_result = line_segment3d_spherical_surface3d_intersections(&segment, &sphere, tolerance);
    assert_eq!(seg_result.topology, IntersectionTopology::Crossing);
    if let IntersectionGeometry::Points(ref pts) = seg_result.geometry {
        assert_eq!(pts.len(), 2);
        assert_eq!(pts[0], Point3D::new(-1.0, 0.0, 0.0));
        assert_eq!(pts[1], Point3D::new(1.0, 0.0, 0.0));
    } else {
        panic!("Expected Points geometry");
    }

    let ray_result = ray3d_spherical_surface3d_intersections(&ray, &sphere, tolerance);
    assert_eq!(ray_result.topology, IntersectionTopology::Crossing);
    if let IntersectionGeometry::Points(ref pts) = ray_result.geometry {
        assert_eq!(pts.len(), 2);
        assert_eq!(pts[0], Point3D::new(-1.0, 0.0, 0.0));
        assert_eq!(pts[1], Point3D::new(1.0, 0.0, 0.0));
    } else {
        panic!("Expected Points geometry");
    }

    let line_result = infinite_line3d_spherical_surface3d_intersections(&line, &sphere, tolerance);
    assert_eq!(line_result.topology, IntersectionTopology::Crossing);
    if let IntersectionGeometry::Points(ref pts) = line_result.geometry {
        assert_eq!(pts.len(), 2);
        assert_eq!(pts[0], Point3D::new(-1.0, 0.0, 0.0));
        assert_eq!(pts[1], Point3D::new(1.0, 0.0, 0.0));
    } else {
        panic!("Expected Points geometry");
    }
}

#[test]
fn ray_ray_intersection_returns_shared_point_only_for_forward_rays() {
    let tolerance = 1e-9;
    let ray_a = Ray3D::new(Point3D::new(-1.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
    let ray_b = Ray3D::new(Point3D::new(0.0, -1.0, 0.0), Vector3D::new(0.0, 1.0, 0.0)).unwrap();
    let opposite = Ray3D::new(Point3D::new(0.0, -1.0, 0.0), Vector3D::new(0.0, -1.0, 0.0)).unwrap();

    let crossing = ray3d_ray3d_intersection(&ray_a, &ray_b, tolerance);
    let disjoint = ray3d_ray3d_intersection(&ray_a, &opposite, tolerance);
    assert_eq!(crossing.topology, IntersectionTopology::Crossing);
    let origin = Point3D::new(0.0_f64, 0.0, 0.0);
    if let IntersectionGeometry::Point(ref p) = crossing.geometry {
        assert!(p.distance_to(&origin) < 1e-9);
    } else {
        panic!("Expected IntersectionGeometry::Point for crossing");
    }
    assert_eq!(disjoint.topology, IntersectionTopology::Disjoint);
}

#[test]
fn conical_solid_point_boundary_guard_keeps_intersection_on_point_helper() {
    const CONICAL_SOLID_DIRECT_CONTAINMENT: &str =
        "ConicalSolid3DContainment::contains_point_tolerance";
    const CONICAL_SOLID_POINT_HELPER: &str = "conical_solid3d_contains_point_with_tolerance";

    fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
        let start_index = source
            .find(start)
            .unwrap_or_else(|| panic!("missing start marker: {start}"));
        let tail = &source[start_index..];
        let end_index = tail
            .find(end)
            .unwrap_or_else(|| panic!("missing end marker: {end}"));
        &tail[..end_index]
    }

    let source = include_str!("cylindrical_and_conical_family.rs");
    let conical_solid_point_section = section(
        source,
        "fn conical_solid3d_point3d_intersection_raw",
        "fn conical_surface3d_point3d_intersection_raw",
    );

    assert!(
        conical_solid_point_section.contains(CONICAL_SOLID_POINT_HELPER),
        "intersection/primitive_3d/cylindrical_and_conical_family.rs should route conical solid point-like checks through the point helper"
    );
    assert!(
        !conical_solid_point_section.contains(CONICAL_SOLID_DIRECT_CONTAINMENT),
        "intersection/primitive_3d/cylindrical_and_conical_family.rs must not call ConicalSolid3DContainment::contains_point_tolerance directly in the representative conical solid section"
    );
}
