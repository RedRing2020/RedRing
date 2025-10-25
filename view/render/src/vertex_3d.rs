use bytemuck::{Pod, Zeroable};
use viewmodel::mesh_converter::VertexData;
use wgpu::vertex_attr_array;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex3D {
    pub position: [f32; 3],
}

impl Vertex3D {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex3D>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &vertex_attr_array![
                0 => Float32x3, // @location(0) に対応
            ],
        }
    }
}

/// メッシュレンダリング用の頂点型（位置 + 法線）
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

impl MeshVertex {
    pub fn new(position: [f32; 3], normal: [f32; 3]) -> Self {
        Self { position, normal }
    }

    /// viewmodelのVertexDataから変換
    pub fn from_vertex_data(vertex_data: &VertexData) -> Self {
        Self {
            position: vertex_data.position,
            normal: vertex_data.normal,
        }
    }

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        const ATTRIBUTES: &[wgpu::VertexAttribute] = &vertex_attr_array![
            0 => Float32x3, // position @location(0)
            1 => Float32x3, // normal @location(1)
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<MeshVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: ATTRIBUTES,
        }
    }
}

/// VertexDataのベクタをMeshVertexのベクタに変換
pub fn convert_vertex_data_to_mesh_vertices(vertex_data: &[VertexData]) -> Vec<MeshVertex> {
    vertex_data
        .iter()
        .map(MeshVertex::from_vertex_data)
        .collect()
}
