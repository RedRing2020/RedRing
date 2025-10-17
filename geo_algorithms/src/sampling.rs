/// 幾何サンプリングとパターン解析
///
/// 2D/3D幾何要素のサンプリング、交差検出、パターン抽出機能

use geo_foundation::ToleranceContext;
// use geo_primitives::Point2D;  // CI/CD compliance: use geo_foundation instead

/// サンプリング結果
#[derive(Debug, Clone)]
pub struct SamplingResult<T> {
    pub points: Vec<T>,
    pub parameter_values: Vec<f64>,
    pub quality_metrics: QualityMetrics,
}

/// 品質指標
#[derive(Debug, Clone)]
pub struct QualityMetrics {
    pub uniformity_score: f64,
    pub coverage_ratio: f64,
    pub density_variance: f64,
}

/// 交差候補
#[derive(Debug, Clone)]
pub struct IntersectionCandidate {
    pub point: Point2D,
    pub parameter: f64,
    pub distance: f64,
    pub confidence: f64,
}

/// 適応的サンプリング
pub struct AdaptiveSampler {
    tolerance: ToleranceContext,
    max_recursion: usize,
    min_samples: usize,
}

impl AdaptiveSampler {
    pub fn new(tolerance: ToleranceContext) -> Self {
        Self {
            tolerance,
            max_recursion: 8,
            min_samples: 10,
        }
    }

    /// 曲率に基づく適応的サンプリング
    pub fn sample_curve_adaptive<F, G>(
        &self,
        evaluator: F,
        curvature_fn: G,
        start: f64,
        end: f64,
    ) -> SamplingResult<Point2D>
    where
        F: Fn(f64) -> Point2D,
        G: Fn(f64) -> f64,
    {
        let mut points = Vec::new();
        let mut parameters = Vec::new();

        self.sample_recursive(&evaluator, &curvature_fn, start, end, 0, &mut points, &mut parameters);

        let quality = self.calculate_quality_metrics(&points, &parameters);

        SamplingResult {
            points,
            parameter_values: parameters,
            quality_metrics: quality,
        }
    }

    fn sample_recursive<F, G>(
        &self,
        evaluator: &F,
        curvature_fn: &G,
        start: f64,
        end: f64,
        depth: usize,
        points: &mut Vec<Point2D>,
        parameters: &mut Vec<f64>,
    ) where
        F: Fn(f64) -> Point2D,
        G: Fn(f64) -> f64,
    {
        // 常に開始点を追加（初回のみ）
        if points.is_empty() {
            points.push(evaluator(start));
            parameters.push(start);
        }

        if depth >= self.max_recursion {
            // 最終点を追加
            points.push(evaluator(end));
            parameters.push(end);
            return;
        }

        let mid = (start + end) * 0.5;
        let curvature = curvature_fn(mid);

        // 高曲率領域では細かくサンプリング
        // 許容誤差を調整：曲率の逆数を使用してより実用的な閾値にする
        let curvature_threshold = 1.0 / self.tolerance.curvature; // 1000.0 相当
        let parametric_threshold = (end - start) > (2.0 * std::f64::consts::PI / 10.0); // パラメータ範囲の1/10

        let should_subdivide = curvature.abs() > curvature_threshold || parametric_threshold;

        if should_subdivide && points.len() < 1000 { // 最大点数制限
            self.sample_recursive(evaluator, curvature_fn, start, mid, depth + 1, points, parameters);
            self.sample_recursive(evaluator, curvature_fn, mid, end, depth + 1, points, parameters);
        } else {
            // 中間点と終点を追加
            points.push(evaluator(mid));
            parameters.push(mid);
            points.push(evaluator(end));
            parameters.push(end);
        }
    }

