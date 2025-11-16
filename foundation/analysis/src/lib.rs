//! RedRing Analysis Crate
//!
//! 数値解析、幾何サンプリング、統計解析機能を提供する独立クレート。
//! 他のクレートに依存しない純粋な数値計算機能を提供します。

pub mod abstract_types; // 数値計算の基盤型（Scalar, Angle, Tolerance等）
pub mod consts;
// pub mod curves; - 楕円機能は geo_core に統合されました
pub mod geometry; // 純粋数学的幾何図形
pub mod linalg; // 高速線形代数モジュール（数値解析専用）
                // pub mod numerical_methods; - numerics/solver に移動済み

// 新しい分類構造
pub mod approximations; // 幾何学的近似計算
                        // pub mod metrics; - 距離・計量計算は numerics に統合されました
pub mod numerics; // 数値計算基盤

// pub mod sampling; - 機能は geo_algorithms/sampling.rs に移動済み

#[cfg(test)]
mod unit_tests;

// 基盤型の再エクスポート
pub use abstract_types::{Angle, AngleType, Scalar, TolerantEq};

// 線形代数の再エクスポート
pub use linalg::matrix::{Matrix3x3, Matrix4x4};
pub use linalg::point2::{Coordinates2D, Point2};
pub use linalg::point3::{Coordinates3D, Point3};
pub use linalg::vector::{Vector2, Vector3, Vector4};

// 定数の再エクスポート
pub use consts::{
    game, precision, test_constants, GeometricTolerance, DEG_TO_RAD, DERIVATIVE_ZERO_THRESHOLD, E,
    GEOMETRIC_ANGLE_TOLERANCE, GEOMETRIC_DISTANCE_TOLERANCE, PI, PI_2, PI_3, PI_4, PI_6,
    RAD_TO_DEG, TAU,
};

// 数値計算関数の再エクスポート（numericsモジュールから）
pub use crate::numerics::{newton_arc_length, newton_inverse, newton_solve, NormedVector};
