//! 移行フェーズ: 数値/最適化アルゴリズムは geo_algorithms クレートへ移動済み。
//! ここでは将来 analysis 固有の定数や軽量補助のみ維持し、主要型を再エクスポートする。

pub use geo_algorithms::{
    ConvergenceInfo,
    NewtonSolver,
    CurveIntersection,
    LeastSquaresFitter,
};

// 例: 分析用の固定定数 (保持要望)
pub const GAUSS_LEGENDRE_4_WEIGHTS: [f64;4] = [0.3478548451, 0.6521451549, 0.6521451549, 0.3478548451];
pub const GAUSS_LEGENDRE_4_POINTS:  [f64;4] = [-0.8611363116, -0.3399810436, 0.3399810436, 0.8611363116];

