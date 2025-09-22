pub fn render_2d_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Render 2D Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/render_2d.wgsl").into()),
    })
}

pub fn wireframe_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Wireframe Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/Wireframe.wgsl").into()),
    })
}
use wgpu::Device;

pub fn render_3d_shader(device: &Device) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Render 3D Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/render_3d.wgsl").into()),
    })
}