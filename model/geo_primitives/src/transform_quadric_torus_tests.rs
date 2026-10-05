//! 二次曲面・トーラス（面・立体）の相似変換テスト

use crate::{
    ConicalSolid3D, ConicalSurface3D, CylindricalSolid3D, CylindricalSurface3D, Direction3D,
    EllipsoidalSolid3D, EllipsoidalSurface3D, Point3D, SimilarityTransform3D, SphericalSolid3D,
    SphericalSurface3D, TorusSolid3D, TorusSurface3D, Vector3D,
};
use analysis::test_constants::TOLERANCE_F64;
use geo_contracts::{
    Angle, ConicalSolid3DContainment, ConicalSolid3DDerived, ConicalSurface3DEvaluation,
    CylindricalSolid3DContainment, CylindricalSolid3DDerived, CylindricalSurface3DEvaluation,
    EllipsoidalSolid3DContainment, EllipsoidalSolid3DDerived, EllipsoidalSurface3DEvaluation,
    SimilarityTransformable3D, SphericalSolid3DContainment, SphericalSolid3DDerived,
    SphericalSurface3DEvaluation, TorusSolid3DContainment, TorusSolid3DDerived,
    TorusSurface3DEvaluation,
};

const SCALE: f64 = 2.0;

/// 斜めの軸まわりに 50° 回転 → 平行移動 → 原点まわりに 2 倍
fn transform() -> SimilarityTransform3D<f64> {
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

fn map(p: (f64, f64, f64)) -> (f64, f64, f64) {
    let q = Point3D::from(p).transform_similarity(&transform()).unwrap();
    (q.x(), q.y(), q.z())
}

/// 各 (u, v) の点が、点の変換と一致することを確認する
///
/// `v_scale` は変換後の v パラメータの倍率。角度パラメータは 1、軸方向の距離パラメータ
/// （円筒面・円錐面の v）はスケール係数となる。
fn assert_surface_maps(
    v_scale: f64,
    original: impl Fn(f64, f64) -> (f64, f64, f64),
    moved: impl Fn(f64, f64) -> (f64, f64, f64),
) {
    for (u, v) in [(0.0, 0.0), (0.4, 0.3), (1.9, -0.7), (4.0, 1.2)] {
        let expected = Point3D::from(map(original(u, v)));
        let actual = Point3D::from(moved(u, v * v_scale));
        assert!(
            actual.distance_to(&expected) < TOLERANCE_F64,
            "(u, v) = ({u}, {v}): {actual:?} != {expected:?}"
        );
    }
}

/// 内外判定が変換の前後で一致することを確認する（境界から離れた点で判定）
fn assert_containment_maps(
    original: impl Fn((f64, f64, f64)) -> bool,
    moved: impl Fn((f64, f64, f64)) -> bool,
) {
    let mut inside = 0;
    let mut outside = 0;
    for x in [-2.7, -1.3, -0.2, 0.3, 1.1, 2.6] {
        for y in [-2.1, -0.4, 0.2, 1.7] {
            for z in [-1.6, -0.3, 0.4, 0.9, 2.3] {
                let p = (x, y, z);
                let before = original(p);
                assert_eq!(before, moved(map(p)), "probe {p:?}");
                if before {
                    inside += 1
                } else {
                    outside += 1
                }
            }
        }
    }
    // 判定が片方に偏っていないこと（テストの有効性の確認）
    assert!(inside > 0 && outside > 0);
}

fn assert_volume_scales(original: f64, moved: f64) {
    let expected = original * SCALE.powi(3);
    assert!((moved - expected).abs() < TOLERANCE_F64 * expected.max(1.0));
}

#[test]
fn spherical_surface_and_solid() {
    let surface = SphericalSurface3D::new_standard(Point3D::new(0.2, 0.1, 0.0), 1.5).unwrap();
    let moved = surface.transform_similarity(&transform()).unwrap();
    assert_surface_maps(
        1.0,
        |u, v| SphericalSurface3DEvaluation::point_at_uv(&surface, u, v),
        |u, v| SphericalSurface3DEvaluation::point_at_uv(&moved, u, v),
    );

    let solid = SphericalSolid3D::new_standard(Point3D::new(0.2, 0.1, 0.0), 1.5).unwrap();
    let moved = solid.transform_similarity(&transform()).unwrap();
    assert_volume_scales(
        SphericalSolid3DDerived::volume(&solid),
        SphericalSolid3DDerived::volume(&moved),
    );
    assert_containment_maps(
        |p| SphericalSolid3DContainment::contains_point(&solid, p),
        |p| SphericalSolid3DContainment::contains_point(&moved, p),
    );
}

#[test]
fn cylindrical_surface_and_solid() {
    let surface = CylindricalSurface3D::new_z_axis(Point3D::new(0.2, 0.1, 0.0), 1.5).unwrap();
    let moved = surface.transform_similarity(&transform()).unwrap();
    assert_surface_maps(
        SCALE,
        |u, v| CylindricalSurface3DEvaluation::point_at_uv(&surface, u, v),
        |u, v| CylindricalSurface3DEvaluation::point_at_uv(&moved, u, v),
    );

    let solid = CylindricalSolid3D::new_z_axis(Point3D::new(0.2, 0.1, -1.0), 1.5, 2.5).unwrap();
    let moved = solid.transform_similarity(&transform()).unwrap();
    assert_volume_scales(
        CylindricalSolid3DDerived::volume(&solid),
        CylindricalSolid3DDerived::volume(&moved),
    );
    assert_containment_maps(
        |p| CylindricalSolid3DContainment::contains_point(&solid, p),
        |p| CylindricalSolid3DContainment::contains_point(&moved, p),
    );
}

#[test]
fn conical_surface_and_solid() {
    let surface = ConicalSurface3D::new(
        Point3D::new(0.2, 0.1, 0.0),
        Vector3D::new(0.0, 0.0, 1.0),
        Vector3D::new(1.0, 0.0, 0.0),
        1.5,
        0.4,
    )
    .unwrap();
    let moved = surface.transform_similarity(&transform()).unwrap();
    assert!((moved.semi_angle_internal() - 0.4).abs() < TOLERANCE_F64);
    assert_surface_maps(
        SCALE,
        |u, v| ConicalSurface3DEvaluation::point_at_uv(&surface, u, v),
        |u, v| ConicalSurface3DEvaluation::point_at_uv(&moved, u, v),
    );

    let solid = ConicalSolid3D::new_standard(Point3D::new(0.2, 0.1, -1.0), 1.8, 2.5).unwrap();
    let moved = solid.transform_similarity(&transform()).unwrap();
    assert_volume_scales(
        ConicalSolid3DDerived::volume(&solid),
        ConicalSolid3DDerived::volume(&moved),
    );
    assert_containment_maps(
        |p| ConicalSolid3DContainment::contains_point(&solid, p),
        |p| ConicalSolid3DContainment::contains_point(&moved, p),
    );
}

#[test]
fn ellipsoidal_surface_and_solid() {
    let surface = EllipsoidalSurface3D::new(
        Point3D::new(0.2, 0.1, 0.0),
        Vector3D::new(0.0, 0.0, 1.0),
        Vector3D::new(1.0, 0.0, 0.0),
        2.0,
        1.5,
        1.0,
    )
    .unwrap();
    let moved = surface.transform_similarity(&transform()).unwrap();
    assert_surface_maps(
        1.0,
        |u, v| EllipsoidalSurface3DEvaluation::point_at_uv(&surface, u, v),
        |u, v| EllipsoidalSurface3DEvaluation::point_at_uv(&moved, u, v),
    );

    let solid =
        EllipsoidalSolid3D::new_standard(Point3D::new(0.2, 0.1, 0.0), 2.0, 1.5, 1.0).unwrap();
    let moved = solid.transform_similarity(&transform()).unwrap();
    assert_volume_scales(
        EllipsoidalSolid3DDerived::volume(&solid),
        EllipsoidalSolid3DDerived::volume(&moved),
    );
    assert_containment_maps(
        |p| EllipsoidalSolid3DContainment::contains_point(&solid, p),
        |p| EllipsoidalSolid3DContainment::contains_point(&moved, p),
    );
}

#[test]
fn torus_surface_and_solid() {
    let surface = TorusSurface3D::standard(2.0, 0.7).unwrap();
    let moved = surface.transform_similarity(&transform()).unwrap();
    assert!((moved.major_radius_internal() - 4.0).abs() < TOLERANCE_F64);
    assert!((moved.minor_radius_internal() - 1.4).abs() < TOLERANCE_F64);
    assert_surface_maps(
        1.0,
        |u, v| TorusSurface3DEvaluation::point_at_uv(&surface, u, v),
        |u, v| TorusSurface3DEvaluation::point_at_uv(&moved, u, v),
    );

    let solid = TorusSolid3D::standard(2.0, 0.7).unwrap();
    let moved = solid.transform_similarity(&transform()).unwrap();
    assert_volume_scales(
        TorusSolid3DDerived::volume(&solid),
        TorusSolid3DDerived::volume(&moved),
    );
    assert_containment_maps(
        |p| TorusSolid3DContainment::contains_point(&solid, p),
        |p| TorusSolid3DContainment::contains_point(&moved, p),
    );
}
