use crate::linalg::matrix::Matrix3x3;
use crate::linalg::vector::Vector2;
use std::f64::consts::PI;

#[cfg(test)]
mod tests {
    use super::*;

    type Matrix3 = Matrix3x3<f64>;
    type Vec2 = Vector2<f64>;

    #[test]
    fn test_2d_translation() {
        let translation = Vec2::new(5.0, 3.0);
        let matrix = Matrix3::translation_2d(&translation);
        let point = Vec2::new(1.0, 2.0);
        let result = matrix.transform_point_2d(&point);

        assert!((result.x() - 6.0).abs() < f64::EPSILON);
        assert!((result.y() - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_2d_rotation() {
        let angle = PI / 2.0;
        let matrix = Matrix3::rotation_2d(angle);
        let point = Vec2::new(1.0, 0.0);
        let result = matrix.transform_point_2d(&point);

        assert!(result.x().abs() < f64::EPSILON);
        assert!((result.y() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_2d_scale() {
        let scale = Vec2::new(2.0, 3.0);
        let matrix = Matrix3::scale_2d(&scale);
        let point = Vec2::new(1.0, 1.0);
        let result = matrix.transform_point_2d(&point);

        assert!((result.x() - 2.0).abs() < f64::EPSILON);
        assert!((result.y() - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_2d_trs_composition() {
        let translation = Vec2::new(10.0, 5.0);
        let rotation = PI / 4.0;
        let scale = Vec2::new(2.0, 2.0);

        let matrix = Matrix3::trs_2d(&translation, rotation, &scale);
        let point = Vec2::new(1.0, 0.0);
        let result = matrix.transform_point_2d(&point);

        let expected_x = 10.0 + 2.0 * (PI / 4.0).cos();
        let expected_y = 5.0 + 2.0 * (PI / 4.0).sin();

        assert!((result.x() - expected_x).abs() < 1e-10);
        assert!((result.y() - expected_y).abs() < 1e-10);
    }

    #[test]
    fn test_vector_multiplication_operator() {
        let translation = Vec2::new(2.0, 3.0);
        let matrix = Matrix3::translation_2d(&translation);
        let point = Vec2::new(1.0, 1.0);

        let result = matrix * point;

        assert!((result.x() - 3.0).abs() < f64::EPSILON);
        assert!((result.y() - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rigid_body_detection() {
        let translation = Vec2::new(5.0, 3.0);
        let rotation = PI / 4.0;
        let rigid_matrix =
            Matrix3::translation_2d(&translation) * Matrix3::rotation_2d(rotation);

        assert!(rigid_matrix.is_rigid_2d());

        let scale = Vec2::new(2.0, 2.0);
        let scaled_matrix = Matrix3::scale_2d(&scale);
        assert!(!scaled_matrix.is_rigid_2d());
    }

    #[test]
    fn test_affine_transform_detection() {
        let affine_matrix =
            Matrix3::trs_2d(&Vec2::new(5.0, 3.0), PI / 4.0, &Vec2::new(2.0, 1.5));

        assert!(affine_matrix.is_affine_transform());
        assert!(affine_matrix.is_pure_affine_2d());
        assert!(affine_matrix.is_valid_affine());
        assert!(!affine_matrix.has_perspective());

        let perspective_matrix = Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.1, 0.05, 1.0);

        assert!(!perspective_matrix.is_affine_transform());
        assert!(perspective_matrix.has_perspective());
    }

    #[test]
    fn test_homogeneous_normalization() {
        let matrix = Matrix3::new(2.0, 0.0, 4.0, 0.0, 2.0, 6.0, 0.0, 0.0, 2.0);

        let normalized = matrix.normalize_homogeneous().unwrap();
        let expected = Matrix3::new(1.0, 0.0, 2.0, 0.0, 1.0, 3.0, 0.0, 0.0, 1.0);

        for i in 0..3 {
            for j in 0..3 {
                assert!((normalized.get(i, j) - expected.get(i, j)).abs() < f64::EPSILON);
            }
        }
    }

    #[test]
    fn test_projective_transformation() {
        let matrix = Matrix3::new(1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.1, 0.0, 1.0);

        let point = Vec2::new(1.0, 1.0);
        let result = matrix.transform_projective_2d(point).unwrap();

        let expected_x = 2.0 / 1.1;
        let expected_y = 2.0 / 1.1;

        assert!((result.x() - expected_x).abs() < 1e-10);
        assert!((result.y() - expected_y).abs() < 1e-10);
    }

    #[test]
    fn test_affine_factory_methods() {
        let linear = [[2.0, 0.5], [0.0, 1.5]];
        let translation = Vec2::new(3.0, 4.0);

        let affine_matrix = Matrix3::affine_2d(linear, translation);

        assert!(affine_matrix.is_affine_transform());
        assert_eq!(affine_matrix.get(0, 0), 2.0);
        assert_eq!(affine_matrix.get(0, 1), 0.5);
        assert_eq!(affine_matrix.get(0, 2), 3.0);
        assert_eq!(affine_matrix.get(1, 0), 0.0);
        assert_eq!(affine_matrix.get(1, 1), 1.5);
        assert_eq!(affine_matrix.get(1, 2), 4.0);
        assert_eq!(affine_matrix.get(2, 0), 0.0);
        assert_eq!(affine_matrix.get(2, 1), 0.0);
        assert_eq!(affine_matrix.get(2, 2), 1.0);
    }
}
