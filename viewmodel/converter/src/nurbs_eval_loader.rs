//! NURBS評価データの読込・生成
//!
//! app層が直接geo_*に依存しないよう、
//! ViewModel層でNURBS評価データを生成します。

use std::path::Path;

use geo_algorithms::{adaptive_tessellation as ga_tess, NurbsCurve3D};
use geo_contracts::NurbsCurve3DConstructor;
use geo_io::svg::{parse_svg_file, SvgError};
use thiserror::Error;

use crate::nurbs_view::{NurbsCurveEvalData, NurbsSurfaceEvalData};

/// NURBS評価データ読込/生成用のエラー
#[derive(Error, Debug)]
pub enum NurbsEvalLoaderError {
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
) -> Result<NurbsCurveEvalData, NurbsEvalLoaderError> {
    let svg_data = parse_svg_file(path)?;
    let nurbs_data = svg_data
        .nurbs_curves
        .first()
        .ok_or(NurbsEvalLoaderError::MissingNurbsCurve)?;

    let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
        nurbs_data.degree,
        nurbs_data.knots.clone(),
        nurbs_data.control_points.clone(),
        nurbs_data.weights.clone(),
    )
    .map_err(NurbsEvalLoaderError::ConstructionError)?;

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
) -> Result<NurbsSurfaceEvalData, NurbsEvalLoaderError> {
    let (surface, param_grid) =
        geo_algorithms::nurbs_fixtures::create_sample_nurbs_surface_with_adaptive_params(tolerance)
            .map_err(NurbsEvalLoaderError::ConstructionError)?;

    let param_grid_foundation = ga_tess::AdaptiveParamGrid {
        u_params: param_grid.u_params,
        v_params: param_grid.v_params,
    };

    Ok(NurbsSurfaceEvalData::from_surface_params(
        &surface,
        &param_grid_foundation,
    ))
}
