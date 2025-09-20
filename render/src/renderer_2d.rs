use wgpu::{Device, Queue, Surface, SurfaceConfiguration, RenderPipeline};
use wgpu::{ShaderModuleDescriptor, ShaderSource, PipelineLayoutDescriptor, RenderPipelineDescriptor};
use wgpu::{VertexState, FragmentState, ColorTargetState, ColorWrites, BlendState, Operations, LoadOp, StoreOp};
use wgpu::{RenderPassDescriptor, RenderPassColorAttachment, CommandEncoderDescriptor, TextureViewDescriptor};
use wgpu::PipelineCompilationOptions;

const SHADER_SOURCE: &str = include_str!("../shaders/render_2d.wgsl");

pub struct Renderer2D {
    pipeline: RenderPipeline,
}

impl Renderer2D {
    pub fn new(device: &Device, config: &SurfaceConfiguration) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("2D Shader"),
            source: ShaderSource::Wgsl(SHADER_SOURCE.into()),
        });

        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("2D Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("2D Pipeline"),
            layout: Some(&layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self { pipeline }
    }

    pub fn render(
        &self,
        device: &Device,
        queue: &Queue,
        surface: &Surface,
        _config: &SurfaceConfiguration,
    ) {
        let frame = surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("2D Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("2D Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: Some(0),
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.draw(0..3, 0..1);
        }

        queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}