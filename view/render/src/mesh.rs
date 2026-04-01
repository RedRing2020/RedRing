use crate::shader;
use crate::uniform_factory;
use crate::vertex_3d::MeshVertex;
use bytemuck::{Pod, Zeroable};
use viewmodel_graphics::build_view_projection_matrix;
use wgpu::util::DeviceExt;

/// メッシュレンダリング用のUniform構造体（簡略版）
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct MeshUniforms {
    pub view_proj: [[f32; 4]; 4], // ビュー・プロジェクション行列
    pub model: [[f32; 4]; 4],     // モデル行列
    pub base_color: [f32; 4],     // メッシュ基本色
}

impl Default for MeshUniforms {
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
            base_color: [1.0, 0.5, 0.2, 1.0],
        }
    }
}

/// メッシュレンダリングリソース
pub struct MeshResources {
    pub render_pipeline: wgpu::RenderPipeline,
    pub wireframe_pipeline: wgpu::RenderPipeline,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub vertex_buffer: Option<wgpu::Buffer>,
    pub index_buffer: Option<wgpu::Buffer>,
    pub index_count: u32,
    pub wireframe_mode: bool,
    pub base_color: [f32; 4],
}

impl MeshResources {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = shader::mesh_shader(device);

        let uniforms = MeshUniforms::default();
        let (bind_group_layout, uniform_buffer, bind_group) =
            uniform_factory::create_uniform_binding(
                device,
                &uniforms,
                wgpu::ShaderStages::VERTEX_FRAGMENT,
                "mesh_bind_group_layout",
                "Mesh Uniform Buffer",
                "mesh_bind_group",
            );

        // Render pipeline layout
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Mesh Render Pipeline Layout"),
                bind_group_layouts: &[Some(&bind_group_layout)],
                immediate_size: 0,
            });

        // Render pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Mesh Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[MeshVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
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

        // ワイヤーフレーム用のパイプライン（ポリゴンモードをLineに変更）
        let wireframe_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Mesh Wireframe Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[MeshVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Line, // ワイヤーフレーム
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
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
            wireframe_pipeline,
            uniform_buffer,
            bind_group_layout,
            bind_group,
            vertex_buffer: None,
            index_buffer: None,
            index_count: 0,
            wireframe_mode: false,
            base_color: [1.0, 0.5, 0.2, 1.0],
        }
    }

    /// Uniformバッファを更新
    pub fn update_uniforms(&self, queue: &wgpu::Queue, uniforms: &MeshUniforms) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[*uniforms]));
    }

    /// カメラ行列を更新
    pub fn update_camera(
        &mut self,
        queue: &wgpu::Queue,
        view_matrix: [[f32; 4]; 4],
        proj_matrix: [[f32; 4]; 4],
    ) {
        let view_proj = build_view_projection_matrix(view_matrix, proj_matrix);

        let uniforms = MeshUniforms {
            view_proj,
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            base_color: self.base_color,
        };

        self.update_uniforms(queue, &uniforms);
    }

    /// メッシュデータを更新
    pub fn update_mesh_data(
        &mut self,
        device: &wgpu::Device,
        vertices: &[MeshVertex],
        indices: &[u32],
    ) {
        // 頂点バッファを作成
        if !vertices.is_empty() {
            self.vertex_buffer = Some(device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Mesh Vertex Buffer"),
                    contents: bytemuck::cast_slice(vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                },
            ));
        }

        // インデックスバッファを作成
        if !indices.is_empty() {
            self.index_buffer = Some(device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Mesh Index Buffer"),
                    contents: bytemuck::cast_slice(indices),
                    usage: wgpu::BufferUsages::INDEX,
                },
            ));
            self.index_count = indices.len() as u32;
        }
    }

    /// ワイヤーフレームモードを切り替え
    pub fn toggle_wireframe(&mut self) {
        self.wireframe_mode = !self.wireframe_mode;
    }

    /// ワイヤーフレームモードかどうか
    pub fn is_wireframe(&self) -> bool {
        self.wireframe_mode
    }

    /// シェーディング時のメッシュ基本色を設定
    pub fn set_base_color(&mut self, base_color: [f32; 4]) {
        self.base_color = base_color;
    }

    /// メッシュをレンダリング
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if let (Some(vertex_buffer), Some(index_buffer)) = (&self.vertex_buffer, &self.index_buffer)
        {
            // ワイヤーフレームモードに応じてパイプラインを選択
            let pipeline = if self.wireframe_mode {
                &self.wireframe_pipeline
            } else {
                &self.render_pipeline
            };

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.index_count, 0, 0..1);
        }
    }
}
