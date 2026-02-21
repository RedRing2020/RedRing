//! Octree可視化レンダリングステージ
//!
//! ViewModel層から変換されたOctreeワイヤーフレームデータを受け取り、
//! GPU上でレンダリングするステージです。
//!
//! # 使用例
//!
//! ```ignore
//! use stage::{OctreeStage, RenderStage};
//! use wgpu::{Device, TextureFormat};
//!
//! let device = /* wgpu Device */;
//! let format = TextureFormat::Bgra8UnormSrgb;
//!
//! let mut stage = OctreeStage::new(&device, format);
//!
//! // ViewModel変換データを設定
//! // stage.set_octree_data(&device, vertices);
//!
//! // カメラ行列更新
//! // stage.update_camera(&queue, view_proj_matrix);
//! ```

use logging_foundation::{frame_interval_from_env, should_log_every_n_frames};
use render::line::LineResources;
use render::vertex_3d::MeshVertex;
use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use wgpu::{CommandEncoder, Device, TextureFormat, TextureView};

use crate::RenderStage;

static OCTREE_STAGE_RENDER_LOG_COUNTER: AtomicU64 = AtomicU64::new(0);

fn should_log_render_trace() -> bool {
    let frame = OCTREE_STAGE_RENDER_LOG_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
    let interval = frame_interval_from_env("REDRING_LOG_FRAME_INTERVAL", 120);
    should_log_every_n_frames(frame, interval)
}

/// Octree可視化ステージ
pub struct OctreeStage {
    resources: LineResources,
    has_data: bool,
}

impl OctreeStage {
    /// 新しいOctreeステージを作成
    ///
    /// # 引数
    ///
    /// - `device`: wgpu Device
    /// - `format`: レンダーターゲットのテクスチャフォーマット
    pub fn new(device: &Device, format: TextureFormat) -> Self {
        let resources = LineResources::new(device, format);

        Self {
            resources,
            has_data: false,
        }
    }

    /// ワイヤーフレームデータを設定（色無しバージョン）
    ///
    /// ViewModel層から変換された頂点データを受け取ります。
    /// Phase 1では色情報を無視し、単色（白）で描画します。
    ///
    /// # 引数
    ///
    /// - `device`: wgpu Device
    /// - `vertices`: ワイヤーフレームの頂点配列（位置のみ使用）
    ///
    /// # Note
    ///
    /// `vertices` は LineList トポロジー用のため、
    /// 2頂点で1線分を表現します。
    /// 将来的には色情報も活用する予定です（Phase 2）。
    pub fn set_wireframe_data(&mut self, device: &Device, vertices: Vec<[f32; 3]>) {
        tracing::info!("Octreeワイヤーフレームデータ設定: {} 頂点", vertices.len());

        // 色無しの頂点を MeshVertex に変換（法線はダミー）
        let mesh_vertices: Vec<MeshVertex> = vertices
            .iter()
            .map(|&position| MeshVertex::new(position, [0.0, 0.0, 1.0]))
            .collect();

        self.resources.update_line_data(device, &mesh_vertices);
        self.has_data = !mesh_vertices.is_empty();
    }

    /// データが設定されているか確認
    pub fn has_data(&self) -> bool {
        self.has_data
    }

    /// 頂点数を取得
    pub fn vertex_count(&self) -> u32 {
        self.resources.vertex_count
    }
}

impl RenderStage for OctreeStage {
    fn render(&mut self, encoder: &mut CommandEncoder, view: &TextureView) {
        let should_trace = should_log_render_trace();

        if !self.has_data {
            tracing::info!("🚨 OctreeStage: has_data=false, 描画スキップ");
            return;
        }

        tracing::info!(
            "✓ OctreeStage.render(): 描画実行, vertex_count={}, vertex_buffer is {}",
            self.resources.vertex_count,
            if self.resources.vertex_buffer.is_some() {
                "Some"
            } else {
                "None"
            }
        );

        if should_trace {
            tracing::trace!(
                "OctreeStage: 描画開始 ({} 頂点)",
                self.resources.vertex_count
            );
        }

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Octree Wireframe Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.1,
                        b: 0.15,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None, // TODO: Depthバッファ統合時に修正
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.resources.render_pipeline);
        render_pass.set_bind_group(0, &self.resources.bind_group, &[]);

        if let Some(ref vertex_buffer) = self.resources.vertex_buffer {
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw(0..self.resources.vertex_count, 0..1);
        }
    }

    /// カメラ行列を更新（RenderStage trait override）
    fn update_camera(
        &mut self,
        queue: &wgpu::Queue,
        view_matrix: [[f32; 4]; 4],
        proj_matrix: [[f32; 4]; 4],
    ) {
        tracing::info!("OctreeStage.update_camera() 呼び出し");
        tracing::info!(
            "  view_matrix[3]: [{:.2}, {:.2}, {:.2}, {:.2}]",
            view_matrix[3][0],
            view_matrix[3][1],
            view_matrix[3][2],
            view_matrix[3][3]
        );
        tracing::info!(
            "  proj_matrix[0]: [{:.2}, {:.2}, {:.2}, {:.2}]",
            proj_matrix[0][0],
            proj_matrix[0][1],
            proj_matrix[0][2],
            proj_matrix[0][3]
        );

        self.resources
            .update_camera(queue, view_matrix, proj_matrix);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_octree_stage_creation() {
        // Note: wgpu Deviceが必要なため、実際のテストは統合テストで実施
        // このテストは構造確認のみ
    }

    #[test]
    fn test_octree_stage_no_data_initially() {
        // Note: wgpu Deviceが必要なため、実際のテストは統合テストで実施
    }
}
