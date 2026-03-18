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
//! - `collision`: 衝突判定・交差判定 (NURBS × Primitives, NURBS × NURBS)
//! - `octree`: 空間分割データ構造 (衝突判定高速化、切削シミュレーション)

pub mod collision;
pub mod octree;

// Point2D API互換性問題により一時的にコメントアウト
// pub mod numerical;
// pub mod statistics;
// pub mod sampling;
// pub mod interpolation;

// 主要な型とトレイトの再エクスポート
// Point2D API互換性問題により一時的にコメントアウト
// pub use numerical::{NewtonSolver, ConvergenceInfo};
// pub use statistics::{BasicStats, PointCluster, RegressionResult};
// pub use sampling::{SamplingResult, QualityMetrics, IntersectionCandidate};
// pub use interpolation::{LinearInterpolator, BezierCurve, CatmullRomSpline};

// Octree関連の公開API
pub use octree::{Octree, OctreeNode, OctreeTolerance};

// geo_foundationからの基本型の再エクスポート

// NURBS型の再エクスポート（ViewModel層からのアクセス用）
pub use geo_contracts::Scalar;
pub use geo_nurbs::adaptive_tessellation;
pub use geo_nurbs::{NurbsCurve3D, NurbsSurface3D};

// geo_primitives の基本型を再エクスポート（ViewModel層がgeo_primitivesに直接依存しないように）
// 基本ポイント・ベクトル型（geo_coreから）
pub use geo_core::{Aabb3D, Point2D, Point3D, Vector2D, Vector3D};
// 基本型（geo_primitivesから）
pub use geo_primitives::{Angle, Direction3D};

// 基本形状
pub use geo_primitives::{
    Arc3D, Circle3D, ConicalSolid3D, ConicalSurface3D, CylindricalSolid3D, CylindricalSurface3D,
    Ellipse3D, EllipseArc3D, EllipsoidalSolid3D, EllipsoidalSurface3D, InfiniteLine3D,
    LineSegment3D, Plane3D, Ray3D, SphericalSolid3D, SphericalSurface3D, TorusSolid3D,
    TorusSurface3D, Triangle3D, TriangleMesh3D,
};

// pub use geo_foundation::geometry2d::Point;  // CI/CD compliance: use geo_foundation instead
// pub use geo_foundation::{Vector2D, Vector3D};  // CI/CD compliance: use geo_foundation instead
// pub use geo_foundation::Point3D;  // CI/CD compliance: use geo_foundation instead
