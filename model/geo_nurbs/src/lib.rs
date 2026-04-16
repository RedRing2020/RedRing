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
    use core::any::TypeId;

    use crate::Scalar;

    #[inline]
    fn select_scalar_value<T: Scalar>(f32_value: f32, f64_value: f64) -> T {
        if TypeId::of::<T>() == TypeId::of::<f32>() {
            T::from_f32(f32_value)
        } else {
            // f32以外はf64側へフォールバックする（geo_contractsの既定値選択と整合）。
            T::from_f64(f64_value)
        }
    }

    /// 幾何判定で使うしきい値群
    pub mod tolerance {
        use crate::Scalar;

        use super::select_scalar_value;

        /// デフォルトの数値許容誤差（f32）
        pub const DEFAULT_TOLERANCE_F32: f32 = 1e-6;

        /// デフォルトの数値許容誤差（f64）
        pub const DEFAULT_TOLERANCE_F64: f64 = 1e-10;

        /// 最小の有効なノット間隔（f32）
        pub const MIN_KNOT_INTERVAL_F32: f32 = 1e-6;

        /// 最小の有効なノット間隔（f64）
        pub const MIN_KNOT_INTERVAL_F64: f64 = 1e-12;

        /// デフォルトの数値許容誤差を型別に取得
        #[must_use]
        pub fn default_tolerance<T: Scalar>() -> T {
            select_scalar_value::<T>(DEFAULT_TOLERANCE_F32, DEFAULT_TOLERANCE_F64)
        }

        /// 最小の有効なノット間隔を型別に取得
        #[must_use]
        pub fn min_knot_interval<T: Scalar>() -> T {
            select_scalar_value::<T>(MIN_KNOT_INTERVAL_F32, MIN_KNOT_INTERVAL_F64)
        }
    }

    /// 数値解法で使うしきい値群
    pub mod solver {
        use crate::Scalar;

        use super::select_scalar_value;

        /// 境界付きニュートン法の収束許容誤差（f32）
        pub const NEWTON_TOLERANCE_F32: f32 = 1e-6;

        /// 境界付きニュートン法の収束許容誤差（f64）
        pub const NEWTON_TOLERANCE_F64: f64 = 1e-10;

        /// 境界付きニュートン法の数値微分ステップ幅（f32）
        pub const NEWTON_DIFF_STEP_F32: f32 = 1e-4;

        /// 境界付きニュートン法の数値微分ステップ幅（f64）
        pub const NEWTON_DIFF_STEP_F64: f64 = 1e-7;

        /// 導関数の中央差分近似で使う微小ステップ幅（f32）
        pub const DERIVATIVE_STEP_F32: f32 = 1e-4;

        /// 導関数の中央差分近似で使う微小ステップ幅（f64）
        pub const DERIVATIVE_STEP_F64: f64 = 1e-8;

        /// 境界付きニュートン法の収束許容誤差を型別に取得
        #[must_use]
        pub fn newton_tolerance<T: Scalar>() -> T {
            select_scalar_value::<T>(NEWTON_TOLERANCE_F32, NEWTON_TOLERANCE_F64)
        }

        /// 境界付きニュートン法の数値微分ステップ幅を型別に取得
        #[must_use]
        pub fn newton_diff_step<T: Scalar>() -> T {
            select_scalar_value::<T>(NEWTON_DIFF_STEP_F32, NEWTON_DIFF_STEP_F64)
        }

        /// 導関数の中央差分近似で使う微小ステップ幅を型別に取得
        #[must_use]
        pub fn derivative_step<T: Scalar>() -> T {
            select_scalar_value::<T>(DERIVATIVE_STEP_F32, DERIVATIVE_STEP_F64)
        }
    }

    /// デフォルトの数値許容誤差（後方互換・f64）
    pub const DEFAULT_TOLERANCE: f64 = tolerance::DEFAULT_TOLERANCE_F64;

    /// 最小の有効なノット間隔（後方互換・f64）
    pub const MIN_KNOT_INTERVAL: f64 = tolerance::MIN_KNOT_INTERVAL_F64;

    /// 最大サポート次数
    pub const MAX_DEGREE: usize = 10;

    /// 最小制御点数（線形曲線の場合）
    pub const MIN_CONTROL_POINTS: usize = 2;

    /// NURBS最近接パラメータ探索で使う境界付きニュートン法の最大反復回数
    pub const NEWTON_MAX_ITER: usize = 20;

    /// NURBS最近接パラメータ探索で使う境界付きニュートン法の収束許容誤差（後方互換・f64）
    pub const NEWTON_TOLERANCE: f64 = solver::NEWTON_TOLERANCE_F64;

    /// NURBS最近接パラメータ探索で使う数値微分ステップ幅（後方互換・f64）
    pub const NEWTON_DIFF_STEP: f64 = solver::NEWTON_DIFF_STEP_F64;

    /// NURBS導関数の中央差分近似で使う微小ステップ幅（後方互換・f64）
    pub const DERIVATIVE_STEP: f64 = solver::DERIVATIVE_STEP_F64;

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

    #[test]
    fn test_typed_constant_accessors_return_expected_values() {
        assert_eq!(
            constants::tolerance::default_tolerance::<f32>().to_bits(),
            constants::tolerance::DEFAULT_TOLERANCE_F32.to_bits()
        );
        assert_eq!(
            constants::tolerance::default_tolerance::<f64>().to_bits(),
            constants::tolerance::DEFAULT_TOLERANCE_F64.to_bits()
        );

        assert_eq!(
            constants::tolerance::min_knot_interval::<f32>().to_bits(),
            constants::tolerance::MIN_KNOT_INTERVAL_F32.to_bits()
        );
        assert_eq!(
            constants::tolerance::min_knot_interval::<f64>().to_bits(),
            constants::tolerance::MIN_KNOT_INTERVAL_F64.to_bits()
        );

        assert_eq!(
            constants::solver::newton_tolerance::<f32>().to_bits(),
            constants::solver::NEWTON_TOLERANCE_F32.to_bits()
        );
        assert_eq!(
            constants::solver::newton_tolerance::<f64>().to_bits(),
            constants::solver::NEWTON_TOLERANCE_F64.to_bits()
        );

        assert_eq!(
            constants::solver::newton_diff_step::<f32>().to_bits(),
            constants::solver::NEWTON_DIFF_STEP_F32.to_bits()
        );
        assert_eq!(
            constants::solver::newton_diff_step::<f64>().to_bits(),
            constants::solver::NEWTON_DIFF_STEP_F64.to_bits()
        );

        assert_eq!(
            constants::solver::derivative_step::<f32>().to_bits(),
            constants::solver::DERIVATIVE_STEP_F32.to_bits()
        );
        assert_eq!(
            constants::solver::derivative_step::<f64>().to_bits(),
            constants::solver::DERIVATIVE_STEP_F64.to_bits()
        );
    }
}
