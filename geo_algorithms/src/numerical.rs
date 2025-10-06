/// 幾何学的交差解析と最適化アルゴリズム
///
/// 交差検出、近似、最適化問題の解法を提供

use geo_core::ToleranceContext;
use geo_primitives::Point2D;
use crate::sampling::IntersectionCandidate;

/// 2次元ベクトル（analysisのlinalgから独立）
#[derive(Debug, Clone, Copy)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

impl Vector2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn x_axis() -> Self {
        Self { x: 1.0, y: 0.0 }
    }

    pub fn y_axis() -> Self {
        Self { x: 0.0, y: 1.0 }
    }

    pub fn normalize(&self) -> Option<Self> {
        let len = (self.x * self.x + self.y * self.y).sqrt();
        if len > 1e-10 {
            Some(Self { x: self.x / len, y: self.y / len })
        } else {
            None
        }
    }
}

/// 数値解法の収束情報
#[derive(Debug, Clone)]
pub struct ConvergenceInfo {
    pub iterations: usize,
    pub residual: f64,
    pub converged: bool,
    pub final_error: f64,
}

/// Newton-Raphson法による方程式求解
pub struct NewtonSolver {
    tolerance: ToleranceContext,
    max_iterations: usize,
}

impl NewtonSolver {
    pub fn new(tolerance: ToleranceContext) -> Self {
        Self {
            tolerance,
            max_iterations: 100,
        }
    }

    /// 1変数関数の根を求める
    pub fn solve_1d<F, G>(
        &self,
        function: F,
        derivative: G,
        initial_guess: f64,
    ) -> Result<(f64, ConvergenceInfo), String>
    where
        F: Fn(f64) -> f64,
        G: Fn(f64) -> f64,
    {
        let mut x = initial_guess;
        let mut info = ConvergenceInfo {
            iterations: 0,
            residual: f64::INFINITY,
            converged: false,
            final_error: f64::INFINITY,
        };

        for i in 0..self.max_iterations {
            let f_val = function(x);
            let df_val = derivative(x);

            if df_val.abs() < self.tolerance.parametric {
                return Err("Derivative too small, cannot continue".to_string());
            }

            let delta = f_val / df_val;
            x -= delta;

            info.iterations = i + 1;
            info.residual = f_val.abs();
            info.final_error = delta.abs();

            if info.residual < self.tolerance.parametric && info.final_error < self.tolerance.parametric {
                info.converged = true;
                break;
            }
        }

        Ok((x, info))
    }

    /// 2変数関数系の解
    pub fn solve_2d<F>(
        &self,
        system: F,
        initial_guess: (f64, f64),
    ) -> Result<((f64, f64), ConvergenceInfo), String>
    where
        F: Fn(f64, f64) -> (f64, f64, [[f64; 2]; 2]), // (f1, f2, jacobian)
    {
        let mut x = initial_guess.0;
        let mut y = initial_guess.1;
        let mut info = ConvergenceInfo {
            iterations: 0,
            residual: f64::INFINITY,
            converged: false,
            final_error: f64::INFINITY,
        };

        for i in 0..self.max_iterations {
            let (f1, f2, jacobian) = system(x, y);

            // ヤコビ行列の逆行列を計算
            let det = jacobian[0][0] * jacobian[1][1] - jacobian[0][1] * jacobian[1][0];
            if det.abs() < self.tolerance.parametric {
                return Err("Singular Jacobian".to_string());
            }

            let inv_det = 1.0 / det;
            let dx = inv_det * (jacobian[1][1] * f1 - jacobian[0][1] * f2);
            let dy = inv_det * (-jacobian[1][0] * f1 + jacobian[0][0] * f2);

            x -= dx;
            y -= dy;

            info.iterations = i + 1;
            info.residual = (f1 * f1 + f2 * f2).sqrt();
            info.final_error = (dx * dx + dy * dy).sqrt();

            if info.residual < self.tolerance.parametric && info.final_error < self.tolerance.parametric {
                info.converged = true;
                break;
            }
        }

        Ok(((x, y), info))
    }
}

/// 曲線間交差検出
pub struct CurveIntersection {
    tolerance: ToleranceContext,
    newton_solver: NewtonSolver,
}

