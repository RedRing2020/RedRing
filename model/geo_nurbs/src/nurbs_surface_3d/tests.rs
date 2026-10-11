//! `NurbsSurface3D` のテスト

use super::*;
use crate::clamped_knot_vector;
use crate::{AdaptiveTessellationSettings, NurbsSurfaceAdaptiveTessellation};
use geo_contracts::Bounded;
use geo_contracts::NurbsSurface3DConstructor;
use geo_contracts::{NurbsSurface3DDerived, NurbsSurface3DProperties};
use geo_contracts::{PrimitiveKind, PrimitiveMetadata};

#[test]
fn test_nurbs_surface_creation() {
    // 2x2 制御点グリッド
    let control_points = vec![
        vec![(0.0, 0.0, 0.0), (0.0, 1.0, 0.0)],
        vec![(1.0, 0.0, 0.0), (1.0, 1.0, 1.0)],
    ];

    let weights = vec![vec![1.0, 1.0], vec![1.0, 1.0]];

    let u_knots = vec![0.0, 0.0, 1.0, 1.0];
    let v_knots = vec![0.0, 0.0, 1.0, 1.0];

    let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::new(
        control_points,
        Some(weights), // 有理サーフェスとして
        u_knots,
        v_knots,
        1, // u_degree
        1, // v_degree
    );

    assert!(surface.is_ok());

    let surface = surface.unwrap();
    assert_eq!(surface.grid_size(), (2, 2));
    assert_eq!(surface.u_degree(), 1);
    assert_eq!(surface.v_degree(), 1);
}

#[test]
fn test_surface_evaluation() {
    // 平面サーフェスのテスト
    let control_points = vec![
        vec![(0.0, 0.0, 0.0), (0.0, 1.0, 0.0)],
        vec![(1.0, 0.0, 0.0), (1.0, 1.0, 0.0)],
    ];

    let weights = vec![vec![1.0, 1.0], vec![1.0, 1.0]];

    let u_knots = vec![0.0, 0.0, 1.0, 1.0];
    let v_knots = vec![0.0, 0.0, 1.0, 1.0];

    let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::new(
        control_points,
        Some(weights), // 有理サーフェスとして
        u_knots,
        v_knots,
        1,
        1,
    )
    .unwrap();

    // 中央点の評価
    let point = surface.evaluate_at(0.5, 0.5);
    assert!((point.x() - 0.5).abs() < 1e-10);
    assert!((point.y() - 0.5).abs() < 1e-10);
    assert!((point.z() - 0.0).abs() < 1e-10);
}

#[test]
fn test_point_at_uv_checked_returns_none_for_out_of_domain_parameter() {
    use geo_contracts::NurbsSurface3DEvaluation;

    let control_points = vec![
        vec![(0.0, 0.0, 0.0), (0.0, 1.0, 0.0)],
        vec![(1.0, 0.0, 0.0), (1.0, 1.0, 0.0)],
    ];
    let weights = vec![vec![1.0, 1.0], vec![1.0, 1.0]];
    let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::new(
        control_points,
        Some(weights),
        vec![2.0, 2.0, 3.0, 3.0],
        vec![5.0, 5.0, 6.0, 6.0],
        1,
        1,
    )
    .unwrap();

    let in_domain = <NurbsSurface3D<f64> as NurbsSurface3DEvaluation<f64>>::point_at_uv_checked(
        &surface, 2.5, 5.5,
    );
    let out_of_domain = <NurbsSurface3D<f64> as NurbsSurface3DEvaluation<f64>>::point_at_uv_checked(
        &surface, 1.9, 5.5,
    );

    assert!(in_domain.is_some());
    assert!(out_of_domain.is_none());
}

