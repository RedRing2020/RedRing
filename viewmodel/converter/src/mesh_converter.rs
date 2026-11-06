//! mesh_converter - GPU用の頂点データの型（面の頂点と、線の頂点）

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

/// GPU用の線（ワイヤーフレーム）の頂点データ
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct WireframeVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl WireframeVertex {
    pub fn new(position: [f32; 3], color: [f32; 3]) -> Self {
        Self { position, color }
    }
}
