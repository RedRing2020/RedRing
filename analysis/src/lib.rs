/// RedRing Analysis Crate
///
/// 数値解析、幾何サンプリング、統計解析機能を提供する独立クレート。
/// geo_coreの高精度型を使用して堅牢な数値計算を実現する。

pub mod consts;
// pub mod numeric;     // 一時的に無効化
pub mod linalg;      // 高速線形代数モジュール（数値解析専用）
pub mod sampling;
pub mod sampling2d;  // 既存の2Dサンプリング
pub mod numerical;   // 新しい数値解法モジュール
pub mod statistics;
pub mod interpolation;
pub mod geometry;    // 幾何特化数値計算

#[cfg(test)]
mod unit_tests;

// 主要な型の再エクスポート
pub use geo_core::{Point2D, Point3D, Vector2D, Vector3D, Scalar, ToleranceContext};

// 定数の再エクスポート
pub use consts::DERIVATIVE_ZERO_THRESHOLD;
pub use geo_core::GEOMETRIC_TOLERANCE;

// 幾何数値関数の再エクスポート
pub use geometry::{
    newton_solve, newton_inverse, newton_arc_length, NormedVector,
    find_span, basis_functions, basis_function_derivatives,
};

// 解析結果の型定義
pub use sampling::{SamplingResult, IntersectionCandidate};
// pub use statistics::{BasicStats, PointCloudStats}; // 実際の型名に修正
// pub use interpolation::{LinearInterpolator, CubicBezier}; // 実際の型名に修正