#[test]
fn test_point_at_uv_checked_returns_none_for_nan_parameter() {
    use geo_contracts::NurbsSurface3DEvaluation;

    let control_points = vec![
        vec![(0.0, 0.0, 0.0), (0.0, 1.0, 0.0)],
        vec![(1.0, 0.0, 0.0), (1.0, 1.0, 0.0)],
    ];
    let weights = vec![vec![1.0, 1.0], vec![1.0, 1.0]];
    let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::new(
        control_points,
        Some(weights),
        vec![2.0, 2.0, 3.0, 3.0],
        vec![5.0, 5.0, 6.0, 6.0],
        1,
        1,
    )
    .unwrap();

    let nan_u = <NurbsSurface3D<f64> as NurbsSurface3DEvaluation<f64>>::point_at_uv_checked(
        &surface,
        f64::NAN,
        5.5,
    );
    let nan_v = <NurbsSurface3D<f64> as NurbsSurface3DEvaluation<f64>>::point_at_uv_checked(
        &surface,
        2.5,
        f64::NAN,
    );

    assert!(nan_u.is_none());
    assert!(nan_v.is_none());
}

#[test]
fn test_constructor_new() {
    // 2x2 制御点グリッド
    let control_points = vec![
        vec![(0.0, 0.0, 0.0), (0.0, 1.0, 0.0)],
        vec![(1.0, 0.0, 0.0), (1.0, 1.0, 1.0)],
    ];

    let weights = vec![vec![1.0, 1.0], vec![1.0, 1.0]];

    let u_knots = vec![0.0, 0.0, 1.0, 1.0];
    let v_knots = vec![0.0, 0.0, 1.0, 1.0];

    let result = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::new(
        control_points,
        Some(weights),
        u_knots,
        v_knots,
        1, // u_degree
        1, // v_degree
    );

    assert!(result.is_ok());
    let surface = result.unwrap();

    // Properties検証
    assert_eq!(
        <NurbsSurface3D<f64> as NurbsSurface3DProperties<f64>>::u_degree(&surface),
        1
    );
    assert_eq!(
        <NurbsSurface3D<f64> as NurbsSurface3DProperties<f64>>::v_degree(&surface),
        1
    );
    assert_eq!(
        <NurbsSurface3D<f64> as NurbsSurface3DProperties<f64>>::u_count(&surface),
        2
    );
    assert_eq!(
        <NurbsSurface3D<f64> as NurbsSurface3DProperties<f64>>::v_count(&surface),
        2
    );
    assert!(<NurbsSurface3D<f64> as NurbsSurface3DProperties<f64>>::is_rational(&surface));
}

#[test]
fn test_constructor_from_control_points() {
    // クランプドノットベクトルを自動生成
    let control_points = vec![
        vec![(0.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 2.0, 0.0)],
        vec![(1.0, 0.0, 0.0), (1.0, 1.0, 0.5), (1.0, 2.0, 0.0)],
        vec![(2.0, 0.0, 0.0), (2.0, 1.0, 0.0), (2.0, 2.0, 1.0)],
    ];

    let result = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::from_control_points(
        control_points,
        2, // u_degree
        2, // v_degree
    );

    assert!(result.is_ok());
    let surface = result.unwrap();

    assert_eq!(
        <NurbsSurface3D<f64> as NurbsSurface3DProperties<f64>>::u_count(&surface),
        3
    );
    assert_eq!(
        <NurbsSurface3D<f64> as NurbsSurface3DProperties<f64>>::v_count(&surface),
        3
    );
    assert_eq!(
        <NurbsSurface3D<f64> as NurbsSurface3DProperties<f64>>::u_degree(&surface),
        2
    );
    assert_eq!(
        <NurbsSurface3D<f64> as NurbsSurface3DProperties<f64>>::v_degree(&surface),
        2
    );
    assert!(!<NurbsSurface3D<f64> as NurbsSurface3DProperties<f64>>::is_rational(&surface));
}

#[test]
fn test_constructor_unit_plane() {
    let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::unit_plane();

    assert_eq!(
        <NurbsSurface3D<f64> as NurbsSurface3DProperties<f64>>::u_degree(&surface),
        1
    );
    assert_eq!(
        <NurbsSurface3D<f64> as NurbsSurface3DProperties<f64>>::v_degree(&surface),
        1
    );

    // 単位平面の点評価
    let (x, y, z) =
        <NurbsSurface3D<f64> as geo_contracts::NurbsSurface3DEvaluation<f64>>::point_at_uv(
            &surface, 0.5, 0.5,
        );
    assert!((x - 0.5).abs() < 1e-10);
    assert!((y - 0.5).abs() < 1e-10);
    assert!(z.abs() < 1e-10);
}

