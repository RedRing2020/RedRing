//! NURBS向けサンプルデータ生成。

use geo_contracts::NurbsSurface3DConstructor;

use crate::{
    AdaptiveParamGrid, AdaptiveTessellationSettings, NurbsSurface3D,
    NurbsSurfaceAdaptiveTessellation, Point3D,
};

/// サンプルNURBS曲面と適応テッセレーション用パラメータグリッドを生成する。
pub fn create_sample_nurbs_surface_with_adaptive_params(
    tolerance: f64,
) -> Result<(NurbsSurface3D<f64>, AdaptiveParamGrid<f64>), String> {
    // 中央が盛り上がった2次曲面（3x3制御点グリッド）
    let control_points = vec![
        vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(0.0, 0.5, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        ],
        vec![
            Point3D::new(0.5, 0.0, 0.0),
            Point3D::new(0.5, 0.5, 0.5),
            Point3D::new(0.5, 1.0, 0.0),
        ],
        vec![
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(1.0, 0.5, 0.0),
            Point3D::new(1.0, 1.0, 0.0),
        ],
    ];

    let u_degree = 2;
    let v_degree = 2;
    let u_knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let v_knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];

    let surface =
        NurbsSurface3D::<f64>::new(u_degree, v_degree, u_knots, v_knots, &control_points, None)
            .map_err(|e| e.to_string())?;

    let settings = AdaptiveTessellationSettings::default_with_tolerance(tolerance);
    let param_grid = NurbsSurfaceAdaptiveTessellation::adaptive_params_surface(&surface, &settings);

    Ok((surface, param_grid))
}
