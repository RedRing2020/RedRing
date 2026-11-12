use crate::shader;
use crate::uniform_factory;
use crate::vertex_3d::MeshVertex;
use bytemuck::{Pod, Zeroable};
use logging_foundation::{frame_interval_from_env, should_log_every_n_frames};
use std::sync::atomic::{AtomicU64, Ordering};
use viewmodel_graphics::build_view_projection_matrix;

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
    pub color: [f32; 4],
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
            color: [1.0, 1.0, 0.0, 1.0],
        }
    }
}

/// 線分の頂点を GPU に置けない（GPU のメモリが足りない）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineDataError {
    /// 置こうとした頂点の数
    pub vertices: usize,
    /// GPU の誤り
    pub reason: String,
}

impl std::fmt::Display for LineDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "cannot place {} line vertices on the GPU: {}",
            self.vertices, self.reason
        )
    }
}

impl std::error::Error for LineDataError {}

/// バッファー 1 つの大きさの上限 `max_buffer_size`（バイト）に収まる、線分の頂点の数（線分を途中で分けないよう
/// 偶数。少なくとも 1 本の線分）
pub fn line_vertices_per_buffer(max_buffer_size: u64) -> usize {
    let vertex_size = std::mem::size_of::<MeshVertex>() as u64;
    // 描くときの頂点の数は u32 で渡す
    let vertices = (max_buffer_size / vertex_size).min(u64::from(u32::MAX));
    (usize::try_from(vertices).unwrap_or(usize::MAX) & !1).max(2)
}

/// 頂点のバイト列 `contents` を置いた頂点のバッファー
///
/// GPU のメモリが足りない・GPU の内部の誤りのときは、アプリを止めずに誤りの理由を返す（`create_buffer_init` は、
/// 置けないときに止まるため使わない）。
fn vertex_buffer(device: &wgpu::Device, contents: &[u8]) -> Result<wgpu::Buffer, String> {
    // 書き込む大きさは、コピーの単位の倍数で 0 より大きくする
    let align = wgpu::COPY_BUFFER_ALIGNMENT;
    let size = (contents.len() as u64).div_ceil(align).max(1) * align;
    let out_of_memory = device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
    let internal = device.push_error_scope(wgpu::ErrorFilter::Internal);
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Line Vertex Buffer"),
        size,
        usage: wgpu::BufferUsages::VERTEX,
        mapped_at_creation: true,
    });
    let mapped = match buffer.get_mapped_range_mut(..) {
        Ok(mut range) => {
            range.slice(..contents.len()).copy_from_slice(contents);
            drop(range);
            buffer.unmap();
            Ok(())
        }
        Err(error) => Err(error.to_string()),
    };
    let internal = pollster::block_on(internal.pop());
    let out_of_memory = pollster::block_on(out_of_memory.pop());
    match (out_of_memory.or(internal), mapped) {
        (Some(error), _) => Err(error.to_string()),
        (None, Err(error)) => Err(error),
        (None, Ok(())) => Ok(buffer),
    }
}

/// 線分レンダリングリソース
pub struct LineResources {
    pub render_pipeline: wgpu::RenderPipeline,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    /// 頂点のバッファーと、その頂点の数（バッファー 1 つの大きさの上限に収まるように分ける）
    pub vertex_buffers: Vec<(wgpu::Buffer, u32)>,
    pub color: [f32; 4],
}

impl LineResources {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = shader::line_shader(device);

        let uniforms = LineUniforms::default();
        let (bind_group_layout, uniform_buffer, bind_group) =
            uniform_factory::create_uniform_binding(
                device,
                &uniforms,
                wgpu::ShaderStages::VERTEX_FRAGMENT,
                "line_bind_group_layout",
                "Line Uniform Buffer",
                "line_bind_group",
            );

