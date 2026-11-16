use super::*;
use crate::{
    Point3D, TopoCircle3D, TopoConicalSurface3D, TopoCylindricalSurface3D, TopoEllipse3D,
    TopoInfiniteLine3D, TopoNurbsSurface3D, TopoSphericalSurface3D, TopoTorusSurface3D, Vector3D,
};
use geo_contracts::{
    Angle, Circle3DConstructor, ConicalSurface3DConstructor, CylindricalSurface3DConstructor,
    Ellipse3DConstructor, InfiniteLine3DConstructor, NurbsCurve3DConstructor,
    NurbsSurface3DConstructor, Plane3DConstructor, SphericalSurface3DConstructor,
    TorusSurface3DConstructor,
};
use geo_core::Direction3D;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI, TAU};

const TOLERANCE: f64 = 1.0e-9;

/// 区間の中の、作るときに確かめた点と別の点でも、曲面の上の点が曲線の点と合うことを確かめる
fn assert_on_surface(
    surface: &SurfaceRef<f64>,
    curve: &CurveRef<f64>,
    range: (f64, f64),
) -> ParameterCurve<f64> {
    let parameter_curve = ParameterCurve::exact(surface, curve, range, TOLERANCE)
        .expect("the curve should have an exact parameter curve");
    for fraction in [0.0, 0.13, 0.37, 0.5, 0.71, 0.96, 1.0] {
        let t = range.0 + (range.1 - range.0) * fraction;
        let uv = parameter_curve.point_at(t).unwrap();
        let on_surface = surface.point_at_uv(uv.x(), uv.y()).unwrap();
        assert!(
            on_surface.distance_to(curve.point_at_parameter(t)) < 1.0e-9,
            "t = {t}"
        );
    }
    parameter_curve
}

fn direction(x: f64, y: f64, z: f64) -> Direction3D<f64> {
    Direction3D::from_vector(Vector3D::new(x, y, z)).unwrap()
}

/// 円の上の点 `point` の、円のパラメータ（角度）
fn angle_on(circle: &TopoCircle3D<f64>, point: Point3D<f64>) -> f64 {
    use geo_contracts::Circle3DProperties;
    let offset = point - circle.center();
    let x = circle.ref_direction().as_vector();
    let y = circle.axis().as_vector().cross(x);
    offset.dot(y).atan2(offset.dot(x))
}

fn tilted_plane() -> SurfaceRef<f64> {
    SurfaceRef::Plane(
        TopoPlane3D::from_three_points(
            Point3D::new(1.0, 2.0, 3.0),
            Point3D::new(2.0, 2.0, 4.0),
            Point3D::new(1.0, 3.0, 3.0),
        )
        .unwrap(),
    )
}

#[test]
fn curves_in_a_plane_map_to_lines_ellipses_and_nurbs_curves() {
    let surface = tilted_plane();
    let normal = direction(-1.0, 0.0, 1.0);
    let in_plane = direction(1.0, 0.0, 1.0);

    let line = TopoInfiniteLine3D::from_two_points(
        Point3D::new(1.0, 2.0, 3.0),
        Point3D::new(3.0, 5.0, 5.0),
    )
    .unwrap();
    assert!(matches!(
        assert_on_surface(&surface, &CurveRef::Line(line), (-1.0, 4.0)),
        ParameterCurve::Line { .. }
    ));

    let circle = TopoCircle3D::new(Point3D::new(2.0, 3.0, 4.0), normal, 1.5).unwrap();
    assert!(matches!(
        assert_on_surface(&surface, &CurveRef::Circle(circle), (0.5, 5.0)),
        ParameterCurve::Ellipse { .. }
    ));
    // 法線の逆を軸にした円（回る向きが逆）も同じように写す
    let reversed =
        TopoCircle3D::new(Point3D::new(2.0, 3.0, 4.0), direction(1.0, 0.0, -1.0), 1.5).unwrap();
    assert_on_surface(&surface, &CurveRef::Circle(reversed), (0.0, TAU));

    let ellipse =
        TopoEllipse3D::new(Point3D::new(2.0, 3.0, 4.0), 2.0, 1.0, normal, in_plane).unwrap();
    assert_on_surface(&surface, &CurveRef::Ellipse(ellipse), (0.0, TAU));

    let nurbs = <TopoNurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
        2,
        vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        &[
            Point3D::new(1.0, 2.0, 3.0),
            Point3D::new(2.0, 4.0, 4.0),
            Point3D::new(3.0, 2.0, 5.0),
        ],
        Some(vec![1.0, 0.5, 1.0]),
    )
    .unwrap();
    assert!(matches!(
        assert_on_surface(&surface, &CurveRef::Nurbs(nurbs), (0.0, 1.0)),
        ParameterCurve::Nurbs(_)
    ));

    // 平面から外れた曲線は、パラメータ曲線にならない
    let off_plane = TopoInfiniteLine3D::from_two_points(
        Point3D::new(1.0, 2.0, 3.0),
        Point3D::new(1.0, 2.0, 4.0),
    )
    .unwrap();
    assert!(
        ParameterCurve::exact(&surface, &CurveRef::Line(off_plane), (0.0, 1.0), TOLERANCE)
            .is_none()
    );
}

