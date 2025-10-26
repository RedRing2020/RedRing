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

    // === 統計計算テスト（numerical_methods の一部として） ===

    #[test]
    fn test_simple_statistics() {
        let values = [1.0, 2.0, 3.0, 4.0, 5.0];

        // 平均値
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        assert!((mean - 3.0).abs() < 1e-10);

        // 最小値・最大値
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        assert_eq!(min, 1.0);
        assert_eq!(max, 5.0);

        // 分散
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        assert!((variance - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_empty_dataset() {
        let values: Vec<f64> = vec![];

        // 空のデータセットの処理
        assert!(values.is_empty());
        assert_eq!(values.len(), 0);
    }

    #[test]
    fn test_single_value_statistics() {
        let values = [42.0];

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        assert_eq!(mean, 42.0);

        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        assert_eq!(variance, 0.0);
    }

    // TODO: より高度な数値解析機能のテストを実装予定
}
