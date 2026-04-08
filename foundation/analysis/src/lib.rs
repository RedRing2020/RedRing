//! RedRing Analysis Crate
//!
//! 数値解析、幾何サンプリング、統計解析機能を提供する独立クレート。
//! 他のクレートに依存しない純粋な数値計算機能を提供します。

pub mod abstract_types; // 数値計算の基盤型（Scalar, Angle, Tolerance等）
pub mod consts;
pub mod geometry; // 純粋数学的幾何図形
pub mod linalg; // 高速線形代数モジュール（数値解析専用）
pub mod numerics; // 数値計算基盤
pub mod units; // 単位系定義とトレランス管理

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
// Newton solver は generic API を正本とし、f64 ラッパーは downstream 互換用に公開を維持する。
pub use crate::linalg::solver::newton::{
    newton_inverse, newton_inverse_generic, newton_solve, newton_solve_2d, newton_solve_bounded,
    newton_solve_bounded_generic, newton_solve_generic, newton_solve_multivariate,
    newton_solve_multivariate_with_solver, newton_solve_with_numeric_derivative_bounded,
    newton_solve_with_numeric_derivative_bounded_generic,
};
pub use crate::numerics::{
    find_span_in_non_decreasing_sequence, newton_arc_length, trapezoidal_rule, NormedVector,
};

// 単位系の再エクスポート
pub use units::{LengthUnit, Tolerance};
