//! RedRing Analysis Crate
//!
//! 数値解析、幾何サンプリング、統計解析機能を提供する独立クレート。
//! geo_coreの高精度型を使用して堅牢な数値計算を実現する。
pub mod consts;
// pub mod numeric;     // 一時的に無効化
pub mod linalg;      // 高速線形代数モジュール（数値解析専用）
pub mod sampling2d;  // 既存の2Dサンプリング（後で整理）
pub mod geometry;    // 幾何特化数値計算（後で整理）

#[cfg(test)]
mod unit_tests;

// geo_algorithmsからの再エクスポート (一時的にコメントアウト)
pub use geo_algorithms::{
    Point2D, Vector2D, Vector3D, Scalar, ToleranceContext,
    // Point3D, GEOMETRIC_TOLERANCE,  // 一時的にコメントアウト
    // NewtonSolver, ConvergenceInfo,
    // BasicStats, PointCluster, RegressionResult,
    // SamplingResult, QualityMetrics, IntersectionCandidate,
    // LinearInterpolator, BezierCurve, CatmullRomSpline,
};

// 定数の再エクスポート
pub use consts::DERIVATIVE_ZERO_THRESHOLD;

// 幾何数値関数の再エクスポート
pub use geometry::{
    newton_solve, newton_inverse, newton_arc_length, NormedVector,
    find_span, basis_functions, basis_function_derivatives,
};
