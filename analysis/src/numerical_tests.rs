// API不一致のため一時的にコメントアウト
// use geo_algorithms::numerical::*;
// 注: geo_core依存関係は除外済み

#[cfg(test)]
mod tests {
    use crate::numerical_methods::newton_solve;

    #[test]
    fn test_newton_solver() {
        // x^2 - 4 = 0 を解く（期待値: x = 2）
        let f = |x: f64| x * x - 4.0;
        let df = |x: f64| 2.0 * x;

        let result = newton_solve(f, df, 1.0, 100, 1e-10);
        assert!(result.is_some());

        if let Some(root) = result {
            assert!((root - 2.0).abs() < 1e-6);
        }
    }

    #[test]
    fn test_newton_solver_cubic() {
        // x^3 - 8 = 0 を解く（期待値: x = 2）
        let f = |x: f64| x * x * x - 8.0;
        let df = |x: f64| 3.0 * x * x;

        let result = newton_solve(f, df, 1.5, 100, 1e-10);
        assert!(result.is_some());

        if let Some(root) = result {
            assert!((root - 2.0).abs() < 1e-6);
        }
    }

    #[test]
    fn test_newton_solver_no_convergence() {
        // 収束しない例（導関数がゼロになる場合）
        let f = |x: f64| x * x;
        let df = |_x: f64| 0.0; // 常にゼロ

        let result = newton_solve(f, df, 1.0, 10, 1e-10);
        assert!(result.is_none()); // 収束しないことを確認
    }

    // プレースホルダーテスト（将来的に数値解析機能のテストを追加）
    #[test]
    fn test_placeholder() {
        assert!(true);
    }
}
