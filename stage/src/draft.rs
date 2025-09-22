// 標準ライブラリ
use std::sync::Arc;

// 外部クレート
use bytemuck;
use wgpu::{
    CommandEncoder, TextureView,
    LoadOp, StoreOp, Operations,
    RenderPassColorAttachment, RenderPassDescriptor,
};
use wgpu::util::DeviceExt;

// 自クレート（stage）
use crate::render_stage::RenderStage;

// 外部クレート（render）
use render::draft::{DraftResources, draw_draft};
use render::vertex_2d::Vertex2D;

pub struct DraftStage {
    resources: DraftResources,
    frame_count: u64,
}

impl DraftStage {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let resources = render::draft::create_draft_resources(&Arc::new(device.clone()), format);
        Self {
            resources,
            frame_count: 0,
        }
    }
}

impl RenderStage for DraftStage {
    fn render(&mut self, encoder: &mut CommandEncoder, view: &TextureView) {
        let color_attachment = RenderPassColorAttachment {
            view,
            resolve_target: None,
            depth_slice: None,
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
            label: Some("Draft Pass"),
            color_attachments: &[Some(color_attachment)],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        };

        let mut render_pass = encoder.begin_render_pass(&render_pass_desc);

        draw_draft(
            &mut render_pass,
            &self.resources.pipeline,
            &self.resources.vertex_buffer,
            self.resources.vertex_count,
        );
    }
    
    fn update(&mut self) {
        self.frame_count += 1;
        let t = self.frame_count as f32 * 0.02;

        let animated_vertices = [
            Vertex2D { position: [t.sin() * 0.5, -0.5] },
            Vertex2D { position: [0.5, t.cos() * 0.5] },
            Vertex2D { position: [0.0, 0.5] },
        ];

        let device = &self.resources.device;
        self.resources.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Animated Vertex Buffer"),
            contents: bytemuck::cast_slice(&animated_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        self.resources.vertex_count = animated_vertices.len() as u32;
    }
}