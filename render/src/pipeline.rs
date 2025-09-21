use wgpu::{
    // パイプライン構築関連
    Device, ShaderModule, RenderPipeline, PipelineLayout, BindGroupLayout, TextureFormat,
    PipelineLayoutDescriptor, RenderPipelineDescriptor, VertexState, FragmentState,
    PrimitiveState, MultisampleState,

    // 描画処理関連
    CommandEncoder, TextureView, LoadOp, StoreOp, Operations,
    RenderPassColorAttachment, RenderPassDescriptor,
};

#[allow(dead_code)]
pub struct PipelineBundle {
    pub pipeline: RenderPipeline,
    pub layout: PipelineLayout,
}

pub fn create_pipeline(
    device: &Device,
    shader: &ShaderModule,
    bind_group_layout: &BindGroupLayout,
    format: TextureFormat,
) -> PipelineBundle {
    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Pipeline Layout"),
        bind_group_layouts: &[bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    });

    PipelineBundle {
        pipeline,
        layout: pipeline_layout,
    }
}

pub fn draw_wireframe(encoder: &mut CommandEncoder, view: &TextureView) {
    let color_attachment = RenderPassColorAttachment {
        view,
        resolve_target: None,
        depth_slice: Some(0),
        ops: Operations {
            load: LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.1, b: 0.1, a: 1.0 }),
            store: StoreOp::Store,
        },
    };

    let render_pass_desc = RenderPassDescriptor {
        label: Some("Wireframe Render Pass"),
        color_attachments: &[Some(color_attachment)],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
    };

    let _render_pass = encoder.begin_render_pass(&render_pass_desc);
}