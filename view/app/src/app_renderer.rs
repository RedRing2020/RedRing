use stage::{DraftStage, OutlineStage, RenderStage, ShadingStage};
use wgpu::{CommandEncoder, Device, SurfaceConfiguration, TextureView};

use crate::snapshot_overlay_renderer::{SnapshotOverlayRenderer, SnapshotOverlayStyle};
use crate::view_rect::ViewRect;
use crate::view_rect_renderer::ViewRectRenderer;

pub struct AppRenderer {
    stage: Box<dyn RenderStage>,
    view_rect_renderer: ViewRectRenderer,
    snapshot_overlay_renderer: SnapshotOverlayRenderer,
}

impl AppRenderer {
    /// 初期化：Draftステージを生成
    pub fn new_draft(device: &Device, config: &SurfaceConfiguration) -> Self {
        let stage = Box::new(DraftStage::new(device, config.format));
        let view_rect_renderer = ViewRectRenderer::new(device, config.format);
        let snapshot_overlay_renderer = SnapshotOverlayRenderer::new(device, config.format);
        Self {
            stage,
            view_rect_renderer,
            snapshot_overlay_renderer,
        }
    }

    /// 初期化：Outlineステージを生成
    pub fn new_outline(device: &Device, config: &SurfaceConfiguration) -> Self {
        let stage = Box::new(OutlineStage::new(device, config.format));
        let view_rect_renderer = ViewRectRenderer::new(device, config.format);
        let snapshot_overlay_renderer = SnapshotOverlayRenderer::new(device, config.format);
        Self {
            stage,
            view_rect_renderer,
            snapshot_overlay_renderer,
        }
    }

    /// 初期化：Shadingステージを生成
    pub fn new_shading(device: &Device, config: &SurfaceConfiguration) -> Self {
        let stage = Box::new(ShadingStage::new(device, config.format));
        let view_rect_renderer = ViewRectRenderer::new(device, config.format);
        let snapshot_overlay_renderer = SnapshotOverlayRenderer::new(device, config.format);
        Self {
            stage,
            view_rect_renderer,
            snapshot_overlay_renderer,
        }
    }

    pub fn update_view_rect_overlay(
        &mut self,
        queue: &wgpu::Queue,
        rect: Option<ViewRect>,
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
