/// 統計処理機能
///
/// 基本統計量の計算、分布解析、回帰分析を提供する
use geo_contracts::{Scalar, ToleranceSettings};

/// 基本統計量
#[derive(Debug, Clone)]
pub struct BasicStats {
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
}

impl BasicStats {
}

/// 2D点群の統計解析
pub struct PointCluster {
    tolerance: ToleranceSettings<f64>,
}

impl PointCluster {
    pub fn new(tolerance: ToleranceSettings<f64>) -> Self {
        Self { tolerance }
    }

    /// 重心の計算
    pub fn centroid(&self, points: &[Point2D]) -> Option<Point2D> {
        if points.is_empty() {
            return None;
        }

        let sum_x: f64 = points.iter().map(|p| p.x().value()).sum();
        let sum_y: f64 = points.iter().map(|p| p.y().value()).sum();
        let count = points.len() as f64;

        Some(Point2D::from_f64(sum_x / count, sum_y / count))
    }
}

/// 回帰分析結果
#[derive(Debug, Clone)]
pub struct RegressionResult {
    pub coefficients: Vec<f64>,
    pub r_squared: f64,
    pub residual_sum_squares: f64,
}
