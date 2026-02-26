use crate::pipeline;
use crate::shader::render_3d_shader;
use crate::vertex_3d::Vertex3D;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use wgpu::{Buffer, RenderPipeline};

pub struct Renderer3D {
    /// 頂点更新時に利用するデバイス参照
    pub device: Arc<wgpu::Device>,
    /// 3D描画用パイプライン
    pub pipeline: wgpu::RenderPipeline,
    /// 三角形頂点バッファ
    pub vertex_buffer: wgpu::Buffer,
    /// 描画頂点数
    pub vertex_count: u32,
}

/// 3Dサンプル描画リソースを作成する。
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

    let pipeline = pipeline::create_basic_triangle_pipeline(
        device,
        &shader,
        format,
        &[Vertex3D::desc()],
        "Renderer3D",
    );

    let vertex_count = vertices.len() as u32;

    Renderer3D {
        device: device.clone(),
        pipeline,
        vertex_buffer,
        vertex_count,
    }
}

/// 3D頂点バッファを非インデックスで描画する。
pub fn draw_renderer_3d<'a>(
    pass: &mut wgpu::RenderPass<'a>,
    pipeline: &'a RenderPipeline,
    vertex_buffer: &'a Buffer,
    vertex_count: u32,
) {
    pipeline::draw_non_indexed(pass, pipeline, vertex_buffer, vertex_count);
}
