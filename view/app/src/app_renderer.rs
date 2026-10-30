use stage::{MeshStage, RenderStage};
use wgpu::{CommandEncoder, Device, SurfaceConfiguration, TextureView};
use winit::{event::WindowEvent, window::Window};

use crate::selection_rect::SelectionRect;
use crate::selection_rect_renderer::SelectionRectRenderer;
use crate::settings_panel_renderer::SettingsPanelRenderer;
use crate::settings_panel_ui::SettingsPanelUiState;
use crate::snapshot_overlay_renderer::{SnapshotOverlayRenderer, SnapshotOverlayStyle};

pub struct RenderFrameContext<'a> {
    pub device: &'a Device,
    pub queue: &'a wgpu::Queue,
    pub viewport_width: u32,
    pub viewport_height: u32,
}

pub struct AppRenderer {
    stage: Box<dyn RenderStage>,
    selection_rect_renderer: SelectionRectRenderer,
    snapshot_overlay_renderer: SnapshotOverlayRenderer,
    settings_panel_renderer: SettingsPanelRenderer,
}

pub struct AppRendererFactory;

impl AppRendererFactory {
    /// 何も表示していない状態（空のメッシュステージ）の描画を作る
    pub fn create_empty(
        window: &Window,
        device: &Device,
        config: &SurfaceConfiguration,
    ) -> AppRenderer {
        let stage = Box::new(MeshStage::new(device, config.format));
        let selection_rect_renderer = SelectionRectRenderer::new(device, config.format);
        let snapshot_overlay_renderer = SnapshotOverlayRenderer::new(device, config.format);
        let settings_panel_renderer = SettingsPanelRenderer::new(window, device, config.format);
        AppRenderer {
            stage,
            selection_rect_renderer,
            snapshot_overlay_renderer,
            settings_panel_renderer,
        }
    }
}

impl AppRenderer {
    pub fn handle_settings_window_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        self.settings_panel_renderer
            .handle_window_event(window, event)
    }

    pub fn is_settings_using_pointer(&self) -> bool {
        self.settings_panel_renderer.is_using_pointer()
    }

    /// 設定パネルの入力欄を編集中（打った文字は入力欄に入る）
    pub fn is_settings_using_keyboard(&self) -> bool {
        self.settings_panel_renderer.is_using_keyboard()
    }

    pub fn handle_settings_mouse_motion(&mut self, delta: (f64, f64)) {
        self.settings_panel_renderer.handle_mouse_motion(delta);
    }

    pub fn prepare_settings_panel(&mut self, window: &Window, panel: &mut SettingsPanelUiState) {
        self.settings_panel_renderer.prepare(window, panel);
    }

    pub fn update_selection_rect_overlay(
        &mut self,
        queue: &wgpu::Queue,
        rect: Option<SelectionRect>,
        viewport_width: u32,
        viewport_height: u32,
    ) {
        self.selection_rect_renderer.update_selection_rect(
            queue,
            rect,
            viewport_width,
            viewport_height,
        );
    }

    pub fn update_snapshot_overlay(
        &mut self,
        queue: &wgpu::Queue,
        progress: Option<f32>,
        style: &SnapshotOverlayStyle,
        viewport_width: u32,
        viewport_height: u32,
    ) {
        self.snapshot_overlay_renderer.update_progress(
            queue,
            progress,
            style,
            viewport_width,
            viewport_height,
        );
    }

    /// ステージ切り替え（将来的なイベント駆動対応）
    pub fn set_stage(&mut self, stage: Box<dyn RenderStage>) {
        self.stage = stage;
    }

    /// 描画処理：現在のステージに委譲
    pub fn render(&mut self, encoder: &mut CommandEncoder, view: &TextureView) {
        self.stage.render(encoder, view);
    }

    /// 深度ビュー付き描画処理
    pub fn render_with_depth(
        &mut self,
        context: &RenderFrameContext<'_>,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        depth_view: &TextureView,
    ) {
        // 各ステージが depth を利用できるよう render_with_depth を呼ぶ
        self.stage.render_with_depth(encoder, view, depth_view);
        self.selection_rect_renderer.render(encoder, view);
        self.snapshot_overlay_renderer.render(encoder, view);
        self.settings_panel_renderer.render(
            context.device,
            context.queue,
            encoder,
            view,
            context.viewport_width,
            context.viewport_height,
        );
    }

    pub fn update(&mut self) {
        self.stage.update();
    }

    /// ステージへの可変参照を取得（カメラ更新など）
    pub fn get_stage_mut(&mut self) -> &mut dyn RenderStage {
        self.stage.as_mut()
    }
}
