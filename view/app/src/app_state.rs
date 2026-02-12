use crate::app_renderer::AppRenderer;
use crate::graphic::{init_graphic, Graphic};
use crate::mouse_input::MouseInput;
use crate::stl_loader;
use analysis::linalg::{quaternion::Quaternionf, vector::Vec3f};
use analysis::{LengthUnit, Tolerance};
use stage::{DraftStage, MeshStage, OctreeStage, OutlineStage, ShadingStage};
use std::path::Path;
use std::sync::Arc;
use viewmodel_graphics::Camera;
use winit::window::Window;

pub struct AppState {
    pub window: Arc<Window>,
    pub graphic: Graphic,
    pub renderer: AppRenderer,
    pub camera: Camera,
    pub mouse_input: MouseInput,

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
            // CAD標準設定
            unit_system: LengthUnit::Millimeter,
            display_tolerance: Tolerance::default(), // 0.01mm
        }
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

        // リサイズ時にカメラのアスペクト比も更新
        self.update_camera_uniforms();
    }

    pub fn render(&mut self) {
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

        tracing::info!("ワイヤーフレーム頂点数: {}", positions.len());

        // OctreeStageを作成してデータ設定
        let mut octree_stage = Box::new(OctreeStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        octree_stage.set_wireframe_data(&self.graphic.device, positions);

        // カメラをワークピース中心に設定（100x100x50mmのワークピース）
        // 平行投影で真上から見る視点
        self.camera.target = Vec3f::new(50.0, 50.0, 25.0);
        self.camera.distance = 200.0;
        self.camera.zoom = 1.0; // zoom=1でdistance=200が描画範囲（±100mm）
        self.camera.rotation = Quaternionf::identity();
        self.camera
            .set_projection_mode(viewmodel_graphics::camera::ProjectionMode::Orthographic);

        tracing::info!(
            "カメラ設定: target=(50, 50, 25), distance=200.0, zoom=1.0, 平行投影・真上視点"
        );

        self.renderer.set_stage(octree_stage);

        // カメラユニフォーム更新
        self.update_camera_uniforms();

        tracing::info!("VoxelOctree可視化デバッグ完了");
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

        tracing::info!("デバッグ形状: LineSegment3D表示（SVGから）");

        let svg_path = Path::new("tests/fixtures/shapes/line.svg");
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
    pub fn load_debug_nurbs(&mut self) {
        use std::path::Path;

        let tolerance = self.tolerance_in_current_unit();
        tracing::info!("デバッグ形状: NurbsCurve3D表示（SVGから）");
        tracing::info!(
            "表示トレランス: {:.6} (単位系: {:?})",
            tolerance,
            self.unit_system
        );

        let svg_path = Path::new("tests/fixtures/shapes/nurbs_curve.svg");
        match crate::svg_loader::load_svg_for_rendering(svg_path, Some(tolerance)) {
            Ok(vertices) => {
                tracing::info!(
                    "SVG読み込み成功: {} 頂点（テッセレーション済み）",
                    vertices.len()
                );

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
        // ステージがMeshStageの場合にワイヤーフレームを切り替え
        if let Some(mesh_stage) = self
            .renderer
            .get_stage_mut()
            .as_any_mut()
            .downcast_mut::<MeshStage>()
        {
            mesh_stage.toggle_wireframe();
            let mode = if mesh_stage.is_wireframe() {
                "ワイヤーフレーム"
            } else {
                "ソリッド"
            };
            tracing::info!("表示モードを{}に切り替え", mode);
        }
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
                    tracing::info!("=== デバッグ形状表示 ===");
                    tracing::info!("s: クリップ空間正方形（単位行列テスト）");
                    tracing::info!("l: LineSegment3D表示");
                    tracing::info!("c: Circle3D表示");
                    tracing::info!("t: Triangle3D表示 (shift+t推奨)");
                    tracing::info!("a: Arc3D表示");
                    tracing::info!("=== その他 ===");
                    tracing::info!(
                        "マウス操作: 左ドラッグ=回転, 中ドラッグ=パン, 右ドラッグ=ズーム"
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
    }

    /// マウス移動を処理
    pub fn handle_mouse_motion(&mut self, delta: (f64, f64)) {
        use crate::mouse_input::MouseOperation;

        let (delta_x, delta_y) = (delta.0 as f32, delta.1 as f32);

        match self.mouse_input.operation {
            MouseOperation::Rotate => {
                self.camera.rotate(delta_x, delta_y);
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

        tracing::debug!("カメラ行列更新: aspect={:.2}", aspect);

        let stage = self.renderer.get_stage_mut();

        // ステージがMeshStageの場合にカメラを更新（メッシュと線の両方）
        if let Some(mesh_stage) = stage.as_any_mut().downcast_mut::<MeshStage>() {
            mesh_stage.update_camera(&self.graphic.queue, view_matrix, projection_matrix);
        }
        // ステージがOctreeStageの場合にカメラを更新
        else if let Some(octree_stage) = stage.as_any_mut().downcast_mut::<OctreeStage>() {
            octree_stage.update_camera(&self.graphic.queue, view_matrix, projection_matrix);
        }
    }
}
