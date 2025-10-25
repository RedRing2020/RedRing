use crate::shader::render_3d_shader;
use crate::vertex_3d::Vertex3D;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use wgpu::{Buffer, RenderPipeline};

pub struct Renderer3D {
    pub device: Arc<wgpu::Device>,
    pub pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub vertex_count: u32,
}

pub fn create_renderer_3d(device: &Arc<wgpu::Device>, format: wgpu::TextureFormat) -> Renderer3D {
    let vertices: &[Vertex3D] = &[
        Vertex3D {
            position: [-0.5, -0.5, 0.0],
        },
        Vertex3D {
            position: [0.5, -0.5, 0.0],
        },
        Vertex3D {
            position: [0.0, 0.5, 0.0],
        },
    ];

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Renderer3D Vertex Buffer"),
        contents: bytemuck::cast_slice(vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let shader = render_3d_shader(device);

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Renderer3D Pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Renderer3D Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[Vertex3D::desc()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None, // 後で追加可能
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    let vertex_count = vertices.len() as u32;

    Renderer3D {
        device: device.clone(),
        pipeline,
        vertex_buffer,
        vertex_count,
    }
}

pub fn draw_renderer_3d<'a>(
    pass: &mut wgpu::RenderPass<'a>,
    pipeline: &'a RenderPipeline,
    vertex_buffer: &'a Buffer,
    vertex_count: u32,
) {
    pass.set_pipeline(pipeline);
    pass.set_vertex_buffer(0, vertex_buffer.slice(..));
    pass.draw(0..vertex_count, 0..1);
}
