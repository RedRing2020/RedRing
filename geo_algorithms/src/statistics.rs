/// 統計処理機能
///
/// 基本統計量の計算、分布解析、回帰分析を提供する

use geo_core::{Scalar, ToleranceContext};
use geo_primitives::Point2D;

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
    /// スカラー値の配列から統計量を計算
    pub fn from_scalars(values: &[Scalar]) -> Self {
        if values.is_empty() {
            return Self {
                mean: 0.0,
                variance: 0.0,
                std_dev: 0.0,
                min: 0.0,
                max: 0.0,
                count: 0,
            };
        }

        let vals: Vec<f64> = values.iter().map(|s| s.value()).collect();
        Self::from_f64_slice(&vals)
    }

    /// f64配列から統計量を計算
    pub fn from_f64_slice(values: &[f64]) -> Self {
        let count = values.len();
        let sum: f64 = values.iter().sum();
        let mean = sum / count as f64;

        let variance = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / count as f64;

        let std_dev = variance.sqrt();
        let min = values.iter().copied().fold(f64::INFINITY, f64::min);
        let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);

        Self {
            mean,
            variance,
            std_dev,
            min,
            max,
            count,
        }
    }
}

/// 2D点群の統計解析
pub struct PointCluster {
    tolerance: ToleranceContext,
}

impl PointCluster {
    pub fn new(tolerance: ToleranceContext) -> Self {
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

    /// 点群の分散（重心からの距離の分散）
    pub fn variance(&self, points: &[Point2D]) -> f64 {
        if let Some(centroid) = self.centroid(points) {
            let distances: Vec<f64> = points.iter()
                .map(|p| centroid.distance_to(p).value())
                .collect();

            BasicStats::from_f64_slice(&distances).variance
        } else {
            0.0
        }
    }

    /// 主成分分析による主軸方向
    pub fn principal_axes(&self, points: &[Point2D]) -> Option<(Point2D, f64)> {
        if points.len() < 2 {
            return None;
        }

        let centroid = self.centroid(points)?;

        // 共分散行列の計算
        let mut cxx = 0.0;
        let mut cyy = 0.0;
        let mut cxy = 0.0;

        for point in points {
            let dx = point.x().value() - centroid.x().value();
            let dy = point.y().value() - centroid.y().value();
            cxx += dx * dx;
            cyy += dy * dy;
            cxy += dx * dy;
        }

        let n = points.len() as f64;
        cxx /= n;
        cyy /= n;
        cxy /= n;

        // 固有値・固有ベクトルの計算
        let trace = cxx + cyy;
        let det = cxx * cyy - cxy * cxy;
        let discriminant = trace * trace - 4.0 * det;

        if discriminant < 0.0 {
            return None;
        }

        let lambda1 = (trace + discriminant.sqrt()) / 2.0;
        let _lambda2 = (trace - discriminant.sqrt()) / 2.0;

        // 第1主成分方向
        let direction = if cxy.abs() > self.tolerance.linear {
            let angle = (2.0 * cxy / (cxx - cyy)).atan() / 2.0;
            Point2D::from_f64(angle.cos(), angle.sin())
        } else if cxx > cyy {
            Point2D::from_f64(1.0, 0.0)
        } else {
            Point2D::from_f64(0.0, 1.0)
        };

        Some((direction, lambda1))
    }
}

/// 回帰分析結果
#[derive(Debug, Clone)]
pub struct RegressionResult {
    pub coefficients: Vec<f64>,
    pub r_squared: f64,
    pub residual_sum_squares: f64,
}