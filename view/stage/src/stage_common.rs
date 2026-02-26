use render::nurbs_eval::NurbsEvalUniforms;
use render::toolpath::ToolPathUniforms;
use viewmodel_graphics::build_view_projection_matrix;

pub const IDENTITY_MODEL_MATRIX: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

pub fn create_depth_texture(
    device: &wgpu::Device,
    size: (u32, u32),
    label: &'static str,
) -> (wgpu::Texture, wgpu::TextureView) {
    let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
    (depth_texture, depth_view)
}

pub fn build_toolpath_uniforms_from_view_proj(view_proj: [[f32; 4]; 4]) -> ToolPathUniforms {
    ToolPathUniforms {
        view_proj,
        model: IDENTITY_MODEL_MATRIX,
    }
}

pub fn build_toolpath_uniforms(
    view_matrix: [[f32; 4]; 4],
    proj_matrix: [[f32; 4]; 4],
) -> ToolPathUniforms {
    let view_proj = build_view_projection_matrix(view_matrix, proj_matrix);
    build_toolpath_uniforms_from_view_proj(view_proj)
}

pub fn build_nurbs_eval_uniforms(
    view_matrix: [[f32; 4]; 4],
    proj_matrix: [[f32; 4]; 4],
) -> NurbsEvalUniforms {
    let view_proj = build_view_projection_matrix(view_matrix, proj_matrix);
    NurbsEvalUniforms {
        view_proj,
        model: IDENTITY_MODEL_MATRIX,
    }
}
