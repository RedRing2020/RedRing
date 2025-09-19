use wgpu::{Device, Queue, SurfaceConfiguration, ShaderModuleDescriptor, ShaderSource, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, ShaderStages, BufferBindingType};
use crate::pipeline::{create_pipeline, PipelineBundle};

pub struct Renderer {
    pub pipeline: PipelineBundle,
}

impl Renderer {
    pub fn new(device: &Device, _queue: &Queue, config: &SurfaceConfiguration) -> Self {
        // WGSLシェーダーの読み込み（埋め込み例）
        let shader_source = ShaderSource::Wgsl(include_str!("shader.wgsl").into());
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Main Shader"),
            source: shader_source,
        });

        // BindGroupLayout の作成（必要に応じて調整）
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Main Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // パイプラインの作成
        let pipeline = create_pipeline(device, &shader_module, &bind_group_layout, config.format);

        Self { pipeline }
    }

    pub fn render_frame(&self) {
        // ここに描画処理を書く（仮の例）
        println!("Rendering frame...");

    }
}