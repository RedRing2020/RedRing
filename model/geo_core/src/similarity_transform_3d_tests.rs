//! SimilarityTransform3D のテスト

use crate::{Aabb3D, Direction3D, Point3D, SimilarityTransform3D, Vector3D};
use analysis::linalg::matrix::Matrix4x4;
use analysis::test_constants::TOLERANCE_F64;
use geo_contracts::{Angle, SimilarityTransform3DCore, SimilarityTransformable3D, TransformError};

fn assert_point_eq(actual: Point3D<f64>, expected: (f64, f64, f64)) {
    assert!(
        (actual.x() - expected.0).abs() < TOLERANCE_F64
            && (actual.y() - expected.1).abs() < TOLERANCE_F64
            && (actual.z() - expected.2).abs() < TOLERANCE_F64,
        "actual {actual:?}, expected {expected:?}"
    );
}

fn z_axis() -> Direction3D<f64> {
    Direction3D::new(0.0, 0.0, 1.0).unwrap()
}

#[test]
fn composes_rotation_translation_and_scale_into_one_transform() {
    let rotation = SimilarityTransform3D::rotation_about_axis(
        Point3D::origin(),
        z_axis(),
        Angle::from_degrees(90.0),
    );
    let translation = SimilarityTransform3D::translation(Vector3D::new(0.0, 0.0, 5.0));
    let scale = SimilarityTransform3D::uniform_scale_about(Point3D::origin(), 2.0).unwrap();
    let transform = rotation.then(&translation).then(&scale);

    // (1, 0, 0) → 回転 (0, 1, 0) → 平行移動 (0, 1, 5) → 2 倍 (0, 2, 10)
    let moved = Point3D::new(1.0, 0.0, 0.0)
        .transform_similarity(&transform)
        .unwrap();
    assert_point_eq(moved, (0.0, 2.0, 10.0));
    assert!((transform.scale_factor() - 2.0).abs() < TOLERANCE_F64);
}

#[test]
fn rotates_about_given_center() {
    let rotation = SimilarityTransform3D::rotation_about_axis(
        Point3D::new(1.0, 1.0, 0.0),
        z_axis(),
        Angle::from_degrees(180.0),
    );
    let moved = Point3D::new(2.0, 1.0, 3.0)
        .transform_similarity(&rotation)
        .unwrap();
    assert_point_eq(moved, (0.0, 1.0, 3.0));
}

#[test]
fn scales_about_given_center() {
    let scale =
        SimilarityTransform3D::uniform_scale_about(Point3D::new(1.0, 1.0, 1.0), 3.0).unwrap();
    let moved = Point3D::new(2.0, 1.0, 1.0)
        .transform_similarity(&scale)
        .unwrap();
    assert_point_eq(moved, (4.0, 1.0, 1.0));
}

#[test]
fn non_positive_scale_is_invalid_parameter() {
    for factor in [0.0, -1.0] {
        assert!(matches!(
            SimilarityTransform3D::uniform_scale_about(Point3D::origin(), factor),
            Err(TransformError::InvalidParameter(_))
        ));
    }
}

#[test]
fn vector_ignores_translation_and_direction_stays_unit() {
    let transform = SimilarityTransform3D::translation(Vector3D::new(10.0, 0.0, 0.0))
        .then(&SimilarityTransform3D::uniform_scale_about(Point3D::origin(), 4.0).unwrap());

    let vector = Vector3D::new(1.0, 2.0, 3.0)
        .transform_similarity(&transform)
        .unwrap();
    assert!((vector - Vector3D::new(4.0, 8.0, 12.0)).length() < TOLERANCE_F64);

    let direction = Direction3D::new(1.0, 1.0, 0.0)
        .unwrap()
        .transform_similarity(&transform)
        .unwrap();
    assert!((direction.as_vector().length() - 1.0).abs() < TOLERANCE_F64);
}

#[test]
fn direction_rotates_with_transform() {
    let rotation = SimilarityTransform3D::rotation_about_axis(
        Point3D::new(5.0, 5.0, 5.0),
        z_axis(),
        Angle::from_degrees(90.0),
    );
    let direction = Direction3D::new(1.0, 0.0, 0.0)
        .unwrap()
        .transform_similarity(&rotation)
        .unwrap();
    assert!((direction.as_vector() - Vector3D::new(0.0, 1.0, 0.0)).length() < TOLERANCE_F64);
}

