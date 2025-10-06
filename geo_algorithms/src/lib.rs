//! RedRing 幾何アルゴリズム (Geometric Algorithms)
//!
//! 幾何プリミティブを使用した高レベルアルゴリズムを提供します。
//! - 数値解析アルゴリズム (Newton法、数値積分など)
//! - 統計解析 (基本統計量、分布解析、回帰分析)
//! - サンプリング (適応サンプリング、ポアソンディスクサンプリング)
//! - 補間・近似 (線形補間、スプライン、ベジエ曲線)
//!
//! ## アーキテクチャ設計方針
//!
//! ```text
//! +------------------------+  アプリケーション層
//! |       analysis         |  純粋な数値計算 (線形代数、クォータニオンなど)
//! +------------------------+
//!             |
//!             v
//! +------------------------+  幾何アルゴリズム層
//! |    geo_algorithms      |  幾何プリミティブを活用したアルゴリズム
//! +------------+-----------+
//!              |
//!              v
//! +------------+-----------+  幾何プリミティブ層
//! | geo_primitives + geo_core |  基本幾何要素と許容誤差管理
//! +------------------------+
//! ```
//!
//! ## モジュール構成
//!
//! - `numerical`: 数値解析アルゴリズム (Newton法、最適化、数値積分)
//! - `statistics`: 統計解析 (基本統計量、回帰分析、主成分分析)
//! - `sampling`: サンプリング手法 (適応サンプリング、パターン解析)
//! - `interpolation`: 補間・近似 (スプライン、ベジエ、NURBS基盤)

pub mod numerical;
pub mod statistics; 
pub mod sampling;
pub mod interpolation;

// 主要な型とトレイトの再エクスポート
pub use numerical::{NewtonSolver, ConvergenceInfo};
pub use statistics::{BasicStats, PointCluster, RegressionResult};
pub use sampling::{SamplingResult, QualityMetrics, IntersectionCandidate};
pub use interpolation::{LinearInterpolator, BezierCurve, CatmullRomSpline};

// geo_coreからの基本型の再エクスポート
pub use geo_core::{Point2D, Point3D, Vector2D, Vector3D, Scalar, ToleranceContext, GEOMETRIC_TOLERANCE};