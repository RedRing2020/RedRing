//! CAM工具経路のGPU描画リソース
//!
//! `viewmodel::toolpath_converter` から変換された頂点データを受け取り、
//! GPU上で工具経路を描画します。
//!
//! # 特徴
//!
//! - LineList トポロジーによる線分描画
//! - 頂点カラー対応（セグメント種別ごとの色分け）
//! - View/Projection行列による3D表示

use crate::shader;
use crate::uniform_factory;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

/// 工具経路頂点（GPU転送用）
///
/// viewmodel::toolpath_converter::Vertex3D と色情報を統合
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct ToolPathVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

impl ToolPathVertex {
    /// 頂点レイアウト記述子
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ToolPathVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // color
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// 工具経路描画用のUniform構造体
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct ToolPathUniforms {
    pub view_proj: [[f32; 4]; 4], // ビュー・プロジェクション行列
    pub model: [[f32; 4]; 4],     // モデル行列
}

impl Default for ToolPathUniforms {
    fn default() -> Self {
        Self {
            view_proj: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

/// 工具経路レンダリングリソース
pub struct ToolPathResources {
    pub render_pipeline: wgpu::RenderPipeline,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub vertex_buffer: Option<wgpu::Buffer>,
    pub vertex_count: u32,
}

impl ToolPathResources {
    /// 新しい工具経路レンダリングリソースを作成
    ///
    /// # 引数
    ///
    /// - `device`: wgpu Device
    /// - `format`: レンダーターゲットのテクスチャフォーマット
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = shader::toolpath_shader(device);

        let uniforms = ToolPathUniforms::default();
        let (bind_group_layout, uniform_buffer, bind_group) =
            uniform_factory::create_uniform_binding(
                device,
                &uniforms,
                wgpu::ShaderStages::VERTEX_FRAGMENT,
                "toolpath_bind_group_layout",
                "ToolPath Uniform Buffer",
                "toolpath_bind_group",
            );

        // Render pipeline layout
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("ToolPath Render Pipeline Layout"),
                bind_group_layouts: &[Some(&bind_group_layout)],
                immediate_size: 0,
            });

        // Render pipeline（LineList トポロジー）
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ToolPath Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[ToolPathVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // 線分なのでカリングなし
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        Self {
            render_pipeline,
            uniform_buffer,
            bind_group_layout,
            bind_group,
            vertex_buffer: None,
            vertex_count: 0,
        }
    }

    /// 頂点データを更新
    ///
    /// # 引数
    ///
    /// - `device`: wgpu Device
    /// - `vertices`: 工具経路の頂点配列
    ///
    /// # Note
    ///
    /// `vertices` は LineList トポロジー用のため、
    /// 2頂点で1線分を表現します。
    pub fn update_vertices(&mut self, device: &wgpu::Device, vertices: &[ToolPathVertex]) {
        if vertices.is_empty() {
            self.vertex_buffer = None;
            self.vertex_count = 0;
            return;
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ToolPath Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        self.vertex_buffer = Some(vertex_buffer);
        self.vertex_count = vertices.len() as u32;
    }

    /// Uniform（カメラ行列）を更新
    ///
    /// # 引数
    ///
    /// - `queue`: wgpu Queue
    /// - `uniforms`: 新しいUniform値
    pub fn update_uniforms(&self, queue: &wgpu::Queue, uniforms: &ToolPathUniforms) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[*uniforms]));
    }

    /// 描画
    ///
    /// # 引数
    ///
    /// - `render_pass`: wgpu RenderPass
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.vertex_buffer.is_none() || self.vertex_count == 0 {
            return;
        }

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));
        render_pass.draw(0..self.vertex_count, 0..1);
    }
}
