use wgpu::{Device, Queue, Surface, SurfaceConfiguration, ShaderModuleDescriptor, ShaderSource, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, ShaderStages, BufferBindingType,
    CommandEncoderDescriptor, RenderPassDescriptor, RenderPassColorAttachment, TextureViewDescriptor, Operations, LoadOp, StoreOp,};
use crate::pipeline::{create_pipeline, PipelineBundle};

pub struct Renderer<'a> {
    pub device: Device,
    pub queue: Queue,
    pub surface: &'a Surface<'a>,
    pub config: SurfaceConfiguration,
    pub pipeline: PipelineBundle,
}

impl<'a> Renderer<'a> {
    pub fn new(
        device: Device,
        queue: Queue,
        surface: &'a Surface<'a>,
        config: SurfaceConfiguration,
    ) -> Self {

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
        let pipeline = create_pipeline(&device, &shader_module, &bind_group_layout, config.format);

        Self {
            device,
            queue,
            surface,
            config,
            pipeline,
        }

    }

    pub fn render_frame(&self) {
        // スワップチェーンのテクスチャ取得
        let frame = self.surface.get_current_texture().expect("Failed to acquire frame");
        let view = frame.texture.create_view(&TextureViewDescriptor::default());

        // コマンドエンコーダー作成
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // レンダーパスの開始（クリア処理付き）
        {
            let _render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // ここに描画処理を追加していく（例：_render_pass.set_pipeline(...)）
        }

        // コマンドをGPUに送信
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}