impl CurveIntersection {
    pub fn new(tolerance: ToleranceContext) -> Self {
        let newton_solver = NewtonSolver::new(tolerance.clone());
        Self {
            tolerance,
            newton_solver,
        }
    }

    /// 2曲線の交差候補を検出
    pub fn find_intersections<F1, F2>(
        &self,
        curve1: F1,
        curve2: F2,
        t1_range: (f64, f64),
        t2_range: (f64, f64),
    ) -> Vec<IntersectionCandidate>
    where
        F1: Fn(f64) -> Point2D + Clone,
        F2: Fn(f64) -> Point2D + Clone,
    {
        let mut candidates = Vec::new();

        // グリッドベースの粗い検索
        let grid_size = 20;
        let step1 = (t1_range.1 - t1_range.0) / grid_size as f64;
        let step2 = (t2_range.1 - t2_range.0) / grid_size as f64;

        for i in 0..grid_size {
            for j in 0..grid_size {
                let t1 = t1_range.0 + i as f64 * step1;
                let t2 = t2_range.0 + j as f64 * step2;

                let p1 = curve1(t1);
                let p2 = curve2(t2);
                let distance = p1.distance_to(&p2).value();

                if distance < self.tolerance.linear * 10.0 {
                    // Newton法で精密化
                    if let Some(refined) = self.refine_intersection(&curve1, &curve2, t1, t2) {
                        candidates.push(refined);
                    }
                }
            }
        }

        // 重複除去
        self.remove_duplicate_candidates(candidates)
    }

    fn refine_intersection<F1, F2>(
        &self,
        curve1: &F1,
        curve2: &F2,
        initial_t1: f64,
        initial_t2: f64,
    ) -> Option<IntersectionCandidate>
    where
        F1: Fn(f64) -> Point2D,
        F2: Fn(f64) -> Point2D,
    {
        // 数値微分による勾配計算
        let h = 1e-8;

        let system = |t1: f64, t2: f64| {
            let p1 = curve1(t1);
            let p2 = curve2(t2);

            let f1 = p1.x().value() - p2.x().value();
            let f2 = p1.y().value() - p2.y().value();

            // ヤコビ行列（数値微分）
            let p1_dt = curve1(t1 + h);
            let p2_dt = curve2(t2 + h);

            let df1_dt1 = (p1_dt.x().value() - p1.x().value()) / h;
            let df1_dt2 = -(p2_dt.x().value() - p2.x().value()) / h;
            let df2_dt1 = (p1_dt.y().value() - p1.y().value()) / h;
            let df2_dt2 = -(p2_dt.y().value() - p2.y().value()) / h;

            let jacobian = [
                [df1_dt1, df1_dt2],
                [df2_dt1, df2_dt2],
            ];

            (f1, f2, jacobian)
        };

        if let Ok(((t1, t2), convergence)) = self.newton_solver.solve_2d(system, (initial_t1, initial_t2)) {
            if convergence.converged {
                let intersection_point = curve1(t1);
                let verification_point = curve2(t2);
                let distance = intersection_point.distance_to(&verification_point).value();

                return Some(IntersectionCandidate {
                    point: intersection_point,
                    parameter: t1,
                    distance,
                    confidence: 1.0 / (1.0 + convergence.final_error),
                });
            }
        }

        None
    }

    fn remove_duplicate_candidates(&self, mut candidates: Vec<IntersectionCandidate>) -> Vec<IntersectionCandidate> {
        candidates.sort_by(|a, b| a.parameter.partial_cmp(&b.parameter).unwrap());

        let mut unique = Vec::new();
        for candidate in candidates {
            let is_duplicate = unique.iter().any(|existing: &IntersectionCandidate| {
                let distance = candidate.point.distance_to(&existing.point).value();
                distance < self.tolerance.linear
            });

            if !is_duplicate {
                unique.push(candidate);
            }
        }

        unique
    }
}

/// 最小二乗フィッティング
///
/// 注意: 座標値と距離はmm単位で処理される
pub struct LeastSquaresFitter {
    tolerance: ToleranceContext,
}

