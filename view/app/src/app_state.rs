use crate::app_renderer::AppRenderer;
use crate::entity_manager::EntityManager;
use crate::graphic::{init_graphic, Graphic};
use crate::mouse_input::MouseInput;
use crate::stl_loader;
use crate::view_rect::ViewRect;
use analysis::linalg::{quaternion::Quaternionf, vector::Vec3f};
use analysis::{LengthUnit, Tolerance};
use stage::{
    DraftStage, MeshStage, NurbsCurveStage, NurbsSurfaceStage, OctreeStage, OutlineStage,
    ShadingStage, ToolPathStage,
};
use std::path::Path;
use std::sync::Arc;
use viewmodel_graphics::{Camera, CameraControlSensitivity};
use winit::window::Window;

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

    cursor_position: Option<(f32, f32)>,
    last_cursor_position: Option<(f32, f32)>,
    arcball_drag_start: Option<(f32, f32)>,
    arcball_virtual_cursor: Option<(f32, f32)>,
    view_rect_drag_origin: Option<(f32, f32)>,
}

impl AppState {
    pub fn new(window: Arc<Window>) -> Self {
        let graphic = init_graphic(window.clone());
        let renderer = AppRenderer::new_draft(&graphic.device, &graphic.config);

        Self {
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
            cursor_position: None,
            last_cursor_position: None,
            arcball_drag_start: None,
            arcball_virtual_cursor: None,
            view_rect_drag_origin: None,
        }
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
        // 毎フレーム カメラ行列を更新（Stage の transform をリアルタイム反映）
        self.update_camera_uniforms();

        self.renderer.update_view_rect_overlay(
            &self.graphic.queue,
            self.active_view_rect,
            self.graphic.config.width,
            self.graphic.config.height,
        );
        self.graphic.render(&mut self.renderer);
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
        use viewmodel::octree_converter::create_sample_voxel_octree_wireframe;

        tracing::info!("VoxelOctree可視化デバッグ開始");

        // ViewModelでサンプルデータ生成（ワイヤーフレーム頂点）
        let positions = create_sample_voxel_octree_wireframe();

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
        octree_stage.set_wireframe_data(&self.graphic.device, positions);

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
    }

