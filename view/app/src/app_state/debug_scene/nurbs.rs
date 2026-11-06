//! AppState のNURBS系デバッグ表示を扱うモジュール。

use super::super::AppState;
use stage::{NurbsCurveStage, NurbsSurfaceStage};
use std::path::Path;

impl AppState {
    /// デバッグ用：NurbsCurve3Dを表示（SVGから読み込み）
    pub fn load_debug_nurbs(&mut self) {
        let tolerance = self.tolerance_in_current_unit();
        tracing::info!("Debug shape: NurbsCurve3D (GPU evaluation)");
        tracing::info!(
            "Display tolerance: {:.6} (unit: {:?})",
            tolerance,
            self.unit_system
        );

        let svg_path = Path::new("tests/fixtures/shapes/nurbs_curve.svg");
        let eval_data =
            match viewmodel::nurbs_eval_loader::load_nurbs_curve_eval_from_svg(svg_path, tolerance)
            {
                Ok(data) => data,
                Err(error) => {
                    tracing::error!(
                        error_kind = logging_foundation::ERROR_KIND_APP,
                        "nurbs curve eval data generation failed: {}",
                        error
                    );
                    return;
                }
            };

        tracing::info!(
            "Generated GPU evaluation data: params={}, control_points={}, degree={}, knots={}",
            eval_data.num_eval_points(),
            eval_data.num_control_points(),
            eval_data.degree,
            eval_data.knots.len()
        );

        self.camera.reset_to_standard_cad_view();

        let mut nurbs_stage = Box::new(NurbsCurveStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        nurbs_stage.set_eval_data(&self.graphic.device, eval_data);

        self.renderer.set_stage(nurbs_stage);
        self.update_camera_uniforms();
    }

    /// デバッグ用: NURBS曲面をGPU評価で表示（曲率のある曲面）
    pub fn load_debug_nurbs_surface(&mut self) {
        tracing::info!("Debug shape: NurbsSurface3D (GPU evaluation) - curved surface");
        let tolerance_value = self.tolerance_in_current_unit();
        tracing::info!(
            "Display tolerance: {:.6} (unit: {:?})",
            tolerance_value,
            self.unit_system
        );
        let eval_data =
            match viewmodel::nurbs_eval_loader::create_sample_nurbs_surface_eval(tolerance_value) {
                Ok(data) => data,
                Err(error) => {
                    tracing::error!(
                        error_kind = logging_foundation::ERROR_KIND_APP,
                        "nurbs surface eval data generation failed: {}",
                        error
                    );
                    return;
                }
            };

        tracing::info!(
            "Generated GPU evaluation data: vertices={}, triangles={}, u_degree={}, v_degree={}",
            eval_data.num_vertices(),
            eval_data.num_triangles(),
            eval_data.u_degree,
            eval_data.v_degree
        );

        self.camera.reset_to_standard_cad_view();

        let mut nurbs_surface_stage = Box::new(NurbsSurfaceStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        nurbs_surface_stage.set_eval_data(&self.graphic.device, eval_data);

        self.renderer.set_stage(nurbs_surface_stage);
        self.update_camera_uniforms();
    }
}
