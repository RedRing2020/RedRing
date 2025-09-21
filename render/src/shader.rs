pub fn wireframe_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Wireframe Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/wireframe.wgsl").into()),
    })
}

pub fn render_2d_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("2D Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/render_2d.wgsl").into()),
    })
}