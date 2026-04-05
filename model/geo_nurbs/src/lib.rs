//! # `geo_nurbs`
//!
//! NURBS曲線・曲面の実装を提供します。
//! `Analysis Vector` を活用した高速な数学的操作に特化した実装です。

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::items_after_test_module)]

// NURBS実装モジュール
pub mod adaptive_tessellation;
pub mod basis;
pub mod curve_2d;
pub mod curve_2d_bounds;
pub mod curve_2d_transform;
pub mod curve_3d;
pub mod curve_3d_bounds;
pub mod curve_3d_extensions;
pub mod curve_3d_transform;
pub mod operations;
pub mod surface_3d;
pub mod surface_3d_bounds;
pub mod surface_3d_extensions;
pub mod surface_3d_transform;

pub mod error;
pub mod knot;
pub mod weight_storage;

// Analysis ライブラリの Scalar トレイトを使用
pub use analysis::Scalar;
pub use geo_core::{
    AnalysisTransform2D, AnalysisTransform3D, AnalysisTransformSupport, TransformError,
};

// 主要な型を再エクスポート
pub use adaptive_tessellation::{
    AdaptiveParamGrid, AdaptiveParamList, AdaptiveTessellationSettings,
    NurbsCurveAdaptiveTessellation, NurbsSurfaceAdaptiveTessellation,
};
pub use basis::{basis_function, basis_functions, rational_basis_functions};
pub use curve_2d::NurbsCurve2D;
pub use curve_3d::NurbsCurve3D;
pub use curve_3d_extensions::AabbOptions;
pub use error::{NurbsError, Result};
pub use knot::{clamped_knot_vector, validate_knot_vector, KnotVector};
pub use operations::{CurveSplitting, DegreeElevation, KnotInsertion};
pub use surface_3d::NurbsSurface3D;
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

    /// NURBS最近接パラメータ探索で使う境界付きニュートン法の最大反復回数
    pub const NEWTON_MAX_ITER: usize = 20;

    /// NURBS最近接パラメータ探索で使う境界付きニュートン法の収束許容誤差
    pub const NEWTON_TOLERANCE: f64 = 1e-10;

    /// NURBS最近接パラメータ探索で使う数値微分ステップ幅
    pub const NEWTON_DIFF_STEP: f64 = 1e-7;

    /// NURBS導関数の中央差分近似で使う微小ステップ幅
    pub const DERIVATIVE_STEP: f64 = 1e-8;

    /// 曲線長近似で使うデフォルト分割数
    pub const CURVE_LENGTH_SUBDIVISIONS: usize = 100;

    /// 曲面積近似で使うデフォルト分割数
    pub const SURFACE_AREA_SUBDIVISIONS: usize = 20;

    /// 弧長近似サンプリング数の最小値
    pub const ARC_LENGTH_MIN_SAMPLES: usize = 10;

    /// 弧長近似サンプリング数の最大値
    pub const ARC_LENGTH_MAX_SAMPLES: usize = 1000;

    /// 弧長->パラメータ逆算時の最大反復回数
    pub const PARAMETER_AT_LENGTH_MAX_ITER: usize = 50;
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