    fn calculate_quality_metrics(&self, points: &[Point2D], parameters: &[f64]) -> QualityMetrics {
        if points.len() < 2 {
            return QualityMetrics {
                uniformity_score: 0.0,
                coverage_ratio: 0.0,
                density_variance: 0.0,
            };
        }

        // 距離の均一性を計算
        let mut distances = Vec::new();
        for i in 1..points.len() {
            let dist = points[i].distance_to(&points[i-1]).value();
            distances.push(dist);
        }

        let mean_distance: f64 = distances.iter().sum::<f64>() / distances.len() as f64;
        let variance: f64 = distances.iter()
            .map(|d| (d - mean_distance).powi(2))
            .sum::<f64>() / distances.len() as f64;

        let uniformity_score = if variance > 0.0 {
            1.0 / (1.0 + variance.sqrt() / mean_distance)
        } else {
            1.0
        };

        QualityMetrics {
            uniformity_score,
            coverage_ratio: (parameters.len() as f64) / self.min_samples as f64,
            density_variance: variance,
        }
    }
}

/// Monte Carlo サンプリング
///
/// 注意: 座標値と距離はmm単位で処理される
pub struct MonteCarloSampler {
    seed: u64,
}

impl MonteCarloSampler {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// 領域内のランダムサンプリング
    pub fn sample_region_2d(
        &self,
        bounds: (Point2D, Point2D), // min, max
        sample_count: usize,
        filter: impl Fn(&Point2D) -> bool,
    ) -> Vec<Point2D> {
        let mut rng = SimpleRng::new(self.seed);
        let mut results = Vec::new();

        let min_x = bounds.0.x().value();
        let min_y = bounds.0.y().value();
        let max_x = bounds.1.x().value();
        let max_y = bounds.1.y().value();

        let mut attempts = 0;
        while results.len() < sample_count && attempts < sample_count * 10 {
            let x = min_x + rng.next_f64() * (max_x - min_x);
            let y = min_y + rng.next_f64() * (max_y - min_y);
            let point = Point2D::from_f64(x, y);

            if filter(&point) {
                results.push(point);
            }
            attempts += 1;
        }

        results
    }
}

/// シンプルな擬似乱数生成器
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.state
    }

    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Poisson ディスクサンプリング（均等分布）
pub struct PoissonDiskSampler {
    radius: f64,
    k: usize, // 試行回数
}

impl PoissonDiskSampler {
    pub fn new(radius: f64) -> Self {
        Self { radius, k: 30 }
    }

    /// 均等分布サンプリング
    pub fn sample_uniform_2d(
        &self,
        bounds: (Point2D, Point2D),
        initial_seed: Option<Point2D>,
    ) -> Vec<Point2D> {
        let mut rng = SimpleRng::new(42);
        let mut points = Vec::new();
        let mut active_list = Vec::new();

        // 初期点
        let first_point = initial_seed.unwrap_or_else(|| {
            let center_x = (bounds.0.x().value() + bounds.1.x().value()) * 0.5;
            let center_y = (bounds.0.y().value() + bounds.1.y().value()) * 0.5;
            Point2D::from_f64(center_x, center_y)
        });

        points.push(first_point);
        active_list.push(0);

        while !active_list.is_empty() {
            let idx = (rng.next_f64() * active_list.len() as f64) as usize;
            let point_idx = active_list[idx];
            let point = points[point_idx];

            let mut found = false;
            for _ in 0..self.k {
                let angle = rng.next_f64() * 2.0 * std::f64::consts::PI;
                let distance = self.radius * (1.0 + rng.next_f64());

                let new_x = point.x().value() + distance * angle.cos();
                let new_y = point.y().value() + distance * angle.sin();
                let new_point = Point2D::from_f64(new_x, new_y);

                if self.is_valid_point(&new_point, &points, bounds) {
                    points.push(new_point);
                    active_list.push(points.len() - 1);
                    found = true;
                    break;
                }
            }

            if !found {
                active_list.swap_remove(idx);
            }
        }

        points
    }

    fn is_valid_point(
        &self,
        point: &Point2D,
        existing_points: &[Point2D],
        bounds: (Point2D, Point2D),
    ) -> bool {
        // 境界チェック
        if point.x().value() < bounds.0.x().value() || point.x().value() > bounds.1.x().value() ||
           point.y().value() < bounds.0.y().value() || point.y().value() > bounds.1.y().value() {
            return false;
        }

        // 距離チェック
        for existing in existing_points {
            if existing.distance_to(point).value() < self.radius {
                return false;
            }
        }

        true
    }
}
