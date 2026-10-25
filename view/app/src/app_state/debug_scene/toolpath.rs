//! AppState のツールパスのデバッグ表示を扱うモジュール。

use super::super::AppState;
use render::vertex_3d::MeshVertex;
use stage::MeshStage;

impl AppState {
    /// サンプル表示用：カッターパスのみを表示（pキー）
    pub fn load_sample_toolpath_only(&mut self) {
        use viewmodel::toolpath_converter::{load_demo_toolpath, toolpath_to_vertices};

        tracing::info!("カッターパス表示デバッグ開始（pキー）");

        let toolpath = load_demo_toolpath();
        let toolpath_vertices =
            toolpath_to_vertices(&toolpath, &self.toolpath_visualization_settings);

        let vertices: Vec<MeshVertex> = toolpath_vertices
            .vertices
            .iter()
            .enumerate()
            .map(|(index, vertex)| {
                let color = toolpath_vertices
                    .colors
                    .get(index / 2)
                    .copied()
                    .unwrap_or([1.0, 1.0, 1.0, 1.0]);
                MeshVertex::new(vertex.position, [color[0], color[1], color[2]])
            })
            .collect();

        if vertices.is_empty() {
            tracing::warn!("カッターパス頂点が空のため表示をスキップ");
            return;
        }

        self.debug_snapshot.clear();
        self.cutting_animation = None;
        self.cutting_animation_job_id = None;

        let positions: Vec<[f32; 3]> = vertices.iter().map(|v| v.position).collect();
        let Some(fit) = Self::build_camera_fit(&positions) else {
            tracing::warn!("カッターパス頂点が空のため表示をスキップ");
            return;
        };

        let mut mesh_stage = Box::new(MeshStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        mesh_stage.set_line_data(&self.graphic.device, vertices);

        self.apply_camera_fit(fit);
        self.renderer.set_stage(mesh_stage);
        self.update_camera_uniforms();

        tracing::info!(
            "カッターパス表示デバッグ完了: 線分数={}",
            toolpath_vertices.colors.len()
        );
    }
}
