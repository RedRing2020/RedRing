use crate::linalg::matrix::DynamicMatrix;
use crate::linalg::vector::Vector;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_matrix_new_and_shape() {
        let matrix = DynamicMatrix::<f64>::new(2, 3, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap();

        assert_eq!(matrix.rows(), 2);
        assert_eq!(matrix.cols(), 3);
        assert_eq!(matrix.shape(), (2, 3));
        assert_eq!(matrix.get(1, 2), 6.0);
    }

    #[test]
    fn test_dynamic_matrix_new_rejects_invalid_shape() {
        let error = DynamicMatrix::<f64>::new(2, 3, vec![1.0, 2.0]).unwrap_err();

        assert_eq!(error, "Matrix data length does not match dimensions");
    }

    #[test]
    fn test_dynamic_matrix_from_rows_round_trip() {
        let matrix = DynamicMatrix::<f32>::from_rows(vec![vec![1.0, 2.0], vec![3.0, 4.0]]).unwrap();

        assert_eq!(matrix.as_slice(), &[1.0, 2.0, 3.0, 4.0]);
        assert_eq!(matrix.to_vec2d(), vec![vec![1.0, 2.0], vec![3.0, 4.0]]);
    }

    #[test]
    fn test_dynamic_matrix_transpose() {
        let matrix =
            DynamicMatrix::<f64>::from_rows(vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]])
                .unwrap();
        let transposed = matrix.transpose();

        assert_eq!(transposed.shape(), (3, 2));
        assert_eq!(
            transposed.to_vec2d(),
            vec![vec![1.0, 4.0], vec![2.0, 5.0], vec![3.0, 6.0]]
        );
    }

    #[test]
    fn test_dynamic_matrix_mul_vector() {
        let matrix = DynamicMatrix::<f64>::from_rows(vec![vec![1.0, 2.0], vec![3.0, 4.0]]).unwrap();
        let vector = Vector::new(vec![5.0, 6.0]);
        let result = matrix.mul_vector(&vector).unwrap();

        assert_eq!(result.data(), &[17.0, 39.0]);
    }

    #[test]
    fn test_dynamic_matrix_set_updates_value() {
        let mut matrix = DynamicMatrix::<f64>::zeros(2, 2).unwrap();
        matrix.set(1, 0, 7.0);

        assert_eq!(matrix.get(1, 0), 7.0);
    }
}