#[test]
fn rulings_and_circles_on_a_cylinder_are_lines_in_uv() {
    let surface = SurfaceRef::Cylinder(
        TopoCylindricalSurface3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 2.0).unwrap(),
    );
    // 母線
    let ruling = TopoInfiniteLine3D::from_two_points(
        Point3D::new(0.0, 2.0, -1.0),
        Point3D::new(0.0, 2.0, 3.0),
    )
    .unwrap();
    assert_on_surface(&surface, &CurveRef::Line(ruling), (0.0, 4.0));
    // 軸に垂直な円（1 周。周期で折り返さず、u が 2π 進む）
    let circle = TopoCircle3D::new_xy_plane(Point3D::new(0.0, 0.0, 1.5), 2.0).unwrap();
    let curve = assert_on_surface(&surface, &CurveRef::Circle(circle), (0.0, TAU));
    let (start, end) = (curve.point_at(0.0).unwrap(), curve.point_at(TAU).unwrap());
    assert!(((end.x() - start.x()).abs() - TAU).abs() < 1.0e-9);
    assert!((end.y() - start.y()).abs() < 1.0e-9);
}

#[test]
fn rulings_and_circles_on_a_cone_are_lines_in_uv() {
    let surface = SurfaceRef::Cone(
        TopoConicalSurface3D::new_standard(
            Point3D::new(0.0, 0.0, 0.0),
            1.0,
            Angle::from_radians(FRAC_PI_4),
        )
        .unwrap(),
    );
    // 頂点（0, 0, -1）を通る母線（頂点を区間の端に含む）
    let ruling = TopoInfiniteLine3D::from_two_points(
        Point3D::new(0.0, 0.0, -1.0),
        Point3D::new(1.0, 0.0, 0.0),
    )
    .unwrap();
    assert_on_surface(
        &surface,
        &CurveRef::Line(ruling),
        (0.0, 2.0_f64.sqrt() * 2.0),
    );
    let circle = TopoCircle3D::new_xy_plane(Point3D::new(0.0, 0.0, 1.0), 2.0).unwrap();
    assert_on_surface(&surface, &CurveRef::Circle(circle), (0.0, TAU));
}

#[test]
fn a_circle_beyond_the_cone_apex_is_a_line_in_uv() {
    // 頂点（z = -1）の先の z = -2 では、円錐の半径が負になり、点は角度 u の反対の側にある
    let surface = SurfaceRef::Cone(
        TopoConicalSurface3D::new_standard(
            Point3D::new(0.0, 0.0, 0.0),
            1.0,
            Angle::from_radians(FRAC_PI_4),
        )
        .unwrap(),
    );
    let circle = TopoCircle3D::new_xy_plane(Point3D::new(0.0, 0.0, -2.0), 1.0).unwrap();
    assert_on_surface(&surface, &CurveRef::Circle(circle), (0.0, TAU));
}

#[test]
fn meridians_and_parallels_on_a_sphere_are_lines_in_uv() {
    let surface = SurfaceRef::Sphere(
        TopoSphericalSurface3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 2.0).unwrap(),
    );
    // 緯線
    let parallel = TopoCircle3D::new_xy_plane(Point3D::new(0.0, 0.0, 1.0), 3.0_f64.sqrt()).unwrap();
    assert_on_surface(&surface, &CurveRef::Circle(parallel), (0.0, TAU));
    // 経線（南極から +X の赤道を通って北極まで。両端が極）
    let meridian =
        TopoCircle3D::new(Point3D::new(0.0, 0.0, 0.0), direction(0.0, 1.0, 0.0), 2.0).unwrap();
    let equator = angle_on(&meridian, Point3D::new(2.0, 0.0, 0.0));
    let range = (equator - FRAC_PI_2, equator + FRAC_PI_2);
    assert_on_surface(&surface, &CurveRef::Circle(meridian), range);
    // 極を越える経線は、(u, v) の直線にならない
    assert!(
        ParameterCurve::exact(
            &surface,
            &CurveRef::Circle(meridian),
            (equator, equator + PI),
            TOLERANCE
        )
        .is_none()
    );
}

