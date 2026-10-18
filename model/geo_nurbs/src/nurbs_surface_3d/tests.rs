//! `NurbsSurface3D` のテスト

use super::*;
use crate::clamped_knot_vector;
use crate::{AdaptiveTessellationSettings, NurbsSurfaceAdaptiveTessellation};
use geo_contracts::{Bounded, PrimitiveKind, PrimitiveMetadata};
use std::f64::consts::FRAC_1_SQRT_2;

/// (0, 0, 0)-(1, 1, 0) の XY 平面上の 1 次の曲面
fn unit_square() -> NurbsSurface3D<f64> {
    NurbsSurface3D::new(
        1,
        1,
        clamped_knot_vector(1, 2),
        clamped_knot_vector(1, 2),
        &[
            vec![Point3D::new(0.0, 0.0, 0.0), Point3D::new(0.0, 1.0, 0.0)],
            vec![Point3D::new(1.0, 0.0, 0.0), Point3D::new(1.0, 1.0, 0.0)],
        ],
        None,
    )
    .unwrap()
}

/// Z 軸まわりの半径 1・高さ 2 の円柱面の 1/4（u: 有理 2 次の円弧、v: 1 次の母線）
fn quarter_cylinder() -> NurbsSurface3D<f64> {
    let column = |x: f64, y: f64| vec![Point3D::new(x, y, 0.0), Point3D::new(x, y, 2.0)];
    NurbsSurface3D::new(
        2,
        1,
        clamped_knot_vector(2, 3),
        clamped_knot_vector(1, 2),
        &[column(1.0, 0.0), column(1.0, 1.0), column(0.0, 1.0)],
        Some(vec![
            vec![1.0, 1.0],
            vec![FRAC_1_SQRT_2, FRAC_1_SQRT_2],
            vec![1.0, 1.0],
        ]),
    )
    .unwrap()
}

