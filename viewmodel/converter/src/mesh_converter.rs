//! mesh_converter - GPU用の頂点データの型

/// GPU用頂点データ
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct VertexData {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

impl VertexData {
    pub fn new(position: [f32; 3], normal: [f32; 3]) -> Self {
        Self { position, normal }
    }
}
