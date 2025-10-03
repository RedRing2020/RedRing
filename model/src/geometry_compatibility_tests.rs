/// Phase 1: 互換性検証テスト
/// 
/// SimpleAdaptedLine と既存のmodel::Line の数値的一致性を確認

#[cfg(test)]
mod compatibility_tests {
    use crate::geometry::geometry3d::{point::Point, vector::Vector, line::Line, direction::Direction};
    use crate::geometry_trait::curve3d::Curve3D;
    use crate::geometry_simple_adapter::{SimpleAdaptedLine, simple_factory};

    #[test]
    fn test_line_compatibility_evaluate() {
        // 同じ始点・終点で作成
        let start = Point::new(0.0, 0.0, 0.0);
        let end = Point::new(3.0, 4.0, 0.0);
        
        // 既存のmodel::Line
        let direction = Direction::from_vector(Vector::new(1.0, 0.0, 0.0)).unwrap();
        let model_line = Line::new(start, direction, start, end);
        
        // 新しいSimpleAdaptedLine
        let adapted_line = SimpleAdaptedLine::new(start, end);
        
        // 複数のパラメータでevaluate結果を比較
        let test_params = [0.0, 0.25, 0.5, 0.75, 1.0];
        
        for &t in &test_params {
            let model_point = model_line.evaluate(t);
            let adapted_point = adapted_line.evaluate(t);
            
            println!("t={}: model({:.6}, {:.6}, {:.6}) vs adapted({:.6}, {:.6}, {:.6})", 
                t, model_point.x(), model_point.y(), model_point.z(),
                adapted_point.x(), adapted_point.y(), adapted_point.z());
            
            assert!((model_point.x() - adapted_point.x()).abs() < 1e-10, 
                "X coordinates differ at t={}: {} vs {}", t, model_point.x(), adapted_point.x());
            assert!((model_point.y() - adapted_point.y()).abs() < 1e-10,
                "Y coordinates differ at t={}: {} vs {}", t, model_point.y(), adapted_point.y());
            assert!((model_point.z() - adapted_point.z()).abs() < 1e-10,
                "Z coordinates differ at t={}: {} vs {}", t, model_point.z(), adapted_point.z());
        }
    }

    #[test]
    fn test_line_compatibility_length() {
        let start = Point::new(1.0, 2.0, 3.0);
        let end = Point::new(4.0, 6.0, 3.0);
        
        // 既存のmodel::Line
        let direction = Direction::from_vector(Vector::new(1.0, 0.0, 0.0)).unwrap();
        let model_line = Line::new(start, direction, start, end);
        
        // 新しいSimpleAdaptedLine
        let adapted_line = SimpleAdaptedLine::new(start, end);
        
        let model_length = model_line.length();
        let adapted_length = adapted_line.length();
        
        println!("Length comparison: model={:.10} vs adapted={:.10}", model_length, adapted_length);
        
        assert!((model_length - adapted_length).abs() < 1e-10,
            "Length differs: {} vs {}", model_length, adapted_length);
    }

    #[test]
    fn test_line_compatibility_derivative() {
        let start = Point::new(0.0, 0.0, 0.0);
        let end = Point::new(2.0, 3.0, 6.0);
        
        // 既存のmodel::Line
        let direction = Direction::from_vector(Vector::new(1.0, 0.0, 0.0)).unwrap();
        let model_line = Line::new(start, direction, start, end);
        
        // 新しいSimpleAdaptedLine
        let adapted_line = SimpleAdaptedLine::new(start, end);
        
        // 直線の微分は定ベクトルなので、複数点で確認
        let test_params = [0.0, 0.5, 1.0];
        
        for &t in &test_params {
            let model_derivative = model_line.derivative(t);
            let adapted_derivative = adapted_line.derivative(t);
            
            println!("t={}: model_deriv({:.6}, {:.6}, {:.6}) vs adapted_deriv({:.6}, {:.6}, {:.6})", 
                t, model_derivative.x(), model_derivative.y(), model_derivative.z(),
                adapted_derivative.x(), adapted_derivative.y(), adapted_derivative.z());
            
            assert!((model_derivative.x() - adapted_derivative.x()).abs() < 1e-10,
                "Derivative X differs at t={}: {} vs {}", t, model_derivative.x(), adapted_derivative.x());
            assert!((model_derivative.y() - adapted_derivative.y()).abs() < 1e-10,
                "Derivative Y differs at t={}: {} vs {}", t, model_derivative.y(), adapted_derivative.y());
            assert!((model_derivative.z() - adapted_derivative.z()).abs() < 1e-10,
                "Derivative Z differs at t={}: {} vs {}", t, model_derivative.z(), adapted_derivative.z());
        }
    }

