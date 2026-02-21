use crate::shader;
use crate::vertex_3d::MeshVertex;
use bytemuck::{Pod, Zeroable};
use logging_foundation::{frame_interval_from_env, should_log_every_n_frames};
use std::sync::atomic::{AtomicU64, Ordering};
use viewmodel_graphics::build_view_projection_matrix;
use wgpu::util::DeviceExt;

static LINE_CAMERA_LOG_COUNTER: AtomicU64 = AtomicU64::new(0);
static LINE_RENDER_LOG_COUNTER: AtomicU64 = AtomicU64::new(0);

fn frame_log_interval() -> u64 {
    frame_interval_from_env("REDRING_LOG_FRAME_INTERVAL", 120)
}

fn should_log(counter: &AtomicU64) -> bool {
    let frame = counter.fetch_add(1, Ordering::Relaxed) + 1;
    should_log_every_n_frames(frame, frame_log_interval())
}

/// ライン描画用のUniform構造体
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct LineUniforms {
    pub view_proj: [[f32; 4]; 4], // ビュー・プロジェクション行列
    pub model: [[f32; 4]; 4],     // モデル行列
}

impl Default for LineUniforms {
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

/// 線分レンダリングリソース
pub struct LineResources {
    pub render_pipeline: wgpu::RenderPipeline,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub vertex_buffer: Option<wgpu::Buffer>,
    pub vertex_count: u32,
}

impl LineResources {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = shader::line_shader(device);

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
            label: Some("line_bind_group_layout"),
        });

        // Uniform buffer
        let uniforms = LineUniforms::default();
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Line Uniform Buffer"),
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
            label: Some("line_bind_group"),
        });

        // Render pipeline layout
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Line Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        // Render pipeline（LineList トポロジー）
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Line Render Pipeline"),
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
                topology: wgpu::PrimitiveTopology::LineList, // 独立した線分として描画
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
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

        Self {
            render_pipeline,
            uniform_buffer,
            bind_group_layout,
            bind_group,
            vertex_buffer: None,
            vertex_count: 0,
        }
    }

    /// Uniformバッファを更新
    pub fn update_uniforms(&self, queue: &wgpu::Queue, uniforms: &LineUniforms) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[*uniforms]));
    }

    /// カメラ行列を更新
    pub fn update_camera(
        &self,
        queue: &wgpu::Queue,
        view_matrix: [[f32; 4]; 4],
        proj_matrix: [[f32; 4]; 4],
    ) {
        let should_dump = should_log(&LINE_CAMERA_LOG_COUNTER);
        if should_dump {
            tracing::trace!("LineResources.update_camera():");
            tracing::trace!(
                "  view[0]: [{:.3}, {:.3}, {:.3}, {:.3}]",
                view_matrix[0][0],
                view_matrix[0][1],
                view_matrix[0][2],
                view_matrix[0][3]
            );
            tracing::trace!(
                "  view[1]: [{:.3}, {:.3}, {:.3}, {:.3}]",
                view_matrix[1][0],
                view_matrix[1][1],
                view_matrix[1][2],
                view_matrix[1][3]
            );
            tracing::trace!(
                "  view[2]: [{:.3}, {:.3}, {:.3}, {:.3}]",
                view_matrix[2][0],
                view_matrix[2][1],
                view_matrix[2][2],
                view_matrix[2][3]
            );
            tracing::trace!(
                "  view[3]: [{:.3}, {:.3}, {:.3}, {:.3}]",
                view_matrix[3][0],
                view_matrix[3][1],
                view_matrix[3][2],
                view_matrix[3][3]
            );
        }

        let view_proj = build_view_projection_matrix(view_matrix, proj_matrix);

        if should_dump {
            tracing::trace!(
                "  view_proj[3]: [{:.3}, {:.3}, {:.3}, {:.3}]",
                view_proj[3][0],
                view_proj[3][1],
                view_proj[3][2],
                view_proj[3][3]
            );
        }

        let model = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];

        let uniforms = LineUniforms { view_proj, model };
        self.update_uniforms(queue, &uniforms);
    }

    /// 線分データを更新
    pub fn update_line_data(&mut self, device: &wgpu::Device, vertices: &[MeshVertex]) {
        tracing::debug!("LineResources.update_line_data(): {} 頂点", vertices.len());

        // 最初の8頂点の座標をログ出力（デバッグ用）
        if !vertices.is_empty() && vertices.len() <= 8 {
            for (i, v) in vertices.iter().enumerate() {
                tracing::trace!(
                    "  頂点[{}]: ({:.3}, {:.3}, {:.3})",
                    i,
                    v.position[0],
                    v.position[1],
                    v.position[2]
                );
            }
        }

        if !vertices.is_empty() {
            self.vertex_buffer = Some(device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Line Vertex Buffer"),
                    contents: bytemuck::cast_slice(vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                },
            ));
            self.vertex_count = vertices.len() as u32;
        }
    }

    /// 線分をレンダリング
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if let Some(vertex_buffer) = &self.vertex_buffer {
            if should_log(&LINE_RENDER_LOG_COUNTER) {
                tracing::trace!(
                    "LineResources.render(): vertex_count={}, pipeline設定開始",
                    self.vertex_count
                );
            }
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw(0..self.vertex_count, 0..1);
            if should_log(&LINE_RENDER_LOG_COUNTER) {
                tracing::trace!("LineResources.render(): draw完了");
            }
        } else {
            tracing::debug!("LineResources.render(): vertex_buffer が None");
        }
    }
}