impl LeastSquaresFitter {
    pub fn new(tolerance: ToleranceContext) -> Self {
        Self { tolerance }
    }

    /// 点群に対する円フィッティング
    pub fn fit_circle(&self, points: &[Point2D]) -> Result<(Point2D, f64), String> {
        if points.len() < 3 {
            return Err("Need at least 3 points for circle fitting".to_string());
        }

        // Kasa method implementation for circle fitting
        let n = points.len() as f64;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_x2 = 0.0;
        let mut sum_y2 = 0.0;
        let mut sum_x3 = 0.0;
        let mut sum_y3 = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_x2y = 0.0;
        let mut sum_xy2 = 0.0;

        for point in points {
            let x = point.x().value();
            let y = point.y().value();
            let x2 = x * x;
            let y2 = y * y;

            sum_x += x;
            sum_y += y;
            sum_x2 += x2;
            sum_y2 += y2;
            sum_x3 += x * x2;
            sum_y3 += y * y2;
            sum_xy += x * y;
            sum_x2y += x2 * y;
            sum_xy2 += x * y2;
        }

        // システム行列の構築
        // [sum_x2  sum_xy  sum_x] [a]   [sum_x3 + sum_xy2]
        // [sum_xy  sum_y2  sum_y] [b] = [sum_x2y + sum_y3]
        // [sum_x   sum_y   n    ] [c]   [sum_x2 + sum_y2 ]

        let a11 = sum_x2;
        let a12 = sum_xy;
        let a13 = sum_x;
        let a21 = sum_xy;
        let a22 = sum_y2;
        let a23 = sum_y;
        let a31 = sum_x;
        let a32 = sum_y;
        let a33 = n;

        let b1 = sum_x3 + sum_xy2;
        let b2 = sum_x2y + sum_y3;
        let b3 = sum_x2 + sum_y2;

        // 3x3 連立方程式を解く (Cramerの公式)
        let det = a11 * (a22 * a33 - a23 * a32) - a12 * (a21 * a33 - a23 * a31) + a13 * (a21 * a32 - a22 * a31);

        if det.abs() < self.tolerance.parametric {
            return Err("Degenerate point set - singular matrix".to_string());
        }

        let det_a = b1 * (a22 * a33 - a23 * a32) - a12 * (b2 * a33 - a23 * b3) + a13 * (b2 * a32 - a22 * b3);
        let det_b = a11 * (b2 * a33 - a23 * b3) - b1 * (a21 * a33 - a23 * a31) + a13 * (a21 * b3 - b2 * a31);
        let det_c = a11 * (a22 * b3 - b2 * a32) - a12 * (a21 * b3 - b2 * a31) + b1 * (a21 * a32 - a22 * a31);

        let a = det_a / det;
        let b = det_b / det;
        let c = det_c / det;

        // 円の中心と半径を計算
        let center_x = a / 2.0;
        let center_y = b / 2.0;
        let center = Point2D::from_f64(center_x, center_y);

        let radius = ((a * a + b * b) / 4.0 + c).sqrt();

        Ok((center, radius))
    }

    /// 直線フィッティング
    pub fn fit_line(&self, points: &[Point2D]) -> Result<(Point2D, Vector2), String> {
        if points.len() < 2 {
            return Err("Need at least 2 points for line fitting".to_string());
        }

        let n = points.len() as f64;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;

        for point in points {
            let x = point.x().value();
            let y = point.y().value();
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_x2 += x * x;
        }

        let denom = n * sum_x2 - sum_x * sum_x;
        if denom.abs() < self.tolerance.parametric {
            // 垂直線の場合
            let avg_x = sum_x / n;
            let avg_y = sum_y / n;
            return Ok((
                Point2D::from_f64(avg_x, avg_y),
                Vector2::y_axis()
            ));
        }

        let slope = (n * sum_xy - sum_x * sum_y) / denom;
        let intercept = (sum_y - slope * sum_x) / n;

        let point = Point2D::from_f64(0.0, intercept);
        let direction = Vector2::new(1.0, slope);
        let direction = direction.normalize().unwrap_or(Vector2::x_axis());

        Ok((point, direction))
    }
}