    /// デバッグ用：CAM工具経路可視化を表示
    pub fn load_debug_toolpath(&mut self) {
        use render::toolpath::ToolPathVertex;
        use viewmodel::toolpath_converter::{
            create_sample_toolpath, ToolPathVisualizationSettings,
        };

        tracing::info!("CAM工具経路可視化デバッグ開始");

        // サンプルToolPathを生成
        let toolpath = create_sample_toolpath();
        tracing::info!(
            "サンプル工具経路生成: approach={}, cutting_levels={}, retract={}",
            toolpath.approach_segments.len(),
            toolpath.contour_levels.len(),
            toolpath.retract_segments.len()
        );

        // 可視化設定（全種別表示）
        let settings = ToolPathVisualizationSettings::default();

        // ViewModelで頂点データに変換
        let toolpath_vertices =
            viewmodel::toolpath_converter::toolpath_to_vertices(&toolpath, &settings);

        tracing::info!(
            "頂点データ変換完了: {} 頂点",
            toolpath_vertices.vertices.len()
        );
        tracing::info!(
            "フェーズ範囲 - approach: {:?}, cutting: {:?}, retract: {:?}",
            toolpath_vertices.phase_ranges.approach,
            toolpath_vertices.phase_ranges.cutting,
            toolpath_vertices.phase_ranges.retract
        );

        // 頂点データを GPU 形式に変換（位置+色）
        // 設計: 2頂点で1線分、1線分に1色が割り当てられている
        // GPU描画: 各頂点に色が必要なので、1色を2頂点分に複製
        let mut gpu_vertices = Vec::with_capacity(toolpath_vertices.vertices.len());
        for (i, vertex) in toolpath_vertices.vertices.iter().enumerate() {
            let color_index = i / 2; // 2頂点ごとに1色
            let color = toolpath_vertices
                .colors
                .get(color_index)
                .copied()
                .unwrap_or([1.0, 1.0, 1.0, 1.0]); // フォールバック: 白色

            gpu_vertices.push(ToolPathVertex {
                position: vertex.position,
                color,
            });
        }

        tracing::info!(
            "GPU頂点データ生成: {} 頂点（{} 線分）",
            gpu_vertices.len(),
            toolpath_vertices.colors.len()
        );

        // 全頂点をログ出力（デバッグ用）
        if !gpu_vertices.is_empty() {
            tracing::info!("=== 全22頂点の座標ダンプ ===");
            for (i, v) in gpu_vertices.iter().enumerate() {
                tracing::info!(
                    "  [{}] pos=({:.1}, {:.1}, {:.1}), color={:?}",
                    i,
                    v.position[0],
                    v.position[1],
                    v.position[2],
                    v.color
                );
            }
            tracing::info!("=== ダンプ終了 ===");
        }

        // ToolPathStageを作成してデータ設定
        let mut toolpath_stage = Box::new(ToolPathStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        toolpath_stage.set_toolpath_data(&self.graphic.device, gpu_vertices);

        // カメラを原点中心に設定（デバッグ用：座標原点中心で生成したツールパスに対応）
        self.camera.target = Vec3f::new(0.0, 0.0, 7.5); // ツールパスZ範囲の中央（0～15の中間）
        self.camera.distance = 150.0; // クリッピングを避けるため適度な距離
        self.camera.zoom = 1.0;

        // 初期表示方向: Z正方向から負の方向（XY平面を真上から見下ろす）
        // identity() のままで forward=(0,0,-1)、camera_pos計算修正により正しく配置される
        self.camera.rotation = Quaternionf::identity();

        self.camera
            .set_projection_mode(viewmodel_graphics::camera::ProjectionMode::Orthographic);

        // 表示範囲を明示的に指定（±50、つまり 100mm × 100mm の正方形、マージン付き）
        self.camera
            .set_orthographic_bounds(-50.0, 50.0, -50.0, 50.0);

        tracing::info!(
            "カメラ設定: target=(0, 0, 7.5), distance={}, display_bounds=(-50～50, -50～50), 平行投影・Z正方向から負の方向",
            self.camera.distance
        );

        // カメラ状態を詳細ログ出力（デバッグ用）
        let view_matrix = self.camera.view_matrix();
        let proj_matrix = self.camera.projection_matrix(
            self.graphic.config.width as f32 / self.graphic.config.height as f32,
        );
        tracing::info!("📊 View行列: {:?}", view_matrix);
        tracing::info!("📊 Projection行列: {:?}", proj_matrix);

        self.renderer.set_stage(toolpath_stage);

        // カメラユニフォーム更新
        self.update_camera_uniforms();

        tracing::info!("CAM工具経路可視化デバッグ完了");
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
                    tracing::info!("o: Octree表示");
                    tracing::info!("p: ToolPath表示");
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
                    // デバッグ: Octree可視化表示
                    self.load_debug_octree();
                }
                "p" => {
                    // デバッグ: CAM工具経路可視化表示
                    self.load_debug_toolpath();
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

    /// マウスホイールを処理（Ctrl+ホイールでズーム）
    pub fn handle_mouse_wheel(&mut self, delta: winit::event::MouseScrollDelta) {
        if !self.mouse_input.ctrl_pressed {
            return;
        }

        let sensitivity = self.camera.control_sensitivity();
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

    /// Mainframe からカメラ操作感度を一括変更
    pub fn set_camera_control_sensitivity(&mut self, sensitivity: CameraControlSensitivity) {
        self.camera.set_control_sensitivity(sensitivity);
    }

    /// 現在のカメラ操作感度を取得
    pub fn camera_control_sensitivity(&self) -> CameraControlSensitivity {
        self.camera.control_sensitivity()
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
