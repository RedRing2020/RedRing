//! AppState のNURBS系デバッグ表示を扱うモジュール。

use super::AppState;
use stage::{NurbsCurveStage, NurbsSurfaceStage};
use std::path::Path;

impl AppState {
    /// デバッグ用：NurbsCurve3Dを表示（SVGから読み込み）
    /// ViewModelレイヤー経由で評価データを生成
    pub fn load_debug_nurbs(&mut self) {
        let tolerance = self.tolerance_in_current_unit();
        tracing::info!("デバッグ形状: NurbsCurve3D表示（GPU評価）");
        tracing::info!(
            "表示トレランス: {:.6} (単位系: {:?})",
            tolerance,
            self.unit_system
        );

        let svg_path = Path::new("tests/fixtures/shapes/nurbs_curve.svg");
        let eval_data =
            match viewmodel::nurbs_debug::load_nurbs_curve_eval_from_svg(svg_path, tolerance) {
                Ok(data) => data,
                Err(error) => {
                    tracing::error!("NURBS曲線データ生成失敗: {}", error);
                    return;
                }
            };

        tracing::info!(
            "GPU評価データ生成完了: params={}, control_points={}, degree={}, knots={}",
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
    /// ViewModelレイヤー経由で評価データを生成
    pub fn load_debug_nurbs_surface(&mut self) {
        tracing::info!("デバッグ形状: NurbsSurface3D表示（GPU評価）- 曲率のある曲面");
        let tolerance_value = self.tolerance_in_current_unit();
        tracing::info!(
            "表示トレランス: {:.6} (単位系: {:?})",
            tolerance_value,
            self.unit_system
        );
        let eval_data =
            match viewmodel::nurbs_debug::create_sample_nurbs_surface_eval(tolerance_value) {
                Ok(data) => data,
                Err(error) => {
                    tracing::error!("NURBS曲面データ生成失敗: {}", error);
                    return;
                }
            };

        tracing::info!(
            "GPU評価データ生成完了: vertices={}, triangles={}, u_degree={}, v_degree={}",
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
