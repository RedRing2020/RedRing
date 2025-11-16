use std::sync::Arc;

use wgpu::{
    CommandEncoder, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, StoreOp,
    TextureView,
};
//use wgpu::util::DeviceExt;

use crate::render_stage::RenderStage;

use render::render_3d::{create_renderer_3d, draw_renderer_3d, Renderer3D};

pub struct ShadingStage {
    renderer: Renderer3D,
    frame_count: u64,
}

impl ShadingStage {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let renderer = create_renderer_3d(&Arc::new(device.clone()), format);
        Self {
            renderer,
            frame_count: 0,
        }
    }
}

impl RenderStage for ShadingStage {
    fn render(&mut self, encoder: &mut CommandEncoder, view: &TextureView) {
        let color_attachment = RenderPassColorAttachment {
            view,
            resolve_target: None,
            depth_slice: None,
            ops: Operations {
                load: LoadOp::Clear(wgpu::Color {
                    r: 0.05,
                    g: 0.05,
                    b: 0.05,
                    a: 1.0,
                }),
                store: StoreOp::Store,
            },
        };

        let render_pass_desc = RenderPassDescriptor {
            label: Some("Shading Pass"),
            color_attachments: &[Some(color_attachment)],
            depth_stencil_attachment: None, // 後で追加可能
            occlusion_query_set: None,
            timestamp_writes: None,
        };

        let mut render_pass = encoder.begin_render_pass(&render_pass_desc);

        draw_renderer_3d(
            &mut render_pass,
            &self.renderer.pipeline,
            &self.renderer.vertex_buffer,
            self.renderer.vertex_count,
        );
    }

    fn update(&mut self) {
        self.frame_count += 1;
        // 今後: アニメーションやカメラ制御などを追加可能
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