#[test]
fn test_properties_knot_vectors() {
    let control_points = vec![
        vec![(0.0, 0.0, 0.0), (0.0, 1.0, 0.0)],
        vec![(1.0, 0.0, 0.0), (1.0, 1.0, 0.0)],
    ];

    let u_knots = vec![0.0, 0.0, 1.0, 1.0];
    let v_knots = vec![0.0, 0.0, 2.0, 2.0];

    let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::new(
        control_points,
        None,
        u_knots.clone(),
        v_knots.clone(),
        1,
        1,
    )
    .unwrap();

    let u_knots_ref = <NurbsSurface3D<f64> as NurbsSurface3DProperties<f64>>::u_knots(&surface);
    let v_knots_ref = <NurbsSurface3D<f64> as NurbsSurface3DProperties<f64>>::v_knots(&surface);

    assert_eq!(u_knots_ref, &u_knots[..]);
    assert_eq!(v_knots_ref, &v_knots[..]);
}

#[test]
fn test_measure_point_at_uv() {
    let control_points = vec![
        vec![(0.0, 0.0, 0.0), (0.0, 1.0, 0.0)],
        vec![(1.0, 0.0, 0.0), (1.0, 1.0, 0.0)],
    ];

    let u_knots = vec![0.0, 0.0, 1.0, 1.0];
    let v_knots = vec![0.0, 0.0, 1.0, 1.0];

    let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::new(
        control_points,
        None,
        u_knots,
        v_knots,
        1,
        1,
    )
    .unwrap();

    // 平面上の点評価
    let (x, y, z) =
        <NurbsSurface3D<f64> as geo_contracts::NurbsSurface3DEvaluation<f64>>::point_at_uv(
            &surface, 0.0, 0.0,
        );
    assert!((x - 0.0).abs() < 1e-10);
    assert!((y - 0.0).abs() < 1e-10);
    assert!(z.abs() < 1e-10);

    let (x, y, z) =
        <NurbsSurface3D<f64> as geo_contracts::NurbsSurface3DEvaluation<f64>>::point_at_uv(
            &surface, 1.0, 1.0,
        );
    assert!((x - 1.0).abs() < 1e-10);
    assert!((y - 1.0).abs() < 1e-10);
    assert!(z.abs() < 1e-10);
}

#[test]
fn test_measure_normal_at() {
    let control_points = vec![
        vec![(0.0, 0.0, 0.0), (0.0, 1.0, 0.0)],
        vec![(1.0, 0.0, 0.0), (1.0, 1.0, 0.0)],
    ];

    let u_knots = vec![0.0, 0.0, 1.0, 1.0];
    let v_knots = vec![0.0, 0.0, 1.0, 1.0];

    let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::new(
        control_points,
        None,
        u_knots,
        v_knots,
        1,
        1,
    )
    .unwrap();

    // 平面の法線ベクトルはz方向
    let (nx, ny, nz) =
        <NurbsSurface3D<f64> as geo_contracts::NurbsSurface3DEvaluation<f64>>::normal_at(
            &surface, 0.5, 0.5,
        );

    // 正規化確認
    let len = (nx * nx + ny * ny + nz * nz).sqrt();
    assert!((len - 1.0).abs() < 1e-10);

    // z方向を向いている
    assert!(nz.abs() > 0.9);
}

#[test]
fn test_measure_surface_area() {
    let control_points = vec![
        vec![(0.0, 0.0, 0.0), (0.0, 1.0, 0.0)],
        vec![(1.0, 0.0, 0.0), (1.0, 1.0, 0.0)],
    ];

    let u_knots = vec![0.0, 0.0, 1.0, 1.0];
    let v_knots = vec![0.0, 0.0, 1.0, 1.0];

    let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::new(
        control_points,
        None,
        u_knots,
        v_knots,
        1,
        1,
    )
    .unwrap();

    // 1x1 平面の面積は約1.0
    let area = <NurbsSurface3D<f64> as NurbsSurface3DDerived<f64>>::surface_area(&surface);
    assert!((area - 1.0).abs() < 0.1); // 数値積分の誤差を許容
}

#[test]
#[allow(clippy::similar_names)]
fn test_measure_tangent_vectors_at() {
    let control_points = vec![
        vec![(0.0, 0.0, 0.0), (0.0, 1.0, 0.0)],
        vec![(1.0, 0.0, 0.0), (1.0, 1.0, 0.0)],
    ];

    let u_knots = vec![0.0, 0.0, 1.0, 1.0];
    let v_knots = vec![0.0, 0.0, 1.0, 1.0];

    let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::new(
        control_points,
        None,
        u_knots,
        v_knots,
        1,
        1,
    )
    .unwrap();

    let ((du_x, du_y, du_z), (dv_x, dv_y, dv_z)) =
        <NurbsSurface3D<f64> as geo_contracts::NurbsSurface3DEvaluation<f64>>::tangent_vectors_at(
            &surface, 0.5, 0.5,
        );

    // 平面のタンジェントベクトル
    // u方向はx軸方向
    assert!((du_x - 1.0).abs() < 0.1);
    assert!(du_y.abs() < 0.1);
    assert!(du_z.abs() < 0.1);

    // v方向はy軸方向
    assert!(dv_x.abs() < 0.1);
    assert!((dv_y - 1.0).abs() < 0.1);
    assert!(dv_z.abs() < 0.1);
}

#[test]
fn test_metadata_and_surface_area_capabilities() {
    let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::unit_plane();

    // PrimitiveKind
    let kind = <NurbsSurface3D<f64> as PrimitiveMetadata>::primitive_kind(&surface);
    assert_eq!(kind, PrimitiveKind::NurbsSurface3D);

    // Derived capability (面積)
    let area = <NurbsSurface3D<f64> as NurbsSurface3DDerived<f64>>::surface_area(&surface);
    assert!((area - 1.0).abs() < 0.1); // 単位平面の面積は1.0
}

#[test]
fn test_bounded_aabb() {
    let control_points = vec![
        vec![(0.0, 0.0, 0.0), (0.0, 2.0, 0.0)],
        vec![(3.0, 0.0, 0.0), (3.0, 2.0, 1.0)],
    ];

    let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::from_control_points(
        control_points,
        1,
        1,
    )
    .unwrap();

    let aabb_opt = <NurbsSurface3D<f64> as Bounded<f64>>::aabb(&surface);
    assert!(aabb_opt.is_some());

    let aabb = aabb_opt.unwrap();

    // 制御点ベースの境界ボックス
    let min = aabb.min();
    let max = aabb.max();

    assert!((min.x() - 0.0).abs() < 1e-10);
    assert!((min.y() - 0.0).abs() < 1e-10);
    assert!((min.z() - 0.0).abs() < 1e-10);

    assert!((max.x() - 3.0).abs() < 1e-10);
    assert!((max.y() - 2.0).abs() < 1e-10);
    assert!((max.z() - 1.0).abs() < 1e-10);
}

#[test]
fn test_surface_adaptive_params_plane() {
    let control_points = vec![
        vec![(0.0, 0.0, 0.0), (0.0, 1.0, 0.0)],
        vec![(1.0, 0.0, 0.0), (1.0, 1.0, 0.0)],
    ];

    let u_knots = clamped_knot_vector(1, 2);
    let v_knots = clamped_knot_vector(1, 2);

    let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::new(
        control_points,
        None,
        u_knots,
        v_knots,
        1,
        1,
    )
    .unwrap();

    let settings = AdaptiveTessellationSettings::default_with_tolerance(0.01);
    let grid = surface.adaptive_params_surface(&settings);

    assert!(grid.u_params.len() > settings.min_segments as usize);
    assert!(grid.v_params.len() > settings.min_segments as usize);

    let (u_min, u_max) = surface.parameter_domain().0;
    let (v_min, v_max) = surface.parameter_domain().1;
    assert!((grid.u_params.first().unwrap() - u_min).abs() < 1e-10);
    assert!((grid.u_params.last().unwrap() - u_max).abs() < 1e-10);
    assert!((grid.v_params.first().unwrap() - v_min).abs() < 1e-10);
    assert!((grid.v_params.last().unwrap() - v_max).abs() < 1e-10);
}
