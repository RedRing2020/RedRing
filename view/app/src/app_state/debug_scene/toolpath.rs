//! AppState のToolpath/Snapshot系デバッグ表示を扱うモジュール。

use super::super::AppState;
use super::camera_fit::CameraFit;
use logging_foundation::{ERROR_KIND_SIMULATION, ERROR_KIND_VALIDATION};
use render::vertex_3d::{convert_vertex_data_to_mesh_vertices, MeshVertex};
use stage::{MeshStage, OctreeStage};
use viewmodel::cam_sim_visualization_converter::CamSimulationVisualizationError;
use viewmodel::snapshot_converter::{CamSimulationSnapshotInput, DomainSnapshotSeries};

enum ToolPathBuildError {
    Converter(CamSimulationVisualizationError),
    EmptyFrames,
    EmptyVertices,
}

struct ToolPathDebugData {
    tool_entity_id: String,
    snapshot_series: DomainSnapshotSeries<CamSimulationSnapshotInput>,
    snapshot_wireframes: Vec<Vec<viewmodel::octree_converter::WireframeVertex>>,
    snapshot_solids: Vec<(Vec<MeshVertex>, Vec<u32>)>,
    toolpath_lines: Vec<MeshVertex>,
    tool_lines_per_frame: Vec<Vec<MeshVertex>>,
    frame_count: usize,
    fit: CameraFit,
}

impl AppState {
    fn build_toolpath_debug_data(&self) -> Result<ToolPathDebugData, ToolPathBuildError> {
        use viewmodel::cam_sim_visualization_converter::create_sample_cam_simulation_visualization_bundle_with_settings;

        let bundle = create_sample_cam_simulation_visualization_bundle_with_settings(
            &self.octree_visualization_settings,
        )
        .map_err(ToolPathBuildError::Converter)?;

        if bundle.snapshot_wireframes.is_empty() {
            return Err(ToolPathBuildError::EmptyFrames);
        }

        let frame_count = bundle.snapshot_wireframes.len();
        let positions: Vec<[f32; 3]> = bundle
            .snapshot_wireframes
            .last()
            .map(|vertices| vertices.iter().map(|v| v.position).collect())
            .unwrap_or_default();
        let fit = Self::build_camera_fit(&positions).ok_or(ToolPathBuildError::EmptyVertices)?;

        let snapshot_solids = bundle
            .snapshot_solid_meshes
            .iter()
            .map(|(vertex_data, indices)| {
                (
                    convert_vertex_data_to_mesh_vertices(vertex_data),
                    indices.clone(),
                )
            })
            .collect();

        let toolpath_lines = bundle
            .toolpath_wireframe
            .iter()
            .map(|v| MeshVertex::new(v.position, v.color))
            .collect();

        let tool_lines_per_frame = bundle
            .snapshot_tool_wireframes
            .iter()
            .map(|frame| {
                frame
                    .iter()
                    .map(|v| {
                        MeshVertex::new(
                            v.position,
                            self.snapshot_shaded_color_settings.tool_wire_color,
                        )
                    })
                    .collect()
            })
            .collect();

        Ok(ToolPathDebugData {
            tool_entity_id: bundle.tool_entity_id,
            snapshot_series: bundle.snapshot_series,
            snapshot_wireframes: bundle.snapshot_wireframes,
            snapshot_solids,
            toolpath_lines,
            tool_lines_per_frame,
            frame_count,
            fit,
        })
    }

    fn apply_toolpath_debug_data(&mut self, data: ToolPathDebugData) {
        let stage_wireframes = data.snapshot_wireframes.clone();

        self.debug_snapshot.series = Some(data.snapshot_series);
        self.debug_snapshot.wireframes = Some(data.snapshot_wireframes);
        self.debug_snapshot.solids = Some(data.snapshot_solids);
        self.debug_snapshot.toolpath_lines = Some(data.toolpath_lines);
        self.debug_snapshot.tool_lines = Some(data.tool_lines_per_frame);
        self.debug_snapshot.shaded_mode = false;
        self.debug_snapshot.cursor = 0;

        let mut octree_stage = Box::new(OctreeStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        octree_stage.set_depth_levels(&self.graphic.device, stage_wireframes);
        octree_stage.set_depth(&self.graphic.device, 0);

        self.apply_camera_fit(data.fit);

        self.renderer.set_stage(octree_stage);
        self.update_camera_uniforms();
        self.log_current_snapshot_frame(true);

        tracing::info!(
            "CAMシミュレーション可視化デバッグ完了: entity_id={}, frame={}/{}（k/j/左上進捗バーで時系列確認、wでWire/Solid切替）",
            data.tool_entity_id,
            1,
            data.frame_count
        );
    }

    /// サンプル表示用：CAMシミュレーション可視化（ToolPath + ワークOctree + 除去結果）を表示
    pub fn load_sample_toolpath(&mut self) {
        tracing::info!("CAMシミュレーション可視化デバッグ開始（Shift+P）");

        let data = match self.build_toolpath_debug_data() {
            Ok(data) => data,
            Err(ToolPathBuildError::EmptyFrames) => {
                tracing::warn!("CAMシミュレーション可視化フレームが空のため表示をスキップ");
                return;
            }
            Err(ToolPathBuildError::EmptyVertices) => {
                tracing::warn!("CAMシミュレーション可視化頂点が空のため表示をスキップ");
                return;
            }
            Err(ToolPathBuildError::Converter(CamSimulationVisualizationError::Validation(
                ref err,
            ))) => {
                use viewmodel::message_catalog::UiLocale;
                use viewmodel::validation_message_catalog::resolve_validation_error;
                use viewmodel::validation_message_mapper::validation_error_to_ui_message;
                let error_key = validation_error_to_ui_message(err).key;
                let message_en = resolve_validation_error(UiLocale::En, err);
                tracing::error!(
                    error_key = %error_key,
                    error_kind = ERROR_KIND_VALIDATION,
                    message = %message_en,
                    "cam simulation visualization failed"
                );
                return;
            }
            Err(ToolPathBuildError::Converter(CamSimulationVisualizationError::Simulation(
                ref err,
            ))) => {
                tracing::error!(
                    error_kind = ERROR_KIND_SIMULATION,
                    "cam simulation visualization failed: {}",
                    err
                );
                return;
            }
            Err(ToolPathBuildError::Converter(CamSimulationVisualizationError::Application(
                ref err,
            ))) => {
                tracing::error!(
                    error_kind = ERROR_KIND_SIMULATION,
                    "cam simulation visualization failed: {}",
                    err
                );
                return;
            }
        };

        self.apply_toolpath_debug_data(data);
    }

    /// サンプル表示用：カッターパスのみを表示（pキー）
    pub fn load_sample_toolpath_only(&mut self) {
        use viewmodel::toolpath_converter::{
            create_sample_toolpath, toolpath_to_vertices, ToolPathVisualizationSettings,
        };

        tracing::info!("カッターパス表示デバッグ開始（pキー）");

        let toolpath = create_sample_toolpath();
        let settings = ToolPathVisualizationSettings::default();
        let toolpath_vertices = toolpath_to_vertices(&toolpath, &settings);

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
