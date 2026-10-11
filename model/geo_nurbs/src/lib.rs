//! # `geo_nurbs`
//!
//! NURBS曲線・曲面の実装を提供します。
//! `Analysis Vector` を活用した高速な数学的操作に特化した実装です。

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::items_after_test_module)]

// NURBS の形状
mod nurbs_curve_2d;
mod nurbs_curve_3d;
mod nurbs_surface_3d;

// 形状が共用する NURBS の計算
mod adaptive_tessellation;
mod basis;
pub mod constants;
mod error;
mod knot;
mod operations;
mod weight_storage;

pub use analysis::Scalar;
pub use geo_core::TransformError;

pub use adaptive_tessellation::{
    AdaptiveParamGrid, AdaptiveParamList, AdaptiveTessellationSettings,
    NurbsCurveAdaptiveTessellation, NurbsSurfaceAdaptiveTessellation,
};
pub use basis::{
    basis_derivatives, basis_function, basis_functions, rational_basis_derivatives,
    rational_basis_functions,
};
pub use error::{NurbsError, Result};
pub use knot::{
    clamped_knot_vector, find_knot_span, get_parameter_domain, open_knot_vector,
    uniform_knot_vector, validate_knot_vector, KnotVector,
};
pub use nurbs_curve_2d::NurbsCurve2D;
pub use nurbs_curve_3d::{AabbOptions, NurbsCurve3D};
pub use nurbs_surface_3d::NurbsSurface3D;
pub use operations::{
    DegreeElevateResult2D, DegreeElevateResult3D, DegreeElevation, KnotInsertResult2D,
    KnotInsertResult3D, KnotInsertion,
};
pub use weight_storage::WeightStorage;
