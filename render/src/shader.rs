pub fn outline_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Outline Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/outline.wgsl").into()),
    })
}

pub fn draft_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Draft Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/draft.wgsl").into()),
    })
}