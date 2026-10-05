//! NURBS 曲線・曲面の相似変換テスト
//!
//! 有理 NURBS（重み付き）で、制御点を変換して重みを保持すれば曲線・曲面上の点が
//! 点の変換と一致することを確認する。

use crate::{NurbsCurve2D, NurbsCurve3D, NurbsSurface3D};
use analysis::test_constants::TOLERANCE_F64;
use geo_contracts::{
    Angle, NurbsCurve2DConstructor, NurbsCurve3DConstructor, NurbsSurface3DConstructor,
    SimilarityTransform2DCore, SimilarityTransform3DCore, SimilarityTransformable2D,
    SimilarityTransformable3D,
};
use geo_core::{
    Direction3D, Point2D, Point3D, SimilarityTransform2D, SimilarityTransform3D, Vector2D, Vector3D,
};
use std::f64::consts::FRAC_1_SQRT_2;

const SCALE: f64 = 2.0;
const PARAMS: [f64; 5] = [0.0, 0.2, 0.5, 0.8, 1.0];

fn transform_3d() -> SimilarityTransform3D<f64> {
    SimilarityTransform3D::rotation_about_axis(
        Point3D::new(0.5, -0.5, 0.0),
        Direction3D::new(1.0, 2.0, 3.0).unwrap(),
        Angle::from_degrees(50.0),
    )
    .then(&SimilarityTransform3D::translation(Vector3D::new(
        1.0, -2.0, 5.0,
    )))
    .then(&SimilarityTransform3D::uniform_scale_about(Point3D::origin(), SCALE).unwrap())
}

fn transform_2d() -> SimilarityTransform2D<f64> {
    SimilarityTransform2D::rotation_about(Point2D::new(0.5, -0.5), Angle::from_degrees(50.0))
        .then(&SimilarityTransform2D::translation(Vector2D::new(
            1.0, -2.0,
        )))
        .then(&SimilarityTransform2D::uniform_scale_about(Point2D::new(0.0, 0.0), SCALE).unwrap())
}

/// 有理 2 次の 1/4 円（半径 1、中心原点）
fn quarter_circle_3d() -> NurbsCurve3D<f64> {
    NurbsCurve3D::new(
        2,
        vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        vec![(1.0, 0.0, 0.0), (1.0, 1.0, 0.0), (0.0, 1.0, 0.0)],
        Some(vec![1.0, FRAC_1_SQRT_2, 1.0]),
    )
    .unwrap()
}

#[test]
fn rational_curve_3d_maps_points_and_keeps_weights() {
    let curve = quarter_circle_3d();
    let moved = curve.transform_similarity(&transform_3d()).unwrap();

    for t in PARAMS {
        let p = curve.evaluate_at(t);
        let expected = transform_3d().apply_point((p.x(), p.y(), p.z()));
        let q = moved.evaluate_at(t);
        let actual = (q.x(), q.y(), q.z());
        let d = Point3D::from(actual).distance_to(&Point3D::from(expected));
        assert!(d < TOLERANCE_F64, "t = {t}: {actual:?} != {expected:?}");
    }
    for i in 0..curve.num_points() {
        assert_eq!(moved.weight(i), curve.weight(i));
    }
    assert_eq!(moved.knot_vector(), curve.knot_vector());
    assert_eq!(moved.degree(), curve.degree());

    // 1/4 円は変換後も円弧であり、半径はスケール係数倍になる
    let center = transform_3d().apply_point((0.0, 0.0, 0.0));
    for t in PARAMS {
        let q = moved.evaluate_at(t);
        let r = Point3D::new(q.x(), q.y(), q.z()).distance_to(&Point3D::from(center));
        assert!((r - SCALE).abs() < TOLERANCE_F64);
    }
}

#[test]
fn rational_curve_2d_maps_points_and_keeps_weights() {
    let curve = NurbsCurve2D::new(
        &[(1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
        Some(vec![1.0, FRAC_1_SQRT_2, 1.0]),
        vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        2,
    )
    .unwrap();
    let moved = curve.transform_similarity(&transform_2d()).unwrap();

    for t in PARAMS {
        let p = curve.evaluate_at(t);
        let (ex, ey) = transform_2d().apply_point((p.x(), p.y()));
        let q = moved.evaluate_at(t);
        let d = Point2D::new(q.x(), q.y()).distance_to(&Point2D::new(ex, ey));
        assert!(d < TOLERANCE_F64, "t = {t}");
    }
    for i in 0..curve.num_points() {
        assert_eq!(moved.weight(i), curve.weight(i));
    }
}

#[test]
fn rational_surface_maps_points_and_keeps_weights() {
    let surface = NurbsSurface3D::new(
        vec![
            vec![(0.0, 0.0, 0.0), (0.0, 1.0, 0.5)],
            vec![(1.0, 0.0, 0.3), (1.0, 1.0, 1.0)],
        ],
        Some(vec![vec![1.0, 0.5], vec![2.0, 1.0]]),
        vec![0.0, 0.0, 1.0, 1.0],
        vec![0.0, 0.0, 1.0, 1.0],
        1,
        1,
    )
    .unwrap();
    let moved = surface.transform_similarity(&transform_3d()).unwrap();

    for u in PARAMS {
        for v in PARAMS {
            let p = surface.evaluate_at(u, v);
            let expected = transform_3d().apply_point((p.x(), p.y(), p.z()));
            let q = moved.evaluate_at(u, v);
            let d = Point3D::new(q.x(), q.y(), q.z()).distance_to(&Point3D::from(expected));
            assert!(d < TOLERANCE_F64, "(u, v) = ({u}, {v})");
        }
    }
    for u in 0..2 {
        for v in 0..2 {
            assert_eq!(moved.weight(u, v), surface.weight(u, v));
        }
    }
}
