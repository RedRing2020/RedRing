use stage::RenderStage;
use wgpu::{CommandEncoder, Device, SurfaceConfiguration, TextureView};

use crate::snapshot_overlay_renderer::{SnapshotOverlayRenderer, SnapshotOverlayStyle};
use crate::stage_factory;
use crate::view_rect::SelectionRect;
use crate::view_rect_renderer::ViewRectRenderer;

pub struct AppRenderer {
    stage: Box<dyn RenderStage>,
    view_rect_renderer: ViewRectRenderer,
    snapshot_overlay_renderer: SnapshotOverlayRenderer,
}

pub struct AppRendererFactory;

impl AppRendererFactory {
    fn create_with_stage(
        device: &Device,
        config: &SurfaceConfiguration,
        stage: Box<dyn RenderStage>,
    ) -> AppRenderer {
        let view_rect_renderer = ViewRectRenderer::new(device, config.format);
        let snapshot_overlay_renderer = SnapshotOverlayRenderer::new(device, config.format);
        AppRenderer {
            stage,
            view_rect_renderer,
            snapshot_overlay_renderer,
        }
    }

    pub fn create_draft(device: &Device, config: &SurfaceConfiguration) -> AppRenderer {
        let stage = stage_factory::create_draft_stage(device, config.format);
        Self::create_with_stage(device, config, stage)
    }

    pub fn create_outline(device: &Device, config: &SurfaceConfiguration) -> AppRenderer {
        let stage = stage_factory::create_outline_stage(device, config.format);
        Self::create_with_stage(device, config, stage)
    }

    pub fn create_shading(device: &Device, config: &SurfaceConfiguration) -> AppRenderer {
        let stage = stage_factory::create_shading_stage(device, config.format);
        Self::create_with_stage(device, config, stage)
    }
}

impl AppRenderer {
    /// 初期化：Draftステージを生成
    pub fn new_draft(device: &Device, config: &SurfaceConfiguration) -> Self {
        AppRendererFactory::create_draft(device, config)
    }

    /// 初期化：Outlineステージを生成
    pub fn new_outline(device: &Device, config: &SurfaceConfiguration) -> Self {
        AppRendererFactory::create_outline(device, config)
    }

    /// 初期化：Shadingステージを生成
    pub fn new_shading(device: &Device, config: &SurfaceConfiguration) -> Self {
        AppRendererFactory::create_shading(device, config)
    }

    pub fn update_view_rect_overlay(
        &mut self,
        queue: &wgpu::Queue,
        rect: Option<SelectionRect>,
        viewport_width: u32,
        viewport_height: u32,
    ) {
        self.view_rect_renderer
            .update_rect(queue, rect, viewport_width, viewport_height);
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
        encoder: &mut CommandEncoder,
        view: &TextureView,
        depth_view: &TextureView,
    ) {
        // 各ステージが depth を利用できるよう render_with_depth を呼ぶ
        self.stage.render_with_depth(encoder, view, depth_view);
        self.view_rect_renderer.render(encoder, view);
        self.snapshot_overlay_renderer.render(encoder, view);
    }

    pub fn update(&mut self) {
        self.stage.update();
    }

    /// ステージへの可変参照を取得（カメラ更新など）
    pub fn get_stage_mut(&mut self) -> &mut dyn RenderStage {
        self.stage.as_mut()
    }
}
