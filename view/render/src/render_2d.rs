use crate::pipeline;
use crate::shader::render_2d_shader;
use crate::vertex_2d::Vertex2D;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use wgpu::{Buffer, RenderPipeline};

pub struct Render2dResources {
    /// 頂点更新時に利用するデバイス参照
    pub device: Arc<wgpu::Device>,
    /// 2D描画用パイプライン
    pub pipeline: wgpu::RenderPipeline,
    /// 三角形頂点バッファ
    pub vertex_buffer: wgpu::Buffer,
    /// 描画頂点数
    pub vertex_count: u32,
}

/// 2Dサンプル描画リソースを作成する。
pub fn create_render_2d_resources(
    device: &Arc<wgpu::Device>,
    format: wgpu::TextureFormat,
) -> Render2dResources {
    let vertices: &[Vertex2D] = &[
        Vertex2D {
            position: [-0.5, -0.5],
        },
        Vertex2D {
            position: [0.5, -0.5],
        },
        Vertex2D {
            position: [0.0, 0.5],
        },
    ];

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Render 2D Vertex Buffer"),
        contents: bytemuck::cast_slice(vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let shader = render_2d_shader(device);

    let pipeline = pipeline::create_basic_triangle_pipeline(
        device,
        &shader,
        format,
        &[Vertex2D::desc()],
        "Render 2D",
    );

    let vertex_count = vertices.len() as u32;

    Render2dResources {
        device: device.clone(),
        pipeline,
        vertex_buffer,
        vertex_count,
    }
}

/// 2D頂点バッファを非インデックスで描画する。
pub fn draw_render_2d<'a>(
    pass: &mut wgpu::RenderPass<'a>,
    pipeline: &'a RenderPipeline,
    vertex_buffer: &'a Buffer,
    vertex_count: u32,
) {
    pipeline::draw_non_indexed(pass, pipeline, vertex_buffer, vertex_count);
}
