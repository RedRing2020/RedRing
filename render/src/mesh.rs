use crate::shader;
use crate::vertex_3d::MeshVertex;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

/// メッシュレンダリング用のUniform構造体（簡略版）
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct MeshUniforms {
    pub view_proj: [[f32; 4]; 4], // ビュー・プロジェクション行列
    pub model: [[f32; 4]; 4],     // モデル行列
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
}

impl MeshResources {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = shader::mesh_shader(device);

        // Uniform bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("mesh_bind_group_layout"),
        });

        // Uniform buffer
        let uniforms = MeshUniforms::default();
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("mesh_bind_group"),
        });

        // Render pipeline layout
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Mesh Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
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
                cull_mode: None, // カリング無効（両面表示）
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None, // 深度バッファ準備後に有効化
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
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
                cull_mode: None,                       // カリング無効（両面表示）
                polygon_mode: wgpu::PolygonMode::Line, // ワイヤーフレーム
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
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
        }
    }

    /// Uniformバッファを更新
    pub fn update_uniforms(&self, queue: &wgpu::Queue, uniforms: &MeshUniforms) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[*uniforms]));
    }

    /// カメラ行列を更新
    pub fn update_camera(
        &self,
        queue: &wgpu::Queue,
        view_matrix: [[f32; 4]; 4],
        proj_matrix: [[f32; 4]; 4],
    ) {
        // ビュー・プロジェクション行列を計算
        let view_proj = multiply_matrices(proj_matrix, view_matrix);

        let uniforms = MeshUniforms {
            view_proj,
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
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

/// 4x4行列の乗算
fn multiply_matrices(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut result = [[0.0; 4]; 4];

    for i in 0..4 {
        for j in 0..4 {
            for (k, &b_elem) in b.iter().enumerate() {
                result[i][j] += a[i][k] * b_elem[j];
            }
        }
    }

    result
}
