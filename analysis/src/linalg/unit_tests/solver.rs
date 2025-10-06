/// ソルバーテスト
///
/// 各種連立方程式ソルバーの統合テスト

use crate::linalg::solver::{GaussianSolver, LUSolver, CramerSolver, LinearSolver};

/// 共通テストデータ
fn get_test_2x2() -> (Vec<Vec<f64>>, Vec<f64>, Vec<f64>) {
    let matrix = vec![
        vec![2.0, 1.0],
        vec![1.0, 3.0],
    ];
    let rhs = vec![5.0, 6.0];
    let expected = vec![1.8, 1.4];
    (matrix, rhs, expected)
}

fn get_test_3x3() -> (Vec<Vec<f64>>, Vec<f64>, Vec<f64>) {
    let matrix = vec![
        vec![2.0, 1.0, -1.0],
        vec![-3.0, -1.0, 2.0],
        vec![-2.0, 1.0, 2.0],
    ];
    let rhs = vec![8.0, -11.0, -3.0];
    let expected = vec![2.0, 3.0, -1.0];
    (matrix, rhs, expected)
}

#[test]
fn test_gaussian_solver_2x2() {
    let (matrix, rhs, expected) = get_test_2x2();
    let solver = GaussianSolver::new(1e-15);
    let result = solver.solve(&matrix, &rhs).unwrap();

    for (i, &exp) in expected.iter().enumerate() {
        assert!((result.solution[i] - exp).abs() < 1e-10);
    }
    assert!(result.converged);
}

#[test]
fn test_lu_solver_2x2() {
    let (matrix, rhs, expected) = get_test_2x2();
    let solver = LUSolver::new(1e-15);
    let result = solver.solve(&matrix, &rhs).unwrap();

    for (i, &exp) in expected.iter().enumerate() {
        assert!((result.solution[i] - exp).abs() < 1e-10);
    }
    assert!(result.converged);
}

#[test]
fn test_cramer_solver_2x2() {
    let (matrix, rhs, expected) = get_test_2x2();
    let solver = CramerSolver::new(1e-15);
    let result = solver.solve(&matrix, &rhs).unwrap();

    for (i, &exp) in expected.iter().enumerate() {
        assert!((result.solution[i] - exp).abs() < 1e-10);
    }
    assert!(result.converged);
}

#[test]
fn test_solver_comparison_3x3() {
    let (matrix, rhs, expected) = get_test_3x3();

    // 3つのソルバーで同じ問題を解く
    let gaussian_solver = GaussianSolver::new(1e-15);
    let lu_solver = LUSolver::new(1e-15);
    let cramer_solver = CramerSolver::new(1e-15);

    let gaussian_result = gaussian_solver.solve(&matrix, &rhs).unwrap();
    let lu_result = lu_solver.solve(&matrix, &rhs).unwrap();
    let cramer_result = cramer_solver.solve(&matrix, &rhs).unwrap();

    // 全て同じ解を得ることを確認
    for i in 0..3 {
        assert!((gaussian_result.solution[i] - expected[i]).abs() < 1e-10);
        assert!((lu_result.solution[i] - expected[i]).abs() < 1e-10);
        assert!((cramer_result.solution[i] - expected[i]).abs() < 1e-10);

        // ソルバー間での一致も確認
        assert!((gaussian_result.solution[i] - lu_result.solution[i]).abs() < 1e-12);
        assert!((lu_result.solution[i] - cramer_result.solution[i]).abs() < 1e-12);
    }
}

#[test]
fn test_large_system_gaussian_vs_lu() {
    // 4x4システムでガウス消去法とLU分解を比較
    let matrix = vec![
        vec![4.0, 1.0, 2.0, 1.0],
        vec![1.0, 5.0, 1.0, 2.0],
        vec![2.0, 1.0, 6.0, 1.0],
        vec![1.0, 2.0, 1.0, 7.0],
    ];
    let rhs = vec![10.0, 15.0, 20.0, 25.0];

    let gaussian_solver = GaussianSolver::new(1e-15);
    let lu_solver = LUSolver::new(1e-15);

    let gaussian_result = gaussian_solver.solve(&matrix, &rhs).unwrap();
    let lu_result = lu_solver.solve(&matrix, &rhs).unwrap();

    // 両ソルバーが同じ解を得ることを確認
    for i in 0..4 {
        let diff: f64 = gaussian_result.solution[i] - lu_result.solution[i];
        assert!(diff.abs() < 1e-12f64);
    }

    // 解の検証（Ax = b）
    for i in 0..4 {
        let mut sum = 0.0f64;
        for j in 0..4 {
            sum += matrix[i][j] * gaussian_result.solution[j];
        }
        assert!((sum - rhs[i]).abs() < 1e-10f64);
    }
}