use super::*;
use crate::{Point3D, TopoCylindricalSurface3D, TopoEllipse3D, TopoNurbsCurve3D, Vector3D};
use geo_contracts::{
    Circle3DConstructor, CylindricalSurface3DConstructor, Ellipse3DConstructor,
    NurbsCurve2DProperties, NurbsCurve3DConstructor, NurbsSurface3DConstructor,
};
use geo_core::Direction3D;
use std::f64::consts::{PI, TAU};

/// 当てはめたパラメータ曲線を、区間を細かく分けた点（小区間の中で測った点と別の点を含む）で曲面の上で評価し、
/// 母曲線の点とのずれが `tolerance` の中にあることを確かめる
fn assert_fitted_within(
    surface: &SurfaceRef<f64>,
    curve: &CurveRef<f64>,
    range: (f64, f64),
    tolerance: f64,
) -> ParameterCurve<f64> {
    let (fitted, deviation) =
        fit(surface, curve, range, tolerance).expect("the curve should be fitted");
    assert!(
        deviation <= tolerance,
        "deviation {deviation} > {tolerance}"
    );
    let samples = 997;
    for index in 0..=samples {
        let t = range.0 + (range.1 - range.0) * (index as f64 / samples as f64);
        let uv = fitted.point_at(t).expect("t is in the domain");
        let on_surface = surface
            .point_at_uv(uv.x(), uv.y())
            .expect("uv is on the surface");
        let distance = on_surface.distance_to(curve.point_at_parameter(t));
        assert!(distance <= tolerance, "t = {t}: {distance} > {tolerance}");
    }
    // 当てはめた曲線と、`new` の作る曲線は、同じ区間を持つ
    assert!(ParameterCurve::new(surface, curve, range, tolerance).is_some());
    fitted
}

fn direction(x: f64, y: f64, z: f64) -> Direction3D<f64> {
    Direction3D::from_vector(Vector3D::new(x, y, z)).unwrap()
}

/// 半径 2 の円柱を、軸に 45 度傾いた平面で切った楕円
fn cylinder_and_ellipse() -> (SurfaceRef<f64>, CurveRef<f64>) {
    let cylinder = SurfaceRef::Cylinder(
        TopoCylindricalSurface3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 2.0).unwrap(),
    );
    let ellipse = TopoEllipse3D::new(
        Point3D::new(0.0, 0.0, 1.0),
        2.0 * 2.0_f64.sqrt(),
        2.0,
        direction(1.0, 0.0, -1.0),
        direction(1.0, 0.0, 1.0),
    )
    .unwrap();
    (cylinder, CurveRef::Ellipse(ellipse))
}

#[test]
fn an_ellipse_on_a_cylinder_is_fitted_within_the_tolerance() {
    let (cylinder, ellipse) = cylinder_and_ellipse();
    // 正確な形にならない組
    assert!(ParameterCurve::exact(&cylinder, &ellipse, (0.0, PI), 1.0e-3).is_none());
    for tolerance in [1.0e-3, 1.0e-6] {
        assert_fitted_within(&cylinder, &ellipse, (0.3, 2.5), tolerance);
        // 1 周: u は周期で折り返さず、2π 進む
        let fitted = assert_fitted_within(&cylinder, &ellipse, (0.0, TAU), tolerance);
        let (start, end) = (fitted.point_at(0.0).unwrap(), fitted.point_at(TAU).unwrap());
        assert!(((end.x() - start.x()).abs() - TAU).abs() < 1.0e-9);
    }
}

#[test]
fn a_finer_tolerance_needs_more_control_points() {
    let (cylinder, ellipse) = cylinder_and_ellipse();
    let count = |tolerance: f64| match fit(&cylinder, &ellipse, (0.0, TAU), tolerance) {
        Some((ParameterCurve::Nurbs(curve), _)) => curve.control_point_count(),
        _ => panic!("a fitted NURBS curve"),
    };
    assert!(count(1.0e-6) > count(1.0e-3));
}

#[test]
fn the_deviation_includes_the_fitting() {
    let (cylinder, ellipse) = cylinder_and_ellipse();
    // 正確な形にならない組のずれは、当てはめたずれ（許容誤差の中）
    let deviation = ParameterCurve::deviation(&cylinder, &ellipse, (0.0, PI), 1.0e-4).unwrap();
    assert!(deviation <= 1.0e-4, "{deviation}");
    // 正確な形になる組のずれは、正確な形のずれ
    let circle = CurveRef::Circle(
        crate::TopoCircle3D::new_xy_plane(Point3D::new(0.0, 0.0, 1.0), 2.0).unwrap(),
    );
    let deviation = ParameterCurve::deviation(&cylinder, &circle, (0.0, PI), 1.0e-4).unwrap();
    assert!(deviation < 1.0e-12, "{deviation}");
}

/// 平面の上の、(u, v) の付け方が一様でない 3 次の NURBS 曲面（xy 平面の [0, 4] × [0, 4]。u・v の
/// どちらも、x・y がパラメータの 3 次式になる）
fn uneven_planar_surface() -> TopoNurbsSurface3D<f64> {
    let positions = [0.0, 0.2, 1.5, 4.0];
    let control_points: Vec<Vec<Point3D<f64>>> = positions
        .iter()
        .map(|&x| positions.iter().map(|&y| Point3D::new(x, y, 0.0)).collect())
        .collect();
    <TopoNurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::new(
        3,
        3,
        vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
        vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
        &control_points,
        None,
    )
    .unwrap()
}

#[test]
fn a_circle_on_an_unevenly_parameterized_nurbs_surface_is_fitted() {
    let surface = SurfaceRef::Nurbs(uneven_planar_surface());
    let circle = CurveRef::Circle(
        crate::TopoCircle3D::new_xy_plane(Point3D::new(2.0, 2.0, 0.0), 1.5).unwrap(),
    );
    for tolerance in [1.0e-3, 1.0e-6] {
        assert_fitted_within(&surface, &circle, (0.0, TAU), tolerance);
        assert_fitted_within(&surface, &circle, (1.0, 2.0), tolerance);
    }
}

/// u の方向に曲がった柱の NURBS 曲面（xy の 3 次の Bezier を z の方向に 0〜1 押し出した形）と、その上の
/// 3 次の曲線（(u, v) = (t, t²)）
fn bent_surface_and_curve() -> (SurfaceRef<f64>, CurveRef<f64>) {
    let profile = [
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 2.0, 0.0),
        Point3D::new(3.0, -1.0, 0.0),
        Point3D::new(4.0, 1.0, 0.0),
    ];
    let control_points: Vec<Vec<Point3D<f64>>> = profile
        .iter()
        .map(|point| vec![*point, Point3D::new(point.x(), point.y(), 1.0)])
        .collect();
    let surface = <TopoNurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::new(
        3,
        1,
        vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
        vec![0.0, 0.0, 1.0, 1.0],
        &control_points,
        None,
    )
    .unwrap();
    // t² の 3 次の Bezier の制御点は 0, 0, 1/3, 1
    let heights = [0.0, 0.0, 1.0 / 3.0, 1.0];
    let curve_points: Vec<Point3D<f64>> = profile
        .iter()
        .zip(heights)
        .map(|(point, z)| Point3D::new(point.x(), point.y(), z))
        .collect();
    let curve = <TopoNurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
        3,
        vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
        &curve_points,
        None,
    )
    .unwrap();
    (SurfaceRef::Nurbs(surface), CurveRef::Nurbs(curve))
}

#[test]
fn a_curve_on_a_bent_nurbs_surface_is_fitted() {
    let (surface, curve) = bent_surface_and_curve();
    let fitted = assert_fitted_within(&surface, &curve, (0.0, 1.0), 1.0e-6);
    // (u, v) は (t, t²)
    for t in [0.0, 0.25, 0.6, 1.0] {
        let uv = fitted.point_at(t).unwrap();
        assert!((uv.x() - t).abs() < 1.0e-5, "t = {t}: {uv:?}");
        assert!((uv.y() - t * t).abs() < 1.0e-5, "t = {t}: {uv:?}");
    }
}

#[test]
fn a_nurbs_curve_is_broken_at_its_knots() {
    // ノットで折れる 1 次の NURBS 曲線（平面の上の折れ線）
    let surface = SurfaceRef::Nurbs(uneven_planar_surface());
    let polyline = <TopoNurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
        1,
        vec![0.0, 0.0, 0.3, 0.7, 1.0, 1.0],
        &[
            Point3D::new(0.5, 0.5, 0.0),
            Point3D::new(3.5, 1.0, 0.0),
            Point3D::new(1.0, 3.5, 0.0),
            Point3D::new(3.0, 3.0, 0.0),
        ],
        None,
    )
    .unwrap();
    let curve = CurveRef::Nurbs(polyline);
    assert_eq!(
        initial_breaks(&curve, (0.0, 1.0)),
        vec![
            0.0, 0.125, 0.25, 0.3, 0.375, 0.5, 0.625, 0.7, 0.75, 0.875, 1.0
        ]
    );
    assert_fitted_within(&surface, &curve, (0.0, 1.0), 1.0e-6);
}

#[test]
fn a_curve_off_the_surface_is_fitted_with_its_deviation() {
    // 平面から 0.01 だけ離れた円は、ずれ 0.01 の当てはめになる（許容誤差を超える）
    let surface = SurfaceRef::Nurbs(uneven_planar_surface());
    let circle = CurveRef::Circle(
        crate::TopoCircle3D::new_xy_plane(Point3D::new(2.0, 2.0, 0.01), 1.5).unwrap(),
    );
    let (_, deviation) = fit(&surface, &circle, (0.0, TAU), 1.0e-6).unwrap();
    assert!((deviation - 0.01).abs() < 1.0e-6, "{deviation}");
    assert!(ParameterCurve::new(&surface, &circle, (0.0, TAU), 1.0e-6).is_none());
    assert!(ParameterCurve::new(&surface, &circle, (0.0, TAU), deviation * 1.5).is_some());
}