#[test]
fn parallels_and_meridians_on_a_torus_are_lines_in_uv() {
    let surface = SurfaceRef::Torus(
        TopoTorusSurface3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 3.0, 1.0).unwrap(),
    );
    // 外側の赤道の円
    let parallel = TopoCircle3D::new_xy_plane(Point3D::new(0.0, 0.0, 0.0), 4.0).unwrap();
    assert_on_surface(&surface, &CurveRef::Circle(parallel), (0.0, TAU));
    // 管の断面の円（1 周。v が 2π 進む）
    let meridian =
        TopoCircle3D::new(Point3D::new(3.0, 0.0, 0.0), direction(0.0, 1.0, 0.0), 1.0).unwrap();
    let curve = assert_on_surface(&surface, &CurveRef::Circle(meridian), (0.0, TAU));
    let (start, end) = (curve.point_at(0.0).unwrap(), curve.point_at(TAU).unwrap());
    assert!(((end.y() - start.y()).abs() - TAU).abs() < 1.0e-9);
}

#[test]
fn a_nurbs_surface_has_no_exact_parameter_curve() {
    let surface = SurfaceRef::Nurbs(
        <TopoNurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::new(
            1,
            1,
            vec![0.0, 0.0, 1.0, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
            &[
                vec![Point3D::new(0.0, 0.0, 0.0), Point3D::new(0.0, 1.0, 0.0)],
                vec![Point3D::new(1.0, 0.0, 0.0), Point3D::new(1.0, 1.0, 0.0)],
            ],
            None,
        )
        .unwrap(),
    );
    let line = TopoInfiniteLine3D::from_two_points(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0),
    )
    .unwrap();

    // NURBS 曲面の上のパラメータ曲線は、当てはめで作る（別の Issue）
    assert!(
        ParameterCurve::exact(&surface, &CurveRef::Line(line), (0.0, 1.0), TOLERANCE).is_none()
    );
}

#[test]
fn the_surface_deviation_is_the_largest_distance_from_the_surface() {
    // 半径 2 の円柱に、半径 2.001 の円を載せる
    let cylinder = SurfaceRef::Cylinder(
        TopoCylindricalSurface3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 2.0).unwrap(),
    );
    let circle = TopoCircle3D::new_xy_plane(Point3D::new(0.0, 0.0, 1.5), 2.001).unwrap();
    let deviation =
        ParameterCurve::surface_deviation(&cylinder, &CurveRef::Circle(circle), (0.0, PI)).unwrap();
    assert!((deviation - 0.001).abs() < 1.0e-9, "{deviation}");
    assert!(
        ParameterCurve::exact(&cylinder, &CurveRef::Circle(circle), (0.0, PI), TOLERANCE).is_none()
    );

    // 平面から、端で 0.002 離れる直線
    let plane = SurfaceRef::Plane(
        TopoPlane3D::from_point_and_normal(Point3D::new(0.0, 0.0, 0.0), direction(0.0, 0.0, 1.0))
            .unwrap(),
    );
    let line = TopoInfiniteLine3D::from_two_points(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.001),
    )
    .unwrap();
    let length = (1.0_f64 + 1.0e-6).sqrt();
    let deviation =
        ParameterCurve::surface_deviation(&plane, &CurveRef::Line(line), (0.0, 2.0 * length))
            .unwrap();
    assert!((deviation - 0.002).abs() < 1.0e-9, "{deviation}");

    // 半頂角 60 度の円錐で、高さ 1 の円より 0.001 だけ大きい円。ずれは高さが同じ点までの距離で測り、
    // その 1.5 倍に広げれば、パラメータ曲線の確かめを通る
    let cone = SurfaceRef::Cone(
        TopoConicalSurface3D::new_standard(
            Point3D::new(0.0, 0.0, 0.0),
            1.0,
            Angle::from_radians(PI / 3.0),
        )
        .unwrap(),
    );
    let radius = 1.0 + 3.0_f64.sqrt() + 0.001;
    let circle =
        CurveRef::Circle(TopoCircle3D::new_xy_plane(Point3D::new(0.0, 0.0, 1.0), radius).unwrap());
    let deviation = ParameterCurve::surface_deviation(&cone, &circle, (0.0, PI)).unwrap();
    assert!((deviation - 0.001).abs() < 1.0e-12, "{deviation}");
    assert!(ParameterCurve::exact(&cone, &circle, (0.0, PI), deviation * 1.5).is_some());

    // 円柱を斜めの平面で切った楕円は、(u, v) で直線にならないが、円柱の上にある
    let ellipse = TopoEllipse3D::new(
        Point3D::new(0.0, 0.0, 1.0),
        2.0 * 2.0_f64.sqrt(),
        2.0,
        direction(1.0, 0.0, -1.0),
        direction(1.0, 0.0, 1.0),
    )
    .unwrap();
    let curve = CurveRef::Ellipse(ellipse);
    let deviation = ParameterCurve::surface_deviation(&cylinder, &curve, (0.0, PI)).unwrap();
    assert!(deviation < 1.0e-12, "{deviation}");
    assert!(ParameterCurve::exact(&cylinder, &curve, (0.0, PI), TOLERANCE).is_none());
}
