pub fn wireframe_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Wireframe Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/wireframe.wgsl").into()),
    })
}