#[test]
fn aabb_accepts_translation_and_uniform_scale() {
    let aabb = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 2.0, 3.0));
    let transform = SimilarityTransform3D::uniform_scale_about(Point3D::origin(), 2.0)
        .unwrap()
        .then(&SimilarityTransform3D::translation(Vector3D::new(
            1.0, 1.0, 1.0,
        )));

    let moved = aabb.transform_similarity(&transform).unwrap();
    assert_point_eq(moved.min(), (1.0, 1.0, 1.0));
    assert_point_eq(moved.max(), (3.0, 5.0, 7.0));
}

#[test]
fn aabb_rejects_rotation_but_accepts_cancelled_rotation() {
    let aabb = Aabb3D::new(Point3D::origin(), Point3D::new(1.0, 1.0, 1.0));
    let rotation = SimilarityTransform3D::rotation_about_axis(
        Point3D::origin(),
        z_axis(),
        Angle::from_degrees(30.0),
    );
    assert!(matches!(
        aabb.transform_similarity(&rotation),
        Err(TransformError::Unsupported(_))
    ));

    // 回転の有無は合成後の行列で判定するため、打ち消し合う回転は回転なしとして扱う
    let inverse = SimilarityTransform3D::rotation_about_axis(
        Point3D::origin(),
        z_axis(),
        Angle::from_degrees(-30.0),
    );
    assert!(aabb.transform_similarity(&rotation.then(&inverse)).is_ok());
}

#[test]
fn from_matrix_accepts_similarity_and_round_trips() {
    let original = SimilarityTransform3D::rotation_about_axis(
        Point3D::new(1.0, 2.0, 3.0),
        z_axis(),
        Angle::from_degrees(45.0),
    )
    .then(&SimilarityTransform3D::uniform_scale_about(Point3D::origin(), 1.5).unwrap());

    let rebuilt = SimilarityTransform3D::from_matrix(original.to_matrix()).unwrap();
    assert!((rebuilt.scale_factor() - 1.5).abs() < TOLERANCE_F64);
    let p = Point3D::new(3.0, -1.0, 2.0);
    let a = p.transform_similarity(&original).unwrap();
    let b = p.transform_similarity(&rebuilt).unwrap();
    assert_point_eq(b, (a.x(), a.y(), a.z()));
}

#[test]
fn from_matrix_rejects_unsupported_and_singular_matrices() {
    let unsupported = [
        Matrix4x4::scale(1.0, 2.0, 1.0),
        Matrix4x4::scale(-1.0, 1.0, 1.0),
        Matrix4x4::perspective(1.0, 1.0, 0.1, 100.0),
    ];
    for matrix in unsupported {
        assert!(matches!(
            SimilarityTransform3D::from_matrix(matrix),
            Err(TransformError::Unsupported(_))
        ));
    }
    assert!(matches!(
        SimilarityTransform3D::from_matrix(Matrix4x4::scale(0.0, 0.0, 0.0)),
        Err(TransformError::InvalidParameter(_))
    ));
}

#[test]
fn renormalized_restores_orthogonality_after_many_compositions() {
    let step = SimilarityTransform3D::rotation_about_axis(
        Point3D::origin(),
        Direction3D::new(1.0, 2.0, 3.0).unwrap(),
        Angle::from_degrees(0.7),
    );
    let mut transform = SimilarityTransform3D::identity();
    for _ in 0..10_000 {
        transform = transform.then(&step);
    }

    let renormalized = transform.renormalized();
    assert!(renormalized.orthogonality_error() <= transform.orthogonality_error());
    assert!(renormalized.orthogonality_error() < TOLERANCE_F64);

    // 再正規化しても写像はほぼ変わらない
    let p = Point3D::new(1.0, -2.0, 0.5);
    let a = p.transform_similarity(&transform).unwrap();
    let b = p.transform_similarity(&renormalized).unwrap();
    assert!(a.distance_to(&b) < TOLERANCE_F64);
}
