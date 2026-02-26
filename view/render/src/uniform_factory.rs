use bytemuck::Pod;
use wgpu::util::DeviceExt;

/// UniformバッファとBindGroup一式を作成する共通ヘルパ。
pub fn create_uniform_binding<T: Pod>(
    device: &wgpu::Device,
    uniforms: &T,
    visibility: wgpu::ShaderStages,
    layout_label: &'static str,
    buffer_label: &'static str,
    bind_group_label: &'static str,
) -> (wgpu::BindGroupLayout, wgpu::Buffer, wgpu::BindGroup) {
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: Some(layout_label),
    });

    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(buffer_label),
        contents: bytemuck::cast_slice(&[*uniforms]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
        label: Some(bind_group_label),
    });

    (bind_group_layout, uniform_buffer, bind_group)
}
