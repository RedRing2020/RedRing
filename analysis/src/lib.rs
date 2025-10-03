/// RedRing Analysis Crate
///
/// 数値解析、幾何サンプリング、統計解析機能を提供する独立クレート。
/// geo_coreの高精度型を使用して堅牢な数値計算を実現する。

pub mod consts;
// pub mod numeric;     // 一時的に無効化
pub mod sampling;
pub mod sampling2d;  // 既存の2Dサンプリング
pub mod numerical;   // 新しい数値解法モジュール
pub mod statistics;
pub mod interpolation;

#[cfg(test)]
mod unit_tests;

// 主要な型の再エクスポート
pub use geo_core::{Point2D, Point3D, Vector2D, Vector3D, Scalar, ToleranceContext};

// 解析結果の型定義
pub use sampling::{SamplingResult, IntersectionCandidate};
// pub use statistics::{BasicStats, PointCloudStats}; // 実際の型名に修正
// pub use interpolation::{LinearInterpolator, CubicBezier}; // 実際の型名に修正