#[test]
fn test_creation_and_properties() {
    let surface = quarter_cylinder();
    assert_eq!(surface.primitive_kind(), PrimitiveKind::NurbsSurface3D);
    assert_eq!((surface.u_degree(), surface.v_degree()), (2, 1));
    assert_eq!((surface.u_count(), surface.v_count()), (3, 2));
    assert_eq!(surface.u_knots(), &[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
    assert_eq!(surface.v_knots(), &[0.0, 0.0, 1.0, 1.0]);
    assert_eq!(
        surface.control_point(1, 1),
        Some(Point3D::new(1.0, 1.0, 2.0))
    );
    assert_eq!(surface.control_point(3, 0), None);
    assert_eq!(surface.weight(1, 0), Some(FRAC_1_SQRT_2));
    assert_eq!(surface.weight(0, 2), None);
    assert_eq!(surface.parameter_domain(), ((0.0, 1.0), (0.0, 1.0)));

    // 座標・重みの配列は u 方向優先
    assert_eq!(&surface.coordinates()[3..6], &[1.0, 0.0, 2.0]);
    assert_eq!(
        surface.weights(),
        Some(&[1.0, 1.0, FRAC_1_SQRT_2, FRAC_1_SQRT_2, 1.0, 1.0][..])
    );
    assert_eq!(unit_square().weights(), None);
}

#[test]
fn test_point_at_uv() {
    let square = unit_square();
    assert_eq!(
        square.point_at_uv(0.0, 0.0),
        Some(Point3D::new(0.0, 0.0, 0.0))
    );
    assert!(
        square
            .point_at_uv(0.25, 0.75)
            .unwrap()
            .distance_to(Point3D::new(0.25, 0.75, 0.0))
            < 1e-12
    );

    // 有理曲面の点は円柱面上
    let cylinder = quarter_cylinder();
    for (u, v) in [(0.0, 0.0), (0.3, 0.5), (0.7, 1.0), (1.0, 0.2)] {
        let point = cylinder.point_at_uv(u, v).unwrap();
        assert!((point.x().hypot(point.y()) - 1.0).abs() < 1e-12);
        assert!((point.z() - 2.0 * v).abs() < 1e-12);
    }
}

#[test]
fn test_normal_at() {
    // 平面の法線は Z 方向
    let normal = unit_square().normal_at(0.5, 0.5).unwrap();
    assert!((normal.as_vector() - Vector3D::new(0.0, 0.0, 1.0)).length() < 1e-12);

    // 円柱面の法線は軸から離れる向き（∂u × ∂v の向き）
    let cylinder = quarter_cylinder();
    for (u, v) in [(0.0, 0.5), (0.4, 0.1), (1.0, 0.9)] {
        let point = cylinder.point_at_uv(u, v).unwrap();
        let radial = Vector3D::new(point.x(), point.y(), 0.0);
        let normal = cylinder.normal_at(u, v).unwrap();
        assert!(
            (normal.as_vector() - radial).length() < 1e-9,
            "(u, v) = ({u}, {v})"
        );
    }
}

/// 単位球の 1/8（x, y, z ≥ 0）。Z 軸まわりの 1/4 円（u）と、XY 平面から +Z 側の極への 1/4 円（v）の
/// 有理 2 次の回転面。v = 1 の辺は +Z 側の極 (0, 0, 1) の 1 点に縮退する
fn sphere_octant() -> NurbsSurface3D<f64> {
    let circle = [(1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
    let profile = [(1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
    let weights = [1.0, FRAC_1_SQRT_2, 1.0];
    let control_points: Vec<Vec<Point3D<f64>>> = circle
        .iter()
        .map(|&(x, y)| {
            profile
                .iter()
                .map(|&(radius, z)| Point3D::new(radius * x, radius * y, z))
                .collect()
        })
        .collect();
    let weight_grid = weights
        .iter()
        .map(|&wu| weights.iter().map(|&wv| wu * wv).collect())
        .collect();
    NurbsSurface3D::new(
        2,
        2,
        clamped_knot_vector(2, 3),
        clamped_knot_vector(2, 3),
        &control_points,
        Some(weight_grid),
    )
    .unwrap()
}

#[test]
fn test_normal_at_sphere_pole() {
    let sphere = sphere_octant();

    // 極以外では、単位球の法線は位置ベクトルと一致する
    for (u, v) in [(0.0, 0.0), (0.3, 0.4), (0.8, 0.9)] {
        let point = sphere.point_at_uv(u, v).unwrap();
        let normal = sphere.normal_at(u, v).unwrap();
        assert!(
            (normal.as_vector() - Vector3D::from_points(Point3D::origin(), point)).length() < 1e-9
        );
    }

    // 縮退した +Z 側の極（v = 1）では、極限の法線として +Z の方向を返す
    for u in [0.0, 0.5, 1.0] {
        assert!(
            sphere
                .point_at_uv(u, 1.0)
                .unwrap()
                .distance_to(Point3D::new(0.0, 0.0, 1.0))
                < 1e-12
        );
        let normal = sphere.normal_at(u, 1.0).unwrap();
        assert!(
            (normal.as_vector() - Vector3D::new(0.0, 0.0, 1.0)).length() < 1e-12,
            "u = {u}"
        );
    }
}

#[test]
fn test_second_partial_derivatives() {
    // 有理曲面の 2 階の偏導関数を、1 階の偏導関数の中心差分と比べる
    let sphere = sphere_octant();
    let (u, v, h) = (0.3, 0.4, 1e-5);
    let derivatives = sphere.partial_derivatives(u, v, 2);
    let first = |u, v| {
        let d = sphere.partial_derivatives(u, v, 1);
        (to_vector(d[1][0]), to_vector(d[0][1]))
    };
    let (u_ahead, v_ahead) = (first(u + h, v), first(u, v + h));
    let (u_behind, v_behind) = (first(u - h, v), first(u, v - h));
    let expected = [
        ((u_ahead.0 - u_behind.0) * (0.5 / h), derivatives[2][0]),
        ((v_ahead.0 - v_behind.0) * (0.5 / h), derivatives[1][1]),
        ((u_ahead.1 - u_behind.1) * (0.5 / h), derivatives[1][1]),
        ((v_ahead.1 - v_behind.1) * (0.5 / h), derivatives[0][2]),
    ];
    for (difference, derivative) in expected {
        assert!((difference - to_vector(derivative)).length() < 1e-6);
    }
}

#[test]
fn test_normal_at_degenerate_edge() {
    // u = 0 の辺が 1 点に縮退した平面上の曲面（三角形）でも、法線は平面の法線
    let surface = NurbsSurface3D::new(
        1,
        1,
        clamped_knot_vector(1, 2),
        clamped_knot_vector(1, 2),
        &[
            vec![Point3D::new(0.0, 0.0, 0.0), Point3D::new(0.0, 0.0, 0.0)],
            vec![Point3D::new(1.0, 0.0, 0.0), Point3D::new(1.0, 1.0, 0.0)],
        ],
        None,
    )
    .unwrap();
    let normal = surface.normal_at(0.0, 0.5).unwrap();
    assert!((normal.as_vector() - Vector3D::new(0.0, 0.0, 1.0)).length() < 1e-12);

    // v = 1 の辺（定義域の終了側）が 1 点に縮退した曲面でも、近づく側から求めた法線は平面の法線
    let surface = NurbsSurface3D::new(
        1,
        1,
        clamped_knot_vector(1, 2),
        clamped_knot_vector(1, 2),
        &[
            vec![Point3D::new(0.0, 0.0, 0.0), Point3D::new(0.5, 1.0, 0.0)],
            vec![Point3D::new(1.0, 0.0, 0.0), Point3D::new(0.5, 1.0, 0.0)],
        ],
        None,
    )
    .unwrap();
    for u in [0.0, 0.5, 1.0] {
        let normal = surface.normal_at(u, 1.0).unwrap();
        assert!(
            (normal.as_vector() - Vector3D::new(0.0, 0.0, 1.0)).length() < 1e-12,
            "u = {u}"
        );
    }

    // 曲面全体が 1 点に縮退している場合は定まらない
    let p = Point3D::new(1.0, 2.0, 3.0);
    let point_surface = NurbsSurface3D::new(
        1,
        1,
        clamped_knot_vector(1, 2),
        clamped_knot_vector(1, 2),
        &[vec![p, p], vec![p, p]],
        None,
    )
    .unwrap();
    assert!(point_surface.normal_at(0.5, 0.5).is_none());
}

#[test]
fn test_evaluation_outside_domain() {
    let square = unit_square();
    assert!(square.point_at_uv(-0.1, 0.5).is_none());
    assert!(square.point_at_uv(0.5, 1.1).is_none());
    assert!(square.normal_at(f64::NAN, 0.5).is_none());
}

#[test]
fn test_invalid_creation() {
    let knots = clamped_knot_vector::<f64>(1, 2);
    let new = |points: &[Vec<Point3D<f64>>], weights| {
        NurbsSurface3D::new(1, 1, knots.clone(), knots.clone(), points, weights).unwrap_err()
    };
    let p = Point3D::new(0.0, 0.0, 0.0);

    assert_eq!(new(&[], None), NurbsError::EmptyControlGrid);
    assert_eq!(
        new(&[vec![p, p], vec![p]], None),
        NurbsError::ControlGridRowMismatch {
            row: 1,
            actual: 1,
            expected: 2,
        }
    );
    assert_eq!(
        new(
            &[vec![p, p], vec![p, p]],
            Some(vec![vec![1.0, 1.0], vec![1.0, -1.0]])
        ),
        NurbsError::NonPositiveWeight {
            index: 3,
            weight: -1.0,
        }
    );
    assert_eq!(
        new(&[vec![p, p]], None),
        NurbsError::InsufficientControlPoints {
            actual: 1,
            required: 2,
            degree: 1,
        }
    );
}

#[test]
fn test_bounded_aabb() {
    let bbox = quarter_cylinder().aabb().unwrap();
    assert_eq!(bbox.min(), Point3D::new(0.0, 0.0, 0.0));
    assert_eq!(bbox.max(), Point3D::new(1.0, 1.0, 2.0));
}

#[test]
fn test_surface_adaptive_params_plane() {
    let surface = unit_square();
    let settings = AdaptiveTessellationSettings::default_with_tolerance(0.01);
    let grid = surface.adaptive_params_surface(&settings);

    assert!(grid.u_params.len() > settings.min_segments as usize);
    assert!(grid.v_params.len() > settings.min_segments as usize);

    let ((u_min, u_max), (v_min, v_max)) = surface.parameter_domain();
    assert!((grid.u_params.first().unwrap() - u_min).abs() < 1e-10);
    assert!((grid.u_params.last().unwrap() - u_max).abs() < 1e-10);
    assert!((grid.v_params.first().unwrap() - v_min).abs() < 1e-10);
    assert!((grid.v_params.last().unwrap() - v_max).abs() < 1e-10);
}
