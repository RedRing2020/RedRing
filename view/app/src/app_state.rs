use crate::app_renderer::AppRenderer;
use crate::camera::Camera;
use crate::graphic::{Graphic, init_graphic};
use crate::mouse_input::MouseInput;
use crate::stl_loader;
use analysis::linalg::vector::Vec3f;
use stage::{DraftStage, MeshStage, OutlineStage, ShadingStage};
use std::path::Path;
use std::sync::Arc;
use winit::window::Window;

/// 4x4行列による3D点の変換
fn transform_point(matrix: &[[f32; 4]; 4], point: Vec3f) -> Vec3f {
    let x = matrix[0][0] * point.x()
        + matrix[0][1] * point.y()
        + matrix[0][2] * point.z()
        + matrix[0][3];
    let y = matrix[1][0] * point.x()
        + matrix[1][1] * point.y()
        + matrix[1][2] * point.z()
        + matrix[1][3];
    let z = matrix[2][0] * point.x()
        + matrix[2][1] * point.y()
        + matrix[2][2] * point.z()
        + matrix[2][3];
    let w = matrix[3][0] * point.x()
        + matrix[3][1] * point.y()
        + matrix[3][2] * point.z()
        + matrix[3][3];

    // 同次座標系から3D座標に変換
    if w != 0.0 {
        Vec3f::new(x / w, y / w, z / w)
    } else {
        Vec3f::new(x, y, z)
    }
}

pub struct AppState {
    pub window: Arc<Window>,
    pub graphic: Graphic,
    pub renderer: AppRenderer,
    pub camera: Camera,
    pub mouse_input: MouseInput,
    // モデル用境界ボックス（ワールド座標系）
    pub model_bounds: Option<(Vec3f, Vec3f)>,
    // ビュー用境界ボックス（カメラ座標系で動的更新）
    pub view_bounds: Option<(Vec3f, Vec3f)>,
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
            model_bounds: None,
            view_bounds: None,
        }
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.graphic.config.width = size.width;
        self.graphic.config.height = size.height;
        self.graphic
            .surface
            .configure(&self.graphic.device, &self.graphic.config);
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

    /// STLファイルを読み込んでメッシュステージに設定
    pub fn load_stl_file(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("STLファイル読み込み開始: {:?}", path);

        // STLファイルを読み込み、レンダリング用データに変換
        let (vertices, indices, bounds) = stl_loader::load_stl_for_rendering(path)?;
        let (min_bounds, max_bounds) = bounds;

        // カメラをメッシュに適応
        use analysis::linalg::vector::Vec3f;
        let min_vec = Vec3f::new(min_bounds[0], min_bounds[1], min_bounds[2]);
        let max_vec = Vec3f::new(max_bounds[0], max_bounds[1], max_bounds[2]);

        // モデル境界ボックスを保存
        self.model_bounds = Some((min_vec, max_vec));

        self.camera.fit_to_mesh(min_vec, max_vec);

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

        // サンプルSTLファイルを作成して境界ボックス付きで読み込み
        let (vertices, indices, bounds) = stl_loader::create_sample_stl_with_bounds(&sample_path)?;
        let (min_bounds, max_bounds) = bounds;

        // カメラをメッシュに適応
        use analysis::linalg::vector::Vec3f;
        let min_vec = Vec3f::new(min_bounds[0], min_bounds[1], min_bounds[2]);
        let max_vec = Vec3f::new(max_bounds[0], max_bounds[1], max_bounds[2]);

        // モデル境界ボックスを保存
        self.model_bounds = Some((min_vec, max_vec));

        self.camera.fit_to_mesh(min_vec, max_vec);

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

    /// カメラをリセット
    pub fn reset_camera(&mut self) {
        self.camera.reset();
        self.update_camera_uniforms();
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

        // ステージがMeshStageの場合にカメラを更新
        if let Some(mesh_stage) = self
            .renderer
            .get_stage_mut()
            .as_any_mut()
            .downcast_mut::<MeshStage>()
        {
            mesh_stage.update_camera(&self.graphic.queue, view_matrix, projection_matrix);
        }

        // カメラが変更されたらビュー用境界ボックスを更新
        self.update_view_bounds();
    }

    /// ビュー用境界ボックスを更新（カメラ座標系での境界ボックス計算）
    fn update_view_bounds(&mut self) {
        if let Some((min_world, max_world)) = self.model_bounds {
            // ワールド座標系の境界ボックスの8つの頂点を計算
            let world_corners = [
                Vec3f::new(min_world.x(), min_world.y(), min_world.z()),
                Vec3f::new(max_world.x(), min_world.y(), min_world.z()),
                Vec3f::new(min_world.x(), max_world.y(), min_world.z()),
                Vec3f::new(max_world.x(), max_world.y(), min_world.z()),
                Vec3f::new(min_world.x(), min_world.y(), max_world.z()),
                Vec3f::new(max_world.x(), min_world.y(), max_world.z()),
                Vec3f::new(min_world.x(), max_world.y(), max_world.z()),
                Vec3f::new(max_world.x(), max_world.y(), max_world.z()),
            ];

            // ビュー変換行列を取得
            let view_matrix = self.camera.view_matrix();

            // 各頂点をカメラ座標系に変換し、包含ボックスを計算
            let mut min_view = Vec3f::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
            let mut max_view = Vec3f::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);

            for corner in &world_corners {
                // ワールド座標をカメラ座標に変換
                let view_pos = transform_point(&view_matrix, *corner);

                min_view = Vec3f::new(
                    min_view.x().min(view_pos.x()),
                    min_view.y().min(view_pos.y()),
                    min_view.z().min(view_pos.z()),
                );
                max_view = Vec3f::new(
                    max_view.x().max(view_pos.x()),
                    max_view.y().max(view_pos.y()),
                    max_view.z().max(view_pos.z()),
                );
            }

            self.view_bounds = Some((min_view, max_view));
        } else {
            self.view_bounds = None;
        }
    }

    /// ビュー用境界ボックスに基づいてカメラをフィット（fキー用）
    pub fn fit_camera_to_view_bounds(&mut self) {
        if let Some((min_bounds, max_bounds)) = self.model_bounds {
            // モデル境界ボックスを使用してカメラをフィット
            self.camera.fit_to_mesh(min_bounds, max_bounds);
            self.update_camera_uniforms();
            tracing::info!("カメラをモデル境界ボックスにフィット");
        } else {
            tracing::warn!("モデル境界ボックスが設定されていません");
        }
    }

    /// 境界ボックス状態をログ出力（dキー用デバッグ機能の拡張）
    pub fn log_bounds_state(&self) {
        if let Some((min_model, max_model)) = self.model_bounds {
            tracing::info!("モデル境界ボックス（ワールド座標系）:");
            tracing::info!(
                "  最小: [{:.3}, {:.3}, {:.3}]",
                min_model.x(),
                min_model.y(),
                min_model.z()
            );
            tracing::info!(
                "  最大: [{:.3}, {:.3}, {:.3}]",
                max_model.x(),
                max_model.y(),
                max_model.z()
            );

            let size = max_model - min_model;
            tracing::info!(
                "  サイズ: [{:.3}, {:.3}, {:.3}]",
                size.x(),
                size.y(),
                size.z()
            );
        } else {
            tracing::info!("モデル境界ボックス: 未設定");
        }

        if let Some((min_view, max_view)) = self.view_bounds {
            tracing::info!("ビュー境界ボックス（カメラ座標系）:");
            tracing::info!(
                "  最小: [{:.3}, {:.3}, {:.3}]",
                min_view.x(),
                min_view.y(),
                min_view.z()
            );
            tracing::info!(
                "  最大: [{:.3}, {:.3}, {:.3}]",
                max_view.x(),
                max_view.y(),
                max_view.z()
            );

            let size = max_view - min_view;
            tracing::info!(
                "  サイズ: [{:.3}, {:.3}, {:.3}]",
                size.x(),
                size.y(),
                size.z()
            );
        } else {
            tracing::info!("ビュー境界ボックス: 未設定");
        }
    }
}
