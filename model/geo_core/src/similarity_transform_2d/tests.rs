//! SimilarityTransform2D のテスト

use crate::{Aabb2D, Direction2D, Point2D, SimilarityTransform2D, Vector2D};
use analysis::linalg::matrix::Matrix3x3;
use analysis::test_constants::TOLERANCE_F64;
use geo_contracts::{Angle, SimilarityTransform2DCore, SimilarityTransformable2D, TransformError};

fn assert_point_eq(actual: Point2D<f64>, expected: (f64, f64)) {
    assert!(
        (actual.x() - expected.0).abs() < TOLERANCE_F64
            && (actual.y() - expected.1).abs() < TOLERANCE_F64,
        "actual {actual:?}, expected {expected:?}"
    );
}

#[test]
fn composes_rotation_translation_and_scale_into_one_transform() {
    let transform =
        SimilarityTransform2D::rotation_about(Point2D::new(0.0, 0.0), Angle::from_degrees(90.0))
            .then(&SimilarityTransform2D::translation(Vector2D::new(0.0, 5.0)))
            .then(
                &SimilarityTransform2D::uniform_scale_about(Point2D::new(0.0, 0.0), 2.0).unwrap(),
            );

    // (1, 0) → 回転 (0, 1) → 平行移動 (0, 6) → 2 倍 (0, 12)
    let moved = Point2D::new(1.0, 0.0)
        .transform_similarity(&transform)
        .unwrap();
    assert_point_eq(moved, (0.0, 12.0));
    assert!((transform.scale_factor() - 2.0).abs() < TOLERANCE_F64);
}

#[test]
fn rotates_and_scales_about_given_center() {
    let rotation =
        SimilarityTransform2D::rotation_about(Point2D::new(1.0, 1.0), Angle::from_degrees(180.0));
    assert_point_eq(
        Point2D::new(2.0, 1.0)
            .transform_similarity(&rotation)
            .unwrap(),
        (0.0, 1.0),
    );

    let scale = SimilarityTransform2D::uniform_scale_about(Point2D::new(1.0, 1.0), 3.0).unwrap();
    assert_point_eq(
        Point2D::new(2.0, 1.0).transform_similarity(&scale).unwrap(),
        (4.0, 1.0),
    );
}

#[test]
fn non_positive_scale_is_invalid_parameter() {
    for factor in [0.0, -1.0] {
        assert!(matches!(
            SimilarityTransform2D::uniform_scale_about(Point2D::new(0.0, 0.0), factor),
            Err(TransformError::InvalidParameter(_))
        ));
    }
}

#[test]
fn vector_ignores_translation_and_direction_stays_unit() {
    let transform = SimilarityTransform2D::translation(Vector2D::new(10.0, 0.0))
        .then(&SimilarityTransform2D::uniform_scale_about(Point2D::new(0.0, 0.0), 4.0).unwrap());

    let vector = Vector2D::new(1.0, 2.0)
        .transform_similarity(&transform)
        .unwrap();
    assert!((vector - Vector2D::new(4.0, 8.0)).length() < TOLERANCE_F64);

    let direction = Direction2D::new(1.0, 1.0)
        .unwrap()
        .transform_similarity(&transform)
        .unwrap();
    assert!((Vector2D::new(direction.x(), direction.y()).length() - 1.0).abs() < TOLERANCE_F64);
}

#[test]
fn aabb_accepts_translation_and_scale_but_rejects_rotation() {
    let aabb = Aabb2D::new(Point2D::new(0.0, 0.0), Point2D::new(1.0, 2.0));
    let transform = SimilarityTransform2D::uniform_scale_about(Point2D::new(0.0, 0.0), 2.0)
        .unwrap()
        .then(&SimilarityTransform2D::translation(Vector2D::new(1.0, 1.0)));
    let moved = aabb.transform_similarity(&transform).unwrap();
    assert_point_eq(moved.min_point(), (1.0, 1.0));
    assert_point_eq(moved.max_point(), (3.0, 5.0));

    let rotation =
        SimilarityTransform2D::rotation_about(Point2D::new(0.0, 0.0), Angle::from_degrees(30.0));
    assert!(matches!(
        aabb.transform_similarity(&rotation),
        Err(TransformError::Unsupported(_))
    ));
}

#[test]
fn from_matrix_accepts_similarity_and_rejects_others() {
    let original = SimilarityTransform2D::rotation_about(
        Point2D::new(1.0_f64, 2.0),
        Angle::from_degrees(45.0),
    )
    .then(&SimilarityTransform2D::uniform_scale_about(Point2D::new(0.0, 0.0), 1.5).unwrap());
    let rebuilt = SimilarityTransform2D::from_matrix(original.to_matrix()).unwrap();
    assert!((rebuilt.scale_factor() - 1.5).abs() < TOLERANCE_F64);

    for matrix in [
        Matrix3x3::scale_2d(&analysis::linalg::vector::Vector2::new(1.0, 2.0)),
        Matrix3x3::reflection_x_2d(),
    ] {
        assert!(matches!(
            SimilarityTransform2D::from_matrix(matrix),
            Err(TransformError::Unsupported(_))
        ));
    }
    assert!(matches!(
        SimilarityTransform2D::from_matrix(Matrix3x3::uniform_scale_2d(0.0)),
        Err(TransformError::InvalidParameter(_))
    ));
}

#[test]
fn renormalized_restores_orthogonality_after_many_compositions() {
    let step =
        SimilarityTransform2D::rotation_about(Point2D::new(0.3, -0.2), Angle::from_degrees(0.7));
    let mut transform = SimilarityTransform2D::identity();
    for _ in 0..10_000 {
        transform = transform.then(&step);
    }

    let renormalized = transform.renormalized();
    assert!(renormalized.orthogonality_error() <= transform.orthogonality_error());
    assert!(renormalized.orthogonality_error() < TOLERANCE_F64);

    let p = Point2D::new(1.0, -2.0);
    let a = p.transform_similarity(&transform).unwrap();
    let b = p.transform_similarity(&renormalized).unwrap();
    assert!((a - b).length() < TOLERANCE_F64);
}