        // Render pipeline layout
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Line Render Pipeline Layout"),
                bind_group_layouts: &[Some(&bind_group_layout)],
                immediate_size: 0,
            });

        // Render pipeline（LineList トポロジー）
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Line Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(MeshVertex::desc())],
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
            multiview_mask: None,
            cache: None,
        });

        Self {
            render_pipeline,
            uniform_buffer,
            bind_group_layout,
            bind_group,
            vertex_buffers: Vec::new(),
            color: [1.0, 1.0, 0.0, 1.0],
        }
    }

    /// Uniformバッファを更新
    pub fn update_uniforms(&self, queue: &wgpu::Queue, uniforms: &LineUniforms) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[*uniforms]));
    }

    /// カメラ行列を更新
    pub fn update_camera(
        &mut self,
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

        let uniforms = LineUniforms {
            view_proj,
            model,
            color: self.color,
        };
        self.update_uniforms(queue, &uniforms);
    }

    /// 線分の描画色を設定
    pub fn set_color(&mut self, color: [f32; 4]) {
        self.color = color;
    }

    /// 線分データを更新する（前の線分は放す）。頂点は、バッファー 1 つの大きさの上限に収まるように分けて置く
    ///
    /// # Errors
    ///
    /// GPU のメモリが足りず、頂点を置けない場合（線分は描かない）。
    pub fn update_line_data(
        &mut self,
        device: &wgpu::Device,
        vertices: &[MeshVertex],
    ) -> Result<(), LineDataError> {
        tracing::debug!(
            "LineResources.update_line_data(): {} vertices",
            vertices.len()
        );

        // 最初の8頂点の座標をログ出力（デバッグ用）
        if !vertices.is_empty() && vertices.len() <= 8 {
            for (i, v) in vertices.iter().enumerate() {
                tracing::trace!(
                    "  vertex[{}]: ({:.3}, {:.3}, {:.3})",
                    i,
                    v.position[0],
                    v.position[1],
                    v.position[2]
                );
            }
        }

        // 前の線分を先に放す（新しい線分を置くメモリに使えるように）
        self.vertex_buffers.clear();
        if vertices.is_empty() {
            return Ok(());
        }
        let per_buffer = line_vertices_per_buffer(device.limits().max_buffer_size);
        let mut buffers = Vec::new();
        for chunk in vertices.chunks(per_buffer) {
            let buffer = vertex_buffer(device, bytemuck::cast_slice(chunk)).map_err(|reason| {
                LineDataError {
                    vertices: vertices.len(),
                    reason,
                }
            })?;
            // 1 つのバッファーの頂点の数は u32 に収まる（`line_vertices_per_buffer`）
            buffers.push((buffer, chunk.len() as u32));
        }
        self.vertex_buffers = buffers;
        Ok(())
    }

    /// 線分をレンダリング
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.vertex_buffers.is_empty() {
            tracing::debug!("LineResources.render(): no vertex buffer");
            return;
        }
        if should_log(&LINE_RENDER_LOG_COUNTER) {
            tracing::trace!(
                "LineResources.render(): {} vertex buffers, setting the pipeline",
                self.vertex_buffers.len()
            );
        }
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        for (buffer, count) in &self.vertex_buffers {
            render_pass.set_vertex_buffer(0, buffer.slice(..));
            render_pass.draw(0..*count, 0..1);
        }
        if should_log(&LINE_RENDER_LOG_COUNTER) {
            tracing::trace!("LineResources.render(): drawn");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_vertices_are_split_into_buffers_within_the_size_limit_without_splitting_a_segment() {
        let vertex_size = std::mem::size_of::<MeshVertex>();
        // 既定の上限 256 MiB に、24 バイトの頂点が 11184810 個入る。線分を分けないよう偶数にする
        let per_buffer = line_vertices_per_buffer(256 << 20);
        assert_eq!(vertex_size, 24);
        assert_eq!(per_buffer, 11_184_810);
        assert!(per_buffer * vertex_size <= 256 << 20);
        assert_eq!(line_vertices_per_buffer(5 * 24), 4);
        // 線分 1 本が入らないほど小さくても、少なくとも 1 本ずつ置く
        assert_eq!(line_vertices_per_buffer(0), 2);
        // 描くときの頂点の数は u32 に収まる
        assert!(line_vertices_per_buffer(u64::MAX) <= u32::MAX as usize);
        assert_eq!(line_vertices_per_buffer(u64::MAX) % 2, 0);
    }
}
