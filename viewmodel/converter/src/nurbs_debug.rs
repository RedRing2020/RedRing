//! NURBSデバッグ表示用の評価データ生成
//!
//! app層が直接geo_*に依存しないよう、
//! ViewModel層でNURBS評価データを生成します。

use std::path::Path;

use geo_algorithms::{adaptive_tessellation as ga_tess, NurbsCurve3D, NurbsSurface3D};
use geo_foundation::{NurbsCurve3DConstructor, NurbsSurface3DConstructor};
use geo_io::svg::{parse_svg_file, SvgError};
use thiserror::Error;

use crate::nurbs_view::{NurbsCurveEvalData, NurbsSurfaceEvalData};

/// NURBSデバッグ用のエラー
#[derive(Error, Debug)]
pub enum NurbsDebugError {
    #[error("SVG parsing error: {0}")]
    SvgError(#[from] SvgError),

    #[error("NURBS curve data not found in SVG")]
    MissingNurbsCurve,

    #[error("NURBS construction error: {0}")]
    ConstructionError(String),
}

/// SVGからNURBS曲線のGPU評価データを生成
pub fn load_nurbs_curve_eval_from_svg(
    path: &Path,
    tolerance: f64,
) -> Result<NurbsCurveEvalData, NurbsDebugError> {
    let svg_data = parse_svg_file(path)?;
    let nurbs_data = svg_data
        .nurbs_curves
        .first()
        .ok_or(NurbsDebugError::MissingNurbsCurve)?;

    let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
        nurbs_data.degree,
        nurbs_data.knots.clone(),
        nurbs_data.control_points.clone(),
        nurbs_data.weights.clone(),
    )
    .map_err(NurbsDebugError::ConstructionError)?;

    let settings = ga_tess::AdaptiveTessellationSettings::default_with_tolerance(tolerance);
    let param_list =
        ga_tess::NurbsCurveAdaptiveTessellation::adaptive_params_curve(&curve, &settings);

    let param_list_foundation = ga_tess::AdaptiveParamList {
        params: param_list.params,
    };

    Ok(NurbsCurveEvalData::from_curve_params(
        &curve,
        &param_list_foundation,
    ))
}

/// サンプルNURBS曲面のGPU評価データを生成
pub fn create_sample_nurbs_surface_eval(
    tolerance: f64,
) -> Result<NurbsSurfaceEvalData, NurbsDebugError> {
    // 中央が盛り上がった2次曲面（3x3制御点グリッド）
    let control_points = vec![
        vec![(0.0, 0.0, 0.0), (0.0, 0.5, 0.0), (0.0, 1.0, 0.0)],
        vec![(0.5, 0.0, 0.0), (0.5, 0.5, 0.5), (0.5, 1.0, 0.0)],
        vec![(1.0, 0.0, 0.0), (1.0, 0.5, 0.0), (1.0, 1.0, 0.0)],
    ];

    let u_degree = 2;
    let v_degree = 2;
    let u_knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let v_knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];

    let surface =
        NurbsSurface3D::<f64>::new(control_points, None, u_knots, v_knots, u_degree, v_degree)
            .map_err(NurbsDebugError::ConstructionError)?;

    let settings = ga_tess::AdaptiveTessellationSettings::default_with_tolerance(tolerance);
    let param_grid =
        ga_tess::NurbsSurfaceAdaptiveTessellation::adaptive_params_surface(&surface, &settings);

    let param_grid_foundation = ga_tess::AdaptiveParamGrid {
        u_params: param_grid.u_params,
        v_params: param_grid.v_params,
    };

    Ok(NurbsSurfaceEvalData::from_surface_params(
        &surface,
        &param_grid_foundation,
    ))
}
