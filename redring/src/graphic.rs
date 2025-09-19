use wgpu::{
    Device, Queue, Surface, SurfaceConfiguration, ShaderModule, RenderPipeline,
    RenderPassDescriptor, RenderPassColorAttachment, Operations, LoadOp, StoreOp,
    Color, ColorTargetState, BlendState, ColorWrites, PipelineLayoutDescriptor,
    RenderPipelineDescriptor, VertexState, FragmentState, PrimitiveState,
    MultisampleState,
};

pub struct Graphic<'a> {
    pub device: Device,
    pub queue: Queue,
    pub surface: &'a Surface<'a>,
    pub config: SurfaceConfiguration,
    pub pipeline: RenderPipeline,
}

impl<'a> Graphic<'a> {
    pub fn new(
        device: Device,
        queue: Queue,
        surface: &'a Surface<'a>,
        config: SurfaceConfiguration,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: Default::default(),
        });

        Self {
            device,
            queue,
            surface,
            config,
            pipeline,
        }
    }

    pub fn render_frame(&self) {
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(e) => {
                eprintln!("Failed to acquire frame: {:?}", e);
                return;
            }
        };

        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}