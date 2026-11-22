use super::*;
use analysis::consts::tessellation::MAX_CHORD_ANGLE_DEGREES;
use geo_contracts::{
    Circle3DConstructor, Ellipse3DConstructor, InfiniteLine3DConstructor, NurbsCurve3DConstructor,
};
use std::f64::consts::{FRAC_PI_2, PI, TAU};

use crate::Direction3D;

fn settings(chord_tolerance: f64) -> CurveDiscretizationSettings {
    CurveDiscretizationSettings {
        chord_tolerance,
        max_angle: Angle::from_degrees(MAX_CHORD_ANGLE_DEGREES),
        max_points: 100_000,
    }
}

/// 点 `points` の弦と曲線 `curve` の離れが許容弦誤差以下、隣り合う弦のなす角が上限以下（曲線そのものが折れる
/// パラメータ `corners` の点は除く）であることを確かめる
fn assert_within_except(
    points: &[CurvePoint],
    settings: &CurveDiscretizationSettings,
    corners: &[f64],
    curve: impl Fn(f64) -> Point3D<f64>,
) {
    assert!(points.len() >= 2);
    for pair in points.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        assert!(a.parameter < b.parameter);
        assert!(a.point.distance_to(curve(a.parameter)) < 1.0e-12);
        for sample in 1..50 {
            let t = a.parameter + (b.parameter - a.parameter) * (sample as f64 / 50.0);
            let distance = distance_to_segment(curve(t), a.point, b.point);
            assert!(
                distance <= settings.chord_tolerance * (1.0 + 1.0e-9),
                "{distance} at {t}"
            );
        }
    }
    for triple in points.windows(3) {
        if corners.contains(&triple[1].parameter) {
            continue;
        }
        let first = Vector3D::from_points(triple[0].point, triple[1].point);
        let second = Vector3D::from_points(triple[1].point, triple[2].point);
        let cosine = first.dot(second) / (first.length() * second.length());
        let angle = cosine.clamp(-1.0, 1.0).acos();
        assert!(
            angle <= settings.max_angle.to_radians() + 1.0e-9,
            "{} degrees",
            angle.to_degrees()
        );
    }
}

fn assert_within(
    points: &[CurvePoint],
    settings: &CurveDiscretizationSettings,
    curve: impl Fn(f64) -> Point3D<f64>,
) {
    assert_within_except(points, settings, &[], curve);
}

fn axis_z() -> Direction3D<f64> {
    Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap()
}

#[test]
fn a_line_is_not_divided() {
    let line = <InfiniteLine3D<f64> as InfiniteLine3DConstructor<f64>>::new(
        Point3D::new(1.0, 2.0, 3.0),
        Vector3D::new(1.0, 0.0, 0.0),
    )
    .unwrap();
    let points = line.discretize((-2.0, 5.0), &settings(0.01)).unwrap();
    assert_eq!(points.len(), 2);
    assert_eq!(points[0].point, Point3D::new(-1.0, 2.0, 3.0));
    assert_eq!(points[1].point, Point3D::new(6.0, 2.0, 3.0));
}

#[test]
fn a_circle_keeps_the_chord_tolerance_and_the_angle() {
    let circle = <Circle3D<f64> as Circle3DConstructor<f64>>::new(
        Point3D::new(0.0, 0.0, 0.0),
        axis_z(),
        10.0,
    )
    .unwrap();
    for (range, tolerance) in [
        ((0.0, PI), 0.01),
        ((0.3, 0.3 + TAU), 0.001),
        ((1.0, 1.2), 0.5),
    ] {
        let settings = settings(tolerance);
        let points = circle.discretize(range, &settings).unwrap();
        assert_eq!(
            (points[0].parameter, points[points.len() - 1].parameter),
            range
        );
        assert_within(&points, &settings, |t| circle.point_at_parameter(t));
    }
}

#[test]
fn a_large_tolerance_on_a_small_circle_keeps_the_angle_limit() {
    // 許容弦誤差が直径より大きくても、値が壊れず、刻みは角度の上限で決まる
    let circle = <Circle3D<f64> as Circle3DConstructor<f64>>::new(
        Point3D::new(0.0, 0.0, 0.0),
        axis_z(),
        0.1,
    )
    .unwrap();
    let settings = settings(0.5);
    let points = circle.discretize((0.0, TAU), &settings).unwrap();
    assert_eq!(points.len(), 19);
    assert!(points.iter().all(|point| point.point.x().is_finite()));
    assert_within(&points, &settings, |t| circle.point_at_parameter(t));
}

#[test]
fn an_ellipse_keeps_the_angle_limit_at_the_ends_of_its_major_axis() {
    let ellipse = <Ellipse3D<f64> as Ellipse3DConstructor<f64>>::new(
        Point3D::new(1.0, 1.0, 0.0),
        3.0,
        1.0,
        axis_z(),
        Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap(),
    )
    .unwrap();
    for tolerance in [0.01, 1.0] {
        let settings = settings(tolerance);
        let points = ellipse.discretize((0.0, TAU), &settings).unwrap();
        assert_within(&points, &settings, |t| ellipse.point_at_parameter(t));
    }
}

/// 2 次の有理の NURBS 曲線（半径 1 の 4 分の 1 の円）
fn quarter_circle() -> NurbsCurve3D<f64> {
    <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
        2,
        vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        &[
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(1.0, 1.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        ],
        Some(vec![1.0, std::f64::consts::FRAC_1_SQRT_2, 1.0]),
    )
    .unwrap()
}

#[test]
fn a_nurbs_curve_keeps_the_chord_tolerance_and_the_angle() {
    // 4 分の 1 の円と、ノットで折れる 1 次の折れ線と、うねる 3 次の曲線
    let polyline = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
        1,
        vec![0.0, 0.0, 1.0, 2.0, 2.0],
        &[
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(1.0, 1.0, 0.0),
        ],
        None,
    )
    .unwrap();
    let wavy = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
        3,
        vec![0.0, 0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0, 1.0],
        &[
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 3.0, 0.0),
            Point3D::new(2.0, -3.0, 1.0),
            Point3D::new(3.0, 3.0, 0.0),
            Point3D::new(4.0, 0.0, 0.0),
        ],
        None,
    )
    .unwrap();
    // 折れ線はノット 1 で曲線そのものが折れる
    for (curve, range, corners) in [
        (quarter_circle(), (0.0, 1.0), vec![]),
        (quarter_circle(), (0.2, 0.7), vec![]),
        (polyline, (0.0, 2.0), vec![1.0]),
        (wavy, (0.0, 1.0), vec![]),
    ] {
        // 許容弦誤差が小さいときは弦との離れで、大きいときは角度の上限で分け方が決まる
        for tolerance in [0.001, 0.1, 0.5] {
            let settings = settings(tolerance);
            let points = curve.discretize(range, &settings).unwrap();
            assert_eq!(
                (points[0].parameter, points[points.len() - 1].parameter),
                range
            );
            assert_within_except(&points, &settings, &corners, |t| {
                curve.point_at_parameter(t).unwrap()
            });
        }
    }
}

#[test]
fn wrong_settings_and_ranges_are_refused() {
    let circle = <Circle3D<f64> as Circle3DConstructor<f64>>::new(
        Point3D::new(0.0, 0.0, 0.0),
        axis_z(),
        1.0,
    )
    .unwrap();
    for wrong in [
        CurveDiscretizationSettings {
            chord_tolerance: 0.0,
            ..settings(0.1)
        },
        CurveDiscretizationSettings {
            chord_tolerance: f64::NAN,
            ..settings(0.1)
        },
        CurveDiscretizationSettings {
            max_angle: Angle::from_degrees(180.0),
            ..settings(0.1)
        },
        CurveDiscretizationSettings {
            max_points: 1,
            ..settings(0.1)
        },
    ] {
        assert_eq!(
            circle.discretize((0.0, 1.0), &wrong).unwrap_err(),
            CurveDiscretizationError::InvalidSettings
        );
    }
    for wrong in [(1.0, 1.0), (1.0, 0.0), (0.0, f64::INFINITY)] {
        assert_eq!(
            circle.discretize(wrong, &settings(0.1)).unwrap_err(),
            CurveDiscretizationError::InvalidRange
        );
    }
    assert_eq!(
        quarter_circle()
            .discretize((0.5, 1.5), &settings(0.1))
            .unwrap_err(),
        CurveDiscretizationError::InvalidRange
    );
}

#[test]
fn too_many_points_are_refused() {
    let circle = <Circle3D<f64> as Circle3DConstructor<f64>>::new(
        Point3D::new(0.0, 0.0, 0.0),
        axis_z(),
        1.0e6,
    )
    .unwrap();
    let settings = CurveDiscretizationSettings {
        max_points: 1000,
        ..settings(1.0e-9)
    };
    assert_eq!(
        circle.discretize((0.0, FRAC_PI_2), &settings).unwrap_err(),
        CurveDiscretizationError::TooManyPoints
    );
    assert_eq!(
        quarter_circle()
            .discretize(
                (0.0, 1.0),
                &CurveDiscretizationSettings {
                    max_points: 3,
                    ..settings
                }
            )
            .unwrap_err(),
        CurveDiscretizationError::TooManyPoints
    );
}
