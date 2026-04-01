use wgpu::{
    BindGroupLayout, Device, FragmentState, MultisampleState, PipelineLayout,
    PipelineLayoutDescriptor, PrimitiveState, RenderPipeline, RenderPipelineDescriptor,
    ShaderModule, TextureFormat, VertexState,
};

#[allow(dead_code)]
pub struct PipelineBundle {
    /// 生成されたレンダーパイプライン本体
    pub pipeline: RenderPipeline,
    /// パイプラインレイアウト
    pub layout: PipelineLayout,
}

/// 汎用パイプラインを生成するヘルパ（bind group layout 指定版）
pub fn create_pipeline(
    device: &Device,
    shader: &ShaderModule,
    bind_group_layout: &BindGroupLayout,
    format: TextureFormat,
) -> PipelineBundle {
    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Pipeline Layout"),
        bind_group_layouts: &[Some(bind_group_layout)],
        immediate_size: 0,
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
        multiview_mask: None,
        cache: None,
    });

    PipelineBundle {
        pipeline,
        layout: pipeline_layout,
    }
}

/// bind group を持たない基本的な TriangleList パイプラインを生成する。
pub fn create_basic_triangle_pipeline(
    device: &Device,
    shader: &ShaderModule,
    format: TextureFormat,
    vertex_layouts: &[wgpu::VertexBufferLayout<'_>],
    label_prefix: &str,
) -> RenderPipeline {
    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some(&format!("{} Pipeline Layout", label_prefix)),
        bind_group_layouts: &[],
        immediate_size: 0,
    });

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some(&format!("{} Pipeline", label_prefix)),
        layout: Some(&pipeline_layout),
        vertex: VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: vertex_layouts,
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
            ..Default::default()
        },
        depth_stencil: None,
        multisample: MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

/// インデックスなし描画を行う共通ヘルパ。
pub fn draw_non_indexed(
    render_pass: &mut wgpu::RenderPass<'_>,
    pipeline: &wgpu::RenderPipeline,
    vertex_buffer: &wgpu::Buffer,
    vertex_count: u32,
) {
    render_pass.set_pipeline(pipeline);
    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
    render_pass.draw(0..vertex_count, 0..1);
}

/// ワイヤーフレーム描画用ヘルパ。
pub fn draw_wireframe<'a>(
    render_pass: &mut wgpu::RenderPass<'a>,
    pipeline: &'a wgpu::RenderPipeline,
    vertex_buffer: &'a wgpu::Buffer,
    vertex_count: u32,
) {
    draw_non_indexed(render_pass, pipeline, vertex_buffer, vertex_count);
}
