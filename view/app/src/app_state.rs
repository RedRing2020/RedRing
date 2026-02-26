use crate::app_renderer::AppRenderer;
use crate::entity_manager::EntityManager;
use crate::graphic::{init_graphic, Graphic};
use crate::mouse_input::MouseInput;
use crate::snapshot_overlay_renderer::SnapshotOverlayStyle;
use crate::stl_loader;
use crate::view_rect::ViewRect;
use analysis::linalg::{quaternion::Quaternionf, vector::Vec3f};
use analysis::{LengthUnit, Tolerance};
use render::vertex_3d::{convert_vertex_data_to_mesh_vertices, MeshVertex};
use stage::{
    DraftStage, MeshStage, NurbsCurveStage, NurbsSurfaceStage, OctreeStage, OutlineStage,
    ShadingStage,
};
use std::path::Path;
use std::sync::Arc;
use viewmodel::octree_converter::{OctreeDebugVisualizationSettings, WireframeVertex};
use viewmodel::snapshot_converter::{CamSimulationSnapshotInput, DomainSnapshotSeries};
use viewmodel_graphics::{Camera, CameraControlSensitivity};
use winit::window::Window;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ViewingOperationSettings {
    pub camera_control_sensitivity: CameraControlSensitivity,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnapshotShadedColorSettings {
    pub work_solid_color: [f32; 4],
    pub tool_wire_color: [f32; 3],
}

impl Default for SnapshotShadedColorSettings {
    fn default() -> Self {
        Self {
            work_solid_color: [0.95, 0.55, 0.25, 1.0],
            tool_wire_color: [0.9, 0.95, 1.0],
        }
    }
}

pub struct AppState {
    pub window: Arc<Window>,
    pub graphic: Graphic,
    pub renderer: AppRenderer,
    pub camera: Camera,
    pub mouse_input: MouseInput,
    pub entity_manager: EntityManager,
    pub active_view_rect: Option<ViewRect>,
    pub last_view_rect: Option<ViewRect>,

    /// アプリケーション単位系（CAD標準: ミリメートル）
    ///
    /// 全ての幾何データはこの単位で解釈されます。
    /// デフォルト: ミリメートル（浮動小数点誤差を最小化）
    pub unit_system: LengthUnit,

    /// 表示トレランス（CAD標準: 0.01mm）
    ///
    /// 曲線のテッセレーション（分割）や近似計算で使用される許容誤差。
    /// この値により、曲線から生成される線分の精度が決まります。
    pub display_tolerance: Tolerance,

    /// Octree可視化設定（setting画面向けの保持値）
    pub octree_visualization_settings: OctreeDebugVisualizationSettings,

    /// ビュー操作設定（setting画面向けの保持値）
    pub viewing_operation_settings: ViewingOperationSettings,

    /// Snapshot進捗バー表示設定
    pub snapshot_overlay_style: SnapshotOverlayStyle,

    /// Snapshotシェーディング時の色設定
    pub snapshot_shaded_color_settings: SnapshotShadedColorSettings,

    /// デバッグ用: シミュレーションスナップショット系列
    debug_snapshot_series: Option<DomainSnapshotSeries<CamSimulationSnapshotInput>>,
    debug_snapshot_wireframes: Option<Vec<Vec<WireframeVertex>>>,
    debug_snapshot_solids: Option<Vec<(Vec<MeshVertex>, Vec<u32>)>>,
    debug_snapshot_toolpath_lines: Option<Vec<MeshVertex>>,
    debug_snapshot_tool_lines: Option<Vec<Vec<MeshVertex>>>,
    debug_snapshot_shaded_mode: bool,
    debug_snapshot_cursor: usize,
    snapshot_scrub_active: bool,

    cursor_position: Option<(f32, f32)>,
    last_cursor_position: Option<(f32, f32)>,
    arcball_drag_start: Option<(f32, f32)>,
    arcball_virtual_cursor: Option<(f32, f32)>,
    view_rect_drag_origin: Option<(f32, f32)>,
}

impl AppState {
    fn apply_viewing_operation_settings(&mut self) {
        self.camera
            .set_control_sensitivity(self.viewing_operation_settings.camera_control_sensitivity);
    }

    pub fn new(window: Arc<Window>) -> Self {
        let graphic = init_graphic(window.clone());
        let renderer = AppRenderer::new_draft(&graphic.device, &graphic.config);
        let viewing_operation_settings = ViewingOperationSettings::default();

        let mut app_state = Self {
            window,
            graphic,
            renderer,
            camera: Camera::new(),
            mouse_input: MouseInput::new(),
            entity_manager: EntityManager::new(),
            active_view_rect: None,
            last_view_rect: None,
            // CAD標準設定
            unit_system: LengthUnit::Millimeter,
            display_tolerance: Tolerance::default(), // 0.01mm
            octree_visualization_settings: OctreeDebugVisualizationSettings::default(),
            viewing_operation_settings,
            snapshot_overlay_style: SnapshotOverlayStyle::default(),
            snapshot_shaded_color_settings: SnapshotShadedColorSettings::default(),
            debug_snapshot_series: None,
            debug_snapshot_wireframes: None,
            debug_snapshot_solids: None,
            debug_snapshot_toolpath_lines: None,
            debug_snapshot_tool_lines: None,
            debug_snapshot_shaded_mode: false,
            debug_snapshot_cursor: 0,
            snapshot_scrub_active: false,
            cursor_position: None,
            last_cursor_position: None,
            arcball_drag_start: None,
            arcball_virtual_cursor: None,
            view_rect_drag_origin: None,
        };

        app_state.apply_viewing_operation_settings();
        app_state
    }

    /// デバッグ用: cam_sim 実行結果をスナップショット系列として読み込む
    pub fn load_debug_simulation_snapshots(&mut self) {
        match viewmodel::snapshot_converter::create_sample_cam_snapshot_domain_series() {
            Ok(series) => {
                let frame_count = series.frames.len();
                self.debug_snapshot_series = Some(series);
                self.debug_snapshot_wireframes = None;
                self.debug_snapshot_solids = None;
                self.debug_snapshot_toolpath_lines = None;
                self.debug_snapshot_tool_lines = None;
                self.debug_snapshot_shaded_mode = false;
                self.debug_snapshot_cursor = 0;
                tracing::info!("シミュレーションスナップショット読込完了: {} フレーム", frame_count);
                self.log_current_snapshot_frame(true);
            }
            Err(error) => {
                tracing::error!("シミュレーションスナップショット読込失敗: {}", error);
            }
        }
    }

    /// デバッグ用: 次フレームへ進めて内容をログ表示
    pub fn cycle_debug_simulation_snapshot(&mut self) {
        let Some(series) = &self.debug_snapshot_series else {
            self.load_debug_simulation_snapshots();
            return;
        };

        if series.frames.is_empty() {
            tracing::warn!("スナップショット系列が空です");
            return;
        }

        self.debug_snapshot_cursor = (self.debug_snapshot_cursor + 1) % series.frames.len();
        self.log_current_snapshot_frame(true);
    }

    /// デバッグ用: 前フレームへ戻して内容をログ表示
    pub fn rewind_debug_simulation_snapshot(&mut self) {
        let Some(series) = &self.debug_snapshot_series else {
            self.load_debug_simulation_snapshots();
            return;
        };

        if series.frames.is_empty() {
            tracing::warn!("スナップショット系列が空です");
            return;
        }

        self.debug_snapshot_cursor = if self.debug_snapshot_cursor == 0 {
            series.frames.len() - 1
        } else {
            self.debug_snapshot_cursor - 1
        };
        self.log_current_snapshot_frame(true);
    }

    fn sync_snapshot_visual_frame(&mut self) {
        if self.debug_snapshot_shaded_mode {
            let Some(snapshot_solids) = &self.debug_snapshot_solids else {
                return;
            };
            if snapshot_solids.is_empty() {
                return;
            }

            let frame_index = self
                .debug_snapshot_cursor
                .min(snapshot_solids.len().saturating_sub(1));
            let (vertices, indices) = snapshot_solids[frame_index].clone();

            let stage = self.renderer.get_stage_mut();
            let Some(mesh_stage) = stage.as_any_mut().downcast_mut::<MeshStage>() else {
                return;
            };
            mesh_stage.set_mesh_data(&self.graphic.device, vertices, indices);
            mesh_stage.set_mesh_base_color(self.snapshot_shaded_color_settings.work_solid_color);
            mesh_stage.clear_overlay_line_data();

            let toolpath_lines = self.debug_snapshot_toolpath_lines.clone().unwrap_or_default();
            if let Some(tool_lines_per_frame) = &self.debug_snapshot_tool_lines {
                let overlay_index = frame_index.min(tool_lines_per_frame.len().saturating_sub(1));
                let tool_lines = tool_lines_per_frame[overlay_index].clone();
                if !tool_lines.is_empty() {
                    mesh_stage.set_overlay_tool_line_data(&self.graphic.device, tool_lines);
                }
            }
            if !toolpath_lines.is_empty() {
                mesh_stage.set_overlay_toolpath_line_data(&self.graphic.device, toolpath_lines);
            }
            return;
        }

        let Some(snapshot_wireframes) = &self.debug_snapshot_wireframes else {
            return;
        };
        if snapshot_wireframes.is_empty() {
            return;
        }

        let frame_index = self
            .debug_snapshot_cursor
            .min(snapshot_wireframes.len().saturating_sub(1));

        let stage = self.renderer.get_stage_mut();
        let Some(octree_stage) = stage.as_any_mut().downcast_mut::<OctreeStage>() else {
            return;
        };

        if octree_stage.max_depth().saturating_add(1) != snapshot_wireframes.len() {
            octree_stage.set_depth_levels(&self.graphic.device, snapshot_wireframes.clone());
        }
        octree_stage.set_depth(&self.graphic.device, frame_index);
    }

    fn log_current_snapshot_frame(&mut self, emit_log: bool) {
        let Some(series) = &self.debug_snapshot_series else {
            return;
        };
        if series.frames.is_empty() {
            return;
        }

        let index = self.debug_snapshot_cursor.min(series.frames.len() - 1);
        let frame = &series.frames[index];
        let payload = frame.payload;

        if emit_log {
            tracing::info!(
                "Snapshot frame {}/{}: seg={}, t={:.3}, dist={:.3}mm, remain={:.3}mm3",
                index + 1,
                series.frames.len(),
                payload.segment_index,
                payload.segment_t,
                payload.accumulated_distance_mm,
                payload.remaining_volume_mm3,
            );
        }

        self.window.set_title(&format!(
            "RedRing | Snapshot {}/{} | seg={} t={:.3} dist={:.3}mm remain={:.3}mm3",
            index + 1,
            series.frames.len(),
            payload.segment_index,
            payload.segment_t,
            payload.accumulated_distance_mm,
            payload.remaining_volume_mm3,
        ));

        self.sync_snapshot_visual_frame();
    }

    fn rebuild_stage_from_entities(&mut self) {
        if !self.entity_manager.is_dirty() {
            return;
        }

        let vertices = self.entity_manager.line_vertices();
        if vertices.is_empty() {
            self.entity_manager.clear_dirty();
            return;
        }

        let mut mesh_stage = Box::new(MeshStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        mesh_stage.set_line_data(&self.graphic.device, vertices);

        self.renderer.set_stage(mesh_stage);
        self.update_camera_uniforms();
        self.entity_manager.clear_dirty();
    }

    /// 現在の単位でのトレランス値を取得
    ///
    /// # Examples
    ///
    /// 単位系がミリメートル、トレランスが0.01mmの場合 → 0.01
    /// 単位系がメートル、トレランスが0.01mmの場合 → 0.00001
    pub fn tolerance_in_current_unit(&self) -> f64 {
        self.display_tolerance.in_unit(self.unit_system)
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.graphic.config.width = size.width;
        self.graphic.config.height = size.height;
        self.graphic
            .surface
            .configure(&self.graphic.device, &self.graphic.config);

        // Depth texture をリサイズ
        self.graphic.depth_texture = self
            .graphic
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Depth Texture (Resized)"),
                size: wgpu::Extent3d {
                    width: size.width,
                    height: size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
        self.graphic.depth_view = self
            .graphic
            .depth_texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // リサイズ時にカメラのアスペクト比も更新
        self.update_camera_uniforms();
    }

    pub fn render(&mut self) {
        // ステージ固有の更新（Octree深さアニメーションなど）
        self.renderer.update();
        {
            let stage = self.renderer.get_stage_mut();
            if let Some(octree_stage) = stage.as_any_mut().downcast_mut::<OctreeStage>() {
                octree_stage.tick_animation(&self.graphic.device);
            }
        }

        // 毎フレーム カメラ行列を更新（Stage の transform をリアルタイム反映）
        self.update_camera_uniforms();

        self.renderer.update_view_rect_overlay(
            &self.graphic.queue,
            self.active_view_rect,
            self.graphic.config.width,
            self.graphic.config.height,
        );
        self.renderer.update_snapshot_overlay(
            &self.graphic.queue,
            self.snapshot_progress_ratio(),
            &self.snapshot_overlay_style,
            self.graphic.config.width,
            self.graphic.config.height,
        );
        self.graphic.render(&mut self.renderer);
    }

    fn snapshot_progress_ratio(&self) -> Option<f32> {
        let series = self.debug_snapshot_series.as_ref()?;
        if series.frames.is_empty() {
            return None;
        }

        Some(if series.frames.len() <= 1 {
            1.0
        } else {
            (self.debug_snapshot_cursor as f32) / ((series.frames.len() - 1) as f32)
        }
        .clamp(0.0, 1.0))
    }

    fn snapshot_track_rect(&self) -> ViewRect {
        ViewRect {
            x: 16.0,
            y: 16.0,
            width: 220.0,
            height: 14.0,
        }
    }

    fn is_cursor_on_snapshot_track(&self, cursor: (f32, f32)) -> bool {
        self.snapshot_track_rect().contains(cursor)
    }

    fn set_snapshot_cursor_from_x(&mut self, x: f32, emit_log: bool) {
        let Some(series) = &self.debug_snapshot_series else {
            return;
        };
        if series.frames.is_empty() {
            return;
        }

        let rect = self.snapshot_track_rect();
        let progress = ((x - rect.x) / rect.width).clamp(0.0, 1.0);
        let next_index = if series.frames.len() <= 1 {
            0
        } else {
            (progress * (series.frames.len() as f32 - 1.0)).round() as usize
        };

        if next_index != self.debug_snapshot_cursor {
            self.debug_snapshot_cursor = next_index;
            self.log_current_snapshot_frame(emit_log);
        }
    }

    pub fn set_stage_draft(&mut self) {
        let stage = Box::new(DraftStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        self.renderer.set_stage(stage);
    }

    pub fn set_stage_outline(&mut self) {
        let stage = Box::new(OutlineStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        self.renderer.set_stage(stage);
    }

    pub fn set_stage_shading(&mut self) {
        let stage = Box::new(ShadingStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        self.renderer.set_stage(stage);
    }

    /// デバッグ用：VoxelOctree可視化を表示
    pub fn load_debug_octree(&mut self) {
        use viewmodel::octree_converter::create_sample_swept_cylinder_wireframe_colored_levels_with_settings;

        tracing::info!("VoxelOctree可視化デバッグ開始");

        // ViewModelでサンプルデータ生成（深さ別ワイヤーフレーム頂点）
        let depth_levels = create_sample_swept_cylinder_wireframe_colored_levels_with_settings(
            &self.octree_visualization_settings,
        );

        let initial_depth = depth_levels
            .iter()
            .position(|vertices| !vertices.is_empty())
            .unwrap_or(0);

        let positions: Vec<[f32; 3]> = depth_levels
            .get(initial_depth)
            .map(|vertices| vertices.iter().map(|v| v.position).collect())
            .unwrap_or_default();

        if positions.is_empty() {
            tracing::warn!("Octreeワイヤーフレーム頂点が空のため表示をスキップ");
            return;
        }

        // 生成頂点のAABBを計算（表示対象を確実に画角内に収めるため）
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut min_z = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        let mut max_z = f32::NEG_INFINITY;

        for pos in &positions {
            min_x = min_x.min(pos[0]);
            min_y = min_y.min(pos[1]);
            min_z = min_z.min(pos[2]);
            max_x = max_x.max(pos[0]);
            max_y = max_y.max(pos[1]);
            max_z = max_z.max(pos[2]);
        }

        let center_x = (min_x + max_x) * 0.5;
        let center_y = (min_y + max_y) * 0.5;
        let center_z = (min_z + max_z) * 0.5;

        let size_x = (max_x - min_x).max(1.0);
        let size_y = (max_y - min_y).max(1.0);
        let size_z = (max_z - min_z).max(1.0);
        let half_extent_xy = (size_x.max(size_y) * 0.5 * 1.4).max(10.0); // 40%マージン + 最小表示サイズ

        tracing::info!("ワイヤーフレーム頂点数: {}", positions.len());

        // 最初の数頂点の座標をログ出力（デバッグ用）
        for (i, pos) in positions.iter().take(8).enumerate() {
            tracing::info!("頂点[{}]: [{:.1}, {:.1}, {:.1}]", i, pos[0], pos[1], pos[2]);
        }

        // OctreeStageを作成してデータ設定
        let mut octree_stage = Box::new(OctreeStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        octree_stage.set_depth_levels(&self.graphic.device, depth_levels);
        octree_stage.set_depth(&self.graphic.device, initial_depth);

        // カメラ設定（頂点範囲へ自動フィット）
        self.camera.target = Vec3f::new(center_x, center_y, center_z);
        self.camera.distance = (size_z * 6.0 + half_extent_xy).max(80.0);
        self.camera.zoom = 1.0;
        self.camera.rotation = Quaternionf::identity();
        self.camera
            .set_projection_mode(viewmodel_graphics::camera::ProjectionMode::Orthographic);

        // 直交投影範囲はカメラ空間基準で設定（view変換後の範囲）
        // target でビュー行列側に平行移動されるため、ここで world center を足すと二重補正になる
        self.camera.set_orthographic_bounds(
            -half_extent_xy,
            half_extent_xy,
            -half_extent_xy,
            half_extent_xy,
        );

        tracing::info!(
            "カメラ設定: target=({:.1}, {:.1}, {:.1}), distance={:.1}, bounds(camspace)=({:.1}..{:.1}, {:.1}..{:.1}), 平行投影",
            center_x,
            center_y,
            center_z,
            self.camera.distance,
            -half_extent_xy,
            half_extent_xy,
            -half_extent_xy,
            half_extent_xy,
        );

        // 実際のカメラ位置を計算して表示
        let view_mat = self.camera.view_matrix();
        tracing::info!(
            "view_matrix[3]: [{:.2}, {:.2}, {:.2}, {:.2}]",
            view_mat[3][0],
            view_mat[3][1],
            view_mat[3][2],
            view_mat[3][3]
        );

        self.renderer.set_stage(octree_stage);

        // カメラユニフォーム更新
        self.update_camera_uniforms();

        tracing::info!("VoxelOctree可視化デバッグ完了");
        tracing::info!("初期表示深さ: {}", initial_depth);
        tracing::info!("o: 深さを1段進める, Shift+O: 深さアニメーション再生");
    }

    /// デバッグ用：CAMシミュレーション可視化（ToolPath + ワークOctree + 除去結果）を表示
    pub fn load_debug_toolpath(&mut self) {
        use viewmodel::cam_sim_visualization_converter::create_sample_cam_simulation_visualization_bundle_with_settings;

        tracing::info!("CAMシミュレーション可視化デバッグ開始（pキー）");

        let bundle = match create_sample_cam_simulation_visualization_bundle_with_settings(
            &self.octree_visualization_settings,
        ) {
            Ok(bundle) => bundle,
            Err(error) => {
                tracing::error!("CAMシミュレーション可視化データ生成失敗: {}", error);
                return;
            }
        };

        if bundle.snapshot_wireframes.is_empty() {
            tracing::warn!("CAMシミュレーション可視化フレームが空のため表示をスキップ");
            return;
        }

        // 進捗表示・スクラブ用スナップショットを更新
        self.debug_snapshot_series = Some(bundle.snapshot_series);
        self.debug_snapshot_wireframes = Some(bundle.snapshot_wireframes);
        self.debug_snapshot_solids = Some(
            bundle
                .snapshot_solid_meshes
                .iter()
                .map(|(vertex_data, indices)| {
                    (
                        convert_vertex_data_to_mesh_vertices(vertex_data),
                        indices.clone(),
                    )
                })
                .collect(),
        );
        self.debug_snapshot_toolpath_lines = Some(
            bundle
                .toolpath_wireframe
                .iter()
                .map(|v| MeshVertex::new(v.position, v.color))
                .collect(),
        );
        self.debug_snapshot_tool_lines = Some(
            bundle
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
                .collect(),
        );
        self.debug_snapshot_shaded_mode = false;
        self.debug_snapshot_cursor = 0;

        let frame_wireframes = self.debug_snapshot_wireframes.as_ref().expect("frame wireframes");
        let frame_count = frame_wireframes.len();
        let positions: Vec<[f32; 3]> = frame_wireframes
            .last()
            .map(|vertices| vertices.iter().map(|v| v.position).collect())
            .unwrap_or_default();

        if positions.is_empty() {
            tracing::warn!("CAMシミュレーション可視化頂点が空のため表示をスキップ");
            return;
        }

        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut min_z = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        let mut max_z = f32::NEG_INFINITY;

        for pos in &positions {
            min_x = min_x.min(pos[0]);
            min_y = min_y.min(pos[1]);
            min_z = min_z.min(pos[2]);
            max_x = max_x.max(pos[0]);
            max_y = max_y.max(pos[1]);
            max_z = max_z.max(pos[2]);
        }

        let center_x = (min_x + max_x) * 0.5;
        let center_y = (min_y + max_y) * 0.5;
        let center_z = (min_z + max_z) * 0.5;
        let size_x = (max_x - min_x).max(1.0);
        let size_y = (max_y - min_y).max(1.0);
        let size_z = (max_z - min_z).max(1.0);
        let half_extent_xy = (size_x.max(size_y) * 0.5 * 1.4).max(10.0);

        let mut octree_stage = Box::new(OctreeStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        octree_stage.set_depth_levels(&self.graphic.device, frame_wireframes.clone());
        octree_stage.set_depth(&self.graphic.device, 0);

        self.camera.target = Vec3f::new(center_x, center_y, center_z);
        self.camera.distance = (size_z * 6.0 + half_extent_xy).max(80.0);
        self.camera.zoom = 1.0;
        self.camera.rotation = Quaternionf::identity();
        self.camera
            .set_projection_mode(viewmodel_graphics::camera::ProjectionMode::Orthographic);
        self.camera.set_orthographic_bounds(
            -half_extent_xy,
            half_extent_xy,
            -half_extent_xy,
            half_extent_xy,
        );

        self.renderer.set_stage(octree_stage);
        self.update_camera_uniforms();
        self.log_current_snapshot_frame(true);

        tracing::info!(
            "CAMシミュレーション可視化デバッグ完了: frame={}/{}（k/スクラブで時系列再生）",
            1,
            frame_count
        );
    }

    /// デバッグ用：カッターパスのみを表示（pキー）
    pub fn load_debug_cutter_path_only(&mut self) {
        use viewmodel::toolpath_converter::{create_sample_toolpath, toolpath_to_vertices, ToolPathVisualizationSettings};

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

        self.debug_snapshot_series = None;
        self.debug_snapshot_wireframes = None;
        self.debug_snapshot_solids = None;
        self.debug_snapshot_toolpath_lines = None;
        self.debug_snapshot_tool_lines = None;
        self.debug_snapshot_shaded_mode = false;
        self.debug_snapshot_cursor = 0;

        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut min_z = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        let mut max_z = f32::NEG_INFINITY;

        for vertex in &vertices {
            let pos = vertex.position;
            min_x = min_x.min(pos[0]);
            min_y = min_y.min(pos[1]);
            min_z = min_z.min(pos[2]);
            max_x = max_x.max(pos[0]);
            max_y = max_y.max(pos[1]);
            max_z = max_z.max(pos[2]);
        }

        let center_x = (min_x + max_x) * 0.5;
        let center_y = (min_y + max_y) * 0.5;
        let center_z = (min_z + max_z) * 0.5;
        let size_x = (max_x - min_x).max(1.0);
        let size_y = (max_y - min_y).max(1.0);
        let size_z = (max_z - min_z).max(1.0);
        let half_extent_xy = (size_x.max(size_y) * 0.5 * 1.4).max(10.0);

        let mut mesh_stage = Box::new(MeshStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        mesh_stage.set_line_data(&self.graphic.device, vertices);

        self.camera.target = Vec3f::new(center_x, center_y, center_z);
        self.camera.distance = (size_z * 6.0 + half_extent_xy).max(80.0);
        self.camera.zoom = 1.0;
        self.camera.rotation = Quaternionf::identity();
        self.camera
            .set_projection_mode(viewmodel_graphics::camera::ProjectionMode::Orthographic);
        self.camera.set_orthographic_bounds(
            -half_extent_xy,
            half_extent_xy,
            -half_extent_xy,
            half_extent_xy,
        );

        self.renderer.set_stage(mesh_stage);
        self.update_camera_uniforms();

        tracing::info!("カッターパス表示デバッグ完了: 線分数={}", toolpath_vertices.colors.len());
    }

    /// STLファイルを読み込んでメッシュステージに設定
    pub fn load_stl_file(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("STLファイル読み込み開始: {:?}", path);

        // STLファイルを読み込み、レンダリング用データに変換
        let (vertices, indices, _bounds) = stl_loader::load_stl_for_rendering(path)?;

        // カメラを標準CAD視点に設定（固定値）
        self.camera.reset_to_standard_cad_view();

        // メッシュステージを作成してSTLデータを設定
        let mut mesh_stage = Box::new(MeshStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        mesh_stage.set_mesh_data(&self.graphic.device, vertices, indices);

        self.renderer.set_stage(mesh_stage);

        tracing::info!("STLファイル読み込み完了");

        // カメラのユニフォームを初期化
        self.update_camera_uniforms();

        Ok(())
    }

    /// サンプルSTLファイルを作成して読み込み
    pub fn load_sample_stl(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let sample_path = std::env::temp_dir().join("redring_sample.stl");

        // サンプルSTLファイルを作成して読み込み
        let (vertices, indices, _bounds) = stl_loader::create_sample_stl_with_bounds(&sample_path)?;

        // カメラを標準CAD視点に設定（固定値）
        self.camera.reset_to_standard_cad_view();

        // メッシュステージを作成してSTLデータを設定
        let mut mesh_stage = Box::new(MeshStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        mesh_stage.set_mesh_data(&self.graphic.device, vertices, indices);

        self.renderer.set_stage(mesh_stage);

        // カメラのユニフォームを初期化
        self.update_camera_uniforms();

        Ok(())
    }

    /// デバッグ用：LineSegment3Dを表示（SVGから読み込み）
    pub fn load_debug_line(&mut self) {
        use std::path::Path;

        tracing::info!("デバッグ形状: LineSegment3D表示（EntityManager経由）");

        let svg_path = Path::new("tests/fixtures/shapes/line.svg");
        match crate::svg_loader::load_svg_for_rendering(svg_path, None) {
            Ok(vertices) => {
                tracing::info!("SVG読み込み成功: {} 頂点", vertices.len());

                let id = self.entity_manager.add_line_entity(vertices);
                let _ = self.entity_manager.select(id);

                self.camera.reset_to_standard_cad_view();
                self.rebuild_stage_from_entities();
            }
            Err(e) => {
                tracing::error!("SVG読み込みエラー: {}", e);
            }
        }
    }

    /// デバッグ用：Circle3Dを表示（SVGから読み込み）
    pub fn load_debug_circle(&mut self) {
        use std::path::Path;

        tracing::info!("デバッグ形状: Circle3D表示（SVGから）");

        let svg_path = Path::new("tests/fixtures/shapes/circle.svg");
        match crate::svg_loader::load_svg_for_rendering(svg_path, None) {
            Ok(vertices) => {
                tracing::info!("SVG読み込み成功: {} 頂点", vertices.len());

                self.camera.reset_to_standard_cad_view();

                let mut mesh_stage = Box::new(MeshStage::new(
                    &self.graphic.device,
                    self.graphic.config.format,
                ));
                mesh_stage.set_line_data(&self.graphic.device, vertices);

                self.renderer.set_stage(mesh_stage);
                self.update_camera_uniforms();
            }
            Err(e) => {
                tracing::error!("SVG読み込みエラー: {}", e);
            }
        }
    }

    /// デバッグ用：クリップ空間座標の単純な正方形（SVGから読み込み）
    pub fn load_debug_clip_square(&mut self) {
        use std::path::Path;

        tracing::warn!("DEBUG: クリップ空間正方形を表示（SVGから）");

        let svg_path = Path::new("tests/fixtures/shapes/clip_square.svg");
        match crate::svg_loader::load_svg_for_rendering(svg_path, None) {
            Ok(vertices) => {
                tracing::warn!("SVG読み込み成功: {} 頂点", vertices.len());

                let mut mesh_stage = Box::new(stage::mesh_stage::MeshStage::new(
                    &self.graphic.device,
                    self.graphic.config.format,
                ));

                mesh_stage.set_line_data(&self.graphic.device, vertices);
                self.renderer.set_stage(mesh_stage);

                self.update_camera_uniforms();
            }
            Err(e) => {
                tracing::error!("SVG読み込みエラー: {}", e);
            }
        }
    }

    /// デバッグ用：Triangle3Dを表示（SVGから読み込み）
    pub fn load_debug_triangle(&mut self) {
        use std::path::Path;

        tracing::info!("デバッグ形状: Triangle3D表示（SVGから）");

        let svg_path = Path::new("tests/fixtures/shapes/triangle.svg");
        match crate::svg_loader::load_svg_for_rendering(svg_path, None) {
            Ok(vertices) => {
                tracing::info!("SVG読み込み成功: {} 頂点", vertices.len());

                // インデックスを生成（TriangleList用）
                let indices: Vec<u32> = vec![0, 1, 2];

                self.camera.reset_to_standard_cad_view();

                let mut mesh_stage = Box::new(MeshStage::new(
                    &self.graphic.device,
                    self.graphic.config.format,
                ));
                mesh_stage.set_mesh_data(&self.graphic.device, vertices, indices);

                self.renderer.set_stage(mesh_stage);
                self.update_camera_uniforms();
            }
            Err(e) => {
                tracing::error!("SVG読み込みエラー: {}", e);
            }
        }
    }

    /// デバッグ用：Arc3Dを表示（SVGから読み込み）
    pub fn load_debug_arc(&mut self) {
        use std::path::Path;

        tracing::info!("デバッグ形状: Arc3D表示（SVGから）");

        let svg_path = Path::new("tests/fixtures/shapes/arc.svg");
        match crate::svg_loader::load_svg_for_rendering(svg_path, None) {
            Ok(vertices) => {
                tracing::info!("SVG読み込み成功: {} 頂点", vertices.len());

                self.camera.reset_to_standard_cad_view();

                let mut mesh_stage = Box::new(MeshStage::new(
                    &self.graphic.device,
                    self.graphic.config.format,
                ));
                mesh_stage.set_line_data(&self.graphic.device, vertices);

                self.renderer.set_stage(mesh_stage);
                self.update_camera_uniforms();
            }
            Err(e) => {
                tracing::error!("SVG読み込みエラー: {}", e);
            }
        }
    }

    /// デバッグ用：NurbsCurve3Dを表示（SVGから読み込み）
    /// ViewModelレイヤー経由で評価データを生成
    pub fn load_debug_nurbs(&mut self) {
        use std::path::Path;

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
                Err(e) => {
                    tracing::error!("NURBS曲線データ生成失敗: {}", e);
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
                Err(e) => {
                    tracing::error!("NURBS曲面データ生成失敗: {}", e);
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

    /// カメラをリセット
    pub fn reset_camera(&mut self) {
        self.camera.reset();
        self.update_camera_uniforms();
    }

    /// 安全な視点にカメラをリセット（標準CAD視点）
    pub fn reset_camera_to_safe_view(&mut self) {
        // 固定の標準CAD視点にリセット
        self.camera.reset_to_standard_cad_view();
        self.update_camera_uniforms();
        tracing::info!("カメラを標準CAD視点にリセット");
    }

    /// 緊急脱出：最小距離を強制確保
    pub fn emergency_camera_escape(&mut self) {
        self.camera.ensure_minimum_distance();
        self.update_camera_uniforms();
        tracing::warn!("緊急カメラ脱出実行");
    }

    /// カメラ状態をログ出力
    pub fn log_camera_state(&self) {
        self.camera.log_state();
    }

    /// ワイヤーフレーム表示を切り替え
    pub fn toggle_wireframe(&mut self) {
        // CAMスナップショット可視化時: Octreeワイヤ ↔ ソリッドメッシュを切替
        if self.debug_snapshot_series.is_some()
            && self.debug_snapshot_wireframes.is_some()
            && self.debug_snapshot_solids.is_some()
        {
            if self.debug_snapshot_shaded_mode {
                let Some(snapshot_wireframes) = &self.debug_snapshot_wireframes else {
                    return;
                };
                if snapshot_wireframes.is_empty() {
                    return;
                }

                let mut octree_stage = Box::new(OctreeStage::new(
                    &self.graphic.device,
                    self.graphic.config.format,
                ));
                octree_stage.set_depth_levels(&self.graphic.device, snapshot_wireframes.clone());
                let frame_index = self
                    .debug_snapshot_cursor
                    .min(snapshot_wireframes.len().saturating_sub(1));
                octree_stage.set_depth(&self.graphic.device, frame_index);
                self.renderer.set_stage(octree_stage);
                self.debug_snapshot_shaded_mode = false;
                self.update_camera_uniforms();
                tracing::info!("Octree表示モード: ワイヤーフレーム");
                return;
            }

            let Some(snapshot_solids) = &self.debug_snapshot_solids else {
                return;
            };
            if snapshot_solids.is_empty() {
                return;
            }

            let frame_index = self
                .debug_snapshot_cursor
                .min(snapshot_solids.len().saturating_sub(1));
            let (vertices, indices) = snapshot_solids[frame_index].clone();

            let mut mesh_stage = Box::new(MeshStage::new(
                &self.graphic.device,
                self.graphic.config.format,
            ));
            mesh_stage.set_mesh_data(&self.graphic.device, vertices, indices);
            mesh_stage.set_mesh_base_color(self.snapshot_shaded_color_settings.work_solid_color);

            let toolpath_lines = self.debug_snapshot_toolpath_lines.clone().unwrap_or_default();
            if let Some(tool_lines_per_frame) = &self.debug_snapshot_tool_lines {
                let overlay_index = frame_index.min(tool_lines_per_frame.len().saturating_sub(1));
                let tool_lines = tool_lines_per_frame[overlay_index].clone();
                if !tool_lines.is_empty() {
                    mesh_stage.set_overlay_tool_line_data(&self.graphic.device, tool_lines);
                }
            }
            if !toolpath_lines.is_empty() {
                mesh_stage.set_overlay_toolpath_line_data(&self.graphic.device, toolpath_lines);
            }
            self.renderer.set_stage(mesh_stage);
            self.debug_snapshot_shaded_mode = true;
            self.update_camera_uniforms();
            tracing::info!("Octree表示モード: シェーディング（ソリッド）");
            return;
        }

        let stage = self.renderer.get_stage_mut();

        // MeshStageの場合
        if let Some(mesh_stage) = stage.as_any_mut().downcast_mut::<MeshStage>() {
            mesh_stage.toggle_wireframe();
            let mode = if mesh_stage.is_wireframe() {
                "ワイヤーフレーム"
            } else {
                "ソリッド"
            };
            tracing::info!("表示モードを{}に切り替え", mode);
            return;
        }

        // NurbsSurfaceStageの場合
        if let Some(nurbs_stage) = stage.as_any_mut().downcast_mut::<NurbsSurfaceStage>() {
            nurbs_stage.toggle_wireframe();
            let mode = if nurbs_stage.is_wireframe() {
                "ワイヤーフレーム"
            } else {
                "ソリッド"
            };
            tracing::info!("表示モードを{}に切り替え", mode);
            return;
        }

        tracing::warn!("現在のステージはワイヤーフレーム表示に対応していません");
    }

    /// キーボード入力を処理
    pub fn handle_keyboard_input(&mut self, key: &winit::keyboard::Key, pressed: bool) {
        self.mouse_input.update_key(key, pressed);

        // キーが押された時のみ処理
        if !pressed {
            return;
        }

        if let winit::keyboard::Key::Character(ch) = key {
            match ch.as_str() {
                "r" => {
                    // リセット（基本）
                    self.camera.reset();
                    self.update_camera_uniforms();
                    tracing::info!("カメラをリセット（rキー）");
                }
                "t" => {
                    // 標準CAD視点
                    self.camera.reset_to_standard_cad_view();
                    self.update_camera_uniforms();
                    tracing::info!("標準CAD視点に設定（tキー）");
                }
                "f" => {
                    // 正面視点（デバッグ用）
                    self.camera.reset_to_front_view();
                    self.update_camera_uniforms();
                    tracing::info!("正面視点に設定（fキー）");
                }
                "e" => {
                    // 緊急脱出
                    self.camera.emergency_camera_escape();
                    self.update_camera_uniforms();
                    tracing::warn!("緊急カメラ脱出実行（eキー）");
                }
                "h" => {
                    // ヘルプ表示
                    tracing::info!("=== カメラ操作ヘルプ ===");
                    tracing::info!("r: カメラリセット");
                    tracing::info!("t: 標準CAD視点");
                    tracing::info!("f: 正面視点");
                    tracing::info!("e: 緊急脱出");
                    tracing::info!("w: ワイヤーフレーム切替");
                    tracing::info!("1: Draftステージ");
                    tracing::info!("2: Outlineステージ");
                    tracing::info!("3: Shadingステージ");
                    tracing::info!("=== デバッグ形状表示 ===");
                    tracing::info!("s: クリップ空間正方形（単位行列テスト）");
                    tracing::info!("l: LineSegment3D表示");
                    tracing::info!("c: Circle3D表示");
                    tracing::info!("T: Triangle3D表示 (Shift+T)");
                    tracing::info!("a: Arc3D表示");
                    tracing::info!("n: NurbsCurve3D表示（GPU評価）");
                    tracing::info!("m: NurbsSurface3D表示（GPU評価）");
                    tracing::info!("o: Octree再分割表示/深さ送り（同一最終形状の粗→細）");
                    tracing::info!("Shift+O: Octree深さアニメーション再生（粗→細）");
                    tracing::info!("p: カッターパスのみ表示（色分け線）");
                    tracing::info!("Shift+P: CAMシミュレーション可視化（ToolPath + ワーク + 除去）");
                    tracing::info!("k/j: Snapshotフレーム送り/巻き戻し");
                    tracing::info!("w: Octree表示モード切替（Wire/Solid）");
                    tracing::info!("=== その他 ===");
                    tracing::info!(
                        "マウス操作: Ctrl+左ドラッグ=回転, Ctrl+中ドラッグ=パン, Ctrl+右ドラッグ=ズーム"
                    );
                }
                "w" => {
                    // ワイヤーフレーム切替
                    self.toggle_wireframe();
                }
                "q" => {
                    // デバッグ: クリップ空間正方形（単位行列テスト）
                    self.load_debug_clip_square();
                }
                "1" => {
                    // ステージ切替: Draft
                    self.set_stage_draft();
                    self.update_camera_uniforms();
                    tracing::info!("ステージ切替: Draft");
                }
                "2" => {
                    // ステージ切替: Outline
                    self.set_stage_outline();
                    self.update_camera_uniforms();
                    tracing::info!("ステージ切替: Outline");
                }
                "3" => {
                    // ステージ切替: Shading
                    self.set_stage_shading();
                    self.update_camera_uniforms();
                    tracing::info!("ステージ切替: Shading");
                }
                "l" => {
                    // デバッグ: LineSegment3D表示
                    self.load_debug_line();
                }
                "c" => {
                    // デバッグ: Circle3D表示
                    self.load_debug_circle();
                }
                "a" => {
                    // デバッグ: Arc3D表示
                    self.load_debug_arc();
                }
                "n" => {
                    // デバッグ: NurbsCurve3D表示（GPU評価）
                    self.load_debug_nurbs();
                }
                "m" => {
                    // デバッグ: NurbsSurface3D表示（GPU評価）
                    self.load_debug_nurbs_surface();
                }
                "o" => {
                    // デバッグ: Octree可視化表示（表示中は深さ送り）
                    let mut handled = false;
                    {
                        let stage = self.renderer.get_stage_mut();
                        if let Some(octree_stage) = stage.as_any_mut().downcast_mut::<OctreeStage>()
                        {
                            octree_stage.cycle_next_depth(&self.graphic.device);
                            tracing::info!(
                                "Octree深さ表示: {}/{}",
                                octree_stage.current_depth(),
                                octree_stage.max_depth()
                            );
                            handled = true;
                        }
                    }

                    if !handled {
                        self.load_debug_octree();
                    }
                }
                "O" => {
                    // デバッグ: Octree深さアニメーション
                    let mut handled = false;
                    {
                        let stage = self.renderer.get_stage_mut();
                        if let Some(octree_stage) = stage.as_any_mut().downcast_mut::<OctreeStage>()
                        {
                            octree_stage.start_depth_animation();
                            tracing::info!("Octree深さアニメーション再生開始");
                            handled = true;
                        }
                    }

                    if !handled {
                        self.load_debug_octree();
                        let stage = self.renderer.get_stage_mut();
                        if let Some(octree_stage) = stage.as_any_mut().downcast_mut::<OctreeStage>()
                        {
                            octree_stage.start_depth_animation();
                            tracing::info!("Octree深さアニメーション再生開始");
                        }
                    }
                }
                "p" => {
                    // デバッグ: カッターパスのみ表示
                    self.load_debug_cutter_path_only();
                }
                "P" => {
                    // デバッグ: CAMシミュレーション可視化（ToolPath + ワーク + 除去）
                    self.load_debug_toolpath();
                }
                "k" => {
                    // デバッグ: シミュレーションスナップショット読み込み/次フレーム
                    if self.debug_snapshot_series.is_some() {
                        self.cycle_debug_simulation_snapshot();
                    } else {
                        self.load_debug_simulation_snapshots();
                    }
                }
                "j" => {
                    // デバッグ: シミュレーションスナップショットを前フレームへ巻き戻し
                    if self.debug_snapshot_series.is_some() {
                        self.rewind_debug_simulation_snapshot();
                    } else {
                        self.load_debug_simulation_snapshots();
                    }
                }
                "T" => {
                    // デバッグ: Triangle3D表示（Shift+T）
                    self.load_debug_triangle();
                }
                _ => {}
            }
        }
    }

    /// マウスボタン入力を処理
    pub fn handle_mouse_button(
        &mut self,
        button: winit::event::MouseButton,
        state: winit::event::ElementState,
    ) {
        self.mouse_input.update_mouse_button(button, state);

        if button != winit::event::MouseButton::Left {
            return;
        }

        match state {
            winit::event::ElementState::Pressed => {
                if let Some(cursor) = self.cursor_position {
                    if self.is_cursor_on_snapshot_track(cursor) {
                        if self.debug_snapshot_series.is_none() {
                            self.load_debug_simulation_snapshots();
                        }
                        self.snapshot_scrub_active = true;
                        self.mouse_input.operation = crate::mouse_input::MouseOperation::None;
                        self.set_snapshot_cursor_from_x(cursor.0, true);
                        tracing::debug!("左クリック: snapshotスクラブ開始");
                        return;
                    }
                }

                if self.mouse_input.ctrl_pressed {
                    self.arcball_drag_start = self.cursor_position;
                    let viewport_width = self.graphic.config.width as f32;
                    let viewport_height = self.graphic.config.height as f32;
                    self.arcball_virtual_cursor = Some(
                        self.cursor_position
                            .unwrap_or((viewport_width * 0.5, viewport_height * 0.5)),
                    );
                    tracing::info!(
                        "Ctrl+左ドラッグ: カメラ回転モード開始 start={:?} current={:?} virtual_start={:?}",
                        self.arcball_drag_start,
                        self.cursor_position,
                        self.arcball_virtual_cursor
                    );
                    return;
                }

                tracing::info!("左ドラッグ: ビュー矩形選択モード（カメラ操作はCtrl+ドラッグ）");

                if let Some(cursor) = self.cursor_position {
                    self.view_rect_drag_origin = Some(cursor);
                    self.active_view_rect = Some(ViewRect::from_points(cursor, cursor));
                }
            }
            winit::event::ElementState::Released => {
                if self.snapshot_scrub_active {
                    self.snapshot_scrub_active = false;
                    tracing::debug!("左ドラッグ: snapshotスクラブ終了");
                    self.log_current_snapshot_frame(true);
                    return;
                }

                if let Some(start) = self.arcball_drag_start.take() {
                    tracing::info!(
                        "Ctrl+左ドラッグ: カメラ回転モード終了 start={:?} end={:?} virtual_end={:?}",
                        start,
                        self.cursor_position,
                        self.arcball_virtual_cursor
                    );
                }
                self.arcball_virtual_cursor = None;

                if self.view_rect_drag_origin.take().is_some() {
                    if let Some(rect) = self.active_view_rect.take() {
                        if !rect.is_empty() {
                            self.last_view_rect = Some(rect);
                        }
                    }
                }
            }
        }
    }

    pub fn handle_cursor_moved(&mut self, x: f32, y: f32) {
        self.last_cursor_position = self.cursor_position;
        self.cursor_position = Some((x, y));

        if self.snapshot_scrub_active {
            self.set_snapshot_cursor_from_x(x, false);
            return;
        }

        if self.mouse_input.operation == crate::mouse_input::MouseOperation::Rotate {
            tracing::debug!(
                "🖱️ CursorMoved(rotate): last={:?} current=({:.1},{:.1}) start={:?}",
                self.last_cursor_position,
                x,
                y,
                self.arcball_drag_start
            );
        }

        if let Some(origin) = self.view_rect_drag_origin {
            self.active_view_rect = Some(ViewRect::from_points(origin, (x, y)));
        }
    }

    /// Ctrl+ホイールでズーム
    pub fn handle_mouse_wheel(&mut self, delta: winit::event::MouseScrollDelta) {
        if !self.mouse_input.ctrl_pressed {
            return;
        }

        let sensitivity = self.viewing_operation_settings.camera_control_sensitivity;
        let scroll_y = match delta {
            winit::event::MouseScrollDelta::LineDelta(_, y) => y,
            winit::event::MouseScrollDelta::PixelDelta(pos) => {
                pos.y as f32 * sensitivity.wheel_pixel_to_line
            }
        };

        if scroll_y.abs() < 1e-6 {
            return;
        }

        self.camera.zoom_wheel(scroll_y);
        self.update_camera_uniforms();

        tracing::debug!(
            "🖱️ MouseWheel(zoom): ctrl=true, scroll_y={:.3}, camera_distance={:.3}, bounds={:?}",
            scroll_y,
            self.camera.distance,
            self.camera.orthographic_bounds
        );
    }

    /// カメラ操作感度を設定
    pub fn set_camera_control_sensitivity(&mut self, sensitivity: CameraControlSensitivity) {
        self.viewing_operation_settings.camera_control_sensitivity = sensitivity;
        self.apply_viewing_operation_settings();
    }

    /// Snapshotシェーディング時のワークソリッド色を設定
    pub fn set_snapshot_work_solid_color(&mut self, color: [f32; 4]) {
        self.snapshot_shaded_color_settings.work_solid_color = color;

        if self.debug_snapshot_shaded_mode {
            let stage = self.renderer.get_stage_mut();
            if let Some(mesh_stage) = stage.as_any_mut().downcast_mut::<MeshStage>() {
                mesh_stage.set_mesh_base_color(color);
            }
        }
    }

    /// Snapshotシェーディング時の工具ワイヤー色を設定
    pub fn set_snapshot_tool_wire_color(&mut self, color: [f32; 3]) {
        self.snapshot_shaded_color_settings.tool_wire_color = color;

        if let Some(tool_lines_per_frame) = &mut self.debug_snapshot_tool_lines {
            for frame in tool_lines_per_frame.iter_mut() {
                for vertex in frame.iter_mut() {
                    vertex.normal = color;
                }
            }
        }

        if self.debug_snapshot_shaded_mode {
            self.sync_snapshot_visual_frame();
        }
    }

    /// Snapshotシェーディング色設定を一括更新
    pub fn set_snapshot_shaded_color_settings(&mut self, settings: SnapshotShadedColorSettings) {
        self.set_snapshot_work_solid_color(settings.work_solid_color);
        self.set_snapshot_tool_wire_color(settings.tool_wire_color);
    }

    /// Snapshotシェーディング時のワークソリッド色を取得
    pub fn snapshot_work_solid_color(&self) -> [f32; 4] {
        self.snapshot_shaded_color_settings.work_solid_color
    }

    /// Snapshotシェーディング時の工具ワイヤー色を取得
    pub fn snapshot_tool_wire_color(&self) -> [f32; 3] {
        self.snapshot_shaded_color_settings.tool_wire_color
    }

    /// Snapshotシェーディング色設定を取得
    pub fn snapshot_shaded_color_settings(&self) -> SnapshotShadedColorSettings {
        self.snapshot_shaded_color_settings
    }

    /// Octree深さグラデーション開始色（浅い）を設定
    pub fn set_octree_gradient_start(&mut self, color: [f32; 3]) {
        self.octree_visualization_settings.gradient_start = color;
    }

    /// Octree表示色を単色で設定（開始色・終了色の両方に同じ色を適用）
    pub fn set_octree_single_color(&mut self, color: [f32; 3]) {
        self.octree_visualization_settings.gradient_start = color;
        self.octree_visualization_settings.gradient_end = color;
    }

    /// Octree表示色をグラデーションで一括設定
    pub fn set_octree_gradient_colors(&mut self, start: [f32; 3], end: [f32; 3]) {
        self.octree_visualization_settings.gradient_start = start;
        self.octree_visualization_settings.gradient_end = end;
    }

    /// Octree深さグラデーション終了色（深い）を設定
    pub fn set_octree_gradient_end(&mut self, color: [f32; 3]) {
        self.octree_visualization_settings.gradient_end = color;
    }

    /// Octree深さグラデーション開始色（浅い）を取得
    pub fn octree_gradient_start(&self) -> [f32; 3] {
        self.octree_visualization_settings.gradient_start
    }

    /// Octree深さグラデーション終了色（深い）を取得
    pub fn octree_gradient_end(&self) -> [f32; 3] {
        self.octree_visualization_settings.gradient_end
    }

    /// Octreeトレランス: point_aabb_half_extent を設定
    pub fn set_octree_tolerance_point_aabb_half_extent(&mut self, value: f64) {
        self.octree_visualization_settings
            .octree_tolerance
            .point_aabb_half_extent = value;
    }

    /// Octreeトレランスを一括設定
    pub fn set_octree_tolerance(
        &mut self,
        point_aabb_half_extent: f64,
        query_expand: f64,
        nearest_prune_margin: f64,
    ) {
        self.octree_visualization_settings
            .octree_tolerance
            .point_aabb_half_extent = point_aabb_half_extent;
        self.octree_visualization_settings
            .octree_tolerance
            .query_expand = query_expand;
        self.octree_visualization_settings
            .octree_tolerance
            .nearest_prune_margin = nearest_prune_margin;
    }

    /// Octreeトレランスを単一値で一括設定（3項目に同値を適用）
    pub fn set_octree_tolerance_uniform(&mut self, value: f64) {
        self.set_octree_tolerance(value, value, value);
    }

    /// Octreeトレランス: query_expand を設定
    pub fn set_octree_tolerance_query_expand(&mut self, value: f64) {
        self.octree_visualization_settings
            .octree_tolerance
            .query_expand = value;
    }

    /// Octreeトレランス: nearest_prune_margin を設定
    pub fn set_octree_tolerance_nearest_prune_margin(&mut self, value: f64) {
        self.octree_visualization_settings
            .octree_tolerance
            .nearest_prune_margin = value;
    }

    /// Octreeトレランス: point_aabb_half_extent を取得
    pub fn octree_tolerance_point_aabb_half_extent(&self) -> f64 {
        self.octree_visualization_settings
            .octree_tolerance
            .point_aabb_half_extent
    }

    /// Octreeトレランス: query_expand を取得
    pub fn octree_tolerance_query_expand(&self) -> f64 {
        self.octree_visualization_settings
            .octree_tolerance
            .query_expand
    }

    /// Octreeトレランス: nearest_prune_margin を取得
    pub fn octree_tolerance_nearest_prune_margin(&self) -> f64 {
        self.octree_visualization_settings
            .octree_tolerance
            .nearest_prune_margin
    }

    /// 回転感度を更新
    pub fn set_camera_rotate_sensitivity(&mut self, rotate: f32) {
        self.viewing_operation_settings
            .camera_control_sensitivity
            .rotate = rotate;
        self.apply_viewing_operation_settings();
    }

    /// Arcballデルタ変換スケールを更新
    pub fn set_camera_arcball_sensitivity(&mut self, arcball: f32) {
        self.viewing_operation_settings
            .camera_control_sensitivity
            .arcball = arcball;
        self.apply_viewing_operation_settings();
    }

    /// パン感度を更新
    pub fn set_camera_pan_sensitivity(&mut self, pan: f32) {
        self.viewing_operation_settings
            .camera_control_sensitivity
            .pan = pan;
        self.apply_viewing_operation_settings();
    }

    /// ドラッグズーム感度を更新
    pub fn set_camera_zoom_drag_sensitivity(&mut self, zoom_drag: f32) {
        self.viewing_operation_settings
            .camera_control_sensitivity
            .zoom_drag = zoom_drag;
        self.apply_viewing_operation_settings();
    }

    /// ホイールズーム感度を更新
    pub fn set_camera_zoom_wheel_sensitivity(&mut self, zoom_wheel: f32) {
        self.viewing_operation_settings
            .camera_control_sensitivity
            .zoom_wheel = zoom_wheel;
        self.apply_viewing_operation_settings();
    }

    /// ピクセルホイール→ライン変換係数を更新
    pub fn set_camera_wheel_pixel_to_line(&mut self, wheel_pixel_to_line: f32) {
        self.viewing_operation_settings
            .camera_control_sensitivity
            .wheel_pixel_to_line = wheel_pixel_to_line;
        self.apply_viewing_operation_settings();
    }

    /// ビュー操作設定を一括設定
    pub fn set_viewing_operation_settings(&mut self, settings: ViewingOperationSettings) {
        self.viewing_operation_settings = settings;
        self.apply_viewing_operation_settings();
    }

    /// ビュー操作設定を取得
    pub fn viewing_operation_settings(&self) -> ViewingOperationSettings {
        self.viewing_operation_settings
    }

    /// カメラ操作感度を取得
    pub fn camera_control_sensitivity(&self) -> CameraControlSensitivity {
        self.viewing_operation_settings.camera_control_sensitivity
    }

    /// 回転感度を取得
    pub fn camera_rotate_sensitivity(&self) -> f32 {
        self.viewing_operation_settings
            .camera_control_sensitivity
            .rotate
    }

    /// Arcballデルタ変換スケールを取得
    pub fn camera_arcball_sensitivity(&self) -> f32 {
        self.viewing_operation_settings
            .camera_control_sensitivity
            .arcball
    }

    /// パン感度を取得
    pub fn camera_pan_sensitivity(&self) -> f32 {
        self.viewing_operation_settings
            .camera_control_sensitivity
            .pan
    }

    /// ドラッグズーム感度を取得
    pub fn camera_zoom_drag_sensitivity(&self) -> f32 {
        self.viewing_operation_settings
            .camera_control_sensitivity
            .zoom_drag
    }

    /// ホイールズーム感度を取得
    pub fn camera_zoom_wheel_sensitivity(&self) -> f32 {
        self.viewing_operation_settings
            .camera_control_sensitivity
            .zoom_wheel
    }

    /// ピクセルホイール→ライン変換係数を取得
    pub fn camera_wheel_pixel_to_line(&self) -> f32 {
        self.viewing_operation_settings
            .camera_control_sensitivity
            .wheel_pixel_to_line
    }

    /// マウス移動を処理
    pub fn handle_mouse_motion(&mut self, delta: (f64, f64)) {
        use crate::mouse_input::MouseOperation;

        let (delta_x, delta_y) = (delta.0 as f32, delta.1 as f32);

        match self.mouse_input.operation {
            MouseOperation::Rotate => {
                let viewport_width = self.graphic.config.width as f32;
                let viewport_height = self.graphic.config.height as f32;
                let (prev_x, prev_y) = self
                    .arcball_virtual_cursor
                    .unwrap_or((viewport_width * 0.5, viewport_height * 0.5));

                let curr_x = (prev_x + delta_x).clamp(0.0, viewport_width);
                let curr_y = (prev_y + delta_y).clamp(0.0, viewport_height);
                self.arcball_virtual_cursor = Some((curr_x, curr_y));

                tracing::debug!(
                    "🎯 MouseMotion(rotate): delta=({:.2},{:.2}) start={:?} last={:?} current={:?} virtual_prev=({:.1},{:.1}) virtual_curr=({:.1},{:.1}) center=({:.1},{:.1})",
                    delta_x,
                    delta_y,
                    self.arcball_drag_start,
                    self.last_cursor_position,
                    self.cursor_position,
                    prev_x,
                    prev_y,
                    curr_x,
                    curr_y,
                    viewport_width * 0.5,
                    viewport_height * 0.5
                );
                self.camera.rotate_arcball(
                    prev_x,
                    prev_y,
                    curr_x,
                    curr_y,
                    viewport_width,
                    viewport_height,
                );
                self.update_camera_uniforms();
            }
            MouseOperation::Pan => {
                self.camera.pan(delta_x, delta_y);
                self.update_camera_uniforms();
            }
            MouseOperation::Zoom => {
                self.camera.zoom(delta_x, delta_y);
                self.update_camera_uniforms();
            }
            MouseOperation::None => {}
        }
    }

    /// カメラのユニフォームを更新
    pub fn update_camera_uniforms(&mut self) {
        let view_matrix = self.camera.view_matrix();
        let aspect = self.graphic.config.width as f32 / self.graphic.config.height as f32;
        let projection_matrix = self.camera.projection_matrix(aspect);

        // 📊 システマティックなカメラ状態ログ（デバッグ時の問題特定用）
        tracing::info!(
            "🎥 カメラ更新: mode={:?}, aspect={:.3}, target=({:.1}, {:.1}, {:.1}), distance={:.1}, bounds={:?}",
            self.camera.projection_mode,
            aspect,
            self.camera.target.x(),
            self.camera.target.y(),
            self.camera.target.z(),
            self.camera.distance,
            self.camera.orthographic_bounds
        );

        let stage = self.renderer.get_stage_mut();
        stage.update_camera(&self.graphic.queue, view_matrix, projection_matrix);
    }
}
