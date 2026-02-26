use crate::pipeline;
use crate::shader::render_3d_shader;
use crate::vertex_3d::Vertex3D;
use wgpu::util::DeviceExt;
use wgpu::{Buffer, RenderPipeline};

pub struct Render3dResources {
    /// 3D描画用パイプライン
    pub pipeline: wgpu::RenderPipeline,
    /// 三角形頂点バッファ
    pub vertex_buffer: wgpu::Buffer,
    /// 描画頂点数
    pub vertex_count: u32,
}

/// 3Dサンプル描画リソースを作成する。
pub fn create_render_3d_resources(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
) -> Render3dResources {
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
        label: Some("Render 3D Vertex Buffer"),
        contents: bytemuck::cast_slice(vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let shader = render_3d_shader(device);

    let pipeline = pipeline::create_basic_triangle_pipeline(
        device,
        &shader,
        format,
        &[Vertex3D::desc()],
        "Render 3D",
    );

    let vertex_count = vertices.len() as u32;

    Render3dResources {
        pipeline,
        vertex_buffer,
        vertex_count,
    }
}

/// 3D頂点バッファを非インデックスで描画する。
pub fn draw_render_3d<'a>(
    pass: &mut wgpu::RenderPass<'a>,
    pipeline: &'a RenderPipeline,
    vertex_buffer: &'a Buffer,
    vertex_count: u32,
) {
    pipeline::draw_non_indexed(pass, pipeline, vertex_buffer, vertex_count);
}
