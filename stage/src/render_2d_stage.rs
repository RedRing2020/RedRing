use wgpu::{
    CommandEncoder, TextureView,
    LoadOp, StoreOp, Operations,
    RenderPassColorAttachment, RenderPassDescriptor,
};

use crate::render_stage::RenderStage;
use render::render_2d::{Render2DResources, draw_render_2d};

/// 2D描画ステージ（高レイヤー抽象）
pub struct Render2DStage {
    resources: Render2DResources,
}

impl Render2DStage {
    /// 初期化：低レイヤーのリソースを構築して保持
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let resources = render::render_2d::create_render_2d_resources(device, format);
        Self { resources }
    }
}

impl RenderStage for Render2DStage {
    /// 描画処理：RenderPass を構築し、描画命令を発行
    fn render(&mut self, encoder: &mut CommandEncoder, view: &TextureView) {
        let color_attachment = RenderPassColorAttachment {
            view,
            resolve_target: None,
            depth_slice: Some(0),
            ops: Operations {
                load: LoadOp::Clear(wgpu::Color {
                    r: 0.2,
                    g: 0.2,
                    b: 0.2,
                    a: 1.0,
                }),
                store: StoreOp::Store,
            },
        };

        let render_pass_desc = RenderPassDescriptor {
            label: Some("2D Render Pass"),
            color_attachments: &[Some(color_attachment)],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        };

        let mut render_pass = encoder.begin_render_pass(&render_pass_desc);

        draw_render_2d(
            &mut render_pass,
            &self.resources.pipeline,
            &self.resources.vertex_buffer,
            self.resources.vertex_count,
        );
    }
    
    fn update(&mut self) {
        // 2Dステージでの状態更新（例：頂点アニメーション）
    }
}