    #[test]
    fn test_line_compatibility_kind() {
        let start = Point::new(0.0, 0.0, 0.0);
        let end = Point::new(1.0, 1.0, 1.0);
        
        // 既存のmodel::Line
        let direction = Direction::from_vector(Vector::new(1.0, 0.0, 0.0)).unwrap();
        let model_line = Line::new(start, direction, start, end);
        
        // 新しいSimpleAdaptedLine
        let adapted_line = SimpleAdaptedLine::new(start, end);
        
        assert_eq!(model_line.kind(), adapted_line.kind(),
            "CurveKind3D should be the same");
    }

    #[test]
    fn test_line_compatibility_domain() {
        let start = Point::new(0.0, 0.0, 0.0);
        let end = Point::new(1.0, 1.0, 1.0);
        
        // 既存のmodel::Line
        let direction = Direction::from_vector(Vector::new(1.0, 0.0, 0.0)).unwrap();
        let model_line = Line::new(start, direction, start, end);
        
        // 新しいSimpleAdaptedLine
        let adapted_line = SimpleAdaptedLine::new(start, end);
        
        assert_eq!(model_line.domain(), adapted_line.domain(),
            "Parameter domain should be the same");
    }

    #[test]
    fn test_factory_compatibility() {
        let start = Point::new(-1.0, -1.0, -1.0);
        let end = Point::new(2.0, 3.0, 4.0);
        
        // ファクトリー関数から作成
        let factory_line: Box<dyn Curve3D> = simple_factory::create_line(start, end);
        
        // 直接作成
        let direct_line = SimpleAdaptedLine::new(start, end);
        
        // 同じ結果を返すか確認
        let test_params = [0.0, 0.3, 0.7, 1.0];
        
        for &t in &test_params {
            let factory_point = factory_line.evaluate(t);
            let direct_point = direct_line.evaluate(t);
            
            assert!((factory_point.x() - direct_point.x()).abs() < 1e-10);
            assert!((factory_point.y() - direct_point.y()).abs() < 1e-10);
            assert!((factory_point.z() - direct_point.z()).abs() < 1e-10);
        }
        
        assert_eq!(factory_line.length(), direct_line.length());
        assert_eq!(factory_line.kind(), direct_line.kind());
    }

    #[test]
    fn test_performance_comparison() {
        use std::time::Instant;
        
        let start = Point::new(0.0, 0.0, 0.0);
        let end = Point::new(100.0, 100.0, 100.0);
        
        // 既存のmodel::Line
        let direction = Direction::from_vector(Vector::new(1.0, 0.0, 0.0)).unwrap();
        let model_line = Line::new(start, direction, start, end);
        
        // 新しいSimpleAdaptedLine
        let adapted_line = SimpleAdaptedLine::new(start, end);
        
        const ITERATIONS: usize = 10000;
        
        // model::Line のパフォーマンス測定
        let start_time = Instant::now();
        for i in 0..ITERATIONS {
            let t = (i as f64) / (ITERATIONS as f64);
            let _ = model_line.evaluate(t);
        }
        let model_duration = start_time.elapsed();
        
        // SimpleAdaptedLine のパフォーマンス測定
        let start_time = Instant::now();
        for i in 0..ITERATIONS {
            let t = (i as f64) / (ITERATIONS as f64);
            let _ = adapted_line.evaluate(t);
        }
        let adapted_duration = start_time.elapsed();
        
        println!("Performance comparison for {} evaluations:", ITERATIONS);
        println!("  model::Line: {:?}", model_duration);
        println!("  SimpleAdaptedLine: {:?}", adapted_duration);
        
        // パフォーマンスは情報として出力するのみ（テスト失敗にはしない）
        let ratio = adapted_duration.as_nanos() as f64 / model_duration.as_nanos() as f64;
        println!("  Performance ratio (adapted/model): {:.2}x", ratio);
        
        // 極端に遅くないことを確認（10倍以上遅くない）
        assert!(ratio < 10.0, "SimpleAdaptedLine is more than 10x slower than model::Line");
    }
}