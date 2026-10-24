//! 色の異なる複数のメッシュの描画（半透明を含む）
//!
//! 不透明なメッシュを先に描き、半透明（色のアルファ < 1）のメッシュは色を混ぜて、深度を書かずに後から描く。

use crate::mesh::MeshUniforms;
use crate::shader;
use crate::vertex_3d::MeshVertex;
use viewmodel_graphics::build_view_projection_matrix;
use wgpu::util::DeviceExt;

/// 描画する 1 つのメッシュ（頂点・頂点番号・色）
pub struct MeshListItem {
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u32>,
    pub color: [f32; 4],
}

/// GPU に置いた 1 つのメッシュ
struct GpuMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    color: [f32; 4],
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl GpuMesh {
    fn is_translucent(&self) -> bool {
        self.color[3] < 1.0
    }
}

/// 色の異なる複数のメッシュの描画リソース
pub struct MeshListResources {
    opaque_pipeline: wgpu::RenderPipeline,
    translucent_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    meshes: Vec<GpuMesh>,
}

impl MeshListResources {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = shader::mesh_shader(device);
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
            label: Some("mesh_list_bind_group_layout"),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Mesh List Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = |label: &str, blend: wgpu::BlendState, depth_write: bool| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&layout),
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
                        blend: Some(blend),
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
                    depth_write_enabled: Some(depth_write),
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
            })
        };
        Self {
            opaque_pipeline: pipeline("Mesh List Opaque Pipeline", wgpu::BlendState::REPLACE, true),
            translucent_pipeline: pipeline(
                "Mesh List Translucent Pipeline",
                // 色だけを混ぜ、描画先のアルファは変えない（ウィンドウが透けないようにする）
                wgpu::BlendState {
                    color: wgpu::BlendState::ALPHA_BLENDING.color,
                    alpha: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::Zero,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
                },
                false,
            ),
            bind_group_layout,
            meshes: Vec::new(),
        }
    }

    /// 描画するメッシュをすべて置き換える（三角形のないメッシュは描かない）
    pub fn set_meshes(&mut self, device: &wgpu::Device, items: Vec<MeshListItem>) {
        self.meshes = items
            .into_iter()
            .filter(|item| !item.vertices.is_empty() && !item.indices.is_empty())
            .map(|item| {
                let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Mesh List Uniform Buffer"),
                    contents: bytemuck::cast_slice(&[MeshUniforms {
                        base_color: item.color,
                        ..MeshUniforms::default()
                    }]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });
                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform_buffer.as_entire_binding(),
                    }],
                    label: Some("mesh_list_bind_group"),
                });
                GpuMesh {
                    vertex_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Mesh List Vertex Buffer"),
                        contents: bytemuck::cast_slice(&item.vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
                    index_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Mesh List Index Buffer"),
                        contents: bytemuck::cast_slice(&item.indices),
                        usage: wgpu::BufferUsages::INDEX,
                    }),
                    index_count: item.indices.len() as u32,
                    color: item.color,
                    uniform_buffer,
                    bind_group,
                }
            })
            .collect();
    }

    /// カメラ行列を更新する
    pub fn update_camera(
        &self,
        queue: &wgpu::Queue,
        view_matrix: [[f32; 4]; 4],
        proj_matrix: [[f32; 4]; 4],
    ) {
        let view_proj = build_view_projection_matrix(view_matrix, proj_matrix);
        for mesh in &self.meshes {
            let uniforms = MeshUniforms {
                view_proj,
                base_color: mesh.color,
                ..MeshUniforms::default()
            };
            queue.write_buffer(&mesh.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
        }
    }

    /// 不透明なメッシュ、半透明のメッシュの順に描く
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        for translucent in [false, true] {
            let pipeline = if translucent {
                &self.translucent_pipeline
            } else {
                &self.opaque_pipeline
            };
            render_pass.set_pipeline(pipeline);
            for mesh in self
                .meshes
                .iter()
                .filter(|mesh| mesh.is_translucent() == translucent)
            {
                render_pass.set_bind_group(0, &mesh.bind_group, &[]);
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
            }
        }
    }
}
