use wgpu::{
    CommandEncoder, TextureView,
    LoadOp, StoreOp, Operations,
    RenderPassColorAttachment, RenderPassDescriptor,
};

use crate::render_stage::RenderStage;
use render::wireframe::{WireframeResources, draw_wireframe};

pub struct OutlineStage {
    resources: WireframeResources,
}

impl OutlineStage {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let resources = render::wireframe::create_wireframe_resources(device, format);
        Self { resources }
    }
}

impl RenderStage for OutlineStage {
    fn render(&mut self, encoder: &mut CommandEncoder, view: &TextureView) {
        let color_attachment = RenderPassColorAttachment {
            view,
            resolve_target: None,
            depth_slice: None,
            ops: Operations {
                load: LoadOp::Clear(wgpu::Color {
                    r: 0.1,
                    g: 0.1,
                    b: 0.1,
                    a: 1.0,
                }),
                store: StoreOp::Store,
            },
        };

        let render_pass_desc = RenderPassDescriptor {
            label: Some("Outline Render Pass"),
            color_attachments: &[Some(color_attachment)],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        };

        let mut render_pass = encoder.begin_render_pass(&render_pass_desc);

        draw_wireframe(
            &mut render_pass,
            &self.resources.pipeline,
            &self.resources.vertex_buffer,
            self.resources.vertex_count,
        );
    }
}