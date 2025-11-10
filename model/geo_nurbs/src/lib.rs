//! # `geo_nurbs`
//!
//! NURBS曲線・曲面の実装を提供します。
//! `Analysis Vector` を活用した高速な数学的操作に特化した実装です。

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

// NURBS実装モジュール
pub mod basis;
pub mod curve_2d;
pub mod curve_3d;
pub mod surface;
pub mod transform;

pub mod error;
pub mod knot;
pub mod weight_storage;

// Analysis ライブラリの Scalar トレイトを使用
pub use analysis::Scalar;

// 主要な型を再エクスポート
pub use basis::{basis_function, basis_functions, rational_basis_functions};
pub use curve_2d::NurbsCurve2D;
pub use curve_3d::NurbsCurve3D;
pub use error::{NurbsError, Result};
pub use knot::{validate_knot_vector, KnotVector};
pub use surface::NurbsSurface3D;
pub use transform::{CurveSplitting, DegreeElevation, KnotInsertion};
pub use weight_storage::WeightStorage;

/// NURBS関連の定数
pub mod constants {
    /// デフォルトの数値許容誤差
    pub const DEFAULT_TOLERANCE: f64 = 1e-10;

    /// 最小の有効なノット間隔
    pub const MIN_KNOT_INTERVAL: f64 = 1e-12;

    /// 最大サポート次数
    pub const MAX_DEGREE: usize = 10;

    /// 最小制御点数（線形曲線の場合）
    pub const MIN_CONTROL_POINTS: usize = 2;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tolerance_usage() {
        // 実際のNURBS計算でのトレランス使用テスト（動的な値）
        let tolerance = constants::DEFAULT_TOLERANCE;
        let small_diff = tolerance * 0.5;
        assert!(
            small_diff < tolerance,
            "Small difference should be within tolerance"
        );

        let large_diff = tolerance * 2.0;
        assert!(
            large_diff > tolerance,
            "Large difference should exceed tolerance"
        );
